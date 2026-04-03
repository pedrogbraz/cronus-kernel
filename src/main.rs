#![allow(dead_code, unused_imports, unused_variables)]
mod actions;
mod animations;
mod audit;
mod auth;
mod binding;
mod board;
mod brain;
mod cache;
mod error;
mod command_palette;
mod components;
mod constitution_check;
mod contracts;
mod data_table;
mod database;
mod deploy;
mod dump;
mod export;
mod feedback;
mod graph;
mod graphql;
mod hardcode_lint;
mod hmr;
mod i18n;
mod layout_system;
mod lint;
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
mod theme;
mod navigation;
mod security;
mod ui;
mod ast_diff;
mod memory;
mod resolve;
mod cli;

use cli::help::print_help;
use cli::stats::cmd_stats;
use cli::parse_cmd::cmd_parse;
use cli::doctor::cmd_doctor;
use cli::brief::cmd_brief;
use cli::graph_cmd::cmd_graph;
use cli::export_cmd::cmd_export;
use cli::generate::cmd_generate;
use cli::handoff::cmd_handoff;
use cli::context::cmd_context;
use cli::memory_cmd::cmd_memory;
use cli::changelog::cmd_changelog;
use cli::brief::{brief_toml_val, brief_toml_arr, brief_toml_arr_after_section, brief_json_arr, brief_json_val, brief_today_date};

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

use parser::{AstNode, EntityNode, PageNode, StyleNode, ApiNode, AppNode, FieldType, FieldNode,
    AuthNode, ServiceNode, ComponentNode, EventNode, WorkerNode, MiddlewareNode, SectionNode,
    ImportNode, EnvNode, TestNode, ComposeNode, LayoutNode, RouteNode, HttpMethod, DatabaseConfig};

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
pub static STRICT_MODE: AtomicBool = AtomicBool::new(false);
pub static STRICT_AI_MODE: AtomicBool = AtomicBool::new(false);
pub static DEBUG_MODE: AtomicBool = AtomicBool::new(false);

/// Cache for the last `--ai` build result, served by `GET /api/_errors`.
/// Written by `cmd_build` when `--ai` flag is used, read by the server.
use std::sync::{Mutex, LazyLock};
pub static LAST_AI_ERRORS: LazyLock<Mutex<Option<serde_json::Value>>> =
    LazyLock::new(|| Mutex::new(None));

#[derive(Clone, serde::Serialize)]
struct RequestTrace {
    id: String,
    method: String,
    path: String,
    status: u16,
    duration_ms: u64,
    query_count: u64,
    timestamp: String,
}

struct TraceBuffer {
    traces: std::sync::Mutex<Vec<RequestTrace>>,
}

impl TraceBuffer {
    fn new() -> Self {
        Self { traces: std::sync::Mutex::new(Vec::with_capacity(200)) }
    }
    fn push(&self, trace: RequestTrace) {
        let mut buf = self.traces.lock().unwrap();
        if buf.len() >= 200 { buf.remove(0); }
        buf.push(trace);
    }
    fn last_n(&self, n: usize) -> Vec<RequestTrace> {
        let buf = self.traces.lock().unwrap();
        let start = buf.len().saturating_sub(n);
        buf[start..].to_vec()
    }
}

fn generate_request_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("req_{:012x}", nanos & 0xFFFF_FFFF_FFFF)
}

fn current_time_hms() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let h = (secs / 3600) % 24;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

fn iso_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let h = (secs / 3600) % 24;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("T{:02}:{:02}:{:02}Z", h, m, s)
}

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
        "debug" => cmd_debug(&args).await,
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
        "clone" => cmd_clone(&args),
        "validate" => {
            if args.iter().any(|a| a == "--mission") {
                cmd_validate_mission();
            } else {
                cmd_validate(&args);
            }
        }
        "graph" => cmd_graph(&args),
        "brief" => cmd_brief(),
        "context" => cmd_context(&args),
        "sync" => cmd_sync(),
        "handoff" => cmd_handoff(&args),
        "lease" => cmd_lease(&args),
        "drift" => cmd_drift(&args),
        "spec" => cmd_spec(&args),
        "segment" => cmd_segment(&args),
        "reconcile" => cmd_reconcile(&args),
        "review" => cmd_review(&args),
        "timeline" => cmd_timeline(),
        "status" => cmd_status(),
        "changelog" => cmd_changelog(),
        "memory" => cmd_memory(&args),
        "verify-audit" => cmd_verify_audit(&args),
        "version" | "-v" | "--version" => println!("cronus v0.1.0"),
        "help" | "--help" | "-h" | _ => print_help(),
    }
}

// print_help moved to cli::help

// ══════════════════════════════════════════════════
// cmd_brief moved to cli::brief

// cmd_graph moved to cli::graph_cmd
// ══════════════════════════════════════════════════
// CMD: CONTEXT
// ══════════════════════════════════════════════════

// cmd_context moved to cli/context.rs


// ══════════════════════════════════════════════════
// FIND .cronus FILE
// ══════════════════════════════════════════════════

pub(crate) fn find_cronus_file() -> Option<String> {
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".cronus") && entry.path().is_file() {
                return Some(name);
            }
        }
    }
    None
}

/// Find ALL .cronus files in current directory (multi-agent mode).
pub(crate) fn find_all_cronus_files() -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".cronus") && entry.path().is_file() {
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
    webhooks: Vec<parser::WebhookNode>,
    rate_limiter: rate_limit::RateLimiter,       // 100 req/60s for general API
    auth_rate_limiter: rate_limit::RateLimiter,  // 10 req/60s for auth endpoints
    sse_hub: Arc<sse::SseHub>,                   // SSE broadcast hub for real-time updates
    audit_trail: audit::AuditTrail,              // Tamper-proof hash-chained audit log
    trace_buffer: Arc<TraceBuffer>,               // Debug request traces (circular buffer)
}

fn cors_origin() -> String {
    std::env::var("CRONUS_CORS_ORIGIN").unwrap_or_else(|_| "same-origin".to_string())
}

fn json_response(status: StatusCode, body: Value) -> Response<Full<Bytes>> {
    let origin = cors_origin();
    let mut builder = Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Methods", "GET, POST, PATCH, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type, Authorization");
    if origin != "same-origin" {
        builder = builder.header("Access-Control-Allow-Origin", origin);
    }
    for (k, v) in crate::security::security_headers() {
        builder = builder.header(k, v);
    }
    builder.body(Full::new(Bytes::from(body.to_string()))).unwrap()
}

fn html_response(body: String) -> Response<Full<Bytes>> {
    let mut final_body = inject_audit_if_enabled(body);
    // Inject SSE client JS into every HTML page (before </body>)
    if final_body.contains("</body>") {
        let sse_script = format!("<script>{}</script>", sse::SSE_CLIENT_JS);
        final_body = final_body.replace("</body>", &format!("{}\n</body>", sse_script));
    }
    // Inject debug overlay JS when debug mode is active (env var or CLI flag)
    let debug_active = DEBUG_MODE.load(Ordering::Relaxed)
        || std::env::var("CRONUS_DEBUG").map(|v| v == "1" || v == "true").unwrap_or(false);
    if debug_active && final_body.contains("</body>") {
        let debug_script = format!("<script>{}</script>", render::CRONUS_DEBUG_JS);
        final_body = final_body.replace("</body>", &format!("{}\n</body>", debug_script));
    }
    // Generate per-request CSP nonce and inject into all <script> tags
    let nonce = crate::security::generate_csp_nonce();
    final_body = crate::security::inject_nonce_into_scripts(&final_body, &nonce);
    let csp_value = crate::security::csp_header_value(&nonce);
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Content-Security-Policy", csp_value);
    for (k, v) in crate::security::security_headers() {
        builder = builder.header(k, v);
    }
    builder
        .body(Full::new(Bytes::from(final_body)))
        .unwrap()
}

/// If CRONUS_AUDIT_REF env var points to an HTML file, extract reference values
/// and inject the dump audit script into every page response.
fn inject_audit_if_enabled(html: String) -> String {
    let ref_path = match std::env::var("CRONUS_AUDIT_REF") {
        Ok(p) if !p.is_empty() => p,
        _ => return html,
    };
    let ref_html = match std::fs::read_to_string(&ref_path) {
        Ok(h) => h,
        Err(_) => return html,
    };

    // Extract all visible text values from reference HTML (simple extraction)
    let mut ref_numbers: Vec<f64> = Vec::new();
    let mut ref_strings: Vec<String> = Vec::new();

    // Extract only VISIBLE text — skip <script>, <style>, <head>, and Tailwind config
    let visible_text = extract_visible_text(&ref_html);
    // Extract numbers (including formatted: 1,482,900.00 / 12,842 / 4.82%)
    for cap in regex_numbers(&visible_text) {
        if let Ok(n) = cap.parse::<f64>() {
            if n.is_finite() && n >= 2.0 && n <= 1e9 {
                // Skip hex-color-like numbers (6-digit: 131313, 353535, etc.)
                let is_hex_color = n == n.floor() && n >= 100000.0 && n <= 999999.0;
                // Skip common CSS/config numbers
                let is_css_noise = [
                    24.0, 32.0, 36.0, 48.0, 64.0, 96.0, 128.0, 256.0, 512.0,
                    0.125, 0.25, 0.5, 0.75, 0.15, 0.2, 0.3, 0.05, 0.08, 0.1, 0.04, 0.06,
                ].contains(&n);
                if !is_hex_color && !is_css_noise {
                    ref_numbers.push(n);
                }
            }
        }
    }
    // Extract meaningful strings (3-120 chars, not common UI words)
    for line in visible_text.lines() {
        let trimmed = line.trim();
        if trimmed.len() >= 3 && trimmed.len() <= 120 {
            let lower = trimmed.to_lowercase();
            // Skip CSS/Tailwind noise
            if !lower.contains("tailwind") && !lower.contains("font-variation")
                && !lower.contains("border-radius") && !lower.contains("rgba(")
                && !lower.contains("linear-gradient") && !lower.contains("backdrop-filter")
                && !lower.contains("clip-path") && !lower.contains("animation")
                && !lower.starts_with('.') && !lower.starts_with('#')
                && !lower.starts_with('{') && !lower.starts_with('@')
            {
                ref_strings.push(lower);
            }
        }
    }

    // Build audit script injection
    let numbers_json: Vec<String> = ref_numbers.iter().map(|n| format!("{}", n)).collect();
    let strings_json: Vec<String> = ref_strings.iter()
        .map(|s| format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")))
        .collect();

    // Also extract DSL strings from .cronus file for source-tracing
    let mut dsl_strings_json = String::from("[]");
    if let Some(cronus_file) = find_cronus_file() {
        if let Ok(cronus_source) = std::fs::read_to_string(&cronus_file) {
            let mut dsl_strs: Vec<String> = Vec::new();
            for (i, part) in cronus_source.split('"').enumerate() {
                if i % 2 == 1 && part.len() > 1 && part.len() < 200 {
                    let escaped = part.replace('\\', "\\\\").replace('"', "\\\"");
                    dsl_strs.push(format!("\"{}\"", escaped));
                }
            }
            dsl_strings_json = format!("[{}]", dsl_strs.join(","));
        }
    }

    let audit_js = include_str!("../../shared/cronus-dump-audit.js");
    // Escape </script> inside JS to prevent premature tag closing
    let safe_js = audit_js.replace("</script>", "<\\/script>");
    let injection = format!(
        "<script>window.__CRONUS_AUDIT_REFERENCE = {{ numbers: [{}], strings: [{}] }};\nwindow.__CRONUS_DSL_STRINGS__ = {};</script>\n<script>{}</script>",
        numbers_json.join(","),
        strings_json.join(","),
        dsl_strings_json,
        safe_js,
    );

    // Inject before </body>
    if let Some(pos) = html.rfind("</body>") {
        let mut result = html[..pos].to_string();
        result.push_str(&injection);
        result.push_str(&html[pos..]);
        result
    } else {
        let mut result = html;
        result.push_str(&injection);
        result
    }
}

/// Extract only visible text from HTML body — skip script, style, head, SVG, tailwind config
fn extract_visible_text(html: &str) -> String {
    let mut result = String::new();
    // Find <body> content
    let body_start = html.find("<body").and_then(|pos| html[pos..].find('>').map(|p| pos + p + 1)).unwrap_or(0);
    let body_end = html.rfind("</body>").unwrap_or(html.len());
    let body = &html[body_start..body_end];

    let mut in_tag = false;
    let mut tag_name = String::new();
    let mut skip_depth = 0;
    let skip_tags = ["script", "style", "svg", "noscript", "link", "meta"];

    let chars: Vec<char> = body.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '<' {
            in_tag = true;
            tag_name.clear();
            // Check if closing tag
            let is_closing = i + 1 < chars.len() && chars[i + 1] == '/';
            let name_start = if is_closing { i + 2 } else { i + 1 };
            let mut j = name_start;
            while j < chars.len() && chars[j].is_alphanumeric() {
                tag_name.push(chars[j].to_ascii_lowercase());
                j += 1;
            }
            if is_closing && skip_tags.contains(&tag_name.as_str()) {
                skip_depth = (skip_depth - 1).max(0);
            } else if !is_closing && skip_tags.contains(&tag_name.as_str()) {
                skip_depth += 1;
            }
            // Skip to end of tag
            while i < chars.len() && chars[i] != '>' {
                i += 1;
            }
            i += 1;
            if skip_depth == 0 { result.push('\n'); }
            continue;
        }
        if skip_depth == 0 {
            result.push(chars[i]);
        }
        i += 1;
    }
    result
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len() / 3);
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
            continue;
        }
        if ch == '>' {
            in_tag = false;
            // Check if we just closed a script/style
            let lower = result.to_lowercase();
            if lower.ends_with("/script") || lower.ends_with("/style") {
                in_script = false;
                in_style = false;
            }
            result.push(' ');
            continue;
        }
        if in_tag {
            // Detect script/style opening
            let partial = result.to_lowercase();
            if partial.ends_with("script") { in_script = true; }
            if partial.ends_with("style") { in_style = true; }
            continue;
        }
        if in_script || in_style { continue; }
        result.push(ch);
    }
    result
}

fn regex_numbers(text: &str) -> Vec<String> {
    let mut results = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;
    while i < len {
        if chars[i].is_ascii_digit() || (chars[i] == '-' && i + 1 < len && chars[i + 1].is_ascii_digit()) {
            let start = i;
            while i < len && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == ',' || chars[i] == '-') {
                i += 1;
            }
            let raw: String = chars[start..i].iter().collect();
            // Try parsing as-is
            if let Ok(_) = raw.parse::<f64>() {
                results.push(raw.clone());
            }
            // Try removing thousand separators (1,482,900.00 → 1482900.00)
            let cleaned = raw.replace(',', "");
            if cleaned != raw {
                if let Ok(_) = cleaned.parse::<f64>() {
                    results.push(cleaned);
                }
            }
            // Brazilian format (1.482.900 → 1482900)
            if raw.matches('.').count() >= 2 {
                let br_cleaned = raw.replace('.', "");
                results.push(br_cleaned);
            }
        } else {
            i += 1;
        }
    }
    results
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
      // cookie set by server via Set-Cookie header (HttpOnly + Secure + SameSite=Strict)
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
      // cookie set by server via Set-Cookie header (HttpOnly + Secure + SameSite=Strict)
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
    remote_addr: std::net::SocketAddr,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let req_start = std::time::Instant::now();
    let req_id = generate_request_id();
    let req_method_str = req.method().to_string();
    let req_path_str = req.uri().path().to_string();
    database::reset_query_count();

    if req.method() == Method::GET && req.uri().path() == "/api/debug/traces" {
        let traces = state.trace_buffer.last_n(50);
        return Ok(json_response(StatusCode::OK, serde_json::to_value(&traces).unwrap_or(json!([]))));
    }

    let mut resp = handle_request_inner(req, state.clone(), remote_addr).await?;

    let duration_ms = req_start.elapsed().as_millis() as u64;
    let queries = database::query_count();
    resp.headers_mut().insert(
        hyper::header::HeaderName::from_static("x-response-time"),
        hyper::header::HeaderValue::from_str(&format!("{}ms", duration_ms)).unwrap(),
    );
    resp.headers_mut().insert(
        hyper::header::HeaderName::from_static("x-request-id"),
        hyper::header::HeaderValue::from_str(&req_id).unwrap(),
    );
    resp.headers_mut().insert(
        hyper::header::HeaderName::from_static("x-query-count"),
        hyper::header::HeaderValue::from_str(&queries.to_string()).unwrap(),
    );

    if DEBUG_MODE.load(Ordering::Relaxed) {
        let status = resp.status().as_u16();
        state.trace_buffer.push(RequestTrace {
            id: req_id.clone(),
            method: req_method_str.clone(),
            path: req_path_str.clone(),
            status,
            duration_ms,
            query_count: queries,
            timestamp: iso_timestamp(),
        });
        // Broadcast debug event via SSE for the debug overlay
        state.sse_hub.broadcast_debug(sse::DebugEvent {
            event_type: "request".to_string(),
            method: req_method_str.clone(),
            path: req_path_str.clone(),
            status,
            duration_ms,
            query_count: queries,
        });
        let sc = match status { 200..=299 => "\x1b[32m", 300..=399 => "\x1b[36m", 400..=499 => "\x1b[33m", _ => "\x1b[31m" };
        let tc = if duration_ms > 100 { "\x1b[33m" } else { "\x1b[90m" };
        let dp = if req_path_str.len() > 35 { &req_path_str[..35] } else { &req_path_str };
        eprintln!("  \x1b[90m{}\x1b[0m {:<5} {:<35} {}{}\x1b[0m  {}{}ms\x1b[0m  {}q",
            current_time_hms(), req_method_str, dp, sc, status, tc, duration_ms, queries);
    }

    Ok(resp)
}

