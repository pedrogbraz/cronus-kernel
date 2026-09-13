//! Dedicated CurrencyInput renderer. DOM matches React:
//! `<div data-slot="currency-input"><span data-slot="currency-input-prefix">$</span>`
//! + `<input data-slot="currency-input-field">`.
//! Not interact `input("currency-input", "number")` (`<label>` + `*-control` + CTRL).

use crate::cronus_ui_kit::{esc, item, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let prefix = prefix_of(comp);
    let placeholder = placeholder_of(comp);
    let value = value_of(comp);
    let aria = aria_label_of(comp);
    let disabled = flag(comp, "disabled");
    let invalid = flag(comp, "invalid");

    let mut root = String::from("data-slot=\"currency-input\"");
    if disabled {
        root.push_str(" data-disabled=\"\"");
        root.push_str(" aria-disabled=\"true\"");
    }
    if invalid {
        root.push_str(" data-invalid=\"\"");
    }

    let mut field = format!(
        "data-slot=\"currency-input-field\" type=\"text\" inputmode=\"decimal\" autocomplete=\"off\" spellcheck=\"false\" placeholder=\"{placeholder}\""
    );
    if !aria.is_empty() {
        field.push_str(&format!(" aria-label=\"{aria}\""));
    }
    if !value.is_empty() {
        field.push_str(&format!(" value=\"{value}\""));
    }
    if disabled {
        field.push_str(" disabled");
    }
    if invalid {
        field.push_str(" aria-invalid=\"true\"");
    }

    format!(
        "<div {root}><span data-slot=\"currency-input-prefix\" aria-hidden=\"true\">{prefix}</span><input {field} /></div>"
    )
}

fn prefix_of(comp: &ComponentNode) -> String {
    if let Some(v) = attr(comp, "prefix").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    if let Some(v) = item(comp, "prefix").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    match attr(comp, "currency")
        .or_else(|| attr(comp, "defaultCurrency"))
        .or_else(|| attr(comp, "default-currency"))
        .unwrap_or("USD")
        .trim()
        .to_ascii_uppercase()
        .as_str()
    {
        "EUR" => "€".into(),
        "GBP" => "£".into(),
        "BRL" => "R$".into(),
        "JPY" => "¥".into(),
        _ => "$".into(),
    }
}

fn aria_label_of(comp: &ComponentNode) -> String {
    if let Some(v) = attr(comp, "aria-label").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    let label = label_of(comp);
    if label.is_empty() {
        String::new()
    } else {
        label
    }
}

fn placeholder_of(comp: &ComponentNode) -> String {
    if let Some(v) = attr(comp, "placeholder").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    if let Some(t) = item(comp, "placeholder").filter(|s| !s.is_empty()) {
        return esc(t);
    }
    if let Some(t) = item(comp, "text").filter(|s| !s.is_empty()) {
        return esc(t);
    }
    "0.00".into()
}

fn value_of(comp: &ComponentNode) -> String {
    attr(comp, "value")
        .or_else(|| item(comp, "value"))
        .filter(|s| !s.is_empty())
        .map(esc)
        .unwrap_or_default()
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
        assert!(!html.contains("currency-input-control"));
        assert!(!html.contains("<label"));
        assert!(!html.contains("type=\"number\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("height:2.5rem;padding:0 0.75rem"));
    }

    #[test]
    fn root_is_div_with_prefix_and_field() {
        let html = render(&stub("currency-input", "Amount"));
        assert!(html.starts_with("<div data-slot=\"currency-input\">"));
        assert!(html.contains("data-slot=\"currency-input-prefix\" aria-hidden=\"true\">$</span>"));
        assert!(html.contains("data-slot=\"currency-input-field\""));
        assert!(html.contains("type=\"text\""));
        assert!(html.contains("inputmode=\"decimal\""));
        assert!(html.contains("aria-label=\"Amount\""));
        assert!(html.contains("placeholder=\"0.00\""));
        assert!(!html.contains("currency-input-control"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"currency-input\"><span data-slot=\"currency-input-prefix\" aria-hidden=\"true\">$</span><input data-slot=\"currency-input-field\" type=\"text\" inputmode=\"decimal\" autocomplete=\"off\" spellcheck=\"false\" placeholder=\"0.00\" aria-label=\"Amount\" /></div>"
        );
    }

    #[test]
    fn currency_prop_sets_prefix() {
        let mut c = stub("currency-input", "Amount");
        c.props.insert("currency".into(), "EUR".into());
        let html = render(&c);
        assert!(html.contains("data-slot=\"currency-input-prefix\" aria-hidden=\"true\">€</span>"));
        reject_interact(&html);
    }

    #[test]
    fn value_and_placeholder() {
        let mut c = stub("currency-input", "Amount");
        c.props.insert("value".into(), "1,234.56".into());
        c.props.insert("placeholder".into(), "0,00".into());
        let html = render(&c);
        assert!(html.contains("value=\"1,234.56\""));
        assert!(html.contains("placeholder=\"0,00\""));
        reject_interact(&html);
    }

    #[test]
    fn disabled_and_invalid() {
        let mut c = stub("currency-input", "Amount");
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
    fn skips_interact_number_control() {
        let c = stub("currency-input", "Amount");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("currency-input", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<label data-slot=\"currency-input\""));
        assert!(interact.contains("data-slot=\"currency-input-control\""));
        assert!(interact.contains("type=\"number\""));
        assert!(interact.contains("style="));
        assert!(!html.contains("currency-input-control"));
        assert!(!html.contains("<label"));
        assert!(!html.contains("type=\"number\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("currency-input", "Amount"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"currency-input\"]"));
        assert!(css.contains("[data-slot=\"currency-input-prefix\"]"));
        assert!(css.contains("[data-slot=\"currency-input-field\"]"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("height: 2.5rem"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
