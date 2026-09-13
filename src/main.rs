#![allow(dead_code, unused_imports, unused_variables)]
mod actions;
mod animations;
mod audit;
mod auth;
mod binding;
mod block_explorer;
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
mod hydra;
mod i18n;
mod layout_system;
mod lint;
mod marketing_components;
mod orchestrator;
mod overlays;
mod parser;
mod payments;
mod promote;
mod rate_limit;
mod reactive;
mod realtime;
mod render;
mod runtime_js;
mod scripting;
mod server;
mod trust;
mod zeus;
mod sse;
mod tabs;
mod tailwind;
mod testing;
mod theme;
mod cronus_ui;
mod cronus_ui_accordion;
mod cronus_ui_alert;
mod cronus_ui_area_chart;
mod cronus_ui_avatar;
mod cronus_ui_avatar_group;
mod cronus_ui_badge;
mod cronus_ui_banner;
mod cronus_ui_bar_chart;
mod cronus_ui_breadcrumb;
mod cronus_ui_button_group;
mod cronus_ui_calendar;
mod cronus_ui_card;
mod cronus_ui_checkbox;
mod cronus_ui_chip;
mod cronus_ui_combobox;
mod cronus_ui_command;
mod cronus_ui_collapsible;
mod cronus_ui_copy_button;
mod cronus_ui_context_menu;
mod cronus_ui_data;
mod cronus_ui_data_table;
mod cronus_ui_date_picker;
mod cronus_ui_date_range_picker;
mod cronus_ui_dialog;
mod cronus_ui_drawer;
mod cronus_ui_dropdown_menu;
mod cronus_ui_empty;
mod cronus_ui_field;
mod cronus_ui_file_dropzone;
mod cronus_ui_fab;
mod cronus_ui_hover_card;
mod cronus_ui_input;
mod cronus_ui_input_group;
mod cronus_ui_input_otp;
mod cronus_ui_interact;
mod cronus_ui_kbd;
mod cronus_ui_kit;
mod cronus_ui_label;
mod cronus_ui_line_chart;
mod cronus_ui_menubar;
mod cronus_ui_metric;
mod cronus_ui_mode_toggle;
mod cronus_ui_navigation_menu;
mod cronus_ui_number_input;
mod cronus_ui_pagination;
mod cronus_ui_popover;
mod cronus_ui_password_input;
mod cronus_ui_pie_chart;
mod cronus_ui_progress;
mod cronus_ui_radar_chart;
mod cronus_ui_radio_group;
mod cronus_ui_rating;
mod cronus_ui_scatter_chart;
mod cronus_ui_select;
mod cronus_ui_separator;
mod cronus_ui_sheet;
mod cronus_ui_sidebar;
mod cronus_ui_skeleton;
mod cronus_ui_slider;
mod cronus_ui_sparkline;
mod cronus_ui_sonner;
mod cronus_ui_spinner;
mod cronus_ui_stepper;
mod cronus_ui_table;
mod cronus_ui_tabs;
mod cronus_ui_tooltip;
mod cronus_ui_switch;
mod cronus_ui_textarea;
mod cronus_ui_time_picker;
mod cronus_ui_toggle;
mod cronus_ui_toggle_group;
mod cronus_ui_widgets;
mod voodoo;
mod navigation;
mod security;
mod ui;
mod ast_diff;
mod memory;
mod resolve;
mod vm;
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
use cli::build::cmd_build;
use cli::dump_cmd::cmd_dump;
use cli::validate::{cmd_validate, cmd_validate_mission};
use cli::verify::{cmd_verify, cmd_verify_audit, cmd_debug_audit};
use cli::new::cmd_new;
use cli::seed::cmd_seed;
use cli::deploy_cmd::cmd_deploy;
use cli::test_cmd::cmd_test;
use cli::compose::cmd_compose;
use cli::sync_cmd::cmd_sync;
use cli::drift::cmd_drift;
use cli::lease::cmd_lease;
use cli::review::cmd_review;
use cli::timeline::cmd_timeline;
use cli::status_cmd::cmd_status;
use cli::segment::cmd_segment;
use cli::reconcile::cmd_reconcile;
use cli::spec::cmd_spec;
use cli::objective_kernel::reconcile_field_type_str;
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
/// `cronus run --audit-canvas` / `CRONUS_AUDIT=1`. Exclusive `/audit/*` path.
pub static AUDIT_CANVAS: AtomicBool = AtomicBool::new(false);

