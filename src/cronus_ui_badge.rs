//! Dedicated Badge renderer. DOM matches React: `<span data-slot="badge" data-variant>`
//! with an optional leading lucide glyph (`icon:`; React `[&_svg]:size-3`).
//! Not the legacy `pill()` stub.

use crate::cronus_ui_kit::{attr_nonempty, esc};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let variant = variant_of(comp);
    let label = label_of(comp);
    let icon = attr_nonempty(comp, "icon")
        .map(|i| crate::cronus_ui_icons::svg_or_empty(i))
        .unwrap_or_default();
    badge_html(&format!("{icon}{label}"), variant)
}

/// One React `Badge`; `inner` is already-escaped HTML. Shared by families that
/// embed a badge (card action, data-table cells, description-list details).
pub fn badge_html(inner: &str, variant: &str) -> String {
    format!(
        "<span data-slot=\"badge\" data-variant=\"{}\">{inner}</span>",
        named_variant(variant)
    )
}

fn variant_of(comp: &ComponentNode) -> &'static str {
    if let Some(v) = comp.props.get("variant") {
        return named_variant(v);
    }
    let style = comp.style.as_deref().unwrap_or("");
    for part in style.split('+') {
        let named = named_variant(part);
        if named != "default" || part == "default" {
            if part == "badge" {
                continue;
            }
            if matches!(
                part,
                "destructive"
                    | "primary"
                    | "secondary"
                    | "outline"
                    | "success"
                    | "warning"
                    | "error"
                    | "info"
                    | "default"
            ) {
                return named;
            }
        }
    }
    "default"
}

fn named_variant(raw: &str) -> &'static str {
    match raw {
        "primary" => "primary",
        "secondary" => "secondary",
        "outline" => "outline",
        "success" => "success",
        "warning" => "warning",
        "destructive" | "danger" => "destructive",
        "error" => "error",
        "info" => "info",
        "default" => "default",
        _ => "default",
    }
}

fn label_of(comp: &ComponentNode) -> String {
    for kind in ["label", "title", "text", "value"] {
        if let Some(t) = item(comp, kind) {
            if !t.is_empty() {
                return esc(t);
            }
        }
    }
    esc(&comp.name)
}

fn item<'a>(comp: &'a ComponentNode, kind: &str) -> Option<&'a str> {
    comp.items
        .iter()
        .find(|i| i.item_type == kind)
        .map(|i| i.text.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn stub(style: &str, label: &str) -> ComponentNode {
        ComponentNode {
            name: "Badge".into(),
            layout: Some("inline".into()),
            style: Some(style.into()),
            items: vec![ComponentItemNode {
                item_type: "label".into(),
                text: label.into(),
                link: None,
                tone: None,
                config: HashMap::new(),
            }],
            props: HashMap::new(),
            params: vec![],
            template: None,
            sections: vec![],
            state: vec![],
            tests: vec![],
            binding: None,
            span: Default::default(),
        }
    }

    #[test]
    fn default_variant_is_default_not_primary() {
        let html = render(&stub("badge", "New"));
        assert!(html.starts_with("<span"));
        assert!(html.contains("data-slot=\"badge\""));
        assert!(html.contains("data-variant=\"default\""));
        assert!(html.contains("New"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_pairs_text_xs_with_1rem_line_height() {
        // Wave 1t: Tailwind text-xs = 0.75rem/1rem → badge height 22px, not 24.
        let css = include_str!("cronus_ui_css/badge.css");
        assert!(css.contains(
            "padding: 0.125rem 0.5rem; font-size: 0.75rem; line-height: 1rem; font-weight: 500;"
        ));
    }

    #[test]
    fn primary_from_style() {
        let html = render(&stub("badge+primary", "Beta"));
        assert!(html.contains("data-variant=\"primary\""));
    }

    #[test]
    fn destructive_from_style() {
        let html = render(&stub("badge+destructive", "Failed"));
        assert!(html.contains("data-variant=\"destructive\""));
    }

    /// Docs "With icon": `icon:check` puts the lucide glyph before the label
    /// (React `<Check aria-hidden />` child, `[&_svg]:size-3`).
    #[test]
    fn icon_prop_renders_glyph_before_label() {
        let mut c = stub("badge+success", "Verified");
        c.props.insert("icon".into(), "check".into());
        let html = render(&c);
        assert!(html.starts_with("<span data-slot=\"badge\" data-variant=\"success\"><svg "));
        assert!(html.contains("data-icon=\"check\""));
        assert!(html.ends_with("</svg>Verified</span>"));
        let css = include_str!("cronus_ui_css/badge.css");
        assert!(css.contains(
            "[data-slot=\"badge\"] svg { width: 0.75rem; height: 0.75rem; flex-shrink: 0; }"
        ));
    }

    #[test]
    fn badge_html_is_the_shared_react_badge() {
        assert_eq!(
            badge_html("Paid", "success"),
            "<span data-slot=\"badge\" data-variant=\"success\">Paid</span>"
        );
        assert_eq!(
            badge_html("x", "bogus"),
            "<span data-slot=\"badge\" data-variant=\"default\">x</span>"
        );
    }
}
