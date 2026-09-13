//! Dedicated Masonry renderer. DOM matches React:
//! `<div data-slot="masonry">` grid of cells from text items.
//! Not catalog `display()` SURF `<section>` and not interact `scroll()`.

use crate::cronus_ui_kit::texts;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let cells = texts(comp)
        .into_iter()
        .map(|t| format!("<div data-slot=\"masonry-cell\">{t}</div>"))
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"masonry\">{cells}</div>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn wall(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("masonry", items.first().copied().unwrap_or("Card"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.starts_with("<section"));
        assert!(!html.contains("<section data-slot=\"masonry\""));
        assert!(!html.contains("<nav data-slot=\"masonry\""));
        assert!(!html.contains("<details"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("padding:1rem;display:flex;flex-direction:column;gap:0.5rem"));
        assert!(!html.contains("max-height:12rem;overflow:auto"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("display("));
        assert!(!html.contains("scroll("));
    }

    #[test]
    fn root_is_masonry_grid_of_cells_not_display_surf() {
        let html = render(&wall(&["Alpha", "Beta"]));
        assert!(html.starts_with("<div data-slot=\"masonry\">"));
        assert!(html.contains("<div data-slot=\"masonry-cell\">Alpha</div>"));
        assert!(html.contains("<div data-slot=\"masonry-cell\">Beta</div>"));
        assert_eq!(html.matches("data-slot=\"masonry-cell\"").count(), 2);
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"masonry\"><div data-slot=\"masonry-cell\">Alpha</div><div data-slot=\"masonry-cell\">Beta</div></div>"
        );
    }

    #[test]
    fn extra_text_items_become_cells() {
        let mut c = stub("masonry", "Alpha");
        c.items.push(extra("text", "Beta"));
        c.items.push(extra("text", "Gamma"));
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"masonry-cell\"").count(), 3);
        assert!(html.contains(">Alpha</div>"));
        assert!(html.contains(">Beta</div>"));
        assert!(html.contains(">Gamma</div>"));
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_one_cell() {
        let html = render(&stub("masonry", "Card"));
        assert!(html.contains("<div data-slot=\"masonry\">"));
        assert_eq!(html.matches("data-slot=\"masonry-cell\"").count(), 1);
        assert!(html.contains("<div data-slot=\"masonry-cell\">Card</div>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_scroll_and_display_surf() {
        let c = wall(&["Alpha", "Beta"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("masonry", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("data-slot=\"masonry\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("max-height:12rem;overflow:auto"));
        assert!(!interact.contains("data-slot=\"masonry-cell\""));
        assert!(html.contains("data-slot=\"masonry-cell\""));
        assert!(!html.contains("<section"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&wall(&["Alpha"]));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"masonry-cell\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"masonry\"]"));
        assert!(css.contains("[data-slot=\"masonry-cell\"]"));
        assert!(css.contains("column-count: 3"));
        assert!(css.contains("column-gap: 1rem"));
        assert!(css.contains("break-inside: avoid"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
