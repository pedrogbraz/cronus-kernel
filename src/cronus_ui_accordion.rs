//! Dedicated Accordion renderer — mirrors React `Accordion` (Radix, single +
//! collapsible) and opens/closes items with zero JS.
//!
//! DOM: an unslotted root `<div data-orientation="vertical">` (Radix Root has
//! no `data-slot`) > per item `<div data-slot="accordion-item">` >
//! `<h3>` > `<label>` (visually hidden state `<input>` + the
//! `accordion-trigger` button with the chevron) and
//! `<div data-slot="accordion-content" role="region">` > `<div>` body.
//! Items are `trigger, body` pairs; `value:"…"` opens the item whose trigger
//! matches, else the first item is open (React fixture `defaultValue`).
//!
//! State is native. Single mode (default, React `type="single" collapsible`):
//! the item inputs are radios sharing a page-unique `name`, so opening one
//! closes the others. A trailing visually hidden "Collapse all" radio in the
//! same group makes the open item collapsible: an unslotted `<label for>` over
//! the open item's header points at it, so clicking the open trigger closes it.
//! `type:"multiple"` switches the inputs to independent checkboxes.
//! `COMPONENT_CHROME` shows the content of the item whose input is checked and
//! rotates its chevron. The trigger keeps React's `<button>` slot but is
//! `aria-hidden`, `tabindex="-1"` and `pointer-events: none`, so clicks land on
//! the label and focus goes to the input.
//!
//! Gaps vs Radix (no JS): the control is exposed as a radio/checkbox instead of
//! a button with `aria-expanded`; no `data-state`; closed content is
//! `display: none` with its body present; in single mode Arrow keys move *and*
//! open (radio group roving, wrapping through "Collapse all"), Space/Enter on
//! the open item does not close it, Home/End are not handled.

use crate::cronus_ui_kit::{attr, content_texts, esc, instance_id, label_of};
use crate::parser::ComponentNode;

