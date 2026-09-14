//! Dedicated CurrencyInput renderer. DOM matches React:
//! `<div data-slot="currency-input">` with the currency selector
//! `<button data-slot="currency-input-selector">` (symbol, ISO code, chevron) and
//! `<input data-slot="currency-input-field">` holding the minor-unit value
//! formatted for the currency locale (`12345` BRL → `123,45`).
//! A single currency (`currencies:"EUR"` or `prefix:`) renders React's static
//! `<span data-slot="currency-input-prefix">` instead of the selector.
//! The selector menu is a Radix dropdown (JS); the kernel renders the closed
//! trigger as a native `disabled` button with React's idle look (not dimmed).
//! Not interact `input("currency-input", "number")` (`<label>` + `*-control` + CTRL).

use crate::cronus_ui_kit::{esc, item, label_of};
use crate::parser::ComponentNode;

struct Currency {
    code: &'static str,
    symbol: &'static str,
    group: &'static str,
    decimal: &'static str,
    fraction: u32,
}

/// React `DEFAULT_CURRENCIES` head (BRL first = default) with each locale's separators.
const CURRENCIES: &[Currency] = &[
    Currency {
        code: "BRL",
        symbol: "R$",
        group: ".",
        decimal: ",",
        fraction: 2,
    },
    Currency {
        code: "USD",
        symbol: "US$",
        group: ",",
        decimal: ".",
        fraction: 2,
    },
    Currency {
        code: "EUR",
        symbol: "€",
        group: ".",
        decimal: ",",
        fraction: 2,
    },
    Currency {
        code: "GBP",
        symbol: "£",
        group: ",",
        decimal: ".",
        fraction: 2,
    },
    Currency {
        code: "JPY",
        symbol: "¥",
        group: ",",
        decimal: ".",
        fraction: 0,
    },
];

/// lucide `chevron-down` (React `size-3.5 text-fg-tertiary`).
const CHEVRON_DOWN: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m6 9 6 6 6-6\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let currency = currency_of(comp);
    let placeholder = placeholder_of(comp, currency);
    let value = value_of(comp, currency);
    let aria = aria_label_of(comp);
    let disabled = flag(comp, "disabled");
    let invalid = flag(comp, "invalid");

    let mut root = String::from("data-slot=\"currency-input\"");
    if disabled {
        root.push_str(" data-disabled=\"\"");
    }
    if invalid {
        root.push_str(" data-invalid=\"\"");
    }

    let symbol = attr(comp, "prefix")
        .filter(|s| !s.is_empty())
        .map(esc)
        .unwrap_or_else(|| currency.symbol.into());
    let face = format!("<span>{symbol}</span><span>{}</span>", currency.code);
    let lead = if single_currency(comp) {
        format!("<span data-slot=\"currency-input-prefix\" aria-hidden=\"true\">{face}</span>")
    } else {
        // The currency menu needs JS: same native trigger as React, `disabled`,
        // idle look kept (only `data-disabled` on the root dims).
        format!(
            "<button type=\"button\" disabled aria-label=\"Select currency\" data-slot=\"currency-input-selector\" aria-haspopup=\"menu\" aria-expanded=\"false\" data-state=\"closed\">{face}{CHEVRON_DOWN}</button>"
        )
    };

    let mut field = String::from(
        "type=\"text\" inputmode=\"decimal\" autocomplete=\"off\" autocorrect=\"off\" spellcheck=\"false\"",
    );
    field.push_str(&format!(" placeholder=\"{placeholder}\""));
    if !aria.is_empty() {
        field.push_str(&format!(" aria-label=\"{aria}\""));
    }
    field.push_str(" data-slot=\"currency-input-field\"");
    if !value.is_empty() {
        field.push_str(&format!(" value=\"{value}\""));
    }
    if disabled {
        field.push_str(" disabled");
    }
    if invalid {
        field.push_str(" aria-invalid=\"true\"");
    }

    format!("<div {root}>{lead}<input {field} /></div>")
}

fn currency_of(comp: &ComponentNode) -> &'static Currency {
    let code = attr(comp, "currency")
        .or_else(|| attr(comp, "defaultCurrency"))
        .or_else(|| attr(comp, "default-currency"))
        .or_else(|| attr(comp, "currencies").and_then(|s| s.split(',').next()))
        .unwrap_or("BRL")
        .trim()
        .to_ascii_uppercase();
    CURRENCIES
        .iter()
        .find(|c| c.code == code)
        .unwrap_or(&CURRENCIES[0])
}

/// React renders the static prefix when the currency list has one entry.
fn single_currency(comp: &ComponentNode) -> bool {
    if attr(comp, "prefix").is_some_and(|s| !s.is_empty()) {
        return true;
    }
    attr(comp, "currencies")
        .map(|s| s.split(',').filter(|c| !c.trim().is_empty()).count() == 1)
        .unwrap_or(false)
}

