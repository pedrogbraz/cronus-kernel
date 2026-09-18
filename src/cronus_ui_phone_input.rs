//! Dedicated PhoneInput renderer. DOM matches React:
//! `<div data-slot="phone-input" role="group">` holding the country trigger
//! `<button data-slot="phone-input-country">` (flag, dial code, lucide chevron),
//! a native `popover="auto"` country list (radios, Select pattern), a 1px
//! `<span aria-hidden>` divider, and `<input data-slot="phone-input-field" type="tel">`
//! whose value is the national number grouped by the country mask (`11 98765 4321`).
//! Picking a country swaps flag/dial via CSS `attr(data-cN)` / `attr(data-dN)`
//! and live.js reformats the national number. `disabled:` keeps the trigger
//! and field disabled and omits the popover.
//! Not interact `input("phone-input", "tel")` (`*-control` + CTRL).

use crate::cronus_ui_kit::{attr, attr_nonempty, esc, flag, item, label_of, widget_id};
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
        code: "PT",
        name: "Portugal",
        dial: "351",
        flag: "🇵🇹",
        groups: &[3, 3, 3],
    },
    Country {
        code: "GB",
        name: "United Kingdom",
        dial: "44",
        flag: "🇬🇧",
        groups: &[4, 3, 3],
    },
    Country {
        code: "ES",
        name: "Spain",
        dial: "34",
        flag: "🇪🇸",
        groups: &[3, 3, 3],
    },
    Country {
        code: "FR",
        name: "France",
        dial: "33",
        flag: "🇫🇷",
        groups: &[1, 2, 2, 2, 2],
    },
    Country {
        code: "DE",
        name: "Germany",
        dial: "49",
        flag: "🇩🇪",
        groups: &[3, 4, 4],
    },
    Country {
        code: "IT",
        name: "Italy",
        dial: "39",
        flag: "🇮🇹",
        groups: &[3, 3, 4],
    },
    Country {
        code: "NL",
        name: "Netherlands",
        dial: "31",
        flag: "🇳🇱",
        groups: &[1, 4, 4],
    },
    Country {
        code: "MX",
        name: "Mexico",
        dial: "52",
        flag: "🇲🇽",
        groups: &[2, 4, 4],
    },
    Country {
        code: "AR",
        name: "Argentina",
        dial: "54",
        flag: "🇦🇷",
        groups: &[2, 4, 4],
    },
    Country {
        code: "CL",
        name: "Chile",
        dial: "56",
        flag: "🇨🇱",
        groups: &[1, 4, 4],
    },
    Country {
        code: "CO",
        name: "Colombia",
        dial: "57",
        flag: "🇨🇴",
        groups: &[3, 3, 4],
    },
    Country {
        code: "CA",
        name: "Canada",
        dial: "1",
        flag: "🇨🇦",
        groups: &[3, 3, 4],
    },
    Country {
        code: "AU",
        name: "Australia",
        dial: "61",
        flag: "🇦🇺",
        groups: &[3, 3, 3],
    },
    Country {
        code: "JP",
        name: "Japan",
        dial: "81",
        flag: "🇯🇵",
        groups: &[2, 4, 4],
    },
    Country {
        code: "IN",
        name: "India",
        dial: "91",
        flag: "🇮🇳",
        groups: &[5, 5],
    },
    Country {
        code: "CN",
        name: "China",
        dial: "86",
        flag: "🇨🇳",
        groups: &[3, 4, 4],
    },
    Country {
        code: "ZA",
        name: "South Africa",
        dial: "27",
        flag: "🇿🇦",
        groups: &[2, 3, 4],
    },
    Country {
        code: "AE",
        name: "United Arab Emirates",
        dial: "971",
        flag: "🇦🇪",
        groups: &[2, 3, 4],
    },
    Country {
        code: "NG",
        name: "Nigeria",
        dial: "234",
        flag: "🇳🇬",
        groups: &[3, 3, 4],
    },
    Country {
        code: "PH",
        name: "Philippines",
        dial: "63",
        flag: "🇵🇭",
        groups: &[3, 3, 4],
    },
];

/// Options whose flag/dial the trigger can show (one CSS rule each).
const SWAP_MAX: usize = 24;

