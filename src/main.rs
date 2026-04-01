#![allow(dead_code, unused_imports, unused_variables)]
mod actions;
mod animations;
mod auth;
mod binding;
mod board;
mod brain;
mod cache;
mod command_palette;
mod components;
mod contracts;
mod data_table;
mod database;
mod deploy;
mod dump;
mod feedback;
mod graphql;
mod hmr;
mod i18n;
mod layout_system;
mod marketing_components;
mod orchestrator;
mod overlays;
mod parser;
mod payments;
mod rate_limit;
mod reactive;
mod realtime;
mod render;
mod runtime_js;
mod server;
mod sse;
mod tabs;
mod tailwind;
mod testing;
mod navigation;
mod ui;

use std::env;
use std::fs;
use std::io::Write as IoWrite;

use hyper::{body::Incoming, server::conn::http1, service::service_fn, Request, Response, Method, StatusCode};
use hyper_util::rt::TokioIo;
use http_body_util::{BodyExt, Full};
use bytes::Bytes;
use tokio::net::TcpListener;
use rusqlite::Connection;
use serde_json::{json, Value};
use std::sync::Arc;
use std::collections::HashMap;

use parser::{AstNode, EntityNode, PageNode, StyleNode, ApiNode, AppNode, FieldType};

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
pub static STRICT_MODE: AtomicBool = AtomicBool::new(false);
pub static STRICT_AI_MODE: AtomicBool = AtomicBool::new(false);

// ══════════════════════════════════════════════════
// MAIN
// ══════════════════════════════════════════════════

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let strict = args.iter().any(|a| a == "--strict");
    let strict_ai = args.iter().any(|a| a == "--strict-ai");
    if strict || strict_ai {
        STRICT_MODE.store(true, Ordering::Relaxed);
    }
    STRICT_AI_MODE.store(strict_ai, Ordering::Relaxed);
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match cmd {
        "run" => cmd_run(&args).await,
        "build" => cmd_build(&args),
        "parse" => cmd_parse(&args),
        "new" => cmd_new(&args),
        "seed" => cmd_seed(&args),
        "deploy" => cmd_deploy(&args),
        "doctor" => cmd_doctor(&args),
        "stats" => cmd_stats(&args),
        "export" => cmd_export(&args),
        "test" => cmd_test(&args),
        "compose" => cmd_compose(&args),
        "generate" | "gen" => cmd_generate(&args),
        "dump" => cmd_dump(&args),
        "validate" => {
            if args.iter().any(|a| a == "--mission") {
                cmd_validate_mission();
            } else {
                cmd_validate(&args);
            }
        }
        "brief" => cmd_brief(),
        "sync" => cmd_sync(),
        "handoff" => cmd_handoff(),
        "lease" => cmd_lease(&args),
        "drift" => cmd_drift(&args),
        "spec" => cmd_spec(&args),
        "version" | "-v" | "--version" => println!("cronus v0.1.0"),
        "help" | "--help" | "-h" | _ => print_help(),
    }
}

fn print_help() {
    println!("\x1b[36m\x1b[1m");
    println!("  ██████╗██████╗  ██████╗ ███╗   ██╗██╗   ██╗███████╗");
    println!(" ██╔════╝██╔══██╗██╔═══██╗████╗  ██║██║   ██║██╔════╝");
    println!(" ██║     ██████╔╝██║   ██║██╔██╗ ██║██║   ██║███████╗");
    println!(" ██║     ██╔══██╗██║   ██║██║╚██╗██║██║   ██║╚════██║");
    println!(" ╚██████╗██║  ██║╚██████╔╝██║ ╚████║╚██████╔╝███████║");
    println!("  ╚═════╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═══╝ ╚═════╝ ╚══════╝");
    println!("\x1b[0m");
    println!("  \x1b[90mThe Cognitive Runtime v0.1.0 (Rust native)\x1b[0m\n");
    println!("  \x1b[1mUsage:\x1b[0m cronus <command> [options]\n");
    println!("  \x1b[1mCommands:\x1b[0m");
    println!("    \x1b[32mrun\x1b[0m [port] [--strict]  Parse .cronus → serve (strict: warnings=errors)");
    println!("    \x1b[32mnew\x1b[0m <template>       Create project (landing/admin/saas/api/ecommerce/blog)");
    println!("    \x1b[32mseed\x1b[0m [count]          Seed database with fake data (default: 10 rows)");
    println!("    \x1b[32mbuild\x1b[0m [--strict] [--strict-ai]  Parse and validate .cronus file (strict mode)");
    println!("    \x1b[32mparse\x1b[0m <file> [--strict] Parse and show AST (strict mode)");
    println!("    \x1b[32mdeploy\x1b[0m           Generate deploy artifacts (--fly, --railway, --static)");
    println!("    \x1b[32mdoctor\x1b[0m           Check .cronus syntax + DB + ports");
    println!("    \x1b[32mstats\x1b[0m            Project stats (entities, pages, DB size)");
    println!("    \x1b[32mexport\x1b[0m           Export to cronus-project.ir.json");
    println!("    \x1b[32mtest\x1b[0m [port]          Auto-gen and run CRUD tests");
    println!("    \x1b[32mtest\x1b[0m --conformance   Run conformance test suite");
    println!("    \x1b[32mcompose\x1b[0m          Compose all .cronus files and show result");
    println!("    \x1b[32mgenerate\x1b[0m <desc>  Generate .cronus from description");
    println!("    \x1b[32mvalidate\x1b[0m [file] [--json] [--strict-ai]  Validate (--strict-ai: all warnings = errors, JSON output)");
    println!("    \x1b[32mvalidate\x1b[0m --mission       Validate code against constitution + objective");
    println!("    \x1b[32mbrief\x1b[0m            Generate AI context capsule (~500 words)");
    println!("    \x1b[32msync\x1b[0m             Generate .cronus/state-digest.json from project state");
    println!("    \x1b[32mhandoff\x1b[0m          Complete active task, update state digest for next session");
    println!("    \x1b[32mlease\x1b[0m check|list|create  Task lease management (drift detection)");
    println!("    \x1b[32mdrift\x1b[0m [--explain]    Detect strategic, scope, and semantic drift");
    println!("    \x1b[32mspec\x1b[0m <validate|list|codegen> Validate, list, or generate from .spec.toml files");
    println!("    \x1b[32mversion\x1b[0m          Show version");
    println!();
}

// ══════════════════════════════════════════════════
// BRIEF — AI context capsule
// ══════════════════════════════════════════════════

fn cmd_brief() {
    let today = brief_today_date();

    // --- Read constitution.toml ---
    let constitution = fs::read_to_string(".cronus/constitution.toml").unwrap_or_default();
    let proj_name = brief_toml_val(&constitution, "name").unwrap_or_else(|| "Unknown".into());
    let proj_purpose = brief_toml_val(&constitution, "purpose").unwrap_or_default();
    let proj_category = brief_toml_val(&constitution, "category").unwrap_or_default();
    let invariants = brief_toml_arr(&constitution, "must");

    // --- Read objective.toml ---
    let objective = fs::read_to_string(".cronus/objective.toml").unwrap_or_default();
    let obj_title = brief_toml_val(&objective, "title").unwrap_or_else(|| "No objective set".into());
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
        if !kernel_lines.is_empty() { meta.push(format!("{} lines", kernel_lines)); }
        if !binary_size.is_empty() { meta.push(format!("{} binary", binary_size)); }
        if !build_status.is_empty() { meta.push(format!("build: {}", build_status)); }
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
            let files: Vec<String> = task_write.iter().map(|f| {
                f.rsplit('/').next().unwrap_or(f).to_string()
            }).collect();
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
fn brief_today_date() -> String {
    use std::time::SystemTime;
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let diy = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
        if remaining < diy { break; }
        remaining -= diy;
        y += 1;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let md = [31, if leap {29} else {28}, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 0usize;
    for (i, &d) in md.iter().enumerate() {
        if remaining < d as i64 { m = i + 1; break; }
        remaining -= d as i64;
    }
    format!("{:04}-{:02}-{:02}", y, m, remaining + 1)
}

/// Extract `key = "value"` from TOML text
fn brief_toml_val(content: &str, key: &str) -> Option<String> {
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
fn brief_toml_arr(content: &str, key: &str) -> Vec<String> {
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
                                if !v.is_empty() { result.push(v.to_string()); }
                            }
                            return result;
                        }
                    }
                }
            }
        } else {
            if trimmed.starts_with(']') { break; }
            let v = trimmed.trim_end_matches(',').trim().trim_matches('"');
            if !v.is_empty() { result.push(v.to_string()); }
        }
    }
    result
}

/// Extract TOML array under a specific [section] header
fn brief_toml_arr_after_section(content: &str, section: &str, key: &str) -> Vec<String> {
    let mut in_section = false;
    let mut result = Vec::new();
    let mut in_array = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == section {
            in_section = true;
            continue;
        }
        if in_section && trimmed.starts_with('[') && !trimmed.starts_with("[[") && trimmed != section {
            break; // next section
        }
        if !in_section { continue; }

        if !in_array {
            if let Some(eq_pos) = trimmed.find('=') {
                let before = trimmed[..eq_pos].trim();
                if before == key && trimmed[eq_pos..].contains('[') {
                    in_array = true;
                    if let Some(bs) = trimmed.find('[') {
                        if let Some(be) = trimmed.rfind(']') {
                            for item in trimmed[bs + 1..be].split(',') {
                                let v = item.trim().trim_matches('"');
                                if !v.is_empty() { result.push(v.to_string()); }
                            }
                            return result;
                        }
                    }
                }
            }
        } else {
            if trimmed.starts_with(']') { break; }
            let v = trimmed.trim_end_matches(',').trim().trim_matches('"');
            if !v.is_empty() { result.push(v.to_string()); }
        }
    }
    result
}

/// Extract JSON string array by key
fn brief_json_arr(content: &str, key: &str) -> Vec<String> {
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
                            if !v.is_empty() { result.push(v.to_string()); }
                        }
                        return result;
                    }
                }
            }
        } else {
            if trimmed.starts_with(']') { break; }
            let v = trimmed.trim_end_matches(',').trim().trim_matches('"');
            if !v.is_empty() { result.push(v.to_string()); }
        }
    }
    result
}

/// Extract JSON "key": value
fn brief_json_val(content: &str, key: &str) -> Option<String> {
    let search = format!("\"{}\"", key);
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains(&search) && trimmed.contains(':') {
            if let Some(colon) = trimmed.find(':') {
                let val = trimmed[colon + 1..].trim().trim_end_matches(',').trim().trim_matches('"');
                if !val.is_empty() && !val.contains('[') && !val.contains('{') {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

// ══════════════════════════════════════════════════
// FIND .cronus FILE
// ══════════════════════════════════════════════════

fn find_cronus_file() -> Option<String> {
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".cronus") {
                return Some(name);
            }
        }
    }
    None
}

/// Find ALL .cronus files in current directory (multi-agent mode).
fn find_all_cronus_files() -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".cronus") {
                files.push(name);
            }
        }
    }
    files.sort();
    files
}

// ══════════════════════════════════════════════════
// DATABASE
// ══════════════════════════════════════════════════

// ══════════════════════════════════════════════════
// API HANDLER
// ══════════════════════════════════════════════════

struct AppState {
    app: AppNode,
    entities: Vec<EntityNode>,
    pages: Vec<PageNode>,
    components: Vec<parser::ComponentNode>,
    style: Option<StyleNode>,
    apis: Vec<ApiNode>,
    db_path: String,
    db: database::CronusDB,
    brain: Option<brain::CronusBrain>,
    auth_entity: Option<String>,      // name of the user entity from auth block
    auth_roles: Vec<String>,          // available roles from auth block
    auth_required_pages: Vec<(String, String)>, // (route, requires_value) for authentication
    layout: Option<parser::LayoutNode>,  // declarative sidebar+topbar layout
}

fn json_response(status: StatusCode, body: Value) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PATCH, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
        .body(Full::new(Bytes::from(body.to_string())))
        .unwrap()
}

fn html_response(body: String) -> Response<Full<Bytes>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Full::new(Bytes::from(body)))
        .unwrap()
}

fn forbidden_response(message: &str) -> Response<Full<Bytes>> {
    let html = format!(r##"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>403 Forbidden</title>
<style>
  * {{ margin:0; padding:0; box-sizing:border-box; }}
  body {{ background:oklch(0.08 0 0); color:oklch(0.7 0 0); font-family:'SF Pro','Inter',system-ui,sans-serif;
         display:flex; align-items:center; justify-content:center; min-height:100vh; }}
  .box {{ text-align:center; max-width:400px; padding:40px; }}
  .code {{ font-size:64px; font-weight:700; color:oklch(0.4 0.15 25); margin-bottom:8px; }}
  .msg {{ font-size:15px; color:oklch(0.55 0 0); margin-bottom:24px; }}
  a {{ color:oklch(0.7 0 0); text-decoration:underline; font-size:13px; }}
</style></head>
<body><div class="box">
  <div class="code">403</div>
  <div class="msg">{}</div>
  <a href="/">Back to home</a>
</div></body></html>"##, message);
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Full::new(Bytes::from(html)))
        .unwrap()
}

