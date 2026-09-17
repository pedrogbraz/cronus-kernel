//! Dedicated DatePicker renderer. DOM matches React `DatePicker`: the outline
//! `Button` trigger `<button data-slot="date-picker-trigger" data-variant="outline">`
//! (calendar glyph + `<span>` label) followed by the popover
//! `<div data-slot="date-picker-content" role="dialog">` (`w-auto p-0`) that
//! wraps the real `Calendar` (`cronus_ui_calendar::render_spec`, single mode).
//! Zero JS: the panel is a native `popover="auto"` opened by `popovertarget`
//! and anchored to the trigger; Esc / outside click dismiss it. Picking a day
//! needs JS, so the calendar's day buttons stay `disabled` (idle look).
//!
//! Props: `value:"YYYY-MM-DD"` (or `defaultValue:`) selects the day and labels
//! the trigger with date-fns `PPP` ("June 15th, 2026"); `placeholder:` (default
//! "Pick a date", also the dialog's accessible name); `defaultMonth:"YYYY-MM"`
//! picks the month when there is no value; `today:"YYYY-MM-DD"`;
//! `disabled:true`; `invalid:true` (`aria-invalid`, error border and ring);
//! `aria-label:"…"`; `name:"…"` mirrors the value into a hidden input for
//! native forms, like React. Not interact `input("date-picker", "date")`.

use crate::cronus_ui_calendar::{self, month_name, parse_day, Spec, FALLBACK_MONTH};
use crate::cronus_ui_kit::{attr, attr_nonempty, esc, flag, item, label_of};
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

pub fn render(comp: &ComponentNode) -> String {
    let placeholder = placeholder_of(comp);
    let value = value_of(comp);
    let label = match value {
        Some(v) => format_ppp(v),
        None => placeholder.clone(),
    };
    let trigger_id = crate::cronus_ui_kit::widget_id(comp, "trigger");
    let pop_id = crate::cronus_ui_kit::widget_id(comp, "cal");
    let mut attrs = format!(
        "type=\"button\" id=\"{trigger_id}\" data-slot=\"date-picker-trigger\" data-variant=\"outline\" popovertarget=\"{pop_id}\" aria-haspopup=\"dialog\""
    );
    if value.is_none() {
        attrs.push_str(" data-empty=\"\"");
    }
    if flag(comp, "disabled") {
        attrs.push_str(" disabled");
    }
    if flag(comp, "invalid") {
        attrs.push_str(" aria-invalid=\"true\"");
    }
    if let Some(aria) = attr_nonempty(comp, "aria-label") {
        attrs.push_str(&format!(" aria-label=\"{}\"", esc(aria)));
    }
    let (year, month) = match value.or_else(|| month_of(comp)) {
        Some((y, m, _)) => (y, m),
        None => FALLBACK_MONTH,
    };
    let calendar = cronus_ui_calendar::render_spec(&Spec {
        year,
        month,
        months: 1,
        selected: value,
        range: None,
        today: attr(comp, "today").and_then(parse_day),
        fixed_weeks: false,
    });
    let hidden = attr_nonempty(comp, "name")
        .map(|name| {
            let v = value
                .map(|(y, m, d)| format!("{y:04}-{m:02}-{d:02}"))
                .unwrap_or_default();
            format!(
                "<input type=\"hidden\" name=\"{}\" value=\"{v}\">",
                esc(name)
            )
        })
        .unwrap_or_default();
    format!(
        "<button {attrs}>{ICON}<span>{label}</span></button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"date-picker-content\" role=\"dialog\" aria-label=\"{placeholder}\" anchor=\"{trigger_id}\">{calendar}</div>{hidden}"
    )
}

fn value_of(comp: &ComponentNode) -> Option<cronus_ui_calendar::Date> {
    attr_nonempty(comp, "value")
        .or_else(|| attr_nonempty(comp, "defaultValue"))
        .or_else(|| item(comp, "value").filter(|s| !s.is_empty()))
        .and_then(parse_day)
}

fn month_of(comp: &ComponentNode) -> Option<cronus_ui_calendar::Date> {
    let raw = attr_nonempty(comp, "defaultMonth")?;
    parse_day(raw).or_else(|| {
        let (y, m) = raw.trim().split_once('-')?;
        let m = m.parse::<u32>().ok().filter(|m| (1..=12).contains(m))?;
        Some((y.parse().ok()?, m, 1))
    })
}

