//! Browser sessions: the `cronus_token` HttpOnly cookie, the CSRF origin
//! check for cookie-authenticated mutations, and the `/api/auth/*` routes.
//!
//! Contract:
//! - Login/signup set the session only as an `HttpOnly; SameSite=Lax` cookie
//!   (`Secure` in production or behind an HTTPS proxy). The JWT is returned
//!   in the body only when a non-browser client asks for it with `?token=1`
//!   or `X-Cronus-Session-Token: 1`.
//! - Cookie Max-Age equals the token lifetime from `session jwt expires:<dur>`.
//! - Any unsafe request that carries the session cookie (and no bearer
//!   header) must present an `Origin`/`Referer` of this host. This single gate
//!   runs before every handler in `handle_request_inner`, so it covers every
//!   cookie identity reader: `access::viewer_from_headers`,
//!   `api_crud::claims_from_headers` and `http_guard::request_role`.

use bytes::Bytes;
use http_body_util::Full;
use hyper::header::{HeaderValue, SET_COOKIE};
use hyper::{HeaderMap, Method, Response, StatusCode};
use serde_json::{json, Value};

use crate::auth;
use crate::authz::error_body;
use crate::http_guard;
use crate::server::response::json_response;
use crate::server::state::AppState;

pub const SESSION_COOKIE: &str = "cronus_token";
pub const TOKEN_RESPONSE_HEADER: &str = "x-cronus-session-token";

// ══════════════════════════════════════════════════
// Cookie
// ══════════════════════════════════════════════════

/// `Secure` when the server runs in production (`--prod` /
/// `CRONUS_ENV=production`) or the request reached us over HTTPS through a
/// proxy. Spoofing the forwarded header can only make a cookie stricter.
pub fn cookie_secure(mode: http_guard::RunMode, headers: &HeaderMap) -> bool {
    mode == http_guard::RunMode::Production
        || headers
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            .map(|p| p.split(',').next().unwrap_or("").trim().eq_ignore_ascii_case("https"))
            .unwrap_or(false)
}

pub fn session_cookie(token: &str, max_age_secs: u64, secure: bool) -> String {
    format!(
        "{}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}{}",
        SESSION_COOKIE,
        token,
        max_age_secs,
        if secure { "; Secure" } else { "" }
    )
}

pub fn clear_session_cookie(secure: bool) -> String {
    session_cookie("", 0, secure)
}

fn cookie_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all("cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|c| c.split(';'))
        .find_map(|part| part.trim().strip_prefix("cronus_token=").map(str::to_string))
        .filter(|t| !t.is_empty())
}

fn with_cookie(mut resp: Response<Full<Bytes>>, cookie: &str) -> Response<Full<Bytes>> {
    if let Ok(v) = HeaderValue::from_str(cookie) {
        resp.headers_mut().append(SET_COOKIE, v);
    }
    resp
}

// ══════════════════════════════════════════════════
// CSRF
// ══════════════════════════════════════════════════

fn authority_of(url: &str) -> Option<String> {
    let rest = url
        .strip_prefix("https://")
        .map(|r| (r, "443"))
        .or_else(|| url.strip_prefix("http://").map(|r| (r, "80")))?;
    let (rest, default_port) = rest;
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    if authority.is_empty() {
        return None;
    }
    Some(normalize_authority(authority, default_port))
}

fn normalize_authority(authority: &str, default_port: &str) -> String {
    let lower = authority.trim().to_ascii_lowercase();
    lower
        .strip_suffix(&format!(":{default_port}"))
        .map(str::to_string)
        .unwrap_or(lower)
}

fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok()).map(str::trim).filter(|s| !s.is_empty())
}

