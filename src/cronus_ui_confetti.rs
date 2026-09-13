//! Dedicated Confetti renderer. DOM is CSS-only (React uses canvas + rAF):
//! `<div data-slot="confetti">` wrapping the label plus six empty
//! `span data-slot="confetti-piece"` dots. Zero canvas, script, or rAF.
//! CSS sprinkle lives in COMPONENT_CHROME. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

const PIECE: &str = "<span data-slot=\"confetti-piece\" aria-hidden=\"true\"></span>";

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"confetti\">{}{}{}{}{}{}{}</div>",
        label_of(comp),
        PIECE,
        PIECE,
        PIECE,
        PIECE,
        PIECE,
        PIECE
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
        assert!(!html.contains("SURF"));
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
        let html = render(&stub("confetti", "Celebrate"));
        assert_eq!(
            html,
            "<div data-slot=\"confetti\">Celebrate<span data-slot=\"confetti-piece\" aria-hidden=\"true\"></span><span data-slot=\"confetti-piece\" aria-hidden=\"true\"></span><span data-slot=\"confetti-piece\" aria-hidden=\"true\"></span><span data-slot=\"confetti-piece\" aria-hidden=\"true\"></span><span data-slot=\"confetti-piece\" aria-hidden=\"true\"></span><span data-slot=\"confetti-piece\" aria-hidden=\"true\"></span></div>"
        );
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"confetti\""));
        assert!(html.contains(">Celebrate<span"));
        assert_eq!(html.matches("data-slot=\"confetti-piece\"").count(), 6);
        assert!(html.contains("aria-hidden=\"true\""));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("confetti", "A <B> & \"C\""));
        assert!(html.starts_with(
            "<div data-slot=\"confetti\">A &lt;B&gt; &amp; &quot;C&quot;<span data-slot=\"confetti-piece\""
        ));
        assert!(!html.contains("<B>"));
        assert!(!html.contains("A <B>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("confetti", "Celebrate");
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
            dedicated_fn_name("confetti"),
            Some("cronus_ui_confetti::render")
        );
        assert_eq!(
            renderer_kind("confetti"),
            RendererKind::Dedicated("cronus_ui_confetti::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("confetti", "Celebrate"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"confetti\""));
            assert_eq!(html.matches("data-slot=\"confetti-piece\"").count(), 6);
        });
    }

    #[test]
    fn chrome_confetti_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"confetti\"]"));
        assert!(css.contains("[data-slot=\"confetti-piece\"]"));
        assert!(css.contains("@keyframes cui-confetti"));
        assert!(css.contains("animation: cui-confetti"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(!css.contains("<canvas"));
        assert!(!css.contains("requestAnimationFrame"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
