//! Dedicated ComposedChart renderer.
//!
//! Default (recharts `ComposedChart`, docs "Default"): `<div
//! data-slot="composed-chart" role="img" aria-label>` › `<div data-slot="chart">
//! <svg viewBox="0 0 432 256">` dashed grid rows, then one layer per series in
//! `series:` order — `area` (gradient 0.35 → 0, monotone, stroke 2, clip
//! reveal), `bar` (radius 4, 10% band inset, grows from the baseline) or
//! `line` (monotone, stroke 2, dash draw) — on a band x axis (recharts uses a
//! band scale as soon as a Bar is present), chart-1..N per series.
//!
//! Motion (`style:composed-chart+motion`, `@cronus-ui/ui/charts`): the visx
//! `ComposedChart` — `<div data-slot="composed-chart" class="v-motion">`,
//! `2 / 1` SVG, 40px margins, time x axis, `Area` (fill-opacity 0.32,
//! `curve:catmull` = `curveCatmullRom.alpha(0.42)`), `SeriesBar` (width
//! min(slot × 0.88, `max-bar-size`), `bar-gap`, radius 4, staggered grow),
//! `Line` (stroke 2.5, edge fade), `XAxis` with `ticks:8` labels, and the
//! shared `ChartRevealClip` padded by half a bar.
//!
//! Data: `series:"costs:Run rate:area,units:Units:bar,revenue:Revenue:line"`
//! + `item "Jul 29" costs:11500 units:40 revenue:16200`.

