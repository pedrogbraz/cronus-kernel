//! Dedicated Popover renderer. DOM mirrors React (`PopoverTrigger asChild` +
//! `Button`): the trigger is `<button data-slot="button" data-variant="primary">`
//! (Radix Slot keeps the Button's own `data-slot`), followed by a native
//! `popover="auto"` `<div data-slot="popover-content" role="dialog">`.
//! Zero JS: the content is closed until the trigger's `popovertarget` opens it
//! (React's audit fixture forces `defaultOpen` and portals the open content out
//! of the canvas; the kernel cannot open a top-layer popover without JS).
//! Not interact `popover("popover")` SURF `<details>` overlay.

use crate::cronus_ui_kit::{attr_nonempty, esc, label_of, texts, widget_id};
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
    let aria = aria_label_of(comp).unwrap_or_else(|| "Details".into());
    let trigger_id = widget_id(comp, "trigger");
    let pop_id = widget_id(comp, "pop");
    format!(
        "<button type=\"button\" id=\"{trigger_id}\" data-slot=\"button\" data-variant=\"primary\" popovertarget=\"{pop_id}\" aria-haspopup=\"dialog\">{trigger}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"popover-content\" role=\"dialog\" aria-label=\"{aria}\" anchor=\"{trigger_id}\">{body}</div>"
    )
}

fn aria_label_of(comp: &ComponentNode) -> Option<String> {
    attr_nonempty(comp, "aria-label").map(|v| esc(v))
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
    fn trigger_is_primary_button_slot_like_react_as_child() {
        let mut c = with_body("Open", "Popover body");
        c.props.insert("aria-label".into(), "Details".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<button type=\"button\" id=\"cui-popover-trigger\" data-slot=\"button\" data-variant=\"primary\" popovertarget=\"cui-popover-pop\" aria-haspopup=\"dialog\">Open</button><div id=\"cui-popover-pop\" popover=\"auto\" data-slot=\"popover-content\" role=\"dialog\" aria-label=\"Details\" anchor=\"cui-popover-trigger\">Popover body</div>"
        );
        assert!(!html.contains("data-slot=\"popover-trigger\""));
        reject_interact(&html);
    }

    #[test]
    fn content_slot_is_the_contract() {
        let html = render(&stub("popover", "More"));
        assert!(html.contains("data-slot=\"popover-content\""));
        assert!(html.contains(">More</button>"));
        assert!(html.contains("aria-label=\"Details\""));
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
        assert!(css.contains("[data-slot=\"popover-content\"]:popover-open {\n  display: block; z-index: 50; width: 18rem;"));
        assert!(css.contains("[data-slot=\"button\"]:has(+ [data-slot=\"popover-content\"])"));
        assert!(css.contains("padding: 0.75rem"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-shadow-lg"));
        assert!(!css.contains("button:has(+ [data-slot=\"popover-content\"])"));
        assert!(!css.contains("zinc-"));
    }
}
