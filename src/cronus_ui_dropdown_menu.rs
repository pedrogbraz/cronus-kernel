//! Dedicated DropdownMenu renderer. DOM mirrors React (`DropdownMenuTrigger
//! asChild` + `Button`): `<button data-slot="button" data-variant="primary">`
//! plus a native `popover="auto"` `<div data-slot="dropdown-menu-content"
//! role="menu">` of `dropdown-menu-item`s. No wrapper slot (React has none).
//! Zero JS: the menu is closed until `popovertarget` opens it (React's audit
//! fixture forces `defaultOpen` and portals the menu out of the canvas).
//! Not interact `popover("dropdown-menu")` (`<details>` SURF box).

use crate::cronus_ui_kit::{choice_texts, label_of, texts, widget_id};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let trigger_id = widget_id(comp, "trigger");
    let pop_id = widget_id(comp, "menu");
    let items = menu_items(comp)
        .into_iter()
        .map(|t| format!("<div data-slot=\"dropdown-menu-item\" role=\"menuitem\">{t}</div>"))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<button type=\"button\" id=\"{trigger_id}\" data-slot=\"button\" data-variant=\"primary\" popovertarget=\"{pop_id}\" aria-haspopup=\"menu\">{label}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"dropdown-menu-content\" role=\"menu\" aria-orientation=\"vertical\" anchor=\"{trigger_id}\">{items}</div>"
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
    fn trigger_is_primary_button_slot_and_menu_is_native_popover() {
        let mut c = stub("dropdown-menu", "Actions");
        c.items.push(extra("text", "Edit"));
        c.items.push(extra("text", "Delete"));
        let html = render(&c);
        assert_eq!(
            html,
            "<button type=\"button\" id=\"cui-dropdown-menu-trigger\" data-slot=\"button\" data-variant=\"primary\" popovertarget=\"cui-dropdown-menu-menu\" aria-haspopup=\"menu\">Actions</button><div id=\"cui-dropdown-menu-menu\" popover=\"auto\" data-slot=\"dropdown-menu-content\" role=\"menu\" aria-orientation=\"vertical\" anchor=\"cui-dropdown-menu-trigger\"><div data-slot=\"dropdown-menu-item\" role=\"menuitem\">Edit</div><div data-slot=\"dropdown-menu-item\" role=\"menuitem\">Delete</div></div>"
        );
        assert!(!html.contains("data-slot=\"dropdown-menu\""));
        assert!(!html.contains("data-slot=\"dropdown-menu-trigger\""));
        reject_interact(&html);
    }

    #[test]
    fn item_kinds_become_menuitems() {
        let html = render(&menu("Actions", &["Edit", "Share"]));
        assert!(html.contains(">Actions</button>"));
        assert!(html.contains("<div data-slot=\"dropdown-menu-item\" role=\"menuitem\">Edit</div>"));
        assert!(html.contains("<div data-slot=\"dropdown-menu-item\" role=\"menuitem\">Share</div>"));
        assert!(!html.contains("role=\"menuitem\">Actions"));
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_opens_empty_menu() {
        let html = render(&stub("dropdown-menu", "Actions"));
        assert!(html.contains(">Actions</button>"));
        assert!(html.contains("data-slot=\"dropdown-menu-content\" role=\"menu\""));
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
        assert!(html.contains("data-slot=\"dropdown-menu-content\""));
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
        assert!(css.contains("[data-slot=\"button\"]:has(+ [data-slot=\"dropdown-menu-content\"])"));
        assert!(css.contains("[data-slot=\"dropdown-menu-content\"]:popover-open {"));
        assert!(css.contains("[data-slot=\"dropdown-menu-item\"] {\n  position: relative; display: flex;"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("min-width: 8rem"));
        assert!(!css.contains("[data-slot=\"dropdown-menu\"] > button"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
