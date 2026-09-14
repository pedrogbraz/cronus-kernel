//! Dedicated Progress renderer. DOM matches React/Radix Root:
//! `<div data-slot="progress" role="progressbar">`, not HTML `<progress>`.
//! The Radix Indicator carries no `data-slot` in React (only `data-state`,
//! `data-value`, `data-max`), so the kernel indicator doesn't either — CSS
//! targets it as `[data-slot="progress"] > div`.

use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let pct = value_of(comp);
    let now = fmt_num(pct);
    let remain = fmt_num(100.0 - pct);
    let state = if pct >= 100.0 { "complete" } else { "loading" };
    let aria = aria_label(comp)
        .map(|a| format!(" aria-label=\"{}\"", crate::cronus_ui_kit::esc(a)))
        .unwrap_or_default();
    format!(
        "<div data-slot=\"progress\" role=\"progressbar\" aria-valuenow=\"{now}\" aria-valuemin=\"0\" aria-valuemax=\"100\" aria-valuetext=\"{now}%\" data-state=\"{state}\" data-value=\"{now}\" data-max=\"100\"{aria}><div data-state=\"{state}\" data-value=\"{now}\" data-max=\"100\" style=\"transform:translateX(-{remain}%)\"></div></div>"
    )
}

fn aria_label(comp: &ComponentNode) -> Option<&str> {
    comp.props
        .get("aria-label")
        .map(String::as_str)
        .or_else(|| {
            comp.items
                .iter()
                .find_map(|i| i.config.get("aria-label").map(String::as_str))
        })
        .filter(|s| !s.is_empty())
}

fn value_of(comp: &ComponentNode) -> f64 {
    if let Some(v) = comp.props.get("value").and_then(|s| parse_num(s)) {
        return clamp(v);
    }
    for i in &comp.items {
        if let Some(v) = parse_num(&i.text) {
            return clamp(v);
        }
    }
    for i in &comp.items {
        if let Some(v) = i.config.get("value").and_then(|s| parse_num(s)) {
            return clamp(v);
        }
    }
    0.0
}

fn parse_num(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    s.parse().ok()
}

fn clamp(v: f64) -> f64 {
    v.clamp(0.0, 100.0)
}

fn fmt_num(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn stub() -> ComponentNode {
        ComponentNode {
            name: "Progress".into(),
            layout: Some("stack".into()),
            style: Some("progress".into()),
            items: vec![ComponentItemNode {
                item_type: "label".into(),
                text: "Upload".into(),
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

    fn assert_progressbar(html: &str, now: &str, remain: &str) {
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"progress\""));
        assert!(html.contains("role=\"progressbar\""));
        assert!(html.contains(&format!("aria-valuenow=\"{now}\"")));
        assert!(html.contains("aria-valuemin=\"0\""));
        assert!(html.contains("aria-valuemax=\"100\""));
        assert!(!html.contains("data-slot=\"progress-indicator\""));
        assert!(html.contains(&format!("data-value=\"{now}\" data-max=\"100\" style=\"transform:translateX(-{remain}%)\"")));
        assert!(!html.contains("<progress"));
        assert!(!html.contains("data-slot=\"progress-control\""));
    }

    #[test]
    fn root_is_div_progressbar_not_html_progress() {
        let html = render(&stub());
        assert_progressbar(&html, "0", "100");
        assert_eq!(
            html,
            "<div data-slot=\"progress\" role=\"progressbar\" aria-valuenow=\"0\" aria-valuemin=\"0\" aria-valuemax=\"100\" aria-valuetext=\"0%\" data-state=\"loading\" data-value=\"0\" data-max=\"100\"><div data-state=\"loading\" data-value=\"0\" data-max=\"100\" style=\"transform:translateX(-100%)\"></div></div>"
        );
    }

    #[test]
    fn indicator_has_no_slot_and_aria_label_from_item_config() {
        // Wave 1t: React's Radix Indicator exposes no data-slot; the emitter
        // attaches `aria-label:"…"` to the label item's config.
        let mut c = stub();
        c.items[0].config.insert("aria-label".into(), "Upload progress".into());
        c.props.insert("value".into(), "50".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"progress\" role=\"progressbar\" aria-valuenow=\"50\" aria-valuemin=\"0\" aria-valuemax=\"100\" aria-valuetext=\"50%\" data-state=\"loading\" data-value=\"50\" data-max=\"100\" aria-label=\"Upload progress\"><div data-state=\"loading\" data-value=\"50\" data-max=\"100\" style=\"transform:translateX(-50%)\"></div></div>"
        );
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"progress\"] > div {"));
        assert!(!css.contains("[data-slot=\"progress-indicator\"]"));
    }

    #[test]
    fn value_from_props() {
        let mut c = stub();
        c.props.insert("value".into(), "40".into());
        let html = render(&c);
        assert_progressbar(&html, "40", "60");
    }

    #[test]
    fn value_from_item_text_number() {
        let mut c = stub();
        c.items[0].item_type = "value".into();
        c.items[0].text = "75".into();
        let html = render(&c);
        assert_progressbar(&html, "75", "25");
    }

    #[test]
    fn value_from_item_config() {
        let mut c = stub();
        c.items[0].config.insert("value".into(), "10".into());
        let html = render(&c);
        assert_progressbar(&html, "10", "90");
    }

    #[test]
    fn skips_interact_html_progress() {
        let html = render(&stub());
        let interact = crate::cronus_ui_interact::render("progress", &stub()).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<progress"));
        assert!(interact.contains("max=\"100\""));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("{ value }"));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"progress\"]"));
        assert!(css.contains("height: 0.5rem"));
        assert!(css.contains("width: 100%"));
        assert!(css.contains("border-radius: 9999px"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("zinc-"));
    }
}
