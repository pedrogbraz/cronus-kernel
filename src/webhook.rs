//! Outbound webhooks declared with `webhook Entity { on create -> POST "url" }`.
//!
//! Security contract (Sprint 1):
//! - Only `http://` URLs are delivered. `https://` is refused with a logged
//!   error: the kernel has no TLS client (hyper/hyper-util are built without a
//!   TLS connector and adding one is a new crate), and sending cleartext to a
//!   URL the author marked as TLS would be worse than not sending.
//! - URL, method and every header are validated strictly; CR/LF and other
//!   control characters are rejected, so no header/request injection.
//! - The target is resolved once and every resolved address is checked:
//!   loopback, private, link-local (cloud metadata), CGNAT, multicast and
//!   reserved ranges are blocked unless `CRONUS_WEBHOOK_ALLOW_PRIVATE=1`.
//!   The connection goes to the vetted address, so DNS rebinding between the
//!   check and the connect is not possible.
//! - The payload is redacted with `authz::redact_sensitive` before sending.
//! - Each request carries `X-Cronus-Timestamp` and
//!   `X-Cronus-Signature: sha256=<hex HMAC-SHA256(secret, "{ts}.{body}")>`.
//!   Secret: `CRONUS_WEBHOOK_SECRET` (>= 32 bytes) or `.cronus/webhook.key`
//!   (generated, mode 0600).
//! - Connect and I/O are bounded by timeouts.

use crate::parser::{EntityNode, WebhookHook};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

pub const ALLOW_PRIVATE_ENV: &str = "CRONUS_WEBHOOK_ALLOW_PRIVATE";
pub const SECRET_ENV: &str = "CRONUS_WEBHOOK_SECRET";
const SECRET_PATH: &str = ".cronus/webhook.key";

const RESOLVE_TIMEOUT: Duration = Duration::from_secs(5);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const IO_TIMEOUT: Duration = Duration::from_secs(10);

