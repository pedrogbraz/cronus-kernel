//! Dedicated Ripple renderer. DOM is CSS-only (React emits 8 rings with
//! inline `--ripple-delay`): `<div data-slot="ripple">` + aria-hidden field
//! of four empty `span data-slot="ripple-ring"` plus content wrapping the
//! label. `@keyframes cui-ripple` lives in COMPONENT_CHROME — never a `<style>`
//! tag. Zero JS, no inline style. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

const RING: &str = "<span data-slot=\"ripple-ring\" aria-hidden=\"true\"></span>";

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"ripple\"><div data-slot=\"ripple-field\" aria-hidden=\"true\">{}{}{}{}</div><div data-slot=\"ripple-content\">{}</div></div>",
        RING,
        RING,
        RING,
        RING,
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
        assert!(!html.contains("<canvas"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("requestAnimationFrame"));
        assert!(!html.contains("--ripple-delay"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("ripple", "Pulse"));
        assert_eq!(
            html,
            "<div data-slot=\"ripple\"><div data-slot=\"ripple-field\" aria-hidden=\"true\"><span data-slot=\"ripple-ring\" aria-hidden=\"true\"></span><span data-slot=\"ripple-ring\" aria-hidden=\"true\"></span><span data-slot=\"ripple-ring\" aria-hidden=\"true\"></span><span data-slot=\"ripple-ring\" aria-hidden=\"true\"></span></div><div data-slot=\"ripple-content\">Pulse</div></div>"
        );
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"ripple\""));
        assert!(html.contains("data-slot=\"ripple-field\""));
        assert!(html.contains("data-slot=\"ripple-ring\""));
        assert!(html.contains("data-slot=\"ripple-content\""));
        assert!(html.contains("aria-hidden=\"true\""));
        assert_eq!(html.matches("data-slot=\"ripple-ring\"").count(), 4);
        assert!(html.contains(">Pulse</div></div>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("ripple", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"ripple\"><div data-slot=\"ripple-field\" aria-hidden=\"true\"><span data-slot=\"ripple-ring\" aria-hidden=\"true\"></span><span data-slot=\"ripple-ring\" aria-hidden=\"true\"></span><span data-slot=\"ripple-ring\" aria-hidden=\"true\"></span><span data-slot=\"ripple-ring\" aria-hidden=\"true\"></span></div><div data-slot=\"ripple-content\">A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("ripple", "Pulse");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.contains("<span>"));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        assert!(!html.contains("<span>Demo</span></div>"));
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("ripple"),
            Some("cronus_ui_ripple::render")
        );
        assert_eq!(
            renderer_kind("ripple"),
            RendererKind::Dedicated("cronus_ui_ripple::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("ripple", "Pulse"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"ripple\""));
            assert!(html.contains("data-slot=\"ripple-field\""));
            assert_eq!(html.matches("data-slot=\"ripple-ring\"").count(), 4);
            assert!(html.contains("data-slot=\"ripple-content\""));
        });
    }

    #[test]
    fn chrome_ripple_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"ripple\"]"));
        assert!(css.contains("[data-slot=\"ripple-field\"]"));
        assert!(css.contains("[data-slot=\"ripple-ring\"]"));
        assert!(css.contains("[data-slot=\"ripple-content\"]"));
        assert!(css.contains("@keyframes cui-ripple"));
        assert!(css.contains("animation: cui-ripple"));
        assert!(css.contains("nth-of-type"));
        assert!(css.contains("animation-delay"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("<canvas"));
        assert!(!css.contains("--ripple-delay"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
    }
}
