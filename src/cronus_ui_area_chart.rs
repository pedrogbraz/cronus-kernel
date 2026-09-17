//! Dedicated AreaChart renderer.
//!
//! Default (recharts `AreaChart`, docs "Default"): `<div data-slot="area-chart"
//! role="img" aria-label>` › `<div data-slot="chart"><svg viewBox="0 0 432 256">`
//! dashed grid rows, one gradient + monotone area per series (chart-1..N,
//! stop 0.4 → 0, `fill-opacity` 0.6, stroke 2), x tick labels (minTickGap 24,
//! `preserveEnd`). The area is revealed by a clip rect (recharts
//! `AreaRevealShape`, 1100ms ease-out).
//!
//! Motion (`style:area-chart+motion`, `@cronus-ui/ui/charts`): the visx
//! `AreaChart` — `<div data-slot="area-chart" class="v-motion">` with a
//! `2 / 1` SVG, 40px margins, `Grid horizontal` (dashed rows behind a
//! horizontal fade mask), `Area` (gradient `fill-opacity` → 0, `curveMonotoneX`,
//! round stroke 2, optional edge fade mask), `XAxis` labels and the shared
//! `ChartRevealClip` whose rect grows 0 → width in 1100ms
//! `cubic-bezier(0.85, 0, 0.15, 1)`.
//!
//! Data: `series:"revenue,costs"` + `item "Jul 29" revenue:16200 costs:11500`
//! (ISO item texts such as `2024-01-01` become a time axis with `Jan 1`
//! labels). Props: `fill-opacity` (0.4), `fade:true` (`fadeEdges`),
//! `curve:natural|catmull|linear`.

