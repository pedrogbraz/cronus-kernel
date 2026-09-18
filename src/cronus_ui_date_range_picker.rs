//! Dedicated DateRangePicker renderer. DOM matches React `DateRangePicker`:
//! the outline `Button` trigger `<button data-slot="date-range-picker-trigger">`
//! (calendar glyph + `<span>` label) and the popover
//! `<div data-slot="date-range-picker-content" role="dialog">` (`w-auto p-0`)
//! holding `<div>` (flex, column on narrow viewports) > optional
//! `<fieldset data-slot="date-range-picker-presets">` (sr-only legend + one
//! ghost `sm` `<button data-slot="date-range-picker-preset">` per `item`) +
//! `<fieldset data-slot="date-range-picker-calendar">` (sr-only legend + the
//! real `Calendar` in range mode, `cronus_ui_calendar::render_spec_named`).
//! Zero JS: the panel is a native `popover="auto"` anchored to the trigger.
//! Days are radios (`name` = `widget_id(comp, "day")`); presets are live
//! buttons (`data-range-preset` from the item label).
//!
//! Props: `from:"YYYY-MM-DD"` / `to:"YYYY-MM-DD"` (or `value:"from..to"`)
//! select the range and label the trigger with date-fns `LLL dd, y`
//! ("Jun 21, 2026 – Jun 27, 2026"; only `from` → "Jun 21, 2026");
//! `placeholder:` (default "Pick a date range", also the dialog name);
//! `numberOfMonths:1` (React default 2); `defaultMonth:"YYYY-MM"`;
//! `today:"YYYY-MM-DD"`; `disabled:true`; `aria-label:"…"`. Presets are
//! `item "Last 7 days"` lines. Not interact `date_range()`.

use crate::cronus_ui_calendar::{self, parse_day, Date, Spec, FALLBACK_MONTH};
use crate::cronus_ui_kit::{
    attr, attr_nonempty, attr_num, choice_texts, esc, flag, item, label_of, widget_id,
};
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
const SHORT_MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub fn render(comp: &ComponentNode) -> String {
    let placeholder = placeholder_of(comp);
    let range = range_of(comp);
    let label = match range {
        Some((from, Some(to))) => format!("{} – {}", format_lll(from), format_lll(to)),
        Some((from, None)) => format_lll(from),
        None => placeholder.clone(),
    };
    let trigger_id = widget_id(comp, "trigger");
    let pop_id = widget_id(comp, "range");
    let mut attrs = format!(
        "type=\"button\" id=\"{trigger_id}\" data-slot=\"date-range-picker-trigger\" data-variant=\"outline\" popovertarget=\"{pop_id}\" aria-haspopup=\"dialog\""
    );
    if range.is_none() {
        attrs.push_str(" data-empty=\"\"");
    }
    if flag(comp, "disabled") {
        attrs.push_str(" disabled");
    }
    if let Some(aria) = attr_nonempty(comp, "aria-label") {
        attrs.push_str(&format!(" aria-label=\"{}\"", esc(aria)));
    }
    let (year, month) = match month_of(comp).or(range.map(|r| r.0)) {
        Some((y, m, _)) => (y, m),
        None => FALLBACK_MONTH,
    };
    let calendar = cronus_ui_calendar::render_spec_named(
        &Spec {
            year,
            month,
            months: attr_num::<u32>(comp, "numberOfMonths")
                .filter(|n| (1..=12).contains(n))
                .unwrap_or(2),
            selected: None,
            range,
            today: attr(comp, "today").and_then(parse_day),
            fixed_weeks: false,
        },
        &widget_id(comp, "day"),
    );
    let presets = presets_html(comp);
    format!(
        "<button {attrs}>{ICON}<span>{label}</span></button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"date-range-picker-content\" role=\"dialog\" aria-label=\"{placeholder}\" anchor=\"{trigger_id}\"><div>{presets}<fieldset data-slot=\"date-range-picker-calendar\"><legend>{placeholder}</legend>{calendar}</fieldset></div></div>"
    )
}

/// date-fns `LLL dd, y` (en-US): "Jun 05, 2026".
fn format_lll((year, month, day): Date) -> String {
    format!("{} {day:02}, {year}", SHORT_MONTHS[(month - 1) as usize])
}

fn placeholder_of(comp: &ComponentNode) -> String {
    if let Some(v) = attr_nonempty(comp, "placeholder") {
        return esc(v);
    }
    if let Some(t) = item(comp, "placeholder").filter(|s| !s.is_empty()) {
        return esc(t);
    }
    let label = label_of(comp);
    if label.is_empty() || label == "date-range-picker" {
        "Pick a date range".into()
    } else {
        label
    }
}

