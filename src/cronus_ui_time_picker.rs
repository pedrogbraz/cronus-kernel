//! Dedicated TimePicker renderer. DOM mirrors React closed state: the root
//! `data-slot="time-picker"` IS the outline trigger `<button>` (Clock glyph +
//! tabular `<span>` value). The panel is a native `popover` sibling
//! (`time-picker-content`, `w-auto p-3`), hidden until opened by
//! `popovertarget` — no JS. Inside: a flex row of columns, each a `<div>` with
//! the `<span>` header (Hr / Min / Sec / AM/PM) over the
//! `<div role="listbox" data-slot="time-picker-column">` of
//! `<label role="option" data-slot="time-picker-option">` + hidden radio
//! (one group per column: `widget_id` hours / minutes / seconds / ampm) for
//! every hour, every `minuteStep`-th minute, every `secondStep`-th second,
//! AM / PM, then the footer with `Now` (`data-time-now`, live.js checks the
//! current-time radios) and `Done` (secondary `sm`, hides the popover with
//! `popovertargetaction="hide"`). Esc / outside click dismiss it. The selected
//! option carries `aria-selected="true"` / `checked` and is the listbox's
//! `aria-activedescendant`; the chrome snaps it into view when the panel opens.
//! Without a value the trigger shows `placeholder:` and carries `data-empty`
//! (tertiary text, like React's `!isSet` span).
//!
//! Props: `value:"HH:mm[:ss]"` (or `defaultValue:`); `hourCycle:24`
//! (`style:time-picker+24`) drops AM/PM for a 0–23 column; `minuteStep:5`,
//! `showSeconds:true`, `secondStep:15`; `placeholder:` (default "Select time");
//! `disabled:true` locks the trigger, the listboxes (`aria-disabled`,
//! `tabindex="-1"`) and `Now`; `aria-label:"…"` names the trigger
//! ("Meeting time: 09:30 AM" once set) and the panel.
//! Not interact `input("time-picker", "time")`.

