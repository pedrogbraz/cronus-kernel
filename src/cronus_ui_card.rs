//! Dedicated Card renderer. DOM matches React: `<div data-slot="card">`
//! with `card-header` / `card-title` (from label), optional `card-description`
//! from extra text, and `card-content` if more.
//! Not interact `card()` (`<section data-slot="card" style=…SURF…>` without card-title).

use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let title = esc(&title_of(comp));
    let extras = extras_of(comp);
    let mut header = format!("<div data-slot=\"card-header\"><div data-slot=\"card-title\">{title}</div>");
    if let Some(desc) = extras.first() {
        header.push_str(&format!(
            "<div data-slot=\"card-description\">{}</div>",
            esc(desc)
        ));
    }
    header.push_str("</div>");
    let mut content = String::new();
    if extras.len() > 1 {
        let body = extras[1..].iter().map(|t| esc(t)).collect::<Vec<_>>().join(" ");
        content = format!("<div data-slot=\"card-content\">{body}</div>");
    }
    format!("<div data-slot=\"card\">{header}{content}</div>")
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
        if i.text.is_empty() {
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

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn stub(title: &str) -> ComponentNode {
        ComponentNode {
            name: "Card".into(),
            layout: Some("stack".into()),
            style: Some("card".into()),
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
        }
    }

    fn extra(item_type: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: item_type.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        }
    }

    #[test]
    fn root_is_div_with_title_not_section() {
        let html = render(&stub("Overview"));
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"card\""));
        assert!(html.contains("data-slot=\"card-header\""));
        assert!(html.contains("data-slot=\"card-title\""));
        assert!(html.contains(">Overview</div>"));
        assert!(!html.contains("<section"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert_eq!(
            html,
            "<div data-slot=\"card\"><div data-slot=\"card-header\"><div data-slot=\"card-title\">Overview</div></div></div>"
        );
    }

    #[test]
    fn description_from_extra_text() {
        let mut c = stub("Overview");
        c.items.push(extra("text", "Weekly usage"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"card-description\""));
        assert!(html.contains(">Weekly usage</div>"));
        assert!(!html.contains("data-slot=\"card-content\""));
        assert!(!html.contains("<section"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn content_when_more() {
        let mut c = stub("Overview");
        c.items.push(extra("text", "Weekly usage"));
        c.items.push(extra("item", "12,400 requests"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"card-description\">Weekly usage</div>"));
        assert!(html.contains("data-slot=\"card-content\">12,400 requests</div>"));
        assert!(html.starts_with("<div data-slot=\"card\">"));
        assert!(!html.contains("<section"));
    }

    #[test]
    fn skips_interact_section() {
        let html = render(&stub("Overview"));
        let interact = crate::cronus_ui_interact::render("card", &stub("Overview")).unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<section data-slot=\"card\""));
        assert!(interact.contains("style="));
        assert!(!interact.contains("data-slot=\"card-title\""));
        assert!(!html.contains("v-data="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"card\"]"));
        assert!(css.contains("[data-slot=\"card-title\"]"));
        assert!(css.contains("[data-slot=\"card-description\"]"));
        assert!(css.contains("display: flex"));
        assert!(css.contains("flex-direction: column"));
        assert!(css.contains("gap: 1.5rem"));
        assert!(css.contains("padding-top: 1.5rem"));
        assert!(css.contains("padding-bottom: 1.5rem"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(css.contains("font-weight: 500"));
        assert!(css.contains("font-size: 0.875rem"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(!css.contains("zinc-"));
    }
}
