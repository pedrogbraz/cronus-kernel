//! Dedicated PhoneInput renderer. DOM matches React:
//! `<div data-slot="phone-input" role="group"><button data-slot="phone-input-country">`
//! + `<input data-slot="phone-input-field" type="tel">`.
//! Not interact `input("phone-input", "tel")` (`<label>` + `*-control` + CTRL).

use crate::cronus_ui_kit::{esc, item, label_of};
use crate::parser::ComponentNode;

struct Country {
    code: &'static str,
    name: &'static str,
    dial: &'static str,
    flag: &'static str,
    mask: &'static str,
}

const BRAZIL: Country = Country {
    code: "BR",
    name: "Brazil",
    dial: "55",
    flag: "🇧🇷",
    mask: "00 00000 0000",
};

const COUNTRIES: &[Country] = &[
    BRAZIL,
    Country {
        code: "US",
        name: "United States",
        dial: "1",
        flag: "🇺🇸",
        mask: "000 000 0000",
    },
    Country {
        code: "GB",
        name: "United Kingdom",
        dial: "44",
        flag: "🇬🇧",
        mask: "0000 000 000",
    },
    Country {
        code: "PT",
        name: "Portugal",
        dial: "351",
        flag: "🇵🇹",
        mask: "000 000 000",
    },
];

pub fn render(comp: &ComponentNode) -> String {
    let country = country_of(comp);
    let aria = aria_label_of(comp);
    let placeholder = placeholder_of(comp, country);
    let value = value_of(comp, country);
    let disabled = flag(comp, "disabled");
    let invalid = flag(comp, "invalid");

    let mut root = format!("data-slot=\"phone-input\" role=\"group\" aria-label=\"{aria}\"");
    if disabled {
        root.push_str(" aria-disabled=\"true\"");
    }
    if invalid {
        root.push_str(" data-invalid=\"true\"");
    }

    let mut btn = format!(
        "type=\"button\" data-slot=\"phone-input-country\" role=\"combobox\" aria-haspopup=\"listbox\" aria-label=\"Select country. {}, +{}\"",
        country.name, country.dial
    );
    if disabled {
        btn.push_str(" disabled");
    }

    let mut field = format!(
        "data-slot=\"phone-input-field\" type=\"tel\" inputmode=\"tel\" autocomplete=\"tel-national\" aria-label=\"{aria}\" placeholder=\"{placeholder}\""
    );
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
        "<div {root}><button {btn}><span aria-hidden=\"true\">{}</span><span>+{}</span></button><input {field} /></div>",
        country.flag, country.dial
    )
}

fn country_of(comp: &ComponentNode) -> &'static Country {
    let raw = attr(comp, "country")
        .or_else(|| attr(comp, "defaultCountry"))
        .or_else(|| attr(comp, "default-country"))
        .unwrap_or("BR");
    let code = raw.trim().to_ascii_uppercase();
    COUNTRIES
        .iter()
        .find(|c| c.code == code)
        .unwrap_or(&BRAZIL)
}

fn aria_label_of(comp: &ComponentNode) -> String {
    if let Some(v) = attr(comp, "aria-label").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    if let Some(v) = attr(comp, "label").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    let label = label_of(comp);
    if label.is_empty() {
        "Phone number".into()
    } else {
        label
    }
}

fn placeholder_of(comp: &ComponentNode, country: &Country) -> String {
    if let Some(v) = attr(comp, "placeholder").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    if let Some(t) = item(comp, "placeholder").filter(|s| !s.is_empty()) {
        return esc(t);
    }
    if let Some(t) = item(comp, "text").filter(|s| !s.is_empty()) {
        return esc(t);
    }
    country.mask.into()
}

fn value_of(comp: &ComponentNode, country: &Country) -> String {
    let raw = attr(comp, "value")
        .or_else(|| item(comp, "value"))
        .unwrap_or("")
        .trim();
    if raw.is_empty() {
        return String::new();
    }
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    let national = digits
        .strip_prefix(country.dial)
        .unwrap_or(digits.as_str());
    esc(national)
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
        assert!(!html.contains("phone-input-control"));
        assert!(!html.contains("<label"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("height:2.5rem;padding:0 0.75rem"));
    }

    #[test]
    fn root_is_group_with_country_and_tel_field() {
        let html = render(&stub("phone-input", "Phone"));
        assert!(html.starts_with("<div data-slot=\"phone-input\" role=\"group\""));
        assert!(html.contains("aria-label=\"Phone\""));
        assert!(html.contains("data-slot=\"phone-input-country\""));
        assert!(html.contains("role=\"combobox\""));
        assert!(html.contains("<span aria-hidden=\"true\">🇧🇷</span>"));
        assert!(html.contains("<span>+55</span>"));
        assert!(html.contains("data-slot=\"phone-input-field\""));
        assert!(html.contains("type=\"tel\""));
        assert!(html.contains("placeholder=\"00 00000 0000\""));
        assert!(!html.contains("phone-input-control"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"phone-input\" role=\"group\" aria-label=\"Phone\"><button type=\"button\" data-slot=\"phone-input-country\" role=\"combobox\" aria-haspopup=\"listbox\" aria-label=\"Select country. Brazil, +55\"><span aria-hidden=\"true\">🇧🇷</span><span>+55</span></button><input data-slot=\"phone-input-field\" type=\"tel\" inputmode=\"tel\" autocomplete=\"tel-national\" aria-label=\"Phone\" placeholder=\"00 00000 0000\" /></div>"
        );
    }

    #[test]
    fn value_fills_national_number() {
        let mut c = stub("phone-input", "Phone");
        c.props.insert("value".into(), "+5511987654321".into());
        let html = render(&c);
        assert!(html.contains("value=\"11987654321\""));
        reject_interact(&html);
    }

    #[test]
    fn country_prop_switches_dial() {
        let mut c = stub("phone-input", "Phone");
        c.props.insert("country".into(), "US".into());
        let html = render(&c);
        assert!(html.contains("<span aria-hidden=\"true\">🇺🇸</span>"));
        assert!(html.contains("<span>+1</span>"));
        assert!(html.contains("Select country. United States, +1"));
        assert!(html.contains("placeholder=\"000 000 0000\""));
        reject_interact(&html);
    }

    #[test]
    fn disabled_and_invalid() {
        let mut c = stub("phone-input", "Phone");
        c.props.insert("disabled".into(), "true".into());
        c.props.insert("invalid".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(" aria-disabled=\"true\""));
        assert!(html.contains(" data-invalid=\"true\""));
        assert!(html.contains(" disabled"));
        assert!(html.contains("aria-invalid=\"true\""));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_label_tel_control() {
        let c = stub("phone-input", "Phone");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("phone-input", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<label data-slot=\"phone-input\""));
        assert!(interact.contains("data-slot=\"phone-input-control\""));
        assert!(interact.contains("type=\"tel\""));
        assert!(interact.contains("style="));
        assert!(!html.contains("phone-input-control"));
        assert!(!html.contains("<label"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("phone-input", "Phone"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"phone-input\"]"));
        assert!(css.contains("[data-slot=\"phone-input-country\"]"));
        assert!(css.contains("[data-slot=\"phone-input-field\"]"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("height: 2.5rem"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
