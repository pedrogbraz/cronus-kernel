//! Dedicated Dock renderer. DOM mirrors React:
//! `<div data-slot="dock" aria-label>` plus each entry as
//! `<a data-slot="dock-item">` when the item has a link, otherwise
//! `<button type="button" data-slot="dock-item">`. Items are icon buttons: the
//! name lives in `title`/`aria-label` and the body is the decorative `GLYPH`
//! (same circle React's dock fixture passes as `icon`), never visible text.
//! The `label` names the dock; it is only an entry when nothing else exists.
//! Not interact `nav("dock")` (generic SURF `<nav>` without dock-item).

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let items = nav_entries(comp)
        .into_iter()
        .map(|(text, href)| item_html(&text, href.as_deref()))
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"dock\"{}>{items}</div>", aria_attr(comp))
}

/// Decorative glyph React's dock fixture passes as `icon`. The accessible name
/// lives only in `title` / `aria-label`; the button shows no visible text.
const GLYPH: &str = "<span aria-hidden=\"true\"><svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" aria-hidden=\"true\"><circle cx=\"12\" cy=\"12\" r=\"7\"></circle></svg></span>";

fn item_html(text: &str, href: Option<&str>) -> String {
    match href {
        Some(h) => format!("<a data-slot=\"dock-item\" href=\"{h}\" title=\"{text}\" aria-label=\"{text}\">{GLYPH}</a>"),
        None => format!(
            "<button type=\"button\" data-slot=\"dock-item\" title=\"{text}\" aria-label=\"{text}\">{GLYPH}</button>"
        ),
    }
}

/// `aria-label:` after an item line lands in that item's config (the
/// tokenizer has no newlines), so look there as well as in props.
fn aria_attr(comp: &ComponentNode) -> String {
    let v = comp
        .props
        .get("aria-label")
        .or_else(|| comp.items.iter().find_map(|i| i.config.get("aria-label")));
    match v.filter(|s| !s.is_empty()) {
        Some(v) => format!(" aria-label=\"{}\"", esc(v)),
        None => String::new(),
    }
}

fn nav_entries(comp: &ComponentNode) -> Vec<(String, Option<String>)> {
    let choice: Vec<(String, Option<String>)> = comp
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
    let text_lines: Vec<(String, Option<String>)> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "text" && !i.text.is_empty())
        .map(entry_of)
        .collect();
    if !text_lines.is_empty() {
        return text_lines;
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
        i.link.as_deref().filter(|s| !s.is_empty()).map(esc),
    )
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
        assert!(!html.contains("<nav"));
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
    fn root_is_div_with_items_like_react() {
        let html = render(&bar(&["Home", "Search"]));
        assert_eq!(
            html,
            format!("<div data-slot=\"dock\"><button type=\"button\" data-slot=\"dock-item\" title=\"Home\" aria-label=\"Home\">{GLYPH}</button><button type=\"button\" data-slot=\"dock-item\" title=\"Search\" aria-label=\"Search\">{GLYPH}</button></div>")
        );
        reject_interact(&html);
    }

    /// Audit fixture: `label "App dock"`, `text` per item, `aria-label:` on the
    /// last text. React: `<div data-slot="dock" aria-label="App dock">` with
    /// Home/Search only.
    #[test]
    fn fixture_label_names_dock_not_an_item() {
        let mut c = stub("dock", "App dock");
        c.items.push(extra("text", "Home"));
        let mut search = extra("text", "Search");
        search.config.insert("aria-label".into(), "App dock".into());
        c.items.push(search);
        let html = render(&c);
        assert_eq!(
            html,
            format!("<div data-slot=\"dock\" aria-label=\"App dock\"><button type=\"button\" data-slot=\"dock-item\" title=\"Home\" aria-label=\"Home\">{GLYPH}</button><button type=\"button\" data-slot=\"dock-item\" title=\"Search\" aria-label=\"Search\">{GLYPH}</button></div>")
        );
        reject_interact(&html);
    }

    #[test]
    fn linked_item_is_anchor() {
        let mut c = stub("dock", "Home");
        c.items.clear();
        c.items.push(linked("item", "Home", "/"));
        c.items.push(linked("item", "Search", "/search"));
        let html = render(&c);
        assert!(html.contains(&format!(
            "<a data-slot=\"dock-item\" href=\"/\" title=\"Home\" aria-label=\"Home\">{GLYPH}</a>"
        )));
        assert!(html.contains(
            &format!("<a data-slot=\"dock-item\" href=\"/search\" title=\"Search\" aria-label=\"Search\">{GLYPH}</a>")
        ));
        assert!(!html.contains("<button"));
        reject_interact(&html);
    }

    #[test]
    fn extra_text_items_become_items_label_does_not() {
        let mut c = stub("dock", "Home");
        c.items.push(extra("text", "Search"));
        c.items.push(extra("text", "Settings"));
        let html = render(&c);
        assert!(!html.contains("title=\"Home\""));
        assert!(html.contains("data-slot=\"dock-item\" title=\"Search\""));
        assert!(html.contains("data-slot=\"dock-item\" title=\"Settings\""));
        assert_eq!(html.matches("data-slot=\"dock-item\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn field_label_is_not_an_item_when_items_exist() {
        let mut c = stub("dock", "Apps");
        c.items.push(extra("item", "Home"));
        c.items.push(extra("item", "Search"));
        let html = render(&c);
        assert!(html.contains(&format!("aria-label=\"Home\">{GLYPH}</button>")));
        assert!(html.contains(&format!("aria-label=\"Search\">{GLYPH}</button>")));
        assert!(!html.contains(">Apps</button>"));
        assert_eq!(html.matches("data-slot=\"dock-item\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_one_item() {
        let html = render(&stub("dock", "Home"));
        assert!(html.starts_with("<div data-slot=\"dock\">"));
        assert!(html.contains(
            &format!("<button type=\"button\" data-slot=\"dock-item\" title=\"Home\" aria-label=\"Home\">{GLYPH}</button>")
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
        // Wave 1s geometry parity (React measured: dock 122x62 r22, item 44x44 r18, svg 22).
        assert!(css.contains("border-radius: calc(var(--cronus-radius, 14px) + 8px);\n  border: 1px solid var(--cronus-border);"));
        assert!(css.contains(
            "background: color-mix(in oklab, var(--cronus-surface-raised) 70%, transparent);"
        ));
        assert!(css.contains("-webkit-backdrop-filter: blur(8px); backdrop-filter: blur(8px);"));
        assert!(css.contains("border: 0; border-radius: var(--cronus-radius-xl); outline: none;\n  background: var(--cronus-surface-overlay); color: var(--cronus-fg);"));
        assert!(css.contains("[data-slot=\"dock-item\"] > span { display: contents; }"));
        assert!(css.contains("[data-slot=\"dock-item\"] svg { width: 50%; height: 50%;"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }

    /// React dock items are icon buttons: the name is only in title/aria-label.
    #[test]
    fn items_show_glyph_not_visible_text() {
        let html = render(&bar(&["Home", "Search"]));
        assert_eq!(html.matches(GLYPH).count(), 2);
        assert!(!html.contains(">Home<"));
        assert!(!html.contains(">Search<"));
        assert!(!html.contains("<svg style"));
        reject_interact(&html);
    }
}