/// lucide `chevron-down` (React `<ChevronDown className="size-3.5" />`).
const CHEVRON_DOWN: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m6 9 6 6 6-6\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let country = country_of(comp);
    let aria = aria_label_of(comp);
    let placeholder = placeholder_of(comp, country);
    let value = value_of(comp, country);
    let disabled = flag(comp, "disabled");
    let invalid = flag(comp, "invalid");
    let groups = groups_attr(country);

    let mut root = format!(
        "data-slot=\"phone-input\" role=\"group\" aria-label=\"{aria}\" data-groups=\"{groups}\""
    );
    if disabled {
        root.push_str(" aria-disabled=\"true\"");
    }
    if invalid {
        root.push_str(" data-invalid=\"true\"");
    }

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

    let country_aria = format!("Select country. {}, +{}", country.name, country.dial);
    if disabled {
        return format!(
            "<div {root}><button type=\"button\" role=\"combobox\" aria-expanded=\"false\" aria-haspopup=\"listbox\" aria-label=\"{country_aria}\" data-slot=\"phone-input-country\" data-state=\"closed\" disabled><span aria-hidden=\"true\">{}</span><span>+{}</span>{CHEVRON_DOWN}</button><span aria-hidden=\"true\"></span><input {field} /></div>",
            country.flag, country.dial
        );
    }

    let trigger_id = widget_id(comp, "trigger");
    let pop_id = widget_id(comp, "listbox");
    let name = widget_id(comp, "country");
    let flag_attrs = swap_attrs("c", |c| c.flag.to_string());
    let dial_attrs = swap_attrs("d", |c| format!("+{}", c.dial));
    let mut items = String::new();
    for (i, c) in COUNTRIES.iter().enumerate() {
        let checked = if c.code == country.code {
            " checked"
        } else {
            ""
        };
        let g = groups_attr(c);
        let mask = mask_of(c);
        items.push_str(&format!(
            "<label data-slot=\"phone-input-country-item\" data-option=\"{}\"><input type=\"radio\" name=\"{name}\" value=\"{}\" data-groups=\"{g}\" data-mask=\"{mask}\" data-dial=\"{}\"{checked}><span aria-hidden=\"true\">{}</span> {} <span>+{}</span></label>",
            i + 1,
            c.code,
            c.dial,
            c.flag,
            esc(c.name),
            c.dial
        ));
    }

    format!(
        "<div {root}><button type=\"button\" id=\"{trigger_id}\" role=\"combobox\" aria-expanded=\"false\" aria-haspopup=\"listbox\" aria-label=\"{country_aria}\" data-slot=\"phone-input-country\" data-state=\"closed\" popovertarget=\"{pop_id}\" aria-controls=\"{pop_id}\"><span aria-hidden=\"true\"{flag_attrs}>{}</span><span{dial_attrs}>+{}</span>{CHEVRON_DOWN}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"phone-input-content\" role=\"listbox\" anchor=\"{trigger_id}\">{items}</div><span aria-hidden=\"true\"></span><input {field} /></div>",
        country.flag, country.dial
    )
}

