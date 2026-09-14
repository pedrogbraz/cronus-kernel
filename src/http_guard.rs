//! HTTP surface hardening (Sprint 1 security, agent C).
//!
//! One place for the policies that sit in front of the live dispatcher
//! (`main.rs::handle_request`): bind address, run mode (dev/production),
//! gating of internal/diagnostic routes, request body limits, panic
//! isolation, HTTP/1 timeouts, client-IP resolution for rate limiting and
//! per-account login backoff.
//!
//! Env/CLI surface:
//! - `--host <ip>` / `CRONUS_HOST`           bind address (default 127.0.0.1)
//! - `--prod` / `CRONUS_ENV=production`      production mode (internal routes 404)
//! - `CRONUS_MAX_BODY_BYTES`                 request body limit (default 1 MiB)
//! - `CRONUS_TRUSTED_PROXIES`                IPs/CIDRs whose X-Forwarded-For is honored
//! - `CRONUS_HEADER_READ_TIMEOUT_SECS`       HTTP/1 header read timeout (default 15)

use bytes::Bytes;
use http_body_util::{BodyExt, Full, LengthLimitError, Limited};
use hyper::body::Body;
use hyper::{HeaderMap, Method, Response, StatusCode};
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::authz::error_body;
use crate::parser::EntityNode;

pub const DEFAULT_MAX_BODY_BYTES: usize = 1024 * 1024;
pub const DEFAULT_HEADER_READ_TIMEOUT_SECS: u64 = 15;

// ══════════════════════════════════════════════════
// Run policy (mode + bind address + trusted proxies)
// ══════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunMode {
    Dev,
    Production,
}

#[derive(Clone, Debug)]
pub struct Policy {
    pub mode: RunMode,
    pub bind_ip: IpAddr,
    pub trusted_proxies: Vec<Cidr>,
    pub max_body_bytes: usize,
}

impl Policy {
    /// Loopback-bound dev server: the only setup where internal routes are
    /// served without an admin session.
    pub fn is_local_dev(&self) -> bool {
        self.mode == RunMode::Dev && self.bind_ip.is_loopback()
    }
}

impl Default for Policy {
    fn default() -> Self {
        Policy {
            mode: RunMode::Dev,
            bind_ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
            trusted_proxies: Vec::new(),
            max_body_bytes: DEFAULT_MAX_BODY_BYTES,
        }
    }
}

static POLICY: OnceLock<Policy> = OnceLock::new();

/// Installs the process-wide policy. First call wins (the server starts once).
pub fn install_policy(p: Policy) -> &'static Policy {
    let _ = POLICY.set(p);
    policy()
}

pub fn policy() -> &'static Policy {
    POLICY.get_or_init(Policy::default)
}

fn arg_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
}

/// `--prod` or `CRONUS_ENV=production|prod` → production. `cronus run` is dev.
pub fn run_mode_from(args: &[String], env_mode: Option<&str>) -> RunMode {
    let env_prod = env_mode
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v == "production" || v == "prod"
        })
        .unwrap_or(false);
    if env_prod || args.iter().any(|a| a == "--prod" || a == "--production") {
        RunMode::Production
    } else {
        RunMode::Dev
    }
}

/// Bind address: `--host` beats `CRONUS_HOST`; default loopback.
/// Audit canvas is always loopback (e2e harness contract).
pub fn resolve_bind_ip(
    args: &[String],
    env_host: Option<&str>,
    audit_canvas: bool,
) -> Result<IpAddr, String> {
    if audit_canvas {
        return Ok(IpAddr::V4(Ipv4Addr::LOCALHOST));
    }
    let raw = arg_value(args, "--host").or(env_host.filter(|s| !s.trim().is_empty()));
    match raw {
        None => Ok(IpAddr::V4(Ipv4Addr::LOCALHOST)),
        Some("localhost") => Ok(IpAddr::V4(Ipv4Addr::LOCALHOST)),
        Some(h) => h
            .trim()
            .parse::<IpAddr>()
            .map_err(|_| format!("invalid --host/CRONUS_HOST value (expected an IP address)")),
    }
}

/// Value of `--host` so the port scanner can skip it.
pub fn host_arg(args: &[String]) -> Option<&str> {
    arg_value(args, "--host")
}

