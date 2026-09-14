//! Dedicated Chart renderer (ChartContainer) plus the static recharts layout
//! engine shared by the chart families (area/bar/line/live-line/pie/radar/
//! ring/scatter/composed/candlestick/funnel/gauge/profit-loss).
//!
//! React draws these with recharts inside `ResponsiveContainer`, which
//! measures its box after mount. The kernel has no JS, so it emits the
//! settled SVG for the reference box: `div[data-slot="chart"]` (w-full h-64,
//! 432×256 in the 480px audit canvas) with an SVG `viewBox="0 0 432 256"`.
//! Plot margins, nice ticks, point/band scales, monotone curves, rounded bar
//! paths, sector paths and `preserveEnd` tick hiding follow recharts 3
//! defaults. At other widths the SVG scales uniformly (`meet`).
//!
//! DOM (chart family, React `ChartFixture`: Bar + XAxis, data = items):
//! `<div data-slot="chart" role="img" aria-label>
//!    <svg viewBox="0 0 432 256" aria-hidden="true">bars + x tick labels</svg>
//!  </div>`

use crate::cronus_ui_kit::{esc, label_of, numeric_items};
use crate::parser::ComponentNode;

/// Reference box of `ChartContainer` (w-full h-64 in the 480px audit canvas).
pub const VIEW_W: f64 = 432.0;
pub const VIEW_H: f64 = 256.0;
/// Cartesian plot area: margin left/right/top 8, x-axis band height 30.
pub const PLOT_L: f64 = 8.0;
pub const PLOT_R: f64 = 424.0;
pub const PLOT_T: f64 = 8.0;
pub const PLOT_B: f64 = 226.0;
/// Polar box: recharts default margin 5 → 422×246, centre 216,128.
pub const POLAR_CX: f64 = 216.0;
pub const POLAR_CY: f64 = 128.0;
/// Recharts default tick fill (`#666`); the chart class override does not win.
pub const TICK_FILL: &str = "#666";
/// Value cycle for demo series when a fixture only names categories.
pub const DEMO_VALUES: [f64; 5] = [4.0, 8.0, 6.0, 10.0, 7.0];

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let mut values = numeric_items(comp);
    if values.is_empty() {
        values = DEMO_VALUES[..3].to_vec();
    }
    let labels: Vec<String> = (1..=values.len()).map(|i| i.to_string()).collect();
    let (lo, hi, _) = nice_domain(0.0, max_of(&values));
    let (centers, band) = band_xs(values.len());
    let mut body = String::new();
    body.push_str(&bars_svg(
        &centers,
        band,
        &values,
        lo,
        hi,
        "var(--cronus-chart-1)",
    ));
    body.push_str(&x_tick_labels(&labels, &centers, 5.0));
    format!(
        "<div data-slot=\"chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        svg(&body)
    )
}

/// `div[data-slot="chart"]` wrapper used inside a family root.
pub fn container(body: &str) -> String {
    format!("<div data-slot=\"chart\">{}</div>", svg(body))
}

pub fn svg(body: &str) -> String {
    format!("<svg viewBox=\"0 0 432 256\" aria-hidden=\"true\">{body}</svg>")
}

/// Coordinate formatting: at most 4 decimals, trailing zeros trimmed.
pub fn num(v: f64) -> String {
    let r = (v * 10_000.0).round() / 10_000.0;
    if r == 0.0 {
        return "0".into();
    }
    let s = format!("{r:.4}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

pub fn max_of(values: &[f64]) -> f64 {
    values.iter().copied().fold(0.0_f64, f64::max)
}

pub fn min_of(values: &[f64]) -> f64 {
    values.iter().copied().fold(0.0_f64, f64::min)
}

/// Non-numeric `text` / `item` / `option` lines: the category names.
pub fn category_labels(comp: &ComponentNode) -> Vec<String> {
    comp.items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "text" | "item" | "option"))
        .map(|i| i.text.trim())
        .filter(|t| !t.is_empty() && t.parse::<f64>().is_err())
        .map(str::to_string)
        .collect()
}

