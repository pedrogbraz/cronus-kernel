//! Dedicated SankeyChart renderer.
//!
//! Default (recharts `Sankey`, docs "Default"): `<div data-slot="sankey-chart"
//! role="img" aria-label>` › `<div data-slot="chart"><svg viewBox="0 0 432 256">`
//! the recharts layout (justify align, 32 relax iterations, node collision
//! resolution) inside the 5px margin: `rect.recharts-sankey-node` (`#0088fe`
//! at 0.8, `node-width` 12) and cubic `path.recharts-sankey-link` (`#333` at
//! 0.2, `node-padding` 16, curvature 0.5). Recharts draws no labels.
//!
//! Motion (`style:sankey-chart+motion`, `@cronus-ui/ui/charts`): the visx
//! `SankeyChart` — `<div data-slot="sankey-chart" class="v-motion">` (`aspect`
//! 16 / 9, margins 40 / 180 / 40 / 180) laid out by d3-sankey (`sankeyCenter`,
//! `node-width`, `node-padding`): `SankeyLink` paths stroked by a source →
//! target gradient at 0.5 that draw in with a dash, `SankeyNode` rects
//! (chart-1..5 by node index, `line-cap` radius) growing from their middle
//! with `labels:vertical` name / `N sessions` labels rotated along the node.
//! Hovering a node or link dims the rest.
//!
//! Data: node items in order (`item "Direct"`) then links `item "Direct ->
//! Site" value:48`; nodes not declared are added in order of appearance.

use crate::cronus_ui_chart::{
    chart_color, container, int_fmt, num, sankey_layout, sankey_link_path, Frame, M_W,
};
use crate::cronus_ui_kit::{attr, attr_num, choice, esc, instance_id, label_of};
use crate::parser::ComponentNode;

struct Flow {
    source: String,
    target: String,
    value: f64,
}

/// Declared nodes (items without `->`) and links.
fn graph(comp: &ComponentNode) -> (Vec<String>, Vec<(usize, usize, f64)>) {
    let mut nodes: Vec<String> = Vec::new();
    let mut flows: Vec<Flow> = Vec::new();
    for row in crate::cronus_ui_data::rows() {
        let src = row.get("source").and_then(|v| v.as_str()).unwrap_or("");
        let dst = row.get("target").and_then(|v| v.as_str()).unwrap_or("");
        let val = row
            .get("value")
            .and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
            .unwrap_or(1.0);
        if !src.is_empty() && !dst.is_empty() {
            flows.push(Flow {
                source: src.into(),
                target: dst.into(),
                value: val,
            });
        }
    }
    if flows.is_empty() {
        for i in &comp.items {
            if !matches!(i.item_type.as_str(), "item" | "text" | "option") {
                continue;
            }
            let val = i
                .config
                .get("value")
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(1.0);
            let src = i.config.get("source").map(String::as_str);
            let dst = i.config.get("target").map(String::as_str);
            if let (Some(s), Some(t)) = (src, dst) {
                flows.push(Flow {
                    source: s.into(),
                    target: t.into(),
                    value: val,
                });
                continue;
            }
            if let Some((a, b)) = i.text.split_once("->") {
                flows.push(Flow {
                    source: a.trim().into(),
                    target: b.trim().into(),
                    value: val,
                });
            } else if !i.text.trim().is_empty() {
                nodes.push(i.text.trim().to_string());
            }
        }
    }
    if flows.is_empty() {
        flows.push(Flow {
            source: "In".into(),
            target: "Out".into(),
            value: 1.0,
        });
    }
    for f in &flows {
        for n in [&f.source, &f.target] {
            if !nodes.contains(n) {
                nodes.push(n.clone());
            }
        }
    }
    let idx = |n: &str| nodes.iter().position(|x| x == n).unwrap_or(0);
    let links = flows
        .iter()
        .map(|f| (idx(&f.source), idx(&f.target), f.value))
        .collect();
    (nodes, links)
}

