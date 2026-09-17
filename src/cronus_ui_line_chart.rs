//! Dedicated LineChart renderer.
//!
//! Default (recharts `LineChart`, docs "Default"): `<div data-slot="line-chart"
//! role="img" aria-label>` › `<div data-slot="chart"><svg viewBox="0 0 432 256">`
//! dashed grid rows, one monotone line per series (chart-1..N, width 2) and
//! x tick labels (minTickGap 24, `preserveEnd`). Lines draw in with a
//! `pathLength="1"` dash (recharts stroke animation, 1100ms ease-out).
//!
//! Motion (`style:line-chart+motion`, `@cronus-ui/ui/charts`): the visx
//! `LineChart` — `<div data-slot="line-chart" class="v-motion">`, `2 / 1`
//! SVG with 40px margins, `Grid horizontal`, one `Line` per series
//! (`curveNatural`, round stroke 2.5, edge fade gradient 0/15/85/100 unless
//! `fade:false`), `XAxis` labels and the shared `ChartRevealClip`.
//!
//! Data: `series:"users,pageviews"` + `item "2024-01-01" users:1350 pageviews:2875`.
//! Props: `curve:natural|monotone|catmull|linear` (natural), `fade` (true),
//! `stroke-width` (2.5).

use crate::cronus_ui_chart::{
    chart_color, chart_data, container, curve_path, d3_ticks, data_ticks, fade_gradient, line_path,
    motion_grid, motion_x_labels, nice_domain, num, point_xs, reveal_clip, row_label, row_times,
    time_series_domain, time_xs, value_grid, x_tick_labels, ChartData, Frame, DEMO_VALUES, M_W,
};
use crate::cronus_ui_kit::{attr, attr_num, choice, instance_id, label_of, truthy};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let data = chart_data(comp, &["Jan", "Feb", "Mar"], &DEMO_VALUES);
    if choice(comp, "variant", &["motion"]) == Some("motion") {
        return render_motion(comp, &label, &data);
    }
    let (lo, hi, ticks) = nice_domain(0.0, data.max().max(0.0));
    let xs = point_xs(data.len());
    let mut body = value_grid(&ticks, lo, hi);
    for (i, s) in data.series.iter().enumerate() {
        body.push_str(
            &line_path(&xs, &s.values, lo, hi, &chart_color(i))
                .replace("<path d=", "<path class=\"draw\" pathLength=\"1\" d="),
        );
    }
    body.push_str(&x_tick_labels(&data.labels, &xs, 24.0));
    format!(
        "<div data-slot=\"line-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

/// visx `LineChart` at the 432px reference width (aspect 2 / 1).
pub(crate) fn render_motion(comp: &ComponentNode, label: &str, data: &ChartData) -> String {
    crate::cronus_ui_chart::note_motion();
    let frame = Frame::aspect(M_W, 2.0);
    let (iw, ih) = (frame.iw(), frame.ih());
    let times = row_times(&data.labels);
    let xs = time_xs(&times, 0.0, iw);
    let (lo, hi) = time_series_domain(data.min().min(0.0), data.max());
    let y = |v: f64| ih - (v - lo) / (hi - lo) * ih;
    let curve = attr(comp, "curve").unwrap_or("natural");
    let fade = attr(comp, "fade").map(truthy).unwrap_or(true);
    let width = attr_num::<f64>(comp, "stroke-width").unwrap_or(2.5);
    let clip = instance_id(comp, "reveal");
    let grid_id = instance_id(comp, "grid");
    let mut defs = String::new();
    let mut series = String::new();
    for (i, s) in data.series.iter().enumerate() {
        let color = chart_color(i);
        let pts: Vec<(f64, f64)> = xs.iter().zip(&s.values).map(|(x, v)| (*x, y(*v))).collect();
        if pts.is_empty() {
            continue;
        }
        let stroke = if fade {
            let gid = instance_id(comp, &format!("line-gradient-{}", s.key));
            defs.push_str(&fade_gradient(&gid, iw, &color, true, true));
            format!("url(#{gid})")
        } else {
            color
        };
        series.push_str(&format!(
            "<g class=\"series\"><path d=\"{}\" fill=\"none\" stroke=\"{stroke}\" stroke-linecap=\"round\" stroke-width=\"{}\"></path></g>",
            curve_path(curve, &pts),
            num(width)
        ));
    }
    defs.push_str(&reveal_clip(&clip, iw, ih + 20.0, 0.0));
    let ticks_y: Vec<f64> = d3_ticks(lo, hi, 5.0).iter().map(|t| y(*t)).collect();
    let labels: Vec<String> = data.labels.iter().map(|l| row_label(l)).collect();
    let axis = motion_x_labels(&data_ticks(&xs, &labels, 5), ih, frame.bottom);
    format!(
        "<div data-slot=\"line-chart\" class=\"v-motion\" role=\"img\" aria-label=\"{label}\">{}<defs>{defs}</defs>{}{}{axis}<g clip-path=\"url(#{clip})\">{series}</g></g></svg></div>",
        frame.open(),
        frame.plot_open(),
        motion_grid(&grid_id, &ticks_y, iw, ih, &[]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn fixture() -> ComponentNode {
        let mut c = stub("line-chart", "Sessions");
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

    fn motion_fixture() -> ComponentNode {
        let mut c = stub("line-chart", "Users and pageviews");
        c.style = Some("line-chart+motion".into());
        c.props.insert("series".into(), "users,pageviews".into());
        for (d, u, p) in [
            ("2024-01-01", "1350", "2875"),
            ("2024-01-02", "1233", "2700"),
            ("2024-01-03", "1425", "3025"),
        ] {
            let mut it = ComponentItemNode {
                item_type: "item".into(),
                text: d.into(),
                link: None,
                tone: None,
                config: Default::default(),
            };
            it.config.insert("users".into(), u.into());
            it.config.insert("pageviews".into(), p.into());
            c.items.push(it);
        }
        c
    }

    #[test]
    fn fixture_nests_chart_container_with_monotone_line() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"line-chart\" role=\"img\" aria-label=\"Sessions\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\">"));
        assert!(html.contains("<path class=\"draw\" pathLength=\"1\" d=\"M8,117C77.3333,62.5,146.6667,8,216,8C285.3333,8,354.6667,35.25,424,62.5\" fill=\"none\" stroke=\"var(--cronus-chart-1)\" stroke-width=\"2\"></path>"));
        assert!(!html.contains(">Jan</tspan>"));
        assert!(html.contains(">Feb</tspan>"));
        assert!(html.contains(">Mar</tspan>"));
        assert!(!html.contains("polyline"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn series_prop_draws_one_line_per_key() {
        let mut c = motion_fixture();
        c.style = Some("line-chart".into());
        let html = render(&c);
        assert_eq!(html.matches("class=\"draw\"").count(), 2);
        assert!(html.contains("stroke=\"var(--cronus-chart-2)\""));
    }

    #[test]
    fn motion_variant_draws_natural_curves_with_edge_fade() {
        let html = render(&motion_fixture());
        assert!(html.starts_with("<div data-slot=\"line-chart\" class=\"v-motion\" role=\"img\" aria-label=\"Users and pageviews\"><svg viewBox=\"0 0 432 216\" aria-hidden=\"true\">"));
        assert!(html.contains("<linearGradient gradientUnits=\"userSpaceOnUse\" id=\"cui-line-chart-line-gradient-users\" x1=\"0\" x2=\"352\" y1=\"0\" y2=\"0\"><stop offset=\"0%\" stop-color=\"var(--cronus-chart-1)\" stop-opacity=\"0\"></stop>"));
        assert!(html.contains("stroke=\"url(#cui-line-chart-line-gradient-pageviews)\" stroke-linecap=\"round\" stroke-width=\"2.5\""));
        // Natural spline: three points → two cubic segments starting at x=0.
        assert!(html.contains("<path d=\"M0,"));
        assert_eq!(html.matches("<path d=\"M0,").count(), 2);
        assert!(html.contains("<g clip-path=\"url(#cui-line-chart-reveal)\">"));
        assert!(html.contains("<text x=\"176\" y=\"156\" text-anchor=\"middle\" dominant-baseline=\"central\" fill=\"var(--cronus-fg-tertiary)\">Jan 2</text>"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn motion_fade_false_uses_solid_stroke() {
        let mut c = motion_fixture();
        c.props.insert("fade".into(), "false".into());
        let html = render(&c);
        assert!(!html.contains("line-gradient"));
        assert!(html.contains("stroke=\"var(--cronus-chart-1)\" stroke-linecap=\"round\""));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"line-chart\"] {\n  display: block; width: 100%; height: 16rem;\n}"
        ));
        assert!(css.contains(
            "[data-slot=\"line-chart\"].v-motion {\n  height: auto; aspect-ratio: 2 / 1;\n}"
        ));
    }
}
