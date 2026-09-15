//! CRONUS Test Runner — Auto-generated CRUD tests
//!
//! `cronus test` reads .cronus, generates and runs HTTP tests against
//! a running server. Tests per entity:
//!   - POST create → 201
//!   - GET list → assert count > 0
//!   - GET by id → 200
//!   - DELETE → 200
//!   - GET deleted → 404

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Instant;

use crate::parser::{EntityNode, FieldType};

// ══════════════════════════════════════════════════
// PUBLIC API
// ══════════════════════════════════════════════════

/// Run all auto-generated tests against a live server.
/// Returns (passed, failed, total).
pub fn run_tests(entities: &[EntityNode], port: u16) -> (usize, usize, usize) {
    let addr = format!("127.0.0.1:{}", port);

    println!();
    println!("  \x1b[36m\x1b[1mCRONUS\x1b[0m \x1b[90m// Test Runner\x1b[0m");
    println!("  \x1b[90mTarget: http://{}\x1b[0m", addr);
    println!();

    // Check server is reachable
    if TcpStream::connect(&addr).is_err() {
        println!("  \x1b[31m✗\x1b[0m Server not reachable at {}", addr);
        println!(
            "  \x1b[90mStart the server first: cronus run {}\x1b[0m",
            port
        );
        println!();
        return (0, 1, 1);
    }

    let mut passed = 0usize;
    let mut failed = 0usize;

    for entity in entities {
        println!("  \x1b[90m── {} ──\x1b[0m", entity.name);
        let lower = entity.name.to_lowercase();

        // 1. POST create
        let body = generate_test_body(entity);
        let (status, response_body, ms) = http_post(&addr, &format!("/api/{}", lower), &body);
        if status >= 200 && status < 300 {
            println!(
                "  \x1b[32m✓\x1b[0m {}.create \x1b[90m... {} ({}ms)\x1b[0m",
                entity.name, status, ms
            );
            passed += 1;
        } else {
            println!(
                "  \x1b[31m✗\x1b[0m {}.create \x1b[90m... expected 2xx got {} ({}ms)\x1b[0m",
                entity.name, status, ms
            );
            failed += 1;
        }

        // Extract ID from response
        let created_id = extract_id(&response_body);

        // 2. GET list
        let (status, list_body, ms) = http_get(&addr, &format!("/api/{}", lower));
        let count = count_array_items(&list_body);
        if status == 200 && count > 0 {
            println!(
                "  \x1b[32m✓\x1b[0m {}.list \x1b[90m... {} ({} items, {}ms)\x1b[0m",
                entity.name, status, count, ms
            );
            passed += 1;
        } else {
            println!(
                "  \x1b[31m✗\x1b[0m {}.list \x1b[90m... {} ({} items, {}ms)\x1b[0m",
                entity.name, status, count, ms
            );
            failed += 1;
        }

        // 3. GET by id (if we got an id from create)
        if let Some(ref id) = created_id {
            let (status, _, ms) = http_get(&addr, &format!("/api/{}/{}", lower, id));
            if status == 200 {
                println!(
                    "  \x1b[32m✓\x1b[0m {}.get_by_id \x1b[90m... {} ({}ms)\x1b[0m",
                    entity.name, status, ms
                );
                passed += 1;
            } else {
                println!(
                    "  \x1b[31m✗\x1b[0m {}.get_by_id \x1b[90m... expected 200 got {} ({}ms)\x1b[0m",
                    entity.name, status, ms
                );
                failed += 1;
            }

            // 4. DELETE
            let (status, _, ms) = http_delete(&addr, &format!("/api/{}/{}", lower, id));
            if status == 200 || status == 204 {
                println!(
                    "  \x1b[32m✓\x1b[0m {}.delete \x1b[90m... {} ({}ms)\x1b[0m",
                    entity.name, status, ms
                );
                passed += 1;
            } else {
                println!("  \x1b[31m✗\x1b[0m {}.delete \x1b[90m... expected 200/204 got {} ({}ms)\x1b[0m", entity.name, status, ms);
                failed += 1;
            }

            // 5. GET deleted → 404
            let (status, _, ms) = http_get(&addr, &format!("/api/{}/{}", lower, id));
            if status == 404 {
                println!(
                    "  \x1b[32m✓\x1b[0m {}.get_deleted \x1b[90m... {} ({}ms)\x1b[0m",
                    entity.name, status, ms
                );
                passed += 1;
            } else {
                println!("  \x1b[31m✗\x1b[0m {}.get_deleted \x1b[90m... expected 404 got {} ({}ms)\x1b[0m", entity.name, status, ms);
                failed += 1;
            }
        } else {
            println!(
                "  \x1b[33m⊘\x1b[0m {}.get_by_id \x1b[90m... skipped (no id from create)\x1b[0m",
                entity.name
            );
            println!(
                "  \x1b[33m⊘\x1b[0m {}.delete \x1b[90m... skipped\x1b[0m",
                entity.name
            );
            println!(
                "  \x1b[33m⊘\x1b[0m {}.get_deleted \x1b[90m... skipped\x1b[0m",
                entity.name
            );
        }

        println!();
    }

    let total = passed + failed;
    let color = if failed == 0 { "\x1b[32m" } else { "\x1b[31m" };
    println!(
        "  {}Tests: {} passed, {} failed, {} total\x1b[0m",
        color, passed, failed, total
    );
    println!();

    (passed, failed, total)
}

