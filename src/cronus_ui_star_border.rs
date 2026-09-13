//! Dedicated StarBorder renderer. DOM matches React:
//! `<div data-slot="star-border">` wrapping label text. Perimeter sparkle
//! animation lives in COMPONENT_CHROME (`::before`/`::after` + `@keyframes`).
//! Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!("<div data-slot=\"star-border\">{}</div>", label_of(comp))
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
        let html = render(&stub("star-border", "Sparkle"));
        assert_eq!(html, "<div data-slot=\"star-border\">Sparkle</div>");
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"star-border\""));
        assert!(html.contains(">Sparkle</div>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("star-border", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"star-border\">A &lt;B&gt; &amp; &quot;C&quot;</div>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("star-border", "Sparkle"));
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.contains("<span>"));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        assert!(!html.contains("<span"));
        reject_fx(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("star-border", "Sparkle"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"star-border\""));
        });
    }

    #[test]
    fn chrome_star_border_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"star-border\"]"));
        assert!(css.contains("@keyframes cui-star-border"));
        assert!(css.contains("animation: cui-star-border"));
        assert!(css.contains("offset-path"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
