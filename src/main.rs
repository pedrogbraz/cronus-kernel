#![allow(dead_code, unused_imports, unused_variables)]
mod animations;
mod auth;
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
mod server;
mod sse;
mod tailwind;
mod testing;
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
pub static STRICT_MODE: AtomicBool = AtomicBool::new(false);

// ══════════════════════════════════════════════════
// MAIN
// ══════════════════════════════════════════════════

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let strict = args.iter().any(|a| a == "--strict");
    STRICT_MODE.store(strict, Ordering::Relaxed);
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match cmd {
        "run" => cmd_run(&args).await,
        "build" => cmd_build(&args),
        "parse" => cmd_parse(&args),
        "new" => cmd_new(&args),
        "deploy" => cmd_deploy(&args),
        "doctor" => cmd_doctor(&args),
        "stats" => cmd_stats(&args),
        "export" => cmd_export(&args),
        "test" => cmd_test(&args),
        "compose" => cmd_compose(&args),
        "generate" | "gen" => cmd_generate(&args),
        "dump" => cmd_dump(&args),
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
    println!("    \x1b[32mnew\x1b[0m <template>       Create project (landing/saas/api/ecommerce/blog)");
    println!("    \x1b[32mbuild\x1b[0m [--strict]      Parse and validate .cronus file (strict mode)");
    println!("    \x1b[32mparse\x1b[0m <file> [--strict] Parse and show AST (strict mode)");
    println!("    \x1b[32mdeploy\x1b[0m           Generate deploy artifacts (--fly, --railway, --static)");
    println!("    \x1b[32mdoctor\x1b[0m           Check .cronus syntax + DB + ports");
    println!("    \x1b[32mstats\x1b[0m            Project stats (entities, pages, DB size)");
    println!("    \x1b[32mexport\x1b[0m           Export to cronus-project.ir.json");
    println!("    \x1b[32mtest\x1b[0m [port]          Auto-gen and run CRUD tests");
    println!("    \x1b[32mtest\x1b[0m --conformance   Run conformance test suite");
    println!("    \x1b[32mcompose\x1b[0m          Compose all .cronus files and show result");
    println!("    \x1b[32mgenerate\x1b[0m <desc>  Generate .cronus from description");
    println!("    \x1b[32mversion\x1b[0m          Show version");
    println!();
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
                            "role": "user"
                        });

                        match state.db.insert(user_table, &user_data) {
                            Ok(user) => {
                                let user_id = user.get("id").and_then(|v| v.as_str()).unwrap_or("");
                                let token = auth::create_token(user_id, "user", &secret);
                                json_response(StatusCode::CREATED, json!({
                                    "token": token,
                                    "user": {"id": user_id, "name": name, "email": email, "role": "user"}
                                }))
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
                                        json_response(StatusCode::OK, json!({
                                            "token": token,
                                            "user": {"id": user_id, "name": u.get("name"), "email": email, "role": role}
                                        }))
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
        let html = ui::render_layout(app_name, &state.pages, accent, &body);
        return Ok(html_response(html));
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
                    let html = ui::render_layout(app_name, &state.pages, accent, &body);
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
        let mut body = ui::render_page(page, &state.entities, accent, theme);

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
        let is_landing = !has_sidebar_component && (page.page_type == "checkout" || (page.page_type == "custom" && page.sections.iter().any(|s| s.section_type == "hero" || s.section_type == "topbar" || s.section_type == "checkout")));
        let dashboard_types = ["sidebar", "card", "page-header", "stat-cards", "product-grid",
            "team-list", "policies", "activity-table", "status-card", "links",
            "live-keys", "test-keys", "webhooks", "quick-links",
            "current-plan", "usage-status", "billing-stats", "payment-methods", "recent-invoices",
            "balance-card", "upcoming-card", "payout-history", "support-banner",
            "checkout-form", "product-summary", "trust-indicators", "testimonial",
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
        let html = if is_checkout {
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
            ui::render_layout_landing(app_name, &body, theme)
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
    let html = ui::render_layout(app_name, &state.pages, accent, &body);
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
    let port: u16 = args.get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0); // 0 = use app port

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
        println!("  \x1b[32m✓\x1b[0m Found {} ({} lines)", files[0], lines);
        (n, lines)
    } else {
        // Multi-agent mode: compose all .cronus files
        println!("  \x1b[36m⚡\x1b[0m Multi-file mode: {} files detected", files.len());
        let mut total = 0;
        for f in &files {
            let lines = fs::read_to_string(f).map(|s| s.lines().count()).unwrap_or(0);
            total += lines;
            println!("    \x1b[32m+\x1b[0m {} ({} lines)", f, lines);
        }
        let n = match parser::parse_directory(".") {
            Ok(n) => n,
            Err(e) => { eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e); std::process::exit(1); }
        };
        println!("  \x1b[32m✓\x1b[0m Composed {} files ({} lines total)", files.len(), total);
        (n, total)
    };

    let file = files[0].clone(); // for HMR watcher

    // Extract AST parts
    let mut app = AppNode { name: "CRONUS App".into(), stack: vec![], port: 5175, database: None };
    let mut entities: Vec<EntityNode> = vec![];
    let mut pages: Vec<PageNode> = vec![];
    let mut style: Option<StyleNode> = None;
    let mut apis: Vec<ApiNode> = vec![];
    let mut cronus_components: Vec<parser::ComponentNode> = vec![];
    let mut route_count = 0;

    for node in &nodes {
        match node {
            AstNode::App(a) => app = a.clone(),
            AstNode::Entity(e) => entities.push(e.clone()),
            AstNode::Page(p) => pages.push(p.clone()),
            AstNode::Style(s) => style = Some(s.clone()),
            AstNode::Api(a) => {
                route_count += a.routes.len();
                apis.push(a.clone());
            }
            AstNode::Component(c) => cronus_components.push(c.clone()),
            _ => {}
        }
    }

    let serve_port = if port > 0 { port } else { app.port };
    let comp_count = cronus_components.len();
    println!("  \x1b[32m✓\x1b[0m Parsed: {} entities, {} pages, {} routes, {} components", entities.len(), pages.len(), route_count, comp_count);

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
    println!("  \x1b[32m✓\x1b[0m Database: ./{} ({} tables)", db_path, table_count);

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
    println!("  \x1b[32m✓\x1b[0m Hydra Brain: online");

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
        // Multi-file mode: watch entire directory
        hmr::start_directory_watcher(".", move || {
            let v = hmr::bump_version();
            eprintln!("  \x1b[33m⚡\x1b[0m File changed — version {} (browser will reload)", v);
        });
        println!("  \x1b[32m✓\x1b[0m HMR watching {} .cronus files", files.len());
    } else {
        let hmr_file = file.clone();
        hmr::start_watcher(&hmr_file, move || {
            let v = hmr::bump_version();
            eprintln!("  \x1b[33m⚡\x1b[0m File changed — version {} (browser will reload)", v);
        });
        println!("  \x1b[32m✓\x1b[0m HMR watching {}", file);
    }
    println!("  \x1b[32m✓\x1b[0m Server running\n");
    println!("  App      → \x1b[1mhttp://localhost:{}\x1b[0m", serve_port);
    println!("  API      → \x1b[1mhttp://localhost:{}/api\x1b[0m", serve_port);
    println!("  GraphQL  → \x1b[1mhttp://localhost:{}/graphql\x1b[0m", serve_port);
    println!("  Theme    → {} / {}", app.name, accent);
    println!("\n  Press Ctrl+C to stop\n");

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
        eprintln!("  \x1b[31m✗\x1b[0m Usage: cronus dump <file.html> [-o output.cronus]");
        std::process::exit(1);
    });

    let html = fs::read_to_string(file).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Error reading {}: {}", file, e);
        std::process::exit(1);
    });

    eprintln!("  \x1b[36m⚡\x1b[0m Dumping {} ({} bytes)...", file, html.len());
    let cronus = dump::dump_html(&html);

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

