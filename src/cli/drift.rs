use std::fs;

use crate::cli::objective_kernel::{
    git_changed_files, git_diff_content, lease_check_scope, DriftResult,
};

pub fn cmd_drift(args: &[String]) {
    let explain = args.iter().any(|a| a == "--explain");
    let mut warnings: Vec<String> = Vec::new();

    println!();
    println!("\x1b[1mCRONUS Drift Analysis\x1b[0m");
    println!("\x1b[90m\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\x1b[0m");

    let changed = git_changed_files();
    let diff_content = git_diff_content();

    // 1. Strategic Drift
    let strategic = drift_check_strategic(&changed);
    match &strategic {
        DriftResult::Ok(msg) => {
            println!("  Strategic: \x1b[32m\u{2713} OK\x1b[0m \u{2014} {}", msg)
        }
        DriftResult::Warn(msg, details) => {
            println!("  Strategic: \x1b[33m\u{26a0} WARN\x1b[0m \u{2014} {}", msg);
            warnings.push(format!("Strategic: {}", msg));
            if explain {
                for d in details {
                    println!("      \x1b[33m\u{2014} {}\x1b[0m", d);
                }
            }
        }
    }

    // 2. Scope Drift
    let scope = drift_check_scope(&changed);
    match &scope {
        DriftResult::Ok(msg) => {
            println!("  Scope:     \x1b[32m\u{2713} OK\x1b[0m \u{2014} {}", msg)
        }
        DriftResult::Warn(msg, details) => {
            println!("  Scope:     \x1b[33m\u{26a0} WARN\x1b[0m \u{2014} {}", msg);
            warnings.push(format!("Scope: {}", msg));
            if explain {
                for d in details {
                    println!("      \x1b[33m\u{2014} {}\x1b[0m", d);
                }
            }
        }
    }

    // 3. Semantic Drift
    let semantic = drift_check_semantic(&changed, &diff_content);
    match &semantic {
        DriftResult::Ok(msg) => {
            println!("  Semantic:  \x1b[32m\u{2713} OK\x1b[0m \u{2014} {}", msg)
        }
        DriftResult::Warn(msg, details) => {
            println!("  Semantic:  \x1b[33m\u{26a0} WARN\x1b[0m \u{2014} {}", msg);
            warnings.push(format!("Semantic: {}", msg));
            if explain {
                for d in details {
                    println!("      \x1b[33m\u{2014} {}\x1b[0m", d);
                }
            }
        }
    }

    println!();
    if warnings.is_empty() {
        println!("  \x1b[32mNo drift detected.\x1b[0m");
    } else {
        println!(
            "  \x1b[33m{} warning{}.\x1b[0m Run `cronus drift --explain` for details.",
            warnings.len(),
            if warnings.len() == 1 { "" } else { "s" }
        );
    }
    println!();
}

/// Strategic drift: are changes aligned with the objective?
fn drift_check_strategic(changed: &[String]) -> DriftResult {
    if changed.is_empty() {
        return DriftResult::Ok("no changes to check".into());
    }

    let objective = fs::read_to_string(".cronus/objective.toml").unwrap_or_default();
    if objective.is_empty() {
        return DriftResult::Ok("no objective.toml \u{2014} skipping".into());
    }

    // Parse out-of-scope items
    let out_of_scope = {
        let mut in_oos = false;
        let mut items = Vec::new();
        for line in objective.lines() {
            let t = line.trim();
            if t == "[out_of_scope]" {
                in_oos = true;
                continue;
            }
            if in_oos && t.starts_with('[') && t != "[out_of_scope]" {
                break;
            }
            if in_oos {
                let v = t
                    .trim_start_matches('"')
                    .trim_end_matches('"')
                    .trim_end_matches(',')
                    .trim_matches('"');
                if !v.is_empty() && !v.starts_with("items") && v != "]" {
                    items.push(v.to_lowercase());
                }
            }
        }
        items
    };

    // Check if changed files relate to out-of-scope items
    let mut drift_details: Vec<String> = Vec::new();
    for file in changed {
        let fl = file.to_lowercase();
        for oos in &out_of_scope {
            for word in oos.split_whitespace() {
                let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
                if clean.len() > 4 && fl.contains(clean) && clean != "support" {
                    let detail = format!("{} (out-of-scope: {})", file, oos);
                    if !drift_details.contains(&detail) {
                        drift_details.push(detail);
                    }
                }
            }
        }
    }

    if !drift_details.is_empty() {
        return DriftResult::Warn(
            format!(
                "{} file(s) may relate to out-of-scope items",
                drift_details.len()
            ),
            drift_details,
        );
    }

    // Check if changes are at least in kernel/project territory
    let relevant = changed.iter().any(|f| {
        f.contains("src/")
            || f.ends_with(".rs")
            || f.ends_with(".cronus")
            || f.contains("specs/")
            || f.contains("tests/")
            || f.contains(".cronus/")
    });

    if relevant {
        DriftResult::Ok("changes align with objective".into())
    } else {
        DriftResult::Warn(
            "changes may not serve current objective".into(),
            changed.to_vec(),
        )
    }
}

