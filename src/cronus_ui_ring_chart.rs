//! Dedicated RingChart renderer.
//!
//! Default (recharts donut, docs "Default"): `<div data-slot="ring-chart"
//! role="img" aria-label>` › `<div data-slot="chart"><svg viewBox="0 0 432 256">`
//! sectors 64..98 with a 3° padding angle (chart-1..N, no stroke), the compact
//! total (`Intl compact`, 1 decimal) and `center:"Channels"` in the hub. One
//! sweep mask replays the recharts pie animation (begin 400ms, 1500ms ease).
//!
//! Motion (`style:ring-chart+motion`, `@cronus-ui/ui/charts`): the docs
//! composition — `<div class="chart-with-legend">` holding `<div
//! data-slot="ring-chart" class="v-motion">` (a `size`px SVG; `Ring`s of
//! `stroke-width` 14 and gap 6 from inner radius 60, scaled to fit 8px
//! padding; a `--cronus-border` track + a round-capped progress arc of
//! `value / max` per row, chart-1..N) with the `RingCenter` total and label,
//! plus the shared `Legend` (`legend:"Channels"`: marker, label, value and
//! a progress bar per row). Rings expand from the centre (i × 80ms) and
//! their progress sweeps in from 600ms + i × 100ms.

use crate::cronus_ui_chart::{
    center_stat, chart_data, compact_number, container, d3_arc, data_items, int_fmt, legend_html,
    marker_class, num, sector_path, sweep_mask, ChartData, LegendRow, DEMO_VALUES, POLAR_CX,
    POLAR_CY,
};
use crate::cronus_ui_kit::{attr, attr_num, choice, esc, instance_id, label_of};
use crate::parser::ComponentNode;

