//! Dedicated GlassCard renderer. DOM matches React:
//! `<div data-slot="glass-card">` wrapping label text. Frost lives in
//! COMPONENT_CHROME (`backdrop-filter`). Not the catalog `display()` SURF
//! `<section>`.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!("<div data-slot=\"glass-card\">{}</div>", label_of(comp))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    const DISPLAY_SURF: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem;";

    fn reject_display(html: &str) {
        assert!(!html.contains(DISPLAY_SURF));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<section"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_wraps_label_text_not_display_surf_section() {
        let html = render(&stub("glass-card", "Frost"));
        assert_eq!(html, "<div data-slot=\"glass-card\">Frost</div>");
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"glass-card\""));
        assert!(html.contains(">Frost</div>"));
        reject_display(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("glass-card", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"glass-card\">A &lt;B&gt; &amp; &quot;C&quot;</div>"
        );
        reject_display(&html);
    }

    #[test]
    fn skips_display_surf_section() {
        let html = render(&stub("glass-card", "Frost"));
        let interact =
            crate::cronus_ui_interact::render("glass-card", &stub("glass-card", "Frost")).unwrap();
        assert!(interact.starts_with("<section data-slot=\"glass-card\""));
        assert!(interact.contains("style="));
        assert!(interact.contains(DISPLAY_SURF));
        assert_ne!(html, interact);
        let display =
            crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("card-stack"))
                .unwrap();
        assert!(display.contains("<section data-slot=\"card-stack\""));
        assert!(display.contains(DISPLAY_SURF));
        assert!(!html.contains("<section"));
        reject_display(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("glass-card", "Frost"));
            reject_display(&html);
            assert!(html.contains("data-slot=\"glass-card\""));
        });
    }

    #[test]
    fn chrome_glass_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"glass-card\"]"));
        assert!(css.contains("backdrop-filter: blur(24px)"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-shadow-lg"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(DISPLAY_SURF));
    }
}