/// Every `text` / `item` / `option` line (numeric or not), e.g. live ticks.
pub fn item_labels(comp: &ComponentNode) -> Vec<String> {
    comp.items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "text" | "item" | "option"))
        .map(|i| i.text.trim())
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect()
}

/// Categories from the component, else `fallback`.
pub fn categories_or(comp: &ComponentNode, fallback: &[&str]) -> Vec<String> {
    let cats = category_labels(comp);
    if cats.is_empty() {
        fallback.iter().map(|s| s.to_string()).collect()
    } else {
        cats
    }
}

/// Numeric items when present, else `cycle` repeated to `n` values.
pub fn values_for(comp: &ComponentNode, n: usize, cycle: &[f64]) -> Vec<f64> {
    let nums = numeric_items(comp);
    if !nums.is_empty() {
        return nums;
    }
    (0..n).map(|i| cycle[i % cycle.len()]).collect()
}

// ── ticks ─────────────────────────────────────────────────────────

fn digit_count(v: f64) -> i32 {
    if v == 0.0 {
        1
    } else {
        v.abs().log10().floor() as i32 + 1
    }
}

/// recharts `getFormatStep` (decimal.js there; epsilon-guarded here).
fn format_step(rough: f64, correction: i32) -> f64 {
    if rough <= 0.0 {
        return 1.0;
    }
    let digits = digit_count(rough);
    let scale_value = 10f64.powi(digits);
    let ratio = rough / scale_value;
    let ratio_scale = if digits != 1 { 0.05 } else { 0.1 };
    let steps = (ratio / ratio_scale - 1e-9).ceil() + correction as f64;
    steps * ratio_scale * scale_value
}

/// recharts `getNiceTickValues` for an auto `[min, max]` domain, tickCount 5.
/// Returns `(domain_lo, domain_hi, ticks)`.
pub fn nice_domain(min: f64, max: f64) -> (f64, f64, Vec<f64>) {
    const COUNT: i32 = 5;
    let (min, max) = if max - min <= 0.0 {
        if max == 0.0 {
            (0.0, 1.0)
        } else {
            (min.min(0.0), max.max(0.0))
        }
    } else {
        (min, max)
    };
    let mut correction = 0;
    loop {
        let step = format_step((max - min) / (COUNT - 1) as f64, correction);
        let middle = if min <= 0.0 && max >= 0.0 {
            0.0
        } else {
            ((min + max) / 2.0 / step).floor() * step
        };
        let below = ((middle - min) / step - 1e-9).ceil().max(0.0) as i32;
        let mut up = ((max - middle) / step - 1e-9).ceil().max(0.0) as i32;
        let count = below + up + 1;
        if count > COUNT && correction < 20 {
            correction += 1;
            continue;
        }
        if count < COUNT {
            up += COUNT - count;
        }
        let lo = middle - below as f64 * step;
        let ticks: Vec<f64> = (0..COUNT)
            .map(|i| round_tick(lo + i as f64 * step))
            .collect();
        let hi = middle + up as f64 * step;
        return (round_tick(lo), round_tick(hi), ticks);
    }
}

/// recharts `getTickValuesFixedDomain` for an explicit domain, tickCount 5.
pub fn fixed_domain_ticks(lo: f64, hi: f64) -> Vec<f64> {
    let step = format_step((hi - lo) / 4.0, 0);
    let mut out = Vec::new();
    let mut v = lo;
    while v < hi - 1e-9 {
        out.push(round_tick(v));
        v += step;
    }
    out.push(round_tick(hi));
    out
}

fn round_tick(v: f64) -> f64 {
    (v * 1e9).round() / 1e9
}

pub fn y_of(v: f64, lo: f64, hi: f64) -> f64 {
    let span = if hi - lo == 0.0 { 1.0 } else { hi - lo };
    PLOT_B - (v - lo) / span * (PLOT_B - PLOT_T)
}

/// Point scale (Line/Area charts): first and last category on the plot edges.
pub fn point_xs(n: usize) -> Vec<f64> {
    match n {
        0 => Vec::new(),
        1 => vec![(PLOT_L + PLOT_R) / 2.0],
        _ => (0..n)
            .map(|i| PLOT_L + (PLOT_R - PLOT_L) * i as f64 / (n - 1) as f64)
            .collect(),
    }
}

