//! Dedicated Kbd renderer. DOM matches React: `<kbd data-slot="kbd">`.
//! Two or more `item` lines are a chord: one `<kbd>` per key inside the docs'
//! plain `inline-flex gap-1` span (no slot). Not interact `pill()`
//! (`<span data-slot="kbd" style=…>`).

use crate::cronus_ui_kit::esc;
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let keys: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if keys.len() >= 2 {
        let inner: String = keys
            .iter()
            .map(|k| format!("<kbd data-slot=\"kbd\">{k}</kbd>"))
            .collect();
        return format!("<span class=\"cui-kbd-chord\">{inner}</span>");
    }
    let label = label_of(comp);
    format!("<kbd data-slot=\"kbd\">{label}</kbd>")
}

fn label_of(comp: &ComponentNode) -> String {
    for kind in ["label", "title", "text", "value", "item"] {
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

    /// Docs "Keys": `item "⌘"` + `item "K"` is the chord span of two keys.
    #[test]
    fn items_render_chord_of_kbds() {
        let mut c = stub("");
        c.items.clear();
        for k in ["⌘", "K"] {
            c.items.push(ComponentItemNode {
                item_type: "item".into(),
                text: k.into(),
                link: None,
                tone: None,
                config: HashMap::new(),
            });
        }
        assert_eq!(
            render(&c),
            "<span class=\"cui-kbd-chord\"><kbd data-slot=\"kbd\">⌘</kbd><kbd data-slot=\"kbd\">K</kbd></span>"
        );
        c.items.pop();
        assert_eq!(render(&c), "<kbd data-slot=\"kbd\">⌘</kbd>");
        let css = include_str!("cronus_ui_css/kbd.css");
        assert!(css.contains(
            ".cui-kbd-chord { display: inline-flex; align-items: center; gap: 0.25rem; }"
        ));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/kbd.css");
        assert!(css.contains("[data-slot=\"kbd\"]"));
        assert!(css.contains("height: 1.25rem"));
        assert!(css.contains("min-width: 1.25rem"));
        assert!(css.contains("font-size: 0.7rem"));
        assert!(css.contains("var(--cronus-font-mono"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(!css.contains("zinc-"));
    }
}
