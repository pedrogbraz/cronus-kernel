//! Dedicated Sparkline renderer. DOM matches React:
//! `<svg data-slot="sparkline">` + `<path data-slot="sparkline-line">` (line type).
//! Optional `<path data-slot="sparkline-area">`. Not the catalog `chart()` stub
//! (`<figure data-slot="sparkline"><figcaption>`).

use crate::cronus_ui_kit::{esc, flag_any, label_of};
use crate::parser::ComponentNode;

const WIDTH: f64 = 96.0;
const HEIGHT: f64 = 28.0;
const STROKE: f64 = 1.5;
const DEFAULT_SERIES: [f64; 7] = [4.0, 8.0, 2.0, 10.0, 6.0, 12.0, 5.0];
const TONES: &[&str] = &["primary", "success", "warning", "error", "info", "fg"];

pub fn render(comp: &ComponentNode) -> String {
    let points = series_of(comp);
    let tone = tone_of(comp);
    let area = flag_any(comp, "area") || style_has(comp, "area");
    let bar = type_is_bar(comp);
    let label = aria_label_of(comp, points.len());
    let attrs = format!(
        "data-slot=\"sparkline\" data-tone=\"{tone}\" role=\"img\" aria-label=\"{label}\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\" preserveAspectRatio=\"none\" fill=\"none\"",
        w = fmt(WIDTH),
        h = fmt(HEIGHT),
    );
    let inner = if bar {
        bar_marks(&points)
    } else {
        line_marks(&points, area)
    };
    format!("<svg {attrs}><title>{label}</title>{inner}</svg>")
}

fn line_marks(points: &[f64], area: bool) -> String {
    if points.is_empty() {
        return String::new();
    }
    let (line, area_d) = line_paths(points);
    let mut out = String::from("<g>");
    if area {
        if let Some(d) = area_d {
            out.push_str(&format!(
                "<path data-slot=\"sparkline-area\" d=\"{d}\" fill=\"color-mix(in oklch, currentColor 18%, transparent)\"></path>"
            ));
        }
    }
    out.push_str(&format!(
        "<path data-slot=\"sparkline-line\" d=\"{line}\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"{sw}\" stroke-linecap=\"round\" stroke-linejoin=\"round\" vector-effect=\"non-scaling-stroke\"></path>",
        sw = fmt(STROKE),
    ));
    out.push_str("</g>");
    out
}

fn bar_marks(points: &[f64]) -> String {
    if points.is_empty() {
        return String::new();
    }
    let pad = STROKE.max(1.0);
    let min = min_of(points);
    let max = max_of(points);
    let span = max - min;
    let count = points.len() as f64;
    let slot = if count > 0.0 { WIDTH / count } else { WIDTH };
    let gap = (slot * 0.3).min(4.0);
    let bar_w = (slot - gap).max(0.5);
    let radius = (bar_w / 2.0).min(2.0);
    let usable = HEIGHT - pad * 2.0;
    let mut out = String::from("<g fill=\"currentColor\">");
    for (index, value) in points.iter().enumerate() {
        let ratio = if span == 0.0 {
            0.0
        } else {
            (value - min) / span
        };
        let bar_h = (ratio * usable).max(1.0);
        let x = index as f64 * slot + gap / 2.0;
        let y = HEIGHT - pad - bar_h;
        out.push_str(&format!(
            "<rect data-slot=\"sparkline-bar\" x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" rx=\"{r}\"></rect>",
            x = fmt(x),
            y = fmt(y),
            w = fmt(bar_w),
            h = fmt(bar_h),
            r = fmt(radius),
        ));
    }
    out.push_str("</g>");
    out
}

