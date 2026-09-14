//! Dedicated DateRangePicker renderer. Always-open static DOM (no JS).
//! `<button data-slot="date-range-picker-trigger">` plus open
//! `<div data-slot="date-range-picker-content">`. Not interact `date_range()`
//! (two native `type="date"` inputs with CTRL styles).

use crate::cronus_ui_kit::{choice_texts, esc, item, label_of, widget_id};
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
    let placeholder = placeholder_of(comp);
    let label = trigger_label(comp, &placeholder);
    let trigger_id = widget_id(comp, "trigger");
    let pop_id = widget_id(comp, "range");
    let mut attrs = format!(
        "type=\"button\" id=\"{trigger_id}\" data-slot=\"date-range-picker-trigger\" data-variant=\"outline\" popovertarget=\"{pop_id}\" aria-haspopup=\"dialog\""
    );
    if label == placeholder {
        attrs.push_str(" data-empty=\"\"");
    }
    if flag(comp, "disabled") {
        attrs.push_str(" disabled");
    }
    if let Some(aria) = attr(comp, "aria-label").filter(|s| !s.is_empty()) {
        attrs.push_str(&format!(" aria-label=\"{}\"", esc(aria)));
    }
    let presets = presets_html(comp);
    let (from, to) = range_of(comp);
    let calendar = calendar_html(&placeholder, from, to);
    format!(
        "<button {attrs}>{ICON}<span>{label}</span></button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"date-range-picker-content\" role=\"dialog\" aria-label=\"{placeholder}\" anchor=\"{trigger_id}\">{presets}{calendar}</div>"
    )
}

fn trigger_label(comp: &ComponentNode, placeholder: &str) -> String {
    if let Some(v) = attr(comp, "value").or_else(|| item(comp, "value")) {
        if !v.is_empty() {
            return esc(v);
        }
    }
    let (from, to) = range_of(comp);
    match (from_raw(comp, "from"), from_raw(comp, "to"), from, to) {
        (Some(f), Some(t), _, _) if !f.is_empty() && !t.is_empty() => {
            format!("{} – {}", esc(f), esc(t))
        }
        (Some(f), None, _, _) if !f.is_empty() => esc(f),
        (_, _, Some(f), Some(t)) => format!("{f:02} – {t:02}"),
        (_, _, Some(f), None) => format!("{f:02}"),
        _ => placeholder.to_string(),
    }
}

fn placeholder_of(comp: &ComponentNode) -> String {
    if let Some(v) = attr(comp, "placeholder").filter(|s| !s.is_empty()) {
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

fn from_raw<'a>(comp: &'a ComponentNode, name: &str) -> Option<&'a str> {
    attr(comp, name).or_else(|| item(comp, name))
}

fn range_of(comp: &ComponentNode) -> (Option<u8>, Option<u8>) {
    (
        parse_day(from_raw(comp, "from")),
        parse_day(from_raw(comp, "to")),
    )
}

fn parse_day(raw: Option<&str>) -> Option<u8> {
    let raw = raw?;
    let day = raw.rsplit('-').next()?.parse::<u8>().ok()?;
    if (1..=28).contains(&day) {
        Some(day)
    } else {
        None
    }
}

