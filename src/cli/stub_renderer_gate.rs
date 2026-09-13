//! Stub + interact gate. A family is `ported` only when the dispatch arm
//! calls a dedicated renderer — not `fx`/`pill`/interact generic HTML.

use crate::cli::audit_codes::{AuditFinding, STUB_RENDERER};
use crate::cronus_ui_widgets::{self, PORTED_FAMILIES};
use crate::parser::AstNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererKind {
    Dedicated(&'static str),
    Interact,
    Stub(&'static str),
    Missing,
}

pub fn dedicated_fn_name(family: &str) -> Option<&'static str> {
    match family {
        "button" => Some("button_from"),
        "badge" => Some("cronus_ui_badge::render"),
        "input" => Some("cronus_ui_input::render"),
        "label" => Some("cronus_ui_label::render"),
        "textarea" => Some("cronus_ui_textarea::render"),
        "checkbox" => Some("cronus_ui_checkbox::render"),
        "switch" => Some("cronus_ui_switch::render"),
        "spinner" => Some("cronus_ui_spinner::render"),
        "separator" => Some("cronus_ui_separator::render"),
        "kbd" => Some("cronus_ui_kbd::render"),
        "toggle" => Some("cronus_ui_toggle::render"),
        "progress" => Some("cronus_ui_progress::render"),
        "slider" => Some("cronus_ui_slider::render"),
        "radio-group" => Some("cronus_ui_radio_group::render"),
        _ => None,
    }
}

pub fn renderer_kind(family: &str) -> RendererKind {
    if PORTED_FAMILIES.contains(&family) {
        return match dedicated_fn_name(family) {
            Some(name) => RendererKind::Dedicated(name),
            None => RendererKind::Missing,
        };
    }
    if crate::cronus_ui_interact::covers(family) {
        return RendererKind::Interact;
    }
    match catalog_stub_kind(family) {
        Some(kind) => RendererKind::Stub(kind),
        None => RendererKind::Missing,
    }
}

/// Catalog kind from the generator table (pill/field/overlay/nav/display/chart/fx).
/// Not a proof of port — interact still wins for unported families.
pub fn catalog_stub_kind(family: &str) -> Option<&'static str> {
    match family {
        "button" => Some("button"),
        "badge" | "chip" | "kbd" | "status-dot" | "spinner" | "skeleton" | "separator"
        | "label" | "progress" | "slider" | "rating" | "copy-button" | "fab" | "toggle"
        | "switch" | "checkbox" | "radio-group" | "toggle-group" | "segmented-control" => {
            Some("pill")
        }
        "input" | "textarea" | "password-input" | "number-input" | "phone-input"
        | "currency-input" | "credit-card-input" | "floating-label-input" | "tags-input"
        | "input-otp" | "input-group" | "color-picker" | "file-dropzone" | "select"
        | "combobox" | "autocomplete" | "multi-select" | "date-picker" | "date-range-picker"
        | "time-picker" | "field" | "form" | "signature-pad" | "rich-text-editor" => Some("field"),
        "dialog" | "alert-dialog" | "confirmation-dialog" | "invite-dialog" | "sheet"
        | "drawer" | "popover" | "hover-card" | "tooltip" | "dropdown-menu" | "context-menu"
        | "menubar" | "command" | "lightbox" | "morphing-popover" | "notification-center"
        | "sonner" => Some("overlay"),
        "tabs" | "accordion" | "collapsible" | "breadcrumb" | "pagination" | "sidebar"
        | "navigation-menu" | "pill-nav" | "expandable-tabs" | "stepper" | "toolbar"
        | "table-of-contents" | "app-shell" | "workspace-switcher" | "dock" | "mode-toggle"
        | "split-button" | "button-group" => Some("nav"),
        "card" | "card-stack" | "glass-card" | "spotlight-card" | "tilt-card" | "flip-card"
        | "banner" | "empty" | "alert" | "metric" | "description-list" | "table"
        | "data-table" | "avatar" | "avatar-group" | "aspect-ratio" | "frame" | "scroll-area"
        | "resizable" | "calendar" | "timeline" | "kanban" | "scheduler" | "json-viewer"
        | "code-block" | "code-tabs" | "terminal" | "tree-view" | "carousel"
        | "logo-carousel" | "masonry" | "usage-meter" | "video-player" => Some("display"),
        "area-chart" | "bar-chart" | "line-chart" | "pie-chart" | "radar-chart"
        | "composed-chart" | "candlestick-chart" | "choropleth-chart" | "funnel-chart"
        | "gauge-chart" | "heatmap" | "heatmap-chart" | "live-line-chart"
        | "profit-loss-chart" | "ring-chart" | "sankey-chart" | "scatter-chart" | "sparkline"
        | "sunburst-chart" | "chart" => Some("chart"),
        "animated-button" | "animated-list" | "animated-number" | "aurora-background"
        | "border-beam" | "click-spark" | "confetti" | "countdown" | "dot-pattern"
        | "flickering-grid" | "glare-hover" | "gradient-border" | "gradient-text"
        | "grid-pattern" | "highlighter" | "image-zoom" | "light-rays" | "magnetic"
        | "marquee" | "meteors" | "noise" | "orbit" | "particles" | "progressive-blur"
        | "retro-grid" | "reveal" | "ripple" | "scramble-text" | "scroll-progress"
        | "shimmer" | "shiny-text" | "sparkles-text" | "spinning-text" | "star-border"
        | "text-effect" | "typing-text" | "word-rotate" | "comparison-slider"
        | "dynamic-island" | "toast" | "motion-presets" => Some("fx"),
        _ => None,
    }
}

