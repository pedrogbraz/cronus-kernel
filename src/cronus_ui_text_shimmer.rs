//! Dedicated TextShimmer renderer. DOM matches React:
//! `<p data-slot="text-shimmer">` with label text. CSS gradient animation
//! in COMPONENT_CHROME. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!("<p data-slot=\"text-shimmer\">{}</p>", label_of(comp))
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
        assert!(!html.contains("<div"));
        assert!(!html.contains("<span"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
    }

    /// React `TextShimmer` renders `<p data-slot="text-shimmer">`; audit e2e
    /// expects tag P.
    #[test]
    fn root_is_p_with_label_like_react() {
        let html = render(&stub("text-shimmer", "Thinking"));
        assert_eq!(html, "<p data-slot=\"text-shimmer\">Thinking</p>");
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("text-shimmer", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<p data-slot=\"text-shimmer\">A &lt;B&gt; &amp; &quot;C&quot;</p>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("text-shimmer", "Thinking"));
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.contains("<span>"));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        reject_fx(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("text-shimmer", "Thinking"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"text-shimmer\""));
        });
    }

    #[test]
    fn chrome_text_shimmer_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"text-shimmer\"] {\n  display: inline-block;\n  margin: 0;"));
        assert!(css.contains("background-clip: text"));
        assert!(css.contains("@keyframes cui-text-shimmer"));
        assert!(css.contains("animation: cui-text-shimmer"));
        assert!(css.contains("var(--cronus-surface-base)"));
        assert!(css.contains("var(--cronus-fg-tertiary"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
