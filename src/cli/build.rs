use serde_json::{json, Value};
use std::fs;

use crate::ast_diff;
use crate::constitution_check;
use crate::contracts;
use crate::hardcode_lint;
use crate::lint;
use crate::parser::{self, AstNode};
use crate::resolve;
use crate::{find_cronus_file, LAST_AI_ERRORS};

pub fn cmd_build(args: &[String]) {
    let strict_ai = args.iter().any(|a| a == "--strict-ai");
    let ai_mode = args
        .iter()
        .any(|a| a == "--ai" || a == "--machine" || a == "--json-errors");
    let strict = args.iter().any(|a| a == "--strict") || strict_ai;
    let strict_audit = args.iter().any(|a| a == "--strict-audit");
    let file = args.iter().skip(2)
        .find(|a| !a.starts_with("--"))
        .cloned()
        .or_else(find_cronus_file)
        .unwrap_or_else(|| {
            if ai_mode {
                let result = json!({
                    "valid": false,
                    "errors": [{"code": "PARSE_001", "severity": "fatal", "category": "filesystem", "message": "No .cronus file found", "location": {}, "fix": {"action": "add", "target": "*.cronus", "hint": "Create a .cronus file in the current directory or specify a path"}}],
                    "context": {"entities": 0, "pages": 0, "resolve_errors": 0, "lint_errors": 0, "constitution_violations": 0, "total_errors": 1}
                });
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
                *LAST_AI_ERRORS.lock().unwrap() = Some(result);
            } else if strict_ai {
                println!("{}", json!({"valid": false, "errors": [{"message": "No .cronus file found"}]}));
            } else {
                eprintln!("  \x1b[31m\u{2717}\x1b[0m No .cronus file found");
            }
            std::process::exit(1);
        });

    let source = fs::read_to_string(&file).unwrap();
    match parser::parse(&source) {
        Ok(nodes) => {
            let (entities, pages, routes) = parser::stats(&nodes);

            // ── AI-Error Protocol mode (--ai / --machine / --json-errors) ──
            if ai_mode {
                let ai_result = build_ai_error_json(&nodes, &file, entities, pages, routes);
                println!("{}", serde_json::to_string_pretty(&ai_result).unwrap());
                let valid = ai_result["valid"].as_bool().unwrap_or(false);
                *LAST_AI_ERRORS.lock().unwrap() = Some(ai_result);
                if !valid {
                    std::process::exit(1);
                }
                // Save snapshot silently in AI mode (no stdout pollution)
                let snapshot = ast_diff::snapshot_from_ast(&nodes);
                let _ = fs::create_dir_all(".cronus");
                if let Ok(json_str) = serde_json::to_string_pretty(&snapshot) {
                    let _ = fs::write(".cronus/ast-snapshot.json", &json_str);
                }
                return;
            }

            // In strict-ai mode, run contract validation and promote warnings to errors
            if strict_ai {
                let mut errors: Vec<Value> = Vec::new();
                for node in &nodes {
                    if let AstNode::Page(page) = node {
                        for section in &page.sections {
                            let section_warnings = contracts::validate_section(section, &[]);
                            for w in section_warnings {
                                let err_json = match w {
                                    contracts::ParseWarning::UnknownSection { ref name, line } => {
                                        json!({"type": "unknown_section", "section": name, "line": line, "severity": "error", "message": format!("Unknown section type '{}'", name)})
                                    }
                                    contracts::ParseWarning::UnknownKey {
                                        ref section,
                                        ref key,
                                        ref item,
                                        line,
                                    } => {
                                        json!({"type": "unknown_key", "section": section, "key": key, "item": item, "line": line, "severity": "error", "message": format!("Unknown key '{}' in section '{}'", key, section)})
                                    }
                                    contracts::ParseWarning::MissingRequired {
                                        ref section,
                                        ref key,
                                        ref item,
                                        line,
                                    } => {
                                        json!({"type": "missing_required", "section": section, "key": key, "item": item, "line": line, "severity": "error", "message": format!("Missing required key '{}' in section '{}'", key, section)})
                                    }
                                    contracts::ParseWarning::AliasUsed {
                                        ref alias,
                                        ref canonical,
                                        line,
                                    } => {
                                        json!({"type": "alias", "alias": alias, "canonical": canonical, "line": line, "severity": "error", "message": format!("'{}' is an alias for '{}'", alias, canonical)})
                                    }
                                    contracts::ParseWarning::MinItemsViolation {
                                        ref section,
                                        expected,
                                        actual,
                                        line,
                                    } => {
                                        json!({"type": "min_items", "section": section, "expected": expected, "actual": actual, "line": line, "severity": "error", "message": format!("Section '{}' requires at least {} items, found {}", section, expected, actual)})
                                    }
                                    contracts::ParseWarning::UnknownConfig {
                                        ref section,
                                        ref key,
                                        line,
                                    } => {
                                        json!({"type": "unknown_config", "section": section, "key": key, "line": line, "severity": "error", "message": format!("Unknown config key '{}' in section '{}'", key, section)})
                                    }
                                };
                                errors.push(err_json);
                            }
                        }
                    }
                }
                // Hardcode lint — render each page and check for hardcoded content
                let mut all_pages = Vec::new();
                let mut all_entities = Vec::new();
                let mut style_node: Option<parser::StyleNode> = None;
                for node in &nodes {
                    match node {
                        AstNode::Page(p) => all_pages.push(p.clone()),
                        AstNode::Entity(e) => all_entities.push(e.clone()),
                        AstNode::Style(s) => style_node = Some(s.clone()),
                        _ => {}
                    }
                }
                let hc_findings =
                    hardcode_lint::lint_all_pages(&all_pages, &all_entities, style_node.as_ref());
                for f in &hc_findings {
                    errors.push(json!({
                        "type": "hardcoded_content",
                        "page": f.page,
                        "text": f.text,
                        "severity": f.severity,
                        "message": format!("Hardcoded text '{}' in page '{}' — should come from .cronus data", f.text, f.page),
                    }));
                }

                let valid = errors.is_empty();
                let result = json!({
                    "valid": valid,
                    "file": file,
                    "errors": errors,
                    "stats": { "entities": entities, "pages": pages, "routes": routes },
                });
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
                if !valid {
                    std::process::exit(1);
                }
            } else if strict {
                // --strict (non-AI) — human-readable hardcode warnings
                let mut all_pages = Vec::new();
                let mut all_entities = Vec::new();
                let mut style_node: Option<parser::StyleNode> = None;
                for node in &nodes {
                    match node {
                        AstNode::Page(p) => all_pages.push(p.clone()),
                        AstNode::Entity(e) => all_entities.push(e.clone()),
                        AstNode::Style(s) => style_node = Some(s.clone()),
                        _ => {}
                    }
                }
                let hc_findings =
                    hardcode_lint::lint_all_pages(&all_pages, &all_entities, style_node.as_ref());
                println!(
                    "  \x1b[32m\u{2713}\x1b[0m {} — {} entities, {} pages, {} routes",
                    file, entities, pages, routes
                );
                if hc_findings.is_empty() {
                    println!("  \x1b[32m\u{2713}\x1b[0m No hardcoded content detected");
                } else {
                    println!(
                        "  \x1b[33m\u{26a0}\x1b[0m {} hardcoded string(s) found:",
                        hc_findings.len()
                    );
                    for f in &hc_findings {
                        println!("    \x1b[33m\u{2192}\x1b[0m [{}] \"{}\"", f.page, f.text);
                    }
                    println!();
                    println!("  \x1b[90mThese strings appear in rendered HTML but don't trace back to .cronus data.\x1b[0m");
                    println!("  \x1b[90mMove them to section title/subtitle/config/items in the .cronus file.\x1b[0m");
                }
            } else {
                println!(
                    "  \x1b[32m\u{2713}\x1b[0m {} — {} entities, {} pages, {} routes",
                    file, entities, pages, routes
                );
                println!("  \x1b[32m\u{2713}\x1b[0m Valid .cronus file");
            }

            // Resolve pass — verify all cross-references (fatal errors)
            let resolve_start = std::time::Instant::now();
            let (_symbol_table, resolve_errors) = resolve::resolve(&nodes);
            let resolve_ms = resolve_start.elapsed().as_millis();
            if !resolve_errors.is_empty() {
                println!();
                println!("  \x1b[1mResolve Pass\x1b[0m");
                for e in &resolve_errors {
                    println!("{}", e);
                }
                println!();
                println!(
                    "  \x1b[31m{} resolve error(s)\x1b[0m — build blocked ({}ms)",
                    resolve_errors.len(),
                    resolve_ms
                );
                std::process::exit(1);
            } else {
                println!(
                    "  \x1b[32m\u{2713}\x1b[0m Resolve pass: all references valid ({}ms)",
                    resolve_ms
                );
            }

            // Zero Hardcode Enforcement — 7 lint rules
            let lint_start = std::time::Instant::now();
            let lint_results = lint::lint_ast(&nodes, strict);
            let lint_ms = lint_start.elapsed().as_millis();
            let errors = lint_results
                .iter()
                .filter(|r| matches!(r.severity, lint::Severity::Error))
                .count();
            let warnings = lint_results
                .iter()
                .filter(|r| matches!(r.severity, lint::Severity::Warning))
                .count();

            if !lint_results.is_empty() {
                println!();
                println!("  \x1b[1mZero Hardcode Lint\x1b[0m ({} rules)", 7);
                for r in &lint_results {
                    println!("{}", r);
                }
                println!();
                if errors > 0 {
                    println!(
                        "  \x1b[31m{} error(s)\x1b[0m, {} warning(s) — build blocked",
                        errors, warnings
                    );
                    std::process::exit(1);
                } else {
                    println!("  {} warning(s)", warnings);
                }
            } else {
                println!(
                    "  \x1b[32m\u{2713}\x1b[0m Zero hardcode lint: all 7 rules passed ({}ms)",
                    lint_ms
                );
            }

            // Constitution enforcement — check rules against AST
            let constitution_app = nodes.iter().find_map(|n| {
                if let AstNode::App(a) = n {
                    Some(a)
                } else {
                    None
                }
            });
            if let Some(app_node) = constitution_app {
                if let Some(ref c) = app_node.constitution {
                    let cv = constitution_check::check_constitution(&nodes, c);
                    let real_violations: Vec<_> =
                        cv.iter().filter(|v| v.rule_type != "info").collect();
                    let info_count = cv.len() - real_violations.len();
                    if !real_violations.is_empty() {
                        println!();
                        println!(
                            "  \x1b[1mConstitution Check\x1b[0m ({} rules)",
                            c.must.len() + c.never.len()
                        );
                        for v in &cv {
                            println!("{}", v);
                        }
                        println!();
                        println!(
                            "  \x1b[31m{} violation(s)\x1b[0m{}",
                            real_violations.len(),
                            if info_count > 0 {
                                format!(", {} informational", info_count)
                            } else {
                                String::new()
                            }
                        );
                        if strict {
                            std::process::exit(1);
                        }
                    } else {
                        println!(
                            "  \x1b[32m\u{2713}\x1b[0m Constitution: all {} rules pass{}",
                            c.must.len() + c.never.len(),
                            if info_count > 0 {
                                format!(" ({} informational)", info_count)
                            } else {
                                String::new()
                            }
                        );
                    }
                }
            }

            // Save AST snapshot for changelog diffing
            save_ast_snapshot(&nodes);

            // ── Auto-audit: run fidelity check if .cronus/audit-ref.html exists ──
            let audit_ref_path = ".cronus/audit-ref.html";
            if std::path::Path::new(audit_ref_path).exists() && !ai_mode {
                println!();
                println!("  \x1b[1mFidelity Audit\x1b[0m (auto — .cronus/audit-ref.html found)");
                match crate::cli::audit_fidelity::run_audit(audit_ref_path) {
                    Some(result) => {
                        crate::cli::audit_fidelity::print_fidelity_line(&result);
                        crate::cli::audit_fidelity::save_audit_results(&result);
                        println!("  \x1b[32m✓\x1b[0m Results saved to .cronus/audit-results.json");
                        if strict_audit && result.fidelity < 95 {
                            println!();
                            crate::cli::audit_fidelity::print_missing_top(&result, 5);
                            eprintln!();
                            eprintln!("  \x1b[31m✗ AUDIT FAILED\x1b[0m — {}% < 95% threshold (--strict-audit)", result.fidelity);
                            std::process::exit(1);
                        }
                    }
                    None => {
                        eprintln!("  \x1b[33m⚠\x1b[0m Could not run audit (no .cronus file or parse error)");
                    }
                }
            }
        }
        Err(e) => {
            if ai_mode {
                let result = json!({
                    "valid": false,
                    "errors": [{"code": "PARSE_001", "severity": "fatal", "category": "syntax", "message": format!("{}", e), "location": {}, "fix": {"action": "replace", "target": "", "hint": "Fix the syntax error in the .cronus file"}}],
                    "context": {"entities": 0, "pages": 0, "resolve_errors": 0, "lint_errors": 0, "constitution_violations": 0, "total_errors": 1}
                });
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
                *LAST_AI_ERRORS.lock().unwrap() = Some(result);
            } else if strict_ai {
                println!(
                    "{}",
                    json!({"valid": false, "errors": [{"type": "parse_error", "message": e, "severity": "error"}]})
                );
            } else {
                eprintln!("  \x1b[31m\u{2717}\x1b[0m Parse error: {}", e);
            }
            std::process::exit(1);
        }
    }
}

