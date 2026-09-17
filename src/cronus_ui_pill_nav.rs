//! Dedicated PillNav renderer. DOM matches React:
//! `<nav data-slot="pill-nav" aria-label>` plus one `pill-nav-item` per
//! item. Items with a link are `<a data-slot="pill-nav-item">` (the first is
//! `aria-current="page"`, its pill a CSS `::before`). Link-less items switch
//! with zero JS: each item is a `<label>` holding a visually hidden radio
//! (page-unique `name`, `value:` picks the checked one, `disabled:true`
//! disables it; ArrowLeft/ArrowRight move like React's roving tabindex) and
//! React's `<button>` as a decorative twin (`tabindex="-1"`,
//! `aria-hidden`). React's sliding thumb (`motion.span layoutId`, spring
//! bounce 0.15 / 0.4s) is one shared `<span class="pill">` anchored to the
//! checked item (`anchor-name` / `position-anchor`) and eased with
//! `--cronus-spring-snappy`. The `label` names the nav (it is never an item).

use crate::cronus_ui_kit::{attr, attr_nonempty, esc, instance_id, label_of, safe_url, truthy};
use crate::parser::{ComponentItemNode, ComponentNode};

struct Entry {
    text: String,
    href: Option<String>,
    disabled: bool,
}

pub fn render(comp: &ComponentNode) -> String {
    let entries = nav_entries(comp);
    let aria = aria_label(comp)
        .map(|a| format!(" aria-label=\"{a}\""))
        .unwrap_or_default();
    if entries.iter().any(|e| e.href.is_some()) {
        let items = entries
            .iter()
            .enumerate()
            .map(|(i, e)| link_html(e, i == 0))
            .collect::<String>();
        return format!("<nav data-slot=\"pill-nav\"{aria}>{items}</nav>");
    }
    let selected = attr(comp, "value")
        .map(esc)
        .and_then(|v| entries.iter().position(|e| e.text == v))
        .unwrap_or(0);
    let name = instance_id(comp, "pill-nav");
    let items = entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let checked = if i == selected { " checked" } else { "" };
            let disabled = if e.disabled { " disabled" } else { "" };
            format!(
                "<label><input type=\"radio\" name=\"{name}\" value=\"{i}\" aria-label=\"{t}\"{checked}{disabled}><button type=\"button\" data-slot=\"pill-nav-item\" tabindex=\"-1\" aria-hidden=\"true\">{t}</button></label>",
                t = e.text
            )
        })
        .collect::<String>();
    format!(
        "<nav data-slot=\"pill-nav\"{aria}>{items}<span aria-hidden=\"true\" class=\"pill\"></span></nav>"
    )
}

fn link_html(e: &Entry, current: bool) -> String {
    let current_attr = if current {
        " aria-current=\"page\""
    } else {
        ""
    };
    match &e.href {
        Some(h) => format!(
            "<a data-slot=\"pill-nav-item\" href=\"{h}\"{current_attr}>{}</a>",
            e.text
        ),
        None => format!(
            "<button type=\"button\" data-slot=\"pill-nav-item\"{current_attr} disabled>{}</button>",
            e.text
        ),
    }
}

fn nav_entries(comp: &ComponentNode) -> Vec<Entry> {
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
        vec![Entry {
            text: label_of(comp),
            href: None,
            disabled: false,
        }]
    } else {
        body
    }
}

fn entry_of(i: &ComponentItemNode) -> Entry {
    Entry {
        text: esc(&i.text),
        href: i.link.as_deref().filter(|s| !s.is_empty()).map(safe_url),
        disabled: i.config.get("disabled").is_some_and(|v| truthy(v)),
    }
}

