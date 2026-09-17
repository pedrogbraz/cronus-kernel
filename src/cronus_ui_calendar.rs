//! Dedicated Calendar renderer. DOM mirrors React (`<div data-slot="calendar">`
//! wrapping react-day-picker v9): root `<div>` (p-3) > months `<div>` >
//! `<nav>` (previous / next month buttons) + one month `<div>` per shown month
//! > caption `<div><span role="status">` + `<table role="grid">` with a Su–Sa
//! `<thead>` and one flex `<tr>` per week of day `<button>`s.
//! Real month layout: leading/trailing outside days (`showOutsideDays`) and only
//! as many weeks as the month needs, or six with `fixedWeeks:true`.
//!
//! Props (React names, `.cronus` spelling): `defaultMonth:"YYYY-MM[-DD]"` picks
//! the month; `selected:"YYYY-MM-DD"` (or `value:`) marks a day in single mode;
//! `mode:range` with `from:"YYYY-MM-DD"` / `to:"YYYY-MM-DD"` (or
//! `selected:"YYYY-MM-DD..YYYY-MM-DD"`) marks a range (`day-range-start`,
//! `day-range-middle`, `day-range-end` on the cells, like react-day-picker's
//! modifier classes); `today:"YYYY-MM-DD"` highlights a day (the kernel has no
//! clock, so nothing is "today" by default); `numberOfMonths:2` shows
//! consecutive months; `fixedWeeks:true` pads to six rows. A `Month YYYY` label
//! also names the month. Cells carry react-day-picker's `data-outside`,
//! `data-today` and `aria-selected`.
//!
//! Zero JS: month navigation and day selection need JS, so every `<button>` is
//! rendered `disabled` with React's idle look (only the nav buttons are dimmed,
//! as in React). `render_spec` is shared with the date pickers, which embed the
//! same calendar inside their popovers.

use crate::cronus_ui_kit::{attr, attr_num, flag, item, label_of};
use crate::parser::ComponentNode;

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
const WEEKDAYS: [(&str, &str); 7] = [
    ("Su", "Sunday"),
    ("Mo", "Monday"),
    ("Tu", "Tuesday"),
    ("We", "Wednesday"),
    ("Th", "Thursday"),
    ("Fr", "Friday"),
    ("Sa", "Saturday"),
];
const CHEVRON_LEFT: &str = concat!(
    "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" ",
    "stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" focusable=\"false\">",
    "<path d=\"m15 18-6-6 6-6\" /></svg>",
);
const CHEVRON_RIGHT: &str = concat!(
    "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" ",
    "stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" focusable=\"false\">",
    "<path d=\"m9 18 6-6-6-6\" /></svg>",
);
/// Month shown when nothing names one (no clock: the render stays deterministic).
pub const FALLBACK_MONTH: (i32, u32) = (2026, 9);

/// A calendar date `(year, month, day)`; tuples compare chronologically.
pub type Date = (i32, u32, u32);

/// What one calendar shows. Built by [`spec_of`] from a component, or by the
/// date pickers from their own props.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Spec {
    pub year: i32,
    pub month: u32,
    /// Consecutive months to show (React `numberOfMonths`, at least 1).
    pub months: u32,
    /// Single-mode selection.
    pub selected: Option<Date>,
    /// Range-mode selection (`from`, optional `to`).
    pub range: Option<(Date, Option<Date>)>,
    pub today: Option<Date>,
    pub fixed_weeks: bool,
}

pub fn render(comp: &ComponentNode) -> String {
    render_spec(&spec_of(comp))
}

/// The `<div data-slot="calendar">` for a spec.
pub fn render_spec(spec: &Spec) -> String {
    let nav = format!(
        "<nav aria-label=\"Navigation bar\"><button type=\"button\" disabled aria-label=\"Go to the Previous Month\">{CHEVRON_LEFT}</button><button type=\"button\" disabled aria-label=\"Go to the Next Month\">{CHEVRON_RIGHT}</button></nav>"
    );
    let (mut year, mut month) = (spec.year, spec.month);
    let mut months = String::new();
    for _ in 0..spec.months.max(1) {
        months.push_str(&month_html(spec, year, month));
        (year, month) = next_month(year, month);
    }
    format!("<div data-slot=\"calendar\"><div><div>{nav}{months}</div></div></div>")
}

