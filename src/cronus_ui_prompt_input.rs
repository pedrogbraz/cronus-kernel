//! Dedicated PromptInput renderer (AI suite). DOM matches React `PromptInput`,
//! `PromptInputBody`, `PromptInputTextarea`, `PromptInputFooter`,
//! `PromptInputTools`, `PromptInputButton` and `PromptInputSubmit`: a real
//! `<form data-slot="prompt-input" action method>` wrapping
//! `prompt-input-composer`, which holds `prompt-input-body` (`display:
//! contents`, around `<textarea name="message">`) and `prompt-input-footer`
//! (`prompt-input-tools` with one ghost `<button data-slot="prompt-input-button">`
//! per `action` item, then the submit button). It posts without JS: `action:`
//! goes through `safe_url`, `method:` is `get` / `post`. `placeholder:` (or the
//! first text / label) defaults to React's label; `value:"…"` prefills the
//! textarea. Tool buttons (`action "Attach" icon:paperclip`) are React's
//! `PromptInputButton`: icon-only `icon-sm` (`class="s-icon-sm"`, the text is
//! the accessible name) or, with `icon-only:false` / no icon, icon + text
//! `sm` (`class="s-sm"`). Attach (`paperclip` / "Attach") is a `<label>`
//! wrapping a hidden `<input type="file">` so picking a file works natively;
//! voice and tools stay enabled `type="button"` (click is a no-op without
//! page runtime). `status:` follows React: `submitted` (spinning loader)
//! / `streaming` (stop square) make the button a `type="button"` (stop needs
//! JS, so it is `disabled`), `error` shows the X icon.

use crate::cronus_ui_kit::{attr, attr_nonempty, esc, item, item_icon, safe_url, truthy};
use crate::parser::{ComponentItemNode, ComponentNode};

const PLACEHOLDER: &str = "What would you like to know?";
const SVG_OPEN: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"";
const SEND: &str = "><path d=\"M14.536 21.686a.5.5 0 0 0 .937-.024l6.5-19a.496.496 0 0 0-.635-.635l-19 6.5a.5.5 0 0 0-.024.937l7.93 3.18a2 2 0 0 1 1.112 1.11z\"></path><path d=\"m21.854 2.147-10.94 10.939\"></path></svg>";
const LOADER: &str =
    " data-icon=\"loader-circle\"><path d=\"M21 12a9 9 0 1 1-6.219-8.56\"></path></svg>";
const SQUARE: &str = "><rect width=\"18\" height=\"18\" x=\"3\" y=\"3\" rx=\"2\"></rect></svg>";
const X: &str = "><path d=\"M18 6 6 18\"></path><path d=\"m6 6 12 12\"></path></svg>";

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
    let value = attr(comp, "value").map(esc).unwrap_or_default();
    format!(
        "<form data-slot=\"prompt-input\"{action}{method}><div data-slot=\"prompt-input-composer\"><div data-slot=\"prompt-input-body\"><textarea data-slot=\"prompt-input-textarea\" name=\"message\" placeholder=\"{placeholder}\">{value}</textarea></div><div data-slot=\"prompt-input-footer\"><div data-slot=\"prompt-input-tools\">{}</div>{}</div></div></form>",
        tools(comp),
        submit(attr(comp, "status").map(str::trim).unwrap_or(""))
    )
}

fn is_attach(item: &ComponentItemNode) -> bool {
    item.text.eq_ignore_ascii_case("attach")
        || item
            .config
            .get("icon")
            .is_some_and(|s| s.eq_ignore_ascii_case("paperclip"))
        || item
            .config
            .get("type")
            .is_some_and(|s| s.eq_ignore_ascii_case("file"))
}

