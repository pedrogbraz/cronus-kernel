//! Dedicated Message renderer (AI suite). DOM matches React `Message` +
//! `MessageAvatar` + `MessageContent`: `<div data-slot="message" data-from>` >
//! optional `<span data-slot="message-avatar">` (the Radix `Avatar` root with
//! `<img data-slot="avatar-image">` + `<span data-slot="avatar-fallback">`) >
//! `<div data-slot="message-content">`. Content is escaped text only: no raw
//! HTML and no markdown rendering. One text is the content itself; several
//! become `<p>` paragraphs.
//!
//! Props: `from:user|assistant|system` (React's union; absent is `user`);
//! `variant:flat` puts React's cva variant on the content as `class="v-flat"`
//! (React writes classes, not an attribute); `avatar:"https://…"` renders the
//! avatar with `name:"Cronus"` as the alt text and its first two characters as
//! the fallback (`me:"ME"` is React's `labels.me` when there is no name).

use crate::cronus_ui_kit::{attr, attr_nonempty, content_texts, esc, item, safe_url};
use crate::parser::ComponentNode;

/// React `MessageAvatar` `labels.me` default.
const ME: &str = "ME";

pub fn render(comp: &ComponentNode) -> String {
    let from = from_of(attr(comp, "from"));
    let flat = attr(comp, "variant").is_some_and(|v| v.trim().eq_ignore_ascii_case("flat"));
    let mut texts: Vec<String> = item(comp, "label")
        .filter(|t| !t.is_empty())
        .map(esc)
        .into_iter()
        .collect();
    texts.extend(content_texts(comp));
    let avatar = attr_nonempty(comp, "avatar")
        .map(|src| avatar_html(src, attr_nonempty(comp, "name"), attr_nonempty(comp, "me")))
        .unwrap_or_default();
    message_html(from, flat, &avatar, &texts)
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

/// `MessageAvatar`: Radix `Avatar` root (`data-slot` overridden to
/// `message-avatar`) with the image and the two-letter fallback.
pub fn avatar_html(src: &str, name: Option<&str>, me: Option<&str>) -> String {
    let me = me.unwrap_or(ME);
    let alt = esc(name.unwrap_or(me));
    let fallback: String = name
        .filter(|n| !n.is_empty())
        .map(|n| n.chars().take(2).collect())
        .unwrap_or_else(|| me.to_string());
    format!(
        "<span data-slot=\"message-avatar\"><img data-slot=\"avatar-image\" alt=\"{alt}\" src=\"{}\"><span data-slot=\"avatar-fallback\">{}</span></span>",
        safe_url(src),
        esc(&fallback)
    )
}

/// One message turn, shared with `cronus_ui_conversation`. `avatar` is an
/// [`avatar_html`] (or empty); `texts` must already be escaped.
pub fn message_html(from: &str, flat: bool, avatar: &str, texts: &[String]) -> String {
    let class = if flat { " class=\"v-flat\"" } else { "" };
    let body = match texts {
        [one] => one.clone(),
        many => many.iter().map(|t| format!("<p>{t}</p>")).collect(),
    };
    format!(
        "<div data-slot=\"message\" data-from=\"{from}\">{avatar}<div data-slot=\"message-content\"{class}>{body}</div></div>"
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

    /// Docs "User and assistant": the assistant turn carries a `MessageAvatar`
    /// (Radix Avatar root re-slotted, image + two-letter fallback).
    #[test]
    fn assistant_message_with_avatar() {
        let mut c = stub("message", "Use semantic tokens. Never a palette scale.");
        c.props.insert("from".into(), "assistant".into());
        c.props
            .insert("avatar".into(), "https://github.com/pedrogbraz.png".into());
        c.props.insert("name".into(), "Cronus".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"message\" data-from=\"assistant\"><span data-slot=\"message-avatar\"><img data-slot=\"avatar-image\" alt=\"Cronus\" src=\"https://github.com/pedrogbraz.png\"><span data-slot=\"avatar-fallback\">Cr</span></span><div data-slot=\"message-content\">Use semantic tokens. Never a palette scale.</div></div>"
        );
        zero_js(&html);
    }

    #[test]
    fn avatar_fallback_is_me_label_and_src_is_safe() {
        let mut c = stub("message", "Hi");
        c.props
            .insert("avatar".into(), "javascript:alert(1)".into());
        let html = render(&c);
        assert!(html.contains("<img data-slot=\"avatar-image\" alt=\"ME\" src=\"#\">"));
        assert!(html.contains("<span data-slot=\"avatar-fallback\">ME</span>"));
        c.props.insert("me".into(), "EU".into());
        assert!(render(&c)
            .contains("alt=\"EU\" src=\"#\"><span data-slot=\"avatar-fallback\">EU</span>"));
        c.props.insert("name".into(), "<b>".into());
        assert!(render(&c).contains(
            "alt=\"&lt;b&gt;\" src=\"#\"><span data-slot=\"avatar-fallback\">&lt;b</span>"
        ));
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
    fn several_texts_become_paragraphs_and_flat_variant_is_a_class() {
        let mut c = stub("message", "First");
        c.items.push(text("Second"));
        c.props.insert("variant".into(), "flat".into());
        c.props.insert("from".into(), "System".into());
        let html = render(&c);
        assert!(html.contains("data-from=\"system\""));
        assert!(html.contains(
            "<div data-slot=\"message-content\" class=\"v-flat\"><p>First</p><p>Second</p></div>"
        ));
        assert!(!html.contains("data-variant"));
    }

    #[test]
    fn from_defaults_to_user_and_unknown_is_assistant() {
        assert_eq!(from_of(None), "user");
        assert_eq!(from_of(Some("bot")), "assistant");
        assert_eq!(from_of(Some(" USER ")), "user");
    }

    #[test]
    fn chrome_covers_variants_and_avatar() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"message\"][data-from=\"user\"] > [data-slot=\"message-content\"] {\n  background: var(--cronus-primary); color: var(--cronus-primary-foreground);\n}"));
        assert!(css.contains("[data-slot=\"message-content\"].v-flat {"));
        assert!(css.contains("[data-slot=\"message\"][data-from=\"user\"] > [data-slot=\"message-content\"].v-flat {"));
        assert!(!css.contains("[data-slot=\"message-content\"][data-variant"));
        assert!(css.contains("[data-slot=\"message-avatar\"] {\n  position: relative; display: flex; width: 2rem; height: 2rem; flex-shrink: 0; overflow: hidden;\n  border-radius: 9999px; box-shadow: 0 0 0 1px var(--cronus-border);\n}"));
        assert!(css.contains("[data-slot=\"message-avatar\"] > [data-slot=\"avatar-image\"] + [data-slot=\"avatar-fallback\"] { display: none; }"));
    }
}
