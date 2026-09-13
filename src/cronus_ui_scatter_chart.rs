//! Dedicated ScatterChart renderer. DOM matches React:
//! `<div data-slot="scatter-chart">` wrapping SVG `<circle>` points.
//! Not the stub `chart()` `<figure><figcaption>` + dummy polyline.

use crate::cronus_ui_kit::{chart_line_points, fmt_coord, label_of, numeric_series};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let series = numeric_series(comp);
    let mut circles = String::new();
    for (x, y) in chart_line_points(&series) {
        circles.push_str(&format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"4\" fill=\"var(--cronus-primary)\"></circle>",
            fmt_coord(x),
            fmt_coord(y),
        ));
    }
    format!(
        "<div data-slot=\"scatter-chart\" role=\"img\" aria-label=\"{label}\"><svg viewBox=\"0 0 200 100\" aria-hidden=\"true\">{circles}</svg></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
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
        assert!(!html.contains("{ value }"));
    }

    #[test]
    fn root_is_div_with_svg_circles_not_figure() {
        let html = render(&stub("scatter-chart", "Points"));
        assert!(html.starts_with("<div data-slot=\"scatter-chart\""));
        assert!(html.contains("role=\"img\""));
        assert!(html.contains("aria-label=\"Points\""));
        assert!(html.contains("<svg"));
        assert!(html.contains("<circle "));
        assert!(html.contains("fill=\"var(--cronus-primary)\""));
        assert!(html.contains("viewBox=\"0 0 200 100\""));
        assert_eq!(html.matches("<circle ").count(), 5);
        reject_stub(&html);
    }

    #[test]
    fn default_series_when_only_label() {
        let html = render(&stub("scatter-chart", "Points"));
        let pts = chart_line_points(&[4.0, 8.0, 6.0, 10.0, 7.0]);
        assert_eq!(pts.len(), 5);
        assert!(html.contains(&format!(
            "cx=\"{}\" cy=\"{}\"",
            fmt_coord(pts[0].0),
            fmt_coord(pts[0].1)
        )));
        assert!(html.contains(&format!(
            "cx=\"{}\" cy=\"{}\"",
            fmt_coord(pts[3].0),
            fmt_coord(pts[3].1)
        )));
        reject_stub(&html);
    }

    #[test]
    fn numeric_items_drive_series() {
        let mut c = stub("scatter-chart", "Points");
        c.items.push(extra("item", "1"));
        c.items.push(extra("item", "3"));
        c.items.push(extra("item", "2"));
        let html = render(&c);
        assert_eq!(html.matches("<circle ").count(), 3);
        let pts = chart_line_points(&[1.0, 3.0, 2.0]);
        assert!(html.contains(&format!(
            "cx=\"{}\" cy=\"{}\"",
            fmt_coord(pts[0].0),
            fmt_coord(pts[0].1)
        )));
        let def = chart_line_points(&[4.0, 8.0, 6.0, 10.0, 7.0]);
        assert_ne!(pts.len(), def.len());
        reject_stub(&html);
    }

    #[test]
    fn comma_list_item_is_series() {
        let mut c = stub("scatter-chart", "Points");
        c.items.push(extra("item", "4, 8, 6"));
        let html = render(&c);
        assert_eq!(html.matches("<circle ").count(), 3);
        reject_stub(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("scatter-chart", "Points"));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"scatter-chart\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"scatter-chart\"]"));
        assert!(css.contains("aspect-ratio: 2 / 1"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
