//! Dedicated ContextMenu renderer (wave1t geometry parity with Radix ContextMenu).
//!
//! React: the trigger is a bare `<span>` (no data-slot) and the open content is
//! `<div data-slot="context-menu-content" role="menu">` (min-w 8rem, p-1, radius lg,
//! border, surface-floating) with one `<div data-slot="context-menu-item"
//! role="menuitem">` per item (px-2 py-1.5, text-sm, radius md).
//! Audit cannot right-click and the kernel has no popper, so the content is
//! always open, in flow under the trigger (React places it at the pointer; the
//! geometry spec measures it against itself). Zero JS.
//!
//! Closed mode (`trigger:"…"` prop, or `open:false` / `defaultOpen:false`; see
//! `cronus_ui_kit::overlay_trigger`): the span becomes an outline `button`
//! (label or `trigger` text) whose `popovertarget` opens the menu as a native
//! `popover="auto"` anchored under it; `trigger-variant:area` renders it as
//! the docs' dashed target region instead (React's trigger is a slotless
//! `<span>`, so the button carries only an `area` class). Gaps: opening on
//! right-click needs a `contextmenu` listener (JS), so the menu opens on click
//! / Enter / Space; menu items perform no action.
//! Rich `item` lines (icons, shortcuts, separators, checkbox, radio group,
//! submenu) share `cronus_ui_dropdown_menu::menu_items` with `context-menu`
//! slots; `width:56` is a `w-56` class on the content.
//! Not interact `popover("context-menu")` (`<details>` SURF box).

