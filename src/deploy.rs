#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS Deploy Engine
//!
//! Generates deployment artifacts: Dockerfile, docker-compose, .dockerignore

pub fn generate_dockerfile(app_name: &str, port: u16) -> String {
    format!(r#"# {app_name} — CRONUS Runtime
# Built with: cronus deploy

FROM rust:1.82-slim AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
COPY src/ src/
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/cronus /usr/local/bin/cronus
COPY *.cronus /app/
WORKDIR /app
EXPOSE {port}
ENV CRONUS_PORT={port}
CMD ["cronus", "run", "{port}"]
"#,
        app_name = app_name,
        port = port,
    )
}

pub fn generate_compose(app_name: &str, port: u16) -> String {
    let slug = app_name.to_lowercase().replace(' ', "-");
    format!(r#"# {app_name} — docker-compose
# Start with: docker compose up

services:
  app:
    build: .
    container_name: {slug}
    ports:
      - "{port}:{port}"
    volumes:
      - ./data:/app/data
    restart: unless-stopped
    environment:
      - CRONUS_PORT={port}
"#,
        app_name = app_name,
        slug = slug,
        port = port,
    )
}

pub fn generate_dockerignore() -> &'static str {
    r#"target/
data.db
*.db-journal
*.db-wal
.git/
.gitignore
node_modules/
.cronus-build/
"#
}

pub fn generate_ir(nodes: &[crate::parser::AstNode]) -> serde_json::Value {
    use crate::parser::AstNode;
    use serde_json::json;

    let mut app = json!(null);
    let mut entities = vec![];
    let mut pages = vec![];
    let mut apis = vec![];
    let mut services = vec![];
    let mut style = json!(null);

    for node in nodes {
        match node {
            AstNode::App(a) => {
                app = json!({
                    "name": a.name,
                    "port": a.port,
                    "stack": a.stack,
                    "database": a.database.as_ref().map(|d| json!({
                        "type": d.db_type,
                        "path": d.path,
                    })),
                });
            }
            AstNode::Entity(e) => {
                let fields: Vec<serde_json::Value> = e.fields.iter().map(|f| {
                    json!({
                        "name": f.name,
                        "type": format!("{:?}", f.field_type).to_lowercase(),
                        "required": f.required,
                        "unique": f.unique,
                        "enum_values": f.enum_values,
                    })
                }).collect();
                entities.push(json!({
                    "name": e.name,
                    "fields": fields,
                }));
            }
            AstNode::Page(p) => {
                let sections: Vec<serde_json::Value> = p.sections.iter().map(|s| {
                    json!({
                        "type": s.section_type,
                        "title": s.title,
                        "subtitle": s.subtitle,
                        "config": s.config,
                    })
                }).collect();
                pages.push(json!({
                    "route": p.route,
                    "type": p.page_type,
                    "entity": p.entity,
                    "title": p.title,
                    "sections": sections,
                }));
            }
            AstNode::Api(a) => {
                let routes: Vec<serde_json::Value> = a.routes.iter().map(|r| {
                    json!({
                        "name": r.name,
                        "method": format!("{:?}", r.method),
                        "path": format!("{}{}", a.prefix, r.path),
                        "auth": r.auth,
                    })
                }).collect();
                apis.extend(routes);
            }
            AstNode::Style(s) => {
                style = json!({
                    "theme": s.theme,
                    "accent": s.accent,
                    "radius": s.radius,
                    "font": s.font,
                });
            }
            AstNode::Service(s) => {
                services.push(json!({
                    "name": s.name,
                    "port": s.port,
                    "config": s.config,
                }));
            }
            _ => {}
        }
    }

    json!({
        "version": "0.2",
        "type": "abstract",
        "app": app,
        "entities": entities,
        "ui": pages,
        "api": apis,
        "services": services,
        "design": style,
        "meta": {
            "generatedAt": generate_timestamp(),
            "generator": "cronus-kernel v0.1.0",
        }
    })
}

fn generate_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    let y = 1970 + (days / 365);
    let rem = days % 365;
    let mo = rem / 30 + 1;
    let d = rem % 30 + 1;
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, d, h, m, s)
}

fn slugify(name: &str) -> String {
    name.to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect()
}

// ── Deploy Target Generators ──

pub fn generate_fly_toml(app_name: &str, port: u16) -> String {
    let slug = slugify(app_name);
    format!(r#"app = "{slug}"
primary_region = "gru"

[build]

[http_service]
  internal_port = {port}
  force_https = true
  auto_stop_machines = true
  auto_start_machines = true
  min_machines_running = 0

[env]
  CRONUS_PORT = "{port}"

[[vm]]
  size = "shared-cpu-1x"
  memory = "256mb"
"#)
}

pub fn generate_railway_config(app_name: &str, port: u16) -> String {
    let _slug = slugify(app_name);
    format!(r#"{{
  "$schema": "https://railway.com/railway.schema.json",
  "build": {{ "builder": "DOCKERFILE" }},
  "deploy": {{
    "startCommand": "cronus run {port}",
    "healthcheckPath": "/api/health",
    "restartPolicyType": "ON_FAILURE"
  }}
}}"#)
}

// ── Static Build ──

