//! Dedicated CreditCardInput renderer. DOM matches React:
//! `<div data-slot="credit-card-input">` wrapping a `<fieldset data-brand>` with a
//! `sr-only` `<legend>` (group label), a `sr-only` polite brand announcement, the
//! brand glyph `<svg>`, the number / expiry / CVC `<input>`s and the lucide
//! `check` validity mark (hidden until valid — validation needs JS, so it stays
//! hidden). Brand detection is static from the initial number (React IIN table).
//! Not interact `input("credit-card-input", "text")` (`<label>` + `*-control` + CTRL).

use crate::cronus_ui_kit::{attr, attr_nonempty, esc, flag, item, label_of};
use crate::parser::ComponentNode;

struct Brand {
    id: &'static str,
    label: &'static str,
    gaps: &'static [usize],
    cvc: usize,
}

const UNKNOWN: Brand = Brand {
    id: "unknown",
    label: "Card",
    gaps: &[4, 8, 12],
    cvc: 3,
};

/// lucide `check` (React `size-4 text-success`, `scale-75 opacity-0` until valid).
const CHECK: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M20 6 9 17l-5-5\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let label = group_label(comp);
    let digits = digits_of(
        comp,
        &["value", "number", "defaultNumber", "default-number"],
    );
    let brand = brand_of(&digits);
    let number = format_number(&digits, brand.gaps);
    let expiry = format_expiry(&digits_of(comp, &["expiry"]));
    let cvc: String = digits_of(comp, &["cvc"]).chars().take(brand.cvc).collect();
    let disabled = flag(comp, "disabled");
    let invalid = flag(comp, "invalid") || attr(comp, "error").is_some_and(|s| !s.is_empty());

    let mut fieldset = format!("<fieldset");
    if disabled {
        fieldset.push_str(" disabled");
    }
    if invalid {
        fieldset.push_str(" data-invalid=\"true\"");
    }
    fieldset.push_str(&format!(" data-brand=\"{}\">", brand.id));

    let announce = if brand.id == "unknown" {
        String::new()
    } else {
        format!("{} card", brand.label)
    };
    let number_placeholder = if brand.id == "amex" {
        "0000 000000 00000"
    } else {
        "0000 0000 0000 0000"
    };
    let number_field = field("Card number", number_placeholder, &number, invalid);
    let expiry_field = field("Expiration date, M M slash Y Y", "MM/YY", &expiry, invalid);
    let cvc_aria = format!("Security code, {} digits", brand.cvc);
    let cvc_placeholder = if brand.cvc == 4 { "CVV" } else { "CVC" };
    let cvc_field = field(&cvc_aria, cvc_placeholder, &cvc, invalid);

    format!(
        "<div data-slot=\"credit-card-input\">{fieldset}<legend class=\"sr-only\">{label}</legend><span class=\"sr-only\" aria-live=\"polite\">{announce}</span>{}{number_field}{expiry_field}{cvc_field}{CHECK}</fieldset></div>",
        glyph(brand.id)
    )
}

fn field(aria: &str, placeholder: &str, value: &str, invalid: bool) -> String {
    let mut attrs = String::from(
        "type=\"text\" inputmode=\"numeric\" autocomplete=\"off\" autocorrect=\"off\" spellcheck=\"false\"",
    );
    attrs.push_str(&format!(" aria-label=\"{aria}\""));
    if invalid {
        attrs.push_str(" aria-invalid=\"true\"");
    }
    attrs.push_str(&format!(" placeholder=\"{placeholder}\""));
    if !value.is_empty() {
        attrs.push_str(&format!(" value=\"{value}\""));
    }
    format!("<input {attrs} />")
}

