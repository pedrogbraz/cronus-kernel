//! Dedicated Countdown renderer. DOM matches React's settled render:
//! `<div data-slot="countdown" role="timer" aria-live="off">` + visually
//! hidden summary `<span class="sr-only">2 days, 14 hours, 25 min, 9 sec</span>`
//! + four `countdown-unit` tiles, each a `countdown-value` (with the inner
//! keyed `<span>` React transitions in from `@starting-style`) over a
//! `countdown-label` caption. The kernel has no clock, so the tiles are static:
//! `value:"D:HH:MM:SS"` / `"HH:MM:SS"` / `"MM:SS"` (a `text` item or `target:`
//! in the same shape also works) or up to four numeric items, right-aligned to
//! seconds; anything else reads `00`. `compact:true` is React's `compact`
//! (`data-compact`, smaller tiles and type). Captions localize with
//! `days:"dias" hours:"horas" minutes:"min" seconds:"seg"` (React `labels`).
//! `aria-label:"…"` names the timer. Zero JS timer. CSS in `countdown.css`.

use crate::cronus_ui_kit::{attr_nonempty, esc, flag, numeric_items};
use crate::parser::ComponentNode;

const UNITS: [(&str, &str); 4] = [
    ("days", "days"),
    ("hours", "hours"),
    ("minutes", "min"),
    ("seconds", "sec"),
];

