use crate::parser::{self, AstNode};
use std::fs;
use std::path::Path;

pub fn cmd_parse(args: &[String]) {
    let file = args.iter().skip(2).find(|a| !a.starts_with("--")).cloned();

    let nodes = if let Some(file) = file {
        let source = fs::read_to_string(&file).unwrap_or_else(|e| {
            eprintln!("  Cannot read {file}: {e}");
            std::process::exit(1);
        });
        parser::parse_source_at(&source, Path::new(&file)).map_err(|e| parser::diagnostic::join(&e))
    } else {
        parser::load_cwd()
    };
    match nodes {
        Ok(nodes) => {
            let (entities, pages, routes) = parser::stats(&nodes);
            println!("Parsed {} nodes:", nodes.len());
            println!("  Entities: {}", entities);
            println!("  Pages:    {}", pages);
            println!("  API:      {} routes", routes);

            for node in &nodes {
                if let AstNode::App(app) = node {
                    println!("  App:      \"{}\" (port {})", app.name, app.port);
                    if let Some(db) = &app.database {
                        println!("  Database: {} {:?}", db.db_type, db.path);
                    }
                    if let Some(ref c) = app.constitution {
                        println!(
                            "  Constitution: {} must, {} never",
                            c.must.len(),
                            c.never.len()
                        );
                        for rule in &c.must {
                            println!("    must: \"{}\"", rule);
                        }
                        for rule in &c.never {
                            println!("    never: \"{}\"", rule);
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    }
}
