//! Dedicated MotionPresets renderer. CSS-only demo of named token presets
//! (`fadeIn`, `fadeInUp`, `scaleIn` from motion-presets.ts). DOM:
//! `<div data-slot="motion-presets">` with three `motion-preset` rows.
//! Keyframes live in COMPONENT_CHROME. Zero JS / framer. Not the catalog
//! `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"motion-presets\"><div data-slot=\"motion-preset\" data-preset=\"fade-in\">{label}</div><div data-slot=\"motion-preset\" data-preset=\"fade-in-up\">fade-in-up</div><div data-slot=\"motion-preset\" data-preset=\"scale-in\">scale-in</div></div>",
        label = label_of(comp)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("<style"));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<span"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
        assert!(!html.contains("framer"));
    }

    #[test]
    fn root_has_three_preset_rows_not_fx_title_box() {
        let html = render(&stub("motion-presets", "Motion"));
        assert_eq!(
            html,
            "<div data-slot=\"motion-presets\"><div data-slot=\"motion-preset\" data-preset=\"fade-in\">Motion</div><div data-slot=\"motion-preset\" data-preset=\"fade-in-up\">fade-in-up</div><div data-slot=\"motion-preset\" data-preset=\"scale-in\">scale-in</div></div>"
        );
        assert!(html.starts_with("<div data-slot=\"motion-presets\">"));
        assert_eq!(html.matches("data-slot=\"motion-preset\"").count(), 3);
        assert!(html.contains("data-preset=\"fade-in\""));
        assert!(html.contains("data-preset=\"fade-in-up\""));
        assert!(html.contains("data-preset=\"scale-in\""));
        assert!(html.contains(">Motion</div>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("motion-presets", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"motion-presets\"><div data-slot=\"motion-preset\" data-preset=\"fade-in\">A &lt;B&gt; &amp; &quot;C&quot;</div><div data-slot=\"motion-preset\" data-preset=\"fade-in-up\">fade-in-up</div><div data-slot=\"motion-preset\" data-preset=\"scale-in\">scale-in</div></div>"
        );
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("motion-presets", "Motion");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.contains("<span>"));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        assert!(!html.contains("<span"));
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("motion-presets"),
            Some("cronus_ui_motion_presets::render")
        );
        assert_eq!(
            renderer_kind("motion-presets"),
            RendererKind::Dedicated("cronus_ui_motion_presets::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("motion-presets", "Motion"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"motion-presets\""));
            assert_eq!(html.matches("data-slot=\"motion-preset\"").count(), 3);
        });
    }

    #[test]
    fn chrome_motion_presets_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"motion-presets\"]"));
        assert!(css.contains("[data-slot=\"motion-preset\"]"));
        assert!(css.contains("@keyframes cui-fade-in"));
        assert!(css.contains("@keyframes cui-fade-in-up"));
        assert!(css.contains("@keyframes cui-scale-in"));
        assert!(css.contains("cubic-bezier(0.16, 1, 0.3, 1)"));
        assert!(css.contains("translateY(16px)"));
        assert!(css.contains("scale(0.96)"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
        assert!(!css.contains("style="));
        assert!(!css.contains("framer"));
    }
}
