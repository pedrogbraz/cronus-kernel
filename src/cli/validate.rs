use std::fs;
use serde_json::{json, Value};
use crate::parser::AstNode;
use crate::{parser, contracts, find_cronus_file};

pub fn cmd_validate(args: &[String]) {
    let strict_ai = args.iter().any(|a| a == "--strict-ai");
    let json_output = args.iter().any(|a| a == "--json") || strict_ai;

    let file = args.iter().skip(2)
        .find(|a| a.ends_with(".cronus"))
        .cloned()
        .or_else(find_cronus_file);

    let file = match file {
        Some(f) => f,
        None => {
            if json_output {
                println!("{}", json!({"valid": false, "errors": [{"message": "No .cronus file found"}], "warnings": []}));
            } else {
                eprintln!("No .cronus file found");
            }
            std::process::exit(1);
        }
    };

    let source = fs::read_to_string(&file).unwrap_or_default();

    let mut errors: Vec<Value> = Vec::new();
    let mut warnings: Vec<Value> = Vec::new();
    let mut stats = json!({});

    match parser::parse(&source) {
        Ok(nodes) => {
            let (entity_count, page_count, api_routes) = parser::stats(&nodes);

            stats = json!({
                "entities": entity_count,
                "pages": page_count,
                "routes": api_routes,
                "lines": source.lines().count(),
            });

            for node in &nodes {
                if let AstNode::Page(page) = node {
                    for section in &page.sections {
                        let section_warnings = contracts::validate_section(section, &[]);
                        for w in section_warnings {
                            let warning_json = match w {
                                contracts::ParseWarning::UnknownSection { ref name, line } => {
                                    json!({"type": "unknown_section", "section": name, "line": line, "message": format!("Unknown section type '{}'", name)})
                                }
                                contracts::ParseWarning::UnknownKey { ref section, ref key, ref item, line } => {
                                    json!({"type": "unknown_key", "section": section, "key": key, "item": item, "line": line, "message": format!("Unknown key '{}' in section '{}'", key, section)})
                                }
                                contracts::ParseWarning::MissingRequired { ref section, ref key, ref item, line } => {
                                    json!({"type": "missing_required", "section": section, "key": key, "item": item, "line": line, "severity": "error", "message": format!("Missing required key '{}' in section '{}'", key, section)})
                                }
                                contracts::ParseWarning::AliasUsed { ref alias, ref canonical, line } => {
                                    json!({"type": "alias", "alias": alias, "canonical": canonical, "line": line, "message": format!("'{}' is an alias for '{}', consider using canonical name", alias, canonical)})
                                }
                                contracts::ParseWarning::MinItemsViolation { ref section, expected, actual, line } => {
                                    json!({"type": "min_items", "section": section, "expected": expected, "actual": actual, "line": line, "message": format!("Section '{}' requires at least {} items, found {}", section, expected, actual)})
                                }
                                contracts::ParseWarning::UnknownConfig { ref section, ref key, line } => {
                                    json!({"type": "unknown_config", "section": section, "key": key, "line": line, "message": format!("Unknown config key '{}' in section '{}'", key, section)})
                                }
                            };
                            warnings.push(warning_json);
                        }
                    }
                }
            }
        }
        Err(e) => {
            errors.push(json!({
                "type": "parse_error",
                "message": e,
                "severity": "error"
            }));
        }
    }

    // strict-ai: promote all warnings to errors
    if strict_ai && !warnings.is_empty() {
        for w in &warnings {
            errors.push(w.clone());
        }
        warnings.clear();
    }

    let valid = errors.is_empty();

    if json_output {
        let result = json!({
            "valid": valid,
            "file": file,
            "errors": errors,
            "warnings": warnings,
            "stats": stats,
        });
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else {
        if valid {
            println!("\n  \x1b[32m✓\x1b[0m {} is valid", file);
            if let Some(entities) = stats.get("entities") {
                println!("    {} entities, {} pages, {} routes",
                    entities, stats.get("pages").unwrap_or(&json!(0)),
                    stats.get("routes").unwrap_or(&json!(0)));
            }
        } else {
            println!("\n  \x1b[31m✗\x1b[0m {} has errors:", file);
            for err in &errors {
                println!("    \x1b[31m✗\x1b[0m {}", err.get("message").and_then(|v| v.as_str()).unwrap_or("unknown error"));
            }
        }
        for w in &warnings {
            println!("    \x1b[33m⚠\x1b[0m {}", w.get("message").and_then(|v| v.as_str()).unwrap_or(""));
        }
        println!();
    }

    if !valid {
        std::process::exit(1);
    }
}

pub fn cmd_validate_mission() {
    use std::process::Command;
    use crate::cli::brief::{brief_toml_val, brief_toml_arr, brief_toml_arr_after_section};
    use crate::cli::objective_kernel::count_files_matching;
    use crate::cli::objective_kernel::lease_file_allowed;

    println!();
    println!("  \x1b[1mMission Validation\x1b[0m");
    println!("  \x1b[90m──────────────────\x1b[0m");

    // 1. Read constitution.toml [forbidden] items
    let constitution = fs::read_to_string(".cronus/constitution.toml").unwrap_or_default();
    let forbidden_items = brief_toml_arr_after_section(&constitution, "[forbidden]", "never");
    let forbidden_fallback = brief_toml_arr(&constitution, "never");
    let forbidden = if !forbidden_items.is_empty() { forbidden_items } else { forbidden_fallback };

    // 2. Read active task write scope for filtering
    let mut write_scope: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(".cronus/tasks") {
        let mut files: Vec<_> = entries.flatten()
            .filter(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                n.starts_with("TASK-") && n.ends_with(".toml")
            })
            .collect();
        files.sort_by_key(|e| e.file_name());
        for entry in files {
            let content = fs::read_to_string(entry.path()).unwrap_or_default();
            if let Some(status) = brief_toml_val(&content, "status") {
                if status == "open" || status == "in_progress" {
                    let scope = brief_toml_arr_after_section(&content, "[scope]", "write");
                    write_scope = if !scope.is_empty() { scope } else { brief_toml_arr(&content, "write") };
                    break;
                }
            }
        }
    }

    // 3. Read git diff content
    let diff_content = {
        let staged = Command::new("git")
            .args(["diff", "--cached"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        if !staged.trim().is_empty() {
            staged
        } else {
            Command::new("git")
                .args(["diff", "HEAD~1..HEAD"])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_default()
        }
    };

    // 4. Check if any forbidden pattern appears in the diff
    let mut constitution_pass = true;
    let mut violations: Vec<String> = Vec::new();
    let mut in_code_file = false;
    let added_code_lines: Vec<String> = diff_content.lines()
        .filter(|l| {
            if l.starts_with("+++ b/") {
                let path = &l[6..];
                let is_code = path.ends_with(".rs") || path.ends_with(".cronus");
                let in_scope = write_scope.is_empty() || lease_file_allowed(path, &write_scope);
                in_code_file = is_code && in_scope;
                return false;
            }
            if l.starts_with("--- ") || l.starts_with("diff --git") {
                return false;
            }
            in_code_file && l.starts_with('+') && !l.starts_with("+++")
        })
        .map(|l| l[1..].to_lowercase())
        .collect();
    let code_text = added_code_lines.join("\n");

    for item in &forbidden {
        let item_lower = item.to_lowercase();
        if item_lower.contains("fake data") && (code_text.contains("math.random") || code_text.contains("mock_data") || code_text.contains("fake_data")) {
            constitution_pass = false;
            violations.push(item.clone());
        }
        if item_lower.contains("export") && (item_lower.contains("react") || item_lower.contains("vue") || item_lower.contains("svelte")) {
            if code_text.contains("use react") || code_text.contains("import react")
                || code_text.contains("use vue") || code_text.contains("import vue")
                || code_text.contains("use svelte") || code_text.contains("import svelte")
                || (code_text.contains("reactdom") || code_text.contains("createapp")) {
                constitution_pass = false;
                violations.push(item.clone());
            }
        }
    }

    if constitution_pass {
        println!("  Constitution: \x1b[32m✓ PASS\x1b[0m (no forbidden patterns)");
    } else {
        println!("  Constitution: \x1b[31m✗ FAIL\x1b[0m");
        for v in &violations {
            println!("    - {}", v);
        }
    }

    // 4. Read objective.toml success criteria
    let objective = fs::read_to_string(".cronus/objective.toml").unwrap_or_default();
    let _obj_title = brief_toml_val(&objective, "title").unwrap_or_else(|| "No objective".into());

    // 5. Check basic alignment
    let mut objective_pass = true;
    let out_of_scope = brief_toml_arr(&objective, "items");
    for item in &out_of_scope {
        let item_lower = item.to_lowercase();
        let keywords: Vec<&str> = item_lower.split_whitespace()
            .filter(|w| w.len() > 6)
            .collect();
        if keywords.len() >= 2 {
            let matches: usize = keywords.iter().filter(|k| code_text.contains(**k)).count();
            let exact_match = code_text.contains(&item_lower);
            if exact_match || (keywords.len() >= 3 && matches >= keywords.len()) {
                objective_pass = false;
                println!("  Objective: \x1b[31m✗ FAIL\x1b[0m (out-of-scope work detected: {})", item);
                break;
            }
        }
    }
    if objective_pass {
        println!("  Objective: \x1b[32m✓ PASS\x1b[0m (changes serve current goal)");
    }

    // 6. Check spec coverage
    let spec_count = count_files_matching("specs", ".spec.toml");
    let test_count = count_files_matching("tests/conformance", ".cronus");
    if spec_count == 0 && test_count == 0 {
        println!("  Spec coverage: \x1b[33mno specs found\x1b[0m");
    } else {
        println!("  Spec coverage: {} specs, {} conformance tests", spec_count, test_count);
    }

    // 7. Build check
    let build_ok = std::path::Path::new("target/release/cronus-kernel").exists()
        || std::path::Path::new("target/release/cronus").exists()
        || std::path::Path::new("cronus-kernel/target/release/cronus").exists();

    if build_ok {
        println!("  Build: \x1b[32m✓ PASS\x1b[0m");
    } else {
        println!("  Build: \x1b[33m⚠ unknown\x1b[0m (no release binary found)");
    }

    println!();
}
