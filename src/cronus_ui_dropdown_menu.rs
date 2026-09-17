//! Dedicated DropdownMenu renderer. DOM mirrors React (`DropdownMenuTrigger
//! asChild` + `Button`): `<button data-slot="button" data-variant="primary">`
//! plus a native `popover="auto"` `<div data-slot="dropdown-menu-content"
//! role="menu">` of `dropdown-menu-item`s. No wrapper slot (React has none).
//! Zero JS: the menu is closed until `popovertarget` opens it (React's audit
//! fixture forces `defaultOpen` and portals the menu out of the canvas).
//!
//! Rich menus (shared with context-menu and menubar through [`menu_items`],
//! which takes the slot prefix): one `item "…"` line per entry, with
//! `icon:` (lucide glyph), `shortcut:"⌘S"` (`…-shortcut` span),
//! `type:label` (`…-label`), `item "-"` (`…-separator`), `type:checkbox`
//! (+ `checked:true`) and `type:radio` (consecutive radios form a
//! `…-radio-group`) rendered as a `<label>` with a visually hidden native
//! input and React's decorative `…-checkbox-item` / `…-radio-item` div
//! (`aria-hidden`, indicator shown by `:has(:checked)`), and `sub:"Parent"`
//! marking an entry of the submenu opened by the `Parent` item, which becomes
//! a `…-sub-trigger` button whose `popovertarget` / `interestfor` opens the
//! nested `…-sub-content` popover beside it (click, keyboard or hover).
//! `trigger-variant:` styles the trigger (React Button variants) and
//! `width:56` (Tailwind `w-56`) is a `w-56` class on the content.
//! Not interact `popover("dropdown-menu")` (`<details>` SURF box).

use crate::cronus_ui_kit::{attr, attr_nonempty, choice_texts, esc, label_of, texts, widget_id};
use crate::parser::{ComponentItemNode, ComponentNode};

const CHEVRON_RIGHT: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m9 18 6-6-6-6\"></path></svg>";
const CHECK: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M20 6 9 17l-5-5\"></path></svg>";
const DOT: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"currentColor\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><circle cx=\"12\" cy=\"12\" r=\"10\"></circle></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let trigger_id = widget_id(comp, "trigger");
    let pop_id = widget_id(comp, "menu");
    let variant = trigger_variant(comp, "primary");
    let items = menu_items(comp, "dropdown-menu", &pop_id);
    let class = width_class(comp);
    format!(
        "<button type=\"button\" id=\"{trigger_id}\" data-slot=\"button\" data-variant=\"{variant}\" popovertarget=\"{pop_id}\" aria-haspopup=\"menu\">{label}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"dropdown-menu-content\" role=\"menu\" aria-orientation=\"vertical\" anchor=\"{trigger_id}\"{class}>{items}</div>"
    )
}

/// `trigger-variant:` prop as a React Button variant, else `default`.
pub fn trigger_variant<'a>(comp: &'a ComponentNode, default: &'a str) -> &'a str {
    match attr(comp, "trigger-variant").map(str::trim) {
        Some("primary") => "primary",
        Some("secondary") => "secondary",
        Some("outline") => "outline",
        Some("ghost") => "ghost",
        Some("link") => "link",
        Some("destructive") | Some("danger") => "destructive",
        _ => default,
    }
}

/// ` class="w-56"` for `width:56` (Tailwind width scale on the docs' content).
pub fn width_class(comp: &ComponentNode) -> String {
    match attr_nonempty(comp, "width").map(str::trim) {
        Some(w @ ("48" | "56" | "64" | "72" | "80")) => format!(" class=\"w-{w}\""),
        _ => String::new(),
    }
}

/// Rich `item` lines rendered with `prefix` slots (`dropdown-menu`,
/// `context-menu`, `menubar`); legacy `text` lines (the audit fixtures) stay
/// plain `…-item`s. `filter` keeps the items of one menubar menu.
pub fn menu_items(comp: &ComponentNode, prefix: &str, id: &str) -> String {
    menu_items_of(comp, prefix, id, |_| true)
}

