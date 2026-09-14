//! Dedicated Card renderer. DOM matches React: `<div data-slot="card">`
//! with `card-header` / `card-title` (from label), optional `card-description`
//! (from a `description:"…"` attribute, else the first extra text), and
//! `card-content` if more.
//! Not interact `card()` (`<section data-slot="card" style=…SURF…>` without card-title).

use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let title = esc(title_of(comp));
    let extras = extras_of(comp);
    let (desc, rest): (Option<&str>, &[&str]) = match attr(comp, "description") {
        Some(d) => (Some(d), &extras[..]),
        None if extras.is_empty() => (None, &extras[..]),
        None => (Some(extras[0]), &extras[1..]),
    };
    let mut header =
        format!("<div data-slot=\"card-header\"><div data-slot=\"card-title\">{title}</div>");
    if let Some(desc) = desc {
        header.push_str(&format!(
            "<div data-slot=\"card-description\">{}</div>",
            esc(desc)
        ));
    }
    header.push_str("</div>");
    let mut content = String::new();
    if !rest.is_empty() {
        let body = rest.iter().map(|t| esc(t)).collect::<Vec<_>>().join(" ");
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

/// Prop or item attribute (`key:"value"` lines attach to the preceding item).
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
        assert!(!html.contains("<section"));
        assert!(!html.contains("style="));
        assert_eq!(
            html,
            "<div data-slot=\"card\"><div data-slot=\"card-header\"><div data-slot=\"card-title\">Overview</div></div></div>"
        );
    }

    #[test]
    fn description_attribute_matches_react_header() {
        let mut c = stub("Subscription");
        c.items[0]
            .config
            .insert("description".into(), "Monthly plan".into());
        assert_eq!(
            render(&c),
            "<div data-slot=\"card\"><div data-slot=\"card-header\"><div data-slot=\"card-title\">Subscription</div><div data-slot=\"card-description\">Monthly plan</div></div></div>"
        );
        let mut p = stub("Subscription");
        p.props.insert("description".into(), "A & B".into());
        assert!(render(&p).contains("<div data-slot=\"card-description\">A &amp; B</div>"));
    }

    #[test]
    fn description_from_extra_text() {
        let mut c = stub("Overview");
        c.items.push(extra("text", "Weekly usage"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"card-description\">Weekly usage</div>"));
        assert!(!html.contains("data-slot=\"card-content\""));
    }

    #[test]
    fn content_when_more() {
        let mut c = stub("Overview");
        c.items.push(extra("text", "Weekly usage"));
        c.items.push(extra("item", "12,400 requests"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"card-description\">Weekly usage</div>"));
        assert!(html.contains("data-slot=\"card-content\">12,400 requests</div>"));
        let mut d = stub("Overview");
        d.items[0]
            .config
            .insert("description".into(), "Weekly usage".into());
        d.items.push(extra("text", "12,400 requests"));
        assert!(render(&d).contains("data-slot=\"card-content\">12,400 requests</div>"));
    }

    #[test]
    fn skips_interact_section() {
        let html = render(&stub("Overview"));
        let interact = crate::cronus_ui_interact::render("card", &stub("Overview")).unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<section data-slot=\"card\""));
        assert!(!html.contains("v-data="));
    }

    #[test]
    fn chrome_is_token_only_and_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("gap: 1.5rem"));
        assert!(css.contains("padding-top: 1.5rem"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(css.contains("[data-slot=\"card-title\"] {\n  min-width: 0; overflow-wrap: break-word;\n  font-family: var(--cronus-font-display, inherit); font-weight: 600; line-height: 1;"));
        assert!(css.contains("grid-column: 1 / -1; font-size: 0.875rem; line-height: 1.25rem;"));
        assert!(css.contains("@media (max-width: 639.98px) {\n  [data-slot=\"card-header\"], [data-slot=\"card-content\"] { padding-left: 1rem; padding-right: 1rem; }"));
        assert!(!css.contains("zinc-"));
    }
}
