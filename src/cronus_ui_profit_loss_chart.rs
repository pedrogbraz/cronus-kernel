//! Dedicated ProfitLossChart renderer.
//!
//! Default (recharts `LineChart`, docs "Default"): `<div
//! data-slot="profit-loss-chart" role="img" aria-label>` › `<div
//! data-slot="chart"><svg viewBox="0 0 432 256">` dashed grid rows over the
//! signed nice domain, a `--cronus-border` reference line at zero, the
//! monotone `--cronus-success` line (width 2, dash draw) and x tick labels.
//!
//! Motion (`style:profit-loss-chart+motion`, `@cronus-ui/ui/charts`): the docs
//! composition — `<div data-slot="profit-loss-chart" class="v-motion">` with
//! the centred `ProfitLossLegend` (Profit / Loss markers) above a visx
//! `LineChart` (`data-slot="line-chart"`, `2 / 1`, 40px margins): `Grid
//! horizontal` with the zero row highlighted, and `ProfitLossLine` — the
//! series split at every zero crossing into linear segments stroked
//! `--cronus-success` above and `--cronus-error` below (width 2.5, round
//! caps) behind the shared `ChartRevealClip`. Hovering a legend entry dims
//! the other sign to 0.25.
//!
//! Data: `item "Jan" value:420` (ISO texts → time axis).

use crate::cronus_ui_chart::{
    chart_data, container, d3_ticks, data_ticks, legend_html, line_path, linear_path, motion_grid,
    motion_x_labels, nice_domain, num, point_xs, reveal_clip, row_label, row_times,
    time_series_domain, time_xs, value_grid, x_tick_labels, y_of, ChartData, Frame, LegendRow, M_W,
    PLOT_L, PLOT_R,
};
use crate::cronus_ui_kit::{attr, choice, esc, instance_id, label_of};
use crate::parser::ComponentNode;

pub const DEMO_PNL: [f64; 3] = [-4.0, 8.0, 2.0];

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let data = chart_data(comp, &["Jan", "Feb", "Mar"], &DEMO_PNL);
    if choice(comp, "variant", &["motion"]) == Some("motion") {
        return render_motion(comp, &label, &data);
    }
    let values = &data.series[0].values;
    let (lo, hi, ticks) = nice_domain(data.min().min(0.0), data.max().max(0.0));
    let xs = point_xs(values.len());
    let zero = y_of(0.0, lo, hi);
    let body = format!(
        "{}<line x1=\"{}\" y1=\"{z}\" x2=\"{}\" y2=\"{z}\" stroke=\"var(--cronus-border)\"></line>{}{}",
        value_grid(&ticks, lo, hi),
        num(PLOT_L),
        num(PLOT_R),
        line_path(&xs, values, lo, hi, "var(--cronus-success)")
            .replace("<path d=", "<path class=\"draw\" pathLength=\"1\" d="),
        x_tick_labels(&data.labels, &xs, 5.0),
        z = num(zero),
    );
    format!(
        "<div data-slot=\"profit-loss-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

/// `splitProfitLossSegments`: `(points, positive)` runs split at zero crossings.
pub fn split_segments(pts: &[(f64, f64)]) -> Vec<(Vec<(f64, f64)>, bool)> {
    if pts.is_empty() {
        return Vec::new();
    }
    let mut sign = pts
        .iter()
        .find(|p| p.1 != 0.0)
        .map(|p| p.1 > 0.0)
        .unwrap_or(true);
    let mut out = Vec::new();
    let mut current = vec![pts[0]];
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        if a.1 != 0.0 && b.1 != 0.0 && (a.1 > 0.0) != (b.1 > 0.0) {
            let t = a.1 / (a.1 - b.1);
            let cross = (a.0 + t * (b.0 - a.0), 0.0);
            current.push(cross);
            out.push((current, sign));
            current = vec![cross, b];
            sign = b.1 > 0.0;
            continue;
        }
        current.push(b);
        if b.1 != 0.0 {
            sign = b.1 > 0.0;
        }
    }
    if !current.is_empty() {
        out.push((current, sign));
    }
    out
}

