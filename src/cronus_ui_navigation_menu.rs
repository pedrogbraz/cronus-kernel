//! Dedicated NavigationMenu renderer. DOM matches React/Radix:
//! `<nav data-slot="navigation-menu"><ul data-slot="navigation-menu-list">`
//! plus each text as `<li data-slot="navigation-menu-item">` with a
//! `<button data-slot="navigation-menu-trigger">`. First item is always-open
//! with `<div data-slot="navigation-menu-content">`.
//! Not interact `nav("navigation-menu")` (generic SURF `<nav>`).

use crate::cronus_ui_kit::{choice_texts, label_of, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let triggers = menu_triggers(comp);
    let first = triggers.first().cloned().unwrap_or_else(|| label_of(comp));
    let first_item = format!(
        "<li data-slot=\"navigation-menu-item\"><button type=\"button\" data-slot=\"navigation-menu-trigger\" data-state=\"open\">{first}</button><div data-slot=\"navigation-menu-content\">{first}</div></li>"
    );
    let others = triggers
        .iter()
        .skip(1)
        .map(|t| {
            format!(
                "<li data-slot=\"navigation-menu-item\"><button type=\"button\" data-slot=\"navigation-menu-trigger\">{t}</button></li>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<nav data-slot=\"navigation-menu\"><ul data-slot=\"navigation-menu-list\">{first_item}{others}</ul></nav>"
    )
}

fn menu_triggers(comp: &ComponentNode) -> Vec<String> {
    let choices = choice_texts(comp);
    if !choices.is_empty() {
        return choices;
    }
    texts(comp)
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

    fn bar(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("navigation-menu", items.first().copied().unwrap_or("Products"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<details"));
        assert!(!html.contains("<summary"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("nav("));
    }

    #[test]
    fn root_is_nav_list_with_open_first_content() {
        let html = render(&bar(&["Products", "Docs"]));
        assert!(html.starts_with("<nav data-slot=\"navigation-menu\">"));
        assert!(html.contains("<ul data-slot=\"navigation-menu-list\">"));
        assert!(html.contains("<li data-slot=\"navigation-menu-item\">"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"navigation-menu-trigger\" data-state=\"open\">Products</button>"
        ));
        assert!(html.contains("<div data-slot=\"navigation-menu-content\">Products</div>"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"navigation-menu-trigger\">Docs</button>"
        ));
        assert_eq!(html.matches("data-slot=\"navigation-menu-item\"").count(), 2);
        assert_eq!(html.matches("data-slot=\"navigation-menu-trigger\"").count(), 2);
        assert_eq!(html.matches("data-slot=\"navigation-menu-content\"").count(), 1);
        reject_interact(&html);
        assert_eq!(
            html,
            "<nav data-slot=\"navigation-menu\"><ul data-slot=\"navigation-menu-list\"><li data-slot=\"navigation-menu-item\"><button type=\"button\" data-slot=\"navigation-menu-trigger\" data-state=\"open\">Products</button><div data-slot=\"navigation-menu-content\">Products</div></li><li data-slot=\"navigation-menu-item\"><button type=\"button\" data-slot=\"navigation-menu-trigger\">Docs</button></li></ul></nav>"
        );
    }

    #[test]
    fn extra_text_items_become_triggers() {
        let mut c = stub("navigation-menu", "Products");
        c.items.push(extra("text", "Docs"));
        c.items.push(extra("text", "Blog"));
        let html = render(&c);
        assert!(html.contains("data-state=\"open\">Products</button>"));
        assert!(html.contains("data-slot=\"navigation-menu-trigger\">Docs</button>"));
        assert!(html.contains("data-slot=\"navigation-menu-trigger\">Blog</button>"));
        assert_eq!(html.matches("data-slot=\"navigation-menu-trigger\"").count(), 3);
        assert_eq!(html.matches("data-slot=\"navigation-menu-content\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn field_label_is_not_a_trigger_when_items_exist() {
        let mut c = stub("navigation-menu", "Menus");
        c.items.push(extra("item", "Products"));
        c.items.push(extra("item", "Docs"));
        let html = render(&c);
        assert!(html.contains("data-state=\"open\">Products</button>"));
        assert!(html.contains("data-slot=\"navigation-menu-trigger\">Docs</button>"));
        assert!(!html.contains(">Menus</button>"));
        assert_eq!(html.matches("data-slot=\"navigation-menu-trigger\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_opens_first_content() {
        let html = render(&stub("navigation-menu", "Products"));
        assert!(html.contains("<nav data-slot=\"navigation-menu\">"));
        assert!(html.contains("<ul data-slot=\"navigation-menu-list\">"));
        assert!(html.contains("data-state=\"open\">Products</button>"));
        assert!(html.contains("<div data-slot=\"navigation-menu-content\">Products</div>"));
        assert_eq!(html.matches("data-slot=\"navigation-menu-trigger\"").count(), 1);
        assert_eq!(html.matches("data-slot=\"navigation-menu-content\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_nav_surf() {
        let c = bar(&["Products", "Docs"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("navigation-menu", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<nav data-slot=\"navigation-menu\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!interact.contains("data-slot=\"navigation-menu-list\""));
        assert!(!interact.contains("data-slot=\"navigation-menu-trigger\""));
        assert!(!interact.contains("data-slot=\"navigation-menu-content\""));
        assert!(html.contains("data-slot=\"navigation-menu-list\""));
        assert!(html.contains("data-slot=\"navigation-menu-trigger\""));
        assert!(html.contains("data-slot=\"navigation-menu-content\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&bar(&["Products"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"navigation-menu\"]"));
        assert!(css.contains("[data-slot=\"navigation-menu-list\"]"));
        assert!(css.contains("[data-slot=\"navigation-menu-item\"]"));
        assert!(css.contains("[data-slot=\"navigation-menu-trigger\"]"));
        assert!(css.contains("[data-slot=\"navigation-menu-content\"]"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("min-width: 12rem"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
