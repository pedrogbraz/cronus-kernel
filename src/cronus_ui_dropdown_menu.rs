//! Dedicated DropdownMenu renderer. Always-open static: trigger button +
//! `<div data-slot="dropdown-menu-content" role="menu">` with each text item
//! `<div data-slot="dropdown-menu-item" role="menuitem">`.
//! Not interact `popover("dropdown-menu")` (`<details>` SURF box).

use crate::cronus_ui_kit::{choice_texts, label_of, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let items = menu_items(comp)
        .into_iter()
        .map(|t| format!("<div data-slot=\"dropdown-menu-item\" role=\"menuitem\">{t}</div>"))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"dropdown-menu\"><button type=\"button\">{label}</button><div data-slot=\"dropdown-menu-content\" role=\"menu\">{items}</div></div>"
    )
}

fn menu_items(comp: &ComponentNode) -> Vec<String> {
    let choices = choice_texts(comp);
    if !choices.is_empty() {
        return choices;
    }
    texts(comp).into_iter().skip(1).collect()
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

    fn menu(label: &str, items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("dropdown-menu", label);
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<details"));
        assert!(!html.contains("<summary"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("position:absolute;z-index:20"));
        assert!(!html.contains("role=\"dialog\""));
        assert!(!html.contains("max-width:28rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("popover("));
    }

    #[test]
    fn root_is_always_open_menu_not_popover_details() {
        let html = render(&menu("Actions", &["Edit", "Share"]));
        assert!(html.starts_with("<div data-slot=\"dropdown-menu\">"));
        assert!(html.contains("<button type=\"button\">Actions</button>"));
        assert!(html.contains("<div data-slot=\"dropdown-menu-content\" role=\"menu\">"));
        assert!(html.contains("<div data-slot=\"dropdown-menu-item\" role=\"menuitem\">Edit</div>"));
        assert!(html.contains("<div data-slot=\"dropdown-menu-item\" role=\"menuitem\">Share</div>"));
        assert!(!html.contains("role=\"menuitem\">Actions"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"dropdown-menu\"><button type=\"button\">Actions</button><div data-slot=\"dropdown-menu-content\" role=\"menu\"><div data-slot=\"dropdown-menu-item\" role=\"menuitem\">Edit</div><div data-slot=\"dropdown-menu-item\" role=\"menuitem\">Share</div></div></div>"
        );
    }

    #[test]
    fn extra_text_items_become_menuitems() {
        let mut c = stub("dropdown-menu", "File");
        c.items.push(extra("text", "New"));
        c.items.push(extra("text", "Open"));
        let html = render(&c);
        assert!(html.contains("<button type=\"button\">File</button>"));
        assert!(html.contains("role=\"menuitem\">New</div>"));
        assert!(html.contains("role=\"menuitem\">Open</div>"));
        assert_eq!(html.matches("role=\"menuitem\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_opens_empty_menu() {
        let html = render(&stub("dropdown-menu", "Actions"));
        assert!(html.contains("<button type=\"button\">Actions</button>"));
        assert!(html.contains("<div data-slot=\"dropdown-menu-content\" role=\"menu\"></div>"));
        assert!(!html.contains("role=\"menuitem\""));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_popover_surf() {
        let c = menu("Actions", &["Edit"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("dropdown-menu", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<details data-slot=\"dropdown-menu\""));
        assert!(interact.contains("<summary"));
        assert!(interact.contains("style="));
        assert!(interact.contains("position:absolute;z-index:20"));
        assert!(!interact.contains("data-slot=\"dropdown-menu-content\""));
        assert!(!interact.contains("role=\"menu\""));
        assert!(html.contains("data-slot=\"dropdown-menu-content\""));
        assert!(html.contains("role=\"menu\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&menu("Actions", &["Edit"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"dropdown-menu\"]"));
        assert!(css.contains("[data-slot=\"dropdown-menu-content\"]"));
        assert!(css.contains("[data-slot=\"dropdown-menu-item\"]"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("min-width: 8rem"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
