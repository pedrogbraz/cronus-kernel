//! Dedicated Select renderer — mirrors React `SelectTrigger` (Radix) closed,
//! with `SelectValue placeholder`.
//!
//! DOM: `<button data-slot="select-trigger" role="combobox"
//! aria-expanded="false" data-state="closed" data-placeholder>` > `<span>`
//! value text + lucide chevron-down. Radix keeps the closed listbox out of the
//! DOM, so nothing else is rendered.
//!
//! Opening the listbox needs JS, so the trigger is the same native button with
//! `disabled` and React's idle look (not dimmed). Not reproduced: the listbox,
//! typeahead, form value.
//!
//! Text: `value:"…"` (drops `data-placeholder`), else the placeholder = label.

use crate::cronus_ui_kit::{attr_nonempty, esc, label_of};
use crate::parser::ComponentNode;

/// lucide `chevron-down` (React `[&_svg]:size-4 [&_svg]:opacity-60`).
const CHEVRON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m6 9 6 6 6-6\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let placeholder = label_of(comp);
    let aria = attr_nonempty(comp, "aria-label")
        .map(esc)
        .unwrap_or_else(|| placeholder.clone());
    let (text, marker) = match attr_nonempty(comp, "value") {
        Some(v) => (esc(v), ""),
        None => (placeholder, " data-placeholder=\"\""),
    };
    format!(
        "<button type=\"button\" role=\"combobox\" aria-expanded=\"false\" aria-autocomplete=\"none\" aria-label=\"{aria}\" data-state=\"closed\"{marker} data-slot=\"select-trigger\" disabled><span>{text}</span>{CHEVRON}</button>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn with_options() -> ComponentNode {
        let mut c = stub("select", "Select a plan");
        for t in ["Select a plan", "Free", "Pro"] {
            c.items.push(ComponentItemNode {
                item_type: "text".into(),
                text: t.into(),
                link: None,
                tone: None,
                config: Default::default(),
            });
        }
        c
    }

    #[test]
    fn closed_trigger_matches_react() {
        let html = render(&with_options());
        assert_eq!(
            html,
            format!(
                "<button type=\"button\" role=\"combobox\" aria-expanded=\"false\" aria-autocomplete=\"none\" aria-label=\"Select a plan\" data-state=\"closed\" data-placeholder=\"\" data-slot=\"select-trigger\" disabled><span>Select a plan</span>{CHEVRON}</button>"
            )
        );
        for bad in [
            "<select",
            "<option",
            "style=",
            "v-model",
            "select-control",
            "onclick",
        ] {
            assert!(!html.contains(bad), "{bad}");
        }
    }

    #[test]
    fn value_replaces_placeholder() {
        let mut c = with_options();
        c.props.insert("value".into(), "Pro".into());
        let html = render(&c);
        assert!(html.contains("<span>Pro</span>"));
        assert!(!html.contains("data-placeholder"));
    }

    #[test]
    fn chrome_idle_look_when_disabled() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"select-trigger\"] {\n  display: flex;"));
        assert!(css.contains("[data-slot=\"select-trigger\"][data-placeholder] {"));
        assert!(!css.contains("[data-slot=\"select\"] select"));
    }
}