pub fn max_body_bytes_from(env_val: Option<&str>) -> usize {
    env_val
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(DEFAULT_MAX_BODY_BYTES)
}

/// Builds the policy from CLI args + process environment.
pub fn policy_from_env(args: &[String], audit_canvas: bool) -> Result<Policy, String> {
    let env = |k: &str| std::env::var(k).ok();
    Ok(Policy {
        mode: run_mode_from(args, env("CRONUS_ENV").as_deref()),
        bind_ip: resolve_bind_ip(args, env("CRONUS_HOST").as_deref(), audit_canvas)?,
        trusted_proxies: parse_trusted_proxies(
            env("CRONUS_TRUSTED_PROXIES").as_deref().unwrap_or(""),
        ),
        max_body_bytes: max_body_bytes_from(env("CRONUS_MAX_BODY_BYTES").as_deref()),
    })
}

// ══════════════════════════════════════════════════
// Trusted proxies + client IP
// ══════════════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cidr {
    net: IpAddr,
    prefix: u8,
}

impl Cidr {
    pub fn parse(s: &str) -> Option<Cidr> {
        let s = s.trim();
        let (ip_s, prefix_s) = match s.split_once('/') {
            Some((a, b)) => (a, Some(b)),
            None => (s, None),
        };
        let net: IpAddr = ip_s.parse().ok()?;
        let max = if net.is_ipv4() { 32 } else { 128 };
        let prefix = match prefix_s {
            Some(p) => p.parse::<u8>().ok().filter(|p| *p <= max)?,
            None => max,
        };
        Some(Cidr { net, prefix })
    }

    pub fn contains(&self, ip: IpAddr) -> bool {
        let ip = canonical(ip);
        match (self.net, ip) {
            (IpAddr::V4(n), IpAddr::V4(i)) => prefix_eq(&n.octets(), &i.octets(), self.prefix),
            (IpAddr::V6(n), IpAddr::V6(i)) => prefix_eq(&n.octets(), &i.octets(), self.prefix),
            _ => false,
        }
    }
}

fn prefix_eq(a: &[u8], b: &[u8], prefix: u8) -> bool {
    let full = (prefix / 8) as usize;
    if a[..full] != b[..full] {
        return false;
    }
    let rem = prefix % 8;
    if rem == 0 {
        return true;
    }
    let mask = 0xFFu8 << (8 - rem);
    (a[full] & mask) == (b[full] & mask)
}

/// Maps IPv4-mapped IPv6 (`::ffff:1.2.3.4`) to IPv4 so lists match either form.
fn canonical(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map(IpAddr::V4).unwrap_or(ip),
        v4 => v4,
    }
}

pub fn parse_trusted_proxies(raw: &str) -> Vec<Cidr> {
    raw.split(',')
        .filter(|s| !s.trim().is_empty())
        .filter_map(Cidr::parse)
        .collect()
}

/// Client IP for rate limiting. `X-Forwarded-For` is honored only when the
/// socket peer is a trusted proxy; then the right-most untrusted hop wins
/// (left-most entries are client-controlled).
pub fn client_ip(peer: SocketAddr, xff: Option<&str>, trusted: &[Cidr]) -> IpAddr {
    let peer_ip = canonical(peer.ip());
    let is_trusted = |ip: IpAddr| trusted.iter().any(|c| c.contains(ip));
    if !is_trusted(peer_ip) {
        return peer_ip;
    }
    let Some(xff) = xff else { return peer_ip };
    let mut hops: Vec<IpAddr> = Vec::new();
    for part in xff.split(',') {
        match part.trim().parse::<IpAddr>() {
            Ok(ip) => hops.push(canonical(ip)),
            // A malformed hop breaks the chain: stop trusting anything left of it.
            Err(_) => hops.clear(),
        }
    }
    hops.into_iter()
        .rev()
        .find(|ip| !is_trusted(*ip))
        .unwrap_or(peer_ip)
}

pub fn client_ip_from_headers(peer: SocketAddr, headers: &HeaderMap) -> IpAddr {
    let xff = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok());
    client_ip(peer, xff, &policy().trusted_proxies)
}

// ══════════════════════════════════════════════════
// Per-account login backoff
// ══════════════════════════════════════════════════