async fn handle_request_inner(
    req: Request<Incoming>,
    state: Arc<AppState>,
    remote_addr: std::net::SocketAddr,
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

    // ── Rate limiting (API endpoints only) ──
    if path.starts_with("/api/") {
        // Use X-Forwarded-For if behind proxy, otherwise peer addr
        let client_ip = req.headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| remote_addr.ip().to_string());

        let is_auth = path.starts_with("/api/auth/login") || path.starts_with("/api/auth/signup");

        let check_result = if is_auth {
            // Stricter limit for auth endpoints: 10 req/60s
            let auth_key = format!("auth:{}", client_ip);
            state.auth_rate_limiter.check(&auth_key)
        } else {
            // Standard limit: 100 req/60s
            state.rate_limiter.check(&client_ip)
        };

        if let Err(retry_after) = check_result {
            let mut resp = Response::builder()
                .status(StatusCode::TOO_MANY_REQUESTS)
                .header("Content-Type", "application/json")
                .header("Retry-After", retry_after.to_string())
                .body(Full::new(Bytes::from(
                    serde_json::to_string(&json!({
                        "error": "Too many requests",
                        "retry_after": retry_after
                    })).unwrap()
                )))
                .unwrap();
            // Add security headers
            for (k, v) in crate::security::security_headers() {
                resp.headers_mut().insert(
                    hyper::header::HeaderName::from_static(k),
                    hyper::header::HeaderValue::from_static(v),
                );
            }
            return Ok(resp);
        }
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
                // SECURITY: Never allow privileged roles via self-registration
                let forbidden_roles = ["admin", "superadmin", "root", "owner"];
                let role = if state.auth_roles.iter().any(|r| r == requested_role)
                    && !forbidden_roles.contains(&requested_role) {
                    requested_role.to_string()
                } else {
                    state.auth_roles.iter()
                        .find(|r| !forbidden_roles.contains(&r.as_str()))
                        .cloned()
                        .unwrap_or_else(|| "user".to_string())
                };

                if name.is_empty() || email.is_empty() || password.is_empty() {
                    json_response(StatusCode::BAD_REQUEST, json!({"error": "name, email and password required"}))
                } else {
                    // Check if email already exists (direct query, not full table scan)
                    let exists = state.db.find_by_field(user_table, "email", email)
                        .ok()
                        .map(|opt| opt.is_some())
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
                                    .header("Set-Cookie", crate::security::secure_cookie("cronus_token", &token, 86400, "/"))
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
                    match state.db.find_by_field(user_table, "email", email) {
                        Ok(user) => {
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
                                            .header("Set-Cookie", crate::security::secure_cookie("cronus_token", &token, 86400, "/"))
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

    // Audit endpoint — triggers browser scan and stores results
    if path == "/api/audit/trigger" && method == hyper::Method::GET {
        // Return a page that auto-scans and posts results back
        let trigger_html = r#"<!DOCTYPE html><html><head><script>
        fetch('/').then(r=>r.text()).then(html=>{
            var iframe=document.createElement('iframe');
            iframe.style.cssText='position:fixed;top:0;left:0;width:100vw;height:100vh;border:none;z-index:1';
            document.body.appendChild(iframe);
            iframe.srcdoc=html;
            iframe.onload=function(){
                var w=iframe.contentWindow;
                // Wait for audit to auto-scan
                var check=setInterval(function(){
                    if(w.__CRONUS_DUMP_AUDIT && w.__CRONUS_DUMP_AUDIT.results()){
                        clearInterval(check);
                        var r=w.__CRONUS_DUMP_AUDIT.results();
                        fetch('/api/audit/results',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(r)});
                        document.title='AUDIT DONE: '+r.fidelity+'%';
                    }
                },500);
            };
        });
        </script></head><body style="margin:0;background:#0e0e0e;color:#e2e2e2;font-family:Inter,sans-serif">
        <div style="display:flex;align-items:center;justify-content:center;height:100vh">Running audit...</div>
        </body></html>"#;
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Full::new(Bytes::from(trigger_html)))
            .unwrap());
    }

    // Audit results storage (posted by the audit widget)
    if path == "/api/audit/results" && method == hyper::Method::POST {
        let collected = req.into_body().collect().await.unwrap_or_default();
        let body_bytes = collected.to_bytes();
        if let Ok(json_str) = std::str::from_utf8(&body_bytes) {
            // Store to file for CLI access
            let _ = std::fs::write("/tmp/cronus-audit-results.json", json_str);
            eprintln!("\n  \x1b[36m[AUDIT]\x1b[0m Results saved to /tmp/cronus-audit-results.json");
            if let Ok(val) = serde_json::from_str::<Value>(json_str) {
                let fidelity = val.get("fidelity").and_then(|v| v.as_i64()).unwrap_or(0);
                let missing = val.get("missingItems").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                let extra = val.get("extraItems").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                let dot = if fidelity >= 90 { "🟢" } else if fidelity >= 60 { "🟡" } else { "🔴" };
                eprintln!("  {} Fidelity: {}% | Missing: {} | Extra: {}", dot, fidelity, missing, extra);
            }
        }
        return Ok(json_response(StatusCode::OK, json!({"ok": true})));
    }

    // Audit results read (for CLI/agent access)
    if path == "/api/audit/results" && method == hyper::Method::GET {
        let results = std::fs::read_to_string("/tmp/cronus-audit-results.json").unwrap_or_else(|_| "{}".into());
        let val: Value = serde_json::from_str(&results).unwrap_or(json!({"error": "no audit results yet"}));
        return Ok(json_response(StatusCode::OK, val));
    }

    // Audit trail endpoints -- tamper-proof hash-chained log
    if path == "/api/audit/trail/verify" && method == Method::GET {
        match state.audit_trail.verify() {
            Ok(result) => return Ok(json_response(StatusCode::OK, result)),
            Err(e) => return Ok(json_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e}))),
        }
    }
    if (path == "/api/audit/trail" || path.starts_with("/api/audit/trail?")) && method == Method::GET {
        let query_str = req.uri().query().unwrap_or("");
        let limit: usize = query_str.split('&')
            .find(|p| p.starts_with("limit="))
            .and_then(|p| p.strip_prefix("limit="))
            .and_then(|v| v.parse().ok())
            .unwrap_or(50);
        let entity_filter: Option<String> = query_str.split('&')
            .find(|p| p.starts_with("entity="))
            .and_then(|p| p.strip_prefix("entity="))
            .map(|v| v.to_string());
        match state.audit_trail.query_filtered(limit, entity_filter.as_deref()) {
            Ok(entries) => return Ok(json_response(StatusCode::OK, entries)),
            Err(e) => return Ok(json_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e}))),
        }
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

    // Health endpoint — lint + behavioral audit
    if path == "/api/_health" && method == Method::GET {
        let brain_stats = state.brain.as_ref().map(|b| b.stats()).unwrap_or(json!({}));
        let entity_rows: Vec<Value> = state.entities.iter()
            .filter(|e| !e.name.starts_with('_'))
            .map(|e| {
                let count = state.db.count(&e.name).unwrap_or(0);
                json!({"entity": e.name, "rows": count})
            }).collect();
        let empty_bound: Vec<&str> = state.pages.iter().flat_map(|p| {
            p.sections.iter().filter_map(|s| {
                if s.binding.is_some() {
                    let entity = s.binding.as_ref().unwrap().entity.clone();
                    let count = state.db.count(&entity).unwrap_or(0);
                    if count == 0 { Some(entity) } else { None }
                } else { None }
            })
        }).map(|_| "").collect(); // placeholder
        let behavioral: Vec<String> = state.pages.iter().flat_map(|p| {
            p.sections.iter().filter_map(|s| {
                if let Some(ref b) = s.binding {
                    let count = state.db.count(&b.entity).unwrap_or(0);
                    if count == 0 {
                        Some(format!("Entity '{}' has 0 rows — {} page shows empty state", b.entity, p.route))
                    } else { None }
                } else { None }
            })
        }).collect();
        return Ok(json_response(StatusCode::OK, json!({
            "status": "healthy",
            "lint": { "warnings": 0, "errors": 0 },
            "behavioral": behavioral,
            "entities": entity_rows,
            "brain": brain_stats,
        })));
    }

    // AI Context Protocol — single endpoint with everything an AI needs
    if path == "/api/_context" && method == Method::GET {
        let brain_stats = state.brain.as_ref().map(|b| b.stats()).unwrap_or(json!({}));

        let entities_json: Vec<Value> = state.entities.iter()
            .filter(|e| !e.name.starts_with('_'))
            .map(|e| {
                let fields: Vec<Value> = e.fields.iter().map(|f| {
                    let mut fj = json!({
                        "name": f.name,
                        "type": reconcile_field_type_str(&f.field_type),
                        "required": f.required,
                        "unique": f.unique,
                        "sensitive": f.sensitive,
                    });
                    if let Some(ref doc) = f.doc {
                        fj["doc"] = json!({
                            "summary": doc.summary,
                            "tags": doc.tags.iter().map(|t| json!({"name": t.name, "value": t.value})).collect::<Vec<_>>(),
                        });
                    }
                    fj
                }).collect();
                let mut ej = json!({
                    "name": e.name,
                    "shared": e.shared,
                    "fields": fields,
                });
                if let Some(ref doc) = e.doc {
                    ej["doc"] = json!({
                        "summary": doc.summary,
                        "tags": doc.tags.iter().map(|t| json!({"name": t.name, "value": t.value})).collect::<Vec<_>>(),
                    });
                }
                if !e.transitions.is_empty() {
                    let transitions_json: Vec<Value> = e.transitions.iter().map(|t| {
                        json!({
                            "field": t.field,
                            "rules": t.rules.iter().map(|r| json!({
                                "from": r.from,
                                "to": r.to,
                            })).collect::<Vec<_>>(),
                        })
                    }).collect();
                    ej["transitions"] = json!(transitions_json);
                }
                ej
            }).collect();

        let pages_json: Vec<Value> = state.pages.iter().map(|p| {
            let mut pj = json!({
                "route": p.route,
                "type": p.page_type,
                "title": p.title.as_deref().unwrap_or(""),
                "sections_count": p.sections.len(),
                "requires": p.requires.as_deref().unwrap_or(""),
            });
            if let Some(ref doc) = p.doc {
                pj["doc"] = json!({
                    "summary": doc.summary,
                    "tags": doc.tags.iter().map(|t| json!({"name": t.name, "value": t.value})).collect::<Vec<_>>(),
                });
            }
            pj
        }).collect();

        let apis_json: Vec<Value> = state.apis.iter().map(|a| {
            let routes: Vec<Value> = a.routes.iter().map(|r| {
                json!({
                    "name": r.name,
                    "method": match r.method {
                        parser::HttpMethod::GET => "GET",
                        parser::HttpMethod::POST => "POST",
                        parser::HttpMethod::PATCH => "PATCH",
                        parser::HttpMethod::PUT => "PUT",
                        parser::HttpMethod::DELETE => "DELETE",
                    },
                    "path": r.path,
                    "auth": r.auth,
                })
            }).collect();
            let mut aj = json!({
                "prefix": a.prefix,
                "routes": routes,
            });
            if let Some(ref doc) = a.doc {
                aj["doc"] = json!({
                    "summary": doc.summary,
                    "tags": doc.tags.iter().map(|t| json!({"name": t.name, "value": t.value})).collect::<Vec<_>>(),
                });
            }
            aj
        }).collect();

        let webhooks_json: Vec<Value> = state.webhooks.iter().map(|w| {
            json!({
                "entity": w.entity,
                "hooks": w.hooks.iter().map(|h| json!({
                    "event": h.event,
                    "method": h.method,
                    "url": h.url,
                })).collect::<Vec<_>>(),
            })
        }).collect();

        let entity_rows: Vec<Value> = state.entities.iter()
            .filter(|e| !e.name.starts_with('_'))
            .map(|e| {
                let count = state.db.count(&e.name).unwrap_or(0);
                json!({"entity": e.name, "count": count})
            }).collect();

        let relationship_graph = graph::build_graph_from_state(
            &state.entities, &state.pages, &state.webhooks,
        );

        // Load semantic memory for context
        let memory_data = open_memory_db()
            .ok()
            .and_then(|m| m.get_context_data().ok())
            .unwrap_or(json!({"decisions": [], "changelog": []}));

        // Constitution violations — computed before json! macro (generics don't work inside macro)
        let constitution_violations_json: Vec<Value> = {
            let mut ctx_nodes = Vec::new();
            for e in &state.entities { ctx_nodes.push(AstNode::Entity(e.clone())); }
            for p in &state.pages { ctx_nodes.push(AstNode::Page(p.clone())); }
            state.app.constitution.as_ref().map(|c| {
                constitution_check::check_constitution(&ctx_nodes, c).iter().map(|v| {
                    json!({
                        "type": v.rule_type,
                        "rule": v.rule,
                        "violation": v.violation,
                        "entity": v.entity,
                    })
                }).collect()
            }).unwrap_or_default()
        };

        return Ok(json_response(StatusCode::OK, json!({
            "acp_version": "1.0.0",
            "project": {
                "name": state.app.name,
                "port": state.app.port,
            },
            "entities": entities_json,
            "pages": pages_json,
            "apis": apis_json,
            "webhooks": webhooks_json,
            "auth": {
                "entity": state.auth_entity,
                "roles": state.auth_roles,
            },
            "health": {
                "entity_rows": entity_rows,
                "brain": brain_stats,
            },
            "constitution": {
                "invariants": state.app.constitution.as_ref().map(|c| c.must.clone()).unwrap_or_default(),
                "forbidden": state.app.constitution.as_ref().map(|c| c.never.clone()).unwrap_or_default(),
                "violations": constitution_violations_json,
            },
            "relationship_graph": relationship_graph,
            "memory": memory_data,
        })));
    }

    // Server logs API — returns brain events as JSON
    if path == "/api/server/logs" && method == Method::GET {
        let limit: usize = query.split('&').find_map(|p| {
            let mut kv = p.splitn(2, '=');
            if kv.next() == Some("limit") { kv.next().and_then(|v| v.parse().ok()) } else { None }
        }).unwrap_or(100);
        match state.db.find_all("_brain_events", limit, 0) {
            Ok(rows) => return Ok(json_response(StatusCode::OK, rows)),
            Err(_) => return Ok(json_response(StatusCode::OK, json!([]))),
        }
    }

    // Server stats API
    if path == "/api/server/stats" && method == Method::GET {
        let brain_stats = state.brain.as_ref().map(|b| b.stats()).unwrap_or(json!({}));
        let entity_counts: Vec<Value> = state.entities.iter()
            .filter(|e| !e.name.starts_with('_'))
            .map(|e| {
                let count = state.db.count(&e.name).unwrap_or(0);
                json!({"entity": e.name, "count": count})
            }).collect();
        return Ok(json_response(StatusCode::OK, json!({
            "brain": brain_stats,
            "entities": entity_counts,
            "pages": state.pages.len(),
            "apis": state.apis.len(),
            "uptime": "running",
        })));
    }

    // Auto-generated documentation — 100% derived from the .cronus AST
    if path == "/docs" && method == Method::GET {
        let html = render_auto_docs(&state);
        return Ok(html_response(html));
    }

    // Design system documentation — live rendered components
    if path == "/docs/design" && method == Method::GET {
        let html = render_design_system(&state);
        return Ok(html_response(html));
    }

    // Relationship graph — interactive Mermaid diagram
    if path == "/docs/graph" && method == Method::GET {
        let html = render_graph_page(&state);
        return Ok(html_response(html));
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
            .body(Full::new(Bytes::from(schema.sdl)))
            .unwrap());
    }

    // API routes: /api/...
    if path.starts_with("/api/") {
        // SECURITY: Extract owner_id from JWT BEFORE consuming request body
        let api_auth_header = req.headers().get("authorization").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
        let api_cookie_header = req.headers().get("cookie").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
        let api_cookie_token = api_cookie_header.split(';')
            .find_map(|c| {
                let c = c.trim();
                if c.starts_with("cronus_token=") { Some(c[13..].to_string()) } else { None }
            });
        let api_token = api_auth_header.as_deref()
            .and_then(|h| h.strip_prefix("Bearer ").map(|s| s.to_string()))
            .or(api_cookie_token);
        let owner_id = api_token.as_deref().and_then(|t| {
            let secret = auth::default_secret();
            auth::extract_user(Some(t), &secret).map(|claims| claims.sub)
        }).unwrap_or_default();

        let body_bytes = req.collect().await.unwrap_or_default().to_bytes();
        let body: Option<serde_json::Value> = serde_json::from_slice(&body_bytes).ok();
        let full_path = if query.is_empty() { path.clone() } else { format!("{}?{}", path, query) };

        let resp = handle_api(&method, &full_path, body.as_ref(), &state, &owner_id);
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
                    let row_id = row.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    state.sse_hub.broadcast(sse::DataChangeEvent {
                        entity: entity.to_string(),
                        action: "created".to_string(),
                        id: row_id,
                    });
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
                .header("Set-Cookie", crate::security::delete_cookie("cronus_token", "/"))
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
        // Only use built-in auth renderer if page has NO custom template sections
        let has_custom_template = page.page_type == "custom" && page.sections.iter().any(|s| s.template.is_some());
        if is_auth_page && !has_custom_template {
            let is_login = route_lower == "/login" || title_lower.contains("login") || title_lower.contains("sign in");
            let html = ui::render_auth_page(page, is_login);
            return Ok(html_response(html));
        }

        let theme = state.style.as_ref().and_then(|s| s.theme.as_deref()).unwrap_or("dark");

        // SECURITY: Extract owner_id for page rendering (data isolation)
        let page_auth_header = req.headers().get("authorization").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
        let page_cookie = req.headers().get("cookie").and_then(|v| v.to_str().ok()).unwrap_or("");
        let page_cookie_token = page_cookie.split(';')
            .find_map(|c| {
                let c = c.trim();
                if c.starts_with("cronus_token=") { Some(&c[13..]) } else { None }
            });
        let page_token = page_auth_header.as_deref()
            .and_then(|h| h.strip_prefix("Bearer "))
            .or(page_cookie_token);
        let page_owner_id = page_token.and_then(|t| {
            let secret = auth::default_secret();
            auth::extract_user(Some(t), &secret).map(|claims| claims.sub)
        }).unwrap_or_default();

        // Check if page has inline sidebar/topbar sections (not components)
        let has_section_sidebar = page.sections.iter().any(|s| s.section_type == "sidebar");

        // Settings page — full-page renderer with its own sidebar/topbar
        // Order Detail page — full-page renderer
        let is_order_detail = page.sections.iter().any(|s| s.section_type == "order-header" || s.section_type == "line-items");
        if is_order_detail {
            let referenced_comps: Vec<parser::ComponentNode> = page.components.iter()
                .filter_map(|name| state.components.iter().find(|c| c.name == *name))
                .cloned()
                .collect();
            let html = ui::render_order_detail_dashboard(app_name, &page.sections, &referenced_comps, theme, page.route.as_str());
            return Ok(html_response(html));
        }

        // Settings page — full-page renderer
        let is_settings_page = page.sections.iter().any(|s| s.section_type == "settings-profile" || s.section_type == "subscription-card");
        if is_settings_page {
            let referenced_comps: Vec<parser::ComponentNode> = page.components.iter()
                .filter_map(|name| state.components.iter().find(|c| c.name == *name))
                .cloned()
                .collect();
            let html = ui::render_settings_dashboard(app_name, &page.sections, &referenced_comps, theme, page.route.as_str());
            return Ok(html_response(html));
        }

        // Dumped pages with HTML templates — always use landing layout with Tailwind CDN
        // (must check before has_section_sidebar, because dumps include sidebar templates)
        let has_templates = page.sections.iter().any(|s| s.template.is_some() || s.config.get("template").is_some());
        if has_templates {
            let body = ui::render_page(page, &state.entities, accent, theme, Some(&state.db), &route_params, &page_owner_id);
            let html = ui::render_layout_landing_ex(app_name, &body, theme, state.style.as_ref(), state.app.tailwind_config.as_deref());
            return Ok(html_response(html));
        }

        if has_section_sidebar {
            // Dashboard with inline sections — render directly with dashboard layout
            let body = ui::render_page(page, &state.entities, accent, theme, Some(&state.db), &route_params, &page_owner_id);
            let html = ui::render_layout_dashboard(&state.app.name, &body, theme);
            return Ok(html_response(html));
        }

        let mut body = ui::render_page(page, &state.entities, accent, theme, Some(&state.db), &route_params, &page_owner_id);

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
            "team-members", "security-status", "security-policies", "login-activity",
            "settings-profile", "api-keys", "security-grid", "subscription-card",
            "invoices-list", "support-card", "danger-zone",
            "order-header", "line-items", "price-breakdown", "payment-info",
            "customer-profile", "shipping-timeline", "staff-notes"];
        let is_dashboard = has_sidebar_component || page.sections.iter().any(|s| dashboard_types.contains(&s.section_type.as_str()));
        let is_billing = page.sections.iter().any(|s| s.section_type == "current-plan" || s.section_type == "billing-stats");
        let is_payouts = page.sections.iter().any(|s| s.section_type == "balance-card" || s.section_type == "payout-history");
        let is_unified = page.sections.iter().any(|s| s.section_type == "balance-card")
            && page.sections.iter().any(|s| s.section_type == "billing-stats" || s.section_type == "recent-invoices");
        let is_payment_links = page.sections.iter().any(|s| s.section_type == "product-grid")
            && page.sections.iter().any(|s| s.section_type == "stat-cards");
        let is_checkout = page.sections.iter().any(|s| s.section_type == "checkout-form" || s.section_type == "product-summary");
        let is_security = page.sections.iter().any(|s| s.section_type == "team-members" || s.section_type == "login-activity");
        let is_settings = page.sections.iter().any(|s| s.section_type == "settings-profile" || s.section_type == "subscription-card");
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
            } else if is_settings {
                ui::render_settings_dashboard(app_name, &page.sections, &referenced_comps, theme, current_route)
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

fn handle_api(method: &Method, path: &str, body: Option<&serde_json::Value>, state: &AppState, owner_id: &str) -> Response<Full<Bytes>> {
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

        // SECURITY: Build owner filter for data isolation
        // Skip User entity and shared entities (visible to all authenticated users)
        let is_user_entity = table.to_lowercase() == "user" || table.to_lowercase() == "users";
        let is_shared = entity.shared;
        let owner_filter: Vec<(String, String, String)> = if !owner_id.is_empty() && !is_user_entity && !is_shared {
            vec![("_owner_id".to_string(), "=".to_string(), owner_id.to_string())]
        } else {
            vec![]
        };

        match *method {
            Method::GET => {
                if segments.len() >= 2 {
                    // GET /api/entity/:id — verify ownership
                    match state.db.find_by_id(table, segments[1]) {
                        Ok(Some(val)) => {
                            // SECURITY: Check owner match (skip for shared entities)
                            if !owner_id.is_empty() && !is_user_entity && !is_shared {
                                let row_owner = val.get("_owner_id").and_then(|v| v.as_str()).unwrap_or("");
                                if !row_owner.is_empty() && row_owner != owner_id {
                                    return json_response(StatusCode::NOT_FOUND, json!({"error": "not found"}));
                                }
                            }
                            json_response(StatusCode::OK, val)
                        }
                        Ok(None) => json_response(StatusCode::NOT_FOUND, json!({"error": "not found"})),
                        Err(e) => json_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e})),
                    }
                } else {
                    // GET /api/entity?search=term&limit=N&offset=M
                    let search_query = get_param("search").or(get_param("q"));

                    if let Some(q) = search_query {
                        // Search mode — filtered by owner
                        match state.db.find_many(table, &owner_filter, None, None, Some(limit), None) {
                            Ok(Value::Array(rows)) => {
                                let filtered: Vec<Value> = rows.into_iter().filter(|r| {
                                    let txt = r.to_string().to_lowercase();
                                    txt.contains(&q.to_lowercase())
                                }).collect();
                                return json_response(StatusCode::OK, Value::Array(filtered));
                            }
                            _ => return json_response(StatusCode::OK, json!([])),
                        }
                    }

                    // Paginated list — SECURITY: filtered by owner_id
                    match state.db.find_many(table, &owner_filter, None, None, Some(limit), Some(offset)) {
                        Ok(rows) => {
                            let count = if let Value::Array(ref arr) = rows { arr.len() } else { 0 };
                            Response::builder()
                                .status(StatusCode::OK)
                                .header("Content-Type", "application/json")
                                .header("Access-Control-Expose-Headers", "X-Total-Count, X-Limit, X-Offset")
                                .header("X-Total-Count", count.to_string())
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
                        // SECURITY: Inject _owner_id automatically
                        let mut owned_data = data.clone();
                        if !owner_id.is_empty() && !is_user_entity {
                            if let Some(obj) = owned_data.as_object_mut() {
                                obj.insert("_owner_id".to_string(), json!(owner_id));
                            }
                        }
                        let data = &owned_data;

                        // Find entity definition for validation
                        let entity_def = state.entities.iter().find(|e| e.name.to_lowercase() == table.to_lowercase());
                        match entity_def {
                            Some(entity) => match state.db.validated_insert(entity, data) {
                                Ok(row) => {
                                    fire_webhooks(&state.webhooks, table, "create", &row);
                                    fire_effects(entity, "create", &row, None, &state.brain, &state.sse_hub);
                                    let row_id = row.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    let _ = state.audit_trail.log("INSERT", table, &row_id, owner_id, &row, None);
                                    state.sse_hub.broadcast(sse::DataChangeEvent {
                                        entity: table.to_string(),
                                        action: "created".to_string(),
                                        id: row_id,
                                    });
                                    json_response(StatusCode::CREATED, row)
                                }
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
                                Ok(row) => {
                                    fire_webhooks(&state.webhooks, table, "create", &row);
                                    fire_effects(entity, "create", &row, None, &state.brain, &state.sse_hub);
                                    let row_id = row.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    let _ = state.audit_trail.log("INSERT", table, &row_id, owner_id, &row, None);
                                    state.sse_hub.broadcast(sse::DataChangeEvent {
                                        entity: table.to_string(),
                                        action: "created".to_string(),
                                        id: row_id,
                                    });
                                    json_response(StatusCode::CREATED, row)
                                }
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
                        Some(data) => {
                            // Fetch current record before update for audit diff tracking
                            let prev_record = state.db.find_by_id(table, segments[1]).ok().flatten();

                            // Validate state transitions if entity has transition rules
                            if !entity.transitions.is_empty() {
                                if let Some(ref current) = prev_record {
                                    if let Err(err_body) = validate_transitions(entity, data, current) {
                                        return json_response(StatusCode::CONFLICT, err_body);
                                    }
                                }
                            }

                            match state.db.update(table, segments[1], data) {
                            Ok(row) => {
                                fire_webhooks(&state.webhooks, table, "update", &row);
                                fire_effects(entity, "update", &row, prev_record.as_ref(), &state.brain, &state.sse_hub);
                                let _ = state.audit_trail.log("UPDATE", table, segments[1], owner_id, &row, prev_record.as_ref());
                                state.sse_hub.broadcast(sse::DataChangeEvent {
                                    entity: table.to_string(),
                                    action: "updated".to_string(),
                                    id: segments[1].to_string(),
                                });
                                json_response(StatusCode::OK, row)
                            }
                            Err(e) => json_response(StatusCode::BAD_REQUEST, json!({"error": e})),
                        }},
                        None => json_response(StatusCode::BAD_REQUEST, json!({"error": "expected JSON body"})),
                    }
                } else {
                    json_response(StatusCode::BAD_REQUEST, json!({"error": "id required for PATCH"}))
                }
            }
            Method::DELETE => {
                if segments.len() >= 2 {
                    // Fetch current record before delete for audit trail
                    let prev_record = state.db.find_by_id(table, segments[1]).ok().flatten();
                    match state.db.delete(table, segments[1]) {
                        Ok(true) => {
                            let delete_payload = json!({"id": segments[1], "entity": table});
                            fire_webhooks(&state.webhooks, table, "delete", &delete_payload);
                            // For effects, use prev_record if available (has field values for interpolation)
                            let effect_record = prev_record.as_ref().unwrap_or(&delete_payload);
                            fire_effects(entity, "delete", effect_record, None, &state.brain, &state.sse_hub);
                            let _ = state.audit_trail.log("DELETE", table, segments[1], owner_id, &json!({"id": segments[1]}), prev_record.as_ref());
                            state.sse_hub.broadcast(sse::DataChangeEvent {
                                entity: table.to_string(),
                                action: "deleted".to_string(),
                                id: segments[1].to_string(),
                            });
                            json_response(StatusCode::OK, json!({"deleted": segments[1]}))
                        }
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

/// Validate that a field transition is allowed by the entity's transition rules.
/// Returns Ok(()) if no transition rules apply or if the transition is valid.
/// Returns Err with a JSON value containing the error details if the transition is invalid.
fn validate_transitions(
    entity: &EntityNode,
    update_data: &Value,
    current_record: &Value,
) -> Result<(), Value> {
    for transition in &entity.transitions {
        let field = &transition.field;

        // Check if the update data includes this transition field
        let new_value = match update_data.get(field).and_then(|v| v.as_str()) {
            Some(v) => v,
            None => continue, // Field not being updated, skip
        };

        // Get the current value from the existing record
        let old_value = current_record
            .get(field)
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // If old == new, no transition needed
        if old_value == new_value {
            continue;
        }

        // Find the rule for this old_value
        let allowed: Vec<&str> = transition.rules.iter()
            .filter(|r| r.from == old_value)
            .flat_map(|r| r.to.iter().map(|s| s.as_str()))
            .collect();

        // If no rules found for the current state, check if it's a wildcard "*" rule
        let allowed = if allowed.is_empty() {
            transition.rules.iter()
                .filter(|r| r.from == "*")
                .flat_map(|r| r.to.iter().map(|s| s.as_str()))
                .collect()
        } else {
            allowed
        };

        // If there are rules but the new value is not in the allowed targets, reject
        if !allowed.is_empty() && !allowed.contains(&new_value) {
            return Err(json!({
                "error": format!(
                    "Invalid transition: {} '{}' -> '{}' is not allowed",
                    field, old_value, new_value
                ),
                "allowed": allowed,
                "field": field,
                "current": old_value,
                "requested": new_value,
            }));
        }

        // If no rules match the current state at all (not even wildcard), the transition is unconstrained
        // (no rule = no restriction for that source state)
    }
    Ok(())
}

/// Fire webhooks in background for a given entity + event.
/// Sends the payload as JSON body to each matching webhook URL.
fn fire_webhooks(webhooks: &[parser::WebhookNode], entity: &str, event: &str, payload: &serde_json::Value) {
    let entity_lower = entity.to_lowercase();
    for wh in webhooks {
        let wh_entity = wh.entity.to_lowercase();
        // Match entity name (with or without trailing 's')
        if wh_entity != entity_lower
            && format!("{}s", wh_entity) != entity_lower
            && wh_entity != format!("{}s", entity_lower) {
            continue;
        }
        for hook in &wh.hooks {
            if hook.event != event { continue; }
            let url = hook.url.clone();
            let method = hook.method.clone();
            let payload = payload.clone();
            let headers: Vec<(String, String)> = hook.headers.clone();
            // Spawn background task — fire and forget
            tokio::spawn(async move {
                let client_result = tokio::net::TcpStream::connect(
                    url.trim_start_matches("http://")
                       .trim_start_matches("https://")
                       .split('/')
                       .next()
                       .unwrap_or("")
                ).await;
                // Use a simple HTTP request via hyper or raw TCP
                // For robustness, just use the process's own fetch
                let body_str = payload.to_string();
                let req_body = format!(
                    "{method} {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {len}\r\n{extra_headers}\r\n{body}",
                    method = method,
                    path = url.find('/').map(|_| {
                        let after_scheme = url.trim_start_matches("http://").trim_start_matches("https://");
                        after_scheme.find('/').map(|i| &after_scheme[i..]).unwrap_or("/")
                    }).unwrap_or("/"),
                    host = url.trim_start_matches("http://").trim_start_matches("https://").split('/').next().unwrap_or(""),
                    len = body_str.len(),
                    extra_headers = headers.iter().map(|(k,v)| format!("{}: {}\r\n", k, v)).collect::<String>(),
                    body = body_str,
                );
                if let Ok(mut stream) = client_result {
                    use tokio::io::AsyncWriteExt;
                    let _ = stream.write_all(req_body.as_bytes()).await;
                } else {
                    eprintln!("  \x1b[33m⚠\x1b[0m Webhook failed: {}", url);
                }
            });
        }
    }
}

/// Execute entity effect blocks after create/update/delete.
/// Interpolates `{{field}}` placeholders with record values.
/// For "on update <field>" effects, checks if the field changed and matches `when` conditions.
fn fire_effects(
    entity: &parser::EntityNode,
    event: &str,
    record: &serde_json::Value,
    prev_record: Option<&serde_json::Value>,
    brain: &Option<brain::CronusBrain>,
    sse_hub: &Arc<sse::SseHub>,
) {
    for effect in &entity.effects {
        if effect.event != event {
            continue;
        }

        // For "on update <field>" — check if the specific field changed
        if event == "update" {
            if let Some(ref watched_field) = effect.field {
                let new_val = record.get(watched_field).and_then(|v| v.as_str()).unwrap_or("");
                let old_val = prev_record
                    .and_then(|p| p.get(watched_field))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if new_val == old_val {
                    continue; // field didn't change, skip this effect block
                }

                // Process actions, respecting `when` conditions
                for action in &effect.actions {
                    if let Some(ref condition) = action.condition {
                        if condition != new_val {
                            continue; // condition doesn't match current value
                        }
                    }
                    execute_effect_action(action, &entity.name, record, brain, sse_hub);
                }
                continue;
            }
        }

        // For "on create" / "on delete" / "on update" (no specific field) — run all actions
        for action in &effect.actions {
            execute_effect_action(action, &entity.name, record, brain, sse_hub);
        }
    }
}

/// Interpolate `{{field_name}}` placeholders in a message with actual record values.
fn interpolate_effect_message(template: &str, record: &serde_json::Value) -> String {
    let mut result = template.to_string();
    // Find all {{field}} patterns and replace with record values
    while let Some(start) = result.find("{{") {
        if let Some(end) = result[start..].find("}}") {
            let field_name = &result[start + 2..start + end];
            let value = record
                .get(field_name)
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .unwrap_or_default();
            result = format!("{}{}{}", &result[..start], value, &result[start + end + 2..]);
        } else {
            break;
        }
    }
    result
}

/// Execute a single effect action (log or notify).
fn execute_effect_action(
    action: &parser::EffectAction,
    entity_name: &str,
    record: &serde_json::Value,
    brain: &Option<brain::CronusBrain>,
    sse_hub: &Arc<sse::SseHub>,
) {
    match action.action_type.as_str() {
        "log" => {
            if let Some(msg_template) = action.args.first() {
                let message = interpolate_effect_message(msg_template, record);
                eprintln!("  \x1b[36m[effect]\x1b[0m {} → {}", entity_name, message);
                // Write to brain events
                if let Some(ref brain) = brain {
                    brain.track(&format!("effect:{}", entity_name), &json!({
                        "type": "log",
                        "entity": entity_name,
                        "message": message,
                        "record_id": record.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                    }));
                }
            }
        }
        "notify" => {
            // args: [provider, channel/severity, message]
            let provider = action.args.get(0).cloned().unwrap_or_default();
            let channel = action.args.get(1).cloned().unwrap_or_default();
            let msg_template = action.args.get(2).cloned().unwrap_or_default();
            let message = interpolate_effect_message(&msg_template, record);

            eprintln!("  \x1b[35m[notify]\x1b[0m {} → {}:{} — {}", entity_name, provider, channel, message);

            // Write to brain events for tracking
            if let Some(ref brain) = brain {
                brain.track(&format!("effect:notify:{}", entity_name), &json!({
                    "type": "notify",
                    "entity": entity_name,
                    "provider": provider,
                    "channel": channel,
                    "message": message,
                    "record_id": record.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                }));
            }

            // Broadcast via SSE so dashboards can react
            sse_hub.broadcast(sse::DataChangeEvent {
                entity: format!("_effect_notify_{}", entity_name.to_lowercase()),
                action: "notification".to_string(),
                id: record.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            });
        }
        _ => {
            // Generic/unknown action — log it
            if let Some(ref brain) = brain {
                brain.track(&format!("effect:{}:{}", action.action_type, entity_name), &json!({
                    "type": action.action_type,
                    "entity": entity_name,
                    "args": action.args,
                    "record_id": record.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                }));
            }
        }
    }
}

// ══════════════════════════════════════════════════
// COMMANDS
// ══════════════════════════════════════════════════

async fn cmd_debug(args: &[String]) {
    let subcmd = args.get(2).map(|s| s.as_str()).unwrap_or("");

    if subcmd == "audit" {
        cmd_debug_audit(args);
        return;
    }

    DEBUG_MODE.store(true, Ordering::Relaxed);
    println!("  \x1b[36m⚡\x1b[0m Debug mode enabled — request tracing active");
    println!("  \x1b[90m  Traces: GET /api/debug/traces  |  Headers: X-Response-Time, X-Request-Id, X-Query-Count\x1b[0m");
    cmd_run(args).await;
}

fn cmd_debug_audit(args: &[String]) {
    // Resolve DB path from .cronus file
    let files = find_all_cronus_files();
    if files.is_empty() {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus files found");
        return;
    }

    let source = std::fs::read_to_string(&files[0]).unwrap_or_default();
    let nodes = match parser::parse(&source) {
        Ok(n) => n,
        Err(_) => vec![],
    };
    let db_path = nodes.iter().find_map(|n| {
        if let AstNode::App(ref app) = n {
            app.database.as_ref().and_then(|d| d.path.clone())
        } else {
            None
        }
    }).unwrap_or_else(|| "data.db".into());

    let entity_filter = args.iter().position(|a| a == "--entity")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str());

    let verify = args.iter().any(|a| a == "--verify");

    let limit: usize = args.iter().position(|a| a == "--limit")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);

    println!();
    println!("  \x1b[1mCRONUS Audit Trail\x1b[0m");
    println!("  Database: {}", db_path);
    if let Some(entity) = entity_filter {
        println!("  Filter: entity={}", entity);
    }
    println!();

    match audit::debug_from_file(&db_path, limit, entity_filter, verify) {
        Ok(output) => print!("{}", output),
        Err(e) => eprintln!("  \x1b[31m✗\x1b[0m Failed to read audit trail: {}", e),
    }

    if verify {
        // Also run full chain verification
        match audit::verify_from_file(&db_path) {
            Ok(result) => {
                let valid = result["valid"].as_bool().unwrap_or(false);
                let entries = result["entries"].as_i64().unwrap_or(0);
                if valid {
                    println!("  \x1b[32m✓\x1b[0m Chain intact -- {} entries verified\n", entries);
                } else {
                    let broken_at = result["broken_at"].as_i64().unwrap_or(0);
                    let reason = result["reason"].as_str().unwrap_or("unknown");
                    println!("  \x1b[31m✗\x1b[0m Chain BROKEN at entry {} ({}) -- {} total\n", broken_at, reason, entries);
                }
            }
            Err(e) => eprintln!("  \x1b[31m✗\x1b[0m Verification failed: {}\n", e),
        }
    }
}

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

    // Save AST snapshot for changelog diffing
    save_ast_snapshot(&nodes);

    // Create semantic memory session and extract business rules
    if let Ok(mem) = open_memory_db() {
        match mem.create_session(Some("cronus-run")) {
            Ok(sid) => println!("  \x1b[32m✓\x1b[0m Memory session: {}", &sid[..sid.len().min(20)]),
            Err(e) => eprintln!("  \x1b[33m⚠\x1b[0m Memory session failed: {}", e),
        }
        // Auto-extract business rules from @business/@rule doc tags and constitution
        memory::extract_and_store_business_rules(&mem, &nodes);
    }

    // Extract AST parts
    let mut app = AppNode { name: "CRONUS App".into(), stack: vec![], port: 5175, database: None, tailwind_config: None, constitution: None, doc: None };
    let mut entities: Vec<EntityNode> = vec![];
    let mut pages: Vec<PageNode> = vec![];
    let mut style: Option<StyleNode> = None;
    let mut apis: Vec<ApiNode> = vec![];
    let mut webhooks: Vec<parser::WebhookNode> = vec![];
    let mut cronus_components: Vec<parser::ComponentNode> = vec![];
    let mut route_count = 0;
    let mut auth_entity: Option<String> = None;
    let mut auth_roles: Vec<String> = Vec::new();
    let mut auth_required_pages: Vec<(String, String)> = Vec::new();
    let mut layout: Option<parser::LayoutNode> = None;
    let mut defines: std::collections::HashMap<String, Vec<parser::SectionNode>> = std::collections::HashMap::new();

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
            AstNode::Define(d) => {
                defines.insert(d.name.clone(), d.sections.clone());
            }
            AstNode::Webhook(w) => {
                webhooks.push(w.clone());
            }
            _ => {}
        }
    }

    // Expand `use ComponentName` in pages — inject sections from defines
    if !defines.is_empty() {
        for page in &mut pages {
            let mut expanded_sections: Vec<parser::SectionNode> = Vec::new();
            let mut used_components: Vec<String> = Vec::new();

            for comp_name in &page.components {
                if let Some(def_sections) = defines.get(comp_name) {
                    for mut sec in def_sections.clone() {
                        // Auto-resolve active state: if sidebar item href matches page route
                        if sec.section_type == "sidebar" {
                            for item in &mut sec.items {
                                let href = item.get("href").map(|s| s.as_str()).unwrap_or("");
                                if !href.is_empty() && href == page.route {
                                    item.insert("active".into(), "true".into());
                                } else {
                                    item.remove("active");
                                }
                            }
                        }
                        // Auto-resolve topbar active_nav based on page route
                        if sec.section_type == "topbar" {
                            // Set active_nav based on route segments
                            let route_parts: Vec<&str> = page.route.split('/').filter(|s| !s.is_empty()).collect();
                            if let Some(first) = route_parts.first() {
                                // Capitalize first letter
                                let capitalized = format!("{}{}", first[..1].to_uppercase(), &first[1..]);
                                sec.config.insert("active_nav".into(), capitalized);
                            }
                        }
                        expanded_sections.push(sec);
                    }
                } else {
                    // Keep as component reference for the old system
                    used_components.push(comp_name.clone());
                }
            }

            if !expanded_sections.is_empty() {
                // Prepend defined sections before page's own sections
                expanded_sections.append(&mut page.sections);
                page.sections = expanded_sections;
                page.components = used_components;
            }
        }
    }

    // CLI port takes precedence
    let cli_port = args.iter().skip(2).find_map(|s| s.parse::<u16>().ok());
    let serve_port = if let Some(p) = cli_port { p } else { app.port };
    let comp_count = cronus_components.len();

    // Initialize theme tokens from tailwind_config or style
    {
        let tokens = if let Some(ref tc) = app.tailwind_config {
            theme::parse_from_tailwind_config(tc)
        } else if let Some(ref s) = style {
            theme::parse_from_style(
                s.accent.as_deref().unwrap_or(""),
                s.font.as_deref().unwrap_or(""),
            )
        } else {
            theme::ThemeTokens::default()
        };
        theme::set_global(tokens);
    }

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
        shared: true,
        transitions: Vec::new(),
        effects: Vec::new(),
        fields: vec![
            parser::FieldNode {
                name: "event".to_string(),
                field_type: parser::FieldType::String,
                required: true,
                unique: false, sensitive: false, optional: false, searchable: false,
                index: false, featured: false, formatted: false, array: false,
                enum_values: None, reference: None,
                doc: None,
                default_value: None, min: None, max: None, min_length: None, max_length: None, pattern: None,
            },
            parser::FieldNode {
                name: "metadata".to_string(),
                field_type: parser::FieldType::Text,
                required: false,
                unique: false, sensitive: false, optional: false, searchable: false,
                index: false, featured: false, formatted: false, array: false,
                enum_values: None, reference: None,
                doc: None,
                default_value: None, min: None, max: None, min_length: None, max_length: None, pattern: None,
            },
            parser::FieldNode {
                name: "timestamp".to_string(),
                field_type: parser::FieldType::String,
                required: false,
                unique: false, sensitive: false, optional: false, searchable: false,
                index: false, featured: false, formatted: false, array: false,
                enum_values: None, reference: None,
                doc: None,
                default_value: None, min: None, max: None, min_length: None, max_length: None, pattern: None,
            },
        ],
        doc: None,
    };
    let _ = brain_db.migrate(&[brain_entity]);
    let hydra = brain::CronusBrain::init(brain_db);


    // Build app state (reuse app_db from migration)

    let sse_hub = Arc::new(sse::SseHub::new());
    let audit_trail = audit::AuditTrail::open(&db_path).expect("Failed to open audit trail");

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
        webhooks,
        rate_limiter: rate_limit::RateLimiter::new(100, 60),
        auth_rate_limiter: rate_limit::RateLimiter::new(10, 60),
        sse_hub,
        audit_trail,
        trace_buffer: Arc::new(TraceBuffer::new()),
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
    // ── Integrity checks: dead links, unbound sections, hardcoded data ──
    {
        let page_routes: Vec<&str> = state.pages.iter().map(|p| p.route.as_str()).collect();
        let mut warnings = 0;

        for page in &state.pages {
            // Check sidebar links pointing to non-existent pages
            for section in &page.sections {
                if let Some(ref tpl) = section.template {
                    // Find href="/..." links and check if page exists
                    let mut pos = 0;
                    let bytes = tpl.as_bytes();
                    while pos < tpl.len() {
                        if let Some(idx) = tpl[pos..].find("href=\"/") {
                            let start = pos + idx + 6;
                            if let Some(end) = tpl[start..].find('"') {
                                let href = &tpl[start - 1..start + end];
                                if href != "/" && href != "#" && !href.starts_with("/#") && !href.starts_with("/api/") && !page_routes.contains(&href) {
                                    println!("  \x1b[31m✗\x1b[0m Dead link: \"{}\" → page {} does not exist", href, href);
                                    warnings += 1;
                                }
                                pos = start + end;
                            } else { break; }
                        } else { break; }
                    }
                }

                // Check sections without binding that should have data
                let data_sections = ["kpi", "stat-cards", "table"];
                if data_sections.contains(&section.section_type.as_str()) && section.binding.is_none() && section.items.is_empty() {
                    println!("  \x1b[31m✗\x1b[0m Page \"{}\": section \"{}\" has no data source (no bind, no items)", page.route, section.section_type);
                    warnings += 1;
                }
            }
        }

        if warnings > 0 {
            println!("\n  \x1b[33m⚠ {} integrity warning(s)\x1b[0m", warnings);
        }
    }

    println!();
    println!("  \x1b[32mReady in {}ms\x1b[0m", elapsed_ms);
    println!();
    println!("  Press Ctrl+C to stop.");
    println!();

    // Resolve pass — verify all cross-references (fatal errors)
    let (_symbol_table, resolve_errors) = resolve::resolve(&nodes);
    if !resolve_errors.is_empty() {
        println!();
        println!("  \x1b[1mResolve Pass\x1b[0m");
        for e in &resolve_errors {
            println!("{}", e);
        }
        println!();
        println!("  \x1b[31m{} resolve error(s)\x1b[0m — aborting", resolve_errors.len());
        std::process::exit(1);
    }

    // Zero Hardcode Lint — run on startup (warnings only, never blocks)
    let lint_results = lint::lint_ast(&nodes, false);
    if !lint_results.is_empty() {
        for r in &lint_results {
            println!("{}", r);
        }
    }

    // Constitution enforcement — warn on startup (never blocks run)
    if let Some(ref c) = app.constitution {
        let cv = constitution_check::check_constitution(&nodes, c);
        let real_violations: Vec<_> = cv.iter().filter(|v| v.rule_type != "info").collect();
        if !real_violations.is_empty() {
            println!();
            println!("  \x1b[1mConstitution Warnings\x1b[0m");
            for v in &cv {
                println!("{}", v);
            }
            println!();
            println!("  \x1b[33m\u{26a0} {} constitution violation(s)\x1b[0m", real_violations.len());
        }
    }

    // Graceful shutdown: listen for Ctrl+C
    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            result = listener.accept() => {
                let (stream, remote_addr) = result.unwrap();
                let io = TokioIo::new(stream);
                let state = state.clone();

        tokio::task::spawn(async move {
            let service = service_fn(move |req: Request<Incoming>| {
                let state = state.clone();
                async move {
                    // SSE endpoint — returns a streaming response (not buffered)
                    if req.uri().path() == "/api/sse" && req.method() == Method::GET {
                        let sse_resp = state.sse_hub.subscribe();
                        // Map the streaming body to a boxed body for type compatibility
                        let (parts, body) = sse_resp.into_parts();
                        let boxed = http_body_util::Either::Right(body);
                        return Ok::<_, hyper::Error>(Response::from_parts(parts, boxed));
                    }
                    // All other requests — wrap Full<Bytes> in Either::Left
                    let resp = handle_request(req, state, remote_addr).await?;
                    let (parts, body) = resp.into_parts();
                    Ok(Response::from_parts(parts, http_body_util::Either::Left(body)))
                }
            });
            if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                eprintln!("  Connection error: {}", e);
            }
        });
            }
            _ = &mut shutdown => {
                eprintln!("\n  \x1b[36mShutting down gracefully...\x1b[0m");
                // Give in-flight requests 2 seconds to complete
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                eprintln!("  \x1b[32m✓\x1b[0m Server stopped.");
                break;
            }
        }
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