/// `PromptInputButton`s from `action` items.
fn tools(comp: &ComponentNode) -> String {
    comp.items
        .iter()
        .filter(|i| i.item_type == "action" && !i.text.is_empty())
        .map(|i| {
            let icon = item_icon(i);
            let icon_only = !icon.is_empty()
                && i.config.get("icon-only").is_none_or(|v| truthy(v));
            let label = esc(&i.text);
            if is_attach(i) {
                if icon_only {
                    format!(
                        "<label data-slot=\"prompt-input-button\" data-variant=\"ghost\" class=\"s-icon-sm\" aria-label=\"{label}\"><input type=\"file\" hidden>{icon}</label>"
                    )
                } else {
                    format!(
                        "<label data-slot=\"prompt-input-button\" data-variant=\"ghost\" class=\"s-sm\"><input type=\"file\" hidden>{icon}{label}</label>"
                    )
                }
            } else if icon_only {
                format!(
                    "<button type=\"button\" data-slot=\"prompt-input-button\" data-variant=\"ghost\" class=\"s-icon-sm\" aria-label=\"{label}\">{icon}</button>"
                )
            } else {
                format!(
                    "<button type=\"button\" data-slot=\"prompt-input-button\" data-variant=\"ghost\" class=\"s-sm\">{icon}{label}</button>"
                )
            }
        })
        .collect()
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
        "<button data-slot=\"prompt-input-submit\" data-variant=\"primary\" aria-label=\"{label}\" type=\"{kind}>{SVG_OPEN}{icon}</button>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn action(text: &str, cfg: &[(&str, &str)]) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "action".into(),
            text: text.into(),
            link: None,
            tone: None,
            config: cfg
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

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

    /// Docs "Composer": `placeholder="Ask anything…"`, empty tools, send.
    #[test]
    fn placeholder_prop_and_prefilled_value() {
        let mut c = stub("prompt-input", "Composer");
        c.props.insert("placeholder".into(), "Ask anything…".into());
        c.props.insert("value".into(), "<draft>".into());
        let html = render(&c);
        assert!(html.contains("placeholder=\"Ask anything…\">&lt;draft&gt;</textarea>"));
        assert!(html.contains(
            "<div data-slot=\"prompt-input-tools\"></div><button data-slot=\"prompt-input-submit\""
        ));
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
        c.props.insert("status".into(), "submitted".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Submit\" type=\"button\" disabled data-busy=\"\"><svg"));
        assert!(
            html.contains(" data-icon=\"loader-circle\"><path d=\"M21 12a9 9 0 1 1-6.219-8.56\">")
        );
        c.props.insert("status".into(), "error".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Submit\" type=\"submit\"><svg"));
        assert!(html.contains("<path d=\"M18 6 6 18\">"));
    }

    /// Toolbar: `action` items are ghost `PromptInputButton`s (icon-only
    /// `icon-sm`, or icon + text `sm`). Attach is a file-picker label.
    #[test]
    fn action_items_are_toolbar_buttons() {
        let mut c = stub("prompt-input", "Ask");
        c.items.push(action("Attach", &[("icon", "paperclip")]));
        c.items.push(action("Voice <x>", &[("icon", "mic")]));
        c.items.push(action(
            "Model",
            &[("icon", "sparkles"), ("icon-only", "false")],
        ));
        c.items.push(action("Tools", &[]));
        let html = render(&c);
        assert!(html.contains("<div data-slot=\"prompt-input-tools\"><label data-slot=\"prompt-input-button\" data-variant=\"ghost\" class=\"s-icon-sm\" aria-label=\"Attach\"><input type=\"file\" hidden><svg"));
        assert!(html.contains("data-icon=\"paperclip\""));
        assert!(html.contains("aria-label=\"Voice &lt;x&gt;\"><svg"));
        assert!(html.contains("class=\"s-sm\"><svg xmlns"));
        assert!(html.contains("</svg>Model</button>"));
        assert!(html.contains("class=\"s-sm\">Tools</button>"));
        assert_eq!(html.matches("data-slot=\"prompt-input-button\"").count(), 4);
        assert_eq!(html.matches("type=\"file\"").count(), 1);
        assert!(!html.contains("prompt-input-button\" data-variant=\"ghost\" class=\"s-icon-sm\" aria-label=\"Attach\" disabled"));
        assert!(!html.contains("aria-label=\"Voice &lt;x&gt;\" disabled"));
    }

    #[test]
    fn placeholder_defaults_to_react_label() {
        let mut c = stub("prompt-input", "");
        c.items.clear();
        assert!(render(&c).contains("placeholder=\"What would you like to know?\""));
    }

    #[test]
    fn chrome_covers_toolbar_and_busy_states() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"prompt-input-button\"] {\n  display: inline-flex; align-items: center; justify-content: center; gap: 0.5rem;"));
        assert!(css.contains("[data-slot=\"prompt-input-button\"].s-icon-sm { width: 2rem; height: 2rem; padding: 0; }"));
        assert!(css.contains("[data-slot=\"prompt-input-button\"].s-sm {"));
        assert!(css.contains(
            "[data-slot=\"prompt-input-button\"]:disabled { opacity: 1; pointer-events: none; }"
        ));
        assert!(css.contains("[data-slot=\"prompt-input-submit\"] > svg[data-icon=\"loader-circle\"] {\n  animation: cui-prompt-input-spin 1s linear infinite;\n}"));
        assert!(css.contains("[data-slot=\"prompt-input-submit\"][data-busy] { opacity: 1; }"));
    }
}
