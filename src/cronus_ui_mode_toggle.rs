//! Dedicated ModeToggle renderer.
//!
//! DOM: `<label>` > visually hidden `<input type="checkbox" data-cui-mode-toggle
//! aria-label="Switch to … mode">` + React's
//! `<button type="button" data-slot="mode-toggle" data-mode="dark|light">` with
//! the sun/moon icon. Not interact onclick
//! `document.documentElement.classList.toggle('dark')` + BASE/SURF styles.
//!
//! Zero JS: the checkbox is the control (click, Space, focus ring on the
//! button). Checking it swaps the page to the other colour mode in CSS:
//! `cronus_ui_css::mode_toggle_css` re-keys the vendored token blocks under
//! `[data-cronus-theme][data-cronus-mode]:has(input[data-cui-mode-toggle]:checked)`,
//! and the icon morphs to the other mode. The button keeps React's slot and
//! look but is decorative (`aria-hidden`, `tabindex="-1"`,
//! `pointer-events: none`); it is `data-disabled` (dimmed) only when the author
//! disabled the toggle, which also disables the checkbox.
//!
//! Gaps vs React (no JS): the choice is not persisted (a reload or another page
//! starts from the document's `data-cronus-mode`); `data-cronus-mode` itself
//! does not change, so `system` mode and mode-keyed looks (`glass`) are not
//! swapped; the checkbox's accessible name does not flip with the state.

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
    let (input_disabled, dimmed) = if author_disabled(comp) {
        (" disabled", " data-disabled=\"\"")
    } else {
        ("", "")
    };
    let size = match crate::cronus_ui_kit::choice(comp, "size", &["sm", "md"]) {
        Some("sm") => " class=\"s-sm\"",
        _ => "",
    };
    format!(
        "<label><input type=\"checkbox\" data-cui-mode-toggle aria-label=\"{aria}\"{input_disabled}><button type=\"button\" data-slot=\"mode-toggle\"{size} data-mode=\"{mode}\" aria-label=\"{aria}\" tabindex=\"-1\" aria-hidden=\"true\"{dimmed}>{ICON}</button></label>"
    )
}

/// React's `disabled` prop: dims the button (`disabled:opacity-50`) and
/// disables the checkbox that carries the state.
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
    fn root_is_checkbox_label_around_react_button() {
        let html = render(&stub("mode-toggle", "Theme"));
        assert!(html.starts_with(
            "<label><input type=\"checkbox\" data-cui-mode-toggle aria-label=\"Switch to dark mode\"><button type=\"button\" data-slot=\"mode-toggle\" data-mode=\"light\" aria-label=\"Switch to dark mode\" tabindex=\"-1\" aria-hidden=\"true\">"
        ));
        assert!(!html.contains("data-disabled"));
        assert!(!html.contains(" disabled"));
        assert!(html.ends_with("</button></label>"));
        assert!(html.contains("data-slot=\"mode-toggle-core\""));
        assert!(html.contains("data-slot=\"mode-toggle-rays\""));
        reject_interact(&html);
    }

    #[test]
    fn author_disabled_dims_button_and_disables_checkbox() {
        let mut d = stub("mode-toggle", "Theme");
        d.props.insert("disabled".into(), "true".into());
        let html = render(&d);
        assert!(html.contains("aria-label=\"Switch to dark mode\" disabled><button"));
        assert!(html.contains("aria-hidden=\"true\" data-disabled=\"\">"));
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

    /// Checked flips the icon to the other mode; the label carries hover/focus.
    #[test]
    fn chrome_checked_morphs_icon_and_label_is_the_hit_target() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "input[data-cui-mode-toggle] + [data-slot=\"mode-toggle\"] { pointer-events: none; }"
        ));
        assert!(css.contains("input[data-cui-mode-toggle]:checked + [data-slot=\"mode-toggle\"][data-mode=\"light\"] [data-slot=\"mode-toggle-core\"] { transform: scale(1.75); }"));
        assert!(css.contains("input[data-cui-mode-toggle]:checked + [data-slot=\"mode-toggle\"][data-mode=\"dark\"] [data-slot=\"mode-toggle-core\"] { transform: scale(1); }"));
        assert!(
            css.contains("input[data-cui-mode-toggle]:focus-visible + [data-slot=\"mode-toggle\"]")
        );
    }

    /// A page with the toggle ships the other-mode token blocks, keyed on the
    /// checked checkbox, for every preset in both directions; a page without
    /// it does not.
    #[test]
    fn page_with_toggle_ships_checked_mode_swap_tokens() {
        let html = render(&stub("mode-toggle", "Theme"));
        let css = crate::cronus_ui_css::audit_stylesheet(&html);
        assert!(css.contains(
            "[data-cronus-theme=\"aurora\"]:not([data-cronus-mode=\"light\"]):not([data-cronus-mode=\"system\"]):has(input[data-cui-mode-toggle]:checked) {"
        ));
        assert!(css.contains(
            "[data-cronus-theme=\"aurora\"][data-cronus-mode=\"light\"]:has(input[data-cui-mode-toggle]:checked) {"
        ));
        assert!(css.contains(
            "[data-cronus-theme=\"neutral\"][data-cronus-mode=\"dark\"]:has(input[data-cui-mode-toggle]:checked) {"
        ));
        let swap = crate::cronus_ui_css::mode_toggle_css();
        assert_eq!(
            swap.matches(":has(input[data-cui-mode-toggle]:checked) {")
                .count(),
            10
        );
        assert!(!swap.contains("@"));
        let plain = crate::cronus_ui_css::audit_stylesheet("<div data-slot=\"card\"></div>");
        assert!(!plain.contains("data-cui-mode-toggle"));
    }
}
