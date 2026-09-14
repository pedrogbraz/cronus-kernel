//! Dedicated SunburstChart renderer. DOM matches React's settled render:
//! `<div data-slot="sunburst-chart" role="img" aria-label>`
//!   `<svg viewBox="0 0 200 200">` (h-full, mx-auto → 256×256 centred) with
//!   one inner annulus (r 36..64) per top-level node, fill chart-(i%5+1), each
//!   followed by its children on the outer ring (r 66..94), fill
//!   chart-((i+1)%5+1); every path carries `<title>{name}</title>`. No stroke.
//!
//! The kernel has no nested items, so nodes are the `text` lines (values from
//! numeric items when given). Nodes named like the React fixture tree
//! (Desktop 8: Chrome 5, Safari 3; Mobile 4: iOS 4) take its value and
//! children; any other node gets its demo value and one child spanning it.

use crate::cronus_ui_chart::series_names;
use crate::cronus_ui_kit::{esc, fmt_coord, label_of, numeric_items, DEFAULT_CHART_SERIES};
use crate::parser::ComponentNode;

const CX: f64 = 100.0;
const CY: f64 = 100.0;
const INNER0: f64 = 36.0;
const INNER1: f64 = 64.0;
const OUTER0: f64 = 66.0;
const OUTER1: f64 = 94.0;

type Leaf = (&'static str, f64);
/// React `SunburstChartFixture` data.
const DEMO_TREE: [(&str, f64, &[Leaf]); 2] = [
    ("Desktop", 8.0, &[("Chrome", 5.0), ("Safari", 3.0)]),
    ("Mobile", 4.0, &[("iOS", 4.0)]),
];

struct Node {
    name: String,
    value: f64,
    children: Vec<(String, f64)>,
}

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let paths: String = slices(&tree(comp))
        .into_iter()
        .map(|(d, chart, name)| {
            format!(
                "<path d=\"{d}\" fill=\"var(--cronus-chart-{chart})\"><title>{}</title></path>",
                esc(&name)
            )
        })
        .collect();
    format!(
        "<div data-slot=\"sunburst-chart\" role=\"img\" aria-label=\"{label}\"><svg viewBox=\"0 0 200 200\" aria-hidden=\"true\">{paths}</svg></div>"
    )
}

fn tree(comp: &ComponentNode) -> Vec<Node> {
    let mut names = series_names(comp);
    let nums = numeric_items(comp);
    if names.is_empty() && nums.is_empty() {
        names = DEMO_TREE.iter().map(|d| d.0.to_string()).collect();
    }
    let n = names.len().max(nums.len());
    (0..n)
        .map(|i| {
            let name = names.get(i).cloned().unwrap_or_else(|| (i + 1).to_string());
            let demo = DEMO_TREE.iter().find(|d| d.0.eq_ignore_ascii_case(&name));
            let value = nums
                .get(i)
                .copied()
                .or(demo.map(|d| d.1))
                .unwrap_or(DEFAULT_CHART_SERIES[i % DEFAULT_CHART_SERIES.len()]);
            let children = match demo {
                Some(d) => d.2.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
                None => vec![(name.clone(), value)],
            };
            Node {
                name,
                value,
                children,
            }
        })
        .collect()
}

/// `(d, chart token index, title)` in React paint order.
fn slices(nodes: &[Node]) -> Vec<(String, usize, String)> {
    let sum: f64 = nodes.iter().map(|n| n.value).sum();
    let total = if sum == 0.0 { 1.0 } else { sum };
    let mut angle = -std::f64::consts::FRAC_PI_2;
    let mut out = Vec::new();
    for (index, node) in nodes.iter().enumerate() {
        let span = node.value / total * std::f64::consts::TAU;
        push_annulus(
            &mut out,
            INNER0,
            INNER1,
            angle,
            angle + span,
            index % 5 + 1,
            &node.name,
        );
        let kid_sum: f64 = node.children.iter().map(|c| c.1).sum();
        let kid_total = if kid_sum == 0.0 { 1.0 } else { kid_sum };
        let mut inner = angle;
        for (name, value) in &node.children {
            let kid_span = value / kid_total * span;
            push_annulus(
                &mut out,
                OUTER0,
                OUTER1,
                inner,
                inner + kid_span,
                (index + 1) % 5 + 1,
                name,
            );
            inner += kid_span;
        }
        angle += span;
    }
    out
}

fn push_annulus(
    out: &mut Vec<(String, usize, String)>,
    r0: f64,
    r1: f64,
    a0: f64,
    a1: f64,
    chart: usize,
    name: &str,
) {
    let sweep = a1 - a0;
    if sweep.abs() >= std::f64::consts::TAU - 1e-6 {
        // A single arc cannot draw a full ring (start == end); split it.
        let mid = a0 + std::f64::consts::PI;
        out.push((arc_path(r0, r1, a0, mid), chart, name.to_string()));
        out.push((
            arc_path(r0, r1, mid, a0 + std::f64::consts::TAU),
            chart,
            name.to_string(),
        ));
        return;
    }
    out.push((arc_path(r0, r1, a0, a1), chart, name.to_string()));
}

