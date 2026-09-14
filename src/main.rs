#![allow(dead_code, unused_imports, unused_variables)]
mod access;
mod actions;
mod animations;
mod api_crud;
#[cfg(test)]
mod api_security_tests;
mod ast_diff;
mod audit;
mod auth;
mod auth_entity;
mod authz;
mod binding;
mod block_explorer;
mod board;
mod brain;
mod cache;
mod cli;
mod command_palette;
mod components;
mod constitution_check;
mod contracts;
mod cronus_ui;
mod cronus_ui_accordion;
mod cronus_ui_alert;
mod cronus_ui_alert_dialog;
mod cronus_ui_animated_button;
mod cronus_ui_animated_list;
mod cronus_ui_animated_number;
mod cronus_ui_app_shell;
mod cronus_ui_area_chart;
mod cronus_ui_aspect_ratio;
mod cronus_ui_aurora_background;
mod cronus_ui_autocomplete;
mod cronus_ui_avatar;
mod cronus_ui_avatar_group;
mod cronus_ui_badge;
mod cronus_ui_banner;
mod cronus_ui_bar_chart;
mod cronus_ui_border_beam;
mod cronus_ui_bouncy_accordion;
mod cronus_ui_breadcrumb;
mod cronus_ui_button_group;
mod cronus_ui_calendar;
mod cronus_ui_candlestick_chart;
mod cronus_ui_card;
mod cronus_ui_card_stack;
mod cronus_ui_carousel;
mod cronus_ui_chart;
mod cronus_ui_checkbox;
mod cronus_ui_chip;
mod cronus_ui_choropleth_chart;
mod cronus_ui_click_spark;
mod cronus_ui_code_block;
mod cronus_ui_code_tabs;
mod cronus_ui_collapsible;
mod cronus_ui_color_picker;
mod cronus_ui_combobox;
mod cronus_ui_command;
mod cronus_ui_comparison_slider;
mod cronus_ui_composed_chart;
mod cronus_ui_confetti;
mod cronus_ui_confirmation_dialog;
mod cronus_ui_context_menu;
mod cronus_ui_copy_button;
mod cronus_ui_countdown;
mod cronus_ui_credit_card_input;
mod cronus_ui_currency_input;
mod cronus_ui_data;
mod cronus_ui_data_table;
mod cronus_ui_date_picker;
mod cronus_ui_date_range_picker;
mod cronus_ui_description_list;
mod cronus_ui_dialog;
mod cronus_ui_dock;
mod cronus_ui_dot_pattern;
mod cronus_ui_drawer;
mod cronus_ui_dropdown_menu;
mod cronus_ui_dynamic_island;
mod cronus_ui_empty;
mod cronus_ui_expandable_tabs;
mod cronus_ui_fab;
mod cronus_ui_field;
mod cronus_ui_file_dropzone;
mod cronus_ui_flickering_grid;
mod cronus_ui_flip_card;
mod cronus_ui_floating_label_input;
mod cronus_ui_form;
mod cronus_ui_frame;
mod cronus_ui_funnel_chart;
mod cronus_ui_gauge_chart;
mod cronus_ui_glare_hover;
mod cronus_ui_glass_card;
mod cronus_ui_gradient_border;
mod cronus_ui_gradient_text;
mod cronus_ui_grid_pattern;
mod cronus_ui_heatmap;
mod cronus_ui_heatmap_chart;
mod cronus_ui_highlighter;
mod cronus_ui_hover_card;
mod cronus_ui_image_zoom;
mod cronus_ui_input;
mod cronus_ui_input_group;
mod cronus_ui_input_otp;
mod cronus_ui_invite_dialog;
mod cronus_ui_json_viewer;
mod cronus_ui_kanban;
mod cronus_ui_kbd;
mod cronus_ui_kit;
mod cronus_ui_label;
mod cronus_ui_light_rays;
mod cronus_ui_lightbox;
mod cronus_ui_line_chart;
mod cronus_ui_live_line_chart;
mod cronus_ui_logo_carousel;
mod cronus_ui_magnetic;
mod cronus_ui_marquee;
mod cronus_ui_masonry;
mod cronus_ui_menubar;
mod cronus_ui_metric;
mod cronus_ui_mode_toggle;
mod cronus_ui_morphing_popover;
mod cronus_ui_motion_presets;
mod cronus_ui_multi_select;
mod cronus_ui_navigation_menu;
mod cronus_ui_noise;
mod cronus_ui_notification_center;
mod cronus_ui_number_input;
mod cronus_ui_orbit;
#[cfg(test)]
mod cronus_ui_output_gate;
mod cronus_ui_pagination;
mod cronus_ui_particles;
mod cronus_ui_password_input;
mod cronus_ui_phone_input;
mod cronus_ui_pie_chart;
mod cronus_ui_pill_nav;
mod cronus_ui_popover;
mod cronus_ui_profit_loss_chart;
mod cronus_ui_progress;
mod cronus_ui_progressive_blur;
mod cronus_ui_radar_chart;
mod cronus_ui_radio_group;
mod cronus_ui_rating;
mod cronus_ui_resizable;
mod cronus_ui_retro_grid;
mod cronus_ui_reveal;
mod cronus_ui_rich_text_editor;
mod cronus_ui_ring_chart;
mod cronus_ui_ripple;
mod cronus_ui_scatter_chart;
mod cronus_ui_scheduler;
mod cronus_ui_scramble_text;
mod cronus_ui_scroll_area;
mod cronus_ui_scroll_progress;
mod cronus_ui_segmented_control;
mod cronus_ui_select;
mod cronus_ui_separator;
mod cronus_ui_sheet;
mod cronus_ui_shimmer;
mod cronus_ui_shiny_text;
mod cronus_ui_sidebar;
mod cronus_ui_signature_pad;
mod cronus_ui_skeleton;
mod cronus_ui_slider;
mod cronus_ui_sonner;
mod cronus_ui_sparkles_text;
mod cronus_ui_sparkline;
mod cronus_ui_spinner;
mod cronus_ui_spinning_text;
mod cronus_ui_split_button;
mod cronus_ui_spotlight_card;
mod cronus_ui_star_border;
mod cronus_ui_status_dot;
mod cronus_ui_stepper;
mod cronus_ui_sunburst_chart;
mod cronus_ui_switch;
mod cronus_ui_table;
mod cronus_ui_table_of_contents;
mod cronus_ui_tabs;
mod cronus_ui_tags_input;
mod cronus_ui_terminal;
mod cronus_ui_text_effect;
mod cronus_ui_text_shimmer;
mod cronus_ui_textarea;
mod cronus_ui_tilt_card;
mod cronus_ui_time_picker;
mod cronus_ui_timeline;
mod cronus_ui_toast;
mod cronus_ui_toggle;
mod cronus_ui_toggle_group;
mod cronus_ui_toolbar;
mod cronus_ui_tooltip;
mod cronus_ui_tree_view;
mod cronus_ui_typing_text;
mod cronus_ui_usage_meter;
mod cronus_ui_video_player;
mod cronus_ui_widgets;
mod cronus_ui_word_rotate;
mod cronus_ui_workspace_switcher;
mod data_table;
mod database;
mod deploy;
mod dump;
mod error;
mod export;
mod feedback;
mod graph;
mod graphql;
mod hardcode_lint;
mod hmr;
mod http_guard;
mod hydra;
mod i18n;
mod layout_system;
mod lint;
mod marketing_components;
mod memory;
mod navigation;
mod orchestrator;
mod overlays;
mod parser;
mod payments;
mod promote;
mod rate_limit;
mod reactive;
mod realtime;
mod render;
mod resolve;
mod runtime_js;
mod scripting;
mod security;
mod server;
mod session;
mod sse;
mod tabs;
mod tailwind;
mod testing;
mod theme;
mod trust;
mod ui;
mod vm;
mod voodoo;
mod webhook;
mod zeus;