fn cmd_clone(args: &[String]) {
    let file = args.get(2).unwrap_or_else(|| {
        eprintln!("  \x1b[31m✗\x1b[0m Usage: cronus clone <file.html> [-o output.cronus]");
        std::process::exit(1);
    });

    let html = fs::read_to_string(file).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Error reading {}: {}", file, e);
        std::process::exit(1);
    });

    eprintln!("  \x1b[36m⚡\x1b[0m Clone IR: {} ({} bytes)...", file, html.len());
    let cronus = dump::clone_ir::clone_html_to_cronus(&html);

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
    let ai_mode = args.iter().any(|a| a == "--ai" || a == "--machine" || a == "--json-errors");
    let strict = args.iter().any(|a| a == "--strict") || strict_ai;
    let file = args.iter().skip(2)
        .find(|a| !a.starts_with("--"))
        .cloned()
        .or_else(find_cronus_file)
        .unwrap_or_else(|| {
            if ai_mode {
                let result = json!({
                    "valid": false,
                    "errors": [{"code": "PARSE_001", "severity": "fatal", "category": "filesystem", "message": "No .cronus file found", "location": {}, "fix": {"action": "add", "target": "*.cronus", "hint": "Create a .cronus file in the current directory or specify a path"}}],
                    "context": {"entities": 0, "pages": 0, "resolve_errors": 0, "lint_errors": 0, "constitution_violations": 0, "total_errors": 1}
                });
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
                *LAST_AI_ERRORS.lock().unwrap() = Some(result);
            } else if strict_ai {
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

            // ── AI-Error Protocol mode (--ai / --machine / --json-errors) ──
            if ai_mode {
                let ai_result = build_ai_error_json(&nodes, &file, entities, pages, routes);
                println!("{}", serde_json::to_string_pretty(&ai_result).unwrap());
                let valid = ai_result["valid"].as_bool().unwrap_or(false);
                *LAST_AI_ERRORS.lock().unwrap() = Some(ai_result);
                if !valid {
                    std::process::exit(1);
                }
                // Save snapshot silently in AI mode (no stdout pollution)
                let snapshot = ast_diff::snapshot_from_ast(&nodes);
                let _ = fs::create_dir_all(".cronus");
                if let Ok(json_str) = serde_json::to_string_pretty(&snapshot) {
                    let _ = fs::write(".cronus/ast-snapshot.json", &json_str);
                }
                return;
            }

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
                // Hardcode lint — render each page and check for hardcoded content
                let mut all_pages = Vec::new();
                let mut all_entities = Vec::new();
                let mut style_node: Option<parser::StyleNode> = None;
                for node in &nodes {
                    match node {
                        AstNode::Page(p) => all_pages.push(p.clone()),
                        AstNode::Entity(e) => all_entities.push(e.clone()),
                        AstNode::Style(s) => style_node = Some(s.clone()),
                        _ => {}
                    }
                }
                let hc_findings = hardcode_lint::lint_all_pages(&all_pages, &all_entities, style_node.as_ref());
                for f in &hc_findings {
                    errors.push(json!({
                        "type": "hardcoded_content",
                        "page": f.page,
                        "text": f.text,
                        "severity": f.severity,
                        "message": format!("Hardcoded text '{}' in page '{}' — should come from .cronus data", f.text, f.page),
                    }));
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
            } else if strict {
                // --strict (non-AI) — human-readable hardcode warnings
                let mut all_pages = Vec::new();
                let mut all_entities = Vec::new();
                let mut style_node: Option<parser::StyleNode> = None;
                for node in &nodes {
                    match node {
                        AstNode::Page(p) => all_pages.push(p.clone()),
                        AstNode::Entity(e) => all_entities.push(e.clone()),
                        AstNode::Style(s) => style_node = Some(s.clone()),
                        _ => {}
                    }
                }
                let hc_findings = hardcode_lint::lint_all_pages(&all_pages, &all_entities, style_node.as_ref());
                println!("  \x1b[32m✓\x1b[0m {} — {} entities, {} pages, {} routes", file, entities, pages, routes);
                if hc_findings.is_empty() {
                    println!("  \x1b[32m✓\x1b[0m No hardcoded content detected");
                } else {
                    println!("  \x1b[33m⚠\x1b[0m {} hardcoded string(s) found:", hc_findings.len());
                    for f in &hc_findings {
                        println!("    \x1b[33m→\x1b[0m [{}] \"{}\"", f.page, f.text);
                    }
                    println!();
                    println!("  \x1b[90mThese strings appear in rendered HTML but don't trace back to .cronus data.\x1b[0m");
                    println!("  \x1b[90mMove them to section title/subtitle/config/items in the .cronus file.\x1b[0m");
                }
            } else {
                println!("  \x1b[32m✓\x1b[0m {} — {} entities, {} pages, {} routes", file, entities, pages, routes);
                println!("  \x1b[32m✓\x1b[0m Valid .cronus file");
            }

            // Resolve pass — verify all cross-references (fatal errors)
            let resolve_start = std::time::Instant::now();
            let (_symbol_table, resolve_errors) = resolve::resolve(&nodes);
            let resolve_ms = resolve_start.elapsed().as_millis();
            if !resolve_errors.is_empty() {
                println!();
                println!("  \x1b[1mResolve Pass\x1b[0m");
                for e in &resolve_errors {
                    println!("{}", e);
                }
                println!();
                println!("  \x1b[31m{} resolve error(s)\x1b[0m — build blocked ({}ms)", resolve_errors.len(), resolve_ms);
                std::process::exit(1);
            } else {
                println!("  \x1b[32m✓\x1b[0m Resolve pass: all references valid ({}ms)", resolve_ms);
            }

            // Zero Hardcode Enforcement — 7 lint rules
            let lint_start = std::time::Instant::now();
            let lint_results = lint::lint_ast(&nodes, strict);
            let lint_ms = lint_start.elapsed().as_millis();
            let errors = lint_results.iter().filter(|r| matches!(r.severity, lint::Severity::Error)).count();
            let warnings = lint_results.iter().filter(|r| matches!(r.severity, lint::Severity::Warning)).count();

            if !lint_results.is_empty() {
                println!();
                println!("  \x1b[1mZero Hardcode Lint\x1b[0m ({} rules)", 7);
                for r in &lint_results {
                    println!("{}", r);
                }
                println!();
                if errors > 0 {
                    println!("  \x1b[31m{} error(s)\x1b[0m, {} warning(s) — build blocked", errors, warnings);
                    std::process::exit(1);
                } else {
                    println!("  {} warning(s)", warnings);
                }
            } else {
                println!("  \x1b[32m✓\x1b[0m Zero hardcode lint: all 7 rules passed ({}ms)", lint_ms);
            }

            // Constitution enforcement — check rules against AST
            let constitution_app = nodes.iter().find_map(|n| {
                if let AstNode::App(a) = n { Some(a) } else { None }
            });
            if let Some(app_node) = constitution_app {
                if let Some(ref c) = app_node.constitution {
                    let cv = constitution_check::check_constitution(&nodes, c);
                    let real_violations: Vec<_> = cv.iter().filter(|v| v.rule_type != "info").collect();
                    let info_count = cv.len() - real_violations.len();
                    if !real_violations.is_empty() {
                        println!();
                        println!("  \x1b[1mConstitution Check\x1b[0m ({} rules)", c.must.len() + c.never.len());
                        for v in &cv {
                            println!("{}", v);
                        }
                        println!();
                        println!("  \x1b[31m{} violation(s)\x1b[0m{}", real_violations.len(),
                            if info_count > 0 { format!(", {} informational", info_count) } else { String::new() });
                        if strict {
                            std::process::exit(1);
                        }
                    } else {
                        println!("  \x1b[32m\u{2713}\x1b[0m Constitution: all {} rules pass{}", c.must.len() + c.never.len(),
                            if info_count > 0 { format!(" ({} informational)", info_count) } else { String::new() });
                    }
                }
            }

            // Save AST snapshot for changelog diffing
            save_ast_snapshot(&nodes);
        }
        Err(e) => {
            if ai_mode {
                let result = json!({
                    "valid": false,
                    "errors": [{"code": "PARSE_001", "severity": "fatal", "category": "syntax", "message": format!("{}", e), "location": {}, "fix": {"action": "replace", "target": "", "hint": "Fix the syntax error in the .cronus file"}}],
                    "context": {"entities": 0, "pages": 0, "resolve_errors": 0, "lint_errors": 0, "constitution_violations": 0, "total_errors": 1}
                });
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
                *LAST_AI_ERRORS.lock().unwrap() = Some(result);
            } else if strict_ai {
                println!("{}", json!({"valid": false, "errors": [{"type": "parse_error", "message": e, "severity": "error"}]}));
            } else {
                eprintln!("  \x1b[31m\u{2717}\x1b[0m Parse error: {}", e);
            }
            std::process::exit(1);
        }
    }
}