/// True when this request must be refused as cross-site.
pub fn csrf_blocks(method: &Method, path: &str, headers: &HeaderMap) -> bool {
    if matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS) {
        return false;
    }
    // Browsers never attach `Authorization` to a cross-site request without
    // a CORS preflight, and bearer identity wins over the cookie downstream.
    let has_bearer = header_str(headers, "authorization")
        .map(|h| h.starts_with("Bearer "))
        .unwrap_or(false);
    if has_bearer {
        return false;
    }
    let has_cookie = cookie_token(headers).is_some();
    let session_route = path.starts_with("/api/auth/");
    if !has_cookie && !session_route {
        return false;
    }
    let source = match header_str(headers, "origin") {
        Some(origin) => authority_of(origin),
        None => match header_str(headers, "referer") {
            Some(referer) => authority_of(referer),
            // Non-browser client without a session cookie (curl, `cronus test`).
            None => return has_cookie,
        },
    };
    let Some(source) = source else { return true }; // `Origin: null` and friends
    let expected = ["host", "x-forwarded-host"]
        .iter()
        .filter_map(|h| header_str(headers, h))
        .map(|h| h.split(',').next().unwrap_or("").trim().to_ascii_lowercase());
    !expected
        .into_iter()
        .any(|host| host == source || normalize_authority(&host, "80") == source || normalize_authority(&host, "443") == source)
}

pub fn csrf_rejection(method: &Method, path: &str, headers: &HeaderMap) -> Option<Response<Full<Bytes>>> {
    if csrf_blocks(method, path, headers) {
        Some(json_response(
            StatusCode::FORBIDDEN,
            error_body("CSRF_REJECTED", "cross-site request blocked"),
        ))
    } else {
        None
    }
}

// ══════════════════════════════════════════════════
// /api/auth/*
// ══════════════════════════════════════════════════

pub(crate) struct AuthOutcome {
    pub response: Response<Full<Bytes>>,
    /// Account whose login attempt must be recorded for backoff.
    pub login_account: Option<String>,
}

impl From<Response<Full<Bytes>>> for AuthOutcome {
    fn from(response: Response<Full<Bytes>>) -> Self {
        AuthOutcome { response, login_account: None }
    }
}

/// Table holding accounts: the entity named in `auth { entity X }`. Apps
/// without an auth block keep the legacy `User`/`Users` entity lookup.
pub(crate) fn auth_table(state: &AppState) -> Option<&str> {
    match state.auth_entity.as_deref() {
        Some(declared) => state
            .entities
            .iter()
            .find(|e| e.name.eq_ignore_ascii_case(declared))
            .map(|e| e.name.as_str()),
        None => state
            .entities
            .iter()
            .find(|e| {
                let lower = e.name.to_ascii_lowercase();
                lower == "user" || lower == "users"
            })
            .map(|e| e.name.as_str()),
    }
}

/// Token in the JSON body only on explicit request from a non-browser client.
pub fn wants_body_token(query: &str, headers: &HeaderMap) -> bool {
    query.split('&').any(|pair| pair == "token=1")
        || header_str(headers, TOKEN_RESPONSE_HEADER) == Some("1")
}

fn session_response(
    state: &AppState,
    status: StatusCode,
    user: Value,
    token: String,
    query: &str,
    headers: &HeaderMap,
) -> Response<Full<Bytes>> {
    let mut body = json!({ "user": user });
    if wants_body_token(query, headers) {
        body["token"] = json!(token);
    }
    let secure = cookie_secure(http_guard::policy().mode, headers);
    with_cookie(
        json_response(status, body),
        &session_cookie(&token, state.session_policy.ttl_secs, secure),
    )
}

pub(crate) fn handle_auth(
    state: &AppState,
    method: &Method,
    path: &str,
    query: &str,
    headers: &HeaderMap,
    body: &[u8],
) -> AuthOutcome {
    let secret = auth::default_secret();
    match (method, path) {
        (&Method::GET, "/api/auth/me") => {
            let claims = crate::access::token_from_headers(headers)
                .and_then(|t| auth::verify_token(&t, &secret).ok());
            match claims {
                Some(c) => json_response(
                    StatusCode::OK,
                    json!({"id": c.sub, "sub": c.sub, "role": c.role, "exp": c.exp}),
                ),
                None => json_response(StatusCode::UNAUTHORIZED, json!({"error": "unauthorized"})),
            }
            .into()
        }
        (&Method::POST, "/api/auth/logout") => {
            let secure = cookie_secure(http_guard::policy().mode, headers);
            with_cookie(json_response(StatusCode::OK, json!({"ok": true})), &clear_session_cookie(secure)).into()
        }
        (&Method::POST, "/api/auth/signup") => signup(state, query, headers, body).into(),
        (&Method::POST, "/api/auth/login") => login(state, query, headers, body),
        _ => json_response(
            StatusCode::OK,
            json!({"info": "auth endpoint", "routes": [
                "/api/auth/signup POST", "/api/auth/login POST",
                "/api/auth/logout POST", "/api/auth/me GET"
            ]}),
        )
        .into(),
    }
}

