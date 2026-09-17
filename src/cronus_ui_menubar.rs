//! Dedicated Menubar renderer. DOM mirrors React/Radix (`<Menubar
//! defaultValue="file">` with one `MenubarMenu`): `<div data-slot="menubar"
//! role="menubar">` with the label as the single `menubar-trigger`
//! (`data-state="open"`: Radix's active menu value) followed by a native
//! `popover="auto"` `<div data-slot="menubar-content" role="menu">` whose
//! `menubar-item`s are the extra texts. Zero JS: the menu has no layout box until
//! the trigger's `popovertarget` opens it; React portals the open menu out of the
//! canvas. Not interact `popover("menubar")` (`<details>` SURF box).
//!
//! Docs menubar: `item "New Tab" menu:"File" shortcut:"⌘T"` lines group into
//! one `MenubarMenu` per distinct `menu:` (declaration order), each a closed
//! `menubar-trigger` (`data-state="closed"`, React's idle bar) whose
//! `popovertarget` / `interestfor` opens its `menubar-content` (click or
//! hover; the open trigger is painted through `:has(+ :popover-open)`). The
//! items use the rich grammar of `cronus_ui_dropdown_menu::menu_items`
//! (shortcuts, separators, checkbox, radio group, submenu) with `menubar` slots.

