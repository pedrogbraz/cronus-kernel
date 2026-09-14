//! Dedicated ProfitLossChart renderer. DOM matches React:
//! `<div data-slot="profit-loss-chart" role="img" aria-label>`
//!   `<div data-slot="chart"><svg viewBox="0 0 432 256">` dashed grid rows,
//!   zero ReferenceLine (`var(--cronus-border)`), monotone line (success),
//!   x tick labels. Values cycle -4, 8, 2 unless numeric items are given.

use crate::cronus_ui_chart::{
    categories_or, container, line_path, max_of, min_of, nice_domain, num, point_xs,
    value_grid, values_for, x_tick_labels, y_of, PLOT_L, PLOT_R,
};
use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub const DEMO_PNL: [f64; 3] = [-4.0, 8.0, 2.0];

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let cats = categories_or(comp, &["Jan", "Feb", "Mar"]);
    let values = values_for(comp, cats.len(), &DEMO_PNL);
    let labels: Vec<String> = if values.len() == cats.len() {
        cats
    } else {
        (1..=values.len()).map(|i| i.to_string()).collect()
    };
    let (lo, hi, ticks) = nice_domain(min_of(&values), max_of(&values));
    let xs = point_xs(values.len());
    let zero = y_of(0.0, lo, hi);
    let body = format!(
        "{}<line x1=\"{}\" y1=\"{z}\" x2=\"{}\" y2=\"{z}\" stroke=\"var(--cronus-border)\"></line>{}{}",
        value_grid(&ticks, lo, hi),
        num(PLOT_L),
        num(PLOT_R),
        line_path(&xs, &values, lo, hi, "var(--cronus-success)"),
        x_tick_labels(&labels, &xs, 5.0),
        z = num(zero),
    );
    format!(
        "<div data-slot=\"profit-loss-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn fixture() -> ComponentNode {
        let mut c = stub("profit-loss-chart", "P/L");
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

    #[test]
    fn fixture_draws_zero_line_and_signed_series() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"profit-loss-chart\" role=\"img\" aria-label=\"P/L\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\">"));
        // Domain -4..12 → zero at y 171.5.
        assert!(html.contains("<line x1=\"8\" y1=\"171.5\" x2=\"424\" y2=\"171.5\" stroke=\"var(--cronus-border)\"></line>"));
        assert!(html.contains("d=\"M8,226C77.3333,144.25,146.6667,62.5,216,62.5C285.3333,62.5,354.6667,103.375,424,144.25\" fill=\"none\" stroke=\"var(--cronus-success)\""));
        assert!(!html.contains(">Jan</tspan>"));
        assert!(html.contains(">Feb</tspan>"));
        assert!(html.contains(">Mar</tspan>"));
        assert!(!html.contains("polyline"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"profit-loss-chart\"] {\n  display: block; width: 100%; height: 16rem;\n}"));
    }
}
