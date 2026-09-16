//! Dedicated WordRotate renderer. DOM mirrors React (`lockWidth`, `announce`):
//! `<span data-slot="word-rotate">` + visually hidden current word (React's
//! polite live region, `sr-only`) + one invisible `data-word-rotate-sizer`
//! span per word (locks the box to the widest word) + the visible first word.
//! Words are the fixture `text` rows; the `label` row is not a word unless it
//! is the only row. Static: no interval swap (zero JS) — the kernel shows the
//! word React settles on at mount. CSS in COMPONENT_CHROME. Not the catalog
//! `fx()` title SURF box.

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let list = words(comp);
    let current = list.first().cloned().unwrap_or_default();
    let sizers: String = list
        .iter()
        .map(|w| format!("<span aria-hidden=\"true\" data-word-rotate-sizer=\"\">{w}</span>"))
        .collect();
    format!(
        "<span data-slot=\"word-rotate\"><span>{current}</span>{sizers}<span aria-hidden=\"true\">{current}</span></span>"
    )
}

fn words(comp: &ComponentNode) -> Vec<String> {
    let out: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type != "label" && i.item_type != "title" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if out.is_empty() {
        vec![label_of(comp)]
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn extra(text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "text".into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
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
    fn fixture_label_plus_words_locks_width_to_all_words() {
        let mut c = stub("word-rotate", "Design");
        c.items.push(extra("Design"));
        c.items.push(extra("System"));
        let html = render(&c);
        assert_eq!(
            html,
            "<span data-slot=\"word-rotate\"><span>Design</span><span aria-hidden=\"true\" data-word-rotate-sizer=\"\">Design</span><span aria-hidden=\"true\" data-word-rotate-sizer=\"\">System</span><span aria-hidden=\"true\">Design</span></span>"
        );
        assert_eq!(html.matches("data-slot=").count(), 1);
        reject_fx(&html);
    }

    #[test]
    fn bare_label_is_the_only_word() {
        let html = render(&stub("word-rotate", "A <B>"));
        assert_eq!(
            html,
            "<span data-slot=\"word-rotate\"><span>A &lt;B&gt;</span><span aria-hidden=\"true\" data-word-rotate-sizer=\"\">A &lt;B&gt;</span><span aria-hidden=\"true\">A &lt;B&gt;</span></span>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("word-rotate", "Ship"));
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
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
        assert!(css.contains("[data-slot=\"word-rotate\"] {\n  position: relative; display: inline-grid;\n  height: 1.2em; overflow: hidden;\n  vertical-align: baseline;\n}"));
        assert!(
            css.contains("[data-slot=\"word-rotate\"] > span:first-child {\n  position: absolute;")
        );
        assert!(css.contains("[data-slot=\"word-rotate\"] > [data-word-rotate-sizer] {\n  visibility: hidden; grid-area: 1 / 1; white-space: nowrap;\n}"));
        assert!(css.contains("[data-slot=\"word-rotate\"] > span:last-child {\n  grid-area: 1 / 1; display: inline-block; white-space: nowrap; justify-self: start;\n}"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
