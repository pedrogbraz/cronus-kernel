//! Dedicated ColorPicker renderer. DOM matches React's closed ColorPicker: a
//! single outline `<button data-slot="color-picker-trigger">` holding the
//! `<span data-slot="color-picker-swatch">` tile and the value text (React has
//! no wrapper element). React paints the tile with an inline `style`; the kernel
//! emits `data-color` and the chrome paints it with typed `attr()`.
//! Opening the editor (saturation area, hue slider, L/C/H inputs, presets) needs
//! JS, so the trigger is React's native button rendered `disabled` with React's
//! idle look (not dimmed); a real `disabled` prop adds `data-disabled` and dims.
//! Not interact `input("color-picker", "color")` as the only control.

use crate::cronus_ui_kit::{esc, item, label_of};
use crate::parser::ComponentNode;

const DEFAULT_VALUE: &str = "oklch(0.62 0.21 256)";

pub fn render(comp: &ComponentNode) -> String {
    let value = value_of(comp);
    let name = name_of(comp);
    let mut btn = format!(
        "type=\"button\" data-slot=\"color-picker-trigger\" data-variant=\"outline\" aria-label=\"{name}: {value}\" aria-haspopup=\"dialog\" aria-expanded=\"false\" data-state=\"closed\""
    );
    if flag(comp, "disabled") {
        btn.push_str(" data-disabled=\"\"");
    }
    btn.push_str(" disabled");
    format!(
        "<button {btn}><span aria-hidden=\"true\" data-slot=\"color-picker-swatch\" data-color=\"{value}\"></span><span>{value}</span></button>"
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

fn name_of(comp: &ComponentNode) -> String {
    if let Some(v) = attr(comp, "aria-label").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    let label = label_of(comp);
    if label.is_empty() || label == "color-picker" {
        "Color".into()
    } else {
        label
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
    fn root_is_single_disabled_trigger_button() {
        let html = render(&stub("color-picker", "Accent"));
        assert!(!html.contains("data-slot=\"color-picker\""), "React has no wrapper slot: {html}");
        assert!(!html.contains("color-picker-content"), "editor needs JS: {html}");
        assert!(!html.contains("color-picker-swatch-button"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<button type=\"button\" data-slot=\"color-picker-trigger\" data-variant=\"outline\" aria-label=\"Accent: oklch(0.62 0.21 256)\" aria-haspopup=\"dialog\" aria-expanded=\"false\" data-state=\"closed\" disabled><span aria-hidden=\"true\" data-slot=\"color-picker-swatch\" data-color=\"oklch(0.62 0.21 256)\"></span><span>oklch(0.62 0.21 256)</span></button>"
        );
    }

    #[test]
    fn value_and_aria_label_name_trigger() {
        let mut c = stub("color-picker", "Accent");
        c.props.insert("value".into(), "oklch(0.72 0.19 145)".into());
        c.props.insert("aria-label".into(), "Color".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Color: oklch(0.72 0.19 145)\""));
        assert!(html.contains("data-color=\"oklch(0.72 0.19 145)\""));
        assert!(html.contains("<span>oklch(0.72 0.19 145)</span>"));
        reject_interact(&html);
    }

    #[test]
    fn disabled_prop_marks_data_disabled() {
        let mut c = stub("color-picker", "Accent");
        c.props.insert("disabled".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(" data-disabled=\"\" disabled>"));
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
        assert!(css.contains("[data-slot=\"color-picker-trigger\"]"));
        assert!(css.contains("[data-slot=\"color-picker-swatch\"]"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }

    /// Wave 1t geometry: outline Button `h-10 px-4 w-full justify-start gap-2`
    /// at text-sm/20px; 20px rounded-md bordered tile painted from `data-color`
    /// (React paints oklch(0.62 0.21 256), not the primary token).
    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"color-picker-trigger\"] {\n  display: inline-flex; align-items: center; justify-content: flex-start; gap: 0.5rem;\n  width: 100%; height: 2.5rem; padding: 0 1rem; box-sizing: border-box; white-space: nowrap;"
        ));
        assert!(css.contains("background-color: attr(data-color type(<color>), var(--cronus-primary));"));
        assert!(css.contains("[data-slot=\"color-picker-trigger\"][data-disabled] { opacity: 0.5; }"));
        assert!(!css.contains("[data-slot=\"color-picker\"] {"));
        assert!(!css.contains("[data-slot=\"color-picker-swatch-button\"]"));
    }
}
