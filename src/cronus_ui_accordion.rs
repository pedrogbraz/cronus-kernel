//! Dedicated Accordion renderer — mirrors React `Accordion` (Radix, single +
//! collapsible) and opens/closes items with zero JS.
//!
//! DOM: an unslotted root `<div data-orientation="vertical">` (Radix Root has
//! no `data-slot`) > per item `<div data-slot="accordion-item">` >
//! `<h3>` > `<label>` (visually hidden `<input type="checkbox">` + the
//! `accordion-trigger` button with the chevron) and
//! `<div data-slot="accordion-content" role="region">` > `<div>` body.
//! Items are `trigger, body` pairs; `value:"…"` opens the item whose trigger
//! matches, else the first item is open (React fixture `defaultValue`).
//!
//! The checkbox is the state: `COMPONENT_CHROME` shows the content of an item
//! whose checkbox is checked and rotates its chevron. The trigger keeps React's
//! `<button>` slot but is `disabled`, `aria-hidden` and `pointer-events: none`,
//! so clicks land on the label and focus goes to the checkbox (Space toggles).
//!
//! Gaps vs Radix (no JS): items open independently (checkbox), so opening one
//! does not close the others (`type="single"`); the control is exposed as a
//! checkbox instead of a button with `aria-expanded`; no `data-state`; closed
//! content is `display: none` with its body present; no Arrow/Home/End roving.

use crate::cronus_ui_kit::{attr, content_texts, esc, label_of, widget_id};
use crate::parser::ComponentNode;

/// `(trigger, body)`: `item` lines are triggers without a body; otherwise the
/// `text` lines pair up (audit fixture); with neither, the label alone.
fn items_of(comp: &ComponentNode) -> Vec<(String, String)> {
    let listed: Vec<(String, String)> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty())
        .map(|i| (esc(&i.text), String::new()))
        .collect();
    if !listed.is_empty() {
        return listed;
    }
    let texts = content_texts(comp);
    if texts.is_empty() {
        return vec![(label_of(comp), String::new())];
    }
    texts
        .chunks(2)
        .map(|c| (c[0].clone(), c.get(1).cloned().unwrap_or_default()))
        .collect()
}

/// lucide `chevron-down` (React `size-4 shrink-0 text-fg-tertiary`).
const CHEVRON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m6 9 6 6 6-6\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let items = items_of(comp);
    let open = attr(comp, "value")
        .map(esc)
        .and_then(|v| items.iter().position(|(t, _)| *t == v))
        .unwrap_or(0);
    let id = widget_id(comp, "accordion");
    let body = items
        .iter()
        .enumerate()
        .map(|(i, (trigger, content))| {
            let checked = if i == open { " checked" } else { "" };
            format!(
                "<div data-slot=\"accordion-item\" data-orientation=\"vertical\"><h3 data-orientation=\"vertical\"><label><input type=\"checkbox\" id=\"{id}-i{i}\" aria-label=\"{trigger}\" aria-controls=\"{id}-c{i}\"{checked}><button type=\"button\" data-slot=\"accordion-trigger\" tabindex=\"-1\" aria-hidden=\"true\" disabled>{trigger}{CHEVRON}</button></label></h3><div data-slot=\"accordion-content\" id=\"{id}-c{i}\" role=\"region\" aria-labelledby=\"{id}-i{i}\"><div>{content}</div></div></div>"
            )
        })
        .collect::<String>();
    format!("<div data-orientation=\"vertical\">{body}</div>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn text(t: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "text".into(),
            text: t.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn fixture() -> ComponentNode {
        let mut c = stub("accordion", "default");
        for t in ["Is it accessible?", "Yes.", "Is it themeable?", "Tokens."] {
            c.items.push(text(t));
        }
        c
    }

    fn reject_js(html: &str) {
        for bad in [
            "onclick", "<script", "style=", "v-data", "<details", "<summary",
        ] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    #[test]
    fn fixture_matches_react_slots() {
        let html = render(&fixture());
        let id = widget_id(&fixture(), "accordion");
        assert!(html
            .starts_with("<div data-orientation=\"vertical\"><div data-slot=\"accordion-item\""));
        assert!(!html.contains("data-slot=\"accordion\""));
        assert_eq!(html.matches("data-slot=\"accordion-item\"").count(), 2);
        assert!(html.contains(&format!(
            "<h3 data-orientation=\"vertical\"><label><input type=\"checkbox\" id=\"{id}-i0\" aria-label=\"Is it accessible?\" aria-controls=\"{id}-c0\" checked><button type=\"button\" data-slot=\"accordion-trigger\" tabindex=\"-1\" aria-hidden=\"true\" disabled>Is it accessible?<svg"
        )));
        assert!(html.contains(&format!(
            "<div data-slot=\"accordion-content\" id=\"{id}-c1\" role=\"region\" aria-labelledby=\"{id}-i1\"><div>Tokens.</div></div>"
        )));
        assert_eq!(html.matches(" checked").count(), 1);
        assert!(!html.contains(">default<"));
        reject_js(&html);
    }

    #[test]
    fn value_opens_matching_item() {
        let mut c = fixture();
        c.props.insert("value".into(), "Is it themeable?".into());
        let html = render(&c);
        assert_eq!(html.matches(" checked").count(), 1);
        assert!(html.contains("aria-controls=\"cui-accordion-accordion-c1\" checked>"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || reject_js(&render(&fixture())));
    }

    #[test]
    fn chrome_opens_checked_item() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"accordion-content\"] {\n  display: none;"));
        assert!(css.contains(
            "[data-slot=\"accordion-item\"]:has(> h3 > label > input:checked) > [data-slot=\"accordion-content\"] {\n  display: block;"
        ));
        assert!(
            css.contains("label:has(> input:checked) > [data-slot=\"accordion-trigger\"] > svg")
        );
        assert!(!css.contains("[data-slot=\"accordion\"] {"));
    }
}
