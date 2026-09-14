use crate::find_cronus_file;
use crate::graph;
use crate::parser;
use std::fs;

pub fn cmd_graph(args: &[String]) {
    let file = args
        .iter()
        .skip(2)
        .find(|a| !a.starts_with("--"))
        .cloned()
        .or_else(find_cronus_file)
        .unwrap_or_else(|| {
            eprintln!("  [31m✗[0m No .cronus file found");
            std::process::exit(1);
        });

    let source = match fs::read_to_string(&file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("  [31m✗[0m Cannot read {}: {}", file, e);
            std::process::exit(1);
        }
    };

    let nodes = match parser::parse(&source) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("  [31m✗[0m Parse error: {}", e);
            std::process::exit(1);
        }
    };

    let relationship_graph = graph::build_graph(&nodes);

    if args.iter().any(|a| a == "--json") {
        println!(
            "{}",
            serde_json::to_string_pretty(&relationship_graph).unwrap_or_default()
        );
    } else {
        // Default: Mermaid diagram
        println!("{}", graph::to_mermaid(&relationship_graph));

        // Print summary
        let rels = relationship_graph
            .entity_relations
            .iter()
            .filter(|r| r.relation_type == "belongs_to")
            .count();
        let binds = relationship_graph.page_bindings.len();
        let hooks = relationship_graph.webhook_flows.len();
        eprintln!(
            "
  {} entity relation(s), {} page binding(s), {} webhook flow(s)",
            rels, binds, hooks
        );
    }
}
