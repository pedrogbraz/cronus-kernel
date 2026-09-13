//! Dedicated CandlestickChart renderer. DOM matches React:
//! `<div data-slot="candlestick-chart">` wrapping SVG candles (rect + wick).
//! Success/error tokens. Not the stub `chart()` dummy polyline.

use crate::cronus_ui_kit::{candle_ohlc, chart_candles, fmt_coord, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let ohlc = candle_ohlc(comp);
    let mut marks = String::new();
    for c in chart_candles(&ohlc) {
        let color = if c.up {
            "var(--cronus-success)"
        } else {
            "var(--cronus-error)"
        };
        marks.push_str(&format!(
            "<line x1=\"{x}\" y1=\"{y0}\" x2=\"{x}\" y2=\"{y1}\" stroke=\"{color}\" stroke-width=\"1\"></line><rect x=\"{bx}\" y=\"{by}\" width=\"{bw}\" height=\"{bh}\" fill=\"{color}\"></rect>",
            x = fmt_coord(c.cx),
            y0 = fmt_coord(c.wick_y0),
            y1 = fmt_coord(c.wick_y1),
            bx = fmt_coord(c.body_x),
            by = fmt_coord(c.body_y),
            bw = fmt_coord(c.body_w),
            bh = fmt_coord(c.body_h),
        ));
    }
    format!(
        "<div data-slot=\"candlestick-chart\" role=\"img\" aria-label=\"{label}\"><svg viewBox=\"0 0 200 100\" aria-hidden=\"true\">{marks}</svg></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::{stub, DEFAULT_CANDLES};
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
    fn root_is_div_with_svg_candles_not_figure() {
        let html = render(&stub("candlestick-chart", "OHLC"));
        assert!(html.starts_with("<div data-slot=\"candlestick-chart\""));
        assert!(html.contains("role=\"img\""));
        assert!(html.contains("aria-label=\"OHLC\""));
        assert!(html.contains("<svg"));
        assert!(html.contains("<rect "));
        assert!(html.contains("<line "));
        assert!(html.contains("fill=\"var(--cronus-success)\""));
        assert!(html.contains("fill=\"var(--cronus-error)\""));
        assert!(html.contains("stroke=\"var(--cronus-success)\""));
        assert!(html.contains("stroke=\"var(--cronus-error)\""));
        assert!(html.contains("viewBox=\"0 0 200 100\""));
        let n = html.matches("<rect ").count();
        assert!(n >= 4 && n <= 6, "candles={n} html={html}");
        assert_eq!(html.matches("<line ").count(), n);
        reject_stub(&html);
    }

    #[test]
    fn default_series_when_only_label() {
        let html = render(&stub("candlestick-chart", "OHLC"));
        let candles = chart_candles(&DEFAULT_CANDLES);
        assert_eq!(candles.len(), 5);
        assert!(candles.iter().any(|c| c.up));
        assert!(candles.iter().any(|c| !c.up));
        assert!(html.contains(&format!(
            "x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"",
            fmt_coord(candles[0].body_x),
            fmt_coord(candles[0].body_y),
            fmt_coord(candles[0].body_w),
            fmt_coord(candles[0].body_h),
        )));
        reject_stub(&html);
    }

    #[test]
    fn numeric_items_drive_series() {
        let mut c = stub("candlestick-chart", "OHLC");
        c.items.push(extra("item", "10"));
        c.items.push(extra("item", "14"));
        c.items.push(extra("item", "8"));
        let html = render(&c);
        assert_eq!(html.matches("<rect ").count(), 3);
        let ohlc = candle_ohlc(&c);
        let candles = chart_candles(&ohlc);
        assert_eq!(candles.len(), 3);
        let def = chart_candles(&DEFAULT_CANDLES);
        assert_ne!(candles.len(), def.len());
        reject_stub(&html);
    }

    #[test]
    fn comma_list_item_is_series() {
        let mut c = stub("candlestick-chart", "OHLC");
        c.items.push(extra("item", "10, 12, 8, 11"));
        let html = render(&c);
        assert_eq!(html.matches("<rect ").count(), 1);
        reject_stub(&html);
    }

    #[test]
    fn skips_chart_figure_stub() {
        let html = render(&stub("candlestick-chart", "OHLC"));
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
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("candlestick-chart", "OHLC"));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"candlestick-chart\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"candlestick-chart\"]"));
        assert!(css.contains("var(--cronus-success)"));
        assert!(css.contains("var(--cronus-error)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
