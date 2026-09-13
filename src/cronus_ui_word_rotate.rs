//! Dedicated WordRotate renderer. DOM matches React:
//! `<span data-slot="word-rotate">` showing the first word from texts
//! (static; no interval swap). CSS in COMPONENT_CHROME. Zero JS.
//! Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::{label_of, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let word = texts(comp)
        .into_iter()
        .next()
        .unwrap_or_else(|| label_of(comp));
    format!("<span data-slot=\"word-rotate\">{word}</span>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn extra(text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "item".into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn words(list: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("word-rotate", list[0]);
        c.items.clear();
        for t in list {
            c.items.push(extra(t));
        }
        c
    }

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
    }

    #[test]
    fn root_is_span_with_first_word_not_fx_title_box() {
        let html = render(&stub("word-rotate", "Ship"));
        assert_eq!(html, "<span data-slot=\"word-rotate\">Ship</span>");
        assert!(html.starts_with("<span "));
        assert!(html.contains("data-slot=\"word-rotate\""));
        assert!(html.contains(">Ship</span>"));
        reject_fx(&html);
    }

    #[test]
    fn shows_first_word_from_texts_statically() {
        let html = render(&words(&["Ship", "Scale", "Build"]));
        assert_eq!(html, "<span data-slot=\"word-rotate\">Ship</span>");
        assert!(!html.contains("Scale"));
        assert!(!html.contains("Build"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("word-rotate", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<span data-slot=\"word-rotate\">A &lt;B&gt; &amp; &quot;C&quot;</span>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("word-rotate", "Ship"));
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.contains("<span>"));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        assert!(!html.contains("<div"));
        reject_fx(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("word-rotate", "Ship"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"word-rotate\""));
        });
    }

    #[test]
    fn chrome_word_rotate_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"word-rotate\"]"));
        assert!(css.contains("overflow: hidden"));
        assert!(css.contains("height: 1.2em"));
        assert!(css.contains("display: inline-grid"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
