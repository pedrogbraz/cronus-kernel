//! Dedicated Sankey renderer. Zero JS SVG flows:
//! `<div data-slot="sankey-chart" role="img">` wrapping the shared chart
//! container. Links come from bound rows (`source`, `target`, `value`) or
//! item text `A -> B` / item config. Fallback is a two-node demo.

use crate::cronus_ui_chart::{container, num, VIEW_H, VIEW_W};
use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

struct Flow {
    source: String,
    target: String,
    value: f64,
}

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let flows = flows_of(comp);
    let mut left: Vec<String> = Vec::new();
    let mut right: Vec<String> = Vec::new();
    for f in &flows {
        if !left.contains(&f.source) {
            left.push(f.source.clone());
        }
        if !right.contains(&f.target) {
            right.push(f.target.clone());
        }
    }
    let mut body = String::new();
    let lh = VIEW_H / left.len().max(1) as f64;
    let rh = VIEW_H / right.len().max(1) as f64;
    for (i, name) in left.iter().enumerate() {
        let y = i as f64 * lh + 8.0;
        body.push_str(&format!(
            "<rect x=\"8\" y=\"{}\" width=\"72\" height=\"{}\" rx=\"4\" fill=\"var(--cronus-chart-1)\"></rect><text x=\"44\" y=\"{}\" text-anchor=\"middle\" fill=\"var(--cronus-fg)\" font-size=\"11\">{}</text>",
            num(y),
            num((lh - 16.0).max(12.0)),
            num(y + (lh - 16.0).max(12.0) / 2.0 + 4.0),
            esc(name)
        ));
    }
    for (i, name) in right.iter().enumerate() {
        let y = i as f64 * rh + 8.0;
        body.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"72\" height=\"{}\" rx=\"4\" fill=\"var(--cronus-chart-2)\"></rect><text x=\"{}\" y=\"{}\" text-anchor=\"middle\" fill=\"var(--cronus-fg)\" font-size=\"11\">{}</text>",
            num(VIEW_W - 80.0),
            num(y),
            num((rh - 16.0).max(12.0)),
            num(VIEW_W - 44.0),
            num(y + (rh - 16.0).max(12.0) / 2.0 + 4.0),
            esc(name)
        ));
    }
    for f in &flows {
        let li = left.iter().position(|n| n == &f.source).unwrap_or(0) as f64;
        let ri = right.iter().position(|n| n == &f.target).unwrap_or(0) as f64;
        let y1 = li * lh + lh / 2.0;
        let y2 = ri * rh + rh / 2.0;
        let x1 = 80.0;
        let x2 = VIEW_W - 80.0;
        let c = (x1 + x2) / 2.0;
        let sw = (2.0 + f.value.abs().sqrt() * 2.0).clamp(2.0, 14.0);
        body.push_str(&format!(
            "<path d=\"M {x1},{y1} C {c},{y1} {c},{y2} {x2},{y2}\" fill=\"none\" stroke=\"var(--cronus-chart-3)\" stroke-width=\"{sw}\" stroke-opacity=\"0.45\"></path>",
            x1 = num(x1),
            y1 = num(y1),
            c = num(c),
            y2 = num(y2),
            x2 = num(x2),
            sw = num(sw),
        ));
    }
    format!(
        "<div data-slot=\"sankey-chart\" role=\"img\" aria-label=\"{label}\">{}</div>",
        container(&body)
    )
}

fn flows_of(comp: &ComponentNode) -> Vec<Flow> {
    let mut out = Vec::new();
    for row in crate::cronus_ui_data::rows() {
        if let Some(f) = flow_from_map(
            row.get("source").and_then(|v| v.as_str()),
            row.get("target").and_then(|v| v.as_str()),
            row.get("value").and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            }),
        ) {
            out.push(f);
        }
    }
    if out.is_empty() {
        for i in &comp.items {
            let src = i.config.get("source").map(String::as_str);
            let dst = i.config.get("target").map(String::as_str);
            let val = i
                .config
                .get("value")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1.0);
            if let Some(f) = flow_from_map(src, dst, Some(val)) {
                out.push(f);
                continue;
            }
            if let Some((a, b)) = i.text.split_once("->") {
                out.push(Flow {
                    source: a.trim().to_string(),
                    target: b.trim().to_string(),
                    value: val,
                });
            }
        }
    }
    if out.is_empty() {
        out.push(Flow {
            source: "In".into(),
            target: "Out".into(),
            value: 1.0,
        });
    }
    out
}

fn flow_from_map(source: Option<&str>, target: Option<&str>, value: Option<f64>) -> Option<Flow> {
    Some(Flow {
        source: source.filter(|s| !s.is_empty())?.to_string(),
        target: target.filter(|s| !s.is_empty())?.to_string(),
        value: value.unwrap_or(1.0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

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
        assert!(html.contains("Paid"));
        assert!(html.contains("Refund"));
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
                assert!(html.contains("Paid"));
                assert!(html.contains("Refund"));
                assert!(!html.contains(">In<"));
            },
        );
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"sankey-chart\"]"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