/// date-fns `PPP` (en-US): "June 15th, 2026".
fn format_ppp((year, month, day): cronus_ui_calendar::Date) -> String {
    let suffix = match (day % 10, day % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{} {day}{suffix}, {year}", month_name(month))
}

fn placeholder_of(comp: &ComponentNode) -> String {
    if let Some(v) = attr_nonempty(comp, "placeholder") {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn reject_interact(html: &str) {
        assert!(!html.contains("type=\"date\""));
        assert!(!html.contains("date-picker-control"));
        assert!(!html.contains("<label"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("height:2.5rem;padding:0 0.75rem"));
    }

    #[test]
    fn root_is_trigger_button_with_real_calendar_popover() {
        let html = render(&stub("date-picker", "Due date"));
        assert!(html.starts_with(
            "<button type=\"button\" id=\"cui-date-picker-trigger\" data-slot=\"date-picker-trigger\" data-variant=\"outline\" popovertarget=\"cui-date-picker-cal\" aria-haspopup=\"dialog\" data-empty=\"\">"
        ));
        assert!(!html.contains("data-slot=\"date-picker\""));
        assert!(html.contains("<span>Due date</span></button><div id=\"cui-date-picker-cal\" popover=\"auto\" data-slot=\"date-picker-content\" role=\"dialog\" aria-label=\"Due date\" anchor=\"cui-date-picker-trigger\"><div data-slot=\"calendar\">"));
        assert!(html.contains(">September 2026</span>"));
        assert!(html.contains("<table role=\"grid\" aria-label=\"September 2026\">"));
        assert!(!html.contains("aria-selected"));
        assert!(!html.contains("<input"));
        assert!(html.ends_with("</table></div></div></div></div></div>"));
        reject_interact(&html);
    }

    #[test]
    fn value_labels_trigger_and_selects_day() {
        let mut c = stub("date-picker", "Due date");
        c.props.insert("value".into(), "2026-09-13".into());
        let html = render(&c);
        assert!(html.contains("<span>September 13th, 2026</span></button>"));
        assert!(!html.contains("data-empty"));
        assert!(html.contains(">September 2026</span>"));
        assert!(html.contains("<td role=\"gridcell\" aria-selected=\"true\"><button type=\"button\" disabled aria-label=\"Sunday, September 13th, 2026, selected\">13</button></td>"));
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
        assert!(html.contains(">June 2026</span>"));
        reject_interact(&html);
    }

    #[test]
    fn ppp_ordinals() {
        assert_eq!(format_ppp((2026, 6, 1)), "June 1st, 2026");
        assert_eq!(format_ppp((2026, 6, 2)), "June 2nd, 2026");
        assert_eq!(format_ppp((2026, 6, 3)), "June 3rd, 2026");
        assert_eq!(format_ppp((2026, 6, 11)), "June 11th, 2026");
        assert_eq!(format_ppp((2026, 6, 22)), "June 22nd, 2026");
        let mut c = stub("date-picker", "Due date");
        c.props.insert("value".into(), "not-a-date".into());
        assert!(render(&c).contains("data-empty"));
    }

    /// Docs "Pick a date": no value, `placeholder="Pick a date"` — the trigger
    /// shows the placeholder (tertiary) and the calendar opens on a month.
    #[test]
    fn placeholder_prop_wins_without_value_and_default_month_picks_month() {
        let mut c = stub("date-picker", "Due date");
        c.props.insert("placeholder".into(), "Pick a date".into());
        c.props.insert("defaultMonth".into(), "2026-06".into());
        c.props.insert("today".into(), "2026-06-15".into());
        let html = render(&c);
        assert!(html.contains("<span>Pick a date</span></button>"));
        assert!(html.contains("aria-label=\"Pick a date\""));
        assert!(html.contains(">June 2026</span>"));
        assert!(html.contains("data-today=\"true\""));
        reject_interact(&html);
    }

    #[test]
    fn disabled_invalid_and_form_name() {
        let mut c = stub("date-picker", "Due date");
        c.props.insert("disabled".into(), "true".into());
        c.props.insert("invalid".into(), "true".into());
        c.props.insert("name".into(), "due".into());
        c.props.insert("defaultValue".into(), "2026-01-02".into());
        let html = render(&c);
        assert!(html.contains(" disabled"));
        assert!(html.contains("aria-invalid=\"true\""));
        assert!(html.ends_with("<input type=\"hidden\" name=\"due\" value=\"2026-01-02\">"));
        assert!(html.contains("<span>January 2nd, 2026</span>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_native_date_input() {
        let c = stub("date-picker", "Due date");
        let html = render(&c);
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
        assert!(!css.contains("[data-slot=\"date-picker-calendar\"]"));
        assert!(css.contains("[data-slot=\"date-picker-content\"]:popover-open {\n  z-index: 50; width: auto; padding: 0;"));
        assert!(css.contains("[data-slot=\"date-picker-trigger\"][aria-invalid=\"true\"] {\n  border-color: var(--cronus-error);\n  box-shadow: 0 0 0 2px color-mix(in oklab, var(--cronus-error) 30%, transparent);"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-shadow-lg"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
