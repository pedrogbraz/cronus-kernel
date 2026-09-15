//! Dedicated PromptInput renderer (AI suite). DOM matches React `PromptInput`,
//! `PromptInputBody`, `PromptInputTextarea`, `PromptInputFooter`,
//! `PromptInputTools` and `PromptInputSubmit`: a real
//! `<form data-slot="prompt-input" action method>` wrapping
//! `prompt-input-composer`, which holds `prompt-input-body` (`display:
//! contents`, around `<textarea name="message">`) and `prompt-input-footer`
//! (an empty `prompt-input-tools` and the submit button). It posts
//! without JS: `action:` goes through `safe_url`, `method:` is `get` / `post`.
//! `placeholder:` (or the first text / label) defaults to React's label.
//! `status:` follows React: `submitted` / `streaming` make the button a
//! `type="button"` (stop needs JS, so it is `disabled`), `error` shows the X
//! icon. React's hidden `<input type="file">` (attachments) sits outside the
//! form and only works with JS, so it is not rendered.

use crate::cronus_ui_kit::{attr, attr_nonempty, esc, item, safe_url};
use crate::parser::ComponentNode;

const PLACEHOLDER: &str = "What would you like to know?";
const SVG_OPEN: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\">";
const SEND: &str = "<path d=\"M14.536 21.686a.5.5 0 0 0 .937-.024l6.5-19a.496.496 0 0 0-.635-.635l-19 6.5a.5.5 0 0 0-.024.937l7.93 3.18a2 2 0 0 1 1.112 1.11z\"></path><path d=\"m21.854 2.147-10.94 10.939\"></path>";
const LOADER: &str = "<path d=\"M21 12a9 9 0 1 1-6.219-8.56\"></path>";
const SQUARE: &str = "<rect width=\"18\" height=\"18\" x=\"3\" y=\"3\" rx=\"2\"></rect>";
const X: &str = "<path d=\"M18 6 6 18\"></path><path d=\"m6 6 12 12\"></path>";

pub fn render(comp: &ComponentNode) -> String {
    let action = attr_nonempty(comp, "action")
        .map(|a| format!(" action=\"{}\"", safe_url(a)))
        .unwrap_or_default();
    let method = attr(comp, "method")
        .map(|m| m.trim().to_ascii_lowercase())
        .filter(|m| m == "get" || m == "post")
        .map(|m| format!(" method=\"{m}\""))
        .unwrap_or_default();
    let placeholder = esc(attr_nonempty(comp, "placeholder")
        .or_else(|| item(comp, "text").filter(|t| !t.is_empty()))
        .or_else(|| item(comp, "label").filter(|t| !t.is_empty()))
        .unwrap_or(PLACEHOLDER));
    format!(
        "<form data-slot=\"prompt-input\"{action}{method}><div data-slot=\"prompt-input-composer\"><div data-slot=\"prompt-input-body\"><textarea data-slot=\"prompt-input-textarea\" name=\"message\" placeholder=\"{placeholder}\"></textarea></div><div data-slot=\"prompt-input-footer\"><div data-slot=\"prompt-input-tools\"></div>{}</div></div></form>",
        submit(attr(comp, "status").map(str::trim).unwrap_or(""))
    )
}

fn submit(status: &str) -> String {
    let (icon, busy) = match status {
        "submitted" => (LOADER, true),
        "streaming" => (SQUARE, true),
        "error" => (X, false),
        _ => (SEND, false),
    };
    let label = if status == "streaming" {
        "Stop"
    } else {
        "Submit"
    };
    let kind = if busy {
        "button\" disabled data-busy=\"\""
    } else {
        "submit\""
    };
    format!(
        "<button data-slot=\"prompt-input-submit\" data-variant=\"primary\" aria-label=\"{label}\" type=\"{kind}>{SVG_OPEN}{icon}</svg></button>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    #[test]
    fn form_posts_without_js() {
        let mut c = stub("prompt-input", "What would you like to know?");
        c.props.insert("action".into(), "/chat".into());
        c.props.insert("method".into(), "POST".into());
        let html = render(&c);
        assert!(html.starts_with("<form data-slot=\"prompt-input\" action=\"/chat\" method=\"post\"><div data-slot=\"prompt-input-composer\"><div data-slot=\"prompt-input-body\"><textarea data-slot=\"prompt-input-textarea\" name=\"message\" placeholder=\"What would you like to know?\"></textarea></div><div data-slot=\"prompt-input-footer\"><div data-slot=\"prompt-input-tools\"></div><button data-slot=\"prompt-input-submit\" data-variant=\"primary\" aria-label=\"Submit\" type=\"submit\"><svg"));
        assert!(html.ends_with("</svg></button></div></div></form>"));
        assert!(!html.contains("disabled"));
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("type=\"file\""));
    }

    #[test]
    fn unsafe_action_and_unknown_method_are_dropped_or_neutralized() {
        let mut c = stub("prompt-input", "Ask");
        c.props
            .insert("action".into(), "javascript:alert(1)".into());
        c.props.insert("method".into(), "dialog".into());
        c.props.insert("placeholder".into(), "\"><script>".into());
        let html = render(&c);
        assert!(!html.contains("javascript:"));
        assert!(!html.contains("method="));
        assert!(html.contains("placeholder=\"&quot;&gt;&lt;script&gt;\""));
    }

    #[test]
    fn busy_status_is_a_disabled_stop_button() {
        let mut c = stub("prompt-input", "Ask");
        c.props.insert("status".into(), "streaming".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Stop\" type=\"button\" disabled data-busy=\"\"><svg"));
        assert!(html.contains("<rect width=\"18\""));
        c.props.insert("status".into(), "error".into());
        assert!(render(&c).contains("aria-label=\"Submit\" type=\"submit\"><svg"));
    }

    #[test]
    fn placeholder_defaults_to_react_label() {
        let mut c = stub("prompt-input", "");
        c.items.clear();
        assert!(render(&c).contains("placeholder=\"What would you like to know?\""));
    }
}
