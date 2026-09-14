//! End-to-end HTTP tests for the live dispatcher.
//!
//! Each test starts the real connection service (`crate::serve`, which runs
//! `handle_request` → `handle_request_inner`) on a loopback socket and talks
//! to it with a hyper HTTP/1 client, so bodies arrive as real `Incoming`
//! streams and every guard (CSRF, internal routes, rate limits, body limit)
//! runs in its production order. This is the safety net for splitting the
//! dispatcher into route modules: it must pass before and after.
//!
//! The process-wide run policy is never installed here, so it is the default
//! loopback dev policy. `AUDIT_CANVAS` is a global flag: the one test that
//! sets it holds the write side of `GLOBALS`; every other test holds the read
//! side.

use crate::auth::{create_token, default_secret};
use crate::parser::{parse, AstNode};
use crate::server::state::AppState;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{HeaderMap, Method, Request, StatusCode};
use hyper_util::rt::TokioIo;
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::net::{TcpListener, TcpStream};

static GLOBALS: RwLock<()> = RwLock::new(());

fn shared_globals() -> RwLockReadGuard<'static, ()> {
    GLOBALS.read().unwrap_or_else(|e| e.into_inner())
}

fn exclusive_globals() -> RwLockWriteGuard<'static, ()> {
    GLOBALS.write().unwrap_or_else(|e| e.into_inner())
}

const SRC: &str = r#"app "Dispatch" { port 5175 }
auth {
  entity User
  login email + password
  session jwt
  roles [admin, user]
}
entity User {
  name string
  email email!
  role string
  password string sensitive
}
entity Note {
  title string!
  secret string sensitive
}
api /notes {
  list   GET    /    auth:jwt
  create POST   /    auth:jwt
  detail GET    /:id auth:jwt
}
page "/about" type:custom {
}
page "/dash" type:custom requires:auth {
  section form { bind Note { query all } }
  section card {
    bind Note { query all }
    item "Archive" {
      on click {
        toast "Archived" success
      }
    }
  }
}
"#;

/// Full app state (pages, components, layout, protected routes) for `src`.
fn app_state(src: &str) -> AppState {
    let mut state = crate::api_security_tests::state_from(src);
    for node in parse(src).expect("parse test app") {
        match node {
            AstNode::Page(p) => {
                let requires = p
                    .requires
                    .clone()
                    .or_else(|| p.config.get("requires").cloned());
                if let Some(r) = requires {
                    state.auth_required_pages.push((p.route.clone(), r));
                }
                state.pages.push(p);
            }
            AstNode::Component(c) => state.components.push(c),
            AstNode::Layout(l) => state.layout = Some(l),
            _ => {}
        }
    }
    state
}

async fn spawn(state: AppState) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    let state = Arc::new(state);
    tokio::spawn(async move {
        loop {
            let Ok((stream, peer)) = listener.accept().await else {
                continue;
            };
            let st = state.clone();
            tokio::spawn(async move {
                let svc = hyper::service::service_fn(move |req| {
                    crate::routes::serve(req, st.clone(), peer)
                });
                let _ = crate::http_guard::http1_builder()
                    .serve_connection(TokioIo::new(stream), svc)
                    .await;
            });
        }
    });
    addr
}

struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    text: String,
}

