//! Dedicated AnimatedList renderer. DOM matches React:
//! `<ul data-slot="animated-list">` plus each row as
//! `<li data-slot="animated-list-item">`. Static list (no JS stagger).
//! CSS in COMPONENT_CHROME. Not the catalog `fx()` title SURF box.
//! Wave 1t: `label` / `title` name the list and are never rows when real
//! rows (`text` / `item`) exist — the emitter writes the fixture id as label.

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let items = rows(comp)
        .into_iter()
        .map(|t| format!("<li data-slot=\"animated-list-item\">{t}</li>"))
        .collect::<Vec<_>>()
        .join("");
    format!("<ul data-slot=\"animated-list\">{items}</ul>")
}

fn rows(comp: &ComponentNode) -> Vec<String> {
    let out: Vec<String> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty() && !matches!(i.item_type.as_str(), "label" | "title"))
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
        assert_eq!(
            html,
            "<ul data-slot=\"animated-list\"><li data-slot=\"animated-list-item\">Alpha</li><li data-slot=\"animated-list-item\">Beta</li></ul>"
        );
        reject_fx(&html);
    }

    /// Audit fixture shape: `label "default"` + `text "Alpha"` + `text "Beta"`.
    /// React renders exactly the two items; the label must not leak as a row.
    #[test]
    fn label_is_not_a_row_when_texts_exist() {
        let mut c = stub("animated-list", "default");
        c.items.push(extra("text", "Alpha"));
        c.items.push(extra("text", "Beta"));
        let html = render(&c);
        assert_eq!(
            html,
            "<ul data-slot=\"animated-list\"><li data-slot=\"animated-list-item\">Alpha</li><li data-slot=\"animated-list-item\">Beta</li></ul>"
        );
        assert!(!html.contains("default"));
        reject_fx(&html);
    }

    #[test]
    fn label_only_still_emits_one_item() {
        let html = render(&stub("animated-list", "Alpha"));
        assert_eq!(
            html,
            "<ul data-slot=\"animated-list\"><li data-slot=\"animated-list-item\">Alpha</li></ul>"
        );
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("animated-list", "A <B> & \"C\""));
        assert!(html
            .contains("<li data-slot=\"animated-list-item\">A &lt;B&gt; &amp; &quot;C&quot;</li>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("animated-list", "Alpha"));
        reject_fx(&html);
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