#[derive(Clone)]
struct RNode {
    x: f64,
    y: f64,
    dx: f64,
    dy: f64,
    value: f64,
    depth: usize,
    source_links: Vec<usize>,
    target_links: Vec<usize>,
}

/// recharts `computeData`: `(nodes[x, y, dx, dy], links[dy, sy, ty])`.
fn recharts_layout(
    n: usize,
    links: &[(usize, usize, f64)],
    width: f64,
    height: f64,
    node_width: f64,
    node_padding: f64,
    iterations: usize,
) -> (Vec<RNode>, Vec<(f64, f64, f64)>) {
    let value = |l: usize| links[l].2;
    let mut tree: Vec<RNode> = (0..n)
        .map(|i| {
            let target_links: Vec<usize> = (0..links.len()).filter(|l| links[*l].0 == i).collect();
            let source_links: Vec<usize> = (0..links.len()).filter(|l| links[*l].1 == i).collect();
            let out: f64 = target_links.iter().map(|l| value(*l)).sum();
            let inn: f64 = source_links.iter().map(|l| value(*l)).sum();
            RNode {
                x: 0.0,
                y: 0.0,
                dx: node_width,
                dy: 0.0,
                value: out.max(inn),
                depth: 0,
                source_links,
                target_links,
            }
        })
        .collect();
    fn update_depth(tree: &mut [RNode], links: &[(usize, usize, f64)], cur: usize) {
        let targets: Vec<usize> = tree[cur].target_links.iter().map(|l| links[*l].1).collect();
        for t in targets {
            let nd = tree[cur].depth + 1;
            if nd > tree[t].depth {
                tree[t].depth = nd;
                update_depth(tree, links, t);
            }
        }
    }
    for i in 0..n {
        if tree[i].source_links.is_empty() {
            update_depth(&mut tree, links, i);
        }
    }
    let max_depth = tree.iter().map(|t| t.depth).max().unwrap_or(0);
    if max_depth >= 1 {
        let child_width = (width - node_width) / max_depth as f64;
        for node in tree.iter_mut() {
            if node.target_links.is_empty() {
                node.depth = max_depth;
            }
            node.x = node.depth as f64 * child_width;
        }
    }
    let mut depth_tree: Vec<Vec<usize>> = vec![Vec::new(); max_depth + 1];
    for (i, node) in tree.iter().enumerate() {
        depth_tree[node.depth].push(i);
    }
    let y_ratio = depth_tree
        .iter()
        .map(|nodes| {
            let v: f64 = nodes.iter().map(|i| tree[*i].value).sum();
            if v == 0.0 {
                f64::INFINITY
            } else {
                (height - (nodes.len() as f64 - 1.0) * node_padding) / v
            }
        })
        .fold(f64::INFINITY, f64::min);
    let y_ratio = if y_ratio.is_finite() { y_ratio } else { 0.0 };
    for nodes in &depth_tree {
        for (i, id) in nodes.iter().enumerate() {
            tree[*id].y = i as f64;
            tree[*id].dy = tree[*id].value * y_ratio;
        }
    }
    let link_dy: Vec<f64> = links.iter().map(|l| l.2 * y_ratio).collect();
    let center_y = |t: &RNode| t.y + t.dy / 2.0;
    let resolve = |tree: &mut Vec<RNode>, depth_tree: &mut Vec<Vec<usize>>| {
        for nodes in depth_tree.iter_mut() {
            nodes.sort_by(|a, b| {
                tree[*a]
                    .y
                    .partial_cmp(&tree[*b].y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let mut y0 = 0.0;
            for id in nodes.iter() {
                let dy = y0 - tree[*id].y;
                if dy > 0.0 {
                    tree[*id].y += dy;
                }
                y0 = tree[*id].y + tree[*id].dy + node_padding;
            }
            y0 = height + node_padding;
            for id in nodes.iter().rev() {
                let dy = tree[*id].y + tree[*id].dy + node_padding - y0;
                if dy > 0.0 {
                    tree[*id].y -= dy;
                    y0 = tree[*id].y;
                } else {
                    break;
                }
            }
        }
    };
    resolve(&mut tree, &mut depth_tree);
    let mut alpha = 1.0;
    for _ in 0..iterations {
        alpha *= 0.99;
        // relaxRightToLeft
        for d in (0..depth_tree.len()).rev() {
            for k in 0..depth_tree[d].len() {
                let id = depth_tree[d][k];
                if tree[id].target_links.is_empty() {
                    continue;
                }
                let sum: f64 = tree[id].target_links.iter().map(|l| value(*l)).sum();
                let weighted: f64 = tree[id]
                    .target_links
                    .iter()
                    .map(|l| center_y(&tree[links[*l].1]) * value(*l))
                    .sum();
                let y = if sum == 0.0 {
                    center_y(&tree[id])
                } else {
                    weighted / sum
                };
                let c = center_y(&tree[id]);
                tree[id].y += (y - c) * alpha;
            }
        }
        resolve(&mut tree, &mut depth_tree);
        // relaxLeftToRight
        for d in 0..depth_tree.len() {
            for k in 0..depth_tree[d].len() {
                let id = depth_tree[d][k];
                if tree[id].source_links.is_empty() {
                    continue;
                }
                let sum: f64 = tree[id].source_links.iter().map(|l| value(*l)).sum();
                let weighted: f64 = tree[id]
                    .source_links
                    .iter()
                    .map(|l| center_y(&tree[links[*l].0]) * value(*l))
                    .sum();
                let y = if sum == 0.0 {
                    center_y(&tree[id])
                } else {
                    weighted / sum
                };
                let c = center_y(&tree[id]);
                tree[id].y += (y - c) * alpha;
            }
        }
        resolve(&mut tree, &mut depth_tree);
    }
    let mut link_sy = vec![0.0; links.len()];
    let mut link_ty = vec![0.0; links.len()];
    let update_links = |tree: &mut Vec<RNode>, sy: &mut Vec<f64>, ty: &mut Vec<f64>| {
        for i in 0..tree.len() {
            let mut targets = tree[i].target_links.clone();
            targets.sort_by(|a, b| {
                tree[links[*a].1]
                    .y
                    .partial_cmp(&tree[links[*b].1].y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let mut sources = tree[i].source_links.clone();
            sources.sort_by(|a, b| {
                tree[links[*a].0]
                    .y
                    .partial_cmp(&tree[links[*b].0].y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let mut s = 0.0;
            for l in &targets {
                sy[*l] = s;
                s += link_dy[*l];
            }
            let mut t = 0.0;
            for l in &sources {
                ty[*l] = t;
                t += link_dy[*l];
            }
            tree[i].target_links = targets;
            tree[i].source_links = sources;
        }
    };
    update_links(&mut tree, &mut link_sy, &mut link_ty);
    // resolveNodeLinkCollisions: nodes fully inside a link that crosses their column.
    let depth_of: Vec<usize> = tree.iter().map(|t| t.depth).collect();
    for d in 0..depth_tree.len() {
        let nodes = depth_tree[d].clone();
        let Some(first) = nodes.first() else { continue };
        let depth_x = tree[*first].x + tree[*first].dx / 2.0;
        let mut obstacles: Vec<(f64, f64)> = Vec::new();
        for (l, (s, t, _)) in links.iter().enumerate() {
            let (sd, td) = (depth_of[*s], depth_of[*t]);
            if d <= sd.min(td) || d >= sd.max(td) {
                continue;
            }
            let sx = tree[*s].x + tree[*s].dx;
            let tx = tree[*t].x;
            let progress = if tx == sx {
                0.0
            } else {
                ((depth_x - sx) / (tx - sx)).clamp(0.0, 1.0)
            };
            let sy = tree[*s].y + link_sy[l] + link_dy[l] / 2.0;
            let ty = tree[*t].y + link_ty[l] + link_dy[l] / 2.0;
            let inv = 1.0 - progress;
            let y = inv.powi(3) * sy
                + 3.0 * inv.powi(2) * progress * sy
                + 3.0 * inv * progress.powi(2) * ty
                + progress.powi(3) * ty;
            obstacles.push((y - link_dy[l] / 2.0, link_dy[l]));
        }
        let contained: Vec<(f64, f64)> = obstacles
            .into_iter()
            .filter(|(oy, ody)| {
                nodes
                    .iter()
                    .any(|id| tree[*id].y >= *oy && tree[*id].y + tree[*id].dy <= oy + ody)
            })
            .collect();
        if contained.is_empty() {
            continue;
        }
        // items: (is_fixed, node id or obstacle index)
        let mut items: Vec<(bool, usize)> = nodes.iter().map(|id| (false, *id)).collect();
        items.extend((0..contained.len()).map(|k| (true, k)));
        let item_y = |tree: &Vec<RNode>, it: &(bool, usize)| {
            if it.0 {
                contained[it.1].0
            } else {
                tree[it.1].y
            }
        };
        items.sort_by(|a, b| {
            item_y(&tree, a)
                .partial_cmp(&item_y(&tree, b))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut next_y: f64 = 0.0;
        for it in &items {
            if it.0 {
                next_y = next_y.max(contained[it.1].0 + contained[it.1].1 + node_padding);
                continue;
            }
            if tree[it.1].y < next_y {
                tree[it.1].y = next_y;
            }
            next_y = tree[it.1].y + tree[it.1].dy + node_padding;
        }
        items.sort_by(|a, b| {
            item_y(&tree, a)
                .partial_cmp(&item_y(&tree, b))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut previous_y = height + node_padding;
        for it in items.iter().rev() {
            if it.0 {
                previous_y = previous_y.min(contained[it.1].0 - node_padding);
                continue;
            }
            let dy = tree[it.1].y + tree[it.1].dy + node_padding - previous_y;
            if dy > 0.0 {
                tree[it.1].y -= dy;
            }
            previous_y = tree[it.1].y;
        }
    }
    update_links(&mut tree, &mut link_sy, &mut link_ty);
    let out_links = (0..links.len())
        .map(|l| (link_dy[l], link_sy[l], link_ty[l]))
        .collect();
    (tree, out_links)
}

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let (names, links) = graph(comp);
    if choice(comp, "variant", &["motion"]) == Some("motion") {
        return render_motion(comp, &label, &names, &links);
    }
    let node_width = attr_num::<f64>(comp, "node-width").unwrap_or(12.0);
    let node_padding = attr_num::<f64>(comp, "node-padding").unwrap_or(16.0);
    let (left, top) = (5.0, 5.0);
    let (tree, layout) = recharts_layout(
        names.len(),
        &links,
        422.0,
        246.0,
        node_width,
        node_padding,
        32,
    );
    let mut body = String::from("<g class=\"recharts-sankey-links\">");
    for (l, (s, t, _)) in links.iter().enumerate() {
        let (dy, sy, ty) = layout[l];
        let sx = tree[*s].x + tree[*s].dx + left;
        let tx = tree[*t].x + left;
        let cx = (sx + tx) / 2.0;
        let y0 = tree[*s].y + sy + dy / 2.0 + top;
        let y1 = tree[*t].y + ty + dy / 2.0 + top;
        body.push_str(&format!(
            "<path class=\"recharts-sankey-link\" d=\"M{sx},{y0}C{cx},{y0} {cx},{y1} {tx},{y1}\" fill=\"none\" stroke=\"#333\" stroke-width=\"{}\" stroke-opacity=\"0.2\"></path>",
            num(dy),
            sx = num(sx),
            tx = num(tx),
            cx = num(cx),
            y0 = num(y0),
            y1 = num(y1),
        ));
    }
    body.push_str("</g><g class=\"recharts-sankey-nodes\">");
    for node in &tree {
        body.push_str(&format!(
            "<rect class=\"recharts-sankey-node\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#0088fe\" fill-opacity=\"0.8\"></rect>",
            num(node.x + left),
            num(node.y + top),
            num(node.dx),
            num(node.dy)
        ));
    }
    body.push_str("</g>");
    format!(
        "<div data-slot=\"sankey-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

/// visx `SankeyChart` (d3-sankey, `sankeyCenter`) at the 432px reference width.
fn render_motion(
    comp: &ComponentNode,
    label: &str,
    names: &[String],
    links: &[(usize, usize, f64)],
) -> String {
    crate::cronus_ui_chart::note_motion();
    let aspect = attr(comp, "aspect")
        .and_then(|a| {
            let (w, h) = a.split_once('/')?;
            Some(w.trim().parse::<f64>().ok()? / h.trim().parse::<f64>().ok()?)
        })
        .unwrap_or(2.0);
    let frame = Frame::aspect(M_W, aspect).margins(40.0, 180.0, 40.0, 180.0);
    let (iw, ih) = (frame.iw(), frame.ih());
    let node_width = attr_num::<f64>(comp, "node-width").unwrap_or(16.0);
    let node_padding = attr_num::<f64>(comp, "node-padding").unwrap_or(24.0);
    let g = sankey_layout(names, links, iw, ih, node_width, node_padding);
    let rx = attr_num::<f64>(comp, "line-cap").unwrap_or(4.0);
    let vertical = attr(comp, "labels") == Some("vertical");
    let n_links = g.links.len().max(1);
    let n_nodes = g.nodes.len().max(1);
    let mut defs = String::new();
    let mut link_html = String::new();
    for (i, l) in g.links.iter().enumerate() {
        let gid = instance_id(comp, &format!("link-gradient-{i}"));
        defs.push_str(&format!(
            "<linearGradient gradientUnits=\"userSpaceOnUse\" id=\"{gid}\" x1=\"{}\" x2=\"{}\" y1=\"0\" y2=\"0\"><stop offset=\"0%\" stop-color=\"{}\" stop-opacity=\"1\"></stop><stop offset=\"100%\" stop-color=\"{}\" stop-opacity=\"1\"></stop></linearGradient>",
            num(g.nodes[l.source].x1),
            num(g.nodes[l.target].x0),
            chart_color(l.source),
            chart_color(l.target)
        ));
        link_html.push_str(&format!(
            "<path class=\"link i{i} n{n_links}\" d=\"{}\" fill=\"none\" pathLength=\"1\" stroke=\"url(#{gid})\" stroke-opacity=\"0.5\" stroke-width=\"{}\"></path>",
            sankey_link_path(&g, l),
            num(l.width.max(1.0))
        ));
    }
    let mut node_html = String::new();
    for (i, node) in g.nodes.iter().enumerate() {
        let (x, y, w, h) = (node.x0, node.y0, node.x1 - node.x0, node.y1 - node.y0);
        let left_side = x < iw / 2.0;
        let cy = y + h / 2.0;
        let label_x = if left_side { x - 12.0 } else { x + w + 12.0 };
        let (name_x, value_x, anchor, rotate) = if vertical {
            let half = 8.0;
            (
                if left_side { half } else { -half },
                if left_side { -half } else { half },
                "middle",
                if left_side { -90.0 } else { 90.0 },
            )
        } else {
            (0.0, 0.0, if left_side { "end" } else { "start" }, 0.0)
        };
        let value_y = if vertical { cy } else { cy + 16.0 };
        node_html.push_str(&format!(
            "<g class=\"node n{n_nodes}\"><rect class=\"bar i{i}\" fill=\"{}\" height=\"{}\" rx=\"{rx}\" ry=\"{rx}\" width=\"{}\" x=\"{}\" y=\"{}\"></rect><g class=\"name i{i}\" transform=\"translate({lx},{cy}) rotate({rot})\"><text dy=\"0.35em\" text-anchor=\"{anchor}\" x=\"{}\">{}</text></g><g class=\"value i{i}\" transform=\"translate({lx},{vy}) rotate({rot})\"><text dy=\"0.35em\" text-anchor=\"{anchor}\" x=\"{}\">{} sessions</text></g></g>",
            chart_color(i),
            num(h),
            num(w),
            num(x),
            num(y),
            num(name_x),
            esc(&node.name),
            num(value_x),
            int_fmt(node.value),
            rx = num(rx),
            lx = num(label_x),
            cy = num(cy),
            vy = num(value_y),
            rot = num(rotate),
        ));
    }
    format!(
        "<div data-slot=\"sankey-chart\" class=\"v-motion\" role=\"img\" aria-label=\"{}\"><svg viewBox=\"0 0 {} {}\" aria-hidden=\"true\"><g transform=\"translate({},{})\"><g class=\"sankey-links\"><defs>{defs}</defs>{link_html}</g><g class=\"sankey-nodes\">{node_html}</g></g></svg></div>",
        esc(label),
        num(frame.w),
        num(frame.h),
        num(frame.left),
        num(frame.top),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn item(text: &str, value: Option<&str>) -> ComponentItemNode {
        let mut it = ComponentItemNode {
            item_type: "item".into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        };
        if let Some(v) = value {
            it.config.insert("value".into(), v.into());
        }
        it
    }

    fn acquisition(style: &str) -> ComponentNode {
        let mut c = stub("sankey-chart", "Acquisition flow");
        c.style = Some(style.into());
        for n in ["Direct", "Ads", "Site", "App", "Paid", "Churn"] {
            c.items.push(item(n, None));
        }
        for (l, v) in [
            ("Direct -> Site", "48"),
            ("Ads -> Site", "32"),
            ("Site -> App", "40"),
            ("Site -> Paid", "24"),
            ("App -> Paid", "28"),
            ("App -> Churn", "12"),
            ("Paid -> Churn", "8"),
        ] {
            c.items.push(item(l, Some(v)));
        }
        c
    }

    #[test]
    fn emits_svg_paths_not_the_figure_stub() {
        let mut c = stub("sankey-chart", "Flow");
        c.items.push(ComponentItemNode {
            item_type: "text".into(),
            text: "Paid -> Refund".into(),
            link: None,
            tone: None,
            config: Default::default(),
        });
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"sankey-chart\" role=\"img\""));
        assert!(html.contains("<svg viewBox=\"0 0 432 256\""));
        // Two nodes, justify align: Paid at x 5, Refund at the right edge (422 - 12 + 5).
        assert!(html.contains("<rect class=\"recharts-sankey-node\" x=\"5\" y=\"5\" width=\"12\" height=\"246\" fill=\"#0088fe\" fill-opacity=\"0.8\"></rect>"));
        assert!(html.contains("<rect class=\"recharts-sankey-node\" x=\"415\" y=\"5\""));
        assert!(html.contains("<path class=\"recharts-sankey-link\" d=\"M17,128C216,128 216,128 415,128\" fill=\"none\" stroke=\"#333\" stroke-width=\"246\" stroke-opacity=\"0.2\"></path>"));
        assert!(!html.contains("<figure"));
        assert!(!html.contains("style="));
        assert_eq!(
            dedicated_fn_name("sankey-chart"),
            Some("cronus_ui_sankey_chart::render")
        );
        assert_eq!(
            renderer_kind("sankey-chart"),
            RendererKind::Dedicated("cronus_ui_sankey_chart::render")
        );
    }

    #[test]
    fn declared_nodes_keep_their_order_and_columns() {
        let html = render(&acquisition("sankey-chart"));
        assert_eq!(html.matches("recharts-sankey-node\"").count(), 6);
        assert_eq!(html.matches("recharts-sankey-link\"").count(), 7);
        // Direct and Ads share the first column; Churn sits in the last (x = 5 + 410).
        assert_eq!(html.matches("x=\"5\" ").count(), 2);
        assert!(html.contains("x=\"415\" "));
        // Column pitch (422 - 12) / 4 = 102.5: Site at 107.5, App at 210, Paid at 312.5.
        assert!(html.contains("x=\"107.5\" "));
        assert!(html.contains("x=\"210\" "));
        assert!(html.contains("x=\"312.5\" "));
    }

    #[test]
    fn bound_rows_drive_flows() {
        use crate::binding::ResolvedData;
        crate::cronus_ui_data::with_binding(
            "Lead",
            &ResolvedData::Rows(vec![serde_json::json!({
                "source": "Paid",
                "target": "Refund",
                "value": 4
            })]),
            || {
                let html = render(&stub("sankey-chart", "Flow"));
                assert_eq!(html.matches("recharts-sankey-node\"").count(), 2);
                assert!(!html.contains(">In<"));
            },
        );
    }

    #[test]
    fn motion_variant_uses_d3_sankey_with_gradient_links_and_labels() {
        let mut c = acquisition("sankey-chart+motion");
        c.props.insert("aspect".into(), "16 / 9".into());
        c.props.insert("node-padding".into(), "24".into());
        c.props.insert("node-width".into(), "16".into());
        c.props.insert("labels".into(), "vertical".into());
        c.props.insert("line-cap".into(), "4".into());
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"sankey-chart\" class=\"v-motion\" role=\"img\" aria-label=\"Acquisition flow\"><svg viewBox=\"0 0 432 243\" aria-hidden=\"true\"><g transform=\"translate(180,40)\"><g class=\"sankey-links\"><defs><linearGradient gradientUnits=\"userSpaceOnUse\" id=\"cui-sankey-chart-link-gradient-0\" x1=\"16\" x2=\"14\" y1=\"0\" y2=\"0\"><stop offset=\"0%\" stop-color=\"var(--cronus-chart-1)\" stop-opacity=\"1\"></stop><stop offset=\"100%\" stop-color=\"var(--cronus-chart-3)\" stop-opacity=\"1\"></stop></linearGradient>"));
        // d3-sankey geometry (72 × 163 plot): Direct 0..83.4 tall, Site column at x 14.
        assert!(html.contains("<path class=\"link i0 n7\" d=\"M16,41.7C15,41.7,15,52.0057,14,52.0057\" fill=\"none\" pathLength=\"1\" stroke=\"url(#cui-sankey-chart-link-gradient-0)\" stroke-opacity=\"0.5\" stroke-width=\"83.4\"></path>"));
        assert!(html.contains("<g class=\"node n6\"><rect class=\"bar i0\" fill=\"var(--cronus-chart-1)\" height=\"83.4\" rx=\"4\" ry=\"4\" width=\"16\" x=\"0\" y=\"0\"></rect><g class=\"name i0\" transform=\"translate(-12,41.7) rotate(-90)\"><text dy=\"0.35em\" text-anchor=\"middle\" x=\"8\">Direct</text></g><g class=\"value i0\" transform=\"translate(-12,41.7) rotate(-90)\"><text dy=\"0.35em\" text-anchor=\"middle\" x=\"-8\">48 sessions</text></g></g>"));
        // Churn sits right of centre: label rotated +90 on its right edge.
        assert!(html.contains("<g class=\"name i5\" transform=\"translate(84,"));
        assert!(html.contains(
            "rotate(90)\"><text dy=\"0.35em\" text-anchor=\"middle\" x=\"-8\">Churn</text>"
        ));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"sankey-chart\"]"));
        assert!(css.contains("[data-slot=\"sankey-chart\"].v-motion .link {"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
