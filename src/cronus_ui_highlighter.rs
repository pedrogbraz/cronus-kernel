//! Dedicated Highlighter renderer. DOM matches React idle:
//! `<span data-slot="highlighter">` plus `highlighter-mark` (aria-hidden)
//! and the label. CSS scaleX draw-in lives in COMPONENT_CHROME. Zero JS,
//! no inline style, no `<style>` tag. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<span data-slot=\"highlighter\"><span data-slot=\"highlighter-mark\" aria-hidden=\"true\"></span>{}</span>",
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
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<div"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setTimeout"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_is_span_with_mark_not_fx_title_box() {
        let html = render(&stub("highlighter", "Important"));
        assert_eq!(
            html,
            "<span data-slot=\"highlighter\"><span data-slot=\"highlighter-mark\" aria-hidden=\"true\"></span>Important</span>"
        );
        assert!(html.starts_with("<span "));
        assert!(html.contains("data-slot=\"highlighter\""));
        assert!(html.contains("data-slot=\"highlighter-mark\""));
        assert!(html.contains("aria-hidden=\"true\""));
        assert!(html.contains(">Important</span>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("highlighter", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<span data-slot=\"highlighter\"><span data-slot=\"highlighter-mark\" aria-hidden=\"true\"></span>A &lt;B&gt; &amp; &quot;C&quot;</span>"
        );
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("highlighter", "Important");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.contains("<span>"));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        assert!(!html.contains("<div"));
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("highlighter"),
            Some("cronus_ui_highlighter::render")
        );
        assert_eq!(
            renderer_kind("highlighter"),
            RendererKind::Dedicated("cronus_ui_highlighter::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("highlighter", "Important"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"highlighter\""));
            assert!(html.contains("data-slot=\"highlighter-mark\""));
        });
    }

    #[test]
    fn chrome_highlighter_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"highlighter\"]"));
        assert!(css.contains("[data-slot=\"highlighter-mark\"]"));
        assert!(css.contains("@keyframes cui-highlighter"));
        assert!(css.contains("animation: cui-highlighter"));
        assert!(css.contains("scaleX(0)"));
        assert!(css.contains("scaleX(1)"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(css.contains("color-mix(in oklch, var(--cronus-primary) 25%"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
    }
}
