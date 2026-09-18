//! Dedicated Rating renderer. DOM matches React:
//! `<div data-slot="rating" role="slider">` plus five
//! `<span data-slot="rating-item"><span data-slot="rating-star">` stars, each an
//! outline lucide star with a clipped, warning-filled overlay star on top.
//! React sizes the overlay with an inline `style="width:N%"`; the kernel emits
//! no inline styles, so `data-state` on the item drives the overlay width in CSS.
//! Interactive by default (one radio per star); `readOnly` / `readonly` /
//! `read-only` keeps the static half-star display. Not `role="radiogroup"`.

use crate::cronus_ui_kit::{item, label_of};
use crate::parser::ComponentNode;

const MAX: u32 = 5;

/// lucide `star`, stroke 2, `currentColor` (React `<Star />`).
const STAR: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let now = value_of(comp);
    let label = label_of(comp);
    let size = crate::cronus_ui_kit::choice(comp, "size", &["sm", "md", "lg"]).unwrap_or("md");
    // React default is interactive. Read-only (half-star demos) when the author
    // sets `readOnly` / `readonly` / `read-only`.
    let read_only = crate::cronus_ui_kit::flag(comp, "readOnly")
        || crate::cronus_ui_kit::flag(comp, "readonly")
        || crate::cronus_ui_kit::flag(comp, "read-only");
    let class = if size == "md" {
        String::new()
    } else {
        format!(" class=\"s-{size}\"")
    };
    let star = |i: u32| {
        let fill = (now - (i - 1) as f64).clamp(0.0, 1.0);
        let state = if fill >= 1.0 {
            "on"
        } else if fill > 0.0 {
            "half"
        } else {
            "off"
        };
        format!(
            "<span aria-hidden=\"true\" data-slot=\"rating-item\" data-state=\"{state}\"><span data-slot=\"rating-star\">{STAR}<span>{STAR}</span></span></span>"
        )
    };
    let text = fmt(now);
    if read_only {
        let items: String = (1..=MAX).map(star).collect();
        return format!(
            "<div data-slot=\"rating\"{class} role=\"slider\" aria-label=\"{label}\" aria-valuemin=\"0\" aria-valuemax=\"{MAX}\" aria-valuenow=\"{text}\" aria-valuetext=\"{text} out of {MAX}\" aria-readonly=\"true\" data-readonly=\"\">{items}</div>"
        );
    }
    // Interactive: zero JS, one radio per star in a label; CSS fills up to the
    // checked (or hovered) star. React's items stay decorative.
    let name = crate::cronus_ui_kit::instance_id(comp, "rating");
    let items: String = (1..=MAX)
        .map(|i| {
            let checked = if now.round() as u32 == i { " checked" } else { "" };
            format!(
                "<label><input type=\"radio\" name=\"{name}\" value=\"{i}\" aria-label=\"{i} of {MAX}\"{checked}>{}</label>",
                star(i)
            )
        })
        .collect();
    format!(
        "<div data-slot=\"rating\"{class} role=\"slider\" aria-label=\"{label}\" aria-valuemin=\"0\" aria-valuemax=\"{MAX}\" aria-valuenow=\"{text}\" aria-valuetext=\"{text} out of {MAX}\" tabindex=\"-1\">{items}</div>"
    )
}

