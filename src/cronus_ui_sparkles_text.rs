//! Dedicated SparklesText renderer. DOM matches React:
//! `<span data-slot="sparkles-text">` with label. CSS sparkles live in
//! COMPONENT_CHROME. Zero JS. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<span data-slot=\"sparkles-text\">{}</span>",
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
        assert!(!html.contains("<div"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_is_span_with_label_not_fx_title_box() {
        let html = render(&stub("sparkles-text", "Magic"));
        assert_eq!(html, "<span data-slot=\"sparkles-text\">Magic</span>");
        assert!(html.starts_with("<span "));
        assert!(html.contains("data-slot=\"sparkles-text\""));
        assert!(html.contains(">Magic</span>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("sparkles-text", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<span data-slot=\"sparkles-text\">A &lt;B&gt; &amp; &quot;C&quot;</span>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("sparkles-text", "Magic"));
        assert!(!html.contains("<div"));
        reject_fx(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("sparkles-text", "Magic"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"sparkles-text\""));
        });
    }

    #[test]
    fn chrome_sparkles_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"sparkles-text\"]"));
        assert!(css.contains("clip-path: polygon"));
        assert!(css.contains("@keyframes cui-sparkle"));
        assert!(css.contains("animation: cui-sparkle"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
