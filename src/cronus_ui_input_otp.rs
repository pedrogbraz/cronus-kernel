//! Dedicated InputOTP renderer. DOM matches React:
//! `<div data-slot="input-otp" role="group">` wrapping
//! `<div data-slot="input-otp-group">` and six `<div data-slot="input-otp-slot">`.
//! Not interact `otp()` (`<fieldset style=BASE>` + `maxlength=1` inputs with CTRL).

use crate::cronus_ui_kit::esc;
use crate::parser::ComponentNode;

const SLOTS: usize = 6;

pub fn render(comp: &ComponentNode) -> String {
    let n = length_of(comp);
    let digits = digits_of(comp);
    let mut group = String::new();
    for i in 0..n {
        let ch = digits.chars().nth(i).map(|c| esc(&c.to_string())).unwrap_or_default();
        group.push_str(&format!("<div data-slot=\"input-otp-slot\">{ch}</div>"));
    }
    let label = aria_label_of(comp);
    format!(
        "<div data-slot=\"input-otp\" role=\"group\" aria-label=\"{label}\"><div data-slot=\"input-otp-group\">{group}</div></div>"
    )
}

fn length_of(comp: &ComponentNode) -> usize {
    let raw = comp
        .props
        .get("length")
        .or_else(|| comp.props.get("maxlength"))
        .and_then(|s| s.parse::<usize>().ok());
    match raw {
        Some(n) if (1..=8).contains(&n) => n,
        _ => SLOTS,
    }
}

fn digits_of(comp: &ComponentNode) -> String {
    let raw = comp
        .props
        .get("value")
        .map(String::as_str)
        .or_else(|| {
            comp.items
                .iter()
                .find(|i| i.item_type == "value" && !i.text.is_empty())
                .map(|i| i.text.as_str())
        })
        .unwrap_or("");
    raw.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn aria_label_of(comp: &ComponentNode) -> String {
    if let Some(v) = comp.props.get("aria-label").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    "One-time passcode".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn reject_interact(html: &str) {
        assert!(!html.contains("<fieldset"));
        assert!(!html.contains("<legend"));
        assert!(!html.contains("<input"));
        assert!(!html.contains("maxlength"));
        assert!(!html.contains("style="));
        assert!(!html.contains("-control"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("font-variant-numeric:tabular-nums"));
    }

    #[test]
    fn root_is_group_of_six_slots_not_fieldset() {
        let html = render(&stub("input-otp", "Code"));
        assert!(html.starts_with(
            "<div data-slot=\"input-otp\" role=\"group\" aria-label=\"One-time passcode\">"
        ));
        assert!(html.contains("<div data-slot=\"input-otp-group\">"));
        assert_eq!(html.matches("data-slot=\"input-otp-slot\"").count(), 6);
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"input-otp\" role=\"group\" aria-label=\"One-time passcode\"><div data-slot=\"input-otp-group\"><div data-slot=\"input-otp-slot\"></div><div data-slot=\"input-otp-slot\"></div><div data-slot=\"input-otp-slot\"></div><div data-slot=\"input-otp-slot\"></div><div data-slot=\"input-otp-slot\"></div><div data-slot=\"input-otp-slot\"></div></div></div>"
        );
    }

    #[test]
    fn value_fills_slots() {
        let mut c = stub("input-otp", "Code");
        c.props.insert("value".into(), "12ab3".into());
        let html = render(&c);
        assert!(html.contains("<div data-slot=\"input-otp-slot\">1</div>"));
        assert!(html.contains("<div data-slot=\"input-otp-slot\">2</div>"));
        assert!(html.contains("<div data-slot=\"input-otp-slot\">3</div>"));
        assert!(!html.contains(">a</div>"));
        assert_eq!(html.matches("data-slot=\"input-otp-slot\"").count(), 6);
        reject_interact(&html);
    }

    #[test]
    fn length_from_props() {
        let mut c = stub("input-otp", "Code");
        c.props.insert("length".into(), "4".into());
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"input-otp-slot\"").count(), 4);
        reject_interact(&html);
    }

    #[test]
    fn aria_label_from_props() {
        let mut c = stub("input-otp", "Code");
        c.props.insert("aria-label".into(), "Login code".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Login code\""));
        assert!(!html.contains("aria-label=\"One-time passcode\""));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_fieldset_ctrl() {
        let c = stub("input-otp", "Code");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("input-otp", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<fieldset data-slot=\"input-otp\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("<input data-slot=\"input-otp-slot\""));
        assert!(interact.contains("maxlength=\"1\""));
        assert!(interact.contains("width:2.5rem;text-align:center;font-variant-numeric:tabular-nums"));
        assert!(!html.contains("<fieldset"));
        assert!(!html.contains("<input"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("input-otp", "Code"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"input-otp\"]"));
        assert!(css.contains("[data-slot=\"input-otp-group\"]"));
        assert!(css.contains("[data-slot=\"input-otp-slot\"]"));
        assert!(css.contains("height: 2.5rem"));
        assert!(css.contains("width: 2.5rem"));
        assert!(css.contains("align-items: center"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
    }
}
