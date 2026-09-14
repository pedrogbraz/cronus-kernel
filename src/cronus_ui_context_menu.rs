//! Dedicated ContextMenu renderer (wave1t geometry parity with Radix ContextMenu).
//!
//! React: the trigger is a bare `<span>` (no data-slot) and the open content is
//! `<div data-slot="context-menu-content" role="menu">` (min-w 8rem, p-1, radius lg,
//! border, surface-floating) with one `<div data-slot="context-menu-item"
//! role="menuitem">` per item (px-2 py-1.5, text-sm, radius md).
//! Audit cannot right-click and the kernel has no popper, so the content is
//! always open, in flow under the trigger (React places it at the pointer; the
//! geometry spec measures it against itself). Zero JS.
//! Not interact `popover("context-menu")` (`<details>` SURF box).

use crate::cronus_ui_kit::{choice_texts, label_of, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let items = menu_items(comp)
        .into_iter()
        .map(|t| format!("<div data-slot=\"context-menu-item\" role=\"menuitem\">{t}</div>"))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<span>{label}</span><div data-slot=\"context-menu-content\" role=\"menu\" aria-orientation=\"vertical\">{items}</div>"
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
        let interact = crate::cronus_ui_interact::render("context-menu", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<details data-slot=\"context-menu\""));
        assert!(!interact.contains("data-slot=\"context-menu-content\""));
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