/// Free failures before backoff starts.
pub const ACCOUNT_FREE_FAILURES: u32 = 5;
const ACCOUNT_MAX_LOCK_SECS: u64 = 15 * 60;
const ACCOUNT_MAX_KEYS: usize = 10_000;
const ACCOUNT_IDLE_SECS: u64 = 60 * 60;

struct AccountState {
    failures: u32,
    locked_until: Option<Instant>,
    last_seen: Instant,
}

/// Exponential backoff per normalized account (email), independent of IP,
/// so rotating source addresses cannot brute-force one account.
pub struct AccountLimiter {
    accounts: Mutex<HashMap<String, AccountState>>,
    max_keys: usize,
}

pub fn normalize_account(email: &str) -> String {
    email.trim().to_lowercase().chars().take(254).collect()
}

impl AccountLimiter {
    pub fn new(max_keys: usize) -> Self {
        AccountLimiter {
            accounts: Mutex::new(HashMap::new()),
            max_keys,
        }
    }

    /// `Err(retry_after_secs)` while the account is locked.
    pub fn check_at(&self, account: &str, now: Instant) -> Result<(), u64> {
        let key = normalize_account(account);
        let map = self.accounts.lock().unwrap_or_else(|e| e.into_inner());
        match map.get(&key).and_then(|s| s.locked_until) {
            Some(until) if until > now => {
                let secs = until.duration_since(now).as_secs_f64().ceil() as u64;
                Err(secs.max(1))
            }
            _ => Ok(()),
        }
    }

    pub fn record_at(&self, account: &str, success: bool, now: Instant) {
        let key = normalize_account(account);
        if key.is_empty() {
            return;
        }
        let mut map = self.accounts.lock().unwrap_or_else(|e| e.into_inner());
        if success {
            map.remove(&key);
            return;
        }
        if !map.contains_key(&key) && map.len() >= self.max_keys {
            Self::prune(&mut map, now);
            if map.len() >= self.max_keys {
                // Evict the least recently seen account that is not locked.
                let victim = map
                    .iter()
                    .filter(|(_, s)| s.locked_until.map_or(true, |u| u <= now))
                    .min_by_key(|(_, s)| s.last_seen)
                    .or_else(|| map.iter().min_by_key(|(_, s)| s.last_seen))
                    .map(|(k, _)| k.clone());
                if let Some(v) = victim {
                    map.remove(&v);
                }
            }
        }
        let st = map.entry(key).or_insert(AccountState {
            failures: 0,
            locked_until: None,
            last_seen: now,
        });
        st.failures = st.failures.saturating_add(1);
        st.last_seen = now;
        if st.failures > ACCOUNT_FREE_FAILURES {
            let exp = (st.failures - ACCOUNT_FREE_FAILURES - 1).min(20);
            let secs = (1u64 << exp).min(ACCOUNT_MAX_LOCK_SECS);
            st.locked_until = Some(now + Duration::from_secs(secs));
        }
    }

    fn prune(map: &mut HashMap<String, AccountState>, now: Instant) {
        map.retain(|_, s| {
            let locked = s.locked_until.map_or(false, |u| u > now);
            locked || now.duration_since(s.last_seen).as_secs() < ACCOUNT_IDLE_SECS
        });
    }

    pub fn cleanup_at(&self, now: Instant) {
        let mut map = self.accounts.lock().unwrap_or_else(|e| e.into_inner());
        Self::prune(&mut map, now);
    }

    pub fn len(&self) -> usize {
        self.accounts
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .len()
    }
}

static ACCOUNTS: OnceLock<AccountLimiter> = OnceLock::new();

pub fn account_limiter() -> &'static AccountLimiter {
    ACCOUNTS.get_or_init(|| AccountLimiter::new(ACCOUNT_MAX_KEYS))
}

/// Login guard: 429 while the account is in backoff.
pub fn login_account_check(email: &str) -> Option<Response<Full<Bytes>>> {
    if email.trim().is_empty() {
        return None;
    }
    account_limiter()
        .check_at(email, Instant::now())
        .err()
        .map(too_many_requests)
}

