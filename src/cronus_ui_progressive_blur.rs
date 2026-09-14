//! Dedicated ProgressiveBlur renderer. React `ProgressiveBlur` is an
//! aria-hidden `absolute` band of three unslotted backdrop-blur layers; the
//! audit fixture wraps it in `relative min-h-32 w-72` after the text.
//! Kernel mirrors that: `progressive-blur-host` (the fixture wrapper) holds
//! the label as bare text, then `<div data-slot="progressive-blur">` with
//! three plain `<div>` layers styled by `> div:nth-child(n)` in
//! COMPONENT_CHROME. No inline style, zero JS. Not the catalog `fx()` box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

const LAYER: &str = "<div></div>";

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"progressive-blur-host\">{}<div data-slot=\"progressive-blur\" aria-hidden=\"true\">{LAYER}{LAYER}{LAYER}</div></div>",
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

    /// React fixture DOM: `<div class="relative min-h-32 w-72">Blur<div
    /// data-slot="progressive-blur" aria-hidden="true">` + 3 unslotted layers.
    #[test]
    fn text_precedes_aria_hidden_band_like_react_fixture() {
        let html = render(&stub("progressive-blur", "Blur"));
        assert_eq!(
            html,
            "<div data-slot=\"progressive-blur-host\">Blur<div data-slot=\"progressive-blur\" aria-hidden=\"true\"><div></div><div></div><div></div></div></div>"
        );
        assert!(!html.contains("progressive-blur-layer"));
        assert!(!html.contains("progressive-blur-content"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("progressive-blur", "A <B> & \"C\""));
        assert!(html.starts_with(
            "<div data-slot=\"progressive-blur-host\">A &lt;B&gt; &amp; &quot;C&quot;<div"
        ));
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
            assert!(html.contains("data-slot=\"progressive-blur\" aria-hidden=\"true\""));
            assert_eq!(html.matches("<div></div>").count(), 3);
        });
    }

    /// Fixture wrapper is `min-h-32 w-72` (288x128) with 24px line-height;
    /// band is 6rem at the bottom with 1/4/12px layers.
    #[test]
    fn chrome_matches_react_fixture_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"progressive-blur-host\"] {\n  position: relative;\n  width: 18rem;\n  min-height: 8rem;\n  line-height: 1.5;"
        ));
        assert!(css.contains("height: 6rem; z-index: 10; pointer-events: none;"));
        assert!(css.contains("[data-slot=\"progressive-blur\"] > div {"));
        for (n, px) in [(1, 1), (2, 4), (3, 12)] {
            let rule = format!(
                "[data-slot=\"progressive-blur\"] > div:nth-child({n}) {{\n  backdrop-filter: blur({px}px);"
            );
            assert!(css.contains(&rule), "{rule}");
        }
        assert!(css.contains("mask-image: linear-gradient(to top, black 30%, transparent 70%)"));
        assert!(css.contains("[data-slot=\"progressive-blur\"] { display: none; }"));
        assert!(!css.contains("progressive-blur-layer"));
        assert!(!css.contains("progressive-blur-content"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
    }
}
