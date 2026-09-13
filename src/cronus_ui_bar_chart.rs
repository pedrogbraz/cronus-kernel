//! Dedicated BarChart renderer. DOM matches React:
//! `<div data-slot="bar-chart">` wrapping an SVG rect series.
//! Not the stub `chart()` `<figure><figcaption>` + flex bar heights.

use crate::cronus_ui_kit::{chart_bars, fmt_coord, label_of, numeric_series};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let series = numeric_series(comp);
    let mut rects = String::new();
    for b in chart_bars(&series) {
        rects.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"2\" fill=\"var(--cronus-primary)\"></rect>",
            fmt_coord(b.x),
            fmt_coord(b.y),
            fmt_coord(b.w),
            fmt_coord(b.h),
        ));
    }
    format!(
        "<div data-slot=\"bar-chart\" role=\"img\" aria-label=\"{label}\"><svg viewBox=\"0 0 200 100\" aria-hidden=\"true\">{rects}</svg></div>"
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
        assert!(!html.contains("display:flex;align-items:flex-end"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("{ value }"));
    }

    #[test]
    fn root_is_div_with_svg_rects_not_figure() {
        let html = render(&stub("bar-chart", "Revenue"));
        assert!(html.starts_with("<div data-slot=\"bar-chart\""));
        assert!(html.contains("role=\"img\""));
        assert!(html.contains("aria-label=\"Revenue\""));
        assert!(html.contains("<svg"));
        assert!(html.contains("<rect "));
        assert!(html.contains("fill=\"var(--cronus-primary)\""));
        assert!(html.contains("viewBox=\"0 0 200 100\""));
        assert_eq!(html.matches("<rect ").count(), 5);
        reject_stub(&html);
    }

    #[test]
    fn default_series_when_only_label() {
        let html = render(&stub("bar-chart", "Revenue"));
        let bars = chart_bars(&[4.0, 8.0, 6.0, 10.0, 7.0]);
        assert_eq!(bars.len(), 5);
        assert!(html.contains(&format!("height=\"{}\"", fmt_coord(bars[0].h))));
        assert!(html.contains(&format!("height=\"{}\"", fmt_coord(bars[3].h))));
        reject_stub(&html);
    }

    #[test]
    fn numeric_items_drive_series() {
        let mut c = stub("bar-chart", "Revenue");
        c.items.push(extra("item", "1"));
        c.items.push(extra("item", "3"));
        c.items.push(extra("item", "2"));
        let html = render(&c);
        assert_eq!(html.matches("<rect ").count(), 3);
        let bars = chart_bars(&[1.0, 3.0, 2.0]);
        assert!(html.contains(&format!(
            "x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"",
            fmt_coord(bars[0].x),
            fmt_coord(bars[0].y),
            fmt_coord(bars[0].w),
            fmt_coord(bars[0].h),
        )));
        let def = chart_bars(&[4.0, 8.0, 6.0, 10.0, 7.0]);
        assert_ne!(bars.len(), def.len());
        reject_stub(&html);
    }

    #[test]
    fn comma_list_item_is_series() {
        let mut c = stub("bar-chart", "Revenue");
        c.items.push(extra("item", "4, 8, 6"));
        let html = render(&c);
        assert_eq!(html.matches("<rect ").count(), 3);
        reject_stub(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("bar-chart", "Revenue"));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"bar-chart\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"bar-chart\"]"));
        assert!(css.contains("aspect-ratio: 2 / 1"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
