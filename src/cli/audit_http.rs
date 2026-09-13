//! Exclusive HTTP path for `cronus run --audit-canvas`.
//!
//! `/audit/*` never falls through to `handle_request_inner` (which would serve
//! `page.config.source` as raw HTML). Other paths in this process are 404.

use crate::cronus_ui_widgets;
use crate::parser::{ComponentNode, PageNode};
use crate::ui::audit_layout::render_audit_document;
use bytes::Bytes;
use http_body_util::Full;
use hyper::{Response, StatusCode};
use std::collections::HashMap;

const PRESETS: &[&str] = &["aurora", "neutral", "midnight", "sunset", "emerald"];
const MODES: &[&str] = &["light", "dark"];
const DIRS: &[&str] = &["ltr", "rtl"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditHttpOutcome {
    pub status: u16,
    pub body: String,
    pub content_type: &'static str,
    pub audit_headers: bool,
}

impl AuditHttpOutcome {
    fn ok(body: String) -> Self {
        Self {
            status: 200,
            body,
            content_type: "text/html; charset=utf-8",
            audit_headers: true,
        }
    }

    fn err(status: u16, body: &str) -> Self {
        Self {
            status,
            body: body.to_string(),
            content_type: "text/plain; charset=utf-8",
            audit_headers: false,
        }
    }
}

pub fn handle_audit_request(
    path: &str,
    query: Option<&str>,
    pages: &[PageNode],
    components: &[ComponentNode],
) -> Response<Full<Bytes>> {
    outcome_to_response(audit_http_outcome(path, query, pages, components))
}

pub fn audit_not_found() -> Response<Full<Bytes>> {
    outcome_to_response(AuditHttpOutcome::err(404, "not found"))
}

pub fn audit_http_outcome(
    path: &str,
    query: Option<&str>,
    pages: &[PageNode],
    components: &[ComponentNode],
) -> AuditHttpOutcome {
    let path = normalize_path(path);
    let q = parse_query(query.unwrap_or(""));
    let preset = q.get("preset").map(String::as_str).unwrap_or("aurora");
    let mode = q.get("mode").map(String::as_str).unwrap_or("dark");
    let dir = q.get("dir").map(String::as_str).unwrap_or("ltr");
    if !PRESETS.contains(&preset) || !MODES.contains(&mode) || !DIRS.contains(&dir) {
        return AuditHttpOutcome::err(400, "bad request: invalid preset, mode, or dir");
    }

    let Some(page) = pages.iter().find(|p| p.route == path) else {
        return AuditHttpOutcome::err(404, "not found");
    };

    if refuses_page(page) {
        return AuditHttpOutcome::err(404, "not found");
    }

    if page.components.len() != 1 {
        return AuditHttpOutcome::err(400, "bad request: expected exactly one `use` component");
    }
    let name = &page.components[0];
    let Some(comp) = components.iter().find(|c| c.name == *name) else {
        return AuditHttpOutcome::err(404, "not found");
    };
    let Some(widget) = cronus_ui_widgets::render(comp) else {
        return AuditHttpOutcome::err(404, "not found");
    };

    let html = render_audit_document(&widget, preset, mode, dir);
    AuditHttpOutcome::ok(html)
}

fn refuses_page(page: &PageNode) -> bool {
    if page.config.contains_key("source") {
        return true;
    }
    if page.config.get("layout").map(String::as_str) == Some("light-app") {
        return true;
    }
    if page
        .sections
        .iter()
        .any(|s| s.template.is_some() || s.style_block.is_some())
    {
        return true;
    }
    let route = page.route.to_ascii_lowercase();
    let title = page.title.as_deref().unwrap_or("").to_ascii_lowercase();
    let forbidden_route = route.contains("/login")
        || route.ends_with("/login")
        || route.contains("/signup")
        || route.contains("/settings")
        || route.contains("order-detail")
        || route.contains("/orders/") && route.contains("/detail");
    let forbidden_title = title.contains("sign in")
        || title.contains("sign up")
        || title.contains("login")
        || title.contains("signup");
    forbidden_route || forbidden_title
}

fn normalize_path(path: &str) -> String {
    if path.len() > 1 && path.ends_with('/') {
        path.trim_end_matches('/').to_string()
    } else {
        path.to_string()
    }
}

fn parse_query(query: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    if query.is_empty() {
        return out;
    }
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut parts = pair.splitn(2, '=');
        let k = parts.next().unwrap_or("");
        let v = parts.next().unwrap_or("");
        out.insert(k.to_string(), v.to_string());
    }
    out
}

