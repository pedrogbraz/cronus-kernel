#![allow(dead_code, unused_imports)]
//! HTTP request routing — extracted from main.rs handle_request_inner.
//!
//! This module contains the main request dispatcher that routes incoming
//! HTTP requests to the appropriate handler: auth, API CRUD, pages,
//! brain endpoints, audit, docs, GraphQL, etc.

use std::sync::Arc;

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::{Method, Request, Response, StatusCode};
use serde_json::{json, Value};

use crate::parser::{self, AstNode, EntityNode};
use crate::{auth, database, graphql, hmr, payments, sse, ui};
use crate::constitution_check;
use crate::graph;
use crate::memory;
use crate::actions;
use crate::security;
use crate::{DEBUG_MODE, LAST_AI_ERRORS};
use crate::open_memory_db;
use crate::cli::objective_kernel::reconcile_field_type_str;

use super::response::{cors_origin, json_response, html_response, forbidden_response};
use super::auth_pages::{generate_login_page, generate_register_page};
use super::docs::{render_auto_docs, render_design_system, render_graph_page};
use super::state::{AppState, RequestTrace, TraceBuffer, generate_request_id, current_time_hms, iso_timestamp};
use super::api::{handle_api, fire_webhooks, fire_effects};

use std::sync::atomic::Ordering;

// ──────────────────────────────────────────────
// Outer request handler (tracing + headers)
// ──────────────────────────────────────────────

