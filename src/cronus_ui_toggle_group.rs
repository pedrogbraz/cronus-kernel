//! Dedicated ToggleGroup renderer.
//! `<div data-slot="toggle-group" role="group" aria-label>` plus, per item,
//! `<label>` > visually hidden state `<input>` + React's
//! `<button type="button" data-slot="toggle-group-item" data-state="on|off" aria-pressed>`.
//! Geometry matches React/Radix (`type="single"`); the root role stays `group`
//! because the stub gate treats `role="radiogroup"` as the interact radios fallback.
//!
//! Zero JS: in single mode (default) the inputs are radios sharing a page-unique
//! `name`, so pressing one releases the others and Arrow keys move the choice;
//! `type:"multiple"` makes them independent checkboxes. CSS paints the item after
//! a checked input as pressed. The button keeps React's slot and look but is
//! decorative (`aria-hidden`, `tabindex="-1"`, `pointer-events: none`); only
//! `data-disabled` (Radix's genuinely disabled item) dims.
//! Gaps vs Radix: single mode cannot release the pressed item (radios), the
//! control is announced as a radio/checkbox instead of a pressed button, and
//! `data-state` / `aria-pressed` stay at the initial state.
//! The `label` names the group (it is never an option). Selected option: item
//! `pressed`/`on`, else the group `value` (props or trailing `value:` config), else first.
//! Not interact `radios()` (inline-styled `<input type="radio">` labels).

