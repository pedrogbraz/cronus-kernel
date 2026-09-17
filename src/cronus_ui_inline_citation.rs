//! Dedicated InlineCitation renderer (AI suite). DOM matches React
//! `InlineCitation`, `InlineCitationText`, `InlineCitationCardTrigger` (a
//! `Badge` secondary under Radix `HoverCardTrigger asChild`, closed) and
//! `InlineCitationCardBody` > `InlineCitationSource`: the root
//! `<span data-slot="inline-citation">` holds the text span, then
//! `<span data-slot="inline-citation-card-trigger" data-variant="secondary"
//! role="button" tabindex="0" data-state="closed">host +N</span>`, then the
//! card body (`w-80 p-0`) with one `<div data-slot="inline-citation-source">`
//! (`<h4>` title, `<p>` url, `<p>` description) per source. The label is the
//! cited text (React's `InlineCitationText`, the docs' `*`); sources are
//! `link "Cronus UI" -> "https://aicronus.com"` items (title + url; a
//! `description:` config adds the third line) or plain text URLs. `host` is
//! the first URL's hostname (the raw string when it is not an absolute URL),
//! like React's `new URL(src).hostname`. Zero JS: the card body has no layout
//! box until the citation is hovered or focused (CSS, with React's pop-in),
//! where React mounts its HoverCard portal.

use crate::cronus_ui_kit::{esc, item};
use crate::parser::ComponentNode;

const UNKNOWN: &str = "unknown";

struct Source {
    title: Option<String>,
    url: String,
    description: Option<String>,
}

pub fn render(comp: &ComponentNode) -> String {
    let text = item(comp, "label").map(esc).unwrap_or_default();
    let sources: Vec<Source> = comp
        .items
        .iter()
        .filter(|i| !matches!(i.item_type.as_str(), "label" | "title") && !i.text.is_empty())
        .map(|i| match i.link.as_deref().filter(|l| !l.is_empty()) {
            Some(url) => Source {
                title: Some(i.text.clone()),
                url: url.to_string(),
                description: i.config.get("description").cloned(),
            },
            None => Source {
                title: None,
                url: i.text.clone(),
                description: i.config.get("description").cloned(),
            },
        })
        .collect();
    let badge = match sources.first() {
        None => UNKNOWN.to_string(),
        Some(first) if sources.len() > 1 => {
            format!("{} +{}", esc(&host_of(&first.url)), sources.len() - 1)
        }
        Some(first) => esc(&host_of(&first.url)),
    };
    let body = if sources.is_empty() {
        String::new()
    } else {
        let rows: String = sources
            .iter()
            .map(|s| {
                let title = s
                    .title
                    .as_deref()
                    .map(|t| format!("<h4>{}</h4>", esc(t)))
                    .unwrap_or_default();
                let description = s
                    .description
                    .as_deref()
                    .filter(|d| !d.is_empty())
                    .map(|d| format!("<p>{}</p>", esc(d)))
                    .unwrap_or_default();
                format!(
                    "<div data-slot=\"inline-citation-source\">{title}<p>{}</p>{description}</div>",
                    esc(&s.url)
                )
            })
            .collect();
        format!("<div data-slot=\"inline-citation-card-body\">{rows}</div>")
    };
    format!(
        "<span data-slot=\"inline-citation\"><span data-slot=\"inline-citation-text\">{text}</span><span data-slot=\"inline-citation-card-trigger\" data-variant=\"secondary\" role=\"button\" tabindex=\"0\" data-state=\"closed\">{badge}</span>{body}</span>"
    )
}

