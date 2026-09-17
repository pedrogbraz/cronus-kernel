//! Dedicated GaugeChart renderer.
//!
//! Default (recharts `RadialBarChart`, docs "Default"): `<div
//! data-slot="gauge-chart" role="img" aria-label>` › `<div data-slot="chart">
//! <svg viewBox="0 0 432 256">` a `#eee` background track (recharts default),
//! the value sector (`value` / `max`, radii 83..107, corner 8, 210° → -30°,
//! chart-1) revealed by a clockwise sweep mask (RadialBar animation, 1500ms
//! ease) and the value in the hub (`text-2xl font-medium`).
//!
//! Motion (`style:gauge-chart+motion`, `@cronus-ui/ui/charts`): the visx
//! `Gauge` — `<div data-slot="gauge-chart" class="v-motion">` sized like the
//! docs wrapper (`max-w-lg`, `aspect-[21/16]`): 40 notches from 135° to 405°
//! (`spacing` 25% of the sweep in gaps, notch width 80% of its slot, outer
//! radius 42% and depth 14% of the size), a `--cronus-border` track at
//! `inactive-opacity` and `round(value / 100 × 40)` active chart-1 notches;
//! `center:428000` + `label:"ARR run rate"` render the `PieCenterShell` stat
//! (`format:currency` → `$428,000`). Notches pop in with a spring
//! (15ms / 20ms stagger).

use crate::cronus_ui_chart::{
    center_stat, container, int_fmt, num, rounded_sector_path, sweep_mask, usd, POLAR_CX, POLAR_CY,
};
use crate::cronus_ui_kit::{attr, attr_num, choice, esc, instance_id, label_of, numeric_items};
use crate::parser::ComponentNode;

