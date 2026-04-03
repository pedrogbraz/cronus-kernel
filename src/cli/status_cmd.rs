use std::fs;

use crate::cli::brief::{brief_toml_val, brief_toml_arr, brief_toml_arr_after_section, brief_today_date};
use crate::cli::objective_kernel::{count_files_matching, status_days_between};

pub fn cmd_status() {
    use std::path::Path;

    // -- 1. Project identity from constitution.toml --
    let constitution = fs::read_to_string(".cronus/constitution.toml").unwrap_or_default();
    let _proj_name = brief_toml_val(&constitution, "name").unwrap_or_else(|| "Unknown".into());

    // -- 2. Objective from objective.toml --
    let objective = fs::read_to_string(".cronus/objective.toml").unwrap_or_default();
    let obj_title = brief_toml_val(&objective, "title").unwrap_or_else(|| "(no objective set)".into());
    let obj_deadline = brief_toml_val(&objective, "deadline").unwrap_or_else(|| "none".into());
    let success_criteria = brief_toml_arr(&objective, "criteria");

    // -- 3. Count completed from done section --
    let done_completed = brief_toml_arr_after_section(&objective, "[done]", "completed");

    // Cross-reference: count how many success criteria are "met"
    let criteria_total = success_criteria.len();
    let criteria_met = done_completed.len();

    // -- 4. Calculate days remaining --
    let today = brief_today_date();
    let days_remaining = status_days_between(&today, &obj_deadline);
    let deadline_display = if days_remaining >= 0 {
        format!("{} ({} days remaining)", obj_deadline, days_remaining)
    } else {
        format!("{} (\x1b[31m{} days overdue\x1b[0m)", obj_deadline, -days_remaining)
    };

    // -- 5. Read tasks --
    let mut done_count = 0usize;
    let mut in_progress_count = 0usize;
    let mut open_count = 0usize;
    let mut active_task_id = String::new();
    let mut active_task_title = String::new();
    let mut active_task_scope = String::new();
    let mut active_task_status = String::new();

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
                let id = brief_toml_val(&content, "id").unwrap_or_default();
                let title = brief_toml_val(&content, "title").unwrap_or_default();
                let status = brief_toml_val(&content, "status").unwrap_or_else(|| "open".into());
                let write_files = brief_toml_arr(&content, "write");

                match status.as_str() {
                    "done" => done_count += 1,
                    "in_progress" => {
                        in_progress_count += 1;
                        if active_task_id.is_empty() {
                            active_task_id = id;
                            active_task_title = title;
                            active_task_status = "in_progress".into();
                            active_task_scope = write_files.iter().map(|f| {
                                f.rsplit('/').next().unwrap_or(f).to_string()
                            }).collect::<Vec<_>>().join(", ");
                        }
                    }
                    _ => {
                        open_count += 1;
                        if active_task_id.is_empty() {
                            active_task_id = id;
                            active_task_title = title;
                            active_task_status = "open".into();
                            active_task_scope = write_files.iter().map(|f| {
                                f.rsplit('/').next().unwrap_or(f).to_string()
                            }).collect::<Vec<_>>().join(", ");
                        }
                    }
                }
            }
        }
    }

    // -- 6. Segments --
    let mut seg_active = 0usize;
    let mut seg_merged = 0usize;
    if let Ok(entries) = fs::read_dir(".cronus/segments") {
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().ends_with(".toml") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let seg_status = brief_toml_val(&content, "status").unwrap_or_default();
                    match seg_status.as_str() {
                        "merged" => seg_merged += 1,
                        _ => seg_active += 1,
                    }
                }
            }
        }
    }

    // -- 7. Build status --
    let build = if Path::new("target/release/cronus-kernel").exists()
        || Path::new("target/release/cronus").exists()
        || Path::new("cronus-kernel/target/release/cronus").exists()
    {
        "\x1b[32mpassing\x1b[0m"
    } else {
        "\x1b[33munknown\x1b[0m"
    };

    // -- 8. Metrics --
    let spec_count = count_files_matching("specs", "spec.toml");
    let test_count = count_files_matching("tests/conformance", ".cronus");
    let example_count = count_files_matching("examples", ".cronus");

    // -- Print --
    let bar_len = 38;
    let header = format!("CRONUS — Project Status");
    println!();
    println!("  \x1b[36m╔{}╗\x1b[0m", "═".repeat(bar_len));
    println!("  \x1b[36m║\x1b[0m  \x1b[1m{:<width$}\x1b[0m \x1b[36m║\x1b[0m", header, width = bar_len - 3);
    println!("  \x1b[36m╚{}╝\x1b[0m", "═".repeat(bar_len));
    println!();
    println!("  \x1b[1mObjective:\x1b[0m {}", obj_title);
    println!("  \x1b[1mDeadline:\x1b[0m  {}", deadline_display);
    if criteria_total > 0 {
        println!("  \x1b[1mProgress:\x1b[0m  {}/{} success criteria met", criteria_met, criteria_total);
    }
    println!();

    if !active_task_id.is_empty() {
        println!("  \x1b[1mActive Task:\x1b[0m {} — {}", active_task_id, active_task_title);
        if !active_task_scope.is_empty() {
            println!("  \x1b[1mScope:\x1b[0m {}", active_task_scope);
        }
        println!("  \x1b[1mStatus:\x1b[0m {}", active_task_status);
        println!();
    }

    println!("  \x1b[1mSegments:\x1b[0m {} active, {} merged", seg_active, seg_merged);
    println!();
    println!("  \x1b[1mBuild:\x1b[0m {}", build);
    println!("  \x1b[1mSpecs:\x1b[0m {} | \x1b[1mTests:\x1b[0m {} | \x1b[1mExamples:\x1b[0m {}", spec_count, test_count, example_count);
    println!();
    println!("  \x1b[1mTasks:\x1b[0m {} done, {} in_progress, {} open",
        done_count, in_progress_count, open_count);
    println!();
}
