//! Dedicated FunnelChart renderer. DOM matches React:
//! `<div data-slot="funnel-chart">` wrapping SVG trapezoids that shrink.
//! Token fills. Not the stub `chart()` `<figure><figcaption>`.

use crate::cronus_ui_kit::{chart_funnel, funnel_series, label_of};
use crate::parser::ComponentNode;

const FILLS: [&str; 4] = [
    "var(--cronus-primary)",
    "var(--cronus-success)",
    "var(--cronus-warning)",
    "var(--cronus-info)",
];

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let series = funnel_series(comp);
    let mut marks = String::new();
    for (index, stage) in chart_funnel(&series).iter().enumerate() {
        let fill = FILLS[index % FILLS.len()];
        marks.push_str(&format!(
            "<polygon points=\"{}\" fill=\"{fill}\"></polygon>",
            stage.points,
        ));
    }
    format!(
        "<div data-slot=\"funnel-chart\" role=\"img\" aria-label=\"{label}\"><svg viewBox=\"0 0 200 100\" aria-hidden=\"true\">{marks}</svg></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::{stub, DEFAULT_FUNNEL_SERIES};
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
    fn root_is_div_with_svg_trapezoids_not_figure() {
        let html = render(&stub("funnel-chart", "Pipeline"));
        assert!(html.starts_with("<div data-slot=\"funnel-chart\""));
        assert!(html.contains("role=\"img\""));
        assert!(html.contains("aria-label=\"Pipeline\""));
        assert!(html.contains("<svg"));
        assert!(html.contains("<polygon "));
        assert!(html.contains("fill=\"var(--cronus-primary)\""));
        assert!(html.contains("fill=\"var(--cronus-success)\""));
        assert!(html.contains("viewBox=\"0 0 200 100\""));
        assert_eq!(html.matches("<polygon ").count(), 5);
        reject_stub(&html);
    }

    #[test]
    fn default_series_shrinks() {
        let html = render(&stub("funnel-chart", "Pipeline"));
        let stages = chart_funnel(&DEFAULT_FUNNEL_SERIES);
        assert_eq!(stages.len(), 5);
        assert!(stages[0].top_w > stages[4].top_w);
        assert!(stages[0].top_w > stages[0].bot_w);
        assert!(html.contains(&format!("points=\"{}\"", stages[0].points)));
        assert!(html.contains(&format!("points=\"{}\"", stages[4].points)));
        assert_ne!(stages[0].points, STUB_POLYLINE);
        reject_stub(&html);
    }

    #[test]
    fn numeric_items_drive_series() {
        let mut c = stub("funnel-chart", "Pipeline");
        c.items.push(extra("item", "9"));
        c.items.push(extra("item", "6"));
        c.items.push(extra("item", "3"));
        let html = render(&c);
        assert_eq!(html.matches("<polygon ").count(), 3);
        let stages = chart_funnel(&[9.0, 6.0, 3.0]);
        assert!(html.contains(&format!("points=\"{}\"", stages[0].points)));
        let def = chart_funnel(&DEFAULT_FUNNEL_SERIES);
        assert_ne!(stages.len(), def.len());
        assert!(!html.contains(&format!("points=\"{}\"", def[0].points)));
        reject_stub(&html);
    }

    #[test]
    fn comma_list_item_is_series() {
        let mut c = stub("funnel-chart", "Pipeline");
        c.items.push(extra("item", "10, 7, 4"));
        let html = render(&c);
        assert_eq!(html.matches("<polygon ").count(), 3);
        reject_stub(&html);
    }

    #[test]
    fn skips_chart_figure_stub() {
        let html = render(&stub("funnel-chart", "Pipeline"));
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
            let html = render(&stub("funnel-chart", "Pipeline"));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"funnel-chart\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"funnel-chart\"]"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
