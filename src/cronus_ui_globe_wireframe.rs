//! Dedicated GlobeWireframe renderer. React projects Natural Earth country
//! outlines through d3 `geoOrthographic` into an `<svg>`, re-rendered every
//! frame while it auto-rotates or is dragged. The kernel cannot fetch the
//! atlas nor run a projection per frame, so it renders the same object as a
//! static graticule globe that the browser rotates:
//!
//! `<div data-slot="globe-wireframe" data-variant role="img" aria-labelledby>`
//! (`globeWireframeVariants`: square, full width, `text-fg`) > sr-only name
//! + `<div class="globe">` holding a `.disc` (solid fill), the static latitude
//! lines (`.lat > i`, 10° steps, straight segments in the equator-on view),
//! the spinning group (`.spin`, `rotate: y` keyframes, 13.3s per turn = the
//! docs' 0.45°/frame) of 18 meridian rings (`.m`, one per 10° of longitude,
//! each split into two halves whose opacity dims while they face away) and
//! the sphere outline (`.rim`). Everything is HTML 3D transforms without
//! `perspective`, which is an orthographic projection — the same camera as
//! d3's. Variants: `wireframe` (graticule at 45% opacity, 1px outline),
//! `wireframesolid` (full-opacity strokes, 1.5px outline at 80%) and `solid`
//! (30% fill, faint strokes, tangent surface dots with back-face culling).
//! Country strokes are the one thing missing (no atlas offline).
//!
//! Inputs: `variant:wireframe|wireframesolid|solid` (prop or style segment),
//! `auto-rotate:false` freezes the spin (`data-static`), `label "Globe"` names
//! it. `autoRotateSpeed` / `strokeWidth` are fixed to the docs' values in CSS.

use crate::cronus_ui_kit::{attr, choice, esc, item, truthy};
use crate::parser::ComponentNode;

const VARIANTS: &[&str] = &["wireframe", "wireframesolid", "solid"];

/// Latitude lines + spinning meridians (with surface dots when `dots`).
/// Shared with globe-3d, whose renderer notes this family's CSS.
pub(crate) fn graticule(dots: bool) -> String {
    let lat: String = (0..17).map(|_| "<i></i>").collect();
    let dot_row: String = if dots {
        (0..14).map(|_| "<i class=\"d\"></i>").collect()
    } else {
        String::new()
    };
    let meridians: String = (0..18)
        .map(|_| format!("<div class=\"m\"><i class=\"a\"></i><i class=\"b\"></i>{dot_row}</div>"))
        .collect();
    format!("<div class=\"lat\">{lat}</div><div class=\"spin\">{meridians}")
}

pub(crate) fn is_static(comp: &ComponentNode) -> bool {
    attr(comp, "auto-rotate")
        .or_else(|| attr(comp, "autoRotate"))
        .or_else(|| attr(comp, "rotate"))
        .is_some_and(|v| !truthy(v))
}

