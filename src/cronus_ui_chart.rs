//! Dedicated Chart renderer (ChartContainer) plus the static recharts layout
//! engine shared by the chart families (area/bar/line/live-line/pie/radar/
//! ring/scatter/composed/candlestick/funnel/gauge/profit-loss).
//!
//! React draws these with recharts inside `ResponsiveContainer`, which
//! measures its box after mount. The kernel has no JS, so it emits the
//! settled SVG for the reference box: `div[data-slot="chart"]` (w-full h-64,
//! 432Ã—256 in the 480px audit canvas) with an SVG `viewBox="0 0 432 256"`.
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
/// Polar box: recharts default margin 5 â†’ 422Ã—246, centre 216,128.
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

/// Category names that are not numeric lists (`"4, 8, 6"` is a series).
pub fn series_names(comp: &ComponentNode) -> Vec<String> {
    category_labels(comp)
        .into_iter()
        .filter(|t| {
            !t.split(|c: char| c == ',' || c.is_whitespace())
                .any(|p| p.parse::<f64>().is_ok())
        })
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

/// Categories from bound rows, then the component, else `fallback`.
pub fn categories_or(comp: &ComponentNode, fallback: &[&str]) -> Vec<String> {
    if let Some((cats, _)) = bound_series() {
        if !cats.is_empty() {
            return cats;
        }
    }
    let cats = category_labels(comp);
    if cats.is_empty() {
        fallback.iter().map(|s| s.to_string()).collect()
    } else {
        cats
    }
}

/// Numeric items when present, else bound values, else `cycle` repeated to `n`.
pub fn values_for(comp: &ComponentNode, n: usize, cycle: &[f64]) -> Vec<f64> {
    let nums = numeric_items(comp);
    if !nums.is_empty() {
        return nums;
    }
    if let Some((_, vals)) = bound_series() {
        if !vals.is_empty() {
            return vals;
        }
    }
    (0..n).map(|i| cycle[i % cycle.len()]).collect()
}

fn bound_series() -> Option<(Vec<String>, Vec<f64>)> {
    let rows = crate::cronus_ui_data::rows();
    if rows.is_empty() {
        return None;
    }
    let mut cats = Vec::new();
    let mut vals = Vec::new();
    for row in rows {
        let label = row
            .get("label")
            .or_else(|| row.get("name"))
            .or_else(|| row.get("title"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let val = row
            .get("value")
            .or_else(|| row.get("amount"))
            .or_else(|| row.get("count"))
            .and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            });
        let Some(val) = val else {
            continue;
        };
        cats.push(if label.is_empty() {
            format!("{}", cats.len() + 1)
        } else {
            label
        });
        vals.push(val);
    }
    if vals.is_empty() {
        None
    } else {
        Some((cats, vals))
    }
}

// â”€â”€ ticks â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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

// â”€â”€ shapes â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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

// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
// Series data read from the `.cronus` (shared by every chart family)
// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

/// One series of a multi-series chart: the item-config key, its legend
/// label and the value per row (0 when a row lacks the key).
#[derive(Debug, Clone, PartialEq)]
pub struct Series {
    pub key: String,
    pub label: String,
    pub values: Vec<f64>,
}

/// Rows (`item "Jan" revenue:12000 profit:4500`) + series (`series:"revenue,profit"`).
#[derive(Debug, Clone, Default)]
pub struct ChartData {
    pub labels: Vec<String>,
    pub series: Vec<Series>,
}

impl ChartData {
    pub fn len(&self) -> usize {
        self.labels.len()
    }
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }
    pub fn max(&self) -> f64 {
        self.series
            .iter()
            .flat_map(|s| s.values.iter().copied())
            .fold(f64::NEG_INFINITY, f64::max)
    }
    pub fn min(&self) -> f64 {
        self.series
            .iter()
            .flat_map(|s| s.values.iter().copied())
            .fold(f64::INFINITY, f64::min)
    }
    /// Per-row sum across series (stacked charts).
    pub fn stacked_max(&self) -> f64 {
        (0..self.len())
            .map(|i| {
                self.series
                    .iter()
                    .map(|s| s.values[i].max(0.0))
                    .sum::<f64>()
            })
            .fold(0.0, f64::max)
    }
}

