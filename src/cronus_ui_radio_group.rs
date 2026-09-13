//! Dedicated RadioGroup renderer. DOM matches React/Radix:
//! `<div data-slot="radio-group" role="radiogroup">` plus
//! `<button type="button" data-slot="radio-group-item" role="radio">`.
//! Not interact `radios()` (`<input type="radio">` inside labels).

use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let items = options(comp);
    let buttons = items
        .iter()
        .map(|(text, checked)| {
            let state = if *checked { "checked" } else { "unchecked" };
            let aria = if *checked { "true" } else { "false" };
            format!(
                "<button type=\"button\" data-slot=\"radio-group-item\" role=\"radio\" aria-checked=\"{aria}\" data-state=\"{state}\" aria-label=\"{}\"></button>",
                esc(text)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"radio-group\" role=\"radiogroup\">{buttons}</div>")
}

fn options(comp: &ComponentNode) -> Vec<(String, bool)> {
    let items: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty())
        .collect();
    if items.is_empty() {
        let label = if comp.name.is_empty() {
            "Option"
        } else {
            comp.name.as_str()
        };
        return vec![(label.to_string(), true)];
    }
    let checked_idx = items
        .iter()
        .position(|i| is_true(i.config.get("checked")))
        .unwrap_or(0);
    items
        .iter()
        .enumerate()
        .map(|(idx, i)| (i.text.clone(), idx == checked_idx))
        .collect()
}

fn is_true(raw: Option<&String>) -> bool {
    matches!(raw.map(String::as_str), Some("true" | "on" | "1"))
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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
        }
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<input"));
        assert!(!html.contains("type=\"radio\""));
        assert!(!html.contains("<label"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
    }

    #[test]
    fn root_is_radiogroup_of_button_radios() {
        let html = render(&stub_options(&["Free", "Pro"]));
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"radio-group\""));
        assert!(html.contains("role=\"radiogroup\""));
        assert_eq!(html.matches("data-slot=\"radio-group-item\"").count(), 2);
        assert!(html.contains("<button type=\"button\" data-slot=\"radio-group-item\" role=\"radio\""));
        assert!(html.contains("aria-label=\"Free\""));
        assert!(html.contains("aria-label=\"Pro\""));
        reject_interact(&html);
    }

    #[test]
    fn first_item_is_selected_by_default() {
        let html = render(&stub_options(&["Free", "Pro", "Team"]));
        assert!(html.contains(
            "aria-checked=\"true\" data-state=\"checked\" aria-label=\"Free\""
        ));
        assert!(html.contains(
            "aria-checked=\"false\" data-state=\"unchecked\" aria-label=\"Pro\""
        ));
        assert!(html.contains(
            "aria-checked=\"false\" data-state=\"unchecked\" aria-label=\"Team\""
        ));
        reject_interact(&html);
    }

    #[test]
    fn checked_true_item_is_selected() {
        let mut c = stub_options(&["Free", "Pro", "Team"]);
        c.items[1]
            .config
            .insert("checked".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(
            "aria-checked=\"false\" data-state=\"unchecked\" aria-label=\"Free\""
        ));
        assert!(html.contains(
            "aria-checked=\"true\" data-state=\"checked\" aria-label=\"Pro\""
        ));
        assert!(html.contains(
            "aria-checked=\"false\" data-state=\"unchecked\" aria-label=\"Team\""
        ));
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
    fn skips_interact_native_radios() {
        let c = stub_options(&["Free", "Pro"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("radio-group", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<input type=\"radio\""));
        assert!(interact.contains("<label"));
        assert!(!interact.contains("data-slot=\"radio-group-item\""));
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
