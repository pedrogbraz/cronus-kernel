//! Dedicated HeatmapChart renderer.
//!
//! Default (`HeatmapChart` = `Heatmap` from `@cronus-ui/ui`, docs "Default"):
//! `<div data-slot="heatmap-chart"><div data-slot="heatmap">` › `div[role=img]`
//! grid of `data-slot="heatmap-day"` cells (7 rows, columns flow) bucketed
//! into `data-level` 0..4 against the series max, each titled `value on
//! date` (`start:"2026-06-01"` + index days), and the Less / More legend.
//! Values come from `data:"3,10,5,…"`.
//!
//! Motion (`style:heatmap-chart+motion`, `@cronus-ui/ui/charts`): the visx
//! `HeatmapChart layout="fluid"` — `<div data-slot="heatmap-chart"
//! class="v-motion">` › SVG with the 28 / 16 / 0 / 40 margins, one `rect`
//! per bin (`rows` per column, cell = plot width / columns, 2px gap, rx 2)
//! filled by contribution level (`--chart-scale-01…05`, counts 0 · 1 · 2 · 3
//! · 4+), month labels along the top (`HeatmapXAxis`), Mon / Wed / Fri on
//! the left (`HeatmapYAxis`) and the `HeatmapLegend` swatches below
//! (`legend:true`). Cells fade in with a scattered stagger (1600ms window).

use crate::cronus_ui_chart::{days_from_civil, parse_date, parse_list, short_month};
use crate::cronus_ui_kit::{
    attr, attr_num, choice, esc, fmt_coord, label_of, numeric_items, truthy,
};
use crate::parser::ComponentNode;

const LEVELS: usize = 5;
const FALLBACK: [f64; 4] = [1.0, 3.0, 5.0, 2.0];
const DAY_LABELS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

fn series(comp: &ComponentNode) -> Vec<f64> {
    let mut out = attr(comp, "data").map(parse_list).unwrap_or_default();
    if out.is_empty() {
        out = numeric_items(comp);
    }
    if out.is_empty() {
        out = FALLBACK.to_vec();
    }
    out
}

