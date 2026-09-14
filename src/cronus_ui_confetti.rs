//! Dedicated Confetti renderer. React renders an empty `aria-hidden` canvas
//! (particles only after a pointer burst, `fireOnMount` defaults to false) plus
//! a relative children div. The zero-JS kernel emits no canvas: the settled
//! idle render is just `<div data-slot="confetti"><div>{label}</div></div>`.
//! Only the root has a `data-slot`, like React (Wave 1t geometry parity — the
//! former six decorative `confetti-piece` spans painted pieces React never
//! shows at idle). Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"confetti\"><div>{}</div></div>",
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
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<canvas"));
        assert!(!html.contains("<span"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("requestAnimationFrame"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
        assert_eq!(html.matches("data-slot=").count(), 1);
    }

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("confetti", "Celebrate"));
        assert_eq!(
            html,
            "<div data-slot=\"confetti\"><div>Celebrate</div></div>"
        );
        assert!(!html.contains("confetti-piece"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("confetti", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"confetti\"><div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
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
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
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
        });
    }

    #[test]
    fn chrome_confetti_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        let start = css.find("[data-slot=\"confetti\"] {").unwrap();
        let block = &css[start..start + css[start..].find('}').unwrap()];
        assert!(block.contains("position: relative;"));
        assert!(block.contains("width: 18rem; min-height: 8rem;"));
        assert!(css.contains("[data-slot=\"confetti\"] > div {"));
        assert!(!css.contains("[data-slot=\"confetti-piece\"]"));
        assert!(!css.contains("@keyframes cui-confetti"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
