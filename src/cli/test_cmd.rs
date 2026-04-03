use std::fs;
use crate::parser::{AstNode, EntityNode};
use crate::{parser, testing, find_cronus_file};

pub fn cmd_test(args: &[String]) {
    if args.iter().any(|a| a == "--conformance") {
        println!("  \x1b[36m⚡\x1b[0m Running conformance suite...\n");
        let base = args.iter()
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

    let file = find_cronus_file().unwrap_or_else(|| {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus file found"); std::process::exit(1);
    });
    let source = fs::read_to_string(&file).unwrap();
    let nodes = parser::parse(&source).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e); std::process::exit(1);
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
    if failed > 0 {
        std::process::exit(1);
    }
}
