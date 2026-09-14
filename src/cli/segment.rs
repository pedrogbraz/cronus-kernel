use std::collections::HashMap;
use std::fs;

use crate::cli::brief::{brief_today_date, brief_toml_arr, brief_toml_val};

pub fn cmd_segment(args: &[String]) {
    let sub = args.get(2).map(|s| s.as_str()).unwrap_or("list");
    match sub {
        "create" => segment_create(args),
        "list" => segment_list(),
        "show" => segment_show(args),
        "check" => segment_check(),
        _ => {
            eprintln!("  \x1b[31mUnknown segment subcommand: {}\x1b[0m", sub);
            eprintln!("  Usage: cronus segment <create|list|show|check>");
            std::process::exit(1);
        }
    }
}

/// cronus segment create "name" --blocks "entity:Product,page:/dashboard"
fn segment_create(args: &[String]) {
    let name = match args.get(3) {
        Some(n) => n.clone(),
        None => {
            eprintln!("  \x1b[31mMissing segment name.\x1b[0m");
            eprintln!("  Usage: cronus segment create \"name\" --blocks \"entity:X,page:/Y\"");
            std::process::exit(1);
        }
    };

    // Parse --blocks flag
    let mut blocks: Vec<String> = Vec::new();
    for (i, arg) in args.iter().enumerate() {
        if arg == "--blocks" {
            if let Some(val) = args.get(i + 1) {
                blocks = val
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }

    if blocks.is_empty() {
        eprintln!("  \x1b[31mMissing --blocks flag.\x1b[0m");
        eprintln!("  Usage: cronus segment create \"name\" --blocks \"entity:X,page:/Y\"");
        std::process::exit(1);
    }

    // Validate block format (type:name)
    for b in &blocks {
        if !b.contains(':') {
            eprintln!(
                "  \x1b[31mInvalid block format: {}\x1b[0m (expected type:name, e.g. entity:User)",
                b
            );
            std::process::exit(1);
        }
    }

    let today = brief_today_date();
    let seg_id = name.clone();

    // Build TOML
    let mut toml = String::new();
    toml.push_str("[segment]\n");
    toml.push_str(&format!("id = \"{}\"\n", seg_id));
    toml.push_str(&format!("created = \"{}\"\n", today));
    toml.push_str("status = \"active\"\n\n");
    toml.push_str("[scope]\n");
    toml.push_str("blocks = [\n");
    for b in &blocks {
        toml.push_str(&format!("  \"{}\",\n", b));
    }
    toml.push_str("]\n\n");
    toml.push_str("[owner]\n");
    toml.push_str("agent = \"\"\n");
    toml.push_str("task = \"\"\n");

    let _ = fs::create_dir_all(".cronus/segments");
    let path = format!(".cronus/segments/SEG-{}.toml", name);
    match fs::write(&path, &toml) {
        Ok(_) => {
            println!("  \x1b[32m✓ Created SEG-{}\x1b[0m [active]", name);
            println!("  Blocks: {}", blocks.join(", "));
            println!("  File: {}", path);
        }
        Err(e) => {
            eprintln!("  \x1b[31mError creating segment: {}\x1b[0m", e);
            std::process::exit(1);
        }
    }
}

/// cronus segment list — list all segments with status and blocks
fn segment_list() {
    let dir = ".cronus/segments";
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => {
            println!("  \x1b[90mNo segments found.\x1b[0m");
            return;
        }
    };

    let mut files: Vec<_> = entries
        .flatten()
        .filter(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            n.starts_with("SEG-") && n.ends_with(".toml")
        })
        .collect();
    files.sort_by_key(|e| e.file_name());

    if files.is_empty() {
        println!("  \x1b[90mNo segments found.\x1b[0m");
        return;
    }

    println!("  \x1b[1mSegments:\x1b[0m");
    for entry in &files {
        let content = fs::read_to_string(entry.path()).unwrap_or_default();
        let fname = entry.file_name().to_string_lossy().to_string();
        let seg_name = fname.trim_start_matches("SEG-").trim_end_matches(".toml");
        let status = brief_toml_val(&content, "status").unwrap_or_else(|| "unknown".into());
        let blocks = brief_toml_arr(&content, "blocks");

        let status_color = match status.as_str() {
            "active" => "\x1b[32m",
            "merged" => "\x1b[36m",
            "abandoned" => "\x1b[90m",
            _ => "\x1b[33m",
        };

        println!(
            "    SEG-{} {}[{}]\x1b[0m — {}",
            seg_name,
            status_color,
            status,
            if blocks.is_empty() {
                "(no blocks)".to_string()
            } else {
                blocks.join(", ")
            }
        );
    }
}

