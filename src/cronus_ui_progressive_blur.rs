//! Dedicated ProgressiveBlur renderer. DOM is host + overlay + content:
//! `<div data-slot="progressive-blur-host">` wrapping an aria-hidden
//! `progressive-blur` overlay of three `progressive-blur-layer`s and
//! `progressive-blur-content` with the label. Backdrop-filter masks live in
//! COMPONENT_CHROME — never inline style. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

const LAYER: &str = "<div data-slot=\"progressive-blur-layer\"></div>";

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"progressive-blur-host\"><div data-slot=\"progressive-blur\" aria-hidden=\"true\">{}{}{}</div><div data-slot=\"progressive-blur-content\">{}</div></div>",
        LAYER,
        LAYER,
        LAYER,
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
        let html = render(&stub("progressive-blur", "Blur"));
        assert_eq!(
            html,
            "<div data-slot=\"progressive-blur-host\"><div data-slot=\"progressive-blur\" aria-hidden=\"true\"><div data-slot=\"progressive-blur-layer\"></div><div data-slot=\"progressive-blur-layer\"></div><div data-slot=\"progressive-blur-layer\"></div></div><div data-slot=\"progressive-blur-content\">Blur</div></div>"
        );
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"progressive-blur-host\""));
        assert!(html.contains("data-slot=\"progressive-blur\""));
        assert!(html.contains("data-slot=\"progressive-blur-layer\""));
        assert!(html.contains("data-slot=\"progressive-blur-content\""));
        assert!(html.contains("aria-hidden=\"true\""));
        assert_eq!(html.matches("data-slot=\"progressive-blur-layer\"").count(), 3);
        assert!(html.contains(">Blur</div></div>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("progressive-blur", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"progressive-blur-host\"><div data-slot=\"progressive-blur\" aria-hidden=\"true\"><div data-slot=\"progressive-blur-layer\"></div><div data-slot=\"progressive-blur-layer\"></div><div data-slot=\"progressive-blur-layer\"></div></div><div data-slot=\"progressive-blur-content\">A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("progressive-blur", "Blur");
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
            dedicated_fn_name("progressive-blur"),
            Some("cronus_ui_progressive_blur::render")
        );
        assert_eq!(
            renderer_kind("progressive-blur"),
            RendererKind::Dedicated("cronus_ui_progressive_blur::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("progressive-blur", "Blur"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"progressive-blur-host\""));
            assert!(html.contains("data-slot=\"progressive-blur\""));
            assert!(html.contains("data-slot=\"progressive-blur-layer\""));
            assert!(html.contains("data-slot=\"progressive-blur-content\""));
            assert_eq!(html.matches("data-slot=\"progressive-blur-layer\"").count(), 3);
        });
    }

    #[test]
    fn chrome_progressive_blur_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"progressive-blur-host\"]"));
        assert!(css.contains("[data-slot=\"progressive-blur\"]"));
        assert!(css.contains("[data-slot=\"progressive-blur-layer\"]"));
        assert!(css.contains("[data-slot=\"progressive-blur-content\"]"));
        assert!(css.contains("backdrop-filter"));
        assert!(css.contains("mask-image"));
        assert!(css.contains("blur(1px)"));
        assert!(css.contains("blur(4px)"));
        assert!(css.contains("blur(12px)"));
        assert!(css.contains("linear-gradient"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
    }
}
