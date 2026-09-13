//! Dedicated AnimatedList renderer. DOM matches React:
//! `<ul data-slot="animated-list">` plus each text as
//! `<li data-slot="animated-list-item">`. Static list (no JS stagger).
//! CSS in COMPONENT_CHROME. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::texts;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let items = texts(comp)
        .into_iter()
        .map(|t| format!("<li data-slot=\"animated-list-item\">{t}</li>"))
        .collect::<Vec<_>>()
        .join("");
    format!("<ul data-slot=\"animated-list\">{items}</ul>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn list(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("animated-list", items.first().copied().unwrap_or("Alpha"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("<div"));
        assert!(!html.contains("<span"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_is_ul_of_items_not_fx_title_box() {
        let html = render(&list(&["Alpha", "Beta"]));
        assert!(html.starts_with("<ul data-slot=\"animated-list\">"));
        assert_eq!(html.matches("data-slot=\"animated-list-item\"").count(), 2);
        assert!(html.contains("<li data-slot=\"animated-list-item\">Alpha</li>"));
        assert!(html.contains("<li data-slot=\"animated-list-item\">Beta</li>"));
        reject_fx(&html);
        assert_eq!(
            html,
            "<ul data-slot=\"animated-list\"><li data-slot=\"animated-list-item\">Alpha</li><li data-slot=\"animated-list-item\">Beta</li></ul>"
        );
    }

    #[test]
    fn extra_text_items_become_rows() {
        let mut c = stub("animated-list", "Alpha");
        c.items.push(extra("text", "Beta"));
        c.items.push(extra("text", "Gamma"));
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"animated-list-item\"").count(), 3);
        assert!(html.contains(">Alpha</li>"));
        assert!(html.contains(">Beta</li>"));
        assert!(html.contains(">Gamma</li>"));
        reject_fx(&html);
    }

    #[test]
    fn label_only_still_emits_one_item() {
        let html = render(&stub("animated-list", "Alpha"));
        assert_eq!(
            html,
            "<ul data-slot=\"animated-list\"><li data-slot=\"animated-list-item\">Alpha</li></ul>"
        );
        assert_eq!(html.matches("data-slot=\"animated-list-item\"").count(), 1);
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("animated-list", "A <B> & \"C\""));
        assert!(html.contains(
            "<li data-slot=\"animated-list-item\">A &lt;B&gt; &amp; &quot;C&quot;</li>"
        ));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("animated-list", "Alpha"));
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.contains("<span>"));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        assert!(!html.contains("<div"));
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::renderer_kind("meteors"),
            crate::cli::stub_renderer_gate::RendererKind::Stub("fx")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&list(&["Alpha"]));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"animated-list-item\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"animated-list\"]"));
        assert!(css.contains("[data-slot=\"animated-list-item\"]"));
        assert!(css.contains("flex-direction: column"));
        assert!(css.contains("gap: 0.5rem"));
        assert!(css.contains("list-style: none"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
