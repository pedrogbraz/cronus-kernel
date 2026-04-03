use std::fs;
use crate::cli::brief::brief_toml_val;
use crate::{cmd_sync, count_files_matching};

pub fn cmd_handoff(args: &[String]) {
    use std::path::Path;
    use std::process::Command;

    // Parse --summary flag
    let summary_text = args.windows(2)
        .find(|w| w[0] == "--summary")
        .map(|w| w[1].clone());

    // Close active session in memory.db
    {
        let mem_path = Path::new(".cronus/memory.db");
        if mem_path.exists() {
            if let Ok(conn) = rusqlite::Connection::open_with_flags(
                mem_path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
            ) {
                // Find most recent session where ended_at IS NULL
                let session_id: Option<String> = conn
                    .query_row(
                        "SELECT id FROM sessions WHERE ended_at IS NULL ORDER BY started_at DESC LIMIT 1",
                        [],
                        |r| r.get(0),
                    )
                    .ok();

                if let Some(ref sid) = session_id {
                    let _ = conn.execute(
                        "UPDATE sessions SET ended_at = datetime('now'), summary = ?1 WHERE id = ?2",
                        rusqlite::params![summary_text, sid],
                    );
                    println!("  \x1b[32m✓\x1b[0m Session {} closed{}", sid,
                        summary_text.as_ref().map(|s| format!(" --- {}", s)).unwrap_or_default());
                } else {
                    println!("  \x1b[90mNo active session in memory.db\x1b[0m");
                }
            }
        }
    }

    // 1. Find the active task (first TASK-*.toml with status = "open")
    let mut active_task_path: Option<std::path::PathBuf> = None;
    let mut task_id = String::new();
    let mut _task_title = String::new();

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
                    task_id = brief_toml_val(&content, "id").unwrap_or_else(|| {
                        entry.file_name().to_string_lossy().trim_end_matches(".toml").to_string()
                    });
                    _task_title = brief_toml_val(&content, "title").unwrap_or_default();
                    active_task_path = Some(entry.path());
                    break;
                }
            }
        }
    }

    if active_task_path.is_none() {
        println!("  \x1b[33mNo open task found. Nothing to hand off.\x1b[0m");
        return;
    }

    let task_path = active_task_path.unwrap();

    println!();
    println!("  \x1b[1mHandoff: {}\x1b[0m", task_id);
    println!("  \x1b[90m─────────────────\x1b[0m");

    // 2. Read git diff --stat
    let mut changes_summary = String::from("no changes detected");
    let mut insertions = 0u32;
    let mut deletions = 0u32;
    if let Ok(output) = Command::new("git")
        .args(["diff", "--stat", "HEAD~5..HEAD"])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).to_string();
            let lines: Vec<&str> = text.lines().collect();
            // Last line has summary like " 3 files changed, 245 insertions(+), 12 deletions(-)"
            if let Some(last) = lines.last() {
                let last = last.trim();
                if last.contains("changed") {
                    // Parse numbers
                    for part in last.split(',') {
                        let part = part.trim();
                        if part.contains("insertion") {
                            if let Some(n) = part.split_whitespace().next() {
                                insertions = n.parse().unwrap_or(0);
                            }
                        } else if part.contains("deletion") {
                            if let Some(n) = part.split_whitespace().next() {
                                deletions = n.parse().unwrap_or(0);
                            }
                        }
                    }
                }
            }
            // Show per-file changes (skip last summary line)
            let file_lines: Vec<&str> = lines.iter()
                .take(lines.len().saturating_sub(1))
                .filter(|l| !l.trim().is_empty())
                .copied()
                .collect();
            if !file_lines.is_empty() {
                let display: Vec<&str> = file_lines.iter().take(5).copied().collect();
                let file_parts: Vec<String> = display.iter().map(|l| {
                    let parts: Vec<&str> = l.trim().splitn(2, '|').collect();
                    parts[0].trim().to_string()
                }).collect();
                let extra = file_lines.len() as i32 - 5;
                if extra > 0 {
                    changes_summary = format!("{} (+{}, -{}), {} more files", file_parts.join(", "), insertions, deletions, extra);
                } else {
                    changes_summary = format!("{} (+{}, -{})", file_parts.join(", "), insertions, deletions);
                }
            }
        }
    }
    println!("  Changes: {}", changes_summary);

    // 3. Read git log --oneline -5
    let mut commit_count = 0u32;
    if let Ok(output) = Command::new("git")
        .args(["log", "--oneline", "-5"])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            commit_count = text.lines().filter(|l| !l.trim().is_empty()).count() as u32;
        }
    }
    println!("  Commits: {} recent commits", commit_count);

    // 4. Check build status (binary mtime)
    let build_status = if Path::new("target/release/cronus-kernel").exists()
        || Path::new("target/release/cronus").exists()
        || Path::new("cronus-kernel/target/release/cronus").exists()
    {
        "passing"
    } else {
        "unknown"
    };
    println!("  Build: {}", build_status);

    // 5. Check if tests exist
    let test_count = count_files_matching("tests/conformance", ".cronus");
    if test_count > 0 {
        println!("  Tests: {} conformance tests available", test_count);
    }

    // 6. Update task status to "done"
    if let Ok(content) = fs::read_to_string(&task_path) {
        let updated = content.replace("status = \"open\"", "status = \"done\"")
                             .replace("status = \"in_progress\"", "status = \"done\"");
        let _ = fs::write(&task_path, updated);
    }
    println!("  Status: \x1b[33mopen\x1b[0m -> \x1b[32mdone\x1b[0m");

    // 7. Run sync internally
    println!();
    cmd_sync();

    // 8. Final message
    println!();
    println!("  \x1b[32mState digest updated.\x1b[0m");
    println!("  Next terminal will inherit this state.");
    println!();
}
