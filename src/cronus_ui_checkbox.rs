//! Dedicated Checkbox renderer. DOM matches React/Radix:
//! `<button type="button" data-slot="checkbox" role="checkbox">`.
//! Not the interact `<label data-slot="checkbox"><input type="checkbox" data-slot="checkbox-control">`.

use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let checked = checked_of(comp);
    let disabled = flag(comp, "disabled");
    let invalid = flag(comp, "invalid");
    let state = if checked { "checked" } else { "unchecked" };
    let aria_checked = if checked { "true" } else { "false" };
    let mut attrs = format!(
        "type=\"button\" data-slot=\"checkbox\" role=\"checkbox\" aria-checked=\"{aria_checked}\" data-state=\"{state}\""
    );
    if disabled {
        attrs.push_str(" disabled");
    }
    if invalid {
        attrs.push_str(" aria-invalid=\"true\"");
    }
    if let Some(label) = aria_label_of(comp) {
        attrs.push_str(&format!(" aria-label=\"{}\"", esc(label)));
    }
    let inner = if checked {
        "<span data-slot=\"checkbox-indicator\"></span>"
    } else {
        ""
    };
    format!("<button {attrs}>{inner}</button>")
}

fn checked_of(comp: &ComponentNode) -> bool {
    if flag(comp, "checked") {
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

fn flag(comp: &ComponentNode, name: &str) -> bool {
    if comp.props.get(name).map(|s| s == "true").unwrap_or(false) {
        return true;
    }
    comp.items
        .iter()
        .any(|i| i.config.get(name).map(|s| s == "true").unwrap_or(false))
}

fn item<'a>(comp: &'a ComponentNode, kind: &str) -> Option<&'a str> {
    comp.items
        .iter()
        .find(|i| i.item_type == kind)
        .map(|i| i.text.as_str())
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
        }
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("data-slot=\"checkbox-control\""));
        assert!(!html.contains("<label"));
        assert!(!html.contains("type=\"checkbox\""));
        assert!(!html.contains("<input"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
    }

    #[test]
    fn root_is_button_not_input_wrapper() {
        let html = render(&stub());
        assert!(html.starts_with("<button "));
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
        assert!(html.contains("data-slot=\"checkbox-indicator\""));
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
        c.items[0]
            .config
            .insert("checked".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("aria-checked=\"true\""));
        assert!(html.contains("data-state=\"checked\""));
        reject_interact(&html);
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
}
