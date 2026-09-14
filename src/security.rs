//! CRONUS Security Module — centralized security primitives
//!
//! Every generated app enforces these automatically. No opt-out.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use base64::Engine as _;
use rand::RngCore;

// ══════════════════════════════════════════════════
// HTML ESCAPING — prevents XSS
// ══════════════════════════════════════════════════

/// Escape HTML special characters. MUST be called on every user/DB value
/// before inserting into HTML. No exceptions.
#[inline]
pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            '/' => out.push_str("&#x2F;"),
            _ => out.push(c),
        }
    }
    out
}

// ══════════════════════════════════════════════════
// SQL SAFETY — prevents injection
// ══════════════════════════════════════════════════

/// Validate that a string is a safe SQL identifier (column/table name).
/// Only allows alphanumeric + underscore. Rejects everything else.
#[inline]
pub fn is_safe_identifier(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Sanitize a column name — returns None if invalid.
pub fn safe_column(name: &str) -> Option<&str> {
    if is_safe_identifier(name) { Some(name) } else { None }
}

// ══════════════════════════════════════════════════
// RATE LIMITING — prevents brute force
// ══════════════════════════════════════════════════

pub struct RateLimiter {
    buckets: Mutex<HashMap<String, (u32, Instant)>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self { buckets: Mutex::new(HashMap::new()) }
    }

    /// Check if request is allowed. Returns false if rate exceeded.
    /// `max_requests` in `window_secs` per key.
    pub fn check(&self, key: &str, max_requests: u32, window_secs: u64) -> bool {
        let mut buckets = self.buckets.lock().unwrap();
        let now = Instant::now();

        let entry = buckets.entry(key.to_string()).or_insert((0, now));

        // Reset window if expired
        if now.duration_since(entry.1).as_secs() >= window_secs {
            entry.0 = 0;
            entry.1 = now;
        }

        if entry.0 >= max_requests {
            false
        } else {
            entry.0 += 1;
            true
        }
    }

    /// Cleanup stale entries (call periodically)
    pub fn cleanup(&self, max_age_secs: u64) {
        let mut buckets = self.buckets.lock().unwrap();
        let now = Instant::now();
        buckets.retain(|_, (_, t)| now.duration_since(*t).as_secs() < max_age_secs);
    }
}

// ══════════════════════════════════════════════════
// SECURITY HEADERS — every response
// ══════════════════════════════════════════════════

/// Returns security headers that MUST be set on every HTTP response.
pub fn security_headers() -> Vec<(&'static str, &'static str)> {
    vec![
        ("x-content-type-options", "nosniff"),
        ("x-frame-options", "SAMEORIGIN"),
        ("x-xss-protection", "0"),
        ("referrer-policy", "strict-origin-when-cross-origin"),
        ("permissions-policy", "camera=(), microphone=(), geolocation=(), payment=()"),
        ("cross-origin-opener-policy", "same-origin"),
    ]
}

// ══════════════════════════════════════════════════
// CSP NONCE — per-request Content Security Policy
// ══════════════════════════════════════════════════

/// Generate a cryptographically random nonce (16 bytes, base64-encoded).
/// A new nonce MUST be generated for every HTML response — never reuse.
pub fn generate_csp_nonce() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

// ── Kernel script marking ─────────────────────────────────────────────
//
// The per-request nonce must only reach scripts the kernel itself authored.
// Adding it to every `<script` of the final HTML (after DB values were
// interpolated) would hand a valid nonce to any stored `<script>` payload.
//
// So kernel templates emit `<script{script_nonce_attr()}>` at generation
// time. The attribute carries a per-process random marker that no request,
// row or template can know; `finalize_script_nonces` swaps it for the
// per-request nonce in `html_response`. Markers are only emitted after the
// HTTP server calls `enable_script_nonces()`, so CLI renders (deploy, audit,
// unit tests) keep producing plain `<script>` tags.

static SCRIPT_NONCES_ENABLED: AtomicBool = AtomicBool::new(false);

fn nonce_marker() -> &'static str {
    static MARKER: OnceLock<String> = OnceLock::new();
    MARKER.get_or_init(|| {
        let mut bytes = [0u8; 18];
        rand::thread_rng().fill_bytes(&mut bytes);
        format!("cronus-nonce-marker-{}", hex::encode(bytes))
    })
}

fn marker_attr() -> &'static str {
    static ATTR: OnceLock<String> = OnceLock::new();
    ATTR.get_or_init(|| format!(" nonce=\"{}\"", nonce_marker()))
}

