use serde_json::{json, Value};
use std::fs;
use std::path::Path;

use crate::cli::brief::brief_toml_arr;
use crate::cli::context_grammar as grammar;
use crate::find_cronus_file;
use crate::parser::{
    self, ApiNode, AppNode, AstNode, AuthNode, EntityNode, FieldNode, FieldType, PageNode,
    StyleNode,
};

pub fn cmd_context(args: &[String]) {
    let compact = args.iter().any(|a| a == "--compact");
    let for_claude = args.iter().any(|a| a == "--for-claude");
    let section_filter = args
        .windows(2)
        .find(|w| w[0] == "--section")
        .map(|w| w[1].clone());

    // Find and parse .cronus file (skip --flag values)
    let skip_values: Vec<&str> = vec!["--section", "--output", "--format"];
    let file = args
        .iter()
        .skip(2)
        .enumerate()
        .filter(|(i, a)| {
            !a.starts_with("--")
                && !args
                    .get(i + 1)
                    .map(|prev| skip_values.contains(&prev.as_str()))
                    .unwrap_or(false)
                && !section_filter
                    .as_ref()
                    .map(|sf| sf == a.as_str())
                    .unwrap_or(false)
        })
        .map(|(_, a)| a.clone())
        .next()
        .or_else(find_cronus_file)
        .unwrap_or_else(|| {
            eprintln!("  \x1b[31m✗\x1b[0m No .cronus file found");
            std::process::exit(1);
        });

    let source = match fs::read_to_string(&file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m Cannot read {}: {}", file, e);
            std::process::exit(1);
        }
    };

    if for_claude {
        // Markdown for an assistant's system prompt. Works on files that do
        // not parse: the build status then carries the parse error.
        print!("{}", render_for_claude(&file, &source));
        return;
    }

    let nodes = match parser::parse(&source) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e);
            std::process::exit(1);
        }
    };

    // Extract AST parts
    let mut app = AppNode {
        name: "CRONUS App".into(),
        stack: vec![],
        port: 5175,
        database: None,
        tailwind_config: None,
        constitution: None,
        graphql: true,
        doc: None,
        span: Default::default(),
    };
    let mut entities: Vec<EntityNode> = vec![];
    let mut pages: Vec<PageNode> = vec![];
    let mut style: Option<StyleNode> = None;
    let mut apis: Vec<ApiNode> = vec![];
    let mut webhooks: Vec<parser::WebhookNode> = vec![];
    let mut auth_node: Option<AuthNode> = None;

    for node in &nodes {
        match node {
            AstNode::App(a) => app = a.clone(),
            AstNode::Entity(e) => entities.push(e.clone()),
            AstNode::Page(p) => pages.push(p.clone()),
            AstNode::Style(s) => style = Some(s.clone()),
            AstNode::Api(a) => apis.push(a.clone()),
            AstNode::Webhook(w) => webhooks.push(w.clone()),
            AstNode::Auth(a) => auth_node = Some(a.clone()),
            _ => {}
        }
    }

    // Collect rules: inline constitution from app block + .cronus/constitution.toml
    let mut rules: Vec<String> = Vec::new();
    if let Some(ref c) = app.constitution {
        rules.extend(c.must.iter().cloned());
    }
    if let Some(ref toml_content) = fs::read_to_string(".cronus/constitution.toml").ok() {
        rules.extend(brief_toml_arr(toml_content, "must"));
    }

    {
        // JSON output
        let entities_json: Vec<Value> = entities
            .iter()
            .map(|e| {
                let fields: Vec<Value> = e
                    .fields
                    .iter()
                    .map(|f| {
                        let mut fj = json!({
                            "name": f.name,
                            "type": format!("{:?}", f.field_type).to_lowercase(),
                            "required": f.required,
                        });
                        if f.unique {
                            fj["unique"] = json!(true);
                        }
                        if f.sensitive {
                            fj["sensitive"] = json!(true);
                        }
                        if f.optional {
                            fj["optional"] = json!(true);
                        }
                        if f.searchable {
                            fj["searchable"] = json!(true);
                        }
                        if f.index {
                            fj["index"] = json!(true);
                        }
                        if f.array {
                            fj["array"] = json!(true);
                        }
                        if let Some(ref vals) = f.enum_values {
                            fj["enum_values"] = json!(vals);
                        }
                        if let Some(ref r) = f.reference {
                            fj["reference"] = json!(r);
                        }
                        fj
                    })
                    .collect();
                json!({
                    "name": e.name,
                    "shared": e.shared,
                    "fields": fields,
                })
            })
            .collect();

        let pages_json: Vec<Value> = pages
            .iter()
            .map(|p| {
                let sections: Vec<Value> = p
                    .sections
                    .iter()
                    .map(|s| {
                        let mut sj = json!({ "type": s.section_type });
                        if let Some(ref t) = s.title {
                            sj["title"] = json!(t);
                        }
                        if let Some(ref t) = s.subtitle {
                            sj["subtitle"] = json!(t);
                        }
                        if !s.config.is_empty() {
                            sj["config"] = json!(s.config);
                        }
                        sj
                    })
                    .collect();
                let mut pj = json!({
                    "route": p.route,
                    "type": p.page_type,
                    "sections": sections,
                });
                if let Some(ref t) = p.title {
                    pj["title"] = json!(t);
                }
                if let Some(ref e) = p.entity {
                    pj["entity"] = json!(e);
                }
                if let Some(ref r) = p.requires {
                    pj["requires"] = json!(r);
                } else if let Some(r) = p.config.get("requires") {
                    pj["requires"] = json!(r);
                }
                pj
            })
            .collect();

        let apis_json: Vec<Value> = apis
            .iter()
            .map(|a| {
                let routes: Vec<Value> = a
                    .routes
                    .iter()
                    .map(|r| {
                        let mut rj = json!({
                            "name": r.name,
                            "method": format!("{:?}", r.method),
                            "path": r.path,
                        });
                        if !r.auth.is_empty() {
                            rj["auth"] = json!(r.auth);
                        }
                        if !r.roles.is_empty() {
                            rj["roles"] = json!(r.roles);
                        }
                        rj
                    })
                    .collect();
                json!({
                    "prefix": a.prefix,
                    "routes": routes,
                })
            })
            .collect();

        let webhooks_json: Vec<Value> = webhooks
            .iter()
            .map(|w| {
                let hooks: Vec<Value> = w
                    .hooks
                    .iter()
                    .map(|h| {
                        json!({
                            "event": h.event,
                            "method": h.method,
                            "url": h.url,
                        })
                    })
                    .collect();
                json!({
                    "entity": w.entity,
                    "hooks": hooks,
                })
            })
            .collect();

        let mut ctx = json!({
            "app": {
                "name": app.name,
                "port": app.port,
                "stack": app.stack,
                "database": app.database.as_ref().map(|d| json!({
                    "type": d.db_type,
                    "path": d.path,
                })),
            },
            "entities": entities_json,
            "pages": pages_json,
            "apis": apis_json,
            "webhooks": webhooks_json,
        });

        if let Some(ref s) = style {
            let mut sj = json!({});
            if let Some(ref t) = s.theme {
                sj["theme"] = json!(t);
            }
            if let Some(ref a) = s.accent {
                sj["accent"] = json!(a);
            }
            if let Some(ref r) = s.radius {
                sj["radius"] = json!(r);
            }
            if let Some(ref f) = s.font {
                sj["font"] = json!(f);
            }
            if !s.config.is_empty() {
                sj["config"] = json!(s.config);
            }
            ctx["style"] = sj;
        }

        if let Some(ref auth) = auth_node {
            ctx["auth"] = json!({
                "entity": auth.entity,
                "session_type": auth.session_type,
                "login_fields": auth.login_fields,
                "roles": auth.roles,
            });
        }

        if !rules.is_empty() {
            ctx["rules"] = json!(rules);
        }

        // --section filter: extract just one key from the JSON object
        let output = if let Some(ref section) = section_filter {
            if let Some(val) = ctx.get(section) {
                val.clone()
            } else {
                let valid: Vec<&str> = ctx
                    .as_object()
                    .map(|o| o.keys().map(|k| k.as_str()).collect())
                    .unwrap_or_default();
                eprintln!(
                    "  \x1b[31m✗\x1b[0m Unknown section '{}'. Valid: {}",
                    section,
                    valid.join(", ")
                );
                std::process::exit(1);
            }
        } else {
            ctx
        };

        if compact {
            println!("{}", serde_json::to_string(&output).unwrap());
        } else {
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
    }
}

