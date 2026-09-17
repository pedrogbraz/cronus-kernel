//! Dedicated PieChart renderer.
//!
//! Default (recharts `PieChart`, docs "Default"): `<div data-slot="pie-chart"
//! role="img" aria-label>` › `<div data-slot="chart"><svg viewBox="0 0 432 256">`
//! one sector per row (`item "Direct" value:320`, chart-1..N, no stroke,
//! outer radius 98.4 at 216,128) sweeping counter-clockwise from 3 o'clock
//! like recharts; a single sweep mask replays the recharts pie animation
//! (begin 400ms, 1500ms ease).
//!
//! Motion (`style:pie-chart+motion`, `@cronus-ui/ui/charts`): the docs
//! composition — `<div class="chart-with-legend">` holding `<div
//! data-slot="pie-chart" class="v-motion">` (a `size`px SVG, d3 pie from 12
//! o'clock (d3 arc: the first slice starts at 9 o'clock), outer radius size / 2 - 10 hover offset, one `PieSlice` path per
//! row revealed by its own sweep mask with a 0.1 + i × 0.08s delay) and the
//! shared `Legend` (`legend:"Traffic"` title, marker + label + value rows).
//! Hovering a slice pops it and fades the others to 0.4.

use crate::cronus_ui_chart::{
    chart_data, container, d3_arc, int_fmt, legend_html, marker_class, sector_path, sweep_mask,
    ChartData, LegendRow, DEMO_VALUES, POLAR_CX, POLAR_CY,
};
use crate::cronus_ui_kit::{attr, attr_num, choice, esc, instance_id, label_of};
use crate::parser::ComponentNode;

