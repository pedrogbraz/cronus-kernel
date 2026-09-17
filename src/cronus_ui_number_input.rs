//! Dedicated NumberInput renderer — mirrors React `NumberInput`.
//!
//! DOM: `<div data-slot="number-input">` > decrement `<button>` (outline icon
//! Button, `data-slot="number-input-decrement"`, lucide minus) >
//! `<input data-slot="number-input-field" type="text" inputmode="numeric"
//! role="spinbutton">` > increment `<button>` (lucide plus).
//!
//! The field stays a native, editable text input. Stepping needs JS, so both
//! steppers are the same native buttons with `disabled` and React's idle look
//! (not dimmed). Not reproduced: Arrow/Page/Home/End stepping, clamping and
//! precision formatting on blur.
//!
//! Props: `value` (number), `min`, `max` (→ `aria-valuemin/max`),
//! `aria-label` (else the label), `placeholder`.

use crate::cronus_ui_kit::{attr_nonempty, attr_num, esc, label_of};
use crate::parser::ComponentNode;

const MINUS: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M5 12h14\"></path></svg>";
const PLUS: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M5 12h14\"></path><path d=\"M12 5v14\"></path></svg>";

fn fmt(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

pub fn render(comp: &ComponentNode) -> String {
    let aria = attr_nonempty(comp, "aria-label")
        .map(esc)
        .unwrap_or_else(|| label_of(comp));
    let value = attr_num::<f64>(comp, "value").filter(|v| v.is_finite());
    let mut field = String::from(
        "data-slot=\"number-input-field\" type=\"text\" inputmode=\"numeric\" role=\"spinbutton\" autocomplete=\"off\"",
    );
    if let Some(p) = attr_nonempty(comp, "placeholder") {
        field.push_str(&format!(" placeholder=\"{}\"", esc(p)));
    }
    if let Some(v) = value {
        // `precision:2` + `prefix:"$"` mirror React's `format` shown while idle.
        let shown = match attr_num::<usize>(comp, "precision") {
            Some(p) => format!("{v:.p$}"),
            None => fmt(v),
        };
        let shown = format!(
            "{}{shown}",
            attr_nonempty(comp, "prefix").map(esc).unwrap_or_default()
        );
        field.push_str(&format!(" value=\"{shown}\""));
    }
    field.push_str(&format!(" aria-label=\"{aria}\""));
    for (key, aria_key) in [("min", "aria-valuemin"), ("max", "aria-valuemax")] {
        if let Some(n) = attr_num::<f64>(comp, key).filter(|v| v.is_finite()) {
            field.push_str(&format!(" {aria_key}=\"{}\"", fmt(n)));
        }
    }
    if let Some(v) = value {
        let v = fmt(v);
        field.push_str(&format!(" aria-valuenow=\"{v}\" aria-valuetext=\"{v}\""));
    }
    format!(
        "<div data-slot=\"number-input\"><button type=\"button\" data-slot=\"number-input-decrement\" data-variant=\"outline\" tabindex=\"-1\" aria-label=\"Decrement\" disabled>{MINUS}</button><input {field}><button type=\"button\" data-slot=\"number-input-increment\" data-variant=\"outline\" tabindex=\"-1\" aria-label=\"Increment\" disabled>{PLUS}</button></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn fixture() -> ComponentNode {
        let mut c = stub("number-input", "Quantity");
        c.props.insert("value".into(), "5".into());
        c.props.insert("aria-label".into(), "Quantity".into());
        c
    }

    #[test]
    fn fixture_matches_react_dom() {
        assert_eq!(
            render(&fixture()),
            format!(
                "<div data-slot=\"number-input\"><button type=\"button\" data-slot=\"number-input-decrement\" data-variant=\"outline\" tabindex=\"-1\" aria-label=\"Decrement\" disabled>{MINUS}</button><input data-slot=\"number-input-field\" type=\"text\" inputmode=\"numeric\" role=\"spinbutton\" autocomplete=\"off\" value=\"5\" aria-label=\"Quantity\" aria-valuenow=\"5\" aria-valuetext=\"5\"><button type=\"button\" data-slot=\"number-input-increment\" data-variant=\"outline\" tabindex=\"-1\" aria-label=\"Increment\" disabled>{PLUS}</button></div>"
            )
        );
    }

    #[test]
    fn bounds_and_garbage() {
        let mut c = stub("number-input", "Qty");
        c.props.insert("value".into(), "abc".into());
        c.props.insert("min".into(), "0".into());
        c.props.insert("max".into(), "2.5".into());
        let html = render(&c);
        assert!(!html.contains("value="));
        assert!(html.contains("aria-valuemin=\"0\" aria-valuemax=\"2.5\""));
        assert!(html.contains("aria-label=\"Qty\""));
        for bad in ["type=\"number\"", "v-model", "v-data", "style=", "<label"] {
            assert!(!html.contains(bad), "{bad}");
        }
    }

    #[test]
    fn chrome_steppers_keep_idle_look() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"number-input\"] {\n  display: flex; align-items: center; gap: 0.25rem;"
        ));
        assert!(css.contains(
            "[data-slot=\"number-input-decrement\"], [data-slot=\"number-input-increment\"] {"
        ));
        assert!(!css.contains("[data-slot=\"number-input\"] input {"));
    }
}
