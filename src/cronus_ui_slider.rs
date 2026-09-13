//! Dedicated Slider renderer. DOM matches React/Radix Root (a **span**):
//! `<span data-slot="slider">` + track/range + thumb `role="slider"`.
//! Not the interact `<label data-slot="slider">…<input type="range" data-slot="slider-control">`.

use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let pct = value_of(comp);
    let now = fmt_num(pct);
    let mut thumb = format!(
        "data-slot=\"slider-thumb\" role=\"slider\" aria-valuenow=\"{now}\" aria-valuemin=\"0\" aria-valuemax=\"100\""
    );
    if let Some(label) = aria_label_of(comp) {
        thumb.push_str(&format!(" aria-label=\"{}\"", esc(label)));
    }
    format!(
        "<span data-slot=\"slider\"><span data-slot=\"slider-track\"><span data-slot=\"slider-range\" style=\"width:{now}%\"></span></span><span {thumb} style=\"left:{now}%\"></span></span>"
    )
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

fn aria_label_of(comp: &ComponentNode) -> Option<&str> {
    if let Some(v) = comp.props.get("aria-label") {
        if !v.is_empty() {
            return Some(v.as_str());
        }
    }
    if let Some(v) = comp
        .items
        .iter()
        .find_map(|i| i.config.get("aria-label").map(String::as_str))
    {
        if !v.is_empty() {
            return Some(v);
        }
    }
    for kind in ["label", "title", "text"] {
        if let Some(t) = item(comp, kind) {
            if !t.is_empty() && parse_num(t).is_none() {
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

fn item<'a>(comp: &'a ComponentNode, kind: &str) -> Option<&'a str> {
    comp.items
        .iter()
        .find(|i| i.item_type == kind)
        .map(|i| i.text.as_str())
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
            name: "Volume".into(),
            layout: Some("stack".into()),
            style: Some("slider".into()),
            items: vec![ComponentItemNode {
                item_type: "label".into(),
                text: "Volume".into(),
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
        assert!(!html.contains("<label"));
        assert!(!html.contains("<input"));
        assert!(!html.contains("type=\"range\""));
        assert!(!html.contains("data-slot=\"slider-control\""));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("{ value }"));
    }

    fn assert_slider(html: &str, now: &str, label: &str) {
        assert!(html.starts_with("<span "));
        assert!(html.contains("data-slot=\"slider\""));
        assert!(html.contains("data-slot=\"slider-track\""));
        assert!(html.contains("data-slot=\"slider-range\""));
        assert!(html.contains(&format!("style=\"width:{now}%\"")));
        assert!(html.contains("data-slot=\"slider-thumb\""));
        assert!(html.contains("role=\"slider\""));
        assert!(html.contains(&format!("aria-valuenow=\"{now}\"")));
        assert!(html.contains("aria-valuemin=\"0\""));
        assert!(html.contains("aria-valuemax=\"100\""));
        assert!(html.contains(&format!("aria-label=\"{label}\"")));
        assert!(html.contains(&format!("style=\"left:{now}%\"")));
        reject_interact(html);
    }

    #[test]
    fn root_is_span_not_label_range() {
        let html = render(&stub());
        assert_slider(&html, "0", "Volume");
        assert_eq!(
            html,
            "<span data-slot=\"slider\"><span data-slot=\"slider-track\"><span data-slot=\"slider-range\" style=\"width:0%\"></span></span><span data-slot=\"slider-thumb\" role=\"slider\" aria-valuenow=\"0\" aria-valuemin=\"0\" aria-valuemax=\"100\" aria-label=\"Volume\" style=\"left:0%\"></span></span>"
        );
    }

    #[test]
    fn value_from_props() {
        let mut c = stub();
        c.props.insert("value".into(), "40".into());
        let html = render(&c);
        assert_slider(&html, "40", "Volume");
    }

    #[test]
    fn value_from_item_text_number() {
        let mut c = stub();
        c.items[0].item_type = "value".into();
        c.items[0].text = "75".into();
        let html = render(&c);
        assert_slider(&html, "75", "Volume");
    }

    #[test]
    fn value_from_item_config() {
        let mut c = stub();
        c.items[0].config.insert("value".into(), "10".into());
        let html = render(&c);
        assert_slider(&html, "10", "Volume");
    }

    #[test]
    fn aria_label_from_props() {
        let mut c = stub();
        c.props.insert("aria-label".into(), "Brightness".into());
        let html = render(&c);
        assert_slider(&html, "0", "Brightness");
        assert!(!html.contains("aria-label=\"Volume\""));
    }

    #[test]
    fn aria_label_from_item_config() {
        let mut c = stub();
        c.items[0]
            .config
            .insert("aria-label".into(), "Opacity".into());
        let html = render(&c);
        assert_slider(&html, "0", "Opacity");
    }

    #[test]
    fn skips_interact_label_range() {
        let html = render(&stub());
        let interact = crate::cronus_ui_interact::render("slider", &stub()).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<label data-slot=\"slider\""));
        assert!(interact.contains("type=\"range\""));
        assert!(interact.contains("data-slot=\"slider-control\""));
        assert!(!html.contains("<label"));
        assert!(!html.contains("type=\"range\""));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"slider\"]"));
        assert!(css.contains("[data-slot=\"slider-track\"]"));
        assert!(css.contains("[data-slot=\"slider-range\"]"));
        assert!(css.contains("[data-slot=\"slider-thumb\"]"));
        assert!(css.contains("position: relative"));
        assert!(css.contains("display: flex"));
        assert!(css.contains("width: 100%"));
        assert!(css.contains("align-items: center"));
        assert!(css.contains("height: 0.375rem"));
        assert!(css.contains("width: 1rem; height: 1rem"));
        assert!(css.contains("border-radius: 9999px"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("zinc-"));
    }
}
