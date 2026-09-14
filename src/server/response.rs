#![allow(dead_code, unused_imports)]
//! HTTP response builders extracted from main.rs.
//!
//! Self-contained helpers that build `Response<Full<Bytes>>` values
//! for JSON, HTML, and error responses.

use bytes::Bytes;
use http_body_util::Full;
use hyper::{Response, StatusCode};
use serde_json::Value;
use std::sync::atomic::Ordering;

// ──────────────────────────────────────────────
// CORS origin
// ──────────────────────────────────────────────

pub(crate) fn cors_origin() -> String {
    std::env::var("CRONUS_CORS_ORIGIN").unwrap_or_else(|_| "same-origin".to_string())
}

// ──────────────────────────────────────────────
// JSON response
// ──────────────────────────────────────────────

pub(crate) fn json_response(status: StatusCode, body: Value) -> Response<Full<Bytes>> {
    let origin = cors_origin();
    let mut builder = Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header(
            "Access-Control-Allow-Methods",
            "GET, POST, PATCH, PUT, DELETE, OPTIONS",
        )
        .header(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        );
    if origin != "same-origin" {
        builder = builder.header("Access-Control-Allow-Origin", origin);
    }
    for (k, v) in crate::security::security_headers() {
        builder = builder.header(k, v);
    }
    builder
        .body(Full::new(Bytes::from(body.to_string())))
        .unwrap()
}

// ──────────────────────────────────────────────
// HTML response
// ──────────────────────────────────────────────

pub(crate) fn html_response(body: String) -> Response<Full<Bytes>> {
    let nonce_attr = crate::security::script_nonce_attr();
    let mut final_body = inject_audit_if_enabled(body);
    final_body = crate::voodoo::inject_into_html(final_body);
    // Inject SSE client JS into every HTML page (before </body>)
    if let Some(pos) = final_body.rfind("</body>") {
        let sse_script = format!(
            "<script{}>{}</script>\n",
            nonce_attr,
            crate::sse::SSE_CLIENT_JS
        );
        final_body.insert_str(pos, &sse_script);
    }
    // Inject debug overlay JS when debug mode is active (env var or CLI flag)
    let debug_active = crate::DEBUG_MODE.load(Ordering::Relaxed)
        || std::env::var("CRONUS_DEBUG")
            .map(|v| v == "1" || v == "true")
            .unwrap_or(false);
    if debug_active {
        if let Some(pos) = final_body.rfind("</body>") {
            let debug_script = format!(
                "<script{}>{}</script>\n",
                nonce_attr,
                crate::render::CRONUS_DEBUG_JS
            );
            final_body.insert_str(pos, &debug_script);
        }
    }
    let eval = if crate::voodoo::enabled() {
        crate::security::EvalPolicy::AllowForVoodoo
    } else {
        crate::security::EvalPolicy::Forbid
    };
    // Only kernel-marked scripts receive the per-request nonce; scripts that
    // arrived through data carry no marker and are blocked by the CSP.
    let csp_value = if crate::security::script_nonces_enabled() {
        let nonce = crate::security::generate_csp_nonce();
        final_body = crate::security::finalize_script_nonces(&final_body, &nonce);
        crate::security::csp_header_value(Some(&nonce), eval)
    } else {
        crate::security::csp_header_value(None, eval)
    };
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Content-Security-Policy", csp_value);
    for (k, v) in crate::security::security_headers() {
        builder = builder.header(k, v);
    }
    builder.body(Full::new(Bytes::from(final_body))).unwrap()
}

// ──────────────────────────────────────────────
// Forbidden response (403 page)
// ──────────────────────────────────────────────

pub(crate) fn forbidden_response(message: &str) -> Response<Full<Bytes>> {
    let html = format!(
        r##"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>403 Forbidden</title>
<style>
  * {{ margin:0; padding:0; box-sizing:border-box; }}
  body {{ background:oklch(0.08 0 0); color:oklch(0.7 0 0); font-family:'SF Pro','Inter',system-ui,sans-serif;
         display:flex; align-items:center; justify-content:center; min-height:100vh; }}
  .box {{ text-align:center; max-width:400px; padding:40px; }}
  .code {{ font-size:64px; font-weight:700; color:oklch(0.4 0.15 25); margin-bottom:8px; }}
  .msg {{ font-size:15px; color:oklch(0.55 0 0); margin-bottom:24px; }}
  a {{ color:oklch(0.7 0 0); text-decoration:underline; font-size:13px; }}
</style></head>
<body><div class="box">
  <div class="code">403</div>
  <div class="msg">{}</div>
  <a href="/">Back to dashboard</a>
</div></body></html>"##,
        message
    );
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Full::new(Bytes::from(html)))
        .unwrap()
}