fn range_of(comp: &ComponentNode) -> Option<(Date, Option<Date>)> {
    let raw = |name: &str| attr_nonempty(comp, name).or_else(|| item(comp, name));
    if let Some(from) = raw("from").and_then(parse_day) {
        let to = raw("to").and_then(parse_day).filter(|to| *to >= from);
        return Some((from, to));
    }
    let value = raw("value").or_else(|| raw("defaultValue"))?;
    let (a, b) = value.split_once("..")?;
    let from = parse_day(a)?;
    Some((from, parse_day(b).filter(|to| *to >= from)))
}

fn month_of(comp: &ComponentNode) -> Option<Date> {
    let raw = attr_nonempty(comp, "defaultMonth")?;
    parse_day(raw).or_else(|| {
        let (y, m) = raw.trim().split_once('-')?;
        let m = m.parse::<u32>().ok().filter(|m| (1..=12).contains(m))?;
        Some((y.parse().ok()?, m, 1))
    })
}

fn presets_html(comp: &ComponentNode) -> String {
    let presets = choice_texts(comp);
    if presets.is_empty() {
        return String::new();
    }
    let locked = flag(comp, "disabled");
    let buttons = presets
        .into_iter()
        .map(|t| {
            let key = range_preset_key(&t);
            let disabled = if locked { " disabled" } else { "" };
            format!(
                "<button type=\"button\"{disabled} data-slot=\"date-range-picker-preset\" data-variant=\"ghost\" data-range-preset=\"{key}\">{t}</button>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<fieldset data-slot=\"date-range-picker-presets\"><legend>Date range presets</legend>{buttons}</fieldset>"
    )
}

/// `Last 7 days` → `7d`, `This week` → `week`, `This month` → `month`.
fn range_preset_key(label: &str) -> String {
    let lower = label.to_ascii_lowercase();
    let num: String = lower.chars().filter(|c| c.is_ascii_digit()).collect();
    if lower.contains("day") && !num.is_empty() {
        return format!("{num}d");
    }
    if lower.contains("week") {
        return if num.is_empty() {
            "week".into()
        } else {
            format!("{num}w")
        };
    }
    if lower.contains("month") {
        return if num.is_empty() {
            "month".into()
        } else {
            format!("{num}m")
        };
    }
    let mut slug = String::new();
    for c in lower.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_end_matches('-').to_string()
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
        assert!(!html.contains("type=\"date\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("height:2.5rem;padding:0 0.75rem"));
        assert!(!html.contains("CTRL"));
    }

    #[test]
    fn root_is_trigger_and_popover_with_two_month_calendar() {
        let html = render(&stub("date-range-picker", "Stay"));
        assert!(html.starts_with(
            "<button type=\"button\" id=\"cui-date-range-picker-trigger\" data-slot=\"date-range-picker-trigger\" data-variant=\"outline\" popovertarget=\"cui-date-range-picker-range\" aria-haspopup=\"dialog\" data-empty=\"\">"
        ));
        assert!(html.contains("<span>Stay</span></button><div id=\"cui-date-range-picker-range\" popover=\"auto\" data-slot=\"date-range-picker-content\" role=\"dialog\" aria-label=\"Stay\" anchor=\"cui-date-range-picker-trigger\"><div><fieldset data-slot=\"date-range-picker-calendar\"><legend>Stay</legend><div data-slot=\"calendar\" data-month=\"2026-09\">"));
        assert!(!html.contains("date-range-picker-presets"));
        assert_eq!(
            html.matches("role=\"grid\"").count(),
            2,
            "React default numberOfMonths=2"
        );
        assert!(html.contains(">September 2026</span>"));
        assert!(html.contains(">October 2026</span>"));
        assert!(!html.contains("aria-selected"));
        assert!(html.contains("name=\"cui-date-range-picker-day\""));
        assert!(html.contains("type=\"radio\""));
        assert!(html.contains("data-cal-nav=\"prev\""));
        assert!(!html.contains("<button type=\"button\" disabled aria-label="));
        assert!(html.ends_with("</table></div></div></div></div></fieldset></div></div>"));
        reject_interact(&html);
    }

    /// Docs "Date range": June 21–27 2026, one month. The trigger reads
    /// date-fns `LLL dd, y` for both ends; the cells carry the range modifiers.
    #[test]
    fn from_to_label_and_range_days() {
        let mut c = stub("date-range-picker", "Stay");
        c.props.insert("from".into(), "2026-06-21".into());
        c.props.insert("to".into(), "2026-06-27".into());
        c.props.insert("numberOfMonths".into(), "1".into());
        c.props
            .insert("aria-label".into(), "Pick a date range".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Pick a date range\"><svg"));
        assert!(html.contains("<span>Jun 21, 2026 – Jun 27, 2026</span></button>"));
        assert!(!html.contains("data-empty"));
        assert_eq!(html.matches("role=\"grid\"").count(), 1);
        assert!(html.contains(">June 2026</span>"));
        assert!(html.contains("class=\"day-range-start\" aria-selected=\"true\""));
        assert!(html.contains("class=\"day-range-end\" aria-selected=\"true\""));
        assert_eq!(html.matches("day-range-middle").count(), 5);
        assert!(html.contains(
            "<input type=\"radio\" name=\"cui-date-range-picker-day\" value=\"2026-06-21\">"
        ));
        reject_interact(&html);

        let mut open = stub("date-range-picker", "Stay");
        open.props.insert("value".into(), "2026-06-05..".into());
        let html = render(&open);
        assert!(html.contains("<span>Jun 05, 2026</span></button>"));
        assert!(html.contains("class=\"day-range-start day-range-end\" aria-selected=\"true\""));
    }

    /// Docs "With presets": `item` lines become the ghost `sm` preset column
    /// beside the calendar (`data-range-preset` from the label).
    #[test]
    fn presets_from_items() {
        let mut c = stub("date-range-picker", "Stay");
        c.items.push(extra("item", "Last 7 days"));
        c.items.push(extra("item", "Last 30 days"));
        c.items.push(extra("item", "This week"));
        c.items.push(extra("item", "This month"));
        let html = render(&c);
        assert!(html.contains("<div><fieldset data-slot=\"date-range-picker-presets\"><legend>Date range presets</legend><button type=\"button\" data-slot=\"date-range-picker-preset\" data-variant=\"ghost\" data-range-preset=\"7d\">Last 7 days</button><button type=\"button\" data-slot=\"date-range-picker-preset\" data-variant=\"ghost\" data-range-preset=\"30d\">Last 30 days</button><button type=\"button\" data-slot=\"date-range-picker-preset\" data-variant=\"ghost\" data-range-preset=\"week\">This week</button><button type=\"button\" data-slot=\"date-range-picker-preset\" data-variant=\"ghost\" data-range-preset=\"month\">This month</button></fieldset><fieldset data-slot=\"date-range-picker-calendar\">"));
        assert!(!html.contains("date-range-picker-preset\" data-variant=\"ghost\">Stay"));
        assert!(!html
            .contains("data-slot=\"date-range-picker-preset\" data-variant=\"ghost\" disabled"));
        reject_interact(&html);
    }

    #[test]
    fn placeholder_default_month_and_disabled() {
        let mut c = stub("date-range-picker", "Stay");
        c.props.insert("disabled".into(), "true".into());
        c.props
            .insert("placeholder".into(), "Pick a reporting range".into());
        c.props.insert("defaultMonth".into(), "2026-03".into());
        let html = render(&c);
        assert!(html.contains(" disabled>"));
        assert!(html.contains("<span>Pick a reporting range</span>"));
        assert!(html.contains("aria-label=\"Pick a reporting range\" anchor="));
        assert!(html.contains("<legend>Pick a reporting range</legend>"));
        assert!(html.contains(">March 2026</span>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_native_date_pair() {
        let c = stub("date-range-picker", "Stay");
        let html = render(&c);
        assert!(!html.contains("type=\"date\""));
        assert!(html.contains("data-slot=\"date-range-picker-trigger\""));
        assert!(html.contains("type=\"radio\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("date-range-picker", "Stay"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"date-range-picker-trigger\"] {\n  display: inline-flex; align-items: center; justify-content: flex-start; gap: 0.5rem;\n  width: 18.75rem; height: 2.5rem; padding: 0 1rem; box-sizing: border-box;"));
        assert!(css.contains("[data-slot=\"date-range-picker-trigger\"][data-empty] { color: var(--cronus-fg-tertiary); }"));
        assert!(css.contains(
            "[data-slot=\"date-range-picker-content\"]:not(:popover-open) { display: none; }"
        ));
        assert!(css.contains("[data-slot=\"date-range-picker-content\"]:popover-open {"));
        assert!(css.contains("[data-slot=\"date-range-picker-content\"] > div {\n  display: flex; flex-direction: column;\n}"));
        assert!(css.contains("[data-slot=\"date-range-picker-calendar\"]"));
        assert!(css.contains("[data-slot=\"date-range-picker-preset\"] {"));
        assert!(!css.contains("[data-slot=\"date-range-picker-month\"]"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-shadow-lg"));
        assert!(css.contains("z-index: 50"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