use crate::cronus_ui_chart::{
    area_paths, band_xs, chart_data, container, curve_path, d3_ticks, data_ticks, fade_gradient,
    line_path, motion_grid, motion_x_labels, nice_domain, num, reveal_clip, rounded_rect_path,
    row_label, row_times, series_color, series_kinds, time_series_domain, time_xs, value_grid,
    x_tick_labels, y_of, ChartData, Frame, DEMO_VALUES, M_W,
};
use crate::cronus_ui_kit::{attr, attr_num, choice, instance_id, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let data = chart_data(comp, &["Jan", "Feb", "Mar"], &DEMO_VALUES);
    let mut kinds = series_kinds(comp, "line");
    if kinds.is_empty() {
        // Legacy fixture: an area under bars.
        kinds = vec!["area".into(), "bar".into()];
    }
    if choice(comp, "variant", &["motion"]) == Some("motion") {
        return render_motion(comp, &label, &data, &kinds);
    }
    let (lo, hi, ticks) = nice_domain(0.0, data.max().max(0.0));
    let (centers, band) = band_xs(data.len());
    let clip = instance_id(comp, "area-clip");
    let mut body = value_grid(&ticks, lo, hi);
    body.push_str(&format!(
        "<defs><clipPath id=\"{clip}\"><rect class=\"reveal\" x=\"8\" y=\"0\" width=\"416\" height=\"256\"></rect></clipPath></defs>"
    ));
    let base = y_of(0f64.max(lo), lo, hi);
    let offset = band * 0.1;
    let size = {
        let raw = band - 2.0 * offset;
        if raw > 1.0 {
            raw.round()
        } else {
            raw
        }
    };
    for (i, s) in data.series.iter().enumerate() {
        let color = series_color(comp, i);
        match kinds.get(i).map(String::as_str).unwrap_or("line") {
            "area" => {
                let gid = instance_id(comp, &format!("area-fill-{}", s.key));
                body.push_str(&format!(
                    "<g clip-path=\"url(#{clip})\">{}</g>",
                    area_paths(&centers, &s.values, lo, hi, &color, &gid, "0.35")
                ));
            }
            "bar" => {
                for (c, v) in centers.iter().zip(&s.values) {
                    let y = y_of(*v, lo, hi);
                    let (top, h) = if y <= base {
                        (y, base - y)
                    } else {
                        (base, y - base)
                    };
                    body.push_str(&format!(
                        "<path class=\"grow\" d=\"{}\" fill=\"{color}\"></path>",
                        rounded_rect_path(c - band / 2.0 + offset, top, size, h, 4.0)
                    ));
                }
            }
            _ => body.push_str(
                &line_path(&centers, &s.values, lo, hi, &color)
                    .replace("<path d=", "<path class=\"draw\" pathLength=\"1\" d="),
            ),
        }
    }
    body.push_str(&x_tick_labels(&data.labels, &centers, 24.0));
    format!(
        "<div data-slot=\"composed-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

/// visx `ComposedChart` at the 432px reference width (aspect 2 / 1).
fn render_motion(comp: &ComponentNode, label: &str, data: &ChartData, kinds: &[String]) -> String {
    crate::cronus_ui_chart::note_motion();
    let frame = Frame::aspect(M_W, 2.0);
    let (iw, ih) = (frame.iw(), frame.ih());
    let n = data.len();
    let times = row_times(&data.labels);
    let xs = time_xs(&times, 0.0, iw);
    let (lo, hi) = time_series_domain(data.min().min(0.0), data.max());
    let y = |v: f64| ih - (v - lo) / (hi - lo) * ih;
    let curve = attr(comp, "curve").unwrap_or("monotone");
    let bar_gap = attr_num::<f64>(comp, "bar-gap").unwrap_or(4.0);
    let max_bar = attr_num::<f64>(comp, "max-bar-size");
    let bar_count = kinds.iter().filter(|k| k.as_str() == "bar").count().max(1);
    // computeSeriesBarWidth: slot = columnWidth, width = min(slot * 0.88, maxBarSize).
    let slot = if n < 2 { iw } else { iw / (n as f64 - 1.0) };
    let mut bar_w = (slot * 0.88).min(max_bar.unwrap_or(f64::INFINITY));
    if bar_count > 1 {
        let max_group = slot * 0.92;
        let needed = bar_count as f64 * bar_w + (bar_count as f64 - 1.0) * bar_gap;
        if needed > max_group && max_group > 0.0 {
            bar_w = ((max_group - (bar_count as f64 - 1.0) * bar_gap) / bar_count as f64).max(4.0);
        }
    }
    let bar_w = bar_w.max(2.0);
    let group_w = bar_count as f64 * bar_w + (bar_count as f64 - 1.0) * bar_gap;
    let clip_pad = if bar_count <= 1 {
        (bar_w / 2.0).ceil()
    } else {
        (group_w / 2.0).ceil()
    };
    let clip = instance_id(comp, "reveal");
    let grid_id = instance_id(comp, "grid");
    let mut defs = String::new();
    let mut layers = String::new();
    let mut bar_index = 0usize;
    for (i, s) in data.series.iter().enumerate() {
        let color = series_color(comp, i);
        let pts: Vec<(f64, f64)> = xs.iter().zip(&s.values).map(|(x, v)| (*x, y(*v))).collect();
        if pts.is_empty() {
            continue;
        }
        match kinds.get(i).map(String::as_str).unwrap_or("line") {
            "area" => {
                let gid = instance_id(comp, &format!("area-gradient-{}", s.key));
                let fill_opacity = attr_num::<f64>(comp, "fill-opacity").unwrap_or(0.32);
                defs.push_str(&format!(
                    "<linearGradient id=\"{gid}\" x1=\"0%\" x2=\"0%\" y1=\"0%\" y2=\"100%\"><stop offset=\"0%\" stop-color=\"{color}\" stop-opacity=\"{}\"></stop><stop offset=\"100%\" stop-color=\"{color}\" stop-opacity=\"0\"></stop></linearGradient>",
                    num(fill_opacity)
                ));
                let line = curve_path(curve, &pts);
                let base = y(lo.max(0.0).min(hi));
                let (first, last) = (pts[0].0, pts[pts.len() - 1].0);
                layers.push_str(&format!(
                    "<g class=\"series\"><path d=\"{line}L{},{b}L{},{b}Z\" fill=\"url(#{gid})\"></path><path d=\"{line}\" fill=\"none\" stroke=\"{color}\" stroke-linecap=\"round\" stroke-width=\"2\"></path></g>",
                    num(last),
                    num(first),
                    b = num(base)
                ));
            }
            "bar" => {
                layers.push_str(&format!("<g class=\"series-bar n{n}\">"));
                for (ri, (x, v)) in xs.iter().zip(&s.values).enumerate() {
                    let vy = y(*v);
                    let left = x - group_w / 2.0 + bar_index as f64 * (bar_w + bar_gap);
                    layers.push_str(&format!(
                        "<rect class=\"grow i{ri}\" fill=\"{color}\" height=\"{}\" rx=\"4\" ry=\"4\" width=\"{}\" x=\"{}\" y=\"{}\"></rect>",
                        num(ih - vy),
                        num(bar_w),
                        num(left),
                        num(vy)
                    ));
                }
                layers.push_str("</g>");
                bar_index += 1;
            }
            _ => {
                let gid = instance_id(comp, &format!("line-gradient-{}", s.key));
                defs.push_str(&fade_gradient(&gid, iw, &color, true, true));
                layers.push_str(&format!(
                    "<g class=\"series\"><path d=\"{}\" fill=\"none\" stroke=\"url(#{gid})\" stroke-linecap=\"round\" stroke-width=\"2.5\"></path></g>",
                    curve_path(curve, &pts)
                ));
            }
        }
    }
    defs.push_str(&reveal_clip(&clip, iw, ih + 20.0, clip_pad));
    let ticks_y: Vec<f64> = d3_ticks(lo, hi, 5.0).iter().map(|t| y(*t)).collect();
    let labels: Vec<String> = data.labels.iter().map(|l| row_label(l)).collect();
    let target = attr_num::<usize>(comp, "ticks").unwrap_or(5);
    let axis = motion_x_labels(&data_ticks(&xs, &labels, target), ih, frame.bottom);
    format!(
        "<div data-slot=\"composed-chart\" class=\"v-motion\" role=\"img\" aria-label=\"{label}\">{}<defs>{defs}</defs>{}{}{axis}<g clip-path=\"url(#{clip})\">{layers}</g></g></svg></div>",
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
        let mut c = stub("composed-chart", "Mix");
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

    fn typed(style: &str) -> ComponentNode {
        let mut c = stub("composed-chart", "Daily mix");
        c.style = Some(style.into());
        c.props.insert(
            "series".into(),
            "costs:Run rate:area,units:Units:bar,revenue:Revenue:line".into(),
        );
        for (d, k, u, r) in [
            ("2024-01-01", "80", "46", "96"),
            ("2024-01-02", "84", "52", "99"),
            ("2024-01-03", "90", "57", "103"),
            ("2024-01-04", "93", "60", "106"),
        ] {
            let mut it = ComponentItemNode {
                item_type: "item".into(),
                text: d.into(),
                link: None,
                tone: None,
                config: Default::default(),
            };
            it.config.insert("costs".into(), k.into());
            it.config.insert("units".into(), u.into());
            it.config.insert("revenue".into(), r.into());
            c.items.push(it);
        }
        c
    }

    #[test]
    fn fixture_matches_recharts_area_and_bars() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"composed-chart\" role=\"img\" aria-label=\"Mix\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\">"));
        // One value series cycling 4, 8, 6 drawn as an area (chart-1) on band centres — legacy fixture.
        assert!(html.contains("<path d=\"M77.3333,117C123.5556,62.5,169.7778,8,216,8C262.2222,8,308.4444,35.25,354.6667,62.5\" fill=\"none\" stroke=\"var(--cronus-chart-1)\" stroke-width=\"2\"></path>"));
        assert!(html.contains("stop-opacity=\"0.35\""));
        assert!(!html.contains("polyline"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn typed_series_draw_area_bar_and_line_layers() {
        let html = render(&typed("composed-chart"));
        assert_eq!(html.matches("<linearGradient").count(), 1);
        assert_eq!(html.matches("class=\"grow\"").count(), 4);
        assert_eq!(html.matches("class=\"draw\"").count(), 1);
        assert!(html.contains("fill=\"var(--cronus-chart-2)\""));
        assert!(html.contains("stroke=\"var(--cronus-chart-3)\""));
        assert!(html.contains("<g clip-path=\"url(#cui-composed-chart-area-clip)\">"));
    }

    #[test]
    fn motion_variant_lays_bars_on_the_time_axis() {
        let mut c = typed("composed-chart+motion");
        c.props.insert("curve".into(), "catmull".into());
        c.props.insert("bar-gap".into(), "0".into());
        c.props.insert("max-bar-size".into(), "32".into());
        c.props.insert("ticks".into(), "8".into());
        c.props.insert(
            "series".into(),
            "costs:Run rate:area:4,units:Units:bar:3,revenue:Revenue:line:1".into(),
        );
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"composed-chart\" class=\"v-motion\" role=\"img\" aria-label=\"Daily mix\"><svg viewBox=\"0 0 432 216\""));
        // slot 352/3 = 117.333 → bar width min(103.2, 32) = 32, centred on each time position.
        assert!(html.contains(
            "<g class=\"series-bar n4\"><rect class=\"grow i0\" fill=\"var(--cronus-chart-3)\""
        ));
        assert!(html.contains("stop-color=\"var(--cronus-chart-4)\" stop-opacity=\"0.32\""));
        assert!(html.contains("<stop offset=\"15%\" stop-color=\"var(--cronus-chart-1)\""));
        assert!(html.contains("width=\"32\" x=\"-16\""));
        assert!(html.contains("width=\"32\" x=\"336\""));
        // Reveal clip padded by half a bar (16px).
        assert!(html.contains("<clipPath id=\"cui-composed-chart-reveal\"><rect class=\"reveal\" height=\"188\" width=\"384\" x=\"-16\" y=\"-16\"></rect></clipPath>"));
        // Catmull-Rom curves for the area and the faded 2.5px line.
        assert!(html.contains("stop-opacity=\"0.32\""));
        assert!(html.contains("stroke=\"url(#cui-composed-chart-line-gradient-revenue)\" stroke-linecap=\"round\" stroke-width=\"2.5\""));
        assert_eq!(html.matches(">Jan ").count(), 4);
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"composed-chart\"] {\n  display: block; width: 100%; height: 16rem;\n}"
        ));
        assert!(css.contains("[data-slot=\"composed-chart\"].v-motion .grow {"));
    }
}
