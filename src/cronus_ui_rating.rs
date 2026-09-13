//! Dedicated Rating renderer. DOM matches React:
//! `<div data-slot="rating" role="slider">` plus five `<span data-slot="rating-item">`.
//! Not interact `role="radiogroup"` with hidden `<input type="radio">` stars.

use crate::cronus_ui_kit::{item, label_of};
use crate::parser::ComponentNode;

const MAX: u32 = 5;

pub fn render(comp: &ComponentNode) -> String {
    let now = value_of(comp);
    let label = label_of(comp);
    let mut items = String::new();
    for i in 1..=MAX {
        let state = if i <= now { "on" } else { "off" };
        items.push_str(&format!(
            "<span data-slot=\"rating-item\" aria-hidden=\"true\" data-state=\"{state}\"></span>"
        ));
    }
    format!(
        "<div data-slot=\"rating\" role=\"slider\" aria-valuemin=\"0\" aria-valuemax=\"{MAX}\" aria-valuenow=\"{now}\" aria-label=\"{label}\">{items}</div>"
    )
}

fn value_of(comp: &ComponentNode) -> u32 {
    if let Some(v) = parse_stars(comp.props.get("value").map(String::as_str)) {
        return v;
    }
    if let Some(v) = parse_stars(item(comp, "value")) {
        return v;
    }
    for i in &comp.items {
        if let Some(v) = parse_stars(Some(i.text.as_str())) {
            return v;
        }
        if let Some(v) = parse_stars(i.config.get("value").map(String::as_str)) {
            return v;
        }
    }
    0
}

fn parse_stars(raw: Option<&str>) -> Option<u32> {
    let s = raw?.trim();
    if s.is_empty() {
        return None;
    }
    let n: f64 = s.parse().ok()?;
    Some(n.clamp(0.0, MAX as f64) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn reject_interact(html: &str) {
        assert!(!html.contains("role=\"radiogroup\""));
        assert!(!html.contains("<input"));
        assert!(!html.contains("type=\"radio\""));
        assert!(!html.contains("<label"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
    }

    fn assert_slider(html: &str, now: u32, label: &str) {
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"rating\""));
        assert!(html.contains("role=\"slider\""));
        assert!(html.contains("aria-valuemin=\"0\""));
        assert!(html.contains("aria-valuemax=\"5\""));
        assert!(html.contains(&format!("aria-valuenow=\"{now}\"")));
        assert!(html.contains(&format!("aria-label=\"{label}\"")));
        assert_eq!(html.matches("data-slot=\"rating-item\"").count(), 5);
        assert_eq!(html.matches("aria-hidden=\"true\"").count(), 5);
        let on = html.matches("data-state=\"on\"").count();
        let off = html.matches("data-state=\"off\"").count();
        assert_eq!(on, now as usize);
        assert_eq!(off, 5 - now as usize);
        reject_interact(html);
    }

    #[test]
    fn root_is_slider_of_spans_not_radiogroup() {
        let html = render(&stub("rating", "Rating"));
        assert_slider(&html, 0, "Rating");
        assert_eq!(
            html,
            "<div data-slot=\"rating\" role=\"slider\" aria-valuemin=\"0\" aria-valuemax=\"5\" aria-valuenow=\"0\" aria-label=\"Rating\"><span data-slot=\"rating-item\" aria-hidden=\"true\" data-state=\"off\"></span><span data-slot=\"rating-item\" aria-hidden=\"true\" data-state=\"off\"></span><span data-slot=\"rating-item\" aria-hidden=\"true\" data-state=\"off\"></span><span data-slot=\"rating-item\" aria-hidden=\"true\" data-state=\"off\"></span><span data-slot=\"rating-item\" aria-hidden=\"true\" data-state=\"off\"></span></div>"
        );
    }

    #[test]
    fn value_from_props() {
        let mut c = stub("rating", "Rating");
        c.props.insert("value".into(), "3".into());
        let html = render(&c);
        assert_slider(&html, 3, "Rating");
    }

    #[test]
    fn value_from_item_number() {
        let mut c = stub("rating", "Rating");
        c.items.push(crate::parser::ComponentItemNode {
            item_type: "value".into(),
            text: "4".into(),
            link: None,
            tone: None,
            config: std::collections::HashMap::new(),
        });
        let html = render(&c);
        assert_slider(&html, 4, "Rating");
    }

    #[test]
    fn value_defaults_to_zero() {
        let html = render(&stub("rating", "Stars"));
        assert_slider(&html, 0, "Stars");
    }

    #[test]
    fn skips_interact_radio_stars() {
        let html = render(&stub("rating", "Rating"));
        let interact =
            crate::cronus_ui_interact::render("rating", &stub("rating", "Rating")).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("role=\"radiogroup\""));
        assert!(interact.contains("<input type=\"radio\""));
        assert!(interact.contains("★"));
        assert!(!html.contains("★"));
        assert!(!html.contains("role=\"radiogroup\""));
        reject_interact(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"rating\"]"));
        assert!(css.contains("[data-slot=\"rating-item\"]"));
        assert!(css.contains("display: inline-flex"));
        assert!(css.contains("align-items: center"));
        assert!(css.contains("content: \"★\""));
        assert!(css.contains("var(--cronus-warning"));
        assert!(!css.contains("zinc-"));
    }
}
