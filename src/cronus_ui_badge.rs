//! Dedicated Badge renderer. DOM matches React: `<span data-slot="badge" data-variant>`.
//! Not `pill()` and not `cronus_ui_interact`.

use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let variant = variant_of(comp);
    let label = label_of(comp);
    format!("<span data-slot=\"badge\" data-variant=\"{variant}\">{label}</span>")
}

fn variant_of(comp: &ComponentNode) -> &'static str {
    if let Some(v) = comp.props.get("variant") {
        return named_variant(v);
    }
    let style = comp.style.as_deref().unwrap_or("");
    for part in style.split('+') {
        let named = named_variant(part);
        if named != "default" || part == "default" {
            if part == "badge" {
                continue;
            }
            if matches!(
                part,
                "destructive"
                    | "primary"
                    | "secondary"
                    | "outline"
                    | "success"
                    | "warning"
                    | "error"
                    | "info"
                    | "default"
            ) {
                return named;
            }
        }
    }
    "default"
}

fn named_variant(raw: &str) -> &'static str {
    match raw {
        "primary" => "primary",
        "secondary" => "secondary",
        "outline" => "outline",
        "success" => "success",
        "warning" => "warning",
        "destructive" | "danger" => "destructive",
        "error" => "error",
        "info" => "info",
        "default" => "default",
        _ => "default",
    }
}

fn label_of(comp: &ComponentNode) -> String {
    for kind in ["label", "title", "text", "value"] {
        if let Some(t) = item(comp, kind) {
            if !t.is_empty() {
                return esc(t);
            }
        }
    }
    esc(&comp.name)
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

    fn stub(style: &str, label: &str) -> ComponentNode {
        ComponentNode {
            name: "Badge".into(),
            layout: Some("inline".into()),
            style: Some(style.into()),
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

    #[test]
    fn default_variant_is_default_not_primary() {
        let html = render(&stub("badge", "New"));
        assert!(html.starts_with("<span"));
        assert!(html.contains("data-slot=\"badge\""));
        assert!(html.contains("data-variant=\"default\""));
        assert!(html.contains("New"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn primary_from_style() {
        let html = render(&stub("badge+primary", "Beta"));
        assert!(html.contains("data-variant=\"primary\""));
    }

    #[test]
    fn destructive_from_style() {
        let html = render(&stub("badge+destructive", "Failed"));
        assert!(html.contains("data-variant=\"destructive\""));
    }
}
