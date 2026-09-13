//! Dedicated LiveLineChart renderer. DOM matches React:
//! `<div data-slot="live-line-chart">` wrapping an SVG polyline series.
//! Static snapshot of a real series — not the stub `chart()` `<figure><figcaption>`
//! dummy polyline, and no JS ticker.

use crate::cronus_ui_kit::{chart_polyline_points, label_of, numeric_series};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let series = numeric_series(comp);
    let pts = chart_polyline_points(&series);
    format!(
        "<div data-slot=\"live-line-chart\" role=\"img\" aria-label=\"{label}\"><svg viewBox=\"0 0 200 100\" aria-hidden=\"true\"><polyline fill=\"none\" stroke=\"var(--cronus-primary)\" stroke-width=\"2\" points=\"{pts}\"></polyline></svg></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::{chart_polyline_points, stub};
    use crate::parser::ComponentItemNode;

    const STUB_POLYLINE: &str = "0,30 20,22 40,26 60,12 80,16 100,8 120,14";

    fn extra(item_type: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: item_type.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_stub(html: &str) {
        assert!(!html.contains("<figure"));
        assert!(!html.contains("figcaption"));
        assert!(!html.contains(STUB_POLYLINE));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("{ value }"));
    }

    #[test]
    fn root_is_div_with_svg_polyline_not_figure() {
        let html = render(&stub("live-line-chart", "Live"));
        assert!(html.starts_with("<div data-slot=\"live-line-chart\""));
        assert!(html.contains("role=\"img\""));
        assert!(html.contains("aria-label=\"Live\""));
        assert!(html.contains("<svg"));
        assert!(html.contains("<polyline "));
        assert!(html.contains("stroke=\"var(--cronus-primary)\""));
        assert!(html.contains("viewBox=\"0 0 200 100\""));
        reject_stub(&html);
    }

    #[test]
    fn default_series_when_only_label() {
        let html = render(&stub("live-line-chart", "Live"));
        let pts = chart_polyline_points(&[4.0, 8.0, 6.0, 10.0, 7.0]);
        assert!(html.contains(&format!("points=\"{pts}\"")));
        assert_ne!(pts, STUB_POLYLINE);
        reject_stub(&html);
    }

    #[test]
    fn numeric_items_drive_series() {
        let mut c = stub("live-line-chart", "Live");
        c.items.push(extra("item", "1"));
        c.items.push(extra("item", "3"));
        c.items.push(extra("item", "2"));
        let html = render(&c);
        let pts = chart_polyline_points(&[1.0, 3.0, 2.0]);
        assert!(html.contains(&format!("points=\"{pts}\"")));
        let def = chart_polyline_points(&[4.0, 8.0, 6.0, 10.0, 7.0]);
        assert_ne!(pts, def);
        assert!(!html.contains(&format!("points=\"{def}\"")));
        reject_stub(&html);
    }

    #[test]
    fn comma_list_item_is_series() {
        let mut c = stub("live-line-chart", "Live");
        c.items.push(extra("item", "4, 8, 6, 10, 7"));
        let html = render(&c);
        let pts = chart_polyline_points(&[4.0, 8.0, 6.0, 10.0, 7.0]);
        assert!(html.contains(&format!("points=\"{pts}\"")));
        reject_stub(&html);
    }

    #[test]
    fn skips_chart_figure_stub() {
        let html = render(&stub("live-line-chart", "Live"));
        assert!(!html.contains("<figure"));
        assert!(!html.contains("<figcaption"));
        let area =
            crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("sankey-chart"))
                .unwrap();
        assert!(area.contains("<figure"));
        assert!(area.contains("figcaption"));
        assert!(area.contains(STUB_POLYLINE));
        assert_ne!(html, area);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("live-line-chart", "Live"));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"live-line-chart\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"live-line-chart\"]"));
        assert!(css.contains("aspect-ratio: 2 / 1"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
