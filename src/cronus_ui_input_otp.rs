//! Dedicated InputOTP renderer. DOM mirrors React (input-otp lib):
//! `<div data-input-otp-container>` holding `input-otp-group` of
//! `input-otp-slot`s plus an overlay `<div><input data-slot="input-otp">`.
//! The real input sits transparent over the slots; zero JS, so slots show the
//! initial value only. Not interact `otp()` (`<fieldset style=BASE>` +
//! `maxlength=1` inputs with CTRL).

use crate::cronus_ui_kit::esc;
use crate::parser::ComponentNode;

const SLOTS: usize = 6;

pub fn render(comp: &ComponentNode) -> String {
    let n = length_of(comp);
    let digits: String = digits_of(comp).chars().take(n).collect();
    let mut group = String::new();
    for i in 0..n {
        let ch = digits
            .chars()
            .nth(i)
            .map(|c| c.to_string())
            .unwrap_or_default();
        group.push_str(&format!("<div data-slot=\"input-otp-slot\">{ch}</div>"));
    }
    let label = aria_label_of(comp);
    format!(
        "<div data-input-otp-container=\"true\"><div data-slot=\"input-otp-group\">{group}</div><div><input data-slot=\"input-otp\" autocomplete=\"one-time-code\" aria-label=\"{label}\" inputmode=\"numeric\" maxlength=\"{n}\" value=\"{digits}\" /></div></div>"
    )
}

fn length_of(comp: &ComponentNode) -> usize {
    let raw = ["length", "maxlength", "maxLength"]
        .iter()
        .find_map(|k| attr(comp, k))
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
    if let Some(v) = attr(comp, "aria-label").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    "One-time passcode".into()
}

/// Props first; `key:value` after an item line lands in that item's config
/// (the tokenizer has no newlines).
fn attr<'a>(comp: &'a ComponentNode, name: &str) -> Option<&'a str> {
    if let Some(v) = comp.props.get(name) {
        return Some(v.as_str());
    }
    comp.items
        .iter()
        .find_map(|i| i.config.get(name).map(String::as_str))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn slots(values: &[&str]) -> String {
        values
            .iter()
            .map(|v| format!("<div data-slot=\"input-otp-slot\">{v}</div>"))
            .collect()
    }

    fn expected(slot_html: &str, label: &str, n: usize, value: &str) -> String {
        format!(
            "<div data-input-otp-container=\"true\"><div data-slot=\"input-otp-group\">{slot_html}</div><div><input data-slot=\"input-otp\" autocomplete=\"one-time-code\" aria-label=\"{label}\" inputmode=\"numeric\" maxlength=\"{n}\" value=\"{value}\" /></div></div>"
        )
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<fieldset"));
        assert!(!html.contains("<legend"));
        assert!(!html.contains("<input data-slot=\"input-otp-slot\""));
        assert!(!html.contains("maxlength=\"1\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("-control"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("font-variant-numeric:tabular-nums"));
    }

    /// React (input-otp lib): container > group of slots + overlay div with
    /// the real `<input data-slot="input-otp">`. Audit e2e expects INPUT.
    #[test]
    fn slot_is_overlay_input_like_react() {
        let html = render(&stub("input-otp", "Code"));
        assert_eq!(
            html,
            expected(
                &slots(&["", "", "", "", "", ""]),
                "One-time passcode",
                6,
                ""
            )
        );
        assert_eq!(html.matches("data-slot=\"input-otp\"").count(), 1);
        assert!(!html.contains("role=\"group\""));
        reject_interact(&html);
    }

    #[test]
    fn value_fills_slots_and_input() {
        let mut c = stub("input-otp", "Code");
        c.props.insert("value".into(), "12ab3".into());
        let html = render(&c);
        assert_eq!(
            html,
            expected(
                &slots(&["1", "2", "3", "", "", ""]),
                "One-time passcode",
                6,
                "123"
            )
        );
        reject_interact(&html);
    }

    #[test]
    fn length_from_props() {
        for key in ["length", "maxLength"] {
            let mut c = stub("input-otp", "Code");
            c.props.insert(key.into(), "4".into());
            let html = render(&c);
            assert_eq!(
                html.matches("data-slot=\"input-otp-slot\"").count(),
                4,
                "{key}"
            );
            assert!(html.contains("maxlength=\"4\""), "{key}");
            reject_interact(&html);
        }
    }

    #[test]
    fn aria_label_from_props_or_item_config() {
        let mut c = stub("input-otp", "Code");
        c.props.insert("aria-label".into(), "Login code".into());
        assert!(render(&c).contains("aria-label=\"Login code\""));
        let mut c = stub("input-otp", "Code");
        c.items[0]
            .config
            .insert("aria-label".into(), "A <B>".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"A &lt;B&gt;\""));
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
        assert!(
            interact.contains("width:2.5rem;text-align:center;font-variant-numeric:tabular-nums")
        );
        assert!(!html.contains("<fieldset"));
        assert!(html.contains("<input data-slot=\"input-otp\""));
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
    fn chrome_overlays_transparent_input_on_slots() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-input-otp-container] {"));
        assert!(css.contains("[data-input-otp-container] > div:last-child {"));
        assert!(css.contains("[data-slot=\"input-otp\"] {"));
        assert!(css.contains("[data-slot=\"input-otp-group\"]"));
        assert!(css.contains("[data-slot=\"input-otp-slot\"]"));
        assert!(css.contains("caret-color: transparent"));
        assert!(css.contains("height: 2.5rem"));
        assert!(css.contains("width: 2.5rem"));
        // wave1s geometry parity: React container inherits 24px line-height,
        // slots are text-sm (20px); left border exists but is 0 wide except first.
        assert!(css.contains(
            "[data-input-otp-container] {\n  position: relative; display: flex; align-items: center; gap: 0.5rem;\n  line-height: 1.5; cursor: text; user-select: none; pointer-events: none;\n}"
        ));
        assert!(css.contains(
            "  border: 0 solid var(--cronus-border);\n  border-top-width: 1px; border-right-width: 1px; border-bottom-width: 1px;\n  font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg);"
        ));
        assert!(
            css.contains("[data-slot=\"input-otp-slot\"]:first-child {\n  border-left-width: 1px;")
        );
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
    }
}
