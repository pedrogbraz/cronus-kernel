//! Dedicated HoverCard renderer. DOM mirrors React (`HoverCardTrigger asChild`
//! + `Button variant="link"`): `<button data-slot="button" data-variant="link">`
//! plus `<div data-slot="hover-card-content">`, inside a slotless `<span>` that
//! only anchors the card. Zero JS: the card has no layout box until the span is
//! hovered or focused (CSS `:hover` / `:focus-within`). React's audit fixture
//! forces `open` and portals the card out of the canvas.
//! Not interact `popover("hover-card")` SURF `<details>` overlay.

use crate::cronus_ui_kit::{label_of, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let ts = texts(comp);
    let trigger = ts.first().cloned().unwrap_or_else(|| label_of(comp));
    let body = ts.iter().skip(1).cloned().collect::<Vec<_>>().join("");
    let body = if body.is_empty() {
        trigger.clone()
    } else {
        body
    };
    format!(
        "<span><button type=\"button\" data-slot=\"button\" data-variant=\"link\">{trigger}</button><div data-slot=\"hover-card-content\">{body}</div></span>"
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
    fn trigger_is_link_button_slot_like_react_as_child() {
        let html = render(&with_body("@cronus", "Cronus UI"));
        assert_eq!(
            html,
            "<span><button type=\"button\" data-slot=\"button\" data-variant=\"link\">@cronus</button><div data-slot=\"hover-card-content\">Cronus UI</div></span>"
        );
        assert!(!html.contains("data-slot=\"hover-card\""));
        assert!(!html.contains("hover-card-trigger"));
        reject_interact(&html);
    }

    #[test]
    fn content_slot_is_the_contract() {
        let html = render(&stub("hover-card", "Preview"));
        assert!(html.contains("data-slot=\"hover-card-content\">Preview</div>"));
        assert!(html.contains(">Preview</button>"));
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
        assert!(css.contains("[data-slot=\"hover-card-content\"] {\n  display: none;"));
        assert!(css.contains("span:hover > [data-slot=\"hover-card-content\"]"));
        assert!(css.contains("[data-slot=\"button\"]:has(+ [data-slot=\"hover-card-content\"])"));
        assert!(css.contains("z-index: 50"));
        assert!(css.contains("width: 16rem"));
        assert!(css.contains("padding: 0.75rem"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-shadow-lg"));
        assert!(!css.contains("[data-slot=\"hover-card-trigger\"]"));
        assert!(!css.contains("zinc-"));
    }
}