const ALLOWED_METHODS: &[&str] = &["POST", "PUT", "PATCH", "DELETE"];
/// Headers the kernel owns; an author-declared header may not set them.
const RESERVED_HEADERS: &[&str] = &[
    "host",
    "content-length",
    "content-type",
    "transfer-encoding",
    "connection",
    "user-agent",
    "x-cronus-event",
    "x-cronus-timestamp",
    "x-cronus-signature",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebhookError {
    InvalidUrl(&'static str),
    TlsUnsupported,
    InvalidMethod,
    InvalidHeader(&'static str),
    Resolve,
    BlockedAddress,
    Secret(String),
    Io(&'static str),
}

impl fmt::Display for WebhookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WebhookError::InvalidUrl(why) => write!(f, "invalid webhook URL: {why}"),
            WebhookError::TlsUnsupported => write!(
                f,
                "https webhooks are not supported (no TLS client in this build); refusing to send in cleartext"
            ),
            WebhookError::InvalidMethod => {
                write!(f, "invalid webhook method (allowed: {})", ALLOWED_METHODS.join(", "))
            }
            WebhookError::InvalidHeader(why) => write!(f, "invalid webhook header: {why}"),
            WebhookError::Resolve => write!(f, "could not resolve webhook host"),
            WebhookError::BlockedAddress => write!(
                f,
                "webhook host resolves to a loopback/private/link-local address (set {ALLOW_PRIVATE_ENV}=1 to allow)"
            ),
            WebhookError::Secret(why) => write!(f, "webhook signing secret unavailable: {why}"),
            WebhookError::Io(stage) => write!(f, "webhook delivery failed while {stage}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub host: String,
    pub port: u16,
    /// Path plus query, always starting with `/`. Fragment removed.
    pub path: String,
}

impl Target {
    fn host_header(&self) -> String {
        let host = if self.host.contains(':') {
            format!("[{}]", self.host)
        } else {
            self.host.clone()
        };
        if self.port == 80 {
            host
        } else {
            format!("{host}:{}", self.port)
        }
    }

    /// Loggable form: never includes path or query (they may carry tokens).
    pub fn origin(&self) -> String {
        format!("http://{}", self.host_header())
    }
}

/// Webhook entity names match with or without a trailing `s` (legacy rule).
pub fn names_match(webhook_entity: &str, entity: &str) -> bool {
    let a = webhook_entity.to_lowercase();
    let b = entity.to_lowercase();
    a == b || format!("{a}s") == b || a == format!("{b}s")
}

pub fn parse_url(url: &str) -> Result<Target, WebhookError> {
    if !url.is_ascii() {
        return Err(WebhookError::InvalidUrl(
            "non-ASCII characters (percent-encode them)",
        ));
    }
    if url.bytes().any(|b| b <= 0x20 || b == 0x7f) {
        return Err(WebhookError::InvalidUrl("control characters or whitespace"));
    }
    let lower = url.to_ascii_lowercase();
    if lower.starts_with("https://") {
        return Err(WebhookError::TlsUnsupported);
    }
    if !lower.starts_with("http://") {
        return Err(WebhookError::InvalidUrl("scheme must be http://"));
    }
    let rest = &url["http://".len()..];
    let split = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (authority, remainder) = rest.split_at(split);

    let remainder = remainder.split('#').next().unwrap_or("");
    let path = if remainder.is_empty() {
        "/".to_string()
    } else if remainder.starts_with('?') {
        format!("/{remainder}")
    } else {
        remainder.to_string()
    };

    if authority.is_empty() {
        return Err(WebhookError::InvalidUrl("missing host"));
    }
    if authority.contains('@') {
        return Err(WebhookError::InvalidUrl(
            "credentials in URL are not allowed",
        ));
    }

    let (host, port_str) = if let Some(inner) = authority.strip_prefix('[') {
        let end = inner
            .find(']')
            .ok_or(WebhookError::InvalidUrl("unterminated IPv6 literal"))?;
        let host = &inner[..end];
        if host.parse::<std::net::Ipv6Addr>().is_err() {
            return Err(WebhookError::InvalidUrl("invalid IPv6 literal"));
        }
        let after = &inner[end + 1..];
        let port = match after {
            "" => None,
            p if p.starts_with(':') => Some(&p[1..]),
            _ => return Err(WebhookError::InvalidUrl("garbage after IPv6 literal")),
        };
        (host.to_string(), port)
    } else {
        let (host, port) = match authority.rsplit_once(':') {
            Some((h, p)) => (h, Some(p)),
            None => (authority, None),
        };
        let valid = !host.is_empty()
            && host.len() <= 253
            && host
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.')
            && !host.starts_with('.')
            && !host.starts_with('-');
        if !valid {
            return Err(WebhookError::InvalidUrl("invalid host"));
        }
        (host.to_ascii_lowercase(), port)
    };

    let port = match port_str {
        None => 80,
        Some(p) if !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()) && p.len() <= 5 => {
            match p.parse::<u16>() {
                Ok(0) | Err(_) => return Err(WebhookError::InvalidUrl("invalid port")),
                Ok(n) => n,
            }
        }
        Some(_) => return Err(WebhookError::InvalidUrl("invalid port")),
    };

    Ok(Target { host, port, path })
}

pub fn validate_method(method: &str) -> Result<&'static str, WebhookError> {
    ALLOWED_METHODS
        .iter()
        .find(|m| **m == method)
        .copied()
        .ok_or(WebhookError::InvalidMethod)
}

pub fn validate_header(name: &str, value: &str) -> Result<(), WebhookError> {
    const TCHAR_EXTRA: &[u8] = b"!#$%&'*+-.^_`|~";
    if name.is_empty()
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || TCHAR_EXTRA.contains(&b))
    {
        return Err(WebhookError::InvalidHeader("name must be an HTTP token"));
    }
    if RESERVED_HEADERS.contains(&name.to_ascii_lowercase().as_str()) {
        return Err(WebhookError::InvalidHeader(
            "name is reserved by the kernel",
        ));
    }
    if value.bytes().any(|b| (b < 0x20 && b != b'\t') || b == 0x7f) {
        return Err(WebhookError::InvalidHeader(
            "value contains control characters",
        ));
    }
    Ok(())
}

/// Addresses a webhook must never reach by default.
pub fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_blocked_v4(v4),
        IpAddr::V6(v6) => {
            let seg = v6.segments();
            // IPv4-mapped/compatible (::ffff:a.b.c.d, ::a.b.c.d) and NAT64 (64:ff9b::/96)
            if let Some(v4) = v6.to_ipv4() {
                if is_blocked_v4(v4) {
                    return true;
                }
            }
            if seg[0] == 0x0064 && seg[1] == 0xff9b && seg[2..6] == [0, 0, 0, 0] {
                let v4 = Ipv4Addr::new(
                    (seg[6] >> 8) as u8,
                    seg[6] as u8,
                    (seg[7] >> 8) as u8,
                    seg[7] as u8,
                );
                if is_blocked_v4(v4) {
                    return true;
                }
            }
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                || (seg[0] & 0xfe00) == 0xfc00 // unique local (incl. fd00:ec2::254 metadata)
                || (seg[0] & 0xffc0) == 0xfe80 // link-local
                || (seg[0] & 0xffc0) == 0xfec0 // deprecated site-local
                || (seg[0] == 0x2001 && seg[1] == 0x0db8) // documentation
        }
    }
}

