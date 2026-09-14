//! Dedicated Noise renderer. DOM mirrors React:
//! `<div data-slot="noise">` + aria-hidden inline `<svg>` with an
//! `feTurbulence` grain filter painted on a full-size `<rect>` + a relative
//! content `<div>` wrapping the label. The filter id is derived from the
//! component name (React uses `useId`). Overlay opacity 0.08, sizing
//! (fixture `h-32 w-72`) and positioning live in COMPONENT_CHROME — no
//! inline style. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::{label_of, widget_id};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let id = widget_id(comp, "grain");
    format!(
        "<div data-slot=\"noise\"><svg aria-hidden=\"true\"><filter id=\"{id}\"><feTurbulence type=\"fractalNoise\" baseFrequency=\"0.8\" numOctaves=\"4\" stitchTiles=\"stitch\"></feTurbulence></filter><rect width=\"100%\" height=\"100%\" filter=\"url(#{id})\"></rect></svg><div>{}</div></div>",
        label_of(comp)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<span"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_has_svg_grain_and_content_div() {
        let html = render(&stub("noise", "Grain"));
        assert_eq!(
            html,
            "<div data-slot=\"noise\"><svg aria-hidden=\"true\"><filter id=\"cui-noise-grain\"><feTurbulence type=\"fractalNoise\" baseFrequency=\"0.8\" numOctaves=\"4\" stitchTiles=\"stitch\"></feTurbulence></filter><rect width=\"100%\" height=\"100%\" filter=\"url(#cui-noise-grain)\"></rect></svg><div>Grain</div></div>"
        );
        assert_eq!(html.matches("data-slot=").count(), 1);
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("noise", "A <B> & \"C\""));
        assert!(html.ends_with("<div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("noise", "Grain"));
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert_ne!(html, fx);
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("noise", "Grain"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"noise\""));
        });
    }

    #[test]
    fn chrome_noise_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"noise\"] {\n  position: relative; overflow: hidden;\n  width: 18rem; height: 8rem;\n}"));
        assert!(css.contains("[data-slot=\"noise\"] > svg {\n  position: absolute; inset: 0;\n  width: 100%; height: 100%;\n  pointer-events: none; opacity: 0.08;\n}"));
        assert!(css.contains("[data-slot=\"noise\"] > div {\n  position: relative;\n}"));
        assert!(!css.contains("[data-slot=\"noise\"]::after"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
