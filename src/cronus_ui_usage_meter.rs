//! Dedicated UsageMeter renderer (linear variant). DOM matches React:
//! `<div data-slot="usage-meter">`
//!   `<div>` (flex baseline justify-between, text-sm)
//!     `<span data-slot="usage-meter-label">{label}</span>` (or `<span></span>`)
//!     `<span data-slot="usage-meter-value"><span>{value} / {max}</span><span>{pct}%</span></span>`
//!   `<div data-slot="usage-meter-track" role="meter" aria-*>`
//!     `<div data-slot="usage-meter-fill" data-tone data-value="{round(ratio×100)}">`
//! No inline style: `data-value` is an integer 0..100, COMPONENT_CHROME maps
//! each `[data-value="N"]` to `--cui-progress-value: N` and the fill width reads
//! it (portable; typed `attr()` is Chromium-only). Tone auto: >90% error,
//! >75% warning, else primary.
//! `value` / `max` / `unit` / `aria-label` come from props or item config.

use crate::cronus_ui_chart::prop;
use crate::cronus_ui_kit::{esc, item};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let max = number(prop(comp, "max")).unwrap_or(100.0);
    let value = number(prop(comp, "value"))
        .or_else(|| comp.items.iter().find_map(|i| number(Some(&i.text))))
        .unwrap_or(40.0);
    let ratio = if value.is_finite() && max.is_finite() && max > 0.0 {
        (value / max).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let percent = (ratio * 100.0).round() as i64;
    let tone = if ratio > 0.9 {
        "error"
    } else if ratio > 0.75 {
        "warning"
    } else {
        "primary"
    };
    let readout = format!("{} / {}", group(value), group(max));
    let value_text = match prop(comp, "unit") {
        Some(u) if !u.is_empty() => format!("{readout} {}", esc(u)),
        _ => readout,
    };
    let label = label_of(comp);
    let aria = prop(comp, "aria-label")
        .map(esc)
        .or_else(|| label.clone())
        .unwrap_or_else(|| value_text.clone());
    let label_html = match &label {
        Some(l) => format!("<span data-slot=\"usage-meter-label\">{l}</span>"),
        None => "<span></span>".to_string(),
    };
    let safe_max = if max.is_finite() && max > 0.0 {
        max
    } else {
        0.0
    };
    let safe_value = if value.is_finite() {
        value.clamp(0.0, safe_max)
    } else {
        0.0
    };
    format!(
        "<div data-slot=\"usage-meter\"><div>{label_html}<span data-slot=\"usage-meter-value\"><span>{value_text}</span><span>{percent}%</span></span></div><div data-slot=\"usage-meter-track\" role=\"meter\" aria-valuenow=\"{now}\" aria-valuemin=\"0\" aria-valuemax=\"{mx}\" aria-valuetext=\"{percent}%\" aria-label=\"{aria}\"><div data-slot=\"usage-meter-fill\" data-tone=\"{tone}\" data-value=\"{fill}\"></div></div></div>",
        now = plain(safe_value),
        mx = plain(safe_max),
        fill = percent,
    )
}

fn label_of(comp: &ComponentNode) -> Option<String> {
    for kind in ["label", "title"] {
        if let Some(t) = item(comp, kind) {
            if !t.is_empty() && t.trim().parse::<f64>().is_err() {
                return Some(esc(t));
            }
        }
    }
    None
}

fn number(s: Option<&str>) -> Option<f64> {
    s.and_then(|v| v.trim().parse::<f64>().ok())
}

fn plain(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

/// `Intl.NumberFormat("en-US")`: thousands grouping, up to 3 fraction digits.
fn group(n: f64) -> String {
    if !n.is_finite() {
        return "0".into();
    }
    let rounded = (n.abs() * 1000.0).round() / 1000.0;
    let int = rounded.trunc() as u64;
    let digits = int.to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    let frac = rounded.fract();
    if frac > 0.0 {
        let f = format!("{frac:.3}");
        out.push_str(f.trim_start_matches('0').trim_end_matches('0'));
    }
    if n < 0.0 {
        format!("-{out}")
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn fixture() -> ComponentNode {
        let mut c = stub("usage-meter", "Tokens");
        c.items[0].config.insert("value".into(), "40".into());
        c.items[0]
            .config
            .insert("aria-label".into(), "Token usage".into());
        c
    }

    #[test]
    fn fixture_matches_react_linear_meter() {
        assert_eq!(
            render(&fixture()),
            "<div data-slot=\"usage-meter\"><div><span data-slot=\"usage-meter-label\">Tokens</span><span data-slot=\"usage-meter-value\"><span>40 / 100</span><span>40%</span></span></div><div data-slot=\"usage-meter-track\" role=\"meter\" aria-valuenow=\"40\" aria-valuemin=\"0\" aria-valuemax=\"100\" aria-valuetext=\"40%\" aria-label=\"Token usage\"><div data-slot=\"usage-meter-fill\" data-tone=\"primary\" data-value=\"40\"></div></div></div>"
        );
    }

    #[test]
    fn no_inline_style_and_auto_tone() {
        let mut c = stub("usage-meter", "Tokens");
        c.props.insert("value".into(), "9500".into());
        c.props.insert("max".into(), "10000".into());
        let html = render(&c);
        assert!(!html.contains("style="));
        assert!(html.contains("<span>9,500 / 10,000</span><span>95%</span>"));
        assert!(html.contains("data-tone=\"error\""));
    }

    #[test]
    fn unlabeled_keeps_spacer_span() {
        let mut c = stub("usage-meter", "");
        c.items.clear();
        let html = render(&c);
        assert!(html.contains("<div><span></span><span data-slot=\"usage-meter-value\">"));
        assert!(html.contains("aria-label=\"40 / 100\""));
    }

    #[test]
    fn fill_data_value_is_rounded_integer_percent() {
        let mut c = stub("usage-meter", "Tokens");
        c.props.insert("value".into(), "1".into());
        c.props.insert("max".into(), "3".into());
        let html = render(&c);
        assert!(
            html.contains("data-slot=\"usage-meter-fill\" data-tone=\"primary\" data-value=\"33\""),
            "{html}"
        );
        assert!(!html.contains("data-value=\"33.33\""));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_sizes_fill_from_per_value_rules() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(!css.contains("attr(data-value type("));
        assert!(css.contains("[data-slot=\"usage-meter-fill\"] {\n  height: 100%; border-radius: 9999px;\n  width: calc(var(--cui-progress-value, 0) * 1%);"));
        assert!(css.contains("[data-slot=\"usage-meter-fill\"], [data-slot=\"scroll-progress-fill\"])[data-value=\"37\"] { --cui-progress-value: 37; }"));
        assert!(css.contains("[data-slot=\"usage-meter\"] > div:first-child {\n  display: flex; align-items: baseline; justify-content: space-between; gap: 0.5rem;\n  font-size: 0.875rem; line-height: 1.25rem;\n}"));
    }
}
