//! Dedicated Toggle renderer. DOM matches React/Radix Toggle: a `<button>`.
//! React does not emit `data-variant` or `data-size`.
//! Not interact `switch()` (`<label data-slot="toggle"><input type="checkbox">`).

use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let on = pressed(comp);
    let state = if on { "on" } else { "off" };
    let aria = if on { "true" } else { "false" };
    format!(
        "<button type=\"button\" data-slot=\"toggle\" data-state=\"{state}\" aria-pressed=\"{aria}\">{label}</button>"
    )
}

fn pressed(comp: &ComponentNode) -> bool {
    if let Some(v) = comp.props.get("pressed") {
        return is_true(v);
    }
    if let Some(v) = comp.props.get("on") {
        return is_true(v);
    }
    if comp.items.iter().any(|i| {
        i.config.get("pressed").map(|s| is_true(s)).unwrap_or(false)
            || i.config.get("on").map(|s| is_true(s)).unwrap_or(false)
    }) {
        return true;
    }
    comp.style
        .as_deref()
        .unwrap_or("")
        .split('+')
        .any(|part| part.trim() == "on")
}

fn is_true(raw: &str) -> bool {
    matches!(raw, "true" | "on" | "1")
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
            name: "Toggle".into(),
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
    fn root_is_button_not_switch_label() {
        let html = render(&stub("toggle", "Bold"));
        assert!(html.starts_with("<button "));
        assert!(html.contains("type=\"button\""));
        assert!(html.contains("data-slot=\"toggle\""));
        assert!(html.contains("data-state=\"off\""));
        assert!(html.contains("aria-pressed=\"false\""));
        assert!(html.contains("Bold"));
        assert!(!html.contains("data-variant"));
        assert!(!html.contains("data-size"));
        assert!(!html.contains("<label"));
        assert!(!html.contains("type=\"checkbox\""));
        assert!(!html.contains("data-slot=\"toggle-control\""));
        assert!(!html.contains("style="));
    }

    #[test]
    fn pressed_prop_turns_on() {
        let mut c = stub("toggle", "Bold");
        c.props.insert("pressed".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("data-state=\"on\""));
        assert!(html.contains("aria-pressed=\"true\""));
    }

    #[test]
    fn pressed_colon_pair_on_item() {
        let mut c = stub("toggle", "Bold");
        c.items[0].config.insert("pressed".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("data-state=\"on\""));
        assert!(html.contains("aria-pressed=\"true\""));
    }

    #[test]
    fn style_on_turns_on() {
        let html = render(&stub("toggle+on", "Italic"));
        assert!(html.contains("data-state=\"on\""));
        assert!(html.contains("aria-pressed=\"true\""));
        assert!(!html.contains("data-variant"));
        assert!(!html.contains("data-size"));
    }

    #[test]
    fn skips_interact_switch() {
        let html = render(&stub("toggle", "Bold"));
        let interact = crate::cronus_ui_interact::render("toggle", &stub("toggle", "Bold")).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<label data-slot=\"toggle\""));
        assert!(interact.contains("data-slot=\"toggle-control\""));
        assert!(interact.contains("type=\"checkbox\""));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"toggle\"]"));
        assert!(css.contains("height: 2.5rem"));
        assert!(css.contains("padding: 0 0.75rem"));
        assert!(css.contains("[data-slot=\"toggle\"][data-state=\"on\"]"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(!css.contains("zinc-"));
    }
}
