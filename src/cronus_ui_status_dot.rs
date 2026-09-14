//! Dedicated StatusDot renderer. DOM matches React:
//! `<span data-slot="status-dot" data-status role="status">` plus an
//! `aria-hidden` `status-dot-indicator` and, by default (no `withLabel`), a
//! visually-hidden `status-dot-sr-label` carrying the status name ("Online").
//! `withLabel:true` renders the visible `status-dot-label` from the label.
//! The emitter's `label` is the fixture id when no label prop exists, so it is
//! never shown unless `withLabel` is set (same as React, which ignores it).
//! Not interact `pill("status-dot")` (BASE/SURF padding pill span).

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let status = status_of(comp);
    let indicator = "<span aria-hidden=\"true\" data-slot=\"status-dot-indicator\"></span>";
    let tail = if with_label(comp) {
        format!(
            "<span data-slot=\"status-dot-label\">{}</span>",
            label_of(comp)
        )
    } else {
        let name = attr(comp, "label")
            .map(esc)
            .unwrap_or_else(|| default_label(status).to_string());
        format!("<span data-slot=\"status-dot-sr-label\">{name}</span>")
    };
    format!(
        "<span data-slot=\"status-dot\" data-status=\"{status}\" role=\"status\">{indicator}{tail}</span>"
    )
}

/// Attribute from props or any item's config (the tokenizer attaches
/// `key:value` lines to the preceding item).
fn attr<'a>(comp: &'a ComponentNode, key: &str) -> Option<&'a str> {
    comp.props
        .get(key)
        .map(String::as_str)
        .or_else(|| {
            comp.items
                .iter()
                .find_map(|i| i.config.get(key).map(String::as_str))
        })
        .filter(|s| !s.is_empty())
}

fn with_label(comp: &ComponentNode) -> bool {
    matches!(
        attr(comp, "withLabel").or_else(|| attr(comp, "with-label")),
        Some("true")
    )
}

fn default_label(status: &str) -> &'static str {
    match status {
        "offline" => "Offline",
        "busy" => "Busy",
        "away" => "Away",
        "success" => "Success",
        "warning" => "Warning",
        "error" => "Error",
        "info" => "Info",
        "neutral" => "Neutral",
        _ => "Online",
    }
}

fn status_of(comp: &ComponentNode) -> &'static str {
    if let Some(v) = attr(comp, "status") {
        if let Some(named) = named_status(v) {
            return named;
        }
    }
    let style = comp.style.as_deref().unwrap_or("");
    for part in style.split('+') {
        let part = part.trim();
        if part == "status-dot" {
            continue;
        }
        if let Some(named) = named_status(part) {
            return named;
        }
    }
    "online"
}

fn named_status(raw: &str) -> Option<&'static str> {
    match raw {
        "online" => Some("online"),
        "offline" => Some("offline"),
        "busy" => Some("busy"),
        "away" => Some("away"),
        "success" => Some("success"),
        "warning" => Some("warning"),
        "error" => Some("error"),
        "info" => Some("info"),
        "neutral" => Some("neutral"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn reject_interact(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("padding:0.15rem 0.55rem"));
        assert!(!html.contains("display:inline-flex;align-items:center;gap:0.35rem"));
        assert!(!html.contains("font-size:0.75rem;font-weight:500"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("pill("));
    }

    #[test]
    fn default_is_indicator_plus_sr_status_name_not_fixture_label() {
        // Wave 1t: the emitter writes `label "default"` (fixture id); React
        // renders the sr-only status name "Online" and no visible label.
        let html = render(&stub("status-dot", "default"));
        assert_eq!(
            html,
            "<span data-slot=\"status-dot\" data-status=\"online\" role=\"status\"><span aria-hidden=\"true\" data-slot=\"status-dot-indicator\"></span><span data-slot=\"status-dot-sr-label\">Online</span></span>"
        );
        assert!(!html.contains("default"));
        assert!(!html.contains("data-slot=\"status-dot-label\""));
        reject_interact(&html);
    }

    #[test]
    fn with_label_renders_visible_label() {
        let mut c = stub("status-dot", "Online");
        c.props.insert("withLabel".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("<span data-slot=\"status-dot-label\">Online</span>"));
        assert!(!html.contains("status-dot-sr-label"));
        reject_interact(&html);
    }

    #[test]
    fn status_from_props() {
        let mut c = stub("status-dot", "Busy");
        c.props.insert("status".into(), "busy".into());
        let html = render(&c);
        assert!(html.contains("data-status=\"busy\""));
        assert!(html.contains("<span data-slot=\"status-dot-sr-label\">Busy</span>"));
        reject_interact(&html);
    }

    #[test]
    fn status_from_style() {
        let mut c = stub("status-dot", "x");
        c.style = Some("status-dot+offline".into());
        let html = render(&c);
        assert!(html.contains("data-status=\"offline\""));
        assert!(html.contains("<span data-slot=\"status-dot-sr-label\">Offline</span>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_pill() {
        let c = stub("status-dot", "Online");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("status-dot", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<span data-slot=\"status-dot\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("padding:0.15rem 0.55rem"));
        assert!(!interact.contains("data-slot=\"status-dot-indicator\""));
        assert!(!interact.contains("role=\"status\""));
        assert!(html.contains("data-slot=\"status-dot-indicator\""));
        assert!(html.contains("data-slot=\"status-dot-sr-label\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("status-dot", "Online"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"status-dot\"]"));
        assert!(css.contains("[data-slot=\"status-dot-indicator\"]"));
        assert!(css.contains("[data-slot=\"status-dot-label\"]"));
        assert!(css.contains("[data-slot=\"status-dot-sr-label\"] {\n  position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;"));
        assert!(css.contains("display: inline-flex"));
        assert!(css.contains("border-radius: 9999px"));
        assert!(css.contains("var(--cronus-success)"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