/// React `BrandGlyph`: monochrome 32x20 marks, lucide `credit-card` when unknown.
fn glyph(brand: &str) -> String {
    let open = "<svg viewBox=\"0 0 32 20\" aria-hidden=\"true\">";
    match brand {
        "visa" => format!("{open}<text x=\"16\" y=\"15\" text-anchor=\"middle\" font-size=\"10\" font-style=\"italic\" font-weight=\"800\" letter-spacing=\"0.4\" fill=\"currentColor\">VISA</text></svg>"),
        "mastercard" => format!("{open}<circle cx=\"12\" cy=\"10\" r=\"6.5\" fill=\"currentColor\" opacity=\"0.9\"></circle><circle cx=\"20\" cy=\"10\" r=\"6.5\" fill=\"currentColor\" opacity=\"0.45\"></circle></svg>"),
        "amex" => format!("{open}<text x=\"16\" y=\"14\" text-anchor=\"middle\" font-size=\"8\" font-weight=\"800\" letter-spacing=\"0.3\" fill=\"currentColor\">AMEX</text></svg>"),
        "elo" => format!("{open}<text x=\"16\" y=\"15\" text-anchor=\"middle\" font-size=\"11\" font-weight=\"800\" letter-spacing=\"0.2\" fill=\"currentColor\">elo</text></svg>"),
        "discover" => format!("{open}<text x=\"12\" y=\"14\" text-anchor=\"middle\" font-size=\"8\" font-weight=\"800\" fill=\"currentColor\">DISC</text><circle cx=\"26\" cy=\"10\" r=\"3.2\" fill=\"currentColor\"></circle></svg>"),
        _ => "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" data-brand-fallback=\"\"><rect width=\"20\" height=\"14\" x=\"2\" y=\"5\" rx=\"2\"></rect><line x1=\"2\" x2=\"22\" y1=\"10\" y2=\"10\"></line></svg>".into(),
    }
}

/// React `CARD_TYPES` order (first match wins; Elo before Visa/Discover).
fn brand_of(d: &str) -> Brand {
    let starts = |ps: &[&str]| ps.iter().any(|p| d.starts_with(p));
    let n = |len: usize| d.get(..len).and_then(|s| s.parse::<u32>().ok());
    if starts(&[
        "4011", "4312", "4389", "4514", "4576", "5041", "5066", "5067", "509", "6277", "6362",
        "6363", "650", "651", "655",
    ]) {
        return Brand {
            id: "elo",
            label: "Elo",
            gaps: &[4, 8, 12],
            cvc: 3,
        };
    }
    if starts(&["34", "37"]) {
        return Brand {
            id: "amex",
            label: "American Express",
            gaps: &[4, 10],
            cvc: 4,
        };
    }
    let mc = n(2).is_some_and(|v| (51..=55).contains(&v) || (23..=26).contains(&v))
        || n(4).is_some_and(|v| (2221..=2229).contains(&v) || v == 2720)
        || n(3).is_some_and(|v| (223..=229).contains(&v) || v == 270 || v == 271);
    if mc {
        return Brand {
            id: "mastercard",
            label: "Mastercard",
            gaps: &[4, 8, 12],
            cvc: 3,
        };
    }
    if d.starts_with('4') {
        return Brand {
            id: "visa",
            label: "Visa",
            gaps: &[4, 8, 12],
            cvc: 3,
        };
    }
    if starts(&["6011", "65", "622"]) || n(3).is_some_and(|v| (644..=649).contains(&v)) {
        return Brand {
            id: "discover",
            label: "Discover",
            gaps: &[4, 8, 12],
            cvc: 3,
        };
    }
    UNKNOWN
}

fn group_label(comp: &ComponentNode) -> String {
    if let Some(v) = attr_nonempty(comp, "label") {
        return esc(v);
    }
    if let Some(v) = attr_nonempty(comp, "aria-label") {
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
        if let Some(v) = attr_nonempty(comp, name) {
            return v.chars().filter(|c| c.is_ascii_digit()).take(19).collect();
        }
        if let Some(t) = item(comp, name).filter(|s| !s.is_empty()) {
            return t.chars().filter(|c| c.is_ascii_digit()).take(19).collect();
        }
    }
    String::new()
}

