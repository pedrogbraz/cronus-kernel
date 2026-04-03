use std::fs;
use serde_json::json;

use crate::cli::objective_kernel::{count_files_matching, chrono_now_iso};

pub fn cmd_sync() {
    use std::path::Path;
    use std::process::Command;

    println!("  Scanning project...");

    // 1. Check constitution.toml
    let has_constitution = Path::new(".cronus/constitution.toml").exists();
    if !has_constitution {
        println!("  \x1b[33m⚠ .cronus/constitution.toml not found\x1b[0m");
    }

    // 2. Read objective.toml
    let mut obj_title = String::from("(no objective set)");
    let mut obj_criteria: Vec<String> = Vec::new();
    if let Ok(content) = fs::read_to_string(".cronus/objective.toml") {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("title") {
                if let Some(val) = trimmed.split('=').nth(1) {
                    obj_title = val.trim().trim_matches('"').to_string();
                }
            }
            // Collect criteria lines (inside criteria = [...])
            if trimmed.starts_with('"') && trimmed.ends_with('"') || trimmed.starts_with('"') && trimmed.ends_with("\",") {
                let clean = trimmed.trim_matches(|c| c == '"' || c == ',' || c == ' ');
                if !clean.is_empty() {
                    obj_criteria.push(clean.to_string());
                }
            }
        }
    } else {
        println!("  \x1b[33m⚠ .cronus/objective.toml not found\x1b[0m");
    }

    // 3. Read tasks
    let mut open_tasks: Vec<String> = Vec::new();
    let mut done_tasks: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(".cronus/tasks") {
        let mut task_files: Vec<_> = entries
            .flatten()
            .filter(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                n.starts_with("TASK-") && n.ends_with(".toml")
            })
            .collect();
        task_files.sort_by_key(|e| e.file_name());

        for entry in &task_files {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                let mut id = String::new();
                let mut title = String::new();
                let mut status = String::from("open");
                for line in content.lines() {
                    let t = line.trim();
                    if t.starts_with("id") && t.contains('=') {
                        if let Some(v) = t.split('=').nth(1) {
                            id = v.trim().trim_matches('"').to_string();
                        }
                    }
                    if t.starts_with("title") && t.contains('=') {
                        if let Some(v) = t.split('=').nth(1) {
                            title = v.trim().trim_matches('"').to_string();
                        }
                    }
                    if t.starts_with("status") && t.contains('=') {
                        if let Some(v) = t.split('=').nth(1) {
                            status = v.trim().trim_matches('"').to_string();
                        }
                    }
                }
                let label = format!("{}: {}", id, title);
                if status == "done" {
                    done_tasks.push(label);
                } else {
                    open_tasks.push(label);
                }
            }
        }
    }

    // 4. Check build status
    let build_status = if Path::new("target/release/cronus-kernel").exists()
        || Path::new("target/release/cronus").exists()
        || Path::new("cronus-kernel/target/release/cronus").exists()
    {
        "passing"
    } else {
        "unknown (no release binary found)"
    };

    // 5. Count specs
    let spec_count = count_files_matching("specs", "spec.toml");

    // 6. Count conformance tests
    let test_count = count_files_matching("tests/conformance", ".cronus");

    // 7. Count examples
    let example_count = count_files_matching("examples", ".cronus");

    // 8. Git log
    let mut recent_commits: Vec<String> = Vec::new();
    if let Ok(output) = Command::new("git")
        .args(["log", "--oneline", "-20"])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    recent_commits.push(trimmed.to_string());
                }
            }
        }
    }

    // 9. Generate state-digest.json
    let now = chrono_now_iso();

    let digest = json!({
        "generated_at": now,
        "generated_by": "cronus sync",
        "objective": obj_title,
        "success_criteria": obj_criteria,
        "completed_tasks": done_tasks,
        "open_tasks": open_tasks,
        "metrics": {
            "build": build_status,
            "specs": spec_count,
            "conformance_tests": test_count,
            "examples": example_count
        },
        "recent_commits": recent_commits
    });

    // Ensure .cronus/ dir exists
    let _ = fs::create_dir_all(".cronus");
    let path = ".cronus/state-digest.json";
    match fs::write(path, serde_json::to_string_pretty(&digest).unwrap_or_default()) {
        Ok(_) => {
            println!("  Build: {}", build_status);
            println!("  Specs: {} | Tests: {} | Examples: {}", spec_count, test_count, example_count);
            println!("  Open tasks: {} | Completed: {}", open_tasks.len(), done_tasks.len());
            println!("  State digest written to \x1b[32m{}\x1b[0m", path);
        }
        Err(e) => {
            eprintln!("  \x1b[31mError writing {}: {}\x1b[0m", path, e);
        }
    }
}
