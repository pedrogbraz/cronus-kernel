//! Voodoo.js as an **opt-in HTML runtime**. Never the authoring language.
//!
//! `.cronus` stays `.cronus`. The kernel emits HTML at `cronus run`. When the
//! author opts in, that HTML may carry `v-data` / `v-model` / `@click` and a
//! pinned Voodoo script tag. Without the opt-in those attributes are omitted
//! (plain `{ count }` would otherwise show up as literal text) and no script
//! is injected — legacy demos keep working.
//!
//! Opt-in (either is enough):
//! - `app "Name" { stack voodoo }`
//! - `style { runtime voodoo }`
//!
//! Uses tokio task-local storage so an `.await` cannot leak the flag onto
//! another request, and so a process-wide AtomicBool cannot flip every page.

use crate::parser::StyleNode;
use std::future::Future;

/// Pinned Voodoo.js CDN (full build). Injected into **emitted HTML** only.
/// `.cronus` authoring stays `.cronus` — JSX is not a source language here.
pub const SCRIPT_SRC: &str = "https://cdn.jsdelivr.net/npm/voodoojs@0.13.0/dist/voodoo.full.min.js";

/// Subresource Integrity for [`SCRIPT_SRC`]. Computed 2026-09-14 as
/// `openssl dgst -sha384 -binary | openssl base64 -A` over the jsDelivr file,
/// and cross-checked byte-identical against `dist/voodoo.full.min.js` inside
/// the npm tarball `voodoojs-0.13.0.tgz`. Bumping the version REQUIRES
/// recomputing this; a mismatch makes browsers refuse to run the script.
pub const VOODOO_SRI: &str = "sha384-T8aMcXnhRtYq6zSQrZOBZrG4JR0nH0lULV6xS1ahoJgI5KEVijdDFczre4phw8Bn";

tokio::task_local! {
    static ENABLED: bool;
}

pub fn wanted(stack: &[String], style: Option<&StyleNode>) -> bool {
    stack.iter().any(|s| s.eq_ignore_ascii_case("voodoo"))
        || style
            .and_then(|s| s.config.get("runtime"))
            .map(|r| r.eq_ignore_ascii_case("voodoo"))
            .unwrap_or(false)
}

pub fn enabled() -> bool {
    ENABLED.try_with(|v| *v).unwrap_or(false)
}

/// Sync scope for tests and for widget renderers called outside HTTP.
pub fn with_enabled<R>(on: bool, f: impl FnOnce() -> R) -> R {
    ENABLED.sync_scope(on, f)
}

/// Async scope wrapping a request. Survives `.await` on the same task.
pub fn scope<F: Future>(on: bool, fut: F) -> impl Future<Output = F::Output> {
    ENABLED.scope(on, fut)
}

pub fn script_tag() -> String {
    let nonce = crate::security::script_nonce_attr();
    format!(
        r#"<script{nonce} src="{SCRIPT_SRC}" integrity="{VOODOO_SRI}" crossorigin="anonymous" data-cronus-runtime="voodoo" defer></script>"#
    )
}

/// Insert the Voodoo script when the request opted in. Idempotent.
pub fn inject_into_html(html: String) -> String {
    if !enabled() {
        return html;
    }
    if html.contains("data-cronus-runtime=\"voodoo\"") || html.contains("voodoojs@") {
        return html;
    }
    let tag = script_tag();
    if let Some(pos) = html.find("</head>") {
        let mut out = String::with_capacity(html.len() + tag.len() + 1);
        out.push_str(&html[..pos]);
        out.push_str(&tag);
        out.push('\n');
        out.push_str(&html[pos..]);
        out
    } else if let Some(pos) = html.rfind("</body>") {
        let mut out = String::with_capacity(html.len() + tag.len() + 1);
        out.push_str(&html[..pos]);
        out.push_str(&tag);
        out.push('\n');
        out.push_str(&html[pos..]);
        out
    } else {
        let mut out = html;
        out.push_str(&tag);
        out
    }
}

pub fn data(json: &str) -> String {
    if enabled() {
        format!(" v-data=\"{}\"", esc_attr(json))
    } else {
        String::new()
    }
}

pub fn model(name: &str) -> String {
    if enabled() {
        format!(" v-model=\"{}\"", esc_attr(name))
    } else {
        String::new()
    }
}

