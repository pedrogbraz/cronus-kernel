//! Dedicated TimePicker renderer. DOM mirrors React closed state: the root
//! `data-slot="time-picker"` IS the outline trigger `<button>`. The panel is a
//! native `popover` sibling (`time-picker-content`), hidden until opened by
//! `popovertarget` — no JS. Not interact `input("time-picker", "time")`.

use crate::cronus_ui_kit::{esc, item, label_of};
use crate::parser::ComponentNode;

const ICON: &str = concat!(
    "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" ",
    "stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" focusable=\"false\">",
    "<circle cx=\"12\" cy=\"12\" r=\"10\" />",
    "<polyline points=\"12 6 12 12 16 14\" />",
    "</svg>",
);

pub fn render(comp: &ComponentNode) -> String {
    let hour_cycle = hour_cycle_of(comp);
    let show_seconds = flag(comp, "showSeconds") || flag(comp, "show-seconds");
    let time = parse_time(comp);
    let label = trigger_label(comp, time, hour_cycle, show_seconds);
    let trigger_id = crate::cronus_ui_kit::widget_id(comp, "trigger");
    let pop_id = crate::cronus_ui_kit::widget_id(comp, "panel");
    let mut btn = format!(
        "type=\"button\" id=\"{trigger_id}\" data-slot=\"time-picker\" data-variant=\"outline\" aria-haspopup=\"dialog\" aria-expanded=\"false\" data-state=\"closed\" popovertarget=\"{pop_id}\""
    );
    if flag(comp, "disabled") {
        btn.push_str(" disabled");
    }
    if let Some(aria) = attr(comp, "aria-label").filter(|s| !s.is_empty()) {
        let shown = if time.is_some() {
            format!("{}: {label}", esc(aria))
        } else {
            esc(aria)
        };
        btn.push_str(&format!(" aria-label=\"{shown}\""));
    }
    let content_label = attr(comp, "aria-label")
        .filter(|s| !s.is_empty())
        .map(esc)
        .unwrap_or_else(|| "Choose a time".into());
    let columns = columns_html(time, hour_cycle, show_seconds);
    format!(
        "<button {btn}>{ICON}<span>{label}</span></button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"time-picker-content\" aria-label=\"{content_label}\" anchor=\"{trigger_id}\">{columns}<div data-slot=\"time-picker-footer\"><button type=\"button\" data-slot=\"time-picker-now\">Now</button><button type=\"button\" data-slot=\"time-picker-done\">Done</button></div></div>"
    )
}

fn trigger_label(
    comp: &ComponentNode,
    time: Option<TimeValue>,
    hour_cycle: u8,
    show_seconds: bool,
) -> String {
    if let Some(t) = time {
        return format_time(t, hour_cycle, show_seconds);
    }
    if let Some(v) = attr(comp, "placeholder").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    let label = label_of(comp);
    if label.is_empty() {
        "Select time".into()
    } else {
        label
    }
}

#[derive(Clone, Copy)]
struct TimeValue {
    hours: u8,
    minutes: u8,
    seconds: u8,
}

fn parse_time(comp: &ComponentNode) -> Option<TimeValue> {
    let raw = attr(comp, "value").or_else(|| item(comp, "value"))?;
    parse_time_str(raw)
}

fn parse_time_str(raw: &str) -> Option<TimeValue> {
    let s = raw.trim();
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return None;
    }
    let hours = parts[0].parse::<u8>().ok()?;
    let minutes = parts[1].split_whitespace().next()?.parse::<u8>().ok()?;
    let seconds = if parts.len() == 3 {
        parts[2]
            .split_whitespace()
            .next()?
            .parse::<u8>()
            .unwrap_or(0)
    } else {
        0
    };
    if hours > 23 || minutes > 59 || seconds > 59 {
        return None;
    }
    Some(TimeValue {
        hours,
        minutes,
        seconds,
    })
}

fn format_time(time: TimeValue, hour_cycle: u8, show_seconds: bool) -> String {
    let second_part = if show_seconds {
        format!(":{}", pad2(time.seconds))
    } else {
        String::new()
    };
    if hour_cycle == 24 {
        return format!("{}:{}{second_part}", pad2(time.hours), pad2(time.minutes));
    }
    let period = if time.hours < 12 { "AM" } else { "PM" };
    let hour12 = if time.hours % 12 == 0 {
        12
    } else {
        time.hours % 12
    };
    format!(
        "{}:{}{second_part} {period}",
        pad2(hour12),
        pad2(time.minutes)
    )
}

