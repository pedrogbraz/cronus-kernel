//! Dedicated Ripple renderer. DOM mirrors React's default `count=8`:
//! `<div data-slot="ripple">` + aria-hidden field `<div>` of eight empty ring
//! `<span>`s + a relative content `<div>` wrapping the label. Only the root
//! carries a `data-slot` (React adds none to field/rings/content). The
//! per-ring stagger React writes as inline `--ripple-delay` is
//! `:nth-of-type` `animation-delay` (0s..7s) in COMPONENT_CHROME, as is the
//! fixture `w-72 min-h-32`. `@keyframes cui-ripple` lives in COMPONENT_CHROME
//! — never a `<style>` tag. Zero JS, no inline style.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

/// React `Ripple` default `count`.
const RINGS: usize = 8;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"ripple\"><div aria-hidden=\"true\">{}</div><div>{}</div></div>",
        "<span></span>".repeat(RINGS),
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
            "<div data-slot=\"ripple\"><div aria-hidden=\"true\"><span></span><span></span><span></span><span></span><span></span><span></span><span></span><span></span></div><div>Pulse</div></div>"
        );
        assert_eq!(html.matches("data-slot=").count(), 1);
        assert_eq!(html.matches("<span></span>").count(), 8);
        assert!(!html.contains("ripple-ring"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("ripple", "A <B> & \"C\""));
        assert!(html.ends_with("<div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"));
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
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
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
            assert_eq!(html.matches("<span></span>").count(), 8);
        });
    }

    #[test]
    fn chrome_ripple_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"ripple\"] {\n  position: relative; overflow: hidden;\n  width: var(--cui-ripple-w, 100%); min-height: 8rem;"));
        assert!(css.contains("[data-slot=\"ripple\"] > [aria-hidden=\"true\"] > span {"));
        assert!(css.contains("[data-slot=\"ripple\"] > div:last-child {\n  position: relative;\n}"));
        assert!(css.contains("> span:nth-of-type(8) { animation-delay: 7s; }"));
        assert!(!css.contains("[data-slot=\"ripple-ring\"]"));
        assert!(css.contains("@keyframes cui-ripple"));
        assert!(css.contains("animation: cui-ripple"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("--ripple-delay"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
    }
}