pub fn menu_items_of(
    comp: &ComponentNode,
    prefix: &str,
    id: &str,
    filter: impl Fn(&ComponentItemNode) -> bool,
) -> String {
    let rich: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty() && filter(i))
        .collect();
    if rich.is_empty() {
        let plain = if comp.items.iter().any(|i| i.item_type == "item") {
            Vec::new()
        } else {
            let choices = choice_texts(comp);
            if choices.is_empty() {
                texts(comp).into_iter().skip(1).collect()
            } else {
                choices
            }
        };
        return plain
            .into_iter()
            .map(|t| format!("<div data-slot=\"{prefix}-item\" role=\"menuitem\">{t}</div>"))
            .collect();
    }
    let top: Vec<&ComponentItemNode> = rich
        .iter()
        .copied()
        .filter(|i| i.config.get("sub").map_or(true, |s| s.trim().is_empty()))
        .collect();
    render_list(&rich, &top, prefix, id, &mut 0)
}

fn render_list(
    all: &[&ComponentItemNode],
    items: &[&ComponentItemNode],
    prefix: &str,
    id: &str,
    counter: &mut usize,
) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < items.len() {
        let item = items[i];
        let kind = item.config.get("type").map(|t| t.trim()).unwrap_or("");
        if kind == "radio" {
            // Consecutive radios share one native group.
            let mut end = i;
            while end < items.len()
                && items[end].config.get("type").map(|t| t.trim()) == Some("radio")
            {
                end += 1;
            }
            *counter += 1;
            let name = format!("{id}-r{counter}");
            let radios: String = items[i..end]
                .iter()
                .map(|r| {
                    let checked = r
                        .config
                        .get("checked")
                        .is_some_and(|v| crate::cronus_ui_kit::truthy(v));
                    let on = if checked { " checked" } else { "" };
                    format!(
                        "<label><input type=\"radio\" name=\"{name}\" aria-label=\"{t}\"{on}><div data-slot=\"{prefix}-radio-item\" role=\"menuitemradio\" aria-hidden=\"true\" tabindex=\"-1\"><span>{DOT}</span>{t}</div></label>",
                        t = esc(&r.text)
                    )
                })
                .collect();
            out.push_str(&format!(
                "<div data-slot=\"{prefix}-radio-group\" role=\"group\">{radios}</div>"
            ));
            i = end;
            continue;
        }
        out.push_str(&render_one(all, item, prefix, id, counter));
        i += 1;
    }
    out
}

