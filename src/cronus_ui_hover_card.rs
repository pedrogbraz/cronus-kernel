//! Dedicated HoverCard renderer. DOM mirrors React (`HoverCardTrigger asChild`
//! + `Button variant="link"`): `<button data-slot="button" data-variant="link">`
//! plus `<div data-slot="hover-card-content">`, inside a slotless `<span>` that
//! only anchors the card. Zero JS: the card is a native `popover="auto"` with no
//! layout box until opened. The trigger's `interestfor` opens it on hover /
//! focus (interest invokers, Chromium) and its `popovertarget` is the portable
//! fallback: click / Enter / Space toggle it, Esc and an outside click dismiss
//! it. Gap: browsers without interest invokers only open it on activation, not
//! on hover. React's audit fixture forces `open` and portals the card out of
//! the canvas.
//! Not interact `popover("hover-card")` SURF `<details>` overlay.

use crate::cronus_ui_kit::{label_of, texts, widget_id};
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
    let trigger_id = widget_id(comp, "link");
    let pop_id = widget_id(comp, "card");
    format!(
        "<span><button type=\"button\" id=\"{trigger_id}\" data-slot=\"button\" data-variant=\"link\" interestfor=\"{pop_id}\" popovertarget=\"{pop_id}\">{trigger}</button><div id=\"{pop_id}\" popover=\"auto\" anchor=\"{trigger_id}\" data-slot=\"hover-card-content\">{body}</div></span>"
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
        let c = with_body("@cronus", "Cronus UI");
        let html = render(&c);
        let tid = crate::cronus_ui_kit::widget_id(&c, "link");
        let pid = crate::cronus_ui_kit::widget_id(&c, "card");
        // interestfor: hover/focus opens (Chromium); popovertarget: click/Enter fallback.
        assert_eq!(
            html,
            format!(
                "<span><button type=\"button\" id=\"{tid}\" data-slot=\"button\" data-variant=\"link\" interestfor=\"{pid}\" popovertarget=\"{pid}\">@cronus</button><div id=\"{pid}\" popover=\"auto\" anchor=\"{tid}\" data-slot=\"hover-card-content\">Cronus UI</div></span>"
            )
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
        assert!(!html.contains("<details"));
        reject_interact(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"hover-card-content\"] {\n  display: none;"));
        // The CSS :hover reveal is gone: it double-rendered next to the native popover.
        assert!(!css.contains(
            "span:hover > [data-slot=\"hover-card-content\"],\nspan:focus-within > [data-slot=\"hover-card-content\"]"
        ));
        assert!(css.contains(
            "[data-slot=\"hover-card-content\"]:popover-open {\n  display: block; margin: 0; transform: none;"
        ));
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
