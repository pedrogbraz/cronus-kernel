//! Dedicated Context renderer (AI suite). DOM mirrors React `Context`
//! (a HoverCard): the trigger `<button data-slot="context-trigger"
//! data-variant="ghost">` (`<span>` percent + the ring `<svg role="img">`) and
//! `<div data-slot="context-content">` > `context-content-header` (percent,
//! `used / max` in compact notation, a `progress` bar) >
//! `context-content-body` (one `context-input-usage` / `-output-usage` /
//! `-reasoning-usage` / `-cache-usage` row per non-zero count, like React) >
//! `context-content-footer` (`Total cost` and the `cost-total:` or "—").
//! Both sit in a slotless `<span>` that anchors the card. Zero JS: the card is
//! a native `popover="auto"` opened by the trigger's `interestfor` (hover /
//! focus) with `popovertarget` as the click fallback, like `hover-card`.
//!
//! Numbers: `used:18432 max:128000 input:12000 output:4432 reasoning:800
//! cached:2048`, optional USD `cost-input:` … `cost-total:` (never estimated).
//! Formatting follows React's `Intl.NumberFormat("en-US")`: compact (`18K`,
//! `4.4K`), percent with one decimal (`14.4%`), currency (`$0.02`). The ring
//! `stroke-dashoffset` and `transform` are SVG attributes, not inline style.
//! The `label` item names the usage (`Model context usage`).

use crate::cronus_ui_kit::{attr_num, esc, item, widget_id};
use crate::parser::ComponentNode;

const CIRCUMFERENCE: f64 = 2.0 * std::f64::consts::PI * 10.0;
const USAGE: &str = "Model context usage";

/// `Intl.NumberFormat("en-US", { notation: "compact" })`.
pub fn compact(n: f64) -> String {
    if !n.is_finite() {
        return "—".into();
    }
    let neg = n < 0.0;
    let mut n = n.abs();
    if n < 1000.0 {
        return format!("{}{}", if neg { "-" } else { "" }, n.round());
    }
    let units = [(1e12, "T"), (1e9, "B"), (1e6, "M"), (1e3, "K")];
    loop {
        let Some((unit, suffix)) = units.iter().find(|(u, _)| n >= *u) else {
            return format!("{}{}", if neg { "-" } else { "" }, n.round());
        };
        let v = n / unit;
        let (text, overflow) = if v < 10.0 {
            let r = (v * 10.0).round() / 10.0;
            if r >= 10.0 {
                ("10".to_string(), false)
            } else if r.fract() == 0.0 {
                (format!("{}", r as i64), false)
            } else {
                (format!("{r:.1}"), false)
            }
        } else {
            let r = v.round();
            if r >= 1000.0 {
                (String::new(), true)
            } else {
                (format!("{}", r as i64), false)
            }
        };
        if overflow {
            n = unit * 1000.0;
            continue;
        }
        return format!("{}{text}{suffix}", if neg { "-" } else { "" });
    }
}

/// `Intl.NumberFormat("en-US", { style: "percent", maximumFractionDigits: 1 })`.
pub fn percent(ratio: f64) -> String {
    let v = (ratio * 1000.0).round() / 10.0;
    if v.fract() == 0.0 {
        format!("{}%", v as i64)
    } else {
        format!("{v:.1}%")
    }
}

/// `Intl.NumberFormat("en-US", { style: "currency", currency: "USD" })`.
pub fn currency(n: f64) -> String {
    let cents = (n.abs() * 100.0).round() as u64;
    let whole = cents / 100;
    let digits = whole.to_string();
    let mut grouped = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    format!(
        "{}${grouped}.{:02}",
        if n < 0.0 && cents > 0 { "-" } else { "" },
        cents % 100
    )
}

fn num(comp: &ComponentNode, key: &str) -> f64 {
    attr_num::<f64>(comp, key)
        .filter(|v| v.is_finite() && *v >= 0.0)
        .unwrap_or(0.0)
}

fn cost(comp: &ComponentNode, key: &str) -> Option<f64> {
    attr_num::<f64>(comp, key).filter(|v| v.is_finite())
}