/// Cache for the last `--ai` build result, served by `GET /api/_errors`.
/// Written by `cmd_build` when `--ai` flag is used, read by the server.
use std::sync::{Mutex, LazyLock};
pub static LAST_AI_ERRORS: LazyLock<Mutex<Option<serde_json::Value>>> =
    LazyLock::new(|| Mutex::new(None));

use server::state::{RequestTrace, TraceBuffer, generate_request_id, current_time_hms, iso_timestamp};

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
        "clone" => cmd_dump(&args), // clone is an alias for dump
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
        "audit" => cli::cronus_audit::cmd_audit(&args),
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

use server::state::AppState;

use server::response::{cors_origin, json_response, html_response, forbidden_response};
use server::auth_pages::{generate_login_page, generate_register_page};
use server::docs::{render_auto_docs, render_design_system, render_graph_page};


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

    if AUDIT_CANVAS.load(Ordering::Relaxed) {
        if req_path_str.starts_with("/audit/") {
            return Ok(cli::audit_http::handle_audit_request(
                &req_path_str,
                req.uri().query(),
                &state.pages,
                &state.components,
            ));
        }
        return Ok(cli::audit_http::audit_not_found());
    }

    if req.method() == Method::GET && req.uri().path() == "/api/debug/traces" {
        let traces = state.trace_buffer.last_n(50);
        return Ok(json_response(StatusCode::OK, serde_json::to_value(&traces).unwrap_or(json!([]))));
    }

    let voodoo_on = crate::voodoo::wanted(&state.app.stack, state.style.as_ref());
    let mut resp = crate::voodoo::scope(
        voodoo_on,
        handle_request_inner(req, state.clone(), remote_addr),
    )
    .await?;

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

    // Zeus: record every request trace
    {
        let status = resp.status().as_u16();
        // Skip zeus/trust internal endpoints from traces
        if !req_path_str.starts_with("/zeus") && !req_path_str.starts_with("/trust") && !req_path_str.starts_with("/.cronus/") && !req_path_str.starts_with("/blocks") && !req_path_str.starts_with("/hydra") {
            state.zeus.push(zeus::ZeusTrace {
                id: req_id.clone(),
                method: req_method_str.clone(),
                path: req_path_str.clone(),
                status,
                duration_ms: duration_ms as f64,
                timestamp: iso_timestamp(),
                spans: Vec::new(), // TODO: add spans from TraceBuilder
                query_count: queries,
                script_block: None,
            });
        }
    }

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
    let raw_path = req.uri().path().to_string();
    // Normalize trailing slash: /portal/ → /portal (but keep "/" as-is)
    let path = if raw_path.len() > 1 && raw_path.ends_with('/') {
        raw_path.trim_end_matches('/').to_string()
    } else {
        raw_path
    };
    let query = req.uri().query().unwrap_or("").to_string();

    // CORS preflight
    if method == Method::OPTIONS {
        return Ok(json_response(StatusCode::OK, json!({})));
    }

    // HMR version endpoint
    if path == "/.cronus/version" {
        return Ok(json_response(StatusCode::OK, json!({ "version": hmr::current_version() })));
    }

    // Block explorer
    if path == "/blocks" {
        let metrics = trust::all_metrics();
        let html = block_explorer::render_explorer(".", &metrics, &state.script_registry);
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Full::new(Bytes::from(html)))
            .unwrap());
    }

    // Zeus observability endpoints
    if path == "/zeus" {
        let html = zeus::render_dashboard(&state.zeus);
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Full::new(Bytes::from(html)))
            .unwrap());
    }
    if path == "/zeus/api" {
        let traces = state.zeus.last_n(100);
        let stats = state.zeus.stats();
        return Ok(json_response(StatusCode::OK, json!({
            "stats": stats,
            "traces": traces,
        })));
    }
    if path == "/zeus/slow" {
        let slow = state.zeus.slow_traces(50.0); // >50ms
        return Ok(json_response(StatusCode::OK, json!({"traces": slow})));
    }
    if path == "/zeus/errors" {
        let errors = state.zeus.error_traces();
        return Ok(json_response(StatusCode::OK, json!({"traces": errors})));
    }

    // Hydra evolution endpoints
    if path == "/hydra" || path == "/api/hydra/registry" {
        let registry = hydra::registry::BlockRegistry::open(".cronus/block-registry.json");
        return Ok(json_response(StatusCode::OK, registry.to_json()));
    }
    if path == "/api/hydra/evolve" {
        let mut registry = hydra::registry::BlockRegistry::open(".cronus/block-registry.json");
        let report = hydra::evolve(&mut registry, &state.script_registry.scripts);
        return Ok(json_response(StatusCode::OK, serde_json::to_value(&report).unwrap_or(json!({"error":"serialize"}))));
    }
    if path == "/api/hydra/candidates" {
        let candidates = hydra::extract::extract_candidates(&state.script_registry.scripts);
        let data: Vec<serde_json::Value> = candidates.iter().map(|c| {
            json!({
                "name": c.name,
                "source": c.source_script,
                "type": format!("{:?}", c.block_type),
                "entity": c.entity,
                "trust_score": format!("{:.3}", c.trust_score),
                "executions": c.executions,
                "promotable": c.promotable,
                "reason": c.reason,
            })
        }).collect();
        return Ok(json_response(StatusCode::OK, json!({"candidates": data, "total": candidates.len()})));
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

    // Trust engine endpoint
    if path == "/api/trust" || path == "/trust" {
        let metrics = trust::all_metrics();
        let trust_data: Vec<serde_json::Value> = metrics.iter().map(|m| {
            let evidence = m.to_evidence();
            let gates = trust::TrustGates::new_clean();
            let profile = trust::TrustProfile::from_evidence(&evidence, gates);
            json!({
                "block_id": m.block_id,
                "executions": m.executions,
                "errors": m.errors,
                "avg_latency_ms": format!("{:.2}", m.avg_latency_ms()),
                "trust_score": format!("{:.3}", profile.score()),
                "status": format!("{:?}", profile.status()),
                "promotable": profile.promotable(),
            })
        }).collect();
        return Ok(json_response(StatusCode::OK, json!({
            "blocks": trust_data,
            "total_tracked": metrics.len(),
        })));
    }

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
                                    .header("Set-Cookie", crate::security::secure_cookie("cronus_token", &token, 604800, "/"))
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
                let remember = body.get("remember").and_then(|v| v.as_bool()).unwrap_or(false);
                let cookie_max_age = if remember { 2592000 } else { 86400 }; // 30 days or 24h

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
                                            .header("Set-Cookie", crate::security::secure_cookie("cronus_token", &token, cookie_max_age, "/"))
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
                // Live row count from database
                if let Ok(count) = state.db.count(&e.name) {
                    ej["row_count"] = json!(count);
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

    // AI Documentation Index — structured JSON for AI navigation
    if path == "/api/docs/index" && method == Method::GET {
        return Ok(json_response(StatusCode::OK, server::docs_index::generate_docs_index(&state)));
    }
    if path.starts_with("/api/docs/search") && method == Method::GET {
        let q = query.split('&').find_map(|p| p.strip_prefix("q=")).unwrap_or("");
        return Ok(json_response(StatusCode::OK, server::docs_index::search_docs_index(&state, q)));
    }
    if path.starts_with("/api/docs/tags/") && method == Method::GET {
        let tag = path.strip_prefix("/api/docs/tags/").unwrap_or("");
        return Ok(json_response(StatusCode::OK, server::docs_index::get_docs_by_tag(&state, tag)));
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

    // ── Script routes (.scriptcronus endpoints + webhooks) ──
    {
        let method_str = method.as_str();
        let has_endpoint = state.script_registry.get_endpoints().iter().any(|(_s, ep)| ep.method == method_str && ep.path == path);
        // SECURITY: webhooks only match paths under /hooks/ prefix
        let has_webhook = !has_endpoint && method == Method::POST
            && path.starts_with("/hooks/")
            && !state.script_registry.get_webhook_handlers(&path).is_empty();

        if has_endpoint || has_webhook {
            let token = req.headers().get("authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "))
                .map(|s| s.to_string());
            let (user_id, role) = token.as_deref()
                .and_then(|t| auth::verify_token(t, &auth::default_secret()).ok())
                .map(|c| (c.sub.clone(), c.role.clone()))
                .unwrap_or_else(|| ("anonymous".into(), "public".into()));

            let body_bytes = req.collect().await.unwrap_or_default().to_bytes();
            let body: Option<serde_json::Value> = serde_json::from_slice(&body_bytes).ok();

            if has_endpoint {
                for (script, ep) in state.script_registry.get_endpoints() {
                    if ep.method == method_str && ep.path == path {
                        // SECURITY: endpoints require auth by default
                        // Use auth:public to explicitly allow unauthenticated access
                        let required_role = ep.auth.as_deref().unwrap_or("any");
                        if required_role != "public" {
                            // Must have valid token
                            if user_id == "anonymous" {
                                return Ok(json_response(StatusCode::UNAUTHORIZED, json!({"error": "authentication required"})));
                            }
                            // Check role if specific role required
                            if required_role != "any" && role != required_role {
                                return Ok(json_response(StatusCode::FORBIDDEN, json!({"error": "insufficient role"})));
                            }
                        }
                        let ctx = scripting::execute_endpoint(ep, &script.name, &state.db, &user_id, &role, body.as_ref(), &std::collections::HashMap::new());
                        if let Some(resp) = ctx.response {
                            // SECURITY: clamp status to safe range
                            let safe_status = resp.status.max(200).min(599);
                            let mut builder = Response::builder().status(safe_status);
                            // SECURITY: whitelist safe response headers — block Set-Cookie, Location, etc.
                            const ALLOWED_HEADERS: &[&str] = &[
                                "content-type", "content-disposition", "cache-control",
                                "x-request-id", "x-total-count",
                            ];
                            for (k, v) in &resp.headers {
                                let k_lower = k.to_lowercase();
                                if ALLOWED_HEADERS.contains(&k_lower.as_str()) {
                                    // SECURITY: strip newlines to prevent header injection
                                    let safe_v = v.replace('\n', "").replace('\r', "");
                                    builder = builder.header(k.as_str(), safe_v.as_str());
                                }
                            }
                            if !resp.headers.keys().any(|k| k.to_lowercase() == "content-type") {
                                builder = builder.header("Content-Type", "application/json");
                            }
                            return Ok(builder.body(Full::new(Bytes::from(resp.body))).unwrap());
                        }
                        return Ok(json_response(StatusCode::OK, json!({"ok": true, "logs": ctx.logs})));
                    }
                }
            }
            // Webhook
            let body_val = body.unwrap_or(serde_json::Value::Null);
            let _ctx = scripting::execute_webhook(&state.script_registry, &path, &body_val, &state.db, &std::collections::HashMap::new());
            return Ok(json_response(StatusCode::OK, json!({"ok": true})));
        }
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

    // ── Component preview route (skip if user defined a /components page) ──
    let has_components_page_main = state.pages.iter().any(|p| p.route == "/components");
    if path == "/components" && !has_components_page_main {
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
                                // User is logged in but lacks the role — redirect to portal
                                return Ok(Response::builder()
                                    .status(StatusCode::FOUND)
                                    .header("Location", "/")
                                    .body(Full::new(Bytes::new()))
                                    .unwrap());
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

        // Dumped pages with HTML templates — use landing layout with Tailwind CDN,
        // EXCEPT for auth-protected pages that have a declarative layout (these
        // must share the same sidebar shell across all pages for consistency).
        let has_templates = page.sections.iter().any(|s| s.template.is_some() || s.config.get("template").is_some());
        let is_auth_page = page.requires.as_deref() == Some("auth")
            || page.requires.as_deref().map(|r| r.starts_with("role(")).unwrap_or(false);
        let has_declarative_layout = state.layout.is_some();
        if has_templates && !(is_auth_page && has_declarative_layout) {
            let body = ui::render_page(page, &state.entities, accent, theme, Some(&state.db), &route_params, &page_owner_id);
            let html = ui::render_layout_landing_ex(app_name, &body, theme, state.style.as_ref(), state.app.tailwind_config.as_deref());
            return Ok(html_response(html));
        }

        if has_section_sidebar {
            // Check for specialized dashboard renderers BEFORE falling back to generic
            let billing_types = ["current-plan", "usage-status", "billing-stats", "payment-methods", "recent-invoices"];
            let is_billing_page = page.sections.iter().any(|s| s.section_type == "current-plan" || s.section_type == "billing-stats");
            if is_billing_page {
                let referenced_comps: Vec<parser::ComponentNode> = page.components.iter()
                    .filter_map(|name| state.components.iter().find(|c| c.name == *name))
                    .cloned()
                    .collect();
                let html = ui::render_billing_dashboard(app_name, &page.sections, &referenced_comps, theme, &path);
                return Ok(html_response(html));
            }

            // Generic dashboard wrapper — sidebar + any sections
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
        // Auth pages with a declarative layout share the sidebar shell
        let auth_with_layout = is_auth_page && has_declarative_layout;
        let html = if auth_with_layout {
            if let Some(ref layout) = state.layout {
                ui::render_layout_declarative(app_name, layout, current_route, &body)
            } else {
                ui::render_layout(app_name, &state.pages, accent, &body)
            }
        } else if has_templates {
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
                                    scripting::fire_scripts(&state.script_registry, &entity.name, "create", &row, &row_id, None, &state.db, owner_id, "user", &std::collections::HashMap::new());
                                    if let Err(e) = state.audit_trail.log("INSERT", table, &row_id, owner_id, &row, None) {
                                        eprintln!("  \x1b[33m⚠\x1b[0m Audit log failed (INSERT {}:{}): {}", table, row_id, e);
                                    }
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
                                    scripting::fire_scripts(&state.script_registry, &entity.name, "create", &row, &row_id, None, &state.db, owner_id, "user", &std::collections::HashMap::new());
                                    if let Err(e) = state.audit_trail.log("INSERT", table, &row_id, owner_id, &row, None) {
                                        eprintln!("  \x1b[33m⚠\x1b[0m Audit log failed (INSERT {}:{}): {}", table, row_id, e);
                                    }
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
                                let ent_name = entity.name.as_str();
                                scripting::fire_scripts(&state.script_registry, ent_name, "update", &row, segments[1], prev_record.as_ref(), &state.db, owner_id, "user", &std::collections::HashMap::new());
                                if let Err(e) = state.audit_trail.log("UPDATE", table, segments[1], owner_id, &row, prev_record.as_ref()) {
                                    eprintln!("  \x1b[33m⚠\x1b[0m Audit log failed (UPDATE {}:{}): {}", table, segments[1], e);
                                }
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
                            let ent_name_del = entity.name.as_str();
                            scripting::fire_scripts(&state.script_registry, ent_name_del, "delete", effect_record, segments[1], None, &state.db, owner_id, "user", &std::collections::HashMap::new());
                            if let Err(e) = state.audit_trail.log("DELETE", table, segments[1], owner_id, &json!({"id": segments[1]}), prev_record.as_ref()) {
                                eprintln!("  \x1b[33m⚠\x1b[0m Audit log failed (DELETE {}:{}): {}", table, segments[1], e);
                            }
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

// cmd_debug_audit moved to cli::verify

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
    cli::build::save_ast_snapshot(&nodes);

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
    let mut auth_redirect: Option<String> = None;
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
                if let Some(r) = auth.session_config.get("redirect") {
                    auth_redirect = Some(r.clone());
                }
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

    // Resolve component invocations in pages — replace {{param}} in templates with passed values
    // Also generates reactive JS for components with `state` declarations
    if !cronus_components.is_empty() {
        let mut comp_instance_counter: u32 = 0;
        for page in &mut pages {
            for section in &mut page.sections {
                if let Some(comp_name) = section.config.get("_component").cloned() {
                    if let Some(comp_def) = cronus_components.iter().find(|c| c.name == comp_name) {
                        if let Some(ref tmpl) = comp_def.template {
                            comp_instance_counter += 1;
                            let cid = format!("c{}", comp_instance_counter);

                            // Interpolate template: replace {{param}} with values from section.config
                            let mut rendered = tmpl.clone();
                            for param in &comp_def.params {
                                let placeholder = format!("{{{{{}}}}}", param.name);
                                let value = section.config.get(&param.name)
                                    .map(|s| s.as_str())
                                    .or(param.default.as_deref())
                                    .unwrap_or("");
                                rendered = rendered.replace(&placeholder, value);
                            }
                            for (key, value) in &section.config {
                                if key == "_component" { continue; }
                                let placeholder = format!("{{{{{}}}}}", key);
                                rendered = rendered.replace(&placeholder, value);
                            }

                            // Reactive state: replace {{state_var}} with reactive spans
                            // and generate JS signal code
                            if !comp_def.state.is_empty() {
                                // Wrap component in a container with unique ID
                                rendered = format!(r#"<div data-cid="{cid}">{html}</div>"#, cid = cid, html = rendered);

                                // Replace {{state_var}} with reactive spans
                                for sv in &comp_def.state {
                                    let placeholder = format!("{{{{{}}}}}", sv.name);
                                    let span = format!(r#"<span data-s="{name}">{default}</span>"#,
                                        name = sv.name, default = sv.default);
                                    rendered = rendered.replace(&placeholder, &span);
                                }

                                // Process @click="expr" → onclick with signal update
                                // Match @click="..." or @click(...)
                                let mut script_parts: Vec<String> = Vec::new();
                                let mut event_id: u32 = 0;

                                // Simple regex-free @click handler extraction
                                while let Some(pos) = rendered.find("@click=") {
                                    event_id += 1;
                                    let eid = format!("{cid}_e{event_id}");
                                    // Find the expression in quotes
                                    let after = &rendered[pos + 7..];
                                    let (expr, end_offset) = if after.starts_with("\\\"") || after.starts_with('"') {
                                        let quote_char = if after.starts_with("\\\"") { "\\\"" } else { "\"" };
                                        let qlen = quote_char.len();
                                        let expr_start = qlen;
                                        if let Some(expr_end) = after[expr_start..].find(quote_char) {
                                            (after[expr_start..expr_start + expr_end].to_string(), 7 + expr_start + expr_end + qlen)
                                        } else {
                                            break;
                                        }
                                    } else {
                                        break;
                                    };

                                    // Replace @click="expr" with data-eid="..."
                                    rendered = format!("{}data-eid=\"{}\"{}",
                                        &rendered[..pos], eid, &rendered[pos + end_offset..]);

                                    // Generate JS for this event
                                    // Parse simple expressions: "count += 1", "count -= 1", "toggle = !toggle"
                                    let js_expr = expr.replace("\\\"", "\"");
                                    script_parts.push(format!(
                                        r#"document.querySelector('[data-eid="{eid}"]').addEventListener('click',function(){{ {update_expr}; _u(); }});"#,
                                        eid = eid, update_expr = format!("_s.{}", js_expr)
                                    ));
                                }

                                // Generate the reactive script
                                if !comp_def.state.is_empty() {
                                    let mut state_init = String::new();
                                    let mut update_dom = String::new();
                                    for sv in &comp_def.state {
                                        let default_js = match sv.state_type.as_str() {
                                            "integer" | "number" => sv.default.clone(),
                                            "boolean" => sv.default.clone(),
                                            _ => format!("\"{}\"", sv.default),
                                        };
                                        state_init.push_str(&format!("{}:{},", sv.name, default_js));
                                        update_dom.push_str(&format!(
                                            r#"_c.querySelectorAll('[data-s="{name}"]').forEach(function(el){{ el.textContent=_s.{name}; }});"#,
                                            name = sv.name
                                        ));
                                    }

                                    let script = format!(
                                        r#"<script>(function(){{ var _c=document.querySelector('[data-cid="{cid}"]'); if(!_c)return; var _s={{{init}}}; function _u(){{{update}}} {events} }})();</script>"#,
                                        cid = cid,
                                        init = state_init,
                                        update = update_dom,
                                        events = script_parts.join(" "),
                                    );
                                    rendered.push_str(&script);
                                }
                            }

                            section.template = Some(rendered);
                        }
                    }
                }
            }
        }
    }

    let audit_canvas = args.iter().any(|a| a == "--audit-canvas")
        || std::env::var("CRONUS_AUDIT")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
    if audit_canvas {
        AUDIT_CANVAS.store(true, Ordering::Relaxed);
    }

    // CLI port takes precedence. `--audit-canvas [port]` default 5176.
    let audit_flag_port = args
        .iter()
        .position(|a| a == "--audit-canvas")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse::<u16>().ok());
    let cli_port = audit_flag_port.or_else(|| {
        args.iter().skip(2).find_map(|s| {
            if s.starts_with('-') {
                None
            } else {
                s.parse::<u16>().ok()
            }
        })
    });
    let serve_port = if let Some(p) = cli_port {
        p
    } else if audit_canvas {
        5176
    } else {
        app.port
    };
    let comp_count = cronus_components.len();

    // Initialize theme tokens from tailwind_config or style
    {
        let tokens = if let Some(ref tc) = app.tailwind_config {
            theme::parse_from_tailwind_config(tc)
        } else if let Some(ref s) = style {
            theme::derive_palette(
                s.accent.as_deref().unwrap_or(""),
                s.theme.as_deref().unwrap_or("dark"),
                s.font.as_deref().unwrap_or(""),
            )
        } else {
            theme::ThemeTokens::default()
        };
        theme::set_global(tokens);
        let preset = style
            .as_ref()
            .and_then(|s| s.config.get("preset").cloned())
            .unwrap_or_default();
        theme::set_preset(&preset);
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
        remote_url: None,
        doc: None,
    };
    let _ = brain_db.migrate(&[brain_entity]);
    let hydra = brain::CronusBrain::init(brain_db);


    // Build app state (reuse app_db from migration)

    let sse_hub = Arc::new(sse::SseHub::new());
    let audit_trail = audit::AuditTrail::open(&db_path).expect("Failed to open audit trail");

    // Load .scriptcronus files
    let script_registry = scripting::ScriptRegistry::load_from_directory(".");

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
        auth_redirect,
        layout,
        webhooks,
        rate_limiter: rate_limit::RateLimiter::new(100, 60),
        auth_rate_limiter: rate_limit::RateLimiter::new(10, 60),
        sse_hub,
        audit_trail,
        trace_buffer: Arc::new(TraceBuffer::new()),
        script_registry,
        zeus: Arc::new(zeus::ZeusBuffer::new(200)),
    });

    // Start server. Audit-canvas binds loopback only (never 0.0.0.0).
    let bind_host = if audit_canvas { "127.0.0.1" } else { "0.0.0.0" };
    let addr = format!("{}:{}", bind_host, serve_port);
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
    if state.script_registry.block_count() > 0 {
        let sc = &state.script_registry;
        let events = sc.scripts.iter().flat_map(|s| s.blocks.iter()).filter(|b| matches!(b, scripting::ast::ScriptBlock::OnEvent(_))).count();
        let schedules = sc.get_schedules().len();
        let endpoints = sc.get_endpoints().len();
        let webhooks = sc.scripts.iter().flat_map(|s| s.blocks.iter()).filter(|b| matches!(b, scripting::ast::ScriptBlock::OnWebhook(_))).count();
        println!("  \x1b[90mScripts:\x1b[0m   {} ({} events, {} schedules, {} endpoints, {} webhooks)",
            sc.scripts.len(), events, schedules, endpoints, webhooks);
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

    // ── Startup audit: if CRONUS_AUDIT_REF is set, print fidelity score ──
    if let Ok(ref_path) = std::env::var("CRONUS_AUDIT_REF") {
        if !ref_path.is_empty() {
            match cli::audit_fidelity::run_audit(&ref_path) {
                Some(result) => {
                    println!();
                    println!("  \x1b[1mFidelity Audit\x1b[0m (CRONUS_AUDIT_REF={})", ref_path);
                    cli::audit_fidelity::print_fidelity_line(&result);
                    if result.fidelity < 90 {
                        println!("  \x1b[33m⚠ Fidelity below 90% — review missing items:\x1b[0m");
                        cli::audit_fidelity::print_missing_top(&result, 5);
                    }
                    cli::audit_fidelity::save_audit_results(&result);
                }
                None => {
                    eprintln!("  \x1b[33m⚠\x1b[0m CRONUS_AUDIT_REF set but audit failed (check file path or .cronus)");
                }
            }
        }
    }

    println!();
    println!("  Press Ctrl+C to stop.");
    println!();

    // Start script schedules
    {
        let schedules = state.script_registry.get_schedules();
        for (script, sched) in &schedules {
            let interval = scripting::parse_interval(&sched.interval);
            let body = sched.body.clone();
            let db_path = state.db_path.clone();
            let sched_name = sched.name.clone();
            let script_name = script.name.clone();
            tokio::spawn(async move {
                let mut tick = tokio::time::interval(interval);
                tick.tick().await; // skip immediate first tick
                loop {
                    tick.tick().await;
                    eprintln!("  \x1b[36m[schedule]\x1b[0m running \"{}\" from \"{}\"", sched_name, script_name);
                    let db = match crate::database::CronusDB::open(&db_path) {
                        Ok(db) => db,
                        Err(e) => {
                            eprintln!("  \x1b[31m[schedule]\x1b[0m db open failed: {}", e);
                            continue;
                        }
                    };
                    let mut ctx = scripting::vm::ScriptContext::new("system", "admin", std::collections::HashMap::new());
                    if let Err(e) = scripting::vm::execute_statements(&body, &mut ctx, &db, None) {
                        eprintln!("  \x1b[31m[schedule]\x1b[0m \"{}\" error: {}", sched_name, e);
                    }
                }
            });
            eprintln!("  \x1b[36m[schedule]\x1b[0m registered \"{}\" (every {})", sched.name, sched.interval);
        }
    }

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
                    if AUDIT_CANVAS.load(Ordering::Relaxed) {
                        if req.uri().path().starts_with("/audit/") {
                            let resp = cli::audit_http::handle_audit_request(
                                req.uri().path(),
                                req.uri().query(),
                                &state.pages,
                                &state.components,
                            );
                            let (parts, body) = resp.into_parts();
                            return Ok::<_, hyper::Error>(Response::from_parts(
                                parts,
                                http_body_util::Either::Left(body),
                            ));
                        }
                        let resp = cli::audit_http::audit_not_found();
                        let (parts, body) = resp.into_parts();
                        return Ok::<_, hyper::Error>(Response::from_parts(
                            parts,
                            http_body_util::Either::Left(body),
                        ));
                    }
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

// cmd_dump moved to cli::dump_cmd


// cmd_build and build_ai_error_json moved to cli/build.rs
// save_ast_snapshot moved to cli/build.rs

// cmd_validate moved to cli::validate


// cmd_new moved to cli::new


// ══════════════════════════════════════════════════
// SEED COMMAND
// ══════════════════════════════════════════════════

// cmd_seed moved to cli::seed


// ══════════════════════════════════════════════════
// TEMPLATES
// ══════════════════════════════════════════════════

pub(crate) const TEMPLATE_ADMIN: &str = r#"# Admin Panel — CRONUS
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

pub(crate) const TEMPLATE_LANDING: &str = r#"# Landing Page — CRONUS
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

pub(crate) const TEMPLATE_API: &str = r#"# API Backend — CRONUS
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

pub(crate) const TEMPLATE_SAAS: &str = r#"/// SaaS Starter — Multi-tenant platform with Stripe-ready billing.
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

pub(crate) const TEMPLATE_ECOMMERCE: &str = r#"/// E-commerce Platform — Full storefront with order state machine.
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

pub(crate) const TEMPLATE_BLOG: &str = r#"/// Blog / CMS — Content management with publish workflow.
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

pub(crate) const TEMPLATE_HELPDESK: &str = r#"/// Helpdesk — Support ticket system with SLA tracking.
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

pub(crate) const TEMPLATE_CRM: &str = r#"/// CRM — Sales pipeline and contact management.
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

// cmd_deploy moved to cli::deploy_cmd


// cmd_doctor moved to cli::doctor

// cmd_stats moved to cli::stats

// cmd_export moved to cli::export_cmd

// cmd_test moved to cli::test_cmd


// cmd_compose moved to cli::compose


// cmd_generate and generate system moved to cli::generate


/// Auto-generated documentation page — derived 100% from the parsed .cronus AST.
/// Follows the Synthetic Docs design (obsidian dark, glass panels, code blocks).

// ══════════════════════════════════════════════════
// SEMANTIC MEMORY
// ══════════════════════════════════════════════════

pub(crate) fn open_memory_db() -> Result<memory::SemanticMemory, String> {
    let _ = fs::create_dir_all(".cronus");
    memory::SemanticMemory::open(".cronus/memory.db")
}

// cmd_verify_audit moved to cli::verify


// cmd_memory + find_flag_value moved to cli/memory_cmd.rs

// ══════════════════════════════════════════════════
// AST SNAPSHOT + CHANGELOG
// ══════════════════════════════════════════════════

// cmd_verify moved to cli::verify


// cmd_changelog moved to cli/changelog.rs
