//! Dedicated ColorPicker renderer. Always-open static DOM (no JS).
//! Wrapper `data-slot="color-picker"` with
//! `<button data-slot="color-picker-trigger">` plus
//! `<div data-slot="color-picker-content">` (swatch).
//! Not interact `input("color-picker", "color")` as the only control.

use crate::cronus_ui_kit::{esc, item, label_of};
use crate::parser::ComponentNode;

const DEFAULT_VALUE: &str = "oklch(0.62 0.21 256)";

const PRESETS: &[&str] = &[
    "oklch(0.62 0.21 256)",
    "oklch(0.65 0.24 24)",
    "oklch(0.72 0.19 145)",
    "oklch(0.8 0.16 86)",
    "oklch(0.62 0.25 304)",
];

pub fn render(comp: &ComponentNode) -> String {
    let value = value_of(comp);
    let disabled = flag(comp, "disabled");
    let aria = trigger_aria(comp, &value);
    let mut btn = String::from(
        "type=\"button\" data-slot=\"color-picker-trigger\" aria-haspopup=\"dialog\"",
    );
    btn.push_str(&format!(" aria-label=\"{aria}\""));
    if disabled {
        btn.push_str(" disabled");
    }
    let swatches = PRESETS
        .iter()
        .map(|swatch| {
            let pressed = if *swatch == value.as_str() {
                "true"
            } else {
                "false"
            };
            format!(
                "<button type=\"button\" data-slot=\"color-picker-swatch-button\" aria-label=\"{swatch}\" aria-pressed=\"{pressed}\"></button>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"color-picker\"><button {btn}><span aria-hidden=\"true\" data-slot=\"color-picker-swatch\"></span><span>{value}</span></button><div data-slot=\"color-picker-content\" aria-label=\"Color picker\"><span aria-hidden=\"true\" data-slot=\"color-picker-swatch\"></span><div role=\"group\" aria-label=\"Preset colors\" data-slot=\"color-picker-swatches\">{swatches}</div></div></div>"
    )
}

fn value_of(comp: &ComponentNode) -> String {
    if let Some(v) = attr(comp, "value").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    if let Some(v) = attr(comp, "defaultValue")
        .or_else(|| attr(comp, "default-value"))
        .filter(|s| !s.is_empty())
    {
        return esc(v);
    }
    if let Some(t) = item(comp, "value").filter(|s| !s.is_empty()) {
        return esc(t);
    }
    DEFAULT_VALUE.into()
}

fn trigger_aria(comp: &ComponentNode, value: &str) -> String {
    if let Some(v) = attr(comp, "aria-label").filter(|s| !s.is_empty()) {
        return format!("{}: {value}", esc(v));
    }
    let label = label_of(comp);
    if label.is_empty() || label == "color-picker" {
        format!("Color: {value}")
    } else {
        format!("{label}: {value}")
    }
}

fn attr<'a>(comp: &'a ComponentNode, name: &str) -> Option<&'a str> {
    if let Some(v) = comp.props.get(name) {
        return Some(v.as_str());
    }
    comp.items
        .iter()
        .find_map(|i| i.config.get(name).map(String::as_str))
}

fn flag(comp: &ComponentNode, name: &str) -> bool {
    attr(comp, name).map(|s| s == "true").unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn reject_interact(html: &str) {
        assert!(!html.contains("color-picker-control"));
        assert!(!html.contains("<label"));
        assert!(!html.contains("type=\"color\""));
        assert!(!html.contains("<input"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("height:2.5rem;padding:0 0.75rem"));
    }

    #[test]
    fn root_is_wrapper_with_trigger_and_open_content() {
        let html = render(&stub("color-picker", "Accent"));
        assert!(html.starts_with("<div data-slot=\"color-picker\">"));
        assert!(html.contains("<button type=\"button\" data-slot=\"color-picker-trigger\""));
        assert!(html.contains("aria-haspopup=\"dialog\""));
        assert!(html.contains("data-slot=\"color-picker-swatch\""));
        assert!(html.contains("<span>oklch(0.62 0.21 256)</span>"));
        assert!(html.contains("data-slot=\"color-picker-content\""));
        assert!(html.contains("data-slot=\"color-picker-swatches\""));
        assert!(html.contains("data-slot=\"color-picker-swatch-button\""));
        assert!(!html.contains("type=\"color\""));
        reject_interact(&html);
    }

    #[test]
    fn value_labels_trigger() {
        let mut c = stub("color-picker", "Accent");
        c.props.insert("value".into(), "oklch(0.72 0.19 145)".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Accent: oklch(0.72 0.19 145)\""));
        assert!(html.contains("<span>oklch(0.72 0.19 145)</span>"));
        assert!(html.contains("aria-pressed=\"true\""));
        reject_interact(&html);
    }

    #[test]
    fn disabled_trigger() {
        let mut c = stub("color-picker", "Accent");
        c.props.insert("disabled".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(" disabled>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_native_color_input() {
        let c = stub("color-picker", "Accent");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("color-picker", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<label data-slot=\"color-picker\""));
        assert!(interact.contains("data-slot=\"color-picker-control\""));
        assert!(interact.contains("type=\"color\""));
        assert!(interact.contains("style="));
        assert!(!html.contains("color-picker-control"));
        assert!(!html.contains("type=\"color\""));
        assert!(html.contains("data-slot=\"color-picker-trigger\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("color-picker", "Accent"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"color-picker\"]"));
        assert!(css.contains("[data-slot=\"color-picker-trigger\"]"));
        assert!(css.contains("[data-slot=\"color-picker-content\"]"));
        assert!(css.contains("[data-slot=\"color-picker-swatch\"]"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-shadow-lg"));
        assert!(css.contains("z-index: 50"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
