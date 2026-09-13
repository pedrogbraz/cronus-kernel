//! Dedicated Empty renderer. DOM matches React:
//! `<div data-slot="empty"><div data-slot="empty-title">` from label.
//! Optional extra text as `data-slot="empty-description"`.
//! Not interact `alert("empty")` (SURF box with raw divs, no empty-title).

use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let title = esc(&title_of(comp));
    let mut inner = format!("<div data-slot=\"empty-title\">{title}</div>");
    if let Some(desc) = extras_of(comp).first() {
        inner.push_str(&format!(
            "<div data-slot=\"empty-description\">{}</div>",
            esc(desc)
        ));
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
        c.items.push(ComponentItemNode {
            item_type: "text".into(),
            text: "Try a different filter.".into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        });
        let html = render(&c);
        assert!(html.contains("data-slot=\"empty-title\">No results</div>"));
        assert!(html.contains("data-slot=\"empty-description\">Try a different filter.</div>"));
        assert!(!html.contains("style="));
        assert!(!html.contains("role=\"status\""));
    }

    #[test]
    fn skips_interact_alert() {
        let html = render(&stub("No results"));
        let interact = crate::cronus_ui_interact::render("empty", &stub("No results")).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("data-slot=\"empty\""));
        assert!(interact.contains("role=\"status\""));
        assert!(interact.contains("style="));
        assert!(!interact.contains("data-slot=\"empty-title\""));
        assert!(!html.contains("v-data="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
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
}