/// Records the login outcome from the response status (401 = failure).
pub fn login_account_record(email: &str, status: StatusCode) {
    if status.is_success() {
        account_limiter().record_at(email, true, Instant::now());
    } else if status == StatusCode::UNAUTHORIZED {
        account_limiter().record_at(email, false, Instant::now());
    }
}

pub fn too_many_requests(retry_after: u64) -> Response<Full<Bytes>> {
    let mut resp = json(
        StatusCode::TOO_MANY_REQUESTS,
        error_body("RATE_LIMITED", "Too many requests"),
    );
    if let Ok(v) = hyper::header::HeaderValue::from_str(&retry_after.to_string()) {
        resp.headers_mut().insert(hyper::header::RETRY_AFTER, v);
    }
    for (k, v) in crate::security::security_headers() {
        resp.headers_mut().insert(
            hyper::header::HeaderName::from_static(k),
            hyper::header::HeaderValue::from_static(v),
        );
    }
    resp
}

// ══════════════════════════════════════════════════
// Internal / diagnostic route gating
// ══════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteClass {
    Public,
    /// Dev tooling: 404 in production; admin unless loopback dev.
    Internal,
    /// Sensitive data (audit trail): admin in every mode.
    AdminAlways,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Gate {
    Allow,
    NotFound,
    Unauthorized,
    Forbidden,
}

fn normalize(path: &str) -> &str {
    if path.len() > 1 && path.ends_with('/') {
        let t = path.trim_end_matches('/');
        if t.is_empty() {
            "/"
        } else {
            t
        }
    } else {
        path
    }
}

pub fn classify(method: &Method, raw_path: &str) -> RouteClass {
    let path = normalize(raw_path);
    let under = |prefix: &str| path == prefix || path.starts_with(&format!("{}/", prefix));
    if under("/api/audit/trail") {
        return RouteClass::AdminAlways;
    }
    let internal = under("/zeus")
        || under("/api/debug")
        || path == "/api/_context"
        || path == "/api/_seed"
        || under("/api/server")
        || under("/api/hydra")
        || path == "/hydra"
        || under("/blocks")
        || path == "/trust"
        || path == "/api/trust"
        || under("/api/brain")
        || path == "/api/_health"
        || path == "/api/_errors"
        || path == "/api/schema"
        || path == "/graphql/schema"
        || under("/docs")
        || under("/api/docs")
        || under("/.cronus/docs")
        || path == "/api/audit/trigger"
        || path == "/api/audit/results";
    let _ = method;
    if internal {
        RouteClass::Internal
    } else {
        RouteClass::Public
    }
}

pub fn gate(class: RouteClass, policy: &Policy, role: Option<&str>) -> Gate {
    let admin = || match role {
        None => Gate::Unauthorized,
        Some("admin") => Gate::Allow,
        Some(_) => Gate::Forbidden,
    };
    match class {
        RouteClass::Public => Gate::Allow,
        RouteClass::AdminAlways => admin(),
        RouteClass::Internal if policy.mode == RunMode::Production => Gate::NotFound,
        RouteClass::Internal if policy.is_local_dev() => Gate::Allow,
        RouteClass::Internal => admin(),
    }
}

/// Role from `Authorization: Bearer` or the `cronus_token` cookie.
pub fn request_role(headers: &HeaderMap) -> Option<String> {
    let secret = crate::auth::default_secret();
    let bearer = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|s| s.trim().to_string());
    let cookie = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|c| {
            c.split(';').find_map(|p| {
                p.trim()
                    .strip_prefix("cronus_token=")
                    .map(|s| s.to_string())
            })
        });
    bearer
        .or(cookie)
        .and_then(|t| crate::auth::verify_token(&t, &secret).ok())
        .map(|c| c.role)
}

/// Returns a response when the route must not be served to this request.
pub fn guard_internal(
    method: &Method,
    path: &str,
    headers: &HeaderMap,
) -> Option<Response<Full<Bytes>>> {
    let class = classify(method, path);
    if class == RouteClass::Public {
        return None;
    }
    let role = request_role(headers);
    match gate(class, policy(), role.as_deref()) {
        Gate::Allow => None,
        Gate::NotFound => Some(not_found()),
        Gate::Unauthorized => Some(json(
            StatusCode::UNAUTHORIZED,
            error_body("UNAUTHORIZED", "Authentication required"),
        )),
        Gate::Forbidden => Some(json(
            StatusCode::FORBIDDEN,
            error_body("FORBIDDEN", "Admin role required"),
        )),
    }
}

