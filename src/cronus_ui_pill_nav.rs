//! Dedicated PillNav renderer. DOM matches React:
//! `<nav data-slot="pill-nav">` plus each text as
//! `<a data-slot="pill-nav-item">` when the item has a link, otherwise
//! `<button type="button" data-slot="pill-nav-item">`. First item is current.
//! Not interact `nav("pill-nav")` (generic SURF `<nav>` without pill-nav-item).

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let items = nav_entries(comp)
        .into_iter()
        .enumerate()
        .map(|(i, (text, href))| item_html(&text, href.as_deref(), i == 0))
        .collect::<Vec<_>>()
        .join("");
    format!("<nav data-slot=\"pill-nav\">{items}</nav>")
}

fn item_html(text: &str, href: Option<&str>, current: bool) -> String {
    let current_attr = if current {
        " aria-current=\"page\""
    } else {
        ""
    };
    match href {
        Some(h) => format!("<a data-slot=\"pill-nav-item\" href=\"{h}\"{current_attr}>{text}</a>"),
        None => format!(
            "<button type=\"button\" data-slot=\"pill-nav-item\"{current_attr}>{text}</button>"
        ),
    }
}

fn nav_entries(comp: &ComponentNode) -> Vec<(String, Option<String>)> {
    let choice: Vec<(String, Option<String>)> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "item" | "tab" | "columns") && !i.text.is_empty())
        .map(entry_of)
        .collect();
    if !choice.is_empty() {
        return choice;
    }
    let other: Vec<(String, Option<String>)> = comp
        .items
        .iter()
        .filter(|i| {
            !matches!(i.item_type.as_str(), "label" | "title" | "text" | "value")
                && !i.text.is_empty()
        })
        .map(entry_of)
        .collect();
    if !other.is_empty() {
        return other;
    }
    let all: Vec<(String, Option<String>)> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty())
        .map(entry_of)
        .collect();
    if all.is_empty() {
        vec![(label_of(comp), None)]
    } else {
        all
    }
}

fn entry_of(i: &ComponentItemNode) -> (String, Option<String>) {
    (
        esc(&i.text),
        i.link
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(esc),
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

    fn linked(kind: &str, text: &str, href: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: Some(href.into()),
            tone: None,
            config: Default::default(),
        }
    }

    fn bar(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("pill-nav", items.first().copied().unwrap_or("Home"));
        c.items.clear();
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
        assert!(!html.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("nav("));
    }

    #[test]
    fn root_is_nav_with_items_first_current_not_surf() {
        let html = render(&bar(&["Home", "Docs"]));
        assert!(html.starts_with("<nav data-slot=\"pill-nav\">"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"pill-nav-item\" aria-current=\"page\">Home</button>"
        ));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"pill-nav-item\">Docs</button>"
        ));
        assert_eq!(html.matches("data-slot=\"pill-nav-item\"").count(), 2);
        assert_eq!(html.matches("aria-current=\"page\"").count(), 1);
        reject_interact(&html);
        assert_eq!(
            html,
            "<nav data-slot=\"pill-nav\"><button type=\"button\" data-slot=\"pill-nav-item\" aria-current=\"page\">Home</button><button type=\"button\" data-slot=\"pill-nav-item\">Docs</button></nav>"
        );
    }

    #[test]
    fn linked_item_is_anchor() {
        let mut c = stub("pill-nav", "Home");
        c.items.clear();
        c.items.push(linked("item", "Home", "/"));
        c.items.push(linked("item", "Docs", "/docs"));
        let html = render(&c);
        assert!(html.contains(
            "<a data-slot=\"pill-nav-item\" href=\"/\" aria-current=\"page\">Home</a>"
        ));
        assert!(html.contains("<a data-slot=\"pill-nav-item\" href=\"/docs\">Docs</a>"));
        assert!(!html.contains("<button"));
        reject_interact(&html);
    }

    #[test]
    fn extra_text_items_become_items() {
        let mut c = stub("pill-nav", "Home");
        c.items.push(extra("text", "Docs"));
        c.items.push(extra("text", "Blog"));
        let html = render(&c);
        assert!(html.contains("aria-current=\"page\">Home</button>"));
        assert!(html.contains("data-slot=\"pill-nav-item\">Docs</button>"));
        assert!(html.contains("data-slot=\"pill-nav-item\">Blog</button>"));
        assert_eq!(html.matches("data-slot=\"pill-nav-item\"").count(), 3);
        reject_interact(&html);
    }

    #[test]
    fn field_label_is_not_an_item_when_items_exist() {
        let mut c = stub("pill-nav", "Menus");
        c.items.push(extra("item", "Home"));
        c.items.push(extra("item", "Docs"));
        let html = render(&c);
        assert!(html.contains("aria-current=\"page\">Home</button>"));
        assert!(html.contains("data-slot=\"pill-nav-item\">Docs</button>"));
        assert!(!html.contains(">Menus</button>"));
        assert_eq!(html.matches("data-slot=\"pill-nav-item\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_current_item() {
        let html = render(&stub("pill-nav", "Home"));
        assert!(html.contains("<nav data-slot=\"pill-nav\">"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"pill-nav-item\" aria-current=\"page\">Home</button>"
        ));
        assert_eq!(html.matches("data-slot=\"pill-nav-item\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_nav_surf() {
        let c = bar(&["Home", "Docs"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("pill-nav", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<nav data-slot=\"pill-nav\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!interact.contains("data-slot=\"pill-nav-item\""));
        assert!(html.contains("data-slot=\"pill-nav-item\""));
        assert!(html.contains("aria-current=\"page\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&bar(&["Home"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"pill-nav\"]"));
        assert!(css.contains("[data-slot=\"pill-nav-item\"]"));
        assert!(css.contains("[data-slot=\"pill-nav-item\"][aria-current=\"page\"]"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("border-radius: 9999px"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