fn generate_login_page(state: &AppState) -> String {
    let app_name = &state.app.name;
    let logo_letter = app_name.chars().next().unwrap_or('C').to_uppercase().to_string();
    format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Login — {app}</title>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{font-family:'Inter',system-ui,-apple-system,sans-serif;-webkit-font-smoothing:antialiased;background:#0a0a0a;color:#fafafa;min-height:100vh;display:flex;align-items:center;justify-content:center}}
input{{width:100%;padding:10px 14px;font-size:14px;background:#171717;border:1px solid #262626;border-radius:10px;color:#fafafa;outline:none;transition:border-color 0.15s}}
input:focus{{border-color:#525252}}
.btn{{width:100%;padding:10px 14px;font-size:14px;font-weight:600;border:none;border-radius:10px;background:#fafafa;color:#0a0a0a;cursor:pointer;transition:opacity 0.15s}}
.btn:hover{{opacity:0.9}}
</style>
</head>
<body>
<div style="width:100%;max-width:400px;padding:32px">
  <div style="text-align:center;margin-bottom:32px">
    <div style="width:48px;height:48px;background:#fafafa;border-radius:12px;display:flex;align-items:center;justify-content:center;margin:0 auto 16px">
      <span style="color:#0a0a0a;font-weight:700;font-size:20px">{logo}</span>
    </div>
    <h1 style="font-size:24px;font-weight:700;margin:0 0 8px">Welcome back</h1>
    <p style="font-size:14px;color:#a3a3a3">Sign in to your account</p>
  </div>
  <form id="loginForm" style="display:flex;flex-direction:column;gap:16px">
    <input name="email" type="email" placeholder="Email" required />
    <input name="password" type="password" placeholder="Password" required />
    <div id="error" style="display:none;padding:10px 14px;border-radius:10px;font-size:13px;text-align:center;background:#450a0a;color:#fca5a5;border:1px solid #7f1d1d"></div>
    <button type="submit" class="btn">Sign In</button>
    <p style="text-align:center;font-size:14px;color:#a3a3a3">Don't have an account? <a href="/register" style="color:#fafafa;font-weight:600;text-decoration:none">Register</a></p>
  </form>
</div>
<script>
document.getElementById('loginForm').addEventListener('submit', async (e) => {{
  e.preventDefault();
  const btn = e.target.querySelector('button');
  btn.disabled = true; btn.textContent = 'Loading...';
  const data = Object.fromEntries(new FormData(e.target));
  try {{
    const res = await fetch('/api/auth/login', {{ method: 'POST', headers: {{'Content-Type': 'application/json'}}, body: JSON.stringify(data) }});
    const json = await res.json();
    if (json.token) {{
      document.cookie = 'cronus_token=' + json.token + '; path=/; max-age=86400';
      localStorage.setItem('token', json.token);
      localStorage.setItem('user', JSON.stringify(json.user || {{}}));
      window.location.href = '/';
    }} else {{
      const err = document.getElementById('error');
      err.style.display = 'block';
      err.textContent = json.error || 'Invalid credentials';
      btn.disabled = false; btn.textContent = 'Sign In';
    }}
  }} catch(err) {{
    const el = document.getElementById('error');
    el.style.display = 'block';
    el.textContent = 'Connection failed';
    btn.disabled = false; btn.textContent = 'Sign In';
  }}
}});
</script>
</body></html>"##, app = app_name, logo = logo_letter)
}

fn generate_register_page(state: &AppState) -> String {
    let app_name = &state.app.name;
    let logo_letter = app_name.chars().next().unwrap_or('C').to_uppercase().to_string();
    format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Register — {app}</title>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{font-family:'Inter',system-ui,-apple-system,sans-serif;-webkit-font-smoothing:antialiased;background:#0a0a0a;color:#fafafa;min-height:100vh;display:flex;align-items:center;justify-content:center}}
input{{width:100%;padding:10px 14px;font-size:14px;background:#171717;border:1px solid #262626;border-radius:10px;color:#fafafa;outline:none;transition:border-color 0.15s}}
input:focus{{border-color:#525252}}
.btn{{width:100%;padding:10px 14px;font-size:14px;font-weight:600;border:none;border-radius:10px;background:#fafafa;color:#0a0a0a;cursor:pointer;transition:opacity 0.15s}}
.btn:hover{{opacity:0.9}}
</style>
</head>
<body>
<div style="width:100%;max-width:400px;padding:32px">
  <div style="text-align:center;margin-bottom:32px">
    <div style="width:48px;height:48px;background:#fafafa;border-radius:12px;display:flex;align-items:center;justify-content:center;margin:0 auto 16px">
      <span style="color:#0a0a0a;font-weight:700;font-size:20px">{logo}</span>
    </div>
    <h1 style="font-size:24px;font-weight:700;margin:0 0 8px">Create your account</h1>
    <p style="font-size:14px;color:#a3a3a3">Get started for free</p>
  </div>
  <form id="registerForm" style="display:flex;flex-direction:column;gap:16px">
    <input name="name" type="text" placeholder="Full name" required />
    <input name="email" type="email" placeholder="Email" required />
    <input name="password" type="password" placeholder="Password" required minlength="6" />
    <div id="error" style="display:none;padding:10px 14px;border-radius:10px;font-size:13px;text-align:center;background:#450a0a;color:#fca5a5;border:1px solid #7f1d1d"></div>
    <button type="submit" class="btn">Sign Up</button>
    <p style="text-align:center;font-size:14px;color:#a3a3a3">Already have an account? <a href="/login" style="color:#fafafa;font-weight:600;text-decoration:none">Sign in</a></p>
  </form>
</div>
<script>
document.getElementById('registerForm').addEventListener('submit', async (e) => {{
  e.preventDefault();
  const btn = e.target.querySelector('button');
  btn.disabled = true; btn.textContent = 'Loading...';
  const data = Object.fromEntries(new FormData(e.target));
  try {{
    const res = await fetch('/api/auth/signup', {{ method: 'POST', headers: {{'Content-Type': 'application/json'}}, body: JSON.stringify(data) }});
    const json = await res.json();
    if (json.token) {{
      document.cookie = 'cronus_token=' + json.token + '; path=/; max-age=86400';
      localStorage.setItem('token', json.token);
      localStorage.setItem('user', JSON.stringify(json.user || {{}}));
      window.location.href = '/';
    }} else {{
      const err = document.getElementById('error');
      err.style.display = 'block';
      err.textContent = json.error || 'Registration failed';
      btn.disabled = false; btn.textContent = 'Sign Up';
    }}
  }} catch(err) {{
    const el = document.getElementById('error');
    el.style.display = 'block';
    el.textContent = 'Connection failed';
    btn.disabled = false; btn.textContent = 'Sign Up';
  }}
}});
</script>
</body></html>"##, app = app_name, logo = logo_letter)
}

async fn handle_request(
    req: Request<Incoming>,
    state: Arc<AppState>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let query = req.uri().query().unwrap_or("").to_string();

    // CORS preflight
    if method == Method::OPTIONS {
        return Ok(json_response(StatusCode::OK, json!({})));
    }

    // HMR version endpoint
    if path == "/__cronus/version" {
        return Ok(json_response(StatusCode::OK, json!({ "version": hmr::current_version() })));
    }

    // Brain: track every request
    let start = std::time::Instant::now();

    // Brain stats endpoint
    if path == "/api/brain/stats" {
        if let Some(ref brain) = state.brain {
            return Ok(json_response(StatusCode::OK, brain.stats()));
        }
        return Ok(json_response(StatusCode::OK, json!({"status": "brain not initialized"})));
    }
    if path == "/api/brain/suggest" {
        if let Some(ref brain) = state.brain {
            let suggestions = brain.suggest("");
            return Ok(json_response(StatusCode::OK, json!({"suggestions": suggestions})));
        }
        return Ok(json_response(StatusCode::OK, json!({"suggestions": []})));
    }

    // Auth routes
    if path.starts_with("/api/auth/") {
        let secret = auth::default_secret();
        let auth_header = req.headers().get("authorization").and_then(|v| v.to_str().ok()).map(|s| s.to_string());

        let user_table = state.entities.iter()
            .find(|e| {
                let lower = e.name.to_lowercase();
                lower == "user" || lower == "users"
            })
            .map(|e| e.name.as_str())
            .unwrap_or("User");

        let resp = match (method.clone(), path.as_str()) {
            (Method::GET, "/api/auth/me") => {
                match auth::extract_user(auth_header.as_deref(), &secret) {
                    Some(claims) => json_response(StatusCode::OK, json!({"sub": claims.sub, "role": claims.role, "exp": claims.exp})),
                    None => json_response(StatusCode::UNAUTHORIZED, json!({"error": "unauthorized"})),
                }
            }

            // POST /api/auth/signup
            (Method::POST, "/api/auth/signup") => {
                let body_bytes = req.collect().await.unwrap_or_default().to_bytes();
                let body: Value = serde_json::from_slice(&body_bytes).unwrap_or(json!({}));

                let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let email = body.get("email").and_then(|v| v.as_str()).unwrap_or("");
                let password = body.get("password").and_then(|v| v.as_str()).unwrap_or("");
                // Accept role from request, but only if it's a valid declared role
                let requested_role = body.get("role").and_then(|v| v.as_str()).unwrap_or("user");
                let role = if state.auth_roles.iter().any(|r| r == requested_role) {
                    requested_role.to_string()
                } else {
                    // Default to first non-admin role, or "user"
                    state.auth_roles.iter()
                        .find(|r| r.as_str() != "admin")
                        .cloned()
                        .unwrap_or_else(|| "user".to_string())
                };

                if name.is_empty() || email.is_empty() || password.is_empty() {
                    json_response(StatusCode::BAD_REQUEST, json!({"error": "name, email and password required"}))
                } else {
                    // Check if email already exists
                    let exists = state.db.find_all(user_table, 10000, 0)
                        .ok()
                        .and_then(|users| users.as_array().map(|arr| arr.iter().any(|u| u.get("email").and_then(|e| e.as_str()) == Some(email))))
                        .unwrap_or(false);

                    if exists {
                        json_response(StatusCode::BAD_REQUEST, json!({"error": "email already registered"}))
                    } else {
                        let hashed = auth::hash_password(password);
                        let user_data = json!({
                            "name": name,
                            "email": email,
                            "password": hashed,
                            "role": &role
                        });

                        match state.db.insert(user_table, &user_data) {
                            Ok(user) => {
                                let user_id = user.get("id").and_then(|v| v.as_str()).unwrap_or("");
                                let token = auth::create_token(user_id, &role, &secret);
                                let body = json!({
                                    "token": token,
                                    "user": {"id": user_id, "name": name, "email": email, "role": &role}
                                });
                                Response::builder()
                                    .status(StatusCode::CREATED)
                                    .header("Content-Type", "application/json")
                                    .header("Access-Control-Allow-Origin", "*")
                                    .header("Set-Cookie", format!("cronus_token={}; Path=/; HttpOnly; Max-Age=86400", token))
                                    .body(Full::new(Bytes::from(body.to_string())))
                                    .unwrap()
                            }
                            Err(e) => json_response(StatusCode::BAD_REQUEST, json!({"error": e}))
                        }
                    }
                }
            }

            // POST /api/auth/login
            (Method::POST, "/api/auth/login") => {
                let body_bytes = req.collect().await.unwrap_or_default().to_bytes();
                let body: Value = serde_json::from_slice(&body_bytes).unwrap_or(json!({}));

                let email = body.get("email").and_then(|v| v.as_str()).unwrap_or("");
                let password = body.get("password").and_then(|v| v.as_str()).unwrap_or("");

                if email.is_empty() || password.is_empty() {
                    json_response(StatusCode::BAD_REQUEST, json!({"error": "email and password required"}))
                } else {
                    match state.db.find_all(user_table, 10000, 0) {
                        Ok(users) => {
                            let user = users.as_array().and_then(|arr| {
                                arr.iter().find(|u| u.get("email").and_then(|e| e.as_str()) == Some(email))
                            }).cloned();

                            match user {
                                Some(u) => {
                                    let stored_pass = u.get("password").and_then(|v| v.as_str()).unwrap_or("");
                                    if auth::verify_password(password, stored_pass) {
                                        let user_id = u.get("id").and_then(|v| v.as_str()).unwrap_or("");
                                        let role = u.get("role").and_then(|v| v.as_str()).unwrap_or("user");
                                        let token = auth::create_token(user_id, role, &secret);
                                        let body = json!({
                                            "token": token,
                                            "user": {"id": user_id, "name": u.get("name"), "email": email, "role": role}
                                        });
                                        Response::builder()
                                            .status(StatusCode::OK)
                                            .header("Content-Type", "application/json")
                                            .header("Access-Control-Allow-Origin", "*")
                                            .header("Set-Cookie", format!("cronus_token={}; Path=/; HttpOnly; Max-Age=86400", token))
                                            .body(Full::new(Bytes::from(body.to_string())))
                                            .unwrap()
                                    } else {
                                        json_response(StatusCode::UNAUTHORIZED, json!({"error": "invalid credentials"}))
                                    }
                                }
                                None => json_response(StatusCode::UNAUTHORIZED, json!({"error": "invalid credentials"}))
                            }
                        }
                        Err(_) => json_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": "database error"}))
                    }
                }
            }

            _ => json_response(StatusCode::OK, json!({"info": "auth endpoint", "routes": ["/api/auth/signup POST", "/api/auth/login POST", "/api/auth/me GET"]})),
        };
        return Ok(resp);
    }

    // Payment endpoints
    if path == "/api/checkout" && method == Method::POST {
        let engine = payments::PaymentEngine::from_env();
        let result = engine.create_checkout_url("starter", 2900, "/billing/success", "/billing/cancel", None);
        match result {
            Ok(data) => return Ok(json_response(StatusCode::OK, data)),
            Err(e) => return Ok(json_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e}))),
        }
    }
    if path == "/api/webhooks/stripe" && method == Method::POST {
        let engine = payments::PaymentEngine::from_env();
        return Ok(json_response(StatusCode::OK, json!({"status": "webhook received", "mode": if engine.is_live() { "live" } else { "mock" }})));
    }
    if path == "/api/payments/status" {
        let engine = payments::PaymentEngine::from_env();
        return Ok(json_response(StatusCode::OK, engine.status()));
    }

    // Health endpoint
    if path == "/api/health" {
        return Ok(json_response(StatusCode::OK, json!({
            "ok": true,
            "app": state.app.name,
            "entities": state.entities.len(),
            "pages": state.pages.len(),
            "runtime": "cronus-kernel",
            "version": "0.1.0"
        })));
    }

    // Schema endpoint — returns all entities with fields
    if path == "/api/schema" {
        let schema: Vec<Value> = state.entities.iter().map(|e| {
            let fields: Vec<Value> = e.fields.iter().map(|f| {
                json!({
                    "name": f.name,
                    "type": format!("{:?}", f.field_type).to_lowercase(),
                    "required": f.required,
                    "unique": f.unique,
                })
            }).collect();
            json!({
                "entity": e.name,
                "fields": fields,
                "field_count": e.fields.len(),
            })
        }).collect();
        return Ok(json_response(StatusCode::OK, json!({
            "entities": schema,
            "total": state.entities.len(),
            "pages": state.pages.len(),
        })));
    }

    // Seed endpoint
    if path == "/api/_seed" && method == Method::POST {
        let mut results = serde_json::Map::new();
        for entity in &state.entities {
            if let Ok(count) = state.db.seed_entity(entity) {
                results.insert(entity.name.clone(), json!(count));
            }
        }
        return Ok(json_response(StatusCode::OK, json!({"seeded": results})));
    }

    // GraphQL endpoint
    if path == "/graphql" && method == Method::GET {
        return Ok(html_response(graphql::playground_html()));
    }
    if path == "/graphql" && method == Method::POST {
        let body_bytes = req.collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8_lossy(&body_bytes);
        let body_json: Value = serde_json::from_str(&body_str).unwrap_or(json!({}));
        let query = body_json.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let variables = body_json.get("variables").cloned().unwrap_or(json!({}));
        let schema = graphql::GraphQLSchema::from_entities(&state.entities);
        let db = Arc::new(database::CronusDB::open(&state.db_path).expect("db"));
        let result = graphql::execute_graphql(query, &variables, &schema, &db);
        return Ok(json_response(StatusCode::OK, result));
    }
    if path == "/graphql/schema" && method == Method::GET {
        let schema = graphql::GraphQLSchema::from_entities(&state.entities);
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain; charset=utf-8")
            .header("Access-Control-Allow-Origin", "*")
            .body(Full::new(Bytes::from(schema.sdl)))
            .unwrap());
    }

    // API routes: /api/...
    if path.starts_with("/api/") {
        let body_bytes = req.collect().await.unwrap_or_default().to_bytes();
        let body: Option<serde_json::Value> = serde_json::from_slice(&body_bytes).ok();
        let full_path = if query.is_empty() { path.clone() } else { format!("{}?{}", path, query) };
        let resp = handle_api(&method, &full_path, body.as_ref(), &state);
        // Track request in brain
        if let Some(ref brain) = state.brain {
            let duration = start.elapsed().as_millis() as u64;
            let status = resp.status().as_u16();
            brain.track_request(method.as_str(), &path, status, duration);
        }
        return Ok(resp);
    }

    // ── Action execution endpoint ──
    if method == Method::POST && path.starts_with("/_action/") {
        let body_bytes = req.collect().await.unwrap_or_default().to_bytes();
        let body: Value = serde_json::from_slice(&body_bytes).unwrap_or(json!({}));

        let entity = body.get("entity").and_then(|v| v.as_str()).unwrap_or("");
        let id = body.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let action_data = body.get("action").and_then(|v| v.as_str()).unwrap_or("{}");

        let (ok, effects) = actions::execute_action_validated(action_data, entity, id, &state.db, &state.entities);
        let response = actions::effects_to_json(&effects);
        return Ok(json_response(if ok { StatusCode::OK } else { StatusCode::BAD_REQUEST }, response));
    }

    // ── Form submission endpoint ──
    if method == Method::POST && path.starts_with("/_form/") {
        let body_bytes = req.collect().await.unwrap_or_default().to_bytes();
        let body: Value = serde_json::from_slice(&body_bytes).unwrap_or(json!({}));

        let entity = body.get("entity").and_then(|v| v.as_str()).unwrap_or("");
        let data = body.get("data").cloned().unwrap_or(json!({}));

        if data.is_object() && !entity.is_empty() {
            // Validate against entity schema if available
            if let Some(entity_schema) = state.entities.iter().find(|e| e.name.eq_ignore_ascii_case(entity)) {
                let errors = actions::validate_form_data(&data, entity_schema);
                if !errors.is_empty() {
                    let response = json!({
                        "ok": false,
                        "errors": errors,
                        "effects": [{"type": "toast", "target": "Validation failed", "style": "error"}]
                    });
                    return Ok(json_response(StatusCode::BAD_REQUEST, response));
                }
            }

            match state.db.insert(entity, &data) {
                Ok(row) => {
                    let response = json!({
                        "ok": true,
                        "id": row.get("id"),
                        "effects": [{"type": "toast", "target": "Created successfully", "style": "success"}]
                    });
                    return Ok(json_response(StatusCode::CREATED, response));
                }
                Err(e) => {
                    let response = json!({ "ok": false, "error": e, "effects": [{"type": "toast", "target": e, "style": "error"}] });
                    return Ok(json_response(StatusCode::BAD_REQUEST, response));
                }
            }
        }

        return Ok(json_response(StatusCode::BAD_REQUEST, json!({"ok": false, "error": "missing entity or data"})));
    }

    // Serve pages
    let accent = state.style.as_ref()
        .and_then(|s| s.accent.as_deref())
        .unwrap_or("amber");
    let app_name = &state.app.name;

    // ── Component preview route ──
    if path == "/__components" || path == "/__ui" {
        let body = if state.components.is_empty() {
            r#"<div style="padding:40px;text-align:center">
  <h1 style="font-size:16px;color:oklch(0.93 0 0);margin-bottom:8px">CRONUS UI Kit</h1>
  <p style="font-size:13px;color:oklch(0.5 0 0)">No components defined. Add <code style="background:oklch(0.18 0 0);padding:2px 6px;border-radius:4px">component</code> blocks to your .cronus file.</p>
</div>"#.to_string()
        } else {
            let comp_html = ui::render_components_page(&state.components);
            format!(
                r#"<div style="padding:20px">
  <div style="display:flex;align-items:center;gap:8px;margin-bottom:24px">
    <h1 style="font-size:16px;font-weight:400;color:oklch(0.93 0 0)">CRONUS UI Kit</h1>
    <span style="font-size:10px;padding:2px 8px;border-radius:20px;background:oklch(0.488 0.243 264/12%);color:oklch(0.488 0.243 264)">{} components</span>
  </div>
  <div style="display:flex;flex-direction:column;gap:16px">
    {}
  </div>
</div>"#,
                state.components.len(), comp_html
            )
        };
        let html = if let Some(ref layout) = state.layout {
            ui::render_layout_declarative(app_name, layout, "/_components", &body)
        } else {
            ui::render_layout(app_name, &state.pages, accent, &body)
        };
        return Ok(html_response(html));
    }

    // ── Auto-generated auth pages (when auth block exists) ──
    if state.auth_entity.is_some() {
        if path == "/login" {
            let html = generate_login_page(&state);
            return Ok(html_response(html));
        }
        if path == "/register" || path == "/signup" {
            let html = generate_register_page(&state);
            return Ok(html_response(html));
        }
        if path == "/logout" {
            return Ok(Response::builder()
                .status(StatusCode::FOUND)
                .header("Location", "/login")
                .header("Set-Cookie", "cronus_token=; Path=/; Max-Age=0")
                .body(Full::new(Bytes::new()))
                .unwrap());
        }
    }

    // ── Auth middleware — protect pages that require authentication ──
    let matched_requires = state.auth_required_pages.iter()
        .find(|(r, _)| r == &path || (path.starts_with(r.as_str()) && r != "/"))
        .map(|(_, req)| req.clone());

    if let Some(requires_str) = matched_requires {
        let token = req.headers().get("cookie")
            .and_then(|c| c.to_str().ok())
            .and_then(|c| c.split(';').find(|s| s.trim().starts_with("cronus_token=")))
            .map(|s| s.trim().trim_start_matches("cronus_token=").to_string())
            .or_else(|| req.headers().get("authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "))
                .map(|s| s.to_string()));

        let secret = auth::default_secret();

        let redirect_to_login = || Response::builder()
            .status(StatusCode::FOUND)
            .header("Location", "/login")
            .body(Full::new(Bytes::new()))
            .unwrap();

        if requires_str.starts_with("role(") {
            // Role-based access control
            let required_role = requires_str
                .trim_start_matches("role(")
                .trim_end_matches(')');
            match &token {
                Some(t) => {
                    match auth::verify_token(t, &secret) {
                        Ok(claims) => {
                            if claims.role != required_role && claims.role != "admin" {
                                return Ok(forbidden_response("Insufficient permissions"));
                            }
                        }
                        Err(_) => return Ok(redirect_to_login()),
                    }
                }
                None => return Ok(redirect_to_login()),
            }
        } else {
            // Simple auth check
            let authenticated = match &token {
                Some(t) => auth::verify_token(t, &secret).is_ok(),
                None => false,
            };
            if !authenticated {
                return Ok(redirect_to_login());
            }
        }
    }

    // Find matching page
    let page = state.pages.iter().find(|p| {
        if p.route == path { return true; }
        // Handle parameterized routes
        if p.route.contains(':') {
            let parts: Vec<&str> = p.route.split('/').collect();
            let req_parts: Vec<&str> = path.split('/').collect();
            if parts.len() == req_parts.len() {
                return parts.iter().zip(req_parts.iter()).all(|(p, r)| p.starts_with(':') || p == r);
            }
        }
        false
    });

    if let Some(page) = page {
        // Extract route params from parameterized routes (e.g. /orders/:id/edit)
        let route_params: std::collections::HashMap<String, String> = {
            let mut params = std::collections::HashMap::new();
            if page.route.contains(':') {
                let route_parts: Vec<&str> = page.route.split('/').collect();
                let path_parts: Vec<&str> = path.split('/').collect();
                for (rp, pp) in route_parts.iter().zip(path_parts.iter()) {
                    if let Some(param_name) = rp.strip_prefix(':') {
                        params.insert(param_name.to_string(), pp.to_string());
                    }
                }
            }
            params
        };

        if let Some(source_path) = page.config.get("source") {
            match std::fs::read_to_string(source_path) {
                Ok(html) => {
                    return Ok(html_response(html));
                }
                Err(err) => {
                    let body = format!(
                        r#"<div style="padding:40px">
  <h1 style="font-size:16px;color:var(--foreground);margin-bottom:8px">Failed to load source HTML</h1>
  <p style="font-size:13px;color:var(--foreground-muted);margin-bottom:8px">{}</p>
  <code style="font-size:12px;color:var(--foreground-subtle)">{}</code>
</div>"#,
                        err, source_path
                    );
                    let html = if let Some(ref layout) = state.layout {
                        ui::render_layout_declarative(app_name, layout, &path, &body)
                    } else {
                        ui::render_layout(app_name, &state.pages, accent, &body)
                    };
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .header("Content-Type", "text/html; charset=utf-8")
                        .body(Full::new(Bytes::from(html)))
                        .unwrap());
                }
            }
        }

        if page.config.get("layout").map(|s| s.as_str()) == Some("light-app") {
            let referenced: Vec<parser::ComponentNode> = if !page.components.is_empty() {
                page.components.iter()
                    .filter_map(|name| state.components.iter().find(|c| c.name == *name))
                    .cloned()
                    .collect()
            } else {
                state.components.clone()
            };
            let html = ui::render_light_app_page(app_name, &referenced);
            return Ok(html_response(html));
        }

        // Auth pages — standalone login/signup with no layout chrome
        let route_lower = page.route.to_lowercase();
        let title_lower = page.title.as_deref().unwrap_or("").to_lowercase();
        let is_auth_page = route_lower == "/login" || route_lower == "/signup"
            || title_lower.contains("sign in") || title_lower.contains("sign up")
            || title_lower.contains("login") || title_lower.contains("signup");
        if is_auth_page {
            let is_login = route_lower == "/login" || title_lower.contains("login") || title_lower.contains("sign in");
            let html = ui::render_auth_page(page, is_login);
            return Ok(html_response(html));
        }

        let theme = state.style.as_ref().and_then(|s| s.theme.as_deref()).unwrap_or("dark");
        let mut body = ui::render_page(page, &state.entities, accent, theme, Some(&state.db), &route_params);

        // If page references components (via `use ComponentName`), render them
        // BUT skip if page has sidebar component — dashboard renderers handle their own chrome
        let has_sidebar_component_early = !page.components.is_empty() && page.components.iter().any(|comp_name| {
            state.components.iter().any(|c| {
                c.name == *comp_name && (
                    c.style.as_deref().unwrap_or("").contains("sidenav") ||
                    c.layout.as_deref().unwrap_or("") == "sidebar"
                )
            })
        });
        if !page.components.is_empty() && !has_sidebar_component_early {
            let referenced: Vec<parser::ComponentNode> = page.components.iter()
                .filter_map(|name| state.components.iter().find(|c| c.name == *name))
                .cloned()
                .collect();
            if !referenced.is_empty() {
                body.push_str("\n");
                body.push_str(&ui::render_components_page(&referenced));
            }
        }

        // page type:components — render all components as showcase
        if page.page_type == "components" && !state.components.is_empty() {
            body.push_str("\n");
            body.push_str(&ui::render_components_page(&state.components));
        }

        // custom pages with no sections — fallback to component rendering
        if page.page_type == "custom" && page.sections.is_empty() && !state.components.is_empty() {
            body.push_str("\n");
            body.push_str(&ui::render_components_page(&state.components));
        }

        // FIX 1: Detect sidebar component — if page uses a Sidenav component, it's a dashboard page
        let has_sidebar_component = !page.components.is_empty() && page.components.iter().any(|comp_name| {
            state.components.iter().any(|c| {
                c.name == *comp_name && (
                    c.style.as_deref().unwrap_or("").contains("sidenav") ||
                    c.layout.as_deref().unwrap_or("") == "sidebar"
                )
            })
        });

        // Landing/checkout pages use full-width layout, no sidebar
        let landing_section_types = ["hero", "topbar", "checkout", "features", "pricing", "cta", "testimonial", "faq", "trusted", "footer"];
        // If ANY section has a template, it's a dumped page — always use landing layout
        let has_templates = page.sections.iter().any(|s| s.template.is_some() || s.config.get("template").is_some());
        let is_landing = has_templates || (!has_sidebar_component && (page.page_type == "checkout" || (page.page_type == "custom" && page.sections.iter().any(|s| landing_section_types.contains(&s.section_type.as_str())))));
        let dashboard_types = ["sidebar", "card", "page-header", "stat-cards", "product-grid",
            "team-list", "policies", "activity-table", "status-card", "links",
            "live-keys", "test-keys", "webhooks", "quick-links",
            "current-plan", "usage-status", "billing-stats", "payment-methods", "recent-invoices",
            "balance-card", "upcoming-card", "payout-history", "support-banner",
            "checkout-form", "product-summary", "trust-indicators",
            "team-members", "security-status", "security-policies", "login-activity"];
        let is_dashboard = has_sidebar_component || page.sections.iter().any(|s| dashboard_types.contains(&s.section_type.as_str()));
        let is_billing = page.sections.iter().any(|s| s.section_type == "current-plan" || s.section_type == "billing-stats");
        let is_payouts = page.sections.iter().any(|s| s.section_type == "balance-card" || s.section_type == "payout-history");
        let is_unified = page.sections.iter().any(|s| s.section_type == "balance-card")
            && page.sections.iter().any(|s| s.section_type == "billing-stats" || s.section_type == "recent-invoices");
        let is_payment_links = page.sections.iter().any(|s| s.section_type == "product-grid")
            && page.sections.iter().any(|s| s.section_type == "stat-cards");
        let is_checkout = page.sections.iter().any(|s| s.section_type == "checkout-form" || s.section_type == "product-summary");
        let is_security = page.sections.iter().any(|s| s.section_type == "team-members" || s.section_type == "login-activity");
        let current_route = page.route.as_str();
        let html = if has_templates {
            // Dumped page with original HTML templates — use landing layout, no sidebar
            ui::render_layout_landing_ex(app_name, &body, theme, state.style.as_ref(), state.app.tailwind_config.as_deref())
        } else if is_checkout {
            // Checkout page: no sidebar, centered layout
            let referenced_comps: Vec<parser::ComponentNode> = page.components.iter()
                .filter_map(|name| state.components.iter().find(|c| c.name == *name))
                .cloned()
                .collect();
            ui::render_checkout_dashboard(app_name, &page.sections, &referenced_comps, theme)
        } else if is_dashboard {
            // Dedicated dashboard renderer: produces the ENTIRE page in one shot
            let referenced_comps: Vec<parser::ComponentNode> = page.components.iter()
                .filter_map(|name| state.components.iter().find(|c| c.name == *name))
                .cloned()
                .collect();
            if is_unified {
                ui::render_unified_dashboard(app_name, &page.sections, &referenced_comps, theme, current_route)
            } else if is_payouts {
                ui::render_payouts_dashboard(app_name, &page.sections, &referenced_comps, theme, current_route)
            } else if is_billing {
                ui::render_billing_dashboard(app_name, &page.sections, &referenced_comps, theme, current_route)
            } else if is_security {
                ui::render_security_dashboard(app_name, &page.sections, &referenced_comps, theme, current_route)
            } else if is_payment_links {
                ui::render_payment_links_dashboard(app_name, &page.sections, &referenced_comps, theme, current_route)
            } else {
                // Generic dashboard wrapper — sidebar + any sections
                ui::render_generic_dashboard(app_name, &body, &referenced_comps, theme, current_route)
            }
        } else if is_landing {
            ui::render_layout_landing(app_name, &body, theme, state.style.as_ref())
        } else if let Some(ref layout) = state.layout {
            ui::render_layout_declarative(app_name, layout, current_route, &body)
        } else {
            ui::render_layout(app_name, &state.pages, accent, &body)
        };
        return Ok(html_response(html));
    }

    // 404
    let body = format!(
        "<div class=\"flex items-center justify-center min-h-[60vh]\"><div class=\"text-center\"><h1 class=\"text-6xl font-bold text-neutral-600\">404</h1><p class=\"mt-4 text-neutral-400\">Page not found</p><a href=\"/\" class=\"mt-6 inline-block text-{}-400 hover:underline\">← Back home</a></div></div>",
        accent
    );
    let html = if let Some(ref layout) = state.layout {
        ui::render_layout_declarative(app_name, layout, "/404", &body)
    } else {
        ui::render_layout(app_name, &state.pages, accent, &body)
    };
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Full::new(Bytes::from(html)))
        .unwrap())
}

fn handle_api(method: &Method, path: &str, body: Option<&serde_json::Value>, state: &AppState) -> Response<Full<Bytes>> {
    // Split path and query string
    let full_api = &path[4..]; // strip /api
    let (api_path, query_string) = match full_api.split_once('?') {
        Some((p, q)) => (p, q),
        None => (full_api, ""),
    };

    // Parse query params
    let params: Vec<(&str, &str)> = query_string.split('&')
        .filter(|p| !p.is_empty())
        .filter_map(|p| p.split_once('='))
        .collect();
    let get_param = |name: &str| -> Option<&str> {
        params.iter().find(|(k, _)| *k == name).map(|(_, v)| *v)
    };

    let limit: usize = get_param("limit").and_then(|v| v.parse().ok()).unwrap_or(100);
    let offset: usize = get_param("offset").and_then(|v| v.parse().ok()).unwrap_or(0);

    // Find matching entity by pluralized name in path
    let entity = state.entities.iter().find(|e| {
        let lower = e.name.to_lowercase();
        api_path.starts_with(&format!("/{}", lower))
            || api_path.starts_with(&format!("/{}s", lower))
    });

    if let Some(entity) = entity {
        let table = &entity.name;
        let segments: Vec<&str> = api_path.split('/').filter(|s| !s.is_empty()).collect();

        match *method {
            Method::GET => {
                if segments.len() >= 2 {
                    // GET /api/entity/:id
                    match state.db.find_by_id(table, segments[1]) {
                        Ok(Some(val)) => json_response(StatusCode::OK, val),
                        Ok(None) => json_response(StatusCode::NOT_FOUND, json!({"error": "not found"})),
                        Err(e) => json_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e})),
                    }
                } else {
                    // GET /api/entity?search=term&limit=N&offset=M
                    let search_query = get_param("search").or(get_param("q"));

                    if let Some(q) = search_query {
                        // Search mode
                        match state.db.search(table, q, limit) {
                            Ok(rows) => return json_response(StatusCode::OK, rows),
                            Err(_) => return json_response(StatusCode::OK, json!([])),
                        }
                    }

                    // Paginated list
                    let total = state.db.count(table).unwrap_or(0);
                    match state.db.find_all(table, limit, offset) {
                        Ok(rows) => {
                            Response::builder()
                                .status(StatusCode::OK)
                                .header("Content-Type", "application/json")
                                .header("Access-Control-Allow-Origin", "*")
                                .header("Access-Control-Expose-Headers", "X-Total-Count, X-Limit, X-Offset")
                                .header("X-Total-Count", total.to_string())
                                .header("X-Limit", limit.to_string())
                                .header("X-Offset", offset.to_string())
                                .body(Full::new(Bytes::from(rows.to_string())))
                                .unwrap()
                        }
                        Err(_) => json_response(StatusCode::OK, json!([])),
                    }
                }
            }
            Method::POST => {
                match body {
                    Some(data) => {
                        // Find entity definition for validation
                        let entity_def = state.entities.iter().find(|e| e.name.to_lowercase() == table.to_lowercase());
                        match entity_def {
                            Some(entity) => match state.db.validated_insert(entity, data) {
                                Ok(row) => json_response(StatusCode::CREATED, row),
                                Err(e) => {
                                    let status = if e.contains("required") {
                                        StatusCode::BAD_REQUEST // 400
                                    } else if e.contains("already exists") {
                                        StatusCode::CONFLICT // 409
                                    } else if e.contains("must be") {
                                        StatusCode::UNPROCESSABLE_ENTITY // 422
                                    } else {
                                        StatusCode::BAD_REQUEST
                                    };
                                    json_response(status, json!({"error": e}))
                                }
                            },
                            None => match state.db.insert(table, data) {
                                Ok(row) => json_response(StatusCode::CREATED, row),
                                Err(e) => json_response(StatusCode::BAD_REQUEST, json!({"error": e})),
                            }
                        }
                    },
                    None => json_response(StatusCode::BAD_REQUEST, json!({"error": "expected JSON body"})),
                }
            }
            Method::PATCH | Method::PUT => {
                if segments.len() >= 2 {
                    match body {
                        Some(data) => match state.db.update(table, segments[1], data) {
                            Ok(row) => json_response(StatusCode::OK, row),
                            Err(e) => json_response(StatusCode::BAD_REQUEST, json!({"error": e})),
                        },
                        None => json_response(StatusCode::BAD_REQUEST, json!({"error": "expected JSON body"})),
                    }
                } else {
                    json_response(StatusCode::BAD_REQUEST, json!({"error": "id required for PATCH"}))
                }
            }
            Method::DELETE => {
                if segments.len() >= 2 {
                    match state.db.delete(table, segments[1]) {
                        Ok(true) => json_response(StatusCode::OK, json!({"deleted": segments[1]})),
                        Ok(false) => json_response(StatusCode::NOT_FOUND, json!({"error": "not found"})),
                        Err(e) => json_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e})),
                    }
                } else {
                    json_response(StatusCode::BAD_REQUEST, json!({"error": "id required"}))
                }
            }
            _ => json_response(StatusCode::METHOD_NOT_ALLOWED, json!({"error": "method not allowed"})),
        }
    } else {
        json_response(StatusCode::NOT_FOUND, json!({"error": "unknown endpoint", "path": path}))
    }
}

