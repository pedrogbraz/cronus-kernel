//! Dedicated Chip renderer. DOM matches React default (non-interactive):
//! `<span data-slot="chip">label</span>`. React does not emit `data-size`.
//! Not interact `pill("chip")` (`<span data-slot="chip" style=…BASE SURF…>`).

use crate::cronus_ui_kit::esc;
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    format!("<span data-slot=\"chip\">{label}</span>")
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

    fn stub(label: &str) -> ComponentNode {
        ComponentNode {
            name: "Chip".into(),
            layout: Some("inline".into()),
            style: Some("chip".into()),
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
    fn root_is_span_without_data_size() {
        let html = render(&stub("Filter"));
        assert!(html.starts_with("<span "));
        assert!(html.contains("data-slot=\"chip\""));
        assert!(html.contains("Filter"));
        assert!(!html.contains("data-size"));
        assert!(!html.contains("data-variant"));
        assert!(!html.contains("style="));
        assert_eq!(html, "<span data-slot=\"chip\">Filter</span>");
    }

    #[test]
    fn skips_interact_pill() {
        let html = render(&stub("New"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"chip\"]"));
        assert!(css.contains("display: inline-flex"));
        assert!(css.contains("height: 1.75rem"));
        assert!(css.contains("border-radius: 9999px"));
        assert!(css.contains("padding: 0 0.625rem"));
        assert!(css.contains("font-size: 0.875rem"));
        assert!(!css.contains("zinc-"));
    }

    #[test]
    fn chrome_pairs_text_sm_with_line_height() {
        // Wave 1t: text-sm = 0.875rem/1.25rem (20px), not the canvas 1.5.
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "padding: 0 0.625rem; font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;"
        ));
    }
}