/// Build the AI-Error Protocol JSON from all validation passes.
/// Collects errors from contract validation, resolve, lint, hardcode lint, and constitution.
fn build_ai_error_json(
    nodes: &[AstNode],
    file: &str,
    entity_count: usize,
    page_count: usize,
    route_count: usize,
) -> Value {
    let mut ai_errors: Vec<Value> = Vec::new();
    let mut resolve_counter = 0usize;
    let mut lint_counter = 0usize;
    let mut constitution_counter = 0usize;
    let mut contract_counter = 0usize;

    // Pass 1: Contract validation (section schemas)
    for node in nodes {
        if let AstNode::Page(page) = node {
            for section in &page.sections {
                let section_warnings = contracts::validate_section(section, &[]);
                for w in section_warnings {
                    contract_counter += 1;
                    let code = format!("CONTRACT_{:03}", contract_counter);
                    let (msg, fix, line) = match w {
                        contracts::ParseWarning::UnknownSection { ref name, line } => (
                            format!("Unknown section type '{}'", name),
                            json!({"action": "replace", "target": name, "hint": "Check valid section types: hero, features, pricing, kpi, table, chart, form, etc."}),
                            line,
                        ),
                        contracts::ParseWarning::UnknownKey {
                            ref section,
                            ref key,
                            ..
                        } => (
                            format!("Unknown key '{}' in section '{}'", key, section),
                            json!({"action": "remove", "target": key, "hint": format!("Remove or replace with a valid key for '{}' sections", section)}),
                            0,
                        ),
                        contracts::ParseWarning::MissingRequired {
                            ref section,
                            ref key,
                            ..
                        } => (
                            format!("Missing required key '{}' in section '{}'", key, section),
                            json!({"action": "add", "target": key, "hint": format!("Add '{}' to the {} section item", key, section)}),
                            0,
                        ),
                        contracts::ParseWarning::AliasUsed {
                            ref alias,
                            ref canonical,
                            line,
                        } => (
                            format!(
                                "'{}' is an alias — use canonical name '{}'",
                                alias, canonical
                            ),
                            json!({"action": "replace", "target": alias, "replacement": canonical}),
                            line,
                        ),
                        contracts::ParseWarning::MinItemsViolation {
                            ref section,
                            expected,
                            actual,
                            line,
                        } => (
                            format!(
                                "Section '{}' requires at least {} items, found {}",
                                section, expected, actual
                            ),
                            json!({"action": "add", "target": "item", "hint": format!("Add {} more item(s) to '{}' section", expected - actual, section)}),
                            line,
                        ),
                        contracts::ParseWarning::UnknownConfig {
                            ref section,
                            ref key,
                            line,
                        } => (
                            format!("Unknown config key '{}' in section '{}'", key, section),
                            json!({"action": "remove", "target": key, "hint": format!("Remove unknown config key from '{}' section", section)}),
                            line,
                        ),
                    };
                    ai_errors.push(json!({
                        "code": code,
                        "severity": "error",
                        "category": "contract",
                        "message": msg,
                        "location": {"line": line, "section": format!("{} ({})", section.section_type, page.route), "block": "section"},
                        "fix": fix,
                    }));
                }
            }
        }
    }

    // Pass 2: Resolve — cross-reference validation (fatal)
    let (_symbol_table, resolve_errors) = resolve::resolve(nodes);
    let resolve_error_count = resolve_errors.len();
    for re in &resolve_errors {
        resolve_counter += 1;
        let code = format!("RESOLVE_{:03}", resolve_counter);
        let target_name = re.message.split('\'').nth(1).unwrap_or("").to_string();

        let fix = if let Some(ref sug) = re.suggestion {
            json!({"action": "replace", "target": &target_name, "replacement": sug})
        } else if re.message.contains("must be an enum") {
            json!({"action": "replace", "target": format!("{} field type", target_name), "hint": "Change field type to enum with valid values"})
        } else {
            json!({"action": "add", "target": &target_name, "hint": "Define this entity or reference"})
        };

        let section_hint = if re.message.contains("page '") {
            format!(
                "page {}",
                re.message
                    .split("page '")
                    .nth(1)
                    .and_then(|s| s.split('\'').next())
                    .unwrap_or("")
            )
        } else if re.message.contains("entity '") {
            format!(
                "entity {}",
                re.message
                    .split("entity '")
                    .last()
                    .and_then(|s| s.split('\'').next())
                    .unwrap_or("")
            )
        } else {
            String::new()
        };

        let category = if re.message.contains("Transition") {
            "state_machine"
        } else {
            "reference"
        };
        let block = if re.message.contains("bind") {
            "bind"
        } else if re.message.contains("Transition") {
            "transition"
        } else {
            "reference"
        };

        ai_errors.push(json!({
            "code": code,
            "severity": "fatal",
            "category": category,
            "message": re.message,
            "suggestion": re.suggestion,
            "location": {"section": section_hint, "block": block},
            "fix": fix,
        }));
    }

    // Pass 3: Lint — Zero Hardcode Enforcement (7 rules)
    let lint_results = lint::lint_ast(nodes, true);
    let lint_error_count = lint_results
        .iter()
        .filter(|r| matches!(r.severity, lint::Severity::Error))
        .count();
    let lint_warning_count = lint_results
        .iter()
        .filter(|r| matches!(r.severity, lint::Severity::Warning))
        .count();
    for lr in &lint_results {
        lint_counter += 1;
        let code = format!("LINT_{:03}", lint_counter);
        let severity = match lr.severity {
            lint::Severity::Error => "error",
            lint::Severity::Warning => "warning",
        };

        let fix = if lr.rule.contains("dead-text") || lr.rule.contains("hardcode") {
            json!({"action": "wrap_in_dynamic", "target": lr.message.split('\'').nth(1).unwrap_or(&lr.message), "hint": &lr.fix})
        } else if lr.rule.contains("dead-link") {
            json!({"action": "replace", "target": lr.message.split('\'').nth(1).unwrap_or(""), "hint": &lr.fix})
        } else if lr.rule.contains("bind-or-empty") {
            json!({"action": "add_bind", "target": &lr.section, "hint": &lr.fix})
        } else if lr.rule.contains("sensitive") {
            json!({"action": "remove", "target": lr.message.split('\'').nth(1).unwrap_or(""), "hint": &lr.fix})
        } else if lr.rule.contains("auth") {
            json!({"action": "add_auth", "target": &lr.page, "hint": &lr.fix})
        } else {
            json!({"action": "replace", "target": "", "hint": &lr.fix})
        };

        ai_errors.push(json!({
            "code": code,
            "severity": severity,
            "category": "hardcode",
            "message": lr.message,
            "location": {"section": lr.section, "page": lr.page, "block": "template"},
            "fix": fix,
        }));
    }

    // Pass 3b: Hardcode lint (rendered HTML analysis)
    let mut all_pages = Vec::new();
    let mut all_entities = Vec::new();
    let mut style_node: Option<parser::StyleNode> = None;
    for node in nodes {
        match node {
            AstNode::Page(p) => all_pages.push(p.clone()),
            AstNode::Entity(e) => all_entities.push(e.clone()),
            AstNode::Style(s) => style_node = Some(s.clone()),
            _ => {}
        }
    }
    let hc_findings = hardcode_lint::lint_all_pages(&all_pages, &all_entities, style_node.as_ref());
    for f in &hc_findings {
        lint_counter += 1;
        let code = format!("LINT_{:03}", lint_counter);
        ai_errors.push(json!({
            "code": code,
            "severity": f.severity,
            "category": "hardcode",
            "message": format!("Hardcoded text '{}' in page '{}' — should come from .cronus data", f.text, f.page),
            "location": {"page": f.page, "block": "template"},
            "fix": {"action": "wrap_in_dynamic", "target": &f.text, "hint": "Use <span id='...'>...</span> populated via JS fetch or move to section data"}
        }));
    }

    // Pass 4: Constitution enforcement
    let mut constitution_violation_count = 0usize;
    let constitution_app = nodes.iter().find_map(|n| {
        if let AstNode::App(a) = n {
            Some(a)
        } else {
            None
        }
    });
    if let Some(app_node) = constitution_app {
        if let Some(ref c) = app_node.constitution {
            let cv = constitution_check::check_constitution(nodes, c);
            let real_violations: Vec<_> = cv.iter().filter(|v| v.rule_type != "info").collect();
            constitution_violation_count = real_violations.len();
            for v in &real_violations {
                constitution_counter += 1;
                let code = format!("CONSTITUTION_{:03}", constitution_counter);
                let fix = if v.rule.to_lowercase().contains("auth") {
                    json!({"action": "add_auth", "target": v.entity.clone().unwrap_or_default(), "hint": format!("Add requires: auth to comply with rule: {}", v.rule)})
                } else if v.rule.to_lowercase().contains("bind") {
                    json!({"action": "add_bind", "target": &v.violation, "hint": format!("Add bind block to comply with rule: {}", v.rule)})
                } else {
                    json!({"action": "add", "target": "", "hint": format!("Fix violation of constitution rule: {}", v.rule)})
                };
                ai_errors.push(json!({
                    "code": code,
                    "severity": "fatal",
                    "category": "constitution",
                    "message": v.violation,
                    "rule": v.rule,
                    "entity": v.entity,
                    "location": {"block": "constitution", "section": v.entity.clone().unwrap_or_default()},
                    "fix": fix,
                }));
            }
        }
    }

    let total_errors = ai_errors.len();
    let valid = total_errors == 0;
    json!({
        "valid": valid,
        "errors": ai_errors,
        "context": {
            "file": file,
            "entities": entity_count,
            "pages": page_count,
            "routes": route_count,
            "resolve_errors": resolve_error_count,
            "lint_errors": lint_error_count + hc_findings.len(),
            "lint_warnings": lint_warning_count,
            "constitution_violations": constitution_violation_count,
            "total_errors": total_errors,
        }
    })
}

pub(crate) fn save_ast_snapshot(nodes: &[AstNode]) {
    let snapshot = ast_diff::snapshot_from_ast(nodes);
    let _ = fs::create_dir_all(".cronus");
    match serde_json::to_string_pretty(&snapshot) {
        Ok(json_str) => {
            if fs::write(".cronus/ast-snapshot.json", &json_str).is_ok() {
                println!(
                    "  \x1b[32m\u{2713}\x1b[0m AST snapshot saved to .cronus/ast-snapshot.json"
                );
            }
        }
        Err(e) => {
            eprintln!(
                "  \x1b[33m\u{26a0}\x1b[0m Failed to serialize AST snapshot: {}",
                e
            );
        }
    }
}