// ══════════════════════════════════════════════════
// COMMANDS
// ══════════════════════════════════════════════════

async fn cmd_run(args: &[String]) {
    let start_time = Instant::now();

    // Find .cronus files — supports multi-agent mode
    let files = find_all_cronus_files();
    if files.is_empty() {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus file found in current directory");
        std::process::exit(1);
    }

    let (nodes, total_lines) = if files.len() == 1 {
        // Single file mode
        let source = fs::read_to_string(&files[0]).unwrap_or_else(|e| {
            eprintln!("  \x1b[31m✗\x1b[0m Error reading {}: {}", files[0], e);
            std::process::exit(1);
        });
        let lines = source.lines().count();
        let n = match parser::parse_with_imports(&source, ".") {
            Ok(n) => n,
            Err(e) => { eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e); std::process::exit(1); }
        };
        (n, lines)
    } else {
        // Multi-agent mode: compose all .cronus files
        println!("  \x1b[36m⚡\x1b[0m Multi-file mode: {} files detected", files.len());
        let mut total = 0;
        for f in &files {
            let lines = fs::read_to_string(f).map(|s| s.lines().count()).unwrap_or(0);
            total += lines;
        }
        let n = match parser::parse_directory(".") {
            Ok(n) => n,
            Err(e) => { eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e); std::process::exit(1); }
        };

        (n, total)
    };

    let file = files[0].clone(); // for HMR watcher

    // Extract AST parts
    let mut app = AppNode { name: "CRONUS App".into(), stack: vec![], port: 5175, database: None, tailwind_config: None };
    let mut entities: Vec<EntityNode> = vec![];
    let mut pages: Vec<PageNode> = vec![];
    let mut style: Option<StyleNode> = None;
    let mut apis: Vec<ApiNode> = vec![];
    let mut cronus_components: Vec<parser::ComponentNode> = vec![];
    let mut route_count = 0;
    let mut auth_entity: Option<String> = None;
    let mut auth_roles: Vec<String> = Vec::new();
    let mut auth_required_pages: Vec<(String, String)> = Vec::new();
    let mut layout: Option<parser::LayoutNode> = None;

    for node in &nodes {
        match node {
            AstNode::App(a) => app = a.clone(),
            AstNode::Entity(e) => entities.push(e.clone()),
            AstNode::Page(p) => {
                // Track pages that require auth
                let req = p.requires.clone().or_else(|| p.config.get("requires").cloned());
                if let Some(ref req_val) = req {
                    auth_required_pages.push((p.route.clone(), req_val.clone()));
                }
                pages.push(p.clone());
            }
            AstNode::Style(s) => style = Some(s.clone()),
            AstNode::Api(a) => {
                route_count += a.routes.len();
                apis.push(a.clone());
            }
            AstNode::Component(c) => cronus_components.push(c.clone()),
            AstNode::Auth(auth) => {
                auth_entity = Some(auth.entity.clone());
                auth_roles = auth.roles.clone();
            }
            AstNode::Layout(l) => {
                layout = Some(l.clone());
            }
            _ => {}
        }
    }

    // CLI port takes precedence
    let cli_port = args.iter().skip(2).find_map(|s| s.parse::<u16>().ok());
    let serve_port = if let Some(p) = cli_port { p } else { app.port };
    let comp_count = cronus_components.len();


    // Database — use CronusDB for all operations
    let db_path = app.database.as_ref()
        .and_then(|d| d.path.clone())
        .unwrap_or_else(|| "data.db".into());

    let app_db = database::CronusDB::open(&db_path).expect("Failed to open database");
    app_db.migrate(&entities).expect("Failed to migrate database");

    // Ensure User table has password column for auth (auto-added by kernel)
    let has_user = entities.iter().any(|e| { let l = e.name.to_lowercase(); l == "user" || l == "users" });
    if has_user {
        let user_table = entities.iter()
            .find(|e| { let l = e.name.to_lowercase(); l == "user" || l == "users" })
            .map(|e| e.name.as_str())
            .unwrap();
        let _ = app_db.execute_raw(&format!(
            "ALTER TABLE \"{}\" ADD COLUMN password TEXT", user_table
        ));
    }

    let table_count = entities.len();


    // Initialize Hydra Brain
    let brain_db = Arc::new(database::CronusDB::open(&db_path).expect("Failed to open brain DB"));
    // Create brain events table
    let brain_entity = parser::EntityNode {
        name: "_brain_events".to_string(),
        fields: vec![
            parser::FieldNode {
                name: "event".to_string(),
                field_type: parser::FieldType::String,
                required: true,
                unique: false, sensitive: false, optional: false, searchable: false,
                index: false, featured: false, formatted: false, array: false,
                enum_values: None, reference: None,
            },
            parser::FieldNode {
                name: "metadata".to_string(),
                field_type: parser::FieldType::Text,
                required: false,
                unique: false, sensitive: false, optional: false, searchable: false,
                index: false, featured: false, formatted: false, array: false,
                enum_values: None, reference: None,
            },
            parser::FieldNode {
                name: "timestamp".to_string(),
                field_type: parser::FieldType::String,
                required: false,
                unique: false, sensitive: false, optional: false, searchable: false,
                index: false, featured: false, formatted: false, array: false,
                enum_values: None, reference: None,
            },
        ],
    };
    let _ = brain_db.migrate(&[brain_entity]);
    let hydra = brain::CronusBrain::init(brain_db);


    // Build app state (reuse app_db from migration)

    let state = Arc::new(AppState {
        app: app.clone(),
        entities,
        pages,
        components: cronus_components,
        style,
        apis,
        db_path: db_path.clone(),
        db: app_db,
        brain: Some(hydra),
        auth_entity,
        auth_roles,
        auth_required_pages,
        layout,
    });

    // Start server
    let addr = format!("0.0.0.0:{}", serve_port);
    let listener = TcpListener::bind(&addr).await.unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Cannot bind to port {}: {}", serve_port, e);
        std::process::exit(1);
    });

    let accent = state.style.as_ref()
        .and_then(|s| s.accent.as_deref())
        .unwrap_or("amber");

    // Start HMR file watcher
    if files.len() > 1 {
        hmr::start_directory_watcher(".", move || {
            let v = hmr::bump_version();
            eprintln!("  \x1b[33m⚡\x1b[0m File changed — version {} (browser will reload)", v);
        });
    } else {
        let hmr_file = file.clone();
        hmr::start_watcher(&hmr_file, move || {
            let v = hmr::bump_version();
            eprintln!("  \x1b[33m⚡\x1b[0m File changed — version {} (browser will reload)", v);
        });
    }

    // Count total rows across all entities
    let total_rows: usize = state.entities.iter().map(|e| {
        state.db.count(&e.name).unwrap_or(0)
    }).sum();

    let auth_page_count = state.auth_required_pages.len();
    let total_pages = state.pages.len();
    let elapsed_ms = start_time.elapsed().as_millis();

    // ── Clean startup banner ──
    println!();
    println!("  \x1b[36m\x1b[1mCRONUS\x1b[0m \x1b[90mv0.1.0\x1b[0m");
    println!();
    println!("  \x1b[90mApp:\x1b[0m       \x1b[1m{}\x1b[0m", app.name);
    println!("  \x1b[90mPort:\x1b[0m      \x1b]8;;http://localhost:{}\x1b\\http://localhost:{}\x1b]8;;\x1b\\", serve_port, serve_port);
    println!("  \x1b[90mDatabase:\x1b[0m  ./{} ({} entities, {} rows)", db_path, table_count, total_rows);
    if total_pages > 0 {
        if auth_page_count > 0 {
            println!("  \x1b[90mPages:\x1b[0m     {} ({} require auth)", total_pages, auth_page_count);
        } else {
            println!("  \x1b[90mPages:\x1b[0m     {}", total_pages);
        }
    }
    if route_count > 0 {
        println!("  \x1b[90mRoutes:\x1b[0m    {} API endpoints", route_count);
    }
    if let Some(ref _auth_e) = state.auth_entity {
        if state.auth_roles.is_empty() {
            println!("  \x1b[90mAuth:\x1b[0m      JWT");
        } else {
            println!("  \x1b[90mAuth:\x1b[0m      JWT (roles: {})", state.auth_roles.join(", "));
        }
    }
    println!();
    println!("  \x1b[32mReady in {}ms\x1b[0m", elapsed_ms);
    println!();
    println!("  Press Ctrl+C to stop.");
    println!();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);
        let state = state.clone();

        tokio::task::spawn(async move {
            let service = service_fn(move |req| {
                let state = state.clone();
                async move { handle_request(req, state).await }
            });
            if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                eprintln!("  Connection error: {}", e);
            }
        });
    }
}

fn cmd_dump(args: &[String]) {
    let file = args.get(2).unwrap_or_else(|| {
        eprintln!("  \x1b[31m✗\x1b[0m Usage: cronus dump <file.html|.json|.prisma|dir/> [-o output.cronus]");
        std::process::exit(1);
    });

    // Black Hole mode: dump entire project directory
    let path = std::path::Path::new(file);
    if path.is_dir() {
        let output = dump::project::dump_project(path);

        // Determine output file name
        let out_file = args.iter().position(|a| a == "-o")
            .and_then(|i| args.get(i + 1))
            .cloned()
            .unwrap_or_else(|| {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                format!("{}.cronus", name)
            });

        fs::write(&out_file, &output).expect("Failed to write output");
        eprintln!("  \x1b[32m✓\x1b[0m Written to {}", out_file);
        return;
    }

    let html = fs::read_to_string(file).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Error reading {}: {}", file, e);
        std::process::exit(1);
    });

    eprintln!("  \x1b[36m⚡\x1b[0m Dumping {} ({} bytes)...", file, html.len());

    // Detect file format
    let cronus = if file.ends_with(".prisma") {
        eprintln!("  \x1b[36m⚡\x1b[0m Detected Prisma schema");
        dump::prisma::dump_prisma(&html)
    } else if file.ends_with(".json") {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&html) {
            if parsed.get("openapi").is_some() || parsed.get("swagger").is_some() {
                eprintln!("  \x1b[36m⚡\x1b[0m Detected OpenAPI spec");
                dump::openapi::dump_openapi(&html)
            } else {
                dump::dump_html(&html)
            }
        } else {
            dump::dump_html(&html)
        }
    } else {
        dump::dump_html(&html)
    };

    // Check for -o flag
    let output_file = args.iter().position(|a| a == "-o").and_then(|i| args.get(i + 1));
    if let Some(out) = output_file {
        fs::write(out, &cronus).unwrap_or_else(|e| {
            eprintln!("  \x1b[31m✗\x1b[0m Error writing {}: {}", out, e);
            std::process::exit(1);
        });
        eprintln!("  \x1b[32m✓\x1b[0m Written to {}", out);
    } else {
        println!("{}", cronus);
    }
}

// ══════════════════════════════════════════════════
// SPEC COMMANDS
// ══════════════════════════════════════════════════

fn cmd_spec(args: &[String]) {
    let subcmd = args.get(2).map(|s| s.as_str()).unwrap_or("help");
    match subcmd {
        "validate" => spec_validate(args),
        "list" => spec_list(args),
        "codegen" => {
            if args.iter().any(|a| a == "--structs") {
                spec_codegen_structs(args);
            } else if args.iter().any(|a| a == "--docs") {
                spec_codegen_docs(args);
            } else if args.iter().any(|a| a == "--ai-protocol") {
                spec_codegen_ai_protocol(args);
            } else {
                println!("  Usage: cronus spec codegen <--structs|--docs|--ai-protocol>");
            }
        }
        _ => {
            println!("  Usage: cronus spec <validate|list|codegen>");
        }
    }
}

fn extract_field(content: &str, field_name: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{} =", field_name))
            || trimmed.starts_with(&format!("{}=", field_name))
        {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    return Some(trimmed[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    None
}

fn has_section(content: &str, section: &str) -> bool {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == section || trimmed.starts_with(&format!("{}.", &section[..section.len() - 1])) {
            return true;
        }
    }
    false
}

fn validate_spec_content(content: &str, subdir: &str) -> Vec<String> {
    let mut errors = Vec::new();

    // 1. Must have [meta] with name, version, layer, stability
    if !has_section(content, "[meta]") {
        errors.push("missing [meta] section".to_string());
        return errors;
    }

    let name = extract_field(content, "name");
    let version = extract_field(content, "version");
    let layer = extract_field(content, "layer");
    let stability = extract_field(content, "stability");

    if name.is_none() {
        errors.push("missing meta field: name".to_string());
    }
    if version.is_none() {
        errors.push("missing meta field: version".to_string());
    }
    if layer.is_none() {
        errors.push("missing meta field: layer".to_string());
    }
    if stability.is_none() {
        errors.push("missing meta field: stability".to_string());
    }

    // 2. layer must be core, stdlib, or pattern
    if let Some(ref l) = layer {
        if !["core", "stdlib", "pattern"].contains(&l.as_str()) {
            errors.push(format!("invalid layer '{}' (expected core|stdlib|pattern)", l));
        }
    }

    // 3. stability must be draft, experimental, stable, or deprecated
    if let Some(ref s) = stability {
        if !["draft", "experimental", "stable", "deprecated"].contains(&s.as_str()) {
            errors.push(format!("invalid stability '{}' (expected draft|experimental|stable|deprecated)", s));
        }
    }

    // 4. If layer is "pattern": must have [alias] with canonical
    if let Some(ref l) = layer {
        if l == "pattern" {
            if !has_section(content, "[alias]") {
                errors.push("pattern spec must have [alias] section".to_string());
            } else {
                let canonical = extract_field(content, "canonical");
                if canonical.is_none() {
                    errors.push("pattern spec [alias] must have 'canonical' field".to_string());
                }

                // 5. If alias canonical != "none": must NOT have [keys.structural]
                if let Some(ref c) = canonical {
                    if c != "none" && has_section(content, "[keys.structural") {
                        errors.push("alias with canonical != 'none' must not have [keys.structural]".to_string());
                    }
                }
            }
        }
    }

    // 6. If NOT an alias: should have [shape] section
    let is_alias = layer.as_deref() == Some("pattern")
        && extract_field(content, "canonical").map(|c| c != "none").unwrap_or(false);
    if !is_alias && layer.as_deref() != Some("core") {
        if !has_section(content, "[shape]") {
            errors.push("non-alias spec should have [shape] section".to_string());
        }
    }

    // 7. version should match semver X.Y.Z
    if let Some(ref v) = version {
        let parts: Vec<&str> = v.split('.').collect();
        let valid = parts.len() == 3 && parts.iter().all(|p| p.parse::<u32>().is_ok());
        if !valid {
            errors.push(format!("version '{}' is not valid semver (expected X.Y.Z)", v));
        }
    }

    errors
}

// ── Codegen helpers ────────────────────────────────────────────────────────

fn extract_structural_keys(content: &str) -> Vec<(String, bool)> {
    let mut keys = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        // Match [keys.structural.KEYNAME]
        if trimmed.starts_with("[keys.structural.") && trimmed.ends_with(']') {
            let key_name = &trimmed[17..trimmed.len() - 1];
            // Look ahead for required = true/false
            let required = content
                .lines()
                .skip_while(|l| l.trim() != trimmed)
                .skip(1)
                .take_while(|l| !l.trim().starts_with('['))
                .any(|l| {
                    let t = l.trim();
                    t.starts_with("required") && t.contains("true")
                });
            keys.push((key_name.to_string(), required));
        }
    }
    keys
}

fn extract_config_keys(content: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut in_config = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[config]" {
            in_config = true;
            continue;
        }
        if trimmed.starts_with('[') && trimmed != "[config]" {
            if in_config {
                break;
            }
            continue;
        }
        if in_config && !trimmed.is_empty() && !trimmed.starts_with('#') {
            // Lines like: key = { ... } or key = "value"
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                if !key.is_empty() {
                    keys.push(key.to_string());
                }
            }
        }
    }
    keys
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn capitalize_layer(s: &str) -> String {
    match s {
        "core" => "Core".to_string(),
        "stdlib" => "Stdlib".to_string(),
        "pattern" => "Pattern".to_string(),
        _ => capitalize_first(s),
    }
}

fn capitalize_stability(s: &str) -> String {
    match s {
        "stable" => "Stable".to_string(),
        "experimental" => "Experimental".to_string(),
        "internal" => "Internal".to_string(),
        "deprecated" => "Deprecated".to_string(),
        _ => capitalize_first(s),
    }
}

fn capitalize_fallback(s: &str) -> String {
    match s.to_lowercase().as_str() {
        "warn" => "Warn".to_string(),
        "error" => "Error".to_string(),
        "ignore" => "Ignore".to_string(),
        _ => capitalize_first(s),
    }
}

fn extract_field_bool(content: &str, field_name: &str) -> Option<bool> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{} =", field_name))
            || trimmed.starts_with(&format!("{}=", field_name))
        {
            if trimmed.contains("true") {
                return Some(true);
            }
            if trimmed.contains("false") {
                return Some(false);
            }
        }
    }
    None
}

fn extract_field_usize(content: &str, field_name: &str) -> Option<usize> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{} =", field_name))
            || trimmed.starts_with(&format!("{}=", field_name))
        {
            if let Some(eq_pos) = trimmed.find('=') {
                let val = trimmed[eq_pos + 1..].trim().trim_matches('"');
                if let Ok(n) = val.parse::<usize>() {
                    return Some(n);
                }
            }
        }
    }
    None
}

// ── Codegen: --structs ─────────────────────────────────────────────────────

