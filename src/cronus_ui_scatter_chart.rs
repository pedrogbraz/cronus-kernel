//! Dedicated ScatterChart renderer. DOM matches React:
//! `<div data-slot="scatter-chart" role="img" aria-label>`
//!   `<div data-slot="chart"><svg viewBox="0 0 432 256">` dashed grid (rows +
//!   columns), numeric X/Y axes (YAxis width 60 → plot x 68..424), one
//!   circle per point (ZAxis range 60 → r 4.5135, chart-1), X labels then
//!   Y labels. Point i is (i + 1, value cycle 4, 8, 6, 10, 7); the `text`
//!   lines only set the point count.

use crate::cronus_ui_chart::{
    container, grid_cols, grid_rows, item_labels, max_of, nice_domain, num, x_tick_labels_at, y_of,
    DEMO_VALUES, PLOT_B, PLOT_R, PLOT_T, TICK_FILL,
};
use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub const SCATTER_PLOT_L: f64 = 68.0;
pub const SCATTER_R: f64 = 4.5135;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let n = item_labels(comp).len().max(1);
    let n = if item_labels(comp).is_empty() { 3 } else { n };
    let ys_val: Vec<f64> = (0..n).map(|i| DEMO_VALUES[i % DEMO_VALUES.len()]).collect();
    let (xlo, xhi, xticks) = nice_domain(0.0, n as f64);
    let (ylo, yhi, yticks) = nice_domain(0.0, max_of(&ys_val));
    let x_of = |v: f64| SCATTER_PLOT_L + (v - xlo) / (xhi - xlo) * (PLOT_R - SCATTER_PLOT_L);
    let tick_ys: Vec<f64> = yticks.iter().map(|t| y_of(*t, ylo, yhi)).collect();
    let tick_xs: Vec<f64> = xticks.iter().map(|t| x_of(*t)).collect();
    let mut body = String::new();
    body.push_str(&grid_rows(&tick_ys, SCATTER_PLOT_L, PLOT_R));
    body.push_str(&grid_cols(&tick_xs, PLOT_T, PLOT_B));
    for (i, v) in ys_val.iter().enumerate() {
        body.push_str(&format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"var(--cronus-chart-1)\"></circle>",
            num(x_of((i + 1) as f64)),
            num(y_of(*v, ylo, yhi)),
            num(SCATTER_R)
        ));
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

    #[test]
    fn fixture_matches_recharts_numeric_axes() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"scatter-chart\" role=\"img\" aria-label=\"Reach\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\"><line x1=\"68\" y1=\"226\" x2=\"424\" y2=\"226\" stroke-dasharray=\"4 4\"></line>"));
        assert!(html.contains(
            "<line x1=\"157\" y1=\"8\" x2=\"157\" y2=\"226\" stroke-dasharray=\"4 4\"></line>"
        ));
        assert!(html.contains("<circle cx=\"186.6667\" cy=\"117\" r=\"4.5135\" fill=\"var(--cronus-chart-1)\"></circle>"));
        assert!(html.contains("<circle cx=\"305.3333\" cy=\"8\""));
        assert!(html.contains("<circle cx=\"424\" cy=\"62.5\""));
        for t in ["0", "0.75", "1.5", "2.25", "3"] {
            assert!(html.contains(&format!("dy=\"0.71em\">{t}</tspan>")));
        }
        assert!(html.contains("<g><text x=\"60\" y=\"8\" text-anchor=\"end\" fill=\"#666\"><tspan x=\"60\" dy=\"0.355em\">8</tspan></text></g>"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"scatter-chart\"] {\n  display: block; width: 100%; height: 16rem;\n}"
        ));
    }
}
