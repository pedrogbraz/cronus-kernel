use crate::open_memory_db;

pub fn cmd_memory(args: &[String]) {
    let sub = args.get(2).map(|s| s.as_str()).unwrap_or("help");

    match sub {
        "sessions" => {
            let mem = match open_memory_db() {
                Ok(m) => m,
                Err(e) => { eprintln!("  \x1b[31m✗\x1b[0m {}", e); return; }
            };
            match mem.get_recent_sessions(20) {
                Ok(sessions) => {
                    if sessions.is_empty() {
                        println!("  No sessions recorded yet.");
                        return;
                    }
                    println!();
                    println!("  \x1b[1mRecent Sessions\x1b[0m ({} total)", sessions.len());
                    println!();
                    for s in &sessions {
                        let id = s["id"].as_str().unwrap_or("?");
                        let started = s["started_at"].as_str().unwrap_or("?");
                        let agent = s["agent"].as_str().unwrap_or("-");
                        let changes = s["changes_count"].as_i64().unwrap_or(0);
                        let ended = if s["ended_at"].is_null() { "active" } else { "done" };
                        let summary = s["summary"].as_str().unwrap_or("");
                        println!("  \x1b[36m{}\x1b[0m  {}  agent={}  changes={}  [{}]",
                            &id[..id.len().min(20)], started, agent, changes, ended);
                        if !summary.is_empty() {
                            println!("    {}", summary);
                        }
                    }
                    println!();
                }
                Err(e) => eprintln!("  \x1b[31m✗\x1b[0m {}", e),
            }
        }
        "decisions" => {
            let mem = match open_memory_db() {
                Ok(m) => m,
                Err(e) => { eprintln!("  \x1b[31m✗\x1b[0m {}", e); return; }
            };
            match mem.get_decisions(20) {
                Ok(decisions) => {
                    if decisions.is_empty() {
                        println!("  No decisions recorded yet.");
                        return;
                    }
                    println!();
                    println!("  \x1b[1mRecent Decisions\x1b[0m ({} total)", decisions.len());
                    println!();
                    for d in &decisions {
                        let date = d["date"].as_str().unwrap_or("?");
                        let decision = d["decision"].as_str().unwrap_or("?");
                        let category = d["category"].as_str().unwrap_or("-");
                        let reason = d["reason"].as_str().unwrap_or("");
                        println!("  \x1b[33m[{}]\x1b[0m {} \x1b[90m({})\x1b[0m", category, decision, date);
                        if !reason.is_empty() {
                            println!("    reason: {}", reason);
                        }
                    }
                    println!();
                }
                Err(e) => eprintln!("  \x1b[31m✗\x1b[0m {}", e),
            }
        }
        "log" => {
            let description = args.get(3).map(|s| s.as_str()).unwrap_or("");
            if description.is_empty() {
                eprintln!("  \x1b[31m✗\x1b[0m Usage: cronus memory log \"description\"");
                return;
            }
            let mem = match open_memory_db() {
                Ok(m) => m,
                Err(e) => { eprintln!("  \x1b[31m✗\x1b[0m {}", e); return; }
            };
            match mem.add_changelog(None, "manual", description) {
                Ok(_) => println!("  \x1b[32m✓\x1b[0m Logged: {}", description),
                Err(e) => eprintln!("  \x1b[31m✗\x1b[0m {}", e),
            }
        }
        "decide" => {
            let decision = args.get(3).map(|s| s.as_str()).unwrap_or("");
            if decision.is_empty() {
                eprintln!("  \x1b[31m✗\x1b[0m Usage: cronus memory decide \"decision\" --reason \"why\" --category \"arch\"");
                return;
            }
            let reason = find_flag_value(args, "--reason");
            let category = find_flag_value(args, "--category");
            let mem = match open_memory_db() {
                Ok(m) => m,
                Err(e) => { eprintln!("  \x1b[31m✗\x1b[0m {}", e); return; }
            };
            match mem.add_decision(None, decision, reason.as_deref(), category.as_deref()) {
                Ok(_) => {
                    println!("  \x1b[32m✓\x1b[0m Decision recorded: {}", decision);
                    if let Some(ref r) = reason { println!("    reason: {}", r); }
                    if let Some(ref c) = category { println!("    category: {}", c); }
                }
                Err(e) => eprintln!("  \x1b[31m✗\x1b[0m {}", e),
            }
        }
        _ => {
            println!();
            println!("  \x1b[1mCRONUS Semantic Memory\x1b[0m");
            println!();
            println!("  Usage: cronus memory <subcommand>");
            println!();
            println!("  \x1b[32msessions\x1b[0m            List recent sessions");
            println!("  \x1b[32mdecisions\x1b[0m           List recorded decisions");
            println!("  \x1b[32mlog\x1b[0m \"desc\"          Add manual changelog entry");
            println!("  \x1b[32mdecide\x1b[0m \"what\" --reason \"why\" --category \"cat\"");
            println!("                      Record an architectural decision");
            println!();
        }
    }
}

/// Extract --flag value from args.
fn find_flag_value(args: &[String], flag: &str) -> Option<String> {
    for (i, a) in args.iter().enumerate() {
        if a == flag {
            return args.get(i + 1).cloned();
        }
    }
    None
}