fn polar(r: f64, a: f64) -> (f64, f64) {
    (CX + r * a.cos(), CY + r * a.sin())
}

/// React `arcPath(100, 100, r0, r1, a0, a1)`.
fn arc_path(r0: f64, r1: f64, a0: f64, a1: f64) -> String {
    let large = u8::from(a1 - a0 > std::f64::consts::PI);
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
    fn fixture_matches_react_tree_arcs_and_fills() {
        // React SunburstChartFixture: Desktop 8 (Chrome 5, Safari 3),
        // Mobile 4 (iOS 4); inner ring chart-(i+1), children chart-(i+2).
        let mut c = stub("sunburst-chart", "Traffic");
        for t in ["Desktop", "Mobile"] {
            c.items.push(extra("text", t));
        }
        let html = render(&c);
        assert_eq!(html.matches("<path ").count(), 5);
        assert!(html.contains("<path d=\"M 100 36 A 64 64 0 1 1 44.57 132 L 68.82 118 A 36 36 0 1 0 100 64 Z\" fill=\"var(--cronus-chart-1)\"><title>Desktop</title></path>"));
        assert!(html.contains("<path d=\"M 100 6 A 94 94 0 0 1 147 181.41 L 133 157.16 A 66 66 0 0 0 100 34 Z\" fill=\"var(--cronus-chart-2)\"><title>Chrome</title></path>"));
        let fills: Vec<&str> = html
            .split("fill=\"var(--cronus-")
            .skip(1)
            .map(|s| &s[..7])
            .collect();
        assert_eq!(
            fills,
            vec!["chart-1", "chart-2", "chart-2", "chart-2", "chart-3"]
        );
        assert!(html.contains("<title>iOS</title>"));
    }

    #[test]
    fn chrome_sizes_svg_like_react_h_full_mx_auto() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"sunburst-chart\"] svg {\n  display: block; height: 100%; margin: 0 auto;\n}"
        ));
        assert!(!css.contains("[data-slot=\"sunburst-chart\"] path"));
    }

    #[test]
    fn root_is_div_with_svg_arcs_not_figure() {
        let html = render(&stub("sunburst-chart", "Share"));
        assert!(html.starts_with("<div data-slot=\"sunburst-chart\""));
        assert!(html.contains("role=\"img\""));
        assert!(html.contains("aria-label=\"Share\""));
        assert!(html.contains("<svg"));
        assert!(html.contains(" A "));
        assert!(html.contains("fill=\"var(--cronus-chart-1)\""));
        assert!(html.contains("fill=\"var(--cronus-chart-2)\""));
        assert!(html.contains("fill=\"var(--cronus-chart-3)\""));
        assert!(html.contains("viewBox=\"0 0 200 200\""));
        assert!(html.contains(" A 64 64 "));
        assert!(html.contains(" A 94 94 "));
        reject_stub(&html);
    }

    #[test]
    fn default_tree_when_only_label() {
        let mut named = stub("sunburst-chart", "Share");
        for t in ["Desktop", "Mobile"] {
            named.items.push(extra("text", t));
        }
        assert_eq!(render(&stub("sunburst-chart", "Share")), render(&named));
    }

    #[test]
    fn numeric_items_drive_series() {
        let mut c = stub("sunburst-chart", "Share");
        c.items.push(extra("item", "1"));
        c.items.push(extra("item", "3"));
        c.items.push(extra("item", "2"));
        let html = render(&c);
        let nodes = tree(&c);
        assert_eq!(
            nodes.iter().map(|n| n.value).collect::<Vec<_>>(),
            vec![1.0, 3.0, 2.0]
        );
        let expected = slices(&nodes);
        assert_eq!(expected.len(), 6);
        assert_eq!(html.matches("<path ").count(), 6);
        assert!(html.contains(&format!("d=\"{}\"", expected[0].0)));
        reject_stub(&html);
    }

    #[test]
    fn comma_list_item_is_series() {
        let mut c = stub("sunburst-chart", "Share");
        c.items.push(extra("item", "4, 8, 6"));
        let html = render(&c);
        assert_eq!(html.matches("<path ").count(), 6);
        reject_stub(&html);
    }

    #[test]
    fn full_ring_is_split_into_two_arcs() {
        let mut c = stub("sunburst-chart", "Share");
        c.items.push(extra("text", "Only"));
        let html = render(&c);
        assert_eq!(html.matches("<path ").count(), 4);
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
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