fn month_html(spec: &Spec, year: i32, month: u32) -> String {
    let caption = format!("{} {year}", MONTHS[(month - 1) as usize]);
    let head = WEEKDAYS
        .iter()
        .map(|(short, long)| format!("<th scope=\"col\" aria-label=\"{long}\">{short}</th>"))
        .collect::<String>();
    let (py, pm) = prev_month(year, month);
    let (ny, nm) = next_month(year, month);
    let lead = weekday(year, month, 1);
    let prev_days = days_in_month(py, pm);
    let mut cells: Vec<String> = Vec::new();
    for i in 0..lead {
        cells.push(cell(spec, (py, pm, prev_days - lead + 1 + i), true));
    }
    for d in 1..=days_in_month(year, month) {
        cells.push(cell(spec, (year, month, d), false));
    }
    let mut next = 1;
    while cells.len() % 7 != 0 || (spec.fixed_weeks && cells.len() < 42) {
        cells.push(cell(spec, (ny, nm, next), true));
        next += 1;
    }
    let rows = cells
        .chunks(7)
        .map(|week| format!("<tr>{}</tr>", week.concat()))
        .collect::<String>();
    format!(
        "<div><div><span role=\"status\" aria-live=\"polite\">{caption}</span></div><table role=\"grid\" aria-label=\"{caption}\"><thead aria-hidden=\"true\"><tr>{head}</tr></thead><tbody>{rows}</tbody></table></div>"
    )
}

fn cell(spec: &Spec, date: Date, outside: bool) -> String {
    let (year, month, day) = date;
    let mut classes: Vec<&str> = Vec::new();
    let mut selected = spec.selected == Some(date);
    if let Some((from, to)) = spec.range {
        let to = to.unwrap_or(from);
        if date == from {
            classes.push("day-range-start");
        }
        if date == to {
            classes.push("day-range-end");
        }
        if date > from && date < to {
            classes.push("day-range-middle");
        }
        selected |= date >= from && date <= to;
    }
    let mut attrs = String::from(" role=\"gridcell\"");
    if !classes.is_empty() {
        attrs.push_str(&format!(" class=\"{}\"", classes.join(" ")));
    }
    if outside {
        attrs.push_str(" data-outside=\"true\"");
    }
    let today = spec.today == Some(date);
    if today {
        attrs.push_str(" data-today=\"true\"");
    }
    if selected {
        attrs.push_str(" aria-selected=\"true\"");
    }
    let label = format!(
        "{}{}, {} {day}{}, {year}{}",
        if today { "Today, " } else { "" },
        WEEKDAYS[weekday(year, month, day) as usize].1,
        MONTHS[(month - 1) as usize],
        ordinal(day),
        if selected { ", selected" } else { "" }
    );
    format!(
        "<td{attrs}><button type=\"button\" disabled aria-label=\"{label}\">{day}</button></td>"
    )
}

fn ordinal(day: u32) -> &'static str {
    match (day % 10, day % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    }
}

fn prev_month(year: i32, month: u32) -> (i32, u32) {
    if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    }
}

fn next_month(year: i32, month: u32) -> (i32, u32) {
    if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    }
}

/// The calendar a component declares. The shown month is `defaultMonth`, else
/// the selection's month, else a `Month YYYY` label, else [`FALLBACK_MONTH`].
/// The selection is explicit only (`selected` / `value` / `from` + `to`): no
/// day is selected by default.
pub fn spec_of(comp: &ComponentNode) -> Spec {
    let date = |key: &str| {
        attr(comp, key)
            .or_else(|| item(comp, key))
            .and_then(parse_date)
    };
    let selected_raw = attr(comp, "selected").or_else(|| attr(comp, "value"));
    let range_mode = attr(comp, "mode").is_some_and(|m| m.trim().eq_ignore_ascii_case("range"))
        || attr(comp, "from").is_some();
    let range = if range_mode {
        let from = date("from").or_else(|| selected_raw.and_then(parse_range).map(|r| r.0));
        from.map(|from| {
            let to = date("to")
                .or_else(|| selected_raw.and_then(parse_range).and_then(|r| r.1))
                .filter(|to| *to >= from);
            (from, to)
        })
    } else {
        None
    };
    let selected = if range_mode {
        None
    } else {
        selected_raw.and_then(parse_day)
    };
    let anchor = selected.or(range.map(|r| r.0));
    let (year, month) = match date("defaultMonth").or(anchor) {
        Some((y, m, _)) => (y, m),
        None => label_month(comp).unwrap_or(FALLBACK_MONTH),
    };
    Spec {
        year,
        month,
        months: attr_num::<u32>(comp, "numberOfMonths")
            .filter(|n| (1..=12).contains(n))
            .unwrap_or(1),
        selected,
        range,
        today: date("today"),
        fixed_weeks: flag(comp, "fixedWeeks"),
    }
}