use crate::cronus_ui_dropdown_menu::{menu_items, width_class};
use crate::cronus_ui_kit::{attr, label_of, overlay_trigger, widget_id};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let mut listed = comp.clone();
    listed.items.retain(|i| i.item_type != "trigger");
    let pop_id = widget_id(comp, "menu");
    let items = menu_items(&listed, "context-menu", &pop_id);
    let class = width_class(comp);
    match overlay_trigger(comp, &label) {
        None => format!(
            "<span>{label}</span><div data-slot=\"context-menu-content\" role=\"menu\" aria-orientation=\"vertical\"{class}>{items}</div>"
        ),
        Some(trigger) => {
            let trigger_id = widget_id(comp, "trigger");
            let button = if attr(comp, "trigger-variant").map(str::trim) == Some("area") {
                format!(
                    "<button type=\"button\" id=\"{trigger_id}\" class=\"area\" popovertarget=\"{pop_id}\" aria-haspopup=\"menu\">{trigger}</button>"
                )
            } else {
                format!(
                    "<button type=\"button\" id=\"{trigger_id}\" data-slot=\"button\" data-variant=\"outline\" popovertarget=\"{pop_id}\" aria-haspopup=\"menu\">{trigger}</button>"
                )
            };
            format!(
                "{button}<div id=\"{pop_id}\" popover=\"auto\" data-slot=\"context-menu-content\" role=\"menu\" aria-orientation=\"vertical\" anchor=\"{trigger_id}\"{class}>{items}</div>"
            )
        }
    }
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
        let mut c = stub("context-menu", label);
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
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("<button"));
        assert!(!html.contains("data-slot=\"context-menu\">"));
    }

    #[test]
    fn react_dom_span_trigger_and_open_menu() {
        let html = render(&menu("Right click", &["Back", "Reload"]));
        assert_eq!(
            html,
            "<span>Right click</span><div data-slot=\"context-menu-content\" role=\"menu\" aria-orientation=\"vertical\"><div data-slot=\"context-menu-item\" role=\"menuitem\">Back</div><div data-slot=\"context-menu-item\" role=\"menuitem\">Reload</div></div>"
        );
        reject_interact(&html);
    }

    #[test]
    fn open_false_renders_click_trigger_and_native_popover_menu() {
        let mut c = stub("context-menu", "Right click");
        c.items.push(extra("text", "Back"));
        c.items.push(extra("text", "Reload"));
        c.props.insert("open".into(), "false".into());
        let html = render(&c);
        let tid = crate::cronus_ui_kit::widget_id(&c, "trigger");
        let pid = crate::cronus_ui_kit::widget_id(&c, "menu");
        assert_eq!(
            html,
            format!(
                "<button type=\"button\" id=\"{tid}\" data-slot=\"button\" data-variant=\"outline\" popovertarget=\"{pid}\" aria-haspopup=\"menu\">Right click</button><div id=\"{pid}\" popover=\"auto\" data-slot=\"context-menu-content\" role=\"menu\" aria-orientation=\"vertical\" anchor=\"{tid}\"><div data-slot=\"context-menu-item\" role=\"menuitem\">Back</div><div data-slot=\"context-menu-item\" role=\"menuitem\">Reload</div></div>"
            )
        );
    }

    #[test]
    fn trigger_item_labels_trigger_and_is_not_a_menu_item() {
        let mut c = stub("context-menu", "Surface");
        c.items.push(extra("text", "Cut"));
        c.items.push(extra("trigger", "Actions"));
        let html = render(&c);
        assert!(html.contains("aria-haspopup=\"menu\">Actions</button>"));
        assert_eq!(html.matches("role=\"menuitem\"").count(), 1);
        assert!(!html.contains("role=\"menuitem\">Actions"));
    }

    #[test]
    fn chrome_closed_mode_hides_until_open() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"context-menu-content\"][popover]:not(:popover-open) { display: none; }"
        ));
        assert!(css.contains("[data-slot=\"context-menu-content\"][popover]:popover-open {\n  margin: 0;\n  animation: cronus-pop-in 180ms var(--ease-out-quart) both;"));
        assert!(css.contains("button.area:has(+ [data-slot=\"context-menu-content\"]) {\n  display: flex; height: 9rem; width: 100%; max-width: 24rem;"));
        assert!(css.contains(
            "[data-slot=\"context-menu-content\"].w-56 { min-width: 14rem; max-width: 14rem; }"
        ));
    }

    #[test]
    fn docs_surface_has_area_trigger_and_rich_items() {
        let mut c = stub("context-menu", "Right-click here");
        c.props.insert("trigger".into(), "Right-click here".into());
        c.props.insert("trigger-variant".into(), "area".into());
        c.props.insert("width".into(), "56".into());
        let mut cut = extra("item", "Cut");
        cut.config.insert("icon".into(), "scissors".into());
        cut.config.insert("shortcut".into(), "⌘X".into());
        c.items.push(cut);
        c.items.push(extra("item", "-"));
        let mut bookmarked = extra("item", "Bookmarked");
        bookmarked.config.insert("type".into(), "checkbox".into());
        bookmarked.config.insert("checked".into(), "true".into());
        c.items.push(bookmarked);
        let mut share = extra("item", "Share");
        share.config.insert("icon".into(), "users".into());
        c.items.push(share);
        let mut invite = extra("item", "Invite people");
        invite.config.insert("sub".into(), "Share".into());
        c.items.push(invite);
        let mut branch = extra("item", "Branch");
        branch.config.insert("type".into(), "label".into());
        c.items.push(branch);
        let mut main = extra("item", "main");
        main.config.insert("type".into(), "radio".into());
        main.config.insert("checked".into(), "true".into());
        c.items.push(main);
        let mut dev = extra("item", "develop");
        dev.config.insert("type".into(), "radio".into());
        c.items.push(dev);
        let html = render(&c);
        let tid = crate::cronus_ui_kit::widget_id(&c, "trigger");
        let pid = crate::cronus_ui_kit::widget_id(&c, "menu");
        assert!(html.starts_with(&format!(
            "<button type=\"button\" id=\"{tid}\" class=\"area\" popovertarget=\"{pid}\" aria-haspopup=\"menu\">Right-click here</button><div id=\"{pid}\" popover=\"auto\" data-slot=\"context-menu-content\" role=\"menu\" aria-orientation=\"vertical\" anchor=\"{tid}\" class=\"w-56\"><div data-slot=\"context-menu-item\" role=\"menuitem\"><svg "
        )), "{html}");
        assert!(html.contains("</svg>Cut<span data-slot=\"context-menu-shortcut\">⌘X</span></div><div data-slot=\"context-menu-separator\" role=\"separator\" aria-orientation=\"horizontal\"></div><label><input type=\"checkbox\" aria-label=\"Bookmarked\" checked><div data-slot=\"context-menu-checkbox-item\" role=\"menuitemcheckbox\""));
        assert!(html.contains(&format!(
            "data-slot=\"context-menu-sub-trigger\" role=\"menuitem\" aria-haspopup=\"menu\" aria-expanded=\"false\" popovertarget=\"{pid}-s1\" interestfor=\"{pid}-s1\"><svg "
        )));
        assert!(html.contains(
            "data-slot=\"context-menu-sub-content\" role=\"menu\" aria-orientation=\"vertical\""
        ));
        assert!(html.contains("<div data-slot=\"context-menu-label\">Branch</div><div data-slot=\"context-menu-radio-group\" role=\"group\"><label><input type=\"radio\" name=\""));
        assert!(html.contains("<div data-slot=\"context-menu-radio-item\" role=\"menuitemradio\" aria-hidden=\"true\" tabindex=\"-1\"><span><svg "));
        assert!(!html.contains("data-slot=\"button\""));
    }

    #[test]
    fn extra_text_items_become_menuitems() {
        let mut c = stub("context-menu", "Right click");
        c.items.push(extra("text", "Back"));
        c.items.push(extra("text", "Reload"));
        let html = render(&c);
        assert!(html.starts_with("<span>Right click</span>"));
        assert!(html.contains("role=\"menuitem\">Back</div>"));
        assert!(html.contains("role=\"menuitem\">Reload</div>"));
        assert_eq!(html.matches("role=\"menuitem\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_opens_empty_menu() {
        let html = render(&stub("context-menu", "Surface"));
        assert!(html.contains("<div data-slot=\"context-menu-content\" role=\"menu\" aria-orientation=\"vertical\"></div>"));
        assert!(!html.contains("role=\"menuitem\""));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_popover_surf() {
        let c = menu("Surface", &["Cut"]);
        let html = render(&c);
        assert!(html.contains("data-slot=\"context-menu-content\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            reject_interact(&render(&menu("Surface", &["Cut"])));
        });
    }

    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        let block = |sel: &str| {
            let start = css
                .find(&format!("{sel} {{"))
                .unwrap_or_else(|| panic!("{sel}"));
            let end = start + css[start..].find('}').unwrap();
            css[start..end].to_string()
        };
        let content = block("[data-slot=\"context-menu-content\"]");
        assert!(content.contains("width: max-content; min-width: 8rem;"));
        assert!(content.contains("padding: 0.25rem;"));
        assert!(content.contains("border: 1px solid var(--cronus-border)"));
        assert!(content.contains("border-radius: var(--cronus-radius-lg)"));
        assert!(content.contains("background: var(--cronus-surface-floating)"));
        let item = block("[data-slot=\"context-menu-item\"]");
        assert!(item.contains("padding: 0.375rem 0.5rem;"));
        assert!(item.contains("border-radius: var(--cronus-radius-md)"));
        assert!(item.contains("font-size: 0.875rem; line-height: 1.25rem;"));
        assert!(!css.contains("button:has(+ [data-slot=\"context-menu-content\"])"));
        assert!(!css.contains("zinc-"));
    }
}
