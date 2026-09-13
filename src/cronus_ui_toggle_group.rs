//! Dedicated ToggleGroup renderer. DOM matches React/Radix:
//! `<div data-slot="toggle-group" role="group">` plus
//! `<button type="button" data-slot="toggle-group-item" data-state="on|off" aria-pressed>`.
//! Not interact `radios()` (`<input type="radio">` inside labels).

use crate::cronus_ui_kit::esc;
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let items = options(comp);
    let buttons = items
        .iter()
        .map(|(text, on)| {
            let state = if *on { "on" } else { "off" };
            let aria = if *on { "true" } else { "false" };
            format!(
                "<button type=\"button\" data-slot=\"toggle-group-item\" data-state=\"{state}\" aria-pressed=\"{aria}\">{}</button>",
                esc(text)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"toggle-group\" role=\"group\">{buttons}</div>")
}

fn options(comp: &ComponentNode) -> Vec<(String, bool)> {
    let items: Vec<&ComponentItemNode> = comp.items.iter().filter(|i| !i.text.is_empty()).collect();
    if items.is_empty() {
        let label = if comp.name.is_empty() {
            "Option"
        } else {
            comp.name.as_str()
        };
        return vec![(label.to_string(), true)];
    }
    let on_idx = items
        .iter()
        .position(|i| is_true(i.config.get("pressed")) || is_true(i.config.get("on")))
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
    use std::collections::HashMap;

    fn stub_options(opts: &[&str]) -> ComponentNode {
        ComponentNode {
            name: "Align".into(),
            layout: Some("inline".into()),
            style: Some("toggle-group".into()),
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
        }
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<input"));
        assert!(!html.contains("type=\"radio\""));
        assert!(!html.contains("<label"));
        assert!(!html.contains("role=\"radiogroup\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
    }

    #[test]
    fn root_is_group_of_toggle_items() {
        let html = render(&stub_options(&["Left", "Center"]));
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"toggle-group\""));
        assert!(html.contains("role=\"group\""));
        assert_eq!(html.matches("data-slot=\"toggle-group-item\"").count(), 2);
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"toggle-group-item\" data-state=\"on\" aria-pressed=\"true\">Left</button>"
        ));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"toggle-group-item\" data-state=\"off\" aria-pressed=\"false\">Center</button>"
        ));
        reject_interact(&html);
    }

    #[test]
    fn first_item_is_on_by_default() {
        let html = render(&stub_options(&["Left", "Center", "Right"]));
        assert!(html.contains("data-state=\"on\" aria-pressed=\"true\">Left</button>"));
        assert!(html.contains("data-state=\"off\" aria-pressed=\"false\">Center</button>"));
        assert!(html.contains("data-state=\"off\" aria-pressed=\"false\">Right</button>"));
        reject_interact(&html);
    }

    #[test]
    fn pressed_true_item_is_on() {
        let mut c = stub_options(&["Left", "Center", "Right"]);
        c.items[1].config.insert("pressed".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("data-state=\"off\" aria-pressed=\"false\">Left</button>"));
        assert!(html.contains("data-state=\"on\" aria-pressed=\"true\">Center</button>"));
        assert!(html.contains("data-state=\"off\" aria-pressed=\"false\">Right</button>"));
        reject_interact(&html);
    }

    #[test]
    fn options_from_text_items() {
        let html = render(&stub_options(&["A", "B", "C"]));
        assert_eq!(html.matches("data-slot=\"toggle-group-item\"").count(), 3);
        assert!(html.contains(">A</button>"));
        assert!(html.contains(">B</button>"));
        assert!(html.contains(">C</button>"));
    }

    #[test]
    fn skips_interact_native_radios() {
        let c = stub_options(&["Left", "Center"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("toggle-group", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<input type=\"radio\""));
        assert!(interact.contains("<label"));
        assert!(interact.contains("role=\"radiogroup\""));
        assert!(!interact.contains("data-slot=\"toggle-group-item\""));
        assert!(html.contains("data-slot=\"toggle-group-item\""));
        reject_interact(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"toggle-group\"]"));
        assert!(css.contains("[data-slot=\"toggle-group-item\"]"));
        assert!(css.contains("display: flex"));
        assert!(css.contains("gap: 0.25rem"));
        assert!(css.contains("height: 2.5rem"));
        assert!(css.contains("[data-slot=\"toggle-group-item\"][data-state=\"on\"]"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(!css.contains("zinc-"));
    }
}
