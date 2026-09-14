//! Dedicated ModeToggle renderer. Static:
//! `<button type="button" data-slot="mode-toggle" data-mode="dark" aria-label="Switch to light mode">`.
//! Not interact onclick `document.documentElement.classList.toggle('dark')` + BASE/SURF styles.

use crate::cronus_ui_kit::{esc, flag};
use crate::parser::ComponentNode;

const ICON: &str = concat!(
    "<svg viewBox=\"0 0 24 24\" fill=\"none\" aria-hidden=\"true\" focusable=\"false\">",
    "<mask id=\"mode-toggle-mask\"><rect width=\"24\" height=\"24\" fill=\"white\" />",
    "<circle data-slot=\"mode-toggle-crescent\" cx=\"24\" cy=\"10\" r=\"6\" fill=\"black\" /></mask>",
    "<circle data-slot=\"mode-toggle-core\" cx=\"12\" cy=\"12\" r=\"6\" fill=\"currentColor\" mask=\"url(#mode-toggle-mask)\" />",
    "<g data-slot=\"mode-toggle-rays\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\">",
    "<line x1=\"12\" y1=\"1\" x2=\"12\" y2=\"3\" />",
    "<line x1=\"12\" y1=\"21\" x2=\"12\" y2=\"23\" />",
    "<line x1=\"1\" y1=\"12\" x2=\"3\" y2=\"12\" />",
    "<line x1=\"21\" y1=\"12\" x2=\"23\" y2=\"12\" />",
    "<line x1=\"4.22\" y1=\"4.22\" x2=\"5.64\" y2=\"5.64\" />",
    "<line x1=\"18.36\" y1=\"18.36\" x2=\"19.78\" y2=\"19.78\" />",
    "<line x1=\"4.22\" y1=\"19.78\" x2=\"5.64\" y2=\"18.36\" />",
    "<line x1=\"18.36\" y1=\"5.64\" x2=\"19.78\" y2=\"4.22\" />",
    "</g></svg>",
);

pub fn render(comp: &ComponentNode) -> String {
    let mode = mode_of(comp);
    let next = if mode == "light" { "dark" } else { "light" };
    let aria = aria_label_of(comp).unwrap_or_else(|| format!("Switch to {next} mode"));
    format!(
        "<button type=\"button\" data-slot=\"mode-toggle\" data-mode=\"{mode}\" aria-label=\"{aria}\"{dimmed} disabled>{ICON}</button>",
        dimmed = if author_disabled(comp) { " data-disabled=\"\"" } else { "" },
    )
}

/// Switching the theme needs JS, so the button is always `disabled`; it is only
/// dimmed (`data-disabled`) when the author disabled it, like React's
/// `disabled:opacity-50`.
fn author_disabled(comp: &ComponentNode) -> bool {
    flag(comp, "disabled")
}

fn mode_of(comp: &ComponentNode) -> &'static str {
    if let Some(v) = comp.props.get("mode") {
        if v == "light" {
            return "light";
        }
        if v == "dark" {
            return "dark";
        }
    }
    let style = comp.style.as_deref().unwrap_or("");
    if style.split('+').any(|part| part.trim() == "dark") {
        return "dark";
    }
    if style.split('+').any(|part| part.trim() == "light") {
        return "light";
    }
    // The audit emitter keeps only `aria-label`, which React words as the *next*
    // mode ("Switch to dark mode" while light). React's fixture defaults to light.
    if aria_label_of(comp).is_some_and(|a| a.eq_ignore_ascii_case("switch to light mode")) {
        return "dark";
    }
    "light"
}

fn aria_label_of(comp: &ComponentNode) -> Option<String> {
    if let Some(v) = comp.props.get("aria-label") {
        if !v.is_empty() {
            return Some(esc(v));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn reject_interact(html: &str) {
        assert!(!html.contains("onclick="));
        assert!(!html.contains("classList.toggle"));
        assert!(!html.contains("document.documentElement"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("<nav "));
        assert!(!html.contains("Theme</button>"));
    }

    #[test]
    fn root_is_static_light_button_not_onclick_theme() {
        let html = render(&stub("mode-toggle", "Theme"));
        assert!(html.starts_with(
            "<button type=\"button\" data-slot=\"mode-toggle\" data-mode=\"light\" aria-label=\"Switch to dark mode\" disabled>"
        ));
        assert!(!html.contains("data-disabled"));
        let mut d = stub("mode-toggle", "Theme");
        d.props.insert("disabled".into(), "true".into());
        assert!(
            render(&d).contains("aria-label=\"Switch to dark mode\" data-disabled=\"\" disabled>")
        );
        assert!(html.ends_with("</button>"));
        assert!(html.contains("data-slot=\"mode-toggle-core\""));
        assert!(html.contains("data-slot=\"mode-toggle-rays\""));
        reject_interact(&html);
    }

    #[test]
    fn emitted_fixture_aria_label_keeps_light_mode() {
        let mut c = stub("mode-toggle", "Switch to dark mode");
        c.props
            .insert("aria-label".into(), "Switch to dark mode".into());
        let html = render(&c);
        assert!(html.contains("data-mode=\"light\" aria-label=\"Switch to dark mode\""));
        reject_interact(&html);
    }

    #[test]
    fn dark_from_props_style_or_aria_label() {
        let mut c = stub("mode-toggle", "Theme");
        c.props.insert("mode".into(), "dark".into());
        assert!(render(&c).contains("data-mode=\"dark\" aria-label=\"Switch to light mode\""));
        let s = render(&stub("mode-toggle+dark", "Theme"));
        assert!(s.contains("data-mode=\"dark\""));
        let mut a = stub("mode-toggle", "Theme");
        a.props
            .insert("aria-label".into(), "Switch to light mode".into());
        assert!(render(&a).contains("data-mode=\"dark\" aria-label=\"Switch to light mode\""));
    }

    #[test]
    fn light_from_props() {
        let mut c = stub("mode-toggle", "Theme");
        c.props.insert("mode".into(), "light".into());
        let html = render(&c);
        assert!(html.contains("data-mode=\"light\""));
        assert!(html.contains("aria-label=\"Switch to dark mode\""));
        assert!(!html.contains("data-mode=\"dark\""));
        reject_interact(&html);
    }

    #[test]
    fn light_from_style() {
        let html = render(&stub("mode-toggle+light", "Theme"));
        assert!(html.contains("data-mode=\"light\""));
        assert!(html.contains("aria-label=\"Switch to dark mode\""));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_classlist_toggle() {
        let c = stub("mode-toggle", "Theme");
        let html = render(&c);
        assert!(!html.contains("onclick="));
        assert!(!html.contains("Theme</button>"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("mode-toggle", "Theme"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"mode-toggle\"]"));
        assert!(css.contains("width: 2.25rem; height: 2.25rem"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(css.contains("[data-slot=\"mode-toggle\"][data-mode=\"dark\"]"));
        assert!(css.contains("[data-slot=\"mode-toggle\"]:hover {\n  background: var(--cronus-surface-overlay); color: var(--cronus-fg);\n}"));
        assert!(css.contains(
            "[data-slot=\"mode-toggle\"] svg { width: 1.25rem; height: 1.25rem; flex-shrink: 0; }"
        ));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains("classList"));
    }
}