/// Called once by the HTTP server before it renders anything.
pub fn enable_script_nonces() {
    SCRIPT_NONCES_ENABLED.store(true, Ordering::Relaxed);
}

pub fn script_nonces_enabled() -> bool {
    SCRIPT_NONCES_ENABLED.load(Ordering::Relaxed)
}

/// Attribute to place right after `<script` in kernel-authored templates:
/// `format!("<script{nonce}>…")`. Empty when nonces are disabled.
pub fn script_nonce_attr() -> &'static str {
    if script_nonces_enabled() { marker_attr() } else { "" }
}

/// Mark every `<script` tag in markup the kernel or the app developer wrote
/// (constants, `.cronus` templates, `source` HTML files).
///
/// ONLY call this on markup that contains no DB/user values yet — marking
/// after interpolation would re-create the regex-nonce hole.
pub fn mark_kernel_scripts(markup: &str) -> String {
    add_attr_to_script_tags(markup, script_nonce_attr())
}

fn add_attr_to_script_tags(html: &str, attr: &str) -> String {
    if attr.is_empty() {
        return html.to_string();
    }
    let lower = html.to_ascii_lowercase();
    let mut out = String::with_capacity(html.len() + attr.len() * 4);
    let mut cursor = 0;
    while let Some(rel) = lower[cursor..].find("<script") {
        let tag_name_end = cursor + rel + "<script".len();
        out.push_str(&html[cursor..tag_name_end]);
        cursor = tag_name_end;
        let next = lower[cursor..].chars().next();
        let is_script_tag = matches!(next, Some('>') | Some(' ') | Some('\n') | Some('\t') | Some('\r') | Some('/'));
        let tag_end = lower[cursor..].find('>').map(|p| cursor + p).unwrap_or(lower.len());
        let already_nonced = lower[cursor..tag_end].contains(" nonce=");
        if is_script_tag && !already_nonced {
            out.push_str(attr);
        }
    }
    out.push_str(&html[cursor..]);
    out
}

/// Swap kernel markers for the per-request nonce. Unmarked scripts (anything
/// that came from data) stay without a nonce and are blocked by the CSP.
pub fn finalize_script_nonces(html: &str, nonce: &str) -> String {
    html.replace(nonce_marker(), nonce)
}

/// Remove markers from HTML that leaves the server without a nonce CSP, so
/// the per-process marker is never disclosed.
pub fn strip_script_nonce_markers(html: &str) -> String {
    html.replace(marker_attr(), "").replace(nonce_marker(), "")
}

/// Whether `eval`-style evaluation must be allowed for this page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvalPolicy {
    /// Kernel runtime only — no `eval`/`new Function`.
    Forbid,
    /// The opt-in Voodoo.js runtime evaluates attribute expressions.
    AllowForVoodoo,
}

/// Exact external scripts the kernel loads. Host-wide CDN allowances are
/// not used: anything on cdn.jsdelivr.net would otherwise be executable.
pub const KERNEL_SCRIPT_URLS: &[&str] = &[
    "https://cdn.tailwindcss.com",
    "https://cdn.jsdelivr.net/npm/chart.js",
    "https://cdn.jsdelivr.net/npm/mermaid/dist/mermaid.min.js",
    crate::voodoo::SCRIPT_SRC,
    UNICORN_STUDIO_SCRIPT_URL,
];

pub const UNICORN_STUDIO_SCRIPT_URL: &str =
    "https://cdn.jsdelivr.net/gh/hiunicornstudio/unicornstudio.js@v1.4.29/dist/unicornStudio.umd.js";

/// Build the Content-Security-Policy header value.
///
/// With a nonce (server enabled script marking) `script-src` has no
/// `'unsafe-inline'`: only kernel-marked scripts run. Without a nonce the
/// legacy inline allowance is kept because nothing is marked.
///
/// `script-src-attr 'unsafe-inline'` stays: kernel renderers outside the
/// session/escaping sprint still emit inline `on*=` handlers.
pub fn csp_header_value(nonce: Option<&str>, eval: EvalPolicy) -> String {
    let mut script_src = String::from("'self'");
    match nonce {
        Some(n) => script_src.push_str(&format!(" 'nonce-{}'", n)),
        None => script_src.push_str(" 'unsafe-inline'"),
    }
    if eval == EvalPolicy::AllowForVoodoo {
        script_src.push_str(" 'unsafe-eval'");
    }
    for url in KERNEL_SCRIPT_URLS {
        script_src.push(' ');
        script_src.push_str(url);
    }
    format!(
        "default-src 'self'; \
         script-src {script_src}; \
         script-src-attr 'unsafe-inline'; \
         style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
         font-src 'self' https://fonts.gstatic.com; \
         img-src 'self' https: data:; \
         connect-src 'self' https://unicorn.studio https://assets.unicorn.studio https://storage.googleapis.com; \
         worker-src blob:; \
         object-src 'none'; \
         base-uri 'self'; \
         frame-ancestors 'self'; \
         frame-src 'self'"
    )
}