/// cronus segment show "name" — show details of one segment
fn segment_show(args: &[String]) {
    let name = match args.get(3) {
        Some(n) => n.clone(),
        None => {
            eprintln!("  \x1b[31mMissing segment name.\x1b[0m");
            eprintln!("  Usage: cronus segment show \"name\"");
            std::process::exit(1);
        }
    };

    let path = format!(".cronus/segments/SEG-{}.toml", name);
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("  \x1b[31mSegment not found: SEG-{}\x1b[0m", name);
            std::process::exit(1);
        }
    };

    let status = brief_toml_val(&content, "status").unwrap_or_else(|| "unknown".into());
    let created = brief_toml_val(&content, "created").unwrap_or_else(|| "unknown".into());
    let blocks = brief_toml_arr(&content, "blocks");
    let agent = brief_toml_val(&content, "agent").unwrap_or_default();
    let task = brief_toml_val(&content, "task").unwrap_or_default();

    let status_color = match status.as_str() {
        "active" => "\x1b[32m",
        "merged" => "\x1b[36m",
        "abandoned" => "\x1b[90m",
        _ => "\x1b[33m",
    };

    println!("  \x1b[1mSEG-{}\x1b[0m", name);
    println!("  Status:  {}[{}]\x1b[0m", status_color, status);
    println!("  Created: {}", created);
    println!("  Blocks:");
    if blocks.is_empty() {
        println!("    \x1b[90m(none)\x1b[0m");
    } else {
        for b in &blocks {
            println!("    - {}", b);
        }
    }
    println!("  Owner:");
    println!(
        "    Agent: {}",
        if agent.is_empty() {
            "\x1b[90m(unassigned)\x1b[0m"
        } else {
            &agent
        }
    );
    println!(
        "    Task:  {}",
        if task.is_empty() {
            "\x1b[90m(none)\x1b[0m"
        } else {
            &task
        }
    );
}

/// cronus segment check — validate no two active segments claim the same block
fn segment_check() {
    let dir = ".cronus/segments";
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => {
            println!("  \x1b[90mNo segments found. Nothing to check.\x1b[0m");
            return;
        }
    };

    let mut files: Vec<_> = entries
        .flatten()
        .filter(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            n.starts_with("SEG-") && n.ends_with(".toml")
        })
        .collect();
    files.sort_by_key(|e| e.file_name());

    // Map block -> list of segment names that claim it (active only)
    let mut block_owners: HashMap<String, Vec<String>> = HashMap::new();

    for entry in &files {
        let content = fs::read_to_string(entry.path()).unwrap_or_default();
        let status = brief_toml_val(&content, "status").unwrap_or_default();
        if status != "active" {
            continue;
        }
        let fname = entry.file_name().to_string_lossy().to_string();
        let seg_name = format!(
            "SEG-{}",
            fname.trim_start_matches("SEG-").trim_end_matches(".toml")
        );
        let blocks = brief_toml_arr(&content, "blocks");
        for b in blocks {
            block_owners.entry(b).or_default().push(seg_name.clone());
        }
    }

    let mut conflicts = 0;
    for (block, owners) in &block_owners {
        if owners.len() > 1 {
            if conflicts == 0 {
                println!("  \x1b[31mSegment conflict detected:\x1b[0m");
            }
            println!(
                "    \x1b[33m{}\x1b[0m claimed by {}",
                block,
                owners.join(" AND ")
            );
            conflicts += 1;
        }
    }

    if conflicts == 0 {
        println!("  \x1b[32m✓ No segment conflicts.\x1b[0m All blocks are uniquely owned.");
    } else {
        std::process::exit(1);
    }
}
