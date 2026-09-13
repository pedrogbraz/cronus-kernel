//! Dedicated ProfitLossChart renderer. DOM matches React:
//! `<div data-slot="profit-loss-chart">` wrapping SVG polylines split at zero.
//! Green/red via success/error tokens. Not the stub `chart()` dummy polyline.

use crate::cronus_ui_kit::{
    chart_signed_line_points, chart_zero_y, fmt_coord, label_of, numeric_items, CHART_PAD,
    CHART_VIEW_W,
};
use crate::parser::ComponentNode;

const DEFAULT_PNL: [f64; 5] = [4.0, -2.0, 6.0, -1.0, 3.0];

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let series = series_of(comp);
    let marks = pnl_marks(&series);
    format!(
        "<div data-slot=\"profit-loss-chart\" role=\"img\" aria-label=\"{label}\"><svg viewBox=\"0 0 200 100\" aria-hidden=\"true\">{marks}</svg></div>"
    )
}

fn series_of(comp: &ComponentNode) -> Vec<f64> {
    let nums = numeric_items(comp);
    if nums.is_empty() {
        DEFAULT_PNL.to_vec()
    } else {
        nums
    }
}

fn pnl_marks(series: &[f64]) -> String {
    let mut out = String::new();
    let y0 = fmt_coord(chart_zero_y(series));
    let x0 = fmt_coord(CHART_PAD);
    let x1 = fmt_coord(CHART_VIEW_W - CHART_PAD);
    out.push_str(&format!(
        "<line x1=\"{x0}\" y1=\"{y0}\" x2=\"{x1}\" y2=\"{y0}\" stroke=\"var(--cronus-border)\" stroke-width=\"1\"></line>"
    ));
    for (points, positive) in split_pnl(series) {
        let stroke = if positive {
            "var(--cronus-success)"
        } else {
            "var(--cronus-error)"
        };
        out.push_str(&format!(
            "<polyline fill=\"none\" stroke=\"{stroke}\" stroke-width=\"2\" points=\"{points}\"></polyline>"
        ));
    }
    out
}

fn split_pnl(series: &[f64]) -> Vec<(String, bool)> {
    if series.is_empty() {
        return Vec::new();
    }
    let pts = chart_signed_line_points(series);
    let zero_y = chart_zero_y(series);
    let mut sign = initial_sign(series);
    let mut current: Vec<(f64, f64)> = vec![pts[0]];
    let mut out = Vec::new();
    for i in 0..series.len().saturating_sub(1) {
        let ya = series[i];
        let yb = series[i + 1];
        let (x0, y0) = pts[i];
        let (x1, y1) = pts[i + 1];
        if ya != 0.0 && yb != 0.0 && ya.signum() != yb.signum() {
            let t = ya / (ya - yb);
            let xc = x0 + t * (x1 - x0);
            current.push((xc, zero_y));
            out.push((join_pts(&current), sign));
            current = vec![(xc, zero_y), (x1, y1)];
            sign = yb > 0.0;
            continue;
        }
        current.push((x1, y1));
        if yb != 0.0 {
            sign = yb > 0.0;
        }
    }
    if !current.is_empty() {
        out.push((join_pts(&current), sign));
    }
    out
}

fn initial_sign(series: &[f64]) -> bool {
    series
        .iter()
        .find(|v| **v != 0.0)
        .map(|v| *v > 0.0)
        .unwrap_or(true)
}

fn join_pts(pts: &[(f64, f64)]) -> String {
    pts.iter()
        .map(|(x, y)| format!("{},{}", fmt_coord(*x), fmt_coord(*y)))
        .collect::<Vec<_>>()
        .join(" ")
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
    fn root_is_div_with_svg_polylines_not_figure() {
        let html = render(&stub("profit-loss-chart", "P/L"));
        assert!(html.starts_with("<div data-slot=\"profit-loss-chart\""));
        assert!(html.contains("role=\"img\""));
        assert!(html.contains("aria-label=\"P/L\""));
        assert!(html.contains("<svg"));
        assert!(html.contains("<polyline "));
        assert!(html.contains("stroke=\"var(--cronus-success)\""));
        assert!(html.contains("stroke=\"var(--cronus-error)\""));
        assert!(html.contains("stroke=\"var(--cronus-border)\""));
        assert!(html.contains("viewBox=\"0 0 200 100\""));
        assert!(html.matches("<polyline ").count() >= 2);
        reject_stub(&html);
    }

    #[test]
    fn default_series_when_only_label() {
        let html = render(&stub("profit-loss-chart", "P/L"));
        let segs = split_pnl(&DEFAULT_PNL);
        assert!(segs.iter().any(|(_, p)| *p));
        assert!(segs.iter().any(|(_, p)| !*p));
        assert!(html.contains(&format!("points=\"{}\"", segs[0].0)));
        assert_ne!(segs[0].0, STUB_POLYLINE);
        reject_stub(&html);
    }

    #[test]
    fn numeric_items_drive_series() {
        let mut c = stub("profit-loss-chart", "P/L");
        c.items.push(extra("item", "1"));
        c.items.push(extra("item", "-3"));
        c.items.push(extra("item", "2"));
        let html = render(&c);
        let segs = split_pnl(&[1.0, -3.0, 2.0]);
        assert!(html.contains(&format!("points=\"{}\"", segs[0].0)));
        let def = split_pnl(&DEFAULT_PNL);
        assert_ne!(segs[0].0, def[0].0);
        assert!(!html.contains(&format!("points=\"{}\"", def[0].0)));
        assert!(html.contains("stroke=\"var(--cronus-success)\""));
        assert!(html.contains("stroke=\"var(--cronus-error)\""));
        reject_stub(&html);
    }

    #[test]
    fn comma_list_item_is_series() {
        let mut c = stub("profit-loss-chart", "P/L");
        c.items.push(extra("item", "4, -2, 6, -1, 3"));
        let html = render(&c);
        let segs = split_pnl(&DEFAULT_PNL);
        assert!(html.contains(&format!("points=\"{}\"", segs[0].0)));
        reject_stub(&html);
    }

    #[test]
    fn skips_chart_figure_stub() {
        let html = render(&stub("profit-loss-chart", "P/L"));
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
            let html = render(&stub("profit-loss-chart", "P/L"));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"profit-loss-chart\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"profit-loss-chart\"]"));
        assert!(css.contains("aspect-ratio: 2 / 1"));
        assert!(css.contains("var(--cronus-success)"));
        assert!(css.contains("var(--cronus-error)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