fn aria_label(comp: &ComponentNode) -> Option<String> {
    attr_nonempty(comp, "aria-label").map(|s| esc(s))
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
        assert!(!html.contains("onkeydown="));
        assert!(!html.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
    }

    /// Audit fixture: the label is the nav name; items become radio + decorative button
    /// pairs and the shared pill sits last.
    #[test]
    fn emitted_fixture_label_is_aria_not_an_item() {
        crate::cronus_ui_kit::reset_instance_ids();
        let html = render(&emitted("Sections", &["Home", "Work"]));
        assert_eq!(
            html,
            "<nav data-slot=\"pill-nav\" aria-label=\"Sections\"><label><input type=\"radio\" name=\"cui-pill-nav-pill-nav\" value=\"0\" aria-label=\"Home\" checked><button type=\"button\" data-slot=\"pill-nav-item\" tabindex=\"-1\" aria-hidden=\"true\">Home</button></label><label><input type=\"radio\" name=\"cui-pill-nav-pill-nav\" value=\"1\" aria-label=\"Work\"><button type=\"button\" data-slot=\"pill-nav-item\" tabindex=\"-1\" aria-hidden=\"true\">Work</button></label><span aria-hidden=\"true\" class=\"pill\"></span></nav>"
        );
        assert!(!html.contains(" disabled"));
        reject_interact(&html);
    }

    /// Docs example: `value:` picks the checked item, `disabled:true` disables one.
    #[test]
    fn value_selects_and_disabled_items_are_disabled_radios() {
        let mut c = stub("pill-nav", "Product sections");
        c.items.clear();
        c.items.push(extra("item", "Overview"));
        c.items.push(extra("item", "Pricing"));
        let mut docs = extra("item", "Docs");
        docs.config.insert("disabled".into(), "true".into());
        c.items.push(docs);
        c.props.insert("value".into(), "Pricing".into());
        c.props
            .insert("aria-label".into(), "Product sections".into());
        let html = render(&c);
        assert!(html.starts_with("<nav data-slot=\"pill-nav\" aria-label=\"Product sections\">"));
        assert!(html.contains("value=\"1\" aria-label=\"Pricing\" checked>"));
        assert!(!html.contains("value=\"0\" aria-label=\"Overview\" checked"));
        assert!(html.contains("value=\"2\" aria-label=\"Docs\" disabled>"));
        assert_eq!(html.matches("data-slot=\"pill-nav-item\"").count(), 3);
        reject_interact(&html);
    }

    #[test]
    fn linked_item_is_anchor() {
        let mut c = stub("pill-nav", "Home");
        c.items.clear();
        let mut home = extra("item", "Home");
        home.link = Some("/".into());
        c.items.push(home);
        c.items.push(extra("item", "Docs"));
        let html = render(&c);
        assert!(html
            .contains("<a data-slot=\"pill-nav-item\" href=\"/\" aria-current=\"page\">Home</a>"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"pill-nav-item\" disabled>Docs</button>"
        ));
        assert!(!html.contains("<input"));
        assert!(!html.contains("class=\"pill\""));
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_current_item() {
        crate::cronus_ui_kit::reset_instance_ids();
        let html = render(&stub("pill-nav", "Home"));
        assert_eq!(
            html,
            "<nav data-slot=\"pill-nav\"><label><input type=\"radio\" name=\"cui-pill-nav-pill-nav\" value=\"0\" aria-label=\"Home\" checked><button type=\"button\" data-slot=\"pill-nav-item\" tabindex=\"-1\" aria-hidden=\"true\">Home</button></label><span aria-hidden=\"true\" class=\"pill\"></span></nav>"
        );
    }

    #[test]
    fn two_navs_on_one_page_get_distinct_names() {
        crate::cronus_ui_kit::reset_instance_ids();
        let a = render(&stub("pill-nav", "Home"));
        let b = render(&stub("pill-nav", "Home"));
        assert!(a.contains("name=\"cui-pill-nav-pill-nav\" "));
        assert!(b.contains("name=\"cui-pill-nav-pill-nav-2\" "));
    }

    #[test]
    fn skips_interact_nav_surf() {
        let c = emitted("Sections", &["Home", "Docs"]);
        let html = render(&c);
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
        assert!(
            css.contains("[data-slot=\"pill-nav\"] {\n  position: relative; display: inline-flex;")
        );
        assert!(css.contains("anchor-scope: --cui-pill;"));
        assert!(css.contains("[data-slot=\"pill-nav-item\"][aria-current=\"page\"]::before"));
        assert!(css.contains("padding: 0.375rem 0.875rem;"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        // Zero JS: the checked radio names the anchor, the shared pill follows it with a spring.
        assert!(css.contains("[data-slot=\"pill-nav\"] > label > input:checked + [data-slot=\"pill-nav-item\"] {\n  color: var(--cronus-fg); anchor-name: --cui-pill;\n}"));
        assert!(
            css.contains("[data-slot=\"pill-nav\"] > .pill {\n  position: absolute; z-index: 0;")
        );
        assert!(css.contains("position-anchor: --cui-pill;"));
        assert!(css.contains("inset-inline-start: anchor(start); top: anchor(top);\n  width: anchor-size(width); height: anchor-size(height);"));
        assert!(css.contains("transition: inset-inline-start var(--cronus-spring-snappy), width var(--cronus-spring-snappy);"));
        assert!(css.contains("[data-slot=\"pill-nav\"] > label > input:disabled + [data-slot=\"pill-nav-item\"] { opacity: 0.5; }"));
        assert!(css.contains("[data-slot=\"pill-nav\"] > label > input:focus-visible + [data-slot=\"pill-nav-item\"]"));
        assert!(!css.contains("zinc-"));
    }
}