fn fmt(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

fn value_of(comp: &ComponentNode) -> f64 {
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
    0.0
}

/// Whole or half stars (`4.5`), clamped to `0..=5`.
fn parse_stars(raw: Option<&str>) -> Option<f64> {
    let s = raw?.trim();
    if s.is_empty() {
        return None;
    }
    let n: f64 = s.parse().ok()?;
    Some((n.clamp(0.0, MAX as f64) * 2.0).round() / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn reject_js(html: &str) {
        assert!(!html.contains("role=\"radiogroup\""));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
    }

    fn assert_stars(html: &str, now: u32, label: &str) {
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"rating\""));
        assert!(html.contains("role=\"slider\""));
        assert!(html.contains("aria-valuemin=\"0\""));
        assert!(html.contains("aria-valuemax=\"5\""));
        assert!(html.contains(&format!("aria-valuenow=\"{now}\"")));
        assert!(html.contains(&format!("aria-valuetext=\"{now} out of 5\"")));
        assert!(html.contains(&format!("aria-label=\"{label}\"")));
        assert_eq!(html.matches("data-slot=\"rating-item\"").count(), 5);
        assert_eq!(html.matches("data-slot=\"rating-star\"").count(), 5);
        assert_eq!(html.matches("<svg ").count(), 10);
        let on = html.matches("data-state=\"on\"").count();
        let off = html.matches("data-state=\"off\"").count();
        assert_eq!(on, now as usize);
        assert_eq!(off, 5 - now as usize);
        reject_js(html);
    }

    fn star(state: &str) -> String {
        format!(
            "<span aria-hidden=\"true\" data-slot=\"rating-item\" data-state=\"{state}\"><span data-slot=\"rating-star\">{STAR}<span>{STAR}</span></span></span>"
        )
    }

    #[test]
    fn root_is_interactive_radios_not_radiogroup() {
        crate::cronus_ui_kit::reset_instance_ids();
        let html = render(&stub("rating", "Rating"));
        assert_stars(&html, 0, "Rating");
        assert_eq!(html.matches("type=\"radio\"").count(), 5);
        assert_eq!(html.matches("<label>").count(), 5);
        assert!(html.contains("tabindex=\"-1\""));
        let item = format!(
            "<label><input type=\"radio\" name=\"cui-rating-rating\" value=\"1\" aria-label=\"1 of 5\">{}</label><label><input type=\"radio\" name=\"cui-rating-rating\" value=\"2\" aria-label=\"2 of 5\">{}</label><label><input type=\"radio\" name=\"cui-rating-rating\" value=\"3\" aria-label=\"3 of 5\">{}</label><label><input type=\"radio\" name=\"cui-rating-rating\" value=\"4\" aria-label=\"4 of 5\">{}</label><label><input type=\"radio\" name=\"cui-rating-rating\" value=\"5\" aria-label=\"5 of 5\">{}</label>",
            star("off"),
            star("off"),
            star("off"),
            star("off"),
            star("off"),
        );
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"rating\" role=\"slider\" aria-label=\"Rating\" aria-valuemin=\"0\" aria-valuemax=\"5\" aria-valuenow=\"0\" aria-valuetext=\"0 out of 5\" tabindex=\"-1\">{item}</div>"
            )
        );
    }

    #[test]
    fn value_from_props() {
        let mut c = stub("rating", "Rating");
        c.props.insert("value".into(), "3".into());
        let html = render(&c);
        assert_stars(&html, 3, "Rating");
        assert!(html.contains("value=\"3\" aria-label=\"3 of 5\" checked>"));
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
        assert_stars(&html, 4, "Rating");
    }

    #[test]
    fn value_defaults_to_zero() {
        let html = render(&stub("rating", "Stars"));
        assert_stars(&html, 0, "Stars");
        assert!(html.contains("type=\"radio\""));
    }

    #[test]
    fn skips_radiogroup_and_inline_js() {
        let html = render(&stub("rating", "Rating"));
        assert!(!html.contains("★"));
        assert!(!html.contains("role=\"radiogroup\""));
        reject_js(&html);
    }

    #[test]
    fn read_only_keeps_half_star_and_no_radios() {
        let mut c = stub("rating", "Rated 4.5 out of 5");
        c.props.insert("value".into(), "4.5".into());
        c.props.insert("readOnly".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("aria-valuenow=\"4.5\""));
        assert!(html.contains("data-state=\"half\""));
        assert_eq!(html.matches("data-state=\"on\"").count(), 4);
        assert_eq!(html.matches("data-state=\"off\"").count(), 0);
        assert!(html.contains("aria-readonly=\"true\""));
        assert!(html.contains("data-readonly=\"\""));
        assert!(!html.contains("<input"));
        assert!(!html.contains("type=\"radio\""));
        assert!(!html.contains("<label"));
        reject_js(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"rating\"]"));
        assert!(css.contains("[data-slot=\"rating-item\"]"));
        assert!(css.contains("[data-slot=\"rating-star\"]"));
        assert!(css.contains("var(--cronus-warning"));
        assert!(!css.contains("zinc-"));
    }

    /// Wave 1t geometry: 20px svg stars (`[&_svg]:size-5`), gap-1, rounded-md
    /// root; stars are SVG (not a 20px `★` glyph with line-height 1), items keep
    /// the inherited 16px/24px type; overlay clipped to 100% / 0% by state.
    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"rating\"] {\n  display: inline-flex; align-items: center; gap: 0.25rem;\n  border-radius: var(--cronus-radius-md); outline: none;\n}"
        ));
        assert!(css.contains("[data-slot=\"rating\"] svg { width: 1.25rem; height: 1.25rem; }"));
        assert!(css.contains(
            "[data-slot=\"rating-item\"][data-state=\"on\"] [data-slot=\"rating-star\"] > span { width: 100%; }"
        ));
        assert!(!css.contains("content: \"★\""));
    }
}
