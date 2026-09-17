//! Dedicated Checkbox renderer. DOM matches React/Radix:
//! `<button type="button" data-slot="checkbox" role="checkbox">`.
//! Not the interact `<label data-slot="checkbox"><input type="checkbox" data-slot="checkbox-control">`.

use crate::cronus_ui_kit::{esc, flag_any};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let checked = checked_of(comp);
    let disabled = flag_any(comp, "disabled");
    let invalid = flag_any(comp, "invalid");
    let state = if checked { "checked" } else { "unchecked" };
    let aria_checked = if checked { "true" } else { "false" };
    let mut attrs = format!(
        "type=\"button\" role=\"checkbox\" aria-checked=\"{aria_checked}\" data-state=\"{state}\" value=\"on\" data-slot=\"checkbox\""
    );
    if disabled {
        attrs.push_str(" disabled");
    }
    if invalid {
        attrs.push_str(" aria-invalid=\"true\"");
    }
    let name = aria_label_of(comp).map(esc);
    if let Some(label) = &name {
        attrs.push_str(&format!(" aria-label=\"{label}\""));
    }
    // Radix Indicator: unslotted span + lucide Check; React renders no label text.
    let inner = CHECK_INDICATOR;
    // Zero JS: the button is decorative; a visually hidden checkbox in the
    // wrapping label carries the state (click, Space, focus ring). A `text`
    // item is the docs' companion `<Label>` (`Checkbox` + `Label htmlFor`).
    let caption = item(comp, "text")
        .filter(|t| !t.is_empty())
        .map(|t| format!("<span data-slot=\"label\">{}</span>", esc(t)))
        .unwrap_or_default();
    let on = if checked { " checked" } else { "" };
    let dis = if disabled { " disabled" } else { "" };
    let inv = if invalid {
        " aria-invalid=\"true\""
    } else {
        ""
    };
    let input_name = name
        .as_deref()
        .map(|n| format!(" aria-label=\"{n}\""))
        .unwrap_or_default();
    let id = crate::cronus_ui_kit::instance_id(comp, "checkbox");
    format!(
        "<label data-control=\"checkbox\"><input type=\"checkbox\" id=\"{id}\"{input_name}{on}{dis}{inv}><button {attrs} tabindex=\"-1\" aria-hidden=\"true\">{inner}</button>{caption}</label>"
    )
}

const CHECK_INDICATOR: &str = "<span data-state=\"checked\"><svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M20 6 9 17l-5-5\"></path></svg></span>";

fn checked_of(comp: &ComponentNode) -> bool {
    if flag_any(comp, "checked") {
        return true;
    }
    comp.style
        .as_deref()
        .unwrap_or("")
        .split('+')
        .any(|part| part.trim() == "checked")
}

fn aria_label_of(comp: &ComponentNode) -> Option<&str> {
    if let Some(v) = comp.props.get("aria-label") {
        return Some(v.as_str());
    }
    if let Some(v) = comp
        .items
        .iter()
        .find_map(|i| i.config.get("aria-label").map(String::as_str))
    {
        return Some(v);
    }
    for kind in ["label", "title", "text", "value"] {
        if let Some(t) = item(comp, kind) {
            if !t.is_empty() {
                return Some(t);
            }
        }
    }
    if comp.name.is_empty() {
        None
    } else {
        Some(comp.name.as_str())
    }
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
            name: "Agree".into(),
            layout: Some("inline".into()),
            style: Some("checkbox".into()),
            items: vec![ComponentItemNode {
                item_type: "label".into(),
                text: "I agree".into(),
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
            span: Default::default(),
        }
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("data-slot=\"checkbox-control\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        // One decorative button after the one native state input.
        assert_eq!(html.matches("<button ").count(), 1);
        assert_eq!(html.matches("<input ").count(), 1);
    }

    #[test]
    fn root_is_button_not_input_wrapper() {
        let html = render(&stub());
        assert!(html.starts_with("<label data-control=\"checkbox\"><input type=\"checkbox\""));
        assert!(html.contains("><button "));
        assert!(html.contains("type=\"button\""));
        assert!(html.contains("data-slot=\"checkbox\""));
        assert!(html.contains("role=\"checkbox\""));
        assert!(html.contains("aria-checked=\"false\""));
        assert!(html.contains("data-state=\"unchecked\""));
        assert!(html.contains("aria-label=\"I agree\""));
        reject_interact(&html);
    }

    #[test]
    fn checked_from_props() {
        let mut c = stub();
        c.props.insert("checked".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("aria-checked=\"true\""));
        assert!(html.contains("data-state=\"checked\""));
        assert!(html.contains(CHECK_INDICATOR));
        assert!(!html.contains("checkbox-indicator"));
        reject_interact(&html);
    }

    #[test]
    fn checked_from_style() {
        let mut c = stub();
        c.style = Some("checkbox+checked".into());
        let html = render(&c);
        assert!(html.contains("aria-checked=\"true\""));
        assert!(html.contains("data-state=\"checked\""));
        reject_interact(&html);
    }

    #[test]
    fn checked_from_item_config() {
        let mut c = stub();
        c.items[0].config.insert("checked".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("aria-checked=\"true\""));
        assert!(html.contains("data-state=\"checked\""));
        reject_interact(&html);
    }

    #[test]
    fn wave1t_off_fixture_has_no_label_text() {
        // app.cronus CheckboxOff: `label "Accept"` + `aria-label:"Accept"` on that item.
        let mut c = stub();
        c.items[0].text = "Accept".into();
        c.items[0]
            .config
            .insert("aria-label".into(), "Accept".into());
        crate::cronus_ui_kit::reset_instance_ids();
        assert_eq!(
            render(&c),
            format!("<label data-control=\"checkbox\"><input type=\"checkbox\" id=\"cui-agree-checkbox\" aria-label=\"Accept\"><button type=\"button\" role=\"checkbox\" aria-checked=\"false\" data-state=\"unchecked\" value=\"on\" data-slot=\"checkbox\" aria-label=\"Accept\" tabindex=\"-1\" aria-hidden=\"true\">{CHECK_INDICATOR}</button></label>")
        );
        let css = crate::cronus_ui::component_chrome_css();
        assert!(!css.contains("checkbox-text"));
        assert!(css.contains(
            "background: var(--cronus-surface-inset);
  box-shadow: var(--cronus-shadow-xs, none);
  transition:"
        ));
        assert!(css.contains(
            "[data-slot=\"checkbox\"] > span > svg { width: 0.875rem; height: 0.875rem; }"
        ));
    }

    #[test]
    fn disabled_and_invalid() {
        let mut c = stub();
        c.props.insert("disabled".into(), "true".into());
        c.props.insert("invalid".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(" disabled"));
        assert!(html.contains("aria-invalid=\"true\""));
        reject_interact(&html);
    }

    #[test]
    fn text_item_is_the_companion_label() {
        let mut c = stub();
        c.items.push(ComponentItemNode {
            item_type: "text".into(),
            text: "Unavailable option".into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        });
        let html = render(&c);
        assert!(
            html.ends_with("</button><span data-slot=\"label\">Unavailable option</span></label>")
        );
        reject_interact(&html);
    }
}
