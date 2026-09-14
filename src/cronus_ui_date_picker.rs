//! Dedicated DatePicker renderer. Always-open static DOM (no JS).
//! `<button type="button" data-slot="date-picker-trigger">` plus
//! `<div data-slot="date-picker-content">` with a tiny static calendar grid.
//! Not interact `input("date-picker", "date")` (`<label>` + `type="date"` + `*-control`).

use crate::cronus_ui_kit::{esc, item, label_of};
use crate::parser::ComponentNode;

const ICON: &str = concat!(
    "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" ",
    "stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" focusable=\"false\">",
    "<rect x=\"3\" y=\"4\" width=\"18\" height=\"18\" rx=\"2\" />",
    "<line x1=\"16\" y1=\"2\" x2=\"16\" y2=\"6\" />",
    "<line x1=\"8\" y1=\"2\" x2=\"8\" y2=\"6\" />",
    "<line x1=\"3\" y1=\"10\" x2=\"21\" y2=\"10\" />",
    "</svg>",
);

const WEEKDAYS: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

pub fn render(comp: &ComponentNode) -> String {
    let label = trigger_label(comp);
    let placeholder = placeholder_of(comp);
    let trigger_id = crate::cronus_ui_kit::widget_id(comp, "trigger");
    let pop_id = crate::cronus_ui_kit::widget_id(comp, "cal");
    let mut attrs = format!(
        "type=\"button\" id=\"{trigger_id}\" data-slot=\"date-picker-trigger\" data-variant=\"outline\" popovertarget=\"{pop_id}\" aria-haspopup=\"dialog\""
    );
    if !has_value(comp) {
        attrs.push_str(" data-empty=\"\"");
    }
    if flag(comp, "disabled") {
        attrs.push_str(" disabled");
    }
    if flag(comp, "invalid") {
        attrs.push_str(" aria-invalid=\"true\"");
    }
    if let Some(aria) = attr(comp, "aria-label").filter(|s| !s.is_empty()) {
        attrs.push_str(&format!(" aria-label=\"{}\"", esc(aria)));
    }
    let grid = calendar_grid(selected_day(comp), &caption_of(comp));
    format!(
        "<button {attrs}>{ICON}<span>{label}</span></button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"date-picker-content\" role=\"dialog\" aria-label=\"{placeholder}\" anchor=\"{trigger_id}\">{grid}</div>"
    )
}

fn has_value(comp: &ComponentNode) -> bool {
    attr(comp, "value")
        .or_else(|| item(comp, "value"))
        .is_some_and(|s| !s.is_empty())
}

fn trigger_label(comp: &ComponentNode) -> String {
    let raw = attr(comp, "value")
        .filter(|s| !s.is_empty())
        .or_else(|| item(comp, "value").filter(|s| !s.is_empty()));
    match raw {
        Some(v) => format_ppp(v).unwrap_or_else(|| esc(v)),
        None => placeholder_of(comp),
    }
}