// ══════════════════════════════════════════════════
// TEST DATA GENERATION
// ══════════════════════════════════════════════════

fn generate_test_body(entity: &EntityNode) -> String {
    let mut fields = Vec::new();

    for field in &entity.fields {
        if field.name == "id" {
            continue;
        }
        let val = match field.field_type {
            FieldType::String | FieldType::Text => {
                if field.name == "name" || field.name == "title" {
                    format!("\"{}\":\"Test {}\"", field.name, entity.name)
                } else if field.name == "slug" {
                    format!("\"{}\":\"test-{}\"", field.name, entity.name.to_lowercase())
                } else if field.name == "password" {
                    format!("\"{}\":\"testpassword123\"", field.name)
                } else {
                    format!("\"{}\":\"test-value\"", field.name)
                }
            }
            FieldType::Email => {
                format!(
                    "\"{}\":\"test-{}@cronus.test\"",
                    field.name,
                    entity.name.to_lowercase()
                )
            }
            FieldType::Number => format!("\"{}\":42", field.name),
            FieldType::Money => format!("\"{}\":2990", field.name),
            FieldType::Percentage => format!("\"{}\":50", field.name),
            FieldType::Boolean => format!("\"{}\":true", field.name),
            FieldType::Date => format!("\"{}\":\"2026-03-29\"", field.name),
            FieldType::DateTime => format!("\"{}\":\"2026-03-29T10:00:00\"", field.name),
            FieldType::File => format!("\"{}\":\"https://test.cronus.dev/file.bin\"", field.name),
            FieldType::Url => format!("\"{}\":\"https://test.cronus.dev\"", field.name),
            FieldType::Enum => {
                if let Some(ref values) = field.enum_values {
                    format!(
                        "\"{}\":\"{}\"",
                        field.name,
                        values.first().unwrap_or(&"default".to_string())
                    )
                } else {
                    format!("\"{}\":\"default\"", field.name)
                }
            }
            FieldType::Relation => continue,
            _ => format!("\"{}\":\"test\"", field.name),
        };
        fields.push(val);
    }

    format!("{{{}}}", fields.join(","))
}

// ══════════════════════════════════════════════════
// HTTP HELPERS (raw TCP, no dependencies)
// ══════════════════════════════════════════════════

fn http_get(addr: &str, path: &str) -> (u16, String, u64) {
    let raw = format!(
        "GET {} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        path
    );
    http_raw(addr, &raw)
}

fn http_post(addr: &str, path: &str, body: &str) -> (u16, String, u64) {
    let raw = format!(
        "POST {} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        path,
        body.len(),
        body
    );
    http_raw(addr, &raw)
}

fn http_delete(addr: &str, path: &str) -> (u16, String, u64) {
    let raw = format!(
        "DELETE {} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        path
    );
    http_raw(addr, &raw)
}

fn http_raw(addr: &str, raw: &str) -> (u16, String, u64) {
    let start = Instant::now();

    let mut stream = match TcpStream::connect(addr) {
        Ok(s) => s,
        Err(_) => return (0, String::new(), 0),
    };

    // Set timeout
    let timeout = std::time::Duration::from_secs(5);
    stream.set_read_timeout(Some(timeout)).ok();
    stream.set_write_timeout(Some(timeout)).ok();

    if stream.write_all(raw.as_bytes()).is_err() {
        return (0, String::new(), 0);
    }

    // Shutdown write side to signal end of request
    let _ = stream.shutdown(std::net::Shutdown::Write);

    let mut response = String::new();
    let _ = stream.read_to_string(&mut response);

    let ms = start.elapsed().as_millis() as u64;
    let status = parse_status(&response);
    let body = extract_body(&response);

    (status, body, ms)
}