use crate::cronus_ui_dropdown_menu::{menu_items, menu_items_of};
use crate::cronus_ui_kit::{esc, label_of, widget_id};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let menus: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item")
        .filter_map(|i| i.config.get("menu"))
        .map(|m| m.trim().to_string())
        .filter(|m| !m.is_empty())
        .fold(Vec::new(), |mut acc, m| {
            if !acc.contains(&m) {
                acc.push(m);
            }
            acc
        });
    if menus.is_empty() {
        let trigger = label_of(comp);
        let trigger_id = widget_id(comp, "trigger");
        let pop_id = widget_id(comp, "menu");
        let items = menu_items(comp, "menubar", &pop_id);
        return format!(
            "<div data-slot=\"menubar\" role=\"menubar\"><button type=\"button\" role=\"menuitem\" id=\"{trigger_id}\" data-slot=\"menubar-trigger\" data-state=\"open\" popovertarget=\"{pop_id}\" aria-haspopup=\"menu\">{trigger}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"menubar-content\" role=\"menu\" anchor=\"{trigger_id}\">{items}</div></div>"
        );
    }
    let body: String = menus
        .iter()
        .enumerate()
        .map(|(n, menu)| {
            let trigger_id = widget_id(comp, &format!("trigger-{}", n + 1));
            let pop_id = widget_id(comp, &format!("menu-{}", n + 1));
            let items = menu_items_of(comp, "menubar", &pop_id, |i| {
                i.config.get("menu").map(|m| m.trim()) == Some(menu.as_str())
            });
            format!(
                "<button type=\"button\" role=\"menuitem\" id=\"{trigger_id}\" data-slot=\"menubar-trigger\" data-state=\"closed\" aria-expanded=\"false\" popovertarget=\"{pop_id}\" interestfor=\"{pop_id}\" aria-haspopup=\"menu\">{}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"menubar-content\" role=\"menu\" anchor=\"{trigger_id}\">{items}</div>",
                esc(menu)
            )
        })
        .collect();
    format!("<div data-slot=\"menubar\" role=\"menubar\">{body}</div>")
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
    fn menu_config_groups_items_into_closed_menus() {
        let mut c = stub("menubar", "Menus");
        let mut tab = extra("item", "New Tab");
        tab.config.insert("menu".into(), "File".into());
        tab.config.insert("shortcut".into(), "⌘T".into());
        c.items.push(tab);
        let mut sep = extra("item", "-");
        sep.config.insert("menu".into(), "File".into());
        c.items.push(sep);
        let mut share = extra("item", "Share");
        share.config.insert("menu".into(), "File".into());
        c.items.push(share);
        let mut email = extra("item", "Email link");
        email.config.insert("menu".into(), "File".into());
        email.config.insert("sub".into(), "Share".into());
        c.items.push(email);
        let mut undo = extra("item", "Undo");
        undo.config.insert("menu".into(), "Edit".into());
        undo.config.insert("shortcut".into(), "⌘Z".into());
        c.items.push(undo);
        let mut full = extra("item", "Always Show Full URLs");
        full.config.insert("menu".into(), "View".into());
        full.config.insert("type".into(), "checkbox".into());
        full.config.insert("checked".into(), "true".into());
        c.items.push(full);
        let mut benoit = extra("item", "Benoit");
        benoit.config.insert("menu".into(), "View".into());
        benoit.config.insert("type".into(), "radio".into());
        benoit.config.insert("checked".into(), "true".into());
        c.items.push(benoit);
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"menubar\" role=\"menubar\"><button type=\"button\" role=\"menuitem\" id=\"cui-menubar-trigger-1\" data-slot=\"menubar-trigger\" data-state=\"closed\" aria-expanded=\"false\" popovertarget=\"cui-menubar-menu-1\" interestfor=\"cui-menubar-menu-1\" aria-haspopup=\"menu\">File</button><div id=\"cui-menubar-menu-1\" popover=\"auto\" data-slot=\"menubar-content\" role=\"menu\" anchor=\"cui-menubar-trigger-1\"><div data-slot=\"menubar-item\" role=\"menuitem\">New Tab<span data-slot=\"menubar-shortcut\">⌘T</span></div><div data-slot=\"menubar-separator\" role=\"separator\" aria-orientation=\"horizontal\"></div><button type=\"button\" id=\"cui-menubar-menu-1-s1-trigger\" data-slot=\"menubar-sub-trigger\""), "{html}");
        assert!(html.contains("data-slot=\"menubar-sub-content\" role=\"menu\" aria-orientation=\"vertical\" anchor=\"cui-menubar-menu-1-s1-trigger\"><div data-slot=\"menubar-item\" role=\"menuitem\">Email link</div></div></div>"));
        assert_eq!(html.matches("data-slot=\"menubar-trigger\"").count(), 3);
        assert!(html.contains(">Edit</button>"));
        assert!(html.contains(">View</button><div id=\"cui-menubar-menu-3\""));
        assert!(html.contains("<label><input type=\"checkbox\" aria-label=\"Always Show Full URLs\" checked><div data-slot=\"menubar-checkbox-item\" role=\"menuitemcheckbox\""));
        assert!(html.contains("<div data-slot=\"menubar-radio-group\" role=\"group\"><label><input type=\"radio\" name=\"cui-menubar-menu-3-r1\" aria-label=\"Benoit\" checked><div data-slot=\"menubar-radio-item\" role=\"menuitemradio\""));
        assert!(!html.contains("data-slot=\"menubar-label\""));
        assert!(!html.contains(">Menus<"));
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
        // Docs bar: the trigger whose menu is open is painted; content pops in.
        assert!(css.contains(
            "[data-slot=\"menubar-trigger\"]:has(+ [data-slot=\"menubar-content\"]:popover-open)"
        ));
        assert!(css.contains(
            "[data-slot=\"menubar-content\"]:popover-open {\n  z-index: 50; min-width: 12rem;"
        ));
        assert!(css.contains("animation: cronus-pop-in 180ms var(--ease-out-quart) both;"));
        assert!(css.contains("[data-slot=\"menubar-shortcut\"] {\n  margin-inline-start: auto;"));
        assert!(css.contains(
            "[data-slot=\"menubar-separator\"] {\n  margin: 0.25rem -0.25rem; height: 1px;"
        ));
        assert!(css.contains("[data-slot=\"menubar-content\"] label:has(> input:checked) > [data-slot=\"menubar-checkbox-item\"] > span > svg,"));
        assert!(css.contains("[data-slot=\"menubar-sub-content\"]:popover-open {"));
    }
}
