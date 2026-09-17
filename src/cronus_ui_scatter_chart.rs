//! Dedicated ScatterChart renderer.
//!
//! Default (recharts `ScatterChart`, docs "Default"): `<div
//! data-slot="scatter-chart" role="img" aria-label>` › `<div data-slot="chart">
//! <svg viewBox="0 0 432 256">` dashed grid rows and columns on nice numeric
//! domains (`[0, max]`), a 60px-wide y axis, one circle (r 4.5135, recharts
//! symbol size 60) per point — `item "Search" x:12 y:4.2` where the text is
//! the series key of `series:"Search,Social"` (chart-1..N) — and the numeric
//! tick labels. Dots pop in (recharts Scatter animation, 400ms ease).
//!
//! Motion (`style:scatter-chart+motion`, `@cronus-ui/ui/charts`): the visx
//! `ScatterChart` — `<div data-slot="scatter-chart" class="v-motion">`, `2 / 1`
//! SVG, 40px margins, time axis padded by radius + 10, `[0, max × 1.1]` nice
//! y domain shared by the `series:` keys, `Grid horizontal`, one `Scatter`
//! marker per row and series (5px dot + 2px ring 2px out, chart-1..N),
//! `XAxis` labels and the shared `ChartRevealClip`. Hovering a marker dims
//! the others to 0.5.

use crate::cronus_ui_chart::{
    container, d3_ticks, data_items, data_ticks, grid_cols, grid_rows, motion_grid,
    motion_x_labels, nice_domain, num, positive_domain, reveal_clip, row_label, row_times,
    series_keys, time_xs, x_tick_labels_at, y_of, Frame, DEMO_VALUES, M_W, PLOT_B, PLOT_R, PLOT_T,
    TICK_FILL,
};
use crate::cronus_ui_kit::{choice, esc, instance_id, label_of};
use crate::parser::ComponentNode;

pub const SCATTER_PLOT_L: f64 = 68.0;
pub const SCATTER_R: f64 = 4.5135;

/// `(series key, x, y)` per item; the legacy fixture (`text "A"` rows without
/// `x:` / `y:`) plots the demo cycle at 1..n.
fn points(comp: &ComponentNode) -> Vec<(String, f64, f64)> {
    let items = data_items(comp);
    let parsed: Vec<(String, f64, f64)> = items
        .iter()
        .filter_map(|i| {
            let x = i.config.get("x")?.trim().parse::<f64>().ok()?;
            let y = i.config.get("y")?.trim().parse::<f64>().ok()?;
            Some((i.text.trim().to_string(), x, y))
        })
        .collect();
    if !parsed.is_empty() {
        return parsed;
    }
    let n = if items.is_empty() { 3 } else { items.len() };
    (0..n)
        .map(|i| {
            (
                "value".to_string(),
                (i + 1) as f64,
                DEMO_VALUES[i % DEMO_VALUES.len()],
            )
        })
        .collect()
}