pub fn is_production() -> bool {
    policy().mode == RunMode::Production
}

pub fn not_found() -> Response<Full<Bytes>> {
    json(StatusCode::NOT_FOUND, error_body("NOT_FOUND", "Not found"))
}

fn json(status: StatusCode, body: Value) -> Response<Full<Bytes>> {
    let mut resp = Response::new(Full::new(Bytes::from(body.to_string())));
    *resp.status_mut() = status;
    resp.headers_mut().insert(
        hyper::header::CONTENT_TYPE,
        hyper::header::HeaderValue::from_static("application/json"),
    );
    resp
}

pub fn internal_error() -> Response<Full<Bytes>> {
    json(
        StatusCode::INTERNAL_SERVER_ERROR,
        error_body("INTERNAL", "Internal server error"),
    )
}

// ══════════════════════════════════════════════════
// Audit trail redaction + audit widget results path
// ══════════════════════════════════════════════════

fn entity_for_table<'a>(entities: &'a [EntityNode], table: &str) -> Option<&'a EntityNode> {
    entities.iter().find(|e| {
        let lower = e.name.to_lowercase();
        let t = table.to_lowercase();
        t == lower || t == format!("{}s", lower) || format!("{}s", t) == lower
    })
}

fn redact_json_str(entity: Option<&EntityNode>, raw: &Value) -> Value {
    let Some(s) = raw.as_str() else {
        return raw.clone();
    };
    let Ok(mut v) = serde_json::from_str::<Value>(s) else {
        // Unparseable payload: do not risk echoing it.
        return Value::Null;
    };
    redact_value(entity, &mut v);
    Value::String(v.to_string())
}

fn redact_value(entity: Option<&EntityNode>, v: &mut Value) {
    match entity {
        Some(e) => crate::authz::redact_sensitive(e, v),
        None => {
            if let Value::Object(m) = v {
                m.remove("password");
                m.remove("password_hash");
            }
        }
    }
}

fn hidden_field(entity: Option<&EntityNode>, field: &str) -> bool {
    field == "password"
        || field == "password_hash"
        || entity.map_or(false, |e| crate::authz::sensitive_names(e).contains(&field))
}

/// Removes sensitive fields from `data`, `prev_data` and `diff` of audit entries.
pub fn redact_audit_entries(entries: &mut Value, entities: &[EntityNode]) {
    let Value::Array(rows) = entries else { return };
    for row in rows.iter_mut() {
        let Value::Object(m) = row else { continue };
        let table = m
            .get("entity")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let entity = entity_for_table(entities, &table);
        for key in ["data", "prev_data"] {
            if let Some(v) = m.get(key).cloned() {
                m.insert(key.to_string(), redact_json_str(entity, &v));
            }
        }
        if let Some(Value::Array(diff)) = m.get_mut("diff") {
            diff.retain(|d| {
                let f = d.get("field").and_then(|v| v.as_str()).unwrap_or("");
                !hidden_field(entity, f)
            });
        }
    }
}

/// Audit widget results live in the project, never in world-writable /tmp.
pub const AUDIT_WIDGET_RESULTS_PATH: &str = ".cronus/audit-widget-results.json";

// ══════════════════════════════════════════════════
// Body limit
// ══════════════════════════════════════════════════

/// Reads a whole body with a hard byte cap. Overflow → 413; transport error → 400.
pub async fn read_body_limited<B>(body: B, max: usize) -> Result<Bytes, Response<Full<Bytes>>>
where
    B: Body<Data = Bytes>,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    match Limited::new(body, max).collect().await {
        Ok(collected) => Ok(collected.to_bytes()),
        Err(e) if e.downcast_ref::<LengthLimitError>().is_some() => Err(json(
            StatusCode::PAYLOAD_TOO_LARGE,
            error_body("PAYLOAD_TOO_LARGE", "Request body too large"),
        )),
        Err(_) => Err(json(
            StatusCode::BAD_REQUEST,
            error_body("BAD_REQUEST", "Could not read request body"),
        )),
    }
}