fn is_blocked_v4(ip: Ipv4Addr) -> bool {
    let o = ip.octets();
    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local() // 169.254.0.0/16 incl. 169.254.169.254 metadata
        || ip.is_unspecified()
        || ip.is_broadcast()
        || ip.is_multicast()
        || ip.is_documentation()
        || o[0] == 0 // 0.0.0.0/8
        || (o[0] == 100 && (o[1] & 0xc0) == 64) // CGNAT 100.64.0.0/10
        || (o[0] == 192 && o[1] == 0 && o[2] == 0) // 192.0.0.0/24 (incl. Oracle metadata)
        || (o[0] == 198 && (o[1] & 0xfe) == 18) // benchmarking 198.18.0.0/15
        || o[0] >= 240 // reserved 240.0.0.0/4
}

/// Picks the address to connect to. If any resolved address is blocked the
/// whole target is refused, so round-robin DNS cannot smuggle one in.
pub fn vet_addrs(addrs: &[SocketAddr], allow_private: bool) -> Result<SocketAddr, WebhookError> {
    let first = *addrs.first().ok_or(WebhookError::Resolve)?;
    if !allow_private && addrs.iter().any(|a| is_blocked_ip(a.ip())) {
        return Err(WebhookError::BlockedAddress);
    }
    Ok(first)
}

pub fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    const BLOCK: usize = 64;
    let mut k = [0u8; BLOCK];
    if key.len() > BLOCK {
        k[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36u8; BLOCK];
    let mut opad = [0x5cu8; BLOCK];
    for i in 0..BLOCK {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }
    let inner = Sha256::new()
        .chain_update(ipad)
        .chain_update(msg)
        .finalize();
    Sha256::new()
        .chain_update(opad)
        .chain_update(inner)
        .finalize()
        .into()
}

pub fn signature(secret: &str, timestamp: u64, body: &str) -> String {
    let mac = hmac_sha256(secret.as_bytes(), format!("{timestamp}.{body}").as_bytes());
    format!("sha256={}", hex::encode(mac))
}

/// The row as it may leave the server: `sensitive` fields and password
/// columns removed. Unknown entity → password columns still removed.
pub fn redacted_body(entities: &[EntityNode], entity: &str, payload: &Value) -> String {
    let mut value = payload.clone();
    match entities.iter().find(|e| names_match(&e.name, entity)) {
        Some(e) => crate::authz::redact_sensitive(e, &mut value),
        None => {
            if let Value::Object(map) = &mut value {
                map.remove("password");
                map.remove("password_hash");
            }
        }
    }
    value.to_string()
}

fn build_request(
    method: &str,
    target: &Target,
    headers: &[(String, String)],
    event: &str,
    body: &str,
    timestamp: u64,
    secret: &str,
) -> String {
    let mut req = format!(
        "{method} {path} HTTP/1.1\r\nHost: {host}\r\nUser-Agent: cronus-webhook\r\nContent-Type: application/json\r\nContent-Length: {len}\r\nConnection: close\r\nX-Cronus-Event: {event}\r\nX-Cronus-Timestamp: {timestamp}\r\nX-Cronus-Signature: {sig}\r\n",
        path = target.path,
        host = target.host_header(),
        len = body.len(),
        sig = signature(secret, timestamp, body),
    );
    for (k, v) in headers {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    req.push_str("\r\n");
    req.push_str(body);
    req
}

/// Validates and delivers one webhook. Returns the HTTP status code.
pub async fn deliver(
    method: &str,
    url: &str,
    headers: &[(String, String)],
    event: &str,
    body: &str,
    secret: &str,
    allow_private: bool,
) -> Result<u16, WebhookError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::time::timeout;

    let method = validate_method(method)?;
    let target = parse_url(url)?;
    for (k, v) in headers {
        validate_header(k, v)?;
    }
    if !event
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(WebhookError::InvalidHeader("event name"));
    }

    let addrs: Vec<SocketAddr> = timeout(
        RESOLVE_TIMEOUT,
        tokio::net::lookup_host((target.host.as_str(), target.port)),
    )
    .await
    .map_err(|_| WebhookError::Resolve)?
    .map_err(|_| WebhookError::Resolve)?
    .collect();
    let addr = vet_addrs(&addrs, allow_private)?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let request = build_request(method, &target, headers, event, body, timestamp, secret);

    let mut stream = timeout(CONNECT_TIMEOUT, tokio::net::TcpStream::connect(addr))
        .await
        .map_err(|_| WebhookError::Io("connecting (timeout)"))?
        .map_err(|_| WebhookError::Io("connecting"))?;
    timeout(IO_TIMEOUT, stream.write_all(request.as_bytes()))
        .await
        .map_err(|_| WebhookError::Io("sending (timeout)"))?
        .map_err(|_| WebhookError::Io("sending"))?;

    let mut buf = [0u8; 256];
    let mut read = 0;
    while read < buf.len() && !buf[..read].contains(&b'\n') {
        let n = timeout(IO_TIMEOUT, stream.read(&mut buf[read..]))
            .await
            .map_err(|_| WebhookError::Io("reading response (timeout)"))?
            .map_err(|_| WebhookError::Io("reading response"))?;
        if n == 0 {
            break;
        }
        read += n;
    }
    let status_line = String::from_utf8_lossy(&buf[..read]);
    status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .ok_or(WebhookError::Io("parsing response status"))
}

