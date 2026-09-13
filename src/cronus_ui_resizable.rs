//! Dedicated Resizable renderer. Static two-panel split, no JS drag.
//! Wrapper `data-slot="resizable"` plus `<div data-slot="resizable-panel-group">`
//! two panels and `data-slot="resizable-handle"`.
//! Not catalog `display()` SURF (`<section>` without panel-group).

use crate::cronus_ui_kit::{choice_texts, label_of, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let (left, right) = panels(comp);
    format!(
        "<div data-slot=\"resizable\"><div data-slot=\"resizable-panel-group\" data-panel-group-direction=\"horizontal\"><div data-slot=\"resizable-panel\">{left}</div><div data-slot=\"resizable-handle\" role=\"separator\" aria-orientation=\"vertical\" aria-valuenow=\"50\" aria-valuemin=\"0\" aria-valuemax=\"100\"></div><div data-slot=\"resizable-panel\">{right}</div></div></div>"
    )
}

fn panels(comp: &ComponentNode) -> (String, String) {
    let mut items = choice_texts(comp);
    if items.is_empty() {
        items = texts(comp);
    }
    let left = items
        .first()
        .cloned()
        .unwrap_or_else(|| label_of(comp));
    let right = items.get(1).cloned().unwrap_or_default();
    (left, right)
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

    fn split(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("resizable", items.first().copied().unwrap_or("Sidebar"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.starts_with("<section"));
        assert!(!html.contains("<section data-slot=\"resizable\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("max-height:12rem;overflow:auto"));
        assert!(!html.contains("padding:1rem;display:flex;flex-direction:column;gap:0.5rem"));
        assert!(!html.contains("display("));
        assert!(html.contains("data-slot=\"resizable-panel-group\""));
        assert!(html.contains("data-slot=\"resizable-handle\""));
    }

    #[test]
    fn root_is_wrapper_with_group_two_panels_and_handle() {
        let html = render(&split(&["Sidebar", "Main"]));
        assert!(html.starts_with("<div data-slot=\"resizable\">"));
        assert!(html.contains(
            "<div data-slot=\"resizable-panel-group\" data-panel-group-direction=\"horizontal\">"
        ));
        assert!(html.contains("<div data-slot=\"resizable-panel\">Sidebar</div>"));
        assert!(html.contains("<div data-slot=\"resizable-panel\">Main</div>"));
        assert!(html.contains(
            "<div data-slot=\"resizable-handle\" role=\"separator\" aria-orientation=\"vertical\" aria-valuenow=\"50\" aria-valuemin=\"0\" aria-valuemax=\"100\"></div>"
        ));
        assert_eq!(html.matches("data-slot=\"resizable-panel\"").count(), 2);
        assert_eq!(html.matches("data-slot=\"resizable-handle\"").count(), 1);
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"resizable\"><div data-slot=\"resizable-panel-group\" data-panel-group-direction=\"horizontal\"><div data-slot=\"resizable-panel\">Sidebar</div><div data-slot=\"resizable-handle\" role=\"separator\" aria-orientation=\"vertical\" aria-valuenow=\"50\" aria-valuemin=\"0\" aria-valuemax=\"100\"></div><div data-slot=\"resizable-panel\">Main</div></div></div>"
        );
    }

    #[test]
    fn extra_text_items_become_panels() {
        let mut c = stub("resizable", "Notes");
        c.items.push(extra("text", "Preview"));
        let html = render(&c);
        assert!(html.contains("<div data-slot=\"resizable-panel\">Notes</div>"));
        assert!(html.contains("<div data-slot=\"resizable-panel\">Preview</div>"));
        assert_eq!(html.matches("data-slot=\"resizable-panel\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_two_panels() {
        let html = render(&stub("resizable", "Sidebar"));
        assert!(html.contains("<div data-slot=\"resizable\">"));
        assert!(html.contains("<div data-slot=\"resizable-panel-group\""));
        assert!(html.contains("<div data-slot=\"resizable-panel\">Sidebar</div>"));
        assert!(html.contains("<div data-slot=\"resizable-handle\""));
        assert_eq!(html.matches("data-slot=\"resizable-panel\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn skips_display_and_scroll_surf() {
        let c = split(&["Sidebar", "Main"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("resizable", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<div data-slot=\"resizable\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("max-height:12rem;overflow:auto"));
        assert!(!interact.contains("data-slot=\"resizable-panel-group\""));
        assert!(!interact.contains("data-slot=\"resizable-handle\""));
        assert!(html.contains("data-slot=\"resizable-panel-group\""));
        assert!(html.contains("data-slot=\"resizable-handle\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&split(&["Sidebar", "Main"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"resizable\"]"));
        assert!(css.contains("[data-slot=\"resizable-panel-group\"]"));
        assert!(css.contains("[data-slot=\"resizable-panel\"]"));
        assert!(css.contains("[data-slot=\"resizable-handle\"]"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
