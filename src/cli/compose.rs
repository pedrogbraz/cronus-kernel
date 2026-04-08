use std::fs;
use crate::parser::AstNode;
use crate::{parser, find_all_cronus_files};
use crate::hydra::compose::{self, ComposeOptions, EntityRemap, FieldDef};

pub fn cmd_compose(args: &[String]) {
    // Mode 1: cronus compose --from <template>
    if let Some(pos) = args.iter().position(|a| a == "--from") {
        if let Some(template_name) = args.get(pos + 1) {
            compose_from_template(template_name, args);
            return;
        } else {
            eprintln!("  \x1b[31m✗\x1b[0m --from requires a template name");
            eprintln!("  Available: saas-billing, blog, crm, helpdesk, ecommerce");
            std::process::exit(1);
        }
    }

    // Mode 2: cronus compose --entity Name field:type field:type
    if let Some(pos) = args.iter().position(|a| a == "--entity") {
        compose_from_entity(args, pos);
        return;
    }

    // Mode 3: cronus compose (existing — compose .cronus files in directory)
    compose_directory();
}

fn compose_from_template(template_name: &str, args: &[String]) {
    match compose::get_template(template_name) {
        Some((entities, mut opts)) => {
            // Override port if specified
            if let Some(pos) = args.iter().position(|a| a == "--port") {
                if let Some(port_str) = args.get(pos + 1) {
                    opts.port = port_str.parse().unwrap_or(5220);
                }
            }

            // Override app name
            let app_name = args.iter().position(|a| a == "--name")
                .and_then(|p| args.get(p + 1))
                .map(|s| s.as_str())
                .unwrap_or(template_name);

            let output = compose::compose_app(app_name, &entities, &opts);

            // Write to file
            let filename = format!("{}.cronus", template_name.replace(' ', "-"));
            fs::write(&filename, &output).unwrap_or_else(|e| {
                eprintln!("  \x1b[31m✗\x1b[0m Failed to write {}: {}", filename, e);
                std::process::exit(1);
            });

            let lines = output.lines().count();
            let entity_count = entities.len() + 1; // +1 for User
            let page_count = entities.len() * 2 + 1; // list + create per entity + dashboard
            let route_count = entities.len() * 5; // CRUD per entity

            println!();
            println!("  \x1b[32m✓\x1b[0m Composed \x1b[1m{}\x1b[0m from template \x1b[36m{}\x1b[0m", filename, template_name);
            println!();
            println!("    Entities:  {}", entity_count);
            println!("    Pages:     {}", page_count);
            println!("    Routes:    {} API endpoints", route_count);
            println!("    Auth:      JWT (admin, user)");
            println!("    Lines:     {}", lines);
            println!();
            println!("  \x1b[90mRun:\x1b[0m  cronus run . {}", opts.port);
            println!();
        }
        None => {
            eprintln!("  \x1b[31m✗\x1b[0m Unknown template: {}", template_name);
            eprintln!("  Available templates:");
            eprintln!("    saas-billing  — Customer, Plan, Subscription, Invoice, PaymentMethod, UsageRecord");
            eprintln!("    blog          — Post, Category, Tag, Comment");
            eprintln!("    crm           — Company, Contact, Deal, Activity");
            eprintln!("    helpdesk      — Agent, Customer, Ticket, Message");
            eprintln!("    ecommerce     — Category, Product, Customer, Order, OrderItem");
            std::process::exit(1);
        }
    }
}

fn compose_from_entity(args: &[String], entity_pos: usize) {
    let entity_name = match args.get(entity_pos + 1) {
        Some(name) => name.clone(),
        None => {
            eprintln!("  \x1b[31m✗\x1b[0m --entity requires a name");
            eprintln!("  Example: cronus compose --entity Product name:text price:money status:text");
            std::process::exit(1);
        }
    };

    // Parse fields from remaining args: name:type name:type!required
    let mut fields = Vec::new();
    for arg in &args[entity_pos + 2..] {
        if arg.starts_with("--") { break; }
        let (name, rest) = arg.split_once(':').unwrap_or((arg, "text"));
        let required = rest.ends_with('!');
        let field_type = rest.trim_end_matches('!');
        let mut fd = FieldDef::new(name, field_type);
        if required { fd = fd.req(); }
        fields.push(fd);
    }

    if fields.is_empty() {
        fields = vec![FieldDef::new("name", "text").req()];
    }

    let app_name = args.iter().position(|a| a == "--name")
        .and_then(|p| args.get(p + 1))
        .map(|s| s.as_str())
        .unwrap_or(&entity_name);

    let entities = vec![EntityRemap::new(&entity_name, fields)];
    let opts = ComposeOptions::default();
    let output = compose::compose_app(app_name, &entities, &opts);

    let filename = format!("{}.cronus", entity_name.to_lowercase());
    fs::write(&filename, &output).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Failed to write {}: {}", filename, e);
        std::process::exit(1);
    });

    println!();
    println!("  \x1b[32m✓\x1b[0m Composed \x1b[1m{}\x1b[0m with entity \x1b[36m{}\x1b[0m", filename, entity_name);
    println!("  \x1b[90mRun:\x1b[0m  cronus run . {}", opts.port);
    println!();
}

fn compose_directory() {
    let files = find_all_cronus_files();
    if files.is_empty() {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus files found");
        std::process::exit(1);
    }
    if files.len() == 1 {
        println!("  Only 1 file ({}). Compose requires 2+ files.", files[0]);
        return;
    }
    println!("  \x1b[36m⚡ CRONUS\x1b[0m Composing {} files:\n", files.len());
    let mut total_lines = 0;
    for f in &files {
        let lines = fs::read_to_string(f).map(|s| s.lines().count()).unwrap_or(0);
        total_lines += lines;
        println!("    + {} ({} lines)", f, lines);
    }
    match parser::parse_directory(".") {
        Ok(nodes) => {
            let (entities, pages, routes) = parser::stats(&nodes);
            println!("\n  \x1b[1mComposed:\x1b[0m");
            println!("    Entities: {}", entities);
            println!("    Pages:    {}", pages);
            println!("    Routes:   {}", routes);
            println!("    Total:    {} nodes from {} lines\n", nodes.len(), total_lines);
            for node in &nodes {
                if let AstNode::Entity(e) = node { println!("    entity {} ({} fields)", e.name, e.fields.len()); }
            }
            for node in &nodes {
                if let AstNode::Page(p) = node { println!("    page {} ({})", p.route, p.page_type); }
            }
            println!();
        }
        Err(e) => { eprintln!("  \x1b[31m✗\x1b[0m {}", e); std::process::exit(1); }
    }
}
