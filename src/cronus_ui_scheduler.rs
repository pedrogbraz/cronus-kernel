//! Dedicated Scheduler renderer. DOM matches React (`scheduler.tsx`, month
//! view, week starting Sunday): `<div data-slot="scheduler">` → header
//! (`<h2 data-slot="scheduler-title">` + Prev / Today / Next outline
//! `data-slot="button"`s) → `<table data-slot="scheduler-grid">` with
//! `<thead data-slot="scheduler-weekdays">` and one row per week; each day cell
//! holds its day number and up to three `scheduler-event` chips.
//!
//! The visible month is a `month:"YYYY-MM"` / `defaultMonth:"YYYY-MM-DD"` config
//! or parsed from the title (`label "June 2026"`). Events are the content texts
//! (`item "Team standup" date:"2026-06-02" color:info`); an event lands on its
//! `date:` config when given and its `color:` (React `SchedulerEventColor`:
//! `primary` (default), `success`, `warning`, `error`, `info`) becomes a
//! `c-<color>` class on the chip, since React writes the tint as classes.
//! Undated events follow the React audit harness placement
//! (`scheduler-fixture.tsx`: first event on the 15th, the rest on the 20th) —
//! the emitter cannot carry event dates, so both sides use one placement.
//! `today:"YYYY-MM-DD"` highlights a day (React: opt-in `today`); without it no
//! day is highlighted (no clock, no default). `weekStartsOn:1` rotates the grid
//! (0 = Sunday … 6 = Saturday); `aria-label:"…"` names the grid (React
//! `ariaLabel`, default "Event calendar").
//!
//! Zero JS divergences, all matching React's idle state: Prev/Today/Next and
//! the event chips are `disabled` buttons (month navigation and
//! `onEventClick` need a runtime). Not interact `calendar("scheduler")`.

use crate::cronus_ui_kit::{attr_nonempty, attr_num, esc, label_of};
use crate::parser::ComponentNode;

const NAME_KINDS: &[&str] = &["label", "title"];
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
const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const WEEKDAY_NAMES: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
const MAX_VISIBLE_EVENTS: usize = 3;
const EVENT_COLORS: [&str; 5] = ["primary", "success", "warning", "error", "info"];
/// Month shown when neither the title nor `month:` names one.
const FALLBACK_MONTH: (i64, u32) = (2026, 1);

const CHEVRON_LEFT: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m15 18-6-6 6-6\"/></svg>";
const CHEVRON_RIGHT: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m9 18 6-6-6-6\"/></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let (year, month) = visible_month(comp);
    let title = format!("{} {year}", MONTHS[(month - 1) as usize]);
    let events = events(comp, year, month);
    let today = attr_nonempty(comp, "today").and_then(parse_ymd);
    let week_start = attr_num::<i64>(comp, "weekStartsOn")
        .filter(|d| (0..=6).contains(d))
        .unwrap_or(0);
    let grid_label = esc(attr_nonempty(comp, "aria-label")
        .or_else(|| attr_nonempty(comp, "ariaLabel"))
        .unwrap_or("Event calendar"));

    let heads = (0..7)
        .map(|i| {
            format!(
                "<th scope=\"col\">{}</th>",
                WEEKDAYS[((week_start + i) % 7) as usize]
            )
        })
        .collect::<String>();

    let first = days_from_civil(year, month, 1);
    let grid_start = first - (weekday(first) - week_start).rem_euclid(7);
    let last = first + days_in_month(year, month) as i64 - 1;
    let grid_end = last + (week_start + 6 - weekday(last)).rem_euclid(7);
    let mut rows = String::new();
    let mut day = grid_start;
    while day <= grid_end {
        rows.push_str("<tr>");
        for _ in 0..7 {
            rows.push_str(&cell(day, year, month, today, &events));
            day += 1;
        }
        rows.push_str("</tr>");
    }

    format!(
        "<div data-slot=\"scheduler\"><div><h2 data-slot=\"scheduler-title\">{title}</h2><div><button type=\"button\" data-slot=\"button\" data-variant=\"outline\" data-size=\"icon-sm\" aria-label=\"Previous month\" disabled>{CHEVRON_LEFT}</button><button type=\"button\" data-slot=\"button\" data-variant=\"outline\" data-size=\"sm\" disabled>Today</button><button type=\"button\" data-slot=\"button\" data-variant=\"outline\" data-size=\"icon-sm\" aria-label=\"Next month\" disabled>{CHEVRON_RIGHT}</button></div></div><table data-slot=\"scheduler-grid\" aria-label=\"{grid_label}\"><thead data-slot=\"scheduler-weekdays\"><tr>{heads}</tr></thead><tbody>{rows}</tbody></table></div>"
    )
}