/// Band scale (any chart with a Bar): band centres and band width.
pub fn band_xs(n: usize) -> (Vec<f64>, f64) {
    let band = (PLOT_R - PLOT_L) / n.max(1) as f64;
    (
        (0..n).map(|i| PLOT_L + band * (i as f64 + 0.5)).collect(),
        band,
    )
}

/// Approximate advance width of a 12px system-sans label (SF Pro Text).
pub fn text_width(s: &str) -> f64 {
    s.chars()
        .map(|c| match c {
            '0'..='9' => 7.56,
            '.' | ',' | ':' | '\'' | '|' | 'i' | 'l' | 'j' => 3.6,
            ' ' => 3.3,
            '-' | 'f' | 't' | 'r' | 'I' => 4.8,
            'm' | 'w' => 10.0,
            'M' | 'W' => 11.0,
            c if c.is_ascii_uppercase() => 8.2,
            _ => 6.8,
        })
        .sum()
}

/// Bottom X-axis labels with recharts `preserveEnd` hiding: the last label is
/// pulled inside the 432px box, then labels are kept right-to-left while they
/// fit and stay `min_gap` apart.
pub fn x_tick_labels(labels: &[String], xs: &[f64], min_gap: f64) -> String {
    x_tick_labels_at(labels, xs, min_gap, PLOT_B + 18.0)
}

pub fn x_tick_labels_at(labels: &[String], xs: &[f64], min_gap: f64, y: f64) -> String {
    let n = labels.len().min(xs.len());
    let mut shown: Vec<Option<f64>> = vec![None; n];
    let mut end = VIEW_W;
    for i in (0..n).rev() {
        let size = text_width(&labels[i]);
        let mut coord = xs[i];
        if i == n - 1 {
            let gap = coord + size / 2.0 - end;
            if gap > 0.0 {
                coord -= gap;
            }
        }
        if coord - size / 2.0 >= 0.0 && coord + size / 2.0 <= end {
            end = coord - (size / 2.0 + min_gap);
            shown[i] = Some(coord);
        }
    }
    let mut out = String::new();
    for (i, coord) in shown.iter().enumerate() {
        if let Some(x) = coord {
            out.push_str(&format!(
                "<g><text x=\"{x}\" y=\"{y}\" text-anchor=\"middle\" fill=\"{TICK_FILL}\"><tspan x=\"{x}\" dy=\"0.71em\">{t}</tspan></text></g>",
                x = num(*x),
                y = num(y),
                t = esc(&labels[i]),
            ));
        }
    }
    out
}

/// Horizontal dashed grid lines (`CartesianGrid vertical={false}`).
pub fn grid_rows(ys: &[f64], x0: f64, x1: f64) -> String {
    ys.iter()
        .map(|y| {
            format!(
                "<line x1=\"{}\" y1=\"{y}\" x2=\"{}\" y2=\"{y}\" stroke-dasharray=\"4 4\"></line>",
                num(x0),
                num(x1),
                y = num(*y)
            )
        })
        .collect()
}

pub fn grid_cols(xs: &[f64], y0: f64, y1: f64) -> String {
    xs.iter()
        .map(|x| {
            format!(
                "<line x1=\"{x}\" y1=\"{}\" x2=\"{x}\" y2=\"{}\" stroke-dasharray=\"4 4\"></line>",
                num(y0),
                num(y1),
                x = num(*x)
            )
        })
        .collect()
}

/// Grid rows for an auto domain over the cartesian plot.
pub fn value_grid(ticks: &[f64], lo: f64, hi: f64) -> String {
    let ys: Vec<f64> = ticks.iter().map(|t| y_of(*t, lo, hi)).collect();
    grid_rows(&ys, PLOT_L, PLOT_R)
}

// ── shapes ────────────────────────────────────────────────────────

