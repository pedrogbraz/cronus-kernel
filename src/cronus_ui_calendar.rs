//! Dedicated Calendar renderer. DOM mirrors the React audit fixture
//! (`<div data-slot="calendar">` wrapping react-day-picker): root `<div>` (p-3) >
//! months `<div>` > `<nav>` (previous / next month buttons) + month `<div>` >
//! caption `<div><span role="status">` + `<table role="grid">` with a Su–Sa
//! `<thead>` and one flex `<tr>` per week of day `<button>`s.
//! Real month layout: leading/trailing outside days (`showOutsideDays`) and only
//! as many weeks as the month needs.
//! Zero JS: month navigation and day selection need JS, so every `<button>` is
//! rendered `disabled` with React's idle look (only the nav buttons are dimmed,
//! as in React). Month comes from `defaultMonth` / `value` (`YYYY-MM[-DD]`) or a
//! `Month YYYY` label. Not interact `calendar()` SURF grid / date input.

use crate::cronus_ui_kit::{item, label_of};
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

pub fn render(comp: &ComponentNode) -> String {
    let (year, month, selected) = month_of(comp);
    let caption = format!("{} {year}", MONTHS[(month - 1) as usize]);
    let head = WEEKDAYS
        .iter()
        .map(|(short, long)| format!("<th scope=\"col\" aria-label=\"{long}\">{short}</th>"))
        .collect::<String>();
    let (py, pm) = if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    };
    let (ny, nm) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let lead = weekday(year, month, 1);
    let prev_days = days_in_month(py, pm);
    let mut cells: Vec<String> = Vec::new();
    for i in 0..lead {
        cells.push(cell(py, pm, prev_days - lead + 1 + i, true, false));
    }
    for d in 1..=days_in_month(year, month) {
        cells.push(cell(year, month, d, false, selected == Some(d)));
    }
    let mut next = 1;
    while cells.len() % 7 != 0 {
        cells.push(cell(ny, nm, next, true, false));
        next += 1;
    }
    let rows = cells
        .chunks(7)
        .map(|week| format!("<tr>{}</tr>", week.concat()))
        .collect::<String>();
    let nav = format!(
        "<nav aria-label=\"Navigation bar\"><button type=\"button\" disabled aria-label=\"Go to the Previous Month\">{CHEVRON_LEFT}</button><button type=\"button\" disabled aria-label=\"Go to the Next Month\">{CHEVRON_RIGHT}</button></nav>"
    );
    format!(
        "<div data-slot=\"calendar\"><div><div>{nav}<div><div><span role=\"status\" aria-live=\"polite\">{caption}</span></div><table role=\"grid\" aria-label=\"{caption}\"><thead aria-hidden=\"true\"><tr>{head}</tr></thead><tbody>{rows}</tbody></table></div></div></div></div>"
    )
}

fn cell(year: i32, month: u32, day: u32, outside: bool, selected: bool) -> String {
    let mut attrs = String::from(" role=\"gridcell\"");
    if outside {
        attrs.push_str(" data-outside=\"true\"");
    }
    if selected {
        attrs.push_str(" aria-selected=\"true\"");
    }
    let label = format!(
        "{}, {} {day}{}, {year}{}",
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

/// `(year, month, selected day)`. Falls back to September 2026 (no clock: the
/// render stays deterministic).
fn month_of(comp: &ComponentNode) -> (i32, u32, Option<u32>) {
    for key in ["defaultMonth", "value"] {
        let raw = comp
            .props
            .get(key)
            .map(String::as_str)
            .or_else(|| {
                comp.items
                    .iter()
                    .find_map(|i| i.config.get(key).map(String::as_str))
            })
            .or_else(|| item(comp, key));
        if let Some((y, m, d)) = raw.and_then(parse_iso) {
            let selected = if key == "value" { d } else { None };
            return (y, m, selected);
        }
    }
    let label = label_of(comp);
    let mut words = label.split_whitespace();
    if let (Some(name), Some(year)) = (words.next(), words.next()) {
        if let (Some(idx), Ok(y)) = (
            MONTHS.iter().position(|m| m.eq_ignore_ascii_case(name)),
            year.parse::<i32>(),
        ) {
            return (y, idx as u32 + 1, None);
        }
    }
    (2026, 9, None)
}

fn parse_iso(raw: &str) -> Option<(i32, u32, Option<u32>)> {
    let mut parts = raw.split('-');
    let y = parts.next()?.parse::<i32>().ok()?;
    let m = parts.next()?.parse::<u32>().ok()?;
    if !(1..=12).contains(&m) {
        return None;
    }
    let d = parts.next().and_then(|d| d.parse::<u32>().ok());
    Some((y, m, d.filter(|d| (1..=days_in_month(y, m)).contains(d))))
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_in_month(y: i32, m: u32) -> u32 {
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

    #[test]
    fn value_selects_day_and_picks_month() {
        let mut c = stub("calendar", "Due");
        c.props.insert("value".into(), "2024-02-29".into());
        let html = render(&c);
        assert!(html.contains(">February 2024</span>"));
        assert!(html.contains("<td role=\"gridcell\" aria-selected=\"true\"><button type=\"button\" disabled aria-label=\"Thursday, February 29th, 2024, selected\">29</button></td>"));
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
    fn skips_interact_surf_grid() {
        let c = stub("calendar", "March");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("calendar", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("data-slot=\"calendar\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("grid-template-columns:repeat(7,1fr)"));
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
        assert!(css.contains("border-collapse: collapse"));
        assert!(css.contains("var(--cronus-fg-tertiary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains("showModal"));
    }
}