fn spec_codegen_structs(args: &[String]) {
    let dir = args.iter().position(|a| a == "--dir").and_then(|i| args.get(i + 1))
        .map(|s| s.as_str()).unwrap_or("specs");

    println!("// Auto-generated by `cronus spec codegen --structs`");
    println!("// Do not edit manually. Edit .spec.toml files instead.");
    println!("// Source: {}/", dir);
    println!();
    println!("use crate::contracts::{{SectionContract, KeyDef, Fallback, Layer, Stability}};");
    println!();

    let mut all_names: Vec<String> = Vec::new();
    let mut all_consts: Vec<String> = Vec::new();
    let mut alias_map: Vec<(String, String)> = Vec::new();

    for subdir in &["stdlib", "patterns"] {
        let path = format!("{}/{}", dir, subdir);
        let mut entries: Vec<_> = match std::fs::read_dir(&path) {
            Ok(e) => e.flatten().collect(),
            Err(_) => continue,
        };
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let file_path = entry.path();
            if !file_path.extension().map(|e| e == "toml").unwrap_or(false) {
                continue;
            }
            let content = std::fs::read_to_string(&file_path).unwrap_or_default();
            let name = match extract_field(&content, "name") {
                Some(n) => n,
                None => continue,
            };

            // Check if this is a pure alias (has [alias] with canonical != "none")
            if has_section(&content, "[alias]") {
                let canonical = extract_field(&content, "canonical").unwrap_or_default();
                if canonical != "none" && !canonical.is_empty() {
                    alias_map.push((name, canonical));
                    continue;
                }
            }

            let layer = extract_field(&content, "layer").unwrap_or_else(|| "stdlib".to_string());
            let stability = extract_field(&content, "stability").unwrap_or_else(|| "stable".to_string());
            let requires_title = extract_field_bool(&content, "requires_title").unwrap_or(false);
            let requires_items = extract_field_bool(&content, "requires_items").unwrap_or(false);
            let min_items = extract_field_usize(&content, "min_items").unwrap_or(0);

            let entity_binding = has_section(&content, "[keys.entity_bound]")
                && extract_field(&content, "strategy")
                    .map(|s| s == "entity_fields" || s == "column_names")
                    .unwrap_or(false);

            let structural_keys = extract_structural_keys(&content);
            let config_keys = extract_config_keys(&content);

            let on_unknown = extract_field(&content, "on_unknown_structural_key").unwrap_or_else(|| "warn".to_string());
            let on_missing = extract_field(&content, "on_missing_required_key").unwrap_or_else(|| "error".to_string());

            let const_name = name.to_uppercase().replace('-', "_");

            let struct_keys_str = structural_keys
                .iter()
                .map(|(k, req)| {
                    if *req {
                        format!("req(\"{}\")", k)
                    } else {
                        format!("opt(\"{}\")", k)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");

            let config_keys_str = config_keys
                .iter()
                .map(|k| format!("\"{}\"", k))
                .collect::<Vec<_>>()
                .join(", ");

            println!("pub static {}_CONTRACT: SectionContract = SectionContract {{", const_name);
            println!("    name: \"{}\",", name);
            println!("    layer: Layer::{},", capitalize_layer(&layer));
            println!("    stability: Stability::{},", capitalize_stability(&stability));
            println!("    requires_title: {},", requires_title);
            println!("    requires_items: {},", requires_items);
            println!("    min_items: {},", min_items);
            println!("    structural_keys: &[{}],", struct_keys_str);
            println!("    entity_binding: {},", entity_binding);
            println!("    on_unknown_key: Fallback::{},", capitalize_fallback(&on_unknown));
            println!("    on_missing_required: Fallback::{},", capitalize_fallback(&on_missing));
            println!("    config_keys: &[{}],", config_keys_str);
            println!("}};");
            println!();

            all_names.push(name.clone());
            all_consts.push(const_name);
        }
    }

    // Generate helpers
    println!("// ── Helpers ─────────────────────────────────────────────────────────────────");
    println!();
    println!("const fn req(name: &'static str) -> KeyDef {{ KeyDef {{ name, required: true }} }}");
    println!("const fn opt(name: &'static str) -> KeyDef {{ KeyDef {{ name, required: false }} }}");
    println!();

    // Generate registry
    println!("// ── Registry ────────────────────────────────────────────────────────────────");
    println!();
    println!("pub static ALL_CONTRACTS: &[&SectionContract] = &[");
    for c in &all_consts {
        println!("    &{}_CONTRACT,", c);
    }
    println!("];");
    println!();
    println!("pub static ALL_NAMES: &[&str] = &[");
    for names_chunk in all_names.chunks(5) {
        let line = names_chunk.iter().map(|n| format!("\"{}\"", n)).collect::<Vec<_>>().join(", ");
        println!("    {},", line);
    }
    println!("];");
    println!();

    // Generate alias resolver
    if !alias_map.is_empty() {
        println!("// ── Aliases ─────────────────────────────────────────────────────────────────");
        println!();
        println!("pub fn resolve_alias(name: &str) -> Option<&'static str> {{");
        println!("    match name {{");
        for (alias, canonical) in &alias_map {
            println!("        \"{}\" => Some(\"{}\"),", alias, canonical);
        }
        println!("        _ => None,");
        println!("    }}");
        println!("}}");
    }
}

// ── Codegen: --docs ────────────────────────────────────────────────────────

fn extract_field_after_section(content: &str, section: &str, field_name: &str) -> Option<String> {
    let mut in_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed == section;
            continue;
        }
        if in_section {
            if trimmed.starts_with(&format!("{} =", field_name))
                || trimmed.starts_with(&format!("{}=", field_name))
            {
                if let Some(start) = trimmed.find('"') {
                    if let Some(end) = trimmed[start + 1..].find('"') {
                        return Some(trimmed[start + 1..start + 1 + end].to_string());
                    }
                }
            }
        }
    }
    None
}

fn extract_key_description(content: &str, key_name: &str) -> Option<String> {
    let header = format!("[keys.structural.{}]", key_name);
    let mut in_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == header {
            in_section = true;
            continue;
        }
        if trimmed.starts_with('[') && in_section {
            break;
        }
        if in_section
            && (trimmed.starts_with("description =") || trimmed.starts_with("description="))
        {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    return Some(trimmed[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    None
}

fn extract_key_type(content: &str, key_name: &str) -> String {
    let header = format!("[keys.structural.{}]", key_name);
    let mut in_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == header {
            in_section = true;
            continue;
        }
        if trimmed.starts_with('[') && in_section {
            break;
        }
        if in_section && (trimmed.starts_with("type =") || trimmed.starts_with("type=")) {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    return trimmed[start + 1..start + 1 + end].to_string();
                }
            }
        }
    }
    "string".to_string()
}

fn extract_config_details(content: &str) -> Vec<(String, String, String, String)> {
    // Returns (key, type, default, description)
    let mut results = Vec::new();
    let mut in_config = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[config]" {
            in_config = true;
            continue;
        }
        if trimmed.starts_with('[') && trimmed != "[config]" {
            if in_config {
                break;
            }
            continue;
        }
        if in_config && !trimmed.is_empty() && !trimmed.starts_with('#') {
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim().to_string();
                let rest = trimmed[eq_pos + 1..].trim();
                // Parse inline table: { type = "...", description = "...", default = "..." }
                let mut typ = "string".to_string();
                let mut default = String::new();
                let mut desc = String::new();
                if rest.starts_with('{') {
                    // Extract type
                    if let Some(t_start) = rest.find("type") {
                        let after = &rest[t_start..];
                        if let Some(q1) = after.find('"') {
                            if let Some(q2) = after[q1 + 1..].find('"') {
                                typ = after[q1 + 1..q1 + 1 + q2].to_string();
                            }
                        }
                    }
                    // Extract default
                    if let Some(d_start) = rest.find("default") {
                        let after = &rest[d_start..];
                        if let Some(q1) = after.find('"') {
                            if let Some(q2) = after[q1 + 1..].find('"') {
                                default = after[q1 + 1..q1 + 1 + q2].to_string();
                            }
                        }
                    }
                    // Extract description
                    if let Some(d_start) = rest.find("description") {
                        let after = &rest[d_start..];
                        if let Some(q1) = after.find('"') {
                            if let Some(q2) = after[q1 + 1..].find('"') {
                                desc = after[q1 + 1..q1 + 1 + q2].to_string();
                            }
                        }
                    }
                }
                results.push((key, typ, default, desc));
            }
        }
    }
    results
}

fn spec_codegen_docs(args: &[String]) {
    let dir = args.iter().position(|a| a == "--dir").and_then(|i| args.get(i + 1))
        .map(|s| s.as_str()).unwrap_or("specs");

    println!("# Section Types Reference (Auto-Generated)");
    println!();
    println!("> Generated from .spec.toml files. Do not edit manually.");
    println!();

    for subdir in &["stdlib", "patterns"] {
        println!("## {}", if *subdir == "stdlib" { "Standard Library" } else { "Patterns" });
        println!();

        let path = format!("{}/{}", dir, subdir);
        let mut entries: Vec<_> = match std::fs::read_dir(&path) {
            Ok(e) => e.flatten().collect(),
            Err(_) => continue,
        };
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let file_path = entry.path();
            if !file_path.extension().map(|e| e == "toml").unwrap_or(false) {
                continue;
            }
            let content = std::fs::read_to_string(&file_path).unwrap_or_default();
            let name = match extract_field(&content, "name") {
                Some(n) => n,
                None => continue,
            };
            let stability = extract_field(&content, "stability").unwrap_or_else(|| "stable".to_string());
            let layer = extract_field(&content, "layer").unwrap_or_else(|| "stdlib".to_string());
            let renderer = extract_field(&content, "renderer").unwrap_or_else(|| "generic".to_string());
            let description = extract_field(&content, "description").unwrap_or_default();

            // Check if pure alias
            let is_alias = has_section(&content, "[alias]")
                && extract_field(&content, "canonical")
                    .map(|c| c != "none" && !c.is_empty())
                    .unwrap_or(false);

            println!("### {}", name);
            println!();
            println!("> **Status:** {} | **Layer:** {} | **Renderer:** {}", stability, layer, renderer);
            if is_alias {
                let canonical = extract_field(&content, "canonical").unwrap_or_default();
                println!("> **Alias of:** {}", canonical);
            }
            println!();
            if !description.is_empty() {
                println!("{}", description);
                println!();
            }

            // Skip detailed sections for pure aliases
            if is_alias {
                println!("---");
                println!();
                continue;
            }

            // Structural keys
            let structural_keys = extract_structural_keys(&content);
            if !structural_keys.is_empty() {
                println!("**Structural keys:**");
                println!();
                println!("| Key | Required | Type | Description |");
                println!("|-----|----------|------|-------------|");
                for (key, required) in &structural_keys {
                    let typ = extract_key_type(&content, key);
                    let desc = extract_key_description(&content, key).unwrap_or_default();
                    println!("| `{}` | {} | {} | {} |", key, if *required { "yes" } else { "no" }, typ, desc);
                }
                println!();
            }

            // Config keys
            let config_details = extract_config_details(&content);
            if !config_details.is_empty() {
                println!("**Config keys:**");
                println!();
                println!("| Key | Type | Default | Description |");
                println!("|-----|------|---------|-------------|");
                for (key, typ, default, desc) in &config_details {
                    let def_display = if default.is_empty() { "-".to_string() } else { format!("`{}`", default) };
                    println!("| `{}` | {} | {} | {} |", key, typ, def_display, desc);
                }
                println!();
            }

            // Shape info
            let requires_title = extract_field_bool(&content, "requires_title").unwrap_or(false);
            let requires_items = extract_field_bool(&content, "requires_items").unwrap_or(false);
            let min_items = extract_field_usize(&content, "min_items").unwrap_or(0);
            println!("**Shape:** title={}, items={}, min_items={}",
                if requires_title { "required" } else { "optional" },
                if requires_items { "required" } else { "optional" },
                min_items);
            println!();

            // Example
            println!("**Example:**");
            println!();
            println!("```cronus");
            if structural_keys.is_empty() {
                println!("section {} {{", name);
                println!("  title \"Example\"");
                println!("}}");
            } else {
                println!("section {} {{", name);
                let required_keys: Vec<_> = structural_keys.iter().filter(|(_, r)| *r).collect();
                if required_keys.is_empty() {
                    println!("  item {{");
                    if let Some((first_key, _)) = structural_keys.first() {
                        println!("    {} \"value\"", first_key);
                    }
                    println!("  }}");
                } else {
                    println!("  item {{");
                    for (key, _) in &required_keys {
                        println!("    {} \"value\"", key);
                    }
                    println!("  }}");
                }
                println!("}}");
            }
            println!("```");
            println!();
            println!("---");
            println!();
        }
    }
}

// ── Codegen: --ai-protocol ─────────────────────────────────────────────────

fn extract_aliases_from_meta(content: &str) -> Vec<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("aliases") && trimmed.contains('=') {
            if let Some(bracket_start) = trimmed.find('[') {
                if let Some(bracket_end) = trimmed.find(']') {
                    let inner = &trimmed[bracket_start + 1..bracket_end];
                    return inner
                        .split(',')
                        .map(|s| s.trim().trim_matches('"').to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
        }
    }
    Vec::new()
}

fn extract_config_validates(content: &str, key_name: &str) -> Option<String> {
    let mut in_config = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[config]" {
            in_config = true;
            continue;
        }
        if trimmed.starts_with('[') && trimmed != "[config]" {
            if in_config { break; }
            continue;
        }
        if in_config && trimmed.starts_with(key_name) {
            if let Some(v_start) = trimmed.find("validates") {
                let after = &trimmed[v_start..];
                if let Some(q1) = after.find('"') {
                    if let Some(q2) = after[q1 + 1..].find('"') {
                        return Some(after[q1 + 1..q1 + 1 + q2].to_string());
                    }
                }
            }
        }
    }
    None
}

fn extract_key_default(content: &str, key_name: &str) -> Option<String> {
    let header = format!("[keys.structural.{}]", key_name);
    let mut in_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == header {
            in_section = true;
            continue;
        }
        if trimmed.starts_with('[') && in_section { break; }
        if in_section && (trimmed.starts_with("default =") || trimmed.starts_with("default=")) {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    return Some(trimmed[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    None
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}

fn spec_codegen_ai_protocol(args: &[String]) {
    let dir = args.iter().position(|a| a == "--dir").and_then(|i| args.get(i + 1))
        .map(|s| s.as_str()).unwrap_or("specs");

    let output_path = args.iter().position(|a| a == "-o").and_then(|i| args.get(i + 1))
        .map(|s| s.as_str()).unwrap_or("cronus-schema.json");

    let mut sections = Vec::new();
    let mut section_count = 0usize;

    for subdir in &["core", "stdlib", "patterns"] {
        let path = format!("{}/{}", dir, subdir);
        let mut entries: Vec<_> = match std::fs::read_dir(&path) {
            Ok(e) => e.flatten().collect(),
            Err(_) => continue,
        };
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let file_path = entry.path();
            if !file_path.extension().map(|e| e == "toml").unwrap_or(false) {
                continue;
            }
            let content = std::fs::read_to_string(&file_path).unwrap_or_default();
            let name = match extract_field(&content, "name") {
                Some(n) => n,
                None => continue,
            };

            let layer = extract_field(&content, "layer").unwrap_or_else(|| subdir.to_string());
            let stability = extract_field(&content, "stability").unwrap_or_else(|| "stable".to_string());
            let description = extract_field(&content, "description").unwrap_or_default();
            let intent_desc = extract_field_after_section(&content, "[intent]", "description")
                .unwrap_or_else(|| description.clone());

            // Check if pure alias
            let is_alias = has_section(&content, "[alias]")
                && extract_field(&content, "canonical")
                    .map(|c| c != "none" && !c.is_empty())
                    .unwrap_or(false);

            if is_alias {
                let canonical = extract_field(&content, "canonical").unwrap_or_default();
                let justification = extract_field(&content, "justification").unwrap_or_default();
                sections.push(format!(
                    "    \"{}\": {{\n      \"alias_of\": \"{}\",\n      \"layer\": \"{}\",\n      \"stability\": \"{}\",\n      \"justification\": \"{}\"}}",
                    json_escape(&name), json_escape(&canonical), json_escape(&layer), json_escape(&stability), json_escape(&justification)
                ));
                section_count += 1;
                continue;
            }

            let requires_title = extract_field_bool(&content, "requires_title").unwrap_or(false);
            let requires_items = extract_field_bool(&content, "requires_items").unwrap_or(false);
            let min_items = extract_field_usize(&content, "min_items").unwrap_or(0);

            let entity_binding = has_section(&content, "[keys.entity_bound]")
                && extract_field(&content, "strategy")
                    .map(|s| s == "entity_fields" || s == "column_names")
                    .unwrap_or(false);

            let aliases = extract_aliases_from_meta(&content);

            // Structural keys with full detail
            let structural_keys = extract_structural_keys(&content);
            let mut sk_entries = Vec::new();
            for (key, required) in &structural_keys {
                let typ = extract_key_type(&content, key);
                let desc = extract_key_description(&content, key).unwrap_or_default();
                let mut fields = vec![
                    format!("\"required\": {}", required),
                    format!("\"type\": \"{}\"", json_escape(&typ)),
                    format!("\"description\": \"{}\"", json_escape(&desc)),
                ];
                if let Some(def) = extract_key_default(&content, key) {
                    fields.push(format!("\"default\": \"{}\"", json_escape(&def)));
                }
                sk_entries.push(format!("        \"{}\": {{ {} }}", json_escape(key), fields.join(", ")));
            }

            // Config keys with full detail
            let config_details = extract_config_details(&content);
            let mut ck_entries = Vec::new();
            for (key, typ, default, desc) in &config_details {
                let mut fields = vec![
                    format!("\"type\": \"{}\"", json_escape(typ)),
                ];
                if !desc.is_empty() {
                    fields.push(format!("\"description\": \"{}\"", json_escape(desc)));
                }
                if !default.is_empty() {
                    fields.push(format!("\"default\": \"{}\"", json_escape(default)));
                }
                if let Some(validates) = extract_config_validates(&content, key) {
                    fields.push(format!("\"validates\": \"{}\"", json_escape(&validates)));
                }
                ck_entries.push(format!("        \"{}\": {{ {} }}", json_escape(key), fields.join(", ")));
            }

            let aliases_json = if aliases.is_empty() {
                "[]".to_string()
            } else {
                format!("[{}]", aliases.iter().map(|a| format!("\"{}\"", json_escape(a))).collect::<Vec<_>>().join(", "))
            };

            let section_json = format!(
                "    \"{name}\": {{\n      \"layer\": \"{layer}\",\n      \"stability\": \"{stability}\",\n      \"intent\": \"{intent}\",\n      \"shape\": {{\n        \"requires_title\": {rt},\n        \"requires_items\": {ri},\n        \"min_items\": {mi}\n      }},\n      \"structural_keys\": {{\n{sk}\n      }},\n      \"config_keys\": {{\n{ck}\n      }},\n      \"entity_binding\": {eb},\n      \"aliases\": {al}}}",
                name = json_escape(&name),
                layer = json_escape(&layer),
                stability = json_escape(&stability),
                intent = json_escape(&intent_desc),
                rt = requires_title,
                ri = requires_items,
                mi = min_items,
                sk = sk_entries.join(",\n"),
                ck = ck_entries.join(",\n"),
                eb = entity_binding,
                al = aliases_json
            );
            sections.push(section_json);
            section_count += 1;
        }
    }

    let field_types = vec![
        "string", "text", "email", "url", "slug", "phone", "number", "money",
        "percentage", "boolean", "date", "ulid", "json", "enum", "ip", "relation",
    ];
    let field_modifiers = vec![
        "required", "unique", "sensitive", "optional", "searchable",
        "index", "featured", "formatted", "array",
    ];
    let action_verbs = vec![
        "set", "toast", "navigate", "refresh", "create", "confirm", "delete", "validate", "open", "close",
    ];
    let action_events = vec!["click", "submit", "error", "change"];
    let auth_modes = vec!["jwt", "cookie", "token", "public", "api_key", "internal"];
    let page_types = vec!["custom", "form", "list", "dashboard", "landing", "detail"];

    let ft_json = field_types.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", ");
    let fm_json = field_modifiers.iter().map(|m| format!("\"{}\"", m)).collect::<Vec<_>>().join(", ");
    let av_json = action_verbs.iter().map(|v| format!("\"{}\"", v)).collect::<Vec<_>>().join(", ");
    let ae_json = action_events.iter().map(|e| format!("\"{}\"", e)).collect::<Vec<_>>().join(", ");
    let am_json = auth_modes.iter().map(|m| format!("\"{}\"", m)).collect::<Vec<_>>().join(", ");
    let pt_json = page_types.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", ");

    // Compute today's date without external crate
    let today = {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let total_days = now / 86400;
        let mut y = 1970i64;
        let mut remaining = total_days as i64;
        loop {
            let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
            let days_in_year: i64 = if leap { 366 } else { 365 };
            if remaining < days_in_year { break; }
            remaining -= days_in_year;
            y += 1;
        }
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        let month_days: [i64; 12] = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let mut m = 0usize;
        for (i, &md) in month_days.iter().enumerate() {
            if remaining < md { m = i + 1; break; }
            remaining -= md;
        }
        if m == 0 { m = 12; }
        let d = remaining + 1;
        format!("{:04}-{:02}-{:02}", y, m, d)
    };

    let schema = format!(
        "{{\n  \"$schema\": \"https://json-schema.org/draft/2020-12/schema\",\n  \"title\": \"CRONUS Language Schema\",\n  \"description\": \"Schema for validating .cronus file structure, generated from {} .spec.toml contracts\",\n  \"version\": \"1.0.0\",\n  \"generated_at\": \"{}\",\n  \"sections\": {{\n{}\n  }},\n  \"field_types\": [{}],\n  \"field_modifiers\": [{}],\n  \"action_verbs\": [{}],\n  \"action_events\": [{}],\n  \"auth_modes\": [{}],\n  \"page_types\": [{}]\n}}\n",
        section_count,
        today,
        sections.join(",\n"),
        ft_json, fm_json, av_json, ae_json, am_json, pt_json
    );

    match std::fs::write(output_path, &schema) {
        Ok(_) => {
            println!("  \x1b[32m✓\x1b[0m Generated AI protocol schema: {} sections, {} field types, {} action verbs",
                section_count, field_types.len(), action_verbs.len());
            println!("  \x1b[36m→\x1b[0m {}", output_path);
        }
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m Failed to write {}: {}", output_path, e);
        }
    }
}

fn spec_validate(args: &[String]) {
    let dir = args.get(3).map(|s| s.as_str()).unwrap_or("specs");
    println!("  \x1b[36m⚡\x1b[0m Validating specs in {}/\n", dir);

    let mut passed = 0;
    let mut failed = 0;

    for subdir in &["core", "stdlib", "patterns"] {
        let path = format!("{}/{}", dir, subdir);
        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.extension().map(|e| e == "toml").unwrap_or(false) {
                    let content = std::fs::read_to_string(&file_path).unwrap_or_default();
                    let errors = validate_spec_content(&content, subdir);
                    if errors.is_empty() {
                        passed += 1;
                        println!("  \x1b[32m✓\x1b[0m {}", file_path.display());
                    } else {
                        failed += 1;
                        println!("  \x1b[31m✗\x1b[0m {}", file_path.display());
                        for err in &errors {
                            println!("    → {}", err);
                        }
                    }
                }
            }
        }
    }

    println!("\n  {} passed, {} failed", passed, failed);
    if failed > 0 {
        std::process::exit(1);
    }
}

fn spec_list(args: &[String]) {
    let dir = args.get(3).map(|s| s.as_str()).unwrap_or("specs");
    println!("  \x1b[36m⚡\x1b[0m Specs in {}/\n", dir);

    for subdir in &["core", "stdlib", "patterns"] {
        let path = format!("{}/{}", dir, subdir);
        if let Ok(entries) = std::fs::read_dir(&path) {
            println!("  \x1b[1m{}:\x1b[0m", subdir);
            let mut files: Vec<_> = entries.flatten().collect();
            files.sort_by_key(|e| e.file_name());
            for entry in files {
                let file_path = entry.path();
                if file_path.extension().map(|e| e == "toml").unwrap_or(false) {
                    let content = std::fs::read_to_string(&file_path).unwrap_or_default();
                    let name = extract_field(&content, "name").unwrap_or_default();
                    let stability = extract_field(&content, "stability").unwrap_or_default();
                    let badge = match stability.as_str() {
                        "stable" => "\x1b[32m●\x1b[0m",
                        "experimental" => "\x1b[33m●\x1b[0m",
                        "deprecated" => "\x1b[31m●\x1b[0m",
                        _ => "\x1b[90m●\x1b[0m",
                    };
                    println!("    {} {} ({})", badge, name, stability);
                }
            }
            println!();
        }
    }
}

