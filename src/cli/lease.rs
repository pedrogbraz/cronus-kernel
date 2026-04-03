use std::fs;

use crate::cli::brief::{brief_toml_val, brief_toml_arr, brief_toml_arr_after_section, brief_today_date};
use crate::cli::objective_kernel::lease_file_allowed;

pub fn cmd_lease(args: &[String]) {
    let sub = args.get(2).map(|s| s.as_str()).unwrap_or("check");
    match sub {
        "check" => lease_check(),
        "list" => lease_list(),
        "create" => lease_create(args),
        _ => {
            eprintln!("  \x1b[31mUnknown lease subcommand: {}\x1b[0m", sub);
            eprintln!("  Usage: cronus lease <check|list|create>");
            std::process::exit(1);
        }
    }
}

/// cronus lease check — validate git diff against active task scope
fn lease_check() {
    use std::process::Command;

    // 1. Find active task (first open/in_progress)
    let mut task_id = String::new();
    let mut task_title = String::new();
    let mut write_scope: Vec<String> = Vec::new();

    if let Ok(entries) = fs::read_dir(".cronus/tasks") {
        let mut task_files: Vec<_> = entries
            .flatten()
            .filter(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                n.starts_with("TASK-") && n.ends_with(".toml")
            })
            .collect();
        task_files.sort_by_key(|e| e.file_name());

        for entry in task_files {
            let content = fs::read_to_string(entry.path()).unwrap_or_default();
            if let Some(status) = brief_toml_val(&content, "status") {
                if status == "open" || status == "in_progress" {
                    task_id = brief_toml_val(&content, "id").unwrap_or_default();
                    task_title = brief_toml_val(&content, "title").unwrap_or_default();
                    write_scope = brief_toml_arr_after_section(&content, "[scope]", "write");
                    break;
                }
            }
        }
    }

    if task_id.is_empty() {
        eprintln!("  \x1b[33mNo active task lease found.\x1b[0m");
        eprintln!("  Create one with: cronus lease create \"title\" --write file1,file2");
        std::process::exit(1);
    }

    // 2. Get staged files only (this runs as a pre-commit hook)
    let mut modified_files: Vec<String> = Vec::new();

    if let Ok(output) = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                let f = line.trim().to_string();
                if !f.is_empty() && !modified_files.contains(&f) {
                    modified_files.push(f);
                }
            }
        }
    }

    if modified_files.is_empty() {
        println!("  Active lease: \x1b[1m{}\x1b[0m — {}", task_id, task_title);
        println!("  No modified files detected.");
        return;
    }

    // 3. Check each file against scope
    println!("  Active lease: \x1b[1m{}\x1b[0m — {}", task_id, task_title);
    println!("  Modified files:");

    let mut blocked_files: Vec<String> = Vec::new();

    for file in &modified_files {
        if lease_file_allowed(file, &write_scope) {
            println!("    \x1b[32m{}\x1b[0m  \x1b[32m✓ ALLOWED\x1b[0m", file);
        } else {
            println!("    \x1b[31m{}\x1b[0m  \x1b[31m✗ BLOCKED (not in scope)\x1b[0m", file);
            blocked_files.push(file.clone());
        }
    }

    if blocked_files.is_empty() {
        println!("\n  \x1b[32m✓ All files within task scope.\x1b[0m");
    } else {
        println!(
            "\n  \x1b[33m⚠ DRIFT DETECTED: {} file{} outside task scope.\x1b[0m",
            blocked_files.len(),
            if blocked_files.len() == 1 { "" } else { "s" }
        );
        for f in &blocked_files {
            println!("  Run `cronus lease expand {}` to add it.", f);
        }
        std::process::exit(1);
    }
}

