//! Dedicated NavigationMenu renderer. DOM mirrors the React audit fixture:
//! `<nav data-slot="navigation-menu" aria-label="Main">` > Radix's slotless
//! `<div>` > `<ul data-slot="navigation-menu-list">` > one
//! `<li data-slot="navigation-menu-item">` per text with a closed
//! `<button data-slot="navigation-menu-trigger">` (label + ChevronDown 14px).
//! The fixture's triggers carry no `NavigationMenuContent`, so nothing opens.
//! Radix toggles need JS: triggers are native buttons rendered `disabled` with
//! React's idle (closed, undimmed) look. No Radix viewport (empty, slotless).
//! Not interact `nav("navigation-menu")` (generic SURF `<nav>`).

use crate::cronus_ui_kit::{choice_texts, texts};
use crate::parser::ComponentNode;

const CHEVRON: &str = concat!(
    "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" ",
    "stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" focusable=\"false\">",
    "<path d=\"m6 9 6 6 6-6\" /></svg>",
);

pub fn render(comp: &ComponentNode) -> String {
    let items = menu_triggers(comp)
        .iter()
        .map(|t| {
            format!(
                "<li data-slot=\"navigation-menu-item\"><button type=\"button\" data-slot=\"navigation-menu-trigger\" data-state=\"closed\" aria-expanded=\"false\" disabled>{t}{CHEVRON}</button></li>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<nav data-slot=\"navigation-menu\" aria-label=\"Main\"><div><ul data-slot=\"navigation-menu-list\">{items}</ul></div></nav>"
    )
}

fn menu_triggers(comp: &ComponentNode) -> Vec<String> {
    let choices = choice_texts(comp);
    if !choices.is_empty() {
        return choices;
    }
    texts(comp)
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
        assert!(!html.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("nav("));
    }

    #[test]
    fn label_and_texts_become_closed_triggers_without_content() {
        let mut c = stub("navigation-menu", "Products");
        c.items.push(extra("text", "Analytics"));
        c.items.push(extra("text", "Docs"));
        let html = render(&c);
        let li = |t: &str| {
            format!(
                "<li data-slot=\"navigation-menu-item\"><button type=\"button\" data-slot=\"navigation-menu-trigger\" data-state=\"closed\" aria-expanded=\"false\" disabled>{t}{CHEVRON}</button></li>"
            )
        };
        assert_eq!(
            html,
            format!(
                "<nav data-slot=\"navigation-menu\" aria-label=\"Main\"><div><ul data-slot=\"navigation-menu-list\">{}{}{}</ul></div></nav>",
                li("Products"),
                li("Analytics"),
                li("Docs")
            )
        );
        assert!(!html.contains("navigation-menu-content"));
        assert!(!html.contains("data-state=\"open\""));
        reject_interact(&html);
    }

    #[test]
    fn field_label_is_not_a_trigger_when_items_exist() {
        let mut c = stub("navigation-menu", "Menus");
        c.items.push(extra("item", "Products"));
        c.items.push(extra("item", "Docs"));
        let html = render(&c);
        assert!(html.contains("aria-expanded=\"false\" disabled>Products<svg"));
        assert!(html.contains("aria-expanded=\"false\" disabled>Docs<svg"));
        assert!(!html.contains(">Menus<svg"));
        assert_eq!(
            html.matches("data-slot=\"navigation-menu-trigger\"")
                .count(),
            2
        );
        reject_interact(&html);
    }

    #[test]
    fn label_only_is_one_trigger() {
        let html = render(&stub("navigation-menu", "Products"));
        assert_eq!(
            html.matches("data-slot=\"navigation-menu-trigger\"")
                .count(),
            1
        );
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_nav_surf() {
        let mut c = stub("navigation-menu", "Menus");
        c.items.push(extra("item", "Products"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"navigation-menu-list\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("navigation-menu", "Products"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"navigation-menu\"]"));
        assert!(css.contains(
            "[data-slot=\"navigation-menu-list\"] {\n  display: flex; flex: 1; list-style: none;"
        ));
        assert!(css.contains("[data-slot=\"navigation-menu-item\"]"));
        assert!(css.contains("[data-slot=\"navigation-menu-trigger\"] > svg {"));
        assert!(css.contains("font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(!css.contains("[data-slot=\"navigation-menu-content\"]"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