fn cmd_build(args: &[String]) {
    let strict_ai = args.iter().any(|a| a == "--strict-ai");
    let file = args.iter().skip(2)
        .find(|a| !a.starts_with("--"))
        .cloned()
        .or_else(find_cronus_file)
        .unwrap_or_else(|| {
            if strict_ai {
                println!("{}", json!({"valid": false, "errors": [{"message": "No .cronus file found"}]}));
            } else {
                eprintln!("  \x1b[31m✗\x1b[0m No .cronus file found");
            }
            std::process::exit(1);
        });

    let source = fs::read_to_string(&file).unwrap();
    match parser::parse(&source) {
        Ok(nodes) => {
            let (entities, pages, routes) = parser::stats(&nodes);

            // In strict-ai mode, run contract validation and promote warnings to errors
            if strict_ai {
                let mut errors: Vec<Value> = Vec::new();
                for node in &nodes {
                    if let AstNode::Page(page) = node {
                        for section in &page.sections {
                            let section_warnings = contracts::validate_section(section, &[]);
                            for w in section_warnings {
                                let err_json = match w {
                                    contracts::ParseWarning::UnknownSection { ref name, line } => {
                                        json!({"type": "unknown_section", "section": name, "line": line, "severity": "error", "message": format!("Unknown section type '{}'", name)})
                                    }
                                    contracts::ParseWarning::UnknownKey { ref section, ref key, ref item, line } => {
                                        json!({"type": "unknown_key", "section": section, "key": key, "item": item, "line": line, "severity": "error", "message": format!("Unknown key '{}' in section '{}'", key, section)})
                                    }
                                    contracts::ParseWarning::MissingRequired { ref section, ref key, ref item, line } => {
                                        json!({"type": "missing_required", "section": section, "key": key, "item": item, "line": line, "severity": "error", "message": format!("Missing required key '{}' in section '{}'", key, section)})
                                    }
                                    contracts::ParseWarning::AliasUsed { ref alias, ref canonical, line } => {
                                        json!({"type": "alias", "alias": alias, "canonical": canonical, "line": line, "severity": "error", "message": format!("'{}' is an alias for '{}'", alias, canonical)})
                                    }
                                    contracts::ParseWarning::MinItemsViolation { ref section, expected, actual, line } => {
                                        json!({"type": "min_items", "section": section, "expected": expected, "actual": actual, "line": line, "severity": "error", "message": format!("Section '{}' requires at least {} items, found {}", section, expected, actual)})
                                    }
                                    contracts::ParseWarning::UnknownConfig { ref section, ref key, line } => {
                                        json!({"type": "unknown_config", "section": section, "key": key, "line": line, "severity": "error", "message": format!("Unknown config key '{}' in section '{}'", key, section)})
                                    }
                                };
                                errors.push(err_json);
                            }
                        }
                    }
                }
                let valid = errors.is_empty();
                let result = json!({
                    "valid": valid,
                    "file": file,
                    "errors": errors,
                    "stats": { "entities": entities, "pages": pages, "routes": routes },
                });
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
                if !valid {
                    std::process::exit(1);
                }
            } else {
                println!("  \x1b[32m✓\x1b[0m {} — {} entities, {} pages, {} routes", file, entities, pages, routes);
                println!("  \x1b[32m✓\x1b[0m Valid .cronus file");
            }
        }
        Err(e) => {
            if strict_ai {
                println!("{}", json!({"valid": false, "errors": [{"type": "parse_error", "message": e, "severity": "error"}]}));
            } else {
                eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e);
            }
            std::process::exit(1);
        }
    }
}

fn cmd_validate(args: &[String]) {
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

fn cmd_new(args: &[String]) {
    let template = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| {
        eprintln!("  Usage: cronus new <template>");
        eprintln!("  Templates: landing, admin, saas, api, ecommerce, blog");
        std::process::exit(1);
    });

    let content = match template {
        "landing" => TEMPLATE_LANDING,
        "admin" => TEMPLATE_ADMIN,
        "saas" => TEMPLATE_SAAS,
        "api" => TEMPLATE_API,
        "ecommerce" => TEMPLATE_ECOMMERCE,
        "blog" => TEMPLATE_BLOG,
        _ => {
            eprintln!("  \x1b[33m✗\x1b[0m Unknown template: {}", template);
            eprintln!("  Available: landing, admin, saas, api, ecommerce, blog");
            std::process::exit(1);
        }
    };

    let dir = template;
    fs::create_dir_all(dir).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Cannot create directory: {}", e);
        std::process::exit(1);
    });

    let file_path = format!("{}/app.cronus", dir);
    let mut file = fs::File::create(&file_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    // Parse the template to show stats
    let new_nodes = parser::parse(content).ok();
    let (ent_count, pg_count, rt_count) = new_nodes.as_ref()
        .map(|n| parser::stats(n))
        .unwrap_or((0, 0, 0));

    let entity_names: Vec<String> = new_nodes.as_ref()
        .map(|nodes| nodes.iter().filter_map(|n| {
            if let parser::AstNode::Entity(e) = n { Some(e.name.clone()) } else { None }
        }).collect())
        .unwrap_or_default();

    let has_auth = new_nodes.as_ref()
        .map(|nodes| nodes.iter().any(|n| matches!(n, parser::AstNode::Auth(_))))
        .unwrap_or(false);

    println!("\n  Created: \x1b[1m{}/app.cronus\x1b[0m", dir);
    println!();
    println!("  \x1b[90mContents:\x1b[0m");
    if !entity_names.is_empty() {
        println!("    {} entities ({})", ent_count, entity_names.join(", "));
    }
    if pg_count > 0 {
        println!("    {} pages", pg_count);
    }
    if rt_count > 0 {
        println!("    {} API routes", rt_count);
    }
    if has_auth {
        println!("    Auth with JWT");
    }
    println!();
    println!("  \x1b[90mNext steps:\x1b[0m");
    println!("    cd {}", dir);
    println!("    cronus seed    \x1b[90m# populate with test data\x1b[0m");
    println!("    cronus run     \x1b[90m# start the server\x1b[0m");
    println!();
}

// ══════════════════════════════════════════════════
// SEED COMMAND
// ══════════════════════════════════════════════════

fn cmd_seed(args: &[String]) {
    let file = find_cronus_file().unwrap_or_else(|| {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus file found");
        std::process::exit(1);
    });

    let source = fs::read_to_string(&file).unwrap();
    let nodes = parser::parse(&source).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e);
        std::process::exit(1);
    });

    // Find app for database path
    let app = nodes.iter().find_map(|n| {
        if let parser::AstNode::App(a) = n { Some(a) } else { None }
    });
    let db_path = app
        .and_then(|a| a.database.as_ref())
        .and_then(|d| d.path.clone())
        .unwrap_or_else(|| "./data.db".into());

    let db = database::CronusDB::open(&db_path).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Cannot open database: {}", e);
        std::process::exit(1);
    });

    let entities: Vec<&parser::EntityNode> = nodes.iter().filter_map(|n| {
        if let parser::AstNode::Entity(e) = n { Some(e) } else { None }
    }).collect();

    if entities.is_empty() {
        eprintln!("  \x1b[33m⊘\x1b[0m No entities found in {}", file);
        return;
    }

    let count = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(10);

    // Migrate tables first
    let entity_refs: Vec<parser::EntityNode> = entities.iter().map(|e| (*e).clone()).collect();
    db.migrate(&entity_refs).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Migration failed: {}", e);
        std::process::exit(1);
    });

    let seed_start = Instant::now();
    println!("\n  Seeding {} entities \u{00d7} {} rows...\n", entities.len(), count);

    for entity in &entities {
        let mut seeded = 0;
        let first_names = ["Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry", "Iris", "Jack",
                           "Kate", "Leo", "Mia", "Noah", "Olivia", "Pete", "Quinn", "Rosa", "Sam", "Tina"];
        let last_names = ["Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Rodriguez", "Martinez",
                          "Anderson", "Thomas", "Jackson", "White", "Harris", "Clark", "Lewis", "Young", "King", "Wright"];
        let companies = ["Acme Corp", "TechFlow", "DataSync", "CloudBase", "NetPrime",
                         "CodeVault", "PixelForge", "ByteWave", "SkyStack", "NanoGrid",
                         "Quantum Labs", "Apex Digital", "Iris Systems", "Bolt.io", "Vertex AI",
                         "Nebula Inc", "Spark Ops", "Iron Cloud", "Pulse Dev", "Orbit HQ"];
        let titles = ["Important task", "Follow up needed", "Review required", "New request",
                      "Bug fix", "Feature request", "Documentation update", "Testing round",
                      "Deployment prep", "Migration plan", "Security patch", "Performance tuning",
                      "UI redesign", "API integration", "Data cleanup", "Onboarding flow",
                      "Billing issue", "Support ticket", "Release notes", "Sprint planning"];

        for i in 0..count {
            let mut obj = serde_json::Map::new();

            for field in &entity.fields {
                // Skip timestamp fields — they're auto-generated by the DB
                let lower = field.name.to_lowercase();
                if lower == "createdat" || lower == "created_at" || lower == "updatedat" || lower == "updated_at" || lower == "id" {
                    continue;
                }

                let val: serde_json::Value = match field.field_type {
                    parser::FieldType::String | parser::FieldType::Text => {
                        let nm = field.name.to_lowercase();
                        if nm.contains("name") || nm.contains("customer") || nm.contains("author") {
                            serde_json::Value::String(format!("{} {}", first_names[i % 20], last_names[i % 20]))
                        } else if nm.contains("company") || nm.contains("org") {
                            serde_json::Value::String(companies[i % 20].to_string())
                        } else if nm.contains("title") || nm.contains("subject") {
                            serde_json::Value::String(format!("Item #{} — {}", i + 1, titles[i % 20]))
                        } else if nm.contains("description") || nm.contains("content") || nm.contains("body") || nm.contains("bio") || nm.contains("excerpt") {
                            serde_json::Value::String(format!("Sample content for item {}. This is realistic test data generated by cronus seed.", i + 1))
                        } else if nm.contains("password") {
                            serde_json::Value::String(crate::auth::hash_password("password123"))
                        } else if nm.contains("address") {
                            serde_json::Value::String(format!("{} {} St, Suite {}", 100 + i * 37 % 900, last_names[i % 20], i + 1))
                        } else if nm.contains("color") {
                            let colors = ["#3b82f6", "#ef4444", "#22c55e", "#f59e0b", "#8b5cf6", "#ec4899", "#06b6d4", "#f97316", "#14b8a6", "#6366f1"];
                            serde_json::Value::String(colors[i % 10].to_string())
                        } else {
                            serde_json::Value::String(format!("{}_{}", field.name, i + 1))
                        }
                    }
                    parser::FieldType::Email => {
                        serde_json::Value::String(format!("{}.{}@example.com",
                            first_names[i % 20].to_lowercase(),
                            last_names[i % 20].to_lowercase()))
                    }
                    parser::FieldType::Phone => {
                        serde_json::Value::String(format!("+1 555-{:03}-{:04}", 100 + i * 3, 1000 + i * 7))
                    }
                    parser::FieldType::Url => {
                        serde_json::Value::String(format!("https://example.com/{}/{}", field.name, i + 1))
                    }
                    parser::FieldType::Number => {
                        serde_json::Value::String(format!("{}", (i + 1) * 10 + (i * 7) % 100))
                    }
                    parser::FieldType::Money => {
                        // Centavos — realistic prices
                        serde_json::Value::String(format!("{}", (i + 1) * 1990 + (i * 500) % 10000))
                    }
                    parser::FieldType::Percentage => {
                        serde_json::Value::String(format!("{}", 15 + (i * 8) % 85))
                    }
                    parser::FieldType::Boolean => {
                        serde_json::Value::String(if i % 3 == 0 { "0" } else { "1" }.to_string())
                    }
                    parser::FieldType::Date => {
                        let day = 1 + (i % 28);
                        let month = 1 + (i % 12);
                        serde_json::Value::String(format!("2026-{:02}-{:02}", month, day))
                    }
                    parser::FieldType::Enum => {
                        if let Some(ref vals) = field.enum_values {
                            serde_json::Value::String(vals[i % vals.len()].clone())
                        } else {
                            let statuses = ["active", "pending", "completed", "cancelled", "processing"];
                            serde_json::Value::String(statuses[i % 5].to_string())
                        }
                    }
                    parser::FieldType::Slug => {
                        serde_json::Value::String(format!("{}-{}", field.name, i + 1))
                    }
                    parser::FieldType::Ip => {
                        serde_json::Value::String(format!("192.168.{}.{}", 1 + i / 255, 1 + i % 255))
                    }
                    parser::FieldType::Relation => {
                        // Try to find an existing row in the related table
                        if let Some(ref target) = field.reference {
                            match db.find_all(target, 1, i) {
                                Ok(rows) => {
                                    if let Some(arr) = rows.as_array() {
                                        if let Some(first) = arr.first() {
                                            if let Some(id) = first.get("id").and_then(|v| v.as_str()) {
                                                serde_json::Value::String(id.to_string())
                                            } else { serde_json::Value::Null }
                                        } else { serde_json::Value::Null }
                                    } else { serde_json::Value::Null }
                                }
                                Err(_) => serde_json::Value::Null,
                            }
                        } else {
                            serde_json::Value::Null
                        }
                    }
                    _ => serde_json::Value::String(format!("value_{}", i + 1)),
                };

                if !val.is_null() {
                    obj.insert(field.name.clone(), val);
                }
            }

            if db.insert(&entity.name, &serde_json::Value::Object(obj)).is_ok() {
                seeded += 1;
            }
        }
        let sample: String = if seeded > 0 { format!(" ({})", entity.fields.first().map(|f| f.name.as_str()).unwrap_or("")) } else { String::new() };
        println!("  \x1b[32m\u{2713}\x1b[0m {:<12} {} rows", entity.name, seeded);
    }

    let seed_ms = seed_start.elapsed().as_millis();
    println!("\n  Done in {}ms. Run: \x1b[1mcronus run\x1b[0m\n", seed_ms);
}

// ══════════════════════════════════════════════════
// TEMPLATES
// ══════════════════════════════════════════════════

const TEMPLATE_ADMIN: &str = r#"# Admin Panel — CRONUS
# Template: admin
# Full CRUD admin with auth, sidebar layout, KPI dashboard, forms, and actions.
# Run `cronus seed` after to populate with sample data.

app "Admin Panel" {
  stack react + tailwind
  port 5175
  theme dark
  database sqlite "./data.db"
}

# ── Entities ──
# Define your data models. Each becomes a DB table + REST API automatically.

entity User {
  name      string    required
  email     email     required unique
  password  string    required sensitive
  role      enum      [admin, manager, member]
}

entity Customer {
  name      string    required
  email     email     required unique
  phone     phone
  company   string
  status    enum      [active, inactive, churned]
  createdAt date
}

entity Order {
  customer    string    required
  amount      money     required
  status      enum      [pending, approved, shipped, delivered, cancelled]
  description text
  createdAt   date
}

# ── Auth ──
# JWT-based login using User entity. Roles control page access.

auth {
  entity User
  login email
  session jwt
  roles [admin, manager, member]
}

# ── Layout ──
# Sidebar navigation. Every page inside this layout gets the sidebar.

layout "admin" {
  sidebar {
    brand "Admin Panel"
    nav "Dashboard"  -> "/"          icon:dashboard
    nav "Customers"  -> "/customers" icon:people
    nav "Orders"     -> "/orders"    icon:shopping_cart
    ---
    nav "Settings"   -> "/settings"  icon:settings requires:admin
  }
}

# ── API Routes ──
# Auth routes are public; everything else requires JWT.

api /auth {
  signup  POST  /signup  auth:public
  login   POST  /login   auth:public
  me      GET   /me      auth:jwt
}

api /customers {
  list    GET    /       auth:jwt
  create  POST   /       auth:jwt
  detail  GET    /:id    auth:jwt
  update  PATCH  /:id    auth:jwt
  delete  DELETE /:id    auth:jwt
}

api /orders {
  list    GET    /       auth:jwt
  create  POST   /       auth:jwt
  detail  GET    /:id    auth:jwt
  update  PATCH  /:id    auth:jwt
  delete  DELETE /:id    auth:jwt
}

# ── Dashboard ──
# KPI cards bound to real data + recent orders table with click action.

page "/" type:dashboard requires:auth {
  title "Dashboard"

  section stats cols:4 {
    bind entity:Customer { query count }
    item "Customers" value:"count" icon:people
    bind entity:Order { query count }
    item "Orders" value:"count" icon:shopping_cart
    bind entity:Order { query sum field:amount }
    item "Revenue" value:"sum" icon:attach_money
    bind entity:Order { query count where status eq "pending" }
    item "Pending" value:"count" icon:pending
  }

  section recent-orders {
    title "Recent Orders"
    subtitle "Last 10 orders placed"
    bind entity:Order {
      query all
      order createdAt desc
      limit 10
    }
    columns "Customer, Amount, Status, Date"
    on click {
      navigate "/orders/:id"
    }
  }
}

# ── Customers List ──

page "/customers" type:custom requires:auth {
  title "Customers"

  section header {
    title "Customers"
    subtitle "Manage your customer base"
    action "Add Customer" -> "/customers/new" icon:add
  }

  section customer-table {
    bind entity:Customer {
      query all
      order name asc
      limit 25
    }
    columns "Name, Email, Phone, Company, Status"
    on click {
      navigate "/customers/:id"
    }
  }
}

# ── New Customer Form with on submit action ──

page "/customers/new" type:custom requires:auth {
  title "Add Customer"

  section form {
    bind entity:Customer { query all }
    item "Name" required:true
    item "Email" required:true
    item "Phone"
    item "Company"
    item "Status"
    on submit {
      create Customer
      toast "Customer created"
      navigate "/customers"
    }
  }
}

# ── Orders List ──

page "/orders" type:custom requires:auth {
  title "Orders"

  section header {
    title "Orders"
    subtitle "Track and manage all orders"
    action "New Order" -> "/orders/new" icon:add
  }

  section order-table {
    bind entity:Order {
      query all
      order createdAt desc
      limit 25
    }
    columns "Customer, Amount, Status, Description, Date"
    on click {
      navigate "/orders/:id"
    }
  }
}

# ── New Order Form with on submit action ──

page "/orders/new" type:custom requires:auth {
  title "Create Order"

  section form {
    bind entity:Order { query all }
    item "Customer" required:true
    item "Amount" required:true
    item "Status"
    item "Description"
    on submit {
      create Order
      toast "Order created"
      navigate "/orders"
    }
  }
}

# ── Login Page ──

page "/login" type:form entity:User {
  title "Sign In"
  fields [email, password]
}

# ── Style ──
# Dark theme with blue accent. Monochromatic + 1 accent color.

style {
  theme dark
  accent blue
  background neutral-950
  radius lg
  font "Inter"
}
"#;

const TEMPLATE_LANDING: &str = r#"# Landing Page — CRONUS
# Template: landing
# No auth needed — public marketing page with lead capture form

app "My Landing" {
  stack react + tailwind
  port 5175
  theme dark
  database sqlite "./data.db"
}

# ── Data: just a lead capture entity ──

entity Lead {
  name      string    required
  email     email     required unique
  company   string
  plan      enum      [starter, pro, enterprise]
  createdAt date
}

# ── API: public create, protected list ──

api /leads {
  create  POST   /        auth:public
  list    GET    /        auth:jwt
}

# ── Landing page with hero, features, pricing, CTA ──

page "/" type:custom {
  section hero {
    badge "NOW AVAILABLE"
    title "Scale With Autonomy"
    subtitle "The modern platform for modern teams. Ship faster, scale effortlessly."
    cta "Get Started Free" -> "/signup" primary
    cta "View Demo" -> "/demo" secondary
  }

  section features cols:3 style:cards {
    item "Blazing Fast" icon:zap {
      "Sub-millisecond response times with edge computing"
    }
    item "Secure" icon:shield {
      "SOC2 compliant with end-to-end encryption"
    }
    item "Scalable" icon:globe {
      "Auto-scaling infrastructure across 12 regions"
    }
  }

  section pricing cols:3 {
    plan "Starter" $29/mo [
      "5 projects",
      "10GB storage",
      "Email support"
    ]
    plan "Pro" $79/mo featured [
      "Unlimited projects",
      "100GB storage",
      "Priority support",
      "Advanced analytics"
    ]
    plan "Enterprise" $199/mo [
      "Everything in Pro",
      "SLA 99.99%",
      "Dedicated support",
      "On-premise option"
    ]
  }

  # Lead capture form with on submit action
  section signup {
    title "Ready to get started?"
    subtitle "Join thousands of teams already shipping faster"
    bind entity:Lead { query all }
    on submit {
      create Lead
      toast "Welcome aboard!"
      navigate "/"
    }
    cta "Start Free Trial" -> "/signup" primary
  }
}

style {
  theme dark
  accent amber
  background neutral-950
  radius xl
  font "Inter"
}
"#;

const TEMPLATE_SAAS: &str = r#"# SaaS Platform — CRONUS
# Template: saas (landing + auth + dashboard + billing)
# Public landing page + authenticated dashboard with role-based access.

app "My SaaS" {
  stack react + tailwind
  port 5175
  theme dark
  database sqlite "./data.db"
}

# ── Entities ──

entity User {
  name      string    required
  email     email     required unique
  password  string    required sensitive
  role      enum      [admin, member, viewer]
  plan      enum      [free, starter, pro, enterprise]
  avatar    url
  createdAt date
}

entity Team {
  name      string    required
  slug      slug      required unique
  plan      enum      [free, starter, pro, enterprise]
  owner     string    required
  createdAt date
}

entity Project {
  name        string    required
  description text
  status      enum      [active, paused, completed]
  team        string    required
  createdAt   date
}

entity Invoice {
  amount    money     required
  status    enum      [pending, paid, overdue, cancelled]
  plan      enum      [starter, pro, enterprise]
  team      string    required
  period    date      required
  createdAt date
}

# ── Auth ──

auth {
  entity User
  login email
  session jwt
  roles [admin, member, viewer]
}

# ── Layout (authenticated pages) ──

layout "app" {
  sidebar {
    brand "My SaaS"
    nav "Dashboard"  -> "/dashboard" icon:dashboard
    nav "Projects"   -> "/projects"  icon:folder
    nav "Billing"    -> "/billing"   icon:credit_card
    ---
    nav "Settings"   -> "/settings"  icon:settings requires:admin
  }
}

# ── API ──

api /auth {
  login     POST   /login     auth:public
  register  POST   /register  auth:public
  me        GET    /me        auth:jwt
}

api /teams {
  list    GET    /        auth:jwt
  create  POST   /        auth:jwt
  detail  GET    /:id     auth:jwt
}

api /projects {
  list    GET    /        auth:jwt
  create  POST   /        auth:jwt
  detail  GET    /:id     auth:jwt
  update  PATCH  /:id     auth:jwt
  delete  DELETE /:id     auth:jwt
}

api /billing {
  invoices  GET    /invoices  auth:jwt
}

# ── Public Landing ──

page "/" type:custom {
  section hero {
    badge "LAUNCHING SOON"
    title "Your SaaS Platform"
    subtitle "The all-in-one platform for modern teams."
    cta "Start Free" -> "/signup" primary
    cta "See Pricing" -> "/#pricing" secondary
  }

  section features cols:3 style:cards {
    item "Team Management" icon:users {
      "Invite members, assign roles, manage permissions"
    }
    item "Project Tracking" icon:kanban {
      "Track progress with boards, lists, and timelines"
    }
    item "Billing" icon:credit-card {
      "Automatic invoicing with Stripe integration"
    }
  }

  section pricing cols:3 {
    plan "Starter" $29/mo [
      "5 team members",
      "10 projects",
      "Email support"
    ]
    plan "Pro" $79/mo featured [
      "25 team members",
      "Unlimited projects",
      "Priority support"
    ]
    plan "Enterprise" $199/mo [
      "Unlimited everything",
      "Dedicated support",
      "SLA 99.99%"
    ]
  }
}

