//! Shared helpers for dedicated cronus-ui family renderers.

use crate::parser::{ComponentItemNode, ComponentNode};

pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn item<'a>(comp: &'a ComponentNode, kind: &str) -> Option<&'a str> {
    comp.items
        .iter()
        .find(|i| i.item_type == kind)
        .map(|i| i.text.as_str())
}

pub fn label_of(comp: &ComponentNode) -> String {
    for kind in ["label", "title", "text", "value"] {
        if let Some(t) = item(comp, kind) {
            if !t.is_empty() {
                return esc(t);
            }
        }
    }
    for i in &comp.items {
        if !i.text.is_empty() {
            return esc(&i.text);
        }
    }
    esc(&comp.name)
}

pub fn texts(comp: &ComponentNode) -> Vec<String> {
    let mut out: Vec<String> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if out.is_empty() {
        out.push(label_of(comp));
    }
    out
}

#[cfg(test)]
pub fn stub(family: &str, label: &str) -> ComponentNode {
    use std::collections::HashMap;
    ComponentNode {
        name: family.to_string(),
        layout: Some("stack".into()),
        style: Some(family.into()),
        items: vec![ComponentItemNode {
            item_type: "label".into(),
            text: label.into(),
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
