//! Dedicated TableOfContents renderer. DOM matches React:
//! `<nav data-slot="table-of-contents" aria-label>` +
//! `<span data-slot="table-of-contents-indicator" aria-hidden>` (idle: opacity 0,
//! no active heading without scroll-spy JS) + `<ul data-slot="table-of-contents-list">`
//! plus each item as `<li><a data-slot="table-of-contents-link" data-depth="0">`.
//! The `label` names the nav (default aria-label); it is never a link.
//! `item "With the CLI" depth:1` nests (React `data-depth`, indented by CSS).
//! Docs scrollspy: `article:true` renders the docs' `flex gap-6` row — the nav
//! (`w-44 shrink-0 self-start`) beside the scrolling `<section
//! aria-label="Article content" tabindex="0">` of one `<section>` per item
//! (`<h3 id>` heading + `description:"…"` paragraph, ids page-unique). Links
//! scroll the container natively (smooth unless reduced motion); the heading
//! reached through a link is `:target`, and CSS paints its row active
//! (`text-fg`, indicator bar glides to it). Gap: scrolling by hand does not
//! move the active row (IntersectionObserver is JS).
//! Not interact `nav("table-of-contents")` (generic SURF `<nav>` without list/link).

use crate::cronus_ui_kit::{attr_nonempty, esc, flag, instance_id, label_of, safe_url};
use crate::parser::{ComponentItemNode, ComponentNode};

/// Rows beyond this index still link and scroll, but never paint active.
pub const MAX_ROWS: usize = 16;

pub fn render(comp: &ComponentNode) -> String {
    let article = flag(comp, "article");
    let prefix = if article {
        format!("{}-", instance_id(comp, "toc"))
    } else {
        String::new()
    };
    let entries = toc_entries(comp, &prefix);
    let links = entries
        .iter()
        .map(|e| {
            format!(
                "<li><a data-slot=\"table-of-contents-link\" href=\"{}\" data-depth=\"{}\">{}</a></li>",
                e.href, e.depth, e.text
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let aria = attr_nonempty(comp, "aria-label")
        .map(|s| esc(s))
        .unwrap_or_else(|| "On this page".into());
    let nav = format!(
        "<nav data-slot=\"table-of-contents\" aria-label=\"{aria}\"><span aria-hidden=\"true\" data-slot=\"table-of-contents-indicator\"></span><ul data-slot=\"table-of-contents-list\">{links}</ul></nav>"
    );
    if !article {
        return nav;
    }
    let sections = entries
        .iter()
        .map(|e| {
            let body = e
                .description
                .as_ref()
                .map(|d| format!("<p>{d}</p>"))
                .unwrap_or_default();
            format!(
                "<section><h3 id=\"{}\">{}</h3>{body}</section>",
                e.id, e.text
            )
        })
        .collect::<String>();
    format!(
        "<div>{nav}<section id=\"{prefix}article\" aria-label=\"Article content\" tabindex=\"0\">{sections}</section></div>"
    )
}

struct Entry {
    text: String,
    href: String,
    id: String,
    depth: u32,
    description: Option<String>,
}

fn toc_entries(comp: &ComponentNode, prefix: &str) -> Vec<Entry> {
    let choice: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| {
            matches!(i.item_type.as_str(), "item" | "tab" | "columns") && !i.text.is_empty()
        })
        .collect();
    let picked: Vec<&ComponentItemNode> = if choice.is_empty() {
        comp.items
            .iter()
            .filter(|i| !matches!(i.item_type.as_str(), "label" | "title") && !i.text.is_empty())
            .collect()
    } else {
        choice
    };
    if picked.is_empty() {
        let label = label_of(comp);
        let id = format!("{prefix}{}", slugify(&label));
        return vec![Entry {
            href: hash_href(None, &label, prefix),
            id,
            text: label,
            depth: 0,
            description: None,
        }];
    }
    picked
        .into_iter()
        .map(|i| Entry {
            text: esc(&i.text),
            href: hash_href(i.link.as_deref(), &i.text, prefix),
            id: format!("{prefix}{}", slugify(&i.text)),
            depth: i
                .config
                .get("depth")
                .and_then(|d| d.trim().parse::<u32>().ok())
                .unwrap_or(0)
                .min(5),
            description: i
                .config
                .get("description")
                .filter(|d| !d.trim().is_empty())
                .map(|d| esc(d)),
        })
        .collect()
}

fn hash_href(link: Option<&str>, text: &str, prefix: &str) -> String {
    if let Some(h) = link.map(str::trim).filter(|s| !s.is_empty()) {
        return safe_url(h);
    }
    let slug = slugify(text);
    if slug.is_empty() {
        "#".into()
    } else {
        format!("#{prefix}{slug}")
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
    fn depth_and_article_scrollspy() {
        let mut c = stub("table-of-contents", "On this page");
        c.props.insert("article".into(), "true".into());
        let mut overview = extra("item", "Overview");
        overview
            .config
            .insert("description".into(), "What it is.".into());
        c.items.push(overview);
        let mut cli = extra("item", "With the CLI");
        cli.config.insert("depth".into(), "1".into());
        c.items.push(cli);
        let html = render(&c);
        let id = crate::cronus_ui_kit::widget_id(&c, "toc");
        assert_eq!(
            html,
            format!(
                "<div><nav data-slot=\"table-of-contents\" aria-label=\"On this page\"><span aria-hidden=\"true\" data-slot=\"table-of-contents-indicator\"></span><ul data-slot=\"table-of-contents-list\"><li><a data-slot=\"table-of-contents-link\" href=\"#{id}-overview\" data-depth=\"0\">Overview</a></li><li><a data-slot=\"table-of-contents-link\" href=\"#{id}-with-the-cli\" data-depth=\"1\">With the CLI</a></li></ul></nav><section id=\"{id}-article\" aria-label=\"Article content\" tabindex=\"0\"><section><h3 id=\"{id}-overview\">Overview</h3><p>What it is.</p></section><section><h3 id=\"{id}-with-the-cli\">With the CLI</h3></section></section></div>"
            )
        );
        reject_interact(&html);
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"table-of-contents-link\"][data-depth=\"1\"] { padding-inline-start: 1.5rem; }"));
        assert!(css.contains("div:has(> section > section:nth-child(2) > h3:target) > [data-slot=\"table-of-contents\"] li:nth-child(2) > [data-slot=\"table-of-contents-link\"]"));
        assert!(css.contains("div:has(> section > section:nth-child(2) > h3:target) > [data-slot=\"table-of-contents\"] > [data-slot=\"table-of-contents-indicator\"] { opacity: 1; transform: translateY(1.953rem); }"));
        assert!(css.contains("div:has(> [data-slot=\"table-of-contents\"] + section) > section {\n  height: 16rem; flex: 1; overflow-y: auto; scroll-behavior: smooth;"));
    }

    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("position: relative; font-size: 0.875rem; line-height: 1.25rem;"));
        assert!(css.contains("[data-slot=\"table-of-contents-indicator\"]"));
        assert!(css.contains("border-start-end-radius: var(--cronus-radius-md);"));
        assert!(css.contains("border-inline-start: 1px solid var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
    }
}