/// `series:"revenue,profit"` / `series:"current:Current,target:Target"` /
/// `series:"units:Units:bar"` (a third segment is the series kind, see
/// [`series_kinds`]).
pub fn series_keys(comp: &ComponentNode) -> Vec<(String, String)> {
    crate::cronus_ui_kit::attr(comp, "series")
        .map(|s| {
            s.split(',')
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .map(|p| {
                    let mut parts = p.splitn(3, ':');
                    let key = parts.next().unwrap_or("").trim().to_string();
                    let label = parts
                        .next()
                        .map(|l| l.trim().to_string())
                        .filter(|l| !l.is_empty())
                        .unwrap_or_else(|| capitalize(&key));
                    (key, label)
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Third `series:` segment per key (`area` / `bar` / `line`), `default` when absent.
pub fn series_kinds<'a>(comp: &'a ComponentNode, default: &'a str) -> Vec<String> {
    crate::cronus_ui_kit::attr(comp, "series")
        .map(|s| {
            s.split(',')
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .map(|p| {
                    p.splitn(4, ':')
                        .nth(2)
                        .map(|k| k.trim().to_string())
                        .filter(|k| !k.is_empty())
                        .unwrap_or_else(|| default.to_string())
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Data rows: `text` / `item` / `option` lines with a non-empty text.
pub fn data_items(comp: &ComponentNode) -> Vec<&crate::parser::ComponentItemNode> {
    comp.items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "text" | "item" | "option"))
        .filter(|i| !i.text.trim().is_empty())
        .collect()
}

fn parse_num(s: &str) -> Option<f64> {
    let t: String = s
        .trim()
        .chars()
        .filter(|c| !matches!(c, ',' | '$' | '%' | '_'))
        .collect();
    t.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// Comma / whitespace separated numbers (`data:"4,8,6"`).
pub fn parse_list(s: &str) -> Vec<f64> {
    s.split(|c: char| c == ',' || c.is_whitespace())
        .filter_map(|p| parse_num(p))
        .collect()
}

/// Rows + series from the component:
/// 1. `series:` keys read from each item's config (`item "Jan" revenue:12000`);
/// 2. else `value:` config on items (`item "Direct" value:320`);
/// 3. else `data:"â€¦"` prop (labels = item texts or 1..n);
/// 4. else numeric item texts (`text "4"`), labels 1..n;
/// 5. else bound rows, then `fallback` labels with the `cycle` values.
pub fn chart_data(comp: &ComponentNode, fallback: &[&str], cycle: &[f64]) -> ChartData {
    let items = data_items(comp);
    let keys = series_keys(comp);
    if !keys.is_empty() {
        let labels: Vec<String> = items.iter().map(|i| i.text.trim().to_string()).collect();
        let series = keys
            .into_iter()
            .map(|(key, label)| Series {
                values: items
                    .iter()
                    .map(|i| i.config.get(&key).and_then(|v| parse_num(v)).unwrap_or(0.0))
                    .collect(),
                key,
                label,
            })
            .collect();
        return ChartData { labels, series };
    }
    if items.iter().any(|i| i.config.contains_key("value")) {
        let labels: Vec<String> = items.iter().map(|i| i.text.trim().to_string()).collect();
        let values = items
            .iter()
            .map(|i| {
                i.config
                    .get("value")
                    .and_then(|v| parse_num(v))
                    .unwrap_or(0.0)
            })
            .collect();
        return ChartData {
            labels,
            series: vec![Series {
                key: "value".into(),
                label: "Value".into(),
                values,
            }],
        };
    }
    if let Some(raw) = crate::cronus_ui_kit::attr(comp, "data") {
        let values = parse_list(raw);
        if !values.is_empty() {
            let texts: Vec<String> = items
                .iter()
                .map(|i| i.text.trim().to_string())
                .filter(|t| parse_num(t).is_none())
                .collect();
            let labels = if texts.len() == values.len() {
                texts
            } else {
                (1..=values.len()).map(|i| i.to_string()).collect()
            };
            return ChartData {
                labels,
                series: vec![Series {
                    key: "value".into(),
                    label: "Value".into(),
                    values,
                }],
            };
        }
    }
    let nums = numeric_items(comp);
    if !nums.is_empty() {
        return ChartData {
            labels: (1..=nums.len()).map(|i| i.to_string()).collect(),
            series: vec![Series {
                key: "value".into(),
                label: "Value".into(),
                values: nums,
            }],
        };
    }
    if let Some((labels, values)) = bound_series() {
        if !values.is_empty() {
            return ChartData {
                labels,
                series: vec![Series {
                    key: "value".into(),
                    label: "Value".into(),
                    values,
                }],
            };
        }
    }
    let cats = category_labels(comp);
    let labels: Vec<String> = if cats.is_empty() {
        fallback.iter().map(|s| s.to_string()).collect()
    } else {
        cats
    };
    let values = (0..labels.len()).map(|i| cycle[i % cycle.len()]).collect();
    ChartData {
        labels,
        series: vec![Series {
            key: "value".into(),
            label: "Value".into(),
            values,
        }],
    }
}

/// `var(--cronus-chart-N)` for series `i` (chart-1..5 cycle, like `chartSeriesConfig`).
pub fn chart_color(i: usize) -> String {
    format!("var(--cronus-chart-{})", i % 5 + 1)
}

/// Colour of series `i`: a fourth `series:` segment picks the palette slot
/// (`revenue:Revenue:line:1` → chart-1, or a `success` / `error` /
/// `primary` token), else the chart-1..5 cycle.
pub fn series_color(comp: &ComponentNode, i: usize) -> String {
    let pick = crate::cronus_ui_kit::attr(comp, "series").and_then(|s| {
        s.split(',')
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .nth(i)
            .and_then(|p| p.splitn(4, ':').nth(3))
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .map(str::to_string)
    });
    match pick.as_deref() {
        Some(n) if n.parse::<usize>().is_ok() => {
            format!(
                "var(--cronus-chart-{})",
                n.parse::<usize>().unwrap_or(1).clamp(1, 5)
            )
        }
        Some(tok) if tok.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') => {
            format!("var(--cronus-{tok})")
        }
        _ => chart_color(i),
    }
}

// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
// Dates (visx time scales) â€” UTC, `Intl.DateTimeFormat("en-US", {month:"short", day:"numeric"})`
// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Days since 1970-01-01 for a proleptic Gregorian date.
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m as i64 + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// `(year, month 1-12, day 1-31)` of a day count since the epoch.
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// `YYYY-MM-DD`, `YYYY-MM-DDTHH:MM[:SS]` (UTC) â†’ milliseconds since the epoch.
pub fn parse_date(s: &str) -> Option<f64> {
    let s = s.trim();
    let (date, time) = match s.split_once(|c| c == 'T' || c == ' ') {
        Some((d, t)) => (d, Some(t.trim_end_matches('Z'))),
        None => (s, None),
    };
    let mut p = date.split('-');
    let y: i64 = p.next()?.parse().ok()?;
    let m: u32 = p.next()?.parse().ok()?;
    let d: u32 = p.next()?.parse().ok()?;
    if p.next().is_some() || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let mut ms = days_from_civil(y, m, d) as f64 * 86_400_000.0;
    if let Some(t) = time {
        let mut q = t.split(':');
        let h: f64 = q.next()?.parse().ok()?;
        let mi: f64 = q.next().unwrap_or("0").parse().ok()?;
        let se: f64 = q.next().unwrap_or("0").parse().ok()?;
        ms += (h * 3600.0 + mi * 60.0 + se) * 1000.0;
    }
    Some(ms)
}

/// `Jan 1` (UTC) â€” `shortDateFmt`.
pub fn short_date(ms: f64) -> String {
    let days = (ms / 86_400_000.0).floor() as i64;
    let (_, m, d) = civil_from_days(days);
    format!("{} {d}", MONTHS[(m - 1) as usize])
}

/// `Jan` (UTC).
pub fn short_month(ms: f64) -> String {
    let days = (ms / 86_400_000.0).floor() as i64;
    let (_, m, _) = civil_from_days(days);
    MONTHS[(m - 1) as usize].to_string()
}

/// Weekday 0 = Sunday (UTC).
pub fn weekday(ms: f64) -> u32 {
    let days = (ms / 86_400_000.0).floor() as i64;
    ((days % 7 + 11) % 7) as u32
}

/// `HH:MM:SS` (24h, UTC) â€” `hmsTimeFmt`.
pub fn hms(ms: f64) -> String {
    let secs = ((ms / 1000.0).floor() as i64).rem_euclid(86_400);
    format!(
        "{:02}:{:02}:{:02}",
        secs / 3600,
        secs % 3600 / 60,
        secs % 60
    )
}

/// `Intl.NumberFormat("en-US")` integer grouping (`12,400`).
pub fn int_fmt(v: f64) -> String {
    let neg = v < 0.0;
    let r = v.abs().round() as u64;
    let s = r.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if neg && r > 0 {
        format!("-{out}")
    } else {
        out
    }
}

/// `$1,240.50` (2 decimals) / `$428,000` (0 decimals).
pub fn usd(v: f64, decimals: usize) -> String {
    let neg = v < 0.0;
    let a = v.abs();
    let whole = if decimals == 0 { a.round() } else { a.floor() };
    let mut s = format!("${}", int_fmt(whole));
    if decimals > 0 {
        let frac = ((a - whole) * 10f64.powi(decimals as i32)).round() as u64;
        let frac = if frac >= 10u64.pow(decimals as u32) {
            10u64.pow(decimals as u32) - 1
        } else {
            frac
        };
        s.push_str(&format!(".{:0width$}", frac, width = decimals));
    }
    if neg {
        format!("-{s}")
    } else {
        s
    }
}

// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
// d3 scales
// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

/// d3-array `tickIncrement`.
pub fn tick_increment(start: f64, stop: f64, count: f64) -> f64 {
    let step = (stop - start) / count.max(0.0);
    if !(step > 0.0) || !step.is_finite() {
        return if step == 0.0 { 0.0 } else { f64::NAN };
    }
    let power = step.log10().floor();
    let error = step / 10f64.powf(power);
    let e10 = 50f64.sqrt();
    let e5 = 10f64.sqrt();
    let e2 = 2f64.sqrt();
    let f = if error >= e10 {
        10.0
    } else if error >= e5 {
        5.0
    } else if error >= e2 {
        2.0
    } else {
        1.0
    };
    if power >= 0.0 {
        f * 10f64.powf(power)
    } else {
        -(10f64.powf(-power)) / f
    }
}

/// d3-scale `scale.nice(count)` for a linear domain.
pub fn d3_nice(lo: f64, hi: f64, count: f64) -> (f64, f64) {
    let (mut start, mut stop) = if hi < lo { (hi, lo) } else { (lo, hi) };
    let mut prestep: Option<f64> = None;
    for _ in 0..10 {
        let step = tick_increment(start, stop, count);
        if prestep == Some(step) {
            break;
        }
        if step > 0.0 {
            start = (start / step).floor() * step;
            stop = (stop / step).ceil() * step;
        } else if step < 0.0 {
            start = (start * step).ceil() / step;
            stop = (stop * step).floor() / step;
        } else {
            break;
        }
        prestep = Some(step);
    }
    (round_tick(start), round_tick(stop))
}

/// d3-array `ticks(start, stop, count)`.
pub fn d3_ticks(start: f64, stop: f64, count: f64) -> Vec<f64> {
    if start == stop && count > 0.0 {
        return vec![start];
    }
    let reverse = stop < start;
    let (start, stop) = if reverse {
        (stop, start)
    } else {
        (start, stop)
    };
    let step = tick_increment(start, stop, count);
    if step == 0.0 || !step.is_finite() {
        return Vec::new();
    }
    let mut out = Vec::new();
    if step > 0.0 {
        let mut r0 = (start / step).round();
        let mut r1 = (stop / step).round();
        if r0 * step < start {
            r0 += 1.0;
        }
        if r1 * step > stop {
            r1 -= 1.0;
        }
        let n = (r1 - r0 + 1.0).max(0.0) as usize;
        for i in 0..n {
            out.push(round_tick((r0 + i as f64) * step));
        }
    } else {
        let step = -step;
        let mut r0 = (start * step).round();
        let mut r1 = (stop * step).round();
        if r0 / step < start {
            r0 += 1.0;
        }
        if r1 / step > stop {
            r1 -= 1.0;
        }
        let n = (r1 - r0 + 1.0).max(0.0) as usize;
        for i in 0..n {
            out.push(round_tick((r0 + i as f64) / step));
        }
    }
    if reverse {
        out.reverse();
    }
    out
}

/// Linear map of `v` from `[d0, d1]` onto `[r0, r1]`.
pub fn lin(v: f64, d0: f64, d1: f64, r0: f64, r1: f64) -> f64 {
    if d1 - d0 == 0.0 {
        return (r0 + r1) / 2.0;
    }
    r0 + (v - d0) / (d1 - d0) * (r1 - r0)
}

// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
// Motion (visx) frame: `ParentSize` box at the 480px audit canvas
// â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•

/// Reference width of a `w-full` motion chart in the 480px audit canvas.
pub const M_W: f64 = 432.0;

/// Cartesian motion frame: margins + inner plot.
#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub w: f64,
    pub h: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Frame {
    /// `aspectRatio` "2 / 1" default with the visx 40px margins.
    pub fn new(w: f64, h: f64) -> Frame {
        Frame {
            w,
            h,
            top: 40.0,
            right: 40.0,
            bottom: 40.0,
            left: 40.0,
        }
    }
    pub fn aspect(w: f64, aspect: f64) -> Frame {
        Frame::new(w, (w / aspect * 10_000.0).round() / 10_000.0)
    }
    pub fn margins(mut self, top: f64, right: f64, bottom: f64, left: f64) -> Frame {
        self.top = top;
        self.right = right;
        self.bottom = bottom;
        self.left = left;
        self
    }
    pub fn iw(&self) -> f64 {
        self.w - self.left - self.right
    }
    pub fn ih(&self) -> f64 {
        self.h - self.top - self.bottom
    }
    /// `<svg viewBox>` + the translated plot group opener.
    pub fn open(&self) -> String {
        format!(
            "<svg viewBox=\"0 0 {} {}\" aria-hidden=\"true\">",
            num(self.w),
            num(self.h)
        )
    }
    pub fn plot_open(&self) -> String {
        format!(
            "<rect fill=\"transparent\" height=\"{}\" width=\"{}\" x=\"0\" y=\"0\"></rect><g transform=\"translate({},{})\"><rect fill=\"transparent\" height=\"{}\" width=\"{}\" x=\"0\" y=\"0\"></rect>",
            num(self.h),
            num(self.w),
            num(self.left),
            num(self.top),
            num(self.ih()),
            num(self.iw())
        )
    }
}

/// `resolveTimeSeriesYDomain` + `niceYDomain` (visx `nice: true`, count 10).
pub fn time_series_domain(min: f64, max: f64) -> (f64, f64) {
    let (lo, hi) = if !min.is_finite() || !max.is_finite() {
        (0.0, 100.0)
    } else if min >= 0.0 {
        (0.0, if max <= 0.0 { 100.0 } else { max * 1.1 })
    } else {
        let pad = {
            let p = (max - min) * 0.05;
            if p == 0.0 {
                1.0
            } else {
                p
            }
        };
        (min - pad, max + pad)
    };
    d3_nice(lo, hi, 10.0)
}

/// Bar / scatter y-domain: `[0, max * 1.1]` nice'd.
pub fn positive_domain(max: f64) -> (f64, f64) {
    let top = if max <= 0.0 { 100.0 } else { max * 1.1 };
    d3_nice(0.0, top, 10.0)
}

/// Milliseconds of each row: ISO date labels, else evenly spaced indices.
pub fn row_times(labels: &[String]) -> Vec<f64> {
    let parsed: Vec<Option<f64>> = labels.iter().map(|l| parse_date(l)).collect();
    if !parsed.is_empty() && parsed.iter().all(Option::is_some) {
        parsed.into_iter().map(|p| p.unwrap_or(0.0)).collect()
    } else {
        (0..labels.len()).map(|i| i as f64 * 86_400_000.0).collect()
    }
}

/// Axis label of a row: `Jan 1` for ISO dates, the text otherwise.
pub fn row_label(label: &str) -> String {
    parse_date(label)
        .map(short_date)
        .unwrap_or_else(|| label.to_string())
}

/// Time scale positions over `[0, iw]` (`scaleTime`, domain = data extent).
pub fn time_xs(times: &[f64], x0: f64, x1: f64) -> Vec<f64> {
    let lo = times.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = times.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    times.iter().map(|t| lin(*t, lo, hi, x0, x1)).collect()
}

// â”€â”€ curves â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

/// d3 `curveLinear`.
pub fn linear_path(pts: &[(f64, f64)]) -> String {
    let mut d = String::new();
    for (i, p) in pts.iter().enumerate() {
        d.push_str(&format!(
            "{}{},{}",
            if i == 0 { "M" } else { "L" },
            num(p.0),
            num(p.1)
        ));
    }
    d
}

fn natural_controls(x: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let n = x.len() - 1;
    let mut a = vec![0.0; n];
    let mut b = vec![0.0; n];
    let mut r = vec![0.0; n];
    a[0] = 0.0;
    b[0] = 2.0;
    r[0] = x[0] + 2.0 * x[1];
    for i in 1..n - 1 {
        a[i] = 1.0;
        b[i] = 4.0;
        r[i] = 4.0 * x[i] + 2.0 * x[i + 1];
    }
    a[n - 1] = 2.0;
    b[n - 1] = 7.0;
    r[n - 1] = 8.0 * x[n - 1] + x[n];
    for i in 1..n {
        let m = a[i] / b[i - 1];
        b[i] -= m;
        r[i] -= m * r[i - 1];
    }
    a[n - 1] = r[n - 1] / b[n - 1];
    for i in (0..n - 1).rev() {
        a[i] = (r[i] - a[i + 1]) / b[i];
    }
    b[n - 1] = (x[n] + a[n - 1]) / 2.0;
    for i in 0..n - 1 {
        b[i] = 2.0 * x[i + 1] - a[i + 1];
    }
    (a, b)
}

/// d3 `curveNatural` (natural cubic spline).
pub fn natural_path(pts: &[(f64, f64)]) -> String {
    match pts.len() {
        0 => return String::new(),
        1 | 2 => return linear_path(pts),
        _ => {}
    }
    let xs: Vec<f64> = pts.iter().map(|p| p.0).collect();
    let ys: Vec<f64> = pts.iter().map(|p| p.1).collect();
    let (ax, bx) = natural_controls(&xs);
    let (ay, by) = natural_controls(&ys);
    let mut d = format!("M{},{}", num(xs[0]), num(ys[0]));
    for i in 1..pts.len() {
        d.push_str(&format!(
            "C{},{},{},{},{},{}",
            num(ax[i - 1]),
            num(ay[i - 1]),
            num(bx[i - 1]),
            num(by[i - 1]),
            num(xs[i]),
            num(ys[i])
        ));
    }
    d
}

/// d3 `curveCatmullRom.alpha(alpha)`.
pub fn catmull_rom_path(pts: &[(f64, f64)], alpha: f64) -> String {
    if pts.len() < 3 {
        return linear_path(pts);
    }
    let mut c = CatmullRom {
        d: format!("M{},{}", num(pts[0].0), num(pts[0].1)),
        alpha,
        point: 0,
        x: [0.0; 3],
        y: [0.0; 3],
        l01_a: 0.0,
        l12_a: 0.0,
        l23_a: 0.0,
        l01_2a: 0.0,
        l12_2a: 0.0,
        l23_2a: 0.0,
    };
    for p in pts {
        c.feed(p.0, p.1);
    }
    // lineEnd: point == 3 â†’ this.point(x2, y2) once more.
    let (lx, ly) = (c.x[2], c.y[2]);
    c.feed(lx, ly);
    c.d
}

struct CatmullRom {
    d: String,
    alpha: f64,
    point: u8,
    /// x0, x1, x2 of the sliding window.
    x: [f64; 3],
    y: [f64; 3],
    l01_a: f64,
    l12_a: f64,
    l23_a: f64,
    l01_2a: f64,
    l12_2a: f64,
    l23_2a: f64,
}

impl CatmullRom {
    fn feed(&mut self, x: f64, y: f64) {
        const EPS: f64 = 1e-12;
        if self.point > 0 {
            let x23 = self.x[2] - x;
            let y23 = self.y[2] - y;
            self.l23_2a = (x23 * x23 + y23 * y23).powf(self.alpha);
            self.l23_a = self.l23_2a.sqrt();
        }
        match self.point {
            0 => self.point = 1,
            1 => self.point = 2,
            _ => {
                self.point = 3;
                let (x0, x1, x2) = (self.x[0], self.x[1], self.x[2]);
                let (y0, y1, y2) = (self.y[0], self.y[1], self.y[2]);
                let (mut cx1, mut cy1, mut cx2, mut cy2) = (x1, y1, x2, y2);
                if self.l01_a > EPS {
                    let a = 2.0 * self.l01_2a + 3.0 * self.l01_a * self.l12_a + self.l12_2a;
                    let n = 3.0 * self.l01_a * (self.l01_a + self.l12_a);
                    cx1 = (x1 * a - x0 * self.l12_2a + x2 * self.l01_2a) / n;
                    cy1 = (y1 * a - y0 * self.l12_2a + y2 * self.l01_2a) / n;
                }
                if self.l23_a > EPS {
                    let b = 2.0 * self.l23_2a + 3.0 * self.l23_a * self.l12_a + self.l12_2a;
                    let m = 3.0 * self.l23_a * (self.l23_a + self.l12_a);
                    cx2 = (x2 * b + x1 * self.l23_2a - x * self.l12_2a) / m;
                    cy2 = (y2 * b + y1 * self.l23_2a - y * self.l12_2a) / m;
                }
                self.d.push_str(&format!(
                    "C{},{},{},{},{},{}",
                    num(cx1),
                    num(cy1),
                    num(cx2),
                    num(cy2),
                    num(x2),
                    num(y2)
                ));
            }
        }
        self.l01_a = self.l12_a;
        self.l12_a = self.l23_a;
        self.l01_2a = self.l12_2a;
        self.l12_2a = self.l23_2a;
        self.x = [self.x[1], self.x[2], x];
        self.y = [self.y[1], self.y[2], y];
    }
}

/// Curve by name: `monotone` (default), `natural`, `catmull`, `linear`.
pub fn curve_path(kind: &str, pts: &[(f64, f64)]) -> String {
    match kind {
        "natural" => natural_path(pts),
        "catmull" | "catmull-rom" | "smooth" => catmull_rom_path(pts, 0.42),
        "linear" => linear_path(pts),
        _ => monotone_path(pts),
    }
}

// â”€â”€ visx chrome helpers â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

/// `ChartRevealClip` â€” the shared `clipPath` whose rect CSS grows 0 â†’ width.
pub fn reveal_clip(id: &str, width: f64, height: f64, padding: f64) -> String {
    format!(
        "<clipPath id=\"{id}\"><rect class=\"reveal\" height=\"{}\" width=\"{}\" x=\"{}\" y=\"{}\"></rect></clipPath>",
        num(height + padding * 2.0),
        num(width + padding * 2.0),
        num(-padding),
        num(-padding)
    )
}

/// Horizontal fade gradient pinned to the viewport (0/15/85/100 stops).
pub fn fade_gradient(id: &str, width: f64, color: &str, left: bool, right: bool) -> String {
    format!(
        "<linearGradient gradientUnits=\"userSpaceOnUse\" id=\"{id}\" x1=\"0\" x2=\"{}\" y1=\"0\" y2=\"0\"><stop offset=\"0%\" stop-color=\"{color}\" stop-opacity=\"{}\"></stop><stop offset=\"15%\" stop-color=\"{color}\" stop-opacity=\"1\"></stop><stop offset=\"85%\" stop-color=\"{color}\" stop-opacity=\"1\"></stop><stop offset=\"100%\" stop-color=\"{color}\" stop-opacity=\"{}\"></stop></linearGradient>",
        num(width),
        if left { 0 } else { 1 },
        if right { 0 } else { 1 },
    )
}

/// `Grid horizontal` â€” dashed rows (`4,4`) behind a 0/10/90/100 fade mask.
pub fn motion_grid(id: &str, ticks_y: &[f64], iw: f64, ih: f64, highlight: &[f64]) -> String {
    let mut out = format!(
        "<g class=\"chart-grid\"><defs><linearGradient id=\"{id}-gradient\" x1=\"0%\" x2=\"100%\" y1=\"0%\" y2=\"0%\"><stop offset=\"0%\" stop-color=\"white\" stop-opacity=\"0\"></stop><stop offset=\"10%\" stop-color=\"white\" stop-opacity=\"1\"></stop><stop offset=\"90%\" stop-color=\"white\" stop-opacity=\"1\"></stop><stop offset=\"100%\" stop-color=\"white\" stop-opacity=\"0\"></stop></linearGradient><mask id=\"{id}\"><rect fill=\"url(#{id}-gradient)\" height=\"{}\" width=\"{}\" x=\"0\" y=\"0\"></rect></mask></defs><g mask=\"url(#{id})\"><g class=\"visx-rows\">",
        num(ih),
        num(iw)
    );
    for y in ticks_y {
        out.push_str(&format!(
            "<line class=\"visx-line\" x1=\"0\" y1=\"{y}\" x2=\"{}\" y2=\"{y}\" stroke=\"var(--cronus-border)\" stroke-dasharray=\"4,4\" stroke-opacity=\"1\" stroke-width=\"1\"></line>",
            num(iw),
            y = num(*y)
        ));
    }
    out.push_str("</g></g>");
    if !highlight.is_empty() {
        out.push_str("<g class=\"chart-grid-highlight-rows\">");
        for y in highlight {
            out.push_str(&format!(
                "<line stroke=\"var(--cronus-fg-muted)\" stroke-dasharray=\"0\" stroke-opacity=\"1\" stroke-width=\"1\" x1=\"0\" x2=\"{}\" y1=\"{y}\" y2=\"{y}\"></line>",
                num(iw),
                y = num(*y)
            ));
        }
        out.push_str("</g>");
    }
    out.push_str("</g>");
    out
}

/// `XAxis` labels: React portals 12px/16px spans at `bottom: 12px`, so the
/// glyph centre sits 20px above the chart bottom; drawn here as `<text>` in
/// plot coordinates with `--cronus-fg-tertiary` (`text-chart-label`).
pub fn motion_x_labels(labels: &[(f64, String)], ih: f64, bottom: f64) -> String {
    let y = ih + bottom - 20.0;
    let mut out = String::from("<g class=\"x-axis\">");
    for (x, label) in labels {
        out.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"central\" fill=\"var(--cronus-fg-tertiary)\">{}</text>",
            num(*x),
            num(y),
            esc(label)
        ));
    }
    out.push_str("</g>");
    out
}

// â”€â”€ XAxis tick selection (`selectEvenlySpacedIndices`) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

fn binomial(n: i64, k: i64) -> f64 {
    if k < 0 || k > n {
        return 0.0;
    }
    let mut r = 1.0;
    for i in 0..k {
        r = r * (n - i) as f64 / (i + 1) as f64;
    }
    r
}

fn compose_positive_sum(sum: i64, parts: i64) -> Vec<Vec<i64>> {
    if parts == 1 {
        return if sum >= 1 { vec![vec![sum]] } else { vec![] };
    }
    let mut out = Vec::new();
    let mut gap = 1;
    while gap <= sum - (parts - 1) {
        for tail in compose_positive_sum(sum - gap, parts - 1) {
            let mut v = vec![gap];
            v.extend(tail);
            out.push(v);
        }
        gap += 1;
    }
    out
}

fn gaps_to_indices(gaps: &[i64]) -> Vec<usize> {
    let mut out = vec![0usize];
    let mut pos = 0i64;
    for g in gaps {
        pos += g;
        out.push(pos as usize);
    }
    out
}

fn indices_for_tick_count(len: usize, count: usize) -> Vec<usize> {
    let span = len as i64 - 1;
    if span <= 0 {
        return vec![0];
    }
    let mut v: Vec<usize> = (0..count)
        .map(|i| ((i as f64 / (count as f64 - 1.0)) * span as f64).round() as usize)
        .collect();
    v.sort_unstable();
    v.dedup();
    if v[0] != 0 {
        v.insert(0, 0);
    }
    if *v.last().unwrap() != span as usize {
        v.push(span as usize);
    }
    v.sort_unstable();
    v.dedup();
    v
}

fn all_index_layouts(len: usize, count: usize) -> Vec<Vec<usize>> {
    let span = len as i64 - 1;
    if span <= 0 {
        return vec![vec![0]];
    }
    let gap_count = count as i64 - 1;
    if gap_count <= 0 {
        return vec![vec![0]];
    }
    if binomial(span - 1, gap_count - 1) > 400.0 {
        return vec![indices_for_tick_count(len, count)];
    }
    compose_positive_sum(span, gap_count)
        .iter()
        .map(|g| gaps_to_indices(g))
        .collect()
}

fn dedupe_by_label(indices: &[usize], labels: &[String]) -> Vec<usize> {
    let mut seen: Vec<&str> = Vec::new();
    let mut out = Vec::new();
    for &i in indices {
        let Some(l) = labels.get(i) else { continue };
        if seen.contains(&l.as_str()) {
            continue;
        }
        seen.push(l);
        out.push(i);
    }
    out
}

struct LayoutScore {
    score: f64,
    symmetry: f64,
    count_distance: f64,
    edge: f64,
}

fn score_layout(indices: &[usize], xs: &[f64], target: usize) -> LayoutScore {
    if indices.len() < 2 {
        return LayoutScore {
            score: f64::INFINITY,
            symmetry: f64::INFINITY,
            count_distance: f64::INFINITY,
            edge: f64::INFINITY,
        };
    }
    let px: Vec<f64> = indices
        .windows(2)
        .map(|w| {
            xs.get(w[1]).copied().unwrap_or(w[1] as f64)
                - xs.get(w[0]).copied().unwrap_or(w[0] as f64)
        })
        .collect();
    let min = px.iter().copied().fold(f64::INFINITY, f64::min);
    let max = px.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mean = px.iter().sum::<f64>() / px.len() as f64;
    let spread = if mean > 0.0 {
        (max - min) / mean
    } else {
        max - min
    };
    let count_distance = (indices.len() as f64 - target as f64).abs();
    let gaps: Vec<i64> = indices
        .windows(2)
        .map(|w| w[1] as i64 - w[0] as i64)
        .collect();
    let smallest = *gaps.iter().min().unwrap();
    let si = gaps.iter().position(|g| *g == smallest).unwrap();
    let interior = if si > 0 && si < gaps.len() - 1 {
        0.08
    } else {
        0.0
    };
    let symmetry = gaps
        .iter()
        .enumerate()
        .map(|(i, g)| (g - gaps[gaps.len() - 1 - i]).abs() as f64)
        .sum::<f64>()
        / gaps.len() as f64;
    let edge = if si == gaps.len() - 1 {
        0.0
    } else if si == 0 {
        1.0
    } else {
        2.0
    };
    LayoutScore {
        score: spread + 0.1 * count_distance + interior + symmetry * 0.02,
        symmetry,
        count_distance,
        edge,
    }
}

fn better(next: &LayoutScore, best: &LayoutScore) -> bool {
    if next.score < best.score - 1e-6 {
        return true;
    }
    if (next.score - best.score).abs() > 1e-6 {
        return false;
    }
    if next.count_distance < best.count_distance {
        return true;
    }
    if next.count_distance > best.count_distance {
        return false;
    }
    if next.symmetry < best.symmetry - 1e-6 {
        return true;
    }
    if next.symmetry > best.symmetry + 1e-6 {
        return false;
    }
    next.edge < best.edge
}

/// `selectEvenlySpacedIndices` â€” the data rows whose labels the XAxis shows.
pub fn select_tick_indices(xs: &[f64], labels: &[String], target: usize) -> Vec<usize> {
    let len = xs.len();
    if len == 0 {
        return Vec::new();
    }
    if len == 1 {
        return vec![0];
    }
    if len <= target {
        return (0..len).collect();
    }
    let min_count = target.saturating_sub(1).max(2);
    let max_count = (target + 1).min(len);
    let mut best = indices_for_tick_count(len, target);
    let mut best_score = score_layout(&best, xs, target);
    for count in min_count..=max_count {
        for raw in all_index_layouts(len, count) {
            let indices = dedupe_by_label(&raw, labels);
            if indices.len() < 2 {
                continue;
            }
            let s = score_layout(&indices, xs, target);
            if better(&s, &best_score) {
                best = indices;
                best_score = s;
            }
        }
    }
    best
}

/// `buildDataAlignedTicks`: `(x, label)` for the selected rows, deduped by label.
pub fn data_ticks(xs: &[f64], labels: &[String], target: usize) -> Vec<(f64, String)> {
    let mut seen: Vec<&str> = Vec::new();
    let mut out = Vec::new();
    for i in select_tick_indices(xs, labels, target) {
        let l = &labels[i];
        if seen.contains(&l.as_str()) {
            continue;
        }
        seen.push(l);
        out.push((xs[i], l.clone()));
    }
    out
}

// â”€â”€ d3-shape arc (cornerRadius, no padAngle) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

const ARC_EPS: f64 = 1e-12;

struct PathCtx {
    d: String,
    x1: Option<f64>,
    y1: Option<f64>,
}

impl PathCtx {
    fn new() -> PathCtx {
        PathCtx {
            d: String::new(),
            x1: None,
            y1: None,
        }
    }
    fn move_to(&mut self, x: f64, y: f64) {
        self.d.push_str(&format!("M{},{}", num(x), num(y)));
        self.x1 = Some(x);
        self.y1 = Some(y);
    }
    fn line_to(&mut self, x: f64, y: f64) {
        self.d.push_str(&format!("L{},{}", num(x), num(y)));
        self.x1 = Some(x);
        self.y1 = Some(y);
    }
    fn arc(&mut self, x: f64, y: f64, r: f64, a0: f64, a1: f64, ccw: bool) {
        let dx = r * a0.cos();
        let dy = r * a0.sin();
        let x0 = x + dx;
        let y0 = y + dy;
        let cw = if ccw { 0 } else { 1 };
        let mut da = if ccw { a0 - a1 } else { a1 - a0 };
        match (self.x1, self.y1) {
            (None, _) | (_, None) => self.move_to(x0, y0),
            (Some(px), Some(py)) => {
                if (px - x0).abs() > ARC_EPS || (py - y0).abs() > ARC_EPS {
                    self.line_to(x0, y0);
                }
            }
        }
        if r == 0.0 {
            return;
        }
        let tau = std::f64::consts::TAU;
        if da < 0.0 {
            da = da % tau + tau;
        }
        if da > tau - ARC_EPS {
            self.d.push_str(&format!(
                "A{r},{r},0,1,{cw},{},{}A{r},{r},0,1,{cw},{},{}",
                num(x - dx),
                num(y - dy),
                num(x0),
                num(y0),
                r = num(r)
            ));
            self.x1 = Some(x0);
            self.y1 = Some(y0);
        } else if da > ARC_EPS {
            let ex = x + r * a1.cos();
            let ey = y + r * a1.sin();
            self.d.push_str(&format!(
                "A{r},{r},0,{},{cw},{},{}",
                u8::from(da >= std::f64::consts::PI),
                num(ex),
                num(ey),
                r = num(r)
            ));
            self.x1 = Some(ex);
            self.y1 = Some(ey);
        }
    }
}

struct Tangents {
    cx: f64,
    cy: f64,
    x01: f64,
    y01: f64,
    x11: f64,
    y11: f64,
}

fn corner_tangents(x0: f64, y0: f64, x1: f64, y1: f64, r1: f64, rc: f64, cw: bool) -> Tangents {
    let x01 = x0 - x1;
    let y01 = y0 - y1;
    let lo = (if cw { rc } else { -rc }) / (x01 * x01 + y01 * y01).sqrt();
    let ox = lo * y01;
    let oy = -lo * x01;
    let x11 = x0 + ox;
    let y11 = y0 + oy;
    let x10 = x1 + ox;
    let y10 = y1 + oy;
    let x00 = (x11 + x10) / 2.0;
    let y00 = (y11 + y10) / 2.0;
    let dx = x10 - x11;
    let dy = y10 - y11;
    let d2 = dx * dx + dy * dy;
    let r = r1 - rc;
    let dd = x11 * y10 - x10 * y11;
    let d = (if dy < 0.0 { -1.0 } else { 1.0 }) * (r * r * d2 - dd * dd).max(0.0).sqrt();
    let cx0 = (dd * dy - dx * d) / d2;
    let cy0 = (-dd * dx - dy * d) / d2;
    let cx1 = (dd * dy + dx * d) / d2;
    let cy1 = (-dd * dx + dy * d) / d2;
    let dx0 = cx0 - x00;
    let dy0 = cy0 - y00;
    let dx1 = cx1 - x00;
    let dy1 = cy1 - y00;
    let (cx, cy) = if dx0 * dx0 + dy0 * dy0 > dx1 * dx1 + dy1 * dy1 {
        (cx1, cy1)
    } else {
        (cx0, cy0)
    };
    Tangents {
        cx,
        cy,
        x01: -ox,
        y01: -oy,
        x11: cx * (r1 / r - 1.0),
        y11: cy * (r1 / r - 1.0),
    }
}

fn intersect(x0: f64, y0: f64, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) -> (f64, f64) {
    let x10 = x1 - x0;
    let y10 = y1 - y0;
    let x32 = x3 - x2;
    let y32 = y3 - y2;
    let mut t = y32 * x10 - x32 * y10;
    if t * t < ARC_EPS {
        return (f64::NAN, f64::NAN);
    }
    t = (x32 * (y0 - y2) - y32 * (x0 - x2)) / t;
    (x0 + t * x10, y0 + t * y10)
}

/// d3-shape `arc()` with `cornerRadius` and `padAngle` 0, angles in radians
/// from 12 o'clock (clockwise), centred on `0,0` like the React charts.
pub fn d3_arc(inner: f64, outer: f64, start: f64, end: f64, corner: f64) -> String {
    let half_pi = std::f64::consts::FRAC_PI_2;
    let pi = std::f64::consts::PI;
    let tau = std::f64::consts::TAU;
    let (mut r0, mut r1) = (inner, outer);
    let a0 = start - half_pi;
    let a1 = end - half_pi;
    let da = (a1 - a0).abs();
    let cw = a1 > a0;
    let mut c = PathCtx::new();
    if r1 < r0 {
        std::mem::swap(&mut r0, &mut r1);
    }
    if !(r1 > ARC_EPS) {
        c.move_to(0.0, 0.0);
    } else if da > tau - ARC_EPS {
        c.move_to(r1 * a0.cos(), r1 * a0.sin());
        c.arc(0.0, 0.0, r1, a0, a1, !cw);
        if r0 > ARC_EPS {
            c.move_to(r0 * a1.cos(), r0 * a1.sin());
            c.arc(0.0, 0.0, r0, a1, a0, cw);
        }
    } else {
        let (a01, a11, a00, a10) = (a0, a1, a0, a1);
        let (da0, da1) = (da, da);
        let rc = ((r1 - r0).abs() / 2.0).min(corner);
        let mut rc0 = rc;
        let mut rc1 = rc;
        let x01 = r1 * a01.cos();
        let y01 = r1 * a01.sin();
        let x10 = r0 * a10.cos();
        let y10 = r0 * a10.sin();
        let x11 = r1 * a11.cos();
        let y11 = r1 * a11.sin();
        let x00 = r0 * a00.cos();
        let y00 = r0 * a00.sin();
        if rc > ARC_EPS && da < pi {
            let oc = if da0 > ARC_EPS {
                intersect(x01, y01, x00, y00, x11, y11, x10, y10)
            } else {
                (x10, y10)
            };
            let ax = x01 - oc.0;
            let ay = y01 - oc.1;
            let bx = x11 - oc.0;
            let by = y11 - oc.1;
            let cosv =
                (ax * bx + ay * by) / ((ax * ax + ay * ay).sqrt() * (bx * bx + by * by).sqrt());
            let kc = 1.0 / (cosv.clamp(-1.0, 1.0).acos() / 2.0).sin();
            let lc = (oc.0 * oc.0 + oc.1 * oc.1).sqrt();
            rc0 = rc.min((r0 - lc) / (kc - 1.0));
            rc1 = rc.min((r1 - lc) / (kc + 1.0));
        }
        if !(da1 > ARC_EPS) {
            c.move_to(x01, y01);
        } else if rc1 > ARC_EPS {
            let t0 = corner_tangents(x00, y00, x01, y01, r1, rc1, cw);
            let t1 = corner_tangents(x11, y11, x10, y10, r1, rc1, cw);
            c.move_to(t0.cx + t0.x01, t0.cy + t0.y01);
            if rc1 < rc {
                c.arc(
                    t0.cx,
                    t0.cy,
                    rc1,
                    t0.y01.atan2(t0.x01),
                    t1.y01.atan2(t1.x01),
                    !cw,
                );
            } else {
                c.arc(
                    t0.cx,
                    t0.cy,
                    rc1,
                    t0.y01.atan2(t0.x01),
                    t0.y11.atan2(t0.x11),
                    !cw,
                );
                c.arc(
                    0.0,
                    0.0,
                    r1,
                    (t0.cy + t0.y11).atan2(t0.cx + t0.x11),
                    (t1.cy + t1.y11).atan2(t1.cx + t1.x11),
                    !cw,
                );
                c.arc(
                    t1.cx,
                    t1.cy,
                    rc1,
                    t1.y11.atan2(t1.x11),
                    t1.y01.atan2(t1.x01),
                    !cw,
                );
            }
        } else {
            c.move_to(x01, y01);
            c.arc(0.0, 0.0, r1, a01, a11, !cw);
        }
        if !(r0 > ARC_EPS) || !(da0 > ARC_EPS) {
            c.line_to(x10, y10);
        } else if rc0 > ARC_EPS {
            let t0 = corner_tangents(x10, y10, x11, y11, r0, -rc0, cw);
            let t1 = corner_tangents(x01, y01, x00, y00, r0, -rc0, cw);
            c.line_to(t0.cx + t0.x01, t0.cy + t0.y01);
            if rc0 < rc {
                c.arc(
                    t0.cx,
                    t0.cy,
                    rc0,
                    t0.y01.atan2(t0.x01),
                    t1.y01.atan2(t1.x01),
                    !cw,
                );
            } else {
                c.arc(
                    t0.cx,
                    t0.cy,
                    rc0,
                    t0.y01.atan2(t0.x01),
                    t0.y11.atan2(t0.x11),
                    !cw,
                );
                c.arc(
                    0.0,
                    0.0,
                    r0,
                    (t0.cy + t0.y11).atan2(t0.cx + t0.x11),
                    (t1.cy + t1.y11).atan2(t1.cx + t1.x11),
                    cw,
                );
                c.arc(
                    t1.cx,
                    t1.cy,
                    rc0,
                    t1.y11.atan2(t1.x11),
                    t1.y01.atan2(t1.x01),
                    !cw,
                );
            }
        } else {
            c.arc(0.0, 0.0, r0, a10, a00, cw);
        }
    }
    c.d.push('Z');
    c.d
}

/// Angular sweep mask for a slice: a thick-stroke circle whose dash CSS
/// grows from the slice start (`pathLength` normalises the arc to 1).
pub fn sweep_mask(
    id: &str,
    cx: f64,
    cy: f64,
    inner: f64,
    outer: f64,
    start: f64,
    end: f64,
) -> String {
    let r = (inner + outer) / 2.0;
    let span = (end - start).abs().max(1e-6);
    let path_length = std::f64::consts::TAU / span;
    let rot = start.to_degrees() - 90.0;
    format!(
        "<mask id=\"{id}\"><circle class=\"sweep\" cx=\"{cx}\" cy=\"{cy}\" r=\"{}\" fill=\"none\" stroke=\"white\" stroke-width=\"{}\" pathLength=\"{}\" transform=\"rotate({} {cx} {cy})\"></circle></mask>",
        num(r),
        num(outer - inner + 2.0),
        num(path_length),
        num(rot),
        cx = num(cx),
        cy = num(cy)
    )
}

// â”€â”€ d3-sankey (nodeAlign center, 6 iterations) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[derive(Debug, Clone)]
pub struct SankeyNode {
    pub name: String,
    pub x0: f64,
    pub x1: f64,
    pub y0: f64,
    pub y1: f64,
    pub value: f64,
    pub depth: usize,
    pub height: usize,
    pub layer: usize,
}

#[derive(Debug, Clone)]
pub struct SankeyLink {
    pub source: usize,
    pub target: usize,
    pub value: f64,
    pub width: f64,
    pub y0: f64,
    pub y1: f64,
}

#[derive(Debug, Clone)]
pub struct SankeyGraph {
    pub nodes: Vec<SankeyNode>,
    pub links: Vec<SankeyLink>,
}

struct SankeyState {
    nodes: Vec<SankeyNode>,
    links: Vec<SankeyLink>,
    /// Outgoing / incoming link ids per node, kept in d3's (stateful) sort order.
    outs: Vec<Vec<usize>>,
    ins: Vec<Vec<usize>>,
    py: f64,
}

impl SankeyState {
    fn sort_outs(&mut self, node: usize) {
        let (nodes, links) = (&self.nodes, &self.links);
        self.outs[node].sort_by(|a, b| {
            nodes[links[*a].target]
                .y0
                .partial_cmp(&nodes[links[*b].target].y0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.cmp(b))
        });
    }
    fn sort_ins(&mut self, node: usize) {
        let (nodes, links) = (&self.nodes, &self.links);
        self.ins[node].sort_by(|a, b| {
            nodes[links[*a].source]
                .y0
                .partial_cmp(&nodes[links[*b].source].y0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.cmp(b))
        });
    }
    /// `reorderNodeLinks`: re-sort the neighbours' link lists.
    fn reorder_node_links(&mut self, node: usize) {
        let sources: Vec<usize> = self.ins[node]
            .iter()
            .map(|l| self.links[*l].source)
            .collect();
        for s in sources {
            self.sort_outs(s);
        }
        let targets: Vec<usize> = self.outs[node]
            .iter()
            .map(|l| self.links[*l].target)
            .collect();
        for t in targets {
            self.sort_ins(t);
        }
    }
    /// `targetTop(source, target)`: target.y0 that makes the link straight.
    fn target_top(&self, source: usize, target: usize) -> f64 {
        let mut y = self.nodes[source].y0 - (self.outs[source].len() as f64 - 1.0) * self.py / 2.0;
        for l in &self.outs[source] {
            if self.links[*l].target == target {
                break;
            }
            y += self.links[*l].width + self.py;
        }
        for l in &self.ins[target] {
            if self.links[*l].source == source {
                break;
            }
            y -= self.links[*l].width;
        }
        y
    }
    /// `sourceTop(source, target)`: source.y0 that makes the link straight.
    fn source_top(&self, source: usize, target: usize) -> f64 {
        let mut y = self.nodes[target].y0 - (self.ins[target].len() as f64 - 1.0) * self.py / 2.0;
        for l in &self.ins[target] {
            if self.links[*l].source == source {
                break;
            }
            y += self.links[*l].width + self.py;
        }
        for l in &self.outs[source] {
            if self.links[*l].target == target {
                break;
            }
            y -= self.links[*l].width;
        }
        y
    }
    fn resolve_collisions(&mut self, column: &[usize], alpha: f64, height: f64) {
        let py = self.py;
        let i = column.len() >> 1;
        let subject = column[i];
        let top = self.nodes[subject].y0 - py;
        let bottom = self.nodes[subject].y1 + py;
        self.bottom_to_top(column, top, i as i64 - 1, alpha);
        self.top_to_bottom(column, bottom, i + 1, alpha);
        self.bottom_to_top(column, height, column.len() as i64 - 1, alpha);
        self.top_to_bottom(column, 0.0, 0, alpha);
    }
    fn top_to_bottom(&mut self, column: &[usize], mut y: f64, start: usize, alpha: f64) {
        for &id in column.iter().skip(start) {
            let dy = (y - self.nodes[id].y0) * alpha;
            if dy > 1e-6 {
                self.nodes[id].y0 += dy;
                self.nodes[id].y1 += dy;
            }
            y = self.nodes[id].y1 + self.py;
        }
    }
    fn bottom_to_top(&mut self, column: &[usize], mut y: f64, start: i64, alpha: f64) {
        let mut i = start;
        while i >= 0 {
            let id = column[i as usize];
            let dy = (self.nodes[id].y1 - y) * alpha;
            if dy > 1e-6 {
                self.nodes[id].y0 -= dy;
                self.nodes[id].y1 -= dy;
            }
            y = self.nodes[id].y0 - self.py;
            i -= 1;
        }
    }
}

fn sort_column(nodes: &[SankeyNode], column: &mut [usize]) {
    column.sort_by(|a, b| {
        nodes[*a]
            .y0
            .partial_cmp(&nodes[*b].y0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// d3-sankey `sankey().nodeWidth(w).nodePadding(p).nodeAlign(sankeyCenter)
/// .extent([[0,0],[width,height]])` â€” node order = declaration order.
pub fn sankey_layout(
    names: &[String],
    raw_links: &[(usize, usize, f64)],
    width: f64,
    height: f64,
    node_width: f64,
    node_padding: f64,
) -> SankeyGraph {
    let n = names.len();
    let nodes: Vec<SankeyNode> = names
        .iter()
        .map(|name| SankeyNode {
            name: name.clone(),
            x0: 0.0,
            x1: 0.0,
            y0: 0.0,
            y1: 0.0,
            value: 0.0,
            depth: 0,
            height: 0,
            layer: 0,
        })
        .collect();
    let links: Vec<SankeyLink> = raw_links
        .iter()
        .filter(|(s, t, _)| *s < n && *t < n)
        .map(|(s, t, v)| SankeyLink {
            source: *s,
            target: *t,
            value: *v,
            width: 0.0,
            y0: 0.0,
            y1: 0.0,
        })
        .collect();
    if n == 0 {
        return SankeyGraph { nodes, links };
    }
    let mut st = SankeyState {
        outs: vec![Vec::new(); n],
        ins: vec![Vec::new(); n],
        nodes,
        links,
        py: node_padding,
    };
    for (i, l) in st.links.iter().enumerate() {
        st.outs[l.source].push(i);
        st.ins[l.target].push(i);
    }
    // computeNodeValues
    for i in 0..n {
        let out: f64 = st.outs[i].iter().map(|l| st.links[*l].value).sum();
        let inn: f64 = st.ins[i].iter().map(|l| st.links[*l].value).sum();
        st.nodes[i].value = out.max(inn);
    }
    // computeNodeDepths / computeNodeHeights (breadth-first, insertion ordered)
    let mut current: Vec<usize> = (0..n).collect();
    let mut x = 0;
    while !current.is_empty() && x <= n {
        let mut next = Vec::new();
        for i in &current {
            st.nodes[*i].depth = x;
            for l in &st.outs[*i] {
                let t = st.links[*l].target;
                if !next.contains(&t) {
                    next.push(t);
                }
            }
        }
        current = next;
        x += 1;
    }
    let mut current: Vec<usize> = (0..n).collect();
    let mut x = 0;
    while !current.is_empty() && x <= n {
        let mut next = Vec::new();
        for i in &current {
            st.nodes[*i].height = x;
            for l in &st.ins[*i] {
                let s = st.links[*l].source;
                if !next.contains(&s) {
                    next.push(s);
                }
            }
        }
        current = next;
        x += 1;
    }
    // computeNodeLayers (sankeyCenter)
    let x = st.nodes.iter().map(|d| d.depth).max().unwrap_or(0) + 1;
    let kx = (width - node_width) / (x as f64 - 1.0).max(1.0);
    let mut columns: Vec<Vec<usize>> = vec![Vec::new(); x];
    for i in 0..n {
        let layer = if !st.ins[i].is_empty() {
            st.nodes[i].depth
        } else if !st.outs[i].is_empty() {
            st.outs[i]
                .iter()
                .map(|l| st.nodes[st.links[*l].target].depth)
                .min()
                .unwrap_or(1)
                .saturating_sub(1)
        } else {
            0
        };
        let layer = layer.min(x - 1);
        st.nodes[i].layer = layer;
        st.nodes[i].x0 = layer as f64 * kx;
        st.nodes[i].x1 = st.nodes[i].x0 + node_width;
        columns[layer].push(i);
    }
    let longest = columns.iter().map(Vec::len).max().unwrap_or(1);
    st.py = node_padding.min(height / (longest as f64 - 1.0).max(f64::MIN_POSITIVE));
    let py = st.py;
    // initializeNodeBreadths
    let ky = columns
        .iter()
        .map(|c| {
            let sum: f64 = c.iter().map(|i| st.nodes[*i].value).sum();
            (height - (c.len() as f64 - 1.0) * py) / sum
        })
        .fold(f64::INFINITY, f64::min);
    for c in &columns {
        let mut y = 0.0;
        for i in c {
            st.nodes[*i].y0 = y;
            st.nodes[*i].y1 = y + st.nodes[*i].value * ky;
            y = st.nodes[*i].y1 + py;
            for l in st.outs[*i].clone() {
                st.links[l].width = st.links[l].value * ky;
            }
        }
        let y = (height - y + py) / (c.len() as f64 + 1.0);
        for (i, id) in c.iter().enumerate() {
            st.nodes[*id].y0 += y * (i as f64 + 1.0);
            st.nodes[*id].y1 += y * (i as f64 + 1.0);
        }
        for i in c {
            st.sort_outs(*i);
            st.sort_ins(*i);
        }
    }
    // computeNodeBreadths: relax
    let iterations = 6;
    for i in 0..iterations {
        let alpha = 0.99f64.powi(i as i32);
        let beta = (1.0 - alpha).max((i as f64 + 1.0) / iterations as f64);
        // relaxRightToLeft
        for ci in (0..columns.len().saturating_sub(1)).rev() {
            for k in 0..columns[ci].len() {
                let source = columns[ci][k];
                let mut y = 0.0;
                let mut w = 0.0;
                for l in st.outs[source].clone() {
                    let target = st.links[l].target;
                    let v = st.links[l].value
                        * (st.nodes[target].layer as f64 - st.nodes[source].layer as f64);
                    y += st.source_top(source, target) * v;
                    w += v;
                }
                if !(w > 0.0) {
                    continue;
                }
                let dy = (y / w - st.nodes[source].y0) * alpha;
                st.nodes[source].y0 += dy;
                st.nodes[source].y1 += dy;
                st.reorder_node_links(source);
            }
            sort_column(&st.nodes, &mut columns[ci]);
            st.resolve_collisions(&columns[ci], beta, height);
        }
        // relaxLeftToRight
        for ci in 1..columns.len() {
            for k in 0..columns[ci].len() {
                let target = columns[ci][k];
                let mut y = 0.0;
                let mut w = 0.0;
                for l in st.ins[target].clone() {
                    let source = st.links[l].source;
                    let v = st.links[l].value
                        * (st.nodes[target].layer as f64 - st.nodes[source].layer as f64);
                    y += st.target_top(source, target) * v;
                    w += v;
                }
                if !(w > 0.0) {
                    continue;
                }
                let dy = (y / w - st.nodes[target].y0) * alpha;
                st.nodes[target].y0 += dy;
                st.nodes[target].y1 += dy;
                st.reorder_node_links(target);
            }
            sort_column(&st.nodes, &mut columns[ci]);
            st.resolve_collisions(&columns[ci], beta, height);
        }
    }
    // computeLinkBreadths
    for i in 0..n {
        let mut y0 = st.nodes[i].y0;
        let mut y1 = y0;
        for l in st.outs[i].clone() {
            st.links[l].y0 = y0 + st.links[l].width / 2.0;
            y0 += st.links[l].width;
        }
        for l in st.ins[i].clone() {
            st.links[l].y1 = y1 + st.links[l].width / 2.0;
            y1 += st.links[l].width;
        }
    }
    SankeyGraph {
        nodes: st.nodes,
        links: st.links,
    }
}

/// `sankeyLinkHorizontal()` â€” cubic between the link's source and target x.
pub fn sankey_link_path(g: &SankeyGraph, l: &SankeyLink) -> String {
    let x0 = g.nodes[l.source].x1;
    let x1 = g.nodes[l.target].x0;
    let xi = (x0 + x1) / 2.0;
    format!(
        "M{},{}C{},{},{},{},{},{}",
        num(x0),
        num(l.y0),
        num(xi),
        num(l.y0),
        num(xi),
        num(l.y1),
        num(x1),
        num(l.y1)
    )
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
        // React ChartFixture bars (values 4, 8, 6 â†’ domain 0..8).
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
        let via = crate::cronus_ui_widgets::render(&stub("chart", "Revenue")).unwrap();
        assert_eq!(via, html);
        assert_eq!(dedicated_fn_name("chart"), Some("cronus_ui_chart::render"));
        assert_eq!(
            renderer_kind("chart"),
            RendererKind::Dedicated("cronus_ui_chart::render")
        );
    }

    #[test]
    fn bound_rows_drive_categories_and_values() {
        use crate::binding::ResolvedData;
        let c = stub("bar-chart", "By status");
        crate::cronus_ui_data::with_binding(
            "Lead",
            &ResolvedData::Rows(vec![
                serde_json::json!({"name": "new", "value": 4}),
                serde_json::json!({"name": "won", "value": 8}),
            ]),
            || {
                assert_eq!(
                    categories_or(&c, &["X"]),
                    vec!["new".to_string(), "won".to_string()]
                );
                assert_eq!(values_for(&c, 2, &DEMO_VALUES), vec![4.0, 8.0]);
            },
        );
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

    // â”€â”€ motion engine (reference values from d3-shape / d3-scale / d3-sankey) â”€â”€

    const CURVE_PTS: [(f64, f64); 5] = [
        (0.0, 100.0),
        (50.0, 20.0),
        (100.0, 80.0),
        (150.0, 40.0),
        (200.0, 60.0),
    ];

    /// d3-path prints 3 decimals; re-round the kernel's 4 to compare.
    fn round3(path: &str) -> String {
        let mut out = String::new();
        let mut tok = String::new();
        let flush = |tok: &mut String, out: &mut String| {
            if !tok.is_empty() {
                let v: f64 = tok.parse().unwrap();
                let r = (v * 1000.0).round() / 1000.0;
                let s = format!("{r:.3}");
                out.push_str(s.trim_end_matches('0').trim_end_matches('.'));
                tok.clear();
            }
        };
        for c in path.chars() {
            if c.is_ascii_digit() || c == '.' || (c == '-' && tok.is_empty()) {
                tok.push(c);
            } else {
                flush(&mut tok, &mut out);
                out.push(c);
            }
        }
        flush(&mut tok, &mut out);
        out
    }

    #[test]
    fn natural_curve_matches_d3() {
        assert_eq!(
            natural_path(&CURVE_PTS),
            "M0,100C16.6667,58.0952,33.3333,16.1905,50,20C66.6667,23.8095,83.3333,73.3333,100,80C116.6667,86.6667,133.3333,50.4762,150,40C166.6667,29.5238,183.3333,44.7619,200,60"
        );
        assert_eq!(natural_path(&CURVE_PTS[..2]), "M0,100L50,20");
    }

    #[test]
    fn catmull_rom_curve_matches_d3() {
        assert_eq!(
            round3(&catmull_rom_path(&CURVE_PTS, 0.42)),
            "M0,100C0,100,32.618,21.55,50,20C66.056,18.568,82.578,77.966,100,80C116.027,81.871,132.682,42.709,150,40C166.103,37.481,200,60,200,60"
        );
    }

    #[test]
    fn d3_nice_and_ticks_match_d3_scale() {
        assert_eq!(d3_nice(0.0, 22000.0 * 1.1, 10.0), (0.0, 26000.0));
        assert_eq!(
            d3_ticks(0.0, 26000.0, 5.0),
            vec![0.0, 5000.0, 10000.0, 15000.0, 20000.0, 25000.0]
        );
        assert_eq!(d3_nice(0.0, 8800.0 * 1.1, 10.0), (0.0, 10000.0));
        assert_eq!(d3_nice(-278.0, 558.0, 10.0), (-300.0, 600.0));
        assert_eq!(
            d3_ticks(-300.0, 600.0, 5.0),
            vec![-200.0, 0.0, 200.0, 400.0, 600.0]
        );
        assert_eq!(d3_nice(94.4, 129.6, 10.0), (90.0, 130.0));
        assert_eq!(time_series_domain(-240.0, 520.0), (-300.0, 600.0));
        assert_eq!(positive_domain(8800.0), (0.0, 10000.0));
    }

    #[test]
    fn d3_arc_matches_d3_shape() {
        let half = std::f64::consts::FRAC_PI_2;
        let tau = std::f64::consts::TAU;
        let progress = d3_arc(60.0, 74.0, -half, -half + tau * 0.42, 7.0);
        assert_eq!(round3(&progress), "M-66.633,0A7,7,0,0,1,-73.595,-7.731A74,74,0,0,1,60.767,-42.23A7,7,0,0,1,58.391,-32.101L58.391,-32.101A7,7,0,0,1,49.271,-34.24A60,60,0,0,0,-59.672,-6.269A7,7,0,0,1,-66.633,0Z");
        assert_eq!(
            d3_arc(60.0, 74.0, -half, 3.0 * half, 7.0),
            "M-74,0A74,74,0,1,1,74,0A74,74,0,1,1,-74,0M-60,0A60,60,0,1,0,60,0A60,60,0,1,0,-60,0Z"
        );
        assert_eq!(
            round3(&d3_arc(
                0.0,
                130.0,
                -half,
                -half + tau * (320.0 / 930.0),
                0.0
            )),
            "M-130,0A130,130,0,0,1,72.452,-107.938L0,0Z"
        );
    }

    #[test]
    fn sankey_layout_matches_d3_sankey() {
        let names: Vec<String> = ["Direct", "Ads", "Site", "App", "Paid", "Churn"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let links = [
            (0, 2, 48.0),
            (1, 2, 32.0),
            (2, 3, 40.0),
            (2, 4, 24.0),
            (3, 4, 28.0),
            (3, 5, 12.0),
            (4, 5, 8.0),
        ];
        let g = sankey_layout(&names, &links, 72.0, 163.0, 16.0, 24.0);
        let pos: Vec<(String, String, String, String)> = g
            .nodes
            .iter()
            .map(|n| (num(n.x0), num(n.x1), num(n.y0), num(n.y1)))
            .collect();
        assert_eq!(pos[0], ("0".into(), "16".into(), "0".into(), "83.4".into()));
        assert_eq!(
            pos[1],
            ("0".into(), "16".into(), "107.4".into(), "163".into())
        );
        assert_eq!(
            pos[2],
            (
                "14".into(),
                "30".into(),
                "10.3057".into(),
                "149.3057".into()
            )
        );
        assert_eq!(
            pos[3],
            (
                "28".into(),
                "44".into(),
                "65.1553".into(),
                "134.6553".into()
            )
        );
        assert_eq!(
            pos[4],
            ("42".into(), "58".into(), "62.715".into(), "153.065".into())
        );
        assert_eq!(
            pos[5],
            (
                "56".into(),
                "72".into(),
                "99.8571".into(),
                "134.6071".into()
            )
        );
        assert_eq!(
            (
                num(g.links[0].width),
                num(g.links[0].y0),
                num(g.links[0].y1)
            ),
            ("83.4".into(), "41.7".into(), "52.0057".into())
        );
        assert_eq!(
            (num(g.links[5].y0), num(g.links[5].y1)),
            ("124.2303".into(), "124.1821".into())
        );
        assert_eq!(
            sankey_link_path(&g, &g.links[0]),
            "M16,41.7C15,41.7,15,52.0057,14,52.0057"
        );
    }

    #[test]
    fn dates_format_like_intl_utc() {
        assert_eq!(parse_date("2024-01-01"), Some(1_704_067_200_000.0));
        assert_eq!(short_date(parse_date("2024-01-01T12:00").unwrap()), "Jan 1");
        assert_eq!(short_date(parse_date("2026-08-27").unwrap()), "Aug 27");
        assert_eq!(weekday(parse_date("2024-01-01").unwrap()), 1);
        assert_eq!(hms(parse_date("2026-08-01T12:00:05").unwrap()), "12:00:05");
        assert_eq!(int_fmt(12400.0), "12,400");
        assert_eq!(usd(428000.0, 0), "$428,000");
        assert_eq!(usd(142.5, 2), "$142.50");
        assert_eq!(row_label("Jul 29"), "Jul 29");
    }

    #[test]
    fn x_axis_tick_selection_matches_react() {
        // 30 daily rows, 5 ticks: layouts > 400 â†’ evenly rounded indices.
        let labels: Vec<String> = (1..=30).map(|d| format!("Jan {d}")).collect();
        let xs: Vec<f64> = (0..30).map(|i| i as f64 * 10.0).collect();
        assert_eq!(select_tick_indices(&xs, &labels, 5), vec![0, 7, 15, 22, 29]);
        // 10 rows: every layout is scored; the most even spacing wins.
        let labels10: Vec<String> = (1..=10).map(|d| format!("Jan {d}")).collect();
        let xs10: Vec<f64> = (0..10).map(|i| i as f64 * 10.0).collect();
        assert_eq!(select_tick_indices(&xs10, &labels10, 5), vec![0, 3, 6, 9]);
        assert_eq!(
            select_tick_indices(&xs10[..3], &labels10[..3], 5),
            vec![0, 1, 2]
        );
        assert_eq!(
            select_tick_indices(&xs10[..8], &labels10[..8], 5),
            vec![0, 2, 5, 7]
        );
        assert_eq!(
            select_tick_indices(&xs[..24], &labels[..24], 5),
            vec![0, 8, 16, 23]
        );
        assert_eq!(
            select_tick_indices(&xs, &labels, 8),
            vec![0, 4, 8, 12, 17, 21, 25, 29]
        );
    }

    #[test]
    fn series_data_reads_item_config_keys() {
        let mut c = stub("bar-chart", "Revenue");
        c.props.insert("series".into(), "revenue,profit:Net".into());
        for (m, r, p) in [("Jan", "12000", "4500"), ("Feb", "15500", "5200")] {
            let mut it = text_item(m);
            it.item_type = "item".into();
            it.config.insert("revenue".into(), r.into());
            it.config.insert("profit".into(), p.into());
            c.items.push(it);
        }
        let d = chart_data(&c, &["X"], &DEMO_VALUES);
        assert_eq!(d.labels, vec!["Jan", "Feb"]);
        assert_eq!(d.series[0].label, "Revenue");
        assert_eq!(d.series[1].label, "Net");
        assert_eq!(d.series[1].values, vec![4500.0, 5200.0]);
        assert_eq!(d.stacked_max(), 20700.0);
        let mut v = stub("pie-chart", "Traffic");
        let mut it = text_item("Direct");
        it.config.insert("value".into(), "320".into());
        v.items.push(it);
        let d = chart_data(&v, &["X"], &DEMO_VALUES);
        assert_eq!(
            (d.labels.clone(), d.series[0].values.clone()),
            (vec!["Direct".to_string()], vec![320.0])
        );
        let mut l = stub("live-line-chart", "Live");
        l.props.insert("data".into(), "4, 8, 6".into());
        let d = chart_data(&l, &["X"], &DEMO_VALUES);
        assert_eq!(d.labels, vec!["1", "2", "3"]);
    }
}
