//! Dedicated SegmentedControl renderer. DOM matches React:
//! `<div data-slot="segmented-control" role="tablist">` plus
//! `<button type="button" data-slot="segmented-control-item" role="tab">`.
//! First item selected. Not interact `radios()` (`<input type="radio">`).

use crate::cronus_ui_kit::esc;
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let items = options(comp);
    let buttons = items
        .iter()
        .map(|(text, on)| {
            let state = if *on { "active" } else { "inactive" };
            let selected = if *on { "true" } else { "false" };
            format!(
                "<button type=\"button\" data-slot=\"segmented-control-item\" role=\"tab\" aria-selected=\"{selected}\" data-state=\"{state}\">{}</button>",
                esc(text)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"segmented-control\" role=\"tablist\">{buttons}</div>")
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
    let on_idx = selected_idx(comp, &items);
    items
        .iter()
        .enumerate()
        .map(|(idx, i)| (i.text.clone(), idx == on_idx))
        .collect()
}

fn selected_idx(comp: &ComponentNode, items: &[&ComponentItemNode]) -> usize {
    if let Some(v) = comp.props.get("value") {
        if let Some(idx) = items.iter().position(|i| i.text == *v) {
            return idx;
        }
    }
    items
        .iter()
        .position(|i| {
            is_true(i.config.get("selected"))
                || is_true(i.config.get("pressed"))
                || is_true(i.config.get("on"))
                || is_true(i.config.get("checked"))
        })
        .unwrap_or(0)
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
            name: "View".into(),
            layout: Some("inline".into()),
            style: Some("segmented-control".into()),
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
        assert!(!html.contains("<script"));
        assert!(!html.contains("radios("));
    }

    #[test]
    fn root_is_tablist_of_segment_items() {
        let html = render(&stub_options(&["Day", "Week"]));
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"segmented-control\""));
        assert!(html.contains("role=\"tablist\""));
        assert_eq!(html.matches("data-slot=\"segmented-control-item\"").count(), 2);
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"segmented-control-item\" role=\"tab\" aria-selected=\"true\" data-state=\"active\">Day</button>"
        ));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"segmented-control-item\" role=\"tab\" aria-selected=\"false\" data-state=\"inactive\">Week</button>"
        ));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"segmented-control\" role=\"tablist\"><button type=\"button\" data-slot=\"segmented-control-item\" role=\"tab\" aria-selected=\"true\" data-state=\"active\">Day</button><button type=\"button\" data-slot=\"segmented-control-item\" role=\"tab\" aria-selected=\"false\" data-state=\"inactive\">Week</button></div>"
        );
    }

    #[test]
    fn first_item_is_selected_by_default() {
        let html = render(&stub_options(&["Day", "Week", "Month"]));
        assert!(html.contains(
            "aria-selected=\"true\" data-state=\"active\">Day</button>"
        ));
        assert!(html.contains(
            "aria-selected=\"false\" data-state=\"inactive\">Week</button>"
        ));
        assert!(html.contains(
            "aria-selected=\"false\" data-state=\"inactive\">Month</button>"
        ));
        reject_interact(&html);
    }

    #[test]
    fn selected_true_item_is_active() {
        let mut c = stub_options(&["Day", "Week", "Month"]);
        c.items[1].config.insert("selected".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(
            "aria-selected=\"false\" data-state=\"inactive\">Day</button>"
        ));
        assert!(html.contains(
            "aria-selected=\"true\" data-state=\"active\">Week</button>"
        ));
        assert!(html.contains(
            "aria-selected=\"false\" data-state=\"inactive\">Month</button>"
        ));
        reject_interact(&html);
    }

    #[test]
    fn value_prop_selects_matching_item() {
        let mut c = stub_options(&["Day", "Week", "Month"]);
        c.props.insert("value".into(), "Month".into());
        let html = render(&c);
        assert!(html.contains(
            "aria-selected=\"true\" data-state=\"active\">Month</button>"
        ));
        assert!(html.contains(
            "aria-selected=\"false\" data-state=\"inactive\">Day</button>"
        ));
        reject_interact(&html);
    }

    #[test]
    fn options_from_text_items() {
        let html = render(&stub_options(&["A", "B", "C"]));
        assert_eq!(html.matches("data-slot=\"segmented-control-item\"").count(), 3);
        assert!(html.contains(">A</button>"));
        assert!(html.contains(">B</button>"));
        assert!(html.contains(">C</button>"));
    }

    #[test]
    fn skips_interact_native_radios() {
        let c = stub_options(&["Day", "Week"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("segmented-control", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<input type=\"radio\""));
        assert!(interact.contains("<label"));
        assert!(interact.contains("role=\"radiogroup\""));
        assert!(!interact.contains("data-slot=\"segmented-control-item\""));
        assert!(html.contains("data-slot=\"segmented-control-item\""));
        assert!(html.contains("role=\"tablist\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub_options(&["Day"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"segmented-control\"]"));
        assert!(css.contains("[data-slot=\"segmented-control-item\"]"));
        assert!(css.contains("[data-slot=\"segmented-control-item\"][data-state=\"active\"]"));
        assert!(css.contains("display: inline-flex"));
        assert!(css.contains("gap: 0.25rem"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