pub const GAUGE_INNER: f64 = 83.0;
pub const GAUGE_OUTER: f64 = 107.0;
pub const GAUGE_CORNER: f64 = 8.0;
pub const GAUGE_START: f64 = 210.0;
pub const GAUGE_END: f64 = -30.0;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let max = attr_num::<f64>(comp, "max")
        .filter(|m| m.is_finite() && *m > 0.0)
        .unwrap_or(100.0);
    let value = attr_num::<f64>(comp, "value")
        .or_else(|| numeric_items(comp).first().copied())
        .unwrap_or(72.0);
    let clamped = if value.is_finite() {
        value.clamp(0.0, max)
    } else {
        0.0
    };
    if choice(comp, "variant", &["motion"]) == Some("motion") {
        return render_motion(comp, &label, clamped);
    }
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
        let mask = instance_id(comp, "sweep");
        body.push_str(&format!(
            "<defs>{}</defs><g mask=\"url(#{mask})\"><path d=\"{}\" fill=\"var(--cronus-chart-1)\"></path></g>",
            sweep_mask(
                &mask,
                POLAR_CX,
                POLAR_CY,
                GAUGE_INNER,
                GAUGE_OUTER,
                -GAUGE_START,
                GAUGE_START - end,
                true,
                ""
            ),
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

/// visx notch `Gauge` in the docs wrapper (432 × 329.14 at the audit width).
fn render_motion(comp: &ComponentNode, label: &str, value: f64) -> String {
    crate::cronus_ui_chart::note_motion();
    let width: f64 = 432.0;
    let height = width * 16.0 / 21.0;
    let size = width.min(height);
    let (cx, cy) = (width / 2.0, height / 2.0);
    let outer = size * 0.42;
    let inner = size * 0.28;
    let total = attr_num::<usize>(comp, "notches").unwrap_or(40).max(1);
    let spacing = attr_num::<f64>(comp, "spacing").unwrap_or(25.0);
    let (start, end) = (135.0_f64, 405.0_f64);
    let total_angle = end - start;
    let available = total_angle * (1.0 - spacing / 100.0);
    let notch_angle = available / total as f64;
    let gap_angle = total_angle * (spacing / 100.0) / (total as f64 - 1.0).max(1.0);
    let active = ((value / 100.0) * total as f64).round() as usize;
    let inactive_opacity = attr_num::<f64>(comp, "inactive-opacity").unwrap_or(0.8);
    let mut notches = String::new();
    let mut paths = Vec::with_capacity(total);
    for i in 0..total {
        let angle = start + i as f64 * (notch_angle + gap_angle) + notch_angle / 2.0;
        let rad = angle.to_radians();
        let half = (notch_angle * 0.8).to_radians() / 2.0;
        let (x1, y1) = (
            cx + (rad - half).cos() * outer,
            cy + (rad - half).sin() * outer,
        );
        let (x2, y2) = (
            cx + (rad + half).cos() * outer,
            cy + (rad + half).sin() * outer,
        );
        let (x3, y3) = (
            cx + (rad + half).cos() * inner,
            cy + (rad + half).sin() * inner,
        );
        let (x4, y4) = (
            cx + (rad - half).cos() * inner,
            cy + (rad - half).sin() * inner,
        );
        let d = format!(
            "M {} {} L {} {} L {} {} L {} {} Z",
            num(x1),
            num(y1),
            num(x2),
            num(y2),
            num(x3),
            num(y3),
            num(x4),
            num(y4)
        );
        notches.push_str(&format!(
            "<path class=\"notch i{i}\" d=\"{d}\" fill=\"var(--cronus-border)\" fill-opacity=\"{}\"></path>",
            num(inactive_opacity)
        ));
        paths.push(d);
    }
    for (i, d) in paths.iter().enumerate().take(active) {
        notches.push_str(&format!(
            "<path class=\"notch a i{i}\" d=\"{d}\" fill=\"var(--cronus-chart-1)\" fill-opacity=\"1\"></path>"
        ));
    }
    let hub = match attr_num::<f64>(comp, "center") {
        Some(c) => {
            let text = match attr(comp, "format") {
                Some("currency") => usd(c, 0),
                Some("percent") => format!("{}%", int_fmt(c)),
                _ => int_fmt(c),
            };
            center_stat(&text, attr(comp, "label").unwrap_or("Total"))
        }
        None => String::new(),
    };
    format!(
        "<div data-slot=\"gauge-chart\" class=\"v-motion\" role=\"img\" aria-label=\"{}\"><svg viewBox=\"0 0 {} {}\" aria-hidden=\"true\">{notches}</svg>{hub}</div>",
        esc(label),
        num(width),
        num(height)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn fixture() -> ComponentNode {
        let mut c = stub("gauge-chart", "Score");
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
        // RadialBar sweep: 72% of the 240° arc, clockwise from the 210° start.
        assert!(html.contains("<mask id=\"cui-gauge-chart-sweep\"><circle class=\"sweep\" cx=\"216\" cy=\"128\" r=\"95\" fill=\"none\" stroke=\"white\" stroke-width=\"26\" pathLength=\"2.0833\" transform=\"rotate(-210 216 128)\"></circle></mask>"));
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
    fn motion_variant_draws_notches_and_currency_hub() {
        let mut c = stub("gauge-chart", "ARR");
        c.style = Some("gauge-chart+motion".into());
        c.props.insert("value".into(), "66".into());
        c.props.insert("center".into(), "428000".into());
        c.props.insert("label".into(), "ARR run rate".into());
        c.props.insert("format".into(), "currency".into());
        c.props.insert("inactive-opacity".into(), "0.4".into());
        c.props.insert("spacing".into(), "25".into());
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"gauge-chart\" class=\"v-motion\" role=\"img\" aria-label=\"ARR\"><svg viewBox=\"0 0 432 329.1429\" aria-hidden=\"true\"><path class=\"notch i0\" d=\"M "));
        // 40 track notches at 0.4 + round(66% × 40) = 26 active chart-1 notches.
        assert_eq!(
            html.matches("fill=\"var(--cronus-border)\" fill-opacity=\"0.4\"")
                .count(),
            40
        );
        assert_eq!(html.matches("class=\"notch a i").count(), 26);
        assert!(html.contains("<path class=\"notch a i25\""));
        assert!(!html.contains("<path class=\"notch a i26\""));
        // First notch sits at 135° + half a notch: outer radius 138.24 around (216, 164.57).
        assert!(html.contains("<path class=\"notch i0\" d=\"M 117."));
        assert!(html.ends_with("</svg><div class=\"chart-center\"><span>$428,000</span><span>ARR run rate</span></div></div>"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_styles_value_text() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"gauge-chart\"] text {\n  fill: var(--cronus-fg); font-size: 1.5rem; font-weight: 500;\n}"));
        assert!(css.contains("[data-slot=\"gauge-chart\"].v-motion .notch {"));
        assert!(css.contains(".notch.a.i25 { animation-delay: 800ms; }"));
    }
}
