//! Dedicated InlineCitation renderer (AI suite). DOM matches React
//! `InlineCitation`, `InlineCitationText` and `InlineCitationCardTrigger` (a
//! `Badge` secondary under Radix `HoverCardTrigger asChild`, closed): the root
//! `<span data-slot="inline-citation">` holds the text span and then
//! `<span data-slot="inline-citation-card-trigger" data-variant="secondary"
//! role="button" tabindex="0" data-state="closed">host +N</span>`. The label is
//! the cited text; each text item is a source URL. `host` is the first URL's
//! hostname (the raw string when it is not an absolute URL), like React's
//! `new URL(src).hostname`. Zero JS: the card body
//! (`inline-citation-card-body`, one `inline-citation-source` per URL) has no
//! layout box until the citation is hovered or focused (CSS), where React
//! mounts its HoverCard portal.

use crate::cronus_ui_kit::{content_texts, esc, item};
use crate::parser::ComponentNode;

const UNKNOWN: &str = "unknown";

pub fn render(comp: &ComponentNode) -> String {
    let text = item(comp, "label").map(esc).unwrap_or_default();
    let sources: Vec<String> = comp
        .items
        .iter()
        .filter(|i| !matches!(i.item_type.as_str(), "label" | "title") && !i.text.is_empty())
        .map(|i| i.text.clone())
        .collect();
    let badge = match sources.first() {
        None => UNKNOWN.to_string(),
        Some(first) if sources.len() > 1 => {
            format!("{} +{}", esc(&host_of(first)), sources.len() - 1)
        }
        Some(first) => esc(&host_of(first)),
    };
    let body = if sources.is_empty() {
        String::new()
    } else {
        let rows: String = content_texts(comp)
            .iter()
            .map(|url| format!("<div data-slot=\"inline-citation-source\"><p>{url}</p></div>"))
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
        let html = render(&c);
        assert!(html.contains(">&lt;b&gt;</span>"));
        assert!(html.contains(">&lt;x&gt;</span>"));
        assert!(!html.contains("<x>"));
    }
}