fn cell(day: i64, year: i64, month: u32, today: Option<i64>, events: &[Event]) -> String {
    let (y, m, d) = civil_from_days(day);
    let outside = if y == year && m == month {
        ""
    } else {
        " data-outside=\"true\""
    };
    let current = if today == Some(day) {
        " aria-current=\"date\""
    } else {
        ""
    };
    let label = format!(
        "{}, {} {d}, {y}",
        WEEKDAY_NAMES[weekday(day) as usize],
        MONTHS[(m - 1) as usize]
    );
    let day_events: Vec<&Event> = events.iter().filter(|e| e.day == day).collect();
    let chips = day_events
        .iter()
        .take(MAX_VISIBLE_EVENTS)
        .map(|e| {
            let class = if e.color == "primary" {
                String::new()
            } else {
                format!(" class=\"c-{}\"", e.color)
            };
            format!(
                "<button type=\"button\" data-slot=\"scheduler-event\"{class} title=\"{t}\" disabled>{t}</button>",
                t = e.title
            )
        })
        .collect::<String>();
    let overflow = day_events.len().saturating_sub(MAX_VISIBLE_EVENTS);
    let more = if overflow > 0 {
        format!("<span data-slot=\"scheduler-event-overflow\">+{overflow} more</span>")
    } else {
        String::new()
    };
    format!("<td aria-label=\"{label}\"{outside}{current}><span>{d}</span><span>{chips}{more}</span></td>")
}

fn visible_month(comp: &ComponentNode) -> (i64, u32) {
    for key in ["month", "defaultMonth"] {
        if let Some((y, m)) = attr_nonempty(comp, key).and_then(parse_ym) {
            return (y, m);
        }
    }
    let title = comp
        .items
        .iter()
        .find(|i| NAME_KINDS.contains(&i.item_type.as_str()) && !i.text.is_empty())
        .map(|i| i.text.clone())
        .unwrap_or_else(|| label_of(comp));
    parse_title(&title).unwrap_or(FALLBACK_MONTH)
}

/// `"June 2026"` → (2026, 6).
fn parse_title(title: &str) -> Option<(i64, u32)> {
    let mut parts = title.split_whitespace();
    let name = parts.next()?;
    let year = parts.next()?.parse::<i64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    let month = MONTHS.iter().position(|m| m.eq_ignore_ascii_case(name))? as u32 + 1;
    Some((year, month))
}

fn parse_ym(v: &str) -> Option<(i64, u32)> {
    let mut parts = v.split('-');
    let y = parts.next()?.parse::<i64>().ok()?;
    let m = parts
        .next()?
        .parse::<u32>()
        .ok()
        .filter(|m| (1..=12).contains(m))?;
    Some((y, m))
}

fn parse_ymd(v: &str) -> Option<i64> {
    let mut parts = v.split('-');
    let y = parts.next()?.parse::<i64>().ok()?;
    let m = parts
        .next()?
        .parse::<u32>()
        .ok()
        .filter(|m| (1..=12).contains(m))?;
    let d = parts
        .next()?
        .parse::<u32>()
        .ok()
        .filter(|d| *d >= 1 && *d <= days_in_month(y, m))?;
    Some(days_from_civil(y, m, d))
}

struct Event {
    day: i64,
    title: String,
    color: &'static str,
}