/// Constitution `must` rules from the app block and `.cronus/constitution.toml`
/// next to the file. Rules that prescribe a JS helper call (`formatPrice()`)
/// are dropped: `.cronus` authors never write JS.
fn project_rules(file: &str, nodes: &[AstNode]) -> Vec<String> {
    let mut rules: Vec<String> = Vec::new();
    for node in nodes {
        if let AstNode::App(a) = node {
            if let Some(ref c) = a.constitution {
                rules.extend(c.must.iter().cloned());
                rules.extend(c.never.iter().map(|n| format!("never: {}", n)));
            }
        }
    }
    let dir = Path::new(file).parent().unwrap_or(Path::new(""));
    if let Ok(toml) = fs::read_to_string(dir.join(".cronus/constitution.toml")) {
        rules.extend(brief_toml_arr(&toml, "must"));
    }
    let js_call = regex::Regex::new(r"[A-Za-z_][A-Za-z0-9_]*\(\)").expect("valid regex");
    rules.retain(|r| !js_call.is_match(r));
    rules
}

fn field_summary(f: &FieldNode) -> String {
    let mut s = match (&f.field_type, &f.reference) {
        (FieldType::Relation, Some(r)) => format!("{} -> {}", f.name, r),
        (FieldType::Enum, _) => format!(
            "{} enum [{}]",
            f.name,
            f.enum_values.clone().unwrap_or_default().join(", ")
        ),
        (t, _) => format!("{} {}", f.name, format!("{:?}", t).to_lowercase()),
    };
    if f.required {
        s.push('!');
    }
    for (on, word) in [
        (f.unique, "unique"),
        (f.sensitive, "sensitive"),
        (f.index, "index"),
        (f.searchable, "searchable"),
    ] {
        if on {
            s.push(' ');
            s.push_str(word);
        }
    }
    if let Some(ref d) = f.default_value {
        s.push_str(&format!(" default:{}", d));
    }
    s
}

