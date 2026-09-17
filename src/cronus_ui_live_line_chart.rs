//! Dedicated LiveLineChart renderer.
//!
//! Default (recharts `LineChart`, docs "Default"): `<div
//! data-slot="live-line-chart" role="img" aria-label>` › `<div
//! data-slot="chart"><svg viewBox="0 0 432 256">` dashed grid rows, the
//! monotone chart-1 line of `data:"162,148,…"` (no entrance animation —
//! React's `isAnimationActive={false}`) and one tick label per point.
//!
//! Motion (`style:live-line-chart+motion`, `@cronus-ui/ui/charts`): the visx
//! `LiveLineChart` — `<div data-slot="live-line-chart" class="v-motion">` with
//! the docs margins (16 / 88 / 40 / 56) and `height` 260: a `window` (30s)
//! time axis ending at "now", the seed points `interval` (600ms) apart
//! trailing the live tip, the `LiveLine` gradient area + stroke (masked to
//! fade in from the left), the dashed current-value line, the pulsing live
//! dot (chart-1 up / chart-5 down momentum) with its `format:currency` badge,
//! `LiveYAxis` labels on the left (mono) and `LiveXAxis` times from `start`.
//! The series group scrolls left one slot per `interval` and loops.

use crate::cronus_ui_chart::{
    chart_data, container, d3_nice, hms, item_labels, line_path, max_of, monotone_path,
    nice_domain, num, parse_date, point_xs, value_grid, x_tick_labels, ChartData, Frame,
    DEMO_VALUES, M_W,
};
use crate::cronus_ui_kit::{attr, attr_num, choice, esc, instance_id, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    if choice(comp, "variant", &["motion"]) == Some("motion") {
        let data = chart_data(comp, &["1", "2", "3"], &DEMO_VALUES);
        return render_motion(comp, &label, &data);
    }
    let mut ticks_text = item_labels(comp);
    let data = chart_data(comp, &["0", "1", "2"], &DEMO_VALUES);
    let values = data.series[0].values.clone();
    let numeric_texts = ticks_text.iter().all(|t| t.parse::<f64>().is_ok());
    if ticks_text.len() != values.len() || numeric_texts || attr(comp, "data").is_some() {
        ticks_text = (1..=values.len()).map(|i| i.to_string()).collect();
    }
    let (lo, hi, ticks) = nice_domain(0.0, max_of(&values));
    let xs = point_xs(values.len());
    let body = format!(
        "{}{}{}",
        value_grid(&ticks, lo, hi),
        line_path(&xs, &values, lo, hi, "var(--cronus-chart-1)"),
        x_tick_labels(&ticks_text, &xs, 5.0)
    );
    format!(
        "<div data-slot=\"live-line-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

/// `LiveYAxis` `pickNiceInterval`: the nice step keeping labels ~`min_gap` apart.
fn nice_interval(range: f64, height: f64, min_gap: f64) -> f64 {
    if range <= 0.0 || height <= 0.0 {
        return 1.0;
    }
    let px_per_unit = height / range;
    let sets: [[f64; 3]; 3] = [[2.0, 2.5, 2.0], [2.0, 2.0, 2.5], [2.5, 2.0, 2.0]];
    let mut best = f64::INFINITY;
    for divs in sets {
        let mut span = 10f64.powf(range.log10().ceil());
        let mut i = 0;
        let mut d = divs[i % 3];
        while (span / d) * px_per_unit >= min_gap {
            span /= d;
            i += 1;
            d = divs[i % 3];
        }
        if span < best {
            best = span;
        }
    }
    if best.is_finite() {
        best
    } else {
        range / 5.0
    }
}

fn format_value(v: f64, format: Option<&str>) -> String {
    match format {
        Some("currency") => crate::cronus_ui_chart::usd(v, 2),
        Some("int") => crate::cronus_ui_chart::int_fmt(v),
        _ => format!("{v:.2}"),
    }
}

/// visx `LiveLineChart` with the docs margins at the 432px reference width.
fn render_motion(comp: &ComponentNode, label: &str, data: &ChartData) -> String {
    crate::cronus_ui_chart::note_motion();
    let height = attr_num::<f64>(comp, "height").unwrap_or(260.0);
    let frame = Frame::new(M_W, height).margins(16.0, 88.0, 40.0, 56.0);
    let (iw, ih) = (frame.iw(), frame.ih());
    let window = attr_num::<f64>(comp, "window").unwrap_or(30.0) * 1000.0;
    let interval = attr_num::<f64>(comp, "interval").unwrap_or(1000.0);
    let format = attr(comp, "format");
    let values = &data.series[0].values;
    let n = values.len();
    // Window [now - window, now]; seed point i sits (n - i) intervals before now.
    let x_of_ms = |ms_before_now: f64| iw - ms_before_now / window * iw;
    let mut pts: Vec<(f64, f64)> = values
        .iter()
        .enumerate()
        .map(|(i, v)| (x_of_ms((n - i) as f64 * interval), *v))
        .collect();
    let last = values.last().copied().unwrap_or(0.0);
    let unit = window / 4.0;
    pts.push((iw, last));
    pts.push((iw + unit / window * iw, last));
    // y domain: 15% padding, then `nice`.
    let min = values.iter().copied().fold(last, f64::min);
    let max = values.iter().copied().fold(last, f64::max);
    let pad = {
        let p = (max - min) * 0.15;
        if p == 0.0 {
            10.0
        } else {
            p
        }
    };
    let (lo, hi) = d3_nice(min - pad, max + pad, 10.0);
    let y = |v: f64| ih - (v - lo) / (hi - lo) * ih;
    let curve: Vec<(f64, f64)> = pts.iter().map(|p| (p.0, y(p.1))).collect();
    let line = monotone_path(&curve);
    let base = ih;
    let area = format!(
        "{line}L{},{b}L{},{b}Z",
        num(curve[curve.len() - 1].0),
        num(curve[0].0),
        b = num(base)
    );
    // Momentum over the last five points (12% of the range).
    let tail = &values[n.saturating_sub(5)..];
    let delta = tail.last().copied().unwrap_or(0.0) - tail.first().copied().unwrap_or(0.0);
    let range = max - min;
    let dot_color = if range > 0.0 && delta > range * 0.12 {
        "var(--cronus-chart-1)"
    } else if range > 0.0 && delta < -range * 0.12 {
        "var(--cronus-chart-5)"
    } else {
        "var(--cronus-chart-1)"
    };
    let (dot_x, dot_y) = (iw, y(last));
    let gid = instance_id(comp, "live-line-grad");
    let aid = instance_id(comp, "live-area-grad");
    let fid = instance_id(comp, "live-fade");
    let mid = instance_id(comp, "live-fade-mask");
    let defs = format!(
        "<linearGradient id=\"{gid}\" x1=\"0\" x2=\"0\" y1=\"0\" y2=\"1\"><stop offset=\"0%\" stop-color=\"var(--cronus-chart-1)\" stop-opacity=\"1\"></stop><stop offset=\"100%\" stop-color=\"var(--cronus-chart-1)\" stop-opacity=\"0.6\"></stop></linearGradient><linearGradient id=\"{aid}\" x1=\"0\" x2=\"0\" y1=\"0\" y2=\"1\"><stop offset=\"0%\" stop-color=\"var(--cronus-chart-1)\" stop-opacity=\"0.1\"></stop><stop offset=\"100%\" stop-color=\"var(--cronus-chart-1)\" stop-opacity=\"0\"></stop></linearGradient><linearGradient id=\"{fid}\" x1=\"0\" x2=\"1\" y1=\"0\" y2=\"0\"><stop offset=\"0%\" stop-color=\"white\" stop-opacity=\"0\"></stop><stop offset=\"4%\" stop-color=\"white\" stop-opacity=\"1\"></stop><stop offset=\"100%\" stop-color=\"white\" stop-opacity=\"1\"></stop></linearGradient><mask id=\"{mid}\"><rect fill=\"url(#{fid})\" height=\"{}\" width=\"{}\" x=\"0\" y=\"-20\"></rect></mask>",
        num(ih + 40.0),
        num(iw)
    );
    let slot = interval / window * iw;
    let series = format!(
        "<g mask=\"url(#{mid})\"><g class=\"live-series s{}\"><path d=\"{area}\" fill=\"url(#{aid})\" stroke-width=\"0\"></path><path d=\"{line}\" fill=\"none\" stroke=\"url(#{gid})\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\"></path></g></g>",
        num(slot).replace('.', "-")
    );
    let badge_text = format_value(last, format);
    let badge_w = badge_text.chars().count() as f64 * 7.5 + 16.0;
    let live = format!(
        "<line opacity=\"0.25\" stroke=\"var(--cronus-chart-1)\" stroke-dasharray=\"4,4\" stroke-width=\"1\" x1=\"0\" x2=\"{iw}\" y1=\"{dy}\" y2=\"{dy}\"></line><g class=\"live-tip\"><g><circle class=\"pulse\" cx=\"{dx}\" cy=\"{dy}\" fill=\"none\" opacity=\"0.4\" r=\"8\" stroke=\"{dot_color}\" stroke-width=\"1.5\"></circle><circle cx=\"{dx}\" cy=\"{dy}\" fill=\"{dot_color}\" opacity=\"0.1\" r=\"6\"></circle><circle cx=\"{dx}\" cy=\"{dy}\" fill=\"{dot_color}\" r=\"4\" stroke=\"var(--cronus-surface-base)\" stroke-width=\"2\"></circle></g><g transform=\"translate({bx},{dy})\"><rect fill=\"var(--cronus-surface-overlay)\" height=\"24\" opacity=\"0.95\" rx=\"6\" width=\"{bw}\" x=\"0\" y=\"-12\"></rect><text class=\"badge\" fill=\"var(--cronus-fg)\" font-size=\"11\" font-weight=\"500\" x=\"8\" y=\"4\">{bt}</text></g></g>",
        iw = num(iw),
        dx = num(dot_x),
        dy = num(dot_y),
        bx = num(dot_x + 12.0),
        bw = num(badge_w),
        bt = esc(&badge_text),
    );
    // LiveYAxis: nice interval, labels right-aligned 8px inside the left margin.
    let step = nice_interval(hi - lo, ih, 36.0);
    let mut y_axis = String::from("<g class=\"y-axis\">");
    let first = ((lo - step * 0.5) / step).ceil() * step;
    let mut v = first;
    while v <= hi + step * 0.5 {
        let ty = y(v);
        if ty >= -10.0 && ty <= ih + 10.0 {
            let edge = ty.min(ih - ty);
            let alpha = if edge >= 28.0 {
                1.0
            } else if edge <= 0.0 {
                0.0
            } else {
                edge / 28.0
            };
            y_axis.push_str(&format!(
                "<text x=\"-8\" y=\"{}\" text-anchor=\"end\" dominant-baseline=\"central\" fill=\"var(--cronus-fg-tertiary)\" opacity=\"{}\">{}</text>",
                num(ty + 8.0),
                num(alpha),
                esc(&format_value(v, format))
            ));
        }
        v += step;
    }
    y_axis.push_str("</g>");
    // LiveXAxis: five wall-clock labels from `start` across the window.
    let start_ms = attr(comp, "start")
        .and_then(|s| parse_date(&format!("1970-01-01T{s}")))
        .unwrap_or(0.0);
    let mut x_axis = String::from("<g class=\"x-axis\">");
    for i in 0..5 {
        let t = start_ms + i as f64 * unit;
        x_axis.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"central\" fill=\"var(--cronus-fg-tertiary)\">{}</text>",
            num(i as f64 / 4.0 * iw),
            num(ih + frame.bottom - 20.0),
            hms(t)
        ));
    }
    x_axis.push_str("</g>");
    format!(
        "<div data-slot=\"live-line-chart\" class=\"v-motion\" role=\"img\" aria-label=\"{}\">{}<defs>{defs}</defs>{}{y_axis}{x_axis}{series}{live}</g></svg></div>",
        esc(label),
        frame.open(),
        frame.plot_open(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn fixture() -> ComponentNode {
        let mut c = stub("live-line-chart", "Live");
        for t in ["0", "1", "2"] {
            c.items.push(ComponentItemNode {
                item_type: "text".into(),
                text: t.into(),
                link: None,
                tone: None,
                config: Default::default(),
            });
        }
        c
    }

    #[test]
    fn fixture_renders_first_paint_series_and_ticks() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"live-line-chart\" role=\"img\" aria-label=\"Live\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\">"));
        // Numeric texts 0, 1, 2 are the series (domain 0..2); ticks 1..3 label the points.
        assert!(html.contains("d=\"M8,226C"));
        assert!(html.contains(",216,117C"));
        assert!(html.contains(",424,8\""));
        assert!(html.contains("<tspan x=\"8\" dy=\"0.71em\">1</tspan>"));
        assert!(html.contains("<tspan x=\"424\" dy=\"0.71em\">3</tspan>"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn data_prop_drives_the_seed_series() {
        let mut c = stub("live-line-chart", "Requests");
        c.props.insert("data".into(), "162,148,171".into());
        let html = render(&c);
        assert!(html.contains("<tspan x=\"216\" dy=\"0.71em\">2</tspan>"));
        // Domain 0..180 (max 171 → nice): 162 → y = 226 - 162/180*218.
        assert!(html.contains("d=\"M8,29.8C"));
        assert!(!html.contains("class=\"draw\""));
    }

    #[test]
    fn motion_variant_renders_live_tip_axes_and_scrolling_series() {
        let mut c = stub("live-line-chart", "Quote");
        c.style = Some("live-line-chart+motion".into());
        c.props
            .insert("data".into(), "142.5,142.9,143.4,143.1".into());
        c.props.insert("format".into(), "currency".into());
        c.props.insert("start".into(), "12:00:00".into());
        c.props.insert("interval".into(), "600".into());
        c.props.insert("window".into(), "30".into());
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"live-line-chart\" class=\"v-motion\" role=\"img\" aria-label=\"Quote\"><svg viewBox=\"0 0 432 260\" aria-hidden=\"true\"><defs>"));
        assert!(html.contains("<g transform=\"translate(56,16)\"><rect fill=\"transparent\" height=\"204\" width=\"288\" x=\"0\" y=\"0\"></rect>"));
        // Series scrolls one 600ms slot (5.76px) per interval; the tip sits on the right edge.
        assert!(html.contains("<g class=\"live-series s5-76\">"));
        assert!(html.contains("<circle class=\"pulse\" cx=\"288\""));
        assert!(html.contains("<text class=\"badge\" fill=\"var(--cronus-fg)\" font-size=\"11\" font-weight=\"500\" x=\"8\" y=\"4\">$143.10</text>"));
        assert!(html.contains("<rect fill=\"var(--cronus-surface-overlay)\" height=\"24\" opacity=\"0.95\" rx=\"6\" width=\"68.5\""));
        // Time labels every 7.5s from 12:00:00; dollar labels on the left axis.
        assert!(html.contains(">12:00:00</text>"));
        assert!(html.contains(">12:00:30</text>"));
        assert!(html.contains("text-anchor=\"end\" dominant-baseline=\"central\" fill=\"var(--cronus-fg-tertiary)\" opacity=\"1\">$143.00</text>"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"live-line-chart\"] {\n  display: block; width: 100%; height: 16rem;\n}"
        ));
        assert!(css.contains("[data-slot=\"live-line-chart\"].v-motion .pulse {"));
        assert!(css.contains("@keyframes cronus-live-shift"));
    }
}
