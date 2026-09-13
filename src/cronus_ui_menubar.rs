//! Dedicated Menubar renderer. DOM matches React/Radix:
//! `<div data-slot="menubar" role="menubar">` plus each text as
//! `<button data-slot="menubar-trigger">` and always-open first
//! `<div data-slot="menubar-content" role="menu">` with a menubar-item.
//! Not interact `popover("menubar")` (`<details>` SURF box).

use crate::cronus_ui_kit::{choice_texts, label_of, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let triggers = menu_triggers(comp);
    let first = triggers.first().cloned().unwrap_or_else(|| label_of(comp));
    let first_btn = format!(
        "<button type=\"button\" data-slot=\"menubar-trigger\" data-state=\"open\">{first}</button>"
    );
    let content = format!(
        "<div data-slot=\"menubar-content\" role=\"menu\"><div data-slot=\"menubar-item\" role=\"menuitem\">{first}</div></div>"
    );
    let others = triggers
        .iter()
        .skip(1)
        .map(|t| format!("<button type=\"button\" data-slot=\"menubar-trigger\">{t}</button>"))
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"menubar\" role=\"menubar\">{first_btn}{content}{others}</div>")
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
        let mut c = stub("menubar", items.first().copied().unwrap_or("File"));
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
        assert!(!html.contains("position:absolute;z-index:20"));
        assert!(!html.contains("role=\"dialog\""));
        assert!(!html.contains("max-width:28rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("popover("));
    }

    #[test]
    fn root_is_menubar_with_always_open_first_menu() {
        let html = render(&bar(&["File", "Edit"]));
        assert!(html.starts_with("<div data-slot=\"menubar\" role=\"menubar\">"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"menubar-trigger\" data-state=\"open\">File</button>"
        ));
        assert!(html.contains("<div data-slot=\"menubar-content\" role=\"menu\">"));
        assert!(html.contains(
            "<div data-slot=\"menubar-item\" role=\"menuitem\">File</div>"
        ));
        assert!(html.contains("<button type=\"button\" data-slot=\"menubar-trigger\">Edit</button>"));
        assert_eq!(html.matches("data-slot=\"menubar-trigger\"").count(), 2);
        assert_eq!(html.matches("data-slot=\"menubar-content\"").count(), 1);
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"menubar\" role=\"menubar\"><button type=\"button\" data-slot=\"menubar-trigger\" data-state=\"open\">File</button><div data-slot=\"menubar-content\" role=\"menu\"><div data-slot=\"menubar-item\" role=\"menuitem\">File</div></div><button type=\"button\" data-slot=\"menubar-trigger\">Edit</button></div>"
        );
    }

    #[test]
    fn extra_text_items_become_triggers() {
        let mut c = stub("menubar", "File");
        c.items.push(extra("text", "Edit"));
        c.items.push(extra("text", "View"));
        let html = render(&c);
        assert!(html.contains("data-state=\"open\">File</button>"));
        assert!(html.contains("data-slot=\"menubar-trigger\">Edit</button>"));
        assert!(html.contains("data-slot=\"menubar-trigger\">View</button>"));
        assert_eq!(html.matches("data-slot=\"menubar-trigger\"").count(), 3);
        assert_eq!(html.matches("data-slot=\"menubar-content\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_opens_first_menu() {
        let html = render(&stub("menubar", "File"));
        assert!(html.contains("<div data-slot=\"menubar\" role=\"menubar\">"));
        assert!(html.contains("data-state=\"open\">File</button>"));
        assert!(html.contains("<div data-slot=\"menubar-content\" role=\"menu\">"));
        assert!(html.contains(
            "<div data-slot=\"menubar-item\" role=\"menuitem\">File</div>"
        ));
        assert_eq!(html.matches("data-slot=\"menubar-trigger\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn field_label_is_not_a_trigger_when_items_exist() {
        let mut c = stub("menubar", "Menus");
        c.items.push(extra("item", "File"));
        c.items.push(extra("item", "Edit"));
        let html = render(&c);
        assert!(html.contains("data-state=\"open\">File</button>"));
        assert!(html.contains("data-slot=\"menubar-trigger\">Edit</button>"));
        assert!(!html.contains(">Menus</button>"));
        assert_eq!(html.matches("data-slot=\"menubar-trigger\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_popover_surf() {
        let c = bar(&["File", "Edit"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("menubar", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<details data-slot=\"menubar\""));
        assert!(interact.contains("<summary"));
        assert!(interact.contains("style="));
        assert!(interact.contains("position:absolute;z-index:20"));
        assert!(!interact.contains("data-slot=\"menubar-trigger\""));
        assert!(!interact.contains("data-slot=\"menubar-content\""));
        assert!(html.contains("data-slot=\"menubar-trigger\""));
        assert!(html.contains("data-slot=\"menubar-content\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&bar(&["File"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"menubar\"]"));
        assert!(css.contains("[data-slot=\"menubar-trigger\"]"));
        assert!(css.contains("[data-slot=\"menubar-content\"]"));
        assert!(css.contains("[data-slot=\"menubar-item\"]"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("min-width: 12rem"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