// ══════════════════════════════════════════════════
// INPUT VALIDATION
// ══════════════════════════════════════════════════

/// Validate an email address (basic but correct).
pub fn is_valid_email(email: &str) -> bool {
    if email.len() < 3 || email.len() > 254 { return false; }
    let parts: Vec<&str> = email.splitn(2, '@').collect();
    if parts.len() != 2 { return false; }
    !parts[0].is_empty() && !parts[1].is_empty() && parts[1].contains('.')
}

/// Validate password strength.
pub fn is_strong_password(password: &str) -> bool {
    password.len() >= 8
}

// Session cookies are built in `crate::session` (HttpOnly, SameSite=Lax,
// Secure in production / behind HTTPS).

#[cfg(test)]
mod tests {
    use super::*;

    // These tests never call `enable_script_nonces()`: the flag is
    // process-wide and would leak markers into renderer tests running in
    // parallel. They drive the marker attribute directly instead.

    #[test]
    fn marking_adds_marker_only_to_real_script_tags() {
        let attr = marker_attr();
        let html = r#"<script>a()</script><SCRIPT src="x.js"></SCRIPT><scripting>no</scripting><script nonce="n">b()</script>"#;
        let marked = add_attr_to_script_tags(html, attr);
        assert_eq!(marked.matches(nonce_marker()).count(), 2, "{marked}");
        assert!(marked.contains("<scripting>"));
        assert!(marked.contains(r#"<script nonce="n">"#), "already-nonced tag left alone");
        assert_eq!(add_attr_to_script_tags(html, ""), html, "disabled = unchanged");
    }

    #[test]
    fn only_marked_scripts_receive_the_request_nonce() {
        // Kernel template marked before interpolation; the row value is not.
        let template = add_attr_to_script_tags("<div>{row}</div><script>kernel()</script>", marker_attr());
        let row = r#"<script nonce="guess">steal()</script><script>steal()</script>"#;
        let page = template.replace("{row}", &html_escape(row)).replace("</div>", &format!("{row}</div>"));
        let out = finalize_script_nonces(&page, "REQNONCE");
        assert_eq!(out.matches(r#"nonce="REQNONCE""#).count(), 1, "{out}");
        assert!(out.contains(r#"<script nonce="REQNONCE">kernel()</script>"#));
        assert!(out.contains("<script>steal()</script>"), "data script stays unnonced");
        assert!(!out.contains(nonce_marker()));
    }

    #[test]
    fn strip_removes_marker_without_a_nonce_csp() {
        let marked = add_attr_to_script_tags("<script>x()</script>", marker_attr());
        let stripped = strip_script_nonce_markers(&marked);
        assert_eq!(stripped, "<script>x()</script>");
    }

    #[test]
    fn marker_is_unguessable_and_stable_per_process() {
        assert!(nonce_marker().starts_with("cronus-nonce-marker-"));
        assert_eq!(nonce_marker().len(), "cronus-nonce-marker-".len() + 36);
        assert_eq!(nonce_marker(), nonce_marker());
    }

    #[test]
    fn csp_with_nonce_drops_unsafe_inline_and_host_wide_cdn() {
        let csp = csp_header_value(Some("abc"), EvalPolicy::Forbid);
        let script_src = csp.split(';').find(|d| d.trim().starts_with("script-src ")).unwrap();
        assert!(script_src.contains("'nonce-abc'"));
        assert!(!script_src.contains("'unsafe-inline'"));
        assert!(!script_src.contains("'unsafe-eval'"));
        assert!(!script_src.split_whitespace().any(|t| t == "https://cdn.jsdelivr.net"), "{script_src}");
        assert!(csp.contains("object-src 'none'") && csp.contains("base-uri 'self'"));
    }

    #[test]
    fn csp_eval_only_for_voodoo_and_inline_only_without_nonce() {
        assert!(csp_header_value(Some("abc"), EvalPolicy::AllowForVoodoo).contains("'unsafe-eval'"));
        let legacy = csp_header_value(None, EvalPolicy::Forbid);
        assert!(legacy.contains("'unsafe-inline'"));
        assert!(!legacy.contains("'nonce-"));
    }
}
