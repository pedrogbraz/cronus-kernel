//! Dedicated Menubar renderer. DOM mirrors React/Radix (`<Menubar
//! defaultValue="file">` with one `MenubarMenu`): `<div data-slot="menubar"
//! role="menubar">` with the label as the single `menubar-trigger`
//! (`data-state="open"`: Radix's active menu value) followed by a native
//! `popover="auto"` `<div data-slot="menubar-content" role="menu">` whose
//! `menubar-item`s are the extra texts. Zero JS: the menu has no layout box until
//! the trigger's `popovertarget` opens it; React portals the open menu out of the
//! canvas. Not interact `popover("menubar")` (`<details>` SURF box).

use crate::cronus_ui_kit::{choice_texts, label_of, texts, widget_id};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let trigger = label_of(comp);
    let trigger_id = widget_id(comp, "trigger");
    let pop_id = widget_id(comp, "menu");
    let items = menu_items(comp)
        .into_iter()
        .map(|t| format!("<div data-slot=\"menubar-item\" role=\"menuitem\">{t}</div>"))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"menubar\" role=\"menubar\"><button type=\"button\" role=\"menuitem\" id=\"{trigger_id}\" data-slot=\"menubar-trigger\" data-state=\"open\" popovertarget=\"{pop_id}\" aria-haspopup=\"menu\">{trigger}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"menubar-content\" role=\"menu\" anchor=\"{trigger_id}\">{items}</div></div>"
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
    fn label_is_the_single_trigger_and_texts_are_menu_items() {
        let mut c = stub("menubar", "File");
        c.items.push(extra("text", "New Tab"));
        c.items.push(extra("text", "Open"));
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"menubar\" role=\"menubar\"><button type=\"button\" role=\"menuitem\" id=\"cui-menubar-trigger\" data-slot=\"menubar-trigger\" data-state=\"open\" popovertarget=\"cui-menubar-menu\" aria-haspopup=\"menu\">File</button><div id=\"cui-menubar-menu\" popover=\"auto\" data-slot=\"menubar-content\" role=\"menu\" anchor=\"cui-menubar-trigger\"><div data-slot=\"menubar-item\" role=\"menuitem\">New Tab</div><div data-slot=\"menubar-item\" role=\"menuitem\">Open</div></div></div>"
        );
        assert_eq!(html.matches("data-slot=\"menubar-trigger\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn item_kinds_become_menu_items_not_triggers() {
        let mut c = stub("menubar", "File");
        c.items.push(extra("item", "New"));
        c.items.push(extra("item", "Open"));
        let html = render(&c);
        assert!(html.contains("data-state=\"open\" popovertarget=\"cui-menubar-menu\" aria-haspopup=\"menu\">File</button>"));
        assert!(html.contains("role=\"menuitem\">New</div>"));
        assert!(html.contains("role=\"menuitem\">Open</div>"));
        assert_eq!(html.matches("data-slot=\"menubar-trigger\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn label_only_keeps_empty_menu() {
        let html = render(&stub("menubar", "File"));
        assert!(html.contains(">File</button>"));
        assert!(html.contains("data-slot=\"menubar-content\" role=\"menu\""));
        assert!(!html.contains("data-slot=\"menubar-item\""));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_popover_surf() {
        let mut c = stub("menubar", "File");
        c.items.push(extra("item", "Edit"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"menubar-content\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("menubar", "File"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"menubar\"]"));
        assert!(css.contains(
            "[data-slot=\"menubar-trigger\"] {\n  display: flex; align-items: center; border: 0;"
        ));
        assert!(css.contains("font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;"));
        assert!(
            css.contains("[data-slot=\"menubar-content\"]:not(:popover-open) { display: none; }")
        );
        assert!(css.contains("[data-slot=\"menubar-item\"]"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("min-width: 12rem"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
