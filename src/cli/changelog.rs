use std::fs;

use crate::ast_diff;
use crate::find_all_cronus_files;
use crate::open_memory_db;
use crate::parser;
use crate::parser::AstNode;

pub fn cmd_changelog() {
    // 1. Load saved snapshot
    let snapshot_path = ".cronus/ast-snapshot.json";
    let old_snapshot: ast_diff::AstSnapshot = match fs::read_to_string(snapshot_path) {
        Ok(contents) => match serde_json::from_str(&contents) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("  \x1b[31m✗\x1b[0m Failed to parse snapshot: {}", e);
                eprintln!("  Run \x1b[1mcronus build\x1b[0m first to create a snapshot.");
                std::process::exit(1);
            }
        },
        Err(_) => {
            eprintln!("  \x1b[31m✗\x1b[0m No snapshot found at {}", snapshot_path);
            eprintln!("  Run \x1b[1mcronus build\x1b[0m first to create a baseline snapshot.");
            std::process::exit(1);
        }
    };

    // 2. Parse current .cronus file(s)
    let files = find_all_cronus_files();
    if files.is_empty() {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus file found");
        std::process::exit(1);
    }

    let nodes = if files.len() == 1 {
        let source = fs::read_to_string(&files[0]).unwrap_or_else(|e| {
            eprintln!("  \x1b[31m✗\x1b[0m Error reading {}: {}", files[0], e);
            std::process::exit(1);
        });
        match parser::parse_with_imports(&source, ".") {
            Ok(n) => n,
            Err(e) => {
                eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        match parser::parse_directory(".") {
            Ok(n) => n,
            Err(e) => {
                eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e);
                std::process::exit(1);
            }
        }
    };

    let new_snapshot = ast_diff::snapshot_from_ast(&nodes);

    // 3. Diff
    let changes = ast_diff::diff_snapshots(&old_snapshot, &new_snapshot);

    // 4. Print + write to semantic memory
    if changes.is_empty() {
        println!("  \x1b[32m✓\x1b[0m No changes since last build");
    } else {
        println!();
        println!(
            "  \x1b[1mChangelog\x1b[0m ({} change{})",
            changes.len(),
            if changes.len() == 1 { "" } else { "s" }
        );
        println!();

        // Write changes to semantic memory
        let mem = open_memory_db().ok();
        for change in &changes {
            println!("{}", change.describe());
            if let Some(ref m) = mem {
                let change_type = match change {
                    ast_diff::AstChange::EntityAdded { .. } => "entity_added",
                    ast_diff::AstChange::EntityRemoved { .. } => "entity_removed",
                    ast_diff::AstChange::EntitySharedChanged { .. } => "entity_changed",
                    ast_diff::AstChange::FieldAdded { .. } => "field_added",
                    ast_diff::AstChange::FieldRemoved { .. } => "field_removed",
                    ast_diff::AstChange::FieldTypeChanged { .. } => "field_changed",
                    ast_diff::AstChange::PageAdded { .. } => "page_added",
                    ast_diff::AstChange::PageRemoved { .. } => "page_removed",
                    ast_diff::AstChange::ApiAdded { .. } => "api_added",
                    ast_diff::AstChange::ApiRemoved { .. } => "api_removed",
                    ast_diff::AstChange::ApiRouteAdded { .. } => "api_route_added",
                    ast_diff::AstChange::ApiRouteRemoved { .. } => "api_route_removed",
                    ast_diff::AstChange::WebhookAdded { .. } => "webhook_added",
                    ast_diff::AstChange::WebhookRemoved { .. } => "webhook_removed",
                    ast_diff::AstChange::StyleChanged { .. } => "style_changed",
                };
                let _ = m.add_changelog(None, change_type, &change.describe().trim().to_string());
            }
        }
        if mem.is_some() {
            println!(
                "  \x1b[32m✓\x1b[0m {} changes saved to semantic memory",
                changes.len()
            );
        }
        println!();
    }
}
