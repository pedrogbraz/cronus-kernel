//! Dedicated Alert renderer. DOM matches React:
//! `<div data-slot="alert" role="status">` (role=`alert` only if destructive)
//! with `<div data-slot="alert-title">` and optional `alert-description`.
//! Not the interact SURF box wrapping raw `<div>text</div>` with no title slot.

use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let role = if is_destructive(comp) {
        "alert"
    } else {
        "status"
    };
    let (title, descs) = title_and_descriptions(comp);
    let mut inner = format!("<div data-slot=\"alert-title\">{title}</div>");
    for d in descs {
        inner.push_str(&format!("<div data-slot=\"alert-description\">{d}</div>"));
    }
    format!("<div data-slot=\"alert\" role=\"{role}\">{inner}</div>")
}

fn is_destructive(comp: &ComponentNode) -> bool {
    if let Some(v) = comp.props.get("variant") {
        if matches!(v.as_str(), "destructive" | "danger") {
            return true;
        }
    }
    if comp.items.iter().any(|i| {
        i.config
            .get("variant")
            .map(|s| matches!(s.as_str(), "destructive" | "danger"))
            .unwrap_or(false)
    }) {
        return true;
    }
    comp.style
        .as_deref()
        .unwrap_or("")
        .split('+')
        .any(|part| matches!(part.trim(), "destructive" | "danger"))
}

fn title_and_descriptions(comp: &ComponentNode) -> (String, Vec<String>) {
    for kind in ["label", "title", "text", "value"] {
        if let Some((idx, t)) = comp
            .items
            .iter()
            .enumerate()
            .find(|(_, i)| i.item_type == kind && !i.text.is_empty())
        {
            return (esc(&t.text), extra_descs(comp, idx));
        }
    }
    (esc(&comp.name), extra_descs(comp, usize::MAX))
}

fn extra_descs(comp: &ComponentNode, skip: usize) -> Vec<String> {
    comp.items
        .iter()
        .enumerate()
        .filter(|(i, item)| *i != skip && !item.text.is_empty())
        .map(|(_, item)| esc(&item.text))
        .collect()
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
            name: "Alert".into(),
            layout: Some("stack".into()),
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

    fn reject_interact(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("padding:0.85rem 1rem"));
        assert!(!html.contains("background:var(--cronus-surface-raised"));
        assert!(!html.contains("color:var(--cronus-fg);font-family:var(--cronus-font-sans"));
        assert!(!html.contains("<div>Demo</div>"));
        assert!(html.contains("data-slot=\"alert-title\""));
        assert!(!html.contains("v-data="));
    }

    #[test]
    fn root_is_div_with_title_slot_not_surf_box() {
        let html = render(&stub("alert", "Saved"));
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"alert\""));
        assert!(html.contains("role=\"status\""));
        assert!(html.contains("<div data-slot=\"alert-title\">Saved</div>"));
        assert!(!html.contains("data-variant"));
        assert!(!html.contains("role=\"alert\""));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"alert\" role=\"status\"><div data-slot=\"alert-title\">Saved</div></div>"
        );
    }

    #[test]
    fn extra_text_is_description() {
        let mut c = stub("alert", "Payment failed");
        c.items.push(ComponentItemNode {
            item_type: "text".into(),
            text: "Your card was declined.".into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        });
        let html = render(&c);
        assert!(html.contains("data-slot=\"alert-title\">Payment failed<"));
        assert!(html.contains("data-slot=\"alert-description\">Your card was declined.<"));
        reject_interact(&html);
    }

    #[test]
    fn destructive_sets_role_alert() {
        let html = render(&stub("alert+destructive", "Outage"));
        assert!(html.contains("role=\"alert\""));
        assert!(!html.contains("role=\"status\""));
        assert!(!html.contains("data-variant"));
        reject_interact(&html);
    }

    #[test]
    fn destructive_from_props() {
        let mut c = stub("alert", "Outage");
        c.props.insert("variant".into(), "destructive".into());
        let html = render(&c);
        assert!(html.contains("role=\"alert\""));
        assert!(!html.contains("data-variant"));
    }

    #[test]
    fn skips_interact_surf_box() {
        let html = render(&stub("alert", "Saved"));
        let interact = crate::cronus_ui_interact::render("alert", &stub("alert", "Saved")).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("role=\"status\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("<div>Saved</div>"));
        assert!(!interact.contains("data-slot=\"alert-title\""));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"alert\"]"));
        assert!(css.contains("[data-slot=\"alert-title\"]"));
        assert!(css.contains("[data-slot=\"alert-description\"]"));
        assert!(css.contains("border-radius: var(--cronus-radius-lg)"));
        assert!(css.contains("padding: 0.75rem 1rem"));
        assert!(css.contains("font-size: 0.875rem"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(!css.contains("zinc-"));
    }
}
