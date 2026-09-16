use std::collections::HashMap;
use std::fs;

use crate::cli::objective_kernel::{reconcile_emit, reconcile_named_map};
use crate::parser::{
    self, ApiNode, AstNode, ComponentNode, EntityNode, EventNode, MiddlewareNode, PageNode,
    ServiceNode, WorkerNode,
};

pub fn cmd_reconcile(args: &[String]) {
    let positional: Vec<&String> = args
        .iter()
        .skip(2)
        .filter(|a| !a.starts_with("--"))
        .collect();
    if positional.len() < 2 {
        eprintln!("  \x1b[31m✗\x1b[0m Usage: cronus reconcile <file-a> <file-b> [--output <file>]");
        std::process::exit(1);
    }
    let file_a = &positional[0];
    let file_b = &positional[1];

    let output_file = args
        .windows(2)
        .find(|w| w[0] == "--output")
        .map(|w| w[1].clone());

    let source_a = fs::read_to_string(file_a.as_str()).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Cannot read {}: {}", file_a, e);
        std::process::exit(1);
    });
    let source_b = fs::read_to_string(file_b.as_str()).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Cannot read {}: {}", file_b, e);
        std::process::exit(1);
    });

    let nodes_a = parser::parse(&source_a).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Parse error in {}: {}", file_a, e);
        std::process::exit(1);
    });
    let nodes_b = parser::parse(&source_b).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Parse error in {}: {}", file_b, e);
        std::process::exit(1);
    });

    let mut conflicts: Vec<String> = Vec::new();
    let mut merged: Vec<AstNode> = Vec::new();

    // --- Categorize nodes from A ---
    let mut app_a: Option<parser::AppNode> = None;
    let mut style_a: Option<parser::StyleNode> = None;
    let mut auth_a: Option<parser::AuthNode> = None;
    let mut entities_a: HashMap<String, EntityNode> = HashMap::new();
    let mut pages_a: HashMap<String, PageNode> = HashMap::new();
    let mut apis_a: HashMap<String, ApiNode> = HashMap::new();
    let mut services_a: HashMap<String, ServiceNode> = HashMap::new();
    let mut components_a: HashMap<String, ComponentNode> = HashMap::new();
    let mut events_a: HashMap<String, EventNode> = HashMap::new();
    let mut workers_a: HashMap<String, WorkerNode> = HashMap::new();
    let mut middlewares_a: HashMap<String, MiddlewareNode> = HashMap::new();
    let mut imports_a: Vec<parser::ImportNode> = Vec::new();
    let mut envs_a: Vec<parser::EnvNode> = Vec::new();
    let mut tests_a: Vec<parser::TestNode> = Vec::new();
    let mut composes_a: Vec<parser::ComposeNode> = Vec::new();
    let mut layouts_a: HashMap<String, parser::LayoutNode> = HashMap::new();

    for node in nodes_a {
        match node {
            AstNode::App(n) => app_a = Some(n),
            AstNode::Style(n) => style_a = Some(n),
            AstNode::Auth(n) => auth_a = Some(n),
            AstNode::Entity(n) => {
                entities_a.insert(n.name.clone(), n);
            }
            AstNode::Page(n) => {
                pages_a.insert(n.route.clone(), n);
            }
            AstNode::Api(n) => {
                apis_a.insert(n.prefix.clone(), n);
            }
            AstNode::Service(n) => {
                services_a.insert(n.name.clone(), n);
            }
            AstNode::Component(n) => {
                components_a.insert(n.name.clone(), n);
            }
            AstNode::Event(n) => {
                events_a.insert(n.name.clone(), n);
            }
            AstNode::Worker(n) => {
                workers_a.insert(n.name.clone(), n);
            }
            AstNode::Middleware(n) => {
                middlewares_a.insert(n.name.clone(), n);
            }
            AstNode::Import(n) => imports_a.push(n),
            AstNode::Env(n) => envs_a.push(n),
            AstNode::Test(n) => tests_a.push(n),
            AstNode::Compose(n) => composes_a.push(n),
            AstNode::Layout(n) => {
                layouts_a.insert(n.name.clone(), n);
            }
            AstNode::Define(_) | AstNode::Webhook(_) | AstNode::Deploy(_) => {}
        }
    }

    // --- Categorize nodes from B ---
    let mut app_b: Option<parser::AppNode> = None;
    let mut style_b: Option<parser::StyleNode> = None;
    let mut auth_b: Option<parser::AuthNode> = None;
    let mut entities_b: HashMap<String, EntityNode> = HashMap::new();
    let mut pages_b: HashMap<String, PageNode> = HashMap::new();
    let mut apis_b: HashMap<String, ApiNode> = HashMap::new();
    let mut services_b: HashMap<String, ServiceNode> = HashMap::new();
    let mut components_b: HashMap<String, ComponentNode> = HashMap::new();
    let mut events_b: HashMap<String, EventNode> = HashMap::new();
    let mut workers_b: HashMap<String, WorkerNode> = HashMap::new();
    let mut middlewares_b: HashMap<String, MiddlewareNode> = HashMap::new();
    let mut imports_b: Vec<parser::ImportNode> = Vec::new();
    let mut envs_b: Vec<parser::EnvNode> = Vec::new();
    let mut tests_b: Vec<parser::TestNode> = Vec::new();
    let mut composes_b: Vec<parser::ComposeNode> = Vec::new();
    let mut layouts_b: HashMap<String, parser::LayoutNode> = HashMap::new();

    for node in nodes_b {
        match node {
            AstNode::App(n) => app_b = Some(n),
            AstNode::Style(n) => style_b = Some(n),
            AstNode::Auth(n) => auth_b = Some(n),
            AstNode::Entity(n) => {
                entities_b.insert(n.name.clone(), n);
            }
            AstNode::Page(n) => {
                pages_b.insert(n.route.clone(), n);
            }
            AstNode::Api(n) => {
                apis_b.insert(n.prefix.clone(), n);
            }
            AstNode::Service(n) => {
                services_b.insert(n.name.clone(), n);
            }
            AstNode::Component(n) => {
                components_b.insert(n.name.clone(), n);
            }
            AstNode::Event(n) => {
                events_b.insert(n.name.clone(), n);
            }
            AstNode::Worker(n) => {
                workers_b.insert(n.name.clone(), n);
            }
            AstNode::Middleware(n) => {
                middlewares_b.insert(n.name.clone(), n);
            }
            AstNode::Import(n) => imports_b.push(n),
            AstNode::Env(n) => envs_b.push(n),
            AstNode::Test(n) => tests_b.push(n),
            AstNode::Compose(n) => composes_b.push(n),
            AstNode::Layout(n) => {
                layouts_b.insert(n.name.clone(), n);
            }
            AstNode::Define(_) | AstNode::Webhook(_) | AstNode::Deploy(_) => {}
        }
    }

    // --- 1. App: take from A (primary) ---
    if let Some(app) = app_a {
        merged.push(AstNode::App(app));
    } else if let Some(app) = app_b {
        merged.push(AstNode::App(app));
    }

    // --- 2. Style: take from A (primary) ---
    if let Some(style) = style_a {
        merged.push(AstNode::Style(style));
    } else if let Some(style) = style_b {
        merged.push(AstNode::Style(style));
    }

    // --- 3. Auth: take from A unless B has it and A doesn't ---
    if let Some(auth) = auth_a {
        merged.push(AstNode::Auth(auth));
    } else if let Some(auth) = auth_b {
        merged.push(AstNode::Auth(auth));
    }

    // --- 4. Imports: union by alias ---
    let mut seen_imports: HashMap<String, bool> = HashMap::new();
    for imp in &imports_a {
        seen_imports.insert(imp.alias.clone(), true);
        merged.push(AstNode::Import(imp.clone()));
    }
    for imp in imports_b {
        if !seen_imports.contains_key(&imp.alias) {
            merged.push(AstNode::Import(imp));
        }
    }

    // --- 5. Entities: merge by name ---
    let mut all_entity_names: Vec<String> = entities_a.keys().cloned().collect();
    for name in entities_b.keys() {
        if !all_entity_names.contains(name) {
            all_entity_names.push(name.clone());
        }
    }
    all_entity_names.sort();

    for name in &all_entity_names {
        match (entities_a.remove(name), entities_b.remove(name)) {
            (Some(a), None) => merged.push(AstNode::Entity(a)),
            (None, Some(b)) => merged.push(AstNode::Entity(b)),
            (Some(a), Some(b)) => {
                let mut merged_fields: Vec<parser::FieldNode> = a.fields.clone();
                for field_b in &b.fields {
                    if let Some(field_a) = merged_fields.iter().find(|f| f.name == field_b.name) {
                        if field_a.field_type != field_b.field_type {
                            conflicts.push(format!(
                                "entity {}: field \"{}\" has type {:?} in A but {:?} in B",
                                name, field_b.name, field_a.field_type, field_b.field_type
                            ));
                        }
                    } else {
                        merged_fields.push(field_b.clone());
                    }
                }
                // Merge transitions from both entities
                let mut merged_transitions = a.transitions.clone();
                merged_transitions.extend(b.transitions.clone());
                // Merge effects from both entities
                let mut merged_effects = a.effects.clone();
                merged_effects.extend(b.effects.clone());
                merged.push(AstNode::Entity(EntityNode {
                    name: name.clone(),
                    fields: merged_fields,
                    reverses: Vec::new(),
                    transitions: merged_transitions,
                    effects: merged_effects,
                    shared: a.shared || b.shared,
                    remote_url: None,
                    doc: None,
                    span: Default::default(),
                }));
            }
            (None, None) => {}
        }
    }

    // --- 6. Pages: merge by route ---
    let mut all_routes: Vec<String> = pages_a.keys().cloned().collect();
    for route in pages_b.keys() {
        if !all_routes.contains(route) {
            all_routes.push(route.clone());
        }
    }
    all_routes.sort();

    for route in &all_routes {
        match (pages_a.remove(route), pages_b.remove(route)) {
            (Some(a), None) => merged.push(AstNode::Page(a)),
            (None, Some(b)) => merged.push(AstNode::Page(b)),
            (Some(a), Some(b)) => {
                let mut merged_sections: Vec<parser::SectionNode> = a.sections.clone();
                for (idx, sec_b) in b.sections.iter().enumerate() {
                    let existing = merged_sections
                        .iter()
                        .enumerate()
                        .find(|(_i, s)| s.section_type == sec_b.section_type);
                    if let Some((pos, _)) = existing {
                        if pos == idx {
                            conflicts.push(format!(
                                "page \"{}\": section type \"{}\" at position {} exists in both A and B",
                                route, sec_b.section_type, idx
                            ));
                        }
                    } else {
                        merged_sections.push(sec_b.clone());
                    }
                }
                merged.push(AstNode::Page(PageNode {
                    route: route.clone(),
                    page_type: a.page_type,
                    entity: a.entity,
                    title: a.title,
                    sections: merged_sections,
                    config: a.config,
                    components: a.components,
                    requires: a.requires,
                    doc: None,
                    span: Default::default(),
                }));
            }
            (None, None) => {}
        }
    }

    // --- 7. APIs: merge by prefix, union of routes ---
    let mut all_prefixes: Vec<String> = apis_a.keys().cloned().collect();
    for prefix in apis_b.keys() {
        if !all_prefixes.contains(prefix) {
            all_prefixes.push(prefix.clone());
        }
    }
    all_prefixes.sort();

    for prefix in &all_prefixes {
        match (apis_a.remove(prefix), apis_b.remove(prefix)) {
            (Some(a), None) => merged.push(AstNode::Api(a)),
            (None, Some(b)) => merged.push(AstNode::Api(b)),
            (Some(a), Some(b)) => {
                let mut merged_routes = a.routes.clone();
                for route_b in &b.routes {
                    let exists = merged_routes
                        .iter()
                        .any(|r| r.method == route_b.method && r.path == route_b.path);
                    if !exists {
                        merged_routes.push(route_b.clone());
                    }
                }
                merged.push(AstNode::Api(ApiNode {
                    prefix: prefix.clone(),
                    routes: merged_routes,
                    doc: None,
                    span: Default::default(),
                }));
            }
            (None, None) => {}
        }
    }

    // --- 8-13. Named blocks: merge by name, report conflicts ---
    reconcile_named_map(
        &mut merged,
        &mut conflicts,
        services_a,
        services_b,
        "service",
        |n| AstNode::Service(n),
    );
    reconcile_named_map(
        &mut merged,
        &mut conflicts,
        components_a,
        components_b,
        "component",
        |n| AstNode::Component(n),
    );
    reconcile_named_map(
        &mut merged,
        &mut conflicts,
        events_a,
        events_b,
        "event",
        |n| AstNode::Event(n),
    );
    reconcile_named_map(
        &mut merged,
        &mut conflicts,
        workers_a,
        workers_b,
        "worker",
        |n| AstNode::Worker(n),
    );
    reconcile_named_map(
        &mut merged,
        &mut conflicts,
        middlewares_a,
        middlewares_b,
        "middleware",
        |n| AstNode::Middleware(n),
    );
    reconcile_named_map(
        &mut merged,
        &mut conflicts,
        layouts_a,
        layouts_b,
        "layout",
        |n| AstNode::Layout(n),
    );

    // --- 14. Envs, Tests, Composes: all from A, unique from B ---
    for env in envs_a {
        merged.push(AstNode::Env(env));
    }
    for env in envs_b {
        if !merged
            .iter()
            .any(|n| matches!(n, AstNode::Env(e) if e.name == env.name))
        {
            merged.push(AstNode::Env(env));
        }
    }
    for test in tests_a {
        merged.push(AstNode::Test(test));
    }
    for test in tests_b {
        if !merged
            .iter()
            .any(|n| matches!(n, AstNode::Test(t) if t.name == test.name))
        {
            merged.push(AstNode::Test(test));
        }
    }
    for comp in composes_a {
        merged.push(AstNode::Compose(comp));
    }
    for comp in composes_b {
        if !merged
            .iter()
            .any(|n| matches!(n, AstNode::Compose(c) if c.name == comp.name))
        {
            merged.push(AstNode::Compose(comp));
        }
    }

    // --- Check for conflicts ---
    if !conflicts.is_empty() {
        eprintln!(
            "\n  \x1b[31m✗ RECONCILE FAILED — {} conflict(s):\x1b[0m\n",
            conflicts.len()
        );
        for (i, c) in conflicts.iter().enumerate() {
            eprintln!("  {}. {}", i + 1, c);
        }
        eprintln!();
        std::process::exit(1);
    }

    // --- Emit merged .cronus ---
    let output = reconcile_emit(&merged);

    if let Some(out_path) = output_file {
        fs::write(&out_path, &output).unwrap_or_else(|e| {
            eprintln!("  \x1b[31m✗\x1b[0m Cannot write {}: {}", out_path, e);
            std::process::exit(1);
        });
        println!(
            "  \x1b[32m✓\x1b[0m Reconciled {} + {} → {}",
            file_a, file_b, out_path
        );
        println!("    {} nodes merged", merged.len());
    } else {
        print!("{}", output);
    }
}
