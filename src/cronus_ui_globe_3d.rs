//! Dedicated Globe3D renderer. React mounts a three.js `<Canvas>` (textured
//! Blue Marble sphere, bump map, atmosphere, OrbitControls auto-rotate) with
//! `<Html>` avatar pins. Zero JS and no `<canvas>`: the kernel renders the
//! same object as a shaded sphere that the browser rotates —
//! `<div data-slot="globe-3d" role="img" aria-label>` (`globe3dVariants`:
//! 500px tall, full width) > `.globe` (a 69% disc, the globe's share of the
//! 45° camera frame) holding the shaded `.disc` (dark ocean tint with a
//! light-from-the-top-left gradient and the atmosphere glow), a faint
//! graticule reused from globe-wireframe (its rings spin with `rotate: y`
//! over 200s, OrbitControls' `autoRotateSpeed` 0.3) and the markers.
//!
//! A marker is nested coarse + fine rotations (`ry-280 > fy-6` for 286° of
//! longitude, `rz-n40 > fz-n1` for 41° north) that carry a `.pin` to the
//! exact surface point: the stem (`.stem`, 7.5% of the radius like React's
//! 0.15 / 2), the red tip (`.tip`, the cone) and the avatar (`.face > img`)
//! kept facing the camera by undoing the same rotations plus the spin — the
//! sprite behaviour of React's `<Html sprite>`. Back-side pins fade through
//! the shared phase-shifted opacity loop. Textures are the one thing missing.
//!
//! Inputs: `item "New York" lat:40.7 lng:-74 -> "https://…avatar.webp"`
//! markers (item text = label / alt; `src:` config also works),
//! `auto-rotate:false` freezes the spin, `label "Globe"` names it.

use crate::cronus_ui_kit::{esc, item, safe_url};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    // The graticule / marker machinery lives in globe-wireframe.css.
    crate::cronus_ui_css::note_family("globe-wireframe");
    let name = item(comp, "label")
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| "Globe".to_string());
    let frozen = if crate::cronus_ui_globe_wireframe::is_static(comp) {
        " data-static=\"\""
    } else {
        ""
    };
    let markers: String = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty())
        .filter_map(|i| {
            let lat: f64 = i.config.get("lat")?.trim().parse().ok()?;
            let lng: f64 = i
                .config
                .get("lng")
                .or_else(|| i.config.get("lon"))?
                .trim()
                .parse()
                .ok()?;
            let src = i
                .link
                .as_deref()
                .or_else(|| i.config.get("src").map(String::as_str))
                .or_else(|| i.config.get("image").map(String::as_str))
                .unwrap_or("");
            Some(marker(lat, lng, &safe_url(src), &esc(&i.text)))
        })
        .collect();
    let grid = crate::cronus_ui_globe_wireframe::graticule(false);
    format!(
        "<div data-slot=\"globe-3d\" role=\"img\" aria-label=\"{name}\"{frozen}><div class=\"globe\" aria-hidden=\"true\"><div class=\"disc\"></div>{grid}{markers}</div><div class=\"rim\"></div></div></div>"
    )
}

/// `(coarse, fine)` class suffixes for an angle: tens and units, clamped to
/// the class set the CSS defines (`ry-0…350` + `fy-0…9`, `rz-n90…90` + `fz`).
fn split(deg: i32) -> (i32, i32) {
    let coarse = (deg / 10) * 10;
    (coarse, deg - coarse)
}

fn signed(n: i32) -> String {
    if n < 0 {
        format!("n{}", -n)
    } else {
        n.to_string()
    }
}

