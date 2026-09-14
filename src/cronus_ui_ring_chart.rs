//! Dedicated RingChart renderer. DOM matches React:
//! `<div data-slot="ring-chart" role="img" aria-label>`
//!   `<div data-slot="chart"><svg viewBox="0 0 432 256">` donut sectors
//!   (inner 64, outer 98, paddingAngle 3, chart-1..5) and the centre label
//!   `<text><tspan>{compact total}</tspan><tspan>{centerLabel}</tspan></text>`.
//! Slices are the `text` lines; values cycle 4, 8, 6, 10, 7.

use crate::cronus_ui_chart::{
    categories_or, compact_number, container, prop, sector_path, values_for, DEMO_VALUES,
    POLAR_CX, POLAR_CY,
};
use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

pub const RING_INNER: f64 = 64.0;
pub const RING_OUTER: f64 = 98.0;
pub const RING_PADDING: f64 = 3.0;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let cats = categories_or(comp, &["Desktop", "Mobile"]);
    let values = values_for(comp, cats.len(), &DEMO_VALUES);
    let sum: f64 = values.iter().map(|v| v.max(0.0)).sum();
    let non_zero = values.iter().filter(|v| **v > 0.0).count();
    let real_total = 360.0 - non_zero as f64 * RING_PADDING;
    let mut body = String::new();
    let mut prev_end: Option<f64> = None;
    for (i, v) in values.iter().enumerate() {
        let start = prev_end.map_or(0.0, |e| e + if *v > 0.0 { RING_PADDING } else { 0.0 });
        let delta = if sum > 0.0 { v.max(0.0) / sum * real_total } else { 0.0 };
        let end = start + delta;
        body.push_str(&format!(
            "<path d=\"{}\" fill=\"var(--cronus-chart-{})\" stroke-width=\"0\"></path>",
            sector_path(POLAR_CX, POLAR_CY, RING_INNER, RING_OUTER, start, end),
            i % 5 + 1
        ));
        prev_end = Some(end);
    }
    let center = esc(prop(comp, "centerLabel").unwrap_or("Total"));
    body.push_str(&format!(
        "<text x=\"216\" y=\"128\" text-anchor=\"middle\" dominant-baseline=\"middle\"><tspan x=\"216\" y=\"128\">{}</tspan><tspan x=\"216\" y=\"148\">{center}</tspan></text>",
        compact_number(sum)
    ));
    format!(
        "<div data-slot=\"ring-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
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

    #[test]
    fn fixture_matches_recharts_donut_and_centre_label() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"ring-chart\" role=\"img\" aria-label=\"Traffic\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\">"));
        assert!(html.contains("<path d=\"M 314,128 A 98,98,0, 0,0, 169.9918,41.4711 L 185.9538,71.4914 A 64,64,0, 0,1, 280,128 Z\" fill=\"var(--cronus-chart-1)\" stroke-width=\"0\"></path>"));
        assert!(html.contains("<path d=\"M 165.5263,43.9976 A 98,98,0, 1,0, 313.8657,133.1289 L 279.9123,131.3495 A 64,64,0, 1,1, 183.0376,73.1413 Z\" fill=\"var(--cronus-chart-2)\""));
        assert!(html.ends_with("<text x=\"216\" y=\"128\" text-anchor=\"middle\" dominant-baseline=\"middle\"><tspan x=\"216\" y=\"128\">12</tspan><tspan x=\"216\" y=\"148\">Total</tspan></text></svg></div></div>"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_styles_centre_label() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"ring-chart\"] tspan:first-child {\n  fill: var(--cronus-fg); font-size: 1.5rem; font-weight: 500;\n}"));
    }
}
