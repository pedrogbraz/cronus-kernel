//! Dedicated ChoroplethChart renderer.
//!
//! Default (`ChoroplethChart` from `@cronus-ui/ui`, docs "Default"): `<div
//! data-slot="choropleth-chart" role="img" aria-label>` › `<svg viewBox="0 0
//! 200 190">` the seven demo region cells (`item "Northwest" value:42`),
//! `--cronus-primary` fill at 0.15 + value / max × 0.85, `--cronus-border`
//! stroke, a `<title>` each.
//!
//! Motion (`style:choropleth-chart+motion`, `@cronus-ui/ui/charts`): the docs
//! example fetches a world GeoJSON at runtime, which the kernel cannot embed,
//! so the same regions stand in for the countries inside the visx
//! `ChoroplethChart` chrome — `<div data-slot="choropleth-chart"
//! class="v-motion">` (`aspect` 16 / 9), features filled by quintile with
//! `--chart-scale-01…05`, `--cronus-surface-base` 0.5px strokes, an 800ms
//! fade-in, hover dim to 0.4 and the docs zoom buttons (`zoom:true`) with
//! `data-choropleth-zoom="in"|"out"` (disabled only when the author sets
//! `disabled:`).

use crate::cronus_ui_chart::series_names;
use crate::cronus_ui_kit::{choice, esc, flag, fmt_coord, label_of, numeric_items};
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
    let regions = selected_regions(comp);
    let values = region_values(comp);
    if choice(comp, "variant", &["motion"]) == Some("motion") {
        return render_motion(comp, &label, &regions, &values);
    }
    let max = values.iter().copied().fold(1.0_f64, f64::max);
    let mut paths = String::new();
    for (region, value) in regions.iter().zip(values.iter()) {
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

fn selected_regions(comp: &ComponentNode) -> Vec<&'static Region> {
    let names = series_names(comp);
    if names.is_empty() {
        return REGIONS.iter().collect();
    }
    names
        .iter()
        .filter_map(|n| REGIONS.iter().find(|r| r.name.eq_ignore_ascii_case(n)))
        .collect()
}

/// `value:` config per named row, else numeric items in order, else the demo values.
fn region_values(comp: &ComponentNode) -> Vec<f64> {
    let regions = selected_regions(comp);
    let named: Vec<Option<f64>> = regions
        .iter()
        .map(|r| {
            comp.items
                .iter()
                .find(|i| i.text.trim().eq_ignore_ascii_case(r.name))
                .and_then(|i| i.config.get("value"))
                .and_then(|v| v.trim().parse::<f64>().ok())
        })
        .collect();
    if named.iter().any(Option::is_some) {
        return regions
            .iter()
            .zip(named)
            .map(|(r, v)| v.unwrap_or(r.demo))
            .collect();
    }
    let nums = numeric_items(comp);
    regions
        .iter()
        .enumerate()
        .map(|(i, r)| nums.get(i).copied().unwrap_or(r.demo))
        .collect()
}

