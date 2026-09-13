//! Dedicated Banner renderer. DOM matches React's inner bar:
//! `<section data-slot="banner" aria-label="Announcement">` with
//! `banner-content` + `banner-title`. Static HTML — no motion.div / JS.
//! Not the interact `alert("banner")` SURF box.

use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let aria = aria_label(comp).unwrap_or("Announcement");
    let (title, descs) = title_and_descriptions(comp);
    let mut content = format!("<span data-slot=\"banner-title\">{title}</span>");
    for d in descs {
        content.push_str(&format!(
            "<span data-slot=\"banner-description\">{d}</span>"
        ));
    }
    format!(
        "<section data-slot=\"banner\" aria-label=\"{}\"><div data-slot=\"banner-content\">{content}</div></section>",
        esc_attr(aria)
    )
}

fn aria_label(comp: &ComponentNode) -> Option<&str> {
    comp.props
        .get("aria-label")
        .map(String::as_str)
        .or_else(|| comp.props.get("label").map(String::as_str))
        .or_else(|| {
            comp.items
                .iter()
                .find_map(|i| i.config.get("aria-label").map(String::as_str))
        })
        .filter(|s| !s.is_empty())
}

fn title_and_descriptions(comp: &ComponentNode) -> (String, Vec<String>) {
    for kind in ["label", "title", "text", "value"] {
        if let Some((idx, t)) = comp
            .items
            .iter()
            .enumerate()
            .find(|(_, i)| i.item_type == kind && !i.text.is_empty())
        {
            return (esc(&t.text), extra_descs(comp, idx));
        }
    }
    (esc(&comp.name), extra_descs(comp, usize::MAX))
}

fn extra_descs(comp: &ComponentNode, skip: usize) -> Vec<String> {
    comp.items
        .iter()
        .enumerate()
        .filter(|(i, item)| *i != skip && !item.text.is_empty())
        .map(|(_, item)| esc(&item.text))
        .collect()
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn esc_attr(s: &str) -> String {
    esc(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn stub(style: &str, label: &str) -> ComponentNode {
        ComponentNode {
            name: "Banner".into(),
            layout: Some("stack".into()),
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
        }
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("padding:0.85rem 1rem"));
        assert!(!html.contains("color:var(--cronus-fg);font-family:var(--cronus-font-sans"));
        assert!(!html.contains("motion.div"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<div>Demo</div>"));
        assert!(!html.contains("role=\"status\""));
    }

    #[test]
    fn root_is_section_with_title_not_surf_box() {
        let html = render(&stub("banner", "New billing"));
        assert!(html.starts_with("<section "));
        assert!(html.contains("data-slot=\"banner\""));
        assert!(html.contains("aria-label=\"Announcement\""));
        assert!(html.contains("<div data-slot=\"banner-content\">"));
        assert!(html.contains("<span data-slot=\"banner-title\">New billing</span>"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<section data-slot=\"banner\" aria-label=\"Announcement\"><div data-slot=\"banner-content\"><span data-slot=\"banner-title\">New billing</span></div></section>"
        );
    }

    #[test]
    fn extra_text_is_description() {
        let mut c = stub("banner", "New billing");
        c.items.push(ComponentItemNode {
            item_type: "text".into(),
            text: "Usage-based invoices are live.".into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        });
        let html = render(&c);
        assert!(html.contains("data-slot=\"banner-title\">New billing<"));
        assert!(html.contains("data-slot=\"banner-description\">Usage-based invoices are live.<"));
        reject_interact(&html);
    }

    #[test]
    fn aria_label_from_props() {
        let mut c = stub("banner", "Sale");
        c.props.insert("aria-label".into(), "Promo".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Promo\""));
        assert!(!html.contains("aria-label=\"Announcement\""));
    }

    #[test]
    fn skips_interact_surf_box() {
        let html = render(&stub("banner", "New billing"));
        let interact =
            crate::cronus_ui_interact::render("banner", &stub("banner", "New billing")).unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<div data-slot=\"banner\""));
        assert!(interact.contains("role=\"status\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("<div>New billing</div>"));
        assert!(!interact.contains("data-slot=\"banner-title\""));
        assert!(!interact.contains("<section"));
        assert!(html.starts_with("<section "));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"banner\"]"));
        assert!(css.contains("[data-slot=\"banner-content\"]"));
        assert!(css.contains("[data-slot=\"banner-title\"]"));
        assert!(css.contains("display: flex"));
        assert!(css.contains("width: 100%"));
        assert!(css.contains("align-items: center"));
        assert!(css.contains("border-bottom: 1px solid var(--cronus-border)"));
        assert!(css.contains("padding: 0.625rem 1rem"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(!css.contains("zinc-"));
    }
}
