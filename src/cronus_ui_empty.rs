//! Dedicated Empty renderer. DOM matches React:
//! `<div data-slot="empty">` with an optional `empty-icon` (`icon:` lucide
//! glyph), `empty-title` (label/title), `empty-description` (the next text)
//! and `empty-content` holding one Button per `action` item (`variant:`).
//! Not interact `alert("empty")` (SURF box with raw divs, no empty-title).

use crate::cronus_ui_kit::{attr_nonempty, esc};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let title = esc(&title_of(comp));
    let mut inner = String::new();
    if let Some(icon) = attr_nonempty(comp, "icon").and_then(crate::cronus_ui_icons::svg) {
        inner.push_str(&format!("<div data-slot=\"empty-icon\">{icon}</div>"));
    }
    inner.push_str(&format!("<div data-slot=\"empty-title\">{title}</div>"));
    if let Some(desc) = extras_of(comp).first() {
        inner.push_str(&format!(
            "<div data-slot=\"empty-description\">{}</div>",
            esc(desc)
        ));
    }
    let actions: String = comp
        .items
        .iter()
        .filter(|i| i.item_type == "action" && !i.text.is_empty())
        .map(|a| {
            let variant = a
                .config
                .get("variant")
                .map(String::as_str)
                .unwrap_or("primary");
            crate::cronus_ui::button_html(
                &esc(&a.text),
                variant,
                "md",
                a.link.as_deref(),
                false,
                None,
            )
        })
        .collect();
    if !actions.is_empty() {
        inner.push_str(&format!("<div data-slot=\"empty-content\">{actions}</div>"));
    }
    format!("<div data-slot=\"empty\">{inner}</div>")
}

fn title_of(comp: &ComponentNode) -> &str {
    for kind in ["label", "title", "text", "value"] {
        if let Some(t) = item(comp, kind) {
            if !t.is_empty() {
                return t;
            }
        }
    }
    for i in &comp.items {
        if !i.text.is_empty() {
            return i.text.as_str();
        }
    }
    &comp.name
}

fn extras_of(comp: &ComponentNode) -> Vec<&str> {
    let title = title_of(comp);
    let mut skipped = false;
    let mut out = Vec::new();
    for i in &comp.items {
        if i.text.is_empty() || i.item_type == "action" {
            continue;
        }
        if !skipped && i.text == title {
            skipped = true;
            continue;
        }
        out.push(i.text.as_str());
    }
    out
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

    fn stub(title: &str) -> ComponentNode {
        ComponentNode {
            name: "Empty".into(),
            layout: Some("stack".into()),
            style: Some("empty".into()),
            items: vec![ComponentItemNode {
                item_type: "label".into(),
                text: title.into(),
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

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        }
    }

    #[test]
    fn root_is_div_with_title_not_alert_surf() {
        let html = render(&stub("No results"));
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"empty\""));
        assert!(html.contains("data-slot=\"empty-title\""));
        assert!(html.contains(">No results</div>"));
        assert!(!html.contains("role=\"status\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert_eq!(
            html,
            "<div data-slot=\"empty\"><div data-slot=\"empty-title\">No results</div></div>"
        );
    }

    #[test]
    fn description_from_extra_text() {
        let mut c = stub("No results");
        c.items.push(extra("text", "Try a different filter."));
        let html = render(&c);
        assert!(html.contains("data-slot=\"empty-title\">No results</div>"));
        assert!(html.contains("data-slot=\"empty-description\">Try a different filter.</div>"));
        assert!(!html.contains("style="));
        assert!(!html.contains("role=\"status\""));
    }

    /// Docs "Empty state": `icon:inbox` opens the composition with the
    /// `empty-icon` circle, the `action` item closes it in `empty-content`.
    #[test]
    fn icon_and_action_render_react_icon_and_content_slots() {
        let mut c = stub("No messages yet");
        c.items[0].item_type = "title".into();
        c.props.insert("icon".into(), "inbox".into());
        c.items
            .push(extra("text", "Start a conversation to get going."));
        c.items.push(extra("action", "New message"));
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"empty\"><div data-slot=\"empty-icon\"><svg "));
        assert!(html.contains("data-icon=\"inbox\""));
        assert!(html.contains("</div><div data-slot=\"empty-title\">No messages yet</div><div data-slot=\"empty-description\">Start a conversation to get going.</div><div data-slot=\"empty-content\"><button type=\"button\" data-slot=\"button\" data-variant=\"primary\" data-size=\"md\" class=\"cui-btn\">New message</button></div></div>"));
        assert!(!html.contains("style="));
        let css = include_str!("cronus_ui_css/empty.css");
        assert!(css.contains("[data-slot=\"empty-icon\"] {\n  display: flex; width: 2.75rem; height: 2.75rem; align-items: center; justify-content: center;"));
        assert!(css.contains("[data-slot=\"empty-icon\"] svg { width: 1.25rem; height: 1.25rem; }"));
        assert!(css.contains("[data-slot=\"empty-content\"] { display: flex; align-items: center; gap: 0.5rem; margin-top: 0.25rem; }"));
    }

    #[test]
    fn skips_interact_alert() {
        let html = render(&stub("No results"));
        assert!(!html.contains("v-data="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/empty.css");
        assert!(css.contains("[data-slot=\"empty\"]"));
        assert!(css.contains("[data-slot=\"empty-title\"]"));
        assert!(css.contains("[data-slot=\"empty-description\"]"));
        assert!(css.contains("flex-direction: column"));
        assert!(css.contains("align-items: center"));
        assert!(css.contains("gap: 0.75rem"));
        assert!(css.contains("border: 1px dashed var(--cronus-border)"));
        assert!(css.contains("padding: 3rem 1.5rem"));
        assert!(css.contains("text-align: center"));
        assert!(!css.contains("zinc-"));
    }

    #[test]
    fn title_is_display_font_text_sm_line_height() {
        // Wave 1t: React empty-title = font-display text-sm (20px line-height).
        let css = include_str!("cronus_ui_css/empty.css");
        assert!(css.contains("[data-slot=\"empty-title\"] {\n  font-family: var(--cronus-font-display, inherit);\n  font-size: 0.875rem; line-height: 1.25rem; font-weight: 600;"));
    }
}
