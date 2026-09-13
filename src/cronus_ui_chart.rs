//! Dedicated Chart renderer (ChartContainer). DOM matches React:
//! `<div data-slot="chart">` wrapping an SVG of one numeric series.
//! Not the stub `chart()` `<figure><figcaption>` + dummy polyline.

use crate::cronus_ui_kit::{
    chart_base_y, chart_line_points, chart_polyline_points, fmt_coord, label_of, numeric_series,
};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let series = numeric_series(comp);
    let pts = chart_polyline_points(&series);
    let d = area_path(&series);
    format!(
        "<div data-slot=\"chart\" role=\"img\" aria-label=\"{label}\"><svg viewBox=\"0 0 200 100\" aria-hidden=\"true\"><path d=\"{d}\" fill=\"var(--cronus-primary)\"></path><polyline fill=\"none\" stroke=\"var(--cronus-primary)\" stroke-width=\"2\" points=\"{pts}\"></polyline></svg></div>"
    )
}

fn area_path(series: &[f64]) -> String {
    let pts = chart_line_points(series);
    if pts.is_empty() {
        return String::new();
    }
    let mut d = String::new();
    for (i, (x, y)) in pts.iter().enumerate() {
        d.push(if i == 0 { 'M' } else { 'L' });
        d.push_str(&fmt_coord(*x));
        d.push(',');
        d.push_str(&fmt_coord(*y));
    }
    let (x0, _) = pts[0];
    let (xn, _) = pts[pts.len() - 1];
    let base = fmt_coord(chart_base_y());
    d.push('L');
    d.push_str(&fmt_coord(xn));
    d.push(',');
    d.push_str(&base);
    d.push('L');
    d.push_str(&fmt_coord(x0));
    d.push(',');
    d.push_str(&base);
    d.push('Z');
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
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
    fn root_is_div_with_svg_series_not_figure() {
        let html = render(&stub("chart", "Revenue"));
        assert!(html.starts_with("<div data-slot=\"chart\""));
        assert!(!html.starts_with("<div data-slot=\"composed-chart\""));
        assert!(html.contains("role=\"img\""));
        assert!(html.contains("aria-label=\"Revenue\""));
        assert!(html.contains("<svg"));
        assert!(html.contains("<path "));
        assert!(html.contains("<polyline "));
        assert!(html.contains("fill=\"var(--cronus-primary)\""));
        assert!(html.contains("stroke=\"var(--cronus-primary)\""));
        assert!(html.contains("viewBox=\"0 0 200 100\""));
        reject_stub(&html);
    }

    #[test]
    fn default_series_when_only_label() {
        let html = render(&stub("chart", "Revenue"));
        let pts = chart_polyline_points(&[4.0, 8.0, 6.0, 10.0, 7.0]);
        assert!(html.contains(&format!("points=\"{pts}\"")));
        assert!(html.contains(&area_path(&[4.0, 8.0, 6.0, 10.0, 7.0])));
        assert_ne!(pts, STUB_POLYLINE);
        reject_stub(&html);
    }

    #[test]
    fn numeric_items_drive_series() {
        let mut c = stub("chart", "Revenue");
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
        let mut c = stub("chart", "Revenue");
        c.items.push(extra("item", "4, 8, 6, 10, 7"));
        let html = render(&c);
        let pts = chart_polyline_points(&[4.0, 8.0, 6.0, 10.0, 7.0]);
        assert!(html.contains(&format!("points=\"{pts}\"")));
        reject_stub(&html);
    }

    #[test]
    fn aria_label_is_escaped() {
        let html = render(&stub("chart", "A <B> & \"C\""));
        assert!(html.contains("aria-label=\"A &lt;B&gt; &amp; &quot;C&quot;\""));
        assert!(!html.contains("A <B>"));
        reject_stub(&html);
    }

    #[test]
    fn skips_chart_figure_stub() {
        let html = render(&stub("chart", "Revenue"));
        assert!(!html.contains("<figure"));
        assert!(!html.contains("<figcaption"));
        assert!(!html.contains(STUB_POLYLINE));
        let area =
            crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("sankey-chart"))
                .unwrap();
        assert!(area.contains("<figure"));
        assert!(area.contains("data-slot=\"sankey-chart\""));
        assert!(area.contains(STUB_POLYLINE));
        assert_ne!(html, area);
        let via = crate::cronus_ui_widgets::render(&stub("chart", "Revenue")).unwrap();
        assert_eq!(via, html);
        assert_eq!(dedicated_fn_name("chart"), Some("cronus_ui_chart::render"));
        assert_eq!(
            renderer_kind("chart"),
            RendererKind::Dedicated("cronus_ui_chart::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("chart", "Revenue"));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"chart\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"chart\"]"));
        assert!(css.contains("aspect-ratio: 2 / 1"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