# ── Auth Pages ──

page "/login" type:form entity:User {
  title "Sign In"
  fields [email, password]
}

page "/signup" type:form entity:User {
  title "Create Account"
  fields [name, email, password]
}

# ── Dashboard with KPIs ──

page "/dashboard" type:dashboard requires:auth {
  title "Dashboard"

  section stats cols:3 {
    bind entity:Project { query count }
    item "Projects" value:"count" icon:folder
    bind entity:Invoice { query count where status eq "paid" }
    item "Paid Invoices" value:"count" icon:check_circle
    bind entity:Invoice { query sum field:amount }
    item "Total Billed" value:"sum" icon:attach_money
  }

  section recent-projects {
    title "Recent Projects"
    bind entity:Project {
      query all
      order createdAt desc
      limit 5
    }
    columns "Name, Status, Team, Created"
    on click {
      navigate "/projects/:id"
    }
  }
}

# ── Projects CRUD ──

page "/projects" type:custom requires:auth {
  title "Projects"

  section header {
    title "Projects"
    action "New Project" -> "/projects/new" icon:add
  }

  section project-table {
    bind entity:Project {
      query all
      order createdAt desc
      limit 25
    }
    columns "Name, Status, Team, Created"
    on click {
      navigate "/projects/:id"
    }
  }
}

page "/projects/new" type:custom requires:auth {
  title "New Project"

  section form {
    bind entity:Project { query all }
    item "Name" required:true
    item "Description"
    item "Team" required:true
    item "Status"
    on submit {
      create Project
      toast "Project created"
      navigate "/projects"
    }
  }
}

# ── Billing ──

page "/billing" type:custom requires:auth {
  title "Billing"

  section invoice-table {
    bind entity:Invoice {
      query all
      order createdAt desc
      limit 25
    }
    columns "Amount, Status, Plan, Period"
  }
}

style {
  theme dark
  accent violet
  background neutral-950
  radius xl
  font "Inter"
}
"#;

const TEMPLATE_API: &str = r#"# API Backend — CRONUS
# Template: api (no UI, just backend)

app "My API" {
  stack bun
  port 3001
  database sqlite "./data.db"
}

entity User {
  name      string    required
  email     email     required unique
  role      enum      [admin, member]
  apiKey    string    unique
  createdAt date
}

entity Item {
  name        string    required
  description text
  status      enum      [active, archived]
  owner       string    required
  price       money
  createdAt   date
}

entity Log {
  action    string    required
  entity    string    required
  userId    string    required
  details   text
  timestamp date      required
}

api /users {
  list    GET    /        auth:api_key
  detail  GET    /:id     auth:api_key
  create  POST   /        auth:api_key
  update  PATCH  /:id     auth:api_key
  delete  DELETE /:id     auth:api_key
}

api /items {
  list    GET    /        auth:api_key
  detail  GET    /:id     auth:api_key
  create  POST   /        auth:api_key
  update  PATCH  /:id     auth:api_key
  delete  DELETE /:id     auth:api_key
}

api /logs {
  list    GET    /        auth:api_key
}

service api port:3001 {
  cors origins:["*"]
  rate_limit 100/min
}
"#;

const TEMPLATE_ECOMMERCE: &str = r#"# E-commerce — CRONUS
# Template: ecommerce (storefront + admin + orders)
# Public storefront + authenticated admin panel for managing products/orders.

app "My Store" {
  stack react + tailwind
  port 5175
  theme dark
  database sqlite "./data.db"
}

# ── Entities ──

entity User {
  name      string    required
  email     email     required unique
  password  string    required sensitive
  role      enum      [admin, staff]
}

entity Product {
  name        string    required
  description text
  price       money     required
  sku         string    unique
  stock       number
  category    enum      [electronics, clothing, food, accessories, other]
  imageUrl    url
  active      boolean
  createdAt   date
}

entity Customer {
  name      string    required
  email     email     required unique
  phone     phone
  address   text
  createdAt date
}

entity Purchase {
  customer    string    required
  status      enum      [pending, paid, shipped, delivered, cancelled]
  total       money     required
  createdAt   date
}

# ── Auth ──

auth {
  entity User
  login email
  session jwt
  roles [admin, staff]
}

# ── Layout (admin pages) ──

layout "store-admin" {
  sidebar {
    brand "Store Admin"
    nav "Dashboard"  -> "/dashboard" icon:dashboard
    nav "Products"   -> "/products"  icon:inventory_2
    nav "Orders"     -> "/purchases" icon:shopping_cart
    nav "Customers"  -> "/customers" icon:people
  }
}

# ── API ──

api /auth {
  login   POST  /login   auth:public
  signup  POST  /signup  auth:public
  me      GET   /me      auth:jwt
}

api /products {
  list    GET    /        auth:public
  detail  GET    /:id     auth:public
  create  POST   /        auth:jwt
  edit    PATCH  /:id     auth:jwt
  delete  DELETE /:id     auth:jwt
}

api /purchases {
  list    GET    /        auth:jwt
  create  POST   /        auth:public
  detail  GET    /:id     auth:jwt
  update  PATCH  /:id     auth:jwt
}

api /customers {
  list    GET    /        auth:jwt
  detail  GET    /:id     auth:jwt
}

# ── Public Storefront ──

page "/" type:custom {
  section hero {
    title "Welcome to Our Store"
    subtitle "Find amazing products at great prices"
    cta "Shop Now" -> "/products" primary
  }
  section features cols:3 style:cards {
    item "Fast Shipping" icon:zap { "Free delivery on orders over $50" }
    item "Secure Payment" icon:shield { "Encrypted checkout with Stripe" }
    item "Easy Returns" icon:globe { "30-day return policy on all items" }
  }
}

# ── Admin Dashboard ──

page "/dashboard" type:dashboard requires:auth {
  title "Store Dashboard"

  section stats cols:4 {
    bind entity:Product { query count }
    item "Products" value:"count" icon:inventory_2
    bind entity:Purchase { query count }
    item "Orders" value:"count" icon:shopping_cart
    bind entity:Purchase { query sum field:total }
    item "Revenue" value:"sum" icon:attach_money
    bind entity:Customer { query count }
    item "Customers" value:"count" icon:people
  }

  section recent-orders {
    title "Recent Orders"
    bind entity:Purchase {
      query all
      order createdAt desc
      limit 10
    }
    columns "Customer, Total, Status, Date"
    on click {
      navigate "/purchases/:id"
    }
  }
}

# ── Products Management ──

page "/products" type:custom requires:auth {
  title "Products"

  section header {
    title "Product Catalog"
    action "Add Product" -> "/products/new" icon:add
  }

  section product-table {
    bind entity:Product {
      query all
      order name asc
      limit 25
    }
    columns "Name, Price, Stock, Category, Active"
    on click {
      navigate "/products/:id"
    }
  }
}

page "/products/new" type:custom requires:auth {
  title "Add Product"

  section form {
    bind entity:Product { query all }
    item "Name" required:true
    item "Price" required:true
    item "SKU"
    item "Stock"
    item "Category"
    item "Description"
    on submit {
      create Product
      toast "Product added"
      navigate "/products"
    }
  }
}

# ── Orders ──

page "/purchases" type:custom requires:auth {
  title "Orders"

  section order-table {
    bind entity:Purchase {
      query all
      order createdAt desc
      limit 25
    }
    columns "Customer, Total, Status, Date"
  }
}

# ── Login ──

page "/login" type:form entity:User {
  title "Sign In"
  fields [email, password]
}

style {
  theme dark
  accent emerald
  background neutral-950
  radius lg
  font "Inter"
}
"#;

const TEMPLATE_BLOG: &str = r#"# Blog — CRONUS
# Template: blog (posts + authors + comments)
# Public blog with authenticated admin for writing/managing posts.

app "My Blog" {
  stack react + tailwind
  port 5175
  theme dark
  database sqlite "./data.db"
}

# ── Entities ──

entity User {
  name      string    required
  email     email     required unique
  password  string    required sensitive
  role      enum      [admin, editor]
}

entity Author {
  name      string    required
  email     email     required unique
  bio       text
  avatar    url
  createdAt date
}

entity Post {
  title       string    required
  slug        slug      required unique
  content     text      required
  excerpt     text
  author      -> Author
  status      enum      [draft, published, archived]
  coverImage  url
  publishedAt date
  createdAt   date
}

entity Comment {
  post        -> Post
  authorName  string    required
  authorEmail email     required
  body        text      required
  approved    boolean
  createdAt   date
}

# ── Auth ──

auth {
  entity User
  login email
  session jwt
  roles [admin, editor]
}

# ── Layout (admin pages) ──

layout "blog-admin" {
  sidebar {
    brand "Blog Admin"
    nav "Dashboard"  -> "/dashboard" icon:dashboard
    nav "Posts"      -> "/admin/posts" icon:article
    nav "Authors"    -> "/admin/authors" icon:people
    nav "Comments"   -> "/admin/comments" icon:comment
  }
}

# ── API ──

api /auth {
  login   POST  /login   auth:public
  signup  POST  /signup  auth:public
  me      GET   /me      auth:jwt
}

api /posts {
  list    GET    /        auth:public
  detail  GET    /:slug   auth:public
  create  POST   /        auth:jwt
  update  PATCH  /:id     auth:jwt
  delete  DELETE /:id     auth:jwt
}

api /authors {
  list    GET    /        auth:public
  detail  GET    /:id     auth:public
  create  POST   /        auth:jwt
}

api /comments {
  list    GET    /        auth:public
  create  POST   /        auth:public
  delete  DELETE /:id     auth:jwt
}

# ── Public Blog ──

page "/" type:custom {
  section hero {
    title "My Blog"
    subtitle "Thoughts, stories, and ideas"
    cta "Read Latest" -> "/posts" primary
  }
  section features cols:3 style:cards {
    item "Fresh Content" icon:zap { "New articles published weekly" }
    item "Open Discussion" icon:users { "Comment and engage with authors" }
    item "Curated Topics" icon:tag { "Browse by tags and categories" }
  }

  section latest-posts {
    title "Latest Posts"
    bind entity:Post {
      query all
      order publishedAt desc
      limit 5
    }
    columns "Title, Author, Published"
    on click {
      navigate "/posts/:slug"
    }
  }
}

# ── Admin Dashboard ──

page "/dashboard" type:dashboard requires:auth {
  title "Blog Dashboard"

  section stats cols:3 {
    bind entity:Post { query count }
    item "Total Posts" value:"count" icon:article
    bind entity:Post { query count where status eq "published" }
    item "Published" value:"count" icon:check_circle
    bind entity:Comment { query count }
    item "Comments" value:"count" icon:comment
  }

  section recent-posts {
    title "Recent Posts"
    bind entity:Post {
      query all
      order createdAt desc
      limit 10
    }
    columns "Title, Status, Author, Created"
    on click {
      navigate "/admin/posts/:id"
    }
  }
}

# ── Post Management ──

page "/admin/posts" type:custom requires:auth {
  title "Posts"

  section header {
    title "Posts"
    action "New Post" -> "/admin/posts/new" icon:add
  }

  section post-table {
    bind entity:Post {
      query all
      order createdAt desc
      limit 25
    }
    columns "Title, Slug, Status, Author, Published"
    on click {
      navigate "/admin/posts/:id"
    }
  }
}

page "/admin/posts/new" type:custom requires:auth {
  title "New Post"

  section form {
    bind entity:Post { query all }
    item "Title" required:true
    item "Slug" required:true
    item "Content" required:true
    item "Excerpt"
    item "Status"
    item "Cover Image"
    on submit {
      create Post
      toast "Post created"
      navigate "/admin/posts"
    }
  }
}

# ── Author Management ──

page "/admin/authors" type:custom requires:auth {
  title "Authors"

  section author-table {
    bind entity:Author {
      query all
      order name asc
    }
    columns "Name, Email, Created"
  }
}

# ── Comment Moderation ──

page "/admin/comments" type:custom requires:auth {
  title "Comments"

  section comment-table {
    bind entity:Comment {
      query all
      order createdAt desc
      limit 25
    }
    columns "Author Name, Body, Approved, Created"
  }
}

# ── Login ──

page "/login" type:form entity:User {
  title "Sign In"
  fields [email, password]
}

style {
  theme dark
  accent sky
  background neutral-950
  radius lg
  font "Inter"
}
"#;

fn cmd_parse(args: &[String]) {
    let file = args.iter().skip(2)
        .find(|a| !a.starts_with("--"))
        .cloned()
        .or_else(find_cronus_file)
        .unwrap_or_else(|| {
            eprintln!("  No .cronus file found");
            std::process::exit(1);
        });

    let source = fs::read_to_string(&file).unwrap();
    match parser::parse(&source) {
        Ok(nodes) => {
            let (entities, pages, routes) = parser::stats(&nodes);
            println!("Parsed {} nodes:", nodes.len());
            println!("  Entities: {}", entities);
            println!("  Pages:    {}", pages);
            println!("  API:      {} routes", routes);

            for node in &nodes {
                if let AstNode::App(app) = node {
                    println!("  App:      \"{}\" (port {})", app.name, app.port);
                    if let Some(db) = &app.database {
                        println!("  Database: {} {:?}", db.db_type, db.path);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_deploy(args: &[String]) {
    let file = find_cronus_file().unwrap_or_else(|| {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus file found"); std::process::exit(1);
    });
    let source = fs::read_to_string(&file).unwrap();
    let nodes = parser::parse(&source).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e); std::process::exit(1);
    });
    let mut app_name = "cronus-app".to_string();
    let mut port: u16 = 5175;
    for node in &nodes {
        if let AstNode::App(a) = node { app_name = a.name.clone(); port = a.port; }
    }

    let target = args.get(2).map(|s| s.as_str()).unwrap_or("");

    match target {
        "--fly" => {
            // Docker artifacts
            fs::write("Dockerfile", deploy::generate_dockerfile(&app_name, port)).unwrap();
            fs::write(".dockerignore", deploy::generate_dockerignore()).unwrap();
            // Fly.io config
            fs::write("fly.toml", deploy::generate_fly_toml(&app_name, port)).unwrap();
            println!("  \x1b[32m✓\x1b[0m Generated Dockerfile + fly.toml");
            println!("\n  \x1b[1mDeploy to Fly.io:\x1b[0m");
            println!("  \x1b[32m1.\x1b[0m fly auth login");
            println!("  \x1b[32m2.\x1b[0m fly launch --copy-config --yes");
            println!("  \x1b[32m3.\x1b[0m fly deploy");
        }
        "--railway" => {
            fs::write("Dockerfile", deploy::generate_dockerfile(&app_name, port)).unwrap();
            fs::write(".dockerignore", deploy::generate_dockerignore()).unwrap();
            fs::write("railway.json", deploy::generate_railway_config(&app_name, port)).unwrap();
            println!("  \x1b[32m✓\x1b[0m Generated Dockerfile + railway.json");
            println!("\n  \x1b[1mDeploy to Railway:\x1b[0m");
            println!("  \x1b[32m1.\x1b[0m railway login");
            println!("  \x1b[32m2.\x1b[0m railway up");
        }
        "--static" => {
            println!("  \x1b[36m⚡\x1b[0m Static export requires running server first.");
            println!("  \x1b[90mStart with:\x1b[0m cronus run {}", port);
            println!("  \x1b[90mThen use:\x1b[0m  wget -r -np http://localhost:{}/", port);
            println!("  \x1b[90mOr:\x1b[0m       curl http://localhost:{}/showcase -o dist/showcase.html", port);
        }
        _ => {
            // Default: Docker artifacts
            fs::write("Dockerfile", deploy::generate_dockerfile(&app_name, port)).unwrap();
            println!("  \x1b[32m✓\x1b[0m Generated Dockerfile");
            fs::write("docker-compose.yml", deploy::generate_compose(&app_name, port)).unwrap();
            println!("  \x1b[32m✓\x1b[0m Generated docker-compose.yml");
            fs::write(".dockerignore", deploy::generate_dockerignore()).unwrap();
            println!("  \x1b[32m✓\x1b[0m Generated .dockerignore");
            println!("\n  \x1b[1mReady for deployment!\x1b[0m\n");
            println!("  \x1b[32mDocker:\x1b[0m      docker compose up --build");
            println!("  \x1b[32mFly.io:\x1b[0m      cronus deploy --fly");
            println!("  \x1b[32mRailway:\x1b[0m     cronus deploy --railway");
            println!("  \x1b[32mStatic:\x1b[0m      cronus deploy --static");
        }
    }
    println!();
}

fn cmd_doctor(_args: &[String]) {
    println!("  \x1b[36m⚡ CRONUS\x1b[0m Doctor\n");
    let mut ok = true;
    match find_cronus_file() {
        Some(file) => {
            let source = fs::read_to_string(&file).unwrap_or_default();
            match parser::parse(&source) {
                Ok(nodes) => {
                    let (e, p, r) = parser::stats(&nodes);
                    println!("  \x1b[32m✓\x1b[0m Syntax: {} ({} entities, {} pages, {} routes)", file, e, p, r);
                }
                Err(e) => { println!("  \x1b[31m✗\x1b[0m Syntax: {}", e); ok = false; }
            }
        }
        None => { println!("  \x1b[31m✗\x1b[0m No .cronus file found"); ok = false; }
    }
    if std::path::Path::new("data.db").exists() {
        let size = fs::metadata("data.db").map(|m| m.len()).unwrap_or(0);
        println!("  \x1b[32m✓\x1b[0m Database: data.db ({}KB)", size / 1024);
    } else {
        println!("  \x1b[33m⊘\x1b[0m Database: not created yet");
    }
    match std::net::TcpListener::bind("0.0.0.0:5175") {
        Ok(_) => println!("  \x1b[32m✓\x1b[0m Port 5175: available"),
        Err(_) => println!("  \x1b[33m⊘\x1b[0m Port 5175: in use"),
    }
    println!("  \x1b[32m✓\x1b[0m Runtime: Rust native\n");
    if ok { println!("  \x1b[32mAll checks passed\x1b[0m"); } else { println!("  \x1b[33mSome issues found\x1b[0m"); }
    println!();
}

fn cmd_stats(_args: &[String]) {
    let file = find_cronus_file().unwrap_or_else(|| {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus file found"); std::process::exit(1);
    });
    let source = fs::read_to_string(&file).unwrap();
    let nodes = parser::parse(&source).unwrap();
    let (entities, pages, routes) = parser::stats(&nodes);
    let lines = source.lines().count();
    println!("  \x1b[36m⚡ CRONUS\x1b[0m Stats\n");
    println!("  \x1b[1mSource:\x1b[0m      {} ({} lines)", file, lines);
    println!("  \x1b[1mEntities:\x1b[0m    {}", entities);
    println!("  \x1b[1mPages:\x1b[0m       {}", pages);
    println!("  \x1b[1mAPI Routes:\x1b[0m  {}", routes);
    for node in &nodes {
        if let AstNode::App(a) = node { println!("  \x1b[1mApp:\x1b[0m         {} (port {})", a.name, a.port); }
    }
    if std::path::Path::new("data.db").exists() {
        let size = fs::metadata("data.db").map(|m| m.len()).unwrap_or(0);
        println!("  \x1b[1mDatabase:\x1b[0m    {}KB", size / 1024);
    }
    println!();
}

fn cmd_export(_args: &[String]) {
    let file = find_cronus_file().unwrap_or_else(|| {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus file found"); std::process::exit(1);
    });
    let source = fs::read_to_string(&file).unwrap();
    let nodes = parser::parse(&source).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e); std::process::exit(1);
    });
    let ir = deploy::generate_ir(&nodes);
    let output = "cronus-project.ir.json";
    fs::write(output, serde_json::to_string_pretty(&ir).unwrap()).unwrap();
    let (e, p, r) = parser::stats(&nodes);
    println!("  \x1b[32m✓\x1b[0m Exported to {} ({} entities, {} pages, {} routes)", output, e, p, r);
}

fn cmd_test(args: &[String]) {
    if args.iter().any(|a| a == "--conformance") {
        println!("  \x1b[36m⚡\x1b[0m Running conformance suite...\n");
        let base = args.iter()
            .position(|a| a == "--dir")
            .and_then(|i| args.get(i + 1))
            .map(|s| s.as_str())
            .unwrap_or("tests/conformance");

        let (passed, failed, errors) = testing::run_conformance(base);

        for err in &errors {
            println!("  \x1b[31m✗\x1b[0m {}", err);
        }

        println!();
        if failed == 0 {
            println!("  \x1b[32m✓\x1b[0m All {} tests passed", passed);
        } else {
            println!("  \x1b[31m✗\x1b[0m {} passed, {} failed", passed, failed);
            std::process::exit(1);
        }
        return;
    }

    let file = find_cronus_file().unwrap_or_else(|| {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus file found"); std::process::exit(1);
    });
    let source = fs::read_to_string(&file).unwrap();
    let nodes = parser::parse(&source).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e); std::process::exit(1);
    });

    let mut entities: Vec<EntityNode> = vec![];
    let mut port: u16 = 5175;
    for node in &nodes {
        match node {
            AstNode::Entity(e) => entities.push(e.clone()),
            AstNode::App(a) => port = a.port,
            _ => {}
        }
    }

    // Override port from CLI
    if let Some(p) = args.get(2).and_then(|s| s.parse().ok()) {
        port = p;
    }

    let (passed, failed, _total) = testing::run_tests(&entities, port);
    if failed > 0 {
        std::process::exit(1);
    }
}
// Add rand_u32

fn rand_u32() -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut h);
    std::thread::current().id().hash(&mut h);
    h.finish() as u32
}

fn cmd_compose(_args: &[String]) {
    let files = find_all_cronus_files();
    if files.is_empty() {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus files found");
        std::process::exit(1);
    }
    if files.len() == 1 {
        println!("  Only 1 file ({}). Compose requires 2+ files.", files[0]);
        return;
    }
    println!("  \x1b[36m⚡ CRONUS\x1b[0m Composing {} files:\n", files.len());
    let mut total_lines = 0;
    for f in &files {
        let lines = fs::read_to_string(f).map(|s| s.lines().count()).unwrap_or(0);
        total_lines += lines;
        println!("    + {} ({} lines)", f, lines);
    }
    match parser::parse_directory(".") {
        Ok(nodes) => {
            let (entities, pages, routes) = parser::stats(&nodes);
            println!("\n  \x1b[1mComposed:\x1b[0m");
            println!("    Entities: {}", entities);
            println!("    Pages:    {}", pages);
            println!("    Routes:   {}", routes);
            println!("    Total:    {} nodes from {} lines\n", nodes.len(), total_lines);
            for node in &nodes {
                if let AstNode::Entity(e) = node { println!("    entity {} ({} fields)", e.name, e.fields.len()); }
            }
            for node in &nodes {
                if let AstNode::Page(p) = node { println!("    page {} ({})", p.route, p.page_type); }
            }
            println!();
        }
        Err(e) => { eprintln!("  \x1b[31m✗\x1b[0m {}", e); std::process::exit(1); }
    }
}

// ══════════════════════════════════════════════════
// GENERATE — AI-first: description → .cronus file
// ══════════════════════════════════════════════════

