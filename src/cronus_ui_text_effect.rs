//! Dedicated TextEffect renderer. DOM mirrors React root:
//! `<p data-slot="text-effect">` with a visually hidden `<span class="sr-only">`
//! carrying the accessible text and an `aria-hidden` span with the visible
//! text (React splits that one into per-word framer spans; the kernel animates
//! the paragraph as a whole). CSS enter animation in COMPONENT_CHROME (off
//! under reduced motion). Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let text = label_of(comp);
    format!(
        "<p data-slot=\"text-effect\"><span class=\"sr-only\">{text}</span><span aria-hidden=\"true\">{text}</span></p>"
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
        assert!(!html.contains("<div"));
        // The meteors fx stub title is a bare `<span>`; ours always carry attrs.
        assert!(!html.contains("<span>"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
    }

    /// React `TextEffect`: `<p data-slot="text-effect">` > `span.sr-only` +
    /// `span[aria-hidden]`. The audit geometry spec reads the paragraph text
    /// as "Headline Headline" on the React side, so the kernel must emit both.
    #[test]
    fn root_is_p_with_sr_only_and_visible_text_like_react() {
        let html = render(&stub("text-effect", "Ship faster"));
        assert_eq!(
            html,
            "<p data-slot=\"text-effect\"><span class=\"sr-only\">Ship faster</span><span aria-hidden=\"true\">Ship faster</span></p>"
        );
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("text-effect", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<p data-slot=\"text-effect\"><span class=\"sr-only\">A &lt;B&gt; &amp; &quot;C&quot;</span><span aria-hidden=\"true\">A &lt;B&gt; &amp; &quot;C&quot;</span></p>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("text-effect", "Ship faster"));
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
            let html = render(&stub("text-effect", "Ship faster"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"text-effect\""));
        });
    }

    /// The reduced-motion override used to sit outside any `@media`, which
    /// disabled the enter animation for everyone.
    #[test]
    fn chrome_text_effect_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        // Wave 1s: React `<p>` is a block (432×24 in the audit canvas) and
        // the settled state has no residual transform (computed `none`).
        assert!(css.contains(
            "[data-slot=\"text-effect\"] {\n  display: block;\n  margin: 0;\n  line-height: 1.5;\n  animation: cui-text-effect 400ms ease-out both;\n}"
        ));
        assert!(css.contains(
            "[data-slot=\"text-effect\"] > .sr-only {\n  position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;"
        ));
        assert!(css.contains(
            "@keyframes cui-text-effect {\n  from { opacity: 0; filter: blur(8px); }\n  to { opacity: 1; filter: none; }\n}"
        ));
        assert!(!css.contains("to { opacity: 1; filter: blur(0); transform: translateY(0); }"));
        assert!(css.contains("@keyframes cui-text-effect"));
        assert!(css.contains("animation: cui-text-effect"));
        assert!(css.contains(
            "@media (prefers-reduced-motion: reduce) {\n  [data-slot=\"text-effect\"] { animation: none; }\n}"
        ));
        assert!(!css.contains("}\n  [data-slot=\"text-effect\"] { animation: none; }\n[data-slot="));
        assert!(css.contains("filter: blur"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
