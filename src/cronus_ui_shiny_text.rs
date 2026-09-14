//! Dedicated ShinyText renderer. DOM matches React:
//! `<span data-slot="shiny-text">` (display: contents) wrapping the painted
//! `<span>` that carries the sheen: `color: transparent` + `background-clip: text`
//! over a fg-tertiary → fg → fg-tertiary gradient at 200% width. React's inline
//! `<style>` keyframes live in COMPONENT_CHROME instead. Zero JS.
//! Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<span data-slot=\"shiny-text\"><span>{}</span></span>",
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
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn contents_slot_wraps_painted_span() {
        let html = render(&stub("shiny-text", "Sheen"));
        assert_eq!(
            html,
            "<span data-slot=\"shiny-text\"><span>Sheen</span></span>"
        );
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("shiny-text", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<span data-slot=\"shiny-text\"><span>A &lt;B&gt; &amp; &quot;C&quot;</span></span>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("shiny-text", "Shimmer");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert_ne!(html, fx);
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("shiny-text"),
            Some("cronus_ui_shiny_text::render")
        );
        assert_eq!(
            renderer_kind("shiny-text"),
            RendererKind::Dedicated("cronus_ui_shiny_text::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("shiny-text", "Shimmer"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"shiny-text\""));
        });
    }

    #[test]
    fn chrome_paints_text_transparent_with_clipped_gradient() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"shiny-text\"] { display: contents; }"));
        let start = css.find("[data-slot=\"shiny-text\"] > span {").unwrap();
        let end = start + css[start..].find('}').unwrap();
        let painted = &css[start..end];
        assert!(painted.contains("color: transparent;"));
        assert!(painted.contains("background-clip: text;"));
        assert!(painted.contains("background-size: 200% 100%;"));
        assert!(painted.contains("var(--cronus-fg-tertiary"));
        assert!(painted.contains("var(--cronus-fg) 50%"));
        assert!(painted.contains("animation: cui-shiny-text 3s linear infinite;"));
        assert!(css.contains("@keyframes cui-shiny-text"));
        assert!(!css.contains(FX_BOX));
    }
}
