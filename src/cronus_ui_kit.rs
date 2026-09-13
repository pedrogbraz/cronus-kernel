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
/// Empty → `DEFAULT_CHART_SERIES`.
pub fn numeric_series(comp: &ComponentNode) -> Vec<f64> {
    let mut out = Vec::new();
    for i in &comp.items {
        push_nums(&i.text, &mut out);
    }
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