/// `read_body_limited` with the installed policy's limit.
pub async fn read_body<B>(body: B) -> Result<Bytes, Response<Full<Bytes>>>
where
    B: Body<Data = Bytes>,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    read_body_limited(body, policy().max_body_bytes).await
}

// ══════════════════════════════════════════════════
// Panic isolation + HTTP/1 builder
// ══════════════════════════════════════════════════

/// Runs a request handler in its own task so a panic becomes a 500 for that
/// request instead of taking down the connection (or, with abort, the process).
pub async fn isolate_panics<F, E>(fut: F) -> Result<Response<Full<Bytes>>, E>
where
    F: Future<Output = Result<Response<Full<Bytes>>, E>> + Send + 'static,
    E: Send + 'static,
{
    match tokio::spawn(fut).await {
        Ok(res) => res,
        Err(join_err) => {
            if join_err.is_panic() {
                eprintln!("  \x1b[31m✗\x1b[0m request handler panicked; returned 500");
            }
            Ok(internal_error())
        }
    }
}

pub fn header_read_timeout_from(env_val: Option<&str>) -> Duration {
    Duration::from_secs(
        env_val
            .and_then(|v| v.trim().parse::<u64>().ok())
            .filter(|n| *n > 0)
            .unwrap_or(DEFAULT_HEADER_READ_TIMEOUT_SECS),
    )
}

pub fn http1_builder() -> hyper::server::conn::http1::Builder {
    let mut b = hyper::server::conn::http1::Builder::new();
    b.timer(hyper_util::rt::TokioTimer::new())
        .header_read_timeout(header_read_timeout_from(
            std::env::var("CRONUS_HEADER_READ_TIMEOUT_SECS")
                .ok()
                .as_deref(),
        ));
    b
}