pub fn looks_like_interact_generic(html: &str) -> bool {
    html.contains("data-slot=\"input-control\"")
        || html.contains("<label data-slot=\"input\"")
        || html.contains("height:2.5rem;padding:0 0.75rem;border-radius:0.5rem;border:1px solid var(--cronus-border);background:var(--cronus-surface-inset")
        || html.contains("data-slot=\"switch-control\"")
        || html.contains("<label data-slot=\"switch\"")
        || (html.contains("data-slot=\"spinner\"") && html.contains("border-top-color"))
        || html.contains("<hr data-slot=\"separator\"")
        || html.contains("data-slot=\"slider-control\"")
        || html.contains("<label data-slot=\"slider\"")
        || html.contains("<input type=\"radio\"")
}

pub fn looks_like_stub_fingerprint(html: &str) -> Option<&'static str> {
    if html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden") {
        return Some("fx");
    }
    if html.contains("<figure data-slot") && html.contains("<figcaption") {
        return Some("chart");
    }
    if html.contains("display:inline-flex;align-items:center;gap:0.35rem;padding:0.15rem 0.55rem;font-size:0.75rem;font-weight:500;") {
        return Some("pill");
    }
    if html.contains("<input data-slot=") && html.contains("-control\"") {
        return Some("field");
    }
    if html.contains("role=\"dialog\"") && html.contains("max-width:28rem;") {
        return Some("overlay");
    }
    if html.contains("<nav data-slot") && html.contains("flex-wrap:wrap;gap:0.25rem") {
        return Some("nav");
    }
    if html.contains("<section data-slot") && html.contains("padding:1rem;display:flex;flex-direction:column;gap:0.5rem;") {
        return Some("display");
    }
    None
}

pub fn check_family(family: &str) -> Result<(), AuditFinding> {
    match renderer_kind(family) {
        RendererKind::Dedicated(_) => Ok(()),
        RendererKind::Stub(kind) => Err(AuditFinding::fail(
            "logic",
            STUB_RENDERER,
            format!("family {family} is stub renderer {kind}"),
        )),
        RendererKind::Interact => Err(AuditFinding::fail(
            "logic",
            STUB_RENDERER,
            format!("family {family} is interact generic, not a dedicated port"),
        )),
        RendererKind::Missing => Err(AuditFinding::fail(
            "logic",
            STUB_RENDERER,
            format!("family {family} has no renderer"),
        )),
    }
}

