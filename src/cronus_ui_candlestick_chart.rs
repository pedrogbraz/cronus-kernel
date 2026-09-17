//! Dedicated CandlestickChart renderer.
//!
//! Default (recharts `ComposedChart` + `CandleMarks`, docs "Default"): `<div
//! data-slot="candlestick-chart" role="img" aria-label>` › `<div
//! data-slot="chart"><svg viewBox="0 0 432 256">` dashed grid rows (hidden y
//! axis, OHLC domain padded by max(2, 8%)), one `<g>` per candle with a 1px
//! wick and a `rx="1"` body (`--cronus-success` up, `--cronus-error` down,
//! width clamp(step / 2, 4, 12)), x tick labels (minTickGap 24).
//!
//! Motion (`style:candlestick-chart+motion`, `@cronus-ui/ui/charts`): the visx
//! `CandlestickChart` — `<div data-slot="candlestick-chart" class="v-motion">`
//! with the docs margins (16 / 16 / 40 / 16) and `height:320`, slot = width /
//! n, candle width slot × 0.8, wick 1.5px, body `rx="1"` + 1px stroke, time
//! scale padded by half a slot, `XAxis` labels. Candles pop in (opacity +
//! scaleY spring) with a 60% stagger; hovering one fades the rest to
//! `faded-opacity` (0.25).
//!
//! Data: `item "Aug 3" open:142 high:148 low:139 close:146` (ISO texts → time
//! axis with `Jan 1` labels).

use crate::cronus_ui_chart::{
    container, d3_nice, data_items, data_ticks, fixed_domain_ticks, grid_rows, motion_x_labels,
    num, point_xs, row_label, row_times, time_xs, x_tick_labels, y_of, Frame, M_W, PLOT_L, PLOT_R,
};
use crate::cronus_ui_kit::{attr_num, choice, label_of};
use crate::parser::ComponentNode;

pub const DEMO_OHLC: [[f64; 4]; 3] = [
    [4.0, 8.0, 2.0, 6.0],
    [6.0, 9.0, 5.0, 5.0],
    [5.0, 7.0, 3.0, 4.0],
];

