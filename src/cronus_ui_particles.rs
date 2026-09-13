//! Dedicated Particles renderer. DOM matches React:
//! `<div data-slot="particles">` wrapping label text. CSS-only dots live in
//! COMPONENT_CHROME (background, no JS canvas). Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!("<div data-slot=\"particles\">{}</div>", label_of(comp))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<span"));
        assert!(!html.contains("<canvas"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("particles", "Specks"));
        assert_eq!(html, "<div data-slot=\"particles\">Specks</div>");
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"particles\""));
        assert!(html.contains(">Specks</div>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("particles", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"particles\">A &lt;B&gt; &amp; &quot;C&quot;</div>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("particles", "Specks"));
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.contains("<span>"));
        assert_ne!(html, fx);
        assert!(!html.contains("<span"));
        reject_fx(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("particles", "Specks"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"particles\""));
        });
    }

    #[test]
    fn chrome_particles_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"particles\"]"));
        assert!(css.contains("radial-gradient"));
        assert!(css.contains("@keyframes cui-particles"));
        assert!(css.contains("animation: cui-particles"));
        assert!(css.contains("color-mix(in oklch, var(--cronus-fg)"));
        assert!(!css.contains("<canvas"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
