//! Dedicated RetroGrid renderer. DOM matches React idle without `<style>`:
//! `<div data-slot="retro-grid">` + `aria-hidden` field + relative content
//! wrapping the label. Perspective floor + `@keyframes cui-retro-grid` live in
//! COMPONENT_CHROME. Zero JS, no inline style. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"retro-grid\"><div data-slot=\"retro-grid-field\" aria-hidden=\"true\"></div><div data-slot=\"retro-grid-content\">{}</div></div>",
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
            "<div data-slot=\"retro-grid\"><div data-slot=\"retro-grid-field\" aria-hidden=\"true\"></div><div data-slot=\"retro-grid-content\">Grid</div></div>"
        );
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"retro-grid\""));
        assert!(html.contains("data-slot=\"retro-grid-field\""));
        assert!(html.contains("data-slot=\"retro-grid-content\""));
        assert!(html.contains("aria-hidden=\"true\""));
        assert!(html.contains(">Grid</div></div>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("retro-grid", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"retro-grid\"><div data-slot=\"retro-grid-field\" aria-hidden=\"true\"></div><div data-slot=\"retro-grid-content\">A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
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
        assert!(fx.contains("<span>"));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        assert!(!html.contains("<span"));
        reject_fx(&html);
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
            assert!(html.contains("data-slot=\"retro-grid-field\""));
            assert!(html.contains("data-slot=\"retro-grid-content\""));
        });
    }

    #[test]
    fn chrome_retro_grid_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"retro-grid\"]"));
        assert!(css.contains("[data-slot=\"retro-grid-field\"]"));
        assert!(css.contains("[data-slot=\"retro-grid-content\"]"));
        assert!(css.contains("perspective"));
        assert!(css.contains("rotateX"));
        assert!(css.contains("@keyframes cui-retro-grid"));
        assert!(css.contains("animation: cui-retro-grid"));
        assert!(css.contains("translateY"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(!css.contains("<canvas"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
    }
}