/// `(label, [open, high, low, close])` per item; demo candles when no
/// item carries OHLC config.
pub fn candles(comp: &ComponentNode) -> Vec<(String, [f64; 4])> {
    let items = data_items(comp);
    let read = |i: &crate::parser::ComponentItemNode, k: &str| {
        i.config.get(k).and_then(|v| v.trim().parse::<f64>().ok())
    };
    let parsed: Vec<(String, [f64; 4])> = items
        .iter()
        .filter_map(|i| {
            Some((
                i.text.trim().to_string(),
                [
                    read(i, "open")?,
                    read(i, "high")?,
                    read(i, "low")?,
                    read(i, "close")?,
                ],
            ))
        })
        .collect();
    if !parsed.is_empty() {
        return parsed;
    }
    let labels: Vec<String> = if items.is_empty() {
        ["Mon", "Tue", "Wed"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    } else {
        items.iter().map(|i| i.text.trim().to_string()).collect()
    };
    labels
        .into_iter()
        .enumerate()
        .map(|(i, l)| (l, DEMO_OHLC[i % DEMO_OHLC.len()]))
        .collect()
}

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let rows = candles(comp);
    if choice(comp, "variant", &["motion"]) == Some("motion") {
        return render_motion(comp, &label, &rows);
    }
    let n = rows.len();
    let min = rows.iter().map(|c| c.1[2]).fold(f64::INFINITY, f64::min);
    let max = rows
        .iter()
        .map(|c| c.1[1])
        .fold(f64::NEG_INFINITY, f64::max);
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
    for (x, (_, [open, high, low, close])) in xs.iter().zip(&rows) {
        let up = close >= open;
        let color = if up {
            "var(--cronus-success)"
        } else {
            "var(--cronus-error)"
        };
        let y_open = y_of(*open, lo, hi);
        let y_close = y_of(*close, lo, hi);
        marks.push_str(&format!(
            "<g class=\"pop\"><line x1=\"{x}\" y1=\"{}\" x2=\"{x}\" y2=\"{}\" stroke=\"{color}\" stroke-width=\"1\"></line><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"1\" fill=\"{color}\"></rect></g>",
            num(y_of(*high, lo, hi)),
            num(y_of(*low, lo, hi)),
            num(x - body_w / 2.0),
            num(y_open.min(y_close)),
            num(body_w),
            num((y_close - y_open).abs().max(2.0)),
            x = num(*x),
        ));
    }
    let labels: Vec<String> = rows.iter().map(|r| r.0.clone()).collect();
    let body = format!(
        "{}<g data-slot=\"candlestick-marks\">{marks}</g>{}",
        grid_rows(&ticks, PLOT_L, PLOT_R),
        x_tick_labels(&labels, &xs, 24.0)
    );
    format!(
        "<div data-slot=\"candlestick-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

/// visx `CandlestickChart` with the docs margins at the 432px reference width.
fn render_motion(comp: &ComponentNode, label: &str, rows: &[(String, [f64; 4])]) -> String {
    crate::cronus_ui_chart::note_motion();
    let height = attr_num::<f64>(comp, "height").unwrap_or(M_W / 2.0);
    let frame = Frame::new(M_W, height).margins(16.0, 16.0, 40.0, 16.0);
    let (iw, ih) = (frame.iw(), frame.ih());
    let n = rows.len().max(1);
    let slot = iw / n as f64;
    let labels: Vec<String> = rows.iter().map(|r| r.0.clone()).collect();
    let times = row_times(&labels);
    let xs = time_xs(&times, slot / 2.0, iw - slot / 2.0);
    let min = rows.iter().map(|c| c.1[2]).fold(f64::INFINITY, f64::min);
    let max = rows
        .iter()
        .map(|c| c.1[1])
        .fold(f64::NEG_INFINITY, f64::max);
    let pad = {
        let p = (max - min) * 0.05;
        if p == 0.0 {
            1.0
        } else {
            p
        }
    };
    let (lo, hi) = d3_nice(min - pad, max + pad, 10.0);
    let y = |v: f64| ih - (v - lo) / (hi - lo) * ih;
    let gap = attr_num::<f64>(comp, "candle-gap").unwrap_or(0.2);
    let candle_w = (slot * (1.0 - gap)).min(slot);
    let faded = attr_num::<f64>(comp, "faded-opacity").unwrap_or(0.3);
    let mut marks = format!(
        "<g class=\"chart-candlesticks n{n} f{}\">",
        (faded * 100.0).round()
    );
    for (i, (x, (_, [open, high, low, close]))) in xs.iter().zip(rows).enumerate() {
        let up = close >= open;
        let color = if up {
            "var(--cronus-success)"
        } else {
            "var(--cronus-error)"
        };
        let (y_open, y_close, y_high, y_low) = (y(*open), y(*close), y(*high), y(*low));
        let body_top = y_open.min(y_close);
        let body_h = {
            let h = (y_close - y_open).abs();
            if h == 0.0 {
                1.0
            } else {
                h
            }
        };
        let wick_top = y_high.min(y_low);
        let wick_h = {
            let h = (y_low - y_high).abs();
            if h == 0.0 {
                1.0
            } else {
                h
            }
        };
        marks.push_str(&format!(
            "<g class=\"candle i{i}\"><rect fill=\"{color}\" height=\"{}\" width=\"1.5\" x=\"{}\" y=\"{}\"></rect><rect fill=\"{color}\" height=\"{}\" rx=\"1\" ry=\"1\" stroke=\"{color}\" stroke-width=\"1\" width=\"{}\" x=\"{}\" y=\"{}\"></rect></g>",
            num(wick_h),
            num(x - 0.75),
            num(wick_top),
            num(body_h),
            num(candle_w),
            num(x - candle_w / 2.0),
            num(body_top),
        ));
    }
    marks.push_str("</g>");
    let short: Vec<String> = labels.iter().map(|l| row_label(l)).collect();
    let axis = motion_x_labels(&data_ticks(&xs, &short, 5), ih, frame.bottom);
    format!(
        "<div data-slot=\"candlestick-chart\" class=\"v-motion\" role=\"img\" aria-label=\"{label}\">{}{}{axis}{marks}</g></svg></div>",
        frame.open(),
        frame.plot_open(),
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

    fn ohlc(style: &str) -> ComponentNode {
        let mut c = stub("candlestick-chart", "Prices");
        c.style = Some(style.into());
        for (d, o, h, l, k) in [
            ("2024-01-01", "100", "108", "96", "104"),
            ("2024-01-02", "104", "112", "101", "109"),
            ("2024-01-03", "109", "115", "105", "108"),
            ("2024-01-04", "108", "114", "102", "110"),
        ] {
            let mut it = ComponentItemNode {
                item_type: "item".into(),
                text: d.into(),
                link: None,
                tone: None,
                config: Default::default(),
            };
            it.config.insert("open".into(), o.into());
            it.config.insert("high".into(), h.into());
            it.config.insert("low".into(), l.into());
            it.config.insert("close".into(), k.into());
            c.items.push(it);
        }
        c
    }

    #[test]
    fn fixture_matches_recharts_candles() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"candlestick-chart\" role=\"img\" aria-label=\"OHLC\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\">"));
        assert!(html.contains("<line x1=\"8\" y1=\"166.5455\" x2=\"424\" y2=\"166.5455\" stroke-dasharray=\"4 4\"></line>"));
        assert!(html.contains("<g data-slot=\"candlestick-marks\"><g class=\"pop\"><line x1=\"8\" y1=\"67.4545\" x2=\"8\" y2=\"186.3636\" stroke=\"var(--cronus-success)\" stroke-width=\"1\"></line><rect x=\"2\" y=\"107.0909\" width=\"12\" height=\"39.6364\" rx=\"1\" fill=\"var(--cronus-success)\"></rect></g>"));
        assert!(html.contains("<rect x=\"210\" y=\"107.0909\" width=\"12\" height=\"19.8182\" rx=\"1\" fill=\"var(--cronus-error)\"></rect>"));
        assert!(html.contains("<rect x=\"418\" y=\"126.9091\""));
        assert!(!html.contains(">Mon</tspan>"));
        assert!(html.contains(">Tue</tspan>"));
        assert!(html.contains(">Wed</tspan>"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn item_config_drives_ohlc() {
        let html = render(&ohlc("candlestick-chart"));
        // Domain 96 - 2 … 115 + 2 (pad max(2, 19 * 0.08)): close 104 → up candle from open 100.
        assert_eq!(html.matches("var(--cronus-success)").count(), 6);
        assert_eq!(html.matches("var(--cronus-error)").count(), 2);
        assert!(html.contains(">2024-01-04</tspan>"));
    }

    #[test]
    fn motion_variant_uses_docs_margins_and_slot_scale() {
        let mut c = ohlc("candlestick-chart+motion");
        c.props.insert("faded-opacity".into(), "0.25".into());
        c.props.insert("height".into(), "320".into());
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"candlestick-chart\" class=\"v-motion\" role=\"img\" aria-label=\"Prices\"><svg viewBox=\"0 0 432 320\" aria-hidden=\"true\">"));
        assert!(html.contains("<g transform=\"translate(16,16)\"><rect fill=\"transparent\" height=\"264\" width=\"400\" x=\"0\" y=\"0\"></rect>"));
        // slot 100 → range [50, 350], candle width 80, wick 1.5 centred; domain nice(95.05, 115.95) = 94..116
        // over 264px: open 100 → 192, close 104 → 144, high 108 → 96, low 96 → 240.
        assert!(html.contains("<g class=\"chart-candlesticks n4 f25\"><g class=\"candle i0\"><rect fill=\"var(--cronus-success)\" height=\"144\" width=\"1.5\" x=\"49.25\" y=\"96\"></rect><rect fill=\"var(--cronus-success)\" height=\"48\" rx=\"1\" ry=\"1\" stroke=\"var(--cronus-success)\" stroke-width=\"1\" width=\"80\" x=\"10\" y=\"144\"></rect></g>"));
        assert!(html.contains("<g class=\"candle i2\"><rect fill=\"var(--cronus-error)\""));
        assert!(html.contains(">Jan 4</text>"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"candlestick-chart\"] {\n  display: block; width: 100%; height: 16rem;\n}"
        ));
        assert!(css.contains("[data-slot=\"candlestick-chart\"].v-motion .candle {"));
        assert!(css
            .contains(".chart-candlesticks.f25:hover .candle:not(:hover) {\n  opacity: 0.25;\n}"));
    }
}