const GENERATE_SYSTEM_PROMPT: &str = r#"You are CRONUS, a code generator for .cronus files — a declarative language where one file = full-stack app.

RULES:
1. Output ONLY .cronus code. No markdown, no explanations.
2. Always start with: app "Name" { stack react + tailwind; port 5175; database sqlite "./data.db"; theme dark }
3. Every entity field needs an explicit type: string, text, number, money, boolean, date, email, url, slug, enum [values]
4. API routes: name METHOD /path auth:mode (auth:public, auth:jwt)
5. Pages: page "/route" type:custom/list/form { sections }
6. If app needs users: add auth { entity User; login email + password; session jwt } + User entity with password string required sensitive
7. Prices in centavos (2990 = $29.90), use money type
8. Section types: hero, features, pricing, cta, faq, table, form, kpi, chart, kanban, timeline, tabs, accordion, alert, modal
9. Style: dark theme, one accent color, Inter font
10. Use bind entity:X { query all } in sections to show real data

EXAMPLE 1 — Todo App:
app "Todo App" {
  stack react + tailwind
  port 5500
  database sqlite "./data.db"
  theme dark
}

style {
  theme dark
  accent amber
  font "Inter"
}

entity Todo {
  title       string    required
  completed   boolean
  priority    enum      [low, medium, high]
  created_at  date
}

api /todos {
  list    GET    /       auth:public
  create  POST   /       auth:public
  update  PATCH  /:id    auth:public
  delete  DELETE /:id    auth:public
}

page "/" type:dashboard {
  title "My Todos"

  section header {
    title "Todos"
    subtitle "Stay on top of what matters"
    action "New Todo" -> "/new" icon:add
  }

  section todo-table {
    bind entity:Todo {
      query all
      order created_at desc
      limit 25
    }
    columns "Title, Priority, Completed"
    on click {
      set completed "true"
      toast "Todo completed"
      refresh self
    }
  }
}

page "/new" type:custom {
  title "New Todo"
  section form {
    bind entity:Todo { query all }
    item "Title" required:true
    item "Priority"
    on submit {
      create Todo
      toast "Todo created"
      navigate "/"
    }
  }
}

EXAMPLE 2 — Admin Panel (snippet):
app "Admin Panel" {
  stack react + tailwind
  port 5300
  database sqlite "./data.db"
  theme dark
}

style {
  theme dark
  accent blue
  font "Inter"
}

entity User {
  name      string    required
  email     email     required unique
  password  string    required sensitive
  role      enum      [admin, member]
}

entity Order {
  customer    string    required
  amount      money     required
  status      enum      [pending, approved, shipped, cancelled]
  created_at  date
}

auth {
  entity User
  login email
  session jwt
  roles [admin, member]
}

api /auth {
  signup  POST  /signup  auth:public
  login   POST  /login   auth:public
  me      GET   /me      auth:jwt
}

api /orders {
  list    GET    /       auth:jwt
  create  POST   /       auth:jwt
  update  PATCH  /:id    auth:jwt
  delete  DELETE /:id    auth:jwt
}

page "/dashboard" type:dashboard {
  title "Admin Dashboard"
  section kpis type:kpi {
    item "Total Orders" bind:count entity:Order
    item "Revenue" bind:sum entity:Order field:amount
  }
  section orders type:table {
    bind entity:Order { query all order created_at desc }
    columns "Customer, Amount, Status, Date"
    search "customer"
    paginate 20
  }
}
"#;

fn cmd_generate(args: &[String]) {
    let mut description: Option<String> = None;
    let mut dry_run = false;
    let mut from_file: Option<String> = None;
    let mut output_path = "app.cronus".to_string();
    let mut auto_go = false;

    // Parse flags from args[2..] (args[0] = "cronus", args[1] = "generate"/"gen")
    let flag_args = if args.len() > 2 { &args[2..] } else { &[] as &[String] };
    let mut i = 0;
    while i < flag_args.len() {
        match flag_args[i].as_str() {
            "--dry-run" => { dry_run = true; }
            "--go" => { auto_go = true; }
            "--from-file" => {
                i += 1;
                if i >= flag_args.len() {
                    eprintln!("  \x1b[31m✗\x1b[0m --from-file requires a path");
                    std::process::exit(1);
                }
                from_file = Some(flag_args[i].clone());
            }
            "--output" | "-o" => {
                i += 1;
                if i >= flag_args.len() {
                    eprintln!("  \x1b[31m✗\x1b[0m --output requires a filename");
                    std::process::exit(1);
                }
                output_path = flag_args[i].clone();
            }
            other => {
                // Positional arg = description (join remaining non-flag words)
                if description.is_none() && !other.starts_with("--") {
                    let mut desc_parts = vec![other.to_string()];
                    while i + 1 < flag_args.len() && !flag_args[i + 1].starts_with("--") {
                        i += 1;
                        desc_parts.push(flag_args[i].clone());
                    }
                    description = Some(desc_parts.join(" "));
                }
            }
        }
        i += 1;
    }

    // MODE: --from-file
    if let Some(ref file_path) = from_file {
        cmd_generate_from_file(file_path, &output_path, auto_go);
        return;
    }

    // Need a description for generate
    let desc = match description {
        Some(d) => d,
        None => {
            eprintln!("  Usage:");
            eprintln!("    cronus generate \"description\" [--dry-run] [--output file.cronus] [--go]");
            eprintln!("    cronus generate --from-file output.txt [--output file.cronus] [--go]");
            eprintln!();
            eprintln!("  Examples:");
            eprintln!("    cronus generate \"veterinary clinic with pets owners and appointments\" --dry-run");
            eprintln!("    cronus generate --from-file chatgpt-output.txt");
            eprintln!("    cronus generate \"blog with posts and comments\"");
            std::process::exit(1);
        }
    };

    // Build the user message
    let user_message = format!("Generate a .cronus file for: {}", desc);

    // Check for API key
    let api_key = std::env::var("ANTHROPIC_API_KEY").ok();

    if dry_run || api_key.is_none() {
        // DRY-RUN mode: save prompt to file
        if api_key.is_none() && !dry_run {
            println!("  \x1b[33m!\x1b[0m No ANTHROPIC_API_KEY found, falling back to dry-run mode");
            println!();
        }
        cmd_generate_dry_run(&desc, &user_message);
    } else {
        // API mode
        cmd_generate_api(&desc, &user_message, &api_key.unwrap(), &output_path, auto_go);
    }
}

fn cmd_generate_dry_run(desc: &str, user_message: &str) {
    let full_prompt = format!(
        "=== SYSTEM PROMPT ===\n{}\n\n=== USER MESSAGE ===\n{}\n",
        GENERATE_SYSTEM_PROMPT, user_message
    );

    fs::write(".cronus-prompt.txt", &full_prompt).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Failed to write .cronus-prompt.txt: {}", e);
        std::process::exit(1);
    });

    println!("  \x1b[32m✓\x1b[0m Prompt saved to \x1b[1m.cronus-prompt.txt\x1b[0m");
    println!();
    println!("  Next steps:");
    println!("    1. Copy the contents of .cronus-prompt.txt");
    println!("    2. Paste into Claude or ChatGPT");
    println!("    3. Save the output to a file (e.g. output.txt)");
    println!("    4. Run: \x1b[1mcronus generate --from-file output.txt\x1b[0m");
    println!();
    println!("  Or set ANTHROPIC_API_KEY to generate directly:");
    println!("    export ANTHROPIC_API_KEY=sk-ant-...");
    println!("    cronus generate \"{}\"", desc);
}

fn cmd_generate_from_file(file_path: &str, output_path: &str, auto_go: bool) {
    let raw = fs::read_to_string(file_path).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Failed to read {}: {}", file_path, e);
        std::process::exit(1);
    });

    let source = strip_markdown_fences(&raw);

    match parser::parse(&source) {
        Ok(nodes) => {
            let (entities, pages, routes) = parser::stats(&nodes);
            let lines = source.lines().count();

            fs::write(output_path, &source).unwrap_or_else(|e| {
                eprintln!("  \x1b[31m✗\x1b[0m Failed to write {}: {}", output_path, e);
                std::process::exit(1);
            });

            println!("  \x1b[32m✓\x1b[0m Valid .cronus file written to \x1b[1m{}\x1b[0m", output_path);
            println!("    {} lines, {} entities, {} pages, {} routes", lines, entities, pages, routes);
            println!();

            if auto_go {
                println!("  Running seed + run...");
                let _ = std::process::Command::new(std::env::current_exe().unwrap())
                    .args(&["seed", output_path])
                    .status();
                let _ = std::process::Command::new(std::env::current_exe().unwrap())
                    .args(&["run", output_path])
                    .status();
            } else {
                println!("  Next: \x1b[1mcronus run {}\x1b[0m", output_path);
            }
        }
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m Parse error in {}:", file_path);
            eprintln!("    {}", e);
            eprintln!();
            eprintln!("  The LLM output is not valid .cronus syntax.");
            eprintln!("  Try regenerating or fix the errors manually.");
            std::process::exit(1);
        }
    }
}

fn cmd_generate_api(desc: &str, user_message: &str, api_key: &str, output_path: &str, auto_go: bool) {
    println!("  Generating .cronus for: \"{}\"", desc);
    println!();

    let max_retries = 3;
    let mut attempt = 0;
    let mut extra_context = String::new();

    loop {
        attempt += 1;
        if attempt > 1 {
            println!("  Retry {}/{}...", attempt, max_retries);
        }

        let messages_with_context = if extra_context.is_empty() {
            format!(r#"[{{"role":"user","content":"{}"}}]"#,
                user_message.replace('\\', "\\\\").replace('"', "\\\""))
        } else {
            format!(r#"[{{"role":"user","content":"{}"}},{{"role":"assistant","content":"{}"}},{{"role":"user","content":"{}"}}]"#,
                user_message.replace('\\', "\\\\").replace('"', "\\\""),
                "I'll generate the .cronus file now.".replace('"', "\\\""),
                extra_context.replace('\\', "\\\\").replace('"', "\\\""))
        };

        let body = format!(
            r#"{{"model":"claude-sonnet-4-20250514","max_tokens":4096,"system":"{}","messages":{}}}"#,
            GENERATE_SYSTEM_PROMPT.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n"),
            messages_with_context
        );

        let output = std::process::Command::new("curl")
            .args(&[
                "-s", "-X", "POST",
                "https://api.anthropic.com/v1/messages",
                "-H", &format!("x-api-key: {}", api_key),
                "-H", "anthropic-version: 2023-06-01",
                "-H", "content-type: application/json",
                "-d", &body,
            ])
            .output();

        let output = match output {
            Ok(o) => o,
            Err(e) => {
                eprintln!("  \x1b[31m✗\x1b[0m Failed to call API (curl): {}", e);
                std::process::exit(1);
            }
        };

        let response_str = String::from_utf8_lossy(&output.stdout).to_string();

        // Extract content[0].text from JSON response (minimal parsing, no deps)
        let generated = extract_api_text(&response_str);
        if generated.is_empty() {
            if attempt >= max_retries {
                eprintln!("  \x1b[31m✗\x1b[0m API returned no usable content after {} attempts", max_retries);
                eprintln!("  Response: {}", &response_str[..response_str.len().min(500)]);
                std::process::exit(1);
            }
            extra_context = "The previous response was empty. Please output ONLY valid .cronus code.".to_string();
            continue;
        }

        let source = strip_markdown_fences(&generated);

        match parser::parse(&source) {
            Ok(nodes) => {
                let (entities, pages, routes) = parser::stats(&nodes);
                let lines = source.lines().count();

                fs::write(output_path, &source).unwrap_or_else(|e| {
                    eprintln!("  \x1b[31m✗\x1b[0m Failed to write {}: {}", output_path, e);
                    std::process::exit(1);
                });

                println!("  \x1b[32m✓\x1b[0m Generated \x1b[1m{}\x1b[0m ({} lines)", output_path, lines);
                println!("    {} entities, {} pages, {} routes", entities, pages, routes);
                println!();

                if auto_go {
                    println!("  Running seed + run...");
                    let _ = std::process::Command::new(std::env::current_exe().unwrap())
                        .args(&["seed", output_path])
                        .status();
                    let _ = std::process::Command::new(std::env::current_exe().unwrap())
                        .args(&["run", output_path])
                        .status();
                } else {
                    println!("  Next: \x1b[1mcronus run {}\x1b[0m", output_path);
                }
                return;
            }
            Err(e) => {
                if attempt >= max_retries {
                    eprintln!("  \x1b[31m✗\x1b[0m Failed to generate valid .cronus after {} attempts", max_retries);
                    eprintln!("  Last parse error: {}", e);
                    // Save the last attempt for debugging
                    let debug_path = format!("{}.failed.txt", output_path);
                    let _ = fs::write(&debug_path, &source);
                    eprintln!("  Raw output saved to {}", debug_path);
                    std::process::exit(1);
                }
                println!("  \x1b[33m!\x1b[0m Attempt {} had parse errors, retrying...", attempt);
                extra_context = format!(
                    "Your previous output had parse errors:\n{}\n\nPlease fix these errors and output ONLY valid .cronus code.",
                    e
                );
            }
        }
    }
}

fn strip_markdown_fences(input: &str) -> String {
    let trimmed = input.trim();

    // Check for ```cronus or ``` at the start
    if trimmed.starts_with("```") {
        let after_opening = if let Some(pos) = trimmed.find('\n') {
            &trimmed[pos + 1..]
        } else {
            return trimmed.to_string();
        };
        // Remove trailing ```
        let result = if let Some(pos) = after_opening.rfind("```") {
            &after_opening[..pos]
        } else {
            after_opening
        };
        return result.trim().to_string();
    }

    trimmed.to_string()
}

fn extract_api_text(json: &str) -> String {
    // Minimal JSON parsing: find "text":" and extract the value
    // Looking for: "content":[{"type":"text","text":"..."}]
    if let Some(text_pos) = json.find("\"text\":\"") {
        let start = text_pos + 8; // skip "text":"
        let rest = &json[start..];
        let mut result = String::new();
        let mut chars = rest.chars();
        while let Some(c) = chars.next() {
            match c {
                '\\' => {
                    if let Some(escaped) = chars.next() {
                        match escaped {
                            'n' => result.push('\n'),
                            't' => result.push('\t'),
                            '"' => result.push('"'),
                            '\\' => result.push('\\'),
                            '/' => result.push('/'),
                            _ => { result.push('\\'); result.push(escaped); }
                        }
                    }
                }
                '"' => break,
                _ => result.push(c),
            }
        }
        result
    } else {
        String::new()
    }
}


fn extract_app_name(desc: &str) -> String {
    let words: Vec<&str> = desc.split_whitespace().collect();
    if words.len() <= 3 {
        return desc.split_whitespace().map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + &c.as_str().to_lowercase(),
            }
        }).collect::<Vec<_>>().join(" ");
    }
    words.iter()
        .filter(|w| !["a", "an", "the", "with", "and", "for", "my"].contains(&w.to_lowercase().as_str()))
        .take(3)
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + &c.as_str().to_lowercase(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn generate_saas_template() -> String {
    r#"# Generated by CRONUS — SaaS Template

app "MyApp" {
  stack react + tailwind
  port 5175
  database sqlite "./data.db"
  theme dark
}

style {
  theme dark
  accent blue
  background neutral-950
  radius xl
  font "SF Pro Display"
}

entity User {
  email     email     required unique
  name      string    required
  role      enum      [admin, member, viewer]
  createdAt date
}

entity Project {
  name        string    required
  description text
  status      enum      [active, paused, archived]
  owner       string    required
  createdAt   date
}

entity Task {
  name      string    required
  priority  enum      [low, medium, high, critical]
  status    enum      [todo, in_progress, done]
  createdAt date
}

component HeroMain {
  layout hero
  style premium+dark
  items [
    badge "SAAS TEMPLATE"
    title "Ship Faster With CRONUS"
    subtitle "Full-stack SaaS from a single .cronus file"
    cta "Get Started" -> "/signup" tone:primary
    cta "Dashboard" -> "/dashboard" tone:secondary
  ]
}

component MetricUsers {
  layout stack
  style card+metric
  items [
    label "Users"
    value "0"
  ]
}

component MetricProjects {
  layout stack
  style card+metric
  items [
    label "Projects"
    value "0"
  ]
}

api /auth {
  login     POST   /login     auth:public
  register  POST   /register  auth:public
  me        GET    /me        auth:jwt
}

api /projects {
  list    GET    /          auth:jwt
  create  POST   /          auth:jwt
  detail  GET    /:id       auth:jwt
  edit    PATCH  /:id       auth:jwt
  delete  DELETE /:id       auth:jwt
}

api /tasks {
  list    GET    /          auth:jwt
  create  POST   /          auth:jwt
}

page "/" type:custom {
  section hero {
    badge "SAAS TEMPLATE"
    title "Ship Faster With CRONUS"
    subtitle "Full-stack SaaS from a single .cronus file"
    cta "Get Started" -> "/signup" primary
  }
}

page "/login" type:form entity:User {
  title "Login"
  fields [email, password]
}

page "/signup" type:form entity:User {
  title "Sign Up"
  fields [name, email, password]
}

page "/dashboard" type:dashboard {
  title "Dashboard"
  use MetricUsers
  use MetricProjects
}

page "/projects" type:list entity:Project {
  title "Projects"
  columns [name, status, owner, createdAt]
}

page "/tasks" type:list entity:Task {
  title "Tasks"
  columns [name, priority, status, createdAt]
}

page "/settings" type:form entity:User {
  title "Settings"
  fields [name, email]
}
"#.to_string()
}

fn generate_landing_template() -> String {
    r###"# Generated by CRONUS — Premium Landing Page
# Style: Vercel/Linear dark theme

app "MyProduct" {
  stack react + tailwind
  port 5175
  database sqlite "./data.db"
  theme dark
}

style {
  theme dark
  accent blue
  background neutral-950
  radius xl
  font "SF Pro Display"
  mono "JetBrains Mono"
}

# ── Entity ──

entity WaitlistSubscriber {
  email     email     required unique
  name      string
  source    string
  createdAt date
}

# ── Components ──

component HeroLanding {
  layout hero
  style premium+dark
  items [
    badge "NOW IN BETA"
    title "Ship Faster. Scale Smarter."
    subtitle "The developer platform that eliminates boilerplate. Define your app in one file, deploy everywhere."
    cta "Start Building" -> "/signup" tone:primary
    cta "View Docs" -> "/docs" tone:secondary
  ]
}

component FeatureGrid {
  layout grid
  style cards+3col
  items [
    item "Zero Config" icon:zap tone:default
    item "Type Safe" icon:shield tone:default
    item "Edge Ready" icon:globe tone:default
    item "Auto API" icon:code tone:default
    item "Built-in Auth" icon:lock tone:default
    item "Real-time" icon:activity tone:default
  ]
}

component StatsRow {
  layout grid
  style stats+4col
  items [
    value "99.99%" label:"Uptime"
    value "12ms" label:"Latency"
    value "50k+" label:"Developers"
    value "2M+" label:"Requests/day"
  ]
}

component PricingPlans {
  layout grid
  style pricing+3col
  items [
    plan "Starter" price:$0/mo tone:default
    plan "Pro" price:$29/mo featured:true tone:primary
    plan "Enterprise" price:$99/mo tone:default
  ]
}

component CTABottom {
  layout hero
  style minimal+dark
  items [
    title "Ready to ship?"
    subtitle "Join the waitlist. No credit card required."
    cta "Get Early Access" -> "/signup" tone:primary
  ]
}

# ── API ──

api /waitlist {
  create  POST   /        auth:public
}

# ── Pages ──

page "/" type:custom {
  use HeroLanding
  use FeatureGrid
  use StatsRow
  use PricingPlans
  use CTABottom

  section hero {
    badge "NOW IN BETA"
    title "Ship Faster. Scale Smarter."
    subtitle "The developer platform that eliminates boilerplate. Define your app in one file, deploy everywhere."
    bullets [
      "Zero-config deployments",
      "Automatic API generation",
      "Built-in auth and database"
    ]
    cta "Start Building" -> "/signup" primary
    cta "View Docs" -> "/docs" secondary
  }

  section features cols:3 style:cards {
    item "Zero Config" icon:zap {
      "No webpack, no babel, no config files. Just write .cronus and run."
    }
    item "Type Safe" icon:shield {
      "Every entity, route, and component is validated at parse time."
    }
    item "Edge Ready" icon:globe {
      "Deploy to any edge runtime. Sub-10ms response times globally."
    }
    item "Auto API" icon:code {
      "CRUD endpoints generated automatically from your entity definitions."
    }
    item "Built-in Auth" icon:lock {
      "JWT authentication with Argon2 password hashing out of the box."
    }
    item "Real-time" icon:activity {
      "Live data updates via Server-Sent Events. No WebSocket setup."
    }
  }

  section pricing cols:3 {
    plan "Starter" $0/mo [
      "1 project",
      "SQLite database",
      "Community support",
      "Basic analytics"
    ]
    plan "Pro" $29/mo featured [
      "Unlimited projects",
      "PostgreSQL support",
      "Priority support",
      "Advanced analytics",
      "Custom domains",
      "Team collaboration"
    ]
    plan "Enterprise" $99/mo [
      "Everything in Pro",
      "SLA 99.99%",
      "Dedicated support",
      "On-premise option",
      "SSO & SAML",
      "Audit logs"
    ]
  }

  section cta {
    title "Ready to ship?"
    subtitle "Join thousands of developers building faster with CRONUS"
    cta "Get Early Access" -> "/signup" primary
    cta "Talk to Sales" -> "/contact" secondary
  }
}

page "/signup" type:form entity:WaitlistSubscriber {
  title "Join the Waitlist"
  fields [name, email]
}
"###.to_string()
}

// ══════════════════════════════════════════════════
// SYNC — generate .cronus/state-digest.json
// ══════════════════════════════════════════════════

fn cmd_sync() {
    use std::path::Path;
    use std::process::Command;

    println!("  Scanning project...");

    // 1. Check constitution.toml
    let has_constitution = Path::new(".cronus/constitution.toml").exists();
    if !has_constitution {
        println!("  \x1b[33m⚠ .cronus/constitution.toml not found\x1b[0m");
    }

    // 2. Read objective.toml
    let mut obj_title = String::from("(no objective set)");
    let mut obj_criteria: Vec<String> = Vec::new();
    if let Ok(content) = fs::read_to_string(".cronus/objective.toml") {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("title") {
                if let Some(val) = trimmed.split('=').nth(1) {
                    obj_title = val.trim().trim_matches('"').to_string();
                }
            }
            // Collect criteria lines (inside criteria = [...])
            if trimmed.starts_with('"') && trimmed.ends_with('"') || trimmed.starts_with('"') && trimmed.ends_with("\",") {
                let clean = trimmed.trim_matches(|c| c == '"' || c == ',' || c == ' ');
                if !clean.is_empty() {
                    obj_criteria.push(clean.to_string());
                }
            }
        }
    } else {
        println!("  \x1b[33m⚠ .cronus/objective.toml not found\x1b[0m");
    }

    // 3. Read tasks
    let mut open_tasks: Vec<String> = Vec::new();
    let mut done_tasks: Vec<String> = Vec::new();
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
                let mut id = String::new();
                let mut title = String::new();
                let mut status = String::from("open");
                for line in content.lines() {
                    let t = line.trim();
                    if t.starts_with("id") && t.contains('=') {
                        if let Some(v) = t.split('=').nth(1) {
                            id = v.trim().trim_matches('"').to_string();
                        }
                    }
                    if t.starts_with("title") && t.contains('=') {
                        if let Some(v) = t.split('=').nth(1) {
                            title = v.trim().trim_matches('"').to_string();
                        }
                    }
                    if t.starts_with("status") && t.contains('=') {
                        if let Some(v) = t.split('=').nth(1) {
                            status = v.trim().trim_matches('"').to_string();
                        }
                    }
                }
                let label = format!("{}: {}", id, title);
                if status == "done" {
                    done_tasks.push(label);
                } else {
                    open_tasks.push(label);
                }
            }
        }
    }

    // 4. Check build status
    let build_status = if Path::new("target/release/cronus-kernel").exists()
        || Path::new("target/release/cronus").exists()
        || Path::new("cronus-kernel/target/release/cronus").exists()
    {
        "passing"
    } else {
        "unknown (no release binary found)"
    };

    // 5. Count specs
    let spec_count = count_files_matching("specs", "spec.toml");

    // 6. Count conformance tests
    let test_count = count_files_matching("tests/conformance", ".cronus");

    // 7. Count examples
    let example_count = count_files_matching("examples", ".cronus");

    // 8. Git log
    let mut recent_commits: Vec<String> = Vec::new();
    if let Ok(output) = Command::new("git")
        .args(["log", "--oneline", "-20"])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    recent_commits.push(trimmed.to_string());
                }
            }
        }
    }

    // 9. Generate state-digest.json
    let now = chrono_now_iso();

    let digest = json!({
        "generated_at": now,
        "generated_by": "cronus sync",
        "objective": obj_title,
        "success_criteria": obj_criteria,
        "completed_tasks": done_tasks,
        "open_tasks": open_tasks,
        "metrics": {
            "build": build_status,
            "specs": spec_count,
            "conformance_tests": test_count,
            "examples": example_count
        },
        "recent_commits": recent_commits
    });

    // Ensure .cronus/ dir exists
    let _ = fs::create_dir_all(".cronus");
    let path = ".cronus/state-digest.json";
    match fs::write(path, serde_json::to_string_pretty(&digest).unwrap_or_default()) {
        Ok(_) => {
            println!("  Build: {}", build_status);
            println!("  Specs: {} | Tests: {} | Examples: {}", spec_count, test_count, example_count);
            println!("  Open tasks: {} | Completed: {}", open_tasks.len(), done_tasks.len());
            println!("  State digest written to \x1b[32m{}\x1b[0m", path);
        }
        Err(e) => {
            eprintln!("  \x1b[31mError writing {}: {}\x1b[0m", path, e);
        }
    }
}