fn marker(lat: f64, lng: f64, src: &str, label: &str) -> String {
    let lat = lat.round().clamp(-90.0, 90.0) as i32;
    let lng = ((lng.round() as i32) % 360 + 360) % 360;
    let (yc, yf) = split(lng);
    let (zc, zf) = split(-lat);
    // Undo: rotateZ(+lat) then rotateY(360 - lng).
    let (uzc, uzf) = split(lat);
    let (uyc, uyf) = split((360 - lng) % 360);
    let img = if src.is_empty() {
        String::new()
    } else {
        format!("<img src=\"{src}\" alt=\"{label}\">")
    };
    // CSS rotateY sends +x away from the viewer, so 0…180 start on the back.
    let back = if lng <= 180 { " back" } else { "" };
    format!(
        "<div class=\"marker{back} ry-{yc}\" title=\"{label}\"><div class=\"fy-{yf}\"><div class=\"rz-{}\"><div class=\"fz-{}\"><div class=\"pin\"><i class=\"stem\"></i><i class=\"tip\"></i><span class=\"bb\"><span class=\"rz-{}\"><span class=\"fz-{}\"><span class=\"ry-{uyc}\"><span class=\"fy-{uyf}\"><span class=\"face\">{img}</span></span></span></span></span></span></div></div></div></div></div>",
        signed(zc),
        signed(zf),
        signed(uzc),
        signed(uzf)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;
    use std::collections::HashMap;

    fn globe() -> ComponentNode {
        stub("globe-3d", "Globe")
    }

    fn pin(c: &mut ComponentNode, label: &str, lat: &str, lng: &str, src: &str) {
        let mut config = HashMap::new();
        config.insert("lat".into(), lat.into());
        config.insert("lng".into(), lng.into());
        c.items.push(ComponentItemNode {
            item_type: "item".into(),
            text: label.into(),
            link: Some(src.into()),
            tone: None,
            config,
        });
    }

    fn reject_js(html: &str) {
        for bad in ["<script", "style=", "onclick", "onmouse", "<canvas", "<svg"] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    #[test]
    fn docs_team_globe_dom() {
        let mut c = globe();
        pin(
            &mut c,
            "New York",
            "40.7128",
            "-74.006",
            "https://assets.aceternity.com/avatars/1.webp",
        );
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"globe-3d\" role=\"img\" aria-label=\"Globe\"><div class=\"globe\" aria-hidden=\"true\"><div class=\"disc\"></div><div class=\"lat\"><i></i>"));
        assert_eq!(html.matches("<div class=\"m\">").count(), 18);
        // -74.006° → 286° = ry-280 + fy-6; 40.7° N → rotateZ(-41) = rz-n40 + fz-n1;
        // undone as rz-40 + fz-1 and rotateY(74) = ry-70 + fy-4.
        assert!(html.contains("<div class=\"marker ry-280\" title=\"New York\"><div class=\"fy-6\"><div class=\"rz-n40\"><div class=\"fz-n1\"><div class=\"pin\"><i class=\"stem\"></i><i class=\"tip\"></i><span class=\"bb\"><span class=\"rz-40\"><span class=\"fz-1\"><span class=\"ry-70\"><span class=\"fy-4\"><span class=\"face\"><img src=\"https://assets.aceternity.com/avatars/1.webp\" alt=\"New York\"></span></span></span></span></span></span></div></div></div></div></div>"));
        assert!(html.ends_with("</div><div class=\"rim\"></div></div></div>"));
        reject_js(&html);
    }

    #[test]
    fn southern_and_eastern_markers() {
        // Sydney: -33.8688, 151.2093 → ry-150 fy-1, rotateZ(+34) = rz-30 fz-4.
        let html = marker(-33.8688, 151.2093, "/a.webp", "Sydney");
        assert!(html.starts_with("<div class=\"marker back ry-150\" title=\"Sydney\"><div class=\"fy-1\"><div class=\"rz-30\"><div class=\"fz-4\">"));
        assert!(html.contains("<span class=\"rz-n30\"><span class=\"fz-n4\"><span class=\"ry-200\"><span class=\"fy-9\">"));
        // Equator / prime meridian: every class is the zero one.
        let html = marker(0.0, 0.0, "", "Null Island");
        assert!(html.contains("<div class=\"marker back ry-0\" title=\"Null Island\"><div class=\"fy-0\"><div class=\"rz-0\"><div class=\"fz-0\">"));
        assert!(
            html.contains("<span class=\"ry-0\"><span class=\"fy-0\"><span class=\"face\"></span>")
        );
        assert!(!html.contains("<img"));
    }

    #[test]
    fn markers_need_coordinates_and_escape() {
        let mut c = globe();
        c.items.push(ComponentItemNode {
            item_type: "item".into(),
            text: "Nowhere".into(),
            link: Some("/x.png".into()),
            tone: None,
            config: HashMap::new(),
        });
        assert!(!render(&c).contains("marker"));
        let mut c = globe();
        pin(&mut c, "<b>x</b> \"q\"", "10", "20", "javascript:alert(1)");
        let html = render(&c);
        assert!(html.contains("title=\"&lt;b&gt;x&lt;/b&gt; &quot;q&quot;\""));
        assert!(html.contains("<img src=\"#\" alt=\"&lt;b&gt;x&lt;/b&gt; &quot;q&quot;\">"));
        reject_js(&html);
    }

    #[test]
    fn frozen_and_named() {
        let mut c = globe();
        c.props.insert("auto-rotate".into(), "false".into());
        c.items[0].text = "Team".into();
        let html = render(&c);
        assert!(html.starts_with(
            "<div data-slot=\"globe-3d\" role=\"img\" aria-label=\"Team\" data-static=\"\">"
        ));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/globe-3d.css");
        assert!(css.contains("[data-slot=\"globe-3d\"] {"));
        assert!(css.contains("height: 31.25rem"));
        assert!(css.contains("--cui-globe-period: 200s"));
        assert!(css.contains(
            "[data-slot=\"globe-3d\"] > .globe { inset: auto; width: auto; height: 69%;"
        ));
        assert!(css.contains("var(--cronus-info)"));
        assert!(!css.contains("#"));
        let shared = include_str!("cronus_ui_css/globe-wireframe.css");
        assert!(shared.contains(".pin > .stem"));
        assert!(shared.contains(".ry-350 { --cui-ry: 350; }"));
        assert!(shared.contains(".rz-n90 { --cui-rz: -90; }"));
        assert!(shared.contains(".fz-n9 { --cui-fz: -9; }"));
        assert!(shared.contains("@keyframes cui-globe-unspin"));
    }

    #[test]
    fn registered_as_dedicated() {
        let c = globe();
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), render(&c));
        assert_eq!(
            dedicated_fn_name("globe-3d"),
            Some("cronus_ui_globe_3d::render")
        );
        assert_eq!(
            renderer_kind("globe-3d"),
            RendererKind::Dedicated("cronus_ui_globe_3d::render")
        );
    }
}
