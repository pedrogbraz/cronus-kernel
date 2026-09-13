//! Dedicated Sidebar renderer. Desktop DOM matches React:
//! `<aside data-slot="sidebar"><div data-slot="sidebar-content"><ul data-slot="sidebar-menu">`
//! plus each text as `<li data-slot="sidebar-menu-item"><a data-slot="sidebar-menu-button">`.
//! Not interact `nav("sidebar")` (generic SURF `<nav>` without sidebar-content/menu).

use crate::cronus_ui_kit::{choice_texts, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let items = menu_items(comp)
        .into_iter()
        .map(|t| {
            format!(
                "<li data-slot=\"sidebar-menu-item\"><a data-slot=\"sidebar-menu-button\" href=\"#\">{t}</a></li>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<aside data-slot=\"sidebar\"><div data-slot=\"sidebar-content\"><ul data-slot=\"sidebar-menu\">{items}</ul></div></aside>"
    )
}

fn menu_items(comp: &ComponentNode) -> Vec<String> {
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
        let mut c = stub("sidebar", items.first().copied().unwrap_or("Home"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<nav data-slot=\"sidebar\""));
        assert!(!html.starts_with("<nav"));
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
    fn root_is_aside_with_content_and_menu_not_nav_surf() {
        let html = render(&bar(&["Home", "Settings"]));
        assert!(html.starts_with("<aside data-slot=\"sidebar\">"));
        assert!(html.contains("<div data-slot=\"sidebar-content\">"));
        assert!(html.contains("<ul data-slot=\"sidebar-menu\">"));
        assert!(html.contains(
            "<li data-slot=\"sidebar-menu-item\"><a data-slot=\"sidebar-menu-button\" href=\"#\">Home</a></li>"
        ));
        assert!(html.contains(
            "<li data-slot=\"sidebar-menu-item\"><a data-slot=\"sidebar-menu-button\" href=\"#\">Settings</a></li>"
        ));
        assert_eq!(html.matches("data-slot=\"sidebar-menu-item\"").count(), 2);
        assert_eq!(html.matches("data-slot=\"sidebar-menu-button\"").count(), 2);
        reject_interact(&html);
        assert_eq!(
            html,
            "<aside data-slot=\"sidebar\"><div data-slot=\"sidebar-content\"><ul data-slot=\"sidebar-menu\"><li data-slot=\"sidebar-menu-item\"><a data-slot=\"sidebar-menu-button\" href=\"#\">Home</a></li><li data-slot=\"sidebar-menu-item\"><a data-slot=\"sidebar-menu-button\" href=\"#\">Settings</a></li></ul></div></aside>"
        );
    }

    #[test]
    fn extra_text_items_become_menu_buttons() {
        let mut c = stub("sidebar", "App");
        c.items.push(extra("text", "Inbox"));
        c.items.push(extra("text", "Search"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"sidebar-menu-button\" href=\"#\">App</a>"));
        assert!(html.contains("data-slot=\"sidebar-menu-button\" href=\"#\">Inbox</a>"));
        assert!(html.contains("data-slot=\"sidebar-menu-button\" href=\"#\">Search</a>"));
        assert_eq!(html.matches("data-slot=\"sidebar-menu-item\"").count(), 3);
        reject_interact(&html);
    }

    #[test]
    fn field_label_is_not_a_button_when_items_exist() {
        let mut c = stub("sidebar", "Nav");
        c.items.push(extra("item", "Home"));
        c.items.push(extra("item", "Settings"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"sidebar-menu-button\" href=\"#\">Home</a>"));
        assert!(html.contains("data-slot=\"sidebar-menu-button\" href=\"#\">Settings</a>"));
        assert!(!html.contains(">Nav</a>"));
        assert_eq!(html.matches("data-slot=\"sidebar-menu-item\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_one_item() {
        let html = render(&stub("sidebar", "Home"));
        assert!(html.contains("<aside data-slot=\"sidebar\">"));
        assert!(html.contains("<div data-slot=\"sidebar-content\">"));
        assert!(html.contains("<ul data-slot=\"sidebar-menu\">"));
        assert!(html.contains(
            "<li data-slot=\"sidebar-menu-item\"><a data-slot=\"sidebar-menu-button\" href=\"#\">Home</a></li>"
        ));
        assert_eq!(html.matches("data-slot=\"sidebar-menu-item\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_nav_surf() {
        let c = bar(&["Home", "Settings"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("sidebar", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<nav data-slot=\"sidebar\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!interact.contains("data-slot=\"sidebar-content\""));
        assert!(!interact.contains("data-slot=\"sidebar-menu\""));
        assert!(!interact.contains("<aside"));
        assert!(html.contains("data-slot=\"sidebar-content\""));
        assert!(html.contains("data-slot=\"sidebar-menu\""));
        assert!(html.contains("<aside data-slot=\"sidebar\">"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&bar(&["Home"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"sidebar\"]"));
        assert!(css.contains("[data-slot=\"sidebar-content\"]"));
        assert!(css.contains("[data-slot=\"sidebar-menu\"]"));
        assert!(css.contains("[data-slot=\"sidebar-menu-item\"]"));
        assert!(css.contains("[data-slot=\"sidebar-menu-button\"]"));
        assert!(css.contains("width: 16rem"));
        assert!(css.contains("var(--cronus-surface-base)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
