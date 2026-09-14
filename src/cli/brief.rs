use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::parser::{self, AstNode};

pub fn cmd_brief() {
    print!("{}", render_brief(Path::new(".")));
}

/// First `*.cronus` file in `root` (sorted by name).
fn first_cronus_file(root: &Path) -> Option<std::path::PathBuf> {
    let mut files: Vec<_> = fs::read_dir(root)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("cronus"))
        .collect();
    files.sort();
    files.into_iter().next()
}

/// The context brief for the project in `root`. Every source is optional:
/// `.cronus/constitution.toml`, `.cronus/objective.toml`,
/// `.cronus/state-digest.json`, `.cronus/tasks/TASK-*.toml` and the `.cronus`
/// file itself. Missing values are omitted, never printed as placeholders.
pub fn render_brief(root: &Path) -> String {
    let mut out = String::new();
    let today = brief_today_date();
    let read = |rel: &str| fs::read_to_string(root.join(rel)).unwrap_or_default();

    // --- The .cronus file ---
    let cronus_file = first_cronus_file(root);
    let mut app_name: Option<String> = None;
    let mut summary: Option<String> = None;
    if let Some(ref path) = cronus_file {
        let fname = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        match fs::read_to_string(path).map(|s| parser::parse(&s)) {
            Ok(Ok(nodes)) => {
                app_name = nodes.iter().find_map(|n| match n {
                    AstNode::App(a) => {
                        Some(a.name.split('|').next().unwrap_or("").trim().to_string())
                    }
                    _ => None,
                });
                let (e, p, r) = parser::stats(&nodes);
                summary = Some(format!(
                    "{}: {} entities, {} pages, {} routes.",
                    fname, e, p, r
                ));
            }
            Ok(Err(e)) => {
                summary = Some(format!(
                    "{} does not parse: {}. Run `cronus build --ai`.",
                    fname, e
                ));
            }
            Err(_) => {}
        }
    }

    // --- Read constitution.toml ---
    let constitution = read(".cronus/constitution.toml");
    let proj_name = brief_toml_val(&constitution, "name")
        .filter(|s| !s.is_empty())
        .or(app_name.filter(|s| !s.is_empty()))
        .or_else(|| {
            root.canonicalize()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        })
        .unwrap_or_else(|| "This project".into());
    let proj_purpose = brief_toml_val(&constitution, "purpose").unwrap_or_default();
    let proj_category = brief_toml_val(&constitution, "category").unwrap_or_default();
    let invariants = brief_toml_arr(&constitution, "must");

    // --- Read objective.toml ---
    let objective = read(".cronus/objective.toml");
    let obj_title = brief_toml_val(&objective, "title").filter(|s| !s.is_empty());
    let obj_deadline = brief_toml_val(&objective, "deadline").filter(|s| !s.is_empty());
    let obj_why = brief_toml_val(&objective, "why").unwrap_or_default();
    let out_of_scope = brief_toml_arr(&objective, "items");

    // --- Read state-digest.json ---
    let digest = read(".cronus/state-digest.json");
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

    if let Ok(entries) = fs::read_dir(root.join(".cronus/tasks")) {
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

    // --- Render brief ---
    let _ = writeln!(out, "# CRONUS Context Brief — {}\n", today);

    let _ = writeln!(out, "## Project");
    let mut line = if proj_category.is_empty() {
        format!("{}.", proj_name)
    } else {
        format!("{} is a {}.", proj_name, proj_category)
    };
    let purpose = proj_purpose.trim().trim_end_matches('.');
    if !purpose.is_empty() {
        line.push_str(&format!(" {}.", purpose));
    }
    let _ = writeln!(out, "{}", line);
    if let Some(ref s) = summary {
        let _ = writeln!(out, "{}", s);
    } else {
        let _ = writeln!(out, "No .cronus file in this directory.");
    }
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
        let _ = writeln!(out, "Rust kernel, {}.", meta.join(", "));
    }
    let _ = writeln!(out);

    let _ = writeln!(out, "## Current Objective");
    match obj_title {
        Some(title) => {
            let title = title.trim_end_matches('.');
            match obj_deadline {
                Some(d) => {
                    let _ = writeln!(out, "{}. Deadline: {}.", title, d);
                }
                None => {
                    let _ = writeln!(out, "{}.", title);
                }
            }
        }
        None => {
            let _ = writeln!(out, "No objective set (.cronus/objective.toml).");
        }
    }
    if !obj_why.is_empty() {
        let _ = writeln!(out, "Why: {}", obj_why);
    }
    let _ = writeln!(out);

    if !task_id.is_empty() {
        let _ = writeln!(out, "## Your Task");
        let _ = writeln!(out, "{}: {}", task_id, task_title);
        if !task_write.is_empty() {
            let files: Vec<String> = task_write
                .iter()
                .map(|f| f.rsplit('/').next().unwrap_or(f).to_string())
                .collect();
            let _ = writeln!(out, "Files: {}", files.join(", "));
        }
        if !task_forbidden.is_empty() {
            let _ = writeln!(out, "Do NOT touch: {}", task_forbidden.join(", "));
        }
        let _ = writeln!(out);

        if !task_done_checks.is_empty() {
            let _ = writeln!(out, "## Done When");
            for check in &task_done_checks {
                let _ = writeln!(out, "- {}", check);
            }
            let _ = writeln!(out);
        }

        let _ = writeln!(out, "## Current State");
        if !task_context.is_empty() {
            let _ = writeln!(out, "{}", task_context);
        }
    }

    if !completed.is_empty() {
        let recent: Vec<&String> = completed.iter().rev().take(5).collect();
        let _ = writeln!(out, "Recent completed:");
        for t in recent {
            let _ = writeln!(out, "  - {}", t);
        }
        let _ = writeln!(out);
    }

    if !risks.is_empty() {
        let _ = writeln!(out, "## Risks");
        for r in &risks {
            let _ = writeln!(out, "- {}", r);
        }
        let _ = writeln!(out);
    }

    if !invariants.is_empty() {
        let _ = writeln!(out, "## Rules");
        for inv in &invariants {
            let _ = writeln!(out, "- {}", inv);
        }
        let _ = writeln!(out);
    }

    if !out_of_scope.is_empty() {
        let _ = writeln!(out, "## Out of Scope");
        for item in &out_of_scope {
            let _ = writeln!(out, "- {}", item);
        }
        let _ = writeln!(out);
    }
    out
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

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("cronus-brief-{}-{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn brief_without_metadata_has_no_placeholders() {
        let dir = temp_dir("empty");
        let out = render_brief(&dir);
        assert!(!out.contains("Unknown is a"), "{out}");
        assert!(!out.contains(" . ."), "{out}");
        assert!(!out.contains("Deadline: none"), "{out}");
        assert!(out.contains("No objective set"), "{out}");
        assert!(out.contains("No .cronus file"), "{out}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn brief_uses_app_name_and_stats_from_cronus_file() {
        let dir = temp_dir("app");
        fs::write(
            dir.join("app.cronus"),
            "app \"Orders\" { port 5175 }\nentity Order { number string! }\n",
        )
        .unwrap();
        let out = render_brief(&dir);
        assert!(out.contains("Orders."), "{out}");
        assert!(out.contains("app.cronus: 1 entities, 0 pages"), "{out}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn brief_reads_constitution_and_objective() {
        let dir = temp_dir("meta");
        fs::create_dir_all(dir.join(".cronus")).unwrap();
        fs::write(
            dir.join(".cronus/constitution.toml"),
            "name = \"Shop\"\ncategory = \"storefront\"\npurpose = \"Sell things\"\nmust = [\"owner scope\"]\n",
        )
        .unwrap();
        fs::write(
            dir.join(".cronus/objective.toml"),
            "title = \"Launch\"\ndeadline = \"2026-10-01\"\n",
        )
        .unwrap();
        let out = render_brief(&dir);
        assert!(out.contains("Shop is a storefront. Sell things."), "{out}");
        assert!(out.contains("Launch. Deadline: 2026-10-01."), "{out}");
        assert!(out.contains("- owner scope"), "{out}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn brief_reports_parse_errors() {
        let dir = temp_dir("broken");
        fs::write(
            dir.join("app.cronus"),
            "page \"/x\" { section kpi { bind X } }",
        )
        .unwrap();
        let out = render_brief(&dir);
        assert!(out.contains("app.cronus does not parse"), "{out}");
        let _ = fs::remove_dir_all(&dir);
    }
}
