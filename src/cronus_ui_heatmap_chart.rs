//! Dedicated HeatmapChart renderer. DOM matches React (chart-catalog alias of
//! Heatmap):
//! `<div data-slot="heatmap-chart">`
//!   `<div data-slot="heatmap">`
//!     `<div role="img" aria-label>` heatmap-day cells (grid-rows-7, flow col)
//!     `<div data-slot="heatmap-legend"><span>Less</span>` 5 swatches `<span>More</span>`
//! Values are the numeric items (fallback 1, 3, 5, 2); aria-label from the
//! `aria-label` prop / item config, else the label.

use crate::cronus_ui_chart::prop;
use crate::cronus_ui_kit::{esc, fmt_coord, label_of, numeric_items};
use crate::parser::ComponentNode;

const LEVELS: usize = 5;
const FALLBACK: [f64; 4] = [1.0, 3.0, 5.0, 2.0];

pub fn render(comp: &ComponentNode) -> String {
    let label = prop(comp, "aria-label")
        .map(esc)
        .unwrap_or_else(|| label_of(comp));
    let mut series = numeric_items(comp);
    if series.is_empty() {
        series = FALLBACK.to_vec();
    }
    let max = series.iter().copied().fold(0.0_f64, f64::max);
    let mut days = String::new();
    for (index, value) in series.iter().enumerate() {
        let level = heatmap_level(*value, max, LEVELS);
        days.push_str(&format!(
            "<div data-slot=\"heatmap-day\" data-level=\"{level}\" title=\"{v} on 2026-06-{d:02}\"></div>",
            v = fmt_coord(*value),
            d = index % 30 + 1,
        ));
    }
    let mut swatches = String::new();
    for level in 0..LEVELS {
        swatches.push_str(&format!(
            "<span data-slot=\"heatmap-legend-swatch\" data-level=\"{level}\"></span>"
        ));
    }
    format!(
        "<div data-slot=\"heatmap-chart\"><div data-slot=\"heatmap\"><div role=\"img\" aria-label=\"{label}\">{days}</div><div data-slot=\"heatmap-legend\"><span>Less</span>{swatches}<span>More</span></div></div></div>"
    )
}

/// Bucket `value` into `0..levels-1` against series `max` (React `getHeatmapLevel`).
fn heatmap_level(value: f64, max: f64, levels: usize) -> usize {
    if !value.is_finite() || value <= 0.0 || !max.is_finite() || max <= 0.0 {
        return 0;
    }
    let buckets = levels.saturating_sub(1).max(1);
    let ratio = (value / max).min(1.0);
    let level = (ratio * buckets as f64).ceil() as usize;
    level.clamp(1, buckets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn fixture() -> ComponentNode {
        let mut c = stub("heatmap-chart", "Activity");
        for t in ["1", "3", "5", "2"] {
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
    fn fixture_wraps_heatmap_like_react() {
        let html = render(&fixture());
        assert_eq!(
            html,
            "<div data-slot=\"heatmap-chart\"><div data-slot=\"heatmap\"><div role=\"img\" aria-label=\"Activity\"><div data-slot=\"heatmap-day\" data-level=\"1\" title=\"1 on 2026-06-01\"></div><div data-slot=\"heatmap-day\" data-level=\"3\" title=\"3 on 2026-06-02\"></div><div data-slot=\"heatmap-day\" data-level=\"4\" title=\"5 on 2026-06-03\"></div><div data-slot=\"heatmap-day\" data-level=\"2\" title=\"2 on 2026-06-04\"></div></div><div data-slot=\"heatmap-legend\"><span>Less</span><span data-slot=\"heatmap-legend-swatch\" data-level=\"0\"></span><span data-slot=\"heatmap-legend-swatch\" data-level=\"1\"></span><span data-slot=\"heatmap-legend-swatch\" data-level=\"2\"></span><span data-slot=\"heatmap-legend-swatch\" data-level=\"3\"></span><span data-slot=\"heatmap-legend-swatch\" data-level=\"4\"></span><span>More</span></div></div></div>"
        );
    }

    #[test]
    fn levels_match_react_buckets() {
        assert_eq!(heatmap_level(0.0, 5.0, 5), 0);
        assert_eq!(heatmap_level(1.0, 5.0, 5), 1);
        assert_eq!(heatmap_level(5.0, 5.0, 5), 4);
    }

    #[test]
    fn dedicated_in_gate() {
        assert_eq!(
            dedicated_fn_name("heatmap-chart"),
            Some("cronus_ui_heatmap_chart::render")
        );
        assert_eq!(
            renderer_kind("heatmap-chart"),
            RendererKind::Dedicated("cronus_ui_heatmap_chart::render")
        );
    }

    #[test]
    fn chrome_grid_has_seven_fr_rows_and_legend_line_height() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"heatmap-chart\"] {\n  display: block; width: 100%;\n}"));
        assert!(css.contains("grid-template-rows: repeat(7, minmax(0, 1fr));"));
        assert!(css
            .contains("font-size: 0.75rem; line-height: 1rem; color: var(--cronus-fg-tertiary);"));
    }
}
