use std::fs;
use std::io::Write;
use crate::parser;
use crate::{
    TEMPLATE_LANDING, TEMPLATE_ADMIN, TEMPLATE_SAAS, TEMPLATE_API,
    TEMPLATE_ECOMMERCE, TEMPLATE_BLOG, TEMPLATE_HELPDESK, TEMPLATE_CRM,
};

pub fn cmd_new(args: &[String]) {
    let all_templates = ["landing", "admin", "saas", "api", "ecommerce", "blog", "helpdesk", "crm"];

    let template = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| {
        eprintln!("  Usage: cronus new <template>");
        eprintln!("  Templates: {}", all_templates.join(", "));
        std::process::exit(1);
    });

    // Try loading from templates/ directory next to the binary first
    let template_from_file = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .map(|dir| dir.join("templates").join(format!("{}.cronus", template)))
        .and_then(|path| fs::read_to_string(&path).ok());

    let content: &str = if let Some(ref file_content) = template_from_file {
        file_content.as_str()
    } else {
        // Fall back to embedded templates
        match template {
            "landing" => TEMPLATE_LANDING,
            "admin" => TEMPLATE_ADMIN,
            "saas" => TEMPLATE_SAAS,
            "api" => TEMPLATE_API,
            "ecommerce" => TEMPLATE_ECOMMERCE,
            "blog" => TEMPLATE_BLOG,
            "helpdesk" => TEMPLATE_HELPDESK,
            "crm" => TEMPLATE_CRM,
            _ => {
                eprintln!("  \x1b[33m✗\x1b[0m Unknown template: {}", template);
                eprintln!("  Available: {}", all_templates.join(", "));
                std::process::exit(1);
            }
        }
    };

    let dir = template;
    fs::create_dir_all(dir).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Cannot create directory: {}", e);
        std::process::exit(1);
    });

    let file_path = format!("{}/app.cronus", dir);
    let mut file = fs::File::create(&file_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    // Parse the template to show stats
    let new_nodes = parser::parse(content).ok();
    let (ent_count, pg_count, rt_count) = new_nodes.as_ref()
        .map(|n| parser::stats(n))
        .unwrap_or((0, 0, 0));

    let entity_names: Vec<String> = new_nodes.as_ref()
        .map(|nodes| nodes.iter().filter_map(|n| {
            if let parser::AstNode::Entity(e) = n { Some(e.name.clone()) } else { None }
        }).collect())
        .unwrap_or_default();

    let has_auth = new_nodes.as_ref()
        .map(|nodes| nodes.iter().any(|n| matches!(n, parser::AstNode::Auth(_))))
        .unwrap_or(false);

    println!("\n  Created: \x1b[1m{}/app.cronus\x1b[0m", dir);
    println!();
    println!("  \x1b[90mContents:\x1b[0m");
    if !entity_names.is_empty() {
        println!("    {} entities ({})", ent_count, entity_names.join(", "));
    }
    if pg_count > 0 {
        println!("    {} pages", pg_count);
    }
    if rt_count > 0 {
        println!("    {} API routes", rt_count);
    }
    if has_auth {
        println!("    Auth with JWT");
    }
    println!();
    println!("  \x1b[90mNext steps:\x1b[0m");
    println!("    cd {}", dir);
    println!("    cronus seed    \x1b[90m# populate with test data\x1b[0m");
    println!("    cronus run     \x1b[90m# start the server\x1b[0m");
    println!();
}