/// Content texts as dated, escaped, colour-keyed events. Undated events use
/// the harness days; an unknown `color:` is React's default `primary`.
fn events(comp: &ComponentNode, year: i64, month: u32) -> Vec<Event> {
    comp.items
        .iter()
        .filter(|i| !i.text.is_empty() && !NAME_KINDS.contains(&i.item_type.as_str()))
        .enumerate()
        .map(|(index, i)| {
            let day = i
                .config
                .get("date")
                .and_then(|v| parse_ymd(v))
                .unwrap_or_else(|| {
                    let d = if index == 0 { 15 } else { 20 };
                    days_from_civil(year, month, d.min(days_in_month(year, month)))
                });
            let color = i
                .config
                .get("color")
                .and_then(|c| {
                    EVENT_COLORS
                        .iter()
                        .find(|k| k.eq_ignore_ascii_case(c.trim()))
                })
                .copied()
                .unwrap_or("primary");
            Event {
                day,
                title: esc(&i.text),
                color,
            }
        })
        .collect()
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_in_month(y: i64, m: u32) -> u32 {
    match m {
        2 if is_leap(y) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Days since 1970-01-01 (proleptic Gregorian, Howard Hinnant's algorithm).
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m as i64 + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

/// 0 = Sunday … 6 = Saturday (1970-01-01 was a Thursday).
fn weekday(days: i64) -> i64 {
    (days + 4).rem_euclid(7)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("grid-template-columns:repeat(7,1fr)"));
        assert!(!html.contains("<input"));
        assert!(!html.contains("calendar("));
    }

    fn audit_source() -> crate::parser::ComponentNode {
        let mut c = stub("scheduler", "June 2026");
        c.items.push(extra("text", "Launch call"));
        c.items.push(extra("text", "Webinar"));
        c
    }

    #[test]
    fn civil_date_math_round_trips() {
        let d = days_from_civil(2026, 6, 1);
        assert_eq!(civil_from_days(d), (2026, 6, 1));
        assert_eq!(weekday(d), 1); // Monday
        assert_eq!(weekday(days_from_civil(2026, 5, 31)), 0);
        assert_eq!(civil_from_days(days_from_civil(2024, 2, 29)), (2024, 2, 29));
        assert_eq!(days_in_month(2024, 2), 29);
    }

    /// Emitted audit source: `label "June 2026"` + texts `Launch call`,
    /// `Webinar` → five Sunday-first weeks (May 31 … Jul 4), the title in the
    /// header, events on the 15th and 20th.
    #[test]
    fn audit_source_renders_june_2026_month_grid() {
        let html = audit_source_html();
        assert!(html.starts_with("<div data-slot=\"scheduler\"><div><h2 data-slot=\"scheduler-title\">June 2026</h2><div><button type=\"button\" data-slot=\"button\" data-variant=\"outline\" data-size=\"icon-sm\" aria-label=\"Previous month\" disabled>"));
        assert!(html.contains("<button type=\"button\" data-slot=\"button\" data-variant=\"outline\" data-size=\"sm\" disabled>Today</button>"));
        assert!(html.contains("<table data-slot=\"scheduler-grid\" aria-label=\"Event calendar\"><thead data-slot=\"scheduler-weekdays\"><tr><th scope=\"col\">Sun</th>"));
        assert_eq!(html.matches("<tr>").count(), 6);
        assert_eq!(html.matches("<td ").count(), 35);
        assert!(html.contains("<tbody><tr><td aria-label=\"Sunday, May 31, 2026\" data-outside=\"true\"><span>31</span><span></span></td><td aria-label=\"Monday, June 1, 2026\"><span>1</span><span></span></td>"));
        assert!(html.contains("<td aria-label=\"Monday, June 15, 2026\"><span>15</span><span><button type=\"button\" data-slot=\"scheduler-event\" title=\"Launch call\" disabled>Launch call</button></span></td>"));
        assert!(html.contains("<td aria-label=\"Saturday, June 20, 2026\"><span>20</span><span><button type=\"button\" data-slot=\"scheduler-event\" title=\"Webinar\" disabled>Webinar</button></span></td>"));
        assert!(html.ends_with("<td aria-label=\"Saturday, July 4, 2026\" data-outside=\"true\"><span>4</span><span></span></td></tr></tbody></table></div>"));
        // No fixture default leaks into apps: without `today`, no day is current.
        assert!(!html.contains("aria-current"));
        assert!(!html.contains(">June 2026</button>"));
        reject_interact(&html);
    }

    /// The audit fixture's explicit props (`defaultMonth`, `today`), emitted as
    /// component attrs, pick the month and the highlighted day.
    #[test]
    fn explicit_default_month_and_today_attrs() {
        let mut c = stub("scheduler", "Team");
        c.items.push(extra("text", "Launch call"));
        c.props.insert("defaultMonth".into(), "2026-06-01".into());
        c.props.insert("today".into(), "2026-06-15".into());
        let html = render(&c);
        assert!(html.contains("<h2 data-slot=\"scheduler-title\">June 2026</h2>"));
        assert!(html.contains(
            "<td aria-label=\"Monday, June 15, 2026\" aria-current=\"date\"><span>15</span>"
        ));
        assert_eq!(html.matches("aria-current").count(), 1);
        reject_interact(&html);
    }

    fn audit_source_html() -> String {
        render(&audit_source())
    }

    #[test]
    fn dated_events_and_today_config() {
        let mut c = stub("scheduler", "March 2026");
        c.items.push(extra("text", "Standup"));
        c.items[1].config.insert("date".into(), "2026-03-03".into());
        c.items[1]
            .config
            .insert("today".into(), "2026-03-10".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Tuesday, March 3, 2026\"><span>3</span><span><button type=\"button\" data-slot=\"scheduler-event\" title=\"Standup\" disabled>Standup</button>"));
        assert!(html.contains("aria-label=\"Tuesday, March 10, 2026\" aria-current=\"date\">"));
        reject_interact(&html);
    }

    /// Docs "Month view": token-coloured events on fixed dates, `today` June 1.
    #[test]
    fn colored_dated_events_and_grid_label() {
        let mut c = stub("scheduler", "Team");
        c.props.insert("month".into(), "2026-06".into());
        c.props.insert("today".into(), "2026-06-01".into());
        c.props
            .insert("aria-label".into(), "Product calendar".into());
        for (title, date, color) in [
            ("Team standup", "2026-06-02", "primary"),
            ("Design review", "2026-06-09", "info"),
            ("v2 ship", "2026-06-12", "success"),
            ("Sprint retro", "2026-06-12", "warning"),
            ("Launch party", "2026-06-24", "bogus"),
        ] {
            let mut item = extra("item", title);
            item.config.insert("date".into(), date.into());
            item.config.insert("color".into(), color.into());
            c.items.push(item);
        }
        let html = render(&c);
        assert!(
            html.contains("<table data-slot=\"scheduler-grid\" aria-label=\"Product calendar\">")
        );
        assert!(html.contains(
            "<td aria-label=\"Monday, June 1, 2026\" aria-current=\"date\"><span>1</span>"
        ));
        assert!(html.contains("<td aria-label=\"Tuesday, June 2, 2026\"><span>2</span><span><button type=\"button\" data-slot=\"scheduler-event\" title=\"Team standup\" disabled>Team standup</button></span></td>"));
        assert!(html.contains("<button type=\"button\" data-slot=\"scheduler-event\" class=\"c-info\" title=\"Design review\" disabled>Design review</button>"));
        assert!(html.contains("<td aria-label=\"Friday, June 12, 2026\"><span>12</span><span><button type=\"button\" data-slot=\"scheduler-event\" class=\"c-success\" title=\"v2 ship\" disabled>v2 ship</button><button type=\"button\" data-slot=\"scheduler-event\" class=\"c-warning\" title=\"Sprint retro\" disabled>Sprint retro</button></span></td>"));
        assert!(html.contains("data-slot=\"scheduler-event\" title=\"Launch party\" disabled>"));
        assert!(!html.contains("c-bogus"));
        reject_interact(&html);
    }

    /// `weekStartsOn:1` rotates the headers and pads the grid from Monday.
    #[test]
    fn week_starts_on_rotates_grid() {
        let mut c = stub("scheduler", "June 2026");
        c.props.insert("weekStartsOn".into(), "1".into());
        let html = render(&c);
        assert!(html.contains("<thead data-slot=\"scheduler-weekdays\"><tr><th scope=\"col\">Mon</th><th scope=\"col\">Tue</th>"));
        assert!(html.contains("<th scope=\"col\">Sun</th></tr></thead><tbody><tr><td aria-label=\"Monday, June 1, 2026\"><span>1</span>"));
        assert!(html.ends_with("<td aria-label=\"Sunday, July 5, 2026\" data-outside=\"true\"><span>5</span><span></span></td></tr></tbody></table></div>"));
        assert_eq!(html.matches("<tr>").count(), 6);
    }

    #[test]
    fn more_than_three_events_collapse_into_overflow() {
        let mut c = stub("scheduler", "June 2026");
        for t in ["A", "B", "C", "D", "E"] {
            c.items.push(extra("text", t));
        }
        let html = render(&c);
        assert!(html.contains("<span data-slot=\"scheduler-event-overflow\">+1 more</span>"));
        assert_eq!(html.matches("data-slot=\"scheduler-event\"").count(), 4);
    }

    #[test]
    fn unparseable_title_falls_back_to_a_fixed_month() {
        let html = render(&stub("scheduler", "Team"));
        assert!(html.contains("<h2 data-slot=\"scheduler-title\">January 2026</h2>"));
        assert!(html.contains("aria-label=\"Thursday, January 1, 2026\""));
    }

    #[test]
    fn skips_interact_calendar_surf() {
        let c = audit_source();
        let html = render(&c);
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = audit_source_html();
            reject_interact(&html);
        });
    }

    /// Geometry parity (Wave 1t): bordered `surface-base` card, 57px header,
    /// `text-xs` weekday heads, 96px cells, 20px `rounded` event chips, and
    /// header buttons pinned to React's Button line boxes.
    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"scheduler\"] {\n  display: block; width: 100%;\n  border-radius: var(--cronus-radius-xl); border: 1px solid var(--cronus-border);\n  background: var(--cronus-surface-base); color: var(--cronus-fg);\n}"));
        assert!(css.contains("[data-slot=\"scheduler-title\"] { font-size: 0.875rem; line-height: 1.25rem; font-weight: 600; color: var(--cronus-fg); }"));
        // `html[data-cronus-theme] h2` (0,1,2) forces weight 400 and -0.025em.
        assert!(css.contains("[data-slot=\"scheduler\"] > div:first-child > [data-slot=\"scheduler-title\"] {\n  font-weight: 600; letter-spacing: normal; font-variant-numeric: tabular-nums;\n}"));
        assert!(css.contains("[data-slot=\"scheduler-grid\"] td {\n  height: 6rem; min-width: 0; padding: 0.375rem; vertical-align: top;\n  border-inline-end: 1px solid var(--cronus-border); border-bottom: 1px solid var(--cronus-border);"));
        assert!(css.contains("[data-slot=\"scheduler-event\"] {\n  display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;\n  border: 0; border-radius: 0.25rem; padding: 0.125rem 0.375rem; text-align: start;\n  font-size: 0.75rem; line-height: 1rem; font-weight: 500;"));
        assert!(css.contains("[data-slot=\"scheduler-event\"].c-success {\n  background: color-mix(in oklab, var(--cronus-success) 15%, transparent); color: var(--cronus-success-text);\n}"));
        assert!(css.contains("[data-slot=\"scheduler-event\"].c-info {"));
        assert!(css.contains(
            "[data-slot=\"scheduler-grid\"] td:hover { background: var(--cronus-surface-overlay); }"
        ));
        assert!(css.contains("[data-slot=\"scheduler-grid\"] td[aria-current=\"date\"] > span:first-child {\n  background: var(--cronus-primary); color: #fff;\n}"));
        assert!(css.contains("var(--cronus-fg-tertiary)"));
        assert!(!css.contains("zinc-"));
    }
}