/// Scope drift: are changed files within the active task lease?
fn drift_check_scope(changed: &[String]) -> DriftResult {
    if changed.is_empty() {
        return DriftResult::Ok("no changes to check".into());
    }

    let (task_id, outside) = lease_check_scope();

    if task_id.is_empty() {
        return DriftResult::Ok("no active task lease \u{2014} skipping".into());
    }

    if outside.is_empty() {
        DriftResult::Ok("all files within lease".into())
    } else {
        DriftResult::Warn(
            format!("{} file(s) outside lease [{}]", outside.len(), task_id),
            outside,
        )
    }
}

/// Semantic drift: new syntax without spec, or constitution violations?
fn drift_check_semantic(changed: &[String], diff_content: &str) -> DriftResult {
    use std::path::Path;

    let mut details: Vec<String> = Vec::new();
    let mut has_real_warning = false;

    // Check 1: parser.rs modified — look for new match arms with unspecced keywords
    let parser_modified = changed.iter().any(|f| f.contains("parser.rs"));
    if parser_modified {
        let mut new_keywords: Vec<String> = Vec::new();
        for line in diff_content.lines() {
            if !line.starts_with('+') || line.starts_with("+++") {
                continue;
            }
            let trimmed = line[1..].trim();
            if trimmed.contains("=>") && trimmed.contains('"') {
                let mut remaining = trimmed;
                while let Some(start) = remaining.find('"') {
                    remaining = &remaining[start + 1..];
                    if let Some(end) = remaining.find('"') {
                        let keyword = &remaining[..end];
                        if keyword.len() > 1
                            && keyword.len() < 30
                            && !keyword.contains(' ')
                            && !keyword.contains('/')
                            && !keyword.contains('.')
                            && keyword != "true"
                            && keyword != "false"
                            && keyword
                                .chars()
                                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                        {
                            let spec_path = format!("specs/core/{}.spec.toml", keyword);
                            if !Path::new(&spec_path).exists()
                                && !new_keywords.contains(&keyword.to_string())
                            {
                                new_keywords.push(keyword.to_string());
                            }
                        }
                        remaining = &remaining[end + 1..];
                    } else {
                        break;
                    }
                }
            }
        }

        if !new_keywords.is_empty() {
            has_real_warning = true;
            for kw in &new_keywords {
                details.push(format!("\"{}\" has no specs/core/{}.spec.toml", kw, kw));
            }
        }
    }

    // Check 2: constitution forbidden violations in added lines
    let added_lines: String = diff_content
        .lines()
        .filter(|l| l.starts_with('+') && !l.starts_with("+++"))
        .map(|l| if l.len() > 1 { &l[1..] } else { "" })
        .collect::<Vec<_>>()
        .join("\n")
        .to_lowercase();

    if added_lines.contains("math.random()") || added_lines.contains("math::random") {
        has_real_warning = true;
        details.push("fake data pattern detected (Math.random)".into());
    }
    if (added_lines.contains("react")
        || added_lines.contains("vue")
        || added_lines.contains("svelte"))
        && added_lines.contains("import")
    {
        has_real_warning = true;
        details.push("framework import detected \u{2014} CRONUS is the runtime".into());
    }
    if added_lines.contains("mock_data")
        || added_lines.contains("fake_data")
        || added_lines.contains("dummy_data")
    {
        has_real_warning = true;
        details.push("fake/mock data pattern detected".into());
    }

    if has_real_warning {
        let msg = if parser_modified {
            format!("parser.rs modified, {} issue(s) found", details.len())
        } else {
            format!("{} constitution concern(s)", details.len())
        };
        DriftResult::Warn(msg, details)
    } else if parser_modified {
        DriftResult::Ok("parser.rs modified, all keywords have specs".into())
    } else {
        DriftResult::Ok("no new syntax, no constitution violations".into())
    }
}