fn row(slot: &str, label: &str, tokens: f64, cost: Option<f64>) -> String {
    if tokens <= 0.0 {
        return String::new();
    }
    let cost = cost
        .map(|c| format!("<span>• {}</span>", currency(c)))
        .unwrap_or_default();
    format!(
        "<div data-slot=\"{slot}\"><span>{label}</span><span>{}{cost}</span></div>",
        compact(tokens)
    )
}

pub fn render(comp: &ComponentNode) -> String {
    let used = num(comp, "used");
    let max = num(comp, "max");
    let ratio = if max > 0.0 { used / max } else { 0.0 };
    let pct = percent(ratio);
    let usage = item(comp, "label")
        .filter(|l| !l.is_empty())
        .map(esc)
        .unwrap_or_else(|| USAGE.into());
    let trigger_id = widget_id(comp, "context-trigger");
    let pop_id = widget_id(comp, "context");
    let dash = format!("{CIRCUMFERENCE} {CIRCUMFERENCE}");
    let offset = CIRCUMFERENCE * (1.0 - ratio.clamp(0.0, 1.0));
    let bar = (ratio * 100.0).clamp(0.0, 100.0);
    let step = bar.round() as i64;
    let now = if bar.fract() == 0.0 {
        format!("{}", bar as i64)
    } else {
        format!("{bar:.1}")
    };
    let state = if step >= 100 { "complete" } else { "loading" };
    let body = [
        row(
            "context-input-usage",
            "Input",
            num(comp, "input"),
            cost(comp, "cost-input"),
        ),
        row(
            "context-output-usage",
            "Output",
            num(comp, "output"),
            cost(comp, "cost-output"),
        ),
        row(
            "context-reasoning-usage",
            "Reasoning",
            num(comp, "reasoning"),
            cost(comp, "cost-reasoning"),
        ),
        row(
            "context-cache-usage",
            "Cache",
            num(comp, "cached"),
            cost(comp, "cost-cache"),
        ),
    ]
    .concat();
    let total = cost(comp, "cost-total")
        .map(currency)
        .unwrap_or_else(|| "—".into());
    format!(
        "<span><button type=\"button\" id=\"{trigger_id}\" data-slot=\"context-trigger\" data-variant=\"ghost\" interestfor=\"{pop_id}\" popovertarget=\"{pop_id}\"><span>{pct}</span><svg aria-label=\"{usage}\" height=\"20\" role=\"img\" viewBox=\"0 0 24 24\" width=\"20\"><circle cx=\"12\" cy=\"12\" fill=\"none\" opacity=\"0.25\" r=\"10\" stroke=\"currentColor\" stroke-width=\"2\"></circle><circle cx=\"12\" cy=\"12\" fill=\"none\" opacity=\"0.7\" r=\"10\" stroke=\"currentColor\" stroke-dasharray=\"{dash}\" stroke-dashoffset=\"{offset}\" stroke-linecap=\"round\" stroke-width=\"2\" transform=\"rotate(-90 12 12)\"></circle></svg></button><div id=\"{pop_id}\" popover=\"auto\" anchor=\"{trigger_id}\" data-slot=\"context-content\"><div data-slot=\"context-content-header\"><div><p>{pct}</p><p>{} / {}</p></div><div data-slot=\"progress\" role=\"progressbar\" aria-valuenow=\"{now}\" aria-valuemin=\"0\" aria-valuemax=\"100\" aria-valuetext=\"{now}%\" aria-label=\"{usage}\" data-state=\"{state}\" data-value=\"{step}\" data-max=\"100\"><div data-state=\"{state}\" data-value=\"{step}\" data-max=\"100\"></div></div></div><div data-slot=\"context-content-body\">{body}</div><div data-slot=\"context-content-footer\"><span>Total cost</span><span>{total}</span></div></div></span>",
        compact(used),
        compact(max)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn docs() -> ComponentNode {
        let mut c = stub("context", "Model context usage");
        for (k, v) in [
            ("used", "18432"),
            ("max", "128000"),
            ("input", "12000"),
            ("output", "4432"),
            ("reasoning", "800"),
            ("cached", "2048"),
        ] {
            c.props.insert(k.into(), v.into());
        }
        c
    }

    #[test]
    fn intl_formats_match_react() {
        assert_eq!(compact(18432.0), "18K");
        assert_eq!(compact(128000.0), "128K");
        assert_eq!(compact(12000.0), "12K");
        assert_eq!(compact(4432.0), "4.4K");
        assert_eq!(compact(800.0), "800");
        assert_eq!(compact(2048.0), "2K");
        assert_eq!(compact(999600.0), "1M");
        assert_eq!(compact(1250000.0), "1.3M");
        assert_eq!(percent(0.144), "14.4%");
        assert_eq!(percent(0.5), "50%");
        assert_eq!(currency(0.0213), "$0.02");
        assert_eq!(currency(1234.5), "$1,234.50");
    }

    #[test]
    fn docs_example_trigger_card_rows_and_footer() {
        let html = render(&docs());
        assert!(html.starts_with("<span><button type=\"button\" id=\"cui-context-context-trigger\" data-slot=\"context-trigger\" data-variant=\"ghost\" interestfor=\"cui-context-context\" popovertarget=\"cui-context-context\"><span>14.4%</span><svg aria-label=\"Model context usage\" height=\"20\" role=\"img\" viewBox=\"0 0 24 24\" width=\"20\"><circle cx=\"12\" cy=\"12\" fill=\"none\" opacity=\"0.25\" r=\"10\" stroke=\"currentColor\" stroke-width=\"2\"></circle><circle cx=\"12\" cy=\"12\" fill=\"none\" opacity=\"0.7\" r=\"10\" stroke=\"currentColor\" stroke-dasharray=\"62.83185307179586 62.83185307179586\" stroke-dashoffset=\"53.78406622945726\" stroke-linecap=\"round\" stroke-width=\"2\" transform=\"rotate(-90 12 12)\"></circle></svg></button><div id=\"cui-context-context\" popover=\"auto\" anchor=\"cui-context-context-trigger\" data-slot=\"context-content\"><div data-slot=\"context-content-header\"><div><p>14.4%</p><p>18K / 128K</p></div><div data-slot=\"progress\" role=\"progressbar\" aria-valuenow=\"14.4\""));
        assert!(html.contains("data-value=\"14\" data-max=\"100\"><div data-state=\"loading\" data-value=\"14\" data-max=\"100\"></div></div></div><div data-slot=\"context-content-body\"><div data-slot=\"context-input-usage\"><span>Input</span><span>12K</span></div><div data-slot=\"context-output-usage\"><span>Output</span><span>4.4K</span></div><div data-slot=\"context-reasoning-usage\"><span>Reasoning</span><span>800</span></div><div data-slot=\"context-cache-usage\"><span>Cache</span><span>2K</span></div></div><div data-slot=\"context-content-footer\"><span>Total cost</span><span>—</span></div></div></span>"));
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("hover-card-content"));
    }

    #[test]
    fn costs_zero_rows_and_hostile_label() {
        let mut c = stub("context", "<b>\"usage\"</b>");
        c.props.insert("used".into(), "50".into());
        c.props.insert("max".into(), "100".into());
        c.props.insert("input".into(), "50".into());
        c.props.insert("cost-input".into(), "0.5".into());
        c.props.insert("cost-total".into(), "1.25".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"&lt;b&gt;&quot;usage&quot;&lt;/b&gt;\""));
        assert!(html.contains("<span>50%</span>"));
        assert!(html.contains("<div data-slot=\"context-input-usage\"><span>Input</span><span>50<span>• $0.50</span></span></div></div>"));
        assert!(!html.contains("context-output-usage"));
        assert!(html.contains("<span>Total cost</span><span>$1.25</span>"));
        let empty = render(&stub("context", "usage"));
        assert!(empty.contains("<span>0%</span>"));
        assert!(empty.contains("<div data-slot=\"context-content-body\"></div>"));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/context.css");
        assert!(css.contains("[data-slot=\"context-trigger\"]"));
        assert!(css.contains("[data-slot=\"context-content\"]:not(:popover-open)"));
        assert!(css.contains("min-width: 240px"));
        assert!(!css.contains("zinc-"));
    }

    #[test]
    fn registered_as_dedicated() {
        use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
        assert_eq!(
            dedicated_fn_name("context"),
            Some("cronus_ui_context::render")
        );
        assert_eq!(
            renderer_kind("context"),
            RendererKind::Dedicated("cronus_ui_context::render")
        );
        let c = stub("context", "usage");
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), render(&c));
    }
}
