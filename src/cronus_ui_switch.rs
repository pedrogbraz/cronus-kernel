//! Dedicated Switch renderer. DOM matches React/Radix:
//! `<button type="button" data-slot="switch" role="switch" aria-checked data-state>`.
//! Not the interact `<label data-slot="switch"><input type="checkbox" role="switch" data-slot="switch-control">`.

use crate::cronus_ui_kit::{attr, esc, flag_any};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let checked = flag_any(comp, "checked");
    let disabled = flag_any(comp, "disabled");
    let state = if checked { "checked" } else { "unchecked" };
    let aria_checked = if checked { "true" } else { "false" };
    let mut attrs = format!(
        "type=\"button\" role=\"switch\" aria-checked=\"{aria_checked}\" data-state=\"{state}\" value=\"on\" data-slot=\"switch\""
    );
    if disabled {
        attrs.push_str(" disabled");
    }
    if let Some(label) = aria_label(comp) {
        attrs.push_str(&format!(" aria-label=\"{}\"", esc(label)));
    }
    // Radix Thumb carries no data-slot; React renders no label text.
    format!("<button {attrs}><span data-state=\"{state}\"></span></button>")
}

fn aria_label(comp: &ComponentNode) -> Option<&str> {
    attr(comp, "aria-label")
        .or_else(|| item(comp, "label"))
        .or_else(|| item(comp, "title"))
        .or_else(|| item(comp, "text"))
        .filter(|s| !s.is_empty())
}

fn item<'a>(comp: &'a ComponentNode, kind: &str) -> Option<&'a str> {
    comp.items
        .iter()
        .find(|i| i.item_type == kind)
        .map(|i| i.text.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn stub() -> ComponentNode {
        ComponentNode {
            name: "Notifications".into(),
            layout: Some("inline".into()),
            style: Some("switch".into()),
            items: vec![ComponentItemNode {
                item_type: "label".into(),
                text: "Notifications".into(),
                link: None,
                tone: None,
                config: HashMap::new(),
            }],
            props: HashMap::new(),
            params: vec![],
            template: None,
            sections: vec![],
            state: vec![],
            tests: vec![],
            binding: None,
        }
    }

    #[test]
    fn root_is_button_not_label_checkbox() {
        let html = render(&stub());
        assert!(html.starts_with("<button "));
        assert!(html.contains("type=\"button\""));
        assert!(html.contains("data-slot=\"switch\""));
        assert!(html.contains("role=\"switch\""));
        assert!(html.contains("aria-checked=\"false\""));
        assert!(html.contains("data-state=\"unchecked\""));
        assert!(html.contains("<span data-state=\"unchecked\"></span></button>"));
        assert!(!html.contains("switch-thumb"));
        assert!(!html.contains("switch-text"));
        assert!(!html.contains("<label"));
        assert!(!html.contains("type=\"checkbox\""));
        assert!(!html.contains("data-slot=\"switch-control\""));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
    }

    #[test]
    fn wave1t_off_fixture_exact_dom() {
        let mut c = stub();
        c.items[0].text = "Airplane mode".into();
        c.items[0]
            .config
            .insert("aria-label".into(), "Airplane mode".into());
        assert_eq!(
            render(&c),
            "<button type=\"button\" role=\"switch\" aria-checked=\"false\" data-state=\"unchecked\" value=\"on\" data-slot=\"switch\" aria-label=\"Airplane mode\"><span data-state=\"unchecked\"></span></button>"
        );
    }

    #[test]
    fn checked_from_props() {
        let mut c = stub();
        c.props.insert("checked".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("aria-checked=\"true\""));
        assert!(html.contains("data-state=\"checked\""));
    }

    #[test]
    fn checked_from_item_colon_pair() {
        let mut c = stub();
        c.items[0].config.insert("checked".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("aria-checked=\"true\""));
        assert!(html.contains("data-state=\"checked\""));
    }

    #[test]
    fn chrome_uses_cronus_tokens() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"switch\"]"));
        assert!(css.contains("[data-slot=\"switch\"] > span {"));
        assert!(css.contains("[data-slot=\"switch\"][data-state=\"checked\"] > span {"));
        assert!(!css.contains("switch-thumb"));
        assert!(css.contains("height: 1.25rem"));
        assert!(css.contains("width: 2.25rem"));
        assert!(css.contains("width: 1rem; height: 1rem"));
        assert!(css.contains("background: var(--cronus-primary)"));
        assert!(!css.contains("zinc-"));
    }
}