fn line_paths(points: &[f64]) -> (String, Option<String>) {
    let pad = STROKE.max(1.0);
    let min = min_of(points);
    let max = max_of(points);
    let count = points.len();
    let step = if count > 1 {
        (WIDTH - pad * 2.0) / (count as f64 - 1.0)
    } else {
        0.0
    };
    let coords: Vec<(f64, f64)> = points
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let x = if count > 1 {
                pad + index as f64 * step
            } else {
                WIDTH / 2.0
            };
            let y = scale_y(*value, min, max, HEIGHT, pad);
            (x, y)
        })
        .collect();
    let line = if coords.len() == 1 {
        let y = coords[0].1;
        format!(
            "M {} {} L {} {}",
            fmt(pad),
            fmt(y),
            fmt(WIDTH - pad),
            fmt(y)
        )
    } else {
        coords
            .iter()
            .enumerate()
            .map(|(index, (x, y))| {
                let cmd = if index == 0 { "M" } else { "L" };
                format!("{cmd} {} {}", fmt(*x), fmt(*y))
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    let baseline = HEIGHT - pad;
    let area = if coords.len() == 1 {
        let y = coords[0].1;
        format!(
            "M {p} {y} L {q} {y} L {q} {b} L {p} {b} Z",
            p = fmt(pad),
            q = fmt(WIDTH - pad),
            y = fmt(y),
            b = fmt(baseline),
        )
    } else {
        let last = coords.last().unwrap();
        let first = coords.first().unwrap();
        format!(
            "{line} L {} {} L {} {} Z",
            fmt(last.0),
            fmt(baseline),
            fmt(first.0),
            fmt(baseline),
        )
    };
    (line, Some(area))
}

fn scale_y(value: f64, min: f64, max: f64, height: f64, pad: f64) -> f64 {
    let span = max - min;
    let usable = height - pad * 2.0;
    if span == 0.0 {
        height / 2.0
    } else {
        pad + (1.0 - (value - min) / span) * usable
    }
}

fn series_of(comp: &ComponentNode) -> Vec<f64> {
    let mut out = Vec::new();
    if let Some(raw) = comp.props.get("data") {
        out.extend(parse_series(raw));
    }
    for i in &comp.items {
        out.extend(parse_series(&i.text));
        if let Some(v) = i.config.get("value") {
            out.extend(parse_series(v));
        }
    }
    if out.is_empty() {
        for row in crate::cronus_ui_data::rows() {
            if let Some(v) = row.get("value") {
                out.extend(parse_series(&json_num(v)));
            }
        }
    }
    if out.is_empty() {
        DEFAULT_SERIES.to_vec()
    } else {
        out
    }
}

fn parse_series(s: &str) -> Vec<f64> {
    s.split(|c: char| c == ',' || c.is_whitespace())
        .filter_map(|part| {
            let part = part.trim();
            if part.is_empty() {
                return None;
            }
            part.parse::<f64>().ok().filter(|n| n.is_finite())
        })
        .collect()
}

fn json_num(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

fn aria_label_of(comp: &ComponentNode, count: usize) -> String {
    if let Some(v) = comp.props.get("aria-label").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    let label = label_of(comp);
    if !label.is_empty() && label != "sparkline" {
        return label;
    }
    let unit = if count == 1 { "point" } else { "points" };
    format!("Trend, {count} {unit}")
}

fn tone_of(comp: &ComponentNode) -> &'static str {
    if let Some(v) = comp.props.get("tone") {
        if let Some(t) = TONES.iter().copied().find(|t| *t == v.as_str()) {
            return t;
        }
    }
    let style = comp.style.as_deref().unwrap_or("");
    for part in style.split('+') {
        let part = part.trim();
        if let Some(t) = TONES.iter().copied().find(|t| *t == part) {
            return t;
        }
    }
    "primary"
}

fn type_is_bar(comp: &ComponentNode) -> bool {
    if comp.props.get("type").map(|s| s == "bar").unwrap_or(false) {
        return true;
    }
    style_has(comp, "bar")
}

fn style_has(comp: &ComponentNode, needle: &str) -> bool {
    comp.style
        .as_deref()
        .unwrap_or("")
        .split('+')
        .any(|part| part.trim() == needle)
}

fn min_of(points: &[f64]) -> f64 {
    points.iter().copied().fold(f64::INFINITY, f64::min)
}

fn max_of(points: &[f64]) -> f64 {
    points.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}

fn fmt(n: f64) -> String {
    let r = (n * 100.0).round() / 100.0;
    if r.fract() == 0.0 {
        format!("{}", r as i64)
    } else {
        format!("{r}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn extra(item_type: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: item_type.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_stub(html: &str) {
        assert!(!html.contains("<figure"));
        assert!(!html.contains("<figcaption"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_is_svg_line_not_figure_stub() {
        let html = render(&stub("sparkline", "Trend"));
        assert!(html.starts_with("<svg "));
        assert!(html.contains("data-slot=\"sparkline\""));
        assert!(html.contains("data-slot=\"sparkline-line\""));
        assert!(html.contains("data-tone=\"primary\""));
        assert!(html.contains("role=\"img\""));
        assert!(html.contains("<title>Trend</title>"));
        assert!(!html.contains("data-slot=\"sparkline-bar\""));
        reject_stub(&html);
    }

    #[test]
    fn numeric_items_drive_line_commands() {
        let mut c = stub("sparkline", "Revenue");
        c.items = vec![
            extra("value", "4"),
            extra("value", "8"),
            extra("value", "2"),
            extra("value", "10"),
        ];
        let html = render(&c);
        let d = html
            .split("data-slot=\"sparkline-line\" d=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        assert_eq!(d.matches('M').count(), 1);
        assert_eq!(d.matches('L').count(), 3);
        reject_stub(&html);
    }

    #[test]
    fn area_path_when_flagged() {
        let mut c = stub("sparkline", "Trend");
        c.props.insert("area".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("data-slot=\"sparkline-area\""));
        assert!(html.contains("data-slot=\"sparkline-line\""));
        reject_stub(&html);
    }

    #[test]
    fn area_from_style() {
        let mut c = stub("sparkline", "Trend");
        c.style = Some("sparkline+area".into());
        let html = render(&c);
        assert!(html.contains("data-slot=\"sparkline-area\""));
        reject_stub(&html);
    }

    #[test]
    fn bar_type_emits_rects() {
        let mut c = stub("sparkline", "Trend");
        c.props.insert("type".into(), "bar".into());
        c.items = vec![
            extra("value", "1"),
            extra("value", "2"),
            extra("value", "3"),
        ];
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"sparkline-bar\"").count(), 3);
        assert!(!html.contains("data-slot=\"sparkline-line\""));
        reject_stub(&html);
    }

    #[test]
    fn skips_chart_figure_stub() {
        let html = render(&stub("sparkline", "Trend"));
        assert!(!html.contains("<figure"));
        assert!(!html.contains("<figcaption"));
        reject_stub(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("sparkline", "Trend"));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"sparkline-line\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"sparkline\"]"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(css.contains("var(--cronus-success)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
