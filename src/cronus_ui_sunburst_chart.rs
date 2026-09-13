//! Dedicated SunburstChart renderer. DOM matches React:
//! `<div data-slot="sunburst-chart">` wrapping SVG annulus arcs/rings.
//! Token fills. Not the catalog `chart()` stub (`<figure><figcaption>`).

use crate::cronus_ui_kit::{fmt_coord, label_of, numeric_series};
use crate::parser::ComponentNode;

const SIZE: f64 = 200.0;
const CX: f64 = 100.0;
const CY: f64 = 100.0;
const INNER0: f64 = 36.0;
const INNER1: f64 = 64.0;
const OUTER0: f64 = 66.0;
const OUTER1: f64 = 94.0;
const FILLS: [&str; 4] = [
    "var(--cronus-primary)",
    "var(--cronus-success)",
    "var(--cronus-warning)",
    "var(--cronus-info)",
];

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let series = numeric_series(comp);
    let paths = sunburst_paths(&series);
    format!(
        "<div data-slot=\"sunburst-chart\" role=\"img\" aria-label=\"{label}\"><svg viewBox=\"0 0 {s} {s}\" aria-hidden=\"true\">{paths}</svg></div>",
        s = fmt_coord(SIZE),
    )
}

fn sunburst_paths(series: &[f64]) -> String {
    slices(series)
        .into_iter()
        .map(|(d, fill)| format!("<path d=\"{d}\" fill=\"{fill}\"></path>"))
        .collect::<Vec<_>>()
        .join("")
}

fn slices(series: &[f64]) -> Vec<(String, &'static str)> {
    let vals: Vec<f64> = if series.is_empty() {
        crate::cronus_ui_kit::DEFAULT_CHART_SERIES.to_vec()
    } else {
        series.to_vec()
    };
    let total: f64 = vals.iter().copied().sum();
    let n = vals.len().max(1) as f64;
    let mut angle = -std::f64::consts::FRAC_PI_2;
    let mut out = Vec::new();
    for (index, value) in vals.iter().enumerate() {
        let span = if total <= 0.0 {
            std::f64::consts::TAU / n
        } else {
            (*value / total) * std::f64::consts::TAU
        };
        let next = angle + span;
        let fill = FILLS[index % FILLS.len()];
        push_annulus(&mut out, INNER0, INNER1, angle, next, fill);
        let mid = angle + span * 0.62;
        let fill_a = FILLS[(index + 1) % FILLS.len()];
        let fill_b = FILLS[(index + 2) % FILLS.len()];
        push_annulus(&mut out, OUTER0, OUTER1, angle, mid, fill_a);
        push_annulus(&mut out, OUTER0, OUTER1, mid, next, fill_b);
        angle = next;
    }
    out
}

fn push_annulus(
    out: &mut Vec<(String, &'static str)>,
    r0: f64,
    r1: f64,
    a0: f64,
    a1: f64,
    fill: &'static str,
) {
    let sweep = a1 - a0;
    if sweep.abs() < 1e-9 {
        return;
    }
    if sweep.abs() >= std::f64::consts::TAU - 1e-6 {
        let mid = a0 + std::f64::consts::PI;
        out.push((arc_path(r0, r1, a0, mid), fill));
        out.push((arc_path(r0, r1, mid, a0 + std::f64::consts::TAU), fill));
        return;
    }
    out.push((arc_path(r0, r1, a0, a1), fill));
}

fn polar(r: f64, a: f64) -> (f64, f64) {
    (CX + r * a.cos(), CY + r * a.sin())
}

fn arc_path(r0: f64, r1: f64, a0: f64, a1: f64) -> String {
    let large = if (a1 - a0).abs() > std::f64::consts::PI {
        1
    } else {
        0
    };
    let (x0, y0) = polar(r1, a0);
    let (x1, y1) = polar(r1, a1);
    let (x2, y2) = polar(r0, a1);
    let (x3, y3) = polar(r0, a0);
    format!(
        "M {} {} A {} {} 0 {} 1 {} {} L {} {} A {} {} 0 {} 0 {} {} Z",
        fmt_coord(x0),
        fmt_coord(y0),
        fmt_coord(r1),
        fmt_coord(r1),
        large,
        fmt_coord(x1),
        fmt_coord(y1),
        fmt_coord(x2),
        fmt_coord(y2),
        fmt_coord(r0),
        fmt_coord(r0),
        large,
        fmt_coord(x3),
        fmt_coord(y3),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const STUB_POLYLINE: &str = "0,30 20,22 40,26 60,12 80,16 100,8 120,14";

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
        assert!(!html.contains("figcaption"));
        assert!(!html.contains(STUB_POLYLINE));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("{ value }"));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_is_div_with_svg_arcs_not_figure() {
        let html = render(&stub("sunburst-chart", "Share"));
        assert!(html.starts_with("<div data-slot=\"sunburst-chart\""));
        assert!(html.contains("role=\"img\""));
        assert!(html.contains("aria-label=\"Share\""));
        assert!(html.contains("<svg"));
        assert!(html.contains("<path "));
        assert!(html.contains(" A "));
        assert!(html.contains("fill=\"var(--cronus-primary)\""));
        assert!(html.contains("fill=\"var(--cronus-success)\""));
        assert!(html.contains("fill=\"var(--cronus-warning)\""));
        assert!(html.contains("fill=\"var(--cronus-info)\""));
        assert!(html.contains("viewBox=\"0 0 200 200\""));
        assert!(html.contains(" A 64 64 "));
        assert!(html.contains(" A 94 94 "));
        reject_stub(&html);
    }

    #[test]
    fn default_series_when_only_label() {
        let html = render(&stub("sunburst-chart", "Share"));
        let expected = slices(&[4.0, 8.0, 6.0, 10.0, 7.0]);
        assert_eq!(expected.len(), 15);
        assert_eq!(html.matches("<path ").count(), 15);
        assert!(html.contains(&format!("d=\"{}\"", expected[0].0)));
        reject_stub(&html);
    }

    #[test]
    fn numeric_items_drive_series() {
        let mut c = stub("sunburst-chart", "Share");
        c.items.push(extra("item", "1"));
        c.items.push(extra("item", "3"));
        c.items.push(extra("item", "2"));
        let html = render(&c);
        let expected = slices(&[1.0, 3.0, 2.0]);
        assert_eq!(expected.len(), 9);
        assert_eq!(html.matches("<path ").count(), 9);
        assert!(html.contains(&format!("d=\"{}\"", expected[0].0)));
        let def = slices(&[4.0, 8.0, 6.0, 10.0, 7.0]);
        assert_ne!(expected.len(), def.len());
        reject_stub(&html);
    }

    #[test]
    fn comma_list_item_is_series() {
        let mut c = stub("sunburst-chart", "Share");
        c.items.push(extra("item", "4, 8, 6"));
        let html = render(&c);
        assert_eq!(html.matches("<path ").count(), 9);
        reject_stub(&html);
    }

    #[test]
    fn skips_chart_figure_stub() {
        let html = render(&stub("sunburst-chart", "Share"));
        assert!(!html.contains("<figure"));
        assert!(!html.contains("<figcaption"));
        let area =
            crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("sankey-chart"))
                .unwrap();
        assert!(area.contains("<figure"));
        assert!(area.contains("data-slot=\"sankey-chart\""));
        assert!(area.contains(STUB_POLYLINE));
        assert_ne!(html, area);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("sunburst-chart", "Share"));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"sunburst-chart\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"sunburst-chart\"]"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
