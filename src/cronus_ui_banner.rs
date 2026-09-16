//! Dedicated Banner renderer. DOM matches React's inner bar:
//! `<section data-slot="banner" aria-label="Announcement">` with
//! `banner-content` + `banner-title` (+ `banner-description` from a
//! `description:"…"` attribute or extra texts). Static HTML — no motion.div,
//! no dismiss button (the audit fixture renders `dismissible={false}`).
//! Not the interact `alert("banner")` SURF box.

use crate::cronus_ui_kit::{attr_nonempty, esc};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let aria = aria_label(comp).unwrap_or("Announcement");
    let (title, mut descs) = title_and_descriptions(comp);
    if let Some(d) = attr_nonempty(comp, "description") {
        descs.insert(0, esc(d));
    }
    let mut content = format!("<span data-slot=\"banner-title\">{title}</span>");
    for d in descs {
        content.push_str(&format!(
            "<span data-slot=\"banner-description\">{d}</span>"
        ));
    }
    format!(
        "<section data-slot=\"banner\" aria-label=\"{}\"><div data-slot=\"banner-content\">{content}</div></section>",
        esc(aria)
    )
}

fn aria_label(comp: &ComponentNode) -> Option<&str> {
    attr_nonempty(comp, "aria-label").or_else(|| {
        comp.props
            .get("label")
            .map(String::as_str)
            .filter(|s| !s.is_empty())
    })
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
            span: Default::default(),
        }
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("motion.div"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("role=\"status\""));
    }

    #[test]
    fn root_is_section_with_title_not_surf_box() {
        let html = render(&stub("banner", "New pricing"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<section data-slot=\"banner\" aria-label=\"Announcement\"><div data-slot=\"banner-content\"><span data-slot=\"banner-title\">New pricing</span></div></section>"
        );
    }

    #[test]
    fn description_attribute_and_extra_text() {
        let mut c = stub("banner", "New billing");
        c.items[0]
            .config
            .insert("description".into(), "Usage-based".into());
        assert!(render(&c).contains(
            "<span data-slot=\"banner-title\">New billing</span><span data-slot=\"banner-description\">Usage-based</span>"
        ));
        let mut t = stub("banner", "New billing");
        t.items.push(ComponentItemNode {
            item_type: "text".into(),
            text: "Usage-based invoices are live.".into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        });
        let html = render(&t);
        assert!(html.contains("data-slot=\"banner-description\">Usage-based invoices are live.<"));
        reject_interact(&html);
    }

    #[test]
    fn aria_label_from_props_or_item_config() {
        let mut c = stub("banner", "Sale");
        c.props.insert("aria-label".into(), "Promo".into());
        assert!(render(&c).contains("aria-label=\"Promo\""));
        let mut i = stub("banner", "Sale");
        i.items[0]
            .config
            .insert("aria-label".into(), "Promo".into());
        assert!(render(&i).contains("aria-label=\"Promo\""));
    }

    #[test]
    fn skips_interact_surf_box() {
        let html = render(&stub("banner", "New billing"));
        assert!(html.starts_with("<section "));
    }

    #[test]
    fn chrome_is_token_only_and_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(
            css.contains("display: flex; width: 100%; align-items: center; gap: 0.25rem 0.75rem;")
        );
        assert!(css.contains("padding: 0.625rem 1rem; font-size: 0.875rem; line-height: 1.25rem; text-align: center;"));
        assert!(css.contains("border-bottom: 1px solid var(--cronus-border)"));
        assert!(css.contains("align-items: center; gap: 0.125rem 0.5rem;"));
        assert!(!css.contains("zinc-"));
    }
}
