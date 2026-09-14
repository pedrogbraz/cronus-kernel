//! Dedicated PillNav renderer. DOM matches React:
//! `<nav data-slot="pill-nav" aria-label>` plus each item as
//! `<a data-slot="pill-nav-item">` when the item has a link, otherwise
//! `<button type="button" data-slot="pill-nav-item">`. First item is current;
//! its pill is a CSS `::before` (React: an absolutely positioned `<span>`), so
//! the item box itself stays transparent like React.
//! The `label` names the nav (it is never an item). Switching a link-less item needs
//! JS, so (wave 1t rule) such items are native `disabled` buttons, not visually dimmed.
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
    let aria = aria_label(comp)
        .map(|a| format!(" aria-label=\"{a}\""))
        .unwrap_or_default();
    format!("<nav data-slot=\"pill-nav\"{aria}>{items}</nav>")
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
            "<button type=\"button\" data-slot=\"pill-nav-item\"{current_attr} disabled>{text}</button>"
        ),
    }
}

fn nav_entries(comp: &ComponentNode) -> Vec<(String, Option<String>)> {
    let choice: Vec<_> = comp
        .items
        .iter()
        .filter(|i| {
            matches!(i.item_type.as_str(), "item" | "tab" | "columns") && !i.text.is_empty()
        })
        .map(entry_of)
        .collect();
    if !choice.is_empty() {
        return choice;
    }
    let body: Vec<_> = comp
        .items
        .iter()
        .filter(|i| !matches!(i.item_type.as_str(), "label" | "title") && !i.text.is_empty())
        .map(entry_of)
        .collect();
    if body.is_empty() {
        vec![(label_of(comp), None)]
    } else {
        body
    }
}

fn entry_of(i: &ComponentItemNode) -> (String, Option<String>) {
    (
        esc(&i.text),
        i.link.as_deref().filter(|s| !s.is_empty()).map(esc),
    )
}

fn aria_label(comp: &ComponentNode) -> Option<String> {
    comp.props
        .get("aria-label")
        .or_else(|| comp.items.iter().find_map(|i| i.config.get("aria-label")))
        .filter(|s| !s.is_empty())
        .map(|s| esc(s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn emitted(label: &str, items: &[&str]) -> ComponentNode {
        let mut c = stub("pill-nav", label);
        for n in items {
            c.items.push(extra("text", n));
        }
        if let Some(last) = c.items.last_mut() {
            last.config.insert("aria-label".into(), label.into());
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<details"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn emitted_fixture_label_is_aria_not_an_item() {
        let html = render(&emitted("Sections", &["Home", "Work"]));
        assert_eq!(
            html,
            "<nav data-slot=\"pill-nav\" aria-label=\"Sections\"><button type=\"button\" data-slot=\"pill-nav-item\" aria-current=\"page\" disabled>Home</button><button type=\"button\" data-slot=\"pill-nav-item\" disabled>Work</button></nav>"
        );
        reject_interact(&html);
    }

    #[test]
    fn linked_item_is_anchor() {
        let mut c = stub("pill-nav", "Home");
        c.items.clear();
        let mut home = extra("item", "Home");
        home.link = Some("/".into());
        c.items.push(home);
        let html = render(&c);
        assert!(html
            .contains("<a data-slot=\"pill-nav-item\" href=\"/\" aria-current=\"page\">Home</a>"));
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_current_item() {
        let html = render(&stub("pill-nav", "Home"));
        assert_eq!(
            html,
            "<nav data-slot=\"pill-nav\"><button type=\"button\" data-slot=\"pill-nav-item\" aria-current=\"page\" disabled>Home</button></nav>"
        );
    }

    #[test]
    fn skips_interact_nav_surf() {
        let c = emitted("Sections", &["Home", "Docs"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("pill-nav", &c).unwrap();
        assert_ne!(html, interact);
        assert!(!interact.contains("data-slot=\"pill-nav-item\""));
        assert!(html.contains("data-slot=\"pill-nav-item\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            reject_interact(&render(&emitted("Sections", &["Home"])));
        });
    }

    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"pill-nav\"] {\n  display: inline-flex;"));
        assert!(css.contains("[data-slot=\"pill-nav-item\"][aria-current=\"page\"]::before"));
        assert!(css.contains("padding: 0.375rem 0.875rem;"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(!css.contains("zinc-"));
    }
}
