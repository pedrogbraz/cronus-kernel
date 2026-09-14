//! Dedicated BorderBeam renderer. DOM matches React idle:
//! `<div data-slot="border-beam">` + `data-slot="border-beam-layer"`
//! (aria-hidden) + `data-slot="border-beam-content"` wrapping the label.
//! `@keyframes cui-border-beam` lives in COMPONENT_CHROME — never a `<style>`
//! tag. Zero JS, no inline style. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"border-beam\"><div data-slot=\"border-beam-layer\" aria-hidden=\"true\"></div><div data-slot=\"border-beam-content\">{}</div></div>",
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
        let html = render(&stub("border-beam", "Beam"));
        assert_eq!(
            html,
            "<div data-slot=\"border-beam\"><div data-slot=\"border-beam-layer\" aria-hidden=\"true\"></div><div data-slot=\"border-beam-content\">Beam</div></div>"
        );
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"border-beam\""));
        assert!(html.contains("data-slot=\"border-beam-layer\""));
        assert!(html.contains("data-slot=\"border-beam-content\""));
        assert!(html.contains("aria-hidden=\"true\""));
        assert!(html.contains(">Beam</div></div>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("border-beam", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"border-beam\"><div data-slot=\"border-beam-layer\" aria-hidden=\"true\"></div><div data-slot=\"border-beam-content\">A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("border-beam", "Beam");
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
            dedicated_fn_name("border-beam"),
            Some("cronus_ui_border_beam::render")
        );
        assert_eq!(
            renderer_kind("border-beam"),
            RendererKind::Dedicated("cronus_ui_border_beam::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("border-beam", "Beam"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"border-beam\""));
            assert!(html.contains("data-slot=\"border-beam-layer\""));
            assert!(html.contains("data-slot=\"border-beam-content\""));
        });
    }

    #[test]
    fn chrome_border_beam_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"border-beam\"]"));
        assert!(css.contains("[data-slot=\"border-beam-layer\"]"));
        assert!(css.contains("[data-slot=\"border-beam-content\"]"));
        assert!(css.contains("@keyframes cui-border-beam"));
        assert!(css.contains("animation: cui-border-beam"));
        assert!(css.contains("offset-path"));
        assert!(css.contains("var(--cronus-fg)"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
    }
}
