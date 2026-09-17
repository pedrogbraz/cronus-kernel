//! Dedicated BarChart renderer.
//!
//! Default (recharts `BarChart`, docs "Default" / "Stacked"): `<div
//! data-slot="bar-chart" role="img" aria-label>` › `<div data-slot="chart">
//! <svg viewBox="0 0 432 256">` dashed grid rows, rounded bars (radius 4,
//! barCategoryGap 10%, barGap 4 between grouped series, chart-1..N) and x
//! tick labels. `stacked:true` stacks the series on one column. Bars grow
//! from the 226px baseline (recharts bar animation, 1100ms ease).
//!
//! Motion (`style:bar-chart+motion`, `@cronus-ui/ui/charts`): the visx
//! `BarChart` — `<div data-slot="bar-chart" class="v-motion">`, `2 / 1` SVG,
//! 40px margins, `scaleBand` padding 0.2, `Grid horizontal`, `Bar` rects
//! (`line-cap:round` → rx min(w/2, 8), `line-cap:4` → rx 4, `groupGap` 4,
//! `stack-gap:N`), `BarXAxis` labels. Every bar grows from the baseline with
//! a per-row stagger (40% of 1100ms spread over the rows).
//!
//! Data: `series:"revenue,profit"` + `item "Jan" revenue:12000 profit:4500`.

