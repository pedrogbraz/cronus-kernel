//! Dedicated FunnelChart renderer.
//!
//! Default (recharts `FunnelChart`, docs "Default"): `<div
//! data-slot="funnel-chart" role="img" aria-label>` › `<div data-slot="chart">
//! <svg viewBox="0 0 432 256">` one trapezoid per stage (`item "Visit"
//! value:8400`, chart-1..N, `#fff` stroke, last shape a triangle) inside the
//! 5px recharts margin and the stage `LabelList` to the right. Trapezoids
//! scale in from their centre (recharts funnel animation, 1500ms ease from
//! 400ms).
//!
//! Motion (`style:funnel-chart+motion`, `@cronus-ui/ui/charts`): the visx
//! `FunnelChart` — `<div data-slot="funnel-chart" class="v-motion">` (aspect
//! 2.2 / 1) › `.segments` flex row with a 4px gap; each `.segment` holds a
//! stretched SVG (`preserveAspectRatio="none"`) with `layers` (3) curved
//! rings (scale 1 - l / layers × 0.35, opacity 0.18 → 0.83, chart-1) that
//! scale in from the left with a 0.12s stagger, and a `.label` cell fading in
//! after it: the `display:` value, the percentage pill and the stage name.
//! Hovering a segment dims the others to 0.4.

use crate::cronus_ui_chart::{
    chart_data, container, data_items, int_fmt, num, ChartData, POLAR_CX,
};
use crate::cronus_ui_kit::{attr_num, choice, esc, label_of};
use crate::parser::ComponentNode;

