//! Dedicated StatusDot renderer. DOM matches React:
//! `<span data-slot="status-dot" data-status role="status">` plus an
//! `aria-hidden` `status-dot-indicator` and, by default (no `withLabel`), a
//! visually-hidden `status-dot-sr-label` carrying the status name ("Online").
//! `with-label:true` renders the visible `status-dot-label` from the label.
//! The emitter's `label` is the fixture id when no label prop exists, so it is
//! never shown unless `withLabel` is set (same as React, which ignores it).
//!
//! React's other variants have no data attributes, so they travel as classes:
//! `size` (`xs` | `sm` | `md` | `lg`, style segment or `size:`) → `s-*` on the
//! root (gap, dot and label sizes), `ring:true` → `ring` on the indicator,
//! `position:top-right|bottom-right` → `pos-*` on the root. `pulse:true` adds
//! React's `status-dot-ping` halo inside the indicator (`animate-ping`, static
//! under reduced motion). `aria-label:` names the region. `avatar:"CN"` (with
//! `src:` / `alt:`) wraps an Avatar and the dot in the docs' `relative
//! inline-flex` span for the badge-on-avatar composition.
//! Not interact `pill("status-dot")` (BASE/SURF padding pill span).

use crate::cronus_ui_kit::{attr_nonempty, choice, esc, flag, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let status = status_of(comp);
    let ping = if flag(comp, "pulse") {
        "<span data-slot=\"status-dot-ping\"></span>"
    } else {
        ""
    };
    let ring = if flag(comp, "ring") {
        " class=\"ring\""
    } else {
        ""
    };
    let indicator = format!(
        "<span aria-hidden=\"true\" data-slot=\"status-dot-indicator\"{ring}>{ping}</span>"
    );
    let tail = if with_label(comp) {
        format!(
            "<span data-slot=\"status-dot-label\">{}</span>",
            label_of(comp)
        )
    } else {
        let name = attr_nonempty(comp, "label")
            .map(esc)
            .unwrap_or_else(|| default_label(status).to_string());
        format!("<span data-slot=\"status-dot-sr-label\">{name}</span>")
    };
    let mut classes: Vec<String> = Vec::new();
    if let Some(size) = choice(comp, "size", &["xs", "sm", "lg"]) {
        classes.push(format!("s-{size}"));
    }
    if let Some(pos) = choice(comp, "position", &["top-right", "bottom-right"]) {
        classes.push(format!("pos-{pos}"));
    }
    let class = if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    };
    let aria = attr_nonempty(comp, "aria-label")
        .map(|a| format!(" aria-label=\"{}\"", esc(a)))
        .unwrap_or_default();
    let dot = format!(
        "<span data-slot=\"status-dot\"{class} data-status=\"{status}\" role=\"status\"{aria}>{indicator}{tail}</span>"
    );
    match attr_nonempty(comp, "avatar") {
        Some(fallback) => {
            let alt = attr_nonempty(comp, "alt").unwrap_or(fallback);
            let avatar = crate::cronus_ui_avatar::avatar_html(
                &esc(fallback),
                attr_nonempty(comp, "src"),
                &esc(alt),
                "",
                "",
            );
            format!("<span class=\"cui-status-dot-anchor\">{avatar}{dot}</span>")
        }
        None => dot,
    }
}

