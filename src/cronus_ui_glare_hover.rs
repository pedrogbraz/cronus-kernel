//! Dedicated GlareHover renderer. DOM matches React idle:
//! `<div data-slot="glare-hover">` + aria-hidden glare layer + relative
//! children div wrapping the label. Only the root has a `data-slot`, like
//! React (Wave 1t geometry parity). Diagonal glare is CSS-only
//! (`:hover` / `:focus-within`); no `--glare-x` JS. Not the catalog `fx()`
//! title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"glare-hover\"><div aria-hidden=\"true\"></div><div>{}</div></div>",
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
        assert!(!html.contains("--glare-x"));
        assert!(!html.contains("data-glare-hovered"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
        assert_eq!(html.matches("data-slot=").count(), 1);
    }

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("glare-hover", "Glare"));
        assert_eq!(
            html,
            "<div data-slot=\"glare-hover\"><div aria-hidden=\"true\"></div><div>Glare</div></div>"
        );
        assert!(!html.contains("glare-hover-layer"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("glare-hover", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"glare-hover\"><div aria-hidden=\"true\"></div><div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("glare-hover", "Glare");
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
            dedicated_fn_name("glare-hover"),
            Some("cronus_ui_glare_hover::render")
        );
        assert_eq!(
            renderer_kind("glare-hover"),
            RendererKind::Dedicated("cronus_ui_glare_hover::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("glare-hover", "Glare"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"glare-hover\""));
        });
    }

    #[test]
    fn chrome_glare_hover_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        let start = css.find("[data-slot=\"glare-hover\"] {").unwrap();
        let block = &css[start..start + css[start..].find('}').unwrap()];
        assert!(block.contains("width: 18rem;"));
        assert!(css.contains("[data-slot=\"glare-hover\"] > [aria-hidden] {"));
        assert!(css.contains("[data-slot=\"glare-hover\"] > div:last-child {"));
        assert!(!css.contains("[data-slot=\"glare-hover-layer\"]"));
        assert!(css.contains("linear-gradient"));
        assert!(css.contains("var(--cronus-fg)"));
        assert!(css.contains("[data-slot=\"glare-hover\"]:hover > [aria-hidden]"));
        assert!(css.contains("[data-slot=\"glare-hover\"]:focus-within > [aria-hidden]"));
        assert!(css.contains("50% 50%"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(!css.contains("--glare-x"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
