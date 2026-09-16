//! Dedicated Label renderer. DOM matches React: `<label data-slot="label">`.
//! Not `pill("label")` → `<span data-slot="label" style=…>`.

use crate::cronus_ui_kit::{attr, esc};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let text = esc(&label_of(comp));
    let mut attrs = String::from("data-slot=\"label\"");
    if let Some(for_id) = html_for(comp) {
        attrs.push_str(&format!(" for=\"{}\"", esc(for_id)));
    }
    format!("<label {attrs}>{text}</label>")
}

fn html_for(comp: &ComponentNode) -> Option<&str> {
    comp.props
        .get("for")
        .or_else(|| comp.props.get("htmlFor"))
        .map(String::as_str)
        .or_else(|| attr(comp, "for"))
}

fn label_of(comp: &ComponentNode) -> &str {
    for kind in ["label", "title", "text", "value"] {
        if let Some(t) = item(comp, kind) {
            if !t.is_empty() {
                return t;
            }
        }
    }
    &comp.name
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

    fn stub(text: &str) -> ComponentNode {
        ComponentNode {
            name: "Email".into(),
            layout: Some("inline".into()),
            style: Some("label".into()),
            items: vec![ComponentItemNode {
                item_type: "label".into(),
                text: text.into(),
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
    fn root_is_label_not_span_pill() {
        let html = render(&stub("Email"));
        assert!(html.starts_with("<label "));
        assert!(html.contains("data-slot=\"label\""));
        assert!(html.contains(">Email</label>"));
        assert!(!html.contains("<span"));
        assert!(!html.contains("data-slot=\"label-control\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("display:inline-flex;align-items:center;gap:0.35rem"));
    }

    #[test]
    fn for_attr_from_props() {
        let mut c = stub("Email");
        c.props.insert("for".into(), "email".into());
        let html = render(&c);
        assert!(html.contains(" for=\"email\""));
    }
}
