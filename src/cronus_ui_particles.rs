//! Dedicated Particles renderer. DOM mirrors React minus the `<canvas>`:
//! `<div data-slot="particles">` + an aria-hidden speck layer `<div>` (React:
//! aria-hidden canvas driven by rAF) + a relative content `<div>` wrapping
//! the label. Specks are CSS radial-gradient dots drifting via
//! `@keyframes cui-particles` in COMPONENT_CHROME (zero JS, no canvas). The
//! root is transparent like React; fixture `h-32 w-72` is mirrored in CSS.
//! Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"particles\"><div aria-hidden=\"true\"></div><div>{}</div></div>",
        label_of(comp)
    )
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
    fn root_has_speck_layer_and_content_div() {
        let html = render(&stub("particles", "Specks"));
        assert_eq!(
            html,
            "<div data-slot=\"particles\"><div aria-hidden=\"true\"></div><div>Specks</div></div>"
        );
        assert_eq!(html.matches("data-slot=").count(), 1);
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("particles", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"particles\"><div aria-hidden=\"true\"></div><div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("particles", "Specks"));
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert_ne!(html, fx);
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
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
        assert!(css.contains("[data-slot=\"particles\"] {\n  position: relative; overflow: hidden;\n  width: 18rem; height: 8rem;\n}"));
        assert!(css.contains("[data-slot=\"particles\"] > [aria-hidden=\"true\"] {"));
        assert!(
            css.contains("[data-slot=\"particles\"] > div:last-child {\n  position: relative;\n}")
        );
        assert!(css.contains("@keyframes cui-particles"));
        assert!(css.contains("animation: cui-particles"));
        assert!(css.contains("color-mix(in oklch, var(--cronus-fg) 35%, transparent)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
