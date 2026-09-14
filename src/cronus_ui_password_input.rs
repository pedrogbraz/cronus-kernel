//! Dedicated PasswordInput renderer — mirrors React `PasswordInput`.
//!
//! DOM: `<div data-slot="password-input">` > unslotted `<div>` (React
//! `div.relative`) > `<input data-slot="input" type="password">` + the
//! absolute `<button data-slot="password-input-toggle" aria-pressed="false">`
//! with lucide eye.
//!
//! The field is a native, editable password input. Revealing the value needs
//! JS, so the toggle is the same native button with `disabled` and React's idle
//! look (not dimmed). Not reproduced: reveal toggle, strength meter.
//!
//! Props: placeholder = `placeholder` prop / `text` item / label,
//! `disabled`, `invalid`.

use crate::cronus_ui_kit::{attr_nonempty, esc, flag_any, item};
use crate::parser::ComponentNode;

const EYE: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0\"></path><circle cx=\"12\" cy=\"12\" r=\"3\"></circle></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let placeholder = attr_nonempty(comp, "placeholder")
        .or_else(|| item(comp, "text"))
        .or_else(|| item(comp, "label"))
        .unwrap_or("");
    let mut field = format!(
        "data-slot=\"input\" type=\"password\" placeholder=\"{}\" autocomplete=\"current-password\"",
        esc(placeholder)
    );
    let disabled = flag_any(comp, "disabled");
    if disabled {
        field.push_str(" disabled");
    }
    if flag_any(comp, "invalid") {
        field.push_str(" aria-invalid=\"true\"");
    }
    format!(
        "<div data-slot=\"password-input\"><div><input {field}><button type=\"button\" data-slot=\"password-input-toggle\" aria-label=\"Show password\" aria-pressed=\"false\" disabled>{EYE}</button></div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    #[test]
    fn fixture_matches_react_dom() {
        assert_eq!(
            render(&stub("password-input", "Enter your password")),
            format!(
                "<div data-slot=\"password-input\"><div><input data-slot=\"input\" type=\"password\" placeholder=\"Enter your password\" autocomplete=\"current-password\"><button type=\"button\" data-slot=\"password-input-toggle\" aria-label=\"Show password\" aria-pressed=\"false\" disabled>{EYE}</button></div></div>"
            )
        );
    }

    #[test]
    fn flags_and_escaping() {
        let mut c = stub("password-input", "\"><b>");
        c.props.insert("disabled".into(), "true".into());
        c.props.insert("invalid".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("placeholder=\"&quot;&gt;&lt;b&gt;\" autocomplete=\"current-password\" disabled aria-invalid=\"true\">"));
        for bad in ["<b>", "v-model", "style=", "<label"] {
            assert!(!html.contains(bad), "{bad}");
        }
    }

    #[test]
    fn chrome_toggle_is_absolute_icon() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"password-input\"] > div {\n  position: relative;"));
        assert!(css.contains("[data-slot=\"password-input-toggle\"] {\n  position: absolute;"));
        assert!(!css.contains("[data-slot=\"password-input\"] input {"));
    }
}