impl Reply {
    fn json(&self) -> Value {
        serde_json::from_str(&self.text).unwrap_or(Value::Null)
    }
    fn header(&self, name: &str) -> String {
        self.headers
            .get_all(name)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

async fn send(
    addr: SocketAddr,
    method: Method,
    target: &str,
    headers: &[(&str, &str)],
    body: Vec<u8>,
) -> Reply {
    let resp = start(addr, method, target, headers, body).await;
    let status = resp.status();
    let headers = resp.headers().clone();
    let bytes = resp.into_body().collect().await.expect("body").to_bytes();
    Reply {
        status,
        headers,
        text: String::from_utf8_lossy(&bytes).to_string(),
    }
}

/// Sends a request and returns the response with its body unread.
async fn start(
    addr: SocketAddr,
    method: Method,
    target: &str,
    headers: &[(&str, &str)],
    body: Vec<u8>,
) -> hyper::Response<hyper::body::Incoming> {
    let stream = TcpStream::connect(addr).await.expect("connect");
    let (mut sender, conn) = hyper::client::conn::http1::handshake(TokioIo::new(stream))
        .await
        .expect("handshake");
    tokio::spawn(conn);
    let host = addr.to_string();
    let mut req = Request::builder()
        .method(method)
        .uri(target)
        .header("host", host.as_str());
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    sender
        .send_request(req.body(Full::new(Bytes::from(body))).expect("request"))
        .await
        .expect("response")
}

async fn get(addr: SocketAddr, target: &str, headers: &[(&str, &str)]) -> Reply {
    send(addr, Method::GET, target, headers, Vec::new()).await
}

async fn post_json(addr: SocketAddr, target: &str, headers: &[(&str, &str)], body: Value) -> Reply {
    let mut h = vec![("content-type", "application/json")];
    h.extend_from_slice(headers);
    send(
        addr,
        Method::POST,
        target,
        &h,
        body.to_string().into_bytes(),
    )
    .await
}

fn bearer(id: &str, role: &str) -> String {
    format!("Bearer {}", create_token(id, role, &default_secret()))
}

fn cookie(id: &str, role: &str) -> String {
    format!("cronus_token={}", create_token(id, role, &default_secret()))
}

fn error_code(reply: &Reply) -> String {
    reply.json()["error"]["code"]
        .as_str()
        .unwrap_or("")
        .to_string()
}

// ══════════════════════════════════════════════════
// Sessions
// ══════════════════════════════════════════════════

#[tokio::test]
async fn signup_and_login_set_httponly_cookie_without_body_token() {
    let _g = shared_globals();
    let addr = spawn(app_state(SRC)).await;
    let email = "dispatch-signup@example.test";
    let password = "correct horse battery staple";

    let signup = post_json(
        addr,
        "/api/auth/signup",
        &[],
        json!({"name": "Ana", "email": email, "password": password}),
    )
    .await;
    assert_eq!(signup.status, StatusCode::CREATED, "{}", signup.text);
    let set_cookie = signup.header("set-cookie");
    assert!(set_cookie.starts_with("cronus_token="), "{set_cookie}");
    assert!(set_cookie.contains("HttpOnly"));
    assert!(signup.json().get("token").is_none(), "{}", signup.text);

    let login = post_json(
        addr,
        "/api/auth/login",
        &[],
        json!({"email": email, "password": password}),
    )
    .await;
    assert_eq!(login.status, StatusCode::OK, "{}", login.text);
    assert!(login.json().get("token").is_none());
    let session = login.header("set-cookie");
    let token_pair = session.split(';').next().expect("cookie pair").to_string();
    assert!(token_pair.len() > "cronus_token=".len());

    let me = get(addr, "/api/auth/me", &[("cookie", &token_pair)]).await;
    assert_eq!(me.status, StatusCode::OK, "{}", me.text);
    assert_eq!(me.json()["id"], login.json()["user"]["id"]);

    let anon_me = get(addr, "/api/auth/me", &[]).await;
    assert_eq!(anon_me.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn csrf_gate_rejects_foreign_origin_cookie_post_before_handlers() {
    let _g = shared_globals();
    let state = app_state(SRC);
    let addr = spawn(state).await;
    let alice = cookie("alice", "user");

    let evil = post_json(
        addr,
        "/api/notes",
        &[("cookie", &alice), ("origin", "https://evil.test")],
        json!({"title": "forged"}),
    )
    .await;
    assert_eq!(evil.status, StatusCode::FORBIDDEN);
    assert_eq!(error_code(&evil), "CSRF_REJECTED");

    // Same gate covers forms and GraphQL.
    let evil_form = post_json(
        addr,
        "/_form/form",
        &[("cookie", &alice), ("origin", "https://evil.test")],
        json!({"entity": "Note", "data": {"title": "x"}}),
    )
    .await;
    assert_eq!(error_code(&evil_form), "CSRF_REJECTED");

    let origin = format!("http://{addr}");
    let same = post_json(
        addr,
        "/api/notes",
        &[("cookie", &alice), ("origin", &origin)],
        json!({"title": "mine"}),
    )
    .await;
    assert_eq!(same.status, StatusCode::CREATED, "{}", same.text);
}

// ══════════════════════════════════════════════════
// REST / GraphQL / SSE / forms / actions
// ══════════════════════════════════════════════════

#[tokio::test]
async fn rest_create_and_list_are_owner_scoped() {
    let _g = shared_globals();
    let addr = spawn(app_state(SRC)).await;
    let alice = bearer("alice", "user");
    let bob = bearer("bob", "user");

    let anon = get(addr, "/api/notes", &[]).await;
    assert_eq!(anon.status, StatusCode::UNAUTHORIZED);

    let created = post_json(
        addr,
        "/api/notes",
        &[("authorization", &alice)],
        json!({"title": "a1", "secret": "hidden"}),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text);
    assert!(!created.text.contains("hidden"));

    let mine = get(addr, "/api/notes", &[("authorization", &alice)]).await;
    assert_eq!(mine.status, StatusCode::OK);
    assert_eq!(
        mine.json().as_array().map(Vec::len),
        Some(1),
        "{}",
        mine.text
    );

    let theirs = get(addr, "/api/notes", &[("authorization", &bob)]).await;
    assert_eq!(theirs.status, StatusCode::OK);
    assert_eq!(theirs.json().as_array().map(Vec::len), Some(0));

    let id = created.json()["id"].as_str().expect("id").to_string();
    let peek = get(
        addr,
        &format!("/api/notes/{id}"),
        &[("authorization", &bob)],
    )
    .await;
    assert_eq!(peek.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn graphql_sse_forms_and_actions_require_a_session() {
    let _g = shared_globals();
    let addr = spawn(app_state(SRC)).await;

    let gql = post_json(addr, "/graphql", &[], json!({"query": "{ notes { id } }"})).await;
    assert_eq!(gql.status, StatusCode::UNAUTHORIZED);
    let schema = get(addr, "/graphql/schema", &[]).await;
    assert_eq!(schema.status, StatusCode::UNAUTHORIZED);
    let playground = get(addr, "/graphql", &[]).await;
    assert_eq!(playground.status, StatusCode::OK);

    let gql_ok = post_json(
        addr,
        "/graphql",
        &[("authorization", &bearer("alice", "user"))],
        json!({"query": "{ notes { id } }"}),
    )
    .await;
    assert_eq!(gql_ok.status, StatusCode::OK);

    let sse = get(addr, "/api/sse", &[]).await;
    assert_eq!(sse.status, StatusCode::UNAUTHORIZED);
    let sse_ok = start(
        addr,
        Method::GET,
        "/api/sse",
        &[("authorization", &bearer("alice", "user"))],
        Vec::new(),
    )
    .await;
    assert_eq!(sse_ok.status(), StatusCode::OK);
    let ct = sse_ok
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    assert!(ct.starts_with("text/event-stream"), "{ct}");
    drop(sse_ok);

    let form = post_json(
        addr,
        "/_form/form",
        &[],
        json!({"entity": "Note", "data": {"title": "x"}}),
    )
    .await;
    assert_eq!(form.status, StatusCode::UNAUTHORIZED, "{}", form.text);
    assert_eq!(error_code(&form), "UNAUTHENTICATED");

    let form_ok = post_json(
        addr,
        "/_form/form",
        &[("authorization", &bearer("alice", "user"))],
        json!({"entity": "Note", "data": {"title": "x"}}),
    )
    .await;
    assert!(form_ok.status.is_success(), "{}", form_ok.text);

    let action = post_json(
        addr,
        "/_action/card",
        &[],
        json!({"action_id": "x", "entity": "Note", "id": "1"}),
    )
    .await;
    assert_eq!(action.status, StatusCode::UNAUTHORIZED, "{}", action.text);
    let undeclared = post_json(
        addr,
        "/_action/card",
        &[("authorization", &bearer("alice", "user"))],
        json!({"action_id": "deadbeef", "entity": "Note", "id": "1"}),
    )
    .await;
    assert_eq!(undeclared.status, StatusCode::FORBIDDEN);
}

// ══════════════════════════════════════════════════
// Pages
// ══════════════════════════════════════════════════

#[tokio::test]
async fn protected_page_redirects_and_public_page_renders() {
    let _g = shared_globals();
    let addr = spawn(app_state(SRC)).await;

    let about = get(addr, "/about", &[]).await;
    assert_eq!(about.status, StatusCode::OK);
    assert!(about.header("content-type").starts_with("text/html"));
    // Trailing slash is normalized.
    assert_eq!(get(addr, "/about/", &[]).await.status, StatusCode::OK);

    let dash = get(addr, "/dash", &[]).await;
    assert_eq!(dash.status, StatusCode::FOUND);
    assert_eq!(dash.header("location"), "/login");

    let dash_in = get(addr, "/dash", &[("cookie", &cookie("alice", "user"))]).await;
    assert_eq!(dash_in.status, StatusCode::OK);

    // Auto auth pages exist because the app has an auth block.
    assert_eq!(get(addr, "/login", &[]).await.status, StatusCode::OK);
    assert_eq!(get(addr, "/register", &[]).await.status, StatusCode::OK);
    let logout = get(addr, "/logout", &[]).await;
    assert_eq!(logout.status, StatusCode::FOUND);
    assert!(logout.header("set-cookie").contains("Max-Age=0"));

    // Built-in component catalog when no /components page is declared.
    assert_eq!(get(addr, "/components", &[]).await.status, StatusCode::OK);
}

#[tokio::test]
async fn not_found_page_does_not_echo_the_path() {
    let _g = shared_globals();
    let addr = spawn(app_state(SRC)).await;
    let marker = "zz-no-such-route-7c1f";
    let page = get(addr, &format!("/{marker}"), &[]).await;
    assert_eq!(page.status, StatusCode::NOT_FOUND);
    assert!(page.header("content-type").starts_with("text/html"));
    assert!(!page.text.contains(marker));

    let api = get(
        addr,
        &format!("/api/{marker}"),
        &[("authorization", &bearer("alice", "user"))],
    )
    .await;
    assert_eq!(api.status, StatusCode::NOT_FOUND);
    assert!(!api.text.contains(marker), "{}", api.text);
}

#[tokio::test]
async fn source_page_serves_file_and_hides_io_errors() {
    let _g = shared_globals();
    let dir = std::env::temp_dir().join(format!("cronus-dispatch-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("tmp dir");
    let file = dir.join("raw.html");
    std::fs::write(&file, "<html><body>raw-source-ok</body></html>").expect("write");
    let missing = dir.join("missing-secret-path.html");

    let mut state = app_state(SRC);
    let mut raw = state.pages[0].clone();
    raw.route = "/raw".into();
    raw.config
        .insert("source".into(), file.to_string_lossy().to_string());
    let mut broken = raw.clone();
    broken.route = "/broken".into();
    broken
        .config
        .insert("source".into(), missing.to_string_lossy().to_string());
    state.pages.push(raw);
    state.pages.push(broken);
    let addr = spawn(state).await;

    let ok = get(addr, "/raw", &[]).await;
    assert_eq!(ok.status, StatusCode::OK);
    assert!(ok.text.contains("raw-source-ok"));

    let err = get(addr, "/broken", &[]).await;
    assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(!err.text.contains("missing-secret-path"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ══════════════════════════════════════════════════
// http_guard: internal routes, body limit, rate limits, audit canvas
// ══════════════════════════════════════════════════

#[tokio::test]
async fn internal_routes_follow_the_run_policy() {
    let _g = shared_globals();
    let addr = spawn(app_state(SRC)).await;

    // Loopback dev (the default policy): dev tooling is served.
    assert_eq!(get(addr, "/zeus/api", &[]).await.status, StatusCode::OK);
    assert_eq!(get(addr, "/api/schema", &[]).await.status, StatusCode::OK);
    assert_eq!(
        get(addr, "/.cronus/version", &[]).await.status,
        StatusCode::OK
    );
    assert_eq!(get(addr, "/api/health", &[]).await.status, StatusCode::OK);

    // Audit trail is admin-only in every mode.
    let trail = get(addr, "/api/audit/trail", &[]).await;
    assert_eq!(trail.status, StatusCode::UNAUTHORIZED);
    let trail_user = get(
        addr,
        "/api/audit/trail",
        &[("authorization", &bearer("alice", "user"))],
    )
    .await;
    assert_eq!(trail_user.status, StatusCode::FORBIDDEN);
    let trail_admin = get(
        addr,
        "/api/audit/trail",
        &[("authorization", &bearer("root", "admin"))],
    )
    .await;
    assert_eq!(trail_admin.status, StatusCode::OK);

    // Production: the same gate the dispatcher calls 404s internal routes,
    // even for admins. (The installed policy is process-wide, so production
    // is checked on the gate rather than by reinstalling it.)
    use crate::http_guard::{classify, gate, Gate, Policy, RunMode};
    let prod = Policy {
        mode: RunMode::Production,
        ..Policy::default()
    };
    for path in ["/zeus", "/api/_context", "/docs", "/graphql/schema"] {
        assert_eq!(
            gate(classify(&Method::GET, path), &prod, Some("admin")),
            Gate::NotFound,
            "{path}"
        );
    }

    // CORS preflight short-circuits.
    let pre = send(addr, Method::OPTIONS, "/api/notes", &[], Vec::new()).await;
    assert_eq!(pre.status, StatusCode::OK);
}

#[tokio::test]
async fn oversized_body_is_413() {
    let _g = shared_globals();
    let addr = spawn(app_state(SRC)).await;
    let limit = crate::http_guard::policy().max_body_bytes;
    let big = vec![b'a'; limit + 1];
    let alice = bearer("alice", "user");
    let resp = send(
        addr,
        Method::POST,
        "/api/notes",
        &[
            ("authorization", &alice),
            ("content-type", "application/json"),
        ],
        big,
    )
    .await;
    assert_eq!(resp.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(error_code(&resp), "PAYLOAD_TOO_LARGE");
}

#[tokio::test]
async fn api_and_auth_rate_limits_apply() {
    let _g = shared_globals();
    let mut state = app_state(SRC);
    state.rate_limiter = crate::rate_limit::RateLimiter::new(2, 60);
    state.auth_rate_limiter = crate::rate_limit::RateLimiter::new(1, 60);
    let addr = spawn(state).await;

    assert_ne!(
        get(addr, "/api/health", &[]).await.status,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_ne!(
        get(addr, "/api/health", &[]).await.status,
        StatusCode::TOO_MANY_REQUESTS
    );
    let limited = get(addr, "/api/health", &[]).await;
    assert_eq!(limited.status, StatusCode::TOO_MANY_REQUESTS);
    assert!(!limited.header("retry-after").is_empty());
    // Pages are not rate limited.
    assert_eq!(get(addr, "/about", &[]).await.status, StatusCode::OK);

    let body = json!({"email": "rl@example.test", "password": "nope"});
    let first = post_json(addr, "/api/auth/login", &[], body.clone()).await;
    assert_ne!(first.status, StatusCode::TOO_MANY_REQUESTS);
    let second = post_json(addr, "/api/auth/login", &[], body).await;
    assert_eq!(second.status, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn audit_canvas_is_exclusive() {
    let _g = exclusive_globals();
    let addr = spawn(app_state(SRC)).await;
    crate::AUDIT_CANVAS.store(true, Ordering::Relaxed);
    let about = get(addr, "/about", &[]).await;
    let health = get(addr, "/api/health", &[]).await;
    let sse = get(addr, "/api/sse", &[]).await;
    crate::AUDIT_CANVAS.store(false, Ordering::Relaxed);

    assert_eq!(about.status, StatusCode::NOT_FOUND);
    assert_eq!(health.status, StatusCode::NOT_FOUND);
    assert_eq!(sse.status, StatusCode::NOT_FOUND);
    assert_eq!(get(addr, "/about", &[]).await.status, StatusCode::OK);
}

#[tokio::test]
async fn unmatched_webhook_path_falls_through_to_404() {
    let _g = shared_globals();
    let addr = spawn(app_state(SRC)).await;
    let hook = post_json(addr, "/hooks/unknown", &[], json!({})).await;
    assert_eq!(hook.status, StatusCode::NOT_FOUND);
    let resp = get(addr, "/about", &[]).await;
    assert!(!resp.header("x-request-id").is_empty());
    assert!(resp.header("x-response-time").ends_with("ms"));
}
