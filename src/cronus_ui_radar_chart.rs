//! Dedicated RadarChart renderer.
//!
//! Default (recharts `RadarChart`, docs "Default"): `<div data-slot="radar-chart"
//! role="img" aria-label>` › `<div data-slot="chart"><svg viewBox="0 0 432 256">`
//! polar grid rings (`#ccc`, one per nice tick), a spoke per metric, one
//! polygon per series (`series:"current,target"`, chart-1..N, fill-opacity
//! 0.25) at 70% of the polar radius and the angle-axis labels (`#808080`).
//! Polygons grow from the centre (recharts Radar animation, 1500ms ease).
//!
//! Motion (`style:radar-chart+motion`, `@cronus-ui/ui/charts`): the docs
//! composition — `<div class="chart-with-legend">` holding `<div
//! data-slot="radar-chart" class="v-motion">` (a `size`px SVG, radius
//! (size - 120) / 2): `RadarGrid` (5 concentric polygons rotated half a step,
//! `--cronus-border` at 0.6, level labels 20…100 in 9px), `RadarAxis` spokes,
//! `RadarLabels` (`label-size`, `label-offset`) and one `RadarArea` per
//! series (fill 0.15, stroke 2, 4px points ringed by the surface) — plus the
//! shared `Legend` (`legend:"Series"`). Grid rings, spokes and areas scale in
//! from the centre with the React delays; hovering an area lifts it.

use crate::cronus_ui_chart::{
    chart_data, container, legend_html, marker_class, nice_domain, num, polar, ChartData,
    LegendRow, DEMO_VALUES, POLAR_CX, POLAR_CY,
};
use crate::cronus_ui_kit::{attr, attr_num, choice, esc, label_of};
use crate::parser::ComponentNode;

