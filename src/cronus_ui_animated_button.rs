//! Dedicated AnimatedButton renderer. DOM matches React idle:
//! `<button type="button" data-slot="animated-button">` with label.
//! CSS hover in COMPONENT_CHROME. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<button type=\"button\" data-slot=\"animated-button\">{}</button>",
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
        assert!(!html.contains("<div"));
        assert!(!html.contains("<span"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("whileHover"));
        assert!(!html.contains("whileTap"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_is_button_with_label_not_fx_title_box() {
        let html = render(&stub("animated-button", "Launch"));
        assert_eq!(
            html,
            "<button type=\"button\" data-slot=\"animated-button\">Launch</button>"
        );
        assert!(html.starts_with("<button "));
        assert!(html.contains("type=\"button\""));
        assert!(html.contains("data-slot=\"animated-button\""));
        assert!(html.contains(">Launch</button>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("animated-button", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<button type=\"button\" data-slot=\"animated-button\">A &lt;B&gt; &amp; &quot;C&quot;</button>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("animated-button", "Launch");
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
            dedicated_fn_name("animated-button"),
            Some("cronus_ui_animated_button::render")
        );
        assert_eq!(
            renderer_kind("animated-button"),
            RendererKind::Dedicated("cronus_ui_animated_button::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("animated-button", "Launch"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"animated-button\""));
        });
    }

    #[test]
    fn chrome_hover_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"animated-button\"]"));
        assert!(css.contains("[data-slot=\"animated-button\"]:hover"));
        assert!(css.contains("translateY(-1px)"));
        assert!(css.contains("scale(0.97)"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(css.contains("var(--cronus-primary-foreground)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
