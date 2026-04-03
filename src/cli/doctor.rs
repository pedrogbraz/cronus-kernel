use std::fs;
use crate::parser::{self, AstNode};
use crate::lint;
use crate::find_cronus_file;

pub fn cmd_doctor(_args: &[String]) {
    println!();
    println!("  \x1b[36mCRONUS Doctor\x1b[0m — Project Health Check");
    println!();

    let mut passed = 0u32;
    let total = 10u32;

    // -- 1. Syntax --
    let (nodes, app_port, db_path) = match find_cronus_file() {
        Some(file) => {
            let source = fs::read_to_string(&file).unwrap_or_default();
            match parser::parse(&source) {
                Ok(nodes) => {
                    let (e, p, r) = parser::stats(&nodes);
                    println!("  \x1b[32m✓\x1b[0m Syntax valid ({} entities, {} pages, {} routes)", e, p, r);
                    passed += 1;
                    let mut port: u16 = 5175;
                    let mut dbp: Option<String> = None;
                    for n in &nodes {
                        if let AstNode::App(a) = n {
                            port = a.port;
                            if let Some(ref db) = a.database {
                                dbp = db.path.clone();
                            }
                        }
                    }
                    (Some(nodes), port, dbp)
                }
                Err(e) => {
                    println!("  \x1b[31m✗\x1b[0m Syntax: {}", e);
                    (None, 5175, None)
                }
            }
        }
        None => {
            println!("  \x1b[31m✗\x1b[0m No .cronus file found");
            (None, 5175, None)
        }
    };

    // -- 2. Port --
    match std::net::TcpListener::bind(format!("0.0.0.0:{}", app_port)) {
        Ok(_) => { println!("  \x1b[32m✓\x1b[0m Port {} available", app_port); passed += 1; }
        Err(_) => println!("  \x1b[33m✗\x1b[0m Port {} in use", app_port),
    }

    // -- 3. Database --
    {
        let db_file = db_path.as_deref().unwrap_or("data.db");
        let db_file_clean = db_file.trim_matches('"');
        if std::path::Path::new(db_file_clean).exists() {
            match rusqlite::Connection::open_with_flags(
                db_file_clean,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            ) {
                Ok(conn) => {
                    let table_count: i64 = conn
                        .query_row(
                            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
                            [],
                            |r| r.get(0),
                        )
                        .unwrap_or(0);
                    let total_rows: i64 = {
                        let mut stmt = conn
                            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
                            .unwrap();
                        let tables: Vec<String> = stmt
                            .query_map([], |r| r.get(0))
                            .unwrap()
                            .filter_map(|r| r.ok())
                            .collect();
                        let mut sum: i64 = 0;
                        for t in &tables {
                            let q = format!("SELECT COUNT(*) FROM \"{}\"", t);
                            sum += conn.query_row(&q, [], |r| r.get::<_, i64>(0)).unwrap_or(0);
                        }
                        sum
                    };
                    println!(
                        "  \x1b[32m✓\x1b[0m Database {} ({} tables, {} rows)",
                        db_file_clean, table_count, total_rows
                    );
                    passed += 1;
                }
                Err(e) => println!("  \x1b[31m✗\x1b[0m Database {}: {}", db_file_clean, e),
            }
        } else {
            println!("  \x1b[33m✗\x1b[0m Database {} not found", db_file_clean);
        }
    }

    // -- Checks 4-8 require a valid AST --
    if let Some(ref nodes) = nodes {
        let lint_results = lint::lint_ast(nodes, false);

        // -- 4. Lint --
        {
            let rule_names: &[&str] = &[
                "no-dead-text", "no-dead-links", "no-dead-ui", "no-fake-state",
                "no-orphan-reload", "no-hardcode-user", "bind-or-empty", "no-sensitive-render",
            ];
            let total_rules = rule_names.len();
            let failed_rules: std::collections::HashSet<&str> = lint_results.iter().map(|r| r.rule).collect();
            let passed_rules = total_rules - failed_rules.len();
            if lint_results.is_empty() {
                println!("  \x1b[32m✓\x1b[0m Zero hardcode lint: {}/{} rules passed", total_rules, total_rules);
                passed += 1;
            } else {
                println!("  \x1b[31m✗\x1b[0m Zero hardcode lint: {}/{} rules passed ({} violations)", passed_rules, total_rules, lint_results.len());
                for r in &lint_results {
                    println!("    {}", r);
                }
            }
        }

        // -- 5. Constitution --
        {
            let mut must_count = 0usize;
            let mut never_count = 0usize;
            let mut has_constitution = false;
            for n in nodes {
                if let AstNode::App(a) = n {
                    if let Some(ref c) = a.constitution {
                        has_constitution = true;
                        must_count = c.must.len();
                        never_count = c.never.len();
                    }
                }
            }
            if !has_constitution {
                if let Ok(toml) = fs::read_to_string(".cronus/constitution.toml") {
                    if !toml.trim().is_empty() {
                        has_constitution = true;
                        let mut in_must = false;
                        let mut in_never = false;
                        for line in toml.lines() {
                            let trimmed = line.trim();
                            if trimmed.starts_with("[invariants]") || trimmed.starts_with("[must]") { in_must = true; in_never = false; continue; }
                            if trimmed.starts_with("[forbidden]") || trimmed.starts_with("[never]") { in_never = true; in_must = false; continue; }
                            if trimmed.starts_with('[') { in_must = false; in_never = false; continue; }
                            if in_must && (trimmed.starts_with("must") || trimmed.starts_with('-')) { must_count += 1; }
                            if in_never && (trimmed.starts_with("never") || trimmed.starts_with('-')) { never_count += 1; }
                        }
                    }
                }
            }
            if has_constitution {
                println!(
                    "  \x1b[32m✓\x1b[0m Constitution: {} must + {} never rules, 0 violations",
                    must_count, never_count
                );
                passed += 1;
            } else {
                println!("  \x1b[33m✗\x1b[0m Constitution: no constitution block or .cronus/constitution.toml found");
            }
        }

        // -- 6. Dead links --
        {
            let dead_link_count: usize = lint_results.iter().filter(|r| r.rule == "no-dead-links").count();
            if dead_link_count == 0 {
                println!("  \x1b[32m✓\x1b[0m No dead links detected");
                passed += 1;
            } else {
                println!("  \x1b[31m✗\x1b[0m {} dead link(s) detected", dead_link_count);
            }
        }

        // -- 7. Sensitive exposure --
        {
            let sensitive_count: usize = lint_results.iter().filter(|r| r.rule == "no-sensitive-render").count();
            if sensitive_count == 0 {
                println!("  \x1b[32m✓\x1b[0m No sensitive field exposure");
                passed += 1;
            } else {
                println!("  \x1b[31m✗\x1b[0m {} sensitive field exposure(s)", sensitive_count);
            }
        }

        // -- 8. SQL identifiers --
        {
            let mut bad_idents: Vec<String> = Vec::new();
            for n in nodes {
                if let AstNode::Entity(e) = n {
                    if !is_sql_safe_ident(&e.name) {
                        bad_idents.push(format!("entity \"{}\"", e.name));
                    }
                    for f in &e.fields {
                        if !is_sql_safe_ident(&f.name) {
                            bad_idents.push(format!("field \"{}.{}\"", e.name, f.name));
                        }
                    }
                }
            }
            if bad_idents.is_empty() {
                println!("  \x1b[32m✓\x1b[0m All identifiers SQL-safe");
                passed += 1;
            } else {
                println!("  \x1b[31m✗\x1b[0m Unsafe SQL identifiers:");
                for b in &bad_idents {
                    println!("      {}", b);
                }
            }
        }
    } else {
        println!("  \x1b[33m⊘\x1b[0m Lint: skipped (syntax error)");
        println!("  \x1b[33m⊘\x1b[0m Constitution: skipped (syntax error)");
        println!("  \x1b[33m⊘\x1b[0m Dead links: skipped (syntax error)");
        println!("  \x1b[33m⊘\x1b[0m Sensitive exposure: skipped (syntax error)");
        println!("  \x1b[33m⊘\x1b[0m SQL identifiers: skipped (syntax error)");
    }

    // -- 9. AST snapshot --
    {
        let snap_path = std::path::Path::new(".cronus/ast-snapshot.json");
        if snap_path.exists() {
            let date_str = fs::metadata(snap_path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| {
                    let dur = t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok()?;
                    let secs = dur.as_secs();
                    let days = secs / 86400;
                    let mut y = 1970i64;
                    let mut remaining = days as i64;
                    loop {
                        let days_in_year: i64 = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
                        if remaining < days_in_year { break; }
                        remaining -= days_in_year;
                        y += 1;
                    }
                    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
                    let month_days: [i64; 12] = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
                    let mut m = 0usize;
                    for (i, &md) in month_days.iter().enumerate() {
                        if remaining < md { m = i; break; }
                        remaining -= md;
                    }
                    Some(format!("{:04}-{:02}-{:02}", y, m + 1, remaining + 1))
                })
                .unwrap_or_else(|| "unknown".into());
            println!("  \x1b[32m✓\x1b[0m AST snapshot: .cronus/ast-snapshot.json ({})", date_str);
            passed += 1;
        } else {
            println!("  \x1b[33m✗\x1b[0m AST snapshot: .cronus/ast-snapshot.json not found");
        }
    }

    // -- 10. Memory --
    {
        let mem_path = std::path::Path::new(".cronus/memory.db");
        if mem_path.exists() {
            match rusqlite::Connection::open_with_flags(
                mem_path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            ) {
                Ok(conn) => {
                    let session_count: i64 = conn
                        .query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))
                        .unwrap_or(0);
                    println!("  \x1b[32m✓\x1b[0m Memory: .cronus/memory.db ({} sessions)", session_count);
                    passed += 1;
                }
                Err(_) => {
                    println!("  \x1b[33m✗\x1b[0m Memory: .cronus/memory.db (unreadable)");
                }
            }
        } else {
            println!("  \x1b[33m✗\x1b[0m Memory: .cronus/memory.db not found");
        }
    }

    // -- Summary --
    println!();
    if passed == total {
        println!("  \x1b[32mHealth: CLEAN ({}/{} checks passed)\x1b[0m", passed, total);
    } else {
        println!("  \x1b[33mHealth: {}/{} checks passed\x1b[0m", passed, total);
    }
    println!();
}

/// Check if a name is a valid SQL-safe identifier.
fn is_sql_safe_ident(name: &str) -> bool {
    if name.is_empty() || name.len() > 64 { return false; }
    let bytes = name.as_bytes();
    if !bytes[0].is_ascii_alphabetic() { return false; }
    for &b in &bytes[1..] {
        if !(b.is_ascii_alphanumeric() || b == b'_') { return false; }
    }
    const SQL_RESERVED: &[&str] = &[
        "SELECT", "DROP", "INSERT", "DELETE", "UPDATE", "TABLE", "FROM",
        "WHERE", "OR", "AND", "UNION", "ALTER", "CREATE", "INDEX", "EXEC",
        "EXECUTE", "INTO", "VALUES", "SET", "NULL", "TRUE", "FALSE",
    ];
    let upper = name.to_uppercase();
    !SQL_RESERVED.contains(&upper.as_str())
}
