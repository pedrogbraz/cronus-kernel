//! Dedicated Sparkline renderer. DOM matches React:
//! `<svg data-slot="sparkline" data-tone>` + `<path data-slot="sparkline-line">`
//! (line type) with an optional gradient-filled `<path data-slot="sparkline-area">`
//! (`area:true` or the `area` style segment), or `<rect data-slot="sparkline-bar">`
//! columns (`type:bar` / `bar` segment). The series is the `data:"4,6,5"` prop
//! or numeric `value` / `item` / `text` lines; `width:` / `height:` size the box
//! (96×28 like React), `full:true` is the docs' `w-full` class. The accessible
//! name is `aria-label:`, else a `label` item, else React's "Trend, N points".
//!
//! With `label` + `value` (+ `trend "+12.5%" tone:up`) items the renderer
//! emits the docs' stat card: `Card > CardContent(flex-col gap-3 pt-6)` holding
//! a Metric and the sparkline. Not the catalog `chart()` stub
//! (`<figure data-slot="sparkline"><figcaption>`).
use crate::cronus_ui_kit::{attr_nonempty, attr_num, esc, flag, flag_any, item};
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
    let width = attr_num::<f64>(comp, "width")
        .filter(|w| *w > 0.0)
        .unwrap_or(WIDTH);
    let height = attr_num::<f64>(comp, "height")
        .filter(|h| *h > 0.0)
        .unwrap_or(HEIGHT);
    let label = aria_label_of(comp, points.len());
    let class = if flag(comp, "full") {
        " class=\"w-full\""
    } else {
        ""
    };
    let attrs = format!(
        "data-slot=\"sparkline\" data-tone=\"{tone}\"{class} role=\"img\" aria-label=\"{label}\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\" preserveAspectRatio=\"none\" fill=\"none\"",
        w = fmt(width),
        h = fmt(height),
    );
    let inner = if bar {
        bar_marks(&points, width, height)
    } else {
        line_marks(&points, area, width, height, &gradient_id(comp))
    };
    let svg = format!("<svg {attrs}><title>{label}</title>{inner}</svg>");
    match stat_metric(comp) {
        Some(metric) => {
            crate::cronus_ui_card::content_card("cui-stat-card", &format!("{metric}{svg}"))
        }
        None => svg,
    }
}

/// The docs' stat card metric when the component carries `label` + `value`.
fn stat_metric(comp: &ComponentNode) -> Option<String> {
    let value = item(comp, "value").filter(|v| !v.is_empty())?;
    let label = item(comp, "label")
        .or_else(|| item(comp, "title"))
        .filter(|l| !l.is_empty())?;
    let delta = comp
        .items
        .iter()
        .find(|i| i.item_type == "trend" && !i.text.is_empty())
        .map(|i| {
            let trend = match i.tone.as_deref().map(str::trim) {
                Some("up") | Some("success") => "up",
                Some("down") | Some("error") | Some("danger") => "down",
                _ => "neutral",
            };
            (trend.to_string(), esc(&i.text))
        });
    Some(crate::cronus_ui_metric::metric_html(
        &esc(label),
        &esc(value),
        delta.as_ref().map(|(t, d)| (t.as_str(), d.as_str())),
    ))
}

/// Page-unique gradient id (React `useId`), so two area sparklines never share a `<defs>` id.
fn gradient_id(comp: &ComponentNode) -> String {
    crate::cronus_ui_kit::instance_id(comp, "gradient")
}