fn series_order(comp: &ComponentNode, pts: &[(String, f64, f64)]) -> Vec<String> {
    let mut keys: Vec<String> = series_keys(comp).into_iter().map(|(k, _)| k).collect();
    for (k, _, _) in pts {
        if !keys.contains(k) {
            keys.push(k.clone());
        }
    }
    keys
}

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let pts = points(comp);
    let keys = series_order(comp, &pts);
    if choice(comp, "variant", &["motion"]) == Some("motion") {
        return render_motion(comp, &label, &pts, &keys);
    }
    let xmax = pts.iter().map(|p| p.1).fold(0.0_f64, f64::max);
    let ymax = pts.iter().map(|p| p.2).fold(0.0_f64, f64::max);
    let (xlo, xhi, xticks) = nice_domain(0.0, xmax);
    let (ylo, yhi, yticks) = nice_domain(0.0, ymax);
    let x_of = |v: f64| SCATTER_PLOT_L + (v - xlo) / (xhi - xlo) * (PLOT_R - SCATTER_PLOT_L);
    let tick_ys: Vec<f64> = yticks.iter().map(|t| y_of(*t, ylo, yhi)).collect();
    let tick_xs: Vec<f64> = xticks.iter().map(|t| x_of(*t)).collect();
    let mut body = String::new();
    body.push_str(&grid_rows(&tick_ys, SCATTER_PLOT_L, PLOT_R));
    body.push_str(&grid_cols(&tick_xs, PLOT_T, PLOT_B));
    for (si, key) in keys.iter().enumerate() {
        for (k, x, y) in &pts {
            if k != key {
                continue;
            }
            body.push_str(&format!(
                "<circle class=\"dot\" cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"var(--cronus-chart-{})\"></circle>",
                num(x_of(*x)),
                num(y_of(*y, ylo, yhi)),
                num(SCATTER_R),
                si % 5 + 1
            ));
        }
    }
    let xlabels: Vec<String> = xticks.iter().map(|t| num(*t)).collect();
    body.push_str(&x_tick_labels_at(&xlabels, &tick_xs, 5.0, PLOT_B + 8.0));
    for (t, y) in yticks.iter().zip(&tick_ys) {
        body.push_str(&format!(
            "<g><text x=\"60\" y=\"{y}\" text-anchor=\"end\" fill=\"{TICK_FILL}\"><tspan x=\"60\" dy=\"0.355em\">{t}</tspan></text></g>",
            y = num(*y),
            t = num(*t),
        ));
    }
    format!(
        "<div data-slot=\"scatter-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

/// visx `ScatterChart` at the 432px reference width (aspect 2 / 1): one
/// row per item (`item "2023-01-01" sessions:213 conversions:125`).
fn render_motion(
    comp: &ComponentNode,
    label: &str,
    _pts: &[(String, f64, f64)],
    _keys: &[String],
) -> String {
    crate::cronus_ui_chart::note_motion();
    let data = crate::cronus_ui_chart::chart_data(comp, &["2024-01-01"], &DEMO_VALUES);
    let frame = Frame::aspect(M_W, 2.0);
    let (iw, ih) = (frame.iw(), frame.ih());
    let radius = 5.0;
    let pad = radius + 10.0;
    let times = row_times(&data.labels);
    let xs = time_xs(&times, pad, (iw - pad).max(pad));
    let (lo, hi) = positive_domain(data.max().max(0.0));
    let y = |v: f64| ih - (v - lo) / (hi - lo) * ih;
    let clip = instance_id(comp, "reveal");
    let grid_id = instance_id(comp, "grid");
    let mut layers = String::new();
    for (si, s) in data.series.iter().enumerate() {
        let color = format!("var(--cronus-chart-{})", si % 5 + 1);
        layers.push_str("<g class=\"markers\">");
        for (x, v) in xs.iter().zip(&s.values) {
            layers.push_str(&format!(
                "<g transform=\"translate({}, {})\"><g class=\"marker\"><circle cx=\"0\" cy=\"0\" fill=\"{color}\" r=\"{}\"></circle><circle cx=\"0\" cy=\"0\" fill=\"none\" r=\"{}\" stroke=\"{color}\" stroke-width=\"2\"></circle></g></g>",
                num(*x),
                num(y(*v)),
                num(radius),
                num(radius + 2.0 + 1.0)
            ));
        }
        layers.push_str("</g>");
    }
    let defs = reveal_clip(&clip, iw, ih + 20.0, 0.0);
    let ticks_y: Vec<f64> = d3_ticks(lo, hi, 5.0).iter().map(|t| y(*t)).collect();
    let labels: Vec<String> = data.labels.iter().map(|l| row_label(l)).collect();
    let axis = motion_x_labels(&data_ticks(&xs, &labels, 5), ih, frame.bottom);
    format!(
        "<div data-slot=\"scatter-chart\" class=\"v-motion\" role=\"img\" aria-label=\"{}\">{}<defs>{defs}</defs>{}{}{axis}<g clip-path=\"url(#{clip})\">{layers}</g></g></svg></div>",
        esc(label),
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
        let mut c = stub("scatter-chart", "Reach");
        for t in ["A", "B", "C"] {
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

    fn channels() -> ComponentNode {
        let mut c = stub("scatter-chart", "Reach versus conversion");
        c.props.insert("series".into(), "Search,Social".into());
        for (k, x, y) in [
            ("Search", "12", "4.2"),
            ("Search", "55", "8.8"),
            ("Social", "18", "9.4"),
            ("Social", "62", "14.6"),
        ] {
            let mut it = ComponentItemNode {
                item_type: "item".into(),
                text: k.into(),
                link: None,
                tone: None,
                config: Default::default(),
            };
            it.config.insert("x".into(), x.into());
            it.config.insert("y".into(), y.into());
            c.items.push(it);
        }
        c
    }

    #[test]
    fn fixture_matches_recharts_numeric_axes() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"scatter-chart\" role=\"img\" aria-label=\"Reach\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\"><line x1=\"68\" y1=\"226\" x2=\"424\" y2=\"226\" stroke-dasharray=\"4 4\"></line>"));
        assert!(html.contains(
            "<line x1=\"157\" y1=\"8\" x2=\"157\" y2=\"226\" stroke-dasharray=\"4 4\"></line>"
        ));
        assert!(html.contains("<circle class=\"dot\" cx=\"186.6667\" cy=\"117\" r=\"4.5135\" fill=\"var(--cronus-chart-1)\"></circle>"));
        assert!(html.contains("<circle class=\"dot\" cx=\"305.3333\" cy=\"8\""));
        assert!(html.contains("<circle class=\"dot\" cx=\"424\" cy=\"62.5\""));
        for t in ["0", "0.75", "1.5", "2.25", "3"] {
            assert!(html.contains(&format!("dy=\"0.71em\">{t}</tspan>")));
        }
        assert!(html.contains("<g><text x=\"60\" y=\"8\" text-anchor=\"end\" fill=\"#666\"><tspan x=\"60\" dy=\"0.355em\">8</tspan></text></g>"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn item_xy_config_plots_two_series() {
        let html = render(&channels());
        assert_eq!(html.matches("class=\"dot\"").count(), 4);
        // x domain 0..80 (max 62 → nice), y 0..16: Social (62, 14.6) → x 68 + 62/80*356, y 226 - 14.6/16*218.
        assert!(html.contains("<circle class=\"dot\" cx=\"343.9\" cy=\"27.075\" r=\"4.5135\" fill=\"var(--cronus-chart-2)\"></circle>"));
        assert!(html.contains("dy=\"0.71em\">80</tspan>"));
    }

    #[test]
    fn motion_variant_plots_ringed_markers_on_a_time_axis() {
        let mut c = stub("scatter-chart", "Sessions versus conversions");
        c.style = Some("scatter-chart+motion".into());
        c.props
            .insert("series".into(), "sessions,conversions".into());
        for (d, s, k) in [
            ("2023-01-01", "140", "125"),
            ("2023-02-01", "180", "100"),
            ("2023-03-01", "220", "70"),
        ] {
            let mut it = ComponentItemNode {
                item_type: "item".into(),
                text: d.into(),
                link: None,
                tone: None,
                config: Default::default(),
            };
            it.config.insert("sessions".into(), s.into());
            it.config.insert("conversions".into(), k.into());
            c.items.push(it);
        }
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"scatter-chart\" class=\"v-motion\" role=\"img\" aria-label=\"Sessions versus conversions\"><svg viewBox=\"0 0 432 216\""));
        // Time range padded by 15px: first marker at x 15; domain 0..260 (220 * 1.1 = 242 → nice).
        assert!(html.contains("<g class=\"markers\"><g transform=\"translate(15, 62.7692)\"><g class=\"marker\"><circle cx=\"0\" cy=\"0\" fill=\"var(--cronus-chart-1)\" r=\"5\"></circle><circle cx=\"0\" cy=\"0\" fill=\"none\" r=\"8\" stroke=\"var(--cronus-chart-1)\" stroke-width=\"2\"></circle></g></g>"));
        assert!(html.contains("<g transform=\"translate(337, 20.9231)\">"));
        assert!(html.contains("fill=\"var(--cronus-chart-2)\" r=\"5\""));
        assert!(html.contains("<g clip-path=\"url(#cui-scatter-chart-reveal)\">"));
        assert!(html.contains(">Jan 1</text>"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"scatter-chart\"] {\n  display: block; width: 100%; height: 16rem;\n}"
        ));
        assert!(css.contains("[data-slot=\"scatter-chart\"].v-motion .marker {"));
    }
}
