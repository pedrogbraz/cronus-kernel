//! Dedicated Alert renderer. DOM matches React:
//! `<div data-slot="alert" role="status">` (role=`alert` only if destructive)
//! with `<div data-slot="alert-title">` and optional `alert-description`
//! (a `description:"…"` attribute first, then extra texts).
//! Not the interact SURF box wrapping raw `<div>text</div>` with no title slot.

use crate::cronus_ui_kit::{attr_nonempty, esc};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let role = if is_destructive(comp) {
        "alert"
    } else {
        "status"
    };
    let (title, mut descs) = title_and_descriptions(comp);
    if let Some(d) = attr_nonempty(comp, "description") {
        descs.insert(0, esc(d));
    }
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
        assert!(!html.contains("<div>Demo</div>"));
        assert!(html.contains("data-slot=\"alert-title\""));
        assert!(!html.contains("v-data="));
    }

    #[test]
    fn root_is_div_with_title_slot_not_surf_box() {
        let html = render(&stub("alert", "Saved"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"alert\" role=\"status\"><div data-slot=\"alert-title\">Saved</div></div>"
        );
    }

    #[test]
    fn description_attribute_matches_react() {
        let mut c = stub("alert", "Heads up");
        c.items[0]
            .config
            .insert("description".into(), "Your trial ends soon.".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"alert\" role=\"status\"><div data-slot=\"alert-title\">Heads up</div><div data-slot=\"alert-description\">Your trial ends soon.</div></div>"
        );
        reject_interact(&html);
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
        reject_interact(&html);
    }

    #[test]
    fn destructive_from_props() {
        let mut c = stub("alert", "Outage");
        c.props.insert("variant".into(), "destructive".into());
        assert!(render(&c).contains("role=\"alert\""));
    }

    #[test]
    fn chrome_is_token_only_and_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("grid-template-columns: 0 1fr; align-items: start; row-gap: 0.25rem;"));
        assert!(css.contains("padding: 0.75rem 1rem; font-size: 0.875rem; line-height: 1.25rem;"));
        assert!(css.contains("border-radius: var(--cronus-radius-lg)"));
        assert!(css
            .contains("[data-slot=\"alert-title\"] {\n  grid-column-start: 2; min-height: 1rem;"));
        assert!(css.contains("[data-slot=\"alert-description\"] {\n  grid-column-start: 2; display: grid; justify-items: start; gap: 0.25rem;"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(!css.contains("zinc-"));
    }
}
