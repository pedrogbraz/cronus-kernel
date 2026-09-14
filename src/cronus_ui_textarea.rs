//! Dedicated Textarea renderer. DOM matches React: `<textarea data-slot="textarea">`.
//! Not the interact `<label data-slot="textarea"><textarea data-slot="textarea-control">`.

use crate::cronus_ui_kit::{attr, esc, flag_any};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let placeholder = item(comp, "text")
        .or_else(|| item(comp, "label"))
        .or_else(|| comp.props.get("placeholder").map(String::as_str))
        .unwrap_or("");
    let disabled = flag_any(comp, "disabled");
    let invalid = flag_any(comp, "invalid");
    let aria_label = attr(comp, "aria-label")
        .or_else(|| item(comp, "title"))
        .or_else(|| item(comp, "label"));
    let value = item(comp, "value")
        .or_else(|| comp.props.get("value").map(String::as_str))
        .unwrap_or("");
    let mut attrs = format!(
        "data-slot=\"textarea\" placeholder=\"{}\"",
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
    format!("<textarea {attrs}>{}</textarea>", esc(value))
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
            name: "Notes".into(),
            layout: Some("stack".into()),
            style: Some("textarea".into()),
            items: vec![ComponentItemNode {
                item_type: "label".into(),
                text: "Notes".into(),
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
    fn root_is_textarea_not_label_wrapper() {
        let html = render(&stub());
        assert!(html.starts_with("<textarea "));
        assert!(html.contains("data-slot=\"textarea\""));
        assert!(html.contains("placeholder=\"Notes\""));
        assert!(!html.contains("data-slot=\"textarea-control\""));
        assert!(!html.contains("<label"));
        assert!(!html.contains("style="));
        assert!(html.ends_with("</textarea>"));
    }

    #[test]
    fn placeholder_from_text_item() {
        let mut c = stub();
        c.items.insert(
            0,
            ComponentItemNode {
                item_type: "text".into(),
                text: "Write a note".into(),
                link: None,
                tone: None,
                config: HashMap::new(),
            },
        );
        let html = render(&c);
        assert!(html.contains("placeholder=\"Write a note\""));
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
