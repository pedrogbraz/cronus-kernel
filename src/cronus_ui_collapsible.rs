//! Dedicated Collapsible renderer. DOM matches React/Radix open state:
//! `<div data-state="open">` (Radix Root carries no data-slot) > trigger
//! `<button aria-expanded="true" data-state="open">` (label) >
//! `<div data-slot="collapsible-content" data-state="open">` (body texts).
//! Toggling needs JS, so the kernel renders the fixture's `defaultOpen` state
//! and the trigger keeps React's `<button>` but is `disabled` (not dimmed).
//! Not interact `accordion("collapsible")` (`<details>` SURF, no collapsible-content).

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let body = comp
        .items
        .iter()
        .filter(|i| i.item_type != "label" && i.item_type != "title" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-state=\"open\"><button type=\"button\" aria-expanded=\"true\" data-state=\"open\" disabled>{label}</button><div data-state=\"open\" data-slot=\"collapsible-content\">{body}</div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn extra(text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "text".into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<details"));
        assert!(!html.contains("<summary"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
    }

    /// wave1t: Radix Root has no `data-slot`; an extra kernel slot would be
    /// an unpaired geometry node.
    #[test]
    fn root_is_unslotted_radix_open_state() {
        let mut c = stub("collapsible", "Toggle");
        c.items.push(extra("Hidden body"));
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-state=\"open\"><button type=\"button\" aria-expanded=\"true\" data-state=\"open\" disabled>Toggle</button><div data-state=\"open\" data-slot=\"collapsible-content\">Hidden body</div></div>"
        );
        assert!(!html.contains("data-slot=\"collapsible\""));
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_opens_empty_content() {
        let html = render(&stub("collapsible", "Show more"));
        assert!(html.contains(">Show more</button>"));
        assert!(html.contains("data-slot=\"collapsible-content\"></div>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_accordion_surf() {
        let mut c = stub("collapsible", "Show more");
        c.items.push(extra("Hidden details here."));
        let html = render(&c);
        assert!(html.contains("data-slot=\"collapsible-content\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("collapsible", "Show more"));
            reject_interact(&html);
        });
    }

    /// wave1t: `text-sm` is 14px/20px (not the canvas 1.5 line-height).
    #[test]
    fn chrome_content_is_text_sm_pair() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"collapsible-content\"] {\n  overflow: hidden; font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg-secondary);\n}"
        ));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