pub const RING_INNER: f64 = 64.0;
pub const RING_OUTER: f64 = 98.0;
pub const RING_PADDING: f64 = 3.0;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let data = chart_data(comp, &["Desktop", "Mobile"], &DEMO_VALUES);
    if choice(comp, "variant", &["motion"]) == Some("motion") {
        return render_motion(comp, &label, &data);
    }
    let values = &data.series[0].values;
    let sum: f64 = values.iter().map(|v| v.max(0.0)).sum();
    let non_zero = values.iter().filter(|v| **v > 0.0).count();
    let real_total = 360.0 - non_zero as f64 * RING_PADDING;
    let mask = instance_id(comp, "sweep");
    let mut body = format!(
        "<defs>{}</defs><g mask=\"url(#{mask})\">",
        sweep_mask(&mask, POLAR_CX, POLAR_CY, RING_INNER, RING_OUTER, 0.0, 360.0, false, "")
    );
    let mut prev_end: Option<f64> = None;
    for (i, v) in values.iter().enumerate() {
        let start = prev_end.map_or(0.0, |e| e + if *v > 0.0 { RING_PADDING } else { 0.0 });
        let delta = if sum > 0.0 {
            v.max(0.0) / sum * real_total
        } else {
            0.0
        };
        let end = start + delta;
        body.push_str(&format!(
            "<path d=\"{}\" fill=\"var(--cronus-chart-{})\" stroke-width=\"0\"></path>",
            sector_path(POLAR_CX, POLAR_CY, RING_INNER, RING_OUTER, start, end),
            i % 5 + 1
        ));
        prev_end = Some(end);
    }
    body.push_str("</g>");
    let center = esc(attr(comp, "center")
        .or_else(|| attr(comp, "centerLabel"))
        .unwrap_or("Total"));
    body.push_str(&format!(
        "<text x=\"216\" y=\"128\" text-anchor=\"middle\" dominant-baseline=\"middle\"><tspan x=\"216\" y=\"128\">{}</tspan><tspan x=\"216\" y=\"148\">{center}</tspan></text>",
        compact_number(sum)
    ));
    format!(
        "<div data-slot=\"ring-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

/// visx `RingChart size={280} strokeWidth={14}` + `RingCenter` + `Legend`.
fn render_motion(comp: &ComponentNode, label: &str, data: &ChartData) -> String {
    crate::cronus_ui_chart::note_motion();
    let size = attr_num::<f64>(comp, "size").unwrap_or(280.0);
    let stroke = attr_num::<f64>(comp, "stroke-width").unwrap_or(12.0);
    let ring_gap = attr_num::<f64>(comp, "ring-gap").unwrap_or(6.0);
    let base_inner = attr_num::<f64>(comp, "inner-radius").unwrap_or(60.0);
    let values = &data.series[0].values;
    let maxes: Vec<f64> = data_items(comp)
        .iter()
        .map(|i| {
            i.config
                .get("max")
                .and_then(|m| m.trim().parse::<f64>().ok())
                .unwrap_or(100.0)
        })
        .collect();
    let n = values.len();
    let center = size / 2.0;
    let available = center - 8.0;
    let design_outer = base_inner + (n as f64 - 1.0) * (stroke + ring_gap) + stroke;
    let scale = (available / design_outer).min(1.0);
    let (sw, gap, base) = (stroke * scale, ring_gap * scale, base_inner * scale);
    let half = std::f64::consts::FRAC_PI_2;
    let tau = std::f64::consts::TAU;
    let mut defs = String::new();
    let mut rings = String::new();
    for (i, v) in values.iter().enumerate() {
        let inner = base + i as f64 * (sw + gap);
        let outer = inner + sw;
        let corner = sw / 2.0;
        let max = maxes
            .get(i)
            .copied()
            .unwrap_or(100.0)
            .max(f64::MIN_POSITIVE);
        let progress = (v / max).clamp(0.0, 1.0);
        let end = -half + tau * progress;
        let mask = instance_id(comp, &format!("ring-{i}"));
        defs.push_str(&sweep_mask(
            &mask,
            center,
            center,
            inner,
            outer,
            -180.0,
            360.0 * progress.max(1e-6),
            true,
            &format!("p i{i}"),
        ));
        let bg = d3_arc(inner, outer, -half, 3.0 * half, corner);
        let fg = if end <= -half + 0.01 {
            String::new()
        } else {
            format!(
                "<g mask=\"url(#{mask})\"><path d=\"{}\" fill=\"var(--cronus-chart-{})\"></path></g>",
                d3_arc(inner, outer, -half, end, corner),
                i % 5 + 1
            )
        };
        rings.push_str(&format!(
            "<g class=\"ring i{i}\"><path d=\"{bg}\" fill=\"var(--cronus-border)\"></path>{fg}</g>"
        ));
    }
    let total: f64 = values.iter().sum();
    let hub = center_stat(&int_fmt(total), attr(comp, "center").unwrap_or("Total"));
    let rows: Vec<LegendRow> = data
        .labels
        .iter()
        .zip(values)
        .enumerate()
        .map(|(i, (l, v))| LegendRow {
            label: l.clone(),
            value: Some(int_fmt(*v)),
            color: marker_class(i),
            progress: Some(
                (v / maxes
                    .get(i)
                    .copied()
                    .unwrap_or(100.0)
                    .max(f64::MIN_POSITIVE)
                    * 100.0)
                    .clamp(0.0, 100.0),
            ),
            grow: false,
        })
        .collect();
    let legend = legend_html(attr(comp, "legend"), &rows, "");
    format!(
        "<div class=\"chart-with-legend\"><div data-slot=\"ring-chart\" class=\"v-motion\" role=\"img\" aria-label=\"{}\"><svg viewBox=\"0 0 {s} {s}\" width=\"{s}\" height=\"{s}\" aria-hidden=\"true\"><defs>{defs}</defs><g transform=\"translate({c},{c})\">{rings}</g></svg>{hub}</div>{legend}</div>",
        esc(label),
        s = num(size),
        c = num(center),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn fixture() -> ComponentNode {
        let mut c = stub("ring-chart", "Traffic");
        for t in ["Desktop", "Mobile"] {
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

    fn channels(style: &str) -> ComponentNode {
        let mut c = stub("ring-chart", "Channel mix");
        c.style = Some(style.into());
        c.props.insert("center".into(), "Channels".into());
        c.props.insert("legend".into(), "Channels".into());
        c.props.insert("size".into(), "280".into());
        c.props.insert("stroke-width".into(), "14".into());
        for (l, v) in [
            ("Email", "42"),
            ("Social", "28"),
            ("Direct", "18"),
            ("Other", "12"),
        ] {
            let mut it = ComponentItemNode {
                item_type: "item".into(),
                text: l.into(),
                link: None,
                tone: None,
                config: Default::default(),
            };
            it.config.insert("value".into(), v.into());
            it.config.insert("max".into(), "100".into());
            c.items.push(it);
        }
        c
    }

    #[test]
    fn fixture_matches_recharts_donut_and_centre_label() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"ring-chart\" role=\"img\" aria-label=\"Traffic\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\"><defs><mask id=\"cui-ring-chart-sweep\">"));
        assert!(html.contains("<path d=\"M 314,128 A 98,98,0, 0,0, 169.9918,41.4711 L 185.9538,71.4914 A 64,64,0, 0,1, 280,128 Z\" fill=\"var(--cronus-chart-1)\" stroke-width=\"0\"></path>"));
        assert!(html.contains("<path d=\"M 165.5263,43.9976 A 98,98,0, 1,0, 313.8657,133.1289 L 279.9123,131.3495 A 64,64,0, 1,1, 183.0376,73.1413 Z\" fill=\"var(--cronus-chart-2)\""));
        assert!(html.ends_with("<text x=\"216\" y=\"128\" text-anchor=\"middle\" dominant-baseline=\"middle\"><tspan x=\"216\" y=\"128\">12</tspan><tspan x=\"216\" y=\"148\">Total</tspan></text></svg></div></div>"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn item_values_and_center_prop_drive_the_donut() {
        let html = render(&channels("ring-chart"));
        assert_eq!(html.matches("stroke-width=\"0\"").count(), 4);
        assert!(html.contains(
            "<tspan x=\"216\" y=\"128\">100</tspan><tspan x=\"216\" y=\"148\">Channels</tspan>"
        ));
    }

    #[test]
    fn motion_variant_scales_rings_to_fit_and_adds_legend_progress() {
        let html = render(&channels("ring-chart+motion"));
        assert!(html.starts_with("<div class=\"chart-with-legend\"><div data-slot=\"ring-chart\" class=\"v-motion\" role=\"img\" aria-label=\"Channel mix\"><svg viewBox=\"0 0 280 280\" width=\"280\" height=\"280\" aria-hidden=\"true\"><defs><mask id=\"cui-ring-chart-ring-0\"><circle class=\"sweep p i0\""));
        // 4 rings: design outer 134 → scale 132/134; innermost ring 59.1045..72.8955 (round caps 6.8955).
        assert!(html.contains("<g class=\"ring i0\"><path d=\"M-72.8955,0A72.8955,72.8955,0,1,1,72.8955,0A72.8955,72.8955,0,1,1,-72.8955,0M-59.1045,0A59.1045,59.1045,0,1,0,59.1045,0A59.1045,59.1045,0,1,0,-59.1045,0Z\" fill=\"var(--cronus-border)\"></path><g mask=\"url(#cui-ring-chart-ring-0)\"><path d=\"M"));
        assert!(html.contains("fill=\"var(--cronus-chart-4)\""));
        assert!(html.contains(
            "</svg><div class=\"chart-center\"><span>100</span><span>Channels</span></div></div>"
        ));
        assert!(html.contains("<div class=\"legend-item grid\"><div class=\"legend-marker c1\"></div><span class=\"legend-label\">Email</span><span class=\"legend-value\"><span>42</span></span><div class=\"legend-progress\" role=\"progressbar\" aria-valuenow=\"42\" aria-valuemin=\"0\" aria-valuemax=\"100\"><svg aria-hidden=\"true\"><rect class=\"c1\" height=\"100%\" rx=\"3\" width=\"42%\"></rect></svg></div></div>"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_styles_centre_label() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"ring-chart\"] tspan:first-child {\n  fill: var(--cronus-fg); font-size: 1.5rem; font-weight: 500;\n}"));
        assert!(css.contains("[data-slot=\"ring-chart\"].v-motion .ring {"));
    }
}
