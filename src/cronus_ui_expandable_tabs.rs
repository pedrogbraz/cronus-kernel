//! Dedicated ExpandableTabs renderer. DOM matches React:
//! `<div data-slot="expandable-tabs">` plus each text as
//! `<button type="button" data-slot="expandable-tabs-item">`. First item is current.
//! Not interact `tabs()` (generic tablist without expandable-tabs-item).

use crate::cronus_ui_kit::{choice_texts, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let items = {
        let c = choice_texts(comp);
        if c.is_empty() {
            texts(comp)
        } else {
            c
        }
    };
    let buttons = items
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let selected = if i == 0 { "true" } else { "false" };
            format!(
                "<button type=\"button\" role=\"tab\" data-slot=\"expandable-tabs-item\" aria-selected=\"{selected}\">{t}</button>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"expandable-tabs\" role=\"tablist\">{buttons}</div>")
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
        let mut c = stub("expandable-tabs", items.first().copied().unwrap_or("Home"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<div role=\"tablist\""));
        assert!(!html.contains("role=\"tabpanel\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("v-show"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_is_div_with_items_first_current_not_generic_tablist() {
        let html = render(&bar(&["Home", "Search"]));
        assert!(html.starts_with("<div data-slot=\"expandable-tabs\""));
        assert!(html.contains("data-slot=\"expandable-tabs\" role=\"tablist\""));
        assert!(html.contains(
            "<button type=\"button\" role=\"tab\" data-slot=\"expandable-tabs-item\" aria-selected=\"true\">Home</button>"
        ));
        assert!(html.contains(
            "<button type=\"button\" role=\"tab\" data-slot=\"expandable-tabs-item\" aria-selected=\"false\">Search</button>"
        ));
        assert_eq!(html.matches("data-slot=\"expandable-tabs-item\"").count(), 2);
        assert_eq!(html.matches("aria-selected=\"true\"").count(), 1);
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"expandable-tabs\" role=\"tablist\"><button type=\"button\" role=\"tab\" data-slot=\"expandable-tabs-item\" aria-selected=\"true\">Home</button><button type=\"button\" role=\"tab\" data-slot=\"expandable-tabs-item\" aria-selected=\"false\">Search</button></div>"
        );
    }

    #[test]
    fn extra_text_items_become_items() {
        let mut c = stub("expandable-tabs", "Home");
        c.items.push(extra("text", "Search"));
        c.items.push(extra("text", "Profile"));
        let html = render(&c);
        assert!(html.contains("aria-selected=\"true\">Home</button>"));
        assert!(html.contains("data-slot=\"expandable-tabs-item\" aria-selected=\"false\">Search</button>"));
        assert!(html.contains("data-slot=\"expandable-tabs-item\" aria-selected=\"false\">Profile</button>"));
        assert_eq!(html.matches("data-slot=\"expandable-tabs-item\"").count(), 3);
        reject_interact(&html);
    }

    #[test]
    fn field_label_is_not_an_item_when_items_exist() {
        let mut c = stub("expandable-tabs", "Menus");
        c.items.push(extra("item", "Home"));
        c.items.push(extra("item", "Search"));
        let html = render(&c);
        assert!(html.contains("aria-selected=\"true\">Home</button>"));
        assert!(html.contains("data-slot=\"expandable-tabs-item\" aria-selected=\"false\">Search</button>"));
        assert!(!html.contains(">Menus</button>"));
        assert_eq!(html.matches("data-slot=\"expandable-tabs-item\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_current_item() {
        let html = render(&stub("expandable-tabs", "Home"));
        assert!(html.contains("<div data-slot=\"expandable-tabs\""));
        assert!(html.contains(
            "<button type=\"button\" role=\"tab\" data-slot=\"expandable-tabs-item\" aria-selected=\"true\">Home</button>"
        ));
        assert_eq!(html.matches("data-slot=\"expandable-tabs-item\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_tabs_generic_tablist() {
        let c = bar(&["Home", "Search"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("expandable-tabs", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<div data-slot=\"expandable-tabs\""));
        assert!(interact.contains("<div role=\"tablist\""));
        assert!(interact.contains("role=\"tabpanel\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("onclick="));
        assert!(!interact.contains("data-slot=\"expandable-tabs-item\""));
        assert!(html.contains("data-slot=\"expandable-tabs-item\""));
        assert!(html.contains("aria-selected=\"true\""));
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
        assert!(css.contains("[data-slot=\"expandable-tabs\"]"));
        assert!(css.contains("[data-slot=\"expandable-tabs-item\"]"));
        assert!(css.contains("[data-slot=\"expandable-tabs-item\"][aria-selected=\"true\"]"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("border-radius: 9999px"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