/// d3 `curveMonotoneX` path through `pts` (starts with `M`).
pub fn monotone_path(pts: &[(f64, f64)]) -> String {
    let mut d = String::new();
    if pts.is_empty() {
        return d;
    }
    d.push_str(&format!("M{},{}", num(pts[0].0), num(pts[0].1)));
    if pts.len() == 1 {
        return d;
    }
    if pts.len() == 2 {
        d.push_str(&format!("L{},{}", num(pts[1].0), num(pts[1].1)));
        return d;
    }
    let sign = |v: f64| if v < 0.0 { -1.0 } else { 1.0 };
    let slope3 = |p0: (f64, f64), p1: (f64, f64), p2: (f64, f64)| {
        let h0 = p1.0 - p0.0;
        let h1 = p2.0 - p1.0;
        let s0 = (p1.1 - p0.1) / if h0 != 0.0 { h0 } else { f64::MIN_POSITIVE };
        let s1 = (p2.1 - p1.1) / if h1 != 0.0 { h1 } else { f64::MIN_POSITIVE };
        let p = (s0 * h1 + s1 * h0) / (h0 + h1);
        let v = (sign(s0) + sign(s1)) * s0.abs().min(s1.abs()).min(0.5 * p.abs());
        if v.is_finite() {
            v
        } else {
            0.0
        }
    };
    let slope2 = |p0: (f64, f64), p1: (f64, f64), t: f64| {
        let h = p1.0 - p0.0;
        if h != 0.0 {
            (3.0 * (p1.1 - p0.1) / h - t) / 2.0
        } else {
            t
        }
    };
    let mut seg = |d: &mut String, p0: (f64, f64), p1: (f64, f64), t0: f64, t1: f64| {
        let dx = (p1.0 - p0.0) / 3.0;
        d.push_str(&format!(
            "C{},{},{},{},{},{}",
            num(p0.0 + dx),
            num(p0.1 + dx * t0),
            num(p1.0 - dx),
            num(p1.1 - dx * t1),
            num(p1.0),
            num(p1.1)
        ));
    };
    let mut t0 = 0.0;
    for i in 2..pts.len() {
        let t1 = slope3(pts[i - 2], pts[i - 1], pts[i]);
        let start = if i == 2 {
            slope2(pts[0], pts[1], t1)
        } else {
            t0
        };
        seg(&mut d, pts[i - 2], pts[i - 1], start, t1);
        t0 = t1;
    }
    let n = pts.len();
    let last = slope2(pts[n - 2], pts[n - 1], t0);
    seg(&mut d, pts[n - 2], pts[n - 1], t0, last);
    d
}

/// Monotone line series.
pub fn line_path(xs: &[f64], values: &[f64], lo: f64, hi: f64, stroke: &str) -> String {
    let pts: Vec<(f64, f64)> = xs
        .iter()
        .zip(values)
        .map(|(x, v)| (*x, y_of(*v, lo, hi)))
        .collect();
    format!(
        "<path d=\"{}\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"2\"></path>",
        monotone_path(&pts)
    )
}

/// Monotone area: gradient fill (fill-opacity 0.6, recharts default) + stroke.
pub fn area_paths(
    xs: &[f64],
    values: &[f64],
    lo: f64,
    hi: f64,
    color: &str,
    gradient_id: &str,
    top_opacity: &str,
) -> String {
    let pts: Vec<(f64, f64)> = xs
        .iter()
        .zip(values)
        .map(|(x, v)| (*x, y_of(*v, lo, hi)))
        .collect();
    if pts.is_empty() {
        return String::new();
    }
    let curve = monotone_path(&pts);
    let base = y_of(lo.max(0.0).min(hi), lo, hi);
    let (first, last) = (pts[0].0, pts[pts.len() - 1].0);
    format!(
        "<defs><linearGradient id=\"{gradient_id}\" x1=\"0\" y1=\"0\" x2=\"0\" y2=\"1\"><stop offset=\"0%\" stop-color=\"{color}\" stop-opacity=\"{top_opacity}\"></stop><stop offset=\"100%\" stop-color=\"{color}\" stop-opacity=\"0\"></stop></linearGradient></defs><path d=\"{curve}L{},{b}L{},{b}Z\" fill=\"url(#{gradient_id})\" fill-opacity=\"0.6\" stroke=\"none\"></path><path d=\"{curve}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"2\"></path>",
        num(last),
        num(first),
        b = num(base),
    )
}