const BOX_X: f64 = 5.0;
const BOX_Y: f64 = 5.0;
const BOX_W: f64 = 422.0;
const BOX_H: f64 = 246.0;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let data = chart_data(comp, &["Visit", "Signup"], &[8.0, 4.0]);
    if choice(comp, "variant", &["motion"]) == Some("motion") {
        return render_motion(comp, &label, &data);
    }
    let stages = &data.labels;
    let n = stages.len();
    let values = &data.series[0].values;
    let max = values
        .iter()
        .copied()
        .fold(0.0_f64, f64::max)
        .max(f64::MIN_POSITIVE);
    let row = BOX_H / n.max(1) as f64;
    let widths: Vec<f64> = values.iter().map(|v| v.max(0.0) / max * BOX_W).collect();
    let mut shapes = String::new();
    let mut labels = String::new();
    for i in 0..n {
        let top = widths[i];
        let bottom = widths.get(i + 1).copied().unwrap_or(0.0);
        let y0 = BOX_Y + row * i as f64;
        let y1 = y0 + row;
        let (tl, tr) = (BOX_X + (BOX_W - top) / 2.0, BOX_X + (BOX_W + top) / 2.0);
        let (bl, br) = (
            BOX_X + (BOX_W - bottom) / 2.0,
            BOX_X + (BOX_W + bottom) / 2.0,
        );
        shapes.push_str(&format!(
            "<path class=\"pop\" d=\"M {tl},{y0}L {tr},{y0}L {br},{y1}L {bl},{y1}L {tl},{y0} Z\" fill=\"var(--cronus-chart-{})\" stroke=\"#fff\"></path>",
            i % 5 + 1,
            tl = num(tl),
            tr = num(tr),
            bl = num(bl),
            br = num(br),
            y0 = num(y0),
            y1 = num(y1),
        ));
        let lx = POLAR_CX + (top + bottom) / 4.0 + 5.0;
        labels.push_str(&format!(
            "<text x=\"{x}\" y=\"{}\" text-anchor=\"start\"><tspan x=\"{x}\" dy=\"0.355em\">{}</tspan></text>",
            num(y0 + row / 2.0),
            esc(&stages[i]),
            x = num(lx),
        ));
    }
    let body = format!("{shapes}<g>{labels}</g>");
    format!(
        "<div data-slot=\"funnel-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

/// `hSegmentPath` in a stretched 100 × 100 box (control points at 55% / 45%).
fn segment_path(norm_start: f64, norm_end: f64, layer_scale: f64) -> String {
    let my = 50.0;
    let h0 = norm_start * 100.0 * 0.44 * layer_scale;
    let h1 = norm_end * 100.0 * 0.44 * layer_scale;
    format!(
        "M 0 {} C 55 {}, 45 {}, 100 {} L 100 {} C 45 {}, 55 {}, 0 {} Z",
        num(my - h0),
        num(my - h0),
        num(my - h1),
        num(my - h1),
        num(my + h1),
        num(my + h1),
        num(my + h0),
        num(my + h0)
    )
}

/// visx `FunnelChart` (horizontal, curved edges, spread labels).
fn render_motion(comp: &ComponentNode, label: &str, data: &ChartData) -> String {
    crate::cronus_ui_chart::note_motion();
    let layers = attr_num::<usize>(comp, "layers").unwrap_or(3).max(1);
    let values = &data.series[0].values;
    let n = values.len();
    let max = values
        .first()
        .copied()
        .unwrap_or(1.0)
        .max(f64::MIN_POSITIVE);
    let displays: Vec<Option<String>> = data_items(comp)
        .iter()
        .map(|i| i.config.get("display").cloned())
        .collect();
    let mut segments = String::new();
    let mut labels = String::new();
    for i in 0..n {
        let start = values[i] / max;
        let end = values[(i + 1).min(n - 1)] / max;
        let mut rings = String::new();
        for l in 0..layers {
            let scale = 1.0 - (l as f64 / layers as f64) * 0.35;
            let opacity = 0.18 + (l as f64 / (layers as f64 - 1.0).max(1.0)) * 0.65;
            rings.push_str(&format!(
                "<path class=\"ring r{l}\" d=\"{}\" fill=\"var(--cronus-chart-1)\" opacity=\"{}\"></path>",
                segment_path(start, end, scale),
                num(opacity)
            ));
        }
        segments.push_str(&format!(
            "<div class=\"segment i{i}\"><div class=\"enter\"><svg aria-hidden=\"true\" role=\"presentation\" preserveAspectRatio=\"none\" viewBox=\"0 0 100 100\">{rings}</svg></div></div>"
        ));
        let pct = (values[i] / max * 100.0).round();
        let display = displays
            .get(i)
            .cloned()
            .flatten()
            .unwrap_or_else(|| int_fmt(values[i]));
        labels.push_str(&format!(
            "<div class=\"label i{i}\"><div class=\"value\"><span>{}</span></div><div class=\"pct\"><span>{}%</span></div><div class=\"stage\"><span>{}</span></div></div>",
            esc(&display),
            num(pct),
            esc(&data.labels[i])
        ));
    }
    format!(
        "<div data-slot=\"funnel-chart\" class=\"v-motion\" role=\"img\" aria-label=\"{}\"><div class=\"segments\">{segments}</div><div class=\"labels\">{labels}</div></div>",
        esc(label)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn fixture() -> ComponentNode {
        let mut c = stub("funnel-chart", "Pipeline");
        for t in ["Visit", "Signup"] {
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

    fn stages(style: &str) -> ComponentNode {
        let mut c = stub("funnel-chart", "Pipeline");
        c.style = Some(style.into());
        for (l, v, d) in [
            ("Visitors", "12400", "12.4k"),
            ("Leads", "6800", "6.8k"),
            ("Qualified", "3200", "3.2k"),
        ] {
            let mut it = ComponentItemNode {
                item_type: "item".into(),
                text: l.into(),
                link: None,
                tone: None,
                config: Default::default(),
            };
            it.config.insert("value".into(), v.into());
            it.config.insert("display".into(), d.into());
            c.items.push(it);
        }
        c
    }

    #[test]
    fn fixture_matches_recharts_trapezoids_and_labels() {
        let html = render(&fixture());
        assert_eq!(
            html,
            "<div data-slot=\"funnel-chart\" role=\"img\" aria-label=\"Pipeline\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\"><path class=\"pop\" d=\"M 5,5L 427,5L 321.5,128L 110.5,128L 5,5 Z\" fill=\"var(--cronus-chart-1)\" stroke=\"#fff\"></path><path class=\"pop\" d=\"M 110.5,128L 321.5,128L 216,251L 216,251L 110.5,128 Z\" fill=\"var(--cronus-chart-2)\" stroke=\"#fff\"></path><g><text x=\"379.25\" y=\"66.5\" text-anchor=\"start\"><tspan x=\"379.25\" dy=\"0.355em\">Visit</tspan></text><text x=\"273.75\" y=\"189.5\" text-anchor=\"start\"><tspan x=\"273.75\" dy=\"0.355em\">Signup</tspan></text></g></svg></div></div>"
        );
    }

    #[test]
    fn item_values_shape_the_stages() {
        let html = render(&stages("funnel-chart"));
        assert_eq!(html.matches("class=\"pop\"").count(), 3);
        // Leads 6800 of 12400 → 231.42px wide second row.
        assert!(html.contains("L 331.7097,87L 100.2903,87L 5,5 Z"));
    }

    #[test]
    fn motion_variant_stacks_curved_layers_and_labels() {
        let html = render(&stages("funnel-chart+motion"));
        assert!(html.starts_with("<div data-slot=\"funnel-chart\" class=\"v-motion\" role=\"img\" aria-label=\"Pipeline\"><div class=\"segments\"><div class=\"segment i0\"><div class=\"enter\"><svg aria-hidden=\"true\" role=\"presentation\" preserveAspectRatio=\"none\" viewBox=\"0 0 100 100\"><path class=\"ring r0\" d=\"M 0 6 C 55 6, 45 25.871, 100 25.871 L 100 74.129 C 45 74.129, 55 94, 0 94 Z\" fill=\"var(--cronus-chart-1)\" opacity=\"0.18\"></path>"));
        assert!(html.contains("class=\"ring r2\" d=\"M 0 16.2667 C 55 16.2667"));
        assert!(html.contains("opacity=\"0.83\""));
        assert!(html.contains("<div class=\"label i0\"><div class=\"value\"><span>12.4k</span></div><div class=\"pct\"><span>100%</span></div><div class=\"stage\"><span>Visitors</span></div></div>"));
        assert!(html.contains("<div class=\"pct\"><span>55%</span></div>"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_colours_labels_with_fg() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"funnel-chart\"] text {\n  fill: var(--cronus-fg);\n}"));
        assert!(css.contains("[data-slot=\"funnel-chart\"].v-motion .segment {"));
        assert!(!css.contains("[data-slot=\"funnel-chart\"] polygon"));
    }
}
