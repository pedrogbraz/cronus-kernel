//! Dedicated FlipCard renderer. DOM matches React:
//! `<div data-slot="flip-card">` plus `flip-card-front` (label) and
//! `flip-card-back` (extra text). Static CSS 3D in COMPONENT_CHROME; no JS.
//! Not the catalog `display()` SURF `<section>`.

use crate::cronus_ui_kit::{label_of, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let front = label_of(comp);
    let back = texts(comp)
        .into_iter()
        .skip(1)
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "<div data-slot=\"flip-card\"><div data-slot=\"flip-card-front\">{front}</div><div data-slot=\"flip-card-back\">{back}</div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const DISPLAY_SURF: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem;";

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
        assert!(!html.contains(DISPLAY_SURF));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<section"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("onmouseenter"));
        assert!(!html.contains("addEventListener"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("display("));
    }

    #[test]
    fn root_is_flip_card_with_front_label_and_back() {
        let html = render(&stub("flip-card", "Front"));
        assert_eq!(
            html,
            "<div data-slot=\"flip-card\"><div data-slot=\"flip-card-front\">Front</div><div data-slot=\"flip-card-back\"></div></div>"
        );
        assert!(html.starts_with("<div data-slot=\"flip-card\">"));
        assert!(html.contains("data-slot=\"flip-card-front\">Front</div>"));
        assert!(html.contains("data-slot=\"flip-card-back\"></div>"));
        reject_display(&html);
    }

    #[test]
    fn extra_text_becomes_back_face() {
        let mut c = stub("flip-card", "Front");
        c.items.push(extra("text", "Back copy"));
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"flip-card\"><div data-slot=\"flip-card-front\">Front</div><div data-slot=\"flip-card-back\">Back copy</div></div>"
        );
        reject_display(&html);
    }

    #[test]
    fn label_is_escaped() {
        let mut c = stub("flip-card", "A <B> & \"C\"");
        c.items.push(extra("text", "D <E>"));
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"flip-card\"><div data-slot=\"flip-card-front\">A &lt;B&gt; &amp; &quot;C&quot;</div><div data-slot=\"flip-card-back\">D &lt;E&gt;</div></div>"
        );
        reject_display(&html);
    }

    #[test]
    fn skips_display_surf_section() {
        let html = render(&stub("flip-card", "Front"));
        let interact =
            crate::cronus_ui_interact::render("flip-card", &stub("flip-card", "Front")).unwrap();
        assert!(interact.starts_with("<section data-slot=\"flip-card\""));
        assert!(interact.contains("style="));
        assert!(interact.contains(DISPLAY_SURF));
        assert!(!interact.contains("data-slot=\"flip-card-front\""));
        assert!(!interact.contains("data-slot=\"flip-card-back\""));
        assert_ne!(html, interact);
        assert!(!html.contains("<section"));
        reject_display(&html);
        assert_eq!(
            dedicated_fn_name("flip-card"),
            Some("cronus_ui_flip_card::render")
        );
        assert_eq!(
            renderer_kind("flip-card"),
            RendererKind::Dedicated("cronus_ui_flip_card::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("flip-card", "Front"));
            reject_display(&html);
            assert!(html.contains("data-slot=\"flip-card-front\""));
            assert!(html.contains("data-slot=\"flip-card-back\""));
        });
    }

    #[test]
    fn chrome_flip_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"flip-card\"]"));
        assert!(css.contains("[data-slot=\"flip-card-front\"]"));
        assert!(css.contains("[data-slot=\"flip-card-back\"]"));
        assert!(css.contains("perspective: 1600px"));
        assert!(css.contains("transform-style: preserve-3d"));
        assert!(css.contains("backface-visibility: hidden"));
        assert!(css.contains("rotateY(180deg)"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(css.contains("var(--cronus-surface-elevated"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(DISPLAY_SURF));
        assert!(!css.contains("onclick"));
    }
}