fn format_number(digits: &str, gaps: &[usize]) -> String {
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && gaps.contains(&i) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn reject_interact(html: &str) {
        assert!(!html.contains("credit-card-input-control"));
        assert!(!html.contains("<label"));
        assert!(!html.contains(" style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("height:2.5rem;padding:0 0.75rem"));
    }

    #[test]
    fn root_is_div_fieldset_legend_glyph_fields_and_check() {
        let html = render(&stub("credit-card-input", "Card"));
        assert!(html
            .starts_with("<div data-slot=\"credit-card-input\"><fieldset data-brand=\"unknown\">"));
        assert!(html.contains("<legend class=\"sr-only\">Card</legend>"));
        assert!(html.contains("placeholder=\"MM/YY\""));
        reject_interact(&html);
        let f = |aria: &str, ph: &str| {
            format!("<input type=\"text\" inputmode=\"numeric\" autocomplete=\"off\" autocorrect=\"off\" spellcheck=\"false\" aria-label=\"{aria}\" placeholder=\"{ph}\" />")
        };
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"credit-card-input\"><fieldset data-brand=\"unknown\"><legend class=\"sr-only\">Card</legend><span class=\"sr-only\" aria-live=\"polite\"></span>{}{}{}{}{CHECK}</fieldset></div>",
                glyph("unknown"),
                f("Card number", "0000 0000 0000 0000"),
                f("Expiration date, M M slash Y Y", "MM/YY"),
                f("Security code, 3 digits", "CVC"),
            )
        );
    }

    /// Fixture: `label "Credit card"`, `value:"4242424242424242"` → React's
    /// innerText "Credit card Visa card VISA" (legend + live region + glyph).
    #[test]
    fn visa_number_announces_brand_and_glyph() {
        let mut c = stub("credit-card-input", "Credit card");
        c.props.insert("value".into(), "4242424242424242".into());
        let html = render(&c);
        assert!(html.contains("<fieldset data-brand=\"visa\">"));
        assert!(html.contains("<legend class=\"sr-only\">Credit card</legend><span class=\"sr-only\" aria-live=\"polite\">Visa card</span><svg viewBox=\"0 0 32 20\" aria-hidden=\"true\"><text "));
        assert!(html.contains(">VISA</text>"));
        assert!(html.contains("value=\"4242 4242 4242 4242\""));
        reject_interact(&html);
    }

    #[test]
    fn brand_table_matches_react_order() {
        assert_eq!(brand_of("4011").id, "elo");
        assert_eq!(brand_of("378282").id, "amex");
        assert_eq!(brand_of("5555").id, "mastercard");
        assert_eq!(brand_of("2221").id, "mastercard");
        assert_eq!(brand_of("4111").id, "visa");
        assert_eq!(brand_of("6011").id, "discover");
        assert_eq!(brand_of("9999").id, "unknown");
        let mut c = stub("credit-card-input", "Card");
        c.props.insert("value".into(), "378282246310005".into());
        let html = render(&c);
        assert!(html.contains("value=\"3782 822463 10005\""));
        assert!(html.contains("placeholder=\"CVV\""));
        assert!(html.contains("aria-label=\"Security code, 4 digits\""));
    }

    #[test]
    fn disabled_and_invalid() {
        let mut c = stub("credit-card-input", "Card");
        c.props.insert("disabled".into(), "true".into());
        c.props.insert("invalid".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("<fieldset disabled data-invalid=\"true\" data-brand=\"unknown\">"));
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

    /// Wave 1t geometry: the root is an unstyled flex column (16px/24px type, no
    /// border); the bordered rounded-xl box is the fieldset (text-sm/20px).
    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"credit-card-input\"] {\n  display: flex; width: 100%; flex-direction: column; gap: 0.375rem;\n}"
        ));
        assert!(css.contains(
            "[data-slot=\"credit-card-input\"] > fieldset {\n  margin: 0; display: flex; min-width: 0; flex-wrap: wrap; align-items: center;\n  column-gap: 0.75rem; row-gap: 0.5rem;"
        ));
        assert!(css.contains("[data-slot=\"credit-card-input\"] input:nth-of-type(1) { min-width: 11ch; flex: 1; letter-spacing: 0.02em; }"));
    }
}