pub fn render(comp: &ComponentNode) -> String {
    let variant = choice(comp, "variant", VARIANTS).unwrap_or("wireframe");
    let name = item(comp, "label")
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| "Globe".to_string());
    let id = crate::cronus_ui_kit::instance_id(comp, "globe-wireframe");
    let frozen = if is_static(comp) {
        " data-static=\"\""
    } else {
        ""
    };
    let grid = graticule(variant == "solid");
    format!(
        "<div data-slot=\"globe-wireframe\" data-variant=\"{variant}\" role=\"img\" aria-labelledby=\"{id}\"{frozen}><span id=\"{id}\" class=\"sr-only\">{name}</span><div class=\"globe\" aria-hidden=\"true\"><div class=\"disc\"></div>{grid}</div><div class=\"rim\"></div></div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::{reset_instance_ids, stub};

    fn globe() -> ComponentNode {
        reset_instance_ids();
        stub("globe-wireframe", "Globe")
    }

    fn reject_js(html: &str) {
        for bad in ["<script", "style=", "onclick", "onmouse", "<canvas", "<svg"] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    #[test]
    fn docs_wireframesolid_dom() {
        let mut c = globe();
        c.style = Some("globe-wireframe+wireframesolid".into());
        c.props.insert("auto-rotate".into(), "true".into());
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"globe-wireframe\" data-variant=\"wireframesolid\" role=\"img\" aria-labelledby=\"cui-globe-wireframe-globe-wireframe\"><span id=\"cui-globe-wireframe-globe-wireframe\" class=\"sr-only\">Globe</span><div class=\"globe\" aria-hidden=\"true\"><div class=\"disc\"></div><div class=\"lat\"><i></i>"));
        assert_eq!(html.matches("<i></i>").count(), 17);
        assert_eq!(
            html.matches("<div class=\"m\"><i class=\"a\"></i><i class=\"b\"></i></div>")
                .count(),
            18
        );
        assert!(html.ends_with("</div><div class=\"rim\"></div></div></div>"));
        assert!(!html.contains("data-static"));
        assert!(!html.contains("class=\"d\""));
        reject_js(&html);
    }

    #[test]
    fn every_variant_and_solid_dots() {
        for v in VARIANTS {
            let mut c = globe();
            c.props.insert("variant".into(), (*v).into());
            let html = render(&c);
            assert!(html.contains(&format!("data-variant=\"{v}\"")));
            assert_eq!(html.contains("<i class=\"d\"></i>"), *v == "solid", "{v}");
        }
        assert!(render(&globe()).contains("data-variant=\"wireframe\""));
        let mut c = globe();
        c.props.insert("variant".into(), "solid".into());
        assert_eq!(render(&c).matches("<i class=\"d\"></i>").count(), 18 * 14);
    }

    #[test]
    fn auto_rotate_false_freezes() {
        let mut c = globe();
        c.props.insert("auto-rotate".into(), "false".into());
        assert!(render(&c)
            .contains("aria-labelledby=\"cui-globe-wireframe-globe-wireframe\" data-static=\"\">"));
    }

    #[test]
    fn escapes_the_name() {
        let mut c = globe();
        c.items[0].text = "<b>Earth</b> \"q\"".into();
        let html = render(&c);
        assert!(html.contains("class=\"sr-only\">&lt;b&gt;Earth&lt;/b&gt; &quot;q&quot;</span>"));
        reject_js(&html);
    }

    #[test]
    fn chrome_is_token_only_and_geometry_is_exact() {
        let css = include_str!("cronus_ui_css/globe-wireframe.css");
        assert!(css.contains("--cui-globe-period: 13.3s"));
        assert!(css.contains(
            ".spin { animation: cui-globe-spin var(--cui-globe-period) linear infinite; }"
        ));
        assert!(css.contains(
            "@keyframes cui-globe-spin { from { rotate: y 0deg; } to { rotate: y 360deg; } }"
        ));
        assert!(css.contains(".m:nth-child(18) { --cui-ry: 170; }"));
        // Equator line spans the full diameter; 60° line is half as wide.
        assert!(css.contains(
            ".lat > i:nth-child(9) { top: 50.00%; width: 100.00%; inset-inline-start: 0.00%; }"
        ));
        assert!(css.contains(
            ".lat > i:nth-child(15) { top: 6.70%; width: 50.00%; inset-inline-start: 25.00%; }"
        ));
        assert!(css.contains("[data-variant=\"solid\"] > .globe > .disc"));
        assert!(css.contains("backface-visibility: hidden"));
        assert!(css.contains("[data-static] .spin { animation: none; }"));
        assert!(!css.contains("perspective:"));
        assert!(!css.contains("#"));
    }

    #[test]
    fn registered_as_dedicated() {
        let c = globe();
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), {
            reset_instance_ids();
            render(&c)
        });
        assert_eq!(
            dedicated_fn_name("globe-wireframe"),
            Some("cronus_ui_globe_wireframe::render")
        );
        assert_eq!(
            renderer_kind("globe-wireframe"),
            RendererKind::Dedicated("cronus_ui_globe_wireframe::render")
        );
    }
}