// ══════════════════════════════════════════════════
// Tests
// ══════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    fn peer(ip: &str) -> SocketAddr {
        SocketAddr::new(ip.parse().unwrap(), 40000)
    }

    #[tokio::test]
    async fn limited_body_over_cap_is_413() {
        let body = Full::new(Bytes::from(vec![b'a'; 2048]));
        let err = read_body_limited(body, 1024)
            .await
            .expect_err("must reject");
        assert_eq!(err.status(), StatusCode::PAYLOAD_TOO_LARGE);
        let ok = read_body_limited(Full::new(Bytes::from_static(b"{}")), 1024)
            .await
            .expect("small ok");
        assert_eq!(&ok[..], b"{}");
    }

    #[test]
    fn max_body_env_parsing() {
        assert_eq!(max_body_bytes_from(None), DEFAULT_MAX_BODY_BYTES);
        assert_eq!(max_body_bytes_from(Some("2048")), 2048);
        assert_eq!(max_body_bytes_from(Some("0")), DEFAULT_MAX_BODY_BYTES);
        assert_eq!(max_body_bytes_from(Some("junk")), DEFAULT_MAX_BODY_BYTES);
    }

    #[test]
    fn xff_ignored_from_untrusted_peer() {
        let trusted = parse_trusted_proxies("10.0.0.1");
        let ip = client_ip(peer("203.0.113.9"), Some("1.2.3.4"), &trusted);
        assert_eq!(ip, "203.0.113.9".parse::<IpAddr>().unwrap());
        // No trusted proxies configured at all: always the socket peer.
        assert_eq!(
            client_ip(peer("127.0.0.1"), Some("9.9.9.9"), &[]),
            "127.0.0.1".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn xff_honored_from_trusted_peer_rightmost_untrusted_hop() {
        let trusted = parse_trusted_proxies("10.0.0.0/8, ::1");
        let ip = client_ip(
            peer("10.1.2.3"),
            Some("6.6.6.6, 198.51.100.7, 10.0.0.5"),
            &trusted,
        );
        assert_eq!(ip, "198.51.100.7".parse::<IpAddr>().unwrap());
        let ip6 = client_ip(peer("::1"), Some("198.51.100.8"), &trusted);
        assert_eq!(ip6, "198.51.100.8".parse::<IpAddr>().unwrap());
        // Garbage header from a trusted proxy falls back to the peer.
        assert_eq!(
            client_ip(peer("10.1.2.3"), Some("nope"), &trusted),
            "10.1.2.3".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn rotating_xff_does_not_evade_ip_limiter() {
        // Regression: 12 logins with rotating X-Forwarded-For used to land in
        // 12 different buckets. With peer-based keys the 11th is limited.
        let limiter = crate::rate_limit::RateLimiter::new(10, 60);
        let mut limited = 0;
        for i in 0..12 {
            let xff = format!("10.9.9.{}", i);
            let ip = client_ip(peer("203.0.113.9"), Some(&xff), &[]);
            if limiter.check(&format!("auth:{}", ip)).is_err() {
                limited += 1;
            }
        }
        assert_eq!(limited, 2);
    }

    #[test]
    fn per_account_backoff_locks_and_success_resets() {
        let l = AccountLimiter::new(100);
        let t0 = Instant::now();
        for _ in 0..ACCOUNT_FREE_FAILURES {
            assert!(l.check_at("Victim@Example.com", t0).is_ok());
            l.record_at(" victim@example.com ", false, t0);
        }
        assert!(
            l.check_at("victim@example.com", t0).is_ok(),
            "free failures do not lock"
        );
        l.record_at("VICTIM@example.com", false, t0);
        assert_eq!(l.check_at("victim@example.com", t0), Err(1));
        l.record_at("victim@example.com", false, t0);
        assert_eq!(
            l.check_at("victim@example.com", t0),
            Err(2),
            "backoff doubles"
        );
        assert!(l
            .check_at("victim@example.com", t0 + Duration::from_secs(3))
            .is_ok());
        assert!(l.check_at("other@example.com", t0).is_ok(), "per account");
        l.record_at("victim@example.com", true, t0);
        assert!(l.check_at("victim@example.com", t0).is_ok());
    }

    #[test]
    fn account_limiter_is_bounded() {
        let l = AccountLimiter::new(3);
        let t0 = Instant::now();
        for i in 0..10 {
            l.record_at(
                &format!("u{}@x.io", i),
                false,
                t0 + Duration::from_millis(i),
            );
        }
        assert!(l.len() <= 3);
        l.cleanup_at(t0 + Duration::from_secs(ACCOUNT_IDLE_SECS + 1));
        assert_eq!(l.len(), 0);
    }

    #[test]
    fn default_bind_address_is_loopback() {
        let ip = resolve_bind_ip(&args(&["cronus", "run"]), None, false).unwrap();
        assert!(ip.is_loopback());
        let env = resolve_bind_ip(&args(&["cronus", "run"]), Some("0.0.0.0"), false).unwrap();
        assert_eq!(env, "0.0.0.0".parse::<IpAddr>().unwrap());
        let flag = resolve_bind_ip(
            &args(&["cronus", "run", "--host", "::"]),
            Some("127.0.0.1"),
            false,
        )
        .unwrap();
        assert_eq!(flag, "::".parse::<IpAddr>().unwrap());
        let audit =
            resolve_bind_ip(&args(&["cronus", "run", "--host", "0.0.0.0"]), None, true).unwrap();
        assert!(audit.is_loopback(), "audit canvas stays loopback");
        assert!(resolve_bind_ip(&args(&["cronus", "run", "--host", "evil"]), None, false).is_err());
        assert!(Policy::default().bind_ip.is_loopback());
    }

    #[test]
    fn run_mode_flag_and_env() {
        assert_eq!(run_mode_from(&args(&["cronus", "run"]), None), RunMode::Dev);
        assert_eq!(
            run_mode_from(&args(&["cronus", "run", "--prod"]), None),
            RunMode::Production
        );
        assert_eq!(
            run_mode_from(&args(&["cronus", "run"]), Some("Production")),
            RunMode::Production
        );
        assert_eq!(
            run_mode_from(&args(&["cronus", "run"]), Some("development")),
            RunMode::Dev
        );
    }

    #[test]
    fn production_mode_hides_internal_routes() {
        let prod = Policy {
            mode: RunMode::Production,
            ..Policy::default()
        };
        for p in [
            "/zeus",
            "/zeus/api",
            "/api/debug/traces",
            "/api/_context",
            "/api/_seed",
            "/api/server/logs",
            "/api/hydra/evolve",
            "/blocks",
            "/blocks/",
            "/api/audit/results",
        ] {
            let class = classify(&Method::GET, p);
            assert_eq!(class, RouteClass::Internal, "{p}");
            assert_eq!(gate(class, &prod, Some("admin")), Gate::NotFound, "{p}");
        }
        assert_eq!(classify(&Method::GET, "/api/tasks"), RouteClass::Public);
        assert_eq!(classify(&Method::GET, "/zeusx"), RouteClass::Public);
        assert_eq!(gate(RouteClass::Public, &prod, None), Gate::Allow);
    }

    #[test]
    fn internal_routes_need_admin_unless_loopback_dev() {
        let local = Policy::default();
        let exposed = Policy {
            bind_ip: "0.0.0.0".parse().unwrap(),
            ..Policy::default()
        };
        assert_eq!(gate(RouteClass::Internal, &local, None), Gate::Allow);
        assert_eq!(
            gate(RouteClass::Internal, &exposed, None),
            Gate::Unauthorized
        );
        assert_eq!(
            gate(RouteClass::Internal, &exposed, Some("user")),
            Gate::Forbidden
        );
        assert_eq!(
            gate(RouteClass::Internal, &exposed, Some("admin")),
            Gate::Allow
        );
        // Audit trail: admin in every mode, including loopback dev.
        let trail = classify(&Method::GET, "/api/audit/trail");
        assert_eq!(trail, RouteClass::AdminAlways);
        assert_eq!(
            classify(&Method::GET, "/api/audit/trail/verify"),
            RouteClass::AdminAlways
        );
        assert_eq!(gate(trail, &local, None), Gate::Unauthorized);
        assert_eq!(gate(trail, &local, Some("admin")), Gate::Allow);
    }

    #[test]
    fn not_found_does_not_echo_path() {
        let resp = not_found();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn audit_trail_redacts_sensitive() {
        let src = "app \"T\" { port 5175 }\nentity User {\n  email email!\n  name string\n  password string! sensitive\n  token string sensitive\n}\n";
        let entities: Vec<EntityNode> = crate::parser::parse(src)
            .expect("parse")
            .into_iter()
            .filter_map(|n| match n {
                crate::parser::AstNode::Entity(e) => Some(e),
                _ => None,
            })
            .collect();
        let mut entries = json!([{
            "entity": "users",
            "data": json!({"id":"1","email":"a@b","password":"$argon2id$x","token":"t2"}).to_string(),
            "prev_data": json!({"id":"1","email":"a@b","password":"$argon2id$old","token":"t1"}).to_string(),
            "diff": [{"field":"token","old":"t1","new":"t2"},{"field":"email","old":"a","new":"a@b"}],
        }, {
            "entity": "unknown_table",
            "data": json!({"password_hash":"h","x":1}).to_string(),
            "prev_data": null,
        }]);
        redact_audit_entries(&mut entries, &entities);
        let data: Value = serde_json::from_str(entries[0]["data"].as_str().unwrap()).unwrap();
        assert_eq!(data, json!({"id":"1","email":"a@b"}));
        let prev: Value = serde_json::from_str(entries[0]["prev_data"].as_str().unwrap()).unwrap();
        assert_eq!(prev, json!({"id":"1","email":"a@b"}));
        assert_eq!(
            entries[0]["diff"],
            json!([{"field":"email","old":"a","new":"a@b"}])
        );
        let other: Value = serde_json::from_str(entries[1]["data"].as_str().unwrap()).unwrap();
        assert_eq!(other, json!({"x":1}));
        assert!(entries[1]["prev_data"].is_null());
    }

    #[tokio::test]
    async fn panicking_handler_becomes_500() {
        let res: Result<Response<Full<Bytes>>, ()> = isolate_panics(async {
            if true {
                panic!("boom");
            }
            Ok(not_found())
        })
        .await;
        assert_eq!(res.unwrap().status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn header_timeout_env() {
        assert_eq!(
            header_read_timeout_from(None),
            Duration::from_secs(DEFAULT_HEADER_READ_TIMEOUT_SECS)
        );
        assert_eq!(header_read_timeout_from(Some("3")), Duration::from_secs(3));
    }
}
