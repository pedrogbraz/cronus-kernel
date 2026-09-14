//! Dedicated TypingText renderer. DOM matches React:
//! `<span data-slot="typing-text">` with full label text (static, no JS
//! typewriter). CSS caret in COMPONENT_CHROME. Not the catalog `fx()` title
//! SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!("<span data-slot=\"typing-text\">{}</span>", label_of(comp))
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
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("setTimeout"));
        assert!(!html.contains("setInterval"));
    }

    #[test]
    fn root_is_span_with_full_label_not_fx_title_box() {
        let html = render(&stub("typing-text", "Ship faster"));
        assert_eq!(html, "<span data-slot=\"typing-text\">Ship faster</span>");
        assert!(html.starts_with("<span "));
        assert!(html.contains("data-slot=\"typing-text\""));
        assert!(html.contains(">Ship faster</span>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("typing-text", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<span data-slot=\"typing-text\">A &lt;B&gt; &amp; &quot;C&quot;</span>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("typing-text", "Ship faster"));
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.contains("<span>"));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        assert!(!html.contains("<div"));
        reject_fx(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("typing-text", "Ship faster"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"typing-text\""));
        });
    }

    #[test]
    fn chrome_typing_caret_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"typing-text\"]"));
        assert!(css.contains("[data-slot=\"typing-text\"]::after"));
        assert!(css.contains("@keyframes cui-typing-caret"));
        assert!(css.contains("animation: cui-typing-caret"));
        assert!(css.contains("var(--cronus-fg)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