use crate::cronus_ui_chart::{
    area_paths, chart_color, chart_data, container, curve_path, d3_ticks, data_ticks,
    fade_gradient, motion_grid, motion_x_labels, nice_domain, num, point_xs, reveal_clip,
    row_label, row_times, time_series_domain, time_xs, value_grid, x_tick_labels, ChartData, Frame,
    DEMO_VALUES, M_W,
};
use crate::cronus_ui_kit::{attr, attr_num, choice, flag, instance_id, label_of};
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
    let clip = instance_id(comp, "area-clip");
    body.push_str(&format!(
        "<defs><clipPath id=\"{clip}\"><rect class=\"reveal\" x=\"8\" y=\"0\" width=\"416\" height=\"256\"></rect></clipPath></defs><g clip-path=\"url(#{clip})\">"
    ));
    for (i, s) in data.series.iter().enumerate() {
        let gid = instance_id(comp, &format!("area-fill-{}", s.key));
        body.push_str(&area_paths(
            &xs,
            &s.values,
            lo,
            hi,
            &chart_color(i),
            &gid,
            "0.4",
        ));
    }
    body.push_str("</g>");
    body.push_str(&x_tick_labels(&data.labels, &xs, 24.0));
    format!(
        "<div data-slot=\"area-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

/// visx `AreaChart` at the 432px reference width (aspect 2 / 1).
fn render_motion(comp: &ComponentNode, label: &str, data: &ChartData) -> String {
    crate::cronus_ui_chart::note_motion();
    let frame = Frame::aspect(M_W, 2.0);
    let (iw, ih) = (frame.iw(), frame.ih());
    let times = row_times(&data.labels);
    let xs = time_xs(&times, 0.0, iw);
    let (lo, hi) = time_series_domain(data.min().min(0.0), data.max());
    let y = |v: f64| ih - (v - lo) / (hi - lo) * ih;
    let fill_opacity = attr_num::<f64>(comp, "fill-opacity").unwrap_or(0.4);
    let fade = flag(comp, "fade");
    let curve = attr(comp, "curve").unwrap_or("monotone");
    let clip = instance_id(comp, "reveal");
    let grid_id = instance_id(comp, "grid");
    let mut defs = String::new();
    let mut series = String::new();
    for (i, s) in data.series.iter().enumerate() {
        let color = chart_color(i);
        let gid = instance_id(comp, &format!("area-gradient-{}", s.key));
        defs.push_str(&format!(
            "<linearGradient id=\"{gid}\" x1=\"0%\" x2=\"0%\" y1=\"0%\" y2=\"100%\"><stop offset=\"0%\" stop-color=\"{color}\" stop-opacity=\"{}\"></stop><stop offset=\"100%\" stop-color=\"{color}\" stop-opacity=\"0\"></stop></linearGradient>",
            num(fill_opacity)
        ));
        let pts: Vec<(f64, f64)> = xs.iter().zip(&s.values).map(|(x, v)| (*x, y(*v))).collect();
        if pts.is_empty() {
            continue;
        }
        let line = curve_path(curve, &pts);
        let base = y(lo.max(0.0).min(hi));
        let (first, last) = (pts[0].0, pts[pts.len() - 1].0);
        let area = format!(
            "{line}L{},{b}L{},{b}Z",
            num(last),
            num(first),
            b = num(base)
        );
        let mut layer = format!(
            "<path d=\"{area}\" fill=\"url(#{gid})\"></path><path d=\"{line}\" fill=\"none\" stroke=\"{color}\" stroke-linecap=\"round\" stroke-width=\"2\"></path>"
        );
        if fade {
            let mask = instance_id(comp, &format!("area-edge-mask-{}", s.key));
            defs.push_str(&fade_gradient(
                &format!("{mask}-gradient"),
                iw,
                "white",
                true,
                true,
            ));
            defs.push_str(&format!(
                "<mask id=\"{mask}\"><rect fill=\"url(#{mask}-gradient)\" height=\"{}\" width=\"{}\" x=\"0\" y=\"0\"></rect></mask>",
                num(ih),
                num(iw)
            ));
            layer = format!("<g mask=\"url(#{mask})\">{layer}</g>");
        }
        series.push_str(&format!("<g class=\"series\">{layer}</g>"));
    }
    defs.push_str(&reveal_clip(&clip, iw, ih + 20.0, 0.0));
    let ticks_y: Vec<f64> = d3_ticks(lo, hi, 5.0).iter().map(|t| y(*t)).collect();
    let labels: Vec<String> = data.labels.iter().map(|l| row_label(l)).collect();
    let axis = motion_x_labels(&data_ticks(&xs, &labels, 5), ih, frame.bottom);
    format!(
        "<div data-slot=\"area-chart\" class=\"v-motion\" role=\"img\" aria-label=\"{label}\">{}<defs>{defs}</defs>{}{}{axis}<g clip-path=\"url(#{clip})\">{series}</g></g></svg></div>",
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
        let mut c = stub("area-chart", "Sessions");
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

    fn series_fixture(variant: Option<&str>) -> ComponentNode {
        let mut c = stub("area-chart", "Revenue versus costs");
        if let Some(v) = variant {
            c.style = Some(format!("area-chart+{v}"));
        }
        c.props.insert("series".into(), "revenue,costs".into());
        for (d, r, k) in [
            ("2024-01-01", "16200", "11500"),
            ("2024-01-02", "14800", "10800"),
            ("2024-01-03", "17100", "12100"),
            ("2024-01-04", "15900", "11200"),
        ] {
            let mut it = ComponentItemNode {
                item_type: "item".into(),
                text: d.into(),
                link: None,
                tone: None,
                config: Default::default(),
            };
            it.config.insert("revenue".into(), r.into());
            it.config.insert("costs".into(), k.into());
            c.items.push(it);
        }
        c
    }

    #[test]
    fn fixture_nests_chart_container_with_recharts_geometry() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"area-chart\" role=\"img\" aria-label=\"Sessions\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\">"));
        assert!(html.contains(
            "<line x1=\"8\" y1=\"226\" x2=\"424\" y2=\"226\" stroke-dasharray=\"4 4\"></line>"
        ));
        assert!(html.contains(
            "<line x1=\"8\" y1=\"8\" x2=\"424\" y2=\"8\" stroke-dasharray=\"4 4\"></line>"
        ));
        assert!(html.contains("<path d=\"M8,117C77.3333,62.5,146.6667,8,216,8C285.3333,8,354.6667,35.25,424,62.5\" fill=\"none\" stroke=\"var(--cronus-chart-1)\" stroke-width=\"2\"></path>"));
        assert!(html.contains("stop-opacity=\"0.4\""));
        assert!(html.contains("fill-opacity=\"0.6\""));
        // recharts AreaRevealShape: the series sit in a clip rect the CSS grows.
        assert!(html.contains("<clipPath id=\"cui-area-chart-area-clip\"><rect class=\"reveal\" x=\"8\" y=\"0\" width=\"416\" height=\"256\"></rect></clipPath>"));
        // minTickGap 24 + preserveEnd: Jan clipped at x=8, Mar pulled inside.
        assert!(!html.contains(">Jan</tspan>"));
        assert!(html.contains("<tspan x=\"216\" dy=\"0.71em\">Feb</tspan>"));
        assert!(html.contains(">Mar</tspan>"));
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("polyline"));
    }

    #[test]
    fn numeric_items_drive_values() {
        let mut c = stub("area-chart", "Revenue");
        for t in ["1", "3", "2"] {
            c.items.push(ComponentItemNode {
                item_type: "item".into(),
                text: t.into(),
                link: None,
                tone: None,
                config: Default::default(),
            });
        }
        let html = render(&c);
        // max 3 → nice domain 0..3 (step 0.75): value 1 at y = 226 - 1/3*218, value 3 at the top.
        assert!(html.contains("M8,153.3333C"));
        assert!(html.contains(",216,8C"));
    }

    #[test]
    fn series_prop_draws_one_area_per_key_in_chart_colors() {
        let html = render(&series_fixture(None));
        assert_eq!(html.matches("<linearGradient").count(), 2);
        assert!(html.contains("stroke=\"var(--cronus-chart-1)\""));
        assert!(html.contains("stroke=\"var(--cronus-chart-2)\""));
        // recharts keeps the raw category text on the x axis.
        assert!(html.contains(">2024-01-04</tspan>"));
        // Domain 0..18000 (max 17100 → nice 18000): costs 11500 → y = 226 - 11500/18000*218.
        assert!(html.contains("M8,86.7222C"));
    }

    #[test]
    fn motion_variant_renders_visx_frame_with_clip_reveal() {
        let html = render(&series_fixture(Some("motion")));
        assert!(html.starts_with("<div data-slot=\"area-chart\" class=\"v-motion\" role=\"img\" aria-label=\"Revenue versus costs\"><svg viewBox=\"0 0 432 216\" aria-hidden=\"true\">"));
        assert!(html.contains("<g transform=\"translate(40,40)\"><rect fill=\"transparent\" height=\"136\" width=\"352\" x=\"0\" y=\"0\"></rect>"));
        assert!(html.contains("<clipPath id=\"cui-area-chart-reveal\"><rect class=\"reveal\" height=\"156\" width=\"352\" x=\"0\" y=\"0\"></rect></clipPath>"));
        assert!(html.contains("<g clip-path=\"url(#cui-area-chart-reveal)\">"));
        // y-domain: max 17100 * 1.1 = 18810 → nice 20000; grid rows at ticks(0, 20000, 5).
        assert_eq!(html.matches("stroke-dasharray=\"4,4\"").count(), 5);
        assert!(
            html.contains("<line class=\"visx-line\" x1=\"0\" y1=\"136\" x2=\"352\" y2=\"136\"")
        );
        // Time axis: Jan 1 … Jan 4 spread over the 352px plot; labels formatted like Intl.
        assert!(html.contains("<text x=\"0\" y=\"156\" text-anchor=\"middle\" dominant-baseline=\"central\" fill=\"var(--cronus-fg-tertiary)\">Jan 1</text>"));
        assert!(html.contains("<text x=\"352\" y=\"156\" text-anchor=\"middle\" dominant-baseline=\"central\" fill=\"var(--cronus-fg-tertiary)\">Jan 4</text>"));
        // Monotone area + round stroke per series, fill via the 0.4 → 0 gradient.
        assert!(html.contains("stop-opacity=\"0.4\""));
        assert!(html.contains(
            "stroke=\"var(--cronus-chart-1)\" stroke-linecap=\"round\" stroke-width=\"2\""
        ));
        assert!(html.contains("L0,136Z\" fill=\"url(#cui-area-chart-area-gradient-revenue)\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("data-slot=\"chart\""));
    }

    #[test]
    fn motion_fade_and_fill_opacity_props() {
        let mut c = series_fixture(Some("motion"));
        c.props.insert("fade".into(), "true".into());
        c.props.insert("fill-opacity".into(), "0.3".into());
        let html = render(&c);
        assert!(html.contains("stop-opacity=\"0.3\""));
        assert!(html.contains("<mask id=\"cui-area-chart-area-edge-mask-revenue\">"));
        assert!(html.contains("<g mask=\"url(#cui-area-chart-area-edge-mask-revenue)\">"));
        assert!(html.contains("offset=\"15%\" stop-color=\"white\" stop-opacity=\"1\""));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&fixture());
            assert!(!html.contains("v-data="));
            assert!(html.contains("data-slot=\"area-chart\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"area-chart\"] {\n  display: block; width: 100%; height: 16rem;\n}"
        ));
        assert!(css.contains(
            "[data-slot=\"area-chart\"].v-motion {\n  height: auto; aspect-ratio: 2 / 1;\n}"
        ));
        assert!(!css.contains("[data-slot=\"area-chart\"] polyline"));
    }
}
