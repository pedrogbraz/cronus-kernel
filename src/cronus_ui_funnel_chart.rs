//! Dedicated FunnelChart renderer. DOM matches React's settled render:
//! `<div data-slot="funnel-chart" role="img" aria-label>`
//!   `<div data-slot="chart"><svg viewBox="0 0 432 256">` one trapezoid per
//!   stage in the 422×246 box (margin 5): top width = value / max × 422,
//!   bottom width = next stage (last → 0), fill chart-1..5, recharts stroke
//!   `#fff`; then the LabelList (`position="right"`, `fill-fg text-xs`).
//! Stages are the `text` lines; values are (n - i) × 4 unless numeric items
//! are given. React animates the trapezoids and only mounts the labels when
//! the animation ends; the kernel emits the settled frame.

use crate::cronus_ui_chart::{categories_or, container, num, POLAR_CX};
use crate::cronus_ui_kit::{esc, label_of, numeric_items};
use crate::parser::ComponentNode;

const BOX_X: f64 = 5.0;
const BOX_Y: f64 = 5.0;
const BOX_W: f64 = 422.0;
const BOX_H: f64 = 246.0;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let stages = categories_or(comp, &["Visit", "Signup"]);
    let n = stages.len();
    let mut values = numeric_items(comp);
    if values.len() != n {
        values = (0..n).map(|i| ((n - i) as f64 * 4.0).max(1.0)).collect();
    }
    let max = values.iter().copied().fold(0.0_f64, f64::max).max(f64::MIN_POSITIVE);
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
        let (bl, br) = (BOX_X + (BOX_W - bottom) / 2.0, BOX_X + (BOX_W + bottom) / 2.0);
        shapes.push_str(&format!(
            "<path d=\"M {tl},{y0}L {tr},{y0}L {br},{y1}L {bl},{y1}L {tl},{y0} Z\" fill=\"var(--cronus-chart-{})\" stroke=\"#fff\"></path>",
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

    #[test]
    fn fixture_matches_recharts_trapezoids_and_labels() {
        let html = render(&fixture());
        assert_eq!(
            html,
            "<div data-slot=\"funnel-chart\" role=\"img\" aria-label=\"Pipeline\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\"><path d=\"M 5,5L 427,5L 321.5,128L 110.5,128L 5,5 Z\" fill=\"var(--cronus-chart-1)\" stroke=\"#fff\"></path><path d=\"M 110.5,128L 321.5,128L 216,251L 216,251L 110.5,128 Z\" fill=\"var(--cronus-chart-2)\" stroke=\"#fff\"></path><g><text x=\"379.25\" y=\"66.5\" text-anchor=\"start\"><tspan x=\"379.25\" dy=\"0.355em\">Visit</tspan></text><text x=\"273.75\" y=\"189.5\" text-anchor=\"start\"><tspan x=\"273.75\" dy=\"0.355em\">Signup</tspan></text></g></svg></div></div>"
        );
    }

    #[test]
    fn chrome_colours_labels_with_fg() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"funnel-chart\"] text {\n  fill: var(--cronus-fg);\n}"));
        assert!(!css.contains("[data-slot=\"funnel-chart\"] polygon"));
    }
}
