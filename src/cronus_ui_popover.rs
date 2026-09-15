//! Dedicated Popover renderer. Always-open static DOM (no hover/JS).
//! Trigger `<button type="button">` from label plus
//! `<div data-slot="popover-content">` from extra text.
//! React Root emits no data-slot; content slot is the contract.
//! Not interact `popover("popover")` SURF `<details>` overlay.

use crate::cronus_ui_kit::{label_of, texts, widget_id};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let ts = texts(comp);
    let trigger = ts.first().cloned().unwrap_or_else(|| label_of(comp));
    let body = ts.iter().skip(1).cloned().collect::<Vec<_>>().join("");
    let trigger_id = widget_id(comp, "trigger");
    let pop_id = widget_id(comp, "pop");
    format!(
        "<button type=\"button\" id=\"{trigger_id}\" popovertarget=\"{pop_id}\">{trigger}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"popover-content\" anchor=\"{trigger_id}\">{body}</div>"
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
        assert!(!html.contains("data-slot=\"popover\">"));
    }

    fn with_body(label: &str, body: &str) -> crate::parser::ComponentNode {
        let mut c = stub("popover", label);
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
        let html = render(&with_body("More", "Extra actions."));
        assert!(html.contains("<button type=\"button\""));
        assert!(html.contains("popovertarget="));
        assert!(html.contains(">More</button>"));
        assert!(html.contains("data-slot=\"popover-content\""));
        assert!(html.contains("popover=\"auto\""));
        assert!(html.contains(">Extra actions.</div>"));
        reject_interact(&html);
    }

    #[test]
    fn content_slot_is_the_contract() {
        let html = render(&stub("popover", "More"));
        assert!(html.contains("data-slot=\"popover-content\""));
        assert!(html.contains(">More</button>"));
        assert!(!html.contains("data-slot=\"popover\">"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_surf_details() {
        let html = render(&with_body("More", "Extra actions."));
        let interact =
            crate::cronus_ui_interact::render("popover", &with_body("More", "Extra actions."))
                .unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<details data-slot=\"popover\""));
        assert!(interact.contains("style="));
        assert!(!interact.contains("data-slot=\"popover-content\""));
        assert!(!html.contains("<details"));
        reject_interact(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"popover-content\"]"));
        assert!(css.contains("z-index: 50"));
        assert!(css.contains("width: 18rem"));
        assert!(css.contains("padding: 0.75rem"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-shadow-lg"));
        assert!(!css.contains("zinc-"));
    }
}
