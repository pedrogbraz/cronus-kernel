use std::fs;

pub fn cmd_brief() {
    let today = brief_today_date();

    // --- Read constitution.toml ---
    let constitution = fs::read_to_string(".cronus/constitution.toml").unwrap_or_default();
    let proj_name = brief_toml_val(&constitution, "name").unwrap_or_else(|| "Unknown".into());
    let proj_purpose = brief_toml_val(&constitution, "purpose").unwrap_or_default();
    let proj_category = brief_toml_val(&constitution, "category").unwrap_or_default();
    let invariants = brief_toml_arr(&constitution, "must");

    // --- Read objective.toml ---
    let objective = fs::read_to_string(".cronus/objective.toml").unwrap_or_default();
    let obj_title =
        brief_toml_val(&objective, "title").unwrap_or_else(|| "No objective set".into());
    let obj_deadline = brief_toml_val(&objective, "deadline").unwrap_or_else(|| "none".into());
    let obj_why = brief_toml_val(&objective, "why").unwrap_or_default();
    let out_of_scope = brief_toml_arr(&objective, "items");

    // --- Read state-digest.json ---
    let digest = fs::read_to_string(".cronus/state-digest.json").unwrap_or_default();
    let completed = brief_json_arr(&digest, "completed_tasks");
    let risks = brief_json_arr(&digest, "current_risks");
    let kernel_lines = brief_json_val(&digest, "kernel_lines").unwrap_or_default();
    let binary_size = brief_json_val(&digest, "binary_size").unwrap_or_default();
    let build_status = brief_json_val(&digest, "build").unwrap_or_default();

    // --- Find first open TASK-*.toml ---
    let mut task_id = String::new();
    let mut task_title = String::new();
    let mut task_context = String::new();
    let mut task_write: Vec<String> = Vec::new();
    let mut task_forbidden: Vec<String> = Vec::new();
    let mut task_done_checks: Vec<String> = Vec::new();

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
                    task_context = brief_toml_val(&content, "context").unwrap_or_default();
                    task_write = brief_toml_arr(&content, "write");
                    task_forbidden = brief_toml_arr_after_section(&content, "[forbidden]", "items");
                    task_done_checks = brief_toml_arr(&content, "checks");
                    break;
                }
            }
        }
    }

    // --- Print brief ---
    println!("# CRONUS Context Brief — {}\n", today);

    println!("## Project");
    println!("{} is a {}. {}.", proj_name, proj_category, proj_purpose);
    if !kernel_lines.is_empty() || !binary_size.is_empty() {
        let mut meta = Vec::new();
        if !kernel_lines.is_empty() {
            meta.push(format!("{} lines", kernel_lines));
        }
        if !binary_size.is_empty() {
            meta.push(format!("{} binary", binary_size));
        }
        if !build_status.is_empty() {
            meta.push(format!("build: {}", build_status));
        }
        println!("Rust kernel, {}.", meta.join(", "));
    }
    println!();

    println!("## Current Objective");
    println!("{}. Deadline: {}.", obj_title, obj_deadline);
    if !obj_why.is_empty() {
        println!("Why: {}", obj_why);
    }
    println!();

    if !task_id.is_empty() {
        println!("## Your Task");
        println!("{}: {}", task_id, task_title);
        if !task_write.is_empty() {
            let files: Vec<String> = task_write
                .iter()
                .map(|f| f.rsplit('/').next().unwrap_or(f).to_string())
                .collect();
            println!("Files: {}", files.join(", "));
        }
        if !task_forbidden.is_empty() {
            println!("Do NOT touch: {}", task_forbidden.join(", "));
        }
        println!();

        if !task_done_checks.is_empty() {
            println!("## Done When");
            for check in &task_done_checks {
                println!("- {}", check);
            }
            println!();
        }

        println!("## Current State");
        if !task_context.is_empty() {
            println!("{}", task_context);
        }
    }

    if !completed.is_empty() {
        let recent: Vec<&String> = completed.iter().rev().take(5).collect();
        println!("Recent completed:");
        for t in recent {
            println!("  - {}", t);
        }
    }
    println!();

    if !risks.is_empty() {
        println!("## Risks");
        for r in &risks {
            println!("- {}", r);
        }
        println!();
    }

    if !invariants.is_empty() {
        println!("## Rules");
        for inv in &invariants {
            println!("- {}", inv);
        }
        println!();
    }

    if !out_of_scope.is_empty() {
        println!("## Out of Scope");
        for item in &out_of_scope {
            println!("- {}", item);
        }
        println!();
    }
}