/// `Month YYYY` label → `(year, month)`.
fn label_month(comp: &ComponentNode) -> Option<(i32, u32)> {
    let label = label_of(comp);
    let mut words = label.split_whitespace();
    let (name, year) = (words.next()?, words.next()?);
    let idx = MONTHS.iter().position(|m| m.eq_ignore_ascii_case(name))?;
    Some((year.parse::<i32>().ok()?, idx as u32 + 1))
}

/// `YYYY-MM[-DD]` → `(year, month, day)`, day 1 when only a month is given.
fn parse_date(raw: &str) -> Option<Date> {
    parse_day(raw).or_else(|| {
        let mut parts = raw.trim().split('-');
        let y = parts.next()?.parse::<i32>().ok()?;
        let m = parts.next()?.parse::<u32>().ok()?;
        (parts.next().is_none() && (1..=12).contains(&m)).then_some((y, m, 1))
    })
}

/// `YYYY-MM-DD` with a valid day.
pub fn parse_day(raw: &str) -> Option<Date> {
    let mut parts = raw.trim().split('-');
    let y = parts.next()?.parse::<i32>().ok()?;
    let m = parts.next()?.parse::<u32>().ok()?;
    let d = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some() || !(1..=12).contains(&m) || !(1..=days_in_month(y, m)).contains(&d) {
        return None;
    }
    Some((y, m, d))
}

/// `YYYY-MM-DD..YYYY-MM-DD` (or `YYYY-MM-DD..`) → `(from, to)`.
fn parse_range(raw: &str) -> Option<(Date, Option<Date>)> {
    let (a, b) = raw.split_once("..")?;
    Some((parse_day(a)?, parse_day(b)))
}

pub fn month_name(month: u32) -> &'static str {
    MONTHS[(month.clamp(1, 12) - 1) as usize]
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

