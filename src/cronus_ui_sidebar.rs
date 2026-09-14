//! Dedicated Sidebar renderer. Desktop DOM matches React `SidebarProvider` +
//! `Sidebar collapsible="none"`:
//! `<div data-slot="sidebar-wrapper"><aside data-slot="sidebar"><nav>`
//! `<div data-slot="sidebar-content"><div data-slot="scroll-area"><div>`
//! `<ul data-slot="sidebar-menu">` plus each item as
//! `<li data-slot="sidebar-menu-item"><button data-slot="sidebar-menu-button">`
//! (`<a>` when the item has a link). First item is active. A link-less button would
//! need JS to navigate, so (wave 1t rule) it is a native `disabled` button, not dimmed.
//! The `label` names the widget (nav `aria-label`); it is never a menu item.
//! Not interact `nav("sidebar")` (generic SURF `<nav>` without sidebar-content/menu).

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let aria = aria_label(comp).unwrap_or_else(|| label_of(comp));
    let mut entries = menu_entries(comp);
    if entries.is_empty() {
        entries.push((label_of(comp), None));
    }
    format!(
        "<div data-slot=\"sidebar-wrapper\" data-state=\"expanded\">{}</div>",
        aside(&aria, &entries)
    )
}

/// `<aside data-slot="sidebar">` subtree. Shared with the AppShell renderer,
/// which composes the same Sidebar like React `AppShell` does.
pub fn aside(aria: &str, entries: &[(String, Option<String>)]) -> String {
    let items = entries
        .iter()
        .enumerate()
        .map(|(i, (text, href))| {
            let state = if i == 0 {
                " data-active=\"true\" aria-current=\"page\""
            } else {
                " data-active=\"false\""
            };
            let button = match href {
                Some(h) => format!("<a data-slot=\"sidebar-menu-button\" href=\"{h}\"{state}>{text}</a>"),
                None => format!(
                    "<button type=\"button\" data-slot=\"sidebar-menu-button\"{state} disabled>{text}</button>"
                ),
            };
            format!("<li data-slot=\"sidebar-menu-item\">{button}</li>")
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<aside data-slot=\"sidebar\" data-variant=\"sidebar\" data-side=\"left\"><nav aria-label=\"{aria}\"><div data-slot=\"sidebar-content\"><div data-slot=\"scroll-area\"><div><ul data-slot=\"sidebar-menu\">{items}</ul></div></div></div></nav></aside>"
    )
}

/// Menu entries: choice items, else every non-label text item. No fallback.
pub fn menu_entries(comp: &ComponentNode) -> Vec<(String, Option<String>)> {
    let choices: Vec<_> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "item" | "tab" | "columns") && !i.text.is_empty())
        .map(entry_of)
        .collect();
    if !choices.is_empty() {
        return choices;
    }
    comp.items
        .iter()
        .filter(|i| !matches!(i.item_type.as_str(), "label" | "title") && !i.text.is_empty())
        .map(entry_of)
        .collect()
}

fn entry_of(i: &ComponentItemNode) -> (String, Option<String>) {
    (
        esc(&i.text),
        i.link.as_deref().filter(|s| !s.is_empty()).map(esc),
    )
}

/// `aria-label` from props or item config (`key:value` lines attach to the previous item).
pub fn aria_label(comp: &ComponentNode) -> Option<String> {
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

    /// Shape the audit emitter writes: `label`, `text` per item, trailing `aria-label:`.
    fn emitted(label: &str, items: &[&str]) -> ComponentNode {
        let mut c = stub("sidebar", label);
        for n in items {
            c.items.push(extra("text", n));
        }
        if let Some(last) = c.items.last_mut() {
            last.config.insert("aria-label".into(), label.into());
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<nav data-slot=\"sidebar\""));
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
    fn emitted_fixture_matches_react_dom_and_label_is_not_an_item() {
        let html = render(&emitted("Sidebar", &["Home", "Inbox"]));
        assert_eq!(
            html,
            "<div data-slot=\"sidebar-wrapper\" data-state=\"expanded\"><aside data-slot=\"sidebar\" data-variant=\"sidebar\" data-side=\"left\"><nav aria-label=\"Sidebar\"><div data-slot=\"sidebar-content\"><div data-slot=\"scroll-area\"><div><ul data-slot=\"sidebar-menu\"><li data-slot=\"sidebar-menu-item\"><button type=\"button\" data-slot=\"sidebar-menu-button\" data-active=\"true\" aria-current=\"page\" disabled>Home</button></li><li data-slot=\"sidebar-menu-item\"><button type=\"button\" data-slot=\"sidebar-menu-button\" data-active=\"false\" disabled>Inbox</button></li></ul></div></div></div></nav></aside></div>"
        );
        assert!(!html.contains(">Sidebar</button>"));
        reject_interact(&html);
    }

    #[test]
    fn linked_item_is_anchor() {
        let mut c = stub("sidebar", "Nav");
        let mut home = extra("item", "Home");
        home.link = Some("/".into());
        c.items.push(home);
        let html = render(&c);
        assert!(html.contains(
            "<a data-slot=\"sidebar-menu-button\" href=\"/\" data-active=\"true\" aria-current=\"page\">Home</a>"
        ));
        assert!(html.contains("<nav aria-label=\"Nav\">"));
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_one_item() {
        let html = render(&stub("sidebar", "Home"));
        assert_eq!(html.matches("data-slot=\"sidebar-menu-item\"").count(), 1);
        assert!(html.contains("data-active=\"true\" aria-current=\"page\" disabled>Home</button>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_nav_surf() {
        let c = emitted("Sidebar", &["Home", "Settings"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("sidebar", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<nav data-slot=\"sidebar\""));
        assert!(!interact.contains("data-slot=\"sidebar-content\""));
        assert!(html.contains("<aside data-slot=\"sidebar\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            reject_interact(&render(&emitted("Sidebar", &["Home"])));
        });
    }

    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"sidebar-wrapper\"]"));
        assert!(css.contains("[data-audit-canvas] > [data-slot=\"sidebar-wrapper\"]:not([data-app-shell])"));
        assert!(css.contains("width: 13rem; height: 14rem; min-height: 0;"));
        assert!(css.contains("[data-slot=\"sidebar-content\"] > [data-slot=\"scroll-area\"]"));
        assert!(css.contains("[data-slot=\"sidebar-menu-button\"][data-active=\"true\"]"));
        assert!(css.contains("font-size: 0.875rem; line-height: 1.25rem;"));
        assert!(css.contains("width: 16rem"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(!css.contains("zinc-"));
    }
}
