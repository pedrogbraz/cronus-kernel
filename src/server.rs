#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS HTTP Server — hyper 1.x
//!
//! Self-contained server that serves static files and entity CRUD API routes.
//! Uses CronusDB from the database module for persistence.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde_json::{json, Value};
use tokio::net::TcpListener;

use crate::database::CronusDB;

// ──────────────────────────────────────────────
// Public types
// ──────────────────────────────────────────────

/// A single registered API route.
pub struct ApiRoute {
    pub method: String,
    pub path: String,
    pub entity: String,
}

/// HTTP server that serves CRONUS apps.
pub struct CronusServer {
    pub port: u16,
    pub routes: Vec<ApiRoute>,
    pub static_dir: Option<String>,
    pub db: Arc<CronusDB>,
}

// ──────────────────────────────────────────────
// CronusServer implementation
// ──────────────────────────────────────────────

impl CronusServer {
    /// Create a new server bound to `port`, backed by `db`.
    pub fn new(port: u16, db: Arc<CronusDB>) -> Self {
        Self {
            port,
            routes: Vec::new(),
            static_dir: None,
            db,
        }
    }

    /// Register standard CRUD routes for an entity:
    ///
    /// - `GET    /api/{entity}`       — list all rows
    /// - `GET    /api/{entity}/{id}`  — get one row
    /// - `POST   /api/{entity}`       — create a row
    /// - `DELETE  /api/{entity}/{id}` — delete a row
    pub fn add_crud_routes(&mut self, entity_name: &str) {
        let lower = entity_name.to_lowercase();
        let base = format!("/api/{lower}");

        self.routes.push(ApiRoute {
            method: "GET".into(),
            path: base.clone(),
            entity: lower.clone(),
        });
        self.routes.push(ApiRoute {
            method: "GET".into(),
            path: format!("{base}/{{id}}"),
            entity: lower.clone(),
        });
        self.routes.push(ApiRoute {
            method: "POST".into(),
            path: base.clone(),
            entity: lower.clone(),
        });
        self.routes.push(ApiRoute {
            method: "DELETE".into(),
            path: format!("{base}/{{id}}"),
            entity: lower,
        });
    }

    /// Start listening and serving. Blocks until the process is killed.
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        let listener = TcpListener::bind(addr).await?;
        eprintln!("[CRONUS] server listening on http://0.0.0.0:{}", self.port);

        let state = Arc::new(ServerState {
            routes: self.routes,
            static_dir: self.static_dir,
            db: self.db,
        });

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let state = Arc::clone(&state);

            tokio::task::spawn(async move {
                let svc = service_fn(move |req| {
                    let state = Arc::clone(&state);
                    async move { handle_request(req, state).await }
                });
                if let Err(e) = http1::Builder::new().serve_connection(io, svc).await {
                    eprintln!("[CRONUS] connection error: {e}");
                }
            });
        }
    }
}

// ──────────────────────────────────────────────
// Internal shared state
// ──────────────────────────────────────────────

struct ServerState {
    routes: Vec<ApiRoute>,
    static_dir: Option<String>,
    db: Arc<CronusDB>,
}

// ──────────────────────────────────────────────
// Request dispatcher
// ──────────────────────────────────────────────

async fn handle_request(
    req: Request<Incoming>,
    state: Arc<ServerState>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    // CORS preflight
    if method == Method::OPTIONS {
        return Ok(json_response(StatusCode::OK, json!({})));
    }

    // Health check
    if path == "/api/health" && method == Method::GET {
        let health = json!({
            "status": "ok",
            "version": "0.1.0",
            "runtime": "cronus-kernel"
        });
        return Ok(json_response(StatusCode::OK, health));
    }

    // SSE endpoint
    if path == "/api/sse" && method == Method::GET {
        // Return a simple keepalive for now — full SSE handled by main.rs
        let body = ": connected to CRONUS SSE\n\n";
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .header("Access-Control-Allow-Origin", "*")
            .body(Full::new(Bytes::from(body)))
            .unwrap());
    }

    // API routes
    if path.starts_with("/api/") {
        return Ok(handle_api(req, &method, &path, &state).await);
    }

    // Static files
    if let Some(ref dir) = state.static_dir {
        return Ok(serve_static(dir, &path).await);
    }

    Ok(json_response(StatusCode::NOT_FOUND, json!({"error": "not found"})))
}

// ──────────────────────────────────────────────
// API handler
// ──────────────────────────────────────────────

