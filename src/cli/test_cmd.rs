use crate::parser::{AstNode, EntityNode};
use crate::{parser, testing};

pub fn cmd_test(args: &[String]) {
    if args.iter().any(|a| a == "--conformance") {
        println!("  \x1b[36m⚡\x1b[0m Running conformance suite...\n");
        let base = args
            .iter()
            .position(|a| a == "--dir")
            .and_then(|i| args.get(i + 1))
            .map(|s| s.as_str())
            .unwrap_or("tests/conformance");

        let (passed, failed, errors) = testing::run_conformance(base);

        for err in &errors {
            println!("  \x1b[31m✗\x1b[0m {}", err);
        }

        println!();
        if failed == 0 {
            println!("  \x1b[32m✓\x1b[0m All {} tests passed", passed);
        } else {
            println!("  \x1b[31m✗\x1b[0m {} passed, {} failed", passed, failed);
            std::process::exit(1);
        }
        return;
    }

    let nodes = parser::load_cwd().unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m {}", e);
        std::process::exit(1);
    });

    let mut entities: Vec<EntityNode> = vec![];
    let mut port: u16 = 5175;
    for node in &nodes {
        match node {
            AstNode::Entity(e) => entities.push(e.clone()),
            AstNode::App(a) => port = a.port,
            _ => {}
        }
    }

    // Override port from CLI
    if let Some(p) = args.get(2).and_then(|s| s.parse().ok()) {
        port = p;
    }

    let (passed, failed, _total) = testing::run_tests(&entities, port);

    // Component tests — report co-located test blocks
    let mut comp_tests = 0u32;
    let mut comp_test_steps = 0u32;
    for node in &nodes {
        if let AstNode::Component(c) = node {
            for test in &c.tests {
                comp_tests += 1;
                comp_test_steps += test.steps.len() as u32;
                println!(
                    "  \x1b[36m◉\x1b[0m {} → \"{}\" ({} steps)",
                    c.name,
                    test.name,
                    test.steps.len()
                );
                for step in &test.steps {
                    println!("    \x1b[90m{}\x1b[0m", step);
                }
            }
        }
    }
    if comp_tests > 0 {
        println!("\n  \x1b[33mℹ\x1b[0m {} component test(s) with {} step(s) (runtime execution coming soon)", comp_tests, comp_test_steps);
    }

    if failed > 0 {
        std::process::exit(1);
    }
}
