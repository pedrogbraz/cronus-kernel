//! Dedicated RingChart renderer. DOM matches React:
//! `<div data-slot="ring-chart">` wrapping SVG donut arc(s).
//! Not the stub `chart()` `<figure><figcaption>` + dummy polyline.

use crate::cronus_ui_kit::{
    chart_rings, fmt_coord, label_of, numeric_series, RING_CX, RING_CY, RING_VIEW,
};
use crate::parser::ComponentNode;

const STROKES: [&str; 4] = [
    "var(--cronus-primary)",
    "var(--cronus-success)",
    "var(--cronus-warning)",
    "var(--cronus-info)",
];

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let series = numeric_series(comp);
    let mut marks = String::new();
    for (index, ring) in chart_rings(&series).iter().enumerate() {
        let sw = fmt_coord(ring.stroke);
        let r = fmt_coord(ring.r);
        let cx = fmt_coord(RING_CX);
        let cy = fmt_coord(RING_CY);
        let stroke = STROKES[index % STROKES.len()];
        marks.push_str(&format!(
            "<circle cx=\"{cx}\" cy=\"{cy}\" r=\"{r}\" fill=\"none\" stroke=\"var(--cronus-border)\" stroke-width=\"{sw}\"></circle>"
        ));
        if !ring.d.is_empty() {
            marks.push_str(&format!(
                "<path d=\"{d}\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"{sw}\" stroke-linecap=\"round\"></path>",
                d = ring.d,
            ));
        }
    }
    format!(
        "<div data-slot=\"ring-chart\" role=\"img\" aria-label=\"{label}\"><svg viewBox=\"0 0 {s} {s}\" aria-hidden=\"true\">{marks}</svg></div>",
        s = fmt_coord(RING_VIEW),
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
    fn root_is_div_with_svg_donut_arcs_not_figure() {
        let html = render(&stub("ring-chart", "Share"));
        assert!(html.starts_with("<div data-slot=\"ring-chart\""));
        assert!(html.contains("role=\"img\""));
        assert!(html.contains("aria-label=\"Share\""));
        assert!(html.contains("<svg"));
        assert!(html.contains("<circle "));
        assert!(html.contains("<path "));
        assert!(html.contains(" d=\""));
        assert!(html.contains(" A "));
        assert!(html.contains("stroke=\"var(--cronus-primary)\""));
        assert!(html.contains("fill=\"none\""));
        assert!(html.contains("viewBox=\"0 0 100 100\""));
        reject_stub(&html);
    }

    #[test]
    fn default_series_when_only_label() {
        let html = render(&stub("ring-chart", "Share"));
        let rings = chart_rings(&[4.0, 8.0, 6.0, 10.0, 7.0]);
        assert_eq!(rings.len(), 5);
        assert_eq!(html.matches("<circle ").count(), 5);
        assert_eq!(html.matches("<path ").count(), 5);
        assert!(html.contains(&format!("d=\"{}\"", rings[0].d)));
        assert!(html.contains("stroke=\"var(--cronus-success)\""));
        reject_stub(&html);
    }

    #[test]
    fn numeric_items_drive_series() {
        let mut c = stub("ring-chart", "Share");
        c.items.push(extra("item", "1"));
        c.items.push(extra("item", "3"));
        c.items.push(extra("item", "2"));
        let html = render(&c);
        assert_eq!(html.matches("<circle ").count(), 3);
        assert_eq!(html.matches("<path ").count(), 3);
        let rings = chart_rings(&[1.0, 3.0, 2.0]);
        assert!(html.contains(&format!("d=\"{}\"", rings[0].d)));
        let def = chart_rings(&[4.0, 8.0, 6.0, 10.0, 7.0]);
        assert_ne!(rings.len(), def.len());
        reject_stub(&html);
    }

    #[test]
    fn comma_list_item_is_series() {
        let mut c = stub("ring-chart", "Share");
        c.items.push(extra("item", "4, 8, 6"));
        let html = render(&c);
        assert_eq!(html.matches("<path ").count(), 3);
        reject_stub(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("ring-chart", "Share"));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"ring-chart\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"ring-chart\"]"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