/// `YYYY-MM-DD` of `start` + `offset` days (2026-06-01 when unset).
fn date_label(start: Option<f64>, offset: usize) -> String {
    let base = start.unwrap_or_else(|| parse_date("2026-06-01").unwrap_or(0.0));
    let days = (base / 86_400_000.0).floor() as i64 + offset as i64;
    let (y, m, d) = crate::cronus_ui_chart::civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

pub fn render(comp: &ComponentNode) -> String {
    let label = attr(comp, "aria-label")
        .map(esc)
        .unwrap_or_else(|| label_of(comp));
    let values = series(comp);
    if choice(comp, "variant", &["motion"]) == Some("motion") {
        return render_motion(comp, &label, &values);
    }
    let start = attr(comp, "start").and_then(parse_date);
    let max = values.iter().copied().fold(0.0_f64, f64::max);
    let mut days = String::new();
    for (index, value) in values.iter().enumerate() {
        let level = heatmap_level(*value, max, LEVELS);
        days.push_str(&format!(
            "<div data-slot=\"heatmap-day\" data-level=\"{level}\" title=\"{v} on {d}\"></div>",
            v = fmt_coord(*value),
            d = date_label(start, index),
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

fn heatmap_level(value: f64, max: f64, levels: usize) -> usize {
    if !value.is_finite() || value <= 0.0 || !max.is_finite() || max <= 0.0 {
        return 0;
    }
    let buckets = levels.saturating_sub(1).max(1);
    let ratio = (value / max).min(1.0);
    let level = (ratio * buckets as f64).ceil() as usize;
    level.clamp(1, buckets)
}

/// `getHeatmapContributionLevel`: 0, 1, 2, 3, 4+.
fn contribution_level(count: f64) -> usize {
    if count <= 0.0 {
        0
    } else {
        (count.round() as usize).min(4)
    }
}

/// Deterministic scatter for the enter stagger (`seededRandom` stand-in).
fn cell_seed(column: usize, row: usize) -> usize {
    let mut h: u32 =
        (column as u32).wrapping_mul(2_654_435_761) ^ (row as u32).wrapping_mul(40_503);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    (h % 10) as usize
}

/// visx `HeatmapChart` (fluid) + axes + legend at the 432px reference width.
fn render_motion(comp: &ComponentNode, label: &str, values: &[f64]) -> String {
    crate::cronus_ui_chart::note_motion();
    let rows = attr_num::<usize>(comp, "rows").unwrap_or(7).max(1);
    let columns = values.len().div_ceil(rows).max(1);
    let (top, right, left) = (28.0, 16.0, 40.0);
    let width = 432.0_f64;
    let iw = width - left - right;
    let cell = iw / columns as f64;
    let gap = 2.0;
    let height = top + rows as f64 * cell;
    let start = attr(comp, "start").and_then(parse_date);
    let base_days = start.map(|s| (s / 86_400_000.0).floor() as i64);
    let mut cells = String::new();
    for c in 0..columns {
        for r in 0..rows {
            let idx = c * rows + r;
            let Some(v) = values.get(idx) else { continue };
            let level = contribution_level(*v);
            cells.push_str(&format!(
                "<rect class=\"visx-heatmap-rect cell s{}\" fill=\"var(--chart-scale-0{})\" height=\"{}\" rx=\"2\" ry=\"2\" width=\"{}\" x=\"{}\" y=\"{}\"></rect>",
                cell_seed(c, r),
                level + 1,
                fmt_coord(cell - gap),
                fmt_coord(cell - gap),
                fmt_coord(c as f64 * cell),
                fmt_coord(r as f64 * cell)
            ));
        }
    }
    // HeatmapXAxis: one label per month change (the column holding the 1st).
    let mut x_axis = String::new();
    if let Some(base) = base_days {
        let mut last = String::new();
        for c in 0..columns {
            let first = base + (c * rows) as i64;
            let anchor = (0..rows)
                .map(|r| first + r as i64)
                .find(|d| crate::cronus_ui_chart::civil_from_days(*d).2 == 1)
                .unwrap_or_else(|| {
                    let (y, m, _) = crate::cronus_ui_chart::civil_from_days(first);
                    days_from_civil(y, m, 1)
                });
            let (y, m, _) = crate::cronus_ui_chart::civil_from_days(anchor);
            let key = format!("{y}-{m}");
            if key == last {
                continue;
            }
            last = key;
            x_axis.push_str(&format!(
                "<text x=\"{}\" y=\"8\" dominant-baseline=\"central\" fill=\"var(--cronus-fg-tertiary)\">{}</text>",
                fmt_coord(left + c as f64 * cell),
                short_month(anchor as f64 * 86_400_000.0)
            ));
        }
    }
    // HeatmapYAxis: odd rows (Mon / Wed / Fri), right-aligned 12px inside the margin.
    let mut y_axis = String::new();
    for r in (1..rows).step_by(2) {
        y_axis.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" text-anchor=\"end\" dominant-baseline=\"central\" fill=\"var(--cronus-fg-tertiary)\">{}</text>",
            fmt_coord(left - 8.0),
            fmt_coord(top + r as f64 * cell + (cell - gap) / 2.0),
            DAY_LABELS[r % 7]
        ));
    }
    let legend = if attr(comp, "legend").map(truthy).unwrap_or(false) {
        let swatches: String = (1..=5)
            .map(|l| format!("<span class=\"swatch l{l}\" aria-hidden=\"true\"></span>"))
            .collect();
        format!("<div class=\"heatmap-legend\"><span>Less</span>{swatches}<span>More</span></div>")
    } else {
        String::new()
    };
    format!(
        "<div data-slot=\"heatmap-chart\" class=\"v-motion\" role=\"img\" aria-label=\"{}\"><div class=\"heatmap-grid\"><svg viewBox=\"0 0 {} {}\" aria-hidden=\"true\"><g class=\"x-axis\">{x_axis}</g><g class=\"y-axis\">{y_axis}</g><g transform=\"translate({},{})\"><g class=\"visx-heatmap-rects\">{cells}</g></g></svg></div>{legend}</div>",
        esc(label),
        fmt_coord(width),
        fmt_coord(height),
        fmt_coord(left),
        fmt_coord(top),
    )
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
    fn data_prop_and_start_date_drive_the_days() {
        let mut c = stub("heatmap-chart", "Calendar heatmap of daily activity.");
        c.props.insert("data".into(), "3,10,5,0".into());
        c.props.insert("start".into(), "2026-06-01".into());
        let html = render(&c);
        assert!(html.contains("data-level=\"2\" title=\"3 on 2026-06-01\""));
        assert!(html.contains("data-level=\"4\" title=\"10 on 2026-06-02\""));
        assert!(html.contains("data-level=\"0\" title=\"0 on 2026-06-04\""));
    }

    #[test]
    fn levels_match_react_buckets() {
        assert_eq!(heatmap_level(0.0, 5.0, 5), 0);
        assert_eq!(heatmap_level(1.0, 5.0, 5), 1);
        assert_eq!(heatmap_level(5.0, 5.0, 5), 4);
        assert_eq!(contribution_level(0.0), 0);
        assert_eq!(contribution_level(3.0), 3);
        assert_eq!(contribution_level(9.0), 4);
    }

    #[test]
    fn motion_variant_lays_out_fluid_cells_axes_and_legend() {
        let mut c = stub("heatmap-chart", "Heatmap chart of daily contributions.");
        c.style = Some("heatmap-chart+motion".into());
        c.props.insert("start".into(), "2024-01-01".into());
        c.props.insert("rows".into(), "7".into());
        c.props.insert("legend".into(), "true".into());
        let counts: Vec<String> = (0..7 * 6)
            .map(|i| ((i / 7) * 3 + (i % 7) * 5) % 5)
            .map(|v| v.to_string())
            .collect();
        c.props.insert("data".into(), counts.join(","));
        let html = render(&c);
        // 6 columns over the 376px plot → 62.67px cells, 2px gap; height 28 + 7 cells.
        assert!(html.starts_with("<div data-slot=\"heatmap-chart\" class=\"v-motion\" role=\"img\" aria-label=\"Heatmap chart of daily contributions.\"><div class=\"heatmap-grid\"><svg viewBox=\"0 0 432 466.67\" aria-hidden=\"true\">"));
        assert!(html.contains("<g transform=\"translate(40,28)\"><g class=\"visx-heatmap-rects\"><rect class=\"visx-heatmap-rect cell s"));
        assert!(html.contains("fill=\"var(--chart-scale-01)\" height=\"60.67\" rx=\"2\" ry=\"2\" width=\"60.67\" x=\"0\" y=\"0\"></rect>"));
        assert!(html.contains("fill=\"var(--chart-scale-05)\""));
        assert_eq!(html.matches("<rect class=").count(), 42);
        // Jan starts in the first column, Feb in the fifth (Jan 29 + 3 days).
        assert!(html.contains("<g class=\"x-axis\"><text x=\"40\" y=\"8\" dominant-baseline=\"central\" fill=\"var(--cronus-fg-tertiary)\">Jan</text><text x=\"290.67\""));
        assert!(html.contains(">Feb</text>"));
        assert!(html.contains("<text x=\"32\" y=\"121\" text-anchor=\"end\" dominant-baseline=\"central\" fill=\"var(--cronus-fg-tertiary)\">Mon</text>"));
        assert!(html.contains(">Fri</text>"));
        assert!(html.ends_with("</svg></div><div class=\"heatmap-legend\"><span>Less</span><span class=\"swatch l1\" aria-hidden=\"true\"></span><span class=\"swatch l2\" aria-hidden=\"true\"></span><span class=\"swatch l3\" aria-hidden=\"true\"></span><span class=\"swatch l4\" aria-hidden=\"true\"></span><span class=\"swatch l5\" aria-hidden=\"true\"></span><span>More</span></div></div>"));
        assert!(!html.contains("style="));
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
        assert!(css.contains("[data-slot=\"heatmap-chart\"].v-motion .cell {"));
    }
}
