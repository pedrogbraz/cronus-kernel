//! Dedicated HoverCard renderer. Always-open static DOM (no hover/JS).
//! Trigger `<button type="button">` from label plus
//! `<div data-slot="hover-card-content">` from extra text.
//! React Root emits no data-slot; content slot is the contract.
//! Not interact `popover("hover-card")` SURF `<details>` overlay.

use crate::cronus_ui_kit::{label_of, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let ts = texts(comp);
    let trigger = ts.first().cloned().unwrap_or_else(|| label_of(comp));
    let body = ts.iter().skip(1).cloned().collect::<Vec<_>>().join("");
    format!(
        "<span data-slot=\"hover-card\"><button type=\"button\">{trigger}</button><div data-slot=\"hover-card-content\">{body}</div></span>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;
    use std::collections::HashMap;

    fn reject_interact(html: &str) {
        assert!(!html.contains("<details"));
        assert!(!html.contains("<summary"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("onmouse"));
    }

    fn with_body(label: &str, body: &str) -> crate::parser::ComponentNode {
        let mut c = stub("hover-card", label);
        c.items.push(ComponentItemNode {
            item_type: "text".into(),
            text: body.into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        });
        c
    }

    #[test]
    fn trigger_button_and_always_open_content() {
        let html = render(&with_body("Preview", "Native disclosure."));
        assert!(html.contains("data-slot=\"hover-card\""));
        assert!(html.contains(">Preview</button>"));
        assert!(html.contains("data-slot=\"hover-card-content\""));
        assert!(html.contains(">Native disclosure.</div>"));
        reject_interact(&html);
    }

    #[test]
    fn content_slot_is_the_contract() {
        let html = render(&stub("hover-card", "Preview"));
        assert!(html.contains("data-slot=\"hover-card-content\""));
        assert!(html.contains("<button type=\"button\">Preview</button>"));
        assert!(html.contains("data-slot=\"hover-card\""));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_surf_details() {
        let html = render(&with_body("Preview", "Native disclosure."));
        let interact = crate::cronus_ui_interact::render(
            "hover-card",
            &with_body("Preview", "Native disclosure."),
        )
        .unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<details data-slot=\"hover-card\""));
        assert!(interact.contains("style="));
        assert!(!interact.contains("data-slot=\"hover-card-content\""));
        assert!(!html.contains("<details"));
        reject_interact(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"hover-card-content\"]"));
        assert!(css.contains("z-index: 50"));
        assert!(css.contains("width: 16rem"));
        assert!(css.contains("padding: 0.75rem"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-shadow-lg"));
        assert!(!css.contains("zinc-"));
    }
}
