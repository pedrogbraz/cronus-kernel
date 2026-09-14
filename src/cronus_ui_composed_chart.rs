//! Dedicated ComposedChart renderer. DOM matches React `ComposedChartFixture`:
//! `<div data-slot="composed-chart" role="img" aria-label>`
//!   `<div data-slot="chart"><svg viewBox="0 0 432 256">` dashed grid rows,
//!   an area series (desktop = (n - i) × 4, chart-1, gradient 0.35) and a bar
//!   series (mobile = (i + 1) × 2, chart-2) on a band scale, x tick labels
//!   (minTickGap 24). Categories are the `text` lines.

use crate::cronus_ui_chart::{
    area_paths, band_xs, bars_svg, categories_or, container, max_of, nice_domain, value_grid,
    x_tick_labels,
};
use crate::cronus_ui_kit::{label_of, widget_id};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let cats = categories_or(comp, &["Jan", "Feb", "Mar"]);
    let n = cats.len();
    let desktop: Vec<f64> = (0..n).map(|i| ((n - i) as f64 * 4.0).max(1.0)).collect();
    let mobile: Vec<f64> = (0..n).map(|i| ((i + 1) as f64 * 2.0).max(1.0)).collect();
    let (lo, hi, ticks) = nice_domain(0.0, max_of(&desktop).max(max_of(&mobile)));
    let (centers, band) = band_xs(n);
    let gid = widget_id(comp, "area-fill");
    let body = format!(
        "{}{}{}{}",
        value_grid(&ticks, lo, hi),
        area_paths(
            &centers,
            &desktop,
            lo,
            hi,
            "var(--cronus-chart-1)",
            &gid,
            "0.35"
        ),
        bars_svg(&centers, band, &mobile, lo, hi, "var(--cronus-chart-2)"),
        x_tick_labels(&cats, &centers, 24.0)
    );
    format!(
        "<div data-slot=\"composed-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn fixture() -> ComponentNode {
        let mut c = stub("composed-chart", "Mix");
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
    fn fixture_matches_recharts_area_and_bars() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"composed-chart\" role=\"img\" aria-label=\"Mix\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\">"));
        assert!(html.contains("<path d=\"M77.3333,8C123.5556,32.2222,169.7778,56.4444,216,80.6667C262.2222,104.8889,308.4444,129.1111,354.6667,153.3333\" fill=\"none\" stroke=\"var(--cronus-chart-1)\" stroke-width=\"2\"></path>"));
        assert!(html.contains("stop-opacity=\"0.35\""));
        assert!(html.contains("<path d=\"M 21.8667,193.6667 A 4,4,0,0,1,25.8667,189.6667"));
        assert!(html.contains("<path d=\"M 299.2,121 A 4,4,0,0,1,303.2,117"));
        assert_eq!(html.matches("fill=\"var(--cronus-chart-2)\"").count(), 3);
        for m in ["Jan", "Feb", "Mar"] {
            assert!(html.contains(&format!(">{m}</tspan>")));
        }
        assert!(!html.contains("polyline"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"composed-chart\"] {\n  display: block; width: 100%; height: 16rem;\n}"
        ));
    }
}