/// recharts `Rectangle` path with a uniform corner radius.
pub fn rounded_rect_path(x: f64, y: f64, w: f64, h: f64, radius: f64) -> String {
    let r = radius.min(w / 2.0).min(h / 2.0).max(0.0);
    if r == 0.0 {
        return format!(
            "M {},{} h {} v {} h {} Z",
            num(x),
            num(y),
            num(w),
            num(h),
            num(-w)
        );
    }
    format!(
        "M {x},{yr} A {r},{r},0,0,1,{xr},{y} L {xwr},{y} A {r},{r},0,0,1,{xw},{yr} L {xw},{yhr} A {r},{r},0,0,1,{xwr},{yh} L {xr},{yh} A {r},{r},0,0,1,{x},{yhr} Z",
        x = num(x),
        y = num(y),
        r = num(r),
        xr = num(x + r),
        yr = num(y + r),
        xw = num(x + w),
        xwr = num(x + w - r),
        yh = num(y + h),
        yhr = num(y + h - r),
    )
}

/// One bar series on a band scale (barCategoryGap 10%, radius 4).
pub fn bars_svg(
    centers: &[f64],
    band: f64,
    values: &[f64],
    lo: f64,
    hi: f64,
    fill: &str,
) -> String {
    let gap = band * 0.1;
    let w = (band - 2.0 * gap).round();
    let base = y_of(0f64.max(lo), lo, hi);
    centers
        .iter()
        .zip(values)
        .map(|(c, v)| {
            let x = c - band / 2.0 + gap;
            let y = y_of(*v, lo, hi);
            let (top, h) = if y <= base {
                (y, base - y)
            } else {
                (base, y - base)
            };
            format!(
                "<path d=\"{}\" fill=\"{fill}\"></path>",
                rounded_rect_path(x, top, w, h, 4.0)
            )
        })
        .collect()
}

/// recharts `polarToCartesian`: angle in degrees, counter-clockwise, y down.
pub fn polar(cx: f64, cy: f64, r: f64, angle: f64) -> (f64, f64) {
    let rad = -angle.to_radians();
    (cx + r * rad.cos(), cy + r * rad.sin())
}

/// recharts `getSectorPath` (no corner radius).
pub fn sector_path(cx: f64, cy: f64, inner: f64, outer: f64, start: f64, end: f64) -> String {
    let delta = (end - start).abs().min(359.999);
    let end = if end >= start {
        start + delta
    } else {
        start - delta
    };
    let large = if delta > 180.0 { 1 } else { 0 };
    let os = polar(cx, cy, outer, start);
    let oe = polar(cx, cy, outer, end);
    let mut d = format!(
        "M {},{} A {o},{o},0, {large},{}, {},{}",
        num(os.0),
        num(os.1),
        u8::from(start > end),
        num(oe.0),
        num(oe.1),
        o = num(outer),
    );
    if inner > 0.0 {
        let is = polar(cx, cy, inner, start);
        let ie = polar(cx, cy, inner, end);
        d.push_str(&format!(
            " L {},{} A {i},{i},0, {large},{}, {},{} Z",
            num(ie.0),
            num(ie.1),
            u8::from(start <= end),
            num(is.0),
            num(is.1),
            i = num(inner),
        ));
    } else {
        d.push_str(&format!(" L {},{} Z", num(cx), num(cy)));
    }
    d
}

