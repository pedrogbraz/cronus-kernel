//! Dedicated PieChart renderer. DOM matches React:
//! `<div data-slot="pie-chart">` wrapping `<svg>` with 3–4 `<path>` slices.
//! Token fills: primary, success, warning, info. Not the catalog `chart()` stub
//! (`<figure data-slot="pie-chart"><figcaption>`).

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

const SIZE: f64 = 100.0;
const CX: f64 = 50.0;
const CY: f64 = 50.0;
const RADIUS: f64 = 40.0;
const DEFAULT_SLICES: [f64; 4] = [40.0, 25.0, 20.0, 15.0];
const FILLS: [&str; 4] = [
    "var(--cronus-primary)",
    "var(--cronus-success)",
    "var(--cronus-warning)",
    "var(--cronus-info)",
];

pub fn render(comp: &ComponentNode) -> String {
    let values = slices_of(comp);
    let label = label_of(comp);
    let paths = pie_paths(&values);
    format!(
        "<div data-slot=\"pie-chart\" role=\"img\" aria-label=\"{label}\"><svg viewBox=\"0 0 {s} {s}\" width=\"{s}\" height=\"{s}\" aria-hidden=\"true\">{paths}</svg></div>",
        s = fmt(SIZE),
    )
}

fn pie_paths(values: &[f64]) -> String {
    let total: f64 = values.iter().copied().sum();
    let n = values.len().max(1) as f64;
    let mut angle = -std::f64::consts::FRAC_PI_2;
    let mut out = String::new();
    for (index, value) in values.iter().enumerate() {
        let sweep = if total <= 0.0 {
            std::f64::consts::TAU / n
        } else {
            (*value / total) * std::f64::consts::TAU
        };
        let next = angle + sweep;
        let fill = FILLS[index % FILLS.len()];
        let d = slice_path(angle, next);
        out.push_str(&format!("<path d=\"{d}\" fill=\"{fill}\"></path>"));
        angle = next;
    }
    out
}

fn slice_path(a0: f64, a1: f64) -> String {
    let x0 = CX + RADIUS * a0.cos();
    let y0 = CY + RADIUS * a0.sin();
    let x1 = CX + RADIUS * a1.cos();
    let y1 = CY + RADIUS * a1.sin();
    let large = if (a1 - a0).abs() > std::f64::consts::PI {
        1
    } else {
        0
    };
    format!(
        "M {cx} {cy} L {x0} {y0} A {r} {r} 0 {large} 1 {x1} {y1} Z",
        cx = fmt(CX),
        cy = fmt(CY),
        x0 = fmt(x0),
        y0 = fmt(y0),
        x1 = fmt(x1),
        y1 = fmt(y1),
        r = fmt(RADIUS),
    )
}

fn slices_of(comp: &ComponentNode) -> Vec<f64> {
    let mut out = Vec::new();
    if let Some(raw) = comp.props.get("data") {
        out.extend(parse_series(raw));
    }
    for i in &comp.items {
        out.extend(parse_series(&i.text));
        if let Some(v) = i.config.get("value") {
            out.extend(parse_series(v));
        }
    }
    if out.is_empty() {
        for row in crate::cronus_ui_data::rows() {
            if let Some(v) = row.get("value") {
                out.extend(parse_series(&json_num(v)));
            }
        }
    }
    out.retain(|n| *n >= 0.0 && n.is_finite());
    if out.len() < 3 {
        DEFAULT_SLICES.to_vec()
    } else if out.len() > 4 {
        out.truncate(4);
        out
    } else {
        out
    }
}

fn parse_series(s: &str) -> Vec<f64> {
    s.split(|c: char| c == ',' || c.is_whitespace())
        .filter_map(|part| {
            let part = part.trim();
            if part.is_empty() {
                return None;
            }
            part.parse::<f64>().ok().filter(|n| n.is_finite())
        })
        .collect()
}

fn json_num(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

fn fmt(n: f64) -> String {
    let r = (n * 100.0).round() / 100.0;
    if r.fract() == 0.0 {
        format!("{}", r as i64)
    } else {
        format!("{r}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

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
        assert!(!html.contains("<figcaption"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_is_div_with_svg_slices_not_figure() {
        let html = render(&stub("pie-chart", "Share"));
        assert!(html.starts_with("<div data-slot=\"pie-chart\""));
        assert!(html.contains("<svg "));
        let paths = html.matches("<path ").count();
        assert!(paths >= 3, "{html}");
        assert!(paths <= 4, "{html}");
        assert!(html.contains("fill=\"var(--cronus-primary)\""));
        assert!(html.contains("fill=\"var(--cronus-success)\""));
        assert!(html.contains("fill=\"var(--cronus-warning)\""));
        assert!(html.contains("fill=\"var(--cronus-info)\""));
        assert!(!html.contains("<figure"));
        assert!(!html.contains("<figcaption"));
        reject_stub(&html);
    }

    #[test]
    fn three_values_emit_three_slices() {
        let mut c = stub("pie-chart", "Share");
        c.items = vec![
            extra("value", "50"),
            extra("value", "30"),
            extra("value", "20"),
        ];
        let html = render(&c);
        assert_eq!(html.matches("<path ").count(), 3);
        assert!(html.contains("fill=\"var(--cronus-primary)\""));
        assert!(html.contains("fill=\"var(--cronus-success)\""));
        assert!(html.contains("fill=\"var(--cronus-warning)\""));
        assert!(!html.contains("fill=\"var(--cronus-info)\""));
        reject_stub(&html);
    }

    #[test]
    fn skips_chart_figure_stub() {
        let html = render(&stub("pie-chart", "Share"));
        assert!(!html.contains("<figure"));
        assert!(!html.contains("<figcaption"));
        let area =
            crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("sankey-chart"))
                .unwrap();
        assert!(area.contains("<figure"));
        assert!(area.contains("data-slot=\"sankey-chart\""));
        assert_ne!(html, area);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("pie-chart", "Share"));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"pie-chart\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"pie-chart\"]"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
