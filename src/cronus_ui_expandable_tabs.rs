//! Dedicated ExpandableTabs renderer. DOM matches React:
//! `<div data-slot="expandable-tabs" role="tablist" aria-label>` plus each item as
//! `<button role="tab" data-slot="expandable-tabs-item" aria-selected>` holding an
//! icon `<span aria-hidden>` (generic circle glyph — .cronus carries no icon
//! node) and a label `<span>`; unselected labels are visually hidden (sr-only)
//! like React's collapsed state. First item is selected.
//! The `label` names the group; it is never a tab.
//!
//! Zero JS: each item sits in a `<label>` with a visually hidden radio (shared
//! page-unique `name`); clicking an item or Arrow keys select it, and CSS
//! expands the label text of the item after the checked radio. The button keeps
//! React's slot and look but is decorative (`aria-hidden`, `tabindex="-1"`,
//! `pointer-events: none`), so the root is a `radiogroup`. Gaps vs React: no
//! `tablist`/`tab` roles or live `aria-selected`, no width animation, no
//! outside-click collapse.
//! Not interact `tabs()` (generic tablist without expandable-tabs-item).

use crate::cronus_ui_kit::{attr_nonempty, choice_texts, esc, instance_id, label_of};
use crate::parser::ComponentNode;

const GLYPH: &str = "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" aria-hidden=\"true\"><circle cx=\"12\" cy=\"12\" r=\"7\"/></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let name = instance_id(comp, "expandable-tabs");
    let buttons = tabs(comp)
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let (selected, checked) = if i == 0 {
                ("true", " checked")
            } else {
                ("false", "")
            };
            format!(
                "<label><input type=\"radio\" name=\"{name}\" value=\"{i}\" aria-label=\"{t}\"{checked}><button type=\"button\" role=\"tab\" data-slot=\"expandable-tabs-item\" aria-selected=\"{selected}\" tabindex=\"-1\" aria-hidden=\"true\"><span aria-hidden=\"true\">{GLYPH}</span><span>{t}</span></button></label>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let aria = attr_nonempty(comp, "aria-label")
        .map(|a| format!(" aria-label=\"{}\"", esc(a)))
        .unwrap_or_default();
    format!("<div data-slot=\"expandable-tabs\" role=\"radiogroup\"{aria}>{buttons}</div>")
}

fn tabs(comp: &ComponentNode) -> Vec<String> {
    let choices = choice_texts(comp);
    if !choices.is_empty() {
        return choices;
    }
    let body: Vec<String> = comp
        .items
        .iter()
        .filter(|i| !matches!(i.item_type.as_str(), "label" | "title") && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if body.is_empty() {
        vec![label_of(comp)]
    } else {
        body
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

    fn reject_interact(html: &str) {
        assert!(!html.contains("<div role=\"tablist\""));
        assert!(!html.contains("role=\"tabpanel\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains(" disabled"));
    }

    #[test]
    fn emitted_fixture_has_icon_and_label_spans_label_is_aria() {
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
                "<div data-slot=\"expandable-tabs\" role=\"radiogroup\" aria-label=\"Sections\"><label><input type=\"radio\" name=\"cui-expandable-tabs-expandable-tabs\" value=\"0\" aria-label=\"Home\" checked><button type=\"button\" role=\"tab\" data-slot=\"expandable-tabs-item\" aria-selected=\"true\" tabindex=\"-1\" aria-hidden=\"true\"><span aria-hidden=\"true\">{GLYPH}</span><span>Home</span></button></label><label><input type=\"radio\" name=\"cui-expandable-tabs-expandable-tabs\" value=\"1\" aria-label=\"Search\"><button type=\"button\" role=\"tab\" data-slot=\"expandable-tabs-item\" aria-selected=\"false\" tabindex=\"-1\" aria-hidden=\"true\"><span aria-hidden=\"true\">{GLYPH}</span><span>Search</span></button></label></div>"
            )
        );
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
        assert!(css.contains("[data-slot=\"expandable-tabs\"] > label > input:not(:checked) + [data-slot=\"expandable-tabs-item\"] > span:last-child {\n  position: absolute; width: 1px; height: 1px;"));
        assert!(css.contains("[data-slot=\"expandable-tabs\"] > label > input:checked + [data-slot=\"expandable-tabs-item\"] > span:last-child {\n  position: static; width: auto; height: auto; margin: 0; clip: auto;\n}"));
    }

    #[test]
    fn label_only_still_emits_selected_item() {
        let html = render(&stub("expandable-tabs", "Home"));
        assert_eq!(
            html.matches("data-slot=\"expandable-tabs-item\"").count(),
            1
        );
        assert!(html.contains("aria-selected=\"true\""));
        assert!(html.contains("<span>Home</span>"));
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
            "[data-slot=\"expandable-tabs-item\"][aria-selected=\"false\"] > span:last-child"
        ));
        assert!(css.contains(
            "font: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 500; cursor: pointer;"
        ));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(!css.contains("zinc-"));
    }
}