pub(crate) async fn handle_request(
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
    if let Ok(v) = hyper::header::HeaderValue::from_str(&format!("{}ms", duration_ms)) {
        resp.headers_mut().insert(hyper::header::HeaderName::from_static("x-response-time"), v);
    }
    if let Ok(v) = hyper::header::HeaderValue::from_str(&req_id) {
        resp.headers_mut().insert(hyper::header::HeaderName::from_static("x-request-id"), v);
    }
    if let Ok(v) = hyper::header::HeaderValue::from_str(&queries.to_string()) {
        resp.headers_mut().insert(hyper::header::HeaderName::from_static("x-query-count"), v);
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

// ──────────────────────────────────────────────
// Inner request dispatcher
// ──────────────────────────────────────────────

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
    if path == "/.cronus/version" {
        return Ok(json_response(StatusCode::OK, json!({ "version": hmr::current_version() })));
    }

    // ── Rate limiting (API endpoints only) ──
    if path.starts_with("/api/") {
        if let Some(resp) = check_rate_limit(&req, &state, &path, remote_addr) {
            return Ok(resp);
        }
    }

    // Brain: track every request
    let start = std::time::Instant::now();

    // ── Brain endpoints ──
    if let Some(resp) = handle_brain_endpoints(&path, &state) {
        return Ok(resp);
    }

    // ── Auth routes ──
    if path.starts_with("/api/auth/") {
        let resp = handle_auth_routes(req, &method, &path, &state).await;
        return Ok(resp);
    }

    // ── Payment endpoints ──
    if let Some(resp) = handle_payment_endpoints(&method, &path) {
        return Ok(resp);
    }

    // ── Audit endpoints (only the ones that need the request body) ──
    // POST /api/audit/results needs to consume the body; handle it before the non-consuming checks.
    if path == "/api/audit/results" && method == Method::POST {
        let collected = req.into_body().collect().await.unwrap_or_default();
        let body_bytes = collected.to_bytes();
        if let Ok(json_str) = std::str::from_utf8(&body_bytes) {
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

    // ── Non-body audit endpoints ──
    if let Some(resp) = handle_audit_endpoints_no_body(&method, &path, &state) {
        return Ok(resp);
    }

    // ── Health endpoint ──
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

    // ── Schema endpoint ──
    if path == "/api/schema" {
        return Ok(handle_schema_endpoint(&state));
    }

    // ── Seed endpoint ──
    if path == "/api/_seed" && method == Method::POST {
        return Ok(handle_seed_endpoint(&state));
    }

    // ── Health endpoint (extended) ──
    if path == "/api/_health" && method == Method::GET {
        return Ok(handle_health_endpoint(&state));
    }

    // ── AI Context Protocol ──
    if path == "/api/_context" && method == Method::GET {
        return Ok(handle_context_endpoint(&state));
    }

    // ── Server logs/stats ──
    if path == "/api/server/logs" && method == Method::GET {
        return Ok(handle_server_logs(&query, &state));
    }
    if path == "/api/server/stats" && method == Method::GET {
        return Ok(handle_server_stats(&state));
    }

    // ── Documentation endpoints ──
    if path == "/docs" && method == Method::GET {
        return Ok(html_response(render_auto_docs(&state)));
    }
    if path == "/docs/design" && method == Method::GET {
        return Ok(html_response(render_design_system(&state)));
    }
    if path == "/docs/graph" && method == Method::GET {
        return Ok(html_response(render_graph_page(&state)));
    }

    // ── GraphQL (POST consumes body, GET doesn't) ──
    if path == "/graphql" && method == Method::GET {
        return Ok(html_response(graphql::playground_html()));
    }
    if path == "/graphql" && method == Method::POST {
        let collected = match req.collect().await {
            Ok(c) => c,
            Err(_) => return Ok(json_response(StatusCode::BAD_REQUEST, json!({"error": "failed to read request body"}))),
        };
        let body_bytes = collected.to_bytes();
        let body_str = String::from_utf8_lossy(&body_bytes);
        let body_json: Value = serde_json::from_str(&body_str).unwrap_or(json!({}));
        let gql_query = body_json.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let variables = body_json.get("variables").cloned().unwrap_or(json!({}));
        let schema = graphql::GraphQLSchema::from_entities(&state.entities);
        let db = Arc::new(database::CronusDB::open(&state.db_path).expect("db"));
        let result = graphql::execute_graphql(gql_query, &variables, &schema, &db);
        return Ok(json_response(StatusCode::OK, result));
    }
    if path == "/graphql/schema" && method == Method::GET {
        let schema = graphql::GraphQLSchema::from_entities(&state.entities);
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(Full::new(Bytes::from(schema.sdl)))
            .unwrap_or_else(|_| Response::new(Full::new(Bytes::from("schema error")))));
    }

    // ── Script endpoints: custom routes from .scriptcronus ──
    let has_script_match = has_script_route(&method, &path, &state);
    if has_script_match {
        if let Some(resp) = handle_script_routes(&method, &path, req, &state).await {
            return Ok(resp);
        }
        // If handle_script_routes returned None (shouldn't happen after check), req was consumed
        return Ok(json_response(StatusCode::INTERNAL_SERVER_ERROR, serde_json::json!({"error":"script route failed"})));
    }

    // ── API routes: /api/... ──
    if path.starts_with("/api/") {
        let resp = handle_api_dispatch(req, &method, &path, &query, &state, start).await;
        return Ok(resp);
    }

    // ── Action execution endpoint ──
    if method == Method::POST && path.starts_with("/_action/") {
        let resp = handle_action_endpoint(req, &state).await;
        return Ok(resp);
    }

    // ── Form submission endpoint ──
    if method == Method::POST && path.starts_with("/_form/") {
        let resp = handle_form_endpoint(req, &state).await;
        return Ok(resp);
    }

    // ── Serve pages ──
    Ok(handle_page_request(req, &method, &path, &state).await)
}

// ──────────────────────────────────────────────
// Rate limiting
// ──────────────────────────────────────────────

fn check_rate_limit(
    req: &Request<Incoming>,
    state: &AppState,
    path: &str,
    remote_addr: std::net::SocketAddr,
) -> Option<Response<Full<Bytes>>> {
    let client_ip = req.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| remote_addr.ip().to_string());

    let is_auth = path.starts_with("/api/auth/login") || path.starts_with("/api/auth/signup");

    let check_result = if is_auth {
        let auth_key = format!("auth:{}", client_ip);
        state.auth_rate_limiter.check(&auth_key)
    } else {
        state.rate_limiter.check(&client_ip)
    };

    if let Err(retry_after) = check_result {
        let rate_body = serde_json::to_string(&json!({
            "error": "Too many requests",
            "retry_after": retry_after
        })).unwrap_or_else(|_| r#"{"error":"Too many requests"}"#.to_string());
        let mut resp = Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .header("Content-Type", "application/json")
            .header("Retry-After", retry_after.to_string())
            .body(Full::new(Bytes::from(rate_body)))
            .unwrap_or_else(|_| Response::new(Full::new(Bytes::from(r#"{"error":"Too many requests"}"#))));
        for (k, v) in security::security_headers() {
            resp.headers_mut().insert(
                hyper::header::HeaderName::from_static(k),
                hyper::header::HeaderValue::from_static(v),
            );
        }
        Some(resp)
    } else {
        None
    }
}

// ──────────────────────────────────────────────
// Brain endpoints
// ──────────────────────────────────────────────

fn handle_brain_endpoints(path: &str, state: &AppState) -> Option<Response<Full<Bytes>>> {
    if path == "/api/brain/stats" {
        if let Some(ref brain) = state.brain {
            return Some(json_response(StatusCode::OK, brain.stats()));
        }
        return Some(json_response(StatusCode::OK, json!({"status": "brain not initialized"})));
    }
    if path == "/api/brain/suggest" {
        if let Some(ref brain) = state.brain {
            let suggestions = brain.suggest("");
            return Some(json_response(StatusCode::OK, json!({"suggestions": suggestions})));
        }
        return Some(json_response(StatusCode::OK, json!({"suggestions": []})));
    }
    None
}

// ──────────────────────────────────────────────
// Auth routes
// ──────────────────────────────────────────────

async fn handle_auth_routes(
    req: Request<Incoming>,
    method: &Method,
    path: &str,
    state: &AppState,
) -> Response<Full<Bytes>> {
    let secret = auth::default_secret();
    let auth_header = req.headers().get("authorization").and_then(|v| v.to_str().ok()).map(|s| s.to_string());

    let user_table = state.entities.iter()
        .find(|e| {
            let lower = e.name.to_lowercase();
            lower == "user" || lower == "users"
        })
        .map(|e| e.name.as_str())
        .unwrap_or("User");

    match (method.clone(), path) {
        (Method::GET, "/api/auth/me") => {
            match auth::extract_user(auth_header.as_deref(), &secret) {
                Some(claims) => json_response(StatusCode::OK, json!({"sub": claims.sub, "role": claims.role, "exp": claims.exp})),
                None => json_response(StatusCode::UNAUTHORIZED, json!({"error": "unauthorized"})),
            }
        }

        (Method::POST, "/api/auth/signup") => {
            let body_bytes = req.collect().await.unwrap_or_default().to_bytes();
            let body: Value = serde_json::from_slice(&body_bytes).unwrap_or(json!({}));

            let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let email = body.get("email").and_then(|v| v.as_str()).unwrap_or("");
            let password = body.get("password").and_then(|v| v.as_str()).unwrap_or("");
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
                                .header("Set-Cookie", security::secure_cookie("cronus_token", &token, 86400, "/"))
                                .body(Full::new(Bytes::from(body.to_string())))
                                .unwrap_or_else(|_| Response::new(Full::new(Bytes::from(r#"{"error":"internal"}"#))))
                        }
                        Err(e) => json_response(StatusCode::BAD_REQUEST, json!({"error": e.to_string()}))
                    }
                }
            }
        }

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
                                    let remember = body.get("remember").and_then(|v| v.as_bool()).unwrap_or(false);
                                    let cookie_max_age = if remember { 30 * 86400 } else { 86400 }; // 30 days vs 24h
                                    let body = json!({
                                        "token": token,
                                        "user": {"id": user_id, "name": u.get("name"), "email": email, "role": role}
                                    });
                                    Response::builder()
                                        .status(StatusCode::OK)
                                        .header("Content-Type", "application/json")
                                        .header("Set-Cookie", security::secure_cookie("cronus_token", &token, cookie_max_age, "/"))
                                        .body(Full::new(Bytes::from(body.to_string())))
                                        .unwrap_or_else(|_| Response::new(Full::new(Bytes::from(r#"{"error":"internal"}"#))))
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
    }
}

// ──────────────────────────────────────────────
// Payment endpoints
// ──────────────────────────────────────────────

fn handle_payment_endpoints(method: &Method, path: &str) -> Option<Response<Full<Bytes>>> {
    if path == "/api/checkout" && *method == Method::POST {
        let engine = payments::PaymentEngine::from_env();
        let result = engine.create_checkout_url("starter", 2900, "/billing/success", "/billing/cancel", None);
        return Some(match result {
            Ok(data) => json_response(StatusCode::OK, data),
            Err(e) => json_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e})),
        });
    }
    if path == "/api/webhooks/stripe" && *method == Method::POST {
        let engine = payments::PaymentEngine::from_env();
        return Some(json_response(StatusCode::OK, json!({"status": "webhook received", "mode": if engine.is_live() { "live" } else { "mock" }})));
    }
    if path == "/api/payments/status" {
        let engine = payments::PaymentEngine::from_env();
        return Some(json_response(StatusCode::OK, engine.status()));
    }
    None
}

// ──────────────────────────────────────────────
// Audit endpoints (non-body-consuming)
// ──────────────────────────────────────────────

fn handle_audit_endpoints_no_body(
    method: &Method,
    path: &str,
    state: &AppState,
) -> Option<Response<Full<Bytes>>> {
    if path == "/api/audit/trigger" && *method == Method::GET {
        let trigger_html = r#"<!DOCTYPE html><html><head><script>
        fetch('/').then(r=>r.text()).then(html=>{
            var iframe=document.createElement('iframe');
            iframe.style.cssText='position:fixed;top:0;left:0;width:100vw;height:100vh;border:none;z-index:1';
            document.body.appendChild(iframe);
            iframe.srcdoc=html;
            iframe.onload=function(){
                var w=iframe.contentWindow;
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
        return Some(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Full::new(Bytes::from(trigger_html)))
            .unwrap_or_else(|_| Response::new(Full::new(Bytes::from("audit error")))));
    }

    if path == "/api/audit/results" && *method == Method::GET {
        let results = std::fs::read_to_string("/tmp/cronus-audit-results.json").unwrap_or_else(|_| "{}".into());
        let val: Value = serde_json::from_str(&results).unwrap_or(json!({"error": "no audit results yet"}));
        return Some(json_response(StatusCode::OK, val));
    }

    if path == "/api/audit/trail/verify" && *method == Method::GET {
        return Some(match state.audit_trail.verify() {
            Ok(result) => json_response(StatusCode::OK, result),
            Err(e) => json_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e})),
        });
    }
    if (path == "/api/audit/trail" || path.starts_with("/api/audit/trail?")) && *method == Method::GET {
        let query_str = path.split('?').nth(1).unwrap_or("");
        let limit: usize = query_str.split('&')
            .find(|p| p.starts_with("limit="))
            .and_then(|p| p.strip_prefix("limit="))
            .and_then(|v| v.parse().ok())
            .unwrap_or(50);
        let entity_filter: Option<String> = query_str.split('&')
            .find(|p| p.starts_with("entity="))
            .and_then(|p| p.strip_prefix("entity="))
            .map(|v| v.to_string());
        return Some(match state.audit_trail.query_filtered(limit, entity_filter.as_deref()) {
            Ok(entries) => json_response(StatusCode::OK, entries),
            Err(e) => json_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e})),
        });
    }

    None
}

// ──────────────────────────────────────────────
// Schema endpoint
// ──────────────────────────────────────────────

fn handle_schema_endpoint(state: &AppState) -> Response<Full<Bytes>> {
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
    json_response(StatusCode::OK, json!({
        "entities": schema,
        "total": state.entities.len(),
        "pages": state.pages.len(),
    }))
}

// ──────────────────────────────────────────────
// Seed endpoint
// ──────────────────────────────────────────────

fn handle_seed_endpoint(state: &AppState) -> Response<Full<Bytes>> {
    let mut results = serde_json::Map::new();
    for entity in &state.entities {
        if let Ok(count) = state.db.seed_entity(entity) {
            results.insert(entity.name.clone(), json!(count));
        }
    }
    json_response(StatusCode::OK, json!({"seeded": results}))
}

// ──────────────────────────────────────────────
// Extended health endpoint
// ──────────────────────────────────────────────

fn handle_health_endpoint(state: &AppState) -> Response<Full<Bytes>> {
    let brain_stats = state.brain.as_ref().map(|b| b.stats()).unwrap_or(json!({}));
    let entity_rows: Vec<Value> = state.entities.iter()
        .filter(|e| !e.name.starts_with('_'))
        .map(|e| {
            let count = state.db.count(&e.name).unwrap_or(0);
            json!({"entity": e.name, "rows": count})
        }).collect();
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
    json_response(StatusCode::OK, json!({
        "status": "healthy",
        "lint": { "warnings": 0, "errors": 0 },
        "behavioral": behavioral,
        "entities": entity_rows,
        "brain": brain_stats,
    }))
}

// ──────────────────────────────────────────────
// AI Context Protocol endpoint
// ──────────────────────────────────────────────

fn handle_context_endpoint(state: &AppState) -> Response<Full<Bytes>> {
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
            // Live row count from database
            if let Ok(count) = state.db.count(&e.name) {
                ej["row_count"] = json!(count);
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

    let memory_data = open_memory_db()
        .ok()
        .and_then(|m| m.get_context_data().ok())
        .unwrap_or(json!({"decisions": [], "changelog": []}));

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

    json_response(StatusCode::OK, json!({
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
    }))
}

// ──────────────────────────────────────────────
// Server logs
// ──────────────────────────────────────────────

fn handle_server_logs(query: &str, state: &AppState) -> Response<Full<Bytes>> {
    let limit: usize = query.split('&').find_map(|p| {
        let mut kv = p.splitn(2, '=');
        if kv.next() == Some("limit") { kv.next().and_then(|v| v.parse().ok()) } else { None }
    }).unwrap_or(100);
    match state.db.find_all("_brain_events", limit, 0) {
        Ok(rows) => json_response(StatusCode::OK, rows),
        Err(_) => json_response(StatusCode::OK, json!([])),
    }
}

// ──────────────────────────────────────────────
// Server stats
// ──────────────────────────────────────────────

fn handle_server_stats(state: &AppState) -> Response<Full<Bytes>> {
    let brain_stats = state.brain.as_ref().map(|b| b.stats()).unwrap_or(json!({}));
    let entity_counts: Vec<Value> = state.entities.iter()
        .filter(|e| !e.name.starts_with('_'))
        .map(|e| {
            let count = state.db.count(&e.name).unwrap_or(0);
            json!({"entity": e.name, "count": count})
        }).collect();
    json_response(StatusCode::OK, json!({
        "brain": brain_stats,
        "entities": entity_counts,
        "pages": state.pages.len(),
        "apis": state.apis.len(),
        "uptime": "running",
    }))
}

// ──────────────────────────────────────────────
// API dispatch (entity CRUD)
// ──────────────────────────────────────────────

async fn handle_api_dispatch(
    req: Request<Incoming>,
    method: &Method,
    path: &str,
    query: &str,
    state: &AppState,
    start: std::time::Instant,
) -> Response<Full<Bytes>> {
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
    let full_path = if query.is_empty() { path.to_string() } else { format!("{}?{}", path, query) };

    let resp = handle_api(method, &full_path, body.as_ref(), state, &owner_id);
    // Track request in brain
    if let Some(ref brain) = state.brain {
        let duration = start.elapsed().as_millis() as u64;
        let status = resp.status().as_u16();
        brain.track_request(method.as_str(), path, status, duration);
    }
    resp
}

// ──────────────────────────────────────────────
// Action execution endpoint
// ──────────────────────────────────────────────

async fn handle_action_endpoint(
    req: Request<Incoming>,
    state: &AppState,
) -> Response<Full<Bytes>> {
    let body_bytes = req.collect().await.unwrap_or_default().to_bytes();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap_or(json!({}));

    let entity = body.get("entity").and_then(|v| v.as_str()).unwrap_or("");
    let id = body.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let action_data = body.get("action").and_then(|v| v.as_str()).unwrap_or("{}");

    let (ok, effects) = actions::execute_action_validated(action_data, entity, id, &state.db, &state.entities);
    let response = actions::effects_to_json(&effects);
    json_response(if ok { StatusCode::OK } else { StatusCode::BAD_REQUEST }, response)
}

// ──────────────────────────────────────────────
// Form submission endpoint
// ──────────────────────────────────────────────

async fn handle_form_endpoint(
    req: Request<Incoming>,
    state: &AppState,
) -> Response<Full<Bytes>> {
    let body_bytes = req.collect().await.unwrap_or_default().to_bytes();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap_or(json!({}));

    let entity = body.get("entity").and_then(|v| v.as_str()).unwrap_or("");
    let data = body.get("data").cloned().unwrap_or(json!({}));

    if data.is_object() && !entity.is_empty() {
        if let Some(entity_schema) = state.entities.iter().find(|e| e.name.eq_ignore_ascii_case(entity)) {
            let errors = actions::validate_form_data(&data, entity_schema);
            if !errors.is_empty() {
                let response = json!({
                    "ok": false,
                    "errors": errors,
                    "effects": [{"type": "toast", "target": "Validation failed", "style": "error"}]
                });
                return json_response(StatusCode::BAD_REQUEST, response);
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
                return json_response(StatusCode::CREATED, response);
            }
            Err(e) => {
                let msg = e.to_string();
                let response = json!({ "ok": false, "error": msg, "effects": [{"type": "toast", "target": msg, "style": "error"}] });
                return json_response(StatusCode::BAD_REQUEST, response);
            }
        }
    }

    json_response(StatusCode::BAD_REQUEST, json!({"ok": false, "error": "missing entity or data"}))
}

// ──────────────────────────────────────────────
// Page rendering
// ──────────────────────────────────────────────

async fn handle_page_request(
    req: Request<Incoming>,
    method: &Method,
    path: &str,
    state: &AppState,
) -> Response<Full<Bytes>> {
    let accent = state.style.as_ref()
        .and_then(|s| s.accent.as_deref())
        .unwrap_or("amber");
    let app_name = &state.app.name;

    // ── Component preview route ──
    if path == "/components" {
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
        return html_response(html);
    }

    // ── Auto-generated auth pages (when auth block exists) ──
    if state.auth_entity.is_some() {
        if path == "/login" {
            let html = generate_login_page(state);
            return html_response(html);
        }
        if path == "/register" || path == "/signup" {
            let html = generate_register_page(state);
            return html_response(html);
        }
        if path == "/logout" {
            return Response::builder()
                .status(StatusCode::FOUND)
                .header("Location", "/login")
                .header("Set-Cookie", security::delete_cookie("cronus_token", "/"))
                .body(Full::new(Bytes::new()))
                .unwrap_or_else(|_| Response::new(Full::new(Bytes::new())));
        }
    }

    // ── Auth middleware — protect pages that require authentication ──
    let matched_requires = state.auth_required_pages.iter()
        .find(|(r, _)| r == path || (path.starts_with(r.as_str()) && r != "/"))
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
            .unwrap_or_else(|_| Response::new(Full::new(Bytes::new())));

        if requires_str.starts_with("role(") {
            let required_role = requires_str
                .trim_start_matches("role(")
                .trim_end_matches(')');
            match &token {
                Some(t) => {
                    match auth::verify_token(t, &secret) {
                        Ok(claims) => {
                            if claims.role != required_role && claims.role != "admin" {
                                // User is logged in but lacks the role — redirect to portal
                                return Response::builder()
                                    .status(StatusCode::FOUND)
                                    .header("Location", "/portal")
                                    .body(Full::new(Bytes::new()))
                                    .unwrap_or_else(|_| Response::new(Full::new(Bytes::new())));
                            }
                        }
                        Err(_) => return redirect_to_login(),
                    }
                }
                None => return redirect_to_login(),
            }
        } else {
            let authenticated = match &token {
                Some(t) => auth::verify_token(t, &secret).is_ok(),
                None => false,
            };
            if !authenticated {
                return redirect_to_login();
            }
        }
    }

    // Find matching page
    let page = state.pages.iter().find(|p| {
        if p.route == path { return true; }
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
        return render_matched_page(req, page, path, state, accent, app_name);
    }

    // 404
    let body = format!(
        "<div class=\"flex items-center justify-center min-h-[60vh]\"><div class=\"text-center\"><h1 class=\"text-6xl font-bold text-neutral-600\">404</h1><p class=\"mt-4 text-neutral-400\">Page not found</p><a href=\"/\" class=\"mt-6 inline-block text-{}-400 hover:underline\">\u{2190} Back home</a></div></div>",
        accent
    );
    let html = if let Some(ref layout) = state.layout {
        ui::render_layout_declarative(app_name, layout, "/404", &body)
    } else {
        ui::render_layout(app_name, &state.pages, accent, &body)
    };
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Full::new(Bytes::from(html)))
        .unwrap_or_else(|_| Response::new(Full::new(Bytes::from("not found"))))
}

// ──────────────────────────────────────────────
// Page rendering (matched page)
// ──────────────────────────────────────────────

fn render_matched_page(
    req: Request<Incoming>,
    page: &parser::PageNode,
    path: &str,
    state: &AppState,
    accent: &str,
    app_name: &str,
) -> Response<Full<Bytes>> {
    // Extract route params
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
                return html_response(html);
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
                    ui::render_layout_declarative(app_name, layout, path, &body)
                } else {
                    ui::render_layout(app_name, &state.pages, accent, &body)
                };
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("Content-Type", "text/html; charset=utf-8")
                    .body(Full::new(Bytes::from(html)))
                    .unwrap_or_else(|_| Response::new(Full::new(Bytes::from("internal error"))));
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
        return html_response(html);
    }

    // Auth pages — standalone login/signup with no layout chrome
    let route_lower = page.route.to_lowercase();
    let title_lower = page.title.as_deref().unwrap_or("").to_lowercase();
    let is_auth_page = route_lower == "/login" || route_lower == "/signup"
        || title_lower.contains("sign in") || title_lower.contains("sign up")
        || title_lower.contains("login") || title_lower.contains("signup");
    let has_custom_template = page.page_type == "custom" && page.sections.iter().any(|s| s.template.is_some());
    if is_auth_page && !has_custom_template {
        let is_login = route_lower == "/login" || title_lower.contains("login") || title_lower.contains("sign in");
        let html = ui::render_auth_page(page, is_login);
        return html_response(html);
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

    let has_section_sidebar = page.sections.iter().any(|s| s.section_type == "sidebar");

    let is_order_detail = page.sections.iter().any(|s| s.section_type == "order-header" || s.section_type == "line-items");
    if is_order_detail {
        let referenced_comps: Vec<parser::ComponentNode> = page.components.iter()
            .filter_map(|name| state.components.iter().find(|c| c.name == *name))
            .cloned()
            .collect();
        let html = ui::render_order_detail_dashboard(app_name, &page.sections, &referenced_comps, theme, page.route.as_str());
        return html_response(html);
    }

    let is_settings_page = page.sections.iter().any(|s| s.section_type == "settings-profile" || s.section_type == "subscription-card");
    if is_settings_page {
        let referenced_comps: Vec<parser::ComponentNode> = page.components.iter()
            .filter_map(|name| state.components.iter().find(|c| c.name == *name))
            .cloned()
            .collect();
        let html = ui::render_settings_dashboard(app_name, &page.sections, &referenced_comps, theme, page.route.as_str());
        return html_response(html);
    }

    let has_templates = page.sections.iter().any(|s| s.template.is_some() || s.config.get("template").is_some());
    // Landing page layout: use clean shell (no extra topbar/footer wrapper) when:
    // - page has templates, OR
    // - page type is "custom" with landing section types (hero, features, etc.)
    let landing_types_early = ["hero", "topbar", "features", "pricing", "cta", "testimonial", "faq", "trusted", "footer"];
    let is_custom_landing = page.page_type == "custom" && page.sections.iter().any(|s| landing_types_early.contains(&s.section_type.as_str()));
    if has_templates || is_custom_landing {
        let body = ui::render_page(page, &state.entities, accent, theme, Some(&state.db), &route_params, &page_owner_id);
        let html = ui::render_layout_landing_ex(app_name, &body, theme, state.style.as_ref(), state.app.tailwind_config.as_deref());
        return html_response(html);
    }

    if has_section_sidebar {
        let body = ui::render_page(page, &state.entities, accent, theme, Some(&state.db), &route_params, &page_owner_id);
        let html = ui::render_layout_dashboard(&state.app.name, &body, theme);
        return html_response(html);
    }

    let mut body = ui::render_page(page, &state.entities, accent, theme, Some(&state.db), &route_params, &page_owner_id);

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

    if page.page_type == "components" && !state.components.is_empty() {
        body.push_str("\n");
        body.push_str(&ui::render_components_page(&state.components));
    }

    if page.page_type == "custom" && page.sections.is_empty() && !state.components.is_empty() {
        body.push_str("\n");
        body.push_str(&ui::render_components_page(&state.components));
    }

    let has_sidebar_component = !page.components.is_empty() && page.components.iter().any(|comp_name| {
        state.components.iter().any(|c| {
            c.name == *comp_name && (
                c.style.as_deref().unwrap_or("").contains("sidenav") ||
                c.layout.as_deref().unwrap_or("") == "sidebar"
            )
        })
    });

    let landing_section_types = ["hero", "topbar", "checkout", "features", "pricing", "cta", "testimonial", "faq", "trusted", "footer"];
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
        ui::render_layout_landing_ex(app_name, &body, theme, state.style.as_ref(), state.app.tailwind_config.as_deref())
    } else if is_checkout {
        let referenced_comps: Vec<parser::ComponentNode> = page.components.iter()
            .filter_map(|name| state.components.iter().find(|c| c.name == *name))
            .cloned()
            .collect();
        ui::render_checkout_dashboard(app_name, &page.sections, &referenced_comps, theme)
    } else if is_dashboard {
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
            ui::render_generic_dashboard(app_name, &body, &referenced_comps, theme, current_route)
        }
    } else if is_landing {
        ui::render_layout_landing(app_name, &body, theme, state.style.as_ref())
    } else if let Some(ref layout) = state.layout {
        ui::render_layout_declarative(app_name, layout, current_route, &body)
    } else {
        ui::render_layout(app_name, &state.pages, accent, &body)
    };
    html_response(html)
}

