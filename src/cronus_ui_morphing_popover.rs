//! Dedicated MorphingPopover renderer. Always-open static DOM (no hover/JS).
//! Trigger `data-slot="morphing-popover-trigger"` from label plus
//! `data-slot="morphing-popover-content"` from extra text. Wrapper
//! `data-slot="morphing-popover"` for the family slot. Not interact
//! `popover("morphing-popover")` SURF `<details>` overlay.

use crate::cronus_ui_kit::{label_of, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let ts = texts(comp);
    let trigger = ts.first().cloned().unwrap_or_else(|| label_of(comp));
    let body = ts.iter().skip(1).cloned().collect::<Vec<_>>().join("");
    format!(
        "<div data-slot=\"morphing-popover\" data-state=\"open\"><button type=\"button\" data-slot=\"morphing-popover-trigger\" aria-haspopup=\"dialog\" aria-expanded=\"true\">{trigger}</button><div data-slot=\"morphing-popover-content\" data-state=\"open\" role=\"dialog\" aria-modal=\"false\">{body}</div></div>"
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
    fn trigger_button_and_always_open_content() {
        let html = render(&with_body("Open menu", "First action"));
        assert!(html.starts_with("<div data-slot=\"morphing-popover\""));
        assert!(html.contains("data-slot=\"morphing-popover-trigger\""));
        assert!(html.contains("<button type=\"button\""));
        assert!(html.contains(">Open menu</button>"));
        assert!(html.contains("data-slot=\"morphing-popover-content\""));
        assert!(html.contains("role=\"dialog\""));
        assert!(html.contains("aria-modal=\"false\""));
        assert!(html.contains(">First action</div>"));
        assert!(html.contains("data-state=\"open\""));
        reject_interact(&html);
    }

    #[test]
    fn content_slot_is_the_contract() {
        let html = render(&stub("morphing-popover", "Open menu"));
        assert!(html.contains("data-slot=\"morphing-popover\""));
        assert!(html.contains("data-slot=\"morphing-popover-trigger\""));
        assert!(html.contains("data-slot=\"morphing-popover-content\""));
        assert!(html.contains(">Open menu</button>"));
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
        assert!(!interact.contains("data-slot=\"morphing-popover-content\""));
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
        assert!(css.contains("[data-slot=\"morphing-popover\"]"));
        assert!(css.contains("[data-slot=\"morphing-popover-trigger\"]"));
        assert!(css.contains("[data-slot=\"morphing-popover-content\"]"));
        assert!(css.contains("z-index: 50"));
        assert!(css.contains("width: 18rem"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-shadow-lg"));
        assert!(!css.contains("zinc-"));
    }
}
