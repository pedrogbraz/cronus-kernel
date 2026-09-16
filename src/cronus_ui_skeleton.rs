//! Dedicated Skeleton renderer. DOM matches React:
//! `<div data-slot="skeleton" aria-hidden="true">` (empty).
//! Not the interact inline `height:0.9rem;width:8rem;…background:var(--cronus-border)`.

use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(_comp: &ComponentNode) -> String {
    "<div data-slot=\"skeleton\" aria-hidden=\"true\"></div>".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn stub() -> ComponentNode {
        ComponentNode {
            name: "Skeleton".into(),
            layout: Some("inline".into()),
            style: Some("skeleton".into()),
            items: vec![ComponentItemNode {
                item_type: "label".into(),
                text: "Demo".into(),
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
    fn root_is_empty_div_not_inline_size() {
        let html = render(&stub());
        assert_eq!(
            html,
            "<div data-slot=\"skeleton\" aria-hidden=\"true\"></div>"
        );
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"skeleton\""));
        assert!(html.contains("aria-hidden=\"true\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("height:0.9rem"));
        assert!(!html.contains("width:8rem"));
        assert!(!html.contains("background:var(--cronus-border)"));
        assert!(!html.contains("Demo"));
        assert!(!html.contains("v-data="));
    }

    #[test]
    fn skips_interact_inline_dump() {
        let html = render(&stub());
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_default_size_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"skeleton\"]"));
        assert!(css.contains("width: 8rem"));
        // Wave 1t: harness default className `h-4 w-32` → 16px, not 14.4px.
        assert!(css.contains("display: block; height: 1rem; width: var(--cui-skeleton-w, 100%);"));
        assert!(css.contains("border-radius: var(--cronus-radius-md)"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(css.contains("@keyframes cui-pulse"));
        assert!(css.contains("animation: cui-pulse"));
        assert!(!css.contains("zinc-"));
    }
}