/// Legend + visx `LineChart` with `ProfitLossLine`.
fn render_motion(comp: &ComponentNode, label: &str, data: &ChartData) -> String {
    crate::cronus_ui_chart::note_motion();
    let frame = Frame::aspect(M_W, 2.0);
    let (iw, ih) = (frame.iw(), frame.ih());
    let values = &data.series[0].values;
    let times = row_times(&data.labels);
    let xs = time_xs(&times, 0.0, iw);
    let (lo, hi) = time_series_domain(data.min(), data.max());
    let y = |v: f64| ih - (v - lo) / (hi - lo) * ih;
    let clip = instance_id(comp, "reveal");
    let grid_id = instance_id(comp, "grid");
    let raw: Vec<(f64, f64)> = xs.iter().zip(values).map(|(x, v)| (*x, *v)).collect();
    let mut segments = String::new();
    for (seg, positive) in split_segments(&raw) {
        let pts: Vec<(f64, f64)> = seg.iter().map(|p| (p.0, y(p.1))).collect();
        segments.push_str(&format!(
            "<g class=\"{}\"><path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2.5\"></path></g>",
            if positive { "pos" } else { "neg" },
            linear_path(&pts),
            if positive {
                "var(--cronus-success)"
            } else {
                "var(--cronus-error)"
            }
        ));
    }
    let ticks_y: Vec<f64> = d3_ticks(lo, hi, 5.0).iter().map(|t| y(*t)).collect();
    let labels: Vec<String> = data.labels.iter().map(|l| row_label(l)).collect();
    let axis = motion_x_labels(&data_ticks(&xs, &labels, 5), ih, frame.bottom);
    let legend = legend_html(
        None,
        &[
            LegendRow {
                label: "Profit".into(),
                value: None,
                color: "success".into(),
                progress: None,
                grow: false,
            },
            LegendRow {
                label: "Loss".into(),
                value: None,
                color: "error".into(),
                progress: None,
                grow: false,
            },
        ],
        "row",
    )
    .replace("class=\"legend-item\"", "class=\"legend-item compact\"");
    let align = attr(comp, "legend").unwrap_or("start");
    format!(
        "<div data-slot=\"profit-loss-chart\" class=\"v-motion\" role=\"img\" aria-label=\"{}\"><div class=\"pl-legend {align}\">{legend}</div><div data-slot=\"line-chart\" class=\"v-motion\">{}<defs>{}</defs>{}{}{axis}<g clip-path=\"url(#{clip})\"><g data-slot=\"profit-loss-chart\">{segments}</g></g></g></svg></div></div>",
        esc(label),
        frame.open(),
        reveal_clip(&clip, iw, ih + 20.0, 0.0),
        frame.plot_open(),
        motion_grid(&grid_id, &ticks_y, iw, ih, &[y(0.0)]),
        align = esc(align),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn fixture() -> ComponentNode {
        let mut c = stub("profit-loss-chart", "P/L");
        for t in ["Jan", "Feb", "Mar"] {
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

    fn pnl(style: &str, rows: &[(&str, &str)]) -> ComponentNode {
        let mut c = stub("profit-loss-chart", "Profit and loss");
        c.style = Some(style.into());
        for (l, v) in rows {
            let mut it = ComponentItemNode {
                item_type: "item".into(),
                text: (*l).into(),
                link: None,
                tone: None,
                config: Default::default(),
            };
            it.config.insert("value".into(), (*v).into());
            c.items.push(it);
        }
        c
    }

    #[test]
    fn fixture_draws_zero_line_and_signed_series() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"profit-loss-chart\" role=\"img\" aria-label=\"P/L\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\">"));
        assert!(html.contains("<line x1=\"8\" y1=\"171.5\" x2=\"424\" y2=\"171.5\" stroke=\"var(--cronus-border)\"></line>"));
        assert!(html.contains("<path class=\"draw\" pathLength=\"1\" d=\"M8,226C77.3333,144.25,146.6667,62.5,216,62.5C285.3333,62.5,354.6667,103.375,424,144.25\" fill=\"none\" stroke=\"var(--cronus-success)\""));
        assert!(!html.contains(">Jan</tspan>"));
        assert!(html.contains(">Feb</tspan>"));
        assert!(html.contains(">Mar</tspan>"));
        assert!(!html.contains("polyline"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn item_values_drive_the_default_line() {
        let html = render(&pnl(
            "profit-loss-chart",
            &[("Jan", "420"), ("Feb", "-180"), ("Mar", "260")],
        ));
        // Domain -200..600 (nice): zero at 226 - 200/800*218 = 171.5.
        assert!(
            html.contains("y1=\"171.5\" x2=\"424\" y2=\"171.5\" stroke=\"var(--cronus-border)\"")
        );
        assert!(html.contains("<path class=\"draw\" pathLength=\"1\" d=\"M8,57.05C"));
    }

    #[test]
    fn segments_split_at_zero_crossings() {
        let segs = split_segments(&[
            (0.0, 420.0),
            (1.0, 180.0),
            (2.0, -240.0),
            (3.0, -90.0),
            (4.0, 310.0),
        ]);
        assert_eq!(segs.len(), 3);
        assert!(segs[0].1);
        assert_eq!(
            segs[0].0.last().map(|p| (num(p.0), p.1)),
            Some(("1.4286".into(), 0.0))
        );
        assert!(!segs[1].1);
        assert_eq!(segs[1].0.len(), 4);
        assert!(segs[2].1);
    }

    #[test]
    fn motion_variant_renders_legend_grid_highlight_and_signed_segments() {
        let mut c = pnl(
            "profit-loss-chart+motion",
            &[
                ("2024-01-01", "420"),
                ("2024-01-05", "180"),
                ("2024-01-10", "-240"),
                ("2024-01-15", "-90"),
                ("2024-01-20", "310"),
                ("2024-01-25", "520"),
            ],
        );
        c.props.insert("legend".into(), "center".into());
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"profit-loss-chart\" class=\"v-motion\" role=\"img\" aria-label=\"Profit and loss\"><div class=\"pl-legend center\"><div class=\"legend-container row\"><div class=\"legend-item compact\"><div class=\"legend-marker success\"></div><span class=\"legend-label\">Profit</span></div><div class=\"legend-item compact\"><div class=\"legend-marker error\"></div><span class=\"legend-label\">Loss</span></div></div></div><div data-slot=\"line-chart\" class=\"v-motion\"><svg viewBox=\"0 0 432 216\""));
        // Domain -300..600: zero row highlighted at y = 136 - 300/900*136.
        assert!(html.contains("<g class=\"chart-grid-highlight-rows\"><line stroke=\"var(--cronus-fg-muted)\" stroke-dasharray=\"0\" stroke-opacity=\"1\" stroke-width=\"1\" x1=\"0\" x2=\"352\" y1=\"90.6667\" y2=\"90.6667\"></line></g>"));
        assert!(html.contains("<g data-slot=\"profit-loss-chart\"><g class=\"pos\"><path d=\"M0,27.2L58.6667,63.4667L90.0952,90.6667\" fill=\"none\" stroke=\"var(--cronus-success)\" stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2.5\"></path></g><g class=\"neg\"><path d=\"M90.0952,90.6667L132,126.9333L205.3333,104.2667L221.8333,90.6667\""));
        assert_eq!(html.matches("<g class=\"pos\">").count(), 2);
        assert!(html.contains("<g clip-path=\"url(#cui-profit-loss-chart-reveal)\">"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"profit-loss-chart\"] {\n  display: block; width: 100%; height: 16rem;\n}"
        ));
        assert!(css.contains("[data-slot=\"profit-loss-chart\"].v-motion .pl-legend.center {"));
    }
}