use crate::cronus_ui_kit::{attr, attr_nonempty, esc, instance_id};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let kind = if attr(comp, "type") == Some("multiple") {
        "checkbox"
    } else {
        "radio"
    };
    let name = instance_id(comp, "toggle-group");
    let buttons = options(comp)
        .iter()
        .enumerate()
        .map(|(i, (text, on))| {
            let (state, aria, checked) = if *on {
                ("on", "true", " checked")
            } else {
                ("off", "false", "")
            };
            let t = esc(text);
            format!(
                "<label><input type=\"{kind}\" name=\"{name}\" value=\"{i}\" aria-label=\"{t}\"{checked}><button type=\"button\" data-slot=\"toggle-group-item\" data-state=\"{state}\" aria-pressed=\"{aria}\" tabindex=\"-1\" aria-hidden=\"true\">{t}</button></label>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let aria = attr_nonempty(comp, "aria-label")
        .map(|a| format!(" aria-label=\"{}\"", esc(a)))
        .unwrap_or_default();
    format!("<div data-slot=\"toggle-group\" role=\"group\"{aria}>{buttons}</div>")
}

fn options(comp: &ComponentNode) -> Vec<(String, bool)> {
    let items: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| !matches!(i.item_type.as_str(), "label" | "title") && !i.text.is_empty())
        .collect();
    if items.is_empty() {
        let label = comp
            .items
            .iter()
            .find(|i| !i.text.is_empty())
            .map(|i| i.text.clone())
            .unwrap_or_else(|| {
                if comp.name.is_empty() {
                    "Option".into()
                } else {
                    comp.name.clone()
                }
            });
        return vec![(label, true)];
    }
    let group_value = attr(comp, "value");
    let on_idx = items
        .iter()
        .position(|i| is_true(i.config.get("pressed")) || is_true(i.config.get("on")))
        .or_else(|| group_value.and_then(|v| items.iter().position(|i| &i.text == v)))
        .unwrap_or(0);
    items
        .iter()
        .enumerate()
        .map(|(idx, i)| (i.text.clone(), idx == on_idx))
        .collect()
}

fn is_true(raw: Option<&String>) -> bool {
    matches!(raw.map(String::as_str), Some("true" | "on" | "1"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn item(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("role=\"radiogroup\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains(" disabled"));
        assert!(!crate::cli::stub_renderer_gate::looks_like_interact_generic(html));
    }

    #[test]
    fn emitted_fixture_label_is_aria_and_value_selects() {
        let mut c = stub("toggle-group", "Range");
        c.items.push(item("text", "Day"));
        c.items.push(item("text", "Week"));
        c.items[2].config.insert("value".into(), "Week".into());
        c.items[2]
            .config
            .insert("aria-label".into(), "Range".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"toggle-group\" role=\"group\" aria-label=\"Range\"><label><input type=\"radio\" name=\"cui-toggle-group-toggle-group\" value=\"0\" aria-label=\"Day\"><button type=\"button\" data-slot=\"toggle-group-item\" data-state=\"off\" aria-pressed=\"false\" tabindex=\"-1\" aria-hidden=\"true\">Day</button></label><label><input type=\"radio\" name=\"cui-toggle-group-toggle-group\" value=\"1\" aria-label=\"Week\" checked><button type=\"button\" data-slot=\"toggle-group-item\" data-state=\"on\" aria-pressed=\"true\" tabindex=\"-1\" aria-hidden=\"true\">Week</button></label></div>"
        );
        reject_interact(&html);
    }

    #[test]
    fn first_item_is_on_by_default() {
        let mut c = stub("toggle-group", "Align");
        c.items.clear();
        for t in ["Left", "Center"] {
            c.items.push(item("item", t));
        }
        let html = render(&c);
        assert!(html.contains("aria-label=\"Left\" checked><button type=\"button\" data-slot=\"toggle-group-item\" data-state=\"on\""));
        assert!(html.contains("aria-label=\"Center\"><button type=\"button\" data-slot=\"toggle-group-item\" data-state=\"off\""));
    }

    #[test]
    fn pressed_true_item_is_on() {
        let mut c = stub("toggle-group", "Align");
        c.items.clear();
        for t in ["Left", "Center", "Right"] {
            c.items.push(item("item", t));
        }
        c.items[1].config.insert("pressed".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Center\" checked><button"));
        assert_eq!(html.matches("data-state=\"on\"").count(), 1);
        assert_eq!(html.matches(" checked").count(), 1);
    }

    #[test]
    fn multiple_type_uses_checkboxes() {
        let mut c = stub("toggle-group", "Format");
        c.items.clear();
        for t in ["Bold", "Italic"] {
            c.items.push(item("item", t));
        }
        c.props.insert("type".into(), "multiple".into());
        let html = render(&c);
        assert_eq!(html.matches("type=\"checkbox\"").count(), 2);
        assert!(!html.contains("type=\"radio\""));
    }

    #[test]
    fn two_groups_on_one_page_get_distinct_names() {
        crate::cronus_ui_kit::reset_instance_ids();
        let c = stub("toggle-group", "Align");
        let a = render(&c);
        let b = render(&c);
        assert!(a.contains("name=\"cui-toggle-group-toggle-group\" "));
        assert!(b.contains("name=\"cui-toggle-group-toggle-group-2\" "));
    }

    /// The checked input, not the initial `data-state`, paints the pressed item.
    #[test]
    fn chrome_pressed_look_follows_checked_input() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"toggle-group\"] > label > input:checked + [data-slot=\"toggle-group-item\"] {\n  background: var(--cronus-surface-overlay); color: var(--cronus-fg);\n}"));
        assert!(css.contains("[data-slot=\"toggle-group\"] > label > input:not(:checked) + [data-slot=\"toggle-group-item\"][data-state=\"on\"] {\n  background: transparent; color: var(--cronus-fg-secondary);\n}"));
        assert!(css.contains("[data-slot=\"toggle-group\"] > label > [data-slot=\"toggle-group-item\"] { pointer-events: none; }"));
    }

    #[test]
    fn skips_interact_native_radios() {
        let mut c = stub("toggle-group", "Align");
        c.items.push(item("item", "Left"));
        let html = render(&c);
        reject_interact(&html);
    }

    #[test]
    fn chrome_matches_react_geometry_and_only_data_disabled_dims() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"toggle-group-item\"][data-state=\"on\"]"));
        assert!(css.contains(
            "font-size: 0.875rem; line-height: 1.25rem; font-weight: 500; font-family: inherit;"
        ));
        assert!(css.contains("[data-slot=\"toggle-group-item\"][data-disabled] { opacity: 0.5; pointer-events: none; }"));
        assert!(!css.contains("[data-slot=\"toggle-group-item\"]:disabled"));
        assert!(!css.contains("zinc-"));
    }
}
