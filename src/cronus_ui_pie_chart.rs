//! Dedicated PieChart renderer. DOM matches React:
//! `<div data-slot="pie-chart" role="img" aria-label>`
//!   `<div data-slot="chart"><svg viewBox="0 0 432 256">` one sector path per
//!   slice (centre 216,128, outer radius 80% of 123 = 98.4, start angle 0,
//!   counter-clockwise), colours chart-1..5.
//! Slices are the `text` lines; values cycle 4, 8, 6, 10, 7 unless numeric
//! items are given. Tooltip needs JS and is not emitted.

use crate::cronus_ui_chart::{
    categories_or, container, sector_path, values_for, DEMO_VALUES, POLAR_CX, POLAR_CY,
};
use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub const PIE_OUTER: f64 = 98.4;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let cats = categories_or(comp, &["Desktop", "Mobile"]);
    let values = values_for(comp, cats.len(), &DEMO_VALUES);
    let total: f64 = values.iter().map(|v| v.max(0.0)).sum();
    let mut body = String::new();
    let mut start = 0.0;
    for (i, v) in values.iter().enumerate() {
        let delta = if total > 0.0 { v.max(0.0) / total * 360.0 } else { 0.0 };
        let end = start + delta;
        body.push_str(&format!(
            "<path d=\"{}\" fill=\"var(--cronus-chart-{})\" stroke-width=\"0\"></path>",
            sector_path(POLAR_CX, POLAR_CY, 0.0, PIE_OUTER, start, end),
            i % 5 + 1
        ));
        start = end;
    }
    format!(
        "<div data-slot=\"pie-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn fixture() -> ComponentNode {
        let mut c = stub("pie-chart", "Traffic");
        for t in ["Desktop", "Mobile"] {
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
    fn fixture_matches_recharts_sectors() {
        let html = render(&fixture());
        assert_eq!(
            html,
            "<div data-slot=\"pie-chart\" role=\"img\" aria-label=\"Traffic\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\"><path d=\"M 314.4,128 A 98.4,98.4,0, 0,0, 166.8,42.7831 L 216,128 Z\" fill=\"var(--cronus-chart-1)\" stroke-width=\"0\"></path><path d=\"M 166.8,42.7831 A 98.4,98.4,0, 1,0, 314.4,128 L 216,128 Z\" fill=\"var(--cronus-chart-2)\" stroke-width=\"0\"></path></svg></div></div>"
        );
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"pie-chart\"] {\n  display: block; width: 100%; height: 16rem;\n}"));
        assert!(!css.contains("[data-slot=\"pie-chart\"] svg { width: 12rem"));
    }
}
