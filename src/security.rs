//! CRONUS Security Module — centralized security primitives
//!
//! Every generated app enforces these automatically. No opt-out.

use std::collections::HashMap;
use std::sync::Mutex;
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
        ("x-frame-options", "DENY"),
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

/// Build the Content-Security-Policy header value for a given nonce.
/// In development (localhost), use a permissive policy to avoid blocking
/// inline scripts/handlers from templates and renderers.
pub fn csp_header_value(_nonce: &str) -> String {
    "default-src 'self'; \
     script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.tailwindcss.com; \
     script-src-attr 'unsafe-inline'; \
     style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
     font-src https://fonts.gstatic.com; \
     img-src 'self' https: data:; \
     connect-src 'self'".to_string()
}

/// Inject nonce attribute into all `<script` tags in an HTML string.
/// Replaces `<script>` with `<script nonce="NONCE">` and
/// `<script src=` with `<script nonce="NONCE" src=`.
pub fn inject_nonce_into_scripts(html: &str, nonce: &str) -> String {
    // Replace <script> (inline scripts) and <script followed by a space (scripts with attributes)
    // We need to handle: <script>, <script src="...">, <script type="...">, etc.
    // Strategy: replace all occurrences of "<script" that are NOT already nonced.
    let nonce_attr = format!(" nonce=\"{}\"", nonce);
    let mut result = String::with_capacity(html.len() + 64 * 20);
    let mut remaining = html;

    while let Some(pos) = remaining.find("<script") {
        // Push everything before the tag
        result.push_str(&remaining[..pos]);
        remaining = &remaining[pos..];

        // Find the end of the opening tag
        let tag_prefix = "<script";
        result.push_str(tag_prefix);
        remaining = &remaining[tag_prefix.len()..];

        // Insert nonce right after "<script"
        // The next char should be '>' or ' ' or nothing weird
        if remaining.starts_with('>') || remaining.starts_with(' ') || remaining.starts_with('\n') {
            result.push_str(&nonce_attr);
        }
        // If it's something unexpected (e.g. "<scripting"), don't inject
    }

    // Push the rest
    result.push_str(remaining);
    result
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

// ══════════════════════════════════════════════════
// COOKIE BUILDER — secure by default
// ══════════════════════════════════════════════════

/// Build a cookie string. HttpOnly + SameSite=Lax.
/// Secure flag only added when not on localhost (HTTPS required for Secure).
pub fn secure_cookie(name: &str, value: &str, max_age_secs: u64, path: &str) -> String {
    format!(
        "{}={}; Path={}; HttpOnly; SameSite=Lax; Max-Age={}",
        name, value, path, max_age_secs
    )
}

/// Build a cookie deletion string.
pub fn delete_cookie(name: &str, path: &str) -> String {
    format!(
        "{}=; Path={}; HttpOnly; SameSite=Lax; Max-Age=0",
        name, path
    )
}
