//! Dedicated ScrambleText renderer. DOM is the idle/static phrase:
//! `<span data-slot="scramble-text">` wrapping `scramble-text-label`.
//! No scramble ticks, no random glyphs, no `setTimeout`. Optional
//! `font-mono` lives in COMPONENT_CHROME. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<span data-slot=\"scramble-text\"><span data-slot=\"scramble-text-label\">{}</span></span>",
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
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_is_span_with_static_label_not_fx_title_box() {
        let html = render(&stub("scramble-text", "Decrypt"));
        assert_eq!(
            html,
            "<span data-slot=\"scramble-text\"><span data-slot=\"scramble-text-label\">Decrypt</span></span>"
        );
        assert!(html.starts_with("<span "));
        assert!(html.contains("data-slot=\"scramble-text\""));
        assert!(html.contains("data-slot=\"scramble-text-label\""));
        assert!(html.contains(">Decrypt</span></span>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("scramble-text", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<span data-slot=\"scramble-text\"><span data-slot=\"scramble-text-label\">A &lt;B&gt; &amp; &quot;C&quot;</span></span>"
        );
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("scramble-text", "Decrypt");
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
            dedicated_fn_name("scramble-text"),
            Some("cronus_ui_scramble_text::render")
        );
        assert_eq!(
            renderer_kind("scramble-text"),
            RendererKind::Dedicated("cronus_ui_scramble_text::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("scramble-text", "Decrypt"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"scramble-text\""));
            assert!(html.contains("data-slot=\"scramble-text-label\""));
        });
    }

    #[test]
    fn chrome_scramble_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"scramble-text\"]"));
        assert!(css.contains("var(--cronus-font-mono, ui-monospace, monospace)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("setTimeout"));
        assert!(!css.contains("<style"));
    }
}