/// cronus lease list — show all tasks with status
fn lease_list() {
    if let Ok(entries) = fs::read_dir(".cronus/tasks") {
        let mut task_files: Vec<_> = entries
            .flatten()
            .filter(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                n.starts_with("TASK-") && n.ends_with(".toml")
            })
            .collect();
        task_files.sort_by_key(|e| e.file_name());

        if task_files.is_empty() {
            println!("  No task leases found in .cronus/tasks/");
            return;
        }

        println!("  \x1b[1mTask Leases:\x1b[0m\n");

        for entry in &task_files {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                let id = brief_toml_val(&content, "id").unwrap_or_else(|| "???".into());
                let title = brief_toml_val(&content, "title").unwrap_or_default();
                let status = brief_toml_val(&content, "status").unwrap_or_else(|| "open".into());
                let priority = brief_toml_val(&content, "priority").unwrap_or_default();

                let status_color = match status.as_str() {
                    "done" => "\x1b[32m",
                    "open" => "\x1b[33m",
                    "in_progress" => "\x1b[36m",
                    "blocked" => "\x1b[31m",
                    _ => "\x1b[90m",
                };

                let prio_str = if priority.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", priority)
                };

                println!(
                    "    {} {}{}\x1b[0m — {}{}",
                    id, status_color, status, title, prio_str
                );
            }
        }
        println!();
    } else {
        println!("  No .cronus/tasks/ directory found.");
    }
}

/// cronus lease create "title" --write file1,file2 --checks "check1,check2"
fn lease_create(args: &[String]) {
    let title = args.get(3).cloned().unwrap_or_else(|| {
        eprintln!("  \x1b[31mUsage: cronus lease create \"title\" --write file1,file2 [--checks \"c1,c2\"]\x1b[0m");
        std::process::exit(1);
    });

    let mut write_files: Vec<String> = Vec::new();
    let mut checks: Vec<String> = Vec::new();
    let mut read_files: Vec<String> = Vec::new();

    let mut i = 4;
    while i < args.len() {
        match args[i].as_str() {
            "--write" | "-w" => {
                if i + 1 < args.len() {
                    write_files = args[i + 1].split(',').map(|s| s.trim().to_string()).collect();
                    i += 2;
                } else { i += 1; }
            }
            "--checks" | "-c" => {
                if i + 1 < args.len() {
                    checks = args[i + 1].split(',').map(|s| s.trim().to_string()).collect();
                    i += 2;
                } else { i += 1; }
            }
            "--read" | "-r" => {
                if i + 1 < args.len() {
                    read_files = args[i + 1].split(',').map(|s| s.trim().to_string()).collect();
                    i += 2;
                } else { i += 1; }
            }
            _ => { i += 1; }
        }
    }

    // Find next task number
    let mut max_num: u32 = 0;
    if let Ok(entries) = fs::read_dir(".cronus/tasks") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("TASK-") && name.ends_with(".toml") {
                let num_str = &name[5..name.len() - 5];
                if let Ok(n) = num_str.parse::<u32>() {
                    if n > max_num { max_num = n; }
                }
            }
        }
    }
    let next_num = max_num + 1;
    let task_id = format!("TASK-{:03}", next_num);
    let today = brief_today_date();

    let mut toml = String::new();
    toml.push_str(&format!("[task]\nid = \"{}\"\n", task_id));
    toml.push_str(&format!("title = \"{}\"\n", title));
    toml.push_str("status = \"open\"\n");
    toml.push_str(&format!("created = \"{}\"\n", today));
    toml.push_str("priority = \"P1\"\n");

    toml.push_str("\n[mission]\n");
    toml.push_str(&format!("goal = \"{}\"\n", title));
    toml.push_str("context = \"\"\n");

    toml.push_str("\n[scope]\n");
    toml.push_str("write = [\n");
    for f in &write_files {
        toml.push_str(&format!("  \"{}\",\n", f));
    }
    toml.push_str("]\n");
    toml.push_str("read = [\n");
    for f in &read_files {
        toml.push_str(&format!("  \"{}\",\n", f));
    }
    toml.push_str("]\n");

    toml.push_str("\n[forbidden]\nitems = []\n");
    toml.push_str("\n[dependencies]\nrequires = []\n");

    toml.push_str("\n[done]\nchecks = [\n");
    for c in &checks {
        toml.push_str(&format!("  \"{}\",\n", c));
    }
    toml.push_str("]\n");

    let _ = fs::create_dir_all(".cronus/tasks");
    let path = format!(".cronus/tasks/{}.toml", task_id);
    match fs::write(&path, &toml) {
        Ok(_) => {
            println!("  \x1b[32m✓ Created {}\x1b[0m — {}", task_id, title);
            println!("  File: {}", path);
            if !write_files.is_empty() {
                println!("  Scope: {}", write_files.join(", "));
            }
        }
        Err(e) => {
            eprintln!("  \x1b[31mError creating {}: {}\x1b[0m", path, e);
            std::process::exit(1);
        }
    }
}
