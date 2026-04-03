use std::fs;
use crate::{parser, find_cronus_file, find_all_cronus_files};
use crate::parser::AstNode;
use crate::audit;

pub fn cmd_verify(args: &[String]) {
    let file = find_cronus_file().unwrap_or_else(|| {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus file found"); std::process::exit(1);
    });
    let source = fs::read_to_string(&file).unwrap();
    let nodes = parser::parse(&source).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e); std::process::exit(1);
    });
    let (entities, pages, routes) = parser::stats(&nodes);
    println!("  \x1b[32m✓\x1b[0m Verified: {} entities, {} pages, {} routes", entities, pages, routes);
}

pub fn cmd_verify_audit(args: &[String]) {
    let files = find_all_cronus_files();
    if files.is_empty() {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus files found");
        return;
    }

    let source = std::fs::read_to_string(&files[0]).unwrap_or_default();
    let nodes = match parser::parse(&source) {
        Ok(n) => n,
        Err(_) => vec![],
    };
    let db_path = nodes.iter().find_map(|n| {
        if let AstNode::App(ref app) = n {
            app.database.as_ref().and_then(|d| d.path.clone())
        } else {
            None
        }
    }).unwrap_or_else(|| "data.db".into());

    println!();
    println!("  \x1b[1mCRONUS Audit Trail Verification\x1b[0m");
    println!("  Database: {}", db_path);
    println!();

    match audit::verify_from_file(&db_path) {
        Ok(result) => {
            let valid = result["valid"].as_bool().unwrap_or(false);
            let entries = result["entries"].as_i64().unwrap_or(0);
            if valid {
                println!("  \x1b[32m✓\x1b[0m Chain intact -- {} entries verified", entries);
            } else {
                let broken_at = result["broken_at"].as_i64().unwrap_or(0);
                let reason = result["reason"].as_str().unwrap_or("unknown");
                println!("  \x1b[31m✗\x1b[0m Chain BROKEN at entry {} ({}) -- {} total entries", broken_at, reason, entries);
            }
        }
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m Failed to verify: {}", e);
        }
    }
    println!();
}

pub fn cmd_debug_audit(args: &[String]) {
    let files = find_all_cronus_files();
    if files.is_empty() {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus files found");
        return;
    }

    let source = std::fs::read_to_string(&files[0]).unwrap_or_default();
    let nodes = match parser::parse(&source) {
        Ok(n) => n,
        Err(_) => vec![],
    };
    let db_path = nodes.iter().find_map(|n| {
        if let AstNode::App(ref app) = n {
            app.database.as_ref().and_then(|d| d.path.clone())
        } else {
            None
        }
    }).unwrap_or_else(|| "data.db".into());

    let entity_filter = args.iter().position(|a| a == "--entity")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str());

    let verify = args.iter().any(|a| a == "--verify");

    let limit: usize = args.iter().position(|a| a == "--limit")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);

    println!();
    println!("  \x1b[1mCRONUS Audit Trail\x1b[0m");
    println!("  Database: {}", db_path);
    if let Some(entity) = entity_filter {
        println!("  Filter: entity={}", entity);
    }
    println!();

    match audit::debug_from_file(&db_path, limit, entity_filter, verify) {
        Ok(output) => print!("{}", output),
        Err(e) => eprintln!("  \x1b[31m✗\x1b[0m Failed to read audit trail: {}", e),
    }

    if verify {
        match audit::verify_from_file(&db_path) {
            Ok(result) => {
                let valid = result["valid"].as_bool().unwrap_or(false);
                let entries = result["entries"].as_i64().unwrap_or(0);
                if valid {
                    println!("  \x1b[32m✓\x1b[0m Chain intact -- {} entries verified\n", entries);
                } else {
                    let broken_at = result["broken_at"].as_i64().unwrap_or(0);
                    let reason = result["reason"].as_str().unwrap_or("unknown");
                    println!("  \x1b[31m✗\x1b[0m Chain BROKEN at entry {} ({}) -- {} total\n", broken_at, reason, entries);
                }
            }
            Err(e) => eprintln!("  \x1b[31m✗\x1b[0m Verification failed: {}\n", e),
        }
    }
}
