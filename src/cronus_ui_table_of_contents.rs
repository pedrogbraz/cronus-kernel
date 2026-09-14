//! Dedicated TableOfContents renderer. DOM matches React:
//! `<nav data-slot="table-of-contents" aria-label>` +
//! `<span data-slot="table-of-contents-indicator" aria-hidden>` (idle: opacity 0,
//! no active heading without scroll-spy JS) + `<ul data-slot="table-of-contents-list">`
//! plus each item as `<li><a data-slot="table-of-contents-link" data-depth="0">`.
//! The `label` names the nav (default aria-label); it is never a link.
//! Not interact `nav("table-of-contents")` (generic SURF `<nav>` without list/link).

use crate::cronus_ui_kit::{attr_nonempty, esc, label_of, safe_url};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let links = toc_entries(comp)
        .into_iter()
        .map(|(text, href)| {
            format!(
                "<li><a data-slot=\"table-of-contents-link\" href=\"{href}\" data-depth=\"0\">{text}</a></li>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let aria = attr_nonempty(comp, "aria-label")
        .map(|s| esc(s))
        .unwrap_or_else(|| "On this page".into());
    format!(
        "<nav data-slot=\"table-of-contents\" aria-label=\"{aria}\"><span aria-hidden=\"true\" data-slot=\"table-of-contents-indicator\"></span><ul data-slot=\"table-of-contents-list\">{links}</ul></nav>"
    )
}

fn toc_entries(comp: &ComponentNode) -> Vec<(String, String)> {
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
    if !body.is_empty() {
        return body;
    }
    let label = label_of(comp);
    let href = hash_href(None, &label);
    vec![(label, href)]
}

fn entry_of(i: &ComponentItemNode) -> (String, String) {
    (esc(&i.text), hash_href(i.link.as_deref(), &i.text))
}

fn hash_href(link: Option<&str>, text: &str) -> String {
    if let Some(h) = link.map(str::trim).filter(|s| !s.is_empty()) {
        return safe_url(h);
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
        } else if (c.is_whitespace() || c == '-' || c == '_')
            && !out.is_empty()
            && !out.ends_with('-')
        {
            out.push('-');
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
        assert!(!html.contains("-control"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn emitted_fixture_has_indicator_and_label_is_not_a_link() {
        let mut c = stub("table-of-contents", "On this page");
        c.items.push(extra("text", "Overview"));
        c.items.push(extra("text", "Usage"));
        c.items[2]
            .config
            .insert("aria-label".into(), "On this page".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<nav data-slot=\"table-of-contents\" aria-label=\"On this page\"><span aria-hidden=\"true\" data-slot=\"table-of-contents-indicator\"></span><ul data-slot=\"table-of-contents-list\"><li><a data-slot=\"table-of-contents-link\" href=\"#overview\" data-depth=\"0\">Overview</a></li><li><a data-slot=\"table-of-contents-link\" href=\"#usage\" data-depth=\"0\">Usage</a></li></ul></nav>"
        );
        reject_interact(&html);
    }

    #[test]
    fn linked_item_keeps_href() {
        let mut c = stub("table-of-contents", "Nav");
        c.items.clear();
        let mut intro = extra("item", "Intro");
        intro.link = Some("#getting-started".into());
        c.items.push(intro);
        let html = render(&c);
        assert!(html.contains("href=\"#getting-started\" data-depth=\"0\">Intro</a>"));
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_one_link() {
        let html = render(&stub("table-of-contents", "Intro"));
        assert_eq!(
            html.matches("data-slot=\"table-of-contents-link\"").count(),
            1
        );
        assert!(html.contains("href=\"#intro\" data-depth=\"0\">Intro</a>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_nav_surf() {
        let mut c = stub("table-of-contents", "Nav");
        c.items.push(extra("item", "Intro"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"table-of-contents-list\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            reject_interact(&render(&stub("table-of-contents", "Intro")));
        });
    }

    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("position: relative; font-size: 0.875rem; line-height: 1.25rem;"));
        assert!(css.contains("[data-slot=\"table-of-contents-indicator\"]"));
        assert!(css.contains("border-start-end-radius: var(--cronus-radius-md);"));
        assert!(css.contains("border-left: 1px solid var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
    }
}