fn pad2(n: u8) -> String {
    format!("{n:02}")
}

fn hour_cycle_of(comp: &ComponentNode) -> u8 {
    if let Some(v) = attr(comp, "hourCycle").or_else(|| attr(comp, "hour-cycle")) {
        if v == "24" {
            return 24;
        }
    }
    if comp
        .style
        .as_deref()
        .unwrap_or("")
        .split('+')
        .any(|part| part.trim() == "24")
    {
        return 24;
    }
    12
}

fn columns_html(time: Option<TimeValue>, hour_cycle: u8, show_seconds: bool) -> String {
    let mut out = String::from("<div data-slot=\"time-picker-columns\">");
    if hour_cycle == 24 {
        out.push_str(&column(
            "Hour",
            "Hr",
            &(0..24).collect::<Vec<_>>(),
            time.map(|t| t.hours),
        ));
    } else {
        let hours = [12u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let selected = time.map(|t| if t.hours % 12 == 0 { 12 } else { t.hours % 12 });
        out.push_str(&column("Hour", "Hr", &hours, selected));
    }
    let minutes = [0u8, 15, 30, 45];
    out.push_str(&column("Minute", "Min", &minutes, time.map(|t| t.minutes)));
    if show_seconds {
        out.push_str(&column("Second", "Sec", &minutes, time.map(|t| t.seconds)));
    }
    if hour_cycle == 12 {
        let selected_period = time.map(|t| if t.hours < 12 { "AM" } else { "PM" });
        out.push_str(&period_column(selected_period));
    }
    out.push_str("</div>");
    out
}

fn column(label: &str, header: &str, values: &[u8], selected: Option<u8>) -> String {
    let opts = values
        .iter()
        .map(|v| {
            let aria = if selected == Some(*v) {
                "true"
            } else {
                "false"
            };
            format!(
                "<div data-slot=\"time-picker-option\" role=\"option\" aria-selected=\"{aria}\">{}</div>",
                pad2(*v)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"time-picker-column\" role=\"listbox\" aria-label=\"{label}\"><span>{header}</span>{opts}</div>"
    )
}

fn period_column(selected: Option<&str>) -> String {
    let opts = ["AM", "PM"]
        .iter()
        .map(|p| {
            let aria = if selected == Some(*p) {
                "true"
            } else {
                "false"
            };
            format!(
                "<div data-slot=\"time-picker-option\" role=\"option\" aria-selected=\"{aria}\">{p}</div>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"time-picker-column\" role=\"listbox\" aria-label=\"AM or PM\"><span>AM/PM</span>{opts}</div>"
    )
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

    fn reject_interact(html: &str) {
        assert!(!html.contains("type=\"time\""));
        assert!(!html.contains("time-picker-control"));
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
    fn root_is_trigger_button_with_native_popover_panel() {
        let html = render(&stub("time-picker", "Time"));
        assert!(html.starts_with(
            "<button type=\"button\" id=\"cui-time-picker-trigger\" data-slot=\"time-picker\" data-variant=\"outline\" aria-haspopup=\"dialog\" aria-expanded=\"false\" data-state=\"closed\" popovertarget=\"cui-time-picker-panel\">"
        ));
        assert_eq!(html.matches("data-slot=\"time-picker\"").count(), 1);
        assert!(html.contains(
            "<span>Time</span></button><div id=\"cui-time-picker-panel\" popover=\"auto\" data-slot=\"time-picker-content\""
        ));
        assert!(html.contains("data-slot=\"time-picker-column\""));
        assert!(html.contains("data-slot=\"time-picker-option\""));
        assert!(html.contains("role=\"listbox\""));
        assert!(html.contains("aria-label=\"Hour\""));
        assert!(html.contains("aria-label=\"Minute\""));
        assert!(html.contains("aria-label=\"AM or PM\""));
        assert!(html.contains("data-slot=\"time-picker-now\">Now</button>"));
        assert!(html.contains("data-slot=\"time-picker-done\">Done</button>"));
        assert!(!html.contains("data-slot=\"time-picker-trigger\""));
        reject_interact(&html);
    }

    /// Audit fixture: emitter writes `label`, `text`, then `value:` and
    /// `aria-label:` which the parser attaches to the text item. React root
    /// `[data-slot="time-picker"]` is the BUTTON labelled "Meeting time: …".
    #[test]
    fn fixture_root_is_button_with_value_aria_label() {
        let mut c = stub("time-picker", "Select time");
        let mut text = ComponentItemNode {
            item_type: "text".into(),
            text: "Select time".into(),
            link: None,
            tone: None,
            config: Default::default(),
        };
        text.config.insert("value".into(), "09:30".into());
        text.config.insert("aria-label".into(), "Meeting time".into());
        c.items.push(text);
        let html = render(&c);
        assert!(html.starts_with("<button type=\"button\""));
        assert!(!html.starts_with("<div"));
        assert!(html.contains(" aria-label=\"Meeting time: 09:30 AM\">"));
        assert!(html.contains("<span>09:30 AM</span></button>"));
        reject_interact(&html);
    }

    #[test]
    fn value_formats_12h_and_selects_options() {
        let mut c = stub("time-picker", "Time");
        c.props.insert("value".into(), "14:30".into());
        let html = render(&c);
        assert!(html.contains("<span>02:30 PM</span></button>"));
        assert!(html.contains("aria-selected=\"true\">02</div>"));
        assert!(html.contains("aria-selected=\"true\">30</div>"));
        assert!(html.contains("aria-selected=\"true\">PM</div>"));
        reject_interact(&html);
    }

    #[test]
    fn hour_cycle_24_skips_period_column() {
        let mut c = stub("time-picker", "Time");
        c.style = Some("time-picker+24".into());
        c.props.insert("value".into(), "14:30".into());
        let html = render(&c);
        assert!(html.contains("<span>14:30</span></button>"));
        assert!(html.contains("aria-selected=\"true\">14</div>"));
        assert!(!html.contains("AM or PM"));
        assert!(!html.contains(">AM</div>"));
        reject_interact(&html);
    }

    #[test]
    fn show_seconds_adds_column() {
        let mut c = stub("time-picker", "Time");
        c.props.insert("showSeconds".into(), "true".into());
        c.props.insert("value".into(), "09:15:45".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Second\""));
        assert!(html.contains("<span>09:15:45 AM</span></button>"));
        assert!(html.contains("aria-selected=\"true\">45</div>"));
        reject_interact(&html);
    }

    #[test]
    fn disabled_trigger() {
        let mut c = stub("time-picker", "Time");
        c.props.insert("disabled".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(" disabled>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_native_time_input() {
        let c = stub("time-picker", "Time");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("time-picker", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<label data-slot=\"time-picker\""));
        assert!(interact.contains("data-slot=\"time-picker-control\""));
        assert!(interact.contains("type=\"time\""));
        assert!(interact.contains("style="));
        assert!(!html.contains("time-picker-control"));
        assert!(!html.contains("type=\"time\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("time-picker", "Time"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"time-picker\"]"));
        assert!(css.contains("[data-slot=\"time-picker\"]:disabled"));
        // wave1s geometry parity: React Button outline md + w-[240px] font-normal.
        assert!(css.contains(
            "[data-slot=\"time-picker\"] {\n  display: inline-flex; align-items: center; justify-content: flex-start; gap: 0.5rem;\n  width: 15rem; height: 2.5rem; padding: 0 1rem; box-sizing: border-box;"
        ));
        assert!(css.contains("  background: transparent;\n  box-shadow: var(--cronus-shadow-xs, none);"));
        assert!(css.contains("font-size: 0.875rem; line-height: 1.25rem; font-weight: 400;"));
        assert!(css.contains(
            "[data-slot=\"time-picker\"] svg {\n  width: 1rem; height: 1rem; flex-shrink: 0; pointer-events: none;\n  color: var(--cronus-fg-tertiary);\n}"
        ));
        assert!(!css.contains("[data-slot=\"date-picker-trigger\"],\n[data-slot=\"time-picker\"]"));
        assert!(!css.contains("[data-slot=\"time-picker-trigger\"]"));
        assert!(!css.contains("[data-slot=\"time-picker\"] > button"));
        assert!(css.contains("[data-slot=\"time-picker-content\"]"));
        assert!(css.contains("[data-slot=\"time-picker-column\"]"));
        assert!(css.contains("[data-slot=\"time-picker-option\"]"));
        assert!(css.contains("[data-slot=\"time-picker-columns\"]"));
        assert!(css.contains("[data-slot=\"time-picker-footer\"]"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-shadow-lg"));
        assert!(css.contains("z-index: 50"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
