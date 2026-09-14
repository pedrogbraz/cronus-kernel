//! Dedicated LineChart renderer. DOM matches React:
//! `<div data-slot="line-chart" role="img" aria-label>`
//!   `<div data-slot="chart"><svg viewBox="0 0 432 256">` dashed grid rows,
//!   monotone line (chart-1, width 2), x tick labels (minTickGap 24).
//! Categories are the `text` lines; values cycle 4, 8, 6, 10, 7 unless
//! numeric items are given. Tooltip/cursor need JS and are not emitted.

use crate::cronus_ui_chart::{
    categories_or, container, line_path, max_of, nice_domain, point_xs, value_grid, values_for,
    x_tick_labels, DEMO_VALUES,
};
use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let cats = categories_or(comp, &["Jan", "Feb", "Mar"]);
    let values = values_for(comp, cats.len(), &DEMO_VALUES);
    let labels: Vec<String> = if values.len() == cats.len() {
        cats
    } else {
        (1..=values.len()).map(|i| i.to_string()).collect()
    };
    let (lo, hi, ticks) = nice_domain(0.0, max_of(&values));
    let xs = point_xs(values.len());
    let body = format!(
        "{}{}{}",
        value_grid(&ticks, lo, hi),
        line_path(&xs, &values, lo, hi, "var(--cronus-chart-1)"),
        x_tick_labels(&labels, &xs, 24.0)
    );
    format!(
        "<div data-slot=\"line-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn fixture() -> ComponentNode {
        let mut c = stub("line-chart", "Sessions");
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
    fn fixture_nests_chart_container_with_monotone_line() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"line-chart\" role=\"img\" aria-label=\"Sessions\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\">"));
        assert!(html.contains("<path d=\"M8,117C77.3333,62.5,146.6667,8,216,8C285.3333,8,354.6667,35.25,424,62.5\" fill=\"none\" stroke=\"var(--cronus-chart-1)\" stroke-width=\"2\"></path>"));
        assert!(!html.contains(">Jan</tspan>"));
        assert!(html.contains(">Feb</tspan>"));
        assert!(html.contains(">Mar</tspan>"));
        assert!(!html.contains("polyline"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"line-chart\"] {\n  display: block; width: 100%; height: 16rem;\n}"));
    }
}