fn signing_secret() -> Result<String, WebhookError> {
    use std::sync::OnceLock;
    if let Ok(s) = std::env::var(SECRET_ENV) {
        if !s.is_empty() {
            if s.len() < crate::auth::MIN_SECRET_BYTES {
                return Err(WebhookError::Secret(format!(
                    "{SECRET_ENV} must be at least {} bytes",
                    crate::auth::MIN_SECRET_BYTES
                )));
            }
            return Ok(s);
        }
    }
    static FILE_SECRET: OnceLock<String> = OnceLock::new();
    Ok(FILE_SECRET
        .get_or_init(|| {
            match crate::auth::load_or_create_key_file(std::path::Path::new(SECRET_PATH)) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("  \x1b[31m✗\x1b[0m {}", e);
                    std::process::exit(1);
                }
            }
        })
        .clone())
}

/// Fire-and-forget entry point used by the HTTP handlers. Logs once, here.
pub async fn fire(hook: WebhookHook, event: String, body: String) {
    let allow_private = std::env::var(ALLOW_PRIVATE_ENV)
        .map(|v| v == "1")
        .unwrap_or(false);
    let origin = parse_url(&hook.url)
        .map(|t| t.origin())
        .unwrap_or_else(|_| "<invalid url>".into());
    let result = match signing_secret() {
        Ok(secret) => {
            deliver(
                &hook.method,
                &hook.url,
                &hook.headers,
                &event,
                &body,
                &secret,
                allow_private,
            )
            .await
        }
        Err(e) => Err(e),
    };
    match result {
        Ok(status) if (200..300).contains(&status) => {}
        Ok(status) => {
            eprintln!("  \x1b[33m⚠\x1b[0m Webhook {event} -> {origin} returned HTTP {status}")
        }
        Err(e) => eprintln!("  \x1b[33m⚠\x1b[0m Webhook {event} -> {origin} not delivered: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── URL validation ──

    #[test]
    fn parses_plain_http_urls() {
        assert_eq!(
            parse_url("http://example.com").unwrap(),
            Target {
                host: "example.com".into(),
                port: 80,
                path: "/".into()
            }
        );
        assert_eq!(
            parse_url("http://Hooks.Example.com:8080/a/b?x=1#frag").unwrap(),
            Target {
                host: "hooks.example.com".into(),
                port: 8080,
                path: "/a/b?x=1".into()
            }
        );
        assert_eq!(parse_url("http://example.com?x=1").unwrap().path, "/?x=1");
        let v6 = parse_url("http://[2606:4700::1111]:9000/x").unwrap();
        assert_eq!(v6.host, "2606:4700::1111");
        assert_eq!(v6.host_header(), "[2606:4700::1111]:9000");
    }

    #[test]
    fn refuses_https_instead_of_sending_cleartext() {
        assert_eq!(
            parse_url("https://example.com/hook"),
            Err(WebhookError::TlsUnsupported)
        );
        assert_eq!(
            parse_url("HTTPS://example.com/hook"),
            Err(WebhookError::TlsUnsupported)
        );
    }

    #[test]
    fn rejects_malformed_urls() {
        for bad in [
            "ftp://example.com",
            "example.com/hook",
            "http://",
            "http:///path",
            "http://user:pass@example.com/",
            "http://example.com:0/",
            "http://example.com:70000/",
            "http://example.com:abc/",
            "http://exa_mple.com/",
            "http://[::1/",
            "http://[nope]/",
            "http://exámple.com/",
        ] {
            assert!(
                matches!(parse_url(bad), Err(WebhookError::InvalidUrl(_))),
                "accepted {bad}"
            );
        }
    }

    #[test]
    fn rejects_crlf_in_url() {
        for bad in [
            "http://example.com/\r\nX-Injected: 1",
            "http://example.com\r\n/",
            "http://example.com/a b",
            "http://example.com/\n",
            "http://example.com/\0",
        ] {
            assert!(
                matches!(parse_url(bad), Err(WebhookError::InvalidUrl(_))),
                "accepted {bad:?}"
            );
        }
    }

    #[test]
    fn rejects_crlf_and_reserved_names_in_headers() {
        assert!(validate_header("Authorization", "Bearer abc").is_ok());
        assert!(validate_header("X-Token", "a\tb").is_ok());
        assert!(validate_header("X-Token", "abc\r\nX-Evil: 1").is_err());
        assert!(validate_header("X-Token", "abc\n").is_err());
        assert!(validate_header("X-Evil\r\nA", "1").is_err());
        assert!(validate_header("Bad Name", "1").is_err());
        assert!(validate_header("Bad:Name", "1").is_err());
        assert!(validate_header("", "1").is_err());
        assert!(validate_header("Host", "internal").is_err());
        assert!(validate_header("content-length", "0").is_err());
        assert!(validate_header("X-Cronus-Signature", "forged").is_err());
    }

    #[test]
    fn only_body_methods_allowed() {
        assert_eq!(validate_method("POST"), Ok("POST"));
        assert_eq!(validate_method("PUT"), Ok("PUT"));
        assert!(validate_method("GET\r\n").is_err());
        assert!(validate_method("CONNECT").is_err());
        assert!(validate_method("post").is_err());
    }

    // ── SSRF: address blocking ──

    #[test]
    fn blocks_loopback_private_link_local_and_metadata() {
        for ip in [
            "127.0.0.1",
            "127.1.2.3",
            "10.0.0.1",
            "172.16.5.4",
            "172.31.255.255",
            "192.168.1.1",
            "169.254.169.254",
            "0.0.0.0",
            "0.1.2.3",
            "100.64.0.1",
            "192.0.0.192",
            "255.255.255.255",
            "224.0.0.1",
            "240.0.0.1",
            "::1",
            "::",
            "fe80::1",
            "fc00::1",
            "fd00:ec2::254",
            "::ffff:127.0.0.1",
            "::ffff:169.254.169.254",
            "::ffff:10.0.0.1",
            "64:ff9b::a9fe:a9fe",
        ] {
            assert!(is_blocked_ip(ip.parse().unwrap()), "{ip} should be blocked");
        }
        for ip in [
            "93.184.216.34",
            "1.1.1.1",
            "172.32.0.1",
            "2606:4700::1111",
            "::ffff:8.8.8.8",
        ] {
            assert!(
                !is_blocked_ip(ip.parse().unwrap()),
                "{ip} should be allowed"
            );
        }
    }

    #[test]
    fn vet_refuses_any_blocked_address_unless_allowed() {
        let public: SocketAddr = "93.184.216.34:80".parse().unwrap();
        let private: SocketAddr = "10.0.0.5:80".parse().unwrap();
        assert_eq!(vet_addrs(&[public], false), Ok(public));
        assert_eq!(
            vet_addrs(&[private], false),
            Err(WebhookError::BlockedAddress)
        );
        assert_eq!(
            vet_addrs(&[public, private], false),
            Err(WebhookError::BlockedAddress)
        );
        assert_eq!(vet_addrs(&[private], true), Ok(private));
        assert_eq!(vet_addrs(&[], true), Err(WebhookError::Resolve));
    }

    #[tokio::test]
    async fn deliver_refuses_localhost_by_default() {
        let err = deliver(
            "POST",
            "http://localhost:9/hook",
            &[],
            "create",
            "{}",
            "k",
            false,
        )
        .await;
        assert_eq!(err, Err(WebhookError::BlockedAddress));
        let err = deliver(
            "POST",
            "http://127.0.0.1:9/hook",
            &[],
            "create",
            "{}",
            "k",
            false,
        )
        .await;
        assert_eq!(err, Err(WebhookError::BlockedAddress));
    }

    #[tokio::test]
    async fn deliver_refuses_https_and_injection_before_connecting() {
        let hdr = vec![("X-A".to_string(), "1\r\nX-B: 2".to_string())];
        assert_eq!(
            deliver(
                "POST",
                "https://example.com/",
                &[],
                "create",
                "{}",
                "k",
                true
            )
            .await,
            Err(WebhookError::TlsUnsupported)
        );
        assert!(matches!(
            deliver(
                "POST",
                "http://127.0.0.1:9/",
                &hdr,
                "create",
                "{}",
                "k",
                true
            )
            .await,
            Err(WebhookError::InvalidHeader(_))
        ));
    }

    // ── signing + redaction ──

    #[test]
    fn hmac_sha256_matches_rfc4231() {
        // RFC 4231 test case 2
        assert_eq!(
            hex::encode(hmac_sha256(b"Jefe", b"what do ya want for nothing?")),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
        // RFC 4231 test case 6 (key longer than block size)
        assert_eq!(
            hex::encode(hmac_sha256(
                &[0xaa; 131],
                b"Test Using Larger Than Block-Size Key - Hash Key First"
            )),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
        );
    }

    fn user_entity() -> EntityNode {
        let src = "entity User {\n  email email!\n  name string\n  ssn string sensitive\n  password string! sensitive\n}\n";
        match crate::parser::parse(src).unwrap().remove(0) {
            crate::parser::AstNode::Entity(e) => e,
            _ => unreachable!(),
        }
    }

    #[test]
    fn body_is_redacted() {
        let entities = vec![user_entity()];
        let row = serde_json::json!({"id": "1", "email": "a@b.c", "name": "A", "ssn": "123", "password": "x", "password_hash": "h"});
        let body: Value = serde_json::from_str(&redacted_body(&entities, "users", &row)).unwrap();
        assert_eq!(
            body,
            serde_json::json!({"id": "1", "email": "a@b.c", "name": "A"})
        );
        // unknown entity still drops password columns
        let body: Value = serde_json::from_str(&redacted_body(&[], "ghost", &row)).unwrap();
        assert!(body.get("password").is_none() && body.get("password_hash").is_none());
    }

    #[tokio::test]
    async fn delivers_signed_request_to_allowed_private_target() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut req = Vec::new();
            let mut buf = [0u8; 1024];
            loop {
                let n = sock.read(&mut buf).await.unwrap();
                req.extend_from_slice(&buf[..n]);
                let text = String::from_utf8_lossy(&req);
                if let Some(idx) = text.find("\r\n\r\n") {
                    let len: usize = text
                        .lines()
                        .find_map(|l| l.strip_prefix("Content-Length: "))
                        .unwrap()
                        .trim()
                        .parse()
                        .unwrap();
                    if req.len() >= idx + 4 + len || n == 0 {
                        break;
                    }
                }
            }
            sock.write_all(b"HTTP/1.1 204 No Content\r\n\r\n")
                .await
                .unwrap();
            String::from_utf8(req).unwrap()
        });

        let body = redacted_body(
            &[user_entity()],
            "User",
            &serde_json::json!({"id": "1", "name": "A", "password": "x"}),
        );
        let url = format!("http://127.0.0.1:{port}/hooks/in?src=cronus");
        let headers = vec![("X-Api-Token".to_string(), "t0k".to_string())];
        let status = deliver("POST", &url, &headers, "create", &body, "s3cret", true)
            .await
            .unwrap();
        assert_eq!(status, 204);

        let req = server.await.unwrap();
        assert!(
            req.starts_with("POST /hooks/in?src=cronus HTTP/1.1\r\n"),
            "{req}"
        );
        assert!(req.contains(&format!("\r\nHost: 127.0.0.1:{port}\r\n")));
        assert!(req.contains("\r\nX-Api-Token: t0k\r\n"));
        assert!(req.contains("\r\nX-Cronus-Event: create\r\n"));
        assert!(!req.contains("password"));
        let ts: u64 = req
            .lines()
            .find_map(|l| l.strip_prefix("X-Cronus-Timestamp: "))
            .unwrap()
            .parse()
            .unwrap();
        let sig = req
            .lines()
            .find_map(|l| l.strip_prefix("X-Cronus-Signature: "))
            .unwrap();
        assert_eq!(sig, signature("s3cret", ts, &body));
        assert!(req.ends_with(&format!("\r\n\r\n{body}")));
    }
}