/// Markdown context for an assistant: the project, its current
/// `cronus build --ai` result, and the canonical grammar with the section and
/// field types derived from the kernel (see `context_grammar`).
pub fn render_for_claude(file: &str, source: &str) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let parsed = parser::parse(source);

    match &parsed {
        Ok(nodes) => {
            let app = nodes.iter().find_map(|n| match n {
                AstNode::App(a) => Some(a),
                _ => None,
            });
            let name = app
                .map(|a| {
                    a.name
                        .split('|')
                        .next()
                        .unwrap_or(&a.name)
                        .trim()
                        .to_string()
                })
                .unwrap_or_else(|| file.to_string());
            let _ = writeln!(out, "# Project: {}\n", name);
            let _ = write!(out, "File: {}", file);
            if let Some(a) = app {
                let db = a
                    .database
                    .as_ref()
                    .map(|d| d.db_type.as_str())
                    .unwrap_or("sqlite (default)");
                let _ = write!(out, " | Port: {} | DB: {}", a.port, db);
            }
            for n in nodes {
                if let AstNode::Auth(auth) = n {
                    let _ = write!(
                        out,
                        " | Auth: entity {} (roles: {})",
                        auth.entity,
                        auth.roles.join(", ")
                    );
                }
            }
            let _ = writeln!(out, "\n");

            let entities: Vec<&EntityNode> = nodes
                .iter()
                .filter_map(|n| match n {
                    AstNode::Entity(e) => Some(e),
                    _ => None,
                })
                .collect();
            if !entities.is_empty() {
                let _ = writeln!(out, "## Entities");
                let auth_entity = nodes.iter().find_map(|n| match n {
                    AstNode::Auth(a) => Some(a.entity.as_str()),
                    _ => None,
                });
                for e in &entities {
                    // Mirrors access.rs: User/Users and the auth entity are account tables.
                    let is_account = e.name == "User"
                        || e.name == "Users"
                        || auth_entity == Some(e.name.as_str());
                    let scope = if is_account {
                        "account: self-only, admin sees all"
                    } else if e.shared {
                        "shared"
                    } else {
                        "owner-scoped"
                    };
                    let fields: Vec<String> = e.fields.iter().map(field_summary).collect();
                    let _ = writeln!(out, "- {} ({}): {}", e.name, scope, fields.join(", "));
                }
                let _ = writeln!(out);
            }

            let apis: Vec<&ApiNode> = nodes
                .iter()
                .filter_map(|n| match n {
                    AstNode::Api(a) => Some(a),
                    _ => None,
                })
                .collect();
            if !apis.is_empty() {
                let _ = writeln!(out, "## APIs (only declared routes are served)");
                for api in &apis {
                    let routes: Vec<String> = api
                        .routes
                        .iter()
                        .map(|r| {
                            let auth = if r.auth.is_empty() {
                                "jwt".to_string()
                            } else {
                                r.auth.clone()
                            };
                            format!("{} {:?} {} auth:{}", r.name, r.method, r.path, auth)
                        })
                        .collect();
                    let _ = writeln!(out, "- {}: {}", api.prefix, routes.join("; "));
                }
                let _ = writeln!(out);
            }

            let pages: Vec<&PageNode> = nodes
                .iter()
                .filter_map(|n| match n {
                    AstNode::Page(p) => Some(p),
                    _ => None,
                })
                .collect();
            if !pages.is_empty() {
                let _ = writeln!(out, "## Pages");
                for p in &pages {
                    let requires = p
                        .requires
                        .as_deref()
                        .or_else(|| p.config.get("requires").map(|s| s.as_str()))
                        .map(|r| format!(" requires:{}", r))
                        .unwrap_or_else(|| " (public)".to_string());
                    let sections: Vec<String> = p
                        .sections
                        .iter()
                        .map(|s| match s.binding {
                            Some(ref b) => format!("{} [bind {}]", s.section_type, b.entity),
                            None => s.section_type.clone(),
                        })
                        .collect();
                    let _ = writeln!(
                        out,
                        "- {} type:{}{} — sections: {}",
                        p.route,
                        p.page_type,
                        requires,
                        if sections.is_empty() {
                            "none".to_string()
                        } else {
                            sections.join(", ")
                        }
                    );
                }
                let _ = writeln!(out);
            }

            let hooks: Vec<&parser::WebhookNode> = nodes
                .iter()
                .filter_map(|n| match n {
                    AstNode::Webhook(w) => Some(w),
                    _ => None,
                })
                .collect();
            if !hooks.is_empty() {
                let _ = writeln!(out, "## Webhooks");
                for wh in hooks {
                    for h in &wh.hooks {
                        let _ = writeln!(
                            out,
                            "- {} on {} -> {} {}",
                            wh.entity, h.event, h.method, h.url
                        );
                    }
                }
                let _ = writeln!(out);
            }

            let rules = project_rules(file, nodes);
            if !rules.is_empty() {
                let _ = writeln!(out, "## Project rules (constitution)");
                for r in &rules {
                    let _ = writeln!(out, "- {}", r);
                }
                let _ = writeln!(out);
            }
        }
        Err(_) => {
            let _ = writeln!(
                out,
                "# Project: {}\n\nThe file does not parse; fix the build error below first.\n",
                file
            );
        }
    }

    // Current build result, same passes as `cronus build --ai`.
    let build = match &parsed {
        Ok(nodes) => {
            let (e, p, r) = parser::stats(nodes);
            crate::cli::build::build_ai_error_json(nodes, file, e, p, r)
        }
        Err(e) => json!({
            "valid": false,
            "errors": [{"code": "PARSE_001", "severity": "fatal", "message": e}],
        }),
    };
    let _ = writeln!(out, "## Build status (`cronus build --ai`)");
    let errors = build["errors"].as_array().cloned().unwrap_or_default();
    if build["valid"].as_bool() == Some(true) {
        let warnings = build["warnings"].as_array().map_or(0, Vec::len);
        let _ = writeln!(out, "Valid: no errors, {warnings} warning(s).\n");
    } else {
        let _ = writeln!(
            out,
            "{} problem(s). Fix these before adding features:",
            errors.len()
        );
        for err in &errors {
            let hint = err["fix"]["hint"]
                .as_str()
                .map(|h| format!(" — fix: {}", h))
                .unwrap_or_default();
            let _ = writeln!(
                out,
                "- [{} {}] {}{}",
                err["code"].as_str().unwrap_or("?"),
                err["severity"].as_str().unwrap_or("error"),
                err["message"].as_str().unwrap_or(""),
                hint
            );
        }
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "# CRONUS language\n");
    let _ = writeln!(out, "{}\n", grammar::grammar_summary());
    let _ = writeln!(out, "## Valid field types");
    let _ = writeln!(out, "{}\n", grammar::FIELD_TYPES.join(", "));
    let _ = writeln!(out, "## Valid section types");
    let _ = writeln!(
        out,
        "Built-in: {}\n",
        grammar::BUILTIN_SECTION_TYPES.join(", ")
    );
    let _ = writeln!(
        out,
        "Aliases (prefer the canonical name): {}\n",
        grammar::SECTION_ALIASES
            .iter()
            .map(|a| format!(
                "{} -> {}",
                a,
                crate::contracts::ContractRegistry::resolve_alias(a).unwrap_or("?")
            ))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let _ = writeln!(
        out,
        "cronus-ui families (component `style:<family>`; also a section type when not built-in): {}\n",
        grammar::family_section_types().join(", ")
    );
    let _ = writeln!(out, "{}", grammar::never_list());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_project(name: &str, source: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("cronus-context-{}-{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".cronus")).unwrap();
        fs::write(dir.join("app.cronus"), source).unwrap();
        dir
    }

    const APP: &str = r#"app "Shop" {
  port 5180
}
entity Product {
  name  string!
  price money! min:0
  secret string sensitive
}
api /products {
  list GET / auth:public
  create POST / auth:jwt
}
page "/products" type:custom requires:auth {
  section table {
    bind Product { query all }
    columns "name, price"
  }
}
"#;

    #[test]
    fn for_claude_lists_project_grammar_and_types() {
        let dir = temp_project("ok", APP);
        fs::write(
            dir.join(".cronus/constitution.toml"),
            "must = [\"prices in centavos — use formatPrice()\", \"every page requires auth\"]\n",
        )
        .unwrap();
        let file = dir.join("app.cronus").to_string_lossy().to_string();
        let out = render_for_claude(&file, APP);

        assert!(out.contains("# Project: Shop"), "{out}");
        assert!(
            out.contains(
                "Product (owner-scoped): name string!, price money!, secret string sensitive"
            ),
            "{out}"
        );
        assert!(out.contains("/products: list GET / auth:public"), "{out}");
        assert!(
            out.contains("/products type:custom requires:auth — sections: table [bind Product]"),
            "{out}"
        );
        assert!(out.contains("## Build status"), "{out}");
        assert!(out.contains("### entity"), "grammar missing: {out}");
        assert!(out.contains("money, percentage"), "field types missing");
        assert!(out.contains("kanban"), "section types missing");
        assert!(out.contains("data-table"), "families missing");
        assert!(out.contains("Never do"), "never list missing");
        assert!(out.contains("every page requires auth"), "{out}");
        assert!(!out.contains("formatPrice"), "JS advice leaked: {out}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn for_claude_marks_account_entities() {
        let src = "auth {\n  entity Member\n  login email + password\n}\nentity Member {\n  email email! unique\n}\nentity Plan shared {\n  name string!\n}\n";
        let out = render_for_claude("app.cronus", src);
        assert!(
            out.contains("- Member (account: self-only, admin sees all)"),
            "{out}"
        );
        assert!(out.contains("- Plan (shared)"), "{out}");
    }

    #[test]
    fn for_claude_reports_current_build_errors() {
        let src = APP.replace("section table", "section tabel");
        let dir = temp_project("bad-section", &src);
        let file = dir.join("app.cronus").to_string_lossy().to_string();
        let out = render_for_claude(&file, &src);
        assert!(out.contains("problem(s)"), "{out}");
        assert!(out.contains("tabel"), "{out}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn for_claude_survives_parse_errors() {
        // `bind X` without braces is a parse error.
        let out = render_for_claude("broken.cronus", "page \"/x\" { section kpi { bind X } }");
        assert!(out.contains("does not parse"), "{out}");
        assert!(out.contains("PARSE_001"), "{out}");
        assert!(out.contains("### bind"), "grammar still printed: {out}");
    }
}