use crate::cronus_ui_kit::{attr, attr_nonempty, attr_num, esc, flag, item, label_of, widget_id};
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
    let disabled = flag(comp, "disabled");
    let time = parse_time(comp);
    let label = trigger_label(comp, time, hour_cycle, show_seconds);
    let trigger_id = widget_id(comp, "trigger");
    let pop_id = widget_id(comp, "panel");
    let mut btn = format!(
        "type=\"button\" id=\"{trigger_id}\" data-slot=\"time-picker\" data-variant=\"outline\" aria-haspopup=\"dialog\" aria-expanded=\"false\" data-state=\"closed\" popovertarget=\"{pop_id}\""
    );
    if time.is_none() {
        btn.push_str(" data-empty=\"\"");
    }
    if disabled {
        btn.push_str(" disabled");
    }
    if let Some(aria) = attr_nonempty(comp, "aria-label") {
        let shown = if time.is_some() {
            format!("{}: {label}", esc(aria))
        } else {
            esc(aria)
        };
        btn.push_str(&format!(" aria-label=\"{shown}\""));
    }
    let content_label = attr_nonempty(comp, "aria-label")
        .map(esc)
        .unwrap_or_else(|| "Choose a time".into());
    let columns = Columns {
        id: widget_id(comp, "col"),
        hours_name: widget_id(comp, "hours"),
        minutes_name: widget_id(comp, "minutes"),
        seconds_name: widget_id(comp, "seconds"),
        period_name: widget_id(comp, "ampm"),
        time,
        hour_cycle,
        show_seconds,
        minute_step: step_of(comp, "minuteStep"),
        second_step: step_of(comp, "secondStep"),
        disabled,
    }
    .html();
    // `Now` is live (`data-time-now`); React dims it only when the picker is
    // disabled.
    let now_attrs = if disabled {
        " disabled aria-disabled=\"true\""
    } else {
        " data-time-now"
    };
    format!(
        "<button {btn}>{ICON}<span>{label}</span></button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"time-picker-content\" aria-label=\"{content_label}\" anchor=\"{trigger_id}\">{columns}<div><button type=\"button\" data-slot=\"time-picker-now\" data-variant=\"ghost\"{now_attrs}>Now</button><button type=\"button\" data-slot=\"time-picker-done\" data-variant=\"secondary\" popovertarget=\"{pop_id}\" popovertargetaction=\"hide\">Done</button></div></div>"
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
    if let Some(v) = attr_nonempty(comp, "placeholder") {
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
    let raw = attr_nonempty(comp, "value")
        .or_else(|| attr_nonempty(comp, "defaultValue"))
        .or_else(|| item(comp, "value"))?;
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
    format!(
        "{}:{}{second_part} {period}",
        pad2(hour12(time.hours)),
        pad2(time.minutes)
    )
}

fn hour12(hours: u8) -> u8 {
    if hours % 12 == 0 {
        12
    } else {
        hours % 12
    }
}

fn pad2(n: u8) -> String {
    format!("{n:02}")
}

fn hour_cycle_of(comp: &ComponentNode) -> u8 {
    if let Some(v) = attr(comp, "hourCycle").or_else(|| attr(comp, "hour-cycle")) {
        if v.trim() == "24" {
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

/// React `Math.max(1, Math.floor(step))`.
fn step_of(comp: &ComponentNode, key: &str) -> u8 {
    attr_num::<f64>(comp, key)
        .filter(|n| n.is_finite())
        .map(|n| n.floor().clamp(1.0, 60.0) as u8)
        .unwrap_or(1)
}

struct Columns {
    id: String,
    hours_name: String,
    minutes_name: String,
    seconds_name: String,
    period_name: String,
    time: Option<TimeValue>,
    hour_cycle: u8,
    show_seconds: bool,
    minute_step: u8,
    second_step: u8,
    disabled: bool,
}

impl Columns {
    fn html(&self) -> String {
        let mut out = String::from("<div>");
        let hours: Vec<u8> = if self.hour_cycle == 24 {
            (0..24).collect()
        } else {
            vec![12, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
        };
        let selected_hour = self.time.map(|t| {
            if self.hour_cycle == 24 {
                t.hours
            } else {
                hour12(t.hours)
            }
        });
        out.push_str(&self.column("Hour", "Hr", &hours, selected_hour, &self.hours_name, false));
        let minutes: Vec<u8> = (0..60).step_by(self.minute_step as usize).collect();
        out.push_str(&self.column(
            "Minute",
            "Min",
            &minutes,
            self.time.map(|t| t.minutes),
            &self.minutes_name,
            true,
        ));
        if self.show_seconds {
            let seconds: Vec<u8> = (0..60).step_by(self.second_step as usize).collect();
            out.push_str(&self.column(
                "Second",
                "Sec",
                &seconds,
                self.time.map(|t| t.seconds),
                &self.seconds_name,
                true,
            ));
        }
        if self.hour_cycle == 12 {
            let period = self.time.map(|t| if t.hours < 12 { "AM" } else { "PM" });
            out.push_str(&self.period_column(period));
        }
        out.push_str("</div>");
        out
    }

    fn column(
        &self,
        label: &str,
        header: &str,
        values: &[u8],
        selected: Option<u8>,
        name: &str,
        pad_value: bool,
    ) -> String {
        let opts: Vec<(String, String, bool)> = values
            .iter()
            .map(|v| {
                let value = if pad_value { pad2(*v) } else { v.to_string() };
                (value, pad2(*v), selected == Some(*v))
            })
            .collect();
        self.listbox(label, header, &opts, name)
    }

    fn period_column(&self, selected: Option<&str>) -> String {
        let opts: Vec<(String, String, bool)> = ["AM", "PM"]
            .iter()
            .map(|p| (p.to_string(), p.to_string(), selected == Some(*p)))
            .collect();
        self.listbox("AM or PM", "AM/PM", &opts, &self.period_name)
    }

    /// `opts`: `(value, label, selected)`.
    fn listbox(
        &self,
        label: &str,
        header: &str,
        opts: &[(String, String, bool)],
        name: &str,
    ) -> String {
        let base = format!(
            "{}-{}",
            self.id,
            label.to_ascii_lowercase().replace(' ', "-")
        );
        let active = opts
            .iter()
            .find(|(_, _, on)| *on)
            .map(|(v, _, _)| format!(" aria-activedescendant=\"{base}-{v}\""))
            .unwrap_or_default();
        let disabled = if self.disabled {
            " aria-disabled=\"true\" tabindex=\"-1\""
        } else {
            " tabindex=\"0\""
        };
        let radio_off = if self.disabled { " disabled" } else { "" };
        let options: String = opts
            .iter()
            .map(|(v, text, on)| {
                let checked = if *on { " checked" } else { "" };
                format!(
                    "<label id=\"{base}-{v}\" role=\"option\" aria-selected=\"{on}\" data-slot=\"time-picker-option\"><input type=\"radio\" name=\"{name}\" value=\"{v}\"{checked}{radio_off}>{text}</label>"
                )
            })
            .collect();
        format!(
            "<div><span>{header}</span><div role=\"listbox\" aria-label=\"{label}\"{active}{disabled} data-slot=\"time-picker-column\">{options}</div></div>"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn reject_interact(html: &str) {
        assert!(!html.contains("type=\"time\""));
        assert!(!html.contains("time-picker-control"));
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
            "<button type=\"button\" id=\"cui-time-picker-trigger\" data-slot=\"time-picker\" data-variant=\"outline\" aria-haspopup=\"dialog\" aria-expanded=\"false\" data-state=\"closed\" popovertarget=\"cui-time-picker-panel\" data-empty=\"\">"
        ));
        assert_eq!(html.matches("data-slot=\"time-picker\"").count(), 1);
        assert!(html.contains(
            "<span>Time</span></button><div id=\"cui-time-picker-panel\" popover=\"auto\" data-slot=\"time-picker-content\" aria-label=\"Choose a time\" anchor=\"cui-time-picker-trigger\"><div><div><span>Hr</span><div role=\"listbox\" aria-label=\"Hour\" tabindex=\"0\" data-slot=\"time-picker-column\"><label id=\"cui-time-picker-col-hour-12\" role=\"option\" aria-selected=\"false\" data-slot=\"time-picker-option\"><input type=\"radio\" name=\"cui-time-picker-hours\" value=\"12\">12</label><label id=\"cui-time-picker-col-hour-1\""
        ));
        assert_eq!(html.matches("role=\"listbox\"").count(), 3);
        assert_eq!(html.matches("role=\"option\"").count(), 12 + 60 + 2);
        assert!(html.contains("name=\"cui-time-picker-hours\""));
        assert!(html.contains("name=\"cui-time-picker-minutes\""));
        assert!(html.contains("name=\"cui-time-picker-ampm\""));
        assert!(html.contains("aria-label=\"Minute\""));
        assert!(html.contains("aria-label=\"AM or PM\""));
        assert!(!html.contains("aria-activedescendant"));
        // Now is live (`data-time-now`); Done closes the native popover.
        assert!(html.ends_with(
            "</div></div><div><button type=\"button\" data-slot=\"time-picker-now\" data-variant=\"ghost\" data-time-now>Now</button><button type=\"button\" data-slot=\"time-picker-done\" data-variant=\"secondary\" popovertarget=\"cui-time-picker-panel\" popovertargetaction=\"hide\">Done</button></div></div>"
        ));
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
        text.config
            .insert("aria-label".into(), "Meeting time".into());
        c.items.push(text);
        let html = render(&c);
        assert!(html.starts_with("<button type=\"button\""));
        assert!(!html.starts_with("<div"));
        assert!(html.contains(" aria-label=\"Meeting time: 09:30 AM\">"));
        assert!(!html.contains("data-empty"));
        assert!(html.contains("<span>09:30 AM</span></button>"));
        assert!(html.contains("data-slot=\"time-picker-content\" aria-label=\"Meeting time\""));
        reject_interact(&html);
    }

    #[test]
    fn value_formats_12h_and_selects_options() {
        let mut c = stub("time-picker", "Time");
        c.props.insert("value".into(), "14:30".into());
        let html = render(&c);
        assert!(html.contains("<span>02:30 PM</span></button>"));
        assert!(html.contains("aria-label=\"Hour\" aria-activedescendant=\"cui-time-picker-col-hour-2\" tabindex=\"0\""));
        assert!(html.contains("id=\"cui-time-picker-col-hour-2\" role=\"option\" aria-selected=\"true\" data-slot=\"time-picker-option\"><input type=\"radio\" name=\"cui-time-picker-hours\" value=\"2\" checked>02</label>"));
        assert!(html.contains("id=\"cui-time-picker-col-minute-30\" role=\"option\" aria-selected=\"true\" data-slot=\"time-picker-option\"><input type=\"radio\" name=\"cui-time-picker-minutes\" value=\"30\" checked>30</label>"));
        assert!(html.contains("id=\"cui-time-picker-col-am-or-pm-PM\" role=\"option\" aria-selected=\"true\" data-slot=\"time-picker-option\"><input type=\"radio\" name=\"cui-time-picker-ampm\" value=\"PM\" checked>PM</label>"));
        assert_eq!(html.matches("aria-selected=\"true\"").count(), 3);
        reject_interact(&html);
    }

    /// Docs "24-hour with seconds": `hourCycle=24 minuteStep=5 showSeconds`.
    #[test]
    fn hour_cycle_24_steps_and_seconds() {
        let mut c = stub("time-picker", "Time");
        c.props.insert("value".into(), "14:45:00".into());
        c.props.insert("hourCycle".into(), "24".into());
        c.props.insert("minuteStep".into(), "5".into());
        c.props.insert("showSeconds".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("<span>14:45:00</span></button>"));
        assert_eq!(html.matches("role=\"listbox\"").count(), 3);
        assert!(html.contains("aria-label=\"Second\""));
        assert!(!html.contains("AM or PM"));
        assert!(!html.contains("name=\"cui-time-picker-ampm\""));
        assert_eq!(html.matches("role=\"option\"").count(), 24 + 12 + 60);
        assert!(html.contains("aria-selected=\"true\" data-slot=\"time-picker-option\"><input type=\"radio\" name=\"cui-time-picker-hours\" value=\"14\" checked>14</label>"));
        assert!(html.contains("aria-selected=\"true\" data-slot=\"time-picker-option\"><input type=\"radio\" name=\"cui-time-picker-minutes\" value=\"45\" checked>45</label>"));
        assert!(html.contains(
            "id=\"cui-time-picker-col-second-00\" role=\"option\" aria-selected=\"true\""
        ));
        reject_interact(&html);

        let mut style = stub("time-picker", "Time");
        style.style = Some("time-picker+24".into());
        style.props.insert("value".into(), "14:30".into());
        assert!(render(&style).contains("<span>14:30</span></button>"));
    }

    #[test]
    fn show_seconds_adds_column_with_second_step() {
        let mut c = stub("time-picker", "Time");
        c.props.insert("showSeconds".into(), "true".into());
        c.props.insert("secondStep".into(), "15".into());
        c.props.insert("value".into(), "09:15:45".into());
        let html = render(&c);
        assert!(html.contains("<span>09:15:45 AM</span></button>"));
        assert!(html.contains(
            "id=\"cui-time-picker-col-second-45\" role=\"option\" aria-selected=\"true\" data-slot=\"time-picker-option\"><input type=\"radio\" name=\"cui-time-picker-seconds\" value=\"45\" checked>"
        ));
        assert_eq!(html.matches("role=\"option\"").count(), 12 + 60 + 4 + 2);
        reject_interact(&html);
    }

    /// Docs "Disabled": `defaultValue="08:00" disabled` — the trigger, every
    /// listbox and `Now` are locked while the value still shows.
    #[test]
    fn disabled_locks_trigger_and_panel() {
        let mut c = stub("time-picker", "Time");
        c.props.insert("disabled".into(), "true".into());
        c.props.insert("defaultValue".into(), "08:00".into());
        c.props.insert("aria-label".into(), "Locked slot".into());
        let html = render(&c);
        assert!(html.contains(" disabled aria-label=\"Locked slot: 08:00 AM\">"));
        assert!(html.contains("<span>08:00 AM</span></button>"));
        assert_eq!(
            html.matches("aria-disabled=\"true\" tabindex=\"-1\"")
                .count(),
            3
        );
        assert!(!html.contains("tabindex=\"0\""));
        assert!(html.contains("data-slot=\"time-picker-now\" data-variant=\"ghost\" disabled aria-disabled=\"true\">Now</button>"));
        assert!(!html.contains("data-time-now"));
        assert!(html.contains(
            "<input type=\"radio\" name=\"cui-time-picker-hours\" value=\"8\" checked disabled>"
        ));
        reject_interact(&html);
    }

    #[test]
    fn steps_follow_react_clamping() {
        let mut c = stub("time-picker", "Time");
        c.props.insert("minuteStep".into(), "0".into());
        assert_eq!(step_of(&c, "minuteStep"), 1);
        c.props.insert("minuteStep".into(), "7.9".into());
        assert_eq!(step_of(&c, "minuteStep"), 7);
        c.props.insert("minuteStep".into(), "x".into());
        assert_eq!(step_of(&c, "minuteStep"), 1);
    }

    #[test]
    fn skips_interact_native_time_input() {
        let c = stub("time-picker", "Time");
        let html = render(&c);
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
        assert!(css
            .contains("  background: transparent;\n  box-shadow: var(--cronus-shadow-xs, none);"));
        assert!(css.contains("font-size: 0.875rem; line-height: 1.25rem; font-weight: 400;"));
        assert!(css.contains(
            "[data-slot=\"time-picker\"] svg {\n  width: 1rem; height: 1rem; flex-shrink: 0; pointer-events: none;\n  color: var(--cronus-fg-tertiary);\n}"
        ));
        assert!(!css.contains("[data-slot=\"date-picker-trigger\"],\n[data-slot=\"time-picker\"]"));
        assert!(!css.contains("[data-slot=\"time-picker-trigger\"]"));
        assert!(!css.contains("[data-slot=\"time-picker\"] > button"));
        assert!(!css.contains("[data-slot=\"time-picker-columns\"]"));
        assert!(!css.contains("[data-slot=\"time-picker-footer\"]"));
        assert!(css.contains("[data-slot=\"time-picker-content\"]:popover-open {"));
        assert!(css.contains("[data-slot=\"time-picker-column\"] {\n  height: 12rem; width: 4rem;"));
        assert!(css.contains(
            "mask-image: linear-gradient(to bottom, transparent, #000 22%, #000 78%, transparent);"
        ));
        assert!(css.contains("[data-slot=\"time-picker-option\"][aria-selected=\"true\"] {\n  background: color-mix(in oklch, var(--cronus-primary), black 30%); color: #fff; font-weight: 600;"));
        assert!(css.contains("[data-slot=\"time-picker-option\"]:has(> input:checked)"));
        assert!(css.contains("[data-slot=\"time-picker-now\"]:not(:disabled)"));
        assert!(css.contains("scroll-snap-align: center"));
        assert!(css.contains("[data-slot=\"time-picker-now\"]"));
        assert!(css.contains("[data-slot=\"time-picker-done\"]"));
        assert!(css.contains("var(--cronus-border-soft)"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-shadow-lg"));
        assert!(css.contains("z-index: 50"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
