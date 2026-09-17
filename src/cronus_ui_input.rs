//! Dedicated Input renderer. DOM matches React: `<input data-slot="input">`.
//! Not the interact `<label data-slot="input"><input data-slot="input-control">`.

use crate::cronus_ui_kit::{attr, esc, flag_any};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let placeholder = item(comp, "text")
        .or_else(|| item(comp, "label"))
        .or_else(|| comp.props.get("placeholder").map(String::as_str))
        .unwrap_or("");
    let ty = comp.props.get("type").map(String::as_str).unwrap_or("text");
    let disabled = flag_any(comp, "disabled");
    let invalid = flag_any(comp, "invalid");
    let aria_label = attr(comp, "aria-label")
        .or_else(|| item(comp, "title"))
        .or_else(|| item(comp, "label"));
    let mut attrs = format!(
        "data-slot=\"input\" type=\"{}\" placeholder=\"{}\"",
        esc(ty),
        esc(placeholder)
    );
    if disabled {
        attrs.push_str(" disabled");
    }
    if invalid {
        attrs.push_str(" aria-invalid=\"true\"");
    }
    if let Some(label) = aria_label {
        attrs.push_str(&format!(" aria-label=\"{}\"", esc(label)));
    }
    if let Some(v) = comp.props.get("value").filter(|v| !v.is_empty()) {
        attrs.push_str(&format!(" value=\"{}\"", esc(v)));
    }
    if let Some(id) = comp.props.get("id").filter(|v| !v.is_empty()) {
        attrs.push_str(&format!(" id=\"{}\"", esc(id)));
    }
    format!("<input {attrs} />")
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
            name: "Name".into(),
            layout: Some("inline".into()),
            style: Some("input".into()),
            items: vec![ComponentItemNode {
                item_type: "text".into(),
                text: "Your name".into(),
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

    #[test]
    fn root_is_input_not_label_wrapper() {
        let html = render(&stub());
        assert!(html.starts_with("<input "));
        assert!(html.contains("data-slot=\"input\""));
        assert!(!html.contains("data-slot=\"input-control\""));
        assert!(!html.contains("<label"));
        assert!(html.contains("placeholder=\"Your name\""));
    }

    #[test]
    fn invalid_sets_aria_invalid() {
        let mut c = stub();
        c.props.insert("invalid".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("aria-invalid=\"true\""));
    }

    #[test]
    fn disabled_attr() {
        let mut c = stub();
        c.props.insert("disabled".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(" disabled"));
    }
}
