//! Shared helpers for dedicated cronus-ui family renderers.

use crate::parser::{ComponentItemNode, ComponentNode};

pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn item<'a>(comp: &'a ComponentNode, kind: &str) -> Option<&'a str> {
    comp.items
        .iter()
        .find(|i| i.item_type == kind)
        .map(|i| i.text.as_str())
}

/// Stable id for native `popovertarget` / `anchor` pairs. ASCII slug of name.
pub fn widget_id(comp: &ComponentNode, part: &str) -> String {
    let mut out = String::from("cui-");
    for ch in comp.name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if out.as_bytes().last() != Some(&b'-') {
            out.push('-');
        }
    }
    if !out.ends_with('-') {
        out.push('-');
    }
    out.push_str(part);
    out
}

pub fn label_of(comp: &ComponentNode) -> String {
    for kind in ["label", "title", "text", "value"] {
        if let Some(t) = item(comp, kind) {
            if !t.is_empty() {
                return esc(t);
            }
        }
    }
    for i in &comp.items {
        if !i.text.is_empty() {
            return esc(&i.text);
        }
    }
    esc(&comp.name)
}

pub fn texts(comp: &ComponentNode) -> Vec<String> {
    let mut out: Vec<String> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if out.is_empty() {
        out.push(label_of(comp));
    }
    out
}

const CHOICE_KINDS: &[&str] = &["item", "tab", "columns"];
const FIELD_KINDS: &[&str] = &["label", "title", "text", "value"];

