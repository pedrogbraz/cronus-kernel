//! Dedicated MorphingPopover renderer. DOM mirrors React (open state; the
//! content is NOT portaled — it is absolutely positioned inside the root):
//! `<div data-slot="morphing-popover" data-state="open">` >
//! `<button data-slot="morphing-popover-trigger" aria-hidden="true">` (label in a
//! `<span>`) + `<div data-slot="morphing-popover-content" role="dialog">` >
//! reveal `<div>` > body. Zero JS: static open state (no morph, no dismiss); the
//! trigger (inert while open in React) is the native button rendered `disabled`.
//! Not interact `popover("morphing-popover")` SURF `<details>` overlay.

use crate::cronus_ui_kit::{attr_nonempty, esc, label_of, texts};
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
    let aria = attr_nonempty(comp, "aria-label")
        .map(|v| esc(v))
        .unwrap_or_else(|| "Details".into());
    format!(
        "<div data-slot=\"morphing-popover\" data-state=\"open\"><button type=\"button\" data-slot=\"morphing-popover-trigger\" data-state=\"open\" aria-haspopup=\"dialog\" aria-expanded=\"true\" aria-hidden=\"true\" tabindex=\"-1\" disabled><span>{trigger}</span></button><div data-slot=\"morphing-popover-content\" data-state=\"open\" role=\"dialog\" aria-modal=\"false\" aria-label=\"{aria}\" tabindex=\"-1\"><div>{body}</div></div></div>"
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
        assert!(!html.contains("position:absolute;z-index:20"));
        assert!(!html.contains("max-width:28rem"));
        assert!(!html.contains("popover("));
    }

    fn with_body(label: &str, body: &str) -> crate::parser::ComponentNode {
        let mut c = stub("morphing-popover", label);
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
    fn open_state_dom_matches_react() {
        let mut c = with_body("Open", "Popover body");
        c.props.insert("aria-label".into(), "Details".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"morphing-popover\" data-state=\"open\"><button type=\"button\" data-slot=\"morphing-popover-trigger\" data-state=\"open\" aria-haspopup=\"dialog\" aria-expanded=\"true\" aria-hidden=\"true\" tabindex=\"-1\" disabled><span>Open</span></button><div data-slot=\"morphing-popover-content\" data-state=\"open\" role=\"dialog\" aria-modal=\"false\" aria-label=\"Details\" tabindex=\"-1\"><div>Popover body</div></div></div>"
        );
        reject_interact(&html);
    }

    #[test]
    fn content_slot_is_the_contract() {
        let html = render(&stub("morphing-popover", "Open menu"));
        assert!(html.contains("data-slot=\"morphing-popover-trigger\""));
        assert!(html.contains("<span>Open menu</span></button>"));
        assert!(html.contains("aria-label=\"Details\""));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_surf_details() {
        let html = render(&with_body("Open menu", "First action"));
        let interact = crate::cronus_ui_interact::render(
            "morphing-popover",
            &with_body("Open menu", "First action"),
        )
        .unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<details data-slot=\"morphing-popover\""));
        assert!(interact.contains("style="));
        assert!(!interact.contains("data-slot=\"morphing-popover-trigger\""));
        assert!(!html.contains("<details"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("morphing-popover", "Open menu"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"morphing-popover-content\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"morphing-popover\"] {\n  position: relative; isolation: isolate;"
        ));
        assert!(css.contains("[data-slot=\"morphing-popover-trigger\"]"));
        assert!(css.contains(
            "[data-slot=\"morphing-popover-content\"] {\n  position: absolute; z-index: 50;"
        ));
        assert!(css.contains("border-radius: 16px"));
        assert!(css.contains("width: 18rem"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-shadow-lg"));
        assert!(!css.contains("zinc-"));
    }
}