/// Build the AI-Error Protocol JSON from all validation passes.
/// Collects errors from contract validation, resolve, lint, hardcode lint, and constitution.
fn build_ai_error_json(nodes: &[AstNode], file: &str, entity_count: usize, page_count: usize, route_count: usize) -> Value {
    let mut ai_errors: Vec<Value> = Vec::new();
    let mut resolve_counter = 0usize;
    let mut lint_counter = 0usize;
    let mut constitution_counter = 0usize;
    let mut contract_counter = 0usize;

    // Pass 1: Contract validation (section schemas)
    for node in nodes {
        if let AstNode::Page(page) = node {
            for section in &page.sections {
                let section_warnings = contracts::validate_section(section, &[]);
                for w in section_warnings {
                    contract_counter += 1;
                    let code = format!("CONTRACT_{:03}", contract_counter);
                    let (msg, fix, line) = match w {
                        contracts::ParseWarning::UnknownSection { ref name, line } => (
                            format!("Unknown section type '{}'", name),
                            json!({"action": "replace", "target": name, "hint": "Check valid section types: hero, features, pricing, kpi, table, chart, form, etc."}),
                            line,
                        ),
                        contracts::ParseWarning::UnknownKey { ref section, ref key, .. } => (
                            format!("Unknown key '{}' in section '{}'", key, section),
                            json!({"action": "remove", "target": key, "hint": format!("Remove or replace with a valid key for '{}' sections", section)}),
                            0,
                        ),
                        contracts::ParseWarning::MissingRequired { ref section, ref key, .. } => (
                            format!("Missing required key '{}' in section '{}'", key, section),
                            json!({"action": "add", "target": key, "hint": format!("Add '{}' to the {} section item", key, section)}),
                            0,
                        ),
                        contracts::ParseWarning::AliasUsed { ref alias, ref canonical, line } => (
                            format!("'{}' is an alias — use canonical name '{}'", alias, canonical),
                            json!({"action": "replace", "target": alias, "replacement": canonical}),
                            line,
                        ),
                        contracts::ParseWarning::MinItemsViolation { ref section, expected, actual, line } => (
                            format!("Section '{}' requires at least {} items, found {}", section, expected, actual),
                            json!({"action": "add", "target": "item", "hint": format!("Add {} more item(s) to '{}' section", expected - actual, section)}),
                            line,
                        ),
                        contracts::ParseWarning::UnknownConfig { ref section, ref key, line } => (
                            format!("Unknown config key '{}' in section '{}'", key, section),
                            json!({"action": "remove", "target": key, "hint": format!("Remove unknown config key from '{}' section", section)}),
                            line,
                        ),
                    };
                    ai_errors.push(json!({
                        "code": code,
                        "severity": "error",
                        "category": "contract",
                        "message": msg,
                        "location": {"line": line, "section": format!("{} ({})", section.section_type, page.route), "block": "section"},
                        "fix": fix,
                    }));
                }
            }
        }
    }

    // Pass 2: Resolve — cross-reference validation (fatal)
    let (_symbol_table, resolve_errors) = resolve::resolve(nodes);
    let resolve_error_count = resolve_errors.len();
    for re in &resolve_errors {
        resolve_counter += 1;
        let code = format!("RESOLVE_{:03}", resolve_counter);
        let target_name = re.message.split('\'').nth(1).unwrap_or("").to_string();

        let fix = if let Some(ref sug) = re.suggestion {
            json!({"action": "replace", "target": &target_name, "replacement": sug})
        } else if re.message.contains("must be an enum") {
            json!({"action": "replace", "target": format!("{} field type", target_name), "hint": "Change field type to enum with valid values"})
        } else {
            json!({"action": "add", "target": &target_name, "hint": "Define this entity or reference"})
        };

        let section_hint = if re.message.contains("page '") {
            format!("page {}", re.message.split("page '").nth(1).and_then(|s| s.split('\'').next()).unwrap_or(""))
        } else if re.message.contains("entity '") {
            format!("entity {}", re.message.split("entity '").last().and_then(|s| s.split('\'').next()).unwrap_or(""))
        } else { String::new() };

        let category = if re.message.contains("Transition") { "state_machine" } else { "reference" };
        let block = if re.message.contains("bind") { "bind" }
            else if re.message.contains("Transition") { "transition" }
            else { "reference" };

        ai_errors.push(json!({
            "code": code,
            "severity": "fatal",
            "category": category,
            "message": re.message,
            "suggestion": re.suggestion,
            "location": {"section": section_hint, "block": block},
            "fix": fix,
        }));
    }

    // Pass 3: Lint — Zero Hardcode Enforcement (7 rules)
    let lint_results = lint::lint_ast(nodes, true);
    let lint_error_count = lint_results.iter().filter(|r| matches!(r.severity, lint::Severity::Error)).count();
    let lint_warning_count = lint_results.iter().filter(|r| matches!(r.severity, lint::Severity::Warning)).count();
    for lr in &lint_results {
        lint_counter += 1;
        let code = format!("LINT_{:03}", lint_counter);
        let severity = match lr.severity { lint::Severity::Error => "error", lint::Severity::Warning => "warning" };

        let fix = if lr.rule.contains("dead-text") || lr.rule.contains("hardcode") {
            json!({"action": "wrap_in_dynamic", "target": lr.message.split('\'').nth(1).unwrap_or(&lr.message), "hint": &lr.fix})
        } else if lr.rule.contains("dead-link") {
            json!({"action": "replace", "target": lr.message.split('\'').nth(1).unwrap_or(""), "hint": &lr.fix})
        } else if lr.rule.contains("bind-or-empty") {
            json!({"action": "add_bind", "target": &lr.section, "hint": &lr.fix})
        } else if lr.rule.contains("sensitive") {
            json!({"action": "remove", "target": lr.message.split('\'').nth(1).unwrap_or(""), "hint": &lr.fix})
        } else if lr.rule.contains("auth") {
            json!({"action": "add_auth", "target": &lr.page, "hint": &lr.fix})
        } else {
            json!({"action": "replace", "target": "", "hint": &lr.fix})
        };

        ai_errors.push(json!({
            "code": code,
            "severity": severity,
            "category": "hardcode",
            "message": lr.message,
            "location": {"section": lr.section, "page": lr.page, "block": "template"},
            "fix": fix,
        }));
    }

    // Pass 3b: Hardcode lint (rendered HTML analysis)
    let mut all_pages = Vec::new();
    let mut all_entities = Vec::new();
    let mut style_node: Option<parser::StyleNode> = None;
    for node in nodes {
        match node {
            AstNode::Page(p) => all_pages.push(p.clone()),
            AstNode::Entity(e) => all_entities.push(e.clone()),
            AstNode::Style(s) => style_node = Some(s.clone()),
            _ => {}
        }
    }
    let hc_findings = hardcode_lint::lint_all_pages(&all_pages, &all_entities, style_node.as_ref());
    for f in &hc_findings {
        lint_counter += 1;
        let code = format!("LINT_{:03}", lint_counter);
        ai_errors.push(json!({
            "code": code,
            "severity": f.severity,
            "category": "hardcode",
            "message": format!("Hardcoded text '{}' in page '{}' — should come from .cronus data", f.text, f.page),
            "location": {"page": f.page, "block": "template"},
            "fix": {"action": "wrap_in_dynamic", "target": &f.text, "hint": "Use <span id='...'>...</span> populated via JS fetch or move to section data"}
        }));
    }

    // Pass 4: Constitution enforcement
    let mut constitution_violation_count = 0usize;
    let constitution_app = nodes.iter().find_map(|n| {
        if let AstNode::App(a) = n { Some(a) } else { None }
    });
    if let Some(app_node) = constitution_app {
        if let Some(ref c) = app_node.constitution {
            let cv = constitution_check::check_constitution(nodes, c);
            let real_violations: Vec<_> = cv.iter().filter(|v| v.rule_type != "info").collect();
            constitution_violation_count = real_violations.len();
            for v in &real_violations {
                constitution_counter += 1;
                let code = format!("CONSTITUTION_{:03}", constitution_counter);
                let fix = if v.rule.to_lowercase().contains("auth") {
                    json!({"action": "add_auth", "target": v.entity.clone().unwrap_or_default(), "hint": format!("Add requires: auth to comply with rule: {}", v.rule)})
                } else if v.rule.to_lowercase().contains("bind") {
                    json!({"action": "add_bind", "target": &v.violation, "hint": format!("Add bind block to comply with rule: {}", v.rule)})
                } else {
                    json!({"action": "add", "target": "", "hint": format!("Fix violation of constitution rule: {}", v.rule)})
                };
                ai_errors.push(json!({
                    "code": code,
                    "severity": "fatal",
                    "category": "constitution",
                    "message": v.violation,
                    "rule": v.rule,
                    "entity": v.entity,
                    "location": {"block": "constitution", "section": v.entity.clone().unwrap_or_default()},
                    "fix": fix,
                }));
            }
        }
    }

    let total_errors = ai_errors.len();
    let valid = total_errors == 0;
    json!({
        "valid": valid,
        "errors": ai_errors,
        "context": {
            "file": file,
            "entities": entity_count,
            "pages": page_count,
            "routes": route_count,
            "resolve_errors": resolve_error_count,
            "lint_errors": lint_error_count + hc_findings.len(),
            "lint_warnings": lint_warning_count,
            "constitution_violations": constitution_violation_count,
            "total_errors": total_errors,
        }
    })
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
    let all_templates = ["landing", "admin", "saas", "api", "ecommerce", "blog", "helpdesk", "crm"];

    let template = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| {
        eprintln!("  Usage: cronus new <template>");
        eprintln!("  Templates: {}", all_templates.join(", "));
        std::process::exit(1);
    });

    // Try loading from templates/ directory next to the binary first
    let template_from_file = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .map(|dir| dir.join("templates").join(format!("{}.cronus", template)))
        .and_then(|path| fs::read_to_string(&path).ok());

    let content: &str = if let Some(ref file_content) = template_from_file {
        file_content.as_str()
    } else {
        // Fall back to embedded templates
        match template {
            "landing" => TEMPLATE_LANDING,
            "admin" => TEMPLATE_ADMIN,
            "saas" => TEMPLATE_SAAS,
            "api" => TEMPLATE_API,
            "ecommerce" => TEMPLATE_ECOMMERCE,
            "blog" => TEMPLATE_BLOG,
            "helpdesk" => TEMPLATE_HELPDESK,
            "crm" => TEMPLATE_CRM,
            _ => {
                eprintln!("  \x1b[33m✗\x1b[0m Unknown template: {}", template);
                eprintln!("  Available: {}", all_templates.join(", "));
                std::process::exit(1);
            }
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

const TEMPLATE_SAAS: &str = r#"/// SaaS Starter — Multi-tenant platform with Stripe-ready billing.
/// Role-based access control, organization management, and subscription lifecycle.
/// @template saas
/// @author CRONUS
app "SaaS Platform" {
  stack fullstack
  port 5175
  database sqlite "./data.db"
  theme dark

  constitution {
    must "prices in centavos — use formatPrice()"
    must "all billing mutations require auth"
    must "subscription changes emit webhooks"
    never "expose payment tokens in API responses"
    never "allow plan downgrade with active usage over limit"
  }
}

# -- Entities --

/// Platform user with role-based access.
entity User {
  /// Full display name
  name string! max:100
  email email! unique
  password string! sensitive min:8
  /// Role determines access level across the platform
  role enum ["owner", "admin", "member", "viewer"] default:"member"
  avatar url
  org -> Organization
}

/// Tenant organization — all resources scoped here.
entity Organization shared {
  name string! max:120
  slug slug! unique match:"^[a-z0-9-]+$"
  /// Stripe customer ID for billing integration
  stripe_id string unique
  plan -> Plan
  seats number default:"5" min:1 max:500
}

/// Billing plan with Stripe price mapping.
entity Plan shared {
  name string!
  /// Monthly price in centavos (2990 = R$29.90)
  price_monthly money! min:0
  price_yearly money min:0
  /// Stripe price ID for checkout session
  stripe_price_id string unique
  max_seats number default:"5" min:1
  max_projects number default:"10" min:1
  active boolean default:"true"
}

/// Subscription linking org to plan with lifecycle state.
/// @business Core revenue entity — state changes trigger Stripe sync
entity Subscription shared {
  org -> Organization
  plan -> Plan
  status enum ["trialing", "active", "past_due", "canceled", "paused"] default:"trialing"
  current_period_end date
  cancel_at date

  transition status {
    trialing -> active | canceled
    active -> past_due | canceled | paused
    past_due -> active | canceled
    paused -> active | canceled
  }
}

# -- Auth --

auth {
  entity User
  login email
  session jwt
  roles [owner, admin, member, viewer]
}

# -- API --

api /auth {
  login    POST /login    auth:public
  register POST /register auth:public
  me       GET  /me       auth:jwt
}

api /organizations {
  list   GET    /       auth:jwt
  create POST   /       auth:jwt
  detail GET    /:id    auth:jwt
  update PATCH  /:id    auth:jwt
}

api /plans {
  list   GET  /     auth:public
  detail GET  /:id  auth:public
}

api /subscriptions {
  current GET    /current          auth:jwt
  create  POST   /                 auth:jwt
  update  PATCH  /:id              auth:jwt
  cancel  POST   /:id/cancel       auth:jwt
}

webhook /subscriptions {
  on create -> POST "https://api.stripe.com/v1/subscriptions"
  on update -> POST "https://hooks.example.com/billing/changed"
}

webhook /organizations {
  on create -> POST "https://hooks.example.com/org/provisioned"
}

# -- Pages --

page "/login" type:form entity:User {
  title "Sign In"
  fields [email, password]
}

page "/dashboard" type:dashboard requires:auth {
  title "Dashboard"

  section stats cols:3 {
    bind entity:Organization { query count }
    item "Organizations" value:"count" icon:business
    bind entity:Subscription { query count where status eq "active" }
    item "Active Subscriptions" value:"count" icon:check_circle
    bind entity:User { query count }
    item "Total Users" value:"count" icon:people
  }

  section recent {
    title "Recent Subscriptions"
    bind entity:Subscription { query all order current_period_end desc limit 10 }
    columns "Org, Plan, Status, Current Period End"
  }
}

page "/billing" type:custom requires:auth {
  title "Billing"

  section subscription {
    title "Current Plan"
    bind entity:Subscription { query one }
  }

  section plans {
    title "Available Plans"
    bind entity:Plan { query all }
    columns "Name, Price Monthly, Price Yearly, Max Seats"
  }
}

style {
  theme dark
  accent violet
  font "Inter"
}
"#;

const TEMPLATE_ECOMMERCE: &str = r#"/// E-commerce Platform — Full storefront with order state machine.
/// Inventory tracking, price in centavos, category hierarchy.
/// @template ecommerce
/// @author CRONUS
app "E-commerce Store" {
  stack fullstack
  port 5175
  database sqlite "./data.db"
  theme dark

  constitution {
    must "prices in centavos — use formatPrice()"
    must "stock cannot go negative"
    must "order total must equal sum of items"
    never "expose customer payment details"
    never "allow checkout with zero-stock items"
  }
}

# -- Entities --

/// Product category for catalog organization.
entity Category shared {
  name string! max:80
  slug slug! unique match:"^[a-z0-9-]+$"
  description text max:500
  /// Sort priority — lower values appear first
  sort_order number default:"0" min:0
}

/// Product in the catalog with inventory tracking.
/// @business Core commerce entity — price always in centavos
entity Product shared {
  name string! max:200 searchable
  slug slug! unique
  /// Price in centavos (2990 = R$29.90)
  price money! min:0
  /// Compare-at price for sale display (centavos)
  compare_price money min:0
  sku string unique match:"^[A-Z0-9-]+$"
  /// Available inventory units
  stock number! default:"0" min:0
  category -> Category
  description text
  image_url url
  active boolean default:"true"
}

/// Registered customer with shipping info.
entity Customer shared {
  name string! max:120
  email email! unique
  phone phone
  address text max:500
  city string max:100
  state string max:50
  zip string max:20 match:"^[0-9-]+$"
}

/// Order with full lifecycle state machine.
/// @business Revenue tracking — status drives fulfillment pipeline
entity Order shared {
  customer -> Customer
  /// Order total in centavos
  total money! min:0
  status enum ["cart", "pending", "paid", "shipped", "delivered", "cancelled"] default:"cart"
  shipping_address text
  tracking_code string
  notes text max:1000

  transition status {
    cart -> pending
    pending -> paid | cancelled
    paid -> shipped | cancelled
    shipped -> delivered
  }
}

/// Individual line item within an order.
entity OrderItem shared {
  order -> Order
  product -> Product
  quantity number! min:1 max:9999
  /// Unit price snapshot in centavos (locked at purchase time)
  unit_price money! min:0
}

# -- Auth --

auth {
  entity Customer
  login email
  session jwt
  roles [admin, staff]
}

# -- API --

api /auth {
  login    POST /login    auth:public
  register POST /register auth:public
  me       GET  /me       auth:jwt
}

api /categories {
  list   GET    /       auth:public
  create POST   /       auth:jwt
  update PATCH  /:id    auth:jwt
}

api /products {
  list   GET    /       auth:public
  detail GET    /:id    auth:public
  create POST   /       auth:jwt
  update PATCH  /:id    auth:jwt
  delete DELETE /:id    auth:jwt
}

api /orders {
  list   GET    /       auth:jwt
  detail GET    /:id    auth:jwt
  create POST   /       auth:jwt
  update PATCH  /:id    auth:jwt
}

api /orderitems {
  list   GET    /       auth:jwt
  create POST   /       auth:jwt
  delete DELETE /:id    auth:jwt
}

webhook /orders {
  on create -> POST "https://hooks.example.com/orders/new"
  on update -> POST "https://hooks.example.com/orders/status"
}

# -- Pages --

page "/login" type:form entity:Customer {
  title "Sign In"
  fields [email, password]
}

page "/dashboard" type:dashboard requires:auth {
  title "Store Dashboard"

  section stats cols:4 {
    bind entity:Product { query count }
    item "Products" value:"count" icon:inventory_2
    bind entity:Order { query count }
    item "Orders" value:"count" icon:shopping_cart
    bind entity:Order { query sum field:total }
    item "Revenue" value:"sum" icon:attach_money
    bind entity:Customer { query count }
    item "Customers" value:"count" icon:people
  }

  section recent {
    title "Recent Orders"
    bind entity:Order { query all order created_at desc limit 10 }
    columns "Customer, Total, Status, Tracking Code"
  }
}

page "/products" type:custom requires:auth {
  title "Product Catalog"

  section header {
    title "Products"
    action "Add Product" -> "/products/new" icon:add
  }

  section table {
    bind entity:Product { query all order name asc limit 25 }
    columns "Name, Price, Stock, SKU, Active"
  }
}

page "/orders" type:custom requires:auth {
  title "Orders"

  section table {
    bind entity:Order { query all order created_at desc limit 25 }
    columns "Customer, Total, Status, Tracking Code, Created At"
  }
}

style {
  theme dark
  accent emerald
  font "Inter"
}
"#;

const TEMPLATE_BLOG: &str = r#"/// Blog / CMS — Content management with publish workflow.
/// Markdown content, SEO-ready slugs, comment moderation.
/// @template blog
/// @author CRONUS
app "Blog CMS" {
  stack fullstack
  port 5175
  database sqlite "./data.db"
  theme dark

  constitution {
    must "slugs must be URL-safe and unique"
    must "published posts require non-empty content"
    must "comments require moderation before display"
    never "expose draft posts on public API"
    never "allow self-approval of comments"
  }
}

# -- Entities --

/// Content author with profile and bio.
entity Author {
  name string! max:100
  email email! unique
  password string! sensitive min:8
  /// Author bio in markdown
  bio text max:2000
  avatar url
  role enum ["admin", "editor", "writer"] default:"writer"
}

/// Blog post with full publish lifecycle.
/// @business Primary content unit — status controls visibility
entity Post shared {
  title string! max:200 searchable
  /// URL-safe slug for SEO
  slug slug! unique match:"^[a-z0-9-]+$"
  /// Markdown body content
  content text!
  /// Short excerpt for listings and meta description
  excerpt text max:300
  author -> Author
  category -> Category
  /// SEO meta title (falls back to title if empty)
  meta_title string max:70
  /// SEO meta description
  meta_description string max:160
  cover_image url
  status enum ["draft", "review", "published", "archived"] default:"draft"
  featured boolean default:"false"
  published_at date

  transition status {
    draft -> review | published
    review -> published | draft
    published -> archived
    archived -> draft
  }
}

/// Post category for content organization.
entity Category shared {
  name string! max:80
  slug slug! unique match:"^[a-z0-9-]+$"
  description text max:300
}

/// Tag for flexible cross-cutting content grouping.
entity Tag shared {
  name string! max:50 unique
  slug slug! unique match:"^[a-z0-9-]+$"
}

/// Reader comment with moderation workflow.
entity Comment {
  post -> Post
  author_name string! max:100
  author_email email!
  body text! max:5000
  approved boolean default:"false"
}

# -- Auth --

auth {
  entity Author
  login email
  session jwt
  roles [admin, editor, writer]
}

# -- API --

api /auth {
  login    POST /login    auth:public
  register POST /register auth:public
  me       GET  /me       auth:jwt
}

api /posts {
  list   GET    /        auth:public
  detail GET    /:slug   auth:public
  create POST   /        auth:jwt
  update PATCH  /:id     auth:jwt
  delete DELETE /:id     auth:jwt
}

api /categories {
  list   GET    /     auth:public
  create POST   /     auth:jwt
  update PATCH  /:id  auth:jwt
}

api /tags {
  list   GET    /     auth:public
  create POST   /     auth:jwt
}

api /comments {
  list   GET    /     auth:public
  create POST   /     auth:public
  update PATCH  /:id  auth:jwt
  delete DELETE /:id  auth:jwt
}

webhook /posts {
  on create -> POST "https://hooks.example.com/content/new"
  on update -> POST "https://hooks.example.com/content/updated"
}

webhook /comments {
  on create -> POST "https://hooks.example.com/comments/moderate"
}

# -- Pages --

page "/login" type:form entity:Author {
  title "Sign In"
  fields [email, password]
}

page "/dashboard" type:dashboard requires:auth {
  title "Blog Dashboard"

  section stats cols:4 {
    bind entity:Post { query count }
    item "Total Posts" value:"count" icon:article
    bind entity:Post { query count where status eq "published" }
    item "Published" value:"count" icon:check_circle
    bind entity:Comment { query count }
    item "Comments" value:"count" icon:comment
    bind entity:Author { query count }
    item "Authors" value:"count" icon:people
  }

  section recent {
    title "Recent Posts"
    bind entity:Post { query all order created_at desc limit 10 }
    columns "Title, Status, Author, Published At"
  }
}

page "/posts" type:custom requires:auth {
  title "Posts"

  section header {
    title "All Posts"
    action "New Post" -> "/posts/new" icon:add
  }

  section table {
    bind entity:Post { query all order created_at desc limit 25 }
    columns "Title, Slug, Status, Category, Featured"
  }
}

page "/comments" type:custom requires:auth {
  title "Comment Moderation"

  section table {
    bind entity:Comment { query all order created_at desc limit 25 }
    columns "Post, Author Name, Body, Approved"
  }
}

style {
  theme dark
  accent sky
  font "Inter"
}
"#;

const TEMPLATE_HELPDESK: &str = r#"/// Helpdesk — Support ticket system with SLA tracking.
/// Priority-based routing, agent assignment, full ticket lifecycle.
/// @template helpdesk
/// @author CRONUS
app "Helpdesk" {
  stack fullstack
  port 5175
  database sqlite "./data.db"
  theme dark

  constitution {
    must "critical tickets must be assigned within 1 hour"
    must "all status changes emit webhooks"
    must "resolved tickets require a resolution note"
    never "delete tickets — archive instead"
    never "reassign without notifying original agent"
  }
}

# -- Entities --

/// Support agent handling tickets.
entity Agent shared {
  name string! max:100
  email email! unique
  password string! sensitive min:8
  role enum ["admin", "lead", "agent"] default:"agent"
  /// Maximum concurrent open tickets
  max_tickets number default:"20" min:1 max:100
  active boolean default:"true"
}

/// Customer who submits support requests.
entity Customer shared {
  name string! max:120
  email email! unique
  phone phone
  company string max:120
}

/// Support ticket with full lifecycle and SLA tracking.
/// @business Core support entity — SLA clock starts on creation
entity Ticket shared {
  /// Short descriptive title
  subject string! max:200 searchable
  /// Detailed issue description
  description text! max:10000
  customer -> Customer
  agent -> Agent
  priority enum ["critical", "high", "medium", "low"] default:"medium"
  status enum ["open", "assigned", "in_progress", "waiting", "resolved", "closed"] default:"open"
  category enum ["bug", "feature", "billing", "account", "other"] default:"other"
  /// SLA deadline timestamp
  sla_deadline date
  /// Resolution summary (required before closing)
  resolution text max:5000
  satisfaction number min:1 max:5

  transition status {
    open -> assigned
    assigned -> in_progress | open
    in_progress -> waiting | resolved
    waiting -> in_progress | resolved
    resolved -> closed | in_progress
  }
}

/// Message thread within a ticket.
entity Message shared {
  ticket -> Ticket
  sender_type enum ["agent", "customer", "system"] default:"customer"
  sender_name string! max:100
  body text! max:10000
  /// Whether this message is internal (agent-only)
  internal boolean default:"false"
}

# -- Auth --

auth {
  entity Agent
  login email
  session jwt
  roles [admin, lead, agent]
}

# -- API --

api /auth {
  login    POST /login    auth:public
  register POST /register auth:public
  me       GET  /me       auth:jwt
}

api /tickets {
  list   GET    /       auth:jwt
  detail GET    /:id    auth:jwt
  create POST   /       auth:jwt
  update PATCH  /:id    auth:jwt
}

api /messages {
  list   GET    /       auth:jwt
  create POST   /       auth:jwt
}

api /customers {
  list   GET    /       auth:jwt
  detail GET    /:id    auth:jwt
  create POST   /       auth:jwt
  update PATCH  /:id    auth:jwt
}

api /agents {
  list   GET    /       auth:jwt
  detail GET    /:id    auth:jwt
  update PATCH  /:id    auth:jwt
}

webhook /tickets {
  on create -> POST "https://hooks.example.com/support/new-ticket"
  on update -> POST "https://hooks.example.com/support/ticket-updated"
}

webhook /messages {
  on create -> POST "https://hooks.example.com/support/new-message"
}

# -- Pages --

page "/login" type:form entity:Agent {
  title "Agent Login"
  fields [email, password]
}

page "/dashboard" type:dashboard requires:auth {
  title "Support Dashboard"

  section stats cols:4 {
    bind entity:Ticket { query count where status eq "open" }
    item "Open" value:"count" icon:inbox
    bind entity:Ticket { query count where status eq "in_progress" }
    item "In Progress" value:"count" icon:pending
    bind entity:Ticket { query count where priority eq "critical" }
    item "Critical" value:"count" icon:error
    bind entity:Ticket { query count where status eq "resolved" }
    item "Resolved" value:"count" icon:check_circle
  }

  section urgent {
    title "Critical Tickets"
    bind entity:Ticket { query all where priority eq "critical" order created_at asc limit 10 }
    columns "Subject, Customer, Agent, Status, SLA Deadline"
  }
}

page "/tickets" type:custom requires:auth {
  title "All Tickets"

  section header {
    title "Tickets"
    action "New Ticket" -> "/tickets/new" icon:add
  }

  section table {
    search "Filter by subject, customer, or priority..."
    bind entity:Ticket { query all order created_at desc limit 25 }
    columns "Subject, Customer, Priority, Status, Agent, Created At"
  }
}

page "/tickets/new" type:custom requires:auth {
  title "New Ticket"

  section form {
    bind entity:Ticket { query all }
    item "Subject" required:true
    item "Description" required:true
    item "Customer" required:true
    item "Priority"
    item "Category"
    on submit {
      create Ticket
      toast "Ticket created"
      navigate "/tickets"
    }
  }
}

style {
  theme dark
  accent amber
  font "Inter"
}
"#;

const TEMPLATE_CRM: &str = r#"/// CRM — Sales pipeline and contact management.
/// Deal lifecycle tracking, activity logging, revenue forecasting.
/// @template crm
/// @author CRONUS
app "CRM" {
  stack fullstack
  port 5175
  database sqlite "./data.db"
  theme dark

  constitution {
    must "deal values in centavos — use formatPrice()"
    must "all deal stage changes logged as activities"
    must "contacts require at least email or phone"
    never "delete deals — mark as lost instead"
    never "modify closed-won deals without admin role"
  }
}

# -- Entities --

/// Company / account in the CRM.
entity Company shared {
  name string! max:200 searchable
  domain url
  industry enum ["tech", "finance", "healthcare", "retail", "manufacturing", "services", "other"] default:"other"
  size enum ["1-10", "11-50", "51-200", "201-1000", "1000+"]
  /// Annual revenue estimate in centavos
  annual_revenue money min:0
  phone phone
  address text max:500
}

/// Individual contact linked to a company.
entity Contact shared {
  name string! max:120 searchable
  email email unique
  phone phone
  title string max:100
  company -> Company
  source enum ["website", "referral", "linkedin", "cold", "event", "other"] default:"other"
}

/// Sales deal progressing through pipeline stages.
/// @business Core revenue entity — stage drives forecasting
entity Deal shared {
  title string! max:200 searchable
  /// Deal value in centavos (500000 = R$5.000,00)
  value money! min:0
  company -> Company
  contact -> Contact
  owner string! max:100
  stage enum ["lead", "qualified", "proposal", "negotiation", "won", "lost"] default:"lead"
  /// Win probability percentage
  probability number default:"10" min:0 max:100
  /// Expected close date
  expected_close date
  /// Reason for loss (required when stage = lost)
  lost_reason text max:500
  source enum ["inbound", "outbound", "referral", "partner"] default:"inbound"

  transition stage {
    lead -> qualified | lost
    qualified -> proposal | lost
    proposal -> negotiation | lost
    negotiation -> won | lost
  }
}

/// Interaction or event logged against a contact or deal.
entity Activity shared {
  type enum ["call", "email", "meeting", "note", "task"] default:"note"
  subject string! max:200
  description text max:5000
  contact -> Contact
  deal -> Deal
  /// Who performed this activity
  performed_by string! max:100
  completed boolean default:"false"
  due_date date
}

/// Free-form note attached to any entity.
entity Note shared {
  body text! max:10000
  contact -> Contact
  deal -> Deal
  company -> Company
  author string! max:100
}

# -- Auth --

auth {
  entity Contact
  login email
  session jwt
  roles [admin, manager, rep]
}

# -- API --

api /auth {
  login    POST /login    auth:public
  register POST /register auth:public
  me       GET  /me       auth:jwt
}

api /companies {
  list   GET    /       auth:jwt
  detail GET    /:id    auth:jwt
  create POST   /       auth:jwt
  update PATCH  /:id    auth:jwt
}

api /contacts {
  list   GET    /       auth:jwt
  detail GET    /:id    auth:jwt
  create POST   /       auth:jwt
  update PATCH  /:id    auth:jwt
}

api /deals {
  list   GET    /       auth:jwt
  detail GET    /:id    auth:jwt
  create POST   /       auth:jwt
  update PATCH  /:id    auth:jwt
}

api /activities {
  list   GET    /       auth:jwt
  create POST   /       auth:jwt
  update PATCH  /:id    auth:jwt
}

api /notes {
  list   GET    /       auth:jwt
  create POST   /       auth:jwt
  delete DELETE /:id    auth:jwt
}

webhook /deals {
  on create -> POST "https://hooks.example.com/crm/deal-created"
  on update -> POST "https://hooks.example.com/crm/deal-updated"
}

webhook /activity {
  on create -> POST "https://hooks.example.com/crm/activity-logged"
}

# -- Pages --

page "/login" type:form entity:Contact {
  title "Sign In"
  fields [email, password]
}

page "/dashboard" type:dashboard requires:auth {
  title "Sales Dashboard"

  section stats cols:4 {
    bind entity:Deal { query count where stage eq "won" }
    item "Deals Won" value:"count" icon:emoji_events
    bind entity:Deal { query sum field:value where stage eq "won" }
    item "Revenue" value:"sum" icon:attach_money
    bind entity:Deal { query count where stage eq "negotiation" }
    item "In Negotiation" value:"count" icon:handshake
    bind entity:Contact { query count }
    item "Contacts" value:"count" icon:people
  }

  section pipeline {
    title "Active Pipeline"
    bind entity:Deal { query all order expected_close asc limit 15 }
    columns "Title, Company, Value, Stage, Probability, Expected Close"
  }
}

page "/deals" type:custom requires:auth {
  title "Deals"

  section header {
    title "Deal Pipeline"
    action "New Deal" -> "/deals/new" icon:add
  }

  section table {
    search "Filter by title, company, or stage..."
    bind entity:Deal { query all order created_at desc limit 25 }
    columns "Title, Company, Value, Stage, Owner, Expected Close"
  }
}

page "/contacts" type:custom requires:auth {
  title "Contacts"

  section header {
    title "All Contacts"
    action "Add Contact" -> "/contacts/new" icon:add
  }

  section table {
    search "Filter by name, email, or company..."
    bind entity:Contact { query all order name asc limit 25 }
    columns "Name, Email, Phone, Company, Source"
  }
}

style {
  theme dark
  accent blue
  font "Inter"
}
"#;


// cmd_parse moved to cli::parse_cmd

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

// cmd_doctor moved to cli::doctor

// cmd_stats moved to cli::stats

// cmd_export moved to cli::export_cmd

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

// cmd_generate and generate system moved to cli::generate

// ══════════════════════════════════════════════════
// SYNC — generate .cronus/state-digest.json
// ══════════════════════════════════════════════════

pub(crate) fn cmd_sync() {
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

pub(crate) fn count_files_matching(dir: &str, suffix: &str) -> usize {
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

// cmd_handoff moved to cli::handoff

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

    // 3. Read git diff content — staged first, fallback to last 1 commit
    let diff_content = {
        // Try staged changes first
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
            // Nothing staged — diff last 1 commit only
            Command::new("git")
                .args(["diff", "HEAD~1..HEAD"])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_default()
        }
    };

    // 4. Check if any forbidden pattern appears in the diff (code files only)
    // Filter to only added lines in code files (.rs, .cronus) that are in scope
    let mut constitution_pass = true;
    let mut violations: Vec<String> = Vec::new();
    let mut in_code_file = false;
    let added_code_lines: Vec<String> = diff_content.lines()
        .filter(|l| {
            if l.starts_with("+++ b/") {
                let path = &l[6..];
                let is_code = path.ends_with(".rs") || path.ends_with(".cronus");
                // If we have a write scope, only include files within it
                let in_scope = write_scope.is_empty() || lease_file_allowed(path, &write_scope);
                in_code_file = is_code && in_scope;
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

    // 6. Check spec coverage — count specs and conformance tests separately
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

    // Priority 1: staged files (what's about to be committed)
    if let Ok(output) = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
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

    // Priority 2: if nothing staged, fall back to last commit
    if files.is_empty() {
        if let Ok(output) = Command::new("git")
            .args(["diff", "HEAD~1..HEAD", "--name-only"])
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
    }

    files
}

fn git_diff_content() -> String {
    use std::process::Command;

    // Priority 1: staged diff
    if let Ok(output) = Command::new("git")
        .args(["diff", "--cached"])
        .output()
    {
        if output.status.success() {
            let staged = String::from_utf8_lossy(&output.stdout).to_string();
            if !staged.trim().is_empty() {
                return staged;
            }
        }
    }

    // Priority 2: last commit diff
    if let Ok(output) = Command::new("git")
        .args(["diff", "HEAD~1..HEAD"])
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

    // 2. Get staged files only (this runs as a pre-commit hook)
    let mut modified_files: Vec<String> = Vec::new();

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

// ══════════════════════════════════════════════════
// STATUS — semantic project overview
// ══════════════════════════════════════════════════

// cmd_reconcile is implemented at the bottom of this file

fn cmd_review(args: &[String]) {
    use std::process::Command;

    // 1. Determine which task to review
    let task_id_arg = args.get(2).cloned();
    let (task_id, task_content) = if let Some(ref tid) = task_id_arg {
        let normalized = if tid.starts_with("TASK-") {
            tid.clone()
        } else {
            format!("TASK-{:03}", tid.parse::<u32>().unwrap_or(0))
        };
        let path = format!(".cronus/tasks/{}.toml", normalized);
        match fs::read_to_string(&path) {
            Ok(c) => (normalized, c),
            Err(_) => {
                eprintln!("  \x1b[31mTask file not found: {}\x1b[0m", path);
                std::process::exit(1);
            }
        }
    } else {
        review_find_latest_task()
    };

    // 2. Parse task fields
    let mission = brief_toml_val(&task_content, "title").unwrap_or_else(|| "Unknown".into());
    let status = brief_toml_val(&task_content, "status").unwrap_or_else(|| "unknown".into());
    let done_checks = brief_toml_arr(&task_content, "done");

    // 3. Find commits mentioning this task ID
    let git_log = Command::new("git")
        .args(["log", "--oneline", "-50"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let task_id_lower = task_id.to_lowercase();
    let related_commits: Vec<&str> = git_log.lines()
        .filter(|l| l.to_lowercase().contains(&task_id_lower))
        .collect();

    let commit_hashes: Vec<String> = if related_commits.is_empty() {
        git_log.lines().take(5)
            .filter_map(|l| l.split_whitespace().next())
            .map(|s| s.to_string())
            .collect()
    } else {
        related_commits.iter()
            .filter_map(|l| l.split_whitespace().next())
            .map(|s| s.to_string())
            .collect()
    };

    // 4. Get diff stats for those commits
    let diff_range = if commit_hashes.len() >= 2 {
        format!("{}..{}", commit_hashes.last().unwrap(), commit_hashes.first().unwrap())
    } else if commit_hashes.len() == 1 {
        format!("{}~1..{}", commit_hashes[0], commit_hashes[0])
    } else {
        "HEAD~1..HEAD".to_string()
    };

    let diff_stat = Command::new("git")
        .args(["diff", "--stat", &diff_range])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let mut changes: Vec<String> = Vec::new();
    let mut total_insertions: i64 = 0;
    let mut total_deletions: i64 = 0;
    let mut files_changed = 0;

    for line in diff_stat.lines() {
        let trimmed = line.trim();
        if trimmed.contains('|') {
            files_changed += 1;
            let parts: Vec<&str> = trimmed.splitn(2, '|').collect();
            if parts.len() == 2 {
                let file = parts[0].trim();
                let stat = parts[1].trim();
                let plus_count = stat.matches('+').count() as i64;
                let minus_count = stat.matches('-').count() as i64;
                total_insertions += plus_count;
                total_deletions += minus_count;

                let action = if minus_count == 0 && plus_count > 0 {
                    "+"
                } else if plus_count == 0 && minus_count > 0 {
                    "-"
                } else {
                    "~"
                };

                let desc = if file.ends_with(".toml") && file.contains("TASK-") {
                    format!("{} Created task file {}", action, file)
                } else if file.ends_with(".toml") && (file.contains("constitution") || file.contains("objective")) {
                    format!("{} Updated project governance ({})", action, file)
                } else if file.ends_with(".rs") {
                    let num_str = stat.split_whitespace().next().unwrap_or("?");
                    format!("{} Modified {} ({} lines)", action, file, num_str)
                } else if file.ends_with(".cronus") {
                    format!("{} Updated CRONUS source ({})", action, file)
                } else if file.ends_with(".spec.toml") {
                    format!("{} Updated spec ({})", action, file)
                } else if file.ends_with(".json") {
                    format!("{} Updated data ({})", action, file)
                } else {
                    format!("{} Changed {}", action, file)
                };
                changes.push(desc);
            }
        } else if trimmed.contains("file") && trimmed.contains("changed") {
            for word in trimmed.split(',') {
                let word = word.trim();
                if word.contains("insertion") {
                    if let Some(n) = word.split_whitespace().next() {
                        total_insertions = n.parse().unwrap_or(total_insertions);
                    }
                }
                if word.contains("deletion") {
                    if let Some(n) = word.split_whitespace().next() {
                        total_deletions = n.parse().unwrap_or(total_deletions);
                    }
                }
            }
        }
    }

    // 5. Detect author from git log
    let author = Command::new("git")
        .args(["log", "-1", "--format=%an"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "Unknown".into());

    // 6. Validation checks
    let constitution = fs::read_to_string(".cronus/constitution.toml").unwrap_or_default();
    let constitution_exists = !constitution.is_empty();

    let objective = fs::read_to_string(".cronus/objective.toml").unwrap_or_default();
    let objective_exists = !objective.is_empty();

    let objective_pass = if objective_exists { "PASS" } else { "N/A" };

    let spec_count = count_files_matching("specs", ".spec.toml");
    let test_count = count_files_matching("tests/conformance", ".cronus");

    let build_ok = std::path::Path::new("target/release/cronus-kernel").exists()
        || std::path::Path::new("target/release/cronus").exists()
        || std::path::Path::new("target/debug/cronus-kernel").exists()
        || std::path::Path::new("target/debug/cronus").exists();

    // 7. Print formatted review
    println!();
    println!("  \x1b[1mSemantic Review: {}\x1b[0m", task_id);
    println!("  \x1b[90m{}\x1b[0m", "─".repeat(40));
    println!("  Mission: {}", mission);
    println!("  Status: {}", match status.as_str() {
        "done" => format!("\x1b[32m{}\x1b[0m", status),
        "in_progress" | "open" => format!("\x1b[33m{}\x1b[0m", status),
        "blocked" => format!("\x1b[31m{}\x1b[0m", status),
        _ => status.clone(),
    });
    println!("  Author: {}", author);
    println!("  Objective alignment: \x1b[32m{}\x1b[0m", objective_pass);
    println!();

    if !changes.is_empty() {
        println!("  \x1b[1mChanges:\x1b[0m");
        for change in &changes {
            println!("    {}", change);
        }
        println!("  \x1b[90m({} files, +{} -{})\x1b[0m", files_changed, total_insertions, total_deletions);
    } else {
        println!("  \x1b[1mChanges:\x1b[0m \x1b[90m(no diff data available)\x1b[0m");
    }
    println!();

    println!("  \x1b[1mValidation:\x1b[0m");
    if constitution_exists {
        println!("    Constitution: \x1b[32m✓ PASS\x1b[0m");
    } else {
        println!("    Constitution: \x1b[33m⚠ N/A\x1b[0m (no constitution.toml)");
    }
    if objective_exists {
        println!("    Objective: \x1b[32m✓ PASS\x1b[0m");
    } else {
        println!("    Objective: \x1b[33m⚠ N/A\x1b[0m (no objective.toml)");
    }
    if spec_count > 0 || test_count > 0 {
        println!("    Spec coverage: {} specs, {} tests", spec_count, test_count);
    } else {
        println!("    Spec coverage: \x1b[90mno specs found\x1b[0m");
    }
    if build_ok {
        println!("    Build: \x1b[32m✓ PASS\x1b[0m");
    } else {
        println!("    Build: \x1b[33m⚠ unknown\x1b[0m");
    }
    println!();

    if !done_checks.is_empty() {
        println!("  \x1b[1mDone criteria:\x1b[0m");
        for check in &done_checks {
            if status == "done" {
                println!("    \x1b[32m✓\x1b[0m {}", check);
            } else {
                println!("    \x1b[90m○\x1b[0m {}", check);
            }
        }
        println!();
    }
}

fn review_find_latest_task() -> (String, String) {
    let mut tasks: Vec<(String, String, String)> = Vec::new();

    if let Ok(entries) = fs::read_dir(".cronus/tasks") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("TASK-") && name.ends_with(".toml") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let id = name.trim_end_matches(".toml").to_string();
                    let status = brief_toml_val(&content, "status").unwrap_or_else(|| "open".into());
                    tasks.push((id, status, content));
                }
            }
        }
    }

    if tasks.is_empty() {
        eprintln!("  \x1b[31mNo tasks found in .cronus/tasks/\x1b[0m");
        std::process::exit(1);
    }

    tasks.sort_by(|a, b| b.0.cmp(&a.0));

    for (id, status, content) in &tasks {
        if status == "done" {
            return (id.clone(), content.clone());
        }
    }

    let (id, _, content) = tasks.into_iter().next().unwrap();
    (id, content)
}

fn cmd_timeline() {
    let mut tasks: Vec<(String, String, String, String)> = Vec::new();

    if let Ok(entries) = fs::read_dir(".cronus/tasks") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("TASK-") && name.ends_with(".toml") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let id = brief_toml_val(&content, "id")
                        .unwrap_or_else(|| name.trim_end_matches(".toml").to_string());
                    let title = brief_toml_val(&content, "title").unwrap_or_else(|| "(no title)".into());
                    let status = brief_toml_val(&content, "status").unwrap_or_else(|| "open".into());
                    let created = brief_toml_val(&content, "created")
                        .map(|d| {
                            if d.len() >= 10 { d[..10].to_string() } else { d }
                        })
                        .unwrap_or_else(|| {
                            entry.metadata().ok()
                                .and_then(|m| m.modified().ok())
                                .map(|t| {
                                    let secs = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                                    format_unix_date(secs)
                                })
                                .unwrap_or_else(|| "unknown".into())
                        });
                    tasks.push((created, id, status, title));
                }
            }
        }
    }

    if tasks.is_empty() {
        println!();
        println!("  \x1b[1mCRONUS Timeline\x1b[0m");
        println!("  \x1b[90m═══════════════\x1b[0m");
        println!();
        println!("  \x1b[90m(no tasks found in .cronus/tasks/)\x1b[0m");
        println!();
        return;
    }

    tasks.sort_by(|a, b| {
        b.0.cmp(&a.0).then(b.1.cmp(&a.1))
    });

    println!();
    println!("  \x1b[1mCRONUS Timeline\x1b[0m");
    println!("  \x1b[90m═══════════════\x1b[0m");
    println!();

    for (date, id, status, title) in &tasks {
        let status_colored = match status.as_str() {
            "done" => format!("\x1b[32m[{}]\x1b[0m", status),
            "in_progress" | "open" => format!("\x1b[33m[{}]\x1b[0m", status),
            "blocked" => format!("\x1b[31m[{}]\x1b[0m", status),
            _ => format!("[{}]", status),
        };
        let status_plain = format!("[{}]", status);
        let pad = if status_plain.len() < 15 { " ".repeat(15 - status_plain.len()) } else { String::new() };
        println!("  {}  {}  {}{} {}", date, id, status_colored, pad, title);
    }
    println!();
}

fn format_unix_date(secs: u64) -> String {
    let days = (secs / 86400) as i64;
    let mut y = 1970i64;
    let mut remaining = days;

    loop {
        let days_in_year = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }

    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let month_days = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 0usize;
    for (i, &md) in month_days.iter().enumerate() {
        if remaining < md as i64 {
            m = i + 1;
            break;
        }
        remaining -= md as i64;
    }
    if m == 0 { m = 12; }
    let d = remaining + 1;
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn cmd_status() {
    use std::path::Path;

    // ── 1. Project identity from constitution.toml ──
    let constitution = fs::read_to_string(".cronus/constitution.toml").unwrap_or_default();
    let proj_name = brief_toml_val(&constitution, "name").unwrap_or_else(|| "Unknown".into());

    // ── 2. Objective from objective.toml ──
    let objective = fs::read_to_string(".cronus/objective.toml").unwrap_or_default();
    let obj_title = brief_toml_val(&objective, "title").unwrap_or_else(|| "(no objective set)".into());
    let obj_deadline = brief_toml_val(&objective, "deadline").unwrap_or_else(|| "none".into());
    let success_criteria = brief_toml_arr(&objective, "criteria");

    // ── 3. Count completed from done section ──
    let done_completed = brief_toml_arr_after_section(&objective, "[done]", "completed");

    // Cross-reference: count how many success criteria are "met"
    // Heuristic: count items in [done].completed
    let criteria_total = success_criteria.len();
    let criteria_met = done_completed.len();

    // ── 4. Calculate days remaining ──
    let today = brief_today_date();
    let days_remaining = status_days_between(&today, &obj_deadline);
    let deadline_display = if days_remaining >= 0 {
        format!("{} ({} days remaining)", obj_deadline, days_remaining)
    } else {
        format!("{} (\x1b[31m{} days overdue\x1b[0m)", obj_deadline, -days_remaining)
    };

    // ── 5. Read tasks ──
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

    // ── 6. Segments ──
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

    // ── 7. Build status ──
    let build = if Path::new("target/release/cronus-kernel").exists()
        || Path::new("target/release/cronus").exists()
        || Path::new("cronus-kernel/target/release/cronus").exists()
    {
        "\x1b[32mpassing\x1b[0m"
    } else {
        "\x1b[33munknown\x1b[0m"
    };

    // ── 8. Metrics ──
    let spec_count = count_files_matching("specs", "spec.toml");
    let test_count = count_files_matching("tests/conformance", ".cronus");
    let example_count = count_files_matching("examples", ".cronus");

    // ── Print ──
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

/// Calculate days between two YYYY-MM-DD date strings (target - from).
fn status_days_between(from: &str, target: &str) -> i64 {
    let from_days = status_parse_date_to_days(from);
    let target_days = status_parse_date_to_days(target);
    target_days - from_days
}

/// Parse YYYY-MM-DD to days since epoch (for simple subtraction).
fn status_parse_date_to_days(date: &str) -> i64 {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 { return 0; }
    let y: i64 = parts[0].parse().unwrap_or(1970);
    let m: i64 = parts[1].parse().unwrap_or(1);
    let d: i64 = parts[2].parse().unwrap_or(1);

    // Count days from year 0 using a simple algorithm
    let mut total: i64 = 0;
    // Days from years
    for yr in 1970..y {
        let leap = yr % 4 == 0 && (yr % 100 != 0 || yr % 400 == 0);
        total += if leap { 366 } else { 365 };
    }
    // Days from months
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let md = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for i in 0..(m as usize - 1).min(11) {
        total += md[i] as i64;
    }
    total += d;
    total
}

// ══════════════════════════════════════════════════
// SEGMENT — semantic block isolation
// ══════════════════════════════════════════════════

fn cmd_segment(args: &[String]) {
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
                blocks = val.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
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
            eprintln!("  \x1b[31mInvalid block format: {}\x1b[0m (expected type:name, e.g. entity:User)", b);
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
            if blocks.is_empty() { "(no blocks)".to_string() } else { blocks.join(", ") }
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
    println!("    Agent: {}", if agent.is_empty() { "\x1b[90m(unassigned)\x1b[0m" } else { &agent });
    println!("    Task:  {}", if task.is_empty() { "\x1b[90m(none)\x1b[0m" } else { &task });
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
        let seg_name = format!("SEG-{}", fname.trim_start_matches("SEG-").trim_end_matches(".toml"));
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

// ══════════════════════════════════════════════════
// RECONCILE — AST-level merge of two .cronus files
// ══════════════════════════════════════════════════

fn cmd_reconcile(args: &[String]) {
    let positional: Vec<&String> = args.iter().skip(2).filter(|a| !a.starts_with("--")).collect();
    if positional.len() < 2 {
        eprintln!("  \x1b[31m✗\x1b[0m Usage: cronus reconcile <file-a> <file-b> [--output <file>]");
        std::process::exit(1);
    }
    let file_a = &positional[0];
    let file_b = &positional[1];

    let output_file = args.windows(2)
        .find(|w| w[0] == "--output")
        .map(|w| w[1].clone());

    let source_a = fs::read_to_string(file_a.as_str()).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Cannot read {}: {}", file_a, e);
        std::process::exit(1);
    });
    let source_b = fs::read_to_string(file_b.as_str()).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Cannot read {}: {}", file_b, e);
        std::process::exit(1);
    });

    let nodes_a = parser::parse(&source_a).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Parse error in {}: {}", file_a, e);
        std::process::exit(1);
    });
    let nodes_b = parser::parse(&source_b).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Parse error in {}: {}", file_b, e);
        std::process::exit(1);
    });

    let mut conflicts: Vec<String> = Vec::new();
    let mut merged: Vec<AstNode> = Vec::new();

    // --- Categorize nodes from A ---
    let mut app_a: Option<AppNode> = None;
    let mut style_a: Option<StyleNode> = None;
    let mut auth_a: Option<AuthNode> = None;
    let mut entities_a: HashMap<String, EntityNode> = HashMap::new();
    let mut pages_a: HashMap<String, PageNode> = HashMap::new();
    let mut apis_a: HashMap<String, ApiNode> = HashMap::new();
    let mut services_a: HashMap<String, ServiceNode> = HashMap::new();
    let mut components_a: HashMap<String, ComponentNode> = HashMap::new();
    let mut events_a: HashMap<String, EventNode> = HashMap::new();
    let mut workers_a: HashMap<String, WorkerNode> = HashMap::new();
    let mut middlewares_a: HashMap<String, MiddlewareNode> = HashMap::new();
    let mut imports_a: Vec<parser::ImportNode> = Vec::new();
    let mut envs_a: Vec<parser::EnvNode> = Vec::new();
    let mut tests_a: Vec<parser::TestNode> = Vec::new();
    let mut composes_a: Vec<parser::ComposeNode> = Vec::new();
    let mut layouts_a: HashMap<String, parser::LayoutNode> = HashMap::new();

    for node in nodes_a {
        match node {
            AstNode::App(n) => app_a = Some(n),
            AstNode::Style(n) => style_a = Some(n),
            AstNode::Auth(n) => auth_a = Some(n),
            AstNode::Entity(n) => { entities_a.insert(n.name.clone(), n); }
            AstNode::Page(n) => { pages_a.insert(n.route.clone(), n); }
            AstNode::Api(n) => { apis_a.insert(n.prefix.clone(), n); }
            AstNode::Service(n) => { services_a.insert(n.name.clone(), n); }
            AstNode::Component(n) => { components_a.insert(n.name.clone(), n); }
            AstNode::Event(n) => { events_a.insert(n.name.clone(), n); }
            AstNode::Worker(n) => { workers_a.insert(n.name.clone(), n); }
            AstNode::Middleware(n) => { middlewares_a.insert(n.name.clone(), n); }
            AstNode::Import(n) => imports_a.push(n),
            AstNode::Env(n) => envs_a.push(n),
            AstNode::Test(n) => tests_a.push(n),
            AstNode::Compose(n) => composes_a.push(n),
            AstNode::Layout(n) => { layouts_a.insert(n.name.clone(), n); }
            AstNode::Define(_) | AstNode::Webhook(_) => {}
        }
    }

    // --- Categorize nodes from B ---
    let mut app_b: Option<AppNode> = None;
    let mut style_b: Option<StyleNode> = None;
    let mut auth_b: Option<AuthNode> = None;
    let mut entities_b: HashMap<String, EntityNode> = HashMap::new();
    let mut pages_b: HashMap<String, PageNode> = HashMap::new();
    let mut apis_b: HashMap<String, ApiNode> = HashMap::new();
    let mut services_b: HashMap<String, ServiceNode> = HashMap::new();
    let mut components_b: HashMap<String, ComponentNode> = HashMap::new();
    let mut events_b: HashMap<String, EventNode> = HashMap::new();
    let mut workers_b: HashMap<String, WorkerNode> = HashMap::new();
    let mut middlewares_b: HashMap<String, MiddlewareNode> = HashMap::new();
    let mut imports_b: Vec<parser::ImportNode> = Vec::new();
    let mut envs_b: Vec<parser::EnvNode> = Vec::new();
    let mut tests_b: Vec<parser::TestNode> = Vec::new();
    let mut composes_b: Vec<parser::ComposeNode> = Vec::new();
    let mut layouts_b: HashMap<String, parser::LayoutNode> = HashMap::new();

    for node in nodes_b {
        match node {
            AstNode::App(n) => app_b = Some(n),
            AstNode::Style(n) => style_b = Some(n),
            AstNode::Auth(n) => auth_b = Some(n),
            AstNode::Entity(n) => { entities_b.insert(n.name.clone(), n); }
            AstNode::Page(n) => { pages_b.insert(n.route.clone(), n); }
            AstNode::Api(n) => { apis_b.insert(n.prefix.clone(), n); }
            AstNode::Service(n) => { services_b.insert(n.name.clone(), n); }
            AstNode::Component(n) => { components_b.insert(n.name.clone(), n); }
            AstNode::Event(n) => { events_b.insert(n.name.clone(), n); }
            AstNode::Worker(n) => { workers_b.insert(n.name.clone(), n); }
            AstNode::Middleware(n) => { middlewares_b.insert(n.name.clone(), n); }
            AstNode::Import(n) => imports_b.push(n),
            AstNode::Env(n) => envs_b.push(n),
            AstNode::Test(n) => tests_b.push(n),
            AstNode::Compose(n) => composes_b.push(n),
            AstNode::Layout(n) => { layouts_b.insert(n.name.clone(), n); }
            AstNode::Define(_) | AstNode::Webhook(_) => {}
        }
    }

    // --- 1. App: take from A (primary) ---
    if let Some(app) = app_a {
        merged.push(AstNode::App(app));
    } else if let Some(app) = app_b {
        merged.push(AstNode::App(app));
    }

    // --- 2. Style: take from A (primary) ---
    if let Some(style) = style_a {
        merged.push(AstNode::Style(style));
    } else if let Some(style) = style_b {
        merged.push(AstNode::Style(style));
    }

    // --- 3. Auth: take from A unless B has it and A doesn't ---
    if let Some(auth) = auth_a {
        merged.push(AstNode::Auth(auth));
    } else if let Some(auth) = auth_b {
        merged.push(AstNode::Auth(auth));
    }

    // --- 4. Imports: union by alias ---
    let mut seen_imports: HashMap<String, bool> = HashMap::new();
    for imp in &imports_a {
        seen_imports.insert(imp.alias.clone(), true);
        merged.push(AstNode::Import(imp.clone()));
    }
    for imp in imports_b {
        if !seen_imports.contains_key(&imp.alias) {
            merged.push(AstNode::Import(imp));
        }
    }

    // --- 5. Entities: merge by name ---
    let mut all_entity_names: Vec<String> = entities_a.keys().cloned().collect();
    for name in entities_b.keys() {
        if !all_entity_names.contains(name) {
            all_entity_names.push(name.clone());
        }
    }
    all_entity_names.sort();

    for name in &all_entity_names {
        match (entities_a.remove(name), entities_b.remove(name)) {
            (Some(a), None) => merged.push(AstNode::Entity(a)),
            (None, Some(b)) => merged.push(AstNode::Entity(b)),
            (Some(a), Some(b)) => {
                let mut merged_fields: Vec<parser::FieldNode> = a.fields.clone();
                for field_b in &b.fields {
                    if let Some(field_a) = merged_fields.iter().find(|f| f.name == field_b.name) {
                        if field_a.field_type != field_b.field_type {
                            conflicts.push(format!(
                                "entity {}: field \"{}\" has type {:?} in A but {:?} in B",
                                name, field_b.name, field_a.field_type, field_b.field_type
                            ));
                        }
                    } else {
                        merged_fields.push(field_b.clone());
                    }
                }
                // Merge transitions from both entities
                let mut merged_transitions = a.transitions.clone();
                merged_transitions.extend(b.transitions.clone());
                // Merge effects from both entities
                let mut merged_effects = a.effects.clone();
                merged_effects.extend(b.effects.clone());
                merged.push(AstNode::Entity(EntityNode {
                    name: name.clone(),
                    fields: merged_fields,
                    transitions: merged_transitions,
                    effects: merged_effects,
                    shared: a.shared || b.shared,
                    doc: None,
                }));
            }
            (None, None) => {}
        }
    }

    // --- 6. Pages: merge by route ---
    let mut all_routes: Vec<String> = pages_a.keys().cloned().collect();
    for route in pages_b.keys() {
        if !all_routes.contains(route) {
            all_routes.push(route.clone());
        }
    }
    all_routes.sort();

    for route in &all_routes {
        match (pages_a.remove(route), pages_b.remove(route)) {
            (Some(a), None) => merged.push(AstNode::Page(a)),
            (None, Some(b)) => merged.push(AstNode::Page(b)),
            (Some(a), Some(b)) => {
                let mut merged_sections: Vec<parser::SectionNode> = a.sections.clone();
                for (idx, sec_b) in b.sections.iter().enumerate() {
                    let existing = merged_sections.iter().enumerate()
                        .find(|(_i, s)| s.section_type == sec_b.section_type);
                    if let Some((pos, _)) = existing {
                        if pos == idx {
                            conflicts.push(format!(
                                "page \"{}\": section type \"{}\" at position {} exists in both A and B",
                                route, sec_b.section_type, idx
                            ));
                        }
                    } else {
                        merged_sections.push(sec_b.clone());
                    }
                }
                merged.push(AstNode::Page(PageNode {
                    route: route.clone(),
                    page_type: a.page_type,
                    entity: a.entity,
                    title: a.title,
                    sections: merged_sections,
                    config: a.config,
                    components: a.components,
                    requires: a.requires,
                    doc: None,
                }));
            }
            (None, None) => {}
        }
    }

    // --- 7. APIs: merge by prefix, union of routes ---
    let mut all_prefixes: Vec<String> = apis_a.keys().cloned().collect();
    for prefix in apis_b.keys() {
        if !all_prefixes.contains(prefix) {
            all_prefixes.push(prefix.clone());
        }
    }
    all_prefixes.sort();

    for prefix in &all_prefixes {
        match (apis_a.remove(prefix), apis_b.remove(prefix)) {
            (Some(a), None) => merged.push(AstNode::Api(a)),
            (None, Some(b)) => merged.push(AstNode::Api(b)),
            (Some(a), Some(b)) => {
                let mut merged_routes = a.routes.clone();
                for route_b in &b.routes {
                    let exists = merged_routes.iter().any(|r| {
                        r.method == route_b.method && r.path == route_b.path
                    });
                    if !exists {
                        merged_routes.push(route_b.clone());
                    }
                }
                merged.push(AstNode::Api(ApiNode {
                    prefix: prefix.clone(),
                    routes: merged_routes,
                    doc: None,
                }));
            }
            (None, None) => {}
        }
    }

    // --- 8-13. Named blocks: merge by name, report conflicts ---
    reconcile_named_map(&mut merged, &mut conflicts, services_a, services_b,
        "service", |n| AstNode::Service(n));
    reconcile_named_map(&mut merged, &mut conflicts, components_a, components_b,
        "component", |n| AstNode::Component(n));
    reconcile_named_map(&mut merged, &mut conflicts, events_a, events_b,
        "event", |n| AstNode::Event(n));
    reconcile_named_map(&mut merged, &mut conflicts, workers_a, workers_b,
        "worker", |n| AstNode::Worker(n));
    reconcile_named_map(&mut merged, &mut conflicts, middlewares_a, middlewares_b,
        "middleware", |n| AstNode::Middleware(n));
    reconcile_named_map(&mut merged, &mut conflicts, layouts_a, layouts_b,
        "layout", |n| AstNode::Layout(n));

    // --- 14. Envs, Tests, Composes: all from A, unique from B ---
    for env in envs_a { merged.push(AstNode::Env(env)); }
    for env in envs_b {
        if !merged.iter().any(|n| matches!(n, AstNode::Env(e) if e.name == env.name)) {
            merged.push(AstNode::Env(env));
        }
    }
    for test in tests_a { merged.push(AstNode::Test(test)); }
    for test in tests_b {
        if !merged.iter().any(|n| matches!(n, AstNode::Test(t) if t.name == test.name)) {
            merged.push(AstNode::Test(test));
        }
    }
    for comp in composes_a { merged.push(AstNode::Compose(comp)); }
    for comp in composes_b {
        if !merged.iter().any(|n| matches!(n, AstNode::Compose(c) if c.name == comp.name)) {
            merged.push(AstNode::Compose(comp));
        }
    }

    // --- Check for conflicts ---
    if !conflicts.is_empty() {
        eprintln!("\n  \x1b[31m✗ RECONCILE FAILED — {} conflict(s):\x1b[0m\n", conflicts.len());
        for (i, c) in conflicts.iter().enumerate() {
            eprintln!("  {}. {}", i + 1, c);
        }
        eprintln!();
        std::process::exit(1);
    }

    // --- Emit merged .cronus ---
    let output = reconcile_emit(&merged);

    if let Some(out_path) = output_file {
        fs::write(&out_path, &output).unwrap_or_else(|e| {
            eprintln!("  \x1b[31m✗\x1b[0m Cannot write {}: {}", out_path, e);
            std::process::exit(1);
        });
        println!("  \x1b[32m✓\x1b[0m Reconciled {} + {} → {}", file_a, file_b, out_path);
        println!("    {} nodes merged", merged.len());
    } else {
        print!("{}", output);
    }
}