fn line_marks(points: &[f64], area: bool, width: f64, height: f64, gradient: &str) -> String {
    if points.is_empty() {
        return String::new();
    }
    let (line, area_d) = line_paths(points, width, height);
    let mut out = String::from("<g>");
    if area {
        if let Some(d) = area_d {
            out.push_str(&format!(
                "<defs><linearGradient id=\"{gradient}\" x1=\"0\" y1=\"0\" x2=\"0\" y2=\"1\"><stop offset=\"0%\" stop-color=\"color-mix(in oklch, currentColor 18%, transparent)\"></stop><stop offset=\"100%\" stop-color=\"color-mix(in oklch, currentColor 0%, transparent)\"></stop></linearGradient></defs><path data-slot=\"sparkline-area\" d=\"{d}\" fill=\"url(#{gradient})\"></path>"
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

fn bar_marks(points: &[f64], width: f64, height: f64) -> String {
    if points.is_empty() {
        return String::new();
    }
    let pad = STROKE.max(1.0);
    let min = min_of(points);
    let max = max_of(points);
    let span = max - min;
    let count = points.len() as f64;
    let slot = if count > 0.0 { width / count } else { width };
    let gap = (slot * 0.3).min(4.0);
    let bar_w = (slot - gap).max(0.5);
    let radius = (bar_w / 2.0).min(2.0);
    let usable = height - pad * 2.0;
    let mut out = String::from("<g fill=\"currentColor\">");
    for (index, value) in points.iter().enumerate() {
        let ratio = if span == 0.0 {
            0.0
        } else {
            (value - min) / span
        };
        let bar_h = (ratio * usable).max(1.0);
        let x = index as f64 * slot + gap / 2.0;
        let y = height - pad - bar_h;
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

fn line_paths(points: &[f64], width: f64, height: f64) -> (String, Option<String>) {
    let pad = STROKE.max(1.0);
    let min = min_of(points);
    let max = max_of(points);
    let count = points.len();
    let step = if count > 1 {
        (width - pad * 2.0) / (count as f64 - 1.0)
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
                width / 2.0
            };
            let y = scale_y(*value, min, max, height, pad);
            (x, y)
        })
        .collect();
    let line = if coords.len() == 1 {
        let y = coords[0].1;
        format!(
            "M {} {} L {} {}",
            fmt(pad),
            fmt(y),
            fmt(width - pad),
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
    let baseline = height - pad;
    let area = if coords.len() == 1 {
        let y = coords[0].1;
        format!(
            "M {p} {y} L {q} {y} L {q} {b} L {p} {b} Z",
            p = fmt(pad),
            q = fmt(width - pad),
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

/// The series: the `data:` prop, else numeric `value` / `item` / `text` lines
/// (a metric `value "$48,290"` or a label never count), else bound rows.
fn series_of(comp: &ComponentNode) -> Vec<f64> {
    let mut out = Vec::new();
    if let Some(raw) = attr_nonempty(comp, "data") {
        out.extend(parse_series(raw));
    }
    if out.is_empty() {
        for i in comp
            .items
            .iter()
            .filter(|i| matches!(i.item_type.as_str(), "value" | "item" | "text"))
        {
            out.extend(parse_series(&i.text));
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

/// Every comma/space separated token must be a finite number, else nothing.
fn parse_series(s: &str) -> Vec<f64> {
    let parts: Vec<&str> = s
        .split(|c: char| c == ',' || c.is_whitespace())
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    let nums: Vec<f64> = parts
        .iter()
        .filter_map(|p| p.parse::<f64>().ok().filter(|n| n.is_finite()))
        .collect();
    if nums.len() == parts.len() {
        nums
    } else {
        Vec::new()
    }
}

fn json_num(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

fn aria_label_of(comp: &ComponentNode, count: usize) -> String {
    if let Some(v) = attr_nonempty(comp, "aria-label") {
        return esc(v);
    }
    if stat_metric(comp).is_none() {
        if let Some(l) = item(comp, "label").filter(|l| !l.is_empty()) {
            return esc(l);
        }
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
    use crate::cronus_ui_kit::{reset_instance_ids, stub};
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
        assert!(html.contains("width=\"96\" height=\"28\" viewBox=\"0 0 96 28\""));
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
        reset_instance_ids();
        let mut c = stub("sparkline", "Trend");
        c.props.insert("area".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("<defs><linearGradient id=\"cui-sparkline-gradient\" x1=\"0\" y1=\"0\" x2=\"0\" y2=\"1\"><stop offset=\"0%\" stop-color=\"color-mix(in oklch, currentColor 18%, transparent)\"></stop>"));
        assert!(html.contains("<path data-slot=\"sparkline-area\" d=\"M 1.5 "));
        assert!(html.contains("fill=\"url(#cui-sparkline-gradient)\""));
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

    /// Docs "Line, area & bar": the `data:` prop is the series, the tone and
    /// mode come from the style, and the default name counts the points; a
    /// label with digits in it is not a series.
    #[test]
    fn data_prop_and_default_name() {
        let mut c = stub("sparkline", "Trend, 8 points");
        c.items.clear();
        c.style = Some("sparkline+info+bar".into());
        c.props.insert("data".into(), "4,6,5,8,7,11,9,13".into());
        let html = render(&c);
        assert!(html.starts_with("<svg data-slot=\"sparkline\" data-tone=\"info\" role=\"img\" aria-label=\"Trend, 8 points\" width=\"96\" height=\"28\""));
        assert_eq!(html.matches("data-slot=\"sparkline-bar\"").count(), 8);
        let mut l = stub("sparkline", "Last 8 days");
        l.props.insert("data".into(), "1,2".into());
        let html = render(&l);
        assert!(html.contains("aria-label=\"Last 8 days\""));
        assert_eq!(html.matches(" L ").count(), 1);
        reject_stub(&html);
    }

    /// Docs "In stat cards": label + value + trend items wrap a Metric and the
    /// `w-full` 36px sparkline in a Card's content.
    #[test]
    fn stat_card_wraps_metric_and_sparkline() {
        let mut c = stub("sparkline", "Revenue");
        c.style = Some("sparkline+success+area".into());
        c.props
            .insert("data".into(), "18,22,19,27,24,31,29,38".into());
        c.props.insert("height".into(), "36".into());
        c.props.insert("full".into(), "true".into());
        c.props
            .insert("aria-label".into(), "Revenue, trending up".into());
        c.items.push(extra("value", "$48,290"));
        let mut t = extra("trend", "+12.5%");
        t.tone = Some("up".into());
        c.items.push(t);
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"card\"><div data-slot=\"card-content\" class=\"cui-stat-card\"><div data-slot=\"metric\"><div data-slot=\"metric-label\">Revenue</div><div data-slot=\"metric-value\">$48,290</div><span data-slot=\"metric-delta\" class=\"t-up\">"));
        assert!(html.contains("<svg data-slot=\"sparkline\" data-tone=\"success\" class=\"w-full\" role=\"img\" aria-label=\"Revenue, trending up\" width=\"96\" height=\"36\" viewBox=\"0 0 96 36\""));
        assert!(html.contains("data-slot=\"sparkline-area\""));
        assert!(html.ends_with("</svg></div></div>"));
        reject_stub(&html);
        let css = include_str!("cronus_ui_css/sparkline.css");
        assert!(css.contains("[data-slot=\"sparkline\"].w-full { width: 100%; }"));
        assert!(css.contains("[data-slot=\"card-content\"].cui-stat-card { display: flex; flex-direction: column; gap: 0.75rem; padding-top: 1.5rem; }"));
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
        let css = include_str!("cronus_ui_css/sparkline.css");
        assert!(css.contains("[data-slot=\"sparkline\"]"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(css.contains("var(--cronus-success)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