pub const RADAR_OUTER: f64 = 86.1;
const EPS: f64 = 1e-5;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let data = chart_data(comp, &["Speed", "Reliability", "Comfort"], &DEMO_VALUES);
    if choice(comp, "variant", &["motion"]) == Some("motion") {
        return render_motion(comp, &label, &data);
    }
    let axes = &data.labels;
    let n = axes.len().max(1);
    let (lo, hi, ticks) = nice_domain(0.0, data.max().max(0.0));
    let angle = |i: usize| 90.0 - 360.0 * i as f64 / n as f64;
    let mut body = String::new();
    for t in &ticks {
        let r = (t - lo) / (hi - lo) * RADAR_OUTER;
        body.push_str(&format!(
            "<path d=\"{}\" fill=\"none\" stroke=\"#ccc\"></path>",
            ring(n, r, angle)
        ));
    }
    for i in 0..n {
        let (x, y) = polar(POLAR_CX, POLAR_CY, RADAR_OUTER, angle(i));
        body.push_str(&format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#ccc\"></line>",
            num(POLAR_CX),
            num(POLAR_CY),
            num(x),
            num(y)
        ));
    }
    for (si, s) in data.series.iter().enumerate() {
        let mut d = String::new();
        let mut first = (POLAR_CX, POLAR_CY);
        for i in 0..n {
            let v = s.values.get(i).copied().unwrap_or(0.0);
            let r = (v - lo) / (hi - lo) * RADAR_OUTER;
            let (x, y) = polar(POLAR_CX, POLAR_CY, r, angle(i));
            if i == 0 {
                first = (x, y);
            }
            d.push_str(&format!(
                "{}{},{}",
                if i == 0 { "M" } else { "L" },
                num(x),
                num(y)
            ));
        }
        d.push_str(&format!("L{},{}Z", num(first.0), num(first.1)));
        let color = format!("var(--cronus-chart-{})", si % 5 + 1);
        body.push_str(&format!(
            "<path class=\"radar-in\" d=\"{d}\" fill=\"{color}\" fill-opacity=\"0.25\" stroke=\"{color}\"></path>"
        ));
    }
    for (i, name) in axes.iter().take(n).enumerate() {
        let a = angle(i);
        let (x, y) = polar(POLAR_CX, POLAR_CY, RADAR_OUTER + 8.0, a);
        let cos = (-a).to_radians().cos();
        let sin = (-a).to_radians().sin();
        let anchor = if cos > EPS {
            "start"
        } else if cos < -EPS {
            "end"
        } else {
            "middle"
        };
        let dy = if cos.abs() <= EPS {
            if sin > 0.0 {
                "0.71em"
            } else {
                "0em"
            }
        } else {
            "0.355em"
        };
        body.push_str(&format!(
            "<g><text x=\"{x}\" y=\"{y}\" text-anchor=\"{anchor}\" fill=\"#808080\"><tspan x=\"{x}\" dy=\"{dy}\">{t}</tspan></text></g>",
            x = num(x),
            y = num(y),
            t = esc(name),
        ));
    }
    format!(
        "<div data-slot=\"radar-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

fn ring(n: usize, r: f64, angle: impl Fn(usize) -> f64) -> String {
    let mut d = String::new();
    for i in 0..n {
        let (x, y) = polar(POLAR_CX, POLAR_CY, r, angle(i));
        d.push_str(&format!(
            "{} {},{}",
            if i == 0 { "M" } else { "L" },
            num(x),
            num(y)
        ));
    }
    d.push('Z');
    d
}

/// visx `RadarChart size={320}` with grid, axis, labels, areas and a legend.
fn render_motion(comp: &ComponentNode, label: &str, data: &ChartData) -> String {
    crate::cronus_ui_chart::note_motion();
    let size = attr_num::<f64>(comp, "size").unwrap_or(320.0);
    let margin = attr_num::<f64>(comp, "margin").unwrap_or(60.0);
    let levels = attr_num::<usize>(comp, "levels").unwrap_or(5).max(1);
    let radius = (size - margin * 2.0) / 2.0;
    let n = data.labels.len().max(1);
    let tau = std::f64::consts::TAU;
    let step = tau / n as f64;
    let get_angle = |i: usize| i as f64 * step - std::f64::consts::FRAC_PI_2;
    let mut body = String::new();
    // RadarGrid: LineRadial through n + 1 angles offset by half a step
    // (visx radial: angle from 12 o'clock, reversed by the [360, 0] scale).
    body.push_str("<g class=\"radar-grid\">");
    for l in 0..levels {
        let r = (l as f64 + 1.0) * radius / levels as f64;
        let mut d = String::new();
        for i in 0..=n {
            let deg = i as f64 * (360.0 / n as f64) + 360.0 / n as f64 / 2.0;
            let a = (360.0 - deg) / 360.0 * tau;
            let (x, y) = (r * a.sin(), -r * a.cos());
            d.push_str(&format!(
                "{}{},{}",
                if i == 0 { "M" } else { "L" },
                num(x),
                num(y)
            ));
        }
        body.push_str(&format!(
            "<g class=\"level i{l}\"><path d=\"{d}\" fill=\"none\" stroke=\"var(--cronus-border)\" stroke-linecap=\"round\" stroke-opacity=\"0.6\" stroke-width=\"1\"></path></g>"
        ));
    }
    for l in 0..levels {
        body.push_str(&format!(
            "<g class=\"level-label i{l}\"><text dominant-baseline=\"middle\" fill=\"var(--cronus-fg-muted)\" font-size=\"9\" text-anchor=\"start\" x=\"4\" y=\"{}\">{}</text></g>",
            num(-((l as f64 + 1.0) * radius) / levels as f64),
            num((l as f64 + 1.0) * 100.0 / levels as f64)
        ));
    }
    body.push_str("</g><g class=\"radar-axis\">");
    for i in 0..n {
        let a = get_angle(i);
        body.push_str(&format!(
            "<line class=\"spoke i{i}\" stroke=\"var(--cronus-border)\" stroke-opacity=\"0.6\" stroke-width=\"1\" x1=\"0\" y1=\"0\" x2=\"{}\" y2=\"{}\"></line>",
            num(radius * a.cos()),
            num(radius * a.sin())
        ));
    }
    body.push_str("</g><g class=\"radar-labels\">");
    let font = attr_num::<f64>(comp, "label-size").unwrap_or(11.0);
    let offset = attr_num::<f64>(comp, "label-offset").unwrap_or(24.0);
    for (i, name) in data.labels.iter().enumerate() {
        let a = get_angle(i);
        let lr = radius + offset;
        body.push_str(&format!(
            "<g class=\"metric i{i}\"><text dominant-baseline=\"middle\" fill=\"var(--cronus-fg-tertiary)\" font-size=\"{}\" font-weight=\"500\" text-anchor=\"middle\" x=\"{}\" y=\"{}\">{}</text></g>",
            num(font),
            num(lr * a.cos()),
            num(lr * a.sin()),
            esc(name)
        ));
    }
    body.push_str("</g>");
    for (si, s) in data.series.iter().enumerate() {
        let color = format!("var(--cronus-chart-{})", si % 5 + 1);
        let pts: Vec<(f64, f64)> = (0..n)
            .map(|i| {
                let v = s.values.get(i).copied().unwrap_or(0.0);
                let r = (v / 100.0) * radius;
                let a = get_angle(i);
                (r * a.cos(), r * a.sin())
            })
            .collect();
        let d = format!(
            "M {} Z",
            pts.iter()
                .map(|p| format!("{},{}", num(p.0), num(p.1)))
                .collect::<Vec<_>>()
                .join(" L ")
        );
        let mut points = String::new();
        for p in &pts {
            points.push_str(&format!(
                "<circle class=\"point\" cx=\"{}\" cy=\"{}\" fill=\"{color}\" r=\"4\" stroke=\"var(--cronus-surface-base)\" stroke-width=\"2\"></circle>",
                num(p.0),
                num(p.1)
            ));
        }
        body.push_str(&format!(
            "<g class=\"area i{si}\"><path d=\"{d}\" fill=\"{color}\" fill-opacity=\"0.15\" stroke=\"{color}\" stroke-linejoin=\"round\" stroke-width=\"2\"></path>{points}</g>"
        ));
    }
    let rows: Vec<LegendRow> = data
        .series
        .iter()
        .enumerate()
        .map(|(i, s)| LegendRow {
            label: s.label.clone(),
            value: None,
            color: marker_class(i),
            progress: None,
            grow: true,
        })
        .collect();
    let legend = legend_html(attr(comp, "legend"), &rows, "");
    format!(
        "<div class=\"chart-with-legend\"><div data-slot=\"radar-chart\" class=\"v-motion\" role=\"img\" aria-label=\"{}\"><svg viewBox=\"0 0 {s} {s}\" width=\"{s}\" height=\"{s}\" aria-hidden=\"true\"><g transform=\"translate({c},{c})\">{body}</g></svg></div>{legend}</div>",
        esc(label),
        s = num(size),
        c = num(size / 2.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn fixture() -> ComponentNode {
        let mut c = stub("radar-chart", "Metrics");
        for t in ["Speed", "Reliability", "Comfort"] {
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

    fn metrics(style: &str) -> ComponentNode {
        let mut c = stub("radar-chart", "Product metrics");
        c.style = Some(style.into());
        c.props
            .insert("series".into(), "current:Current,target:Target".into());
        c.props.insert("legend".into(), "Series".into());
        for (m, cu, ta) in [
            ("Speed", "80", "90"),
            ("Reliability", "70", "85"),
            ("Comfort", "60", "75"),
            ("Safety", "90", "95"),
            ("Efficiency", "75", "80"),
        ] {
            let mut it = ComponentItemNode {
                item_type: "item".into(),
                text: m.into(),
                link: None,
                tone: None,
                config: Default::default(),
            };
            it.config.insert("current".into(), cu.into());
            it.config.insert("target".into(), ta.into());
            c.items.push(it);
        }
        c
    }

    #[test]
    fn fixture_matches_recharts_polar_layout() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"radar-chart\" role=\"img\" aria-label=\"Metrics\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\">"));
        assert!(html.contains("<path d=\"M 216,41.9L 290.5648,171.05L 141.4352,171.05Z\" fill=\"none\" stroke=\"#ccc\"></path>"));
        assert!(html.contains("<path class=\"radar-in\" d=\"M216,84.95L290.5648,171.05L160.0764,160.2875L216,84.95Z\" fill=\"var(--cronus-chart-1)\" fill-opacity=\"0.25\""));
        assert!(html.contains("<g><text x=\"216\" y=\"33.9\" text-anchor=\"middle\" fill=\"#808080\"><tspan x=\"216\" dy=\"0em\">Speed</tspan></text></g>"));
        assert!(html.contains("text-anchor=\"start\" fill=\"#808080\"><tspan x=\"297.493\" dy=\"0.355em\">Reliability</tspan>"));
        assert!(html.contains("text-anchor=\"end\" fill=\"#808080\"><tspan x=\"134.507\" dy=\"0.355em\">Comfort</tspan>"));
        assert!(!html.contains("polyline"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn series_prop_draws_one_polygon_per_key() {
        let html = render(&metrics("radar-chart"));
        assert_eq!(html.matches("class=\"radar-in\"").count(), 2);
        assert!(html.contains("fill=\"var(--cronus-chart-2)\" fill-opacity=\"0.25\""));
        assert_eq!(html.matches("stroke=\"#ccc\"></line>").count(), 5);
    }

    #[test]
    fn motion_variant_draws_grid_axis_labels_areas_and_legend() {
        let mut c = metrics("radar-chart+motion");
        c.props.insert("size".into(), "320".into());
        c.props.insert("label-size".into(), "10".into());
        c.props.insert("label-offset".into(), "16".into());
        let html = render(&c);
        assert!(html.starts_with("<div class=\"chart-with-legend\"><div data-slot=\"radar-chart\" class=\"v-motion\" role=\"img\" aria-label=\"Product metrics\"><svg viewBox=\"0 0 320 320\" width=\"320\" height=\"320\" aria-hidden=\"true\"><g transform=\"translate(160,160)\"><g class=\"radar-grid\"><g class=\"level i0\"><path d=\"M-11.7557,-16.1803L-19.0211,6.1803L0,20L19.0211,6.1803L11.7557,-16.1803L-11.7557,-16.1803\" fill=\"none\" stroke=\"var(--cronus-border)\" stroke-linecap=\"round\" stroke-opacity=\"0.6\" stroke-width=\"1\"></path></g>"));
        assert!(html.contains("<g class=\"level-label i4\"><text dominant-baseline=\"middle\" fill=\"var(--cronus-fg-muted)\" font-size=\"9\" text-anchor=\"start\" x=\"4\" y=\"-100\">100</text></g>"));
        assert!(html.contains("<line class=\"spoke i0\" stroke=\"var(--cronus-border)\" stroke-opacity=\"0.6\" stroke-width=\"1\" x1=\"0\" y1=\"0\" x2=\"0\" y2=\"-100\"></line>"));
        assert!(html.contains("<g class=\"metric i0\"><text dominant-baseline=\"middle\" fill=\"var(--cronus-fg-tertiary)\" font-size=\"10\" font-weight=\"500\" text-anchor=\"middle\" x=\"0\" y=\"-116\">Speed</text></g>"));
        // Current: speed 80 → 80px up the first spoke.
        assert!(html.contains("<g class=\"area i0\"><path d=\"M 0,-80 L "));
        assert!(html.contains("<circle class=\"point\" cx=\"0\" cy=\"-80\" fill=\"var(--cronus-chart-1)\" r=\"4\" stroke=\"var(--cronus-surface-base)\" stroke-width=\"2\"></circle>"));
        assert!(html.contains("<g class=\"area i1\"><path d=\"M 0,-90 L "));
        assert!(html.contains("<div class=\"legend-container\"><h3>Series</h3><div class=\"legend-item\"><div class=\"legend-marker c1\"></div><span class=\"legend-label grow\">Current</span></div>"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"radar-chart\"] {\n  display: block; width: 100%; height: 16rem;\n}"
        ));
        assert!(css.contains("[data-slot=\"radar-chart\"].v-motion .area {"));
    }
}