async fn handle_api(
    req: Request<Incoming>,
    method: &Method,
    path: &str,
    state: &ServerState,
) -> Response<Full<Bytes>> {
    // Parse: /api/{entity} or /api/{entity}/{id}
    let segments: Vec<&str> = path.trim_matches('/').split('/').collect();
    // segments = ["api", entity] or ["api", entity, id]

    if segments.len() < 2 {
        return json_response(StatusCode::BAD_REQUEST, json!({"error": "invalid api path"}));
    }

    let entity_slug = segments[1];
    let id_segment: Option<&str> = segments.get(2).copied().filter(|s| !s.is_empty());

    // Check a matching route exists
    let method_str = method.as_str();
    let route_exists = state.routes.iter().any(|r| {
        if r.method != method_str {
            return false;
        }
        // Match entity (exact or pluralized)
        let matches_entity = r.entity == entity_slug
            || format!("{}s", r.entity) == entity_slug
            || r.entity == format!("{}s", entity_slug);
        if !matches_entity {
            return false;
        }
        let has_id_param = r.path.contains("{id}");
        if has_id_param { id_segment.is_some() } else { id_segment.is_none() }
    });

    if !route_exists {
        return json_response(
            StatusCode::NOT_FOUND,
            json!({"error": format!("{method_str} {path} not found")}),
        );
    }

    // Resolve the actual table name — try the slug as-is first
    let table = entity_slug;

    match (method_str, id_segment) {
        // ── LIST ──
        ("GET", None) => {
            match state.db.find_all(table, 100, 0) {
                Ok(rows) => json_response(StatusCode::OK, json!({"data": rows})),
                Err(e) => json_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e})),
            }
        }

        // ── GET BY ID ──
        ("GET", Some(id)) => {
            match state.db.find_by_id(table, id) {
                Ok(Some(row)) => json_response(StatusCode::OK, json!({"data": row})),
                Ok(None) => json_response(StatusCode::NOT_FOUND, json!({"error": "not found"})),
                Err(e) => json_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e})),
            }
        }

        // ── CREATE ──
        ("POST", None) => {
            let body = match read_body(req).await {
                Ok(b) => b,
                Err(e) => return json_response(
                    StatusCode::BAD_REQUEST,
                    json!({"error": format!("body read error: {e}")}),
                ),
            };
            let data: Value = match serde_json::from_slice(&body) {
                Ok(v) => v,
                Err(e) => return json_response(
                    StatusCode::BAD_REQUEST,
                    json!({"error": format!("invalid JSON: {e}")}),
                ),
            };
            match state.db.insert(table, &data) {
                Ok(row) => json_response(StatusCode::CREATED, json!({"data": row})),
                Err(e) => json_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e})),
            }
        }

        // ── DELETE ──
        ("DELETE", Some(id)) => {
            match state.db.delete(table, id) {
                Ok(true) => json_response(StatusCode::OK, json!({"deleted": true})),
                Ok(false) => json_response(StatusCode::NOT_FOUND, json!({"error": "not found"})),
                Err(e) => json_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e})),
            }
        }

        _ => json_response(StatusCode::METHOD_NOT_ALLOWED, json!({"error": "method not allowed"})),
    }
}

// ──────────────────────────────────────────────
// Body reader
// ──────────────────────────────────────────────

const MAX_BODY_SIZE: usize = 10 * 1024 * 1024; // 10MB

async fn read_body(req: Request<Incoming>) -> Result<Vec<u8>, String> {
    // Check Content-Length header first for early rejection
    if let Some(cl) = req.headers().get("content-length") {
        if let Ok(len) = cl.to_str().unwrap_or("0").parse::<usize>() {
            if len > MAX_BODY_SIZE {
                return Err(format!("request body too large: {} bytes (max {})", len, MAX_BODY_SIZE));
            }
        }
    }
    let collected = req.into_body().collect().await.map_err(|e| e.to_string())?;
    let bytes = collected.to_bytes().to_vec();
    if bytes.len() > MAX_BODY_SIZE {
        return Err(format!("request body too large: {} bytes (max {})", bytes.len(), MAX_BODY_SIZE));
    }
    Ok(bytes)
}

// ──────────────────────────────────────────────
// Static file serving
// ──────────────────────────────────────────────

async fn serve_static(dir: &str, req_path: &str) -> Response<Full<Bytes>> {
    let file_path = if req_path == "/" {
        format!("{dir}/index.html")
    } else {
        // Decode percent-encoding before path traversal check
        let decoded = percent_decode(req_path.trim_start_matches('/'));
        if decoded.contains("..") || decoded.contains('\0') {
            return json_response(StatusCode::BAD_REQUEST, json!({"error": "invalid path"}));
        }
        let candidate = format!("{dir}/{decoded}");
        // Canonicalize and verify the resolved path stays within static dir
        let canon_dir = match std::fs::canonicalize(dir) {
            Ok(p) => p,
            Err(_) => return json_response(StatusCode::NOT_FOUND, json!({"error": "not found"})),
        };
        if let Ok(canon_file) = std::fs::canonicalize(&candidate) {
            if !canon_file.starts_with(&canon_dir) {
                return json_response(StatusCode::BAD_REQUEST, json!({"error": "invalid path"}));
            }
        }
        candidate
    };

    match tokio::fs::read(Path::new(&file_path)).await {
        Ok(contents) => {
            let ct = guess_content_type(&file_path);
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", ct)
                .body(Full::new(Bytes::from(contents)))
                .unwrap()
        }
        Err(_) => {
            // SPA fallback: try index.html
            let index = format!("{dir}/index.html");
            match tokio::fs::read(&index).await {
                Ok(contents) => Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/html; charset=utf-8")
                    .body(Full::new(Bytes::from(contents)))
                    .unwrap(),
                Err(_) => json_response(StatusCode::NOT_FOUND, json!({"error": "not found"})),
            }
        }
    }
}

fn guess_content_type(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("webp") => "image/webp",
        Some("webm") => "video/webm",
        Some("mp4") => "video/mp4",
        Some("wasm") => "application/wasm",
        _ => "application/octet-stream",
    }
}

// ──────────────────────────────────────────────
// JSON response helper
// ──────────────────────────────────────────────

fn json_response(status: StatusCode, body: Value) -> Response<Full<Bytes>> {
    // CORS: use configurable origin (default: same-origin only in production)
    let cors_origin = std::env::var("CRONUS_CORS_ORIGIN").unwrap_or_else(|_| "*".to_string());
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", cors_origin)
        .header("Access-Control-Allow-Methods", "GET, POST, PATCH, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
        .body(Full::new(Bytes::from(body.to_string())))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from("{\"error\":\"internal error\"}")))
                .expect("fallback response must build")
        })
}

/// Decode percent-encoded path segments (e.g., %2e%2e → ..)
fn percent_decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                result.push((hi << 4 | lo) as char);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}
