//! Dedicated SunburstChart renderer.
//!
//! Default (`SunburstChart` from `@cronus-ui/ui`, docs "Default"): `<div
//! data-slot="sunburst-chart" role="img" aria-label>` › `<svg viewBox="0 0 200
//! 200">` two rings from 12 o'clock — top-level rows (`item "Product" value:48`,
//! radii 36..64, chart-(i+1)) and their children (`item "App" value:28
//! parent:"Product"`, radii 66..94, chart-(i+2)) with a `<title>` each.
//!
//! Motion (`style:sunburst-chart+motion`, `@cronus-ui/ui/charts`): the visx
//! `SunburstChart` — `<div data-slot="sunburst-chart" class="v-motion">`
//! (`size` 360, `root:"Revenue"`): the breadcrumb, a square SVG centred on
//! 0,0 whose `buildArcs` rings (radius = size / 2 - hover padding, equal ring
//! width per depth) hold one `SunburstSegment` per node — category colour
//! chart-1..5, fill-opacity 1 / 0.85 by depth, `--cronus-surface-base`
//! stroke — plus `SunburstLabels` (11px 600, surface-stroked) on the wide
//! segments and the `SunburstHint` line. Segments scale in from the centre
//! with an angular sweep (ring × 0.12s + index × 0.08s) and labels fade in
//! after them; hovering a segment dims the unrelated ones to 0.25.

use crate::cronus_ui_chart::{data_items, num, series_names, sweep_mask};
use crate::cronus_ui_kit::{
    attr, attr_num, choice, esc, fmt_coord, instance_id, label_of, numeric_items,
    DEFAULT_CHART_SERIES,
};
use crate::parser::ComponentNode;

const CX: f64 = 100.0;
const CY: f64 = 100.0;
const INNER0: f64 = 36.0;
const INNER1: f64 = 64.0;
const OUTER0: f64 = 66.0;
const OUTER1: f64 = 94.0;