pub const PIE_OUTER: f64 = 98.4;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let data = chart_data(comp, &["Desktop", "Mobile"], &DEMO_VALUES);
    if choice(comp, "variant", &["motion"]) == Some("motion") {
        return render_motion(comp, &label, &data);
    }
    let values = &data.series[0].values;
    let total: f64 = values.iter().map(|v| v.max(0.0)).sum();
    let mask = instance_id(comp, "sweep");
    let mut body = format!(
        "<defs>{}</defs><g mask=\"url(#{mask})\">",
        sweep_mask(&mask, POLAR_CX, POLAR_CY, 0.0, PIE_OUTER, 0.0, 360.0, false, "")
    );
    let mut start = 0.0;
    for (i, v) in values.iter().enumerate() {
        let delta = if total > 0.0 {
            v.max(0.0) / total * 360.0
        } else {
            0.0
        };
        let end = start + delta;
        body.push_str(&format!(
            "<path d=\"{}\" fill=\"var(--cronus-chart-{})\" stroke-width=\"0\"></path>",
            sector_path(POLAR_CX, POLAR_CY, 0.0, PIE_OUTER, start, end),
            i % 5 + 1
        ));
        start = end;
    }
    body.push_str("</g>");
    format!(
        "<div data-slot=\"pie-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

/// visx `PieChart size={280}` + `Legend`, the docs "Motion" composition.
fn render_motion(comp: &ComponentNode, label: &str, data: &ChartData) -> String {
    crate::cronus_ui_chart::note_motion();
    let size = attr_num::<f64>(comp, "size").unwrap_or(280.0);
    let center = size / 2.0;
    let outer = center - 10.0;
    let values = &data.series[0].values;
    let total: f64 = values.iter().map(|v| v.max(0.0)).sum();
    let tau = std::f64::consts::TAU;
    let mut defs = String::new();
    let mut slices = String::new();
    let mut a0 = -std::f64::consts::FRAC_PI_2;
    for (i, v) in values.iter().enumerate() {
        let span = if total > 0.0 {
            v.max(0.0) / total * tau
        } else {
            0.0
        };
        let a1 = a0 + span;
        let mask = instance_id(comp, &format!("slice-{i}"));
        defs.push_str(&sweep_mask(
            &mask,
            center,
            center,
            0.0,
            outer,
            (a0 - std::f64::consts::FRAC_PI_2).to_degrees(),
            span.to_degrees(),
            true,
            &format!("i{i}"),
        ));
        slices.push_str(&format!(
            "<g mask=\"url(#{mask})\"><path class=\"slice\" d=\"{}\" fill=\"var(--cronus-chart-{})\"></path></g>",
            d3_arc(0.0, outer, a0, a1, 0.0),
            i % 5 + 1
        ));
        a0 = a1;
    }
    let rows: Vec<LegendRow> = data
        .labels
        .iter()
        .zip(values)
        .enumerate()
        .map(|(i, (l, v))| LegendRow {
            label: l.clone(),
            value: Some(int_fmt(*v)),
            color: marker_class(i),
            progress: None,
            grow: true,
        })
        .collect();
    let legend = legend_html(attr(comp, "legend"), &rows, "");
    format!(
        "<div class=\"chart-with-legend\"><div data-slot=\"pie-chart\" class=\"v-motion\" role=\"img\" aria-label=\"{}\"><svg viewBox=\"0 0 {s} {s}\" width=\"{s}\" height=\"{s}\" aria-hidden=\"true\"><defs>{defs}</defs><g transform=\"translate({c},{c})\">{slices}</g></svg></div>{legend}</div>",
        esc(label),
        s = crate::cronus_ui_chart::num(size),
        c = crate::cronus_ui_chart::num(center),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn fixture() -> ComponentNode {
        let mut c = stub("pie-chart", "Traffic");
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

    fn traffic(style: &str) -> ComponentNode {
        let mut c = stub("pie-chart", "Traffic sources");
        c.style = Some(style.into());
        c.props.insert("legend".into(), "Traffic".into());
        for (l, v) in [
            ("Direct", "320"),
            ("Organic", "280"),
            ("Referral", "190"),
            ("Social", "140"),
        ] {
            let mut it = ComponentItemNode {
                item_type: "item".into(),
                text: l.into(),
                link: None,
                tone: None,
                config: Default::default(),
            };
            it.config.insert("value".into(), v.into());
            c.items.push(it);
        }
        c
    }

    #[test]
    fn fixture_matches_recharts_sectors() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"pie-chart\" role=\"img\" aria-label=\"Traffic\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\"><defs><mask id=\"cui-pie-chart-sweep\"><circle class=\"sweep\" cx=\"216\" cy=\"128\" r=\"49.2\" fill=\"none\" stroke=\"white\" stroke-width=\"100.4\" pathLength=\"1\" transform=\"rotate(0 216 128) matrix(1 0 0 -1 0 256)\"></circle></mask></defs><g mask=\"url(#cui-pie-chart-sweep)\">"));
        assert!(html.contains("<path d=\"M 314.4,128 A 98.4,98.4,0, 0,0, 166.8,42.7831 L 216,128 Z\" fill=\"var(--cronus-chart-1)\" stroke-width=\"0\"></path><path d=\"M 166.8,42.7831 A 98.4,98.4,0, 1,0, 314.4,128 L 216,128 Z\" fill=\"var(--cronus-chart-2)\" stroke-width=\"0\"></path></g></svg></div></div>"));
    }

    #[test]
    fn item_values_drive_the_sectors() {
        let html = render(&traffic("pie-chart"));
        assert_eq!(html.matches("<path d=").count(), 4);
        // Direct 320 of 930 → 123.87° counter-clockwise from 3 o'clock.
        assert!(html.contains("<path d=\"M 314.4,128 A 98.4,98.4,0, 0,0, 161."));
        assert!(html.contains("fill=\"var(--cronus-chart-4)\""));
    }

    #[test]
    fn motion_variant_composes_d3_pie_with_legend() {
        let html = render(&traffic("pie-chart+motion"));
        assert!(html.starts_with("<div class=\"chart-with-legend\"><div data-slot=\"pie-chart\" class=\"v-motion\" role=\"img\" aria-label=\"Traffic sources\"><svg viewBox=\"0 0 280 280\" width=\"280\" height=\"280\" aria-hidden=\"true\"><defs><mask id=\"cui-pie-chart-slice-0\"><circle class=\"sweep i0\" cx=\"140\" cy=\"140\" r=\"65\" fill=\"none\" stroke=\"white\" stroke-width=\"132\" pathLength=\"2.9063\" transform=\"rotate(-180 140 140)\"></circle></mask>"));
        // d3 pie from 12 o'clock: first slice 320/930 of the turn, outer radius 130.
        assert!(html.contains("<g mask=\"url(#cui-pie-chart-slice-0)\"><path class=\"slice\" d=\"M-130,0A130,130,0,0,1,72.452"));
        assert!(html.contains("<g transform=\"translate(140,140)\">"));
        assert!(html.contains("<div class=\"legend-container\"><h3>Traffic</h3><div class=\"legend-item\"><div class=\"legend-marker c1\"></div><span class=\"legend-label grow\">Direct</span><span class=\"legend-value\"><span>320</span></span></div>"));
        assert!(html.ends_with("</div></div>"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"pie-chart\"] {\n  display: block; width: 100%; height: 16rem;\n}"
        ));
        assert!(css.contains("[data-slot=\"pie-chart\"].v-motion .slice {"));
        assert!(!css.contains("[data-slot=\"pie-chart\"] svg { width: 12rem"));
    }
}
