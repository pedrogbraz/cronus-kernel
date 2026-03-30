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
    // Simple ISO-ish timestamp
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