use crate::cronus_ui_chart::{
    band_xs, chart_color, chart_data, container, d3_ticks, motion_grid, motion_x_labels,
    nice_domain, num, positive_domain, rounded_rect_path, value_grid, x_tick_labels, y_of,
    ChartData, Frame, DEMO_VALUES, M_W,
};
use crate::cronus_ui_kit::{attr, attr_num, choice, flag, instance_id, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let data = chart_data(comp, &["Jan", "Feb", "Mar"], &DEMO_VALUES);
    let stacked = flag(comp, "stacked");
    if choice(comp, "variant", &["motion"]) == Some("motion") {
        return render_motion(comp, &label, &data, stacked);
    }
    let max = if stacked {
        data.stacked_max()
    } else {
        data.max().max(0.0)
    };
    let (lo, hi, ticks) = nice_domain(0.0, max);
    let (centers, band) = band_xs(data.len());
    let mut body = value_grid(&ticks, lo, hi);
    let n = data.series.len().max(1);
    // recharts `getBarPositions`: offset = 10% of the band, barGap 4.
    let offset = band * 0.1;
    let groups = if stacked { 1 } else { n };
    let size = {
        let raw = (band - 2.0 * offset - (groups as f64 - 1.0) * 4.0) / groups as f64;
        if raw > 1.0 {
            raw.round()
        } else {
            raw
        }
    };
    let base = y_of(0f64.max(lo), lo, hi);
    let mut sums = vec![0.0; data.len()];
    for (si, s) in data.series.iter().enumerate() {
        let fill = chart_color(si);
        for (ri, (c, v)) in centers.iter().zip(&s.values).enumerate() {
            let x = if stacked {
                c - band / 2.0 + offset
            } else {
                c - band / 2.0 + offset + (size + 4.0) * si as f64
            };
            let (top, h) = if stacked {
                let y0 = y_of(sums[ri], lo, hi);
                let y1 = y_of(sums[ri] + v.max(0.0), lo, hi);
                sums[ri] += v.max(0.0);
                (y1, y0 - y1)
            } else {
                let y = y_of(*v, lo, hi);
                if y <= base {
                    (y, base - y)
                } else {
                    (base, y - base)
                }
            };
            body.push_str(&format!(
                "<path class=\"grow\" d=\"{}\" fill=\"{fill}\"></path>",
                rounded_rect_path(x, top, size, h, 4.0)
            ));
        }
    }
    body.push_str(&x_tick_labels(&data.labels, &centers, 5.0));
    format!(
        "<div data-slot=\"bar-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

/// visx `BarChart` at the 432px reference width (aspect 2 / 1).
fn render_motion(comp: &ComponentNode, label: &str, data: &ChartData, stacked: bool) -> String {
    let frame = Frame::aspect(M_W, 2.0);
    let (iw, ih) = (frame.iw(), frame.ih());
    let n = data.len();
    let ns = data.series.len().max(1);
    // d3 scaleBand: padding 0.2 (inner = outer), align 0.5.
    let padding = attr_num::<f64>(comp, "bar-gap").unwrap_or(0.2);
    let step = iw / (n as f64 - padding + padding * 2.0).max(1.0);
    let band_w = step * (1.0 - padding);
    let start = (iw - step * (n as f64 - padding)) * 0.5;
    let band_x = |i: usize| start + step * i as f64;
    let max = if stacked {
        data.stacked_max()
    } else {
        data.max().max(0.0)
    };
    let (lo, hi) = positive_domain(max);
    let y = |v: f64| ih - (v - lo) / (hi - lo) * ih;
    let group_gap = if ns > 1 { 4.0 } else { 0.0 };
    let bar_w = if stacked {
        band_w
    } else {
        (band_w - group_gap * (ns as f64 - 1.0)) / ns as f64
    };
    let stack_gap = attr_num::<f64>(comp, "stack-gap").unwrap_or(0.0);
    let line_cap = attr(comp, "line-cap").unwrap_or("round");
    let corner = match line_cap {
        "round" => (bar_w / 2.0).min(8.0),
        "butt" => 0.0,
        v => v.parse::<f64>().unwrap_or(0.0),
    };
    let mut bars = format!("<g class=\"bars n{n}\">");
    let mut sums = vec![0.0; n];
    for (si, s) in data.series.iter().enumerate() {
        let fill = chart_color(si);
        let last = si == ns - 1;
        let rounding = !stacked || stack_gap > 0.0 || last;
        let rx = if rounding { corner } else { 0.0 };
        bars.push_str(&format!("<g class=\"bar-series-{}\">", s.key));
        for (ri, v) in s.values.iter().enumerate() {
            let value_pos = y(*v);
            let mut h = ih - value_pos;
            let (x, top) = if stacked {
                let offset_y = y(sums[ri]);
                sums[ri] += v;
                let gap_offset = si as f64 * stack_gap;
                let top = offset_y - h - gap_offset;
                if !last && stack_gap > 0.0 {
                    h = (h - stack_gap).max(0.0);
                }
                (band_x(ri), top)
            } else {
                (band_x(ri) + si as f64 * (bar_w + group_gap), value_pos)
            };
            bars.push_str(&format!(
                "<rect class=\"grow i{ri}\" fill=\"{fill}\" height=\"{}\" rx=\"{}\" ry=\"{}\" width=\"{}\" x=\"{}\" y=\"{}\"></rect>",
                num(h),
                num(rx),
                num(rx),
                num(bar_w),
                num(x),
                num(top)
            ));
        }
        bars.push_str("</g>");
    }
    bars.push_str("</g>");
    let grid_id = instance_id(comp, "grid");
    let ticks_y: Vec<f64> = d3_ticks(lo, hi, 5.0).iter().map(|t| y(*t)).collect();
    let labels: Vec<(f64, String)> = data
        .labels
        .iter()
        .enumerate()
        .map(|(i, l)| (band_x(i) + band_w / 2.0, l.clone()))
        .collect();
    let shown: Vec<(f64, String)> = if labels.len() <= 12 {
        labels
    } else {
        let stepn = labels.len().div_ceil(12);
        labels
            .into_iter()
            .enumerate()
            .filter(|(i, _)| i % stepn == 0)
            .map(|(_, l)| l)
            .collect()
    };
    let axis = motion_x_labels(&shown, ih, frame.bottom);
    format!(
        "<div data-slot=\"bar-chart\" class=\"v-motion\" role=\"img\" aria-label=\"{label}\">{}{}{}{axis}{bars}</g></svg></div>",
        frame.open(),
        frame.plot_open(),
        motion_grid(&grid_id, &ticks_y, iw, ih, &[]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn fixture() -> ComponentNode {
        let mut c = stub("bar-chart", "Sessions");
        for t in ["Jan", "Feb", "Mar"] {
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

    fn months(style: &str, stacked: bool) -> ComponentNode {
        let mut c = stub("bar-chart", "Revenue and profit");
        c.style = Some(style.into());
        c.props.insert("series".into(), "revenue,profit".into());
        if stacked {
            c.props.insert("stacked".into(), "true".into());
        }
        for (m, r, p) in [
            ("Jan", "12000", "4500"),
            ("Feb", "15500", "5200"),
            ("Mar", "11000", "3800"),
        ] {
            let mut it = ComponentItemNode {
                item_type: "item".into(),
                text: m.into(),
                link: None,
                tone: None,
                config: Default::default(),
            };
            it.config.insert("revenue".into(), r.into());
            it.config.insert("profit".into(), p.into());
            c.items.push(it);
        }
        c
    }

    #[test]
    fn fixture_nests_chart_container_with_recharts_bars() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"bar-chart\" role=\"img\" aria-label=\"Sessions\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\">"));
        assert_eq!(html.matches("stroke-dasharray=\"4 4\"").count(), 5);
        assert!(html.contains(
            "<path class=\"grow\" d=\"M 21.8667,121 A 4,4,0,0,1,25.8667,117 L 128.8667,117"
        ));
        assert!(html.contains("<path class=\"grow\" d=\"M 160.5333,12 A 4,4,0,0,1,164.5333,8"));
        assert!(html.contains("<path class=\"grow\" d=\"M 299.2,66.5 A 4,4,0,0,1,303.2,62.5"));
        for m in ["Jan", "Feb", "Mar"] {
            assert!(html.contains(&format!("dy=\"0.71em\">{m}</tspan>")));
        }
        assert!(!html.contains("style="));
        assert!(!html.contains("<rect"));
    }

    #[test]
    fn category_count_drives_bar_count() {
        let mut c = fixture();
        c.items.push(ComponentItemNode {
            item_type: "text".into(),
            text: "Apr".into(),
            link: None,
            tone: None,
            config: Default::default(),
        });
        let html = render(&c);
        assert_eq!(html.matches("fill=\"var(--cronus-chart-1)\"").count(), 4);
    }

    #[test]
    fn grouped_series_follow_recharts_bar_gap() {
        let html = render(&months("bar-chart", false));
        // band 138.667, offset 13.867: size = round((138.667 - 27.733 - 4) / 2) = 53.
        assert!(html.contains("<path class=\"grow\" d=\"M 21.8667,"));
        assert!(html.contains("L 70.8667,")); // first bar right edge x + 53 - 4
        assert!(html.contains("<path class=\"grow\" d=\"M 78.8667,")); // second series: + 53 + 4
        assert_eq!(html.matches("fill=\"var(--cronus-chart-2)\"").count(), 3);
        // Domain 0..16000 (max 15500): 12000 → y = 226 - 12000/16000*218 = 62.5.
        assert!(html.contains("M 21.8667,66.5 A 4,4,0,0,1,25.8667,62.5"));
    }

    #[test]
    fn stacked_series_share_one_column() {
        let html = render(&months("bar-chart", true));
        // One stack per category, full 10%-inset band (111 wide); stacked max 20700 → domain 0..22000.
        assert!(html.contains("L 128.8667,"));
        // profit sits on top of revenue: Jan revenue 12000 → y 107.0909, profit to 16500 → y 62.5.
        assert!(html.contains("A 4,4,0,0,1,25.8667,107.0909 L 128.8667,107.0909"));
        assert!(html.contains("A 4,4,0,0,1,25.8667,62.5 L 128.8667,62.5 A 4,4,0,0,1,132.8667,66.5 L 132.8667,103.0909"));
    }

    #[test]
    fn motion_variant_uses_band_scale_and_staggered_grow() {
        let html = render(&months("bar-chart+motion", false));
        assert!(html.starts_with("<div data-slot=\"bar-chart\" class=\"v-motion\" role=\"img\" aria-label=\"Revenue and profit\"><svg viewBox=\"0 0 432 216\" aria-hidden=\"true\">"));
        // scaleBand: n 3, padding 0.2 → step 352/3.2 = 110, bandwidth 88, start 22; two bars of 42 with a 4px gap.
        // Domain 15500 * 1.1 = 17050 → nice 18000: revenue 12000 → h 90.6667, profit 4500 → h 34.
        assert!(html.contains("<g class=\"bars n3\"><g class=\"bar-series-revenue\"><rect class=\"grow i0\" fill=\"var(--cronus-chart-1)\" height=\"90.6667\" rx=\"8\" ry=\"8\" width=\"42\" x=\"22\" y=\"45.3333\"></rect>"));
        assert!(html.contains("<g class=\"bar-series-profit\"><rect class=\"grow i0\" fill=\"var(--cronus-chart-2)\" height=\"34\" rx=\"8\" ry=\"8\" width=\"42\" x=\"68\" y=\"102\"></rect>"));
        // Grid rows at ticks(0, 18000, 5) = 0, 5000, 10000, 15000.
        assert_eq!(html.matches("class=\"visx-line\"").count(), 4);
        assert!(html.contains(">Feb</text>"));
        assert!(html.contains("<text x=\"176\" y=\"156\""));
        assert!(!html.contains("style="));
    }

    #[test]
    fn motion_stacked_gap_and_numeric_line_cap() {
        let mut c = months("bar-chart+motion", true);
        c.props.insert("stack-gap".into(), "3".into());
        c.props.insert("line-cap".into(), "4".into());
        let html = render(&c);
        // Stacked: full band (88) wide, rx 4 on every segment, gap 3 between them.
        // Stacked max 20700 * 1.1 = 22770 → nice 24000: Jan revenue 12000 → h 68 - 3 gap from y 68;
        // profit 4500 → h 25.5 sitting 3px above it.
        assert!(html.contains("height=\"65\" rx=\"4\" ry=\"4\" width=\"88\" x=\"22\" y=\"68\""));
        assert!(html.contains("height=\"25.5\" rx=\"4\" ry=\"4\" width=\"88\" x=\"22\" y=\"39.5\""));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"bar-chart\"] {\n  display: block; width: 100%; height: 16rem;\n}"
        ));
        assert!(css.contains("[data-slot=\"bar-chart\"].v-motion .grow {"));
        assert!(css.contains("transform-origin: 0 176px;"));
    }
}