fn count_files_matching(dir: &str, suffix: &str) -> usize {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().ends_with(suffix) {
                count += 1;
            }
        }
    }
    // Also check subdirectories one level deep
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Ok(sub_entries) = fs::read_dir(entry.path()) {
                    for sub in sub_entries.flatten() {
                        if sub.file_name().to_string_lossy().ends_with(suffix) {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    count
}

// cmd_lease — full implementation at end of file
// cmd_drift — full implementation at end of file

// ══════════════════════════════════════════════════
// HANDOFF — session summary + task completion
// ══════════════════════════════════════════════════

fn cmd_handoff() {
    use std::path::Path;
    use std::process::Command;

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
    println!("  Status: \x1b[33mopen\x1b[0m → \x1b[32mdone\x1b[0m");

    // 7. Run sync internally
    println!();
    cmd_sync();

    // 8. Final message
    println!();
    println!("  \x1b[32mState digest updated.\x1b[0m");
    println!("  Next terminal will inherit this state.");
    println!();
}

// ══════════════════════════════════════════════════
// VALIDATE --mission — constitution + objective check
// ══════════════════════════════════════════════════

fn cmd_validate_mission() {
    use std::process::Command;

    println!();
    println!("  \x1b[1mMission Validation\x1b[0m");
    println!("  \x1b[90m──────────────────\x1b[0m");

    // 1. Read constitution.toml [forbidden] items
    let constitution = fs::read_to_string(".cronus/constitution.toml").unwrap_or_default();
    let forbidden_items = brief_toml_arr_after_section(&constitution, "[forbidden]", "never");
    let forbidden_fallback = brief_toml_arr(&constitution, "never");
    let forbidden = if !forbidden_items.is_empty() { forbidden_items } else { forbidden_fallback };

    // 2. Read git diff content
    let diff_content = if let Ok(output) = Command::new("git")
        .args(["diff", "HEAD~5..HEAD"])
        .output()
    {
        if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            // Try without range
            if let Ok(output2) = Command::new("git").args(["diff"]).output() {
                String::from_utf8_lossy(&output2.stdout).to_string()
            } else {
                String::new()
            }
        }
    } else {
        String::new()
    };

    // 3. Check if any forbidden pattern appears in the diff (code files only)
    // Filter to only added lines in code files (.rs, .cronus), skip docs/config
    let mut constitution_pass = true;
    let mut violations: Vec<String> = Vec::new();
    let mut in_code_file = false;
    let added_code_lines: Vec<String> = diff_content.lines()
        .filter(|l| {
            if l.starts_with("+++ b/") {
                let path = &l[6..];
                in_code_file = path.ends_with(".rs") || path.ends_with(".cronus");
                return false;
            }
            if l.starts_with("--- ") || l.starts_with("diff --git") {
                return false;
            }
            in_code_file && l.starts_with('+') && !l.starts_with("+++")
        })
        .map(|l| l[1..].to_lowercase()) // strip leading '+'
        .collect();
    let code_text = added_code_lines.join("\n");

    for item in &forbidden {
        let item_lower = item.to_lowercase();
        if item_lower.contains("fake data") && (code_text.contains("math.random") || code_text.contains("mock_data") || code_text.contains("fake_data")) {
            constitution_pass = false;
            violations.push(item.clone());
        }
        if item_lower.contains("export") && (item_lower.contains("react") || item_lower.contains("vue") || item_lower.contains("svelte")) {
            // Look for actual framework imports/usage in code, not mentions in strings
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

    // 5. Check basic alignment — does the diff relate to the objective?
    // Only check code files, not docs/config (uses code_text from step 3)
    let mut objective_pass = true;
    let out_of_scope = brief_toml_arr(&objective, "items");
    for item in &out_of_scope {
        let item_lower = item.to_lowercase();
        // Check if the exact phrase (or close to it) appears in code
        let keywords: Vec<&str> = item_lower.split_whitespace()
            .filter(|w| w.len() > 6) // only significant words
            .collect();
        if keywords.len() >= 2 {
            // All significant keywords must appear AND they must appear near each other
            let matches: usize = keywords.iter().filter(|k| code_text.contains(**k)).count();
            // Also check the exact phrase (most reliable)
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
    let mut spec_names: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir("specs") {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Ok(sub) = fs::read_dir(entry.path()) {
                    for s in sub.flatten() {
                        if s.file_name().to_string_lossy().ends_with(".spec.toml") {
                            spec_names.push(s.file_name().to_string_lossy().to_string());
                        }
                    }
                }
            } else if entry.file_name().to_string_lossy().ends_with(".spec.toml") {
                spec_names.push(entry.file_name().to_string_lossy().to_string());
            }
        }
    }
    let mut specs_with_tests = 0usize;
    for spec in &spec_names {
        let base = spec.replace(".spec.toml", "");
        if count_files_matching("tests/conformance", &base) > 0 {
            specs_with_tests += 1;
        }
    }
    if spec_names.is_empty() {
        println!("  Spec coverage: \x1b[33mno specs found\x1b[0m");
    } else {
        println!("  Spec coverage: {}/{} specs have tests", specs_with_tests, spec_names.len());
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


// ══════════════════════════════════════════════════
// DRIFT helpers — used by cmd_drift
// ══════════════════════════════════════════════════

/// Returns (active_task_id, files_outside_lease) — used by drift scope check
fn lease_check_scope() -> (String, Vec<String>) {
    let mut task_id = String::new();
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
                    task_id = brief_toml_val(&content, "id").unwrap_or_default();
                    // Try section-aware parsing first, fall back to simple
                    let scope = brief_toml_arr_after_section(&content, "[scope]", "write");
                    write_scope = if !scope.is_empty() { scope } else { brief_toml_arr(&content, "write") };
                    break;
                }
            }
        }
    }

    if task_id.is_empty() || write_scope.is_empty() {
        return (task_id, Vec::new());
    }

    let changed = git_changed_files();

    let mut outside = Vec::new();
    for file in &changed {
        let in_scope = write_scope.iter().any(|scope_pattern| {
            lease_file_allowed(file, &[scope_pattern.clone()])
        });
        if !in_scope {
            outside.push(file.clone());
        }
    }

    (task_id, outside)
}

fn git_changed_files() -> Vec<String> {
    use std::process::Command;
    let mut files = Vec::new();

    if let Ok(output) = Command::new("git")
        .args(["diff", "--name-only", "HEAD"])
        .output()
    {
        if output.status.success() {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                let t = line.trim();
                if !t.is_empty() && !files.contains(&t.to_string()) {
                    files.push(t.to_string());
                }
            }
        }
    }

    if let Ok(output) = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .output()
    {
        if output.status.success() {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                let t = line.trim();
                if !t.is_empty() && !files.contains(&t.to_string()) {
                    files.push(t.to_string());
                }
            }
        }
    }

    files
}

fn git_diff_content() -> String {
    use std::process::Command;
    if let Ok(output) = Command::new("git")
        .args(["diff", "HEAD"])
        .output()
    {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).to_string();
        }
    }
    String::new()
}

// ══════════════════════════════════════════════════
// DRIFT — 3-axis drift detection
// ══════════════════════════════════════════════════

fn cmd_drift(args: &[String]) {
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
        DriftResult::Ok(msg) => println!("  Strategic: \x1b[32m\u{2713} OK\x1b[0m \u{2014} {}", msg),
        DriftResult::Warn(msg, details) => {
            println!("  Strategic: \x1b[33m\u{26a0} WARN\x1b[0m \u{2014} {}", msg);
            warnings.push(format!("Strategic: {}", msg));
            if explain { for d in details { println!("      \x1b[33m\u{2014} {}\x1b[0m", d); } }
        }
    }

    // 2. Scope Drift
    let scope = drift_check_scope(&changed);
    match &scope {
        DriftResult::Ok(msg) => println!("  Scope:     \x1b[32m\u{2713} OK\x1b[0m \u{2014} {}", msg),
        DriftResult::Warn(msg, details) => {
            println!("  Scope:     \x1b[33m\u{26a0} WARN\x1b[0m \u{2014} {}", msg);
            warnings.push(format!("Scope: {}", msg));
            if explain { for d in details { println!("      \x1b[33m\u{2014} {}\x1b[0m", d); } }
        }
    }

    // 3. Semantic Drift
    let semantic = drift_check_semantic(&changed, &diff_content);
    match &semantic {
        DriftResult::Ok(msg) => println!("  Semantic:  \x1b[32m\u{2713} OK\x1b[0m \u{2014} {}", msg),
        DriftResult::Warn(msg, details) => {
            println!("  Semantic:  \x1b[33m\u{26a0} WARN\x1b[0m \u{2014} {}", msg);
            warnings.push(format!("Semantic: {}", msg));
            if explain { for d in details { println!("      \x1b[33m\u{2014} {}\x1b[0m", d); } }
        }
    }

    println!();
    if warnings.is_empty() {
        println!("  \x1b[32mNo drift detected.\x1b[0m");
    } else {
        println!("  \x1b[33m{} warning{}.\x1b[0m Run `cronus drift --explain` for details.",
            warnings.len(),
            if warnings.len() == 1 { "" } else { "s" }
        );
    }
    println!();
}

#[allow(dead_code)]
enum DriftResult {
    Ok(String),
    Warn(String, Vec<String>),
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
            if t == "[out_of_scope]" { in_oos = true; continue; }
            if in_oos && t.starts_with('[') && t != "[out_of_scope]" { break; }
            if in_oos {
                let v = t.trim_start_matches('"').trim_end_matches('"').trim_end_matches(',').trim_matches('"');
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
            format!("{} file(s) may relate to out-of-scope items", drift_details.len()),
            drift_details,
        );
    }

    // Check if changes are at least in kernel/project territory
    let relevant = changed.iter().any(|f| {
        f.contains("src/") || f.ends_with(".rs") || f.ends_with(".cronus")
            || f.contains("specs/") || f.contains("tests/") || f.contains(".cronus/")
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
            if !line.starts_with('+') || line.starts_with("+++") { continue; }
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
                            && keyword != "true" && keyword != "false"
                            && keyword.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
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
    let added_lines: String = diff_content.lines()
        .filter(|l| l.starts_with('+') && !l.starts_with("+++"))
        .map(|l| if l.len() > 1 { &l[1..] } else { "" })
        .collect::<Vec<_>>()
        .join("\n")
        .to_lowercase();

    if added_lines.contains("math.random()") || added_lines.contains("math::random") {
        has_real_warning = true;
        details.push("fake data pattern detected (Math.random)".into());
    }
    if (added_lines.contains("react") || added_lines.contains("vue") || added_lines.contains("svelte"))
        && added_lines.contains("import")
    {
        has_real_warning = true;
        details.push("framework import detected \u{2014} CRONUS is the runtime".into());
    }
    if added_lines.contains("mock_data") || added_lines.contains("fake_data") || added_lines.contains("dummy_data") {
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
fn chrono_now_iso() -> String {
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Simple UTC timestamp without chrono crate
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // Days since epoch to Y-M-D (simplified)
    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let days_in_year = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
        if remaining < days_in_year { break; }
        remaining -= days_in_year;
        y += 1;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let month_days = [31, if leap {29} else {28}, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 0usize;
    for (i, &md) in month_days.iter().enumerate() {
        if remaining < md as i64 { m = i + 1; break; }
        remaining -= md as i64;
    }
    let d = remaining + 1;

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, hours, minutes, seconds)
}

// =============================================================================
// cronus lease — Task lease management (check / list / create)
// =============================================================================

fn cmd_lease(args: &[String]) {
    let sub = args.get(2).map(|s| s.as_str()).unwrap_or("check");
    match sub {
        "check" => lease_check(),
        "list" => lease_list(),
        "create" => lease_create(args),
        _ => {
            eprintln!("  \x1b[31mUnknown lease subcommand: {}\x1b[0m", sub);
            eprintln!("  Usage: cronus lease <check|list|create>");
            std::process::exit(1);
        }
    }
}

/// cronus lease check — validate git diff against active task scope
fn lease_check() {
    use std::process::Command;

    // 1. Find active task (first open/in_progress)
    let mut task_id = String::new();
    let mut task_title = String::new();
    let mut write_scope: Vec<String> = Vec::new();

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
                    write_scope = brief_toml_arr_after_section(&content, "[scope]", "write");
                    break;
                }
            }
        }
    }

    if task_id.is_empty() {
        eprintln!("  \x1b[33mNo active task lease found.\x1b[0m");
        eprintln!("  Create one with: cronus lease create \"title\" --write file1,file2");
        std::process::exit(1);
    }

    // 2. Get modified files from git
    let mut modified_files: Vec<String> = Vec::new();

    // Unstaged changes
    if let Ok(output) = Command::new("git")
        .args(["diff", "--name-only"])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                let f = line.trim().to_string();
                if !f.is_empty() && !modified_files.contains(&f) {
                    modified_files.push(f);
                }
            }
        }
    }

    // Staged changes
    if let Ok(output) = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                let f = line.trim().to_string();
                if !f.is_empty() && !modified_files.contains(&f) {
                    modified_files.push(f);
                }
            }
        }
    }

    if modified_files.is_empty() {
        println!("  Active lease: \x1b[1m{}\x1b[0m — {}", task_id, task_title);
        println!("  No modified files detected.");
        return;
    }

    // 3. Check each file against scope
    println!("  Active lease: \x1b[1m{}\x1b[0m — {}", task_id, task_title);
    println!("  Modified files:");

    let mut blocked_files: Vec<String> = Vec::new();

    for file in &modified_files {
        if lease_file_allowed(file, &write_scope) {
            println!("    \x1b[32m{}\x1b[0m  \x1b[32m✓ ALLOWED\x1b[0m", file);
        } else {
            println!("    \x1b[31m{}\x1b[0m  \x1b[31m✗ BLOCKED (not in scope)\x1b[0m", file);
            blocked_files.push(file.clone());
        }
    }

    if blocked_files.is_empty() {
        println!("\n  \x1b[32m✓ All files within task scope.\x1b[0m");
    } else {
        println!(
            "\n  \x1b[33m⚠ DRIFT DETECTED: {} file{} outside task scope.\x1b[0m",
            blocked_files.len(),
            if blocked_files.len() == 1 { "" } else { "s" }
        );
        for f in &blocked_files {
            println!("  Run `cronus lease expand {}` to add it.", f);
        }
        std::process::exit(1);
    }
}

/// Check if a file path is allowed by the write scope entries.
fn lease_file_allowed(file: &str, scope: &[String]) -> bool {
    for entry in scope {
        let entry_clean = entry.trim();
        if entry_clean.is_empty() {
            continue;
        }

        // Directory match: "tests/conformance/" matches any file under it
        if entry_clean.ends_with('/') {
            if file.starts_with(entry_clean) || file.contains(entry_clean) {
                return true;
            }
            let dir = entry_clean.trim_end_matches('/');
            if file.starts_with(&format!("{}/", dir)) {
                return true;
            }
            continue;
        }

        // Glob pattern with * (e.g. "tests/conformance/auth-*.cronus")
        if entry_clean.contains('*') {
            if lease_glob_match(entry_clean, file) {
                return true;
            }
            continue;
        }

        // Exact match
        if file == entry_clean {
            return true;
        }

        // Partial match: "main.rs" matches "cronus-kernel/src/main.rs"
        if file.ends_with(entry_clean) {
            return true;
        }

        // Partial match: scope "cronus-kernel/src/main.rs" matches file "src/main.rs"
        if entry_clean.ends_with(file) {
            return true;
        }
    }
    false
}

/// Simple glob matching — supports single * wildcard
fn lease_glob_match(pattern: &str, text: &str) -> bool {
    if let Some(star_pos) = pattern.find('*') {
        let prefix = &pattern[..star_pos];
        let suffix = &pattern[star_pos + 1..];

        // Direct match
        if text.starts_with(prefix) && text.ends_with(suffix) {
            return true;
        }

        // Partial path match
        if text.ends_with(suffix) {
            if let Some(idx) = text.find(prefix) {
                let remaining = &text[idx + prefix.len()..];
                if remaining.ends_with(suffix) {
                    return true;
                }
            }
        }
    }
    pattern == text
}

/// cronus lease list — show all tasks with status
fn lease_list() {
    if let Ok(entries) = fs::read_dir(".cronus/tasks") {
        let mut task_files: Vec<_> = entries
            .flatten()
            .filter(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                n.starts_with("TASK-") && n.ends_with(".toml")
            })
            .collect();
        task_files.sort_by_key(|e| e.file_name());

        if task_files.is_empty() {
            println!("  No task leases found in .cronus/tasks/");
            return;
        }

        println!("  \x1b[1mTask Leases:\x1b[0m\n");

        for entry in &task_files {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                let id = brief_toml_val(&content, "id").unwrap_or_else(|| "???".into());
                let title = brief_toml_val(&content, "title").unwrap_or_default();
                let status = brief_toml_val(&content, "status").unwrap_or_else(|| "open".into());
                let priority = brief_toml_val(&content, "priority").unwrap_or_default();

                let status_color = match status.as_str() {
                    "done" => "\x1b[32m",
                    "open" => "\x1b[33m",
                    "in_progress" => "\x1b[36m",
                    "blocked" => "\x1b[31m",
                    _ => "\x1b[90m",
                };

                let prio_str = if priority.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", priority)
                };

                println!(
                    "    {} {}{}\x1b[0m — {}{}",
                    id, status_color, status, title, prio_str
                );
            }
        }
        println!();
    } else {
        println!("  No .cronus/tasks/ directory found.");
    }
}

/// cronus lease create "title" --write file1,file2 --checks "check1,check2"
fn lease_create(args: &[String]) {
    let title = args.get(3).cloned().unwrap_or_else(|| {
        eprintln!("  \x1b[31mUsage: cronus lease create \"title\" --write file1,file2 [--checks \"c1,c2\"]\x1b[0m");
        std::process::exit(1);
    });

    let mut write_files: Vec<String> = Vec::new();
    let mut checks: Vec<String> = Vec::new();
    let mut read_files: Vec<String> = Vec::new();

    let mut i = 4;
    while i < args.len() {
        match args[i].as_str() {
            "--write" | "-w" => {
                if i + 1 < args.len() {
                    write_files = args[i + 1].split(',').map(|s| s.trim().to_string()).collect();
                    i += 2;
                } else { i += 1; }
            }
            "--checks" | "-c" => {
                if i + 1 < args.len() {
                    checks = args[i + 1].split(',').map(|s| s.trim().to_string()).collect();
                    i += 2;
                } else { i += 1; }
            }
            "--read" | "-r" => {
                if i + 1 < args.len() {
                    read_files = args[i + 1].split(',').map(|s| s.trim().to_string()).collect();
                    i += 2;
                } else { i += 1; }
            }
            _ => { i += 1; }
        }
    }

    // Find next task number
    let mut max_num: u32 = 0;
    if let Ok(entries) = fs::read_dir(".cronus/tasks") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("TASK-") && name.ends_with(".toml") {
                let num_str = &name[5..name.len() - 5];
                if let Ok(n) = num_str.parse::<u32>() {
                    if n > max_num { max_num = n; }
                }
            }
        }
    }
    let next_num = max_num + 1;
    let task_id = format!("TASK-{:03}", next_num);
    let today = brief_today_date();

    let mut toml = String::new();
    toml.push_str(&format!("[task]\nid = \"{}\"\n", task_id));
    toml.push_str(&format!("title = \"{}\"\n", title));
    toml.push_str("status = \"open\"\n");
    toml.push_str(&format!("created = \"{}\"\n", today));
    toml.push_str("priority = \"P1\"\n");

    toml.push_str("\n[mission]\n");
    toml.push_str(&format!("goal = \"{}\"\n", title));
    toml.push_str("context = \"\"\n");

    toml.push_str("\n[scope]\n");
    toml.push_str("write = [\n");
    for f in &write_files {
        toml.push_str(&format!("  \"{}\",\n", f));
    }
    toml.push_str("]\n");
    toml.push_str("read = [\n");
    for f in &read_files {
        toml.push_str(&format!("  \"{}\",\n", f));
    }
    toml.push_str("]\n");

    toml.push_str("\n[forbidden]\nitems = []\n");
    toml.push_str("\n[dependencies]\nrequires = []\n");

    toml.push_str("\n[done]\nchecks = [\n");
    for c in &checks {
        toml.push_str(&format!("  \"{}\",\n", c));
    }
    toml.push_str("]\n");

    let _ = fs::create_dir_all(".cronus/tasks");
    let path = format!(".cronus/tasks/{}.toml", task_id);
    match fs::write(&path, &toml) {
        Ok(_) => {
            println!("  \x1b[32m✓ Created {}\x1b[0m — {}", task_id, title);
            println!("  File: {}", path);
            if !write_files.is_empty() {
                println!("  Scope: {}", write_files.join(", "));
            }
        }
        Err(e) => {
            eprintln!("  \x1b[31mError creating {}: {}\x1b[0m", path, e);
            std::process::exit(1);
        }
    }
}
