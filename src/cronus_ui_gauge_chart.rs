//! Dedicated GaugeChart renderer. DOM matches React:
//! `<div data-slot="gauge-chart">` wrapping an SVG horseshoe arc.
//! Token stroke. Static value ~60. Not the stub `chart()` `<figure><figcaption>`.

use crate::cronus_ui_kit::{
    chart_gauge, fmt_coord, label_of, numeric_items, GAUGE_CX, GAUGE_CY, GAUGE_DEFAULT,
    GAUGE_STROKE, GAUGE_VIEW,
};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let value = value_of(comp);
    let g = chart_gauge(value);
    let sw = fmt_coord(GAUGE_STROKE);
    let cx = fmt_coord(GAUGE_CX);
    let cy = fmt_coord(GAUGE_CY);
    let s = fmt_coord(GAUGE_VIEW);
    let v = fmt_coord(g.value);
    let mut marks = String::new();
    marks.push_str(&format!(
        "<path d=\"{d}\" fill=\"none\" stroke=\"var(--cronus-border)\" stroke-width=\"{sw}\" stroke-linecap=\"round\"></path>",
        d = g.track_d,
    ));
    if !g.value_d.is_empty() {
        marks.push_str(&format!(
            "<path d=\"{d}\" fill=\"none\" stroke=\"var(--cronus-primary)\" stroke-width=\"{sw}\" stroke-linecap=\"round\"></path>",
            d = g.value_d,
        ));
    }
    marks.push_str(&format!(
        "<text x=\"{cx}\" y=\"{cy}\" text-anchor=\"middle\" dominant-baseline=\"middle\" fill=\"var(--cronus-fg)\" font-size=\"16\">{v}</text>"
    ));
    format!(
        "<div data-slot=\"gauge-chart\" role=\"img\" aria-label=\"{label}\"><svg viewBox=\"0 0 {s} {s}\" aria-hidden=\"true\">{marks}</svg></div>"
    )
}

fn value_of(comp: &ComponentNode) -> f64 {
    numeric_items(comp)
        .into_iter()
        .next()
        .unwrap_or(GAUGE_DEFAULT)
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
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_is_div_with_svg_arc_not_figure() {
        let html = render(&stub("gauge-chart", "Score"));
        assert!(html.starts_with("<div data-slot=\"gauge-chart\""));
        assert!(html.contains("role=\"img\""));
        assert!(html.contains("aria-label=\"Score\""));
        assert!(html.contains("<svg"));
        assert!(html.contains("<path "));
        assert!(html.contains(" d=\""));
        assert!(html.contains(" A "));
        assert!(html.contains("stroke=\"var(--cronus-primary)\""));
        assert!(html.contains("stroke=\"var(--cronus-border)\""));
        assert!(html.contains("fill=\"none\""));
        assert!(html.contains("viewBox=\"0 0 100 100\""));
        assert_eq!(html.matches("<path ").count(), 2);
        reject_stub(&html);
    }

    #[test]
    fn default_value_is_sixty() {
        let html = render(&stub("gauge-chart", "Score"));
        let g = chart_gauge(GAUGE_DEFAULT);
        assert_eq!(g.value, 60.0);
        assert!(html.contains(">60</text>"));
        assert!(html.contains(&format!("d=\"{}\"", g.track_d)));
        assert!(html.contains(&format!("d=\"{}\"", g.value_d)));
        assert_ne!(g.value_d, STUB_POLYLINE);
        reject_stub(&html);
    }

    #[test]
    fn numeric_items_drive_value() {
        let mut c = stub("gauge-chart", "Score");
        c.items.push(extra("item", "40"));
        let html = render(&c);
        let g = chart_gauge(40.0);
        assert!(html.contains(">40</text>"));
        assert!(html.contains(&format!("d=\"{}\"", g.value_d)));
        let def = chart_gauge(GAUGE_DEFAULT);
        assert_ne!(g.value_d, def.value_d);
        assert!(!html.contains(&format!("d=\"{}\"", def.value_d)));
        reject_stub(&html);
    }

    #[test]
    fn skips_chart_figure_stub() {
        let html = render(&stub("gauge-chart", "Score"));
        assert!(!html.contains("<figure"));
        assert!(!html.contains("<figcaption"));
        let area =
            crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("sankey-chart"))
                .unwrap();
        assert!(area.contains("<figure"));
        assert!(area.contains("data-slot=\"sankey-chart\""));
        assert!(area.contains(STUB_POLYLINE));
        assert_ne!(html, area);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("gauge-chart", "Score"));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"gauge-chart\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"gauge-chart\"]"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
