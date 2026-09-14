//! Dedicated Spinner renderer. DOM matches React: SVG with `data-slot="spinner"`.
//! Not the interact `<div data-slot="spinner" … border-top-color>`.

use crate::cronus_ui_kit::{attr_nonempty, esc};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let aria = aria_label(comp).unwrap_or("Loading");
    format!(
        "<svg data-slot=\"spinner\" role=\"status\" aria-label=\"{}\" viewBox=\"0 0 24 24\" fill=\"none\" xmlns=\"http://www.w3.org/2000/svg\"><circle cx=\"12\" cy=\"12\" r=\"10\" stroke=\"currentColor\" stroke-width=\"3\" opacity=\"0.25\"></circle><path d=\"M12 2a10 10 0 0 1 10 10\" stroke=\"currentColor\" stroke-width=\"3\" stroke-linecap=\"round\" opacity=\"0.9\"></path></svg>",
        esc(aria)
    )
}

fn aria_label(comp: &ComponentNode) -> Option<&str> {
    attr_nonempty(comp, "aria-label")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn stub() -> ComponentNode {
        ComponentNode {
            name: "Spinner".into(),
            layout: Some("inline".into()),
            style: Some("spinner".into()),
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
        }
    }

    #[test]
    fn root_is_svg_not_border_div() {
        let html = render(&stub());
        assert!(html.starts_with("<svg "));
        assert!(html.contains("data-slot=\"spinner\""));
        assert!(html.contains("role=\"status\""));
        assert!(html.contains("aria-label=\"Loading\""));
        assert!(html.contains("viewBox=\"0 0 24 24\""));
        assert!(html.contains("<circle "));
        assert!(html.contains("cx=\"12\""));
        assert!(html.contains("cy=\"12\""));
        assert!(html.contains("r=\"10\""));
        assert!(!html.contains("<div"));
        assert!(!html.contains("border-top-color"));
        assert!(!html.contains("v-data="));
    }

    #[test]
    fn aria_label_from_props() {
        let mut c = stub();
        c.props.insert("aria-label".into(), "Saving".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Saving\""));
        assert!(!html.contains("aria-label=\"Loading\""));
    }

    #[test]
    fn chrome_size_5_and_spin() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"spinner\"]"));
        // Wave 1t: display block (Tailwind preflight `svg { display: block }`);
        // inline-block sat on the 24px line box and dropped the svg 3.8px.
        assert!(css.contains("width: 1.25rem; height: 1.25rem; display: block;"));
        assert!(!css.contains("width: 1.25rem; height: 1.25rem; display: inline-block;"));
        assert!(css.contains("@keyframes spin"));
        assert!(css.contains("animation: spin 1s linear infinite"));
        assert!(!css.contains("border-top-color"));
        assert!(!css.contains("zinc-"));
    }
}