fn outcome_to_response(out: AuditHttpOutcome) -> Response<Full<Bytes>> {
    let mut builder = Response::builder()
        .status(StatusCode::from_u16(out.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR))
        .header("Content-Type", out.content_type);
    if out.audit_headers {
        builder = builder
            .header("X-Cronus-Engine", "cronus-lang/0.1.0")
            .header("X-Cronus-Audit", "1");
    }
    builder
        .body(Full::new(Bytes::from(out.body)))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{self, AstNode};

    fn parse_app(src: &str) -> (Vec<PageNode>, Vec<ComponentNode>) {
        let nodes = parser::parse(src).expect("parse");
        let pages = nodes
            .iter()
            .filter_map(|n| match n {
                AstNode::Page(p) => Some(p.clone()),
                _ => None,
            })
            .collect();
        let components = nodes
            .iter()
            .filter_map(|n| match n {
                AstNode::Component(c) => Some(c.clone()),
                _ => None,
            })
            .collect();
        (pages, components)
    }

    fn clean_src() -> &'static str {
        r#"
app "audit-fixtures" {
  port 5176
}

style {
  preset aurora
  theme dark
}

component ButtonPrimaryMd layout:inline style:button+primary+md {
  label "Save profile"
}

page "/audit/button/primary-md" type:custom {
  use ButtonPrimaryMd
}
"#
    }

    #[test]
    fn happy_path_injects_canvas_and_audit_headers() {
        let (pages, components) = parse_app(clean_src());
        let out = audit_http_outcome(
            "/audit/button/primary-md",
            Some("preset=aurora&mode=light"),
            &pages,
            &components,
        );
        assert_eq!(out.status, 200);
        assert!(out.audit_headers);
        assert!(out.body.contains("data-audit-canvas"));
        assert!(out.body.contains("data-cronus-mode=\"light\""));
        assert!(out.body.contains("data-slot=\"button\""));
        assert!(out.body.contains("Save profile"));
        assert!(!out.body.contains("<script"));
        assert!(!out.body.contains("preview-frame"));
        let resp = outcome_to_response(out);
        assert_eq!(
            resp.headers()
                .get("X-Cronus-Audit")
                .and_then(|v| v.to_str().ok()),
            Some("1")
        );
        assert_eq!(
            resp.headers()
                .get("X-Cronus-Engine")
                .and_then(|v| v.to_str().ok()),
            Some("cronus-lang/0.1.0")
        );
    }

    #[test]
    fn audit_http_ignores_page_source() {
        let stolen = "STOLEN_HTML_MARKER_SHOULD_NOT_APPEAR";
        let dir = std::env::temp_dir();
        let stolen_path = dir.join("cronus-audit-stolen.html");
        std::fs::write(&stolen_path, stolen).unwrap();
        let src = format!(
            r#"
app "audit-fixtures" {{ port 5176 }}
component ButtonPrimaryMd layout:inline style:button+primary+md {{
  label "Save profile"
}}
page "/audit/button/primary-md" type:custom {{
  source:"./stolen.html"
  use ButtonPrimaryMd
}}
"#
        );
        let (pages, components) = parse_app(&src);
        assert_eq!(
            pages[0].config.get("source").map(String::as_str),
            Some("./stolen.html")
        );
        let out = audit_http_outcome("/audit/button/primary-md", None, &pages, &components);
        assert_eq!(out.status, 404);
        assert!(!out.audit_headers);
        assert!(!out.body.contains(stolen));
        assert!(!out.body.contains("STOLEN_HTML_MARKER"));
        let _ = std::fs::remove_file(stolen_path);
    }

    #[test]
    fn invalid_query_is_400_without_audit_header() {
        let (pages, components) = parse_app(clean_src());
        let out = audit_http_outcome(
            "/audit/button/primary-md",
            Some("preset=not-a-theme"),
            &pages,
            &components,
        );
        assert_eq!(out.status, 400);
        assert!(!out.audit_headers);
    }

    #[test]
    fn missing_page_is_404_without_audit_header() {
        let (pages, components) = parse_app(clean_src());
        let out = audit_http_outcome("/audit/missing", None, &pages, &components);
        assert_eq!(out.status, 404);
        assert!(!out.audit_headers);
    }

    #[test]
    fn dark_and_light_differ() {
        let (pages, components) = parse_app(clean_src());
        let light = audit_http_outcome(
            "/audit/button/primary-md",
            Some("mode=light"),
            &pages,
            &components,
        );
        let dark = audit_http_outcome(
            "/audit/button/primary-md",
            Some("mode=dark"),
            &pages,
            &components,
        );
        assert!(light.body.contains("data-cronus-mode=\"light\""));
        assert!(dark.body.contains("data-cronus-mode=\"dark\""));
        assert_ne!(light.body, dark.body);
    }
}
