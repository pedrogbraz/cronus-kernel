//! Dedicated LightRays renderer. DOM mirrors React without the injected
//! `<style>`: `<div data-slot="light-rays">` + an aria-hidden field `<div>`
//! holding the rotating ray `<div>` + a relative content `<div>` wrapping the
//! label. Only the root carries a `data-slot` (React adds none to the inner
//! layers). Fixture `w-72 min-h-32` is mirrored in COMPONENT_CHROME.
//! `@keyframes cui-light-rays` lives in COMPONENT_CHROME. Zero JS, no
//! inline style. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"light-rays\"><div aria-hidden=\"true\"><div></div></div><div>{}</div></div>",
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
        let html = render(&stub("light-rays", "Rays"));
        assert_eq!(
            html,
            "<div data-slot=\"light-rays\"><div aria-hidden=\"true\"><div></div></div><div>Rays</div></div>"
        );
        assert_eq!(html.matches("data-slot=").count(), 1);
        assert!(!html.contains("light-rays-field"));
        assert!(!html.contains("light-rays-content"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("light-rays", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"light-rays\"><div aria-hidden=\"true\"><div></div></div><div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("light-rays", "Rays");
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
            dedicated_fn_name("light-rays"),
            Some("cronus_ui_light_rays::render")
        );
        assert_eq!(
            renderer_kind("light-rays"),
            RendererKind::Dedicated("cronus_ui_light_rays::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("light-rays", "Rays"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"light-rays\""));
        });
    }

    #[test]
    fn chrome_light_rays_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"light-rays\"] {\n  position: relative;\n  overflow: hidden;\n  width: var(--cui-light-rays-w, 100%);\n  min-height: 8rem;"));
        assert!(css.contains("[data-slot=\"light-rays\"] > [aria-hidden=\"true\"] > div {"));
        assert!(
            css.contains("[data-slot=\"light-rays\"] > div:last-child {\n  position: relative;\n}")
        );
        assert!(!css.contains("[data-slot=\"light-rays-field\"]"));
        assert!(css.contains("repeating-conic-gradient"));
        assert!(css.contains("@keyframes cui-light-rays"));
        assert!(css.contains("animation: cui-light-rays"));
        assert!(css.contains("rotate(360deg)"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
        assert!(!css.contains("style="));
    }
}
