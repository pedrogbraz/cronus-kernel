use std::fs;
use crate::parser::{self, AstNode};
use crate::find_cronus_file;

pub fn cmd_stats(_args: &[String]) {
    let file = find_cronus_file().unwrap_or_else(|| {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus file found"); std::process::exit(1);
    });
    let source = fs::read_to_string(&file).unwrap();
    let nodes = parser::parse(&source).unwrap();
    let (entities, pages, routes) = parser::stats(&nodes);
    let lines = source.lines().count();
    println!("  \x1b[36m⚡ CRONUS\x1b[0m Stats\n");
    println!("  \x1b[1mSource:\x1b[0m      {} ({} lines)", file, lines);
    println!("  \x1b[1mEntities:\x1b[0m    {}", entities);
    println!("  \x1b[1mPages:\x1b[0m       {}", pages);
    println!("  \x1b[1mAPI Routes:\x1b[0m  {}", routes);
    for node in &nodes {
        if let AstNode::App(a) = node { println!("  \x1b[1mApp:\x1b[0m         {} (port {})", a.name, a.port); }
    }
    if std::path::Path::new("data.db").exists() {
        let size = fs::metadata("data.db").map(|m| m.len()).unwrap_or(0);
        println!("  \x1b[1mDatabase:\x1b[0m    {}KB", size / 1024);
    }
    println!();
}
