//! Live HTTP dispatch.
//!
//! `serve` is the connection service (audit canvas, SSE stream, panic
//! isolation). `handle_request` applies the pre-route guards and records
//! traces. `handle_request_inner` is the ordered route table: each group
//! either answers (`Ok(response)`) or hands the request back (`Err(req)`) to
//! the next group. Order is precedence; keep guards where they are.

use crate::cli::objective_kernel::reconcile_field_type_str;
use crate::open_memory_db;
use crate::parser::AstNode;
use crate::server::auth_pages::{generate_login_page, generate_register_page};
use crate::server::docs::{render_auto_docs, render_design_system, render_graph_page};
use crate::server::response::{html_response, json_response};
use crate::server::state::{
    current_time_hms, generate_request_id, iso_timestamp, AppState, RequestTrace,
};
use crate::{
    access, actions, api_crud, auth, authz, block_explorer, cli, constitution_check, database,
    graph, graphql, hmr, http_guard, hydra, parser, payments, scripting, security, server, session,
    sse, trust, ui, zeus, AUDIT_CANVAS, DEBUG_MODE,
};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::{Method, Request, Response, StatusCode};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use std::sync::Arc;

mod audit_log;
mod billing;
mod devtools;
mod diagnostics;
mod forms;
mod gql;
mod introspection;
mod pages;
mod rest;
mod scripts;
mod sessions;
mod throttle;

/// A route group's verdict: answered, or not this group's request.
type Routed = Result<Response<Full<Bytes>>, Request<Incoming>>;

/// Request facts every route group reads.
struct Ctx {
    state: Arc<AppState>,
    method: Method,
    /// Trailing slash removed (`/portal/` → `/portal`; `/` stays).
    path: String,
    query: String,
    remote_addr: SocketAddr,
}

/// Answer from a route group, or continue with the request it handed back.
macro_rules! next {
    ($routed:expr) => {
        match $routed {
            Ok(resp) => return Ok(resp),
            Err(req) => req,
        }
    };
}

/// Response body of the connection service: buffered, or the SSE stream.
pub(crate) type ServeBody = http_body_util::Either<
    Full<Bytes>,
    http_body_util::combinators::UnsyncBoxBody<Bytes, std::convert::Infallible>,
>;

/// The per-request service run by the server loop (and the HTTP test suite).
pub(crate) async fn serve(
    req: Request<Incoming>,
    state: Arc<AppState>,
    remote_addr: std::net::SocketAddr,
) -> Result<Response<ServeBody>, hyper::Error> {
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
            return Ok(Response::from_parts(
                parts,
                http_body_util::Either::Left(body),
            ));
        }
        let resp = cli::audit_http::audit_not_found();
        let (parts, body) = resp.into_parts();
        return Ok(Response::from_parts(
            parts,
            http_body_util::Either::Left(body),
        ));
    }
    if req.uri().path() == "/api/sse" && req.method() == Method::GET {
        // SECURITY: session required; events filtered per viewer.
        let sse_access = access::Access::from_headers(req.headers(), state.auth_entity.clone());
        let Some(is_admin) = sse_access.viewer.as_ref().map(|v| v.is_admin()) else {
            let resp = json_response(
                StatusCode::UNAUTHORIZED,
                authz::error_body("UNAUTHENTICATED", "Sign in required"),
            );
            let (parts, body) = resp.into_parts();
            return Ok(Response::from_parts(
                parts,
                http_body_util::Either::Left(body),
            ));
        };
        let sse_state = state.clone();
        let sse_resp = state.sse_hub.subscribe_filtered(
            move |ev| access::can_see_event(&sse_state.db, &sse_access, &sse_state.entities, ev),
            is_admin,
        );
        // Map the streaming body to a boxed body for type compatibility
        let (parts, body) = sse_resp.into_parts();
        let boxed = http_body_util::Either::Right(body.boxed_unsync());
        return Ok(Response::from_parts(parts, boxed));
    }
    // All other requests — wrap Full<Bytes> in Either::Left
    let resp = http_guard::isolate_panics(handle_request(req, state, remote_addr)).await?;
    let (parts, body) = resp.into_parts();
    Ok(Response::from_parts(
        parts,
        http_body_util::Either::Left(body),
    ))
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
    // In production the same pass removes the HMR client that layout-rendered
    // pages (404, source-page errors) carry outside `html_response`.
    let production = crate::http_guard::is_production();
    if (crate::security::script_nonces_enabled() || production)
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
        let mut html = String::from_utf8_lossy(&bytes).into_owned();
        if production {
            html = crate::server::response::strip_hmr_client(
                html,
                crate::security::script_nonce_attr(),
            );
        }
        let cleaned = crate::security::strip_script_nonce_markers(&html);
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
    remote_addr: SocketAddr,
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

    let ctx = Ctx {
        state,
        method,
        path,
        query,
        remote_addr,
    };

    let req = next!(devtools::route(req, &ctx));
    let req = next!(throttle::route(req, &ctx));
    // Brain: track every request
    let start = std::time::Instant::now();
    let req = next!(diagnostics::route(req, &ctx));
    let req = next!(sessions::route(req, &ctx).await);
    let req = next!(billing::route(req, &ctx));
    let req = next!(audit_log::route(req, &ctx).await);
    let req = next!(introspection::route(req, &ctx));
    let req = next!(gql::route(req, &ctx).await);
    let req = next!(scripts::route(req, &ctx).await);
    let req = next!(rest::route(req, &ctx, start).await);
    let req = next!(forms::route(req, &ctx).await);
    let _req = next!(pages::route(req, &ctx));
    pages::not_found(&ctx.state)
}
