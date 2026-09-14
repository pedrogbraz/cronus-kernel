//! Dedicated RetroGrid renderer. DOM mirrors React without the injected
//! `<style>`: `<div data-slot="retro-grid">` + aria-hidden mask `<div>` >
//! tilted floor `<div>` > scrolling grid `<div>`, then a relative content
//! `<div>` wrapping the label. Only the root carries a `data-slot`. Fixture
//! `w-72 min-h-32` is mirrored in COMPONENT_CHROME. Perspective floor +
//! `@keyframes cui-retro-grid` live in COMPONENT_CHROME. Zero JS, no inline
//! style. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"retro-grid\"><div aria-hidden=\"true\"><div><div></div></div></div><div>{}</div></div>",
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
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("retro-grid", "Grid"));
        assert_eq!(
            html,
            "<div data-slot=\"retro-grid\"><div aria-hidden=\"true\"><div><div></div></div></div><div>Grid</div></div>"
        );
        assert_eq!(html.matches("data-slot=").count(), 1);
        assert!(!html.contains("retro-grid-field"));
        assert!(!html.contains("retro-grid-content"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("retro-grid", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"retro-grid\"><div aria-hidden=\"true\"><div><div></div></div></div><div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("retro-grid", "Grid");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
        assert_eq!(
            dedicated_fn_name("retro-grid"),
            Some("cronus_ui_retro_grid::render")
        );
        assert_eq!(
            renderer_kind("retro-grid"),
            RendererKind::Dedicated("cronus_ui_retro_grid::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("retro-grid", "Grid"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"retro-grid\""));
        });
    }

    #[test]
    fn chrome_retro_grid_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"retro-grid\"] {\n  position: relative; overflow: hidden;\n  width: var(--cui-retro-grid-w, 100%); min-height: 8rem;"));
        assert!(css.contains("[data-slot=\"retro-grid\"] > [aria-hidden=\"true\"] > div > div {"));
        assert!(
            css.contains("[data-slot=\"retro-grid\"] > div:last-child {\n  position: relative;\n}")
        );
        assert!(!css.contains("[data-slot=\"retro-grid-field\"]"));
        assert!(css.contains("perspective: 240px"));
        assert!(css.contains("rotateX(60deg)"));
        assert!(css.contains("@keyframes cui-retro-grid"));
        assert!(css.contains("animation: cui-retro-grid"));
        assert!(css.contains("translateY(48px)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
    }
}