/// Get today's date as YYYY-MM-DD (no chrono dependency)
pub fn brief_today_date() -> String {
    use std::time::SystemTime;
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let diy = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
            366
        } else {
            365
        };
        if remaining < diy {
            break;
        }
        remaining -= diy;
        y += 1;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let md = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 0usize;
    for (i, &d) in md.iter().enumerate() {
        if remaining < d as i64 {
            m = i + 1;
            break;
        }
        remaining -= d as i64;
    }
    format!("{:04}-{:02}-{:02}", y, m, remaining + 1)
}

/// Extract `key = "value"` from TOML text
pub fn brief_toml_val(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        // Match: key = "value" or key= "value"
        let before_eq = match trimmed.find('=') {
            Some(pos) => trimmed[..pos].trim(),
            None => continue,
        };
        if before_eq == key {
            let after_eq = trimmed[trimmed.find('=').unwrap() + 1..].trim();
            if after_eq.starts_with('"') && after_eq.len() >= 2 {
                let inner = &after_eq[1..];
                let end = inner.find('"').unwrap_or(inner.len());
                return Some(inner[..end].to_string());
            }
        }
    }
    None
}

/// Extract TOML array: key = ["a", "b"] or multiline
pub fn brief_toml_arr(content: &str, key: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut in_array = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if !in_array {
            if let Some(eq_pos) = trimmed.find('=') {
                let before = trimmed[..eq_pos].trim();
                if before == key && trimmed[eq_pos..].contains('[') {
                    in_array = true;
                    if let Some(bs) = trimmed.find('[') {
                        if let Some(be) = trimmed.rfind(']') {
                            for item in trimmed[bs + 1..be].split(',') {
                                let v = item.trim().trim_matches('"');
                                if !v.is_empty() {
                                    result.push(v.to_string());
                                }
                            }
                            return result;
                        }
                    }
                }
            }
        } else {
            if trimmed.starts_with(']') {
                break;
            }
            let v = trimmed.trim_end_matches(',').trim().trim_matches('"');
            if !v.is_empty() {
                result.push(v.to_string());
            }
        }
    }
    result
}

/// Extract TOML array under a specific [section] header
pub fn brief_toml_arr_after_section(content: &str, section: &str, key: &str) -> Vec<String> {
    let mut in_section = false;
    let mut result = Vec::new();
    let mut in_array = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == section {
            in_section = true;
            continue;
        }
        if in_section
            && trimmed.starts_with('[')
            && !trimmed.starts_with("[[")
            && trimmed != section
        {
            break; // next section
        }
        if !in_section {
            continue;
        }

        if !in_array {
            if let Some(eq_pos) = trimmed.find('=') {
                let before = trimmed[..eq_pos].trim();
                if before == key && trimmed[eq_pos..].contains('[') {
                    in_array = true;
                    if let Some(bs) = trimmed.find('[') {
                        if let Some(be) = trimmed.rfind(']') {
                            for item in trimmed[bs + 1..be].split(',') {
                                let v = item.trim().trim_matches('"');
                                if !v.is_empty() {
                                    result.push(v.to_string());
                                }
                            }
                            return result;
                        }
                    }
                }
            }
        } else {
            if trimmed.starts_with(']') {
                break;
            }
            let v = trimmed.trim_end_matches(',').trim().trim_matches('"');
            if !v.is_empty() {
                result.push(v.to_string());
            }
        }
    }
    result
}

/// Extract JSON string array by key
pub fn brief_json_arr(content: &str, key: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut in_array = false;
    let search = format!("\"{}\"", key);

    for line in content.lines() {
        let trimmed = line.trim();
        if !in_array {
            if trimmed.contains(&search) && trimmed.contains('[') {
                in_array = true;
                if let Some(colon) = trimmed.find(':') {
                    let after = &trimmed[colon + 1..];
                    if let (Some(bs), Some(be)) = (after.find('['), after.rfind(']')) {
                        for item in after[bs + 1..be].split(',') {
                            let v = item.trim().trim_matches('"');
                            if !v.is_empty() {
                                result.push(v.to_string());
                            }
                        }
                        return result;
                    }
                }
            }
        } else {
            if trimmed.starts_with(']') {
                break;
            }
            let v = trimmed.trim_end_matches(',').trim().trim_matches('"');
            if !v.is_empty() {
                result.push(v.to_string());
            }
        }
    }
    result
}

/// Extract JSON "key": value
pub fn brief_json_val(content: &str, key: &str) -> Option<String> {
    let search = format!("\"{}\"", key);
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains(&search) && trimmed.contains(':') {
            if let Some(colon) = trimmed.find(':') {
                let val = trimmed[colon + 1..]
                    .trim()
                    .trim_end_matches(',')
                    .trim()
                    .trim_matches('"');
                if !val.is_empty() && !val.contains('[') && !val.contains('{') {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}