fn cmd_build(args: &[String]) {
    let file = args.get(2).cloned().or_else(find_cronus_file).unwrap_or_else(|| {
        eprintln!("  \x1b[31m✗\x1b[0m No .cronus file found");
        std::process::exit(1);
    });

    let source = fs::read_to_string(&file).unwrap();
    match parser::parse(&source) {
        Ok(nodes) => {
            let (entities, pages, routes) = parser::stats(&nodes);
            println!("  \x1b[32m✓\x1b[0m {} — {} entities, {} pages, {} routes", file, entities, pages, routes);
            println!("  \x1b[32m✓\x1b[0m Valid .cronus file");
        }
        Err(e) => {
            eprintln!("  \x1b[31m✗\x1b[0m Parse error: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_new(args: &[String]) {
    let template = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| {
        eprintln!("  Usage: cronus new <template>");
        eprintln!("  Templates: landing, saas, api, ecommerce, blog");
        std::process::exit(1);
    });

    let content = match template {
        "landing" => TEMPLATE_LANDING,
        "saas" => TEMPLATE_SAAS,
        "api" => TEMPLATE_API,
        "ecommerce" => TEMPLATE_ECOMMERCE,
        "blog" => TEMPLATE_BLOG,
        _ => {
            eprintln!("  \x1b[33m✗\x1b[0m Unknown template: {}", template);
            eprintln!("  Available: landing, saas, api, ecommerce, blog");
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

    println!("  \x1b[32m✓\x1b[0m Created {}/{} (template: {})", dir, "app.cronus", template);
    println!("  Next: \x1b[1mcd {} && cronus run\x1b[0m", dir);
}

// ══════════════════════════════════════════════════
// TEMPLATES
// ══════════════════════════════════════════════════

const TEMPLATE_LANDING: &str = r#"# Landing Page — CRONUS
# Template: landing

app "My Landing" {
  stack react + tailwind
  port 5175
  theme dark
  database sqlite "./data.db"
}

entity Lead {
  name      string    required
  email     email     required unique
  company   string
  plan      enum      [starter, pro, enterprise]
  createdAt date
}

api /leads {
  create  POST   /        auth:public
  list    GET    /        auth:jwt
}

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

  section cta {
    title "Ready to get started?"
    subtitle "Join thousands of teams already shipping faster"
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

app "My SaaS" {
  stack react + tailwind
  port 5175
  theme dark
  database sqlite "./data.db"
}

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

page "/login" type:form entity:User {
  title "Sign In"
  fields [email, password]
}

page "/signup" type:form entity:User {
  title "Create Account"
  fields [name, email, password]
}

page "/dashboard" type:dashboard {
  title "Dashboard"
}

page "/projects" type:list entity:Project {
  title "Projects"
  columns [name, status, team, createdAt]
}

page "/billing" type:list entity:Invoice {
  title "Billing"
  columns [amount, status, plan, period]
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
# Template: ecommerce (storefront + cart + orders)

app "My Store" {
  stack react + tailwind
  port 5175
  theme dark
  database sqlite "./data.db"
}

entity Product {
  name        string    required
  description text
  price       money     required
  currency    enum      [usd, brl, eur]
  sku         string    unique
  stock       number
  category    string
  imageUrl    url
  active      boolean
  createdAt   date
}

entity CartItem {
  product     -> Product
  quantity    number    required
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
  customer    -> Customer
  status      enum      [pending, paid, shipped, delivered, cancelled]
  total       money     required
  currency    enum      [usd, brl, eur]
  createdAt   date
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
}

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

page "/products" type:list entity:Product {
  title "Products"
  columns [name, price, stock, category, active]
}

page "/purchases" type:list entity:Purchase {
  title "Orders"
  columns [status, total, currency, createdAt]
}

page "/dashboard" type:dashboard {
  title "Dashboard"
}

style {
  theme dark
  accent emerald
  font "Inter"
}
"#;

const TEMPLATE_BLOG: &str = r#"# Blog — CRONUS
# Template: blog (posts + authors + comments + tags)

app "My Blog" {
  stack react + tailwind
  port 5175
  theme dark
  database sqlite "./data.db"
}

entity Author {
  name      string    required
  email     email     required unique
  bio       text
  avatar    url
  createdAt date
}

entity Tag {
  name      string    required unique
  slug      slug      required unique
  color     string
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

api /tags {
  list    GET    /        auth:public
  create  POST   /        auth:jwt
}

api /comments {
  list    GET    /        auth:public
  create  POST   /        auth:public
  delete  DELETE /:id     auth:jwt
}

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
}

page "/posts" type:list entity:Post {
  title "All Posts"
  columns [title, author, status, publishedAt]
}

page "/authors" type:list entity:Author {
  title "Authors"
  columns [name, email, createdAt]
}

page "/dashboard" type:dashboard {
  title "Dashboard"
}

style {
  theme dark
  accent sky
  font "Inter"
}
"#;

fn cmd_parse(args: &[String]) {
    let file = args.get(2).cloned().unwrap_or_else(|| {
        eprintln!("Usage: cronus parse <file.cronus>");
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

fn cmd_generate(args: &[String]) {
    let desc = if args.len() > 2 {
        args[2..].join(" ")
    } else {
        eprintln!("  Usage: cronus generate \"description of your app\"");
        eprintln!("  Example: cronus generate \"saas with users, projects, billing\"");
        std::process::exit(1);
    };

    // Check for template shortcuts first
    let desc_lower = desc.to_lowercase();
    if desc_lower == "saas" || desc_lower == "landing" {
        let content = match desc_lower.as_str() {
            "saas" => generate_saas_template(),
            "landing" => generate_landing_template(),
            _ => unreachable!(),
        };
        let filename = format!("{}.cronus", desc_lower);
        fs::write(&filename, &content).expect("Failed to write template");
        println!("  \x1b[32m✓\x1b[0m Generated {} ({} lines)", filename, content.lines().count());
        return;
    }

    let app_name = extract_app_name(&desc);

    let mut entities = Vec::new();
    let mut pages = Vec::new();
    let mut apis = Vec::new();
    let mut accent = "blue";

    // Detect entities from keywords
    if desc_lower.contains("user") || desc_lower.contains("auth") || desc_lower.contains("login") {
        entities.push(r#"entity User {
  email       email     required unique
  name        string    required
  password    string    sensitive
  role        enum      [admin, member]
  avatarUrl   url
  createdAt   date
}"#);
        apis.push(r#"api /auth {
  signup    POST   /signup     auth:public
  login     POST   /login      auth:public
  me        GET    /me         auth:jwt
}

api /users {
  list      GET    /           auth:jwt
  detail    GET    /:id        auth:jwt
  edit      PATCH  /:id        auth:jwt
}"#);
        pages.push(r#"page "/login" type:form entity:User {
  title "Sign In"
  fields [email, password]
}

page "/signup" type:form entity:User {
  title "Create Account"
  fields [name, email, password]
}"#);
    }

    if desc_lower.contains("product") || desc_lower.contains("item") || desc_lower.contains("catalog") {
        entities.push(r#"entity Product {
  name        string    required
  description text
  price       money     required
  currency    enum      [usd, brl, eur]
  category    string
  imageUrl    url
  active      boolean
  createdAt   date
}"#);
        apis.push(r#"api /products {
  list    GET    /        auth:public
  create  POST   /        auth:jwt
  detail  GET    /:id     auth:public
  edit    PATCH  /:id     auth:jwt
  delete  DELETE /:id     auth:jwt
}"#);
        pages.push(r#"page "/products" type:list entity:Product {
  title "Products"
  columns [name, price, category, active]
}"#);
    }

    if desc_lower.contains("order") || desc_lower.contains("purchase") || desc_lower.contains("sale") {
        entities.push(r#"entity Order {
  customer    -> User
  status      enum      [pending, paid, shipped, delivered, cancelled]
  total       money     required
  currency    enum      [usd, brl, eur]
  createdAt   date
}"#);
        apis.push(r#"api /orders {
  list    GET    /        auth:jwt
  create  POST   /        auth:public
  detail  GET    /:id     auth:jwt
}"#);
        pages.push(r#"page "/orders" type:list entity:Order {
  title "Orders"
  columns [status, total, currency, createdAt]
}"#);
    }

    if desc_lower.contains("project") || desc_lower.contains("task") || desc_lower.contains("kanban") {
        entities.push(r#"entity Project {
  name        string    required
  description text
  status      enum      [active, paused, completed, archived]
  owner       -> User
  createdAt   date
}"#);
        entities.push(r#"entity Task {
  project     -> Project
  title       string    required
  description text
  status      enum      [todo, in_progress, review, done]
  priority    enum      [low, medium, high, urgent]
  assignee    -> User
  dueDate     date
  createdAt   date
}"#);
        apis.push(r#"api /projects {
  list    GET    /        auth:jwt
  create  POST   /        auth:jwt
  detail  GET    /:id     auth:jwt
  edit    PATCH  /:id     auth:jwt
  delete  DELETE /:id     auth:jwt
}

api /tasks {
  list    GET    /        auth:jwt
  create  POST   /        auth:jwt
  detail  GET    /:id     auth:jwt
  edit    PATCH  /:id     auth:jwt
  delete  DELETE /:id     auth:jwt
}"#);
        pages.push(r#"page "/projects" type:list entity:Project {
  title "Projects"
  columns [name, status, createdAt]
}

page "/tasks" type:list entity:Task {
  title "Tasks"
  columns [title, status, priority, dueDate]
}"#);
    }

    if desc_lower.contains("billing") || desc_lower.contains("subscription") || desc_lower.contains("plan") {
        entities.push(r#"entity Plan {
  name        string    required
  price       money     required
  currency    enum      [usd, brl, eur]
  interval    enum      [monthly, yearly]
  features    text
  active      boolean
  createdAt   date
}"#);
        entities.push(r#"entity Subscription {
  user        -> User
  plan        -> Plan
  status      enum      [active, canceled, past_due]
  currentPeriodEnd date
  createdAt   date
}"#);
        apis.push(r#"api /plans {
  list    GET    /        auth:public
}

api /subscriptions {
  list    GET    /        auth:jwt
  create  POST   /        auth:jwt
  cancel  DELETE /:id     auth:jwt
}"#);
        pages.push(r#"page "/billing" type:list entity:Subscription {
  title "Billing"
  columns [status, currentPeriodEnd]
}"#);
    }

    if desc_lower.contains("blog") || desc_lower.contains("post") || desc_lower.contains("article") {
        entities.push(r#"entity Post {
  title       string    required
  slug        slug      required unique
  content     text      required
  excerpt     text
  status      enum      [draft, published, archived]
  author      -> User
  publishedAt date
  createdAt   date
}"#);
        apis.push(r#"api /posts {
  list    GET    /        auth:public
  create  POST   /        auth:jwt
  detail  GET    /:slug   auth:public
  edit    PATCH  /:slug   auth:jwt
  delete  DELETE /:slug   auth:jwt
}"#);
        pages.push(r#"page "/posts" type:list entity:Post {
  title "Posts"
  columns [title, status, publishedAt]
}"#);
    }

    if desc_lower.contains("review") || desc_lower.contains("rating") || desc_lower.contains("feedback") {
        entities.push(r#"entity Review {
  user        -> User
  rating      number    required
  comment     text
  status      enum      [pending, approved, rejected]
  createdAt   date
}"#);
        apis.push(r#"api /reviews {
  list    GET    /        auth:public
  create  POST   /        auth:jwt
}"#);
        pages.push(r#"page "/reviews" type:list entity:Review {
  title "Reviews"
  columns [rating, status, comment]
}"#);
    }

    if desc_lower.contains("notification") || desc_lower.contains("alert") {
        entities.push(r#"entity Notification {
  user        -> User
  title       string    required
  body        text
  channel     enum      [email, push, sms]
  status      enum      [pending, sent, read]
  createdAt   date
}"#);
        apis.push(r#"api /notifications {
  list    GET    /        auth:jwt
}"#);
    }

    // Detect theme
    if desc_lower.contains("ecommerce") || desc_lower.contains("store") || desc_lower.contains("shop") {
        accent = "emerald";
    } else if desc_lower.contains("finance") || desc_lower.contains("billing") {
        accent = "violet";
    } else if desc_lower.contains("health") || desc_lower.contains("medical") {
        accent = "teal";
    }

    // If no entities detected, generate a basic one
    if entities.is_empty() {
        entities.push(r#"entity Item {
  name        string    required
  description text
  status      enum      [active, archived]
  createdAt   date
}"#);
        apis.push(r#"api /items {
  list    GET    /        auth:public
  create  POST   /        auth:jwt
  detail  GET    /:id     auth:public
  edit    PATCH  /:id     auth:jwt
  delete  DELETE /:id     auth:jwt
}"#);
        pages.push(r#"page "/items" type:list entity:Item {
  title "Items"
  columns [name, status, createdAt]
}"#);
    }

    // Always add dashboard
    pages.insert(0, r#"page "/dashboard" type:dashboard {
  title "Dashboard"
}"#);

    // Build the .cronus file
    let mut output = String::new();
    output.push_str(&format!(r#"# {} — Generated by CRONUS
# From: "{}"

app "{}" {{
  stack react + tailwind
  port 5175
  database sqlite "./data.db"
  theme dark
}}

"#, app_name, desc, app_name));

    output.push_str("# ── Entities ──\n\n");
    for e in &entities { output.push_str(e); output.push_str("\n\n"); }

    output.push_str("# ── API Routes ──\n\n");
    for a in &apis { output.push_str(a); output.push_str("\n\n"); }

    output.push_str("# ── Pages ──\n\n");

    // Landing page
    output.push_str(&format!(r#"page "/" type:custom {{
  section hero {{
    title "{app_name}"
    subtitle "Built with CRONUS — {desc}"
    cta "Get Started" -> "/signup" primary
  }}
}}

"#));

    for p in &pages { output.push_str(p); output.push_str("\n\n"); }

    output.push_str(&format!(r#"style {{
  theme dark
  accent {accent}
  font "Inter"
}}
"#));

    // Write file
    let filename = "app.cronus";
    std::fs::write(filename, &output).unwrap_or_else(|e| {
        eprintln!("  \x1b[31m✗\x1b[0m Error writing {}: {}", filename, e);
        std::process::exit(1);
    });

    let line_count = output.lines().count();
    let entity_count = entities.len();
    let page_count = pages.len();

    println!("  \x1b[32m✓\x1b[0m Generated {} ({} lines)", filename, line_count);
    println!("    {} entities, {} pages, {} accent", entity_count, page_count, accent);
    println!();
    println!("  Next: \x1b[1mcronus run\x1b[0m");
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
