//! Dedicated Kbd renderer. DOM matches React: `<kbd data-slot="kbd">`.
//! Not interact `pill()` (`<span data-slot="kbd" style=…>`).

use crate::cronus_ui_kit::esc;
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    format!("<kbd data-slot=\"kbd\">{label}</kbd>")
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

    fn stub(keys: &str) -> ComponentNode {
        ComponentNode {
            name: "Kbd".into(),
            layout: Some("inline".into()),
            style: Some("kbd".into()),
            items: vec![ComponentItemNode {
                item_type: "label".into(),
                text: keys.into(),
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
    fn root_is_kbd_not_span_pill() {
        let html = render(&stub("⌘K"));
        assert!(html.starts_with("<kbd "));
        assert!(html.contains("data-slot=\"kbd\""));
        assert!(html.contains("⌘K"));
        assert!(!html.contains("<span"));
        assert!(!html.contains("style="));
        assert_eq!(html, "<kbd data-slot=\"kbd\">⌘K</kbd>");
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"kbd\"]"));
        assert!(css.contains("height: 1.25rem"));
        assert!(css.contains("min-width: 1.25rem"));
        assert!(css.contains("font-size: 0.7rem"));
        assert!(css.contains("var(--cronus-font-mono"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(!css.contains("zinc-"));
    }
}