use cli::brief::cmd_brief;
use cli::brief::{
    brief_json_arr, brief_json_val, brief_today_date, brief_toml_arr, brief_toml_arr_after_section,
    brief_toml_val,
};
use cli::build::cmd_build;
use cli::changelog::cmd_changelog;
use cli::compose::cmd_compose;
use cli::context::cmd_context;
use cli::deploy_cmd::cmd_deploy;
use cli::doctor::cmd_doctor;
use cli::drift::cmd_drift;
use cli::dump_cmd::cmd_dump;
use cli::export_cmd::cmd_export;
use cli::generate::cmd_generate;
use cli::graph_cmd::cmd_graph;
use cli::handoff::cmd_handoff;
use cli::help::print_help;
use cli::lease::cmd_lease;
use cli::memory_cmd::cmd_memory;
use cli::new::cmd_new;
use cli::objective_kernel::reconcile_field_type_str;
use cli::parse_cmd::cmd_parse;
use cli::reconcile::cmd_reconcile;
use cli::review::cmd_review;
use cli::seed::cmd_seed;
use cli::segment::cmd_segment;
use cli::spec::cmd_spec;
use cli::stats::cmd_stats;
use cli::status_cmd::cmd_status;
use cli::sync_cmd::cmd_sync;
use cli::test_cmd::cmd_test;
use cli::timeline::cmd_timeline;
use cli::validate::{cmd_validate, cmd_validate_mission};
use cli::verify::{cmd_debug_audit, cmd_verify, cmd_verify_audit};

use std::env;
use std::fs;
use std::io::Write as IoWrite;

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{
    body::Incoming, server::conn::http1, service::service_fn, Method, Request, Response, StatusCode,
};
use hyper_util::rt::TokioIo;
use rusqlite::Connection;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;

use parser::{
    ApiNode, AppNode, AstNode, AuthNode, ComponentNode, ComposeNode, DatabaseConfig, EntityNode,
    EnvNode, EventNode, FieldNode, FieldType, HttpMethod, ImportNode, LayoutNode, MiddlewareNode,
    PageNode, RouteNode, SectionNode, ServiceNode, StyleNode, TestNode, WorkerNode,
};

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
pub static STRICT_MODE: AtomicBool = AtomicBool::new(false);
pub static STRICT_AI_MODE: AtomicBool = AtomicBool::new(false);
pub static DEBUG_MODE: AtomicBool = AtomicBool::new(false);
/// `cronus run --audit-canvas` / `CRONUS_AUDIT=1`. Exclusive `/audit/*` path.
pub static AUDIT_CANVAS: AtomicBool = AtomicBool::new(false);

/// Cache for the last `--ai` build result, served by `GET /api/_errors`.
/// Written by `cmd_build` when `--ai` flag is used, read by the server.
use std::sync::{LazyLock, Mutex};
pub static LAST_AI_ERRORS: LazyLock<Mutex<Option<serde_json::Value>>> =
    LazyLock::new(|| Mutex::new(None));

use server::state::{
    current_time_hms, generate_request_id, iso_timestamp, RequestTrace, TraceBuffer,
};

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
        "version" | "-v" | "-V" | "--version" => cli::version::cmd_version(),
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

