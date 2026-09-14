use serde_json::{json, Value};
use std::fs;

use crate::cli::brief::brief_toml_arr;
use crate::find_cronus_file;
use crate::parser::{self, ApiNode, AppNode, AstNode, AuthNode, EntityNode, PageNode, StyleNode};

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
        doc: None,
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

    if for_claude {
        // Structured Markdown output for Claude system prompt
        let app_name = app.name.split('|').next().unwrap_or(&app.name).trim();
        let db_type = app
            .database
            .as_ref()
            .map(|d| d.db_type.as_str())
            .unwrap_or("none");
        let theme = style
            .as_ref()
            .and_then(|s| s.theme.as_deref())
            .unwrap_or("default");

        println!("# Project: {}", app_name);
        print!("Port: {} | DB: {} | Theme: {}", app.port, db_type, theme);
        if let Some(ref auth) = auth_node {
            print!(" | Auth: {} ({})", auth.entity, auth.session_type);
        }
        println!("\n");

        // Entities
        if !entities.is_empty() {
            println!("## Entities");
            for e in &entities {
                let shared = if e.shared { "shared" } else { "local" };
                let fields: Vec<String> = e.fields.iter().map(|f| f.name.clone()).collect();
                println!("- {} ({}): {}", e.name, shared, fields.join(", "));
            }
            println!();
        }

        // Pages
        if !pages.is_empty() {
            println!("## Pages");
            for p in &pages {
                let title = p.title.as_deref().unwrap_or("");
                let section_types: Vec<&str> =
                    p.sections.iter().map(|s| s.section_type.as_str()).collect();
                let desc = if !title.is_empty() && !section_types.is_empty() {
                    format!("{} — {}", title, section_types.join(", "))
                } else if !title.is_empty() {
                    title.to_string()
                } else if !section_types.is_empty() {
                    section_types.join(", ")
                } else {
                    String::new()
                };
                let auth_req = p
                    .requires
                    .as_deref()
                    .or_else(|| p.config.get("requires").map(|s| s.as_str()));
                let auth_str = auth_req
                    .map(|r| format!(" [requires: {}]", r))
                    .unwrap_or_default();
                println!("- {} ({}) — {}{}", p.route, p.page_type, desc, auth_str);
            }
            println!();
        }

        // APIs
        if !apis.is_empty() {
            println!("## APIs");
            for api in &apis {
                let methods: Vec<String> = api
                    .routes
                    .iter()
                    .map(|r| format!("{:?} {}", r.method, r.name))
                    .collect();
                println!("- {}: {}", api.prefix, methods.join(", "));
            }
            println!();
        }

        // Webhooks
        if !webhooks.is_empty() {
            println!("## Webhooks");
            for wh in &webhooks {
                for hook in &wh.hooks {
                    println!(
                        "- {} {} -> {} {}",
                        wh.entity, hook.event, hook.method, hook.url
                    );
                }
            }
            println!();
        }

        // Rules from constitution
        if !rules.is_empty() {
            println!("## Rules");
            for r in &rules {
                println!("- {}", r);
            }
            println!();
        }
    } else {
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
