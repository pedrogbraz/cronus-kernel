//! Dedicated RadarChart renderer. DOM matches React:
//! `<div data-slot="radar-chart">` wrapping SVG polygon/polyline pentagon.
//! Not the stub `chart()` `<figure><figcaption>` + dummy polyline.

use crate::cronus_ui_kit::{
    fmt_coord, label_of, numeric_series, radar_axis_points, radar_grid_points,
    radar_polygon_points, radar_polyline_points, RADAR_AXES, RADAR_LEVELS, RADAR_VIEW,
};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let series = numeric_series(comp);
    let poly = radar_polygon_points(&series);
    let line = radar_polyline_points(&series);
    let mut marks = String::new();
    for lvl in 1..=RADAR_LEVELS {
        let frac = lvl as f64 / RADAR_LEVELS as f64;
        let g = radar_grid_points(frac);
        marks.push_str(&format!(
            "<polyline fill=\"none\" stroke=\"var(--cronus-border)\" stroke-width=\"1\" points=\"{g}\"></polyline>"
        ));
    }
    for i in 0..RADAR_AXES {
        let g = radar_axis_points(i);
        marks.push_str(&format!(
            "<polyline fill=\"none\" stroke=\"var(--cronus-border)\" stroke-width=\"1\" points=\"{g}\"></polyline>"
        ));
    }
    marks.push_str(&format!(
        "<polygon fill=\"var(--cronus-primary)\" fill-opacity=\"0.28\" stroke=\"none\" points=\"{poly}\"></polygon><polyline fill=\"none\" stroke=\"var(--cronus-primary)\" stroke-width=\"2\" points=\"{line}\"></polyline>"
    ));
    format!(
        "<div data-slot=\"radar-chart\" role=\"img\" aria-label=\"{label}\"><svg viewBox=\"0 0 {s} {s}\" aria-hidden=\"true\">{marks}</svg></div>",
        s = fmt_coord(RADAR_VIEW),
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
    fn root_is_div_with_svg_pentagon_not_figure() {
        let html = render(&stub("radar-chart", "Coverage"));
        assert!(html.starts_with("<div data-slot=\"radar-chart\""));
        assert!(html.contains("role=\"img\""));
        assert!(html.contains("aria-label=\"Coverage\""));
        assert!(html.contains("<svg"));
        assert!(html.contains("<polygon "));
        assert!(html.contains("<polyline "));
        assert!(html.contains("fill=\"var(--cronus-primary)\""));
        assert!(html.contains("stroke=\"var(--cronus-primary)\""));
        assert!(html.contains("viewBox=\"0 0 100 100\""));
        assert_eq!(html.matches("<polygon ").count(), 1);
        reject_stub(&html);
    }

    #[test]
    fn default_series_when_only_label() {
        let html = render(&stub("radar-chart", "Coverage"));
        let poly = radar_polygon_points(&[4.0, 8.0, 6.0, 10.0, 7.0]);
        let line = radar_polyline_points(&[4.0, 8.0, 6.0, 10.0, 7.0]);
        assert!(html.contains(&format!("points=\"{poly}\"")));
        assert!(html.contains(&format!("points=\"{line}\"")));
        assert_eq!(poly.split_whitespace().count(), 5);
        reject_stub(&html);
    }

    #[test]
    fn numeric_items_drive_series() {
        let mut c = stub("radar-chart", "Coverage");
        c.items.push(extra("item", "1"));
        c.items.push(extra("item", "3"));
        c.items.push(extra("item", "2"));
        c.items.push(extra("item", "4"));
        c.items.push(extra("item", "5"));
        let html = render(&c);
        let pts = radar_polygon_points(&[1.0, 3.0, 2.0, 4.0, 5.0]);
        assert!(html.contains(&format!("points=\"{pts}\"")));
        let def = radar_polygon_points(&[4.0, 8.0, 6.0, 10.0, 7.0]);
        assert_ne!(pts, def);
        assert!(!html.contains(&format!("points=\"{def}\"")));
        reject_stub(&html);
    }

    #[test]
    fn comma_list_item_is_series() {
        let mut c = stub("radar-chart", "Coverage");
        c.items.push(extra("item", "4, 8, 6, 10, 7"));
        let html = render(&c);
        let pts = radar_polygon_points(&[4.0, 8.0, 6.0, 10.0, 7.0]);
        assert!(html.contains(&format!("points=\"{pts}\"")));
        reject_stub(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("radar-chart", "Coverage"));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"radar-chart\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"radar-chart\"]"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