pub fn click(expr: &str) -> String {
    if enabled() {
        format!(" @click=\"{}\"", esc_attr(expr))
    } else {
        String::new()
    }
}

pub fn show(expr: &str) -> String {
    if enabled() {
        format!(" v-show=\"{}\"", esc_attr(expr))
    } else {
        String::new()
    }
}

pub fn bind(attr: &str, expr: &str) -> String {
    if enabled() {
        format!(" :{attr}=\"{}\"", esc_attr(expr))
    } else {
        String::new()
    }
}

/// `{ expr }` only when Voodoo is on. Otherwise the static fallback.
pub fn interp(expr: &str, fallback: &str) -> String {
    if enabled() {
        format!("{{ {expr} }}")
    } else {
        fallback.to_string()
    }
}

fn esc_attr(s: &str) -> String {
    s.replace('&', "&amp;").replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::StyleNode;
    use std::collections::HashMap;

    #[test]
    fn stack_voodoo_is_wanted() {
        assert!(wanted(&["voodoo".into()], None));
        assert!(wanted(&["react".into(), "Voodoo".into()], None));
        assert!(!wanted(&["react".into(), "tailwind".into()], None));
        assert!(!wanted(&[], None));
    }

    #[test]
    fn style_runtime_voodoo_is_wanted() {
        let mut config = HashMap::new();
        config.insert("runtime".into(), "voodoo".into());
        let style = StyleNode {
            theme: None,
            accent: None,
            radius: None,
            font: None,
            config,
        };
        assert!(wanted(&[], Some(&style)));
        let mut other = HashMap::new();
        other.insert("runtime".into(), "none".into());
        let style = StyleNode {
            theme: None,
            accent: None,
            radius: None,
            font: None,
            config: other,
        };
        assert!(!wanted(&[], Some(&style)));
    }

    #[test]
    fn default_enabled_is_false() {
        assert!(!enabled());
        assert!(data("{ n: 0 }").is_empty());
        assert_eq!(interp("n", "0"), "0");
    }

    #[test]
    fn attrs_and_interp_only_when_on() {
        with_enabled(true, || {
            assert!(enabled());
            assert!(data("{ n: 0 }").contains("v-data="));
            assert!(model("n").contains("v-model="));
            assert!(click("n++").contains("@click="));
            assert_eq!(interp("n", "0"), "{ n }");
        });
        assert!(!enabled());
    }

    #[test]
    fn script_tag_pins_sri_and_crossorigin() {
        assert!(VOODOO_SRI.starts_with("sha384-"), "VOODOO_SRI must be a sha384 SRI hash");
        // 48-byte digest -> 64 base64 chars
        assert_eq!(VOODOO_SRI.len(), "sha384-".len() + 64);
        let tag = script_tag();
        assert!(tag.contains(&format!(r#"integrity="{VOODOO_SRI}""#)));
        assert!(tag.contains(r#"crossorigin="anonymous""#));
    }

    #[test]
    fn inject_skips_when_off() {
        let html = "<html><head></head><body>x</body></html>".to_string();
        assert_eq!(inject_into_html(html.clone()), html);
        assert!(!html.contains("voodoojs"));
    }

    #[test]
    fn inject_when_on_is_idempotent() {
        with_enabled(true, || {
            let once = inject_into_html("<html><head></head><body>x</body></html>".into());
            assert!(once.contains(SCRIPT_SRC));
            assert!(once.contains("data-cronus-runtime=\"voodoo\""));
            assert!(once.contains("defer"));
            let twice = inject_into_html(once.clone());
            assert_eq!(twice.matches("voodoojs@").count(), 1);
        });
    }

    #[test]
    fn parser_stack_and_style_runtime() {
        let src = r#"
app "X" { stack voodoo port 5298 }
style { theme dark runtime voodoo preset aurora }
"#;
        let nodes = crate::parser::parse(src).expect("parse");
        let app = nodes.iter().find_map(|n| match n {
            crate::parser::AstNode::App(a) => Some(a),
            _ => None,
        }).expect("app");
        let style = nodes.iter().find_map(|n| match n {
            crate::parser::AstNode::Style(s) => Some(s),
            _ => None,
        }).expect("style");
        assert!(app.stack.iter().any(|s| s == "voodoo"));
        assert_eq!(style.config.get("runtime").map(String::as_str), Some("voodoo"));
        assert!(wanted(&app.stack, Some(style)));
    }
}
