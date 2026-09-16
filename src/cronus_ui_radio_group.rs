//! Dedicated RadioGroup renderer. DOM matches React/Radix:
//! `<div data-slot="radio-group" role="radiogroup">` plus, per option,
//! `<label>` > visually hidden `<input type="radio">` + React's
//! `<button type="button" data-slot="radio-group-item" role="radio">` with the
//! Radix indicator.
//!
//! Zero JS: the radios share a page-unique `name`, so clicking an option (or
//! Arrow keys) selects it natively; CSS shows the indicator and the primary
//! border on the item after the checked radio (each item carries the
//! indicator; unchecked ones are `display: none`). The button keeps React's
//! slot and look but is decorative (`aria-hidden`, `tabindex="-1"`,
//! `pointer-events: none`). Gaps vs Radix: `data-state` / `aria-checked` stay
//! at the initial state; a form `name`/`value` is the index-free option text.
//! Not interact `radios()` (inline-styled `<input type="radio">` labels).

use crate::cronus_ui_kit::{attr_nonempty, esc, instance_id};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let items = options(comp);
    let name = instance_id(comp, "radio-group");
    let buttons = items
        .iter()
        .map(|(text, checked)| {
            let state = if *checked { "checked" } else { "unchecked" };
            let aria = if *checked { "true" } else { "false" };
            let on = if *checked { " checked" } else { "" };
            format!(
                "<label><input type=\"radio\" name=\"{name}\" value=\"{v}\" aria-label=\"{v}\"{on}><button type=\"button\" role=\"radio\" aria-checked=\"{aria}\" data-state=\"{state}\" value=\"{v}\" data-slot=\"radio-group-item\" aria-label=\"{v}\" tabindex=\"-1\" aria-hidden=\"true\">{CIRCLE_INDICATOR}</button></label>",
                v = esc(text)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let name = group_label(comp);
    format!(
        "<div role=\"radiogroup\" aria-required=\"false\" dir=\"ltr\" data-slot=\"radio-group\" aria-label=\"{name}\">{buttons}</div>"
    )
}

/// Radix Indicator: unslotted span + lucide Circle (size-2, fill primary).
const CIRCLE_INDICATOR: &str = "<span data-state=\"checked\"><svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><circle cx=\"12\" cy=\"12\" r=\"10\"></circle></svg></span>";

/// Label / title name the group; they are never options.
const NAME_KINDS: &[&str] = &["label", "title"];

fn group_label(comp: &ComponentNode) -> String {
    match attr_nonempty(comp, "aria-label") {
        Some(v) => esc(v),
        None => crate::cronus_ui_kit::label_of(comp),
    }
}

fn options(comp: &ComponentNode) -> Vec<(String, bool)> {
    let items: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| !NAME_KINDS.contains(&i.item_type.as_str()) && !i.text.is_empty())
        .collect();
    if items.is_empty() {
        let label = if comp.name.is_empty() {
            "Option"
        } else {
            comp.name.as_str()
        };
        return vec![(label.to_string(), false)];
    }
    // Radix: `value` selects the matching item; without it nothing is checked.
    let selected = attr_nonempty(comp, "value");
    let checked_idx = items
        .iter()
        .position(|i| selected == Some(i.text.as_str()))
        .or_else(|| items.iter().position(|i| is_true(i.config.get("checked"))));
    items
        .iter()
        .enumerate()
        .map(|(idx, i)| (i.text.clone(), Some(idx) == checked_idx))
        .collect()
}

fn is_true(raw: Option<&String>) -> bool {
    matches!(raw.map(String::as_str), Some("true" | "on" | "1"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn stub_options(opts: &[&str]) -> ComponentNode {
        ComponentNode {
            name: "Plan".into(),
            layout: Some("stack".into()),
            style: Some("radio-group".into()),
            items: opts
                .iter()
                .map(|t| ComponentItemNode {
                    item_type: "item".into(),
                    text: (*t).into(),
                    link: None,
                    tone: None,
                    config: HashMap::new(),
                })
                .collect(),
            props: HashMap::new(),
            params: vec![],
            template: None,
            sections: vec![],
            state: vec![],
            tests: vec![],
            binding: None,
            span: Default::default(),
        }
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains(" disabled"));
        assert!(!crate::cli::stub_renderer_gate::looks_like_interact_generic(html));
    }

    #[test]
    fn root_is_radiogroup_of_native_radios_around_react_buttons() {
        let html = render(&stub_options(&["Free", "Pro"]));
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"radio-group\""));
        assert!(html.contains("role=\"radiogroup\""));
        assert_eq!(html.matches("data-slot=\"radio-group-item\"").count(), 2);
        assert_eq!(
            html.matches("<input type=\"radio\" name=\"cui-plan-radio-group\"")
                .count(),
            2
        );
        assert!(html.contains("<button type=\"button\" role=\"radio\""));
        assert!(html.contains("aria-label=\"Free\""));
        assert!(html.contains("aria-label=\"Pro\""));
        reject_interact(&html);
    }

    #[test]
    fn nothing_is_selected_without_value() {
        // Radix RadioGroup without value/defaultValue checks no item.
        let html = render(&stub_options(&["Free", "Pro", "Team"]));
        assert_eq!(html.matches("aria-checked=\"true\"").count(), 0);
        assert_eq!(html.matches("aria-checked=\"false\"").count(), 3);
        assert!(!html.contains(" checked"));
        reject_interact(&html);
    }

    #[test]
    fn two_groups_on_one_page_get_distinct_names() {
        crate::cronus_ui_kit::reset_instance_ids();
        let a = render(&stub_options(&["Free", "Pro"]));
        let b = render(&stub_options(&["Free", "Pro"]));
        assert!(a.contains("name=\"cui-plan-radio-group\" "));
        assert!(b.contains("name=\"cui-plan-radio-group-2\" "));
    }

    /// The checked radio, not the initial `data-state`, shows the indicator.
    #[test]
    fn chrome_indicator_follows_checked_radio() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"radio-group\"] > label > input:not(:checked) + [data-slot=\"radio-group-item\"] > span { display: none; }"));
        assert!(css.contains("[data-slot=\"radio-group\"] > label > input:checked + [data-slot=\"radio-group-item\"] { border-color: var(--cronus-primary); }"));
        assert!(css.contains("[data-slot=\"radio-group\"] > label > input:not(:checked) + [data-slot=\"radio-group-item\"][data-state=\"checked\"] { border-color: var(--cronus-border); }"));
    }

    #[test]
    fn wave1t_default_fixture_text_options_and_value() {
        // app.cronus RadioGroupDefault: label "Plan", text "Free", text "Pro",
        // then value:"Pro" / aria-label:"Plan" attach to the last item's config.
        let mut c = stub_options(&["Free", "Pro"]);
        for i in c.items.iter_mut() {
            i.item_type = "text".into();
        }
        c.items[1].config.insert("value".into(), "Pro".into());
        c.items[1].config.insert("aria-label".into(), "Plan".into());
        c.items.insert(
            0,
            ComponentItemNode {
                item_type: "label".into(),
                text: "Plan".into(),
                link: None,
                tone: None,
                config: HashMap::new(),
            },
        );
        c.name = "RadioGroupDefault".into();
        assert_eq!(
            render(&c),
            format!(
                "<div role=\"radiogroup\" aria-required=\"false\" dir=\"ltr\" data-slot=\"radio-group\" aria-label=\"Plan\"><label><input type=\"radio\" name=\"cui-radiogroupdefault-radio-group\" value=\"Free\" aria-label=\"Free\"><button type=\"button\" role=\"radio\" aria-checked=\"false\" data-state=\"unchecked\" value=\"Free\" data-slot=\"radio-group-item\" aria-label=\"Free\" tabindex=\"-1\" aria-hidden=\"true\">{CIRCLE_INDICATOR}</button></label><label><input type=\"radio\" name=\"cui-radiogroupdefault-radio-group\" value=\"Pro\" aria-label=\"Pro\" checked><button type=\"button\" role=\"radio\" aria-checked=\"true\" data-state=\"checked\" value=\"Pro\" data-slot=\"radio-group-item\" aria-label=\"Pro\" tabindex=\"-1\" aria-hidden=\"true\">{CIRCLE_INDICATOR}</button></label></div>"
            )
        );
        let css = crate::cronus_ui::component_chrome_css();
        assert!(!css.contains("[data-slot=\"radio-group-item\"][data-state=\"checked\"]::after"));
        assert!(css.contains("[data-slot=\"radio-group-item\"] > span > svg {"));
    }

    #[test]
    fn checked_true_item_is_selected() {
        let mut c = stub_options(&["Free", "Pro", "Team"]);
        c.items[1].config.insert("checked".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("aria-checked=\"false\" data-state=\"unchecked\" value=\"Free\""));
        assert!(html.contains("aria-checked=\"true\" data-state=\"checked\" value=\"Pro\""));
        assert!(html.contains("aria-checked=\"false\" data-state=\"unchecked\" value=\"Team\""));
        reject_interact(&html);
    }

    #[test]
    fn options_from_text_items() {
        let html = render(&stub_options(&["A", "B", "C"]));
        assert_eq!(html.matches("role=\"radio\"").count(), 3);
        assert!(html.contains("aria-label=\"A\""));
        assert!(html.contains("aria-label=\"B\""));
        assert!(html.contains("aria-label=\"C\""));
    }

    #[test]
    fn field_label_is_not_a_radio() {
        let mut c = stub_options(&["Free", "Pro"]);
        c.items.insert(
            0,
            ComponentItemNode {
                item_type: "label".into(),
                text: "Plan".into(),
                link: None,
                tone: None,
                config: HashMap::new(),
            },
        );
        let html = render(&c);
        assert!(html.contains("role=\"radiogroup\""));
        assert!(html.contains("data-slot=\"radio-group\" aria-label=\"Plan\""));
        assert_eq!(html.matches("role=\"radio\"").count(), 2);
        assert!(html.contains("aria-label=\"Free\""));
        assert!(html.contains("aria-label=\"Pro\""));
        for chunk in html.split("data-slot=\"radio-group-item\"").skip(1) {
            assert!(
                !chunk.contains("aria-label=\"Plan\""),
                "Plan leaked as radio: {html}"
            );
        }
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_native_radios() {
        let c = stub_options(&["Free", "Pro"]);
        let html = render(&c);
        assert!(html.contains("data-slot=\"radio-group-item\""));
        reject_interact(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"radio-group\"]"));
        assert!(css.contains("[data-slot=\"radio-group-item\"]"));
        assert!(css.contains("display: grid"));
        assert!(css.contains("gap: 0.5rem"));
        assert!(css.contains("width: 1rem; height: 1rem"));
        assert!(css.contains("border-radius: 9999px"));
        assert!(!css.contains("zinc-"));
    }
}