// ──────────────────────────────────────────────
// Audit injection helpers
// ──────────────────────────────────────────────

/// If CRONUS_AUDIT_REF env var points to an HTML file, extract reference values
/// and inject the dump audit script into every page response.
fn inject_audit_if_enabled(html: String) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    let ref_path = match std::env::var("CRONUS_AUDIT_REF") {
        Ok(p) if !p.is_empty() => p,
        _ => return html,
    };
    let ref_html = match std::fs::read_to_string(&ref_path) {
        Ok(h) => h,
        Err(_) => return html,
    };

    // Extract all visible text values from reference HTML (simple extraction)
    let mut ref_numbers: Vec<f64> = Vec::new();
    let mut ref_strings: Vec<String> = Vec::new();

    // Extract only VISIBLE text -- skip <script{script_nonce}>, <style>, <head>, and Tailwind config
    let visible_text = extract_visible_text(&ref_html);
    // Extract numbers (including formatted: 1,482,900.00 / 12,842 / 4.82%)
    for cap in regex_numbers(&visible_text) {
        if let Ok(n) = cap.parse::<f64>() {
            if n.is_finite() && n >= 2.0 && n <= 1e9 {
                // Skip hex-color-like numbers (6-digit: 131313, 353535, etc.)
                let is_hex_color = n == n.floor() && n >= 100000.0 && n <= 999999.0;
                // Skip common CSS/config numbers
                let is_css_noise = [
                    24.0, 32.0, 36.0, 48.0, 64.0, 96.0, 128.0, 256.0, 512.0, 0.125, 0.25, 0.5,
                    0.75, 0.15, 0.2, 0.3, 0.05, 0.08, 0.1, 0.04, 0.06,
                ]
                .contains(&n);
                if !is_hex_color && !is_css_noise {
                    ref_numbers.push(n);
                }
            }
        }
    }
    // Extract meaningful strings (3-120 chars, not common UI words)
    for line in visible_text.lines() {
        let trimmed = line.trim();
        if trimmed.len() >= 3 && trimmed.len() <= 120 {
            let lower = trimmed.to_lowercase();
            // Skip CSS/Tailwind noise
            if !lower.contains("tailwind")
                && !lower.contains("font-variation")
                && !lower.contains("border-radius")
                && !lower.contains("rgba(")
                && !lower.contains("linear-gradient")
                && !lower.contains("backdrop-filter")
                && !lower.contains("clip-path")
                && !lower.contains("animation")
                && !lower.starts_with('.')
                && !lower.starts_with('#')
                && !lower.starts_with('{')
                && !lower.starts_with('@')
            {
                ref_strings.push(lower);
            }
        }
    }

    // Build audit script injection
    let numbers_json: Vec<String> = ref_numbers.iter().map(|n| format!("{}", n)).collect();
    let strings_json: Vec<String> = ref_strings
        .iter()
        .map(|s| format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")))
        .collect();

    // Also extract DSL strings from .cronus file for source-tracing
    let mut dsl_strings_json = String::from("[]");
    if let Some(cronus_file) = crate::find_cronus_file() {
        if let Ok(cronus_source) = std::fs::read_to_string(&cronus_file) {
            let mut dsl_strs: Vec<String> = Vec::new();
            for (i, part) in cronus_source.split('"').enumerate() {
                if i % 2 == 1 && part.len() > 1 && part.len() < 200 {
                    let escaped = part.replace('\\', "\\\\").replace('"', "\\\"");
                    dsl_strs.push(format!("\"{}\"", escaped));
                }
            }
            dsl_strings_json = format!("[{}]", dsl_strs.join(","));
        }
    }

    let audit_js = include_str!("cronus-dump-audit.js");
    // Escape </script> inside JS to prevent premature tag closing
    let safe_js = audit_js.replace("</script>", "<\\/script>");
    let nonce = crate::security::script_nonce_attr();
    let injection = format!(
        "<script{nonce}>window.__CRONUS_AUDIT_REFERENCE = {{ numbers: [{}], strings: [{}] }};\nwindow.__CRONUS_DSL_STRINGS__ = {};</script>\n<script{nonce}>{}</script>",
        numbers_json.join(","),
        strings_json.join(","),
        dsl_strings_json,
        safe_js,
    );

    // Inject before </body>
    if let Some(pos) = html.rfind("</body>") {
        let mut result = html[..pos].to_string();
        result.push_str(&injection);
        result.push_str(&html[pos..]);
        result
    } else {
        let mut result = html;
        result.push_str(&injection);
        result
    }
}

