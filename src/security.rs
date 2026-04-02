//! CRONUS Security Module — centralized security primitives
//!
//! Every generated app enforces these automatically. No opt-out.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

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
        ("X-Content-Type-Options", "nosniff"),
        ("X-Frame-Options", "DENY"),
        ("X-XSS-Protection", "0"),
        ("Referrer-Policy", "strict-origin-when-cross-origin"),
        ("Permissions-Policy", "camera=(), microphone=(), geolocation=(), payment=()"),
        ("Cross-Origin-Opener-Policy", "same-origin"),
    ]
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

/// Build a secure cookie string. HttpOnly + SameSite=Strict + Secure always set.
pub fn secure_cookie(name: &str, value: &str, max_age_secs: u64, path: &str) -> String {
    format!(
        "{}={}; Path={}; HttpOnly; SameSite=Strict; Secure; Max-Age={}",
        name, value, path, max_age_secs
    )
}

/// Build a cookie deletion string.
pub fn delete_cookie(name: &str, path: &str) -> String {
    format!(
        "{}=; Path={}; HttpOnly; SameSite=Strict; Secure; Max-Age=0",
        name, path
    )
}