// ──────────────────────────────────────────────
// Script endpoint + webhook handler
// ──────────────────────────────────────────────

fn has_script_route(method: &Method, path: &str, state: &AppState) -> bool {
    let registry = &state.script_registry;
    if registry.scripts.is_empty() { return false; }
    let method_str = method.as_str();
    // Check endpoints
    for (_script, endpoint) in registry.get_endpoints() {
        if endpoint.method == method_str && endpoint.path == path {
            return true;
        }
    }
    // Check webhooks (POST only)
    if method == Method::POST && !registry.get_webhook_handlers(path).is_empty() {
        return true;
    }
    false
}

async fn handle_script_routes(
    method: &Method,
    path: &str,
    req: Request<Incoming>,
    state: &AppState,
) -> Option<Response<Full<Bytes>>> {
    let registry = &state.script_registry;
    if registry.scripts.is_empty() { return None; }

    // Check script endpoints
    let method_str = method.as_str();
    for (script, endpoint) in registry.get_endpoints() {
        if endpoint.method == method_str && endpoint.path == path {
            // Auth check
            if let Some(ref required_role) = endpoint.auth {
                // Extract token from Authorization header
                let token = req.headers().get("Authorization")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.strip_prefix("Bearer "));
                let secret = state.app.config.get("jwt_secret")
                    .map(|s| s.as_str())
                    .unwrap_or("cronus-secret");
                match token {
                    Some(t) => {
                        if let Ok(claims) = crate::auth::verify_token(t, secret) {
                            if required_role != "any" && claims.role != *required_role {
                                return Some(json_response(StatusCode::FORBIDDEN, serde_json::json!({"error": "insufficient role"})));
                            }
                            let body = collect_body(req).await;
                            let ctx = crate::scripting::execute_endpoint(endpoint, &script.name, &state.db, &claims.sub, &claims.role, body.as_ref(), &std::collections::HashMap::new());
                            return Some(script_response(ctx));
                        } else {
                            return Some(json_response(StatusCode::UNAUTHORIZED, serde_json::json!({"error": "invalid token"})));
                        }
                    }
                    None => {
                        return Some(json_response(StatusCode::UNAUTHORIZED, serde_json::json!({"error": "authentication required"})));
                    }
                }
            } else {
                // No auth required
                let body = collect_body(req).await;
                let ctx = crate::scripting::execute_endpoint(endpoint, &script.name, &state.db, "anonymous", "public", body.as_ref(), &std::collections::HashMap::new());
                return Some(script_response(ctx));
            }
        }
    }

    // Check script webhook receivers
    if method == Method::POST {
        let webhook_handlers = registry.get_webhook_handlers(path);
        if !webhook_handlers.is_empty() {
            let body = collect_body(req).await;
            let body_ref = body.as_ref().unwrap_or(&serde_json::Value::Null);
            let ctx = crate::scripting::execute_webhook(registry, path, body_ref, &state.db, &std::collections::HashMap::new());
            if let Some(ctx) = ctx {
                if let Some(resp) = ctx.response {
                    return Some(Response::builder()
                        .status(resp.status)
                        .header("Content-Type", "application/json")
                        .body(Full::new(Bytes::from(resp.body)))
                        .unwrap());
                }
            }
            return Some(json_response(StatusCode::OK, serde_json::json!({"ok": true})));
        }
    }

    None
}

async fn collect_body(req: Request<Incoming>) -> Option<serde_json::Value> {
    let body_bytes = req.into_body().collect().await.ok()?.to_bytes();
    serde_json::from_slice(&body_bytes).ok()
}

fn script_response(ctx: crate::scripting::vm::ScriptContext) -> Response<Full<Bytes>> {
    if let Some(resp) = ctx.response {
        let mut builder = Response::builder().status(resp.status);
        for (k, v) in &resp.headers {
            builder = builder.header(k.as_str(), v.as_str());
        }
        if !resp.headers.contains_key("Content-Type") {
            builder = builder.header("Content-Type", "application/json");
        }
        builder.body(Full::new(Bytes::from(resp.body)))
            .unwrap_or_else(|_| json_response(StatusCode::INTERNAL_SERVER_ERROR, serde_json::json!({"error": "response build failed"})))
    } else {
        json_response(StatusCode::OK, serde_json::json!({"ok": true, "logs": ctx.logs}))
    }
}