/// recharts `getSectorWithCorner` (cornerIsExternal = false).
pub fn rounded_sector_path(
    cx: f64,
    cy: f64,
    inner: f64,
    outer: f64,
    corner: f64,
    start: f64,
    end: f64,
) -> String {
    let sign = if end - start < 0.0 { -1.0 } else { 1.0 };
    let tangent = |radius: f64, angle: f64, sign: f64, external: bool| {
        let center_radius = corner * if external { 1.0 } else { -1.0 } + radius;
        let theta = (corner / center_radius).asin().to_degrees();
        let center_angle = angle + sign * theta;
        let circle = polar(cx, cy, radius, center_angle);
        let line = polar(cx, cy, center_radius * theta.to_radians().cos(), angle);
        (circle, line, theta)
    };
    let (soct, solt, sot) = tangent(outer, start, sign, false);
    let (eoct, eolt, eot) = tangent(outer, end, -sign, false);
    let outer_arc = (start - end).abs() - sot - eot;
    if outer_arc < 0.0 {
        return sector_path(cx, cy, inner, outer, start, end);
    }
    let neg = u8::from(sign < 0.0);
    let mut d = format!(
        "M {},{} A{c},{c},0,0,{neg},{},{} A{o},{o},0,{},{neg},{},{} A{c},{c},0,0,{neg},{},{}",
        num(solt.0),
        num(solt.1),
        num(soct.0),
        num(soct.1),
        u8::from(outer_arc > 180.0),
        num(eoct.0),
        num(eoct.1),
        num(eolt.0),
        num(eolt.1),
        c = num(corner),
        o = num(outer),
    );
    if inner > 0.0 {
        let (sict, silt, sit) = tangent(inner, start, sign, true);
        let (eict, eilt, eit) = tangent(inner, end, -sign, true);
        let inner_arc = (start - end).abs() - sit - eit;
        d.push_str(&format!(
            "L{},{} A{c},{c},0,0,{neg},{},{} A{i},{i},0,{},{},{},{} A{c},{c},0,0,{neg},{},{}Z",
            num(eilt.0),
            num(eilt.1),
            num(eict.0),
            num(eict.1),
            u8::from(inner_arc > 180.0),
            u8::from(sign > 0.0),
            num(sict.0),
            num(sict.1),
            num(silt.0),
            num(silt.1),
            c = num(corner),
            i = num(inner),
        ));
    } else {
        d.push_str(&format!("L{},{}Z", num(cx), num(cy)));
    }
    d
}

