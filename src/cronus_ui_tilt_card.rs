//! Dedicated TiltCard renderer. DOM matches React:
//! `<div data-slot="tilt-card">` wrapping label text. Optional CSS 3D lives
//! in COMPONENT_CHROME (static defaults; no pointer JS). Not the catalog
//! `display()` SURF `<section>`.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!("<div data-slot=\"tilt-card\">{}</div>", label_of(comp))
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
        let html = render(&stub("tilt-card", "Tilt"));
        assert_eq!(html, "<div data-slot=\"tilt-card\">Tilt</div>");
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"tilt-card\""));
        assert!(html.contains(">Tilt</div>"));
        reject_display(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("tilt-card", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"tilt-card\">A &lt;B&gt; &amp; &quot;C&quot;</div>"
        );
        reject_display(&html);
    }

    #[test]
    fn skips_display_surf_section() {
        let html = render(&stub("tilt-card", "Tilt"));
        let interact =
            crate::cronus_ui_interact::render("tilt-card", &stub("tilt-card", "Tilt")).unwrap();
        assert!(interact.starts_with("<section data-slot=\"tilt-card\""));
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
            let html = render(&stub("tilt-card", "Tilt"));
            reject_display(&html);
            assert!(html.contains("data-slot=\"tilt-card\""));
        });
    }

    #[test]
    fn chrome_tilt_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"tilt-card\"]"));
        assert!(css.contains("transform-style: preserve-3d"));
        assert!(css.contains("perspective(1000px)"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(DISPLAY_SURF));
    }
}
