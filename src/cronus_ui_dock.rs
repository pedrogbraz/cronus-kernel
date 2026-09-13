//! Dedicated Dock renderer. DOM:
//! `<nav data-slot="dock">` plus each text as
//! `<a data-slot="dock-item">` when the item has a link, otherwise
//! `<button type="button" data-slot="dock-item">`.
//! Not interact `nav("dock")` (generic SURF `<nav>` without dock-item).

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let items = nav_entries(comp)
        .into_iter()
        .map(|(text, href)| item_html(&text, href.as_deref()))
        .collect::<Vec<_>>()
        .join("");
    format!("<nav data-slot=\"dock\">{items}</nav>")
}

fn item_html(text: &str, href: Option<&str>) -> String {
    match href {
        Some(h) => format!("<a data-slot=\"dock-item\" href=\"{h}\" title=\"{text}\" aria-label=\"{text}\">{text}</a>"),
        None => format!(
            "<button type=\"button\" data-slot=\"dock-item\" title=\"{text}\" aria-label=\"{text}\">{text}</button>"
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
        let mut c = stub("dock", items.first().copied().unwrap_or("Home"));
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
    fn root_is_nav_with_items_not_surf() {
        let html = render(&bar(&["Home", "Search"]));
        assert!(html.starts_with("<nav data-slot=\"dock\">"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"dock-item\" title=\"Home\" aria-label=\"Home\">Home</button>"
        ));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"dock-item\" title=\"Search\" aria-label=\"Search\">Search</button>"
        ));
        assert_eq!(html.matches("data-slot=\"dock-item\"").count(), 2);
        reject_interact(&html);
        assert_eq!(
            html,
            "<nav data-slot=\"dock\"><button type=\"button\" data-slot=\"dock-item\" title=\"Home\" aria-label=\"Home\">Home</button><button type=\"button\" data-slot=\"dock-item\" title=\"Search\" aria-label=\"Search\">Search</button></nav>"
        );
    }

    #[test]
    fn linked_item_is_anchor() {
        let mut c = stub("dock", "Home");
        c.items.clear();
        c.items.push(linked("item", "Home", "/"));
        c.items.push(linked("item", "Search", "/search"));
        let html = render(&c);
        assert!(html.contains(
            "<a data-slot=\"dock-item\" href=\"/\" title=\"Home\" aria-label=\"Home\">Home</a>"
        ));
        assert!(html.contains(
            "<a data-slot=\"dock-item\" href=\"/search\" title=\"Search\" aria-label=\"Search\">Search</a>"
        ));
        assert!(!html.contains("<button"));
        reject_interact(&html);
    }

    #[test]
    fn extra_text_items_become_items() {
        let mut c = stub("dock", "Home");
        c.items.push(extra("text", "Search"));
        c.items.push(extra("text", "Settings"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"dock-item\" title=\"Home\""));
        assert!(html.contains("data-slot=\"dock-item\" title=\"Search\""));
        assert!(html.contains("data-slot=\"dock-item\" title=\"Settings\""));
        assert_eq!(html.matches("data-slot=\"dock-item\"").count(), 3);
        reject_interact(&html);
    }

    #[test]
    fn field_label_is_not_an_item_when_items_exist() {
        let mut c = stub("dock", "Apps");
        c.items.push(extra("item", "Home"));
        c.items.push(extra("item", "Search"));
        let html = render(&c);
        assert!(html.contains("aria-label=\"Home\">Home</button>"));
        assert!(html.contains("aria-label=\"Search\">Search</button>"));
        assert!(!html.contains(">Apps</button>"));
        assert_eq!(html.matches("data-slot=\"dock-item\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_one_item() {
        let html = render(&stub("dock", "Home"));
        assert!(html.contains("<nav data-slot=\"dock\">"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"dock-item\" title=\"Home\" aria-label=\"Home\">Home</button>"
        ));
        assert_eq!(html.matches("data-slot=\"dock-item\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_nav_surf() {
        let c = bar(&["Home", "Search"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("dock", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<nav data-slot=\"dock\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!interact.contains("data-slot=\"dock-item\""));
        assert!(html.contains("data-slot=\"dock-item\""));
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
        assert!(css.contains("[data-slot=\"dock\"]"));
        assert!(css.contains("[data-slot=\"dock-item\"]"));
        assert!(css.contains("display: inline-flex"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("width: 2.75rem; height: 2.75rem"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