fn presets_html(comp: &ComponentNode) -> String {
    let presets = choice_texts(comp);
    if presets.is_empty() {
        return String::new();
    }
    let buttons = presets
        .into_iter()
        .map(|t| {
            format!(
                "<button type=\"button\" disabled data-slot=\"date-range-picker-preset\">{t}</button>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<fieldset data-slot=\"date-range-picker-presets\"><legend>Date range presets</legend>{buttons}</fieldset>"
    )
}

fn calendar_html(placeholder: &str, from: Option<u8>, to: Option<u8>) -> String {
    let mut out = format!(
        "<fieldset data-slot=\"date-range-picker-calendar\"><legend>{placeholder}</legend>"
    );
    out.push_str(&month_grid(from, to));
    out.push_str(&month_grid(None, None));
    out.push_str("</fieldset>");
    out
}

fn month_grid(from: Option<u8>, to: Option<u8>) -> String {
    let mut out = String::from("<div data-slot=\"date-range-picker-month\" role=\"grid\">");
    out.push_str("<div role=\"row\">");
    for d in WEEKDAYS {
        out.push_str(&format!(
            "<span data-slot=\"date-range-picker-weekday\" role=\"columnheader\">{d}</span>"
        ));
    }
    out.push_str("</div>");
    for week in 0..4 {
        out.push_str("<div role=\"row\">");
        for offset in 1..=7 {
            let day = week * 7 + offset;
            let day_u8 = day as u8;
            let (aria, range) = day_state(day_u8, from, to);
            out.push_str(&format!(
                "<button type=\"button\" disabled data-slot=\"date-range-picker-day\" role=\"gridcell\" aria-selected=\"{aria}\"{range}>{day}</button>"
            ));
        }
        out.push_str("</div>");
    }
    out.push_str("</div>");
    out
}

fn day_state(day: u8, from: Option<u8>, to: Option<u8>) -> (&'static str, &'static str) {
    match (from, to) {
        (Some(f), Some(t)) if day == f && day == t => ("true", " data-range=\"start\""),
        (Some(f), Some(_)) if day == f => ("true", " data-range=\"start\""),
        (Some(_), Some(t)) if day == t => ("true", " data-range=\"end\""),
        (Some(f), Some(t)) if day > f && day < t => ("true", " data-range=\"middle\""),
        (Some(f), None) if day == f => ("true", " data-range=\"start\""),
        _ => ("false", ""),
    }
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
        assert!(!html.contains("<input"));
        assert!(!html.contains("<label"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("height:2.5rem;padding:0 0.75rem"));
        assert!(!html.contains("CTRL"));
    }

    #[test]
    fn root_is_trigger_and_open_content_not_two_date_inputs() {
        let html = render(&stub("date-range-picker", "Stay"));
        assert!(html.starts_with(
            "<button type=\"button\" id=\"cui-date-range-picker-trigger\" data-slot=\"date-range-picker-trigger\" data-variant=\"outline\" popovertarget=\"cui-date-range-picker-range\" aria-haspopup=\"dialog\" data-empty=\"\">"
        ));
        assert!(html.contains("<span>Stay</span></button><div id=\"cui-date-range-picker-range\" popover=\"auto\" data-slot=\"date-range-picker-content\" role=\"dialog\" aria-label=\"Stay\" anchor=\"cui-date-range-picker-trigger\">"));
        assert!(html.contains("data-slot=\"date-range-picker-calendar\""));
        assert!(html.contains("data-slot=\"date-range-picker-day\""));
        assert_eq!(
            html.matches("data-slot=\"date-range-picker-month\"")
                .count(),
            2
        );
        assert!(!html.contains("type=\"date\""));
        assert_eq!(html.matches("<input").count(), 0);
        reject_interact(&html);
    }

    #[test]
    fn from_to_label_and_range_days() {
        let mut c = stub("date-range-picker", "Stay");
        c.props.insert("from".into(), "2026-09-01".into());
        c.props.insert("to".into(), "2026-09-13".into());
        let html = render(&c);
        assert!(html.contains("<span>2026-09-01 – 2026-09-13</span></button>"));
        assert!(!html.contains("data-empty"));
        assert!(html.contains("data-range=\"start\">1</button>"));
        assert!(html.contains("data-range=\"end\">13</button>"));
        assert!(html.contains("data-range=\"middle\">7</button>"));
        reject_interact(&html);
    }

    #[test]
    fn presets_from_items() {
        let mut c = stub("date-range-picker", "Stay");
        c.items.push(extra("item", "Last 7 days"));
        c.items.push(extra("item", "This month"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"date-range-picker-presets\""));
        assert!(html.contains(
            "<button type=\"button\" disabled data-slot=\"date-range-picker-preset\">Last 7 days</button>"
        ));
        assert!(html.contains(
            "<button type=\"button\" disabled data-slot=\"date-range-picker-preset\">This month</button>"
        ));
        assert!(!html.contains("date-range-picker-preset\">Stay"));
        reject_interact(&html);
    }

    #[test]
    fn disabled_trigger() {
        let mut c = stub("date-range-picker", "Stay");
        c.props.insert("disabled".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(" disabled>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_native_date_pair() {
        let c = stub("date-range-picker", "Stay");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("date-range-picker", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<label data-slot=\"date-range-picker\""));
        assert!(interact.contains("<input type=\"date\""));
        assert_eq!(interact.matches("<input type=\"date\"").count(), 2);
        assert!(interact.contains("style="));
        assert!(interact.contains("height:2.5rem;padding:0 0.75rem"));
        assert!(!html.contains("<input"));
        assert!(!html.contains("type=\"date\""));
        assert!(html.contains("data-slot=\"date-range-picker-trigger\""));
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
        assert!(css.contains("[data-slot=\"date-range-picker-content\"]:not(:popover-open) { display: none; }"));
        assert!(css.contains("[data-slot=\"date-range-picker-content\"]:popover-open {"));
        assert!(css.contains("[data-slot=\"date-range-picker-calendar\"]"));
        assert!(css.contains("[data-slot=\"date-range-picker-preset\"]"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-shadow-lg"));
        assert!(css.contains("z-index: 50"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