pub fn check_ast(nodes: &[AstNode]) -> Vec<AuditFinding> {
    let mut out = Vec::new();
    for node in nodes {
        if let AstNode::Component(comp) = node {
            if let Some(family) = cronus_ui_widgets::family_of(comp) {
                if let Err(mut f) = check_family(family) {
                    f.family = Some(family.to_string());
                    out.push(f);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_widgets::{render, FAMILIES};

    fn stub(family: &str) -> crate::parser::ComponentNode {
        crate::cronus_ui_widgets::test_stub(family)
    }

    #[test]
    fn button_is_dedicated() {
        assert_eq!(renderer_kind("button"), RendererKind::Dedicated("button_from"));
        assert!(check_family("button").is_ok());
        let html = render(&stub("button")).unwrap();
        assert!(html.contains("data-slot=\"button\""));
        assert!(looks_like_stub_fingerprint(&html).is_none());
        assert!(!looks_like_interact_generic(&html));
    }

    #[test]
    fn area_chart_is_stub() {
        assert_eq!(renderer_kind("area-chart"), RendererKind::Stub("chart"));
        let err = check_family("area-chart").unwrap_err();
        assert_eq!(err.code, STUB_RENDERER);
        let html = render(&stub("area-chart")).unwrap();
        assert_eq!(looks_like_stub_fingerprint(&html), Some("chart"));
    }

    #[test]
    fn meteors_is_stub_fx() {
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert!(check_family("meteors").is_err());
        let html = render(&stub("meteors")).unwrap();
        assert_eq!(looks_like_stub_fingerprint(&html), Some("fx"));
    }

    #[test]
    fn every_family_is_classified() {
        for family in FAMILIES {
            assert!(
                !matches!(renderer_kind(family), RendererKind::Missing),
                "{family} missing RendererKind"
            );
        }
    }

    #[test]
    fn ported_families_are_dedicated_and_skip_interact() {
        for family in PORTED_FAMILIES {
            assert!(
                matches!(renderer_kind(family), RendererKind::Dedicated(_)),
                "{family}"
            );
            let html = render(&stub(family)).expect(family);
            if let Some(ih) = crate::cronus_ui_interact::render(family, &stub(family)) {
                assert_ne!(html, ih, "{family} still equals interact HTML");
            }
            assert!(
                !looks_like_interact_generic(&html),
                "{family} looks like interact: {html}"
            );
            assert!(
                looks_like_stub_fingerprint(&html).is_none(),
                "{family} looks like stub: {html}"
            );
        }
    }

    #[test]
    fn no_new_stub_families() {
        assert_eq!(FAMILIES.len(), 173);
        for family in PORTED_FAMILIES {
            assert!(FAMILIES.contains(family));
            assert!(cronus_ui_widgets::dedicated_render(family, &stub(family)).is_some());
        }
    }

    #[test]
    fn dedicated_module_no_sidecar_assets() {
        let files = [
            "src/cronus_ui.rs",
            "src/cronus_ui_widgets.rs",
            "src/cronus_ui_badge.rs",
            "src/cronus_ui_checkbox.rs",
            "src/cronus_ui_input.rs",
            "src/cronus_ui_label.rs",
            "src/cronus_ui_textarea.rs",
            "src/cronus_ui_switch.rs",
            "src/cronus_ui_spinner.rs",
            "src/cronus_ui_separator.rs",
            "src/cronus_ui_kbd.rs",
            "src/cronus_ui_toggle.rs",
            "src/cronus_ui_progress.rs",
            "src/cronus_ui_slider.rs",
            "src/cronus_ui_radio_group.rs",
            "src/cli/audit_http.rs",
            "src/ui/audit_layout.rs",
        ];
        for file in files {
            let src = std::fs::read_to_string(file).unwrap_or_else(|_| panic!("read {file}"));
            for (i, line) in src.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }
                if trimmed.contains("include_str!") && !trimmed.contains("cronus_ui_tokens.css") {
                    panic!("{file}:{} sidecar include_str: {trimmed}", i + 1);
                }
                if (trimmed.contains("fs::read(")
                    || trimmed.contains("fs::read_to_string")
                    || trimmed.contains("std::fs::read_to_string"))
                    && !trimmed.contains("cronus_ui_tokens")
                    && !file.contains("audit_http")
                    && !file.contains("source_language")
                {
                    // audit_http must never read disk; this file is checked below.
                    if file.ends_with("audit_http.rs") {
                        panic!("{file}:{} must not read disk: {trimmed}", i + 1);
                    }
                }
            }
        }
        let http = std::fs::read_to_string("src/cli/audit_http.rs").unwrap();
        assert!(!http.contains("fs::read"), "audit_http must never read disk");
        assert!(
            !http.contains("read_to_string"),
            "audit_http must never read disk"
        );
    }
}
