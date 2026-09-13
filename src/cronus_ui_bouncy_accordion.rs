//! Dedicated BouncyAccordion renderer. Stacked items from texts.
//! Root `<div data-slot="bouncy-accordion">` plus
//! `data-slot="bouncy-accordion-trigger"` per item. First item open with
//! content. Not interact `accordion()` SURF `<details>` (no trigger slot).

use crate::cronus_ui_kit::choice_texts;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let items = {
        let c = choice_texts(comp);
        if c.is_empty() {
            crate::cronus_ui_kit::texts(comp)
        } else {
            c
        }
    };
    let inner = items
        .into_iter()
        .enumerate()
        .map(|(i, t)| {
            if i == 0 {
                format!(
                    "<button type=\"button\" data-slot=\"bouncy-accordion-trigger\" aria-expanded=\"true\">{t}</button><div data-slot=\"bouncy-accordion-content\">{t}</div>"
                )
            } else {
                format!(
                    "<button type=\"button\" data-slot=\"bouncy-accordion-trigger\" aria-expanded=\"false\">{t}</button>"
                )
            }
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"bouncy-accordion\">{inner}</div>")
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

    fn stack(label: &str, items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("bouncy-accordion", label);
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
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("<nav "));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_items_first_open_with_content() {
        let html = render(&stack("Hint", &["Type Shit", "Schedule"]));
        assert!(html.starts_with("<div data-slot=\"bouncy-accordion\">"));
        assert!(html.contains("data-slot=\"bouncy-accordion-trigger\""));
        assert!(html.contains("aria-expanded=\"true\">Type Shit</button>"));
        assert!(html.contains(
            "<div data-slot=\"bouncy-accordion-content\">Type Shit</div>"
        ));
        assert!(html.contains("aria-expanded=\"false\">Schedule</button>"));
        assert_eq!(html.matches("data-slot=\"bouncy-accordion-trigger\"").count(), 2);
        assert_eq!(html.matches("data-slot=\"bouncy-accordion-content\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn label_only_opens_first_with_content() {
        let html = render(&stub("bouncy-accordion", "Type Shit"));
        assert!(html.contains("<div data-slot=\"bouncy-accordion\">"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"bouncy-accordion-trigger\" aria-expanded=\"true\">Type Shit</button>"
        ));
        assert!(html.contains(
            "<div data-slot=\"bouncy-accordion-content\">Type Shit</div>"
        ));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_accordion_surf_details() {
        let c = stack("Hint", &["Type Shit", "Schedule"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("accordion", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<details"));
        assert!(interact.contains("<summary"));
        assert!(interact.contains("style="));
        assert!(!interact.contains("data-slot=\"bouncy-accordion-trigger\""));
        assert!(html.contains("data-slot=\"bouncy-accordion-trigger\""));
        assert!(!html.contains("<details"));
        reject_interact(&html);
        assert!(crate::cronus_ui_interact::render("bouncy-accordion", &c).is_none());
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("bouncy-accordion", "Type Shit"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"bouncy-accordion-trigger\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"bouncy-accordion\"]"));
        assert!(css.contains("[data-slot=\"bouncy-accordion-trigger\"]"));
        assert!(css.contains("[data-slot=\"bouncy-accordion-content\"]"));
        assert!(css.contains("max-width: 300px"));
        assert!(css.contains("min-height: 45px"));
        assert!(css.contains("var(--cronus-surface-base)"));
        assert!(css.contains("var(--cronus-fg-tertiary"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
