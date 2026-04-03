use std::fs;
use crate::parser::{self, AstNode};
use crate::find_cronus_file;

pub fn cmd_parse(args: &[String]) {
    let file = args.iter().skip(2)
        .find(|a| !a.starts_with("--"))
        .cloned()
        .or_else(find_cronus_file)
        .unwrap_or_else(|| {
            eprintln!("  No .cronus file found");
            std::process::exit(1);
        });

    let source = fs::read_to_string(&file).unwrap();
    match parser::parse(&source) {
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
                        println!("  Constitution: {} must, {} never", c.must.len(), c.never.len());
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
