//! Dedicated Heatmap renderer. DOM matches React:
//! `<div data-slot="heatmap">` + `data-slot="heatmap-day"` cells and
//! `heatmap-legend`. Not the catalog `chart()` `<figure><figcaption>` stub.
//!
//! Numeric items drive the days. Without any, the renderer shows a demo series
//! identical to the React audit harness fallback (`heatmap-fixture.tsx`
//! `FALLBACK_VALUES`, dated from 2026-06-01): the emitter cannot carry the
//! fixture's `data` array, so both implementations fall back to the same
//! placeholder instead of two unrelated ones.

use crate::cronus_ui_kit::{attr_nonempty, fmt_coord, label_of, numeric_items};
use crate::parser::ComponentNode;

const LEVELS: usize = 5;

/// React `heatmap-fixture.tsx` `FALLBACK_VALUES` (one value per day).
const DEMO_SERIES: [f64; 14] = [
    0.0, 1.0, 4.0, 2.0, 8.0, 3.0, 1.0, 0.0, 2.0, 6.0, 4.0, 1.0, 3.0, 5.0,
];

pub fn render(comp: &ComponentNode) -> String {
    let label = aria_label(comp).unwrap_or_else(|| label_of(comp));
    let items = numeric_items(comp);
    let demo = items.is_empty();
    let series = if demo { DEMO_SERIES.to_vec() } else { items };
    let max = series.iter().copied().fold(0.0_f64, f64::max);
    let mut days = String::new();
    for (index, value) in series.iter().enumerate() {
        let level = heatmap_level(*value, max, LEVELS);
        let date = if demo {
            format!("2026-06-{:02}", index + 1)
        } else {
            iso_date(index)
        };
        days.push_str(&format!(
            "<div data-slot=\"heatmap-day\" data-level=\"{level}\" title=\"{v} on {date}\"></div>",
            v = fmt_coord(*value),
        ));
    }
    let mut swatches = String::new();
    for level in 0..LEVELS {
        swatches.push_str(&format!(
            "<span data-slot=\"heatmap-legend-swatch\" data-level=\"{level}\"></span>"
        ));
    }
    format!(
        "<div data-slot=\"heatmap\"><div role=\"img\" aria-label=\"{label}\">{days}</div><div data-slot=\"heatmap-legend\"><span>Less</span>{swatches}<span>More</span></div></div>"
    )
}

fn aria_label(comp: &ComponentNode) -> Option<String> {
    attr_nonempty(comp, "aria-label").map(|v| crate::cronus_ui_kit::esc(v))
}

/// Bucket `value` into `0..levels-1` against series `max`. Level 0 is empty;
/// positive values spread across the remaining buckets (React `getHeatmapLevel`).
fn heatmap_level(value: f64, max: f64, levels: usize) -> usize {
    if !value.is_finite() || value <= 0.0 || !max.is_finite() || max <= 0.0 {
        return 0;
    }
    let buckets = levels.saturating_sub(1).max(1);
    let ratio = (value / max).min(1.0);
    let level = (ratio * buckets as f64).ceil() as usize;
    level.clamp(1, buckets)
}

fn iso_date(index: usize) -> String {
    let months = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut day_of_year = index;
    let mut month = 0;
    while month < 12 && day_of_year >= months[month] {
        day_of_year -= months[month];
        month += 1;
    }
    if month >= 12 {
        month = 11;
        day_of_year = 30;
    }
    format!("2026-{:02}-{:02}", month + 1, day_of_year + 1)
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
        assert!(!html.contains("<section"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_is_div_with_day_cells_not_figure() {
        let html = render(&stub("heatmap", "Activity"));
        assert!(html
            .starts_with("<div data-slot=\"heatmap\"><div role=\"img\" aria-label=\"Activity\">"));
        assert!(html.contains("data-slot=\"heatmap-legend\""));
        assert!(html.contains(">Less</span>"));
        assert!(html.contains(">More</span>"));
        reject_stub(&html);
    }

    /// Label-only source (the emitted audit fixture) renders the React harness
    /// fallback: 14 days whose levels match `getHeatmapLevel` against max 8.
    #[test]
    fn label_only_renders_react_harness_demo_series() {
        let html = render(&stub("heatmap", "Activity"));
        assert_eq!(html.matches("data-slot=\"heatmap-day\"").count(), 14);
        let levels: Vec<&str> = html
            .split("data-slot=\"heatmap-day\" data-level=\"")
            .skip(1)
            .map(|rest| &rest[..1])
            .collect();
        assert_eq!(
            levels,
            ["0", "1", "2", "1", "4", "2", "1", "0", "1", "3", "2", "1", "2", "3"]
        );
        assert!(html.contains("data-level=\"0\" title=\"0 on 2026-06-01\""));
        assert!(html.contains("data-level=\"3\" title=\"5 on 2026-06-14\""));
        reject_stub(&html);
    }

    #[test]
    fn aria_label_config_wins_over_label() {
        let mut c = stub("heatmap", "Heat");
        c.items[0]
            .config
            .insert("aria-label".into(), "Activity".into());
        let html = render(&c);
        assert!(html.contains("role=\"img\" aria-label=\"Activity\""));
    }

    #[test]
    fn numeric_items_drive_days() {
        let mut c = stub("heatmap", "Activity");
        c.items.push(extra("item", "1"));
        c.items.push(extra("item", "3"));
        c.items.push(extra("item", "2"));
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"heatmap-day\"").count(), 3);
        assert!(html.contains("title=\"1 on 2026-01-01\""));
        assert!(html.contains("title=\"3 on 2026-01-02\""));
        reject_stub(&html);
    }

    #[test]
    fn comma_list_item_is_series() {
        let mut c = stub("heatmap", "Activity");
        c.items.push(extra("item", "4, 8, 6"));
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"heatmap-day\"").count(), 3);
        reject_stub(&html);
    }

    #[test]
    fn legend_has_five_swatches() {
        let html = render(&stub("heatmap", "Activity"));
        assert_eq!(
            html.matches("data-slot=\"heatmap-legend-swatch\"").count(),
            5
        );
        reject_stub(&html);
    }

    #[test]
    fn skips_chart_figure_stub() {
        let html = render(&stub("heatmap", "Activity"));
        let area =
            crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("sankey-chart"))
                .unwrap();
        assert!(area.contains("<figure"));
        assert_ne!(html, area);
        reject_stub(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("heatmap", "Activity"));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"heatmap-day\""));
        });
    }

    /// Geometry parity (Wave 1t): the legend is `text-xs` (12px/16px); the
    /// rule is scoped to `heatmap` so `heatmap-chart` keeps its own block.
    #[test]
    fn chrome_is_token_only_with_text_xs_legend() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"heatmap\"] > [data-slot=\"heatmap-legend\"] { line-height: 1rem; }"
        ));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("grid-auto-flow: column"));
        assert!(!css.contains("zinc-"));
    }

    #[test]
    fn heatmap_level_matches_react_buckets() {
        assert_eq!(heatmap_level(0.0, 10.0, 5), 0);
        assert_eq!(heatmap_level(4.0, 10.0, 5), 2);
        assert_eq!(heatmap_level(8.0, 10.0, 5), 4);
        assert_eq!(heatmap_level(10.0, 10.0, 5), 4);
        assert_eq!(heatmap_level(1.0, 10.0, 5), 1);
    }
}
