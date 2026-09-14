//! Dedicated BarChart renderer. DOM matches React:
//! `<div data-slot="bar-chart" role="img" aria-label>`
//!   `<div data-slot="chart"><svg viewBox="0 0 432 256">` dashed grid rows,
//!   rounded bars (radius 4, barCategoryGap 10%, chart-1), x tick labels.
//! Categories are the `text` lines; values cycle 4, 8, 6, 10, 7 unless
//! numeric items are given. Tooltip/cursor need JS and are not emitted.

use crate::cronus_ui_chart::{
    band_xs, bars_svg, categories_or, container, max_of, nice_domain, value_grid, values_for,
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
    let (centers, band) = band_xs(values.len());
    let body = format!(
        "{}{}{}",
        value_grid(&ticks, lo, hi),
        bars_svg(&centers, band, &values, lo, hi, "var(--cronus-chart-1)"),
        x_tick_labels(&labels, &centers, 5.0)
    );
    format!(
        "<div data-slot=\"bar-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn fixture() -> ComponentNode {
        let mut c = stub("bar-chart", "Sessions");
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
    fn fixture_nests_chart_container_with_recharts_bars() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"bar-chart\" role=\"img\" aria-label=\"Sessions\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\">"));
        assert_eq!(html.matches("stroke-dasharray=\"4 4\"").count(), 5);
        assert!(html.contains("<path d=\"M 21.8667,121 A 4,4,0,0,1,25.8667,117 L 128.8667,117"));
        assert!(html.contains("<path d=\"M 160.5333,12 A 4,4,0,0,1,164.5333,8"));
        assert!(html.contains("<path d=\"M 299.2,66.5 A 4,4,0,0,1,303.2,62.5"));
        for m in ["Jan", "Feb", "Mar"] {
            assert!(html.contains(&format!("dy=\"0.71em\">{m}</tspan>")));
        }
        assert!(!html.contains("style="));
        assert!(!html.contains("<rect"));
    }

    #[test]
    fn category_count_drives_bar_count() {
        let mut c = fixture();
        c.items.push(ComponentItemNode {
            item_type: "text".into(),
            text: "Apr".into(),
            link: None,
            tone: None,
            config: Default::default(),
        });
        let html = render(&c);
        assert_eq!(html.matches("fill=\"var(--cronus-chart-1)\"").count(), 4);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"bar-chart\"] {\n  display: block; width: 100%; height: 16rem;\n}"
        ));
    }
}
