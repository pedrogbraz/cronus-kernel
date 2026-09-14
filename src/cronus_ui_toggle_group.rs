//! Dedicated ToggleGroup renderer.
//! `<div data-slot="toggle-group" role="group" aria-label>` plus
//! `<button type="button" data-slot="toggle-group-item" data-state="on|off" aria-pressed disabled>`.
//! Geometry matches React/Radix (`type="single"`); roles stay `group`/`aria-pressed`
//! because the stub gate treats `role="radiogroup"` as the interact radios fallback.
//! Switching the pressed item needs JS, so (wave 1t rule) items are native buttons
//! marked `disabled`, without visual attenuation; chrome only dims `data-disabled`
//! (Radix's attribute for a genuinely disabled item).
//! The `label` names the group (it is never an option). Selected option: item
//! `pressed`/`on`, else the group `value` (props or trailing `value:` config), else first.
//! Not interact `radios()` (`<input type="radio">` inside labels).

use crate::cronus_ui_kit::{attr, attr_nonempty, esc};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let buttons = options(comp)
        .iter()
        .map(|(text, on)| {
            let state = if *on { "on" } else { "off" };
            let aria = if *on { "true" } else { "false" };
            format!(
                "<button type=\"button\" data-slot=\"toggle-group-item\" data-state=\"{state}\" aria-pressed=\"{aria}\" disabled>{}</button>",
                esc(text)
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
        assert!(!html.contains("<input"));
        assert!(!html.contains("<label"));
        assert!(!html.contains("role=\"radiogroup\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
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
            "<div data-slot=\"toggle-group\" role=\"group\" aria-label=\"Range\"><button type=\"button\" data-slot=\"toggle-group-item\" data-state=\"off\" aria-pressed=\"false\" disabled>Day</button><button type=\"button\" data-slot=\"toggle-group-item\" data-state=\"on\" aria-pressed=\"true\" disabled>Week</button></div>"
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
        assert!(html.contains("data-state=\"on\" aria-pressed=\"true\" disabled>Left</button>"));
        assert!(html.contains("data-state=\"off\" aria-pressed=\"false\" disabled>Center</button>"));
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
        assert!(html.contains("data-state=\"on\" aria-pressed=\"true\" disabled>Center</button>"));
        assert_eq!(html.matches("data-state=\"on\"").count(), 1);
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