fn auth_not_configured() -> Response<Full<Bytes>> {
    json_response(StatusCode::NOT_FOUND, error_body("AUTH_NOT_CONFIGURED", "authentication is not configured"))
}

fn signup(state: &AppState, query: &str, headers: &HeaderMap, body: &[u8]) -> Response<Full<Bytes>> {
    let Some(table) = auth_table(state) else { return auth_not_configured() };
    let body: Value = serde_json::from_slice(body).unwrap_or(json!({}));
    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let email = body.get("email").and_then(|v| v.as_str()).unwrap_or("");
    let password = body.get("password").and_then(|v| v.as_str()).unwrap_or("");
    let requested_role = body.get("role").and_then(|v| v.as_str()).unwrap_or("user");
    // SECURITY: never allow privileged roles via self-registration.
    let forbidden_roles = ["admin", "superadmin", "root", "owner"];
    let role = if state.auth_roles.iter().any(|r| r == requested_role) && !forbidden_roles.contains(&requested_role) {
        requested_role.to_string()
    } else {
        state
            .auth_roles
            .iter()
            .find(|r| !forbidden_roles.contains(&r.as_str()))
            .cloned()
            .unwrap_or_else(|| "user".to_string())
    };

    if name.is_empty() || email.is_empty() || password.is_empty() {
        return json_response(StatusCode::BAD_REQUEST, json!({"error": "name, email and password required"}));
    }
    if let Err(message) = auth::validate_new_password(password) {
        return json_response(StatusCode::BAD_REQUEST, error_body("VALIDATION_FAILED", &message));
    }
    let exists = state.db.find_by_field(table, "email", email).ok().map(|o| o.is_some()).unwrap_or(false);
    if exists {
        return json_response(StatusCode::BAD_REQUEST, json!({"error": "email already registered"}));
    }
    let user_data = json!({
        "name": name,
        "email": email,
        "password": auth::hash_password(password),
        "role": &role
    });
    match state.db.insert(table, &user_data) {
        Ok(user) => {
            let user_id = user.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let token = auth::create_session_token(user_id, &role, &secret_or_default(), state.session_policy.ttl_secs);
            session_response(
                state,
                StatusCode::CREATED,
                json!({"id": user_id, "name": name, "email": email, "role": &role}),
                token,
                query,
                headers,
            )
        }
        Err(e) => {
            // Logged once here; the client never sees DB/SQL detail.
            eprintln!("[auth] signup insert into {} failed: {}", table, e);
            json_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": "could not create account"}))
        }
    }
}

fn secret_or_default() -> String {
    auth::default_secret()
}