pub fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        2 if is_leap(y) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// 0 = Sunday (Sakamoto).
fn weekday(y: i32, m: u32, d: u32) -> u32 {
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if m < 3 { y - 1 } else { y };
    ((y + y / 4 - y / 100 + y / 400 + T[(m - 1) as usize] + d as i32).rem_euclid(7)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn reject_interact(html: &str) {
        assert!(!html.contains("<dialog"));
        assert!(!html.contains("showModal"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("<input"));
        assert!(!html.contains("type=\"date\""));
        assert!(!html.contains("grid-template-columns:repeat(7,1fr)"));
        // JS-only controls are native buttons, always disabled.
        assert_eq!(
            html.matches("<button ").count(),
            html.matches("<button type=\"button\" disabled ").count()
        );
    }

    fn visible_text(html: &str) -> String {
        let mut out = String::new();
        let mut in_tag = false;
        for ch in html.chars() {
            match ch {
                '<' => {
                    in_tag = true;
                    out.push(' ');
                }
                '>' => in_tag = false,
                c if !in_tag => out.push(c),
                _ => {}
            }
        }
        out.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn june_2026_matches_react_day_picker_text_and_weeks() {
        let html = render(&stub("calendar", "June 2026"));
        assert!(html.starts_with(&format!(
            "<div data-slot=\"calendar\"><div><div><nav aria-label=\"Navigation bar\"><button type=\"button\" disabled aria-label=\"Go to the Previous Month\">{CHEVRON_LEFT}</button><button type=\"button\" disabled aria-label=\"Go to the Next Month\">{CHEVRON_RIGHT}</button></nav><div><div><span role=\"status\" aria-live=\"polite\">June 2026</span></div><table role=\"grid\" aria-label=\"June 2026\"><thead aria-hidden=\"true\"><tr><th scope=\"col\" aria-label=\"Sunday\">Su</th>"
        )));
        assert_eq!(
            visible_text(&html),
            "June 2026 Su Mo Tu We Th Fr Sa 31 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 1 2 3 4"
        );
        assert_eq!(html.matches("<tr>").count(), 6);
        assert!(html.contains("<td role=\"gridcell\" data-outside=\"true\"><button type=\"button\" disabled aria-label=\"Sunday, May 31st, 2026\">31</button></td><td role=\"gridcell\"><button type=\"button\" disabled aria-label=\"Monday, June 1st, 2026\">1</button></td>"));
        assert!(
            html.contains("aria-label=\"Saturday, July 4th, 2026\">4</button></td></tr></tbody>")
        );
        assert!(!html.contains("aria-selected"));
        reject_interact(&html);
    }

    /// No fixture default leaks into apps: a month alone (label or
    /// `defaultMonth`) selects nothing.
    #[test]
    fn month_without_selected_selects_nothing() {
        assert!(!render(&stub("calendar", "June 2026")).contains("aria-selected"));
        let mut c = stub("calendar", "Due");
        c.props.insert("defaultMonth".into(), "2026-06-01".into());
        let html = render(&c);
        assert!(html.contains(">June 2026</span>"));
        assert!(!html.contains("aria-selected"), "{html}");
        assert!(!render(&stub("calendar", "March")).contains("aria-selected"));
    }

    /// Explicit `selected` (the React fixture's `selected` prop, emitted as an
    /// attr) marks that day; outside the shown month it marks nothing.
    #[test]
    fn selected_attr_marks_day() {
        let mut c = stub("calendar", "June 2026");
        c.props.insert("defaultMonth".into(), "2026-06-01".into());
        c.props.insert("selected".into(), "2026-06-01".into());
        let html = render(&c);
        assert!(html.contains("<td role=\"gridcell\" aria-selected=\"true\"><button type=\"button\" disabled aria-label=\"Monday, June 1st, 2026, selected\">1</button></td>"), "{html}");
        assert_eq!(html.matches("aria-selected").count(), 1);
        reject_interact(&html);

        let mut only = stub("calendar", "Due");
        only.props.insert("selected".into(), "2026-07-14".into());
        let html = render(&only);
        assert!(html.contains(">July 2026</span>"));
        assert!(html.contains("aria-label=\"Tuesday, July 14th, 2026, selected\">14</button>"));

        let mut elsewhere = stub("calendar", "June 2026");
        elsewhere
            .props
            .insert("defaultMonth".into(), "2026-06-01".into());
        elsewhere
            .props
            .insert("selected".into(), "2026-07-14".into());
        assert!(!render(&elsewhere).contains("aria-selected"));
    }

    #[test]
    fn value_selects_day_and_picks_month() {
        let mut c = stub("calendar", "Due");
        c.props.insert("value".into(), "2024-02-29".into());
        let html = render(&c);
        assert!(html.contains(">February 2024</span>"));
        assert!(html.contains("<td role=\"gridcell\" aria-selected=\"true\"><button type=\"button\" disabled aria-label=\"Thursday, February 29th, 2024, selected\">29</button></td>"));
        reject_interact(&html);
    }

    /// Docs "Single date": `selected` June 21 2026, `defaultMonth` June 2026,
    /// `fixedWeeks` (June 2026 already spans six rows).
    #[test]
    fn docs_single_date_example() {
        let mut c = stub("calendar", "Single date");
        c.props.insert("selected".into(), "2026-06-21".into());
        c.props.insert("defaultMonth".into(), "2026-06-01".into());
        c.props.insert("fixedWeeks".into(), "true".into());
        let html = render(&c);
        assert_eq!(html.matches("<tr>").count(), 7, "head + six weeks");
        assert!(html.contains("aria-label=\"Sunday, June 21st, 2026, selected\">21</button>"));
        reject_interact(&html);
    }

    /// `fixedWeeks` pads a five-row month to six rows of outside days.
    #[test]
    fn fixed_weeks_pads_to_six_rows() {
        let mut c = stub("calendar", "March 2026");
        assert_eq!(render(&c).matches("<tr>").count(), 6);
        c.props.insert("fixedWeeks".into(), "true".into());
        let html = render(&c);
        assert_eq!(html.matches("<tr>").count(), 7);
        assert!(html
            .contains("aria-label=\"Saturday, April 11th, 2026\">11</button></td></tr></tbody>"));
    }

    #[test]
    fn today_attr_marks_cell() {
        let mut c = stub("calendar", "June 2026");
        c.props.insert("today".into(), "2026-06-15".into());
        let html = render(&c);
        assert!(html.contains("<td role=\"gridcell\" data-today=\"true\"><button type=\"button\" disabled aria-label=\"Today, Monday, June 15th, 2026\">15</button></td>"), "{html}");
        assert_eq!(html.matches("data-today").count(), 1);
    }

    /// Range mode: start / middle / end cells are selected and classed like
    /// react-day-picker's modifiers; a `to` before `from` is dropped.
    #[test]
    fn range_mode_marks_start_middle_end() {
        let mut c = stub("calendar", "Stay");
        c.props.insert("mode".into(), "range".into());
        c.props.insert("from".into(), "2026-06-21".into());
        c.props.insert("to".into(), "2026-06-27".into());
        let html = render(&c);
        assert!(html.contains(">June 2026</span>"));
        assert!(html.contains("<td role=\"gridcell\" class=\"day-range-start\" aria-selected=\"true\"><button type=\"button\" disabled aria-label=\"Sunday, June 21st, 2026, selected\">21</button></td><td role=\"gridcell\" class=\"day-range-middle\" aria-selected=\"true\">"), "{html}");
        assert!(html.contains("<td role=\"gridcell\" class=\"day-range-end\" aria-selected=\"true\"><button type=\"button\" disabled aria-label=\"Saturday, June 27th, 2026, selected\">27</button></td>"));
        assert_eq!(html.matches("aria-selected=\"true\"").count(), 7);
        assert_eq!(html.matches("day-range-middle").count(), 5);
        reject_interact(&html);

        let mut one = stub("calendar", "Stay");
        one.props
            .insert("selected".into(), "2026-06-21..2026-06-21".into());
        one.props.insert("mode".into(), "range".into());
        let html = render(&one);
        assert!(html.contains("class=\"day-range-start day-range-end\" aria-selected=\"true\""));
        assert_eq!(html.matches("aria-selected=\"true\"").count(), 1);

        let mut backwards = stub("calendar", "Stay");
        backwards.props.insert("from".into(), "2026-06-21".into());
        backwards.props.insert("to".into(), "2026-06-01".into());
        assert_eq!(
            render(&backwards).matches("aria-selected=\"true\"").count(),
            1
        );
    }

    #[test]
    fn number_of_months_shows_consecutive_months_with_one_nav() {
        let mut c = stub("calendar", "Stay");
        c.props.insert("defaultMonth".into(), "2026-12-01".into());
        c.props.insert("numberOfMonths".into(), "2".into());
        let html = render(&c);
        assert_eq!(html.matches("<nav ").count(), 1);
        assert_eq!(html.matches("role=\"grid\"").count(), 2);
        assert!(html.contains(">December 2026</span>"));
        assert!(html.contains(">January 2027</span>"));
        reject_interact(&html);
    }

    #[test]
    fn unknown_label_falls_back_deterministically() {
        let html = render(&stub("calendar", "March"));
        assert!(html.contains(">September 2026</span>"));
        reject_interact(&html);
    }

    #[test]
    fn weekday_is_sakamoto() {
        assert_eq!(weekday(2026, 6, 1), 1);
        assert_eq!(weekday(2026, 9, 14), 1);
        assert_eq!(weekday(2024, 2, 29), 4);
    }

    #[test]
    fn dates_parse_strictly() {
        assert_eq!(parse_day("2026-02-29"), None);
        assert_eq!(parse_day("2024-02-29"), Some((2024, 2, 29)));
        assert_eq!(parse_date("2026-13"), None);
        assert_eq!(parse_date("2026-06"), Some((2026, 6, 1)));
        assert_eq!(
            parse_range("2026-06-01..2026-06-03"),
            Some(((2026, 6, 1), Some((2026, 6, 3))))
        );
        assert_eq!(parse_range("2026-06-01.."), Some(((2026, 6, 1), None)));
    }

    #[test]
    fn skips_interact_surf_grid() {
        let c = stub("calendar", "March");
        let html = render(&c);
        assert!(html.contains("data-slot=\"calendar\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("calendar", "March"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"calendar\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css
            .contains("[data-slot=\"calendar\"] {\n  display: block; color: var(--cronus-fg);\n}"));
        assert!(css.contains("[data-slot=\"calendar\"] > div { padding: 0.75rem; }"));
        assert!(css.contains("@media (min-width: 40rem) {\n  [data-slot=\"calendar\"] > div > div { flex-direction: row; }\n}"));
        assert!(
            css.contains("[data-slot=\"calendar\"] tbody tr { width: 100%; margin-top: 0.5rem; }")
        );
        assert!(css.contains("[data-slot=\"calendar\"] nav > button {"));
        assert!(css.contains("[data-slot=\"calendar\"] td > button {"));
        assert!(css.contains("[data-slot=\"calendar\"] td[aria-selected=\"true\"] { background: var(--cronus-surface-overlay); }"));
        assert!(css.contains("[data-slot=\"calendar\"] td.day-range-middle > button {"));
        assert!(css.contains("[data-slot=\"calendar\"] td[data-today=\"true\"] > button {"));
        assert!(css.contains("border-collapse: collapse"));
        assert!(css.contains("var(--cronus-fg-tertiary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains("showModal"));
    }
}
