//! Build report — one diagnostics model behind both `cronus build` (human
//! output) and `cronus build --ai` (JSON, `schema_version` 1).
//! Codes, schema and exit codes are documented in LANGUAGE.md §15.9.

use serde_json::{json, Map, Value};
use std::collections::HashMap;

use super::locate::{Pos, SourceIndex};
use crate::constitution_check;
use crate::contracts::{self, ContractRegistry, ParseWarning};
use crate::hardcode_lint;
use crate::lint;
use crate::parser::diagnostic::{self, codes};
use crate::parser::{self, AstNode, ParseError};
use crate::resolve;

pub const SCHEMA_VERSION: u64 = 1;

/// Process exit codes of `cronus build`.
pub mod exit {
    /// No errors (warnings allowed).
    pub const VALID: i32 = 0;
    /// The file was read but has at least one error.
    pub const INVALID: i32 = 1;
    /// Usage or I/O problem: no file found, file unreadable.
    pub const USAGE: i32 = 2;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl Severity {
    fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fix {
    /// replace | remove | add | edit | add_bind | add_auth | wrap_in_dynamic
    pub action: &'static str,
    pub target: String,
    pub replacement: Option<String>,
    pub hint: Option<String>,
}

impl Fix {
    fn replace(target: impl Into<String>, replacement: impl Into<String>) -> Self {
        Fix {
            action: "replace",
            target: target.into(),
            replacement: Some(replacement.into()),
            hint: None,
        }
    }

    fn hint(action: &'static str, target: impl Into<String>, hint: impl Into<String>) -> Self {
        Fix {
            action,
            target: target.into(),
            replacement: None,
            hint: Some(hint.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub category: &'static str,
    pub message: String,
    pub pos: Pos,
    /// Extra location context, e.g. `("page", "/")`, `("section", "kpi")`.
    pub context: Vec<(&'static str, String)>,
    pub fix: Fix,
    /// Originating lint/constitution rule, when there is one.
    pub rule: Option<String>,
}

/// Validation strictness. `--ai`, `--strict-ai` and `--strict` use `strict`;
/// it mirrors the runtime's strict mode, where contract violations stop a
/// section from rendering.
#[derive(Debug, Clone, Copy)]
pub struct Profile {
    pub strict: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Stats {
    pub entities: usize,
    pub pages: usize,
    pub routes: usize,
}

#[derive(Debug)]
pub struct Report {
    pub file: String,
    pub diagnostics: Vec<Diagnostic>,
    pub stats: Stats,
    pub usage_error: bool,
}

impl Report {
    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
    }

    pub fn warnings(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
    }

    pub fn valid(&self) -> bool {
        !self.usage_error && self.errors().next().is_none()
    }

    pub fn exit_code(&self) -> i32 {
        if self.usage_error {
            exit::USAGE
        } else if self.valid() {
            exit::VALID
        } else {
            exit::INVALID
        }
    }

    pub fn to_json(&self) -> Value {
        let render = |it: &mut dyn Iterator<Item = &Diagnostic>| -> Vec<Value> {
            it.map(|d| diagnostic_json(&self.file, d)).collect()
        };
        let errors = render(&mut self.errors());
        let warnings = render(&mut self.warnings());
        json!({
            "schema_version": SCHEMA_VERSION,
            "valid": self.valid(),
            "exit_code": self.exit_code(),
            "file": self.file,
            "errors": errors,
            "warnings": warnings,
            "context": {
                "entities": self.stats.entities,
                "pages": self.stats.pages,
                "routes": self.stats.routes,
                "error_count": errors.len(),
                "warning_count": warnings.len(),
            }
        })
    }

    /// `file:line:col: severity[CODE]: message` plus a fix line per diagnostic.
    pub fn render_human(&self, color: bool) -> String {
        let mut out = String::new();
        for d in &self.diagnostics {
            let (open, close) = match (color, d.severity) {
                (false, _) => ("", ""),
                (true, Severity::Error) => ("\x1b[31m", "\x1b[0m"),
                (true, Severity::Warning) => ("\x1b[33m", "\x1b[0m"),
            };
            out.push_str(&format!(
                "{}:{}:{}: {}{}[{}]{}: {}\n",
                self.file,
                d.pos.line,
                d.pos.col,
                open,
                d.severity.as_str(),
                d.code,
                close,
                d.message
            ));
            match (&d.fix.replacement, &d.fix.hint) {
                (Some(r), _) => out.push_str(&format!(
                    "    fix: replace '{}' with '{}'\n",
                    d.fix.target, r
                )),
                (None, Some(h)) => out.push_str(&format!("    hint: {}\n", h)),
                (None, None) => {}
            }
        }
        out
    }

    /// The single final verdict line.
    pub fn verdict(&self, color: bool) -> String {
        let paint = |code: &str, s: &str| {
            if color {
                format!("\x1b[{}m{}\x1b[0m", code, s)
            } else {
                s.to_string()
            }
        };
        let warnings = self.warnings().count();
        if self.usage_error {
            let msg = self
                .diagnostics
                .first()
                .map(|d| d.message.as_str())
                .unwrap_or("usage error");
            return format!("  {} build failed: {}", paint("31", "\u{2717}"), msg);
        }
        if self.valid() {
            format!(
                "  {} {} is valid — {} entities, {} pages, {} routes, {} warning(s)",
                paint("32", "\u{2713}"),
                self.file,
                self.stats.entities,
                self.stats.pages,
                self.stats.routes,
                warnings
            )
        } else {
            format!(
                "  {} {} — build blocked: {} error(s), {} warning(s)",
                paint("31", "\u{2717}"),
                self.file,
                self.errors().count(),
                warnings
            )
        }
    }
}

fn diagnostic_json(file: &str, d: &Diagnostic) -> Value {
    let mut loc = Map::new();
    loc.insert("file".into(), json!(file));
    loc.insert("line".into(), json!(d.pos.line));
    loc.insert("col".into(), json!(d.pos.col));
    if d.pos.len > 0 {
        loc.insert(
            "span".into(),
            json!({
                "start": {"line": d.pos.line, "col": d.pos.col},
                "end": {"line": d.pos.line, "col": d.pos.col + d.pos.len},
            }),
        );
    }
    for (k, v) in &d.context {
        loc.insert((*k).into(), json!(v));
    }
    let mut fix = Map::new();
    fix.insert("action".into(), json!(d.fix.action));
    fix.insert("target".into(), json!(d.fix.target));
    if let Some(r) = &d.fix.replacement {
        fix.insert("replacement".into(), json!(r));
    }
    if let Some(h) = &d.fix.hint {
        fix.insert("hint".into(), json!(h));
    }
    let mut obj = Map::new();
    obj.insert("code".into(), json!(d.code));
    obj.insert("severity".into(), json!(d.severity.as_str()));
    obj.insert("category".into(), json!(d.category));
    obj.insert("message".into(), json!(d.message));
    if let Some(rule) = &d.rule {
        obj.insert("rule".into(), json!(rule));
    }
    obj.insert("location".into(), Value::Object(loc));
    obj.insert("fix".into(), Value::Object(fix));
    Value::Object(obj)
}

// ── Report constructors ──────────────────────────────────────────────────────

/// Usage / I/O failure (exit code 2).
pub fn usage_error(file: &str, code: &str, message: String, hint: &str) -> Report {
    Report {
        file: file.to_string(),
        diagnostics: vec![Diagnostic {
            code: code.to_string(),
            severity: Severity::Error,
            category: "usage",
            message,
            pos: Pos::START,
            context: Vec::new(),
            fix: Fix::hint("edit", file, hint),
            rule: None,
        }],
        stats: Stats::default(),
        usage_error: true,
    }
}

pub fn from_parse_errors(file: &str, errors: &[ParseError]) -> Report {
    let diagnostics = errors
        .iter()
        .map(|e| {
            let target = e.target.clone().unwrap_or_default();
            let fix = match (&e.replacement, &e.hint) {
                (Some(r), hint) => Fix {
                    hint: hint.clone(),
                    ..Fix::replace(target, r.clone())
                },
                (None, Some(h)) => {
                    let action = if e.code == codes::MISSING_FIELD_TYPE {
                        "add"
                    } else {
                        "edit"
                    };
                    Fix::hint(action, target, h.clone())
                }
                (None, None) => Fix::hint("edit", target, default_parse_hint(e)),
            };
            Diagnostic {
                code: e.code.to_string(),
                severity: Severity::Error,
                category: match e.code.split('_').next() {
                    Some("TYPE") => "type",
                    Some("FIELD") => "validation",
                    Some("ENV") => "env",
                    Some("BIND") => "bind",
                    Some("ACTION") => "action",
                    Some("LANG") => "language",
                    Some("COMPOSE") => "compose",
                    _ => "syntax",
                },
                message: e.message.clone(),
                pos: Pos {
                    line: e.line,
                    col: e.col,
                    len: e.len,
                },
                context: Vec::new(),
                fix,
                rule: None,
            }
        })
        .collect();
    Report {
        file: file.to_string(),
        diagnostics,
        stats: Stats::default(),
        usage_error: false,
    }
}

fn default_parse_hint(e: &ParseError) -> String {
    match e
        .message
        .strip_prefix("expected ")
        .and_then(|rest| rest.split(", found").next())
    {
        Some(expected) => format!(
            "insert {} at this position, or remove the unexpected token",
            expected
        ),
        None => "see LANGUAGE.md §2 for the grammar".to_string(),
    }
}

/// Run every validation pass over a parsed AST.
pub fn validate(file: &str, source: &str, nodes: &[AstNode], profile: Profile) -> Report {
    let idx = SourceIndex::new(source);
    let mut out = Vec::new();
    structure_pass(nodes, &idx, &mut out);
    contract_pass(nodes, &idx, profile, &mut out);
    resolve_pass(nodes, &idx, &mut out);
    lint_pass(nodes, &idx, profile, &mut out);
    if profile.strict {
        hardcode_pass(nodes, &idx, &mut out);
    }
    constitution_pass(nodes, &idx, profile, &mut out);
    env_pass(nodes, &mut out);
    let (entities, pages, routes) = parser::stats(nodes);
    Report {
        file: file.to_string(),
        diagnostics: out,
        stats: Stats {
            entities,
            pages,
            routes,
        },
        usage_error: false,
    }
}

/// `APP_STRIPE_KEY`: SCREAMING_SNAKE with a non-empty prefix before `_`.
fn has_env_prefix(name: &str) -> bool {
    name.chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        && name.starts_with(|c: char| c.is_ascii_uppercase())
        && name
            .split_once('_')
            .is_some_and(|(prefix, rest)| !prefix.is_empty() && !rest.is_empty())
}

/// ENV_003 (warning): declared env variables should carry an app prefix
/// (`APP_STRIPE_KEY`) so they never collide with system variables.
fn env_pass(nodes: &[AstNode], out: &mut Vec<Diagnostic>) {
    let vars = nodes.iter().flat_map(|n| match n {
        AstNode::Env(env) => env.schema.as_slice(),
        _ => &[],
    });
    for var in vars.filter(|v| !has_env_prefix(&v.name)) {
        out.push(Diagnostic {
            code: "ENV_003".into(),
            severity: Severity::Warning,
            category: "env",
            message: format!(
                "environment variable '{}' has no uppercase prefix such as APP_",
                var.name
            ),
            pos: Pos {
                line: var.line,
                col: var.col,
                len: var.name.chars().count(),
            },
            context: Vec::new(),
            fix: Fix::hint(
                "edit",
                var.name.clone(),
                format!(
                    "use SCREAMING_SNAKE with a prefix, e.g. 'APP_{}'",
                    var.name.to_ascii_uppercase()
                ),
            ),
            rule: None,
        });
    }
}

/// STRUCTURE_001: a `page`/`entity` declared in the source that is missing
/// from the AST. That only happens when an earlier statement consumed a `}`
/// and the parser nested the declaration inside another block — the file
/// "parses" but silently loses pages/entities.
fn structure_pass(nodes: &[AstNode], idx: &SourceIndex, out: &mut Vec<Diagnostic>) {
    let mut parsed_pages: Vec<&str> = Vec::new();
    let mut parsed_entities: Vec<&str> = Vec::new();
    for node in nodes {
        match node {
            AstNode::Page(p) => parsed_pages.push(&p.route),
            AstNode::Entity(e) => parsed_entities.push(&e.name),
            _ => {}
        }
    }
    let mut lost = |kind: &'static str, name: &str, pos: Pos, parsed: &mut Vec<&str>| {
        match parsed.iter().position(|p| *p == name) {
            Some(i) => {
                parsed.remove(i);
            }
            None => out.push(Diagnostic {
                code: "STRUCTURE_001".into(),
                severity: Severity::Error,
                category: "structure",
                message: format!(
                    "{} '{}' is declared here but missing from the parsed app; an earlier block consumed its closing '}}'",
                    kind, name
                ),
                pos,
                context: vec![(if kind == "page" { "page" } else { "entity" }, name.to_string())],
                fix: Fix::hint(
                    "edit",
                    name,
                    "check the block before this declaration for a statement missing its argument or an unbalanced '{'",
                ),
                rule: None,
            }),
        }
    };
    for (route, pos) in idx.declared_pages() {
        lost("page", route, pos, &mut parsed_pages);
    }
    for (name, pos) in idx.declared_entities() {
        lost("entity", name, pos, &mut parsed_entities);
    }
}

fn contract_pass(
    nodes: &[AstNode],
    idx: &SourceIndex,
    profile: Profile,
    out: &mut Vec<Diagnostic>,
) {
    let blocking = if profile.strict {
        Severity::Error
    } else {
        Severity::Warning
    };
    let mut pages_seen: HashMap<&str, usize> = HashMap::new();
    for node in nodes {
        let AstNode::Page(page) = node else {
            continue;
        };
        let page_nth = bump(&mut pages_seen, &page.route);
        let mut types_seen: HashMap<&str, usize> = HashMap::new();
        for section in &page.sections {
            let nth = bump(&mut types_seen, &section.section_type);
            let section_pos = idx
                .section(&page.route, page_nth, &section.section_type, nth)
                .or_else(|| idx.page(&page.route, page_nth))
                .unwrap_or(Pos::START);
            let context = vec![
                ("page", page.route.clone()),
                ("section", section.section_type.clone()),
            ];
            for w in contracts::validate_section(section, &[]) {
                let (code, severity, message, fix, pos) = match w {
                    ParseWarning::UnknownSection { name, .. } => {
                        let fix = match diagnostic::closest(&name, ContractRegistry::all_names()) {
                            Some(s) => Fix::replace(&name, s),
                            None => Fix::hint(
                                "replace",
                                &name,
                                format!(
                                    "valid section types: {}",
                                    ContractRegistry::all_names().join(", ")
                                ),
                            ),
                        };
                        (
                            "CONTRACT_001",
                            blocking,
                            format!("unknown section type '{}'", name),
                            fix,
                            section_pos,
                        )
                    }
                    ParseWarning::UnknownKey {
                        section: sec, key, ..
                    } => (
                        "CONTRACT_002",
                        blocking,
                        format!("unknown key '{}' in section '{}'", key, sec),
                        Fix::hint(
                            "remove",
                            &key,
                            format!("remove it or use a key defined for '{}' sections", sec),
                        ),
                        idx.word(&key, Some(section_pos)).unwrap_or(section_pos),
                    ),
                    ParseWarning::MissingRequired {
                        section: sec, key, ..
                    } => (
                        "CONTRACT_003",
                        blocking,
                        format!("missing required key '{}' in section '{}'", key, sec),
                        Fix::hint("add", &key, format!("add '{}:<value>' to each item", key)),
                        section_pos,
                    ),
                    ParseWarning::AliasUsed {
                        alias, canonical, ..
                    } => (
                        "CONTRACT_004",
                        Severity::Warning,
                        format!(
                            "section type '{}' is an alias; use the canonical name '{}'",
                            alias, canonical
                        ),
                        Fix::replace(&alias, &canonical),
                        section_pos,
                    ),
                    ParseWarning::MinItemsViolation {
                        section: sec,
                        expected,
                        actual,
                        ..
                    } => (
                        "CONTRACT_005",
                        blocking,
                        format!(
                            "section '{}' requires at least {} item(s), found {}",
                            sec, expected, actual
                        ),
                        Fix::hint(
                            "add",
                            "item",
                            format!("add {} more item(s)", expected.saturating_sub(actual)),
                        ),
                        section_pos,
                    ),
                    ParseWarning::UnknownConfig {
                        section: sec, key, ..
                    } => (
                        "CONTRACT_006",
                        blocking,
                        format!("unknown config key '{}' in section '{}'", key, sec),
                        Fix::hint("remove", &key, format!("'{}' sections do not read it", sec)),
                        idx.word(&key, Some(section_pos)).unwrap_or(section_pos),
                    ),
                };
                out.push(Diagnostic {
                    code: code.into(),
                    severity,
                    category: "contract",
                    message,
                    pos,
                    context: context.clone(),
                    fix,
                    rule: None,
                });
            }
        }
    }
}

fn bump<'a>(seen: &mut HashMap<&'a str, usize>, key: &'a str) -> usize {
    let n = seen.entry(key).or_insert(0);
    *n += 1;
    *n - 1
}

/// Quoted names in a validator message: `Entity 'X' not found (page '/')` → [X, /].
fn quoted(message: &str) -> Vec<&str> {
    message.split('\'').skip(1).step_by(2).collect()
}

fn resolve_pass(nodes: &[AstNode], idx: &SourceIndex, out: &mut Vec<Diagnostic>) {
    let (_symbols, errors) = resolve::resolve(nodes);
    for re in errors {
        let names = quoted(&re.message);
        let target = names.first().copied().unwrap_or("");
        let route = re
            .message
            .split("page '")
            .nth(1)
            .and_then(|s| s.split('\'').next());
        let page_pos = route.and_then(|r| idx.page(r, 0));
        let pos = match page_pos {
            Some(p) => idx.word(target, Some(p)).unwrap_or(p),
            None => idx
                .word(target, None)
                .or_else(|| names.get(1).and_then(|e| idx.entity(e)))
                .unwrap_or(Pos::START),
        };
        let is_state_machine =
            re.message.starts_with("Transition") || re.message.contains("must be an enum");
        let fix = match &re.suggestion {
            Some(s) => Fix::replace(target, s),
            None if is_state_machine => Fix::hint(
                "edit",
                target,
                "declare the field as 'enum [a, b, c]' before the transition block",
            ),
            None => Fix::hint(
                "add",
                target,
                format!("define '{}' or correct the reference", target),
            ),
        };
        let mut context = Vec::new();
        if let Some(r) = route {
            context.push(("page", r.to_string()));
        }
        out.push(Diagnostic {
            code: if is_state_machine {
                "RESOLVE_002"
            } else {
                "RESOLVE_001"
            }
            .into(),
            severity: Severity::Error,
            category: if is_state_machine {
                "state_machine"
            } else {
                "reference"
            },
            message: re.message.clone(),
            pos,
            context,
            fix,
            rule: None,
        });
    }
}

/// Stable numeric code per lint rule; unknown rules share LINT_099 (see `rule`).
fn lint_code(rule: &str) -> &'static str {
    match rule {
        "no-dead-text" => "LINT_001",
        "no-dead-links" => "LINT_002",
        "bind-or-empty" => "LINT_003",
        "no-sensitive-render" => "LINT_004",
        "no-sensitive-select" => "LINT_005",
        "no-hardcode-user" => "LINT_006",
        "no-fake-state" => "LINT_007",
        "no-dead-ui" => "LINT_008",
        "no-orphan-reload" => "LINT_009",
        "form-submit-handler" => "LINT_010",
        "shared-entity-auth" => "LINT_011",
        _ => "LINT_099",
    }
}

fn lint_pass(nodes: &[AstNode], idx: &SourceIndex, profile: Profile, out: &mut Vec<Diagnostic>) {
    for lr in lint::lint_ast(nodes, profile.strict) {
        // lint names sections "type (route)" or "type (define:Name)"
        let (section_type, owner) = lr
            .section
            .rsplit_once(" (")
            .map(|(t, o)| (t, o.trim_end_matches(')')))
            .unwrap_or((lr.section.as_str(), lr.page.as_str()));
        let target = lr
            .message
            .split('\'')
            .nth(1)
            .or_else(|| lr.message.split('"').nth(1))
            .unwrap_or("")
            .to_string();
        let anchor = match owner.strip_prefix("define:") {
            Some(def) => idx.word(def, None),
            None => idx
                .section(owner, 0, section_type, 0)
                .or_else(|| idx.page(owner, 0)),
        };
        let pos = anchor
            .map(|a| {
                idx.word(&target, Some(a))
                    .filter(|w| w.line == a.line)
                    .unwrap_or(a)
            })
            .or_else(|| idx.word(&target, None))
            .unwrap_or(Pos::START);
        let action = match lr.rule {
            r if r.contains("dead-text") || r.contains("hardcode") => "wrap_in_dynamic",
            r if r.contains("bind-or-empty") => "add_bind",
            r if r.contains("sensitive") => "remove",
            r if r.contains("auth") => "add_auth",
            _ => "replace",
        };
        let fix_target = match action {
            "add_bind" => lr.section.clone(),
            "add_auth" => lr.page.clone(),
            _ => target,
        };
        let mut context = Vec::new();
        if !lr.page.is_empty() {
            context.push(("page", lr.page.clone()));
        }
        if !lr.section.is_empty() {
            context.push(("section", section_type.to_string()));
        }
        out.push(Diagnostic {
            code: lint_code(lr.rule).into(),
            severity: match lr.severity {
                lint::Severity::Error => Severity::Error,
                lint::Severity::Warning => Severity::Warning,
            },
            category: "lint",
            message: lr.message.clone(),
            pos,
            context,
            fix: Fix::hint(action, fix_target, lr.fix.clone()),
            rule: Some(lr.rule.to_string()),
        });
    }
}

fn hardcode_pass(nodes: &[AstNode], idx: &SourceIndex, out: &mut Vec<Diagnostic>) {
    let mut pages = Vec::new();
    let mut entities = Vec::new();
    let mut style = None;
    for node in nodes {
        match node {
            AstNode::Page(p) => pages.push(p.clone()),
            AstNode::Entity(e) => entities.push(e.clone()),
            AstNode::Style(s) => style = Some(s.clone()),
            _ => {}
        }
    }
    for f in hardcode_lint::lint_all_pages(&pages, &entities, style.as_ref()) {
        let page_pos = idx.page(&f.page, 0);
        let pos = page_pos
            .and_then(|p| idx.word(&f.text, Some(p)))
            .or(page_pos)
            .unwrap_or(Pos::START);
        out.push(Diagnostic {
            code: "LINT_020".into(),
            severity: if f.severity == "error" {
                Severity::Error
            } else {
                Severity::Warning
            },
            category: "lint",
            message: format!(
                "hardcoded text '{}' in page '{}' does not come from .cronus data",
                f.text, f.page
            ),
            pos,
            context: vec![("page", f.page.clone())],
            fix: Fix::hint(
                "wrap_in_dynamic",
                &f.text,
                "move the text into section data (title/subtitle/items) or bind it to an entity",
            ),
            rule: Some("hardcoded-content".into()),
        });
    }
}

fn constitution_pass(
    nodes: &[AstNode],
    idx: &SourceIndex,
    profile: Profile,
    out: &mut Vec<Diagnostic>,
) {
    let Some(constitution) = nodes.iter().find_map(|n| match n {
        AstNode::App(a) => a.constitution.as_ref(),
        _ => None,
    }) else {
        return;
    };
    for v in constitution_check::check_constitution(nodes, constitution) {
        if v.rule_type == "info" {
            continue;
        }
        let entity = v.entity.clone().unwrap_or_default();
        let pos = idx
            .entity(&entity)
            .or_else(|| idx.constitution())
            .unwrap_or(Pos::START);
        let rule = v.rule.to_string();
        let lower = rule.to_lowercase();
        let fix = if lower.contains("auth") {
            Fix::hint(
                "add_auth",
                &entity,
                format!("add requires:auth to satisfy '{}'", rule),
            )
        } else if lower.contains("bind") {
            Fix::hint(
                "add_bind",
                v.violation.to_string(),
                format!("add a bind block to satisfy '{}'", rule),
            )
        } else {
            Fix::hint("edit", "", format!("change the app to satisfy '{}'", rule))
        };
        let mut context = Vec::new();
        if !entity.is_empty() {
            context.push(("entity", entity.clone()));
        }
        out.push(Diagnostic {
            code: "CONSTITUTION_001".into(),
            severity: if profile.strict {
                Severity::Error
            } else {
                Severity::Warning
            },
            category: "constitution",
            message: v.violation.to_string(),
            pos,
            context,
            fix,
            rule: Some(rule),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(src: &str, strict: bool) -> Report {
        match parser::parse_diagnostics(src) {
            Ok(nodes) => validate("app.cronus", src, &nodes, Profile { strict }),
            Err(errs) => from_parse_errors("app.cronus", &errs),
        }
    }

    fn assert_no_zero_positions(json: &Value) {
        for key in ["errors", "warnings"] {
            for d in json[key].as_array().unwrap() {
                let loc = &d["location"];
                assert!(loc["line"].as_u64().unwrap() >= 1, "{}", d);
                assert!(loc["col"].as_u64().unwrap() >= 1, "{}", d);
                assert_eq!(loc["file"], "app.cronus");
            }
        }
    }

    #[test]
    fn field_constraint_error_has_validation_category_and_location() {
        let r = report("entity T {\n  slug slug match:\"(\"\n}\n", true);
        let j = r.to_json();
        assert_eq!(j["valid"], false);
        let e = &j["errors"][0];
        assert_eq!(e["code"], "FIELD_001", "{j}");
        assert_eq!(e["category"], "validation");
        assert_eq!(e["location"]["line"], 2);
        assert_eq!(e["location"]["col"], 13);
        assert!(e["fix"]["hint"].is_string());
        assert_no_zero_positions(&j);
    }

    #[test]
    fn env_names_without_a_prefix_are_warnings() {
        let src = "app \"A\" { port 5175 }\nenv {\n  STRIPE string\n  APP_OK string\n}\n";
        let j = report(src, true).to_json();
        let env: Vec<&Value> = j["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|w| w["code"] == "ENV_003")
            .collect();
        assert_eq!(env.len(), 1, "{j}");
        assert_eq!(env[0]["location"]["line"], 3);
        assert_eq!(env[0]["fix"]["target"], "STRIPE");
        assert!(j["errors"]
            .as_array()
            .unwrap()
            .iter()
            .all(|e| e["code"] != "ENV_003"));
    }

    fn codes_of(j: &Value) -> Vec<String> {
        j["errors"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["code"].as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn unknown_where_operator_is_bind_001_not_silent_eq() {
        let src = "page \"/p\" type:custom {\n  section table {\n    bind Task { query all where status in:\"paid\" }\n    columns \"status\"\n  }\n}\n";
        let j = report(src, true).to_json();
        assert_eq!(j["valid"], false, "{j}");
        let e = j["errors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["code"] == "BIND_001")
            .unwrap_or_else(|| panic!("{j}"));
        assert_eq!(e["category"], "bind");
        assert!(e["message"].as_str().unwrap().contains("in"), "{e}");
        assert_no_zero_positions(&j);
    }

    #[test]
    fn unknown_query_kind_is_bind_002() {
        let src = "page \"/p\" type:custom {\n  section table {\n    bind Task { query every }\n    columns \"title\"\n  }\n}\n";
        let j = report(src, true).to_json();
        assert!(codes_of(&j).contains(&"BIND_002".to_string()), "{j}");
        assert_eq!(j["errors"][0]["category"], "bind");
    }

    #[test]
    fn indexed_modifier_is_field_004_with_index_fix() {
        let src = "entity Task {\n  title string indexed\n}\n";
        let j = report(src, true).to_json();
        let e = j["errors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["code"] == "FIELD_004")
            .unwrap_or_else(|| panic!("{j}"));
        assert_eq!(e["fix"]["replacement"], "index");
        assert_eq!(e["category"], "validation");
    }

    #[test]
    fn unknown_action_verb_is_action_001() {
        let src = "page \"/p\" type:custom {\n  section form {\n    bind Task { query all }\n    on submit { log \"x\" toast \"ok\" info }\n  }\n}\n";
        let j = report(src, true).to_json();
        assert!(codes_of(&j).iter().any(|c| c == "ACTION_001"), "{j}");
    }

    #[test]
    fn create_action_is_not_action_001() {
        let src = "entity Task { title string! }\npage \"/p\" type:custom {\n  section form {\n    bind Task { query all }\n    on submit { create Task toast \"ok\" info }\n  }\n}\n";
        let j = report(src, true).to_json();
        assert!(
            codes_of(&j).iter().all(|c| c != "ACTION_001"),
            "create must stay parseable for forms: {j}"
        );
    }

    #[test]
    fn hollow_top_level_service_is_lang_001() {
        let src = "app \"A\" { port 5175 }\nservice mailer { }\n";
        let j = report(src, true).to_json();
        let e = j["errors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["code"] == "LANG_001")
            .unwrap_or_else(|| panic!("{j}"));
        assert_eq!(e["category"], "language");
        assert!(e["message"].as_str().unwrap().contains("service"), "{e}");
    }

    #[test]
    fn type_typo_report_has_location_span_and_replacement() {
        let r = report("entity Task {\n  title strin!\n}\n", true);
        let j = r.to_json();
        assert_eq!(j["schema_version"], 1);
        assert_eq!(j["valid"], false);
        assert_eq!(j["exit_code"], 1);
        let e = &j["errors"][0];
        assert_eq!(e["code"], "TYPE_001");
        assert_eq!(e["category"], "type");
        assert_eq!(
            e["location"],
            json!({"file": "app.cronus", "line": 2, "col": 9,
                   "span": {"start": {"line": 2, "col": 9}, "end": {"line": 2, "col": 14}}})
        );
        assert_eq!(
            e["fix"]["replacement"], "string",
            "fix must carry the concrete replacement: {}",
            e
        );
        assert_eq!(e["fix"]["target"], "strin");
        assert_eq!(r.exit_code(), exit::INVALID);
    }

    #[test]
    fn alias_section_is_a_warning_with_replacement_and_real_line() {
        let src = "entity Task {\n  title string!\n}\npage \"/\" {\n  section stats { bind Task { aggregate count } }\n}\n";
        let j = report(src, true).to_json();
        let w = j["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|w| w["code"] == "CONTRACT_004")
            .unwrap_or_else(|| panic!("alias warning missing: {}", j));
        assert_eq!(w["fix"]["replacement"], "kpi");
        assert_eq!(
            (
                w["location"]["line"].as_u64(),
                w["location"]["col"].as_u64()
            ),
            (Some(5), Some(11))
        );
        assert_eq!(w["location"]["page"], "/");
        assert!(j["errors"]
            .as_array()
            .unwrap()
            .iter()
            .all(|e| e["code"] != "CONTRACT_004"));
        assert_no_zero_positions(&j);
    }

    #[test]
    fn unknown_section_suggests_closest_contract_name() {
        let src = "page \"/\" {\n  section tabel { }\n}\n";
        let j = report(src, true).to_json();
        let e = j["errors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["code"] == "CONTRACT_001")
            .unwrap_or_else(|| panic!("{}", j));
        assert_eq!(e["fix"]["replacement"], "table");
        assert_eq!(e["location"]["line"], 2);
        assert_no_zero_positions(&j);
    }

    #[test]
    fn contract_violations_are_warnings_outside_strict_profile() {
        let src = "page \"/\" {\n  section tabel { }\n}\n";
        let r = report(src, false);
        assert!(r.errors().all(|d| !d.code.starts_with("CONTRACT_")));
        assert!(r.warnings().any(|d| d.code == "CONTRACT_001"));
    }

    #[test]
    fn resolve_error_points_at_the_reference() {
        let src = "entity Task {\n  title string!\n}\npage \"/\" {\n  section table { bind Tsk { query all } }\n}\n";
        let j = report(src, true).to_json();
        let e = j["errors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["code"] == "RESOLVE_001")
            .unwrap_or_else(|| panic!("{}", j));
        assert_eq!(
            (
                e["location"]["line"].as_u64(),
                e["location"]["col"].as_u64()
            ),
            (Some(5), Some(24))
        );
        assert_eq!(e["fix"]["replacement"], "Task");
        assert_no_zero_positions(&j);
    }

    #[test]
    fn declared_page_missing_from_ast_is_structure_001() {
        // A hand-built AST standing in for a parser that nested page "/b"
        // inside "/a": the source declares two pages, the AST holds one.
        let src = "page \"/a\" {\n  section hero { }\n}\npage \"/b\" {\n  section hero { }\n}\n";
        let only_first =
            parser::parse("page \"/a\" {\n  section hero { }\n}\n").unwrap_or_default();
        let r = validate("app.cronus", src, &only_first, Profile { strict: false });
        let e = r
            .errors()
            .find(|d| d.code == "STRUCTURE_001")
            .unwrap_or_else(|| panic!("{:?}", r.diagnostics));
        assert_eq!((e.pos.line, e.pos.col), (4, 6));
        assert!(e.message.contains("page '/b'"), "{}", e.message);
        assert!(!r.valid());
        // the real parser keeps both pages → no structure error
        let nodes = parser::parse(src).unwrap_or_default();
        let ok = validate("app.cronus", src, &nodes, Profile { strict: false });
        assert!(ok.diagnostics.iter().all(|d| d.code != "STRUCTURE_001"));
    }

    #[test]
    fn valid_minimal_app_is_valid_with_empty_arrays() {
        let r = report(
            "app \"T\" {\n  port 5175\n}\nentity Task {\n  title string!\n}\n",
            true,
        );
        let j = r.to_json();
        assert_eq!(j["valid"], true, "{}", j);
        assert_eq!(j["exit_code"], 0);
        assert!(j["errors"].as_array().unwrap().is_empty());
        assert!(j["warnings"].is_array());
        assert_eq!(j["context"]["entities"], 1);
    }

    #[test]
    fn usage_error_exits_2_with_located_error() {
        let r = usage_error(
            "missing.cronus",
            "IO_001",
            "cannot read".into(),
            "check the path",
        );
        assert_eq!(r.exit_code(), exit::USAGE);
        let j = r.to_json();
        assert_eq!(j["errors"][0]["code"], "IO_001");
        assert_eq!(j["errors"][0]["location"]["line"], 1);
        assert_eq!(j["errors"][0]["location"]["col"], 1);
    }

    #[test]
    fn human_render_is_one_line_per_diagnostic_with_fix() {
        let r = report("entity Task {\n  title strin!\n}\n", false);
        assert_eq!(
            r.render_human(false),
            "app.cronus:2:9: error[TYPE_001]: unknown field type 'strin' for field 'title'\n    fix: replace 'strin' with 'string'\n"
        );
        assert_eq!(
            r.verdict(false),
            "  \u{2717} app.cronus — build blocked: 1 error(s), 0 warning(s)"
        );
    }
}