/// `Intl.NumberFormat(locale, {min/max fraction digits, useGrouping})` of
/// `cents / 10^fraction`, for the supported locales.
fn format_minor(cents: u64, c: &Currency) -> String {
    let scale = 10u64.pow(c.fraction);
    let major = (cents / scale).to_string();
    let mut grouped = String::new();
    for (i, ch) in major.chars().enumerate() {
        if i > 0 && (major.len() - i) % 3 == 0 {
            grouped.push_str(c.group);
        }
        grouped.push(ch);
    }
    if c.fraction == 0 {
        return grouped;
    }
    format!(
        "{grouped}{}{:0width$}",
        c.decimal,
        cents % scale,
        width = c.fraction as usize
    )
}

fn aria_label_of(comp: &ComponentNode) -> String {
    if let Some(v) = attr(comp, "aria-label").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    label_of(comp)
}

fn placeholder_of(comp: &ComponentNode, currency: &Currency) -> String {
    if let Some(v) = attr(comp, "placeholder").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    if let Some(t) = item(comp, "placeholder").filter(|s| !s.is_empty()) {
        return esc(t);
    }
    format_minor(0, currency)
}

/// Integer values are minor units (React `defaultValue`); anything else is shown verbatim.
fn value_of(comp: &ComponentNode, currency: &Currency) -> String {
    let Some(raw) = attr(comp, "value")
        .or_else(|| item(comp, "value"))
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return String::new();
    };
    match raw.parse::<u64>() {
        Ok(cents) => format_minor(cents, currency),
        Err(_) => esc(raw),
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
    fn root_is_div_with_selector_and_field() {
        let html = render(&stub("currency-input", "Amount"));
        assert!(html.starts_with("<div data-slot=\"currency-input\">"));
        assert!(!html.contains("currency-input-prefix"));
        reject_interact(&html);
        assert_eq!(
            html,
            format!("<div data-slot=\"currency-input\"><button type=\"button\" disabled aria-label=\"Select currency\" data-slot=\"currency-input-selector\" aria-haspopup=\"menu\" aria-expanded=\"false\" data-state=\"closed\"><span>R$</span><span>BRL</span>{CHEVRON_DOWN}</button><input type=\"text\" inputmode=\"decimal\" autocomplete=\"off\" autocorrect=\"off\" spellcheck=\"false\" placeholder=\"0,00\" aria-label=\"Amount\" data-slot=\"currency-input-field\" /></div>")
        );
    }

    #[test]
    fn minor_units_format_per_locale() {
        let mut c = stub("currency-input", "Amount");
        c.props.insert("value".into(), "12345".into());
        assert!(render(&c).contains("value=\"123,45\""));
        c.props.insert("value".into(), "123456789".into());
        assert!(render(&c).contains("value=\"1.234.567,89\""));
        c.props.insert("currency".into(), "USD".into());
        let html = render(&c);
        assert!(html.contains("value=\"1,234,567.89\""));
        assert!(html.contains("<span>US$</span><span>USD</span>"));
        assert!(html.contains("placeholder=\"0.00\""));
        c.props.insert("currency".into(), "JPY".into());
        c.props.insert("value".into(), "1500".into());
        assert!(render(&c).contains("value=\"1,500\""));
    }

    #[test]
    fn single_currency_renders_static_prefix() {
        let mut c = stub("currency-input", "Amount");
        c.props.insert("currencies".into(), "EUR".into());
        let html = render(&c);
        assert!(html.contains(
            "<span data-slot=\"currency-input-prefix\" aria-hidden=\"true\"><span>€</span><span>EUR</span></span>"
        ));
        assert!(!html.contains("<button"));
        reject_interact(&html);
    }

    #[test]
    fn non_integer_value_and_placeholder_pass_through() {
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
        assert!(html.contains("<button type=\"button\" disabled"));
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
        assert!(css.contains("[data-slot=\"currency-input-selector\"]"));
        assert!(css.contains("[data-slot=\"currency-input-field\"]"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("height: 2.5rem"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }

    /// Wave 1t geometry: the root was an unstyled block (432x24); React is a
    /// 40px bordered rounded-lg inset field with a border-e selector (ps-3
    /// pe-2.5, gap-1.5) and an end-aligned text-sm/20px field.
    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"currency-input\"] {\n  display: flex; height: 2.5rem; width: 100%; align-items: stretch; overflow: hidden;\n  box-sizing: border-box;\n  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);\n  background: var(--cronus-surface-inset); color: var(--cronus-fg);\n  box-shadow: var(--cronus-shadow-xs, none);\n}"
        ));
        assert!(css.contains("[data-slot=\"currency-input-selector\"] { padding: 0 0.625rem 0 0.75rem; cursor: default; }"));
        assert!(css.contains(
            "[data-slot=\"currency-input-field\"] {\n  min-width: 0; flex: 1; border: 0; outline: none; background: transparent;\n  padding: 0 0.75rem; text-align: end; font-variant-numeric: tabular-nums;\n  color: var(--cronus-fg); font-size: 0.875rem; line-height: 1.25rem;\n}"
        ));
    }
}