fn login(state: &AppState, query: &str, headers: &HeaderMap, body: &[u8]) -> AuthOutcome {
    let Some(table) = auth_table(state) else { return auth_not_configured().into() };
    let body: Value = serde_json::from_slice(body).unwrap_or(json!({}));
    let email = body.get("email").and_then(|v| v.as_str()).unwrap_or("");
    let password = body.get("password").and_then(|v| v.as_str()).unwrap_or("");
    if let Some(locked) = http_guard::login_account_check(email) {
        return locked.into();
    }
    let login_account = Some(email.to_string());

    let response = if email.is_empty() || password.is_empty() {
        json_response(StatusCode::BAD_REQUEST, json!({"error": "email and password required"}))
    } else {
        match state.db.find_by_field(table, "email", email) {
            Ok(Some(u)) => {
                let stored = u.get("password").and_then(|v| v.as_str()).unwrap_or("");
                if auth::verify_password(password, stored) {
                    let user_id = u.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let role = u.get("role").and_then(|v| v.as_str()).unwrap_or("user");
                    let token = auth::create_session_token(user_id, role, &secret_or_default(), state.session_policy.ttl_secs);
                    session_response(
                        state,
                        StatusCode::OK,
                        json!({"id": user_id, "name": u.get("name"), "email": email, "role": role}),
                        token,
                        query,
                        headers,
                    )
                } else {
                    json_response(StatusCode::UNAUTHORIZED, json!({"error": "invalid credentials"}))
                }
            }
            Ok(None) => json_response(StatusCode::UNAUTHORIZED, json!({"error": "invalid credentials"})),
            Err(e) => {
                eprintln!("[auth] login lookup in {} failed: {}", table, e);
                json_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": "database error"}))
            }
        }
    };
    AuthOutcome { response, login_account }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&'static str, &str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (k, v) in pairs {
            h.append(*k, v.parse().unwrap());
        }
        h
    }

    #[test]
    fn cookie_is_httponly_lax_and_secure_only_when_required() {
        let dev = session_cookie("tok", 3600, false);
        assert_eq!(dev, "cronus_token=tok; Path=/; HttpOnly; SameSite=Lax; Max-Age=3600");
        assert!(session_cookie("tok", 3600, true).ends_with("; Secure"));
        assert!(clear_session_cookie(true).contains("Max-Age=0"));

        use http_guard::RunMode;
        assert!(!cookie_secure(RunMode::Dev, &HeaderMap::new()));
        assert!(cookie_secure(RunMode::Production, &HeaderMap::new()));
        assert!(cookie_secure(RunMode::Dev, &headers(&[("x-forwarded-proto", "https")])));
    }

    #[test]
    fn csrf_allows_same_origin_cookie_mutation() {
        let h = headers(&[("host", "localhost:5175"), ("origin", "http://localhost:5175"), ("cookie", "cronus_token=t")]);
        assert!(!csrf_blocks(&Method::POST, "/api/notes", &h));
        let proxied = headers(&[("host", "app:5175"), ("x-forwarded-host", "example.com"), ("origin", "https://example.com"), ("cookie", "cronus_token=t")]);
        assert!(!csrf_blocks(&Method::DELETE, "/api/notes/1", &proxied));
        let referer = headers(&[("host", "localhost:5175"), ("referer", "http://localhost:5175/notes"), ("cookie", "cronus_token=t")]);
        assert!(!csrf_blocks(&Method::PATCH, "/api/notes/1", &referer));
    }

    #[test]
    fn csrf_blocks_cross_site_or_originless_cookie_mutation() {
        let evil = headers(&[("host", "localhost:5175"), ("origin", "https://evil.test"), ("cookie", "cronus_token=t")]);
        assert!(csrf_blocks(&Method::POST, "/api/notes", &evil));
        assert!(csrf_blocks(&Method::POST, "/_forms/Note", &evil));
        let null_origin = headers(&[("host", "localhost:5175"), ("origin", "null"), ("cookie", "cronus_token=t")]);
        assert!(csrf_blocks(&Method::PUT, "/graphql", &null_origin));
        let bare = headers(&[("host", "localhost:5175"), ("cookie", "cronus_token=t")]);
        assert!(csrf_blocks(&Method::DELETE, "/api/notes/1", &bare));
        let lookalike = headers(&[("host", "localhost:5175"), ("origin", "http://localhost:5175.evil.test"), ("cookie", "cronus_token=t")]);
        assert!(csrf_blocks(&Method::POST, "/api/notes", &lookalike));
    }

    #[test]
    fn csrf_ignores_reads_bearer_clients_and_cookieless_api_calls() {
        let evil_cookie = headers(&[("host", "h"), ("origin", "https://evil.test"), ("cookie", "cronus_token=t")]);
        assert!(!csrf_blocks(&Method::GET, "/api/notes", &evil_cookie));
        let bearer = headers(&[("host", "h"), ("authorization", "Bearer x")]);
        assert!(!csrf_blocks(&Method::POST, "/api/notes", &bearer));
        let anon = headers(&[("host", "h"), ("origin", "https://evil.test")]);
        assert!(!csrf_blocks(&Method::POST, "/api/posts", &anon));
    }

    #[test]
    fn csrf_blocks_cross_site_login_but_not_cli_login() {
        let evil = headers(&[("host", "localhost:5175"), ("origin", "https://evil.test")]);
        assert!(csrf_blocks(&Method::POST, "/api/auth/login", &evil));
        let cli = headers(&[("host", "localhost:5175")]);
        assert!(!csrf_blocks(&Method::POST, "/api/auth/login", &cli));
    }

    #[test]
    fn body_token_only_on_explicit_request() {
        assert!(!wants_body_token("", &HeaderMap::new()));
        assert!(wants_body_token("a=b&token=1", &HeaderMap::new()));
        assert!(!wants_body_token("token=10", &HeaderMap::new()));
        assert!(wants_body_token("", &headers(&[("x-cronus-session-token", "1")])));
    }

    #[test]
    fn parser_expires_feeds_session_policy() {
        let src = "app \"S\" { port 5175 }\nauth {\n  entity User\n  login email + password\n  session jwt expires:2h\n  roles [admin, user]\n}\nentity User {\n  email email!\n}\n";
        let nodes = crate::parser::parse(src).expect("parse");
        let auth_node = nodes
            .iter()
            .find_map(|n| match n {
                crate::parser::AstNode::Auth(a) => Some(a),
                _ => None,
            })
            .expect("auth node");
        let policy = auth::SessionPolicy::from_session_config(&auth_node.session_config).unwrap();
        assert_eq!(policy.ttl_secs, 7200);
    }

    // ── /api/auth/* over the real handler + in-memory SQLite ──

    const APP: &str = "app \"S\" { port 5175 }\nauth {\n  entity User\n  login email + password\n  session jwt\n  roles [admin, user]\n}\nentity User {\n  name string\n  email email!\n  role string\n  password string sensitive\n}\n";
    const PASSWORD: &str = "correct horse battery staple";

    struct Reply {
        status: StatusCode,
        headers: HeaderMap,
        body: Value,
    }

    fn call(state: &AppState, method: Method, target: &str, headers: &HeaderMap, body: Value) -> Reply {
        use http_body_util::BodyExt;
        let (path, query) = target.split_once('?').unwrap_or((target, ""));
        let raw = serde_json::to_vec(&body).unwrap();
        let resp = handle_auth(state, &method, path, query, headers, &raw).response;
        let status = resp.status();
        let headers = resp.headers().clone();
        let bytes = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(resp.into_body().collect())
            .unwrap()
            .to_bytes();
        Reply { status, headers, body: serde_json::from_slice(&bytes).unwrap_or(Value::Null) }
    }

    fn set_cookie(reply: &Reply) -> String {
        reply.headers.get(SET_COOKIE).and_then(|v| v.to_str().ok()).unwrap_or("").to_string()
    }

    fn signup_body(email: &str) -> Value {
        json!({"name": "Ana", "email": email, "password": PASSWORD})
    }

    #[test]
    fn signup_and_login_set_only_the_httponly_cookie() {
        let s = crate::api_security_tests::state_from(APP);
        let signup = call(&s, Method::POST, "/api/auth/signup", &HeaderMap::new(), signup_body("ana@session.test"));
        assert_eq!(signup.status, StatusCode::CREATED);
        assert!(signup.body.get("token").is_none(), "token must not reach browser JS");
        assert_eq!(signup.body["user"]["email"], "ana@session.test");
        let cookie = set_cookie(&signup);
        assert!(cookie.starts_with("cronus_token=") && cookie.contains("; HttpOnly; SameSite=Lax"));
        assert!(cookie.contains(&format!("Max-Age={}", auth::DEFAULT_SESSION_TTL_SECS)));

        let login = call(&s, Method::POST, "/api/auth/login", &HeaderMap::new(), json!({"email": "ana@session.test", "password": PASSWORD}));
        assert_eq!(login.status, StatusCode::OK);
        assert!(login.body.get("token").is_none());
        assert!(set_cookie(&login).contains("HttpOnly"));
    }

    #[test]
    fn body_token_on_request_and_expires_honoured() {
        let mut s = crate::api_security_tests::state_from(APP);
        s.session_policy = auth::SessionPolicy { ttl_secs: 7200 };
        call(&s, Method::POST, "/api/auth/signup", &HeaderMap::new(), signup_body("cli@session.test"));
        let login = call(&s, Method::POST, "/api/auth/login?token=1", &HeaderMap::new(), json!({"email": "cli@session.test", "password": PASSWORD}));
        assert_eq!(login.status, StatusCode::OK);
        let token = login.body["token"].as_str().expect("token for explicit CLI request");
        let claims = auth::verify_token(token, &auth::default_secret()).unwrap();
        assert_eq!(claims.exp - claims.iat, 7200);
        assert!(!claims.jti.is_empty());
        assert!(set_cookie(&login).contains("Max-Age=7200"));
    }

    #[test]
    fn me_reads_cookie_and_logout_clears_it() {
        let s = crate::api_security_tests::state_from(APP);
        let signup = call(&s, Method::POST, "/api/auth/signup", &HeaderMap::new(), signup_body("me@session.test"));
        let cookie_pair = set_cookie(&signup).split(';').next().unwrap().to_string();

        let anon = call(&s, Method::GET, "/api/auth/me", &HeaderMap::new(), Value::Null);
        assert_eq!(anon.status, StatusCode::UNAUTHORIZED);
        let me = call(&s, Method::GET, "/api/auth/me", &headers(&[("cookie", cookie_pair.as_str())]), Value::Null);
        assert_eq!(me.status, StatusCode::OK);
        assert_eq!(me.body["id"], signup.body["user"]["id"]);
        assert!(me.body.get("token").is_none());

        let out = call(&s, Method::POST, "/api/auth/logout", &headers(&[("cookie", cookie_pair.as_str())]), Value::Null);
        assert_eq!(out.status, StatusCode::OK);
        assert!(set_cookie(&out).starts_with("cronus_token=; ") && set_cookie(&out).contains("Max-Age=0"));
    }

    #[test]
    fn signup_uses_declared_auth_entity_not_user_table() {
        let src = "app \"S\" { port 5175 }\nauth {\n  entity Account\n  login email + password\n  session jwt\n  roles [admin, member]\n}\nentity User {\n  name string\n  email email!\n  role string\n  password string sensitive\n}\nentity Account {\n  name string\n  email email!\n  role string\n  password string sensitive\n}\n";
        let s = crate::api_security_tests::state_from(src);
        let signup = call(&s, Method::POST, "/api/auth/signup", &HeaderMap::new(), signup_body("acc@session.test"));
        assert_eq!(signup.status, StatusCode::CREATED);
        assert_eq!(s.db.count("Account").unwrap(), 1);
        assert_eq!(s.db.count("User").unwrap(), 0);
        assert_eq!(signup.body["user"]["role"], "member");
        let login = call(&s, Method::POST, "/api/auth/login", &HeaderMap::new(), json!({"email": "acc@session.test", "password": PASSWORD}));
        assert_eq!(login.status, StatusCode::OK);
    }

    #[test]
    fn signup_failure_does_not_leak_database_detail() {
        // Auth entity without `name`/`role` columns: the insert fails inside SQLite.
        let src = "app \"S\" { port 5175 }\nauth {\n  entity Member\n  login email + password\n  session jwt\n  roles [user]\n}\nentity Member {\n  email email!\n  password string sensitive\n}\n";
        let s = crate::api_security_tests::state_from(src);
        let signup = call(&s, Method::POST, "/api/auth/signup", &HeaderMap::new(), signup_body("leak@session.test"));
        assert_eq!(signup.status, StatusCode::INTERNAL_SERVER_ERROR, "body: {}", signup.body);
        assert_eq!(signup.body, json!({"error": "could not create account"}));
        assert!(set_cookie(&signup).is_empty());
    }
}
