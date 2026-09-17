//! Dedicated ExpandableTabs renderer. DOM mirrors React (`<div
//! data-slot="expandable-tabs">` of `expandable-tabs-item` buttons, each an
//! icon cell + label) with zero JS: every item is a `<label>` holding a
//! visually hidden radio (page-unique `name`; `value:` picks the checked
//! one, `disabled:true` disables it) and React's `<button role="tab">` as a
//! decorative twin (`tabindex="-1"`, `aria-hidden`). Items are `item`
//! lines; `icon:` names the lucide glyph (React's `icon`), a plain ring
//! when absent. The label sits in a `grid-template-columns` cell that
//! COMPONENT_CHROME opens from `0fr` to `1fr` on the checked item (React's
//! `motion.span` width `0 → auto`, 220ms `[0.22, 1, 0.36, 1]`, with
//! opacity). The `label` names the group (it is never a tab).

use crate::cronus_ui_kit::{attr, attr_nonempty, esc, instance_id, item_icon, label_of, truthy};
use crate::parser::{ComponentItemNode, ComponentNode};

const GLYPH: &str = "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" aria-hidden=\"true\"><circle cx=\"12\" cy=\"12\" r=\"7\"/></svg>";

struct Tab {
    text: String,
    icon: String,
    disabled: bool,
}

