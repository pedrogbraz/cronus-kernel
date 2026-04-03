use std::fs;

use crate::cli::brief::{brief_toml_val, brief_toml_arr};
use crate::cli::objective_kernel::count_files_matching;

pub fn cmd_review(args: &[String]) {
    use std::process::Command;

    // 1. Determine which task to review
    let task_id_arg = args.get(2).cloned();
    let (task_id, task_content) = if let Some(ref tid) = task_id_arg {
        let normalized = if tid.starts_with("TASK-") {
            tid.clone()
        } else {
            format!("TASK-{:03}", tid.parse::<u32>().unwrap_or(0))
        };
        let path = format!(".cronus/tasks/{}.toml", normalized);
        match fs::read_to_string(&path) {
            Ok(c) => (normalized, c),
            Err(_) => {
                eprintln!("  \x1b[31mTask file not found: {}\x1b[0m", path);
                std::process::exit(1);
            }
        }
    } else {
        review_find_latest_task()
    };

    // 2. Parse task fields
    let mission = brief_toml_val(&task_content, "title").unwrap_or_else(|| "Unknown".into());
    let status = brief_toml_val(&task_content, "status").unwrap_or_else(|| "unknown".into());
    let done_checks = brief_toml_arr(&task_content, "done");

    // 3. Find commits mentioning this task ID
    let git_log = Command::new("git")
        .args(["log", "--oneline", "-50"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let task_id_lower = task_id.to_lowercase();
    let related_commits: Vec<&str> = git_log.lines()
        .filter(|l| l.to_lowercase().contains(&task_id_lower))
        .collect();

    let commit_hashes: Vec<String> = if related_commits.is_empty() {
        git_log.lines().take(5)
            .filter_map(|l| l.split_whitespace().next())
            .map(|s| s.to_string())
            .collect()
    } else {
        related_commits.iter()
            .filter_map(|l| l.split_whitespace().next())
            .map(|s| s.to_string())
            .collect()
    };

    // 4. Get diff stats for those commits
    let diff_range = if commit_hashes.len() >= 2 {
        format!("{}..{}", commit_hashes.last().unwrap(), commit_hashes.first().unwrap())
    } else if commit_hashes.len() == 1 {
        format!("{}~1..{}", commit_hashes[0], commit_hashes[0])
    } else {
        "HEAD~1..HEAD".to_string()
    };

    let diff_stat = Command::new("git")
        .args(["diff", "--stat", &diff_range])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let mut changes: Vec<String> = Vec::new();
    let mut total_insertions: i64 = 0;
    let mut total_deletions: i64 = 0;
    let mut files_changed = 0;

    for line in diff_stat.lines() {
        let trimmed = line.trim();
        if trimmed.contains('|') {
            files_changed += 1;
            let parts: Vec<&str> = trimmed.splitn(2, '|').collect();
            if parts.len() == 2 {
                let file = parts[0].trim();
                let stat = parts[1].trim();
                let plus_count = stat.matches('+').count() as i64;
                let minus_count = stat.matches('-').count() as i64;
                total_insertions += plus_count;
                total_deletions += minus_count;

                let action = if minus_count == 0 && plus_count > 0 {
                    "+"
                } else if plus_count == 0 && minus_count > 0 {
                    "-"
                } else {
                    "~"
                };

                let desc = if file.ends_with(".toml") && file.contains("TASK-") {
                    format!("{} Created task file {}", action, file)
                } else if file.ends_with(".toml") && (file.contains("constitution") || file.contains("objective")) {
                    format!("{} Updated project governance ({})", action, file)
                } else if file.ends_with(".rs") {
                    let num_str = stat.split_whitespace().next().unwrap_or("?");
                    format!("{} Modified {} ({} lines)", action, file, num_str)
                } else if file.ends_with(".cronus") {
                    format!("{} Updated CRONUS source ({})", action, file)
                } else if file.ends_with(".spec.toml") {
                    format!("{} Updated spec ({})", action, file)
                } else if file.ends_with(".json") {
                    format!("{} Updated data ({})", action, file)
                } else {
                    format!("{} Changed {}", action, file)
                };
                changes.push(desc);
            }
        } else if trimmed.contains("file") && trimmed.contains("changed") {
            for word in trimmed.split(',') {
                let word = word.trim();
                if word.contains("insertion") {
                    if let Some(n) = word.split_whitespace().next() {
                        total_insertions = n.parse().unwrap_or(total_insertions);
                    }
                }
                if word.contains("deletion") {
                    if let Some(n) = word.split_whitespace().next() {
                        total_deletions = n.parse().unwrap_or(total_deletions);
                    }
                }
            }
        }
    }

    // 5. Detect author from git log
    let author = Command::new("git")
        .args(["log", "-1", "--format=%an"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "Unknown".into());

    // 6. Validation checks
    let constitution = fs::read_to_string(".cronus/constitution.toml").unwrap_or_default();
    let constitution_exists = !constitution.is_empty();

    let objective = fs::read_to_string(".cronus/objective.toml").unwrap_or_default();
    let objective_exists = !objective.is_empty();

    let objective_pass = if objective_exists { "PASS" } else { "N/A" };

    let spec_count = count_files_matching("specs", ".spec.toml");
    let test_count = count_files_matching("tests/conformance", ".cronus");

    let build_ok = std::path::Path::new("target/release/cronus-kernel").exists()
        || std::path::Path::new("target/release/cronus").exists()
        || std::path::Path::new("target/debug/cronus-kernel").exists()
        || std::path::Path::new("target/debug/cronus").exists();

    // 7. Print formatted review
    println!();
    println!("  \x1b[1mSemantic Review: {}\x1b[0m", task_id);
    println!("  \x1b[90m{}\x1b[0m", "─".repeat(40));
    println!("  Mission: {}", mission);
    println!("  Status: {}", match status.as_str() {
        "done" => format!("\x1b[32m{}\x1b[0m", status),
        "in_progress" | "open" => format!("\x1b[33m{}\x1b[0m", status),
        "blocked" => format!("\x1b[31m{}\x1b[0m", status),
        _ => status.clone(),
    });
    println!("  Author: {}", author);
    println!("  Objective alignment: \x1b[32m{}\x1b[0m", objective_pass);
    println!();

    if !changes.is_empty() {
        println!("  \x1b[1mChanges:\x1b[0m");
        for change in &changes {
            println!("    {}", change);
        }
        println!("  \x1b[90m({} files, +{} -{})\x1b[0m", files_changed, total_insertions, total_deletions);
    } else {
        println!("  \x1b[1mChanges:\x1b[0m \x1b[90m(no diff data available)\x1b[0m");
    }
    println!();

    println!("  \x1b[1mValidation:\x1b[0m");
    if constitution_exists {
        println!("    Constitution: \x1b[32m✓ PASS\x1b[0m");
    } else {
        println!("    Constitution: \x1b[33m⚠ N/A\x1b[0m (no constitution.toml)");
    }
    if objective_exists {
        println!("    Objective: \x1b[32m✓ PASS\x1b[0m");
    } else {
        println!("    Objective: \x1b[33m⚠ N/A\x1b[0m (no objective.toml)");
    }
    if spec_count > 0 || test_count > 0 {
        println!("    Spec coverage: {} specs, {} tests", spec_count, test_count);
    } else {
        println!("    Spec coverage: \x1b[90mno specs found\x1b[0m");
    }
    if build_ok {
        println!("    Build: \x1b[32m✓ PASS\x1b[0m");
    } else {
        println!("    Build: \x1b[33m⚠ unknown\x1b[0m");
    }
    println!();

    if !done_checks.is_empty() {
        println!("  \x1b[1mDone criteria:\x1b[0m");
        for check in &done_checks {
            if status == "done" {
                println!("    \x1b[32m✓\x1b[0m {}", check);
            } else {
                println!("    \x1b[90m○\x1b[0m {}", check);
            }
        }
        println!();
    }
}

fn review_find_latest_task() -> (String, String) {
    let mut tasks: Vec<(String, String, String)> = Vec::new();

    if let Ok(entries) = fs::read_dir(".cronus/tasks") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("TASK-") && name.ends_with(".toml") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let id = name.trim_end_matches(".toml").to_string();
                    let status = brief_toml_val(&content, "status").unwrap_or_else(|| "open".into());
                    tasks.push((id, status, content));
                }
            }
        }
    }

    if tasks.is_empty() {
        eprintln!("  \x1b[31mNo tasks found in .cronus/tasks/\x1b[0m");
        std::process::exit(1);
    }

    tasks.sort_by(|a, b| b.0.cmp(&a.0));

    for (id, status, content) in &tasks {
        if status == "done" {
            return (id.clone(), content.clone());
        }
    }

    let (id, _, content) = tasks.into_iter().next().unwrap();
    (id, content)
}
