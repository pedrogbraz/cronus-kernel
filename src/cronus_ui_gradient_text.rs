//! Dedicated GradientText renderer. DOM matches React:
//! `<span data-slot="gradient-text">` with label. CSS gradient fill
//! in COMPONENT_CHROME. Zero JS. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<span data-slot=\"gradient-text\">{}</span>",
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
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_is_span_with_label_not_fx_title_box() {
        let html = render(&stub("gradient-text", "Proud of"));
        assert_eq!(html, "<span data-slot=\"gradient-text\">Proud of</span>");
        assert!(html.starts_with("<span "));
        assert!(html.contains("data-slot=\"gradient-text\""));
        assert!(html.contains(">Proud of</span>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("gradient-text", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<span data-slot=\"gradient-text\">A &lt;B&gt; &amp; &quot;C&quot;</span>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("gradient-text", "Proud of");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        assert!(!html.contains("<div"));
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("gradient-text"),
            Some("cronus_ui_gradient_text::render")
        );
        assert_eq!(
            renderer_kind("gradient-text"),
            RendererKind::Dedicated("cronus_ui_gradient_text::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("gradient-text", "Proud of"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"gradient-text\""));
        });
    }

    #[test]
    fn chrome_gradient_fill_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"gradient-text\"]"));
        assert!(css.contains("background-clip: text"));
        assert!(css.contains("linear-gradient(135deg"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(css.contains("var(--cronus-accent)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
