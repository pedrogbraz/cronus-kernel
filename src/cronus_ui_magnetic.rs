//! Dedicated Magnetic renderer. DOM matches React idle:
//! `<div data-slot="magnetic">` wrapping `data-slot="magnetic-target"` and
//! the escaped label. CSS hover translate lives in COMPONENT_CHROME. Zero
//! rAF / `data-magnetic-active`. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"magnetic\"><div data-slot=\"magnetic-target\">{}</div></div>",
        label_of(comp)
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
        assert!(!html.contains("@keyframes"));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<span"));
        assert!(!html.contains("<canvas"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("requestAnimationFrame"));
        assert!(!html.contains("data-magnetic-active"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("magnetic", "Magnet"));
        assert_eq!(
            html,
            "<div data-slot=\"magnetic\"><div data-slot=\"magnetic-target\">Magnet</div></div>"
        );
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"magnetic\""));
        assert!(html.contains("data-slot=\"magnetic-target\""));
        assert!(html.contains(">Magnet</div></div>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("magnetic", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"magnetic\"><div data-slot=\"magnetic-target\">A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("magnetic", "Magnet");
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
            dedicated_fn_name("magnetic"),
            Some("cronus_ui_magnetic::render")
        );
        assert_eq!(
            renderer_kind("magnetic"),
            RendererKind::Dedicated("cronus_ui_magnetic::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("magnetic", "Magnet"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"magnetic\""));
            assert!(html.contains("data-slot=\"magnetic-target\""));
        });
    }

    #[test]
    fn chrome_magnetic_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        // Fixture `w-72` on the inline-block wrapper; the target is a block
        // `<div>` in React (no display utility), so it spans the wrapper.
        assert!(css.contains("[data-slot=\"magnetic\"] {\n  display: inline-block;\n  width: var(--cui-magnetic-w, 100%);\n  color: var(--cronus-fg);\n}"));
        assert!(css.contains("[data-slot=\"magnetic-target\"] {\n  display: block;\n"));
        assert!(css.contains("translate(4px"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(css.contains("transform: none"));
        assert!(css.contains("var(--cronus-fg)"));
        assert!(!css.contains("requestAnimationFrame"));
        assert!(!css.contains("data-magnetic-active"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
    }
}
