//! Dedicated GaugeChart renderer. DOM matches React:
//! `<div data-slot="gauge-chart" role="img" aria-label>`
//!   `<div data-slot="chart"><svg viewBox="0 0 432 256">` RadialBar background
//!   (recharts `#eee`) and value sector (chart-1) from 210° to -30°, radius
//!   band 83..107 (innerRadius 80 / outerRadius 110, 10% band gap), corner
//!   radius 8, then the centred value `<text>` (fill-fg text-2xl font-medium).
//! `value` / `max` come from props or item config (`value:72` after the
//! label line), else the first numeric item, else 72.

use crate::cronus_ui_chart::{container, num, prop, rounded_sector_path, POLAR_CX, POLAR_CY};
use crate::cronus_ui_kit::{label_of, numeric_items};
use crate::parser::ComponentNode;

pub const GAUGE_INNER: f64 = 83.0;
pub const GAUGE_OUTER: f64 = 107.0;
pub const GAUGE_CORNER: f64 = 8.0;
pub const GAUGE_START: f64 = 210.0;
pub const GAUGE_END: f64 = -30.0;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let max = prop(comp, "max")
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|m| m.is_finite() && *m > 0.0)
        .unwrap_or(100.0);
    let value = prop(comp, "value")
        .and_then(|v| v.trim().parse::<f64>().ok())
        .or_else(|| numeric_items(comp).first().copied())
        .unwrap_or(72.0);
    let clamped = if value.is_finite() {
        value.clamp(0.0, max)
    } else {
        0.0
    };
    let mut body = format!(
        "<path d=\"{}\" fill=\"#eee\"></path>",
        rounded_sector_path(
            POLAR_CX,
            POLAR_CY,
            GAUGE_INNER,
            GAUGE_OUTER,
            GAUGE_CORNER,
            GAUGE_START,
            GAUGE_END
        )
    );
    body.push_str(&format!(
        "<text x=\"50%\" y=\"50%\" text-anchor=\"middle\" dominant-baseline=\"middle\">{}</text>",
        num(clamped)
    ));
    if clamped > 0.0 {
        let end = GAUGE_START + (GAUGE_END - GAUGE_START) * clamped / max;
        body.push_str(&format!(
            "<path d=\"{}\" fill=\"var(--cronus-chart-1)\"></path>",
            rounded_sector_path(
                POLAR_CX,
                POLAR_CY,
                GAUGE_INNER,
                GAUGE_OUTER,
                GAUGE_CORNER,
                GAUGE_START,
                end
            )
        ));
    }
    format!(
        "<div data-slot=\"gauge-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn fixture() -> ComponentNode {
        let mut c = stub("gauge-chart", "Score");
        // Parser attaches `value:72` to the preceding `label` item.
        c.items[0].config.insert("value".into(), "72".into());
        c
    }

    #[test]
    fn fixture_reads_value_from_item_config() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"gauge-chart\" role=\"img\" aria-label=\"Score\"><div data-slot=\"chart\"><svg viewBox=\"0 0 432 256\" aria-hidden=\"true\"><path d=\"M 130.5439,177.3381 A8,8,0,0,1,119.3151,173.837 A107,107,0,1,1,312.6849,173.837"));
        assert!(html.contains("fill=\"#eee\"></path><text x=\"50%\" y=\"50%\" text-anchor=\"middle\" dominant-baseline=\"middle\">72</text>"));
        assert!(html.contains("A107,107,0,0,1,295.7223,56.6323 A8,8,0,0,1,294.5986,68.3404"));
        assert!(html.contains("fill=\"var(--cronus-chart-1)\""));
        assert!(!html.contains(">60<"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn zero_value_has_no_value_sector() {
        let mut c = stub("gauge-chart", "Score");
        c.props.insert("value".into(), "0".into());
        let html = render(&c);
        assert!(html.contains(">0</text>"));
        assert!(!html.contains("var(--cronus-chart-1)"));
    }

    #[test]
    fn chrome_styles_value_text() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"gauge-chart\"] text {\n  fill: var(--cronus-fg); font-size: 1.5rem; font-weight: 500;\n}"));
    }
}
