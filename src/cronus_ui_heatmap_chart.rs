//! Dedicated HeatmapChart renderer. DOM matches React:
//! `<div data-slot="heatmap-chart">` wrapping heatmap-day cells + legend.
//! Not the catalog `chart()` `<figure><figcaption>` stub. Distinct from
//! `data-slot="heatmap"`.

use crate::cronus_ui_kit::{fmt_coord, label_of, numeric_series};
use crate::parser::ComponentNode;

const LEVELS: usize = 5;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let series = numeric_series(comp);
    let max = series.iter().copied().fold(0.0_f64, f64::max);
    let mut days = String::new();
    for (index, value) in series.iter().enumerate() {
        let level = heatmap_level(*value, max, LEVELS);
        let date = iso_date(index);
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
        "<div data-slot=\"heatmap-chart\"><div role=\"img\" aria-label=\"{label}\">{days}</div><div data-slot=\"heatmap-legend\"><span>Less</span>{swatches}<span>More</span></div></div>"
    )
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
        assert!(!html.contains("<section"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("{ value }"));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_is_div_with_day_cells_not_figure() {
        let html = render(&stub("heatmap-chart", "Activity"));
        assert!(html.starts_with("<div data-slot=\"heatmap-chart\">"));
        assert!(!html.starts_with("<div data-slot=\"heatmap\">"));
        assert!(html.contains("role=\"img\""));
        assert!(html.contains("aria-label=\"Activity\""));
        assert!(html.contains("data-slot=\"heatmap-day\""));
        assert!(html.matches("data-slot=\"heatmap-day\"").count() >= 5);
        assert!(html.contains("data-slot=\"heatmap-legend\""));
        assert!(html.contains("data-slot=\"heatmap-legend-swatch\""));
        assert!(html.contains(">Less</span>"));
        assert!(html.contains(">More</span>"));
        reject_stub(&html);
    }

    #[test]
    fn default_series_when_only_label() {
        let html = render(&stub("heatmap-chart", "Activity"));
        let series = [4.0, 8.0, 6.0, 10.0, 7.0];
        let max = 10.0;
        assert_eq!(html.matches("data-slot=\"heatmap-day\"").count(), 5);
        for (i, v) in series.iter().enumerate() {
            let level = heatmap_level(*v, max, LEVELS);
            assert!(html.contains(&format!(
                "data-slot=\"heatmap-day\" data-level=\"{level}\" title=\"{} on {}\"",
                fmt_coord(*v),
                iso_date(i)
            )));
        }
        reject_stub(&html);
    }

    #[test]
    fn numeric_items_drive_days() {
        let mut c = stub("heatmap-chart", "Activity");
        c.items.push(extra("item", "1"));
        c.items.push(extra("item", "3"));
        c.items.push(extra("item", "2"));
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"heatmap-day\"").count(), 3);
        assert!(html.contains("title=\"1 on 2026-01-01\""));
        assert!(html.contains("title=\"3 on 2026-01-02\""));
        assert!(html.contains("title=\"2 on 2026-01-03\""));
        reject_stub(&html);
    }

    #[test]
    fn comma_list_item_is_series() {
        let mut c = stub("heatmap-chart", "Activity");
        c.items.push(extra("item", "4, 8, 6"));
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"heatmap-day\"").count(), 3);
        reject_stub(&html);
    }

    #[test]
    fn legend_has_five_swatches() {
        let html = render(&stub("heatmap-chart", "Activity"));
        assert_eq!(html.matches("data-slot=\"heatmap-legend-swatch\"").count(), 5);
        assert!(html.contains("data-level=\"0\""));
        assert!(html.contains("data-level=\"4\""));
        reject_stub(&html);
    }

    #[test]
    fn aria_label_is_escaped() {
        let html = render(&stub("heatmap-chart", "A <B> & \"C\""));
        assert!(html.contains("aria-label=\"A &lt;B&gt; &amp; &quot;C&quot;\""));
        assert!(!html.contains("A <B>"));
        reject_stub(&html);
    }

    #[test]
    fn skips_chart_figure_stub() {
        let html = render(&stub("heatmap-chart", "Activity"));
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
        let via = crate::cronus_ui_widgets::render(&stub("heatmap-chart", "Activity")).unwrap();
        assert_eq!(via, html);
        assert_eq!(
            dedicated_fn_name("heatmap-chart"),
            Some("cronus_ui_heatmap_chart::render")
        );
        assert_eq!(
            renderer_kind("heatmap-chart"),
            RendererKind::Dedicated("cronus_ui_heatmap_chart::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("heatmap-chart", "Activity"));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"heatmap-chart\""));
            assert!(html.contains("data-slot=\"heatmap-day\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"heatmap-chart\"]"));
        assert!(css.contains("[data-slot=\"heatmap-day\"]"));
        assert!(css.contains("[data-slot=\"heatmap-legend\"]"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(css.contains("color-mix"));
        assert!(css.contains("grid-auto-flow: column"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
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
