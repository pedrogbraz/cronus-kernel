//! Dedicated Reveal renderer. DOM matches React:
//! `<div data-slot="reveal">` wrapping label text. CSS fade-in-up in
//! COMPONENT_CHROME. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!("<div data-slot=\"reveal\">{}</div>", label_of(comp))
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
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("reveal", "Headline"));
        assert_eq!(html, "<div data-slot=\"reveal\">Headline</div>");
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"reveal\""));
        assert!(html.contains(">Headline</div>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("reveal", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"reveal\">A &lt;B&gt; &amp; &quot;C&quot;</div>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("reveal", "Headline"));
        assert!(!html.contains("<span"));
        reject_fx(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("reveal", "Headline"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"reveal\""));
        });
    }

    #[test]
    fn chrome_reveal_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"reveal\"]"));
        assert!(css.contains("@keyframes cui-reveal"));
        assert!(css.contains("animation: cui-reveal"));
        assert!(css.contains("translateY(16px)"));
        assert!(css.contains("var(--ease-out-quart)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
