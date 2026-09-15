//! Dedicated Message renderer (AI suite). DOM matches React `Message` +
//! `MessageContent`: `<div data-slot="message" data-from>` >
//! `<div data-slot="message-content">`. Content is escaped text only: no raw
//! HTML and no markdown rendering. One text is the content itself; several
//! become `<p>` paragraphs. `variant:flat` adds `data-variant="flat"` on the
//! content (React's cva variant writes classes, not an attribute).
//! `MessageAvatar` (Radix image loading) is not rendered.

use crate::cronus_ui_kit::{attr, content_texts, esc, item};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let from = from_of(attr(comp, "from"));
    let flat = attr(comp, "variant").is_some_and(|v| v.trim().eq_ignore_ascii_case("flat"));
    let mut texts: Vec<String> = item(comp, "label")
        .filter(|t| !t.is_empty())
        .map(esc)
        .into_iter()
        .collect();
    texts.extend(content_texts(comp));
    message_html(from, flat, &texts)
}

/// React's `from` union. Absent means `user` (the audit default); an
/// unknown value is an `assistant` turn.
pub fn from_of(raw: Option<&str>) -> &'static str {
    let Some(raw) = raw.map(str::trim) else {
        return "user";
    };
    if raw.eq_ignore_ascii_case("user") {
        "user"
    } else if raw.eq_ignore_ascii_case("system") {
        "system"
    } else {
        "assistant"
    }
}

/// One message turn, shared with `cronus_ui_conversation`. `texts` must
/// already be escaped.
pub fn message_html(from: &str, flat: bool, texts: &[String]) -> String {
    let variant = if flat { " data-variant=\"flat\"" } else { "" };
    let body = match texts {
        [one] => one.clone(),
        many => many.iter().map(|t| format!("<p>{t}</p>")).collect(),
    };
    format!(
        "<div data-slot=\"message\" data-from=\"{from}\"><div data-slot=\"message-content\"{variant}>{body}</div></div>"
    )
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

    fn zero_js(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick"));
        assert!(!html.contains("v-data"));
    }

    #[test]
    fn user_message_matches_react_dom() {
        let mut c = stub("message", "Can you summarize the release notes?");
        c.props.insert("from".into(), "user".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"message\" data-from=\"user\"><div data-slot=\"message-content\">Can you summarize the release notes?</div></div>"
        );
        zero_js(&html);
    }

    #[test]
    fn content_is_escaped_never_raw_html_or_markdown() {
        let mut c = stub("message", "<img src=x onerror=alert(1)> **bold**");
        c.props.insert("from".into(), "assistant".into());
        let html = render(&c);
        assert!(html.contains("&lt;img src=x onerror=alert(1)&gt; **bold**"));
        assert!(!html.contains("<img"));
        assert!(!html.contains("<strong"));
    }

    #[test]
    fn several_texts_become_paragraphs_and_flat_variant_is_an_attribute() {
        let mut c = stub("message", "First");
        c.items.push(text("Second"));
        c.props.insert("variant".into(), "flat".into());
        c.props.insert("from".into(), "System".into());
        let html = render(&c);
        assert!(html.contains("data-from=\"system\""));
        assert!(html.contains(
            "<div data-slot=\"message-content\" data-variant=\"flat\"><p>First</p><p>Second</p></div>"
        ));
    }

    #[test]
    fn from_defaults_to_user_and_unknown_is_assistant() {
        assert_eq!(from_of(None), "user");
        assert_eq!(from_of(Some("bot")), "assistant");
        assert_eq!(from_of(Some(" USER ")), "user");
    }
}
