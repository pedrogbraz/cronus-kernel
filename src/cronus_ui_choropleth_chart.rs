//! Dedicated ChoroplethChart renderer. DOM matches React:
//! `<div data-slot="choropleth-chart">` wrapping SVG filled regions.
//! Token fills. Not the catalog `chart()` stub (`<figure><figcaption>`).

use crate::cronus_ui_kit::{fmt_coord, label_of, numeric_items};
use crate::parser::ComponentNode;

struct Region {
    name: &'static str,
    d: &'static str,
    demo: f64,
}

const REGIONS: [Region; 7] = [
    Region {
        name: "Northwest",
        d: "M10 10 h 80 v 50 h -80 z",
        demo: 42.0,
    },
    Region {
        name: "Northeast",
        d: "M100 10 h 80 v 50 h -80 z",
        demo: 78.0,
    },
    Region {
        name: "West",
        d: "M10 70 h 50 v 50 h -50 z",
        demo: 31.0,
    },
    Region {
        name: "Central",
        d: "M70 70 h 50 v 50 h -50 z",
        demo: 95.0,
    },
    Region {
        name: "East",
        d: "M130 70 h 50 v 50 h -50 z",
        demo: 58.0,
    },
    Region {
        name: "Southwest",
        d: "M10 130 h 80 v 50 h -80 z",
        demo: 22.0,
    },
    Region {
        name: "Southeast",
        d: "M100 130 h 80 v 50 h -80 z",
        demo: 67.0,
    },
];

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let values = region_values(comp);
    let max = values.iter().copied().fold(1.0_f64, f64::max);
    let mut paths = String::new();
    for (region, value) in REGIONS.iter().zip(values.iter()) {
        let opacity = 0.15 + (*value / max).clamp(0.0, 1.0) * 0.85;
        paths.push_str(&format!(
            "<path d=\"{d}\" fill=\"var(--cronus-primary)\" stroke=\"var(--cronus-border)\" fill-opacity=\"{op}\"><title>{name}: {v}</title></path>",
            d = region.d,
            op = fmt_coord(opacity),
            name = region.name,
            v = fmt_coord(*value),
        ));
    }
    format!(
        "<div data-slot=\"choropleth-chart\" role=\"img\" aria-label=\"{label}\"><svg viewBox=\"0 0 200 190\" aria-hidden=\"true\">{paths}</svg></div>"
    )
}

fn region_values(comp: &ComponentNode) -> Vec<f64> {
    let nums = numeric_items(comp);
    REGIONS
        .iter()
        .enumerate()
        .map(|(i, r)| nums.get(i).copied().unwrap_or(r.demo))
        .collect()
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
    fn root_is_div_with_svg_regions_not_figure() {
        let html = render(&stub("choropleth-chart", "Map"));
        assert!(html.starts_with("<div data-slot=\"choropleth-chart\""));
        assert!(html.contains("role=\"img\""));
        assert!(html.contains("aria-label=\"Map\""));
        assert!(html.contains("<svg"));
        assert!(html.contains("<path "));
        assert!(html.contains("fill=\"var(--cronus-primary)\""));
        assert!(html.contains("stroke=\"var(--cronus-border)\""));
        assert!(html.contains("viewBox=\"0 0 200 190\""));
        assert_eq!(html.matches("<path ").count(), 7);
        assert!(html.contains("M10 10 h 80 v 50 h -80 z"));
        assert!(html.contains("<title>Northwest: 42</title>"));
        assert!(html.contains("<title>Central: 95</title>"));
        reject_stub(&html);
    }

    #[test]
    fn default_regions_when_only_label() {
        let html = render(&stub("choropleth-chart", "Map"));
        let values = region_values(&stub("choropleth-chart", "Map"));
        assert_eq!(values, vec![42.0, 78.0, 31.0, 95.0, 58.0, 22.0, 67.0]);
        let max = 95.0;
        let op = fmt_coord(0.15 + (42.0 / max) * 0.85);
        assert!(html.contains(&format!("fill-opacity=\"{op}\"")));
        reject_stub(&html);
    }

    #[test]
    fn numeric_items_drive_values() {
        let mut c = stub("choropleth-chart", "Map");
        c.items.push(extra("item", "10"));
        c.items.push(extra("item", "20"));
        c.items.push(extra("item", "30"));
        let html = render(&c);
        assert_eq!(html.matches("<path ").count(), 7);
        assert!(html.contains("<title>Northwest: 10</title>"));
        assert!(html.contains("<title>Northeast: 20</title>"));
        assert!(html.contains("<title>West: 30</title>"));
        assert!(html.contains("<title>Central: 95</title>"));
        reject_stub(&html);
    }

    #[test]
    fn comma_list_item_is_series() {
        let mut c = stub("choropleth-chart", "Map");
        c.items.push(extra("item", "4, 8, 6"));
        let html = render(&c);
        assert!(html.contains("<title>Northwest: 4</title>"));
        assert!(html.contains("<title>Northeast: 8</title>"));
        assert!(html.contains("<title>West: 6</title>"));
        reject_stub(&html);
    }

    #[test]
    fn skips_chart_figure_stub() {
        let html = render(&stub("choropleth-chart", "Map"));
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
            let html = render(&stub("choropleth-chart", "Map"));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"choropleth-chart\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"choropleth-chart\"]"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
