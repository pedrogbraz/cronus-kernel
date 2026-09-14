//! Dedicated Toolbar renderer. DOM matches React:
//! `<div data-slot="toolbar" role="toolbar" aria-orientation="horizontal" aria-label>`
//! plus each item as `<button type="button" data-slot="toolbar-button">`.
//! The `label` names the toolbar (it is never a button). Editor commands need JS,
//! so (wave 1t rule) buttons are native `disabled` buttons, not visually dimmed.
//! Not interact `nav("toolbar")` (generic SURF `<nav>`).

use crate::cronus_ui_kit::{attr_nonempty, choice_texts, esc, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let buttons = toolbar_buttons(comp)
        .into_iter()
        .map(|t| {
            format!("<button type=\"button\" data-slot=\"toolbar-button\" disabled>{t}</button>")
        })
        .collect::<Vec<_>>()
        .join("");
    let aria = attr_nonempty(comp, "aria-label")
        .map(|a| format!(" aria-label=\"{}\"", esc(a)))
        .unwrap_or_default();
    format!(
        "<div data-slot=\"toolbar\" role=\"toolbar\" aria-orientation=\"horizontal\"{aria}>{buttons}</div>"
    )
}

fn toolbar_buttons(comp: &ComponentNode) -> Vec<String> {
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
        assert!(!html.starts_with("<nav"));
        assert!(!html.contains("<details"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn emitted_fixture_label_is_aria_not_a_button() {
        let mut c = stub("toolbar", "Formatting");
        c.items.push(extra("text", "Bold"));
        c.items.push(extra("text", "Italic"));
        c.items[2]
            .config
            .insert("aria-label".into(), "Formatting".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"toolbar\" role=\"toolbar\" aria-orientation=\"horizontal\" aria-label=\"Formatting\"><button type=\"button\" data-slot=\"toolbar-button\" disabled>Bold</button><button type=\"button\" data-slot=\"toolbar-button\" disabled>Italic</button></div>"
        );
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_one_button() {
        let html = render(&stub("toolbar", "Bold"));
        assert_eq!(html.matches("data-slot=\"toolbar-button\"").count(), 1);
        assert!(html.contains(">Bold</button>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_nav_surf() {
        let mut c = stub("toolbar", "Format");
        c.items.push(extra("item", "Bold"));
        let html = render(&c);
        assert!(html.contains("role=\"toolbar\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            reject_interact(&render(&stub("toolbar", "Bold")));
        });
    }

    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"toolbar\"]"));
        assert!(css.contains(
            "font: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 500; cursor: pointer;\n  background: transparent; color: var(--cronus-fg-secondary);"
        ));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(!css.contains("zinc-"));
    }
}
