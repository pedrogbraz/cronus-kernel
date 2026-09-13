//! Dedicated StatusDot renderer. DOM matches React:
//! `<span data-slot="status-dot" role="status" data-status>` plus
//! `status-dot-indicator` and `status-dot-label` from the label.
//! Not interact `pill("status-dot")` (BASE/SURF padding pill span).

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let status = status_of(comp);
    let label = label_of(comp);
    format!(
        "<span data-slot=\"status-dot\" role=\"status\" data-status=\"{status}\"><span data-slot=\"status-dot-indicator\"></span><span data-slot=\"status-dot-label\">{label}</span></span>"
    )
}

fn status_of(comp: &ComponentNode) -> &'static str {
    if let Some(v) = comp.props.get("status") {
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
    fn root_is_status_dot_with_indicator_and_label() {
        let html = render(&stub("status-dot", "Online"));
        assert!(html.starts_with(
            "<span data-slot=\"status-dot\" role=\"status\" data-status=\"online\">"
        ));
        assert!(html.contains("<span data-slot=\"status-dot-indicator\"></span>"));
        assert!(html.contains("<span data-slot=\"status-dot-label\">Online</span>"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<span data-slot=\"status-dot\" role=\"status\" data-status=\"online\"><span data-slot=\"status-dot-indicator\"></span><span data-slot=\"status-dot-label\">Online</span></span>"
        );
    }

    #[test]
    fn status_from_props() {
        let mut c = stub("status-dot", "Busy");
        c.props.insert("status".into(), "busy".into());
        let html = render(&c);
        assert!(html.contains("data-status=\"busy\""));
        assert!(html.contains(">Busy</span>"));
        reject_interact(&html);
    }

    #[test]
    fn status_from_style() {
        let mut c = stub("status-dot", "Offline");
        c.style = Some("status-dot+offline".into());
        let html = render(&c);
        assert!(html.contains("data-status=\"offline\""));
        assert!(html.contains("<span data-slot=\"status-dot-label\">Offline</span>"));
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
        assert!(!interact.contains("data-slot=\"status-dot-label\""));
        assert!(!interact.contains("role=\"status\""));
        assert!(html.contains("data-slot=\"status-dot-indicator\""));
        assert!(html.contains("data-slot=\"status-dot-label\""));
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
        assert!(css.contains("display: inline-flex"));
        assert!(css.contains("border-radius: 9999px"));
        assert!(css.contains("var(--cronus-success)"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
