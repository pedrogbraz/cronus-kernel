//! Dedicated SpinningText renderer. DOM is a CSS-rotated orbit:
//! `<div data-slot="spinning-text">` with a visually hidden label span
//! (AT) plus `spinning-text-orbit` (aria-hidden). Keyframes live in
//! COMPONENT_CHROME — never a `<style>` tag or per-glyph inline transform.
//! Zero JS. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    format!(
        "<div data-slot=\"spinning-text\"><span data-slot=\"spinning-text-label\">{label}</span><span data-slot=\"spinning-text-orbit\" aria-hidden=\"true\">{label}</span></div>"
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
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setTimeout"));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("translateY"));
        assert!(!html.contains("rotate("));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_is_div_with_label_and_orbit_not_fx_title_box() {
        let html = render(&stub("spinning-text", "Cronus"));
        assert_eq!(
            html,
            "<div data-slot=\"spinning-text\"><span data-slot=\"spinning-text-label\">Cronus</span><span data-slot=\"spinning-text-orbit\" aria-hidden=\"true\">Cronus</span></div>"
        );
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"spinning-text\""));
        assert!(html.contains("data-slot=\"spinning-text-label\""));
        assert!(html.contains("data-slot=\"spinning-text-orbit\""));
        assert!(html.contains("aria-hidden=\"true\""));
        assert!(html.contains(">Cronus</span></div>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("spinning-text", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"spinning-text\"><span data-slot=\"spinning-text-label\">A &lt;B&gt; &amp; &quot;C&quot;</span><span data-slot=\"spinning-text-orbit\" aria-hidden=\"true\">A &lt;B&gt; &amp; &quot;C&quot;</span></div>"
        );
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("spinning-text", "Cronus");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.contains("<span>"));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("spinning-text"),
            Some("cronus_ui_spinning_text::render")
        );
        assert_eq!(
            renderer_kind("spinning-text"),
            RendererKind::Dedicated("cronus_ui_spinning_text::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("spinning-text", "Cronus"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"spinning-text\""));
            assert!(html.contains("data-slot=\"spinning-text-label\""));
            assert!(html.contains("data-slot=\"spinning-text-orbit\""));
        });
    }

    #[test]
    fn chrome_spinning_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"spinning-text\"]"));
        assert!(css.contains("[data-slot=\"spinning-text-label\"]"));
        assert!(css.contains("[data-slot=\"spinning-text-orbit\"]"));
        assert!(css.contains("@keyframes cui-spinning-text"));
        assert!(css.contains("animation: cui-spinning-text"));
        assert!(css.contains("rotate(360deg)"));
        assert!(css.contains("clip: rect(0, 0, 0, 0)"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
        assert!(!css.contains("style="));
    }
}