fn groups_attr(country: &Country) -> String {
    country
        .groups
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn swap_attrs(prefix: &str, pick: impl Fn(&Country) -> String) -> String {
    COUNTRIES
        .iter()
        .take(SWAP_MAX)
        .enumerate()
        .map(|(i, c)| format!(" data-{prefix}{}=\"{}\"", i + 1, esc(&pick(c))))
        .collect()
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
    if let Some(v) = attr_nonempty(comp, "aria-label") {
        return esc(v);
    }
    if let Some(v) = attr_nonempty(comp, "label") {
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
    if let Some(v) = attr_nonempty(comp, "placeholder") {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn reject_interact(html: &str) {
        assert!(!html.contains("phone-input-control"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("height:2.5rem;padding:0 0.75rem"));
    }

    #[test]
    fn root_is_group_with_country_divider_and_tel_field() {
        let c = stub("phone-input", "Phone");
        let html = render(&c);
        let tid = crate::cronus_ui_kit::widget_id(&c, "trigger");
        let pid = crate::cronus_ui_kit::widget_id(&c, "listbox");
        let name = crate::cronus_ui_kit::widget_id(&c, "country");
        assert!(html.starts_with("<div data-slot=\"phone-input\" role=\"group\""));
        assert!(html.contains("aria-label=\"Phone\""));
        assert!(html.contains("data-groups=\"2,5,4\""));
        assert!(html.contains(&format!("id=\"{tid}\"")));
        assert!(html.contains(&format!("popovertarget=\"{pid}\"")));
        assert!(html.contains(&format!("aria-controls=\"{pid}\"")));
        assert!(html.contains("aria-haspopup=\"listbox\""));
        assert!(html.contains("aria-expanded=\"false\""));
        assert!(html.contains("data-state=\"closed\""));
        assert!(!html.contains("phone-input-country\" data-state=\"closed\" disabled"));
        assert!(html.contains(&format!(
            "id=\"{pid}\" popover=\"auto\" data-slot=\"phone-input-content\" role=\"listbox\" anchor=\"{tid}\""
        )));
        assert!(html.contains(&format!(
            "name=\"{name}\" value=\"BR\" data-groups=\"2,5,4\" data-mask=\"00 00000 0000\" data-dial=\"55\" checked"
        )));
        assert!(html.contains("</button><div id="));
        assert!(html.contains("</div><span aria-hidden=\"true\"></span><input "));
        assert!(html.contains("placeholder=\"00 00000 0000\""));
        assert!(html.contains("🇧🇷"));
        assert!(html.contains("+55"));
        assert!(html.contains("data-c1=\"🇧🇷\""));
        assert!(html.contains("data-d1=\"+55\""));
        assert!(html.contains("data-c22="));
        assert!(html.contains("value=\"PH\""));
        assert_eq!(
            html.matches("data-slot=\"phone-input-country-item\"")
                .count(),
            22
        );
        assert!(!html.contains(" disabled"));
        reject_interact(&html);
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
        assert!(html.contains("🇺🇸"));
        assert!(html.contains(">+1</span>"));
        assert!(html.contains("Select country. United States, +1"));
        assert!(html.contains("placeholder=\"000 000 0000\""));
        assert!(html.contains("data-groups=\"3,3,4\""));
        assert!(html.contains(
            "value=\"US\" data-groups=\"3,3,4\" data-mask=\"000 000 0000\" data-dial=\"1\" checked"
        ));
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
        assert!(!html.contains("popover="));
        assert!(!html.contains("popovertarget="));
        assert!(!html.contains("phone-input-content"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_label_tel_control() {
        let c = stub("phone-input", "Phone");
        let html = render(&c);
        assert!(!html.contains("phone-input-control"));
        assert!(html.contains("<input type=\"tel\""));
        assert!(html.contains("<label data-slot=\"phone-input-country-item\""));
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
            "[data-slot=\"phone-input\"] {\n  display: flex; height: 2.5rem; width: 100%; align-items: center;\n  box-sizing: border-box;\n  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);\n  background: var(--cronus-surface-inset); color: var(--cronus-fg);\n"
        ));
        assert!(css.contains(
            "[data-slot=\"phone-input\"] > span[aria-hidden=\"true\"] {\n  height: 1.25rem; width: 1px; flex-shrink: 0; background: var(--cronus-border);\n}"
        ));
        assert!(css.contains("border-start-start-radius: var(--cronus-radius-lg); border-end-start-radius: var(--cronus-radius-lg);"));
        assert!(css.contains("[data-slot=\"phone-input-country\"] > span:first-child { font-size: 1rem; line-height: 1; }"));
        assert!(css.contains("cursor: pointer;"));
        assert!(css.contains("[data-slot=\"phone-input-content\"]:popover-open {"));
        assert!(css.contains(
            "[data-option=\"1\"] > :checked) > span:first-child::before { content: attr(data-c1); }"
        ));
        assert!(css.contains(
            "[data-option=\"24\"] > :checked) > span:nth-child(2)::before { content: attr(data-d24); }"
        ));
        assert!(!css.contains("border-right: 1px solid var(--cronus-border);\n}\n[data-slot=\"phone-input-country\"]:disabled"));
    }
}