pub fn generate_static_build(pages_html: &[(String, String)]) -> Vec<(String, String)> {
    let mut files: Vec<(String, String)> = Vec::new();
    let mut all_css = String::new();
    let mut all_js = String::new();

    for (route, html) in pages_html {
        // Extract route path → file path
        let path = if route == "/" || route.is_empty() {
            "dist/index.html".to_string()
        } else {
            let clean = route.trim_start_matches('/').trim_end_matches('/');
            format!("dist/{clean}/index.html")
        };

        // Extract <style> blocks into shared CSS
        let mut page_html = html.clone();
        let mut search_from = 0;
        while let Some(start) = page_html[search_from..].find("<style>") {
            let abs_start = search_from + start;
            if let Some(end) = page_html[abs_start..].find("</style>") {
                let abs_end = abs_start + end + "</style>".len();
                let style_block = &page_html[abs_start + "<style>".len()..abs_start + end];
                all_css.push_str(style_block);
                all_css.push('\n');
                page_html = format!("{}{}", &page_html[..abs_start], &page_html[abs_end..]);
            } else {
                search_from = abs_start + 1;
            }
        }

        // Extract <script> blocks into shared JS
        search_from = 0;
        while let Some(start) = page_html[search_from..].find("<script>") {
            let abs_start = search_from + start;
            if let Some(end) = page_html[abs_start..].find("</script>") {
                let abs_end = abs_start + end + "</script>".len();
                let script_block = &page_html[abs_start + "<script>".len()..abs_start + end];
                all_js.push_str(script_block);
                all_js.push('\n');
                page_html = format!("{}{}", &page_html[..abs_start], &page_html[abs_end..]);
            } else {
                search_from = abs_start + 1;
            }
        }

        // Add CSS/JS links to <head>
        let final_html = if page_html.contains("</head>") {
            page_html.replacen(
                "</head>",
                r#"<link rel="stylesheet" href="/assets/cronus.css"><script src="/assets/cronus.js" defer></script></head>"#,
                1,
            )
        } else {
            format!(
                r#"<!DOCTYPE html><html><head><link rel="stylesheet" href="/assets/cronus.css"><script src="/assets/cronus.js" defer></script></head><body>{page_html}</body></html>"#
            )
        };

        files.push((path, final_html));
    }

    files.push(("dist/assets/cronus.css".to_string(), all_css));
    files.push(("dist/assets/cronus.js".to_string(), all_js));

    files
}

// ── Health Endpoint ──

pub fn generate_health_endpoint(entity_count: usize, page_count: usize) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    format!(r#"{{"status":"ok","version":"0.1.0","uptime":{uptime},"entities":{entity_count},"pages":{page_count}}}"#)
}

// ── Production Headers ──

pub fn generate_production_headers() -> Vec<(String, String)> {
    vec![
        ("X-Content-Type-Options".into(), "nosniff".into()),
        ("X-Frame-Options".into(), "DENY".into()),
        ("X-XSS-Protection".into(), "1; mode=block".into()),
        ("Strict-Transport-Security".into(), "max-age=31536000".into()),
        ("Content-Security-Policy".into(), "default-src 'self' 'unsafe-inline' 'unsafe-eval'".into()),
        ("Cache-Control".into(), "public, max-age=3600".into()),
    ]
}

// ── HTML Minification ──

pub fn minify_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut i = 0;
    let bytes = html.as_bytes();
    let len = bytes.len();

    while i < len {
        // Preserve <pre>, <script>, <style> content verbatim
        if i + 5 < len {
            let tag_start = &html[i..];
            for preserve_tag in &["<pre", "<script", "<style"] {
                if tag_start.starts_with(preserve_tag) {
                    let close_tag = format!("</{}>", &preserve_tag[1..]);
                    if let Some(end) = html[i..].find(&close_tag) {
                        let block_end = i + end + close_tag.len();
                        result.push_str(&html[i..block_end]);
                        i = block_end;
                        continue;
                    }
                }
            }
        }

        // Remove HTML comments <!-- ... -->
        if i + 4 < len && &html[i..i + 4] == "<!--" {
            if let Some(end) = html[i..].find("-->") {
                i += end + 3;
                continue;
            }
        }

        // Collapse multiple whitespace into single space
        if bytes[i].is_ascii_whitespace() {
            // Skip all contiguous whitespace
            while i < len && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            // Only add space if we're not at start and not between > and <
            if !result.is_empty() && !result.ends_with('>') && i < len && bytes[i] != b'<' {
                result.push(' ');
            }
            continue;
        }

        result.push(bytes[i] as char);
        i += 1;
    }

    result
}

// ── Gzip Middleware Reference ──

pub fn generate_gzip_middleware_code() -> String {
    r#"// CRONUS Gzip Middleware — add to server response pipeline
// Requires: flate2 = "1.0" in Cargo.toml

use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

fn gzip_response(body: &[u8], accept_encoding: &str) -> (Vec<u8>, Vec<(String, String)>) {
    if accept_encoding.contains("gzip") && body.len() > 1024 {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(body).unwrap_or_default();
        let compressed = encoder.finish().unwrap_or_default();
        if compressed.len() < body.len() {
            return (compressed, vec![
                ("Content-Encoding".into(), "gzip".into()),
                ("Vary".into(), "Accept-Encoding".into()),
            ]);
        }
    }
    (body.to_vec(), vec![])
}
"#.to_string()
}
