//! Dedicated Toolbar renderer. DOM matches React:
//! `<div data-slot="toolbar" role="toolbar">` plus each text as
//! `<button type="button" data-slot="toolbar-button">`.
//! Not interact `nav("toolbar")` (generic SURF `<nav>`).

use crate::cronus_ui_kit::{choice_texts, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let buttons = toolbar_buttons(comp)
        .into_iter()
        .map(|t| {
            format!("<button type=\"button\" data-slot=\"toolbar-button\">{t}</button>")
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"toolbar\" role=\"toolbar\">{buttons}</div>")
}

fn toolbar_buttons(comp: &ComponentNode) -> Vec<String> {
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

    fn bar(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("toolbar", items.first().copied().unwrap_or("Bold"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.starts_with("<nav"));
        assert!(!html.contains("<nav data-slot=\"toolbar\""));
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
    fn root_is_toolbar_with_buttons_not_nav_surf() {
        let html = render(&bar(&["Bold", "Italic"]));
        assert!(html.starts_with("<div data-slot=\"toolbar\" role=\"toolbar\">"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"toolbar-button\">Bold</button>"
        ));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"toolbar-button\">Italic</button>"
        ));
        assert_eq!(html.matches("data-slot=\"toolbar-button\"").count(), 2);
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"toolbar\" role=\"toolbar\"><button type=\"button\" data-slot=\"toolbar-button\">Bold</button><button type=\"button\" data-slot=\"toolbar-button\">Italic</button></div>"
        );
    }

    #[test]
    fn extra_text_items_become_buttons() {
        let mut c = stub("toolbar", "Bold");
        c.items.push(extra("text", "Italic"));
        c.items.push(extra("text", "Underline"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"toolbar-button\">Bold</button>"));
        assert!(html.contains("data-slot=\"toolbar-button\">Italic</button>"));
        assert!(html.contains("data-slot=\"toolbar-button\">Underline</button>"));
        assert_eq!(html.matches("data-slot=\"toolbar-button\"").count(), 3);
        reject_interact(&html);
    }

    #[test]
    fn field_label_is_not_a_button_when_items_exist() {
        let mut c = stub("toolbar", "Format");
        c.items.push(extra("item", "Bold"));
        c.items.push(extra("item", "Italic"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"toolbar-button\">Bold</button>"));
        assert!(html.contains("data-slot=\"toolbar-button\">Italic</button>"));
        assert!(!html.contains(">Format</button>"));
        assert_eq!(html.matches("data-slot=\"toolbar-button\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_one_button() {
        let html = render(&stub("toolbar", "Bold"));
        assert!(html.contains("<div data-slot=\"toolbar\" role=\"toolbar\">"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"toolbar-button\">Bold</button>"
        ));
        assert_eq!(html.matches("data-slot=\"toolbar-button\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_nav_surf() {
        let c = bar(&["Bold", "Italic"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("toolbar", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<nav data-slot=\"toolbar\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!interact.contains("data-slot=\"toolbar-button\""));
        assert!(!interact.contains("role=\"toolbar\""));
        assert!(html.contains("data-slot=\"toolbar-button\""));
        assert!(html.contains("role=\"toolbar\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&bar(&["Bold"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"toolbar\"]"));
        assert!(css.contains("[data-slot=\"toolbar-button\"]"));
        assert!(css.contains("display: inline-flex"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("height: 2rem"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
