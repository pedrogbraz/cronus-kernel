//! Dedicated ExpandableTabs renderer. DOM matches React:
//! `<div data-slot="expandable-tabs" role="tablist" aria-label>` plus each item as
//! `<button role="tab" data-slot="expandable-tabs-item" aria-selected>` holding an
//! icon `<span aria-hidden>` (generic circle glyph — .cronus carries no icon
//! node) and a label `<span>`; unselected labels are visually hidden (sr-only)
//! like React's collapsed state. First item is selected.
//! The `label` names the tablist; it is never a tab. Switching tabs needs JS, so
//! (wave 1t rule) tabs are native `disabled` buttons, not visually dimmed.
//! Not interact `tabs()` (generic tablist without expandable-tabs-item).

use crate::cronus_ui_kit::{attr_nonempty, choice_texts, esc, label_of};
use crate::parser::ComponentNode;

const GLYPH: &str = "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" aria-hidden=\"true\"><circle cx=\"12\" cy=\"12\" r=\"7\"/></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let buttons = tabs(comp)
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let selected = if i == 0 { "true" } else { "false" };
            format!(
                "<button type=\"button\" role=\"tab\" data-slot=\"expandable-tabs-item\" aria-selected=\"{selected}\" disabled><span aria-hidden=\"true\">{GLYPH}</span><span>{t}</span></button>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let aria = attr_nonempty(comp, "aria-label")
        .map(|a| format!(" aria-label=\"{}\"", esc(a)))
        .unwrap_or_default();
    format!("<div data-slot=\"expandable-tabs\" role=\"tablist\"{aria}>{buttons}</div>")
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
                "<div data-slot=\"expandable-tabs\" role=\"tablist\" aria-label=\"Sections\"><button type=\"button\" role=\"tab\" data-slot=\"expandable-tabs-item\" aria-selected=\"true\" disabled><span aria-hidden=\"true\">{GLYPH}</span><span>Home</span></button><button type=\"button\" role=\"tab\" data-slot=\"expandable-tabs-item\" aria-selected=\"false\" disabled><span aria-hidden=\"true\">{GLYPH}</span><span>Search</span></button></div>"
            )
        );
        reject_interact(&html);
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