/// `(trigger, body)`: `item` lines are triggers whose body is their
/// `description:"…"` (the docs' answer text); otherwise the `text` lines pair
/// up (audit fixture); with neither, the label alone.
fn items_of(comp: &ComponentNode) -> Vec<(String, String)> {
    let listed: Vec<(String, String)> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty())
        .map(|i| {
            (
                esc(&i.text),
                i.config
                    .get("description")
                    .filter(|d| !d.trim().is_empty())
                    .map(|d| esc(d))
                    .unwrap_or_default(),
            )
        })
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
    let multiple = attr(comp, "type") == Some("multiple");
    let id = instance_id(comp, "accordion");
    let body = items
        .iter()
        .enumerate()
        .map(|(i, (trigger, content))| {
            let checked = if i == open { " checked" } else { "" };
            let (input, collapse) = if multiple {
                ("type=\"checkbox\"".to_string(), String::new())
            } else {
                (
                    format!("type=\"radio\" name=\"{id}\""),
                    format!("<label for=\"{id}-none\" aria-hidden=\"true\"></label>"),
                )
            };
            format!(
                "<div data-slot=\"accordion-item\" data-orientation=\"vertical\"><h3 data-orientation=\"vertical\"><label><input {input} id=\"{id}-i{i}\" aria-label=\"{trigger}\" aria-controls=\"{id}-c{i}\"{checked}><button type=\"button\" data-slot=\"accordion-trigger\" tabindex=\"-1\" aria-hidden=\"true\">{trigger}{CHEVRON}</button></label>{collapse}</h3><div data-slot=\"accordion-content\" id=\"{id}-c{i}\" role=\"region\" aria-labelledby=\"{id}-i{i}\"><div>{content}</div></div></div>"
            )
        })
        .collect::<String>();
    let none = if multiple {
        String::new()
    } else {
        format!("<input type=\"radio\" name=\"{id}\" id=\"{id}-none\" aria-label=\"Collapse all\">")
    };
    format!("<div data-orientation=\"vertical\">{body}{none}</div>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::{reset_instance_ids, stub, widget_id};
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
            "onclick",
            "<script",
            "style=",
            "v-data",
            "<details",
            "<summary",
            " disabled",
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
            "<h3 data-orientation=\"vertical\"><label><input type=\"radio\" name=\"{id}\" id=\"{id}-i0\" aria-label=\"Is it accessible?\" aria-controls=\"{id}-c0\" checked><button type=\"button\" data-slot=\"accordion-trigger\" tabindex=\"-1\" aria-hidden=\"true\">Is it accessible?<svg"
        )));
        assert!(html.contains(&format!(
            "<div data-slot=\"accordion-content\" id=\"{id}-c1\" role=\"region\" aria-labelledby=\"{id}-i1\"><div>Tokens.</div></div>"
        )));
        assert_eq!(html.matches(" checked").count(), 1);
        assert!(!html.contains(">default<"));
        reject_js(&html);
    }

    /// Single + collapsible: one radio group (exclusive), a "Collapse all"
    /// radio in the same group and a collapse label per header.
    #[test]
    fn single_mode_is_one_exclusive_collapsible_radio_group() {
        let html = render(&fixture());
        let id = widget_id(&fixture(), "accordion");
        assert_eq!(html.matches(&format!("name=\"{id}\"")).count(), 3);
        assert!(html.ends_with(&format!(
            "<input type=\"radio\" name=\"{id}\" id=\"{id}-none\" aria-label=\"Collapse all\"></div>"
        )));
        assert_eq!(
            html.matches(&format!(
                "</label><label for=\"{id}-none\" aria-hidden=\"true\"></label></h3>"
            ))
            .count(),
            2
        );
        assert!(!html.contains("type=\"checkbox\""));
    }

    #[test]
    fn multiple_mode_uses_independent_checkboxes() {
        let mut c = fixture();
        c.props.insert("type".into(), "multiple".into());
        let html = render(&c);
        assert_eq!(html.matches("type=\"checkbox\"").count(), 2);
        assert!(!html.contains("type=\"radio\""));
        assert!(!html.contains("Collapse all"));
        assert!(!html.contains("<label for="));
        reject_js(&html);
    }

    #[test]
    fn two_accordions_on_one_page_do_not_share_a_group() {
        reset_instance_ids();
        let a = render(&fixture());
        let b = render(&fixture());
        assert!(a.contains("name=\"cui-accordion-accordion\" "));
        assert!(b.contains("name=\"cui-accordion-accordion-2\" "));
        assert!(!b.contains("name=\"cui-accordion-accordion\" "));
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
    fn item_descriptions_are_the_bodies() {
        let mut c = stub("accordion", "FAQ");
        let mut q = text("Is it accessible?");
        q.item_type = "item".into();
        q.config.insert(
            "description".into(),
            "Yes. It follows the WAI-ARIA disclosure pattern.".into(),
        );
        c.items.push(q);
        let html = render(&c);
        assert!(
            html.contains("<div>Yes. It follows the WAI-ARIA disclosure pattern.</div></div>"),
            "{html}"
        );
        assert!(html.contains("aria-label=\"Is it accessible?\""));
    }

    #[test]
    fn chrome_opens_checked_item() {
        let css = crate::cronus_ui::component_chrome_css();
        // Height animates like React's cronus-accordion-down/up (grid rows 0fr -> 1fr).
        assert!(css.contains(
            "[data-slot=\"accordion-content\"] {\n  display: grid; grid-template-rows: 0fr;"
        ));
        assert!(css.contains("transition: grid-template-rows 220ms var(--ease-out-quart);"));
        assert!(css.contains(
            "[data-slot=\"accordion-item\"]:has(> h3 > label > input:checked) > [data-slot=\"accordion-content\"] {\n  grid-template-rows: 1fr;"
        ));
        assert!(css.contains(
            "[data-slot=\"accordion-content\"] > div {\n  min-height: 0; overflow: hidden;"
        ));
        assert!(
            css.contains("label:has(> input:checked) > [data-slot=\"accordion-trigger\"] > svg")
        );
        assert!(!css.contains("[data-slot=\"accordion\"] {"));
    }

    /// The collapse label covers only the open item's header; the group's
    /// "Collapse all" radio is visually hidden.
    #[test]
    fn chrome_collapse_label_covers_open_header() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"accordion-item\"] > h3 > label[for] {\n  position: absolute; inset: 0; display: none;"));
        assert!(css.contains(
            "[data-slot=\"accordion-item\"] > h3:has(> label > input:checked) > label[for] { display: block; }"
        ));
        assert!(css.contains("div:has(> [data-slot=\"accordion-item\"]) > input {"));
    }
}