pub fn render(comp: &ComponentNode) -> String {
    let tabs = tabs(comp);
    let selected = attr(comp, "value")
        .map(esc)
        .and_then(|v| tabs.iter().position(|t| t.text == v))
        .unwrap_or(0);
    let name = instance_id(comp, "expandable-tabs");
    let buttons = tabs
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let (aria_selected, checked) = if i == selected {
                ("true", " checked")
            } else {
                ("false", "")
            };
            let disabled = if t.disabled { " disabled" } else { "" };
            format!(
                "<label><input type=\"radio\" name=\"{name}\" value=\"{i}\" aria-label=\"{text}\"{checked}{disabled}><button type=\"button\" role=\"tab\" data-slot=\"expandable-tabs-item\" aria-selected=\"{aria_selected}\" tabindex=\"-1\" aria-hidden=\"true\"><span aria-hidden=\"true\">{icon}</span><span><span>{text}</span></span></button></label>",
                text = t.text,
                icon = t.icon
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let aria = attr_nonempty(comp, "aria-label")
        .map(|a| format!(" aria-label=\"{}\"", esc(a)))
        .unwrap_or_default();
    format!("<div data-slot=\"expandable-tabs\" role=\"radiogroup\"{aria}>{buttons}</div>")
}

fn tab_of(i: &ComponentItemNode) -> Tab {
    let icon = item_icon(i);
    Tab {
        text: esc(&i.text),
        icon: if icon.is_empty() {
            GLYPH.to_string()
        } else {
            icon
        },
        disabled: i.config.get("disabled").is_some_and(|v| truthy(v)),
    }
}

fn tabs(comp: &ComponentNode) -> Vec<Tab> {
    let choices: Vec<Tab> = comp
        .items
        .iter()
        .filter(|i| {
            matches!(i.item_type.as_str(), "item" | "tab" | "columns") && !i.text.is_empty()
        })
        .map(tab_of)
        .collect();
    if !choices.is_empty() {
        return choices;
    }
    let body: Vec<Tab> = comp
        .items
        .iter()
        .filter(|i| !matches!(i.item_type.as_str(), "label" | "title") && !i.text.is_empty())
        .map(tab_of)
        .collect();
    if body.is_empty() {
        vec![Tab {
            text: label_of(comp),
            icon: GLYPH.to_string(),
            disabled: false,
        }]
    } else {
        body
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

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
        assert!(!html.contains("<div role=\"tablist\""));
        assert!(!html.contains("role=\"tabpanel\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn emitted_fixture_has_icon_and_label_spans_label_is_aria() {
        crate::cronus_ui_kit::reset_instance_ids();
        let mut c = stub("expandable-tabs", "Sections");
        c.items.push(extra("text", "Home"));
        c.items.push(extra("text", "Search"));
        c.items[2]
            .config
            .insert("aria-label".into(), "Sections".into());
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"expandable-tabs\" role=\"radiogroup\" aria-label=\"Sections\"><label><input type=\"radio\" name=\"cui-expandable-tabs-expandable-tabs\" value=\"0\" aria-label=\"Home\" checked><button type=\"button\" role=\"tab\" data-slot=\"expandable-tabs-item\" aria-selected=\"true\" tabindex=\"-1\" aria-hidden=\"true\"><span aria-hidden=\"true\">{GLYPH}</span><span><span>Home</span></span></button></label><label><input type=\"radio\" name=\"cui-expandable-tabs-expandable-tabs\" value=\"1\" aria-label=\"Search\"><button type=\"button\" role=\"tab\" data-slot=\"expandable-tabs-item\" aria-selected=\"false\" tabindex=\"-1\" aria-hidden=\"true\"><span aria-hidden=\"true\">{GLYPH}</span><span><span>Search</span></span></button></label></div>"
            )
        );
        assert!(!html.contains(" disabled"));
        reject_interact(&html);
    }

    /// Docs example: `items` with lucide icons; `value:` picks the checked tab.
    #[test]
    fn item_icons_replace_the_ring_and_value_selects() {
        let mut c = stub("expandable-tabs", "Sections");
        c.items.clear();
        for (text, icon) in [
            ("Home", "home"),
            ("Search", "search"),
            ("Settings", "settings"),
        ] {
            let mut i = extra("item", text);
            i.config.insert("icon".into(), icon.into());
            c.items.push(i);
        }
        c.items[2].config.insert("disabled".into(), "true".into());
        c.props.insert("value".into(), "Search".into());
        let html = render(&c);
        assert!(!html.contains(GLYPH));
        assert_eq!(html.matches("<svg").count(), 3);
        assert!(html.contains("value=\"1\" aria-label=\"Search\" checked>"));
        assert!(html.contains("aria-selected=\"true\" tabindex=\"-1\" aria-hidden=\"true\"><span aria-hidden=\"true\"><svg"));
        assert!(html.contains("value=\"2\" aria-label=\"Settings\" disabled>"));
        assert_eq!(html.matches("aria-selected=\"true\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn two_sets_on_one_page_get_distinct_names() {
        crate::cronus_ui_kit::reset_instance_ids();
        let a = render(&stub("expandable-tabs", "Home"));
        let b = render(&stub("expandable-tabs", "Home"));
        assert!(a.contains("name=\"cui-expandable-tabs-expandable-tabs\" "));
        assert!(b.contains("name=\"cui-expandable-tabs-expandable-tabs-2\" "));
    }

    /// The checked radio, not the initial `aria-selected`, expands an item.
    #[test]
    fn chrome_expansion_follows_checked_radio() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"expandable-tabs\"] > label > input:checked + [data-slot=\"expandable-tabs-item\"] {\n  color: var(--cronus-fg); background: var(--cronus-surface-floating);"));
        // React: motion.span width 0 → auto + opacity, 220ms [0.22, 1, 0.36, 1].
        assert!(css.contains("[data-slot=\"expandable-tabs-item\"] > span:last-child {\n  display: grid; grid-template-columns: 0fr; opacity: 0; margin-inline-start: -0.5rem;\n  transition: grid-template-columns 220ms var(--cronus-ease), opacity 220ms var(--cronus-ease), margin-inline-start 220ms var(--cronus-ease);\n}"));
        assert!(css.contains("[data-slot=\"expandable-tabs-item\"] > span:last-child > span { min-width: 0; overflow: hidden; white-space: nowrap; }"));
        assert!(css.contains("[data-slot=\"expandable-tabs\"] > label > input:checked + [data-slot=\"expandable-tabs-item\"] > span:last-child {\n  grid-template-columns: 1fr; opacity: 1; margin-inline-start: 0;\n}"));
        assert!(css.contains("[data-slot=\"expandable-tabs\"] > label > input:disabled + [data-slot=\"expandable-tabs-item\"] { opacity: 0.5; }"));
        assert!(!css.contains(
            "[data-slot=\"expandable-tabs-item\"][aria-selected=\"false\"] > span:last-child"
        ));
    }

    #[test]
    fn label_only_still_emits_selected_item() {
        let html = render(&stub("expandable-tabs", "Home"));
        assert_eq!(
            html.matches("data-slot=\"expandable-tabs-item\"").count(),
            1
        );
        assert!(html.contains("aria-selected=\"true\""));
        assert!(html.contains("<span><span>Home</span></span>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_tabs_generic_tablist() {
        let mut c = stub("expandable-tabs", "Menus");
        c.items.push(extra("item", "Home"));
        c.items.push(extra("item", "Search"));
        let html = render(&c);
        assert!(!html.contains("<span>Menus</span>"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            reject_interact(&render(&stub("expandable-tabs", "Home")));
        });
    }

    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"expandable-tabs-item\"] > span[aria-hidden=\"true\"]"));
        assert!(css.contains(
            "font: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 500; cursor: pointer;"
        ));
        assert!(css.contains("padding: 0.375rem 0.625rem;"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(!css.contains("zinc-"));
    }
}
