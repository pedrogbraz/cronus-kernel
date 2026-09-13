//! Dedicated CreditCardInput renderer. DOM matches React:
//! `<div data-slot="credit-card-input">` plus native number / expiry / CVC
//! `<input>` fields. Not interact `input("credit-card-input", "text")`
//! (`<label>` + `*-control` + CTRL).

use crate::cronus_ui_kit::{esc, item, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let aria = group_aria(comp);
    let number = format_number(&digits_of(comp, &["value", "number", "defaultNumber", "default-number"]));
    let expiry = format_expiry(&digits_of(comp, &["expiry"]));
    let cvc = digits_of(comp, &["cvc"]);
    let disabled = flag(comp, "disabled");
    let invalid = flag(comp, "invalid") || attr(comp, "error").is_some_and(|s| !s.is_empty());

    let mut root = format!("data-slot=\"credit-card-input\" aria-label=\"{aria}\"");
    if disabled {
        root.push_str(" data-disabled=\"\"");
        root.push_str(" aria-disabled=\"true\"");
    }
    if invalid {
        root.push_str(" data-invalid=\"\"");
    }

    let number_field = field(
        "Card number",
        "0000 0000 0000 0000",
        &number,
        disabled,
        invalid,
    );
    let expiry_field = field(
        "Expiration date, M M slash Y Y",
        "MM/YY",
        &expiry,
        disabled,
        invalid,
    );
    let cvc_field = field("Security code, 3 digits", "CVC", &cvc, disabled, invalid);

    format!("<div {root}>{number_field}{expiry_field}{cvc_field}</div>")
}

fn field(aria: &str, placeholder: &str, value: &str, disabled: bool, invalid: bool) -> String {
    let mut attrs = format!(
        "type=\"text\" inputmode=\"numeric\" autocomplete=\"off\" spellcheck=\"false\" aria-label=\"{aria}\" placeholder=\"{placeholder}\""
    );
    if !value.is_empty() {
        attrs.push_str(&format!(" value=\"{value}\""));
    }
    if disabled {
        attrs.push_str(" disabled");
    }
    if invalid {
        attrs.push_str(" aria-invalid=\"true\"");
    }
    format!("<input {attrs} />")
}

fn group_aria(comp: &ComponentNode) -> String {
    if let Some(v) = attr(comp, "aria-label").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    if let Some(v) = attr(comp, "label").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    let label = label_of(comp);
    if label.is_empty() || label == "credit-card-input" {
        "Credit card".into()
    } else {
        label
    }
}

fn digits_of(comp: &ComponentNode, names: &[&str]) -> String {
    for name in names {
        if let Some(v) = attr(comp, name).filter(|s| !s.is_empty()) {
            return v.chars().filter(|c| c.is_ascii_digit()).take(19).collect();
        }
        if let Some(t) = item(comp, name).filter(|s| !s.is_empty()) {
            return t.chars().filter(|c| c.is_ascii_digit()).take(19).collect();
        }
    }
    String::new()
}

fn format_number(digits: &str) -> String {
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && matches!(i, 4 | 8 | 12) {
            out.push(' ');
        }
        out.push(ch);
    }
    esc(&out)
}

fn format_expiry(digits: &str) -> String {
    let d: String = digits.chars().take(4).collect();
    if d.len() >= 2 {
        esc(&format!("{}/{}", &d[..2], &d[2..]))
    } else {
        esc(&d)
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
        assert!(!html.contains("credit-card-input-control"));
        assert!(!html.contains("<label"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("height:2.5rem;padding:0 0.75rem"));
    }

    #[test]
    fn root_is_div_with_number_field() {
        let html = render(&stub("credit-card-input", "Card"));
        assert!(html.starts_with("<div data-slot=\"credit-card-input\""));
        assert!(html.contains("aria-label=\"Card\""));
        assert!(html.contains("<input type=\"text\" inputmode=\"numeric\""));
        assert!(html.contains("aria-label=\"Card number\""));
        assert!(html.contains("placeholder=\"0000 0000 0000 0000\""));
        assert!(html.contains("placeholder=\"MM/YY\""));
        assert!(html.contains("placeholder=\"CVC\""));
        assert!(!html.contains("credit-card-input-control"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"credit-card-input\" aria-label=\"Card\"><input type=\"text\" inputmode=\"numeric\" autocomplete=\"off\" spellcheck=\"false\" aria-label=\"Card number\" placeholder=\"0000 0000 0000 0000\" /><input type=\"text\" inputmode=\"numeric\" autocomplete=\"off\" spellcheck=\"false\" aria-label=\"Expiration date, M M slash Y Y\" placeholder=\"MM/YY\" /><input type=\"text\" inputmode=\"numeric\" autocomplete=\"off\" spellcheck=\"false\" aria-label=\"Security code, 3 digits\" placeholder=\"CVC\" /></div>"
        );
    }

    #[test]
    fn value_formats_grouped_number() {
        let mut c = stub("credit-card-input", "Card");
        c.props.insert("value".into(), "4111111111111111".into());
        let html = render(&c);
        assert!(html.contains("value=\"4111 1111 1111 1111\""));
        reject_interact(&html);
    }

    #[test]
    fn disabled_and_invalid() {
        let mut c = stub("credit-card-input", "Card");
        c.props.insert("disabled".into(), "true".into());
        c.props.insert("invalid".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(" data-disabled=\"\""));
        assert!(html.contains(" data-invalid=\"\""));
        assert!(html.contains(" disabled"));
        assert!(html.contains("aria-invalid=\"true\""));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_label_text_control() {
        let c = stub("credit-card-input", "Card");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("credit-card-input", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<label data-slot=\"credit-card-input\""));
        assert!(interact.contains("data-slot=\"credit-card-input-control\""));
        assert!(interact.contains("type=\"text\""));
        assert!(interact.contains("style="));
        assert!(!html.contains("credit-card-input-control"));
        assert!(!html.contains("<label"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("credit-card-input", "Card"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"credit-card-input\"]"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("font-variant-numeric: tabular-nums"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