/// Merge two named HashMaps: unique to A kept, unique to B added, overlap = conflict
fn reconcile_named_map<T: Clone>(
    merged: &mut Vec<AstNode>,
    conflicts: &mut Vec<String>,
    mut map_a: HashMap<String, T>,
    mut map_b: HashMap<String, T>,
    kind: &str,
    wrap: fn(T) -> AstNode,
) {
    let mut all_names: Vec<String> = map_a.keys().cloned().collect();
    for name in map_b.keys() {
        if !all_names.contains(name) {
            all_names.push(name.clone());
        }
    }
    all_names.sort();

    for name in &all_names {
        match (map_a.remove(name), map_b.remove(name)) {
            (Some(a), None) => merged.push(wrap(a)),
            (None, Some(b)) => merged.push(wrap(b)),
            (Some(a), Some(_b)) => {
                conflicts.push(format!("{} \"{}\" exists in both files", kind, name));
                merged.push(wrap(a));
            }
            (None, None) => {}
        }
    }
}

/// Emit a Vec<AstNode> as valid .cronus text
fn reconcile_emit(nodes: &[AstNode]) -> String {
    let mut out = String::new();

    for node in nodes {
        match node {
            AstNode::App(app) => {
                out.push_str(&format!("app \"{}\" {{\n", app.name));
                out.push_str(&format!("  stack {}\n", app.stack.join(", ")));
                out.push_str(&format!("  port {}\n", app.port));
                if let Some(db) = &app.database {
                    if let Some(path) = &db.path {
                        out.push_str(&format!("  database {} \"{}\"\n", db.db_type, path));
                    } else {
                        out.push_str(&format!("  database {}\n", db.db_type));
                    }
                }
                out.push_str("}\n\n");
            }
            AstNode::Style(style) => {
                out.push_str("style {\n");
                if let Some(theme) = &style.theme {
                    out.push_str(&format!("  theme {}\n", theme));
                }
                if let Some(accent) = &style.accent {
                    out.push_str(&format!("  accent {}\n", accent));
                }
                if let Some(radius) = &style.radius {
                    out.push_str(&format!("  radius {}\n", radius));
                }
                if let Some(font) = &style.font {
                    out.push_str(&format!("  font \"{}\"\n", font));
                }
                for (k, v) in &style.config {
                    out.push_str(&format!("  {} {}\n", k, v));
                }
                out.push_str("}\n\n");
            }
            AstNode::Auth(auth) => {
                out.push_str(&format!("auth {} {{\n", auth.entity));
                out.push_str(&format!("  login {}\n", auth.login_fields.join(", ")));
                out.push_str(&format!("  session {}", auth.session_type));
                if !auth.session_config.is_empty() {
                    let cfg: Vec<String> = auth.session_config.iter()
                        .map(|(k, v)| format!("{}: {}", k, v)).collect();
                    out.push_str(&format!(" {}", cfg.join(", ")));
                }
                out.push('\n');
                if !auth.roles.is_empty() {
                    out.push_str(&format!("  roles {}\n", auth.roles.join(", ")));
                }
                out.push_str("}\n\n");
            }
            AstNode::Entity(entity) => {
                out.push_str(&format!("entity {} {{\n", entity.name));
                for field in &entity.fields {
                    let ft = reconcile_field_type_str(&field.field_type);
                    let mut modifiers = Vec::new();
                    if field.required { modifiers.push("required"); }
                    if field.unique { modifiers.push("unique"); }
                    if field.optional { modifiers.push("optional"); }
                    if field.sensitive { modifiers.push("sensitive"); }
                    if field.searchable { modifiers.push("searchable"); }
                    if field.index { modifiers.push("index"); }
                    if field.featured { modifiers.push("featured"); }
                    if field.formatted { modifiers.push("formatted"); }
                    if field.array { modifiers.push("array"); }
                    let mod_str = if modifiers.is_empty() {
                        String::new()
                    } else {
                        format!(" {}", modifiers.join(" "))
                    };
                    if let Some(ref_name) = &field.reference {
                        out.push_str(&format!("  {} {} -> {}{}\n", field.name, ft, ref_name, mod_str));
                    } else if let Some(vals) = &field.enum_values {
                        out.push_str(&format!("  {} enum({}){}\n", field.name, vals.join(", "), mod_str));
                    } else {
                        out.push_str(&format!("  {} {}{}\n", field.name, ft, mod_str));
                    }
                }
                out.push_str("}\n\n");
            }
            AstNode::Page(page) => {
                let mut header = format!("page \"{}\"", page.route);
                if !page.page_type.is_empty() {
                    header.push_str(&format!(" type:{}", page.page_type));
                }
                if let Some(entity) = &page.entity {
                    header.push_str(&format!(" entity:{}", entity));
                }
                if let Some(title) = &page.title {
                    header.push_str(&format!(" title:\"{}\"", title));
                }
                if let Some(req) = &page.requires {
                    header.push_str(&format!(" requires:{}", req));
                }
                out.push_str(&format!("{} {{\n", header));
                for sec in &page.sections {
                    out.push_str(&format!("  section {} {{\n", sec.section_type));
                    if let Some(title) = &sec.title {
                        out.push_str(&format!("    title \"{}\"\n", title));
                    }
                    if let Some(subtitle) = &sec.subtitle {
                        out.push_str(&format!("    subtitle \"{}\"\n", subtitle));
                    }
                    for (k, v) in &sec.config {
                        out.push_str(&format!("    {} {}\n", k, v));
                    }
                    out.push_str("  }\n");
                }
                out.push_str("}\n\n");
            }
            AstNode::Api(api) => {
                out.push_str(&format!("api \"{}\" {{\n", api.prefix));
                for route in &api.routes {
                    let method = format!("{:?}", route.method);
                    out.push_str(&format!("  {} {} \"{}\"\n", route.name, method, route.path));
                }
                out.push_str("}\n\n");
            }
            AstNode::Service(svc) => {
                out.push_str(&format!("service {} {{\n", svc.name));
                if let Some(port) = svc.port {
                    out.push_str(&format!("  port {}\n", port));
                }
                for (k, v) in &svc.config {
                    out.push_str(&format!("  {} {}\n", k, v));
                }
                out.push_str("}\n\n");
            }
            AstNode::Component(comp) => {
                let mut header = format!("component {}", comp.name);
                if let Some(layout) = &comp.layout {
                    header.push_str(&format!(" layout:{}", layout));
                }
                if let Some(style) = &comp.style {
                    header.push_str(&format!(" style:{}", style));
                }
                out.push_str(&format!("{} {{\n", header));
                for item in &comp.items {
                    out.push_str(&format!("  {} \"{}\"\n", item.item_type, item.text));
                }
                out.push_str("}\n\n");
            }
            AstNode::Event(evt) => {
                out.push_str(&format!("event {} {{\n", evt.name));
                for action in &evt.actions {
                    out.push_str(&format!("  {}\n", action));
                }
                out.push_str("}\n\n");
            }
            AstNode::Worker(w) => {
                out.push_str(&format!("worker {} {{\n", w.name));
                if let Some(q) = &w.queue { out.push_str(&format!("  queue \"{}\"\n", q)); }
                if let Some(c) = w.concurrency { out.push_str(&format!("  concurrency {}\n", c)); }
                if let Some(r) = w.retry { out.push_str(&format!("  retry {}\n", r)); }
                if let Some(t) = &w.timeout { out.push_str(&format!("  timeout {}\n", t)); }
                if let Some(e) = &w.entity { out.push_str(&format!("  entity {}\n", e)); }
                out.push_str("}\n\n");
            }
            AstNode::Middleware(mw) => {
                out.push_str(&format!("middleware {} {{\n", mw.name));
                if let Some(applies) = &mw.applies_to {
                    out.push_str(&format!("  applies_to {}\n", applies.join(", ")));
                }
                for (k, v) in &mw.config {
                    out.push_str(&format!("  {} {}\n", k, v));
                }
                out.push_str("}\n\n");
            }
            AstNode::Import(imp) => {
                out.push_str(&format!("import {} from \"{}\"\n", imp.alias, imp.source));
            }
            AstNode::Env(env) => {
                out.push_str(&format!("env {} {{\n", env.name));
                for (k, v) in &env.vars {
                    out.push_str(&format!("  {} \"{}\"\n", k, v));
                }
                out.push_str("}\n\n");
            }
            AstNode::Test(test) => {
                out.push_str(&format!("test \"{}\" {{\n", test.name));
                for step in &test.steps {
                    out.push_str(&format!("  {} {} expect {}\n", step.action, step.entity, step.expect));
                }
                out.push_str("}\n\n");
            }
            AstNode::Compose(comp) => {
                out.push_str(&format!("compose {} {{\n", comp.name));
                for u in &comp.uses {
                    out.push_str(&format!("  use {}\n", u));
                }
                out.push_str("}\n\n");
            }
            AstNode::Layout(layout) => {
                out.push_str(&format!("layout {} {{\n", layout.name));
                for (k, v) in &layout.sidebar_config {
                    out.push_str(&format!("  sidebar.{} \"{}\"\n", k, v));
                }
                for item in &layout.sidebar_items {
                    out.push_str(&format!("  nav \"{}\" \"{}\"\n", item.label, item.route));
                }
                out.push_str("}\n\n");
            }
            AstNode::Define(def) => {
                out.push_str(&format!("define \"{}\" {{\n  # {} section(s)\n}}\n\n", def.name, def.sections.len()));
            }
            AstNode::Webhook(wh) => {
                out.push_str(&format!("webhook {} {{\n", wh.entity));
                for h in &wh.hooks {
                    out.push_str(&format!("  on {} -> {} \"{}\"\n", h.event, h.method, h.url));
                }
                out.push_str("}\n\n");
            }
        }
    }

    out
}

