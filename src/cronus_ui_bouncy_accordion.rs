//! Dedicated BouncyAccordion renderer. DOM matches React idle:
//! `<div data-slot="bouncy-accordion">` > hint `<p>` (React default label
//! "Click on items to expand & collapse") > `<ul>` of `<li>` rows, each with a
//! `button data-slot="bouncy-accordion-trigger"` holding the 45px title row and
//! the description span. Closed rows are clipped to 45px; the open row (first
//! item, React's `defaultValue` fallback) grows to auto height with a 10px
//! block margin. Stack radii (20px on group ends) are pure CSS sibling rules.
//! Springs and click-to-toggle need JS: triggers keep React's `<button>` but
//! are `disabled` (not dimmed, React does not dim them at idle).
//! Not interact `accordion()` SURF `<details>` (no trigger slot).

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

/// React BouncyAccordion `DEFAULT_LABELS.hint`.
const HINT: &str = "Click on items to expand &amp; collapse";

pub fn render(comp: &ComponentNode) -> String {
    let mut items: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type != "label" && i.item_type != "title" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if items.is_empty() {
        items.push(label_of(comp));
    }
    let rows = items
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let (state, expanded) = if i == 0 { ("open", "true") } else { ("closed", "false") };
            format!(
                "<li data-state=\"{state}\"><button type=\"button\" data-slot=\"bouncy-accordion-trigger\" aria-expanded=\"{expanded}\" disabled><span><span>{t}</span></span><span>{t}</span></button></li>"
            )
        })
        .collect::<String>();
    format!("<div data-slot=\"bouncy-accordion\"><p>{HINT}</p><ul>{rows}</ul></div>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
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
        assert!(!html.contains("zinc-"));
    }

    /// wave1t: fixture emits `label "default"` (fixture id) + texts; the label
    /// is not an item, and there is no content slot.
    #[test]
    fn rows_from_texts_first_open_matches_react_dom() {
        let mut c = stub("bouncy-accordion", "default");
        c.items.push(extra("text", "Type"));
        c.items.push(extra("text", "Schedule"));
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"bouncy-accordion\"><p>Click on items to expand &amp; collapse</p><ul><li data-state=\"open\"><button type=\"button\" data-slot=\"bouncy-accordion-trigger\" aria-expanded=\"true\" disabled><span><span>Type</span></span><span>Type</span></button></li><li data-state=\"closed\"><button type=\"button\" data-slot=\"bouncy-accordion-trigger\" aria-expanded=\"false\" disabled><span><span>Schedule</span></span><span>Schedule</span></button></li></ul></div>"
        );
        assert!(!html.contains("default"));
        assert!(!html.contains("bouncy-accordion-content"));
        reject_interact(&html);
    }

    #[test]
    fn label_only_opens_single_row() {
        let html = render(&stub("bouncy-accordion", "Type"));
        assert_eq!(html.matches("data-slot=\"bouncy-accordion-trigger\"").count(), 1);
        assert!(html.contains("<li data-state=\"open\">"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_accordion_surf_details() {
        let mut c = stub("bouncy-accordion", "Hint");
        c.items.push(extra("item", "Type"));
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("accordion", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<details"));
        assert!(!interact.contains("data-slot=\"bouncy-accordion-trigger\""));
        reject_interact(&html);
        assert!(crate::cronus_ui_interact::render("bouncy-accordion", &c).is_none());
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("bouncy-accordion", "Type"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_matches_react_stack_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"bouncy-accordion\"] > ul > li {\n  position: relative; overflow: hidden; height: 45px;"));
        assert!(css.contains("[data-slot=\"bouncy-accordion\"] > ul > li[data-state=\"open\"] {\n  height: auto; margin-block: 10px;"));
        assert!(css.contains("margin: 0 0 5rem; max-width: 18ch;"));
        assert!(css.contains("font-size: 0.75rem; line-height: 1.25; text-transform: uppercase;"));
        assert!(css.contains("[data-slot=\"bouncy-accordion\"] > ul > li:has(+ li[data-state=\"open\"])"));
        assert!(css.contains(
            "[data-slot=\"bouncy-accordion-trigger\"] {\n  display: flex; flex-direction: column; width: 100%; padding: 0 0.5rem;"
        ));
        assert!(!css.contains("[data-slot=\"bouncy-accordion-content\"]"));
        assert!(!css.contains("zinc-"));
    }
}
