//! Dedicated LiveLineChart renderer. DOM matches React's first paint:
//! `<div data-slot="live-line-chart" role="img" aria-label>`
//!   `<div data-slot="chart"><svg viewBox="0 0 432 256">` dashed grid rows,
//!   monotone line (chart-1), tick labels (every `text` line is a tick).
//! React appends a random point every `interval` ms with `setInterval`; the
//! zero-JS kernel renders the initial series only (values cycle 4, 8, 6).

use crate::cronus_ui_chart::{
    container, item_labels, line_path, max_of, nice_domain, point_xs, value_grid, x_tick_labels,
    DEMO_VALUES,
};
use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let mut ticks_text = item_labels(comp);
    if ticks_text.is_empty() {
        ticks_text = vec!["0".into(), "1".into(), "2".into()];
    }
    let values: Vec<f64> = (0..ticks_text.len())
        .map(|i| DEMO_VALUES[i % DEMO_VALUES.len()])
        .collect();
    let (lo, hi, ticks) = nice_domain(0.0, max_of(&values));
    let xs = point_xs(values.len());
    let body = format!(
        "{}{}{}",
        value_grid(&ticks, lo, hi),
        line_path(&xs, &values, lo, hi, "var(--cronus-chart-1)"),
        x_tick_labels(&ticks_text, &xs, 5.0)
    );
    format!(
        "<div data-slot=\"live-line-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn fixture() -> ComponentNode {
        let mut c = stub("live-line-chart", "Live");
        for t in ["0", "1", "2"] {
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
    fn fixture_renders_first_paint_series_and_ticks() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"live-line-chart\" role=\"img\" aria-label=\"Live\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\">"));
        assert!(html.contains(
            "d=\"M8,117C77.3333,62.5,146.6667,8,216,8C285.3333,8,354.6667,35.25,424,62.5\""
        ));
        assert!(html.contains("<tspan x=\"8\" dy=\"0.71em\">0</tspan>"));
        assert!(html.contains("<tspan x=\"216\" dy=\"0.71em\">1</tspan>"));
        assert!(html.contains("<tspan x=\"424\" dy=\"0.71em\">2</tspan>"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"live-line-chart\"] {\n  display: block; width: 100%; height: 16rem;\n}"
        ));
    }
}