/// Options/rows for select, radio-group, tabs, accordion, table.
/// `label` / `title` / `text` / `value` name the field — they are not choices.
pub fn choice_texts(comp: &ComponentNode) -> Vec<String> {
    let choices: Vec<String> = comp
        .items
        .iter()
        .filter(|i| CHOICE_KINDS.contains(&i.item_type.as_str()) && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if !choices.is_empty() {
        return choices;
    }
    comp.items
        .iter()
        .filter(|i| !FIELD_KINDS.contains(&i.item_type.as_str()) && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect()
}

/// Fallback series when a chart family has no numeric items.
pub const DEFAULT_CHART_SERIES: [f64; 5] = [4.0, 8.0, 6.0, 10.0, 7.0];

pub const CHART_VIEW_W: f64 = 200.0;
pub const CHART_VIEW_H: f64 = 100.0;
pub const CHART_PAD: f64 = 10.0;

pub fn chart_base_y() -> f64 {
    CHART_VIEW_H - CHART_PAD
}

pub fn chart_inner_w() -> f64 {
    CHART_VIEW_W - CHART_PAD * 2.0
}

pub fn chart_inner_h() -> f64 {
    CHART_VIEW_H - CHART_PAD * 2.0
}

/// Numeric values from item text (whole number or comma/whitespace lists).
/// Empty when no numbers are present.
pub fn numeric_items(comp: &ComponentNode) -> Vec<f64> {
    let mut out = Vec::new();
    for i in &comp.items {
        push_nums(&i.text, &mut out);
    }
    out
}

/// Numeric values from item text (whole number or comma/whitespace lists).
/// Empty → `DEFAULT_CHART_SERIES`.
pub fn numeric_series(comp: &ComponentNode) -> Vec<f64> {
    let out = numeric_items(comp);
    if out.is_empty() {
        DEFAULT_CHART_SERIES.to_vec()
    } else {
        out
    }
}

fn push_nums(s: &str, out: &mut Vec<f64>) {
    let t = s.trim();
    if t.is_empty() {
        return;
    }
    if let Ok(n) = t.parse::<f64>() {
        out.push(n);
        return;
    }
    for part in t.split(|c: char| c == ',' || c.is_whitespace()) {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        if let Ok(n) = p.parse::<f64>() {
            out.push(n);
        }
    }
}

pub fn fmt_coord(n: f64) -> String {
    let r = (n * 100.0).round() / 100.0;
    if (r - r.round()).abs() < 1e-9 {
        format!("{}", r.round() as i64)
    } else {
        let s = format!("{r:.2}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// Map a series onto viewBox `0 0 200 100` with 10px padding.
pub fn chart_line_points(series: &[f64]) -> Vec<(f64, f64)> {
    if series.is_empty() {
        return Vec::new();
    }
    let n = series.len();
    let min = series.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = series.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let span = (max - min).max(1.0);
    let inner_w = chart_inner_w();
    let inner_h = chart_inner_h();
    series
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let x = if n == 1 {
                CHART_PAD + inner_w / 2.0
            } else {
                CHART_PAD + inner_w * (i as f64) / ((n - 1) as f64)
            };
            let y = CHART_PAD + inner_h * (1.0 - (*v - min) / span);
            (x, y)
        })
        .collect()
}

pub fn chart_polyline_points(series: &[f64]) -> String {
    chart_line_points(series)
        .into_iter()
        .map(|(x, y)| format!("{},{}", fmt_coord(x), fmt_coord(y)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Domain for a signed series: always includes 0 so the zero line is in view.
pub fn chart_signed_domain(series: &[f64]) -> (f64, f64) {
    let min = series.iter().cloned().fold(0.0_f64, f64::min);
    let max = series.iter().cloned().fold(0.0_f64, f64::max);
    let span = (max - min).max(1.0);
    (min, span)
}

/// Map a signed series onto viewBox `0 0 200 100` with 10px padding.
pub fn chart_signed_line_points(series: &[f64]) -> Vec<(f64, f64)> {
    if series.is_empty() {
        return Vec::new();
    }
    let n = series.len();
    let (min, span) = chart_signed_domain(series);
    let inner_w = chart_inner_w();
    let inner_h = chart_inner_h();
    series
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let x = if n == 1 {
                CHART_PAD + inner_w / 2.0
            } else {
                CHART_PAD + inner_w * (i as f64) / ((n - 1) as f64)
            };
            let y = CHART_PAD + inner_h * (1.0 - (*v - min) / span);
            (x, y)
        })
        .collect()
}

pub fn chart_zero_y(series: &[f64]) -> f64 {
    let (min, span) = chart_signed_domain(series);
    CHART_PAD + chart_inner_h() * (1.0 - (0.0 - min) / span)
}

pub struct ChartBar {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// Vertical bars in the same 200×100 padded viewBox. Baseline is 0 when all values ≥ 0.
pub fn chart_bars(series: &[f64]) -> Vec<ChartBar> {
    if series.is_empty() {
        return Vec::new();
    }
    let n = series.len() as f64;
    let max = series
        .iter()
        .cloned()
        .fold(0.0_f64, f64::max)
        .max(1.0);
    let inner_w = chart_inner_w();
    let inner_h = chart_inner_h();
    let slot = inner_w / n;
    let w = slot * (2.0 / 3.0);
    let inset = (slot - w) / 2.0;
    let base = chart_base_y();
    series
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let h = inner_h * (*v / max).max(0.0);
            ChartBar {
                x: CHART_PAD + slot * i as f64 + inset,
                y: base - h,
                w,
                h,
            }
        })
        .collect()
}

/// Pentagon radar (5 axes). ViewBox `0 0 100 100`, first axis at top.
pub const RADAR_AXES: usize = 5;
pub const RADAR_VIEW: f64 = 100.0;
pub const RADAR_CX: f64 = 50.0;
pub const RADAR_CY: f64 = 50.0;
pub const RADAR_RADIUS: f64 = 36.0;
pub const RADAR_LEVELS: usize = 4;

pub fn radar_values(series: &[f64]) -> [f64; RADAR_AXES] {
    let mut out = [0.0; RADAR_AXES];
    if series.is_empty() {
        return DEFAULT_CHART_SERIES;
    }
    for (i, v) in series.iter().take(RADAR_AXES).enumerate() {
        out[i] = *v;
    }
    out
}

pub fn radar_vertex(index: usize, radius: f64) -> (f64, f64) {
    let angle =
        -std::f64::consts::FRAC_PI_2 + (index as f64) * std::f64::consts::TAU / (RADAR_AXES as f64);
    (
        RADAR_CX + radius * angle.cos(),
        RADAR_CY + radius * angle.sin(),
    )
}

fn join_points(pts: &[(f64, f64)], close: bool) -> String {
    let mut s: Vec<String> = pts
        .iter()
        .map(|(x, y)| format!("{},{}", fmt_coord(*x), fmt_coord(*y)))
        .collect();
    if close && !s.is_empty() {
        s.push(s[0].clone());
    }
    s.join(" ")
}

pub fn radar_data_vertices(series: &[f64]) -> Vec<(f64, f64)> {
    let vals = radar_values(series);
    let max = vals.iter().copied().fold(1.0_f64, f64::max);
    (0..RADAR_AXES)
        .map(|i| {
            let r = RADAR_RADIUS * (vals[i] / max).clamp(0.0, 1.0);
            radar_vertex(i, r)
        })
        .collect()
}

pub fn radar_polygon_points(series: &[f64]) -> String {
    join_points(&radar_data_vertices(series), false)
}

pub fn radar_polyline_points(series: &[f64]) -> String {
    join_points(&radar_data_vertices(series), true)
}

pub fn radar_grid_points(level_frac: f64) -> String {
    let r = RADAR_RADIUS * level_frac.clamp(0.0, 1.0);
    let pts: Vec<(f64, f64)> = (0..RADAR_AXES).map(|i| radar_vertex(i, r)).collect();
    join_points(&pts, true)
}

pub fn radar_axis_points(index: usize) -> String {
    let (x, y) = radar_vertex(index, RADAR_RADIUS);
    format!(
        "{},{} {},{}",
        fmt_coord(RADAR_CX),
        fmt_coord(RADAR_CY),
        fmt_coord(x),
        fmt_coord(y)
    )
}

/// Concentric donut rings. ViewBox `0 0 100 100`.
pub const RING_VIEW: f64 = 100.0;
pub const RING_CX: f64 = 50.0;
pub const RING_CY: f64 = 50.0;
pub const RING_OUTER: f64 = 40.0;

pub struct ChartRing {
    pub r: f64,
    pub stroke: f64,
    pub d: String,
}

pub fn chart_rings(series: &[f64]) -> Vec<ChartRing> {
    let vals: Vec<f64> = if series.is_empty() {
        DEFAULT_CHART_SERIES.to_vec()
    } else {
        series.to_vec()
    };
    let n = vals.len().max(1);
    let slot = RING_OUTER / n as f64;
    let stroke = (slot * 0.62).max(3.0);
    let max = vals.iter().copied().fold(1.0_f64, f64::max);
    (0..n)
        .map(|i| {
            let r = RING_OUTER - slot * i as f64 - stroke / 2.0;
            let progress = (vals[i] / max).clamp(0.0, 1.0);
            ChartRing {
                r,
                stroke,
                d: ring_arc_d(r, progress),
            }
        })
        .collect()
}

fn ring_arc_d(r: f64, progress: f64) -> String {
    if progress <= 0.0 || r <= 0.0 {
        return String::new();
    }
    let a0 = -std::f64::consts::FRAC_PI_2;
    let p = progress.clamp(0.0, 1.0);
    if p >= 0.999 {
        let x0 = RING_CX;
        let y0 = RING_CY - r;
        let x1 = RING_CX;
        let y1 = RING_CY + r;
        return format!(
            "M {} {} A {} {} 0 0 1 {} {} A {} {} 0 0 1 {} {}",
            fmt_coord(x0),
            fmt_coord(y0),
            fmt_coord(r),
            fmt_coord(r),
            fmt_coord(x1),
            fmt_coord(y1),
            fmt_coord(r),
            fmt_coord(r),
            fmt_coord(x0),
            fmt_coord(y0),
        );
    }
    let sweep = p * std::f64::consts::TAU;
    let a1 = a0 + sweep;
    let x0 = RING_CX + r * a0.cos();
    let y0 = RING_CY + r * a0.sin();
    let x1 = RING_CX + r * a1.cos();
    let y1 = RING_CY + r * a1.sin();
    let large = if sweep > std::f64::consts::PI { 1 } else { 0 };
    format!(
        "M {} {} A {} {} 0 {} 1 {} {}",
        fmt_coord(x0),
        fmt_coord(y0),
        fmt_coord(r),
        fmt_coord(r),
        large,
        fmt_coord(x1),
        fmt_coord(y1),
    )
}

/// Horseshoe gauge. ViewBox `0 0 100 100`. Track is 240° through the top.
pub const GAUGE_VIEW: f64 = 100.0;
pub const GAUGE_CX: f64 = 50.0;
pub const GAUGE_CY: f64 = 54.0;
pub const GAUGE_R: f64 = 36.0;
pub const GAUGE_STROKE: f64 = 8.0;
pub const GAUGE_DEFAULT: f64 = 60.0;
/// SVG polar: 0 at 3 o'clock, clockwise (y-down). 150° = lower-left.
const GAUGE_A0: f64 = 150.0 * std::f64::consts::PI / 180.0;
const GAUGE_SWEEP: f64 = 240.0 * std::f64::consts::PI / 180.0;

pub struct ChartGauge {
    pub value: f64,
    pub track_d: String,
    pub value_d: String,
}

/// Clamp `value` to 0–100 and build track + fill arc paths.
pub fn chart_gauge(value: f64) -> ChartGauge {
    let v = if value.is_finite() {
        value.clamp(0.0, 100.0)
    } else {
        0.0
    };
    ChartGauge {
        value: v,
        track_d: gauge_arc_d(1.0),
        value_d: gauge_arc_d(v / 100.0),
    }
}

fn gauge_arc_d(progress: f64) -> String {
    if progress <= 0.0 || GAUGE_R <= 0.0 {
        return String::new();
    }
    let p = progress.clamp(0.0, 1.0);
    let sweep = GAUGE_SWEEP * p;
    let a1 = GAUGE_A0 + sweep;
    let x0 = GAUGE_CX + GAUGE_R * GAUGE_A0.cos();
    let y0 = GAUGE_CY + GAUGE_R * GAUGE_A0.sin();
    let x1 = GAUGE_CX + GAUGE_R * a1.cos();
    let y1 = GAUGE_CY + GAUGE_R * a1.sin();
    let large = if sweep > std::f64::consts::PI { 1 } else { 0 };
    format!(
        "M {} {} A {} {} 0 {} 1 {} {}",
        fmt_coord(x0),
        fmt_coord(y0),
        fmt_coord(GAUGE_R),
        fmt_coord(GAUGE_R),
        large,
        fmt_coord(x1),
        fmt_coord(y1),
    )
}

#[cfg(test)]
pub fn stub(family: &str, label: &str) -> ComponentNode {
    use std::collections::HashMap;
    ComponentNode {
        name: family.to_string(),
        layout: Some("stack".into()),
        style: Some(family.into()),
        items: vec![ComponentItemNode {
            item_type: "label".into(),
            text: label.into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        }],
        props: HashMap::new(),
        params: vec![],
        template: None,
        sections: vec![],
        state: vec![],
        tests: vec![],
        binding: None,
    }
}
