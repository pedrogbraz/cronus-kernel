use crate::deploy;
use crate::export;
use crate::find_cronus_file;
use crate::parser;
use std::fs;

pub fn cmd_export(args: &[String]) {
    let file = find_cronus_file().unwrap_or_else(|| {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus file found");
        std::process::exit(1);
    });
    let source = fs::read_to_string(&file).unwrap();
    let nodes = parser::parse(&source).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e);
        std::process::exit(1);
    });

    // Parse --format flag (default: json)
    let format = args
        .iter()
        .position(|a| a == "--format")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("json");

    // Parse --output flag (optional: write to file instead of stdout)
    let output_file = args
        .iter()
        .position(|a| a == "--output" || a == "-o")
        .and_then(|i| args.get(i + 1))
        .cloned();

    let result = match format {
        "json" => export::export_json(&nodes),
        "openapi" => export::export_openapi(&nodes),
        "sql" => export::export_sql(&nodes),
        "typescript" | "ts" => export::export_typescript(&nodes),
        "ir" => {
            // Legacy: export IR format
            let ir = deploy::generate_ir(&nodes);
            serde_json::to_string_pretty(&ir).unwrap()
        }
        other => {
            eprintln!("  \x1b[31m✗\x1b[0m Unknown format: {}", other);
            eprintln!("    Supported: json, openapi, sql, typescript (ts), ir");
            std::process::exit(1);
        }
    };

    if let Some(ref path) = output_file {
        fs::write(path, &result).unwrap();
        let (e, p, r) = parser::stats(&nodes);
        eprintln!(
            "  \x1b[32m✓\x1b[0m Exported {} to {} ({} entities, {} pages, {} routes)",
            format, path, e, p, r
        );
    } else {
        println!("{}", result);
    }
}