/// date-fns `PPP` (en-US) for a local `YYYY-MM-DD`: "June 15th, 2026".
fn format_ppp(raw: &str) -> Option<String> {
    let mut parts = raw.split('-');
    let year: u32 = parts.next()?.parse().ok()?;
    let month: usize = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;
    if parts.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let suffix = match (day % 10, day % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    Some(format!("{} {day}{suffix}, {year}", MONTHS[month - 1]))
}

fn placeholder_of(comp: &ComponentNode) -> String {
    if let Some(v) = attr(comp, "placeholder").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    if let Some(t) = item(comp, "placeholder").filter(|s| !s.is_empty()) {
        return esc(t);
    }
    let label = label_of(comp);
    if label.is_empty() {
        "Pick a date".into()
    } else {
        label
    }
}

fn selected_day(comp: &ComponentNode) -> Option<u8> {
    let raw = attr(comp, "value").or_else(|| item(comp, "value"))?;
    let day = raw.rsplit('-').next()?.parse::<u8>().ok()?;
    if (1..=28).contains(&day) {
        Some(day)
    } else {
        None
    }
}

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

fn caption_of(comp: &ComponentNode) -> String {
    let raw = attr(comp, "value")
        .or_else(|| item(comp, "value"))
        .unwrap_or("");
    if let Some((year, month)) = parse_year_month(raw) {
        if (1..=12).contains(&month) {
            return format!("{} {year}", MONTHS[(month as usize) - 1]);
        }
    }
    "September 2026".into()
}

fn parse_year_month(raw: &str) -> Option<(u16, u8)> {
    let mut parts = raw.split('-');
    let year = parts.next()?.parse().ok()?;
    let month = parts.next()?.parse().ok()?;
    Some((year, month))
}

fn calendar_grid(selected: Option<u8>, caption: &str) -> String {
    let mut out = String::from("<div data-slot=\"date-picker-calendar\" role=\"grid\">");
    out.push_str(&format!(
        "<div data-slot=\"date-picker-caption\">{caption}</div>"
    ));
    out.push_str("<div role=\"row\">");
    for d in WEEKDAYS {
        out.push_str(&format!(
            "<span data-slot=\"date-picker-weekday\" role=\"columnheader\">{d}</span>"
        ));
    }
    out.push_str("</div>");
    for week in 0..4 {
        out.push_str("<div role=\"row\">");
        for offset in 1..=7 {
            let day = week * 7 + offset;
            let selected_attr = if selected == Some(day as u8) {
                " aria-selected=\"true\""
            } else {
                " aria-selected=\"false\""
            };
            out.push_str(&format!(
                "<button type=\"button\" disabled data-slot=\"date-picker-day\" role=\"gridcell\"{selected_attr}>{day}</button>"
            ));
        }
        out.push_str("</div>");
    }
    out.push_str("</div>");
    out
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
        assert!(!html.contains("type=\"date\""));
        assert!(!html.contains("date-picker-control"));
        assert!(!html.contains("<label"));
        assert!(!html.contains("<input"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("height:2.5rem;padding:0 0.75rem"));
    }

    #[test]
    fn root_is_trigger_button_not_native_date_input() {
        let html = render(&stub("date-picker", "Due date"));
        assert!(html.starts_with(
            "<button type=\"button\" id=\"cui-date-picker-trigger\" data-slot=\"date-picker-trigger\" data-variant=\"outline\" popovertarget=\"cui-date-picker-cal\" aria-haspopup=\"dialog\" data-empty=\"\">"
        ));
        assert!(!html.contains("data-slot=\"date-picker\""));
        assert!(html.contains("data-slot=\"date-picker-content\" role=\"dialog\""));
        assert!(html.contains("popovertarget="));
        assert!(html.contains("<span>Due date</span></button>"));
        assert!(html.contains("data-slot=\"date-picker-content\""));
        assert!(html.contains("data-slot=\"date-picker-calendar\""));
        assert!(html.contains("data-slot=\"date-picker-caption\""));
        assert!(html.contains("September 2026"));
        assert!(html.contains("data-slot=\"date-picker-day\""));
        assert!(html.contains("role=\"grid\""));
        assert!(!html.contains("type=\"date\""));
        reject_interact(&html);
    }

    #[test]
    fn value_labels_trigger_and_selects_day() {
        let mut c = stub("date-picker", "Due date");
        c.props.insert("value".into(), "2026-09-13".into());
        let html = render(&c);
        assert!(html.contains("<span>September 13th, 2026</span></button>"));
        assert!(!html.contains("data-empty"));
        assert!(html.contains("September 2026"));
        assert!(html.contains("aria-selected=\"true\">13</button>"));
        assert!(!html.contains("type=\"date\""));
        reject_interact(&html);
    }

    #[test]
    fn emitted_fixture_trigger_matches_react_ppp_label() {
        let mut c = stub("date-picker", "Pick a date");
        c.items.push(crate::parser::ComponentItemNode {
            item_type: "text".into(),
            text: "Pick a date".into(),
            link: None,
            tone: None,
            config: [
                ("value".to_string(), "2026-06-15".to_string()),
                ("aria-label".to_string(), "Due date".to_string()),
            ]
            .into_iter()
            .collect(),
        });
        let html = render(&c);
        assert!(html.starts_with(
            "<button type=\"button\" id=\"cui-date-picker-trigger\" data-slot=\"date-picker-trigger\" data-variant=\"outline\" popovertarget=\"cui-date-picker-cal\" aria-haspopup=\"dialog\" aria-label=\"Due date\">"
        ));
        assert!(html.contains(
            "<span>June 15th, 2026</span></button><div id=\"cui-date-picker-cal\" popover=\"auto\""
        ));
        reject_interact(&html);
    }

    #[test]
    fn ppp_ordinals() {
        assert_eq!(format_ppp("2026-06-01").as_deref(), Some("June 1st, 2026"));
        assert_eq!(format_ppp("2026-06-02").as_deref(), Some("June 2nd, 2026"));
        assert_eq!(format_ppp("2026-06-03").as_deref(), Some("June 3rd, 2026"));
        assert_eq!(format_ppp("2026-06-11").as_deref(), Some("June 11th, 2026"));
        assert_eq!(format_ppp("2026-06-22").as_deref(), Some("June 22nd, 2026"));
        assert_eq!(format_ppp("not-a-date"), None);
    }

    #[test]
    fn placeholder_prop_wins_without_value() {
        let mut c = stub("date-picker", "Due date");
        c.props.insert("placeholder".into(), "Pick a date".into());
        let html = render(&c);
        assert!(html.contains("<span>Pick a date</span></button>"));
        assert!(html.contains("aria-label=\"Pick a date\""));
        reject_interact(&html);
    }

    #[test]
    fn disabled_and_invalid() {
        let mut c = stub("date-picker", "Due date");
        c.props.insert("disabled".into(), "true".into());
        c.props.insert("invalid".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(" disabled"));
        assert!(html.contains("aria-invalid=\"true\""));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_native_date_input() {
        let c = stub("date-picker", "Due date");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("date-picker", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<label data-slot=\"date-picker\""));
        assert!(interact.contains("data-slot=\"date-picker-control\""));
        assert!(interact.contains("type=\"date\""));
        assert!(interact.contains("style="));
        assert!(!html.contains("date-picker-control"));
        assert!(!html.contains("type=\"date\""));
        assert!(html.contains("data-slot=\"date-picker-trigger\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("date-picker", "Due date"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"date-picker-trigger\"] {\n  display: inline-flex; align-items: center; justify-content: flex-start; gap: 0.5rem;\n  width: 15rem; height: 2.5rem; padding: 0 1rem; box-sizing: border-box;"));
        assert!(css.contains(
            "font: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 400;"
        ));
        assert!(css.contains(
            "[data-slot=\"date-picker-trigger\"][data-empty] { color: var(--cronus-fg-tertiary); }"
        ));
        assert!(!css.contains("[data-slot=\"date-picker\"],"));
        assert!(css.contains("[data-slot=\"date-picker-content\"]"));
        assert!(css.contains("[data-slot=\"date-picker-calendar\"]"));
        assert!(css.contains("[data-slot=\"date-picker-day\"]"));
        assert!(css.contains("[data-slot=\"date-picker-caption\"]"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-shadow-lg"));
        assert!(css.contains("z-index: 50"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