pub fn render(comp: &ComponentNode) -> String {
    let values = values_of(comp);
    let labels: Vec<String> = UNITS
        .iter()
        .map(|(unit, label)| esc(attr_nonempty(comp, unit).unwrap_or(label)))
        .collect();
    let summary = labels
        .iter()
        .zip(values.iter())
        .map(|(label, value)| format!("{} {label}", value.parse::<u64>().unwrap_or(0)))
        .collect::<Vec<_>>()
        .join(", ");
    let units = UNITS
        .iter()
        .zip(labels.iter())
        .zip(values.iter())
        .map(|(((unit, _), label), value)| {
            format!(
                "<div data-slot=\"countdown-unit\" data-unit=\"{unit}\" aria-hidden=\"true\"><span data-slot=\"countdown-value\"><span>{value}</span></span><span data-slot=\"countdown-label\">{label}</span></div>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let mut attrs = String::from("data-slot=\"countdown\" role=\"timer\" aria-live=\"off\"");
    if let Some(aria) = attr_nonempty(comp, "aria-label") {
        attrs.push_str(&format!(" aria-label=\"{}\"", esc(aria)));
    }
    if flag(comp, "compact") {
        attrs.push_str(" data-compact=\"\"");
    }
    format!("<div {attrs}><span class=\"sr-only\">{summary}</span>{units}</div>")
}

fn values_of(comp: &ComponentNode) -> [String; 4] {
    for i in &comp.items {
        if let Some(parts) = parse_clock(&i.text) {
            return parts;
        }
    }
    if let Some(v) = comp.props.get("value") {
        if let Some(parts) = parse_clock(v) {
            return parts;
        }
    }
    if let Some(v) = comp.props.get("target") {
        if let Some(parts) = parse_clock(v) {
            return parts;
        }
    }
    left_pad_nums(&numeric_items(comp))
}

fn parse_clock(raw: &str) -> Option<[String; 4]> {
    let t = raw.trim();
    if !t.contains(':') {
        return None;
    }
    let parts: Vec<&str> = t.split(':').collect();
    if parts.len() < 2 || parts.len() > 4 {
        return None;
    }
    let mut nums = Vec::with_capacity(parts.len());
    for p in parts {
        if p.is_empty() || !p.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        nums.push(p.parse::<u64>().ok()?);
    }
    Some(left_pad_u64(&nums))
}

fn left_pad_nums(nums: &[f64]) -> [String; 4] {
    let vals: Vec<u64> = nums
        .iter()
        .take(4)
        .map(|n| n.abs().floor() as u64)
        .collect();
    left_pad_u64(&vals)
}

fn left_pad_u64(nums: &[u64]) -> [String; 4] {
    let mut out = ["00", "00", "00", "00"].map(str::to_string);
    if nums.is_empty() {
        return out;
    }
    let take = nums.len().min(4);
    let start = 4 - take;
    for (i, n) in nums.iter().take(take).enumerate() {
        out[start + i] = pad2(*n);
    }
    out
}

fn pad2(n: u64) -> String {
    format!("{n:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("setTimeout"));
        assert!(!html.contains("requestAnimationFrame"));
        assert!(!html.contains("Date.now"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    fn unit<'a>(html: &'a str, name: &str) -> &'a str {
        let needle = format!("data-unit=\"{name}\"");
        let start = html
            .find(&needle)
            .unwrap_or_else(|| panic!("{name} in {html}"));
        &html[start..]
    }

    #[test]
    fn root_is_timer_with_four_static_units_not_fx_title_box() {
        let html = render(&stub("countdown", "Demo"));
        assert!(html.starts_with("<div data-slot=\"countdown\" role=\"timer\" aria-live=\"off\">"));
        assert_eq!(html.matches("data-slot=\"countdown-unit\"").count(), 4);
        assert_eq!(html.matches("data-slot=\"countdown-value\"").count(), 4);
        assert_eq!(html.matches("data-slot=\"countdown-label\"").count(), 4);
        assert!(html.contains(
            "<div data-slot=\"countdown-unit\" data-unit=\"days\" aria-hidden=\"true\"><span data-slot=\"countdown-value\"><span>00</span></span><span data-slot=\"countdown-label\">days</span></div>"
        ));
        assert!(html.contains(">hours</span>"));
        assert!(html.contains(">min</span>"));
        assert!(html.contains(">sec</span>"));
        assert!(unit(&html, "seconds").contains("><span>00</span>"));
        assert!(!html.contains("data-compact"));
        reject_fx(&html);
    }

    /// Wave 1t: React's settled render (past target) carries a visually hidden
    /// summary before the tiles; its text is part of the root's innerText.
    #[test]
    fn sr_only_summary_precedes_tiles() {
        let html = render(&stub("countdown", "Launch"));
        assert!(html.starts_with(
            "<div data-slot=\"countdown\" role=\"timer\" aria-live=\"off\"><span class=\"sr-only\">0 days, 0 hours, 0 min, 0 sec</span><div data-slot=\"countdown-unit\" data-unit=\"days\""
        ));
        assert!(!html.contains("Launch"));
        reject_fx(&html);
    }

    #[test]
    fn clock_text_fills_static_values() {
        let mut c = stub("countdown", "Launch");
        c.items.push(extra("text", "00:00:00"));
        let html = render(&c);
        assert!(unit(&html, "hours").contains("><span>00</span>"));
        assert!(unit(&html, "minutes").contains("><span>00</span>"));
        assert!(unit(&html, "seconds").contains("><span>00</span>"));
        reject_fx(&html);
    }

    #[test]
    fn numeric_texts_fill_units_from_the_right() {
        let mut c = stub("countdown", "Launch");
        c.items.push(extra("item", "1"));
        c.items.push(extra("item", "2"));
        c.items.push(extra("item", "3"));
        c.items.push(extra("item", "4"));
        let html = render(&c);
        assert!(unit(&html, "days").contains("><span>01</span>"));
        assert!(unit(&html, "hours").contains("><span>02</span>"));
        assert!(unit(&html, "minutes").contains("><span>03</span>"));
        assert!(unit(&html, "seconds").contains("><span>04</span>"));
        assert!(html.contains("<span class=\"sr-only\">1 days, 2 hours, 3 min, 4 sec</span>"));
        reject_fx(&html);
    }

    #[test]
    fn three_numbers_are_hours_minutes_seconds() {
        let mut c = stub("countdown", "Launch");
        c.items.push(extra("text", "12"));
        c.items.push(extra("text", "34"));
        c.items.push(extra("text", "56"));
        let html = render(&c);
        assert!(unit(&html, "days").contains("><span>00</span>"));
        assert!(unit(&html, "hours").contains("><span>12</span>"));
        assert!(unit(&html, "minutes").contains("><span>34</span>"));
        assert!(unit(&html, "seconds").contains("><span>56</span>"));
        reject_fx(&html);
    }

    /// Docs "Launch countdown": 2 days 14 h 25 min 9 s from now, named.
    #[test]
    fn clock_from_props_value_with_days_and_aria_label() {
        let mut c = stub("countdown", "Launch");
        c.props.insert("value".into(), "2:14:25:09".into());
        c.props
            .insert("aria-label".into(), "Launch countdown".into());
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"countdown\" role=\"timer\" aria-live=\"off\" aria-label=\"Launch countdown\"><span class=\"sr-only\">2 days, 14 hours, 25 min, 9 sec</span>"));
        assert!(unit(&html, "days").contains("><span>02</span>"));
        assert!(unit(&html, "hours").contains("><span>14</span>"));
        assert!(unit(&html, "minutes").contains("><span>25</span>"));
        assert!(unit(&html, "seconds").contains("><span>09</span>"));
        reject_fx(&html);
    }

    /// Docs "Compact with completion": `compact` is `data-compact` on the root.
    #[test]
    fn compact_and_custom_labels() {
        let mut c = stub("countdown", "Offer");
        c.props.insert("compact".into(), "true".into());
        c.props.insert("value".into(), "00:00:15".into());
        c.props.insert("days".into(), "dias".into());
        c.props.insert("seconds".into(), "seg".into());
        let html = render(&c);
        assert!(html.starts_with(
            "<div data-slot=\"countdown\" role=\"timer\" aria-live=\"off\" data-compact=\"\"><span class=\"sr-only\">0 dias, 0 hours, 0 min, 15 seg</span>"
        ));
        assert!(html.contains("<span data-slot=\"countdown-label\">dias</span>"));
        assert!(html.contains("<span data-slot=\"countdown-label\">seg</span>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped_out_of_tiles() {
        let html = render(&stub("countdown", "A <B> & \"C\""));
        assert!(!html.contains("<B>"));
        assert!(!html.contains("A <B>"));
        assert!(html.contains("data-slot=\"countdown-label\">days</span>"));
        let mut c = stub("countdown", "x");
        c.props.insert("hours".into(), "<h>".into());
        assert!(render(&c).contains("data-slot=\"countdown-label\">&lt;h&gt;</span>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("countdown", "Demo");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("countdown"),
            Some("cronus_ui_countdown::render")
        );
        assert_eq!(
            renderer_kind("countdown"),
            RendererKind::Dedicated("cronus_ui_countdown::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("countdown", "Demo"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"countdown-unit\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"countdown\"]"));
        assert!(css.contains("[data-slot=\"countdown\"] > .sr-only {"));
        assert!(css.contains("[data-slot=\"countdown\"][data-compact] { gap: 0.25rem; }"));
        assert!(css.contains("[data-slot=\"countdown-unit\"]"));
        assert!(css.contains(
            "[data-slot=\"countdown\"][data-compact] > [data-slot=\"countdown-unit\"] {"
        ));
        assert!(css.contains("[data-slot=\"countdown-value\"]"));
        assert!(css.contains("[data-slot=\"countdown-value\"] > span {\n  display: block;\n  transition: transform 300ms var(--ease-out-quart), opacity 300ms var(--ease-out-quart);"));
        assert!(css.contains("@starting-style {\n  [data-slot=\"countdown-value\"] > span { transform: translateY(-0.375rem); opacity: 0; }\n}"));
        assert!(css.contains("[data-slot=\"countdown-label\"]"));
        assert!(css.contains("font-variant-numeric: tabular-nums"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(css.contains("var(--cronus-fg-tertiary"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
