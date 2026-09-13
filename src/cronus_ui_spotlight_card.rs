//! Dedicated SpotlightCard renderer. DOM matches React:
//! `<div data-slot="spotlight-card">` wrapping label text. CSS radial
//! spotlight lives in COMPONENT_CHROME (`::after` at `--spot-x` / `--spot-y`).
//! Not catalog `display()` SURF `<section>`. Not interact `card()`.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!("<div data-slot=\"spotlight-card\">{}</div>", label_of(comp))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const DISPLAY_BOX: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem";

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_display(html: &str) {
        assert!(!html.contains(DISPLAY_BOX));
        assert!(!html.contains("<section"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("font-weight:500"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("display("));
    }

    #[test]
    fn root_is_div_wrapping_label_not_display_section() {
        let html = render(&stub("spotlight-card", "Hover me"));
        assert_eq!(html, "<div data-slot=\"spotlight-card\">Hover me</div>");
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"spotlight-card\""));
        assert!(html.contains(">Hover me</div>"));
        reject_display(&html);
    }

    #[test]
    fn extra_text_does_not_add_nodes() {
        let mut c = stub("spotlight-card", "Hover me");
        c.items.push(extra("text", "More"));
        let html = render(&c);
        assert_eq!(html, "<div data-slot=\"spotlight-card\">Hover me</div>");
        assert!(!html.contains("More"));
        reject_display(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("spotlight-card", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"spotlight-card\">A &lt;B&gt; &amp; &quot;C&quot;</div>"
        );
        reject_display(&html);
    }

    #[test]
    fn skips_display_surf_and_interact_card() {
        let c = stub("spotlight-card", "Hover me");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("spotlight-card", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<section data-slot=\"spotlight-card\""));
        assert!(interact.contains("style="));
        assert!(interact.contains(DISPLAY_BOX));
        assert!(interact.contains("font-weight:500"));
        assert!(!html.contains("<section"));
        reject_display(&html);
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains("padding:0.75rem 1rem;position:relative;overflow:hidden"));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("spotlight-card", "Hover me"));
            reject_display(&html);
            assert!(html.contains("data-slot=\"spotlight-card\""));
        });
    }

    #[test]
    fn chrome_spotlight_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"spotlight-card\"]"));
        assert!(css.contains("radial-gradient"));
        assert!(css.contains("--spot-x"));
        assert!(css.contains("--spot-y"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(css.contains("overflow: hidden"));
        assert!(css.contains("padding: 1.5rem"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(DISPLAY_BOX));
    }
}
