use std::fs;

use crate::cli::brief::brief_toml_val;
use crate::cli::objective_kernel::format_unix_date;

pub fn cmd_timeline() {
    let mut tasks: Vec<(String, String, String, String)> = Vec::new();

    if let Ok(entries) = fs::read_dir(".cronus/tasks") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("TASK-") && name.ends_with(".toml") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let id = brief_toml_val(&content, "id")
                        .unwrap_or_else(|| name.trim_end_matches(".toml").to_string());
                    let title = brief_toml_val(&content, "title").unwrap_or_else(|| "(no title)".into());
                    let status = brief_toml_val(&content, "status").unwrap_or_else(|| "open".into());
                    let created = brief_toml_val(&content, "created")
                        .map(|d| {
                            if d.len() >= 10 { d[..10].to_string() } else { d }
                        })
                        .unwrap_or_else(|| {
                            entry.metadata().ok()
                                .and_then(|m| m.modified().ok())
                                .map(|t| {
                                    let secs = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                                    format_unix_date(secs)
                                })
                                .unwrap_or_else(|| "unknown".into())
                        });
                    tasks.push((created, id, status, title));
                }
            }
        }
    }

    if tasks.is_empty() {
        println!();
        println!("  \x1b[1mCRONUS Timeline\x1b[0m");
        println!("  \x1b[90m═══════════════\x1b[0m");
        println!();
        println!("  \x1b[90m(no tasks found in .cronus/tasks/)\x1b[0m");
        println!();
        return;
    }

    tasks.sort_by(|a, b| {
        b.0.cmp(&a.0).then(b.1.cmp(&a.1))
    });

    println!();
    println!("  \x1b[1mCRONUS Timeline\x1b[0m");
    println!("  \x1b[90m═══════════════\x1b[0m");
    println!();

    for (date, id, status, title) in &tasks {
        let status_colored = match status.as_str() {
            "done" => format!("\x1b[32m[{}]\x1b[0m", status),
            "in_progress" | "open" => format!("\x1b[33m[{}]\x1b[0m", status),
            "blocked" => format!("\x1b[31m[{}]\x1b[0m", status),
            _ => format!("[{}]", status),
        };
        let status_plain = format!("[{}]", status);
        let pad = if status_plain.len() < 15 { " ".repeat(15 - status_plain.len()) } else { String::new() };
        println!("  {}  {}  {}{} {}", date, id, status_colored, pad, title);
    }
    println!();
}
