//! Dedicated AreaChart renderer. DOM matches React:
//! `<div data-slot="area-chart" role="img" aria-label>`
//!   `<div data-slot="chart"><svg viewBox="0 0 432 256">` dashed grid rows,
//!   gradient + monotone area (chart-1), x tick labels (minTickGap 24).
//! Categories are the `text` lines; values cycle 4, 8, 6, 10, 7 unless
//! numeric items are given. Tooltip/cursor need JS and are not emitted.

use crate::cronus_ui_chart::{
    area_paths, categories_or, container, max_of, nice_domain, point_xs, value_grid,
    values_for, x_tick_labels, DEMO_VALUES,
};
use crate::cronus_ui_kit::{label_of, widget_id};
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
    let gid = widget_id(comp, "area-fill");
    let body = format!(
        "{}{}{}",
        value_grid(&ticks, lo, hi),
        area_paths(&xs, &values, lo, hi, "var(--cronus-chart-1)", &gid, "0.4"),
        x_tick_labels(&labels, &xs, 24.0)
    );
    format!(
        "<div data-slot=\"area-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn fixture() -> ComponentNode {
        let mut c = stub("area-chart", "Sessions");
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
    fn fixture_nests_chart_container_with_recharts_geometry() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"area-chart\" role=\"img\" aria-label=\"Sessions\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\">"));
        assert!(html.contains("<line x1=\"8\" y1=\"226\" x2=\"424\" y2=\"226\" stroke-dasharray=\"4 4\"></line>"));
        assert!(html.contains("<line x1=\"8\" y1=\"8\" x2=\"424\" y2=\"8\" stroke-dasharray=\"4 4\"></line>"));
        assert!(html.contains("<path d=\"M8,117C77.3333,62.5,146.6667,8,216,8C285.3333,8,354.6667,35.25,424,62.5\" fill=\"none\" stroke=\"var(--cronus-chart-1)\" stroke-width=\"2\"></path>"));
        assert!(html.contains("stop-opacity=\"0.4\""));
        assert!(html.contains("fill-opacity=\"0.6\""));
        // minTickGap 24 + preserveEnd: Jan clipped at x=8, Mar pulled inside.
        assert!(!html.contains(">Jan</tspan>"));
        assert!(html.contains("<tspan x=\"216\" dy=\"0.71em\">Feb</tspan>"));
        assert!(html.contains(">Mar</tspan>"));
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("polyline"));
    }

    #[test]
    fn numeric_items_drive_values() {
        let mut c = stub("area-chart", "Revenue");
        for t in ["1", "3", "2"] {
            c.items.push(ComponentItemNode {
                item_type: "item".into(),
                text: t.into(),
                link: None,
                tone: None,
                config: Default::default(),
            });
        }
        let html = render(&c);
        // max 3 → nice domain 0..3 (step 0.75): value 1 at y = 226 - 1/3*218, value 3 at the top.
        assert!(html.contains("M8,153.3333C"));
        assert!(html.contains(",216,8C"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&fixture());
            assert!(!html.contains("v-data="));
            assert!(html.contains("data-slot=\"area-chart\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"area-chart\"] {\n  display: block; width: 100%; height: 16rem;\n}"));
        assert!(!css.contains("[data-slot=\"area-chart\"] polyline"));
    }
}