type Leaf = (&'static str, f64);
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
    if choice(comp, "variant", &["motion"]) == Some("motion") {
        return render_motion(comp, &label);
    }
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

/// Rows with a `parent:` config are children; a parent without `value:`
/// sums its children. Without any `parent:` rows the legacy demo tree applies.
fn tree(comp: &ComponentNode) -> Vec<Node> {
    let items = data_items(comp);
    if items.iter().any(|i| i.config.contains_key("parent")) {
        let mut nodes: Vec<Node> = Vec::new();
        for i in &items {
            let value = i
                .config
                .get("value")
                .and_then(|v| v.trim().parse::<f64>().ok());
            match i.config.get("parent") {
                Some(parent) => {
                    let parent = parent.trim();
                    if !nodes.iter().any(|n| n.name == parent) {
                        nodes.push(Node {
                            name: parent.to_string(),
                            value: 0.0,
                            children: Vec::new(),
                        });
                    }
                    let node = nodes.iter_mut().find(|n| n.name == parent).unwrap();
                    node.children
                        .push((i.text.trim().to_string(), value.unwrap_or(0.0)));
                }
                None => nodes.push(Node {
                    name: i.text.trim().to_string(),
                    value: value.unwrap_or(0.0),
                    children: Vec::new(),
                }),
            }
        }
        for n in nodes.iter_mut() {
            if n.value == 0.0 {
                n.value = n.children.iter().map(|c| c.1).sum();
            }
        }
        return nodes;
    }
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

/// One `buildArcs` arc: normalised angles, depth, top-level category.
struct Arc {
    name: String,
    depth: usize,
    category: usize,
    a0: f64,
    a1: f64,
}

/// visx `arcPathFromRadii`: angles from 12 o'clock, x = sin, y = -cos.
fn visx_arc(a0: f64, a1: f64, inner: f64, outer: f64) -> String {
    let large = u8::from(a1 - a0 > std::f64::consts::PI);
    let (ox0, oy0) = (a0.sin() * outer, -a0.cos() * outer);
    let (ox1, oy1) = (a1.sin() * outer, -a1.cos() * outer);
    if inner < 1.0 {
        return format!(
            "M 0 0 L {} {} A {o} {o} 0 {large} 1 {} {} Z",
            num(ox0),
            num(oy0),
            num(ox1),
            num(oy1),
            o = num(outer)
        );
    }
    let (ix1, iy1) = (a1.sin() * inner, -a1.cos() * inner);
    let (ix0, iy0) = (a0.sin() * inner, -a0.cos() * inner);
    format!(
        "M {} {} A {o} {o} 0 {large} 1 {} {} L {} {} A {i} {i} 0 {large} 0 {} {} Z",
        num(ox0),
        num(oy0),
        num(ox1),
        num(oy1),
        num(ix1),
        num(iy1),
        num(ix0),
        num(iy0),
        o = num(outer),
        i = num(inner)
    )
}

/// visx `SunburstChart` with breadcrumb, segments, labels and hint.
fn render_motion(comp: &ComponentNode, label: &str) -> String {
    crate::cronus_ui_chart::note_motion();
    let size = attr_num::<f64>(comp, "size").unwrap_or(520.0);
    let root = attr(comp, "root").unwrap_or("Revenue");
    let nodes = tree(comp);
    let max_depth = if nodes.iter().any(|n| !n.children.is_empty()) {
        2
    } else {
        1
    };
    let hover_pop: f64 = 8.0;
    // defaultSunburstGrowPadding: one segment grow per path level plus one.
    let full = size / 2.0;
    let root_ring = full / max_depth as f64;
    let path_len = (max_depth as f64 - 1.0).max(1.0);
    let grow = hover_pop
        .min(root_ring * 0.1)
        .min(root_ring * 0.28 / path_len);
    let padding = (grow * path_len + grow).ceil();
    let radius = (full - padding).max(8.0);
    let ring_w = radius / max_depth as f64;
    let top = -std::f64::consts::FRAC_PI_2;
    let tau = std::f64::consts::TAU;
    // buildArcs (normalised) in tree order.
    let total: f64 = nodes
        .iter()
        .map(|n| n.value)
        .sum::<f64>()
        .max(f64::MIN_POSITIVE);
    let mut arcs: Vec<Arc> = Vec::new();
    let mut cursor = 0.0;
    for (ci, n) in nodes.iter().enumerate() {
        let span = n.value / total;
        arcs.push(Arc {
            name: n.name.clone(),
            depth: 1,
            category: ci,
            a0: cursor,
            a1: cursor + span,
        });
        let kid_total: f64 = n
            .children
            .iter()
            .map(|c| c.1)
            .sum::<f64>()
            .max(f64::MIN_POSITIVE);
        let mut inner = cursor;
        for (name, v) in &n.children {
            let ks = v / kid_total * span;
            arcs.push(Arc {
                name: name.clone(),
                depth: 2,
                category: ci,
                a0: inner,
                a1: inner + ks,
            });
            inner += ks;
        }
        cursor += span;
    }
    // Enter timing: per ring, ordered clockwise: (ring × 0.12 + index × 0.08)s.
    let mut delay_ms: Vec<f64> = vec![0.0; arcs.len()];
    for depth in 1..=max_depth {
        let mut ring: Vec<usize> = (0..arcs.len())
            .filter(|i| arcs[*i].depth == depth)
            .collect();
        ring.sort_by(|a, b| {
            arcs[*a]
                .a0
                .partial_cmp(&arcs[*b].a0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for (k, i) in ring.iter().enumerate() {
            delay_ms[*i] = ((depth as f64 - 1.0) * 0.12 + k as f64 * 0.08) * 1000.0;
        }
    }
    let max_delay = delay_ms.iter().copied().fold(0.0, f64::max);
    let mut defs = String::new();
    let mut segments = String::new();
    let mut labels = String::new();
    // Segments render deeper rings first (React sorts by depth descending).
    let mut order: Vec<usize> = (0..arcs.len()).collect();
    order.sort_by(|a, b| arcs[*b].depth.cmp(&arcs[*a].depth).then(a.cmp(b)));
    for i in order {
        let arc = &arcs[i];
        let (a0, a1) = (top + arc.a0 * tau, top + arc.a1 * tau);
        let inner = (arc.depth as f64 - 1.0) * ring_w;
        let outer = arc.depth as f64 * ring_w;
        let mask = instance_id(comp, &format!("segment-{i}"));
        let step = (delay_ms[i] / 10.0).round() as usize;
        defs.push_str(&sweep_mask(
            &mask,
            0.0,
            0.0,
            inner,
            outer,
            (a0 - std::f64::consts::FRAC_PI_2).to_degrees(),
            (a1 - a0).to_degrees(),
            true,
            &format!("d{step}"),
        ));
        let opacity = if arc.depth <= 1 {
            1.0
        } else {
            (1.0 - (arc.depth as f64 - 1.0) * 0.15).max(0.45)
        };
        segments.push_str(&format!(
            "<g class=\"segment d{step} c{}\"><g mask=\"url(#{mask})\"><path d=\"{}\" fill=\"var(--cronus-chart-{})\" fill-opacity=\"{}\" stroke=\"var(--cronus-surface-base)\" stroke-linejoin=\"round\" stroke-width=\"1\"><title>{}</title></path></g></g>",
            arc.category,
            visx_arc(a0, a1, inner, outer),
            arc.category % 5 + 1,
            num(opacity),
            esc(&arc.name)
        ));
        // SunburstLabels: centroid, rotated along the ring, only when wide enough.
        let r = (inner + outer) / 2.0;
        if (a1 - a0) * r >= 26.0 && outer - inner >= 16.0 {
            let mid = (a0 + a1) / 2.0;
            let (x, y) = (mid.sin() * r, -mid.cos() * r);
            let mut deg = mid.to_degrees() - 90.0;
            if deg > 90.0 {
                deg -= 180.0;
            }
            if deg < -90.0 {
                deg += 180.0;
            }
            labels.push_str(&format!(
                "<text dominant-baseline=\"middle\" text-anchor=\"middle\" transform=\"rotate({} {x} {y})\" x=\"{x}\" y=\"{y}\">{}</text>",
                num(deg),
                esc(&arc.name),
                x = num(x),
                y = num(y)
            ));
        }
    }
    let labels_delay = ((max_delay + 1100.0 * 0.85) / 100.0).round();
    let crumbs = format!(
        "<ol class=\"crumbs\"><li><span class=\"current\">{}</span></li></ol>",
        esc(root)
    );
    format!(
        "<div data-slot=\"sunburst-chart\" class=\"v-motion\" role=\"img\" aria-label=\"{}\">{crumbs}<div class=\"canvas\"><svg viewBox=\"{} {} {s} {s}\" width=\"100%\" aria-label=\"Sunburst chart of {}\" role=\"img\"><defs>{defs}</defs>{segments}<g class=\"labels l{}\">{labels}</g></svg></div><div class=\"hint\" aria-live=\"polite\">Click a segment to zoom in · hover to inspect</div></div>",
        esc(label),
        num(-full),
        num(-full),
        esc(root),
        labels_delay,
        s = num(size)
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

    fn node(text: &str, value: Option<&str>, parent: Option<&str>) -> ComponentItemNode {
        let mut it = extra("item", text);
        if let Some(v) = value {
            it.config.insert("value".into(), v.into());
        }
        if let Some(p) = parent {
            it.config.insert("parent".into(), p.into());
        }
        it
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
        let mut c = stub("sunburst-chart", "Traffic");
        for t in ["Desktop", "Mobile"] {
            c.items.push(extra("text", t));
        }
        let html = render(&c);
        assert_eq!(html.matches("<path ").count(), 5);
        assert!(html.contains("<path d=\"M 100 36 A 64 64 0 1 1 44.57 132 L 68.82 118 A 36 36 0 1 0 100 64 Z\" fill=\"var(--cronus-chart-1)\"><title>Desktop</title></path>"));
        assert!(html.contains("<path d=\"M 100 6 A 94 94 0 0 1 147 181.41 L 133 157.16 A 66 66 0 0 0 100 34 Z\" fill=\"var(--cronus-chart-2)\"><title>Chrome</title></path>"));
        assert!(html.contains("<title>iOS</title>"));
    }

    #[test]
    fn parent_rows_build_the_docs_tree() {
        let mut c = stub("sunburst-chart", "Revenue by product line");
        c.items.push(node("Product", Some("48"), None));
        c.items.push(node("App", Some("28"), Some("Product")));
        c.items.push(node("API", Some("20"), Some("Product")));
        c.items.push(node("Services", Some("32"), None));
        c.items.push(node("Support", Some("18"), Some("Services")));
        c.items
            .push(node("Consulting", Some("14"), Some("Services")));
        let html = render(&c);
        assert_eq!(html.matches("<path ").count(), 6);
        // Product 48 of 80 → 216° from 12 o'clock on the inner ring.
        assert!(html.contains("<path d=\"M 100 36 A 64 64 0 1 1 62.3"));
        assert!(html.contains("L 78.84 129.12 A 36 36 0 1 0 100 64 Z\" fill=\"var(--cronus-chart-1)\"><title>Product</title></path>"));
        assert!(html.contains("<title>Consulting</title>"));
        let fills: Vec<&str> = html
            .split("fill=\"var(--cronus-")
            .skip(1)
            .map(|s| &s[..7])
            .collect();
        assert_eq!(
            fills,
            vec!["chart-1", "chart-2", "chart-2", "chart-2", "chart-3", "chart-3"]
        );
    }

    #[test]
    fn motion_variant_builds_visx_arcs_labels_and_chrome() {
        let mut c = stub("sunburst-chart", "Revenue by product line");
        c.style = Some("sunburst-chart+motion".into());
        c.props.insert("root".into(), "Revenue".into());
        c.props.insert("size".into(), "360".into());
        c.items.push(node("Product", None, None));
        c.items
            .push(node("Enterprise", Some("198"), Some("Product")));
        c.items.push(node("Pro", Some("145"), Some("Product")));
        c.items.push(node("Services", None, None));
        c.items
            .push(node("Consulting", Some("160"), Some("Services")));
        c.items.push(node("Support", Some("90"), Some("Services")));
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"sunburst-chart\" class=\"v-motion\" role=\"img\" aria-label=\"Revenue by product line\"><ol class=\"crumbs\"><li><span class=\"current\">Revenue</span></li></ol><div class=\"canvas\"><svg viewBox=\"-180 -180 360 360\" width=\"100%\" aria-label=\"Sunburst chart of Revenue\" role=\"img\"><defs>"));
        // radius 164 (16px hover padding), ring width 82; leaves render first (depth 2 → 1).
        assert!(html.contains("<g class=\"segment d12 c0\"><g mask=\"url(#cui-sunburst-chart-segment-1)\"><path d=\"M -164 0 A 164 164 0 0 1"));
        assert!(html.contains("fill=\"var(--cronus-chart-1)\" fill-opacity=\"0.85\" stroke=\"var(--cronus-surface-base)\" stroke-linejoin=\"round\" stroke-width=\"1\"><title>Enterprise</title>"));
        // Product: 343 of 593 → inner ring from 9 o'clock, full opacity, delay 0.
        assert!(html.contains("<g class=\"segment d0 c0\"><g mask=\"url(#cui-sunburst-chart-segment-0)\"><path d=\"M 0 0 L -82 0 A 82 82 0 1 1"));
        assert!(html.contains("<g class=\"labels l13\">"));
        assert!(html
            .contains("dominant-baseline=\"middle\" text-anchor=\"middle\" transform=\"rotate("));
        assert!(html.contains(">Product</text>"));
        assert!(html.ends_with("</svg></div><div class=\"hint\" aria-live=\"polite\">Click a segment to zoom in · hover to inspect</div></div>"));
        reject_stub(&html);
    }

    #[test]
    fn chrome_sizes_svg_like_react_h_full_mx_auto() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"sunburst-chart\"] svg {\n  display: block; height: 100%; margin: 0 auto;\n}"
        ));
        assert!(css.contains("[data-slot=\"sunburst-chart\"].v-motion .segment {"));
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
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("sunburst-chart", "Share"));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"sunburst-chart\""));
        });
    }
}
