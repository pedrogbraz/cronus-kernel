//! Dedicated PhoneInput renderer. DOM matches React:
//! `<div data-slot="phone-input" role="group">` holding the country trigger
//! `<button data-slot="phone-input-country">` (flag, dial code, lucide chevron),
//! a 1px `<span aria-hidden>` divider, and `<input data-slot="phone-input-field" type="tel">`
//! whose value is the national number grouped by the country mask (`11 98765 4321`).
//! The country list is a Radix popover in React (JS); the kernel renders the
//! closed trigger (`aria-expanded="false"`, `data-state="closed"`) as a native
//! `disabled` button with React's idle look (not dimmed).
//! Not interact `input("phone-input", "tel")` (`<label>` + `*-control` + CTRL).

use crate::cronus_ui_kit::{esc, item, label_of};
use crate::parser::ComponentNode;

struct Country {
    code: &'static str,
    name: &'static str,
    dial: &'static str,
    flag: &'static str,
    /// National digit groups (React `format`), e.g. BR `[2, 5, 4]`.
    groups: &'static [usize],
}

const BRAZIL: Country = Country {
    code: "BR",
    name: "Brazil",
    dial: "55",
    flag: "🇧🇷",
    groups: &[2, 5, 4],
};

const COUNTRIES: &[Country] = &[
    BRAZIL,
    Country {
        code: "US",
        name: "United States",
        dial: "1",
        flag: "🇺🇸",
        groups: &[3, 3, 4],
    },
    Country {
        code: "GB",
        name: "United Kingdom",
        dial: "44",
        flag: "🇬🇧",
        groups: &[4, 3, 3],
    },
    Country {
        code: "PT",
        name: "Portugal",
        dial: "351",
        flag: "🇵🇹",
        groups: &[3, 3, 3],
    },
];

/// lucide `chevron-down` (React `<ChevronDown className="size-3.5" />`).
const CHEVRON_DOWN: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m6 9 6 6 6-6\"></path></svg>";

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

    // The country list needs JS: same native trigger as React, `disabled`,
    // keeping React's idle look (only a disabled group dims, via the root).
    let btn = format!(
        "type=\"button\" role=\"combobox\" aria-expanded=\"false\" aria-haspopup=\"listbox\" aria-label=\"Select country. {}, +{}\" data-slot=\"phone-input-country\" data-state=\"closed\" disabled",
        country.name, country.dial
    );

    let mut field = format!(
        "type=\"tel\" inputmode=\"tel\" autocomplete=\"tel-national\" aria-label=\"{aria}\" data-slot=\"phone-input-field\" placeholder=\"{placeholder}\""
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
        "<div {root}><button {btn}><span aria-hidden=\"true\">{}</span><span>+{}</span>{CHEVRON_DOWN}</button><span aria-hidden=\"true\"></span><input {field} /></div>",
        country.flag, country.dial
    )
}

fn country_of(comp: &ComponentNode) -> &'static Country {
    let raw = attr(comp, "country")
        .or_else(|| attr(comp, "defaultCountry"))
        .or_else(|| attr(comp, "default-country"))
        .unwrap_or("BR");
    let code = raw.trim().to_ascii_uppercase();
    COUNTRIES.iter().find(|c| c.code == code).unwrap_or(&BRAZIL)
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

fn mask_of(country: &Country) -> String {
    country
        .groups
        .iter()
        .map(|n| "0".repeat(*n))
        .collect::<Vec<_>>()
        .join(" ")
}

fn placeholder_of(comp: &ComponentNode, country: &Country) -> String {
    if let Some(v) = attr(comp, "placeholder").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    if let Some(t) = item(comp, "placeholder").filter(|s| !s.is_empty()) {
        return esc(t);
    }
    mask_of(country)
}

/// React `formatNational`: digits split by the country groups, max length = sum.
fn format_national(digits: &str, groups: &[usize]) -> String {
    let max: usize = groups.iter().sum();
    let digits: Vec<char> = digits.chars().take(max).collect();
    let mut parts = Vec::new();
    let mut index = 0;
    for size in groups {
        if index >= digits.len() {
            break;
        }
        let end = (index + size).min(digits.len());
        parts.push(digits[index..end].iter().collect::<String>());
        index = end;
    }
    parts.join(" ")
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
    let national = if raw.starts_with('+') {
        digits.strip_prefix(country.dial).unwrap_or(digits.as_str())
    } else {
        digits.as_str()
    };
    esc(&format_national(national, country.groups))
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
    fn root_is_group_with_country_divider_and_tel_field() {
        let html = render(&stub("phone-input", "Phone"));
        assert!(html.starts_with("<div data-slot=\"phone-input\" role=\"group\""));
        assert!(html.contains("aria-label=\"Phone\""));
        assert!(html.contains("<span aria-hidden=\"true\">🇧🇷</span><span>+55</span><svg "));
        assert!(html.contains("</button><span aria-hidden=\"true\"></span><input "));
        assert!(html.contains("placeholder=\"00 00000 0000\""));
        reject_interact(&html);
        assert_eq!(
            html,
            format!("<div data-slot=\"phone-input\" role=\"group\" aria-label=\"Phone\"><button type=\"button\" role=\"combobox\" aria-expanded=\"false\" aria-haspopup=\"listbox\" aria-label=\"Select country. Brazil, +55\" data-slot=\"phone-input-country\" data-state=\"closed\" disabled><span aria-hidden=\"true\">🇧🇷</span><span>+55</span>{CHEVRON_DOWN}</button><span aria-hidden=\"true\"></span><input type=\"tel\" inputmode=\"tel\" autocomplete=\"tel-national\" aria-label=\"Phone\" data-slot=\"phone-input-field\" placeholder=\"00 00000 0000\" /></div>")
        );
    }

    #[test]
    fn value_fills_grouped_national_number() {
        let mut c = stub("phone-input", "Phone");
        c.props.insert("value".into(), "+5511987654321".into());
        let html = render(&c);
        assert!(html.contains("value=\"11 98765 4321\""));
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

    /// Wave 1t geometry: root keeps the inherited 16px/24px type (no text-sm),
    /// the trigger is rounded-s-lg with no border (the divider is its own 1px
    /// span), flag is text-base leading-none, chevron size-3.5, controls 20px lines.
    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"phone-input\"] {\n  display: flex; height: 2.5rem; width: 100%; align-items: center;\n  box-sizing: border-box;\n  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);\n  background: var(--cronus-surface-inset); color: var(--cronus-fg);\n}"
        ));
        assert!(css.contains(
            "[data-slot=\"phone-input\"] > span[aria-hidden=\"true\"] {\n  height: 1.25rem; width: 1px; flex-shrink: 0; background: var(--cronus-border);\n}"
        ));
        assert!(css.contains("border-start-start-radius: var(--cronus-radius-lg); border-end-start-radius: var(--cronus-radius-lg);"));
        assert!(css.contains("[data-slot=\"phone-input-country\"] > span:first-child { font-size: 1rem; line-height: 1; }"));
        assert!(!css.contains("border-right: 1px solid var(--cronus-border);\n}\n[data-slot=\"phone-input-country\"]:disabled"));
    }
}
