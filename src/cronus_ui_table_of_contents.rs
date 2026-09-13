//! Dedicated TableOfContents renderer. DOM matches React:
//! `<nav data-slot="table-of-contents"><ul data-slot="table-of-contents-list">`
//! plus each text as `<a data-slot="table-of-contents-link">`.
//! Not interact `nav("table-of-contents")` (generic SURF `<nav>` without list/link).

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let links = toc_entries(comp)
        .into_iter()
        .map(|(text, href)| {
            format!(
                "<li><a data-slot=\"table-of-contents-link\" href=\"{href}\">{text}</a></li>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<nav data-slot=\"table-of-contents\" aria-label=\"On this page\"><ul data-slot=\"table-of-contents-list\">{links}</ul></nav>"
    )
}

fn toc_entries(comp: &ComponentNode) -> Vec<(String, String)> {
    let choice: Vec<(String, String)> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "item" | "tab" | "columns") && !i.text.is_empty())
        .map(entry_of)
        .collect();
    if !choice.is_empty() {
        return choice;
    }
    let other: Vec<(String, String)> = comp
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
    let all: Vec<(String, String)> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty())
        .map(entry_of)
        .collect();
    if all.is_empty() {
        let label = label_of(comp);
        let href = hash_href(None, &label);
        vec![(label, href)]
    } else {
        all
    }
}

fn entry_of(i: &ComponentItemNode) -> (String, String) {
    let text = esc(&i.text);
    let href = hash_href(i.link.as_deref(), &i.text);
    (text, href)
}

fn hash_href(link: Option<&str>, text: &str) -> String {
    if let Some(h) = link.map(str::trim).filter(|s| !s.is_empty()) {
        return esc(h);
    }
    let slug = slugify(text);
    if slug.is_empty() {
        "#".into()
    } else {
        format!("#{slug}")
    }
}

fn slugify(s: &str) -> String {
    let mut out = String::new();
    for c in s.trim().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if c.is_whitespace() || c == '-' || c == '_' {
            if !out.is_empty() && !out.ends_with('-') {
                out.push('-');
            }
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
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
        let mut c = stub("table-of-contents", items.first().copied().unwrap_or("Intro"));
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
    fn root_is_nav_list_with_links_not_surf() {
        let html = render(&bar(&["Intro", "API"]));
        assert!(html.starts_with("<nav data-slot=\"table-of-contents\""));
        assert!(html.contains("<ul data-slot=\"table-of-contents-list\">"));
        assert!(html.contains(
            "<li><a data-slot=\"table-of-contents-link\" href=\"#intro\">Intro</a></li>"
        ));
        assert!(html.contains(
            "<li><a data-slot=\"table-of-contents-link\" href=\"#api\">API</a></li>"
        ));
        assert_eq!(html.matches("data-slot=\"table-of-contents-link\"").count(), 2);
        reject_interact(&html);
        assert_eq!(
            html,
            "<nav data-slot=\"table-of-contents\" aria-label=\"On this page\"><ul data-slot=\"table-of-contents-list\"><li><a data-slot=\"table-of-contents-link\" href=\"#intro\">Intro</a></li><li><a data-slot=\"table-of-contents-link\" href=\"#api\">API</a></li></ul></nav>"
        );
    }

    #[test]
    fn extra_text_items_become_links() {
        let mut c = stub("table-of-contents", "Intro");
        c.items.push(extra("text", "API"));
        c.items.push(extra("text", "Deploy"));
        let html = render(&c);
        assert!(html.contains("href=\"#intro\">Intro</a>"));
        assert!(html.contains("href=\"#api\">API</a>"));
        assert!(html.contains("href=\"#deploy\">Deploy</a>"));
        assert_eq!(html.matches("data-slot=\"table-of-contents-link\"").count(), 3);
        reject_interact(&html);
    }

    #[test]
    fn field_label_is_not_a_link_when_items_exist() {
        let mut c = stub("table-of-contents", "On this page");
        c.items.push(extra("item", "Intro"));
        c.items.push(extra("item", "API"));
        let html = render(&c);
        assert!(html.contains("href=\"#intro\">Intro</a>"));
        assert!(html.contains("href=\"#api\">API</a>"));
        assert!(!html.contains(">On this page</a>"));
        assert_eq!(html.matches("data-slot=\"table-of-contents-link\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_one_link() {
        let html = render(&stub("table-of-contents", "Intro"));
        assert!(html.contains("<nav data-slot=\"table-of-contents\""));
        assert!(html.contains("<ul data-slot=\"table-of-contents-list\">"));
        assert!(html.contains(
            "<li><a data-slot=\"table-of-contents-link\" href=\"#intro\">Intro</a></li>"
        ));
        assert_eq!(html.matches("data-slot=\"table-of-contents-link\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn linked_item_keeps_href() {
        let mut c = stub("table-of-contents", "Nav");
        c.items.clear();
        c.items.push(linked("item", "Intro", "#getting-started"));
        let html = render(&c);
        assert!(html.contains(
            "<a data-slot=\"table-of-contents-link\" href=\"#getting-started\">Intro</a>"
        ));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_nav_surf() {
        let c = bar(&["Intro", "API"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("table-of-contents", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<nav data-slot=\"table-of-contents\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!interact.contains("data-slot=\"table-of-contents-list\""));
        assert!(!interact.contains("data-slot=\"table-of-contents-link\""));
        assert!(html.contains("data-slot=\"table-of-contents-list\""));
        assert!(html.contains("data-slot=\"table-of-contents-link\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&bar(&["Intro"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"table-of-contents\"]"));
        assert!(css.contains("[data-slot=\"table-of-contents-list\"]"));
        assert!(css.contains("[data-slot=\"table-of-contents-link\"]"));
        assert!(css.contains("border-left: 1px solid var(--cronus-border)"));
        assert!(css.contains("var(--cronus-fg-tertiary"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