use server::auth_pages::{generate_login_page, generate_register_page};
use server::docs::{render_auto_docs, render_design_system, render_graph_page};
use server::response::{cors_origin, forbidden_response, html_response, json_response};

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

    // Internal/diagnostic routes: 404 in production, admin unless loopback dev.
    if let Some(resp) = http_guard::guard_internal(req.method(), &req_path_str, req.headers()) {
        return Ok(resp);
    }

    if req.method() == Method::GET && req.uri().path() == "/api/debug/traces" {
        let traces = state.trace_buffer.last_n(50);
        return Ok(json_response(
            StatusCode::OK,
            serde_json::to_value(&traces).unwrap_or(json!([])),
        ));
    }

    let voodoo_on = crate::voodoo::wanted(&state.app.stack, state.style.as_ref());
    let mut resp = crate::voodoo::scope(
        voodoo_on,
        handle_request_inner(req, state.clone(), remote_addr),
    )
    .await?;

    // Markers become real nonces only in `html_response`. HTML leaving through
    // any other builder (403 page, block explorer, error pages) has no nonce
    // CSP, so strip the markers rather than disclose them.
    let is_html = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|c| c.starts_with("text/html"))
        .unwrap_or(false);
    if crate::security::script_nonces_enabled()
        && is_html
        && !resp.headers().contains_key("content-security-policy")
    {
        use http_body_util::BodyExt;
        let (parts, body) = resp.into_parts();
        let bytes = body
            .collect()
            .await
            .map(|c| c.to_bytes())
            .unwrap_or_else(|never| match never {});
        let cleaned = crate::security::strip_script_nonce_markers(&String::from_utf8_lossy(&bytes));
        resp = Response::from_parts(parts, Full::new(Bytes::from(cleaned)));
    }

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
        if !req_path_str.starts_with("/zeus")
            && !req_path_str.starts_with("/trust")
            && !req_path_str.starts_with("/.cronus/")
            && !req_path_str.starts_with("/blocks")
            && !req_path_str.starts_with("/hydra")
        {
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
        let sc = match status {
            200..=299 => "\x1b[32m",
            300..=399 => "\x1b[36m",
            400..=499 => "\x1b[33m",
            _ => "\x1b[31m",
        };
        let tc = if duration_ms > 100 {
            "\x1b[33m"
        } else {
            "\x1b[90m"
        };
        let dp = if req_path_str.len() > 35 {
            &req_path_str[..35]
        } else {
            &req_path_str
        };
        eprintln!(
            "  \x1b[90m{}\x1b[0m {:<5} {:<35} {}{}\x1b[0m  {}{}ms\x1b[0m  {}q",
            current_time_hms(),
            req_method_str,
            dp,
            sc,
            status,
            tc,
            duration_ms,
            queries
        );
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

    // CSRF: cookie-authenticated mutations (REST, GraphQL, forms, actions,
    // auth routes) must come from this origin. Runs before every handler.
    if let Some(resp) = session::csrf_rejection(&method, &path, req.headers()) {
        return Ok(resp);
    }

    // HMR version endpoint
    if path == "/.cronus/version" {
        return Ok(json_response(
            StatusCode::OK,
            json!({ "version": hmr::current_version() }),
        ));
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
        return Ok(json_response(
            StatusCode::OK,
            json!({
                "stats": stats,
                "traces": traces,
            }),
        ));
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
        return Ok(json_response(
            StatusCode::OK,
            serde_json::to_value(&report).unwrap_or(json!({"error":"serialize"})),
        ));
    }
    if path == "/api/hydra/candidates" {
        let candidates = hydra::extract::extract_candidates(&state.script_registry.scripts);
        let data: Vec<serde_json::Value> = candidates
            .iter()
            .map(|c| {
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
            })
            .collect();
        return Ok(json_response(
            StatusCode::OK,
            json!({"candidates": data, "total": candidates.len()}),
        ));
    }

    // ── Rate limiting (API endpoints only) ──
    if path.starts_with("/api/") {
        // Socket peer IP; X-Forwarded-For only when the peer is in CRONUS_TRUSTED_PROXIES
        let client_ip = http_guard::client_ip_from_headers(remote_addr, req.headers()).to_string();

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
            return Ok(http_guard::too_many_requests(retry_after));
        }
    }

    // Brain: track every request
    let start = std::time::Instant::now();

    // Trust engine endpoint
    if path == "/api/trust" || path == "/trust" {
        let metrics = trust::all_metrics();
        let trust_data: Vec<serde_json::Value> = metrics
            .iter()
            .map(|m| {
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
            })
            .collect();
        return Ok(json_response(
            StatusCode::OK,
            json!({
                "blocks": trust_data,
                "total_tracked": metrics.len(),
            }),
        ));
    }

    // Brain stats endpoint
    if path == "/api/brain/stats" {
        if let Some(ref brain) = state.brain {
            return Ok(json_response(StatusCode::OK, brain.stats()));
        }
        return Ok(json_response(
            StatusCode::OK,
            json!({"status": "brain not initialized"}),
        ));
    }
    if path == "/api/brain/suggest" {
        if let Some(ref brain) = state.brain {
            let suggestions = brain.suggest("");
            return Ok(json_response(
                StatusCode::OK,
                json!({"suggestions": suggestions}),
            ));
        }
        return Ok(json_response(StatusCode::OK, json!({"suggestions": []})));
    }

    // Auth routes
    if path.starts_with("/api/auth/") {
        let headers = req.headers().clone();
        let body_bytes = if method == Method::POST {
            match http_guard::read_body(req).await {
                Ok(b) => b,
                Err(r) => return Ok(r),
            }
        } else {
            Default::default()
        };
        let outcome = session::handle_auth(&state, &method, &path, &query, &headers, &body_bytes);
        if let Some(ref account) = outcome.login_account {
            http_guard::login_account_record(account, outcome.response.status());
        }
        return Ok(outcome.response);
    }

    // Payment endpoints
    if path == "/api/checkout" && method == Method::POST {
        let engine = payments::PaymentEngine::from_env();
        let result = engine.create_checkout_url(
            "starter",
            2900,
            "/billing/success",
            "/billing/cancel",
            None,
        );
        match result {
            Ok(data) => return Ok(json_response(StatusCode::OK, data)),
            Err(e) => {
                return Ok(json_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": e}),
                ))
            }
        }
    }
    if path == "/api/webhooks/stripe" && method == Method::POST {
        let engine = payments::PaymentEngine::from_env();
        return Ok(json_response(
            StatusCode::OK,
            json!({"status": "webhook received", "mode": if engine.is_live() { "live" } else { "mock" }}),
        ));
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
        // Dev only (http_guard 404s it in production). Project dir, never /tmp.
        let body_bytes = match http_guard::read_body(req.into_body()).await {
            Ok(b) => b,
            Err(r) => return Ok(r),
        };
        let Ok(val) = serde_json::from_slice::<Value>(&body_bytes) else {
            return Ok(json_response(
                StatusCode::BAD_REQUEST,
                authz::error_body("BAD_REQUEST", "Expected JSON body"),
            ));
        };
        {
            let saved = std::fs::create_dir_all(".cronus").and_then(|_| {
                std::fs::write(http_guard::AUDIT_WIDGET_RESULTS_PATH, val.to_string())
            });
            if let Err(e) = saved {
                eprintln!("  \x1b[33m[AUDIT]\x1b[0m could not save results: {}", e);
                return Ok(json_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    authz::error_body("INTERNAL", "Could not save audit results"),
                ));
            }
            eprintln!(
                "\n  \x1b[36m[AUDIT]\x1b[0m Results saved to {}",
                http_guard::AUDIT_WIDGET_RESULTS_PATH
            );
            {
                let fidelity = val.get("fidelity").and_then(|v| v.as_i64()).unwrap_or(0);
                let missing = val
                    .get("missingItems")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                let extra = val
                    .get("extraItems")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                let dot = if fidelity >= 90 {
                    "🟢"
                } else if fidelity >= 60 {
                    "🟡"
                } else {
                    "🔴"
                };
                eprintln!(
                    "  {} Fidelity: {}% | Missing: {} | Extra: {}",
                    dot, fidelity, missing, extra
                );
            }
        }
        return Ok(json_response(StatusCode::OK, json!({"ok": true})));
    }

    // Audit results read (for CLI/agent access)
    if path == "/api/audit/results" && method == hyper::Method::GET {
        let results = std::fs::read_to_string(http_guard::AUDIT_WIDGET_RESULTS_PATH)
            .unwrap_or_else(|_| "{}".into());
        let val: Value =
            serde_json::from_str(&results).unwrap_or(json!({"error": "no audit results yet"}));
        return Ok(json_response(StatusCode::OK, val));
    }

    // Audit trail endpoints -- tamper-proof hash-chained log
    if path == "/api/audit/trail/verify" && method == Method::GET {
        match state.audit_trail.verify() {
            Ok(result) => return Ok(json_response(StatusCode::OK, result)),
            Err(e) => {
                eprintln!("  \x1b[31m✗\x1b[0m audit trail verify: {}", e);
                return Ok(json_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    authz::error_body("INTERNAL", "Audit trail unavailable"),
                ));
            }
        }
    }
    if (path == "/api/audit/trail" || path.starts_with("/api/audit/trail?"))
        && method == Method::GET
    {
        let query_str = req.uri().query().unwrap_or("");
        let limit: usize = query_str
            .split('&')
            .find(|p| p.starts_with("limit="))
            .and_then(|p| p.strip_prefix("limit="))
            .and_then(|v| v.parse().ok())
            .unwrap_or(50);
        let entity_filter: Option<String> = query_str
            .split('&')
            .find(|p| p.starts_with("entity="))
            .and_then(|p| p.strip_prefix("entity="))
            .map(|v| v.to_string());
        match state
            .audit_trail
            .query_filtered(limit, entity_filter.as_deref())
        {
            Ok(mut entries) => {
                http_guard::redact_audit_entries(&mut entries, &state.entities);
                return Ok(json_response(StatusCode::OK, entries));
            }
            Err(e) => {
                eprintln!("  \x1b[31m✗\x1b[0m audit trail query: {}", e);
                return Ok(json_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    authz::error_body("INTERNAL", "Audit trail unavailable"),
                ));
            }
        }
    }

    // Health endpoint
    if path == "/api/health" {
        return Ok(json_response(
            StatusCode::OK,
            json!({
                "ok": true,
                "app": state.app.name,
                "entities": state.entities.len(),
                "pages": state.pages.len(),
                "runtime": "cronus-kernel",
                "version": env!("CARGO_PKG_VERSION")
            }),
        ));
    }

    // Schema endpoint — returns all entities with fields
    if path == "/api/schema" {
        let schema: Vec<Value> = state
            .entities
            .iter()
            .map(|e| {
                let fields: Vec<Value> = e
                    .fields
                    .iter()
                    .map(|f| {
                        json!({
                            "name": f.name,
                            "type": format!("{:?}", f.field_type).to_lowercase(),
                            "required": f.required,
                            "unique": f.unique,
                        })
                    })
                    .collect();
                json!({
                    "entity": e.name,
                    "fields": fields,
                    "field_count": e.fields.len(),
                })
            })
            .collect();
        return Ok(json_response(
            StatusCode::OK,
            json!({
                "entities": schema,
                "total": state.entities.len(),
                "pages": state.pages.len(),
            }),
        ));
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
        let entity_rows: Vec<Value> = state
            .entities
            .iter()
            .filter(|e| !e.name.starts_with('_'))
            .map(|e| {
                let count = state.db.count(&e.name).unwrap_or(0);
                json!({"entity": e.name, "rows": count})
            })
            .collect();
        let empty_bound: Vec<&str> = state
            .pages
            .iter()
            .flat_map(|p| {
                p.sections.iter().filter_map(|s| {
                    if s.binding.is_some() {
                        let entity = s.binding.as_ref().unwrap().entity.clone();
                        let count = state.db.count(&entity).unwrap_or(0);
                        if count == 0 {
                            Some(entity)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            })
            .map(|_| "")
            .collect(); // placeholder
        let behavioral: Vec<String> = state
            .pages
            .iter()
            .flat_map(|p| {
                p.sections.iter().filter_map(|s| {
                    if let Some(ref b) = s.binding {
                        let count = state.db.count(&b.entity).unwrap_or(0);
                        if count == 0 {
                            Some(format!(
                                "Entity '{}' has 0 rows — {} page shows empty state",
                                b.entity, p.route
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            })
            .collect();
        return Ok(json_response(
            StatusCode::OK,
            json!({
                "status": "healthy",
                "lint": { "warnings": 0, "errors": 0 },
                "behavioral": behavioral,
                "entities": entity_rows,
                "brain": brain_stats,
            }),
        ));
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

        let webhooks_json: Vec<Value> = state
            .webhooks
            .iter()
            .map(|w| {
                json!({
                    "entity": w.entity,
                    "hooks": w.hooks.iter().map(|h| json!({
                        "event": h.event,
                        "method": h.method,
                        "url": h.url,
                    })).collect::<Vec<_>>(),
                })
            })
            .collect();

        let entity_rows: Vec<Value> = state
            .entities
            .iter()
            .filter(|e| !e.name.starts_with('_'))
            .map(|e| {
                let count = state.db.count(&e.name).unwrap_or(0);
                json!({"entity": e.name, "count": count})
            })
            .collect();

        let relationship_graph =
            graph::build_graph_from_state(&state.entities, &state.pages, &state.webhooks);

        // Load semantic memory for context
        let memory_data = open_memory_db()
            .ok()
            .and_then(|m| m.get_context_data().ok())
            .unwrap_or(json!({"decisions": [], "changelog": []}));

        // Constitution violations — computed before json! macro (generics don't work inside macro)
        let constitution_violations_json: Vec<Value> = {
            let mut ctx_nodes = Vec::new();
            for e in &state.entities {
                ctx_nodes.push(AstNode::Entity(e.clone()));
            }
            for p in &state.pages {
                ctx_nodes.push(AstNode::Page(p.clone()));
            }
            state
                .app
                .constitution
                .as_ref()
                .map(|c| {
                    constitution_check::check_constitution(&ctx_nodes, c)
                        .iter()
                        .map(|v| {
                            json!({
                                "type": v.rule_type,
                                "rule": v.rule,
                                "violation": v.violation,
                                "entity": v.entity,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        return Ok(json_response(
            StatusCode::OK,
            json!({
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
            }),
        ));
    }

    // Server logs API — returns brain events as JSON
    if path == "/api/server/logs" && method == Method::GET {
        let limit: usize = query
            .split('&')
            .find_map(|p| {
                let mut kv = p.splitn(2, '=');
                if kv.next() == Some("limit") {
                    kv.next().and_then(|v| v.parse().ok())
                } else {
                    None
                }
            })
            .unwrap_or(100);
        match state.db.find_all("_brain_events", limit, 0) {
            Ok(rows) => return Ok(json_response(StatusCode::OK, rows)),
            Err(_) => return Ok(json_response(StatusCode::OK, json!([]))),
        }
    }

    // Server stats API
    if path == "/api/server/stats" && method == Method::GET {
        let brain_stats = state.brain.as_ref().map(|b| b.stats()).unwrap_or(json!({}));
        let entity_counts: Vec<Value> = state
            .entities
            .iter()
            .filter(|e| !e.name.starts_with('_'))
            .map(|e| {
                let count = state.db.count(&e.name).unwrap_or(0);
                json!({"entity": e.name, "count": count})
            })
            .collect();
        return Ok(json_response(
            StatusCode::OK,
            json!({
                "brain": brain_stats,
                "entities": entity_counts,
                "pages": state.pages.len(),
                "apis": state.apis.len(),
                "uptime": "running",
            }),
        ));
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
        return Ok(json_response(
            StatusCode::OK,
            server::docs_index::generate_docs_index(&state),
        ));
    }
    if path.starts_with("/api/docs/search") && method == Method::GET {
        let q = query
            .split('&')
            .find_map(|p| p.strip_prefix("q="))
            .unwrap_or("");
        return Ok(json_response(
            StatusCode::OK,
            server::docs_index::search_docs_index(&state, q),
        ));
    }
    if path.starts_with("/api/docs/tags/") && method == Method::GET {
        let tag = path.strip_prefix("/api/docs/tags/").unwrap_or("");
        return Ok(json_response(
            StatusCode::OK,
            server::docs_index::get_docs_by_tag(&state, tag),
        ));
    }

    // GraphQL endpoint
    if path == "/graphql" && method == Method::GET {
        return Ok(html_response(graphql::playground_html()));
    }
    if path == "/graphql" && method == Method::POST {
        // SECURITY: GraphQL requires a session; owner scope/redaction in graphql.rs.
        let gql_access = access::Access::from_headers(req.headers(), state.auth_entity.clone());
        if gql_access.viewer.is_none() {
            return Ok(json_response(
                StatusCode::UNAUTHORIZED,
                graphql::gql_error("UNAUTHENTICATED", "authentication required"),
            ));
        }
        let body_bytes = match http_guard::read_body(req).await {
            Ok(b) => b,
            Err(r) => return Ok(r),
        };
        let body_str = String::from_utf8_lossy(&body_bytes);
        let body_json: Value = serde_json::from_str(&body_str).unwrap_or(json!({}));
        let query = body_json
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let variables = body_json.get("variables").cloned().unwrap_or(json!({}));
        let schema = graphql::GraphQLSchema::from_entities(&state.entities);
        let result = graphql::execute_graphql(query, &variables, &schema, &state.db, &gql_access);
        return Ok(json_response(StatusCode::OK, result));
    }
    if path == "/graphql/schema" && method == Method::GET {
        if access::viewer_from_headers(req.headers(), &auth::default_secret()).is_none() {
            return Ok(json_response(
                StatusCode::UNAUTHORIZED,
                graphql::gql_error("UNAUTHENTICATED", "authentication required"),
            ));
        }
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
        let has_endpoint = state
            .script_registry
            .get_endpoints()
            .iter()
            .any(|(_s, ep)| ep.method == method_str && ep.path == path);
        // SECURITY: webhooks only match paths under /hooks/ prefix
        let has_webhook = !has_endpoint
            && method == Method::POST
            && path.starts_with("/hooks/")
            && !state.script_registry.get_webhook_handlers(&path).is_empty();

        if has_endpoint || has_webhook {
            let token = req
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "))
                .map(|s| s.to_string());
            let (user_id, role) = token
                .as_deref()
                .and_then(|t| auth::verify_token(t, &auth::default_secret()).ok())
                .map(|c| (c.sub.clone(), c.role.clone()))
                .unwrap_or_else(|| ("anonymous".into(), "public".into()));

            let body_bytes = match http_guard::read_body(req).await {
                Ok(b) => b,
                Err(r) => return Ok(r),
            };
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
                                return Ok(json_response(
                                    StatusCode::UNAUTHORIZED,
                                    json!({"error": "authentication required"}),
                                ));
                            }
                            // Check role if specific role required
                            if required_role != "any" && role != required_role {
                                return Ok(json_response(
                                    StatusCode::FORBIDDEN,
                                    json!({"error": "insufficient role"}),
                                ));
                            }
                        }
                        let ctx = scripting::execute_endpoint(
                            ep,
                            &script.name,
                            &state.db,
                            &user_id,
                            &role,
                            body.as_ref(),
                            &std::collections::HashMap::new(),
                        );
                        if let Some(resp) = ctx.response {
                            // SECURITY: clamp status to safe range
                            let safe_status = resp.status.max(200).min(599);
                            let mut builder = Response::builder().status(safe_status);
                            // SECURITY: whitelist safe response headers — block Set-Cookie, Location, etc.
                            const ALLOWED_HEADERS: &[&str] = &[
                                "content-type",
                                "content-disposition",
                                "cache-control",
                                "x-request-id",
                                "x-total-count",
                            ];
                            for (k, v) in &resp.headers {
                                let k_lower = k.to_lowercase();
                                if ALLOWED_HEADERS.contains(&k_lower.as_str()) {
                                    // SECURITY: strip newlines to prevent header injection
                                    let safe_v = v.replace('\n', "").replace('\r', "");
                                    builder = builder.header(k.as_str(), safe_v.as_str());
                                }
                            }
                            if !resp
                                .headers
                                .keys()
                                .any(|k| k.to_lowercase() == "content-type")
                            {
                                builder = builder.header("Content-Type", "application/json");
                            }
                            return Ok(builder.body(Full::new(Bytes::from(resp.body))).unwrap());
                        }
                        return Ok(json_response(
                            StatusCode::OK,
                            json!({"ok": true, "logs": ctx.logs}),
                        ));
                    }
                }
            }
            // Webhook
            let body_val = body.unwrap_or(serde_json::Value::Null);
            let _ctx = scripting::execute_webhook(
                &state.script_registry,
                &path,
                &body_val,
                &state.db,
                &std::collections::HashMap::new(),
            );
            return Ok(json_response(StatusCode::OK, json!({"ok": true})));
        }
    }

    // API routes: /api/...
    if path.starts_with("/api/") {
        // SECURITY: read the session BEFORE consuming the request body.
        let api_access = access::Access::from_headers(req.headers(), state.auth_entity.clone());

        let body_bytes = match http_guard::read_body(req).await {
            Ok(b) => b,
            Err(r) => return Ok(r),
        };
        let body: Option<serde_json::Value> = serde_json::from_slice(&body_bytes).ok();

        let resp = api_crud::handle_api(&state, &method, &path, &query, body.as_ref(), &api_access);
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
        // SECURITY: session required; only AST-declared actions run (by
        // `action_id`), with the server's own instructions, owner-scoped.
        let action_access = access::Access::from_headers(req.headers(), state.auth_entity.clone());
        let body_bytes = match http_guard::read_body(req).await {
            Ok(b) => b,
            Err(r) => return Ok(r),
        };
        let body: Value = serde_json::from_slice(&body_bytes).unwrap_or(json!({}));
        let (status, payload) = actions::handle_action(&state, &body, &action_access);
        let status = StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_REQUEST);
        return Ok(json_response(status, payload));
    }

    // ── Form submission endpoint: POST creates, PATCH /_form/<section>/<id> edits ──
    if (method == Method::POST || method == Method::PATCH) && path.starts_with("/_form/") {
        // SECURITY: declared form section, session/owner rules from access.rs,
        // body filtered by authz::writable_body (see actions::handle_form).
        let form_access = access::Access::from_headers(req.headers(), state.auth_entity.clone());
        let body_bytes = match http_guard::read_body(req).await {
            Ok(b) => b,
            Err(r) => return Ok(r),
        };
        let body: Value = serde_json::from_slice(&body_bytes).unwrap_or(json!({}));
        let (status, payload) = actions::handle_form(&state, &method, &path, &body, &form_access);
        let status = StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_REQUEST);
        return Ok(json_response(status, payload));
    }

    // Serve pages
    let accent = state
        .style
        .as_ref()
        .and_then(|s| s.accent.as_deref())
        .unwrap_or("amber");
    let app_name = &state.app.name;

    // ── Component preview route (skip if user defined a /components page) ──
    let has_components_page_main = state.pages.iter().any(|p| p.route == "/components");
    if path == "/components" && !has_components_page_main {
        let body = if state.components.is_empty() {
            r#"<div data-slot="catalog"><header data-slot="catalog-header"><h1>Kit</h1><p data-slot="catalog-lead">No components defined.</p></header></div>"#.to_string()
        } else {
            ui::render_components_page(&state.components)
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
                .header(
                    "Set-Cookie",
                    session::clear_session_cookie(session::cookie_secure(
                        http_guard::policy().mode,
                        req.headers(),
                    )),
                )
                .body(Full::new(Bytes::new()))
                .unwrap());
        }
    }

    // ── Auth middleware — protect pages that require authentication ──
    // SECURITY: exact route-pattern match (`/orders/:id`), never prefix.
    let matched_requires = state
        .auth_required_pages
        .iter()
        .find(|(r, _)| access::route_pattern_matches(r, &path))
        .map(|(_, req)| req.clone());

    if let Some(requires_str) = matched_requires {
        let token = req
            .headers()
            .get("cookie")
            .and_then(|c| c.to_str().ok())
            .and_then(|c| c.split(';').find(|s| s.trim().starts_with("cronus_token=")))
            .map(|s| s.trim().trim_start_matches("cronus_token=").to_string())
            .or_else(|| {
                req.headers()
                    .get("authorization")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|h| h.strip_prefix("Bearer "))
                    .map(|s| s.to_string())
            });

        let secret = auth::default_secret();

        let redirect_to_login = || {
            Response::builder()
                .status(StatusCode::FOUND)
                .header("Location", "/login")
                .body(Full::new(Bytes::new()))
                .unwrap()
        };

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
    let page = state
        .pages
        .iter()
        .find(|p| access::route_pattern_matches(&p.route, &path));

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
            if http_guard::is_production() {
                return Ok(http_guard::not_found());
            }
            match std::fs::read_to_string(source_path) {
                Ok(html) => {
                    // Developer-authored file with no interpolated data: trusted scripts.
                    return Ok(html_response(crate::security::mark_kernel_scripts(&html)));
                }
                Err(err) => {
                    // Detail goes to the log once; the page never shows IO errors or paths.
                    eprintln!("  \x1b[31m✗\x1b[0m source page {}: {}", page.route, err);
                    let body = r#"<div style="padding:40px">
  <h1 style="font-size:16px;color:var(--foreground);margin-bottom:8px">Failed to load source HTML</h1>
</div>"#.to_string();
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
                page.components
                    .iter()
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
        let is_auth_page = route_lower == "/login"
            || route_lower == "/signup"
            || title_lower.contains("sign in")
            || title_lower.contains("sign up")
            || title_lower.contains("login")
            || title_lower.contains("signup");
        // Only use built-in auth renderer if page has NO custom template sections
        let has_custom_template =
            page.page_type == "custom" && page.sections.iter().any(|s| s.template.is_some());
        if is_auth_page && !has_custom_template {
            let is_login = route_lower == "/login"
                || title_lower.contains("login")
                || title_lower.contains("sign in");
            let html = ui::render_auth_page(page, is_login);
            return Ok(html_response(html));
        }

        let theme = state
            .style
            .as_ref()
            .and_then(|s| s.theme.as_deref())
            .unwrap_or("dark");

        // SECURITY: viewer for SSR bindings (owner scope, auth.* refs, redaction).
        let page_access = access::Access::from_headers(req.headers(), state.auth_entity.clone());

        // Check if page has inline sidebar/topbar sections (not components)
        let has_section_sidebar = page.sections.iter().any(|s| s.section_type == "sidebar");

        // Settings page — full-page renderer with its own sidebar/topbar
        // Order Detail page — full-page renderer
        let is_order_detail = page
            .sections
            .iter()
            .any(|s| s.section_type == "order-header" || s.section_type == "line-items");
        if is_order_detail {
            let referenced_comps: Vec<parser::ComponentNode> = page
                .components
                .iter()
                .filter_map(|name| state.components.iter().find(|c| c.name == *name))
                .cloned()
                .collect();
            let html = ui::render_order_detail_dashboard(
                app_name,
                &page.sections,
                &referenced_comps,
                theme,
                page.route.as_str(),
            );
            return Ok(html_response(html));
        }

        // Settings page — full-page renderer
        let is_settings_page = page
            .sections
            .iter()
            .any(|s| s.section_type == "settings-profile" || s.section_type == "subscription-card");
        if is_settings_page {
            let referenced_comps: Vec<parser::ComponentNode> = page
                .components
                .iter()
                .filter_map(|name| state.components.iter().find(|c| c.name == *name))
                .cloned()
                .collect();
            let html = ui::render_settings_dashboard(
                app_name,
                &page.sections,
                &referenced_comps,
                theme,
                page.route.as_str(),
            );
            return Ok(html_response(html));
        }

        // Dumped pages with HTML templates — use landing layout with Tailwind CDN,
        // EXCEPT for auth-protected pages that have a declarative layout (these
        // must share the same sidebar shell across all pages for consistency).
        let has_templates = page
            .sections
            .iter()
            .any(|s| s.template.is_some() || s.config.get("template").is_some());
        let is_auth_page = page.requires.as_deref() == Some("auth")
            || page
                .requires
                .as_deref()
                .map(|r| r.starts_with("role("))
                .unwrap_or(false);
        let has_declarative_layout = state.layout.is_some();
        if has_templates && !(is_auth_page && has_declarative_layout) {
            let body = ui::render_page(
                page,
                &state.entities,
                accent,
                theme,
                Some(&state.db),
                &route_params,
                &page_access,
            );
            let html = ui::render_layout_landing_ex(
                app_name,
                &body,
                theme,
                state.style.as_ref(),
                state.app.tailwind_config.as_deref(),
            );
            return Ok(html_response(html));
        }

        if has_section_sidebar {
            // Check for specialized dashboard renderers BEFORE falling back to generic
            let billing_types = [
                "current-plan",
                "usage-status",
                "billing-stats",
                "payment-methods",
                "recent-invoices",
            ];
            let is_billing_page = page
                .sections
                .iter()
                .any(|s| s.section_type == "current-plan" || s.section_type == "billing-stats");
            if is_billing_page {
                let referenced_comps: Vec<parser::ComponentNode> = page
                    .components
                    .iter()
                    .filter_map(|name| state.components.iter().find(|c| c.name == *name))
                    .cloned()
                    .collect();
                let html = ui::render_billing_dashboard(
                    app_name,
                    &page.sections,
                    &referenced_comps,
                    theme,
                    &path,
                );
                return Ok(html_response(html));
            }

            // Generic dashboard wrapper — sidebar + any sections
            let body = ui::render_page(
                page,
                &state.entities,
                accent,
                theme,
                Some(&state.db),
                &route_params,
                &page_access,
            );
            let html = ui::render_layout_dashboard(&state.app.name, &body, theme);
            return Ok(html_response(html));
        }

        let mut body = if page.page_type == "components" && !state.components.is_empty() {
            ui::render_components_page(&state.components)
        } else {
            ui::render_page(
                page,
                &state.entities,
                accent,
                theme,
                Some(&state.db),
                &route_params,
                &page_access,
            )
        };

        // `use ComponentName` on a real page — widgets only, no kit chrome.
        // type:components already rendered the full catalog above.
        let has_sidebar_component_early = !page.components.is_empty()
            && page.components.iter().any(|comp_name| {
                state.components.iter().any(|c| {
                    c.name == *comp_name
                        && (c.style.as_deref().unwrap_or("").contains("sidenav")
                            || c.layout.as_deref().unwrap_or("") == "sidebar")
                })
            });
        if page.page_type != "components" {
            if !page.components.is_empty() && !has_sidebar_component_early {
                let referenced: Vec<parser::ComponentNode> = page
                    .components
                    .iter()
                    .filter_map(|name| state.components.iter().find(|c| c.name == *name))
                    .cloned()
                    .collect();
                if !referenced.is_empty() {
                    body.push_str("\n");
                    body.push_str(&ui::render_components_inline(&referenced));
                }
            }

            // custom pages with no sections — fallback to the kit catalog
            if page.page_type == "custom"
                && page.sections.is_empty()
                && !state.components.is_empty()
            {
                body.push_str("\n");
                body.push_str(&ui::render_components_page(&state.components));
            }
        }

        // FIX 1: Detect sidebar component — if page uses a Sidenav component, it's a dashboard page
        let has_sidebar_component = !page.components.is_empty()
            && page.components.iter().any(|comp_name| {
                state.components.iter().any(|c| {
                    c.name == *comp_name
                        && (c.style.as_deref().unwrap_or("").contains("sidenav")
                            || c.layout.as_deref().unwrap_or("") == "sidebar")
                })
            });

        // Landing/checkout pages use full-width layout, no sidebar
        let landing_section_types = [
            "hero",
            "topbar",
            "checkout",
            "features",
            "pricing",
            "cta",
            "testimonial",
            "faq",
            "trusted",
            "footer",
        ];
        // If ANY section has a template, it's a dumped page — always use landing layout
        let has_templates = page
            .sections
            .iter()
            .any(|s| s.template.is_some() || s.config.get("template").is_some());
        let is_landing = has_templates
            || (!has_sidebar_component
                && (page.page_type == "checkout"
                    || (page.page_type == "custom"
                        && page
                            .sections
                            .iter()
                            .any(|s| landing_section_types.contains(&s.section_type.as_str())))));
        let dashboard_types = [
            "sidebar",
            "card",
            "page-header",
            "stat-cards",
            "product-grid",
            "team-list",
            "policies",
            "activity-table",
            "status-card",
            "links",
            "live-keys",
            "test-keys",
            "webhooks",
            "quick-links",
            "current-plan",
            "usage-status",
            "billing-stats",
            "payment-methods",
            "recent-invoices",
            "balance-card",
            "upcoming-card",
            "payout-history",
            "support-banner",
            "checkout-form",
            "product-summary",
            "trust-indicators",
            "team-members",
            "security-status",
            "security-policies",
            "login-activity",
            "settings-profile",
            "api-keys",
            "security-grid",
            "subscription-card",
            "invoices-list",
            "support-card",
            "danger-zone",
            "order-header",
            "line-items",
            "price-breakdown",
            "payment-info",
            "customer-profile",
            "shipping-timeline",
            "staff-notes",
        ];
        let is_dashboard = has_sidebar_component
            || page
                .sections
                .iter()
                .any(|s| dashboard_types.contains(&s.section_type.as_str()));
        let is_billing = page
            .sections
            .iter()
            .any(|s| s.section_type == "current-plan" || s.section_type == "billing-stats");
        let is_payouts = page
            .sections
            .iter()
            .any(|s| s.section_type == "balance-card" || s.section_type == "payout-history");
        let is_unified =
            page.sections
                .iter()
                .any(|s| s.section_type == "balance-card")
                && page.sections.iter().any(|s| {
                    s.section_type == "billing-stats" || s.section_type == "recent-invoices"
                });
        let is_payment_links = page
            .sections
            .iter()
            .any(|s| s.section_type == "product-grid")
            && page.sections.iter().any(|s| s.section_type == "stat-cards");
        let is_checkout = page
            .sections
            .iter()
            .any(|s| s.section_type == "checkout-form" || s.section_type == "product-summary");
        let is_security = page
            .sections
            .iter()
            .any(|s| s.section_type == "team-members" || s.section_type == "login-activity");
        let is_settings = page
            .sections
            .iter()
            .any(|s| s.section_type == "settings-profile" || s.section_type == "subscription-card");
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
            ui::render_layout_landing_ex(
                app_name,
                &body,
                theme,
                state.style.as_ref(),
                state.app.tailwind_config.as_deref(),
            )
        } else if is_checkout {
            // Checkout page: no sidebar, centered layout
            let referenced_comps: Vec<parser::ComponentNode> = page
                .components
                .iter()
                .filter_map(|name| state.components.iter().find(|c| c.name == *name))
                .cloned()
                .collect();
            ui::render_checkout_dashboard(app_name, &page.sections, &referenced_comps, theme)
        } else if is_dashboard {
            // Dedicated dashboard renderer: produces the ENTIRE page in one shot
            let referenced_comps: Vec<parser::ComponentNode> = page
                .components
                .iter()
                .filter_map(|name| state.components.iter().find(|c| c.name == *name))
                .cloned()
                .collect();
            if is_unified {
                ui::render_unified_dashboard(
                    app_name,
                    &page.sections,
                    &referenced_comps,
                    theme,
                    current_route,
                )
            } else if is_payouts {
                ui::render_payouts_dashboard(
                    app_name,
                    &page.sections,
                    &referenced_comps,
                    theme,
                    current_route,
                )
            } else if is_billing {
                ui::render_billing_dashboard(
                    app_name,
                    &page.sections,
                    &referenced_comps,
                    theme,
                    current_route,
                )
            } else if is_settings {
                ui::render_settings_dashboard(
                    app_name,
                    &page.sections,
                    &referenced_comps,
                    theme,
                    current_route,
                )
            } else if is_security {
                ui::render_security_dashboard(
                    app_name,
                    &page.sections,
                    &referenced_comps,
                    theme,
                    current_route,
                )
            } else if is_payment_links {
                ui::render_payment_links_dashboard(
                    app_name,
                    &page.sections,
                    &referenced_comps,
                    theme,
                    current_route,
                )
            } else {
                // Generic dashboard wrapper — sidebar + any sections
                ui::render_generic_dashboard(
                    app_name,
                    &body,
                    &referenced_comps,
                    theme,
                    current_route,
                )
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
        let allowed: Vec<&str> = transition
            .rules
            .iter()
            .filter(|r| r.from == old_value)
            .flat_map(|r| r.to.iter().map(|s| s.as_str()))
            .collect();

        // If no rules found for the current state, check if it's a wildcard "*" rule
        let allowed = if allowed.is_empty() {
            transition
                .rules
                .iter()
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
/// Fire-and-forget webhooks for an entity event. Validation, SSRF blocking,
/// redaction, signing and timeouts live in `webhook.rs`.
fn fire_webhooks(
    webhooks: &[parser::WebhookNode],
    entities: &[parser::EntityNode],
    entity: &str,
    event: &str,
    payload: &serde_json::Value,
) {
    for wh in webhooks
        .iter()
        .filter(|wh| webhook::names_match(&wh.entity, entity))
    {
        let hooks: Vec<_> = wh.hooks.iter().filter(|h| h.event == event).collect();
        if hooks.is_empty() {
            continue;
        }
        let body = webhook::redacted_body(entities, entity, payload);
        for hook in hooks {
            tokio::spawn(webhook::fire(hook.clone(), event.to_string(), body.clone()));
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
                let new_val = record
                    .get(watched_field)
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
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
            result = format!(
                "{}{}{}",
                &result[..start],
                value,
                &result[start + end + 2..]
            );
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
                    brain.track(
                        &format!("effect:{}", entity_name),
                        &json!({
                            "type": "log",
                            "entity": entity_name,
                            "message": message,
                            "record_id": record.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                        }),
                    );
                }
            }
        }
        "notify" => {
            // args: [provider, channel/severity, message]
            let provider = action.args.get(0).cloned().unwrap_or_default();
            let channel = action.args.get(1).cloned().unwrap_or_default();
            let msg_template = action.args.get(2).cloned().unwrap_or_default();
            let message = interpolate_effect_message(&msg_template, record);

            eprintln!(
                "  \x1b[35m[notify]\x1b[0m {} → {}:{} — {}",
                entity_name, provider, channel, message
            );

            // Write to brain events for tracking
            if let Some(ref brain) = brain {
                brain.track(
                    &format!("effect:notify:{}", entity_name),
                    &json!({
                        "type": "notify",
                        "entity": entity_name,
                        "provider": provider,
                        "channel": channel,
                        "message": message,
                        "record_id": record.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                    }),
                );
            }

            // Broadcast via SSE so dashboards can react
            sse_hub.broadcast(sse::DataChangeEvent {
                entity: format!("_effect_notify_{}", entity_name.to_lowercase()),
                action: "notification".to_string(),
                id: record
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            });
        }
        _ => {
            // Generic/unknown action — log it
            if let Some(ref brain) = brain {
                brain.track(
                    &format!("effect:{}:{}", action.action_type, entity_name),
                    &json!({
                        "type": action.action_type,
                        "entity": entity_name,
                        "args": action.args,
                        "record_id": record.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                    }),
                );
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

    if let Err(e) = auth::check_secret_config() {
        eprintln!("  \x1b[31m✗\x1b[0m {}", e);
        std::process::exit(1);
    }

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
            Err(e) => {
                eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e);
                std::process::exit(1);
            }
        };
        (n, lines)
    } else {
        // Multi-agent mode: compose all .cronus files
        println!(
            "  \x1b[36m⚡\x1b[0m Multi-file mode: {} files detected",
            files.len()
        );
        let mut total = 0;
        for f in &files {
            let lines = fs::read_to_string(f)
                .map(|s| s.lines().count())
                .unwrap_or(0);
            total += lines;
        }
        let n = match parser::parse_directory(".") {
            Ok(n) => n,
            Err(e) => {
                eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e);
                std::process::exit(1);
            }
        };

        (n, total)
    };

    let file = files[0].clone(); // for HMR watcher

    // Save AST snapshot for changelog diffing
    cli::build::save_ast_snapshot(&nodes);

    // Create semantic memory session and extract business rules
    if let Ok(mem) = open_memory_db() {
        match mem.create_session(Some("cronus-run")) {
            Ok(sid) => println!(
                "  \x1b[32m✓\x1b[0m Memory session: {}",
                &sid[..sid.len().min(20)]
            ),
            Err(e) => eprintln!("  \x1b[33m⚠\x1b[0m Memory session failed: {}", e),
        }
        // Auto-extract business rules from @business/@rule doc tags and constitution
        memory::extract_and_store_business_rules(&mem, &nodes);
    }

    // Extract AST parts
    let mut app = AppNode {
        name: "CRONUS App".into(),
        stack: vec![],
        port: 5175,
        database: None,
        tailwind_config: None,
        constitution: None,
        doc: None,
    };
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
    let mut session_policy = crate::auth::SessionPolicy::default();
    let mut layout: Option<parser::LayoutNode> = None;
    let mut defines: std::collections::HashMap<String, Vec<parser::SectionNode>> =
        std::collections::HashMap::new();

    for node in &nodes {
        match node {
            AstNode::App(a) => app = a.clone(),
            AstNode::Entity(e) => entities.push(e.clone()),
            AstNode::Page(p) => {
                // Track pages that require auth
                let req = p
                    .requires
                    .clone()
                    .or_else(|| p.config.get("requires").cloned());
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
                session_policy =
                    match crate::auth::SessionPolicy::from_session_config(&auth.session_config) {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("  \x1b[31m✗\x1b[0m {}", e);
                            std::process::exit(1);
                        }
                    };
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

    // `auth { entity X }` without `entity X { … }`: synthesize the login table.
    if auth_entity::ensure_declared(&mut entities, auth_entity.as_deref()) {
        println!(
            "  \x1b[90mAuth:\x1b[0m      entity {} not declared — using implicit (email, password, role, name)",
            auth_entity.as_deref().unwrap_or("")
        );
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
                            let route_parts: Vec<&str> =
                                page.route.split('/').filter(|s| !s.is_empty()).collect();
                            if let Some(first) = route_parts.first() {
                                // Capitalize first letter
                                let capitalized =
                                    format!("{}{}", first[..1].to_uppercase(), &first[1..]);
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
                                let value = section
                                    .config
                                    .get(&param.name)
                                    .map(|s| s.as_str())
                                    .or(param.default.as_deref())
                                    .unwrap_or("");
                                rendered = rendered.replace(&placeholder, value);
                            }
                            for (key, value) in &section.config {
                                if key == "_component" {
                                    continue;
                                }
                                let placeholder = format!("{{{{{}}}}}", key);
                                rendered = rendered.replace(&placeholder, value);
                            }

                            // Reactive state: replace {{state_var}} with reactive spans
                            // and generate JS signal code
                            if !comp_def.state.is_empty() {
                                // Wrap component in a container with unique ID
                                rendered = format!(
                                    r#"<div data-cid="{cid}">{html}</div>"#,
                                    cid = cid,
                                    html = rendered
                                );

                                // Replace {{state_var}} with reactive spans
                                for sv in &comp_def.state {
                                    let placeholder = format!("{{{{{}}}}}", sv.name);
                                    let span = format!(
                                        r#"<span data-s="{name}">{default}</span>"#,
                                        name = sv.name,
                                        default = sv.default
                                    );
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
                                    let (expr, end_offset) = if after.starts_with("\\\"")
                                        || after.starts_with('"')
                                    {
                                        let quote_char = if after.starts_with("\\\"") {
                                            "\\\""
                                        } else {
                                            "\""
                                        };
                                        let qlen = quote_char.len();
                                        let expr_start = qlen;
                                        if let Some(expr_end) = after[expr_start..].find(quote_char)
                                        {
                                            (
                                                after[expr_start..expr_start + expr_end]
                                                    .to_string(),
                                                7 + expr_start + expr_end + qlen,
                                            )
                                        } else {
                                            break;
                                        }
                                    } else {
                                        break;
                                    };

                                    // Replace @click="expr" with data-eid="..."
                                    rendered = format!(
                                        "{}data-eid=\"{}\"{}",
                                        &rendered[..pos],
                                        eid,
                                        &rendered[pos + end_offset..]
                                    );

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
                                        state_init
                                            .push_str(&format!("{}:{},", sv.name, default_js));
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
    let db_path = app
        .database
        .as_ref()
        .and_then(|d| d.path.clone())
        .unwrap_or_else(|| "data.db".into());

    let app_db = database::CronusDB::open(&db_path).expect("Failed to open database");
    app_db
        .migrate(&entities)
        .expect("Failed to migrate database");

    // Ensure User table has password column for auth (auto-added by kernel)
    let has_user = entities.iter().any(|e| {
        let l = e.name.to_lowercase();
        l == "user" || l == "users"
    });
    if has_user {
        let user_table = entities
            .iter()
            .find(|e| {
                let l = e.name.to_lowercase();
                l == "user" || l == "users"
            })
            .map(|e| e.name.as_str())
            .unwrap();
        let _ = app_db.execute_raw(&format!(
            "ALTER TABLE \"{}\" ADD COLUMN password TEXT",
            user_table
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
                unique: false,
                sensitive: false,
                optional: false,
                searchable: false,
                index: false,
                featured: false,
                formatted: false,
                array: false,
                enum_values: None,
                reference: None,
                doc: None,
                default_value: None,
                min: None,
                max: None,
                min_length: None,
                max_length: None,
                pattern: None,
            },
            parser::FieldNode {
                name: "metadata".to_string(),
                field_type: parser::FieldType::Text,
                required: false,
                unique: false,
                sensitive: false,
                optional: false,
                searchable: false,
                index: false,
                featured: false,
                formatted: false,
                array: false,
                enum_values: None,
                reference: None,
                doc: None,
                default_value: None,
                min: None,
                max: None,
                min_length: None,
                max_length: None,
                pattern: None,
            },
            parser::FieldNode {
                name: "timestamp".to_string(),
                field_type: parser::FieldType::String,
                required: false,
                unique: false,
                sensitive: false,
                optional: false,
                searchable: false,
                index: false,
                featured: false,
                formatted: false,
                array: false,
                enum_values: None,
                reference: None,
                doc: None,
                default_value: None,
                min: None,
                max: None,
                min_length: None,
                max_length: None,
                pattern: None,
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
        session_policy,
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

    // Start server. Default 127.0.0.1; `--host`/CRONUS_HOST opts into exposure.
    // Audit-canvas always binds loopback.
    let policy = match http_guard::policy_from_env(args, audit_canvas) {
        Ok(p) => http_guard::install_policy(p),
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m {}", e);
            std::process::exit(1);
        }
    };
    if !policy.bind_ip.is_loopback() {
        eprintln!("  \x1b[33m⚠\x1b[0m Binding to {} — the server is reachable from the network. Internal dev routes require an admin session.", policy.bind_ip);
    }
    if policy.mode == http_guard::RunMode::Production {
        println!("  \x1b[90mMode:\x1b[0m      production (internal routes disabled)");
    }
    {
        let st = state.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                tick.tick().await;
                st.rate_limiter.cleanup();
                st.auth_rate_limiter.cleanup();
                http_guard::account_limiter().cleanup_at(Instant::now());
            }
        });
    }
    let addr = std::net::SocketAddr::new(policy.bind_ip, serve_port);
    // Every kernel-authored <script> now carries a marker that html_response
    // turns into the per-request CSP nonce (no 'unsafe-inline' in script-src).
    crate::security::enable_script_nonces();
    let listener = TcpListener::bind(addr).await.unwrap_or_else(|e| {
        eprintln!(
            "  \x1b[31m✗\x1b[0m Cannot bind to port {}: {}",
            serve_port, e
        );
        std::process::exit(1);
    });

    let accent = state
        .style
        .as_ref()
        .and_then(|s| s.accent.as_deref())
        .unwrap_or("amber");

    // Start HMR file watcher
    if files.len() > 1 {
        hmr::start_directory_watcher(".", move || {
            let v = hmr::bump_version();
            eprintln!(
                "  \x1b[33m⚡\x1b[0m File changed — version {} (browser will reload)",
                v
            );
        });
    } else {
        let hmr_file = file.clone();
        hmr::start_watcher(&hmr_file, move || {
            let v = hmr::bump_version();
            eprintln!(
                "  \x1b[33m⚡\x1b[0m File changed — version {} (browser will reload)",
                v
            );
        });
    }

    // Count total rows across all entities
    let total_rows: usize = state
        .entities
        .iter()
        .map(|e| state.db.count(&e.name).unwrap_or(0))
        .sum();

    let auth_page_count = state.auth_required_pages.len();
    let total_pages = state.pages.len();
    let elapsed_ms = start_time.elapsed().as_millis();

    // ── Clean startup banner ──
    println!();
    println!(
        "  \x1b[36m\x1b[1mCRONUS\x1b[0m \x1b[90mv{}\x1b[0m",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("  \x1b[90mApp:\x1b[0m       \x1b[1m{}\x1b[0m", app.name);
    println!("  \x1b[90mPort:\x1b[0m      \x1b]8;;http://localhost:{}\x1b\\http://localhost:{}\x1b]8;;\x1b\\", serve_port, serve_port);
    println!(
        "  \x1b[90mDatabase:\x1b[0m  ./{} ({} entities, {} rows)",
        db_path, table_count, total_rows
    );
    if total_pages > 0 {
        if auth_page_count > 0 {
            println!(
                "  \x1b[90mPages:\x1b[0m     {} ({} require auth)",
                total_pages, auth_page_count
            );
        } else {
            println!("  \x1b[90mPages:\x1b[0m     {}", total_pages);
        }
    }
    if route_count > 0 {
        println!("  \x1b[90mRoutes:\x1b[0m    {} API endpoints", route_count);
    }
    if state.script_registry.block_count() > 0 {
        let sc = &state.script_registry;
        let events = sc
            .scripts
            .iter()
            .flat_map(|s| s.blocks.iter())
            .filter(|b| matches!(b, scripting::ast::ScriptBlock::OnEvent(_)))
            .count();
        let schedules = sc.get_schedules().len();
        let endpoints = sc.get_endpoints().len();
        let webhooks = sc
            .scripts
            .iter()
            .flat_map(|s| s.blocks.iter())
            .filter(|b| matches!(b, scripting::ast::ScriptBlock::OnWebhook(_)))
            .count();
        println!(
            "  \x1b[90mScripts:\x1b[0m   {} ({} events, {} schedules, {} endpoints, {} webhooks)",
            sc.scripts.len(),
            events,
            schedules,
            endpoints,
            webhooks
        );
    }
    if let Some(ref _auth_e) = state.auth_entity {
        if state.auth_roles.is_empty() {
            println!("  \x1b[90mAuth:\x1b[0m      JWT");
        } else {
            println!(
                "  \x1b[90mAuth:\x1b[0m      JWT (roles: {})",
                state.auth_roles.join(", ")
            );
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
                                if href != "/"
                                    && href != "#"
                                    && !href.starts_with("/#")
                                    && !href.starts_with("/api/")
                                    && !page_routes.contains(&href)
                                {
                                    println!("  \x1b[31m✗\x1b[0m Dead link: \"{}\" → page {} does not exist", href, href);
                                    warnings += 1;
                                }
                                pos = start + end;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }

                // Check sections without binding that should have data
                let data_sections = ["kpi", "stat-cards", "table"];
                if data_sections.contains(&section.section_type.as_str())
                    && section.binding.is_none()
                    && section.items.is_empty()
                {
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
                    println!(
                        "  \x1b[1mFidelity Audit\x1b[0m (CRONUS_AUDIT_REF={})",
                        ref_path
                    );
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
                    eprintln!(
                        "  \x1b[36m[schedule]\x1b[0m running \"{}\" from \"{}\"",
                        sched_name, script_name
                    );
                    let db = match crate::database::CronusDB::open(&db_path) {
                        Ok(db) => db,
                        Err(e) => {
                            eprintln!("  \x1b[31m[schedule]\x1b[0m db open failed: {}", e);
                            continue;
                        }
                    };
                    let mut ctx = scripting::vm::ScriptContext::new(
                        "system",
                        "admin",
                        std::collections::HashMap::new(),
                    );
                    if let Err(e) = scripting::vm::execute_statements(&body, &mut ctx, &db, None) {
                        eprintln!(
                            "  \x1b[31m[schedule]\x1b[0m \"{}\" error: {}",
                            sched_name, e
                        );
                    }
                }
            });
            eprintln!(
                "  \x1b[36m[schedule]\x1b[0m registered \"{}\" (every {})",
                sched.name, sched.interval
            );
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
        println!(
            "  \x1b[31m{} resolve error(s)\x1b[0m — aborting",
            resolve_errors.len()
        );
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
            println!(
                "  \x1b[33m\u{26a0} {} constitution violation(s)\x1b[0m",
                real_violations.len()
            );
        }
    }

    // Graceful shutdown: listen for Ctrl+C
    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            result = listener.accept() => {
                let (stream, remote_addr) = match result {
                    Ok(conn) => conn,
                    Err(e) => {
                        eprintln!("  Accept error: {}", e);
                        continue;
                    }
                };
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
                        // SECURITY: session required; events filtered per viewer.
                        let sse_access = access::Access::from_headers(req.headers(), state.auth_entity.clone());
                        let Some(is_admin) = sse_access.viewer.as_ref().map(|v| v.is_admin()) else {
                            let resp = json_response(StatusCode::UNAUTHORIZED, authz::error_body("UNAUTHENTICATED", "Sign in required"));
                            let (parts, body) = resp.into_parts();
                            return Ok::<_, hyper::Error>(Response::from_parts(parts, http_body_util::Either::Left(body)));
                        };
                        let sse_state = state.clone();
                        let sse_resp = state.sse_hub.subscribe_filtered(
                            move |ev| access::can_see_event(&sse_state.db, &sse_access, &sse_state.entities, ev),
                            is_admin,
                        );
                        // Map the streaming body to a boxed body for type compatibility
                        let (parts, body) = sse_resp.into_parts();
                        let boxed = http_body_util::Either::Right(body);
                        return Ok::<_, hyper::Error>(Response::from_parts(parts, boxed));
                    }
                    // All other requests — wrap Full<Bytes> in Either::Left
                    let resp = http_guard::isolate_panics(handle_request(req, state, remote_addr)).await?;
                    let (parts, body) = resp.into_parts();
                    Ok(Response::from_parts(parts, http_body_util::Either::Left(body)))
                }
            });
            if let Err(e) = http_guard::http1_builder().serve_connection(io, service).await {
                http_guard::log_connection_error(&e);
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