/// Extract only visible text from HTML body -- skip script, style, head, SVG, tailwind config
fn extract_visible_text(html: &str) -> String {
    let mut result = String::new();
    // Find <body> content
    let body_start = html
        .find("<body")
        .and_then(|pos| html[pos..].find('>').map(|p| pos + p + 1))
        .unwrap_or(0);
    let body_end = html.rfind("</body>").unwrap_or(html.len());
    let body = &html[body_start..body_end];

    let mut in_tag = false;
    let mut tag_name = String::new();
    let mut skip_depth = 0;
    let skip_tags = ["script", "style", "svg", "noscript", "link", "meta"];

    let chars: Vec<char> = body.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '<' {
            in_tag = true;
            tag_name.clear();
            // Check if closing tag
            let is_closing = i + 1 < chars.len() && chars[i + 1] == '/';
            let name_start = if is_closing { i + 2 } else { i + 1 };
            let mut j = name_start;
            while j < chars.len() && chars[j].is_alphanumeric() {
                tag_name.push(chars[j].to_ascii_lowercase());
                j += 1;
            }
            if is_closing && skip_tags.contains(&tag_name.as_str()) {
                skip_depth = (skip_depth - 1).max(0);
            } else if !is_closing && skip_tags.contains(&tag_name.as_str()) {
                skip_depth += 1;
            }
            // Skip to end of tag
            while i < chars.len() && chars[i] != '>' {
                i += 1;
            }
            i += 1;
            if skip_depth == 0 {
                result.push('\n');
            }
            continue;
        }
        if skip_depth == 0 {
            result.push(chars[i]);
        }
        i += 1;
    }
    result
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len() / 3);
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
            continue;
        }
        if ch == '>' {
            in_tag = false;
            // Check if we just closed a script/style
            let lower = result.to_lowercase();
            if lower.ends_with("/script") || lower.ends_with("/style") {
                in_script = false;
                in_style = false;
            }
            result.push(' ');
            continue;
        }
        if in_tag {
            // Detect script/style opening
            let partial = result.to_lowercase();
            if partial.ends_with("script") {
                in_script = true;
            }
            if partial.ends_with("style") {
                in_style = true;
            }
            continue;
        }
        if in_script || in_style {
            continue;
        }
        result.push(ch);
    }
    result
}

fn regex_numbers(text: &str) -> Vec<String> {
    let mut results = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;
    while i < len {
        if chars[i].is_ascii_digit()
            || (chars[i] == '-' && i + 1 < len && chars[i + 1].is_ascii_digit())
        {
            let start = i;
            while i < len
                && (chars[i].is_ascii_digit()
                    || chars[i] == '.'
                    || chars[i] == ','
                    || chars[i] == '-')
            {
                i += 1;
            }
            let raw: String = chars[start..i].iter().collect();
            // Try parsing as-is
            if let Ok(_) = raw.parse::<f64>() {
                results.push(raw.clone());
            }
            // Try removing thousand separators (1,482,900.00 -> 1482900.00)
            let cleaned = raw.replace(',', "");
            if cleaned != raw {
                if let Ok(_) = cleaned.parse::<f64>() {
                    results.push(cleaned);
                }
            }
            // Brazilian format (1.482.900 -> 1482900)
            if raw.matches('.').count() >= 2 {
                let br_cleaned = raw.replace('.', "");
                results.push(br_cleaned);
            }
        } else {
            i += 1;
        }
    }
    results
}