fn with_label(comp: &ComponentNode) -> bool {
    attr_nonempty(comp, "withLabel")
        .or_else(|| attr_nonempty(comp, "with-label"))
        .is_some_and(crate::cronus_ui_kit::truthy)
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
    if let Some(v) = attr_nonempty(comp, "status") {
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
        c.props.remove("withLabel");
        c.props.insert("with-label".into(), "true".into());
        assert!(render(&c).contains("<span data-slot=\"status-dot-label\">Online</span>"));
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

    /// Docs "Pulse & sizes": `pulse:true` adds the ping halo; the size
    /// segment becomes a root class (React has no data-size).
    #[test]
    fn pulse_and_size_classes() {
        let mut c = stub("status-dot", "Live");
        c.style = Some("status-dot+error".into());
        c.props.insert("pulse".into(), "true".into());
        c.props.insert("with-label".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("<span aria-hidden=\"true\" data-slot=\"status-dot-indicator\"><span data-slot=\"status-dot-ping\"></span></span><span data-slot=\"status-dot-label\">Live</span>"));
        let mut s = stub("status-dot", "Online");
        s.style = Some("status-dot+xs".into());
        s.props.insert("aria-label".into(), "Online (xs)".into());
        assert_eq!(
            render(&s),
            "<span data-slot=\"status-dot\" class=\"s-xs\" data-status=\"online\" role=\"status\" aria-label=\"Online (xs)\"><span aria-hidden=\"true\" data-slot=\"status-dot-indicator\"></span><span data-slot=\"status-dot-sr-label\">Online</span></span>"
        );
        s.style = Some("status-dot+md".into());
        assert!(!render(&s).contains("class=\"s-"));
        let css = include_str!("cronus_ui_css/status-dot.css");
        assert!(
            css.contains("@keyframes cui-ping { 75%, 100% { transform: scale(2); opacity: 0; } }")
        );
        assert!(css.contains("[data-slot=\"status-dot\"].s-xs [data-slot=\"status-dot-indicator\"] { width: 0.375rem; height: 0.375rem; }"));
        assert!(css.contains("[data-slot=\"status-dot\"].s-lg [data-slot=\"status-dot-label\"] { font-size: 1rem; line-height: 1.5rem; }"));
        assert!(css.contains("[data-slot=\"status-dot\"][data-status=\"offline\"] [data-slot=\"status-dot-ping\"] {\n  inset: -0.125rem; background: transparent; border: 2px solid var(--cronus-fg-muted);\n}"));
    }

    /// Docs "On an avatar": `avatar:` wraps an Avatar + the dot in the
    /// `relative inline-flex` span; `position` and `ring` are classes.
    #[test]
    fn avatar_overlay_with_position_and_ring() {
        let mut c = stub("status-dot", "Busy");
        c.style = Some("status-dot+busy".into());
        c.props.insert("position".into(), "bottom-right".into());
        c.props.insert("ring".into(), "true".into());
        c.props.insert("aria-label".into(), "Ada is busy".into());
        c.props.insert("avatar".into(), "AL".into());
        assert_eq!(
            render(&c),
            "<span class=\"cui-status-dot-anchor\"><span data-slot=\"avatar\"><span data-slot=\"avatar-fallback\">AL</span></span><span data-slot=\"status-dot\" class=\"pos-bottom-right\" data-status=\"busy\" role=\"status\" aria-label=\"Ada is busy\"><span aria-hidden=\"true\" data-slot=\"status-dot-indicator\" class=\"ring\"></span><span data-slot=\"status-dot-sr-label\">Busy</span></span></span>"
        );
        c.props
            .insert("src".into(), "https://github.com/shadcn.png".into());
        c.props.insert("alt".into(), "@shadcn".into());
        assert!(render(&c).contains("<img data-slot=\"avatar-image\" src=\"https://github.com/shadcn.png\" alt=\"@shadcn\">"));
        let css = include_str!("cronus_ui_css/status-dot.css");
        assert!(css.contains("[data-slot=\"status-dot\"].pos-bottom-right { position: absolute; bottom: 0; inset-inline-end: 0; }"));
        assert!(css.contains("[data-slot=\"status-dot-indicator\"].ring { box-shadow: 0 0 0 2px var(--cronus-surface-base); }"));
        assert!(
            css.contains(".cui-status-dot-anchor { position: relative; display: inline-flex; }")
        );
    }

    #[test]
    fn skips_interact_pill() {
        let c = stub("status-dot", "Online");
        let html = render(&c);
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
        let css = include_str!("cronus_ui_css/status-dot.css");
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