fn parse_status(response: &str) -> u16 {
    // HTTP/1.1 200 OK
    response
        .lines()
        .next()
        .and_then(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            parts.get(1).and_then(|s| s.parse().ok())
        })
        .unwrap_or(0)
}

fn extract_body(response: &str) -> String {
    // Body is after \r\n\r\n
    if let Some(idx) = response.find("\r\n\r\n") {
        response[idx + 4..].to_string()
    } else {
        String::new()
    }
}

fn extract_id(body: &str) -> Option<String> {
    // Simple: find "id":"<value>" in response JSON
    let needle = "\"id\":\"";
    if let Some(start) = body.find(needle) {
        let rest = &body[start + needle.len()..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    // Also try "id": " (with space)
    let needle2 = "\"id\": \"";
    if let Some(start) = body.find(needle2) {
        let rest = &body[start + needle2.len()..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

fn count_array_items(body: &str) -> usize {
    // Simple count: number of {"id": patterns
    body.matches("\"id\"").count()
}

// ══════════════════════════════════════════════════
// CONFORMANCE TEST RUNNER
// ══════════════════════════════════════════════════

/// Run conformance tests against parse-positive/, parse-negative/, and warnings/ dirs.
/// Returns (passed, failed, error_messages).
pub fn run_conformance(base_dir: &str) -> (usize, usize, Vec<String>) {
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut errors: Vec<String> = Vec::new();

    // 1. parse-positive/: every .cronus file must parse without error
    let pos_dir = format!("{}/parse-positive", base_dir);
    if let Ok(entries) = std::fs::read_dir(&pos_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "cronus").unwrap_or(false) {
                let source = std::fs::read_to_string(&path).unwrap_or_default();
                match crate::parser::parse(&source) {
                    Ok(_) => {
                        passed += 1;
                    }
                    Err(e) => {
                        failed += 1;
                        errors.push(format!("FAIL [parse-positive] {}: {}", path.display(), e));
                    }
                }
            }
        }
    }

    // 2. parse-negative/: every .cronus file must FAIL to parse
    let neg_dir = format!("{}/parse-negative", base_dir);
    if let Ok(entries) = std::fs::read_dir(&neg_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "cronus").unwrap_or(false) {
                let source = std::fs::read_to_string(&path).unwrap_or_default();
                match crate::parser::parse(&source) {
                    Ok(_) => {
                        failed += 1;
                        errors.push(format!(
                            "FAIL [parse-negative] {}: expected parse error, but succeeded",
                            path.display()
                        ));
                    }
                    Err(_) => {
                        passed += 1;
                    }
                }
            }
        }
    }

    // 3. warnings/: every .cronus file must parse OK, then validate_section should produce warnings
    let warn_dir = format!("{}/warnings", base_dir);
    if let Ok(entries) = std::fs::read_dir(&warn_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "cronus").unwrap_or(false) {
                let source = std::fs::read_to_string(&path).unwrap_or_default();
                let expected = source
                    .lines()
                    .find(|l| l.starts_with("# EXPECTED WARNING:"))
                    .map(|l| {
                        l.trim_start_matches("# EXPECTED WARNING:")
                            .trim()
                            .to_string()
                    });

                match crate::parser::parse(&source) {
                    Ok(nodes) => {
                        let mut found_warning = false;
                        for node in &nodes {
                            if let crate::parser::AstNode::Page(page) = node {
                                for section in &page.sections {
                                    let warnings = crate::contracts::validate_section(section, &[]);
                                    if !warnings.is_empty() {
                                        found_warning = true;
                                    }
                                }
                            }
                        }
                        if found_warning {
                            passed += 1;
                        } else if expected.is_some() {
                            failed += 1;
                            errors.push(format!(
                                "FAIL [warnings] {}: expected warning but got none",
                                path.display()
                            ));
                        } else {
                            passed += 1;
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        errors.push(format!(
                            "FAIL [warnings] {}: parse error: {}",
                            path.display(),
                            e
                        ));
                    }
                }
            }
        }
    }

    (passed, failed, errors)
}
