//! Dedicated CandlestickChart renderer. DOM matches React:
//! `<div data-slot="candlestick-chart" role="img" aria-label>`
//!   `<div data-slot="chart"><svg viewBox="0 0 432 256">` dashed grid rows for
//!   the hidden YAxis domain `[low - pad, high + pad]` (pad = max(2, 8% span)),
//!   `<g data-slot="candlestick-marks">` with one wick line + body rect per
//!   candle on a point scale (body width clamp(step / 2, 4, 12), rx 1,
//!   success when close ≥ open else error), x tick labels (minTickGap 24).
//! Categories are the `text` lines; OHLC cycles the React fixture candles.
//! The transparent hover hit-rects and tooltip need JS and are not emitted.

use crate::cronus_ui_chart::{
    categories_or, container, fixed_domain_ticks, grid_rows, num, point_xs, x_tick_labels,
    y_of, PLOT_L, PLOT_R,
};
use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

/// open, high, low, close — the React `OHLC_DATA` candles.
pub const DEMO_OHLC: [[f64; 4]; 3] = [[4.0, 8.0, 2.0, 6.0], [6.0, 9.0, 5.0, 5.0], [5.0, 7.0, 3.0, 4.0]];

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let cats = categories_or(comp, &["Mon", "Tue", "Wed"]);
    let n = cats.len();
    let ohlc: Vec<[f64; 4]> = (0..n).map(|i| DEMO_OHLC[i % DEMO_OHLC.len()]).collect();
    let min = ohlc.iter().map(|c| c[2]).fold(f64::INFINITY, f64::min);
    let max = ohlc.iter().map(|c| c[1]).fold(f64::NEG_INFINITY, f64::max);
    let pad = (2.0_f64).max((max - min) * 0.08);
    let (lo, hi) = (min - pad, max + pad);
    let ticks: Vec<f64> = fixed_domain_ticks(lo, hi)
        .iter()
        .map(|t| y_of(*t, lo, hi))
        .collect();
    let xs = point_xs(n);
    let step = (PLOT_R - PLOT_L) / n.max(1) as f64;
    let body_w = (step * 0.5).clamp(4.0, 12.0);
    let mut marks = String::new();
    for (x, [open, high, low, close]) in xs.iter().zip(&ohlc) {
        let up = close >= open;
        let color = if up { "var(--cronus-success)" } else { "var(--cronus-error)" };
        let y_open = y_of(*open, lo, hi);
        let y_close = y_of(*close, lo, hi);
        marks.push_str(&format!(
            "<g><line x1=\"{x}\" y1=\"{}\" x2=\"{x}\" y2=\"{}\" stroke=\"{color}\" stroke-width=\"1\"></line><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"1\" fill=\"{color}\"></rect></g>",
            num(y_of(*high, lo, hi)),
            num(y_of(*low, lo, hi)),
            num(x - body_w / 2.0),
            num(y_open.min(y_close)),
            num(body_w),
            num((y_close - y_open).abs().max(2.0)),
            x = num(*x),
        ));
    }
    let body = format!(
        "{}<g data-slot=\"candlestick-marks\">{marks}</g>{}",
        grid_rows(&ticks, PLOT_L, PLOT_R),
        x_tick_labels(&cats, &xs, 24.0)
    );
    format!(
        "<div data-slot=\"candlestick-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn fixture() -> ComponentNode {
        let mut c = stub("candlestick-chart", "OHLC");
        for t in ["Mon", "Tue", "Wed"] {
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
    fn fixture_matches_recharts_candles() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"candlestick-chart\" role=\"img\" aria-label=\"OHLC\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\">"));
        assert!(html.contains("<line x1=\"8\" y1=\"166.5455\" x2=\"424\" y2=\"166.5455\" stroke-dasharray=\"4 4\"></line>"));
        assert!(html.contains("<g data-slot=\"candlestick-marks\"><g><line x1=\"8\" y1=\"67.4545\" x2=\"8\" y2=\"186.3636\" stroke=\"var(--cronus-success)\" stroke-width=\"1\"></line><rect x=\"2\" y=\"107.0909\" width=\"12\" height=\"39.6364\" rx=\"1\" fill=\"var(--cronus-success)\"></rect></g>"));
        assert!(html.contains("<rect x=\"210\" y=\"107.0909\" width=\"12\" height=\"19.8182\" rx=\"1\" fill=\"var(--cronus-error)\"></rect>"));
        assert!(html.contains("<rect x=\"418\" y=\"126.9091\""));
        assert!(!html.contains(">Mon</tspan>"));
        assert!(html.contains(">Tue</tspan>"));
        assert!(html.contains(">Wed</tspan>"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"candlestick-chart\"] {\n  display: block; width: 100%; height: 16rem;\n}"));
    }
}
