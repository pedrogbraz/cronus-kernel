//! Dedicated Kanban renderer. DOM matches React slots:
//! `<div data-slot="kanban">` plus at least one
//! `<div data-slot="kanban-column">` and each text as
//! `<div data-slot="kanban-card">`. Static board (no DnD JS).
//! Not interact `kanban()` (inline SURF columns) or catalog `display()` SURF
//! `<section>`.

use crate::cronus_ui_kit::texts;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let cards = texts(comp)
        .into_iter()
        .map(|t| format!("<div data-slot=\"kanban-card\">{t}</div>"))
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"kanban\"><div data-slot=\"kanban-column\">{cards}</div></div>")
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

    fn board(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("kanban", items.first().copied().unwrap_or("Todo"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<section"));
        assert!(!html.contains("<nav"));
        assert!(!html.contains("<pre"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("draggable"));
        assert!(!html.contains("dnd-kit"));
        assert!(!html.contains("min-width:10rem"));
        assert!(!html.contains("padding:1rem;display:flex;flex-direction:column;gap:0.5rem"));
        assert!(!html.contains("display("));
        assert!(!html.contains("kanban("));
    }

    #[test]
    fn root_is_div_of_column_and_cards_not_section() {
        let html = render(&board(&["Ship login", "Write docs"]));
        assert!(html.starts_with("<div data-slot=\"kanban\">"));
        assert_eq!(html.matches("data-slot=\"kanban-column\"").count(), 1);
        assert_eq!(html.matches("data-slot=\"kanban-card\"").count(), 2);
        assert!(html.contains("<div data-slot=\"kanban-card\">Ship login</div>"));
        assert!(html.contains("<div data-slot=\"kanban-card\">Write docs</div>"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"kanban\"><div data-slot=\"kanban-column\"><div data-slot=\"kanban-card\">Ship login</div><div data-slot=\"kanban-card\">Write docs</div></div></div>"
        );
    }

    #[test]
    fn extra_text_items_become_cards() {
        let mut c = stub("kanban", "Ship login");
        c.items.push(extra("text", "Write docs"));
        c.items.push(extra("text", "QA pass"));
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"kanban-column\"").count(), 1);
        assert_eq!(html.matches("data-slot=\"kanban-card\"").count(), 3);
        assert!(html.contains(">Ship login</div>"));
        assert!(html.contains(">Write docs</div>"));
        assert!(html.contains(">QA pass</div>"));
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_one_column_and_card() {
        let html = render(&stub("kanban", "Todo"));
        assert!(html.starts_with("<div data-slot=\"kanban\">"));
        assert_eq!(html.matches("data-slot=\"kanban-column\"").count(), 1);
        assert_eq!(html.matches("data-slot=\"kanban-card\"").count(), 1);
        assert!(html.contains("<div data-slot=\"kanban-card\">Todo</div>"));
        reject_interact(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("kanban", "A <B> & \"C\""));
        assert!(html.contains(
            "<div data-slot=\"kanban-card\">A &lt;B&gt; &amp; &quot;C&quot;</div>"
        ));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_surf_columns_and_display_section() {
        let c = board(&["Ship login", "Write docs"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("kanban", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<div data-slot=\"kanban\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("min-width:10rem"));
        assert!(!interact.contains("data-slot=\"kanban-column\""));
        assert!(!interact.contains("data-slot=\"kanban-card\""));
        assert!(html.contains("data-slot=\"kanban-column\""));
        assert!(html.contains("data-slot=\"kanban-card\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&board(&["Ship login"]));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"kanban-card\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"kanban\"]"));
        assert!(css.contains("[data-slot=\"kanban-column\"]"));
        assert!(css.contains("[data-slot=\"kanban-card\"]"));
        assert!(css.contains("overflow-x: auto"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
