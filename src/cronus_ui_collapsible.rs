//! Dedicated Collapsible renderer. DOM matches React/Radix open state:
//! unslotted root (Radix Root carries no data-slot) > unslotted trigger
//! `<button data-state="open">` (label) >
//! `<div data-slot="collapsible-content" data-state="open">` (body texts).
//!
//! Zero JS: the root is `<details open>` and the trigger button sits in its
//! `<summary>`, so the summary toggles the content natively (click, Enter,
//! Space) and exposes the expanded state. The button keeps React's element and
//! look but is decorative (`aria-hidden`, `tabindex="-1"`,
//! `pointer-events: none`). Starts open like the fixture's `defaultOpen`.
//! Gaps vs Radix: `data-state` stays `open` after toggling (CSS hides closed
//! content via `<details>`), the focusable element is the `<summary>`, not the
//! `<button>`. Not interact `accordion("collapsible")` (SURF, no
//! collapsible-content).

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
        "<details open data-state=\"open\"><summary><button type=\"button\" data-state=\"open\" tabindex=\"-1\" aria-hidden=\"true\">{label}</button></summary><div data-state=\"open\" data-slot=\"collapsible-content\">{body}</div></details>"
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
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains(" disabled"));
        assert!(!crate::cli::stub_renderer_gate::looks_like_interact_generic(html));
    }

    /// wave1t: Radix Root has no `data-slot`; an extra kernel slot would be
    /// an unpaired geometry node. The native disclosure is `<details open>`.
    #[test]
    fn root_is_unslotted_native_disclosure_open_by_default() {
        let mut c = stub("collapsible", "Toggle");
        c.items.push(extra("Hidden body"));
        let html = render(&c);
        assert_eq!(
            html,
            "<details open data-state=\"open\"><summary><button type=\"button\" data-state=\"open\" tabindex=\"-1\" aria-hidden=\"true\">Toggle</button></summary><div data-state=\"open\" data-slot=\"collapsible-content\">Hidden body</div></details>"
        );
        assert!(!html.contains("data-slot=\"collapsible\""));
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_opens_empty_content() {
        let html = render(&stub("collapsible", "Show more"));
        assert!(html.contains(">Show more</button></summary>"));
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

    /// The summary lays out like React's root > button (block line box, no
    /// marker) and hands clicks through the decorative button.
    #[test]
    fn chrome_summary_is_markerless_block_trigger() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("details:has(> [data-slot=\"collapsible-content\"]) > summary {\n  display: block; list-style: none; cursor: pointer;"));
        assert!(css.contains("details:has(> [data-slot=\"collapsible-content\"]) > summary::-webkit-details-marker { display: none; }"));
        assert!(css.contains("details:has(> [data-slot=\"collapsible-content\"]) > summary > button { pointer-events: none; }"));
    }
}