/// visx `ChoroplethChart` chrome with the demo regions as features.
fn render_motion(
    comp: &ComponentNode,
    label: &str,
    regions: &[&'static Region],
    values: &[f64],
) -> String {
    crate::cronus_ui_chart::note_motion();
    let max = values.iter().copied().fold(1.0_f64, f64::max);
    let mut features = String::new();
    for (region, value) in regions.iter().zip(values) {
        let level = ((value / max).clamp(0.0, 1.0) * 5.0).ceil().max(1.0) as usize;
        features.push_str(&format!(
            "<path class=\"feature\" d=\"{}\" fill=\"var(--chart-scale-0{level})\" stroke=\"var(--cronus-surface-base)\" stroke-width=\"0.5\"><title>{}: {}</title></path>",
            region.d,
            region.name,
            fmt_coord(*value)
        ));
    }
    let zoom = if flag(comp, "zoom") {
        let plus = crate::cronus_ui_icons::svg_or_empty("plus");
        let minus = crate::cronus_ui_icons::svg_or_empty("minus");
        let disabled = flag(comp, "disabled");
        format!(
            "<div class=\"zoom\">{}{}</div>",
            crate::cronus_ui::button_html(
                &plus,
                "secondary",
                "icon",
                None,
                disabled,
                Some("Zoom in")
            )
            .replacen("<button ", "<button data-choropleth-zoom=\"in\" ", 1),
            crate::cronus_ui::button_html(
                &minus,
                "secondary",
                "icon",
                None,
                disabled,
                Some("Zoom out")
            )
            .replacen("<button ", "<button data-choropleth-zoom=\"out\" ", 1)
        )
    } else {
        String::new()
    };
    format!(
        "<div data-slot=\"choropleth-chart\" class=\"v-motion\" role=\"img\" aria-label=\"{}\"><svg viewBox=\"0 0 200 190\" aria-hidden=\"true\"><g class=\"features\">{features}</g></svg>{zoom}</div>",
        esc(label)
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
    fn named_regions_select_cells_like_react_data() {
        let mut c = stub("choropleth-chart", "Regions");
        for t in ["Northwest", "Northeast", "Central", "Southeast"] {
            c.items.push(extra("text", t));
        }
        let html = render(&c);
        assert_eq!(html.matches("<path ").count(), 4);
        assert!(html.contains("<title>Northwest: 42</title>"));
        assert!(html.contains("<title>Central: 95</title>"));
        assert!(html.contains("<title>Southeast: 67</title>"));
        assert!(!html.contains("<title>West:"));
        assert!(!html.contains("M10 70 h 50 v 50 h -50 z"));
        let nw = html.find("Northwest").unwrap();
        let se = html.find("Southeast").unwrap();
        assert!(nw < se);
    }

    #[test]
    fn value_config_drives_named_regions() {
        let mut c = stub("choropleth-chart", "Regions");
        let mut nw = extra("item", "Northwest");
        nw.config.insert("value".into(), "10".into());
        let mut ce = extra("item", "Central");
        ce.config.insert("value".into(), "40".into());
        c.items.push(nw);
        c.items.push(ce);
        let html = render(&c);
        assert_eq!(html.matches("<path ").count(), 2);
        assert!(html.contains("fill-opacity=\"0.36\"><title>Northwest: 10</title>"));
        assert!(html.contains("fill-opacity=\"1\"><title>Central: 40</title>"));
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
    fn motion_variant_uses_scale_fills_and_zoom_buttons() {
        let mut c = stub("choropleth-chart", "World");
        c.style = Some("choropleth-chart+motion".into());
        c.props.insert("zoom".into(), "true".into());
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"choropleth-chart\" class=\"v-motion\" role=\"img\" aria-label=\"World\"><svg viewBox=\"0 0 200 190\" aria-hidden=\"true\"><g class=\"features\"><path class=\"feature\" d=\"M10 10 h 80 v 50 h -80 z\" fill=\"var(--chart-scale-03)\" stroke=\"var(--cronus-surface-base)\" stroke-width=\"0.5\"><title>Northwest: 42</title></path>"));
        assert!(html.contains("fill=\"var(--chart-scale-05)\" stroke=\"var(--cronus-surface-base)\" stroke-width=\"0.5\"><title>Central: 95</title>"));
        assert!(html.contains("<div class=\"zoom\"><button"));
        assert!(html.contains("data-choropleth-zoom=\"in\""));
        assert!(html.contains("data-choropleth-zoom=\"out\""));
        assert!(html.contains("aria-label=\"Zoom in\""));
        assert!(html.contains("aria-label=\"Zoom out\""));
        assert!(!html.contains(" disabled"));
        reject_stub(&html);
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
        assert!(css.contains("[data-slot=\"choropleth-chart\"].v-motion .feature {"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