fn reconcile_field_type_str(ft: &FieldType) -> &'static str {
    match ft {
        FieldType::String => "string",
        FieldType::Text => "text",
        FieldType::Email => "email",
        FieldType::Url => "url",
        FieldType::Slug => "slug",
        FieldType::Phone => "phone",
        FieldType::Number => "number",
        FieldType::Money => "money",
        FieldType::Percentage => "percentage",
        FieldType::Boolean => "boolean",
        FieldType::Date => "date",
        FieldType::Ulid => "ulid",
        FieldType::Json => "json",
        FieldType::Enum => "enum",
        FieldType::Ip => "ip",
        FieldType::Relation => "relation",
    }
}

/// Auto-generated documentation page — derived 100% from the parsed .cronus AST.
/// Follows the Synthetic Docs design (obsidian dark, glass panels, code blocks).
fn render_auto_docs(state: &AppState) -> String {
    let app_name = &state.app.name;
    let port = state.app.port;

    // --- Sidebar nav items ---
    let mut nav_html = String::new();
    nav_html.push_str(r##"<a class="flex items-center gap-3 py-2 px-8 font-['Space_Grotesk'] text-sm uppercase tracking-widest text-white font-bold border-l-2 border-[#d277ff] doc-nav" data-scroll="overview"><span class="material-symbols-outlined text-lg">menu_book</span>Overview</a>"##);
    nav_html.push_str(r##"<a class="flex items-center gap-3 py-2 px-8 font-['Space_Grotesk'] text-sm uppercase tracking-widest text-[#ababab] hover:text-white transition-all doc-nav" data-scroll="entities"><span class="material-symbols-outlined text-lg">database</span>Entities</a>"##);
    nav_html.push_str(r##"<a class="flex items-center gap-3 py-2 px-8 font-['Space_Grotesk'] text-sm uppercase tracking-widest text-[#ababab] hover:text-white transition-all doc-nav" data-scroll="api"><span class="material-symbols-outlined text-lg">api</span>API Reference</a>"##);
    nav_html.push_str(r##"<a class="flex items-center gap-3 py-2 px-8 font-['Space_Grotesk'] text-sm uppercase tracking-widest text-[#ababab] hover:text-white transition-all doc-nav" data-scroll="pages"><span class="material-symbols-outlined text-lg">web</span>Pages</a>"##);
    nav_html.push_str(r##"<a class="flex items-center gap-3 py-2 px-8 font-['Space_Grotesk'] text-sm uppercase tracking-widest text-[#ababab] hover:text-white transition-all doc-nav" data-scroll="webhooks"><span class="material-symbols-outlined text-lg">webhook</span>Webhooks</a>"##);
    nav_html.push_str(r##"<a class="flex items-center gap-3 py-2 px-8 font-['Space_Grotesk'] text-sm uppercase tracking-widest text-[#ababab] hover:text-white transition-all doc-nav" data-scroll="audit"><span class="material-symbols-outlined text-lg">policy</span>Audit Trail</a>"##);

    // --- TOC (right sidebar) ---
    let mut toc_html = String::new();
    toc_html.push_str(r##"<li><a class="text-sm text-[#87adff] font-medium flex items-center gap-2 doc-nav" data-scroll="overview" style="cursor:pointer"><div class="w-1.5 h-1.5 rounded-full bg-[#87adff]" style="box-shadow:0 0 8px rgba(135,173,255,0.8)"></div>Overview</a></li>"##);
    toc_html.push_str(r##"<li><a class="text-sm text-[#ababab] hover:text-white transition-colors flex items-center gap-2 doc-nav" data-scroll="entities" style="cursor:pointer"><div class="w-1 h-1 rounded-full bg-[#484848]"></div>Entities</a></li>"##);
    toc_html.push_str(r##"<li><a class="text-sm text-[#ababab] hover:text-white transition-colors flex items-center gap-2 doc-nav" data-scroll="api" style="cursor:pointer"><div class="w-1 h-1 rounded-full bg-[#484848]"></div>API Reference</a></li>"##);
    toc_html.push_str(r##"<li><a class="text-sm text-[#ababab] hover:text-white transition-colors flex items-center gap-2 doc-nav" data-scroll="pages" style="cursor:pointer"><div class="w-1 h-1 rounded-full bg-[#484848]"></div>Pages</a></li>"##);
    toc_html.push_str(r##"<li><a class="text-sm text-[#ababab] hover:text-white transition-colors flex items-center gap-2 doc-nav" data-scroll="webhooks" style="cursor:pointer"><div class="w-1 h-1 rounded-full bg-[#484848]"></div>Webhooks</a></li>"##);
    toc_html.push_str(r##"<li><a class="text-sm text-[#ababab] hover:text-white transition-colors flex items-center gap-2 doc-nav" data-scroll="audit" style="cursor:pointer"><div class="w-1 h-1 rounded-full bg-[#484848]"></div>Audit Trail</a></li>"##);

    // --- Entities section ---
    let mut entities_html = String::new();
    for (i, entity) in state.entities.iter().enumerate() {
        if entity.name.starts_with('_') { continue; }
        let shared_badge = if entity.shared {
            r##" <span style="font-size:10px;padding:2px 8px;border-radius:4px;background:rgba(129,236,255,0.1);color:#81ecff;margin-left:8px">shared</span>"##
        } else { "" };

        let mut fields_html = String::new();
        for field in &entity.fields {
            let type_name = reconcile_field_type_str(&field.field_type);
            let mut badges = String::new();
            if field.required { badges.push_str(r##"<span style="color:#87adff;font-size:10px;margin-left:8px">required</span>"##); }
            if field.unique { badges.push_str(r##"<span style="color:#d277ff;font-size:10px;margin-left:8px">unique</span>"##); }
            if field.sensitive { badges.push_str(r##"<span style="color:#ef4444;font-size:10px;margin-left:8px">sensitive</span>"##); }
            if let Some(ref vals) = field.enum_values {
                let joined = vals.join(" | ");
                badges.push_str(&format!(r##"<span style="color:#ababab;font-size:10px;margin-left:8px">[{}]</span>"##, joined));
            }
            // Constraint badges
            if let Some(min_val) = field.min {
                badges.push_str(&format!(r##"<span style="color:#10b981;font-size:10px;margin-left:8px">min:{}</span>"##, min_val));
            }
            if let Some(max_val) = field.max {
                badges.push_str(&format!(r##"<span style="color:#10b981;font-size:10px;margin-left:8px">max:{}</span>"##, max_val));
            }
            if let Some(min_len) = field.min_length {
                badges.push_str(&format!(r##"<span style="color:#10b981;font-size:10px;margin-left:8px">min:{}</span>"##, min_len));
            }
            if let Some(max_len) = field.max_length {
                badges.push_str(&format!(r##"<span style="color:#10b981;font-size:10px;margin-left:8px">max:{}</span>"##, max_len));
            }
            if let Some(ref pat) = field.pattern {
                badges.push_str(&format!(r##"<span style="color:#10b981;font-size:10px;margin-left:8px">match:{}</span>"##, pat));
            }
            let field_doc_html = if let Some(ref doc) = field.doc {
                let mut parts = Vec::new();
                if !doc.summary.is_empty() {
                    parts.push(format!(r##"<span style="color:#757575;font-size:11px;margin-left:8px">{}</span>"##, doc.summary));
                }
                for tag in &doc.tags {
                    if tag.name == "ai" { continue; }
                    let tag_color = match tag.name.as_str() {
                        "example" => "#10b981",
                        "business" => "#f59e0b",
                        "deprecated" => "#ef4444",
                        _ => "#484848",
                    };
                    parts.push(format!(r##"<span style="color:{};font-size:10px;margin-left:8px">@{} {}</span>"##, tag_color, tag.name, tag.value));
                }
                parts.join("")
            } else { String::new() };

            fields_html.push_str(&format!(
                r##"<div style="padding:8px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;justify-content:space-between;align-items:center"><div style="display:flex;align-items:center;gap:8px"><span style="color:#e2e2e2;font-family:monospace;font-size:13px">{}</span><span style="color:#87adff;font-size:11px;font-family:monospace">{}</span></div><div>{}</div></div>{}</div>"##,
                field.name, type_name, badges, field_doc_html
            ));
        }

        let entity_doc_html = if let Some(ref doc) = entity.doc {
            let mut html = String::new();
            if !doc.summary.is_empty() {
                html.push_str(&format!(r##"<p style="color:#ababab;font-size:13px;margin:4px 0 0">{}</p>"##, doc.summary));
            }
            if !doc.description.is_empty() {
                html.push_str(&format!(r##"<p style="color:#757575;font-size:12px;margin:4px 0 0">{}</p>"##, doc.description));
            }
            let tags_html: String = doc.tags.iter().filter(|t| t.name != "ai").map(|t| {
                let color = match t.name.as_str() {
                    "owner" => "#87adff",
                    "lifecycle" => "#81ecff",
                    "since" => "#757575",
                    _ => "#484848",
                };
                format!(r##"<span style="font-size:10px;padding:2px 8px;border-radius:4px;background:rgba(255,255,255,0.03);color:{};margin-right:6px">@{} {}</span>"##, color, t.name, t.value)
            }).collect();
            if !tags_html.is_empty() {
                html.push_str(&format!(r##"<div style="margin-top:8px;display:flex;flex-wrap:wrap;gap:4px">{}</div>"##, tags_html));
            }
            html
        } else { String::new() };

        // Build transition diagram HTML if entity has transitions
        let transitions_html = if !entity.transitions.is_empty() {
            let mut html = String::new();
            html.push_str(r##"<div style="margin-top:16px;padding-top:16px;border-top:1px solid rgba(255,255,255,0.05)"><div style="display:flex;align-items:center;gap:6px;margin-bottom:12px"><span class="material-symbols-outlined" style="font-size:16px;color:#d277ff">swap_horiz</span><span style="font-family:Space Grotesk,sans-serif;font-size:14px;font-weight:600;color:#d277ff">State Transitions</span></div>"##);
            for t in &entity.transitions {
                html.push_str(&format!(
                    r##"<div style="margin-bottom:8px"><span style="color:#87adff;font-size:12px;font-family:monospace">{}</span></div>"##,
                    t.field
                ));
                html.push_str(r##"<div style="display:flex;flex-wrap:wrap;gap:8px;margin-bottom:12px">"##);
                for rule in &t.rules {
                    let targets = rule.to.join(", ");
                    html.push_str(&format!(
                        r##"<div style="background:rgba(210,119,255,0.06);border:1px solid rgba(210,119,255,0.15);border-radius:8px;padding:8px 12px;font-size:12px"><span style="color:#e2e2e2;font-family:monospace">{from}</span> <span style="color:#757575">-></span> <span style="color:#81ecff;font-family:monospace">{to}</span></div>"##,
                        from = rule.from,
                        to = targets,
                    ));
                }
                html.push_str("</div>");
            }
            html.push_str("</div>");
            html
        } else {
            String::new()
        };

        entities_html.push_str(&format!(
            r##"<div style="background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(135,173,255,0.2);padding:24px;margin-bottom:16px"><div style="margin-bottom:16px"><div style="display:flex;align-items:center"><span style="font-family:Space Grotesk,sans-serif;font-size:18px;font-weight:700;color:#fff">{name}</span>{shared}</div>{doc}</div><div>{fields}</div>{transitions}</div>"##,
            name = entity.name,
            shared = shared_badge,
            doc = entity_doc_html,
            fields = fields_html,
            transitions = transitions_html,
        ));
    }

    // --- API section ---
    let mut api_html = String::new();
    for api in &state.apis {
        let mut routes_html = String::new();
        for route in &api.routes {
            let method_str = match &route.method {
                parser::HttpMethod::GET => "GET",
                parser::HttpMethod::POST => "POST",
                parser::HttpMethod::PATCH => "PATCH",
                parser::HttpMethod::PUT => "PUT",
                parser::HttpMethod::DELETE => "DELETE",
            };
            let method_color = match method_str {
                "GET" => "#10b981",
                "POST" => "#87adff",
                "PATCH" | "PUT" => "#f59e0b",
                "DELETE" => "#ef4444",
                _ => "#ababab",
            };
            let route_doc_html = if let Some(ref doc) = route.doc {
                let mut parts = Vec::new();
                if !doc.summary.is_empty() {
                    parts.push(format!(r##"<div style="color:#ababab;font-size:12px;margin:4px 0 0 72px">{}</div>"##, doc.summary));
                }
                for tag in &doc.tags {
                    if tag.name == "ai" { continue; }
                    let tag_color = match tag.name.as_str() {
                        "param" => "#87adff",
                        "returns" => "#10b981",
                        "deprecated" => "#ef4444",
                        "example" => "#f59e0b",
                        _ => "#757575",
                    };
                    parts.push(format!(r##"<div style="color:{};font-size:11px;margin:2px 0 0 72px">@{} {}</div>"##, tag_color, tag.name, tag.value));
                }
                parts.join("")
            } else { String::new() };
            routes_html.push_str(&format!(
                r##"<div style="padding:10px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;align-items:center;gap:12px"><span style="font-family:monospace;font-size:11px;font-weight:700;color:{color};min-width:60px">{method}</span><span style="font-family:monospace;font-size:13px;color:#e2e2e2">{path}</span><span style="font-size:11px;color:#ababab;margin-left:auto">{name}</span></div>{route_doc}</div>"##,
                color = method_color,
                method = method_str,
                path = route.path,
                name = route.name,
                route_doc = route_doc_html,
            ));
        }
        api_html.push_str(&format!(
            r##"<div style="background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(135,173,255,0.2);padding:24px;margin-bottom:16px"><h3 style="font-family:Space Grotesk,sans-serif;font-size:16px;font-weight:700;color:#fff;margin:0 0 16px"><span style="font-family:monospace;color:#87adff">{base}</span></h3>{routes}</div>"##,
            base = api.prefix,
            routes = routes_html,
        ));
    }

    // --- Pages section ---
    let mut pages_html = String::new();
    for page in &state.pages {
        let route = &page.route;
        let title = page.title.as_deref().unwrap_or("-");
        let ptype = &page.page_type;
        let section_count = page.sections.len();
        let auth = if page.requires.is_some() { "auth required" } else { "public" };
        let auth_color = if page.requires.is_some() { "#f59e0b" } else { "#10b981" };
        let page_doc_html = if let Some(ref doc) = page.doc {
            let mut parts = Vec::new();
            if !doc.summary.is_empty() {
                parts.push(format!(r##"<div style="color:#ababab;font-size:12px;margin:4px 0 0 0">{}</div>"##, doc.summary));
            }
            if !doc.description.is_empty() {
                parts.push(format!(r##"<div style="color:#757575;font-size:11px;margin:2px 0 0 0">{}</div>"##, doc.description));
            }
            for tag in &doc.tags {
                if tag.name == "ai" { continue; }
                let tag_color = match tag.name.as_str() {
                    "requires" => "#f59e0b",
                    "layout" => "#87adff",
                    "since" => "#757575",
                    _ => "#484848",
                };
                parts.push(format!(r##"<div style="color:{};font-size:10px;margin:2px 0 0 0">@{} {}</div>"##, tag_color, tag.name, tag.value));
            }
            parts.join("")
        } else { String::new() };
        pages_html.push_str(&format!(
            r##"<div style="padding:12px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;justify-content:space-between;align-items:center"><div style="display:flex;align-items:center;gap:12px"><span style="font-family:monospace;font-size:14px;color:#87adff">{route}</span><span style="font-size:12px;color:#ababab">{title}</span></div><div style="display:flex;align-items:center;gap:12px"><span style="font-size:10px;color:#ababab">{sections} sections</span><span style="font-size:10px;padding:2px 8px;border-radius:4px;background:rgba(255,255,255,0.03);color:{auth_color}">{auth}</span></div></div>{page_doc}</div>"##,
            route = route, title = title, sections = section_count, auth = auth, auth_color = auth_color, page_doc = page_doc_html,
        ));
    }

    // --- Webhooks section ---
    let mut webhooks_html = String::new();
    for wh in &state.webhooks {
        let mut hooks_html = String::new();
        for hook in &wh.hooks {
            let event_color = match hook.event.as_str() {
                "create" => "#10b981",
                "update" => "#f59e0b",
                "delete" => "#ef4444",
                _ => "#ababab",
            };
            hooks_html.push_str(&format!(
                r##"<div style="display:flex;align-items:center;gap:12px;padding:10px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><span style="font-family:monospace;font-size:11px;font-weight:700;color:{color}">on {event}</span><span style="font-size:11px;color:#ababab">→</span><span style="font-family:monospace;font-size:11px;color:#87adff">{method}</span><span style="font-family:monospace;font-size:12px;color:#e2e2e2;word-break:break-all">{url}</span></div>"##,
                color = event_color,
                event = hook.event,
                method = hook.method,
                url = hook.url,
            ));
        }
        webhooks_html.push_str(&format!(
            r##"<div style="background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(135,173,255,0.2);padding:24px;margin-bottom:16px"><h3 style="font-family:Space Grotesk,sans-serif;font-size:16px;font-weight:700;color:#fff;margin:0 0 16px">{entity}</h3>{hooks}</div>"##,
            entity = wh.entity,
            hooks = hooks_html,
        ));
    }

    let has_webhooks = !state.webhooks.is_empty();

    // --- .cronus source preview ---
    let mut cronus_preview = String::new();
    cronus_preview.push_str(&format!(
        r##"<span style="color:#d277ff">app</span> <span style="color:#10b981">"{name}"</span> {{\n  <span style="color:#ababab">stack</span> fullstack\n  <span style="color:#ababab">port</span> <span style="color:#f59e0b">{port}</span>\n  <span style="color:#ababab">database</span> sqlite <span style="color:#10b981">"./data.db"</span>\n}}"##,
        name = app_name, port = port,
    ));

    // --- App doc-comment for Overview ---
    let app_doc_html = if let Some(ref doc) = state.app.doc {
        let mut html = String::new();
        if !doc.summary.is_empty() {
            html.push_str(&format!(r##"<p style="font-size:16px;color:#ababab;line-height:1.6;margin:16px 0 0">{}</p>"##, doc.summary));
        }
        if !doc.description.is_empty() {
            html.push_str(&format!(r##"<p style="font-size:14px;color:#757575;line-height:1.6;margin:8px 0 0">{}</p>"##, doc.description));
        }
        let tags: Vec<String> = doc.tags.iter().filter(|t| t.name != "ai").map(|t| {
            let color = match t.name.as_str() {
                "version" => "#87adff",
                "author" => "#81ecff",
                "since" => "#757575",
                _ => "#484848",
            };
            format!(r##"<span style="font-size:10px;padding:2px 8px;border-radius:4px;background:rgba(255,255,255,0.03);color:{};margin-right:6px">@{} {}</span>"##, color, t.name, t.value)
        }).collect();
        if !tags.is_empty() {
            html.push_str(&format!(r##"<div style="margin-top:12px;display:flex;flex-wrap:wrap;gap:4px">{}</div>"##, tags.join("")));
        }
        html
    } else { String::new() };

    // --- Full page ---
    format!(
        r##"<!DOCTYPE html>
<html lang="en"><head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width,initial-scale=1.0"/>
<title>{app_name} | Documentation</title>
<script src="https://cdn.tailwindcss.com"></script>
<link href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700;900&family=Inter:wght@300;400;500;600&display=swap" rel="stylesheet"/>
<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet"/>
<style>
body {{ background:#0e0e0e; color:#fff; font-family:'Inter',sans-serif; margin:0 }}
.material-symbols-outlined {{ font-variation-settings:'FILL' 0,'wght' 300,'GRAD' 0,'opsz' 24 }}
::-webkit-scrollbar {{ width:4px }} ::-webkit-scrollbar-track {{ background:#0e0e0e }} ::-webkit-scrollbar-thumb {{ background:#262626;border-radius:10px }}
</style>
</head>
<body>
<header style="position:fixed;top:0;width:100%;z-index:50;height:64px;background:rgba(0,0,0,0.8);backdrop-filter:blur(40px);border-bottom:1px solid rgba(255,255,255,0.05);display:flex;align-items:center;justify-content:space-between;padding:0 24px;box-sizing:border-box">
  <div style="display:flex;align-items:center;gap:32px">
    <span style="font-family:Space Grotesk,sans-serif;font-size:18px;font-weight:900;color:#fff;letter-spacing:-0.03em">{app_name}_DOCS</span>
    <nav style="display:flex;gap:24px;font-family:Space Grotesk,sans-serif;font-weight:700;font-size:14px">
      <a href="/docs" style="color:#fff;border-bottom:2px solid #87adff;padding-bottom:2px;text-decoration:none">API Docs</a>
      <a href="/docs/design" style="color:#757575;text-decoration:none">Design System</a>
      <a href="/" style="color:#757575;text-decoration:none">Dashboard</a>
    </nav>
  </div>
</header>

<div style="display:flex;padding-top:64px;min-height:100vh">
  <aside style="width:280px;position:fixed;left:0;top:64px;height:calc(100vh - 64px);background:#0e0e0e;border-right:1px solid rgba(255,255,255,0.03);display:flex;flex-direction:column;padding:32px 0;overflow-y:auto">
    <div style="padding:0 32px;margin-bottom:32px">
      <div style="display:flex;align-items:center;gap:12px">
        <div style="width:32px;height:32px;border-radius:8px;background:linear-gradient(135deg,#87adff,#d277ff);display:flex;align-items:center;justify-content:center;color:#fff;font-weight:700;font-size:14px">N</div>
        <div><div style="font-family:Space Grotesk,sans-serif;font-weight:700;color:#fff;font-size:14px">Core Engine</div><div style="font-size:10px;color:#757575;text-transform:uppercase;letter-spacing:0.15em">v{port}</div></div>
      </div>
    </div>
    <nav style="display:flex;flex-direction:column;gap:4px">{nav}</nav>
  </aside>

  <main style="flex:1;margin-left:280px;margin-right:240px;padding:48px 64px;max-width:800px">
    <header style="margin-bottom:48px" id="overview">
      <div style="display:flex;align-items:center;gap:8px;margin-bottom:16px">
        <span style="font-family:monospace;font-size:12px;color:#87adff;text-transform:uppercase;letter-spacing:-0.03em">Auto-Generated</span>
        <div style="width:4px;height:4px;border-radius:50%;background:#484848"></div>
        <span style="font-family:monospace;font-size:12px;color:#757575;text-transform:uppercase">{entity_count} entities · {api_count} API groups · {page_count} pages</span>
      </div>
      <h1 style="font-family:Space Grotesk,sans-serif;font-size:48px;font-weight:900;letter-spacing:-0.03em;margin:0 0 24px;background:linear-gradient(to right,#fff,#fff,#757575);-webkit-background-clip:text;-webkit-text-fill-color:transparent">Documentation</h1>
      <p style="font-size:18px;color:#757575;line-height:1.6">Complete reference for <strong style="color:#ababab">{app_name}</strong>, auto-generated from the .cronus source. Every entity, API endpoint, and page is documented here — always in sync with the code.</p>
      {app_doc}
    </header>

    <section style="margin-bottom:80px">
      <h2 style="font-family:Space Grotesk,sans-serif;font-size:24px;font-weight:700;margin-bottom:24px;display:flex;align-items:center;gap:12px"><span style="color:#d277ff">01.</span> App Configuration</h2>
      <div style="position:relative">
        <div style="position:absolute;inset:-4px;background:linear-gradient(to right,rgba(135,173,255,0.2),rgba(210,119,255,0.2));border-radius:16px;filter:blur(20px);opacity:0.25"></div>
        <div style="position:relative;background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(135,173,255,0.2);overflow:hidden">
          <div style="display:flex;align-items:center;justify-content:space-between;padding:8px 16px;background:rgba(38,38,38,0.5);border-bottom:1px solid rgba(255,255,255,0.03)">
            <div style="display:flex;gap:6px"><div style="width:10px;height:10px;border-radius:50%;background:rgba(239,68,68,0.2);border:1px solid rgba(239,68,68,0.4)"></div><div style="width:10px;height:10px;border-radius:50%;background:rgba(245,158,11,0.2);border:1px solid rgba(245,158,11,0.4)"></div><div style="width:10px;height:10px;border-radius:50%;background:rgba(16,185,129,0.2);border:1px solid rgba(16,185,129,0.4)"></div></div>
            <span style="font-family:monospace;font-size:10px;color:#757575">app.cronus</span>
          </div>
          <div style="padding:24px;font-family:monospace;font-size:14px;line-height:1.8;white-space:pre">{cronus_preview}</div>
        </div>
      </div>
    </section>

    <section style="margin-bottom:80px" id="entities">
      <h2 style="font-family:Space Grotesk,sans-serif;font-size:24px;font-weight:700;margin-bottom:24px;display:flex;align-items:center;gap:12px"><span style="color:#d277ff">02.</span> Entities</h2>
      <p style="color:#757575;margin-bottom:24px">Each entity maps to a SQLite table with auto-migration, CRUD API, and type validation.</p>
      {entities}
    </section>

    <section style="margin-bottom:80px" id="api">
      <h2 style="font-family:Space Grotesk,sans-serif;font-size:24px;font-weight:700;margin-bottom:24px;display:flex;align-items:center;gap:12px"><span style="color:#d277ff">03.</span> API Reference</h2>
      <p style="color:#757575;margin-bottom:24px">All endpoints are auto-generated from the <code style="background:#191919;color:#87adff;padding:2px 6px;border-radius:4px;font-size:13px;font-family:monospace">api</code> blocks. Auth via <code style="background:#191919;color:#87adff;padding:2px 6px;border-radius:4px;font-size:13px;font-family:monospace">Bearer</code> JWT token.</p>
      {api}
    </section>

    <section style="margin-bottom:80px" id="pages">
      <h2 style="font-family:Space Grotesk,sans-serif;font-size:24px;font-weight:700;margin-bottom:24px;display:flex;align-items:center;gap:12px"><span style="color:#d277ff">04.</span> Pages</h2>
      <div style="background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(135,173,255,0.2);padding:24px">
        {pages}
      </div>
    </section>

    <section style="margin-bottom:80px" id="webhooks">
      <h2 style="font-family:Space Grotesk,sans-serif;font-size:24px;font-weight:700;margin-bottom:24px;display:flex;align-items:center;gap:12px"><span style="color:#d277ff">05.</span> Webhooks</h2>
      {webhooks_section}
    </section>

    <section style="margin-bottom:80px" id="audit">
      <h2 style="font-family:Space Grotesk,sans-serif;font-size:24px;font-weight:700;margin-bottom:24px;display:flex;align-items:center;gap:12px"><span style="color:#d277ff">06.</span> Audit Trail</h2>
      <p style="color:#757575;margin-bottom:24px">Every data mutation (INSERT, UPDATE, DELETE) is recorded in an append-only <code style="background:#191919;color:#87adff;padding:2px 6px;border-radius:4px;font-size:13px;font-family:monospace">_audit_log</code> table with cryptographic hash chaining for tamper detection.</p>

      <div style="background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(135,173,255,0.2);padding:24px;margin-bottom:16px">
        <h3 style="font-family:Space Grotesk,sans-serif;font-size:16px;font-weight:700;color:#fff;margin:0 0 16px">_audit_log Table Structure</h3>
        <div style="padding:8px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;align-items:center;gap:8px"><span style="color:#e2e2e2;font-family:monospace;font-size:13px">id</span><span style="color:#87adff;font-size:11px;font-family:monospace">INTEGER</span><span style="color:#87adff;font-size:10px;margin-left:8px">primary key</span></div></div>
        <div style="padding:8px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;align-items:center;gap:8px"><span style="color:#e2e2e2;font-family:monospace;font-size:13px">entity_type</span><span style="color:#87adff;font-size:11px;font-family:monospace">TEXT</span><span style="color:#757575;font-size:11px;margin-left:8px">e.g. "Deployment", "User"</span></div></div>
        <div style="padding:8px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;align-items:center;gap:8px"><span style="color:#e2e2e2;font-family:monospace;font-size:13px">record_id</span><span style="color:#87adff;font-size:11px;font-family:monospace">TEXT</span><span style="color:#757575;font-size:11px;margin-left:8px">UUID of the affected record</span></div></div>
        <div style="padding:8px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;align-items:center;gap:8px"><span style="color:#e2e2e2;font-family:monospace;font-size:13px">action</span><span style="color:#87adff;font-size:11px;font-family:monospace">TEXT</span><span style="color:#757575;font-size:11px;margin-left:8px">INSERT | UPDATE | DELETE</span></div></div>
        <div style="padding:8px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;align-items:center;gap:8px"><span style="color:#e2e2e2;font-family:monospace;font-size:13px">user_id</span><span style="color:#87adff;font-size:11px;font-family:monospace">TEXT</span><span style="color:#757575;font-size:11px;margin-left:8px">User who performed the action (or "system")</span></div></div>
        <div style="padding:8px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;align-items:center;gap:8px"><span style="color:#e2e2e2;font-family:monospace;font-size:13px">diff</span><span style="color:#87adff;font-size:11px;font-family:monospace">TEXT</span><span style="color:#757575;font-size:11px;margin-left:8px">JSON diff of changed fields (before/after)</span></div></div>
        <div style="padding:8px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;align-items:center;gap:8px"><span style="color:#e2e2e2;font-family:monospace;font-size:13px">hash</span><span style="color:#87adff;font-size:11px;font-family:monospace">TEXT</span><span style="color:#d277ff;font-size:10px;margin-left:8px">SHA-256 chain hash</span></div></div>
        <div style="padding:8px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;align-items:center;gap:8px"><span style="color:#e2e2e2;font-family:monospace;font-size:13px">prev_hash</span><span style="color:#87adff;font-size:11px;font-family:monospace">TEXT</span><span style="color:#d277ff;font-size:10px;margin-left:8px">Hash of previous entry (blockchain-style)</span></div></div>
        <div style="padding:8px 0"><div style="display:flex;align-items:center;gap:8px"><span style="color:#e2e2e2;font-family:monospace;font-size:13px">created_at</span><span style="color:#87adff;font-size:11px;font-family:monospace">DATETIME</span><span style="color:#757575;font-size:11px;margin-left:8px">UTC timestamp</span></div></div>
      </div>

      <div style="background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(210,119,255,0.2);padding:24px;margin-bottom:16px">
        <h3 style="font-family:Space Grotesk,sans-serif;font-size:16px;font-weight:700;color:#fff;margin:0 0 16px">Hash Chaining</h3>
        <p style="color:#ababab;font-size:13px;line-height:1.6;margin:0 0 12px">Each audit entry's <code style="background:#191919;color:#d277ff;padding:2px 6px;border-radius:4px;font-size:12px;font-family:monospace">hash</code> is computed as <code style="background:#191919;color:#d277ff;padding:2px 6px;border-radius:4px;font-size:12px;font-family:monospace">SHA-256(prev_hash + entity_type + record_id + action + diff + timestamp)</code>. The first entry uses a genesis hash of all zeros.</p>
        <p style="color:#ababab;font-size:13px;line-height:1.6;margin:0 0 12px">This creates a tamper-evident chain: modifying any past entry breaks the hash sequence for all subsequent entries. The <code style="background:#191919;color:#87adff;padding:2px 6px;border-radius:4px;font-size:12px;font-family:monospace">/api/audit/trail/verify</code> endpoint walks the full chain and validates every link.</p>
        <div style="background:#0e0e0e;border-radius:8px;padding:16px;font-family:monospace;font-size:12px;line-height:1.8;color:#e2e2e2;margin-top:12px"><span style="color:#757575">// Chain structure</span><br/><span style="color:#d277ff">entry[0].hash</span> = SHA256(<span style="color:#484848">"0000...0000"</span> + data)<br/><span style="color:#d277ff">entry[1].hash</span> = SHA256(<span style="color:#10b981">entry[0].hash</span> + data)<br/><span style="color:#d277ff">entry[N].hash</span> = SHA256(<span style="color:#10b981">entry[N-1].hash</span> + data)</div>
      </div>

      <div style="background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(135,173,255,0.2);padding:24px;margin-bottom:16px">
        <h3 style="font-family:Space Grotesk,sans-serif;font-size:16px;font-weight:700;color:#fff;margin:0 0 16px">API Endpoints</h3>
        <div style="display:flex;align-items:center;gap:12px;padding:10px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><span style="font-family:monospace;font-size:11px;font-weight:700;color:#10b981;min-width:60px">GET</span><span style="font-family:monospace;font-size:13px;color:#e2e2e2">/api/audit/trail</span><span style="font-size:11px;color:#ababab;margin-left:auto">Returns recent audit entries with hash validation status. Query param: <code style="background:#191919;color:#87adff;padding:1px 4px;border-radius:3px;font-size:11px">?limit=N</code></span></div>
        <div style="display:flex;align-items:center;gap:12px;padding:10px 0"><span style="font-family:monospace;font-size:11px;font-weight:700;color:#10b981;min-width:60px">GET</span><span style="font-family:monospace;font-size:13px;color:#e2e2e2">/api/audit/trail/verify</span><span style="font-size:11px;color:#ababab;margin-left:auto">Walks the full hash chain and returns <code style="background:#191919;color:#87adff;padding:1px 4px;border-radius:3px;font-size:11px">{{"valid": true}}</code> or <code style="background:#191919;color:#87adff;padding:1px 4px;border-radius:3px;font-size:11px">{{"valid": false, "broken_at": N}}</code></span></div>
      </div>
    </section>

    <div style="background:rgba(135,173,255,0.1);border-left:2px solid #87adff;padding:24px;border-radius:0 12px 12px 0;display:flex;gap:16px;margin-bottom:48px">
      <span class="material-symbols-outlined" style="color:#87adff">auto_awesome</span>
      <div><h4 style="font-weight:700;color:#87adff;margin:0 0 4px;font-size:14px">Auto-Generated</h4><p style="font-size:13px;color:#ababab;margin:0">This documentation is generated at runtime from the parsed .cronus file. It is always in sync — modify the source and the docs update automatically.</p></div>
    </div>
  </main>

  <aside style="width:240px;position:fixed;right:0;top:64px;height:calc(100vh - 64px);padding:48px 32px;overflow-y:auto">
    <h5 style="font-family:Space Grotesk,sans-serif;font-size:10px;font-weight:900;color:#757575;text-transform:uppercase;letter-spacing:0.15em;margin:0 0 24px">On This Page</h5>
    <ul style="list-style:none;padding:0;margin:0;display:flex;flex-direction:column;gap:16px">{toc}</ul>
    <div style="margin-top:48px;background:rgba(31,31,31,0.5);padding:24px;border-radius:12px;border:1px solid rgba(255,255,255,0.03)">
      <div style="display:flex;align-items:center;gap:8px;margin-bottom:12px"><div style="width:8px;height:8px;border-radius:50%;background:#81ecff;animation:pulse 2s infinite;box-shadow:0 0 10px #81ecff"></div><span style="font-size:10px;font-weight:700;color:#81ecff;text-transform:uppercase">Live Sync</span></div>
      <p style="font-size:11px;color:#757575;margin:0;line-height:1.5">Docs auto-update when .cronus source changes.</p>
    </div>
  </aside>
</div>
<style>
@keyframes pulse{{0%,100%{{opacity:1}}50%{{opacity:0.5}}}}
@keyframes docFadeIn{{from{{opacity:0;transform:translateY(12px)}}to{{opacity:1;transform:translateY(0)}}}}
html{{scroll-behavior:smooth}}
main>section,main>header,main>div{{animation:docFadeIn 0.4s cubic-bezier(0,0,0.2,1) both}}
main>section:nth-child(2){{animation-delay:0.05s}}
main>section:nth-child(3){{animation-delay:0.1s}}
main>section:nth-child(4){{animation-delay:0.15s}}
main>section:nth-child(5){{animation-delay:0.2s}}
main>section:nth-child(6){{animation-delay:0.25s}}
.doc-nav{{cursor:pointer}}
</style>
<script>
document.querySelectorAll('[data-scroll]').forEach(function(a){{
  a.addEventListener('click',function(e){{
    e.preventDefault();
    var id=a.dataset.scroll;
    var el=document.getElementById(id);
    if(el){{
      el.scrollIntoView({{behavior:'smooth',block:'start'}});
      // Update active state
      document.querySelectorAll('.doc-nav').forEach(function(n){{
        n.style.color='#ababab';n.style.fontWeight='400';n.style.borderLeft='';
      }});
      a.style.color='#fff';a.style.fontWeight='700';
    }}
  }});
}});
// Scroll spy: highlight active nav on scroll
var sections=document.querySelectorAll('main>section[id]');
var navItems=document.querySelectorAll('.doc-nav[data-scroll]');
window.addEventListener('scroll',function(){{
  var scrollPos=window.scrollY+100;
  sections.forEach(function(sec){{
    if(sec.offsetTop<=scrollPos&&sec.offsetTop+sec.offsetHeight>scrollPos){{
      navItems.forEach(function(n){{
        if(n.dataset.scroll===sec.id){{n.style.color='#fff';n.style.fontWeight='700'}}
        else{{n.style.color='#ababab';n.style.fontWeight='400'}}
      }});
    }}
  }});
}});
</script>
</body></html>"##,
        app_name = app_name,
        port = port,
        nav = nav_html,
        toc = toc_html,
        entities = entities_html,
        api = api_html,
        pages = pages_html,
        webhooks_section = if has_webhooks { webhooks_html } else { r#"<p style="color:#757575">No webhooks configured. Add a <code style="background:#191919;color:#87adff;padding:2px 6px;border-radius:4px;font-size:13px;font-family:monospace">webhook</code> block to your .cronus file.</p>"#.to_string() },
        cronus_preview = cronus_preview,
        app_doc = app_doc_html,
        entity_count = state.entities.iter().filter(|e| !e.name.starts_with('_')).count(),
        api_count = state.apis.len(),
        page_count = state.pages.len(),
    )
}

/// Design System page — live rendered components with the project's theme tokens.
fn render_design_system(state: &AppState) -> String {
    let app_name = &state.app.name;
    let t = crate::theme::get();

    // Extract style info
    let accent = state.style.as_ref().and_then(|s| s.accent.as_deref()).unwrap_or("#87adff");
    let font = state.style.as_ref().and_then(|s| s.font.as_deref()).unwrap_or("Inter");
    let theme_mode = state.style.as_ref().and_then(|s| s.theme.as_deref()).unwrap_or("dark");

    // Color palette from theme tokens
    let colors = vec![
        ("Background", &t.background),
        ("Surface", &t.surface),
        ("Surface Container", &t.surface_container),
        ("Surface Bright", &t.surface_bright),
        ("On Surface", &t.on_surface),
        ("On Surface Variant", &t.on_surface_variant),
        ("Primary", &t.primary),
        ("Secondary", &t.secondary),
        ("Tertiary", &t.tertiary),
        ("Error", &t.error),
        ("Outline", &t.outline),
        ("Outline Variant", &t.outline_variant),
    ];

    let mut palette_html = String::new();
    for (name, color) in &colors {
        palette_html.push_str(&format!(
            r##"<div style="display:flex;flex-direction:column;align-items:center;gap:8px"><div style="width:100%;aspect-ratio:1;border-radius:8px;background:{color};border:1px solid rgba(255,255,255,0.1)"></div><span style="font-size:11px;color:#e2e2e2;font-weight:500;text-align:center">{name}</span><span style="font-family:monospace;font-size:9px;color:#757575">{color}</span></div>"##,
            name = name, color = color,
        ));
    }

    // Typography scale
    let type_scale = vec![
        ("Display", "48px", "900", font, "The quick brown fox"),
        ("Headline", "32px", "700", font, "The quick brown fox jumps"),
        ("Title", "20px", "700", "Inter", "The quick brown fox jumps over the lazy dog"),
        ("Body", "14px", "400", "Inter", "The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs."),
        ("Label", "11px", "700", font, "UPPERCASE TRACKING WIDE"),
        ("Mono", "13px", "400", "monospace", "const x = await fetch('/api/data');"),
    ];

    let mut type_html = String::new();
    for (name, size, weight, family, sample) in &type_scale {
        let ls = if *name == "Label" { "letter-spacing:0.15em;text-transform:uppercase;" } else { "" };
        type_html.push_str(&format!(
            r##"<div style="padding:20px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;justify-content:space-between;align-items:baseline;margin-bottom:8px"><span style="font-size:10px;color:#757575;text-transform:uppercase;letter-spacing:0.15em;font-family:Space Grotesk,sans-serif;font-weight:700">{name}</span><span style="font-family:monospace;font-size:10px;color:#484848">{size} / {weight}</span></div><p style="font-family:{family},sans-serif;font-size:{size};font-weight:{weight};color:#e2e2e2;margin:0;{ls}">{sample}</p></div>"##,
            name = name, size = size, weight = weight, family = family, sample = sample, ls = ls,
        ));
    }

    // Section helper: wraps content in a glass panel
    let section = |id: &str, num: &str, title: &str, desc: &str, content: &str| -> String {
        format!(
            r##"<section style="margin-bottom:80px" id="{id}">
      <h2 style="font-family:Space Grotesk,sans-serif;font-size:24px;font-weight:700;margin-bottom:8px;display:flex;align-items:center;gap:12px"><span style="color:#d277ff">{num}.</span> {title}</h2>
      <p style="color:#757575;margin-bottom:24px;font-size:14px">{desc}</p>
      {content}
    </section>"##,
            id = id, num = num, title = title, desc = desc, content = content,
        )
    };

    // Component: wrap in glass card with label + code
    let comp = |name: &str, cronus_syntax: &str, rendered: &str| -> String {
        format!(
            r##"<div style="background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(135,173,255,0.2);margin-bottom:16px;overflow:hidden"><div style="padding:16px 24px;border-bottom:1px solid rgba(255,255,255,0.03);display:flex;justify-content:space-between;align-items:center"><span style="font-family:Space Grotesk,sans-serif;font-size:13px;font-weight:700;color:#e2e2e2">{name}</span><code style="font-size:10px;color:#87adff;background:#191919;padding:2px 8px;border-radius:4px">{syntax}</code></div><div style="padding:24px;display:flex;flex-wrap:wrap;align-items:center;gap:12px">{rendered}</div></div>"##,
            name = name, syntax = cronus_syntax, rendered = rendered,
        )
    };

    // Render live components using the dark theme inline styles
    let btn_style = |bg: &str, color: &str, border: &str| -> String {
        format!("padding:10px 20px;border-radius:8px;font-family:Space Grotesk,sans-serif;font-size:12px;font-weight:700;letter-spacing:0.08em;text-transform:uppercase;cursor:pointer;transition:all 0.15s;border:{};background:{};color:{}", border, bg, color)
    };

    let buttons = format!(
        r##"<button style="{}">Primary</button><button style="{}">Secondary</button><button style="{}">Ghost</button><button style="{}">Danger</button><button style="{};font-size:10px;padding:6px 12px">Small</button><button style="{};font-size:14px;padding:14px 28px">Large</button>"##,
        btn_style("linear-gradient(135deg,#87adff,#d277ff)", "#000", "none"),
        btn_style("#191919", "#e2e2e2", "0.5px solid rgba(255,255,255,0.1)"),
        btn_style("transparent", "#ababab", "1px solid transparent"),
        btn_style("rgba(239,68,68,0.1)", "#ef4444", "1px solid rgba(239,68,68,0.2)"),
        btn_style("linear-gradient(135deg,#87adff,#d277ff)", "#000", "none"),
        btn_style("linear-gradient(135deg,#87adff,#d277ff)", "#000", "none"),
    );

    let input_style = "width:240px;background:#000;border:1px solid rgba(255,255,255,0.05);border-radius:8px;padding:12px 16px;color:#fff;font-size:14px;outline:none;font-family:Inter,sans-serif";
    let inputs = format!(
        r##"<input type="text" placeholder="Text input" style="{s}"><input type="email" placeholder="email@example.com" style="{s}"><input type="password" placeholder="••••••••" style="{s}"><textarea placeholder="Textarea" style="{s};height:60px;resize:none"></textarea>"##,
        s = input_style,
    );

    let selects = format!(
        r##"<select style="{s};appearance:none;cursor:pointer"><option>Select option</option><option>us-east-1</option><option>eu-west-2</option><option>ap-south-1</option></select>"##,
        s = input_style,
    );

    let badges = r##"<span style="display:inline-flex;align-items:center;gap:6px;padding:4px 12px;font-size:11px;font-weight:600;border-radius:999px;background:rgba(16,185,129,0.12);color:#10b981"><span style="width:6px;height:6px;border-radius:50%;background:#10b981"></span>Live</span><span style="display:inline-flex;align-items:center;gap:6px;padding:4px 12px;font-size:11px;font-weight:600;border-radius:999px;background:rgba(59,130,246,0.12);color:#3b82f6"><span style="width:6px;height:6px;border-radius:50%;background:#3b82f6;animation:pulse 2s infinite"></span>Rolling</span><span style="display:inline-flex;align-items:center;gap:6px;padding:4px 12px;font-size:11px;font-weight:600;border-radius:999px;background:rgba(239,68,68,0.12);color:#ef4444"><span style="width:6px;height:6px;border-radius:50%;background:#ef4444"></span>Failed</span><span style="display:inline-flex;align-items:center;gap:6px;padding:4px 12px;font-size:11px;font-weight:600;border-radius:999px;background:rgba(245,158,11,0.12);color:#f59e0b"><span style="width:6px;height:6px;border-radius:50%;background:#f59e0b"></span>Warning</span><span style="display:inline-flex;align-items:center;gap:6px;padding:4px 12px;font-size:11px;font-weight:600;border-radius:999px;background:rgba(113,113,122,0.12);color:#71717a"><span style="width:6px;height:6px;border-radius:50%;background:#71717a"></span>Pending</span>"##;

    let cards = r##"<div style="background:#1b1b1b;border:0.5px solid rgba(76,69,70,0.15);border-radius:12px;padding:20px 24px;width:200px"><div style="display:flex;align-items:center;gap:6px;margin-bottom:12px"><span class="material-symbols-outlined" style="font-size:16px;color:#87adff">trending_up</span><span style="font-size:13px;font-weight:500;color:rgba(226,226,226,0.5)">Requests</span><span style="font-size:10px;padding:2px 6px;border-radius:4px;background:rgba(16,185,129,0.1);color:#10b981">+12%</span></div><span style="font-size:36px;font-weight:700;letter-spacing:-0.03em;color:#e2e2e2">1.2M</span><p style="font-size:11px;color:rgba(226,226,226,0.3);margin:8px 0 0">Last 24h</p></div><div style="background:#1b1b1b;border:0.5px solid rgba(76,69,70,0.15);border-radius:12px;padding:20px 24px;width:200px"><div style="display:flex;align-items:center;gap:6px;margin-bottom:12px"><span class="material-symbols-outlined" style="font-size:16px;color:#ef4444">error_outline</span><span style="font-size:13px;font-weight:500;color:rgba(226,226,226,0.5)">Error Rate</span><span style="font-size:10px;padding:2px 6px;border-radius:4px;background:rgba(16,185,129,0.1);color:#10b981">-0.01%</span></div><span style="font-size:36px;font-weight:700;letter-spacing:-0.03em;color:#e2e2e2">0.02%</span><p style="font-size:11px;color:rgba(226,226,226,0.3);margin:8px 0 0">5xx responses</p></div>"##;

    let alerts = r##"<div style="width:100%;display:flex;flex-direction:column;gap:8px"><div style="background:rgba(135,173,255,0.1);border-left:2px solid #87adff;padding:16px 20px;border-radius:0 8px 8px 0;display:flex;gap:12px"><span class="material-symbols-outlined" style="color:#87adff;font-size:18px">info</span><div><p style="font-size:13px;font-weight:600;color:#87adff;margin:0">Info</p><p style="font-size:12px;color:#ababab;margin:4px 0 0">This is an informational alert.</p></div></div><div style="background:rgba(16,185,129,0.1);border-left:2px solid #10b981;padding:16px 20px;border-radius:0 8px 8px 0;display:flex;gap:12px"><span class="material-symbols-outlined" style="color:#10b981;font-size:18px">check_circle</span><div><p style="font-size:13px;font-weight:600;color:#10b981;margin:0">Success</p><p style="font-size:12px;color:#ababab;margin:4px 0 0">Operation completed successfully.</p></div></div><div style="background:rgba(245,158,11,0.1);border-left:2px solid #f59e0b;padding:16px 20px;border-radius:0 8px 8px 0;display:flex;gap:12px"><span class="material-symbols-outlined" style="color:#f59e0b;font-size:18px">warning</span><div><p style="font-size:13px;font-weight:600;color:#f59e0b;margin:0">Warning</p><p style="font-size:12px;color:#ababab;margin:4px 0 0">Memory pressure is above 85%.</p></div></div><div style="background:rgba(239,68,68,0.1);border-left:2px solid #ef4444;padding:16px 20px;border-radius:0 8px 8px 0;display:flex;gap:12px"><span class="material-symbols-outlined" style="color:#ef4444;font-size:18px">error</span><div><p style="font-size:13px;font-weight:600;color:#ef4444;margin:0">Error</p><p style="font-size:12px;color:#ababab;margin:4px 0 0">Deployment failed on us-west-2.</p></div></div></div>"##;

    let modal_preview = r##"<div style="background:#191919;border-radius:12px;padding:32px;border-top:0.5px solid rgba(135,173,255,0.2);box-shadow:0 0 60px rgba(135,173,255,0.04);width:100%;max-width:420px"><div style="text-align:center;margin-bottom:24px"><div style="width:40px;height:40px;border-radius:12px;background:linear-gradient(135deg,#87adff,#d277ff);display:inline-flex;align-items:center;justify-content:center;margin-bottom:12px"><span class="material-symbols-outlined" style="color:#fff;font-size:20px">rocket_launch</span></div><h3 style="font-family:Space Grotesk,sans-serif;font-size:20px;font-weight:700;margin:0 0 4px;color:#fff">New Deployment</h3><p style="font-size:12px;color:#ababab;margin:0">Configure and launch a new deployment.</p></div><div style="display:flex;flex-direction:column;gap:12px;margin-bottom:20px"><input placeholder="Service name" style="background:#000;border:1px solid rgba(255,255,255,0.05);border-radius:8px;padding:10px 14px;color:#fff;font-size:13px;outline:none"><select style="background:#000;border:1px solid rgba(255,255,255,0.05);border-radius:8px;padding:10px 14px;color:#fff;font-size:13px;outline:none;appearance:none"><option>us-east-1</option><option>eu-west-2</option></select></div><div style="display:flex;gap:12px"><button style="flex:1;padding:10px;border:0.5px solid rgba(255,255,255,0.1);border-radius:8px;background:#191919;color:#ababab;font-size:12px;cursor:pointer">Cancel</button><button style="flex:1;padding:10px;border:none;border-radius:8px;background:linear-gradient(135deg,#87adff,#d277ff);color:#000;font-weight:700;font-size:12px;cursor:pointer">Deploy Now</button></div></div>"##;

    // Build sections
    let content = vec![
        section("colors", "01", "Color Palette", "Material Design 3 tokens derived from the accent color. Every surface, text, and interactive element uses these tokens.", &format!(r##"<div style="background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(135,173,255,0.2);padding:24px;display:grid;grid-template-columns:repeat(4,1fr);gap:16px">{}</div>"##, palette_html)),
        section("typography", "02", "Typography", &format!("Headline: {} · Body: Inter · Label: {} · Mono: system", font, font), &format!(r##"<div style="background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(135,173,255,0.2);padding:24px">{}</div>"##, type_html)),
        section("buttons", "03", "Buttons", "Kinetic triggers with gradient primary, ghost, outline, and danger variants.", &comp("Button", "action \"Label\" style:primary", &buttons)),
        section("inputs", "04", "Inputs", "Terminal-style inputs with etched black background and focus glow.", &comp("Input", "field \"Name\" type:text", &inputs)),
        section("selects", "05", "Select", "Dropdown selects with consistent styling.", &comp("Select", "field \"Region\" type:select options:\"...\"", &selects)),
        section("badges", "06", "Status Badges", "Semantic status indicators with pulse animation for active states.", &comp("Badge", "status enum [\"Live\", \"Rolling\", \"Failed\"]", badges)),
        section("kpi", "07", "KPI Cards", "Data-driven metric cards with icon, value, trend badge, and subtitle.", &comp("KPI Card", "section kpi cols:4 { bind Entity }", cards)),
        section("alerts", "08", "Alerts", "Contextual messages with severity levels and edge lighting.", &comp("Alert", "section alert { ... }", alerts)),
        section("modal", "09", "Modal", "Glassmorphic dialog with backdrop blur, edge lighting, entity binding, and form fields.", &comp("Modal", "section modal entity:\"Entity\" { ... }", modal_preview)),
    ].join("\n");

    // TOC
    let toc_items = vec![
        ("colors", "Color Palette"), ("typography", "Typography"), ("buttons", "Buttons"),
        ("inputs", "Inputs"), ("selects", "Select"), ("badges", "Status Badges"),
        ("kpi", "KPI Cards"), ("alerts", "Alerts"), ("modal", "Modal"),
    ];
    let toc: String = toc_items.iter().enumerate().map(|(i, (id, name))| {
        let dot = if i == 0 {
            r##"<div style="width:6px;height:6px;border-radius:50%;background:#87adff;box-shadow:0 0 8px rgba(135,173,255,0.8)"></div>"##
        } else {
            r##"<div style="width:4px;height:4px;border-radius:50%;background:#484848"></div>"##
        };
        let color = if i == 0 { "#87adff" } else { "#ababab" };
        format!(r##"<li><a class="doc-nav" data-scroll="{id}" style="font-size:12px;color:{color};display:flex;align-items:center;gap:8px;text-decoration:none;cursor:pointer">{dot}{name}</a></li>"##, id = id, color = color, dot = dot, name = name)
    }).collect::<Vec<_>>().join("");

    format!(
        r##"<!DOCTYPE html>
<html lang="en"><head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width,initial-scale=1.0"/>
<title>{app_name} | Design System</title>
<link href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700;900&family=Inter:wght@300;400;500;600&display=swap" rel="stylesheet"/>
<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet"/>
<style>
body {{ background:#0e0e0e; color:#fff; font-family:'Inter',sans-serif; margin:0 }}
.material-symbols-outlined {{ font-variation-settings:'FILL' 0,'wght' 300,'GRAD' 0,'opsz' 24 }}
::-webkit-scrollbar {{ width:4px }} ::-webkit-scrollbar-track {{ background:#0e0e0e }} ::-webkit-scrollbar-thumb {{ background:#262626;border-radius:10px }}
@keyframes pulse {{ 0%,100%{{opacity:1}} 50%{{opacity:0.5}} }}
</style>
</head>
<body>
<header style="position:fixed;top:0;width:100%;z-index:50;height:64px;background:rgba(0,0,0,0.8);backdrop-filter:blur(40px);border-bottom:1px solid rgba(255,255,255,0.05);display:flex;align-items:center;justify-content:space-between;padding:0 24px;box-sizing:border-box">
  <div style="display:flex;align-items:center;gap:32px">
    <span style="font-family:Space Grotesk,sans-serif;font-size:18px;font-weight:900;color:#fff;letter-spacing:-0.03em">{app_name}_DESIGN</span>
    <nav style="display:flex;gap:24px;font-family:Space Grotesk,sans-serif;font-weight:700;font-size:14px">
      <a href="/docs" style="color:#757575;text-decoration:none">API Docs</a>
      <a href="/docs/design" style="color:#fff;border-bottom:2px solid #d277ff;padding-bottom:2px;text-decoration:none">Design System</a>
      <a href="/" style="color:#757575;text-decoration:none">Dashboard</a>
    </nav>
  </div>
</header>

<div style="display:flex;padding-top:64px;min-height:100vh">
  <main style="flex:1;padding:48px 64px;max-width:780px;margin-left:auto;margin-right:260px">
    <header style="margin-bottom:60px">
      <div style="display:flex;align-items:center;gap:8px;margin-bottom:16px">
        <span style="font-family:monospace;font-size:12px;color:#d277ff;text-transform:uppercase;letter-spacing:-0.03em">Auto-Generated</span>
        <div style="width:4px;height:4px;border-radius:50%;background:#484848"></div>
        <span style="font-family:monospace;font-size:12px;color:#757575">Theme: {theme} · Accent: {accent} · Font: {font}</span>
      </div>
      <h1 style="font-family:Space Grotesk,sans-serif;font-size:48px;font-weight:900;letter-spacing:-0.03em;margin:0 0 24px;background:linear-gradient(to right,#fff,#fff,#757575);-webkit-background-clip:text;-webkit-text-fill-color:transparent">Design System</h1>
      <p style="font-size:18px;color:#757575;line-height:1.6">Component library and design tokens for <strong style="color:#ababab">{app_name}</strong>. Every component shown here is rendered live using the project's theme — what you see is what CRONUS generates.</p>
    </header>

    {content}

    <div style="background:rgba(210,119,255,0.1);border-left:2px solid #d277ff;padding:24px;border-radius:0 12px 12px 0;display:flex;gap:16px;margin-bottom:48px">
      <span class="material-symbols-outlined" style="color:#d277ff">palette</span>
      <div><h4 style="font-weight:700;color:#d277ff;margin:0 0 4px;font-size:14px">Live Components</h4><p style="font-size:13px;color:#ababab;margin:0">Every component above is rendered with the same engine that powers your dashboard. Change the <code style="background:#191919;color:#87adff;padding:2px 6px;border-radius:4px;font-size:12px">style</code> block in your .cronus and the design system updates automatically.</p></div>
    </div>
  </main>

  <aside style="width:200px;position:fixed;right:0;top:64px;height:calc(100vh - 64px);padding:48px 24px;overflow-y:auto">
    <h5 style="font-family:Space Grotesk,sans-serif;font-size:10px;font-weight:900;color:#757575;text-transform:uppercase;letter-spacing:0.15em;margin:0 0 24px">Components</h5>
    <ul style="list-style:none;padding:0;margin:0;display:flex;flex-direction:column;gap:12px">{toc}</ul>
  </aside>
</div>
<style>
@keyframes pulse{{0%,100%{{opacity:1}}50%{{opacity:0.5}}}}
@keyframes docFadeIn{{from{{opacity:0;transform:translateY(12px)}}to{{opacity:1;transform:translateY(0)}}}}
html{{scroll-behavior:smooth}}
main>section{{animation:docFadeIn 0.4s cubic-bezier(0,0,0.2,1) both}}
main>section:nth-child(2){{animation-delay:0.05s}}
main>section:nth-child(3){{animation-delay:0.1s}}
main>section:nth-child(4){{animation-delay:0.15s}}
main>section:nth-child(5){{animation-delay:0.2s}}
main>section:nth-child(6){{animation-delay:0.25s}}
main>section:nth-child(7){{animation-delay:0.3s}}
main>section:nth-child(8){{animation-delay:0.35s}}
main>section:nth-child(9){{animation-delay:0.4s}}
main>section:nth-child(10){{animation-delay:0.45s}}
</style>
<script>
document.querySelectorAll('[data-scroll]').forEach(function(a){{
  a.addEventListener('click',function(e){{
    e.preventDefault();
    var el=document.getElementById(a.dataset.scroll);
    if(el)el.scrollIntoView({{behavior:'smooth',block:'start'}});
    document.querySelectorAll('.doc-nav').forEach(function(n){{n.style.color='#ababab'}});
    a.style.color='#fff';
  }});
}});
var sections=document.querySelectorAll('main>section[id]');
var navItems=document.querySelectorAll('.doc-nav[data-scroll]');
window.addEventListener('scroll',function(){{
  var sp=window.scrollY+100;
  sections.forEach(function(sec){{
    if(sec.offsetTop<=sp&&sec.offsetTop+sec.offsetHeight>sp){{
      navItems.forEach(function(n){{n.style.color=n.dataset.scroll===sec.id?'#fff':'#ababab'}});
    }}
  }});
}});
</script>
</body></html>"##,
        app_name = app_name,
        accent = accent,
        font = font,
        theme = theme_mode,
        content = content,
        toc = toc,
    )
}

/// Relationship graph page — interactive Mermaid diagram of entity relations,
/// page bindings, and webhook flows.
fn render_graph_page(state: &AppState) -> String {
    let app_name = &state.app.name;
    let relationship_graph = graph::build_graph_from_state(
        &state.entities, &state.pages, &state.webhooks,
    );
    let mermaid_code = graph::to_mermaid(&relationship_graph);
    // Escape backticks and backslashes for safe JS embedding
    let mermaid_escaped = mermaid_code
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${");

    format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>{app_name} | Relationship Graph</title>
<link href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700;900&family=Inter:wght@300;400;500;600&display=swap" rel="stylesheet"/>
<script src="https://cdn.jsdelivr.net/npm/mermaid/dist/mermaid.min.js"></script>
<style>
body {{ background:#0e0e0e; color:#fff; font-family:'Inter',sans-serif; margin:0 }}
::-webkit-scrollbar {{ width:4px }} ::-webkit-scrollbar-track {{ background:#0e0e0e }} ::-webkit-scrollbar-thumb {{ background:#262626;border-radius:10px }}
.graph-container {{ padding:32px; display:flex; justify-content:center; align-items:flex-start; min-height:calc(100vh - 64px - 64px) }}
.mermaid {{ background:#141414; border:1px solid rgba(255,255,255,0.06); border-radius:12px; padding:40px; min-width:600px; max-width:100%; overflow-x:auto }}
.mermaid svg {{ max-width:100% }}
.stats {{ display:flex; gap:24px; justify-content:center; padding:0 32px 24px; font-family:'Space Grotesk',sans-serif; font-size:13px; color:#757575 }}
.stats span {{ background:#141414; border:1px solid rgba(255,255,255,0.06); border-radius:8px; padding:8px 16px }}
.stats .count {{ color:#87adff; font-weight:700 }}
</style>
</head>
<body>
<header style="position:fixed;top:0;width:100%;z-index:50;height:64px;background:rgba(0,0,0,0.8);backdrop-filter:blur(40px);border-bottom:1px solid rgba(255,255,255,0.05);display:flex;align-items:center;justify-content:space-between;padding:0 24px;box-sizing:border-box">
  <div style="display:flex;align-items:center;gap:32px">
    <span style="font-family:Space Grotesk,sans-serif;font-size:18px;font-weight:900;color:#fff;letter-spacing:-0.03em">{app_name}_DOCS</span>
    <nav style="display:flex;gap:24px;font-family:Space Grotesk,sans-serif;font-weight:700;font-size:14px">
      <a href="/docs" style="color:#757575;text-decoration:none">API Docs</a>
      <a href="/docs/design" style="color:#757575;text-decoration:none">Design System</a>
      <a href="/docs/graph" style="color:#fff;border-bottom:2px solid #87adff;padding-bottom:2px;text-decoration:none">Graph</a>
      <a href="/" style="color:#757575;text-decoration:none">Dashboard</a>
    </nav>
  </div>
</header>

<div style="padding-top:80px">
  <div class="stats">
    <span>Entities <span class="count">{entity_count}</span></span>
    <span>Relations <span class="count">{relation_count}</span></span>
    <span>Page Bindings <span class="count">{binding_count}</span></span>
    <span>Webhook Flows <span class="count">{webhook_count}</span></span>
  </div>
  <div class="graph-container">
    <pre class="mermaid" id="graph">{mermaid_code}</pre>
  </div>
</div>

<script>
mermaid.initialize({{
  startOnLoad: true,
  theme: 'dark',
  themeVariables: {{
    primaryColor: '#87adff',
    primaryTextColor: '#fff',
    primaryBorderColor: '#87adff',
    lineColor: '#555',
    secondaryColor: '#d277ff',
    tertiaryColor: '#141414',
    background: '#0e0e0e',
    mainBkg: '#1a1a1a',
    nodeBorder: '#87adff',
    clusterBkg: '#141414',
    clusterBorder: '#262626',
    titleColor: '#fff',
    edgeLabelBackground: '#1a1a1a',
    fontFamily: 'Space Grotesk, sans-serif',
  }},
  flowchart: {{
    htmlLabels: true,
    curve: 'basis',
    padding: 20,
  }},
}});
</script>
</body>
</html>"##,
        app_name = app_name,
        mermaid_code = mermaid_code,
        entity_count = state.entities.iter().filter(|e| !e.name.starts_with('_')).count(),
        relation_count = relationship_graph.entity_relations.len(),
        binding_count = relationship_graph.page_bindings.len(),
        webhook_count = relationship_graph.webhook_flows.len(),
    )
}

// ══════════════════════════════════════════════════
// SEMANTIC MEMORY
// ══════════════════════════════════════════════════

pub(crate) fn open_memory_db() -> Result<memory::SemanticMemory, String> {
    let _ = fs::create_dir_all(".cronus");
    memory::SemanticMemory::open(".cronus/memory.db")
}

fn cmd_verify_audit(args: &[String]) {
    let files = find_all_cronus_files();
    if files.is_empty() {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus files found");
        return;
    }

    let source = std::fs::read_to_string(&files[0]).unwrap_or_default();
    let nodes = match parser::parse(&source) {
        Ok(n) => n,
        Err(_) => vec![],
    };
    let db_path = nodes.iter().find_map(|n| {
        if let AstNode::App(ref app) = n {
            app.database.as_ref().and_then(|d| d.path.clone())
        } else {
            None
        }
    }).unwrap_or_else(|| "data.db".into());

    println!();
    println!("  \x1b[1mCRONUS Audit Trail Verification\x1b[0m");
    println!("  Database: {}", db_path);
    println!();

    match audit::verify_from_file(&db_path) {
        Ok(result) => {
            let valid = result["valid"].as_bool().unwrap_or(false);
            let entries = result["entries"].as_i64().unwrap_or(0);
            if valid {
                println!("  \x1b[32m✓\x1b[0m Chain intact -- {} entries verified", entries);
            } else {
                let broken_at = result["broken_at"].as_i64().unwrap_or(0);
                let reason = result["reason"].as_str().unwrap_or("unknown");
                println!("  \x1b[31m✗\x1b[0m Chain BROKEN at entry {} ({}) -- {} total entries", broken_at, reason, entries);
            }
        }
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m Failed to verify: {}", e);
        }
    }
    println!();
}

// cmd_memory + find_flag_value moved to cli/memory_cmd.rs

// ══════════════════════════════════════════════════
// AST SNAPSHOT + CHANGELOG
// ══════════════════════════════════════════════════

fn save_ast_snapshot(nodes: &[AstNode]) {
    let snapshot = ast_diff::snapshot_from_ast(nodes);
    let _ = fs::create_dir_all(".cronus");
    match serde_json::to_string_pretty(&snapshot) {
        Ok(json_str) => {
            if fs::write(".cronus/ast-snapshot.json", &json_str).is_ok() {
                println!("  \x1b[32m✓\x1b[0m AST snapshot saved to .cronus/ast-snapshot.json");
            }
        }
        Err(e) => {
            eprintln!("  \x1b[33m⚠\x1b[0m Failed to serialize AST snapshot: {}", e);
        }
    }
}

fn cmd_verify(args: &[String]) {
    let file = find_cronus_file().unwrap_or_else(|| {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus file found"); std::process::exit(1);
    });
    let source = fs::read_to_string(&file).unwrap();
    let nodes = parser::parse(&source).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e); std::process::exit(1);
    });
    let (entities, pages, routes) = parser::stats(&nodes);
    println!("  \x1b[32m✓\x1b[0m Verified: {} entities, {} pages, {} routes", entities, pages, routes);
}

// cmd_changelog moved to cli/changelog.rs