/// `Intl.NumberFormat("en-US", { notation: "compact", maximumFractionDigits: 1 })`.
pub fn compact_number(v: f64) -> String {
    let abs = v.abs();
    let (scaled, suffix) = if abs >= 1e12 {
        (v / 1e12, "T")
    } else if abs >= 1e9 {
        (v / 1e9, "B")
    } else if abs >= 1e6 {
        (v / 1e6, "M")
    } else if abs >= 1e3 {
        (v / 1e3, "K")
    } else {
        (v, "")
    };
    let r = (scaled * 10.0).round() / 10.0;
    let s = if r.fract() == 0.0 {
        format!("{}", r as i64)
    } else {
        format!("{r:.1}")
    };
    format!("{s}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn text_item(t: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "text".into(),
            text: t.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn fixture() -> ComponentNode {
        let mut c = stub("chart", "Series");
        for t in ["4", "8", "6"] {
            c.items.push(text_item(t));
        }
        c
    }

    fn reject_stub(html: &str) {
        assert!(!html.contains("<figure"));
        assert!(!html.contains("figcaption"));
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("<canvas"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn fixture_matches_recharts_bar_geometry() {
        let html = render(&fixture());
        assert!(html.starts_with(
            "<div data-slot=\"chart\" role=\"img\" aria-label=\"Series\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\">"
        ));
        // React ChartFixture bars (values 4, 8, 6 → domain 0..8).
        assert!(html.contains("<path d=\"M 21.8667,121 A 4,4,0,0,1,25.8667,117 L 128.8667,117 A 4,4,0,0,1,132.8667,121 L 132.8667,222 A 4,4,0,0,1,128.8667,226 L 25.8667,226 A 4,4,0,0,1,21.8667,222 Z\" fill=\"var(--cronus-chart-1)\"></path>"));
        assert!(html.contains("M 160.5333,12 A 4,4,0,0,1,164.5333,8"));
        assert!(html.contains("M 299.2,66.5 A 4,4,0,0,1,303.2,62.5"));
        // Tick labels 1 2 3 at band centres.
        assert!(html.contains("<g><text x=\"77.3333\" y=\"244\" text-anchor=\"middle\" fill=\"#666\"><tspan x=\"77.3333\" dy=\"0.71em\">1</tspan></text></g>"));
        assert!(html.contains(">2</tspan>"));
        assert!(html.contains(">3</tspan>"));
        assert!(!html.contains("polyline"));
        reject_stub(&html);
    }

    #[test]
    fn nice_ticks_follow_recharts() {
        assert_eq!(
            nice_domain(0.0, 8.0),
            (0.0, 8.0, vec![0.0, 2.0, 4.0, 6.0, 8.0])
        );
        assert_eq!(nice_domain(0.0, 12.0).1, 12.0);
        assert_eq!(nice_domain(0.0, 6.0).1, 8.0);
        assert_eq!(
            nice_domain(-4.0, 8.0),
            (-4.0, 12.0, vec![-4.0, 0.0, 4.0, 8.0, 12.0])
        );
        assert_eq!(nice_domain(0.0, 3.0).2, vec![0.0, 0.75, 1.5, 2.25, 3.0]);
        assert_eq!(
            fixed_domain_ticks(0.0, 11.0),
            vec![0.0, 3.0, 6.0, 9.0, 11.0]
        );
    }

    #[test]
    fn monotone_curve_matches_d3() {
        let pts = [(8.0, 117.0), (216.0, 8.0), (424.0, 62.5)];
        assert_eq!(
            monotone_path(&pts),
            "M8,117C77.3333,62.5,146.6667,8,216,8C285.3333,8,354.6667,35.25,424,62.5"
        );
    }

    #[test]
    fn preserve_end_hides_clipped_first_label() {
        let labels: Vec<String> = ["Jan", "Feb", "Mar"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let html = x_tick_labels(&labels, &point_xs(3), 24.0);
        assert!(!html.contains(">Jan<"));
        assert!(html.contains(">Feb<"));
        assert!(html.contains(">Mar<"));
        let band = x_tick_labels(&labels, &band_xs(3).0, 5.0);
        assert!(band.contains(">Jan<"));
    }

    #[test]
    fn sectors_match_recharts() {
        assert_eq!(
            sector_path(216.0, 128.0, 0.0, 98.4, 0.0, 120.0),
            "M 314.4,128 A 98.4,98.4,0, 0,0, 166.8,42.7831 L 216,128 Z"
        );
        let gauge = rounded_sector_path(216.0, 128.0, 83.0, 107.0, 8.0, 210.0, -30.0);
        assert!(gauge.starts_with(
            "M 130.5439,177.3381 A8,8,0,0,1,119.3151,173.837 A107,107,0,1,1,312.6849,173.837"
        ));
        assert_eq!(compact_number(12.0), "12");
        assert_eq!(compact_number(1250.0), "1.3K");
    }

    #[test]
    fn prop_reads_item_config() {
        let mut c = stub("gauge-chart", "Score");
        c.items[0].config.insert("value".into(), "72".into());
        assert_eq!(crate::cronus_ui_kit::attr(&c, "value"), Some("72"));
    }

    #[test]
    fn skips_chart_figure_stub() {
        let html = render(&stub("chart", "Revenue"));
        reject_stub(&html);
        let sankey =
            crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("sankey-chart"))
                .unwrap();
        assert!(sankey.contains("<figure"));
        let via = crate::cronus_ui_widgets::render(&stub("chart", "Revenue")).unwrap();
        assert_eq!(via, html);
        assert_eq!(dedicated_fn_name("chart"), Some("cronus_ui_chart::render"));
        assert_eq!(
            renderer_kind("chart"),
            RendererKind::Dedicated("cronus_ui_chart::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&fixture());
            reject_stub(&html);
            assert!(html.contains("data-slot=\"chart\""));
        });
    }

    #[test]
    fn chrome_sizes_container_like_react() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"chart\"] {\n  display: flex; justify-content: center;\n  width: 100%; height: 16rem;"));
        assert!(css.contains("font-size: 0.75rem; line-height: 1rem;"));
        assert!(css.contains("color-mix(in oklch, var(--cronus-border) 50%, transparent)"));
    }
}
