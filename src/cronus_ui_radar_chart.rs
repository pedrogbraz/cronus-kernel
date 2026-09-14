//! Dedicated RadarChart renderer. DOM matches React:
//! `<div data-slot="radar-chart" role="img" aria-label>`
//!   `<div data-slot="chart"><svg viewBox="0 0 432 256">` PolarGrid (one
//!   polygon per radius tick + spokes, recharts `#ccc`), the series polygon
//!   (chart-1, fill-opacity 0.25) and PolarAngleAxis labels (`#808080`).
//! Centre 216,128, outerRadius 70% of 123 = 86.1, first axis at 90°.
//! Axes are the `text` lines; values cycle 4, 8, 6, 10, 7.

use crate::cronus_ui_chart::{
    categories_or, container, max_of, nice_domain, num, polar, values_for, DEMO_VALUES,
    POLAR_CX, POLAR_CY,
};
use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

pub const RADAR_OUTER: f64 = 86.1;
const EPS: f64 = 1e-5;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let axes = categories_or(comp, &["Speed", "Reliability", "Comfort"]);
    let values = values_for(comp, axes.len(), &DEMO_VALUES);
    let n = axes.len().min(values.len()).max(1);
    let (lo, hi, ticks) = nice_domain(0.0, max_of(&values));
    let angle = |i: usize| 90.0 - 360.0 * i as f64 / n as f64;
    let mut body = String::new();
    for t in &ticks {
        let r = (t - lo) / (hi - lo) * RADAR_OUTER;
        body.push_str(&format!(
            "<path d=\"{}\" fill=\"none\" stroke=\"#ccc\"></path>",
            ring(n, r, angle)
        ));
    }
    for i in 0..n {
        let (x, y) = polar(POLAR_CX, POLAR_CY, RADAR_OUTER, angle(i));
        body.push_str(&format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#ccc\"></line>",
            num(POLAR_CX),
            num(POLAR_CY),
            num(x),
            num(y)
        ));
    }
    let mut d = String::new();
    for i in 0..n {
        let r = (values[i] - lo) / (hi - lo) * RADAR_OUTER;
        let (x, y) = polar(POLAR_CX, POLAR_CY, r, angle(i));
        d.push_str(&format!("{}{},{}", if i == 0 { "M" } else { "L" }, num(x), num(y)));
    }
    let (x0, y0) = polar(POLAR_CX, POLAR_CY, (values[0] - lo) / (hi - lo) * RADAR_OUTER, angle(0));
    d.push_str(&format!("L{},{}Z", num(x0), num(y0)));
    body.push_str(&format!(
        "<path d=\"{d}\" fill=\"var(--cronus-chart-1)\" fill-opacity=\"0.25\" stroke=\"var(--cronus-chart-1)\"></path>"
    ));
    for (i, name) in axes.iter().take(n).enumerate() {
        let a = angle(i);
        let (x, y) = polar(POLAR_CX, POLAR_CY, RADAR_OUTER + 8.0, a);
        let cos = (-a).to_radians().cos();
        let sin = (-a).to_radians().sin();
        let anchor = if cos > EPS {
            "start"
        } else if cos < -EPS {
            "end"
        } else {
            "middle"
        };
        let dy = if cos.abs() <= EPS {
            if sin > 0.0 {
                "0.71em"
            } else {
                "0em"
            }
        } else {
            "0.355em"
        };
        body.push_str(&format!(
            "<g><text x=\"{x}\" y=\"{y}\" text-anchor=\"{anchor}\" fill=\"#808080\"><tspan x=\"{x}\" dy=\"{dy}\">{t}</tspan></text></g>",
            x = num(x),
            y = num(y),
            t = esc(name),
        ));
    }
    format!(
        "<div data-slot=\"radar-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

fn ring(n: usize, r: f64, angle: impl Fn(usize) -> f64) -> String {
    let mut d = String::new();
    for i in 0..n {
        let (x, y) = polar(POLAR_CX, POLAR_CY, r, angle(i));
        d.push_str(&format!("{} {},{}", if i == 0 { "M" } else { "L" }, num(x), num(y)));
    }
    d.push('Z');
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn fixture() -> ComponentNode {
        let mut c = stub("radar-chart", "Metrics");
        for t in ["Speed", "Reliability", "Comfort"] {
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
    fn fixture_matches_recharts_polar_layout() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"radar-chart\" role=\"img\" aria-label=\"Metrics\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\">"));
        assert!(html.contains("<path d=\"M 216,41.9L 290.5648,171.05L 141.4352,171.05Z\" fill=\"none\" stroke=\"#ccc\"></path>"));
        assert!(html.contains("<path d=\"M216,84.95L290.5648,171.05L160.0764,160.2875L216,84.95Z\" fill=\"var(--cronus-chart-1)\" fill-opacity=\"0.25\""));
        assert!(html.contains("<g><text x=\"216\" y=\"33.9\" text-anchor=\"middle\" fill=\"#808080\"><tspan x=\"216\" dy=\"0em\">Speed</tspan></text></g>"));
        assert!(html.contains("text-anchor=\"start\" fill=\"#808080\"><tspan x=\"297.493\" dy=\"0.355em\">Reliability</tspan>"));
        assert!(html.contains("text-anchor=\"end\" fill=\"#808080\"><tspan x=\"134.507\" dy=\"0.355em\">Comfort</tspan>"));
        assert!(!html.contains("polyline"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"radar-chart\"] {\n  display: block; width: 100%; height: 16rem;\n}"));
    }
}