fn render_one(
    all: &[&ComponentItemNode],
    item: &ComponentItemNode,
    prefix: &str,
    id: &str,
    counter: &mut usize,
) -> String {
    let text = esc(&item.text);
    if item.text.trim() == "-" {
        return format!(
            "<div data-slot=\"{prefix}-separator\" role=\"separator\" aria-orientation=\"horizontal\"></div>"
        );
    }
    let kind = item.config.get("type").map(|t| t.trim()).unwrap_or("");
    if kind == "label" {
        return format!("<div data-slot=\"{prefix}-label\">{text}</div>");
    }
    let icon = crate::cronus_ui_kit::item_icon(item);
    let shortcut = item
        .config
        .get("shortcut")
        .filter(|s| !s.trim().is_empty())
        .map(|s| format!("<span data-slot=\"{prefix}-shortcut\">{}</span>", esc(s)))
        .unwrap_or_default();
    if kind == "checkbox" {
        let checked = item
            .config
            .get("checked")
            .is_some_and(|v| crate::cronus_ui_kit::truthy(v));
        let on = if checked { " checked" } else { "" };
        return format!(
            "<label><input type=\"checkbox\" aria-label=\"{text}\"{on}><div data-slot=\"{prefix}-checkbox-item\" role=\"menuitemcheckbox\" aria-hidden=\"true\" tabindex=\"-1\"><span>{CHECK}</span>{text}{shortcut}</div></label>"
        );
    }
    let children: Vec<&ComponentItemNode> = all
        .iter()
        .copied()
        .filter(|c| c.config.get("sub").map(|s| s.trim()) == Some(item.text.trim()))
        .collect();
    if !children.is_empty() {
        *counter += 1;
        let sub_id = format!("{id}-s{counter}");
        let trigger_id = format!("{sub_id}-trigger");
        let inner = render_list(all, &children, prefix, id, counter);
        return format!(
            "<button type=\"button\" id=\"{trigger_id}\" data-slot=\"{prefix}-sub-trigger\" role=\"menuitem\" aria-haspopup=\"menu\" aria-expanded=\"false\" popovertarget=\"{sub_id}\" interestfor=\"{sub_id}\">{icon}{text}{CHEVRON_RIGHT}</button><div id=\"{sub_id}\" popover=\"auto\" data-slot=\"{prefix}-sub-content\" role=\"menu\" aria-orientation=\"vertical\" anchor=\"{trigger_id}\">{inner}</div>"
        );
    }
    let disabled = if item
        .config
        .get("disabled")
        .is_some_and(|v| crate::cronus_ui_kit::truthy(v))
    {
        " data-disabled=\"\" aria-disabled=\"true\""
    } else {
        ""
    };
    format!(
        "<div data-slot=\"{prefix}-item\" role=\"menuitem\"{disabled}>{icon}{text}{shortcut}</div>"
    )
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

    fn cfg(kind: &str, text: &str, pairs: &[(&str, &str)]) -> ComponentItemNode {
        let mut i = extra(kind, text);
        for (k, v) in pairs {
            i.config.insert((*k).into(), (*v).into());
        }
        i
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
        assert!(
            html.contains("<div data-slot=\"dropdown-menu-item\" role=\"menuitem\">Share</div>")
        );
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
    fn rich_items_render_label_separator_icons_shortcuts_checkbox_and_submenu() {
        let mut c = stub("dropdown-menu", "Open menu");
        c.props.insert("trigger-variant".into(), "outline".into());
        c.props.insert("width".into(), "56".into());
        c.items
            .push(cfg("item", "My account", &[("type", "label")]));
        c.items.push(extra("item", "-"));
        c.items.push(cfg(
            "item",
            "Profile",
            &[("icon", "user"), ("shortcut", "⇧⌘P")],
        ));
        c.items.push(cfg(
            "item",
            "Show status bar",
            &[("type", "checkbox"), ("checked", "true")],
        ));
        c.items
            .push(cfg("item", "Invite team", &[("icon", "users")]));
        c.items.push(cfg(
            "item",
            "Email invite",
            &[("icon", "user-plus"), ("sub", "Invite team")],
        ));
        c.items.push(cfg(
            "item",
            "Copy invite link",
            &[("icon", "copy"), ("sub", "Invite team")],
        ));
        let html = render(&c);
        let pid = widget_id(&c, "menu");
        assert!(html.contains(&format!(
            "data-variant=\"outline\" popovertarget=\"{pid}\" aria-haspopup=\"menu\">Open menu</button>"
        )));
        assert!(html.contains("anchor=\"cui-dropdown-menu-trigger\" class=\"w-56\"><div data-slot=\"dropdown-menu-label\">My account</div><div data-slot=\"dropdown-menu-separator\" role=\"separator\" aria-orientation=\"horizontal\"></div><div data-slot=\"dropdown-menu-item\" role=\"menuitem\"><svg "), "{html}");
        assert!(html.contains("data-icon=\"user\""));
        assert!(html
            .contains("</svg>Profile<span data-slot=\"dropdown-menu-shortcut\">⇧⌘P</span></div>"));
        assert!(html.contains("<label><input type=\"checkbox\" aria-label=\"Show status bar\" checked><div data-slot=\"dropdown-menu-checkbox-item\" role=\"menuitemcheckbox\" aria-hidden=\"true\" tabindex=\"-1\"><span><svg "));
        assert!(html.contains(&format!(
            "<button type=\"button\" id=\"{pid}-s1-trigger\" data-slot=\"dropdown-menu-sub-trigger\" role=\"menuitem\" aria-haspopup=\"menu\" aria-expanded=\"false\" popovertarget=\"{pid}-s1\" interestfor=\"{pid}-s1\"><svg "
        )), "{html}");
        assert!(html.contains(&format!(
            "</svg>Invite team{CHEVRON_RIGHT}</button><div id=\"{pid}-s1\" popover=\"auto\" data-slot=\"dropdown-menu-sub-content\" role=\"menu\" aria-orientation=\"vertical\" anchor=\"{pid}-s1-trigger\"><div data-slot=\"dropdown-menu-item\" role=\"menuitem\"><svg "
        )), "{html}");
        assert!(html.contains("data-icon=\"user-plus\""));
        // Submenu entries are not top-level items.
        assert_eq!(html.matches("Email invite").count(), 1);
        assert!(html.contains("Copy invite link</div></div></div>"));
        reject_interact(&html);
    }

    #[test]
    fn radio_items_form_native_groups() {
        let mut c = stub("dropdown-menu", "Branch");
        c.items.push(cfg(
            "item",
            "main",
            &[("type", "radio"), ("checked", "true")],
        ));
        c.items.push(cfg("item", "develop", &[("type", "radio")]));
        let html = render(&c);
        let pid = widget_id(&c, "menu");
        assert!(html.contains(&format!(
            "<div data-slot=\"dropdown-menu-radio-group\" role=\"group\"><label><input type=\"radio\" name=\"{pid}-r1\" aria-label=\"main\" checked><div data-slot=\"dropdown-menu-radio-item\" role=\"menuitemradio\" aria-hidden=\"true\" tabindex=\"-1\"><span>{DOT}</span>main</div></label><label><input type=\"radio\" name=\"{pid}-r1\" aria-label=\"develop\"><div data-slot=\"dropdown-menu-radio-item\""
        )), "{html}");
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_popover_surf() {
        let c = menu("Actions", &["Edit"]);
        let html = render(&c);
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
        assert!(css.contains(
            "[data-slot=\"dropdown-menu-item\"] {\n  position: relative; display: flex;"
        ));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("min-width: 8rem"));
        assert!(!css.contains("[data-slot=\"dropdown-menu\"] > button"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        // Rich menu parts.
        assert!(css.contains("[data-slot=\"dropdown-menu-label\"] {\n  padding: 0.375rem 0.5rem; font-size: 0.75rem; line-height: 1rem; color: var(--cronus-fg-tertiary);"));
        assert!(css.contains("[data-slot=\"dropdown-menu-separator\"] {\n  margin: 0.25rem -0.25rem; height: 1px; background: var(--cronus-border);"));
        assert!(css.contains("[data-slot=\"dropdown-menu-shortcut\"] {\n  margin-inline-start: auto; font-size: 0.75rem; line-height: 1rem; letter-spacing: 0.1em; color: var(--cronus-fg-tertiary);"));
        assert!(css.contains("[data-slot=\"dropdown-menu-content\"] label:has(> input:checked) > [data-slot=\"dropdown-menu-checkbox-item\"] > span > svg,"));
        assert!(css.contains("[data-slot=\"dropdown-menu-sub-content\"]:popover-open {"));
        assert!(css.contains("animation: cronus-pop-in 180ms var(--ease-out-quart) both;"));
        assert!(css.contains("[data-slot=\"dropdown-menu-content\"].w-56:popover-open { min-width: 14rem; max-width: 14rem; }"));
    }
}