/// WHATWG-ish hostname: `scheme://[user@]host[:port]/…` → lowercase host.
/// Anything that is not an absolute `scheme://` URL stays as written.
fn host_of(src: &str) -> String {
    let raw = src.trim();
    let Some((scheme, rest)) = raw.split_once("://") else {
        return raw.to_string();
    };
    let valid_scheme = scheme.starts_with(|c: char| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
    if !valid_scheme {
        return raw.to_string();
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host_port = authority.rsplit('@').next().unwrap_or("");
    let host = if host_port.starts_with('[') {
        host_port
            .find(']')
            .map(|end| &host_port[..=end])
            .unwrap_or(host_port)
    } else {
        host_port.split(':').next().unwrap_or("")
    };
    if host.is_empty() {
        raw.to_string()
    } else {
        host.to_ascii_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn text(t: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "text".into(),
            text: t.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn link(t: &str, href: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "link".into(),
            text: t.into(),
            link: Some(href.into()),
            tone: None,
            config: Default::default(),
        }
    }

    #[test]
    fn two_sources_match_react_trigger() {
        let mut c = stub("inline-citation", "Cronus compiles.");
        c.items.push(text("https://cronus.dev/docs"));
        c.items.push(text("https://github.com/cooud/cronus"));
        let html = render(&c);
        assert!(html.starts_with("<span data-slot=\"inline-citation\"><span data-slot=\"inline-citation-text\">Cronus compiles.</span><span data-slot=\"inline-citation-card-trigger\" data-variant=\"secondary\" role=\"button\" tabindex=\"0\" data-state=\"closed\">cronus.dev +1</span><div data-slot=\"inline-citation-card-body\">"));
        assert!(html.contains("<div data-slot=\"inline-citation-source\"><p>https://github.com/cooud/cronus</p></div>"));
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
    }

    /// Docs "Cited claim": `*` text, one titled source.
    #[test]
    fn link_items_render_titled_sources() {
        let mut c = stub("inline-citation", "*");
        let mut src = link("Cronus UI", "https://aicronus.com");
        src.config
            .insert("description".into(), "The product UI system.".into());
        c.items.push(src);
        let html = render(&c);
        assert_eq!(
            html,
            "<span data-slot=\"inline-citation\"><span data-slot=\"inline-citation-text\">*</span><span data-slot=\"inline-citation-card-trigger\" data-variant=\"secondary\" role=\"button\" tabindex=\"0\" data-state=\"closed\">aicronus.com</span><div data-slot=\"inline-citation-card-body\"><div data-slot=\"inline-citation-source\"><h4>Cronus UI</h4><p>https://aicronus.com</p><p>The product UI system.</p></div></div></span>"
        );
    }

    #[test]
    fn no_sources_is_unknown_without_body() {
        let html = render(&stub("inline-citation", "Claim"));
        assert!(html.contains("data-state=\"closed\">unknown</span></span>"));
        assert!(!html.contains("card-body"));
    }

    #[test]
    fn hostname_rules_follow_url_parsing() {
        assert_eq!(
            host_of("https://User@Example.COM:8080/a?b#c"),
            "example.com"
        );
        assert_eq!(host_of("http://[::1]:3000/"), "[::1]");
        assert_eq!(host_of("example.com/page"), "example.com/page");
        assert_eq!(host_of("1ab://x"), "1ab://x");
    }

    #[test]
    fn text_and_host_are_escaped() {
        let mut c = stub("inline-citation", "<b>");
        c.items.push(text("<x>"));
        c.items.push(link("<t>", "https://a.b/<p>"));
        let html = render(&c);
        assert!(html.contains(">&lt;b&gt;</span>"));
        assert!(html.contains(">&lt;x&gt; +1</span>"));
        assert!(html.contains("<div data-slot=\"inline-citation-source\"><p>&lt;x&gt;</p></div>"));
        assert!(html.contains("<h4>&lt;t&gt;</h4><p>https://a.b/&lt;p&gt;</p>"));
        assert!(!html.contains("<x>"));
    }

    #[test]
    fn chrome_matches_hover_card_body() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"inline-citation-card-body\"] {\n  display: none; position: absolute; z-index: 50; inset-block-start: calc(100% + 4px);"));
        assert!(css.contains(
            "inline-size: 20rem; max-inline-size: 90vw; box-sizing: border-box; padding: 0;"
        ));
        assert!(css.contains("animation: cronus-pop-in 180ms var(--ease-out-quart) both;"));
        assert!(css.contains("[data-slot=\"inline-citation-source\"] > h4 {"));
        assert!(css.contains("[data-slot=\"inline-citation-source\"] > p + p {"));
    }
}
