//! Dedicated ReceiveButton renderer (Skiper 45 Family Receive). DOM matches
//! React: `<div data-slot="receive-button">` (28rem tall, content pinned to
//! the bottom) > the uppercase hint with its gradient tick, the blurred
//! `receive-button-overlay`, and the shell that is the sky CTA pill when
//! closed and the confirmation card (`receive-button-dialog`: fingerprint
//! badge + `Confirm`, the `receive-button-close` X, the description,
//! `receive-button-cancel` and the same `receive-button-trigger` CTA) when
//! open.
//!
//! Zero JS: a visually hidden checkbox inside the trigger `<label>` holds the
//! open state (click, Space, focus ring). Cancel, the close X and the overlay
//! are `<label for>` the same checkbox, so each collapses the card like
//! React's handlers; the decorative buttons after them keep the slots and
//! the looks. `:has(:checked)` grows the shell (40px → auto,
//! `interpolate-size` where supported, `--cronus-spring-soft`), swaps the
//! radius 30 → 24 and fades the overlay in. Escape needs JS and is the one
//! dismissal not reproduced; `onConfirm` has no kernel equivalent.
//!
//! Inputs: `label "Receive"` (CTA), `title "Confirm"`, `text "Are you sure…"`
//! (description), `hint:"Toggle layout with animation"`, `open:true`.

use crate::cronus_ui_kit::{attr, esc, flag, item, label_of, truthy};
use crate::parser::ComponentNode;

const FINGERPRINT_ICON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" data-icon=\"fingerprint\"><path d=\"M12 10a2 2 0 0 0-2 2c0 1.02-.1 2.51-.26 4\"/><path d=\"M14 13.12c0 2.38 0 6.38-1 8.88\"/><path d=\"M17.29 21.02c.12-.6.43-2.3.5-3.02\"/><path d=\"M2 12a10 10 0 0 1 18-6\"/><path d=\"M2 16h.01\"/><path d=\"M21.8 16c.2-2 .131-5.354 0-6\"/><path d=\"M5 19.5C5.5 18 6 15 6 12a6 6 0 0 1 .34-2\"/><path d=\"M8.65 22c.21-.66.45-1.32.57-2\"/><path d=\"M9 6.8a6 6 0 0 1 9 5.2v2\"/></svg>";

pub(crate) fn fingerprint_icon() -> &'static str {
    FINGERPRINT_ICON
}

pub fn render(comp: &ComponentNode) -> String {
    let cta = label_of(comp);
    let title = item(comp, "title")
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| "Confirm".to_string());
    let description = item(comp, "text")
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| "Are you sure you want to receive hell load of money?".to_string());
    let hint = match attr(comp, "hint") {
        Some(h) => esc(h.trim()),
        None => "Toggle layout with animation".to_string(),
    };
    let hint_html = if hint.is_empty() {
        String::new()
    } else {
        format!("<p>{hint}</p>")
    };
    let open = flag(comp, "open") || attr(comp, "default-open").is_some_and(|v| truthy(v));
    let checked = if open { " checked" } else { "" };
    let expanded = if open { "true" } else { "false" };
    let id = crate::cronus_ui_kit::instance_id(comp, "receive-button");
    let title_id = format!("{id}-title");
    let desc_id = format!("{id}-desc");
    let plus = crate::cronus_ui_icons::svg_or_empty("plus");
    format!(
        "<div data-slot=\"receive-button\">{hint_html}<label for=\"{id}\" aria-hidden=\"true\"><button type=\"button\" data-slot=\"receive-button-overlay\" aria-label=\"Close\" tabindex=\"-1\"></button></label><div data-slot=\"receive-button-dialog\" role=\"dialog\" aria-modal=\"true\" aria-labelledby=\"{title_id}\" aria-describedby=\"{desc_id}\"><div class=\"head\"><div><span class=\"ico\">{FINGERPRINT_ICON}</span><h2 id=\"{title_id}\">{title}</h2></div><label for=\"{id}\" aria-hidden=\"true\"><button type=\"button\" data-slot=\"receive-button-close\" aria-label=\"Close\" tabindex=\"-1\">{plus}</button></label></div><p id=\"{desc_id}\">{description}</p><div class=\"row\"><label for=\"{id}\" aria-hidden=\"true\"><button type=\"button\" data-slot=\"receive-button-cancel\" tabindex=\"-1\">Cancel</button></label><label><input type=\"checkbox\" id=\"{id}\" aria-label=\"{cta}\" aria-haspopup=\"dialog\"{checked}><button type=\"button\" data-slot=\"receive-button-trigger\" aria-haspopup=\"dialog\" aria-expanded=\"{expanded}\" tabindex=\"-1\" aria-hidden=\"true\"><span>{cta}</span></button></label></div></div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::{reset_instance_ids, stub};

    fn receive() -> ComponentNode {
        reset_instance_ids();
        stub("receive-button", "Receive")
    }

    fn reject_js(html: &str) {
        for bad in [
            "<script", "style=", "onclick", "onmouse", "onkey", "<canvas", "<dialog", "popover",
        ] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    #[test]
    fn docs_family_receive_dom() {
        let html = render(&receive());
        assert!(html.starts_with("<div data-slot=\"receive-button\"><p>Toggle layout with animation</p><label for=\"cui-receive-button-receive-button\" aria-hidden=\"true\"><button type=\"button\" data-slot=\"receive-button-overlay\" aria-label=\"Close\" tabindex=\"-1\"></button></label><div data-slot=\"receive-button-dialog\" role=\"dialog\" aria-modal=\"true\" aria-labelledby=\"cui-receive-button-receive-button-title\" aria-describedby=\"cui-receive-button-receive-button-desc\"><div class=\"head\"><div><span class=\"ico\"><svg"));
        assert!(html.contains("data-icon=\"fingerprint\""));
        assert!(html.contains("</svg></span><h2 id=\"cui-receive-button-receive-button-title\">Confirm</h2></div><label for=\"cui-receive-button-receive-button\" aria-hidden=\"true\"><button type=\"button\" data-slot=\"receive-button-close\" aria-label=\"Close\" tabindex=\"-1\"><svg"));
        assert!(html.contains("data-icon=\"plus\""));
        assert!(html.ends_with("</svg></button></label></div><p id=\"cui-receive-button-receive-button-desc\">Are you sure you want to receive hell load of money?</p><div class=\"row\"><label for=\"cui-receive-button-receive-button\" aria-hidden=\"true\"><button type=\"button\" data-slot=\"receive-button-cancel\" tabindex=\"-1\">Cancel</button></label><label><input type=\"checkbox\" id=\"cui-receive-button-receive-button\" aria-label=\"Receive\" aria-haspopup=\"dialog\"><button type=\"button\" data-slot=\"receive-button-trigger\" aria-haspopup=\"dialog\" aria-expanded=\"false\" tabindex=\"-1\" aria-hidden=\"true\"><span>Receive</span></button></label></div></div></div>"));
        reject_js(&html);
    }

    #[test]
    fn open_state_and_custom_copy() {
        let mut c = receive();
        c.props.insert("open".into(), "true".into());
        c.props.insert("hint".into(), "".into());
        c.items.push(crate::parser::ComponentItemNode {
            item_type: "title".into(),
            text: "Approve".into(),
            link: None,
            tone: None,
            config: Default::default(),
        });
        c.items.push(crate::parser::ComponentItemNode {
            item_type: "text".into(),
            text: "Send 1 ETH?".into(),
            link: None,
            tone: None,
            config: Default::default(),
        });
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"receive-button\"><label for="));
        assert!(html.contains("aria-haspopup=\"dialog\" checked><button type=\"button\" data-slot=\"receive-button-trigger\" aria-haspopup=\"dialog\" aria-expanded=\"true\""));
        assert!(html.contains("<h2 id=\"cui-receive-button-receive-button-title\">Approve</h2>"));
        assert!(html.contains("<p id=\"cui-receive-button-receive-button-desc\">Send 1 ETH?</p>"));
    }

    #[test]
    fn escapes_hostile_input() {
        let mut c = receive();
        c.items[0].text = "<b>Get</b> \"$\"".into();
        c.props.insert("hint".into(), "<i>h</i>".into());
        let html = render(&c);
        assert!(html.contains("<p>&lt;i&gt;h&lt;/i&gt;</p>"));
        assert!(html.contains("aria-label=\"&lt;b&gt;Get&lt;/b&gt; &quot;$&quot;\""));
        assert!(html.contains("<span>&lt;b&gt;Get&lt;/b&gt; &quot;$&quot;</span>"));
        reject_js(&html);
    }

    #[test]
    fn chrome_is_token_only_or_component_painted() {
        let css = include_str!("cronus_ui_css/receive-button.css");
        assert!(css.contains("[data-slot=\"receive-button\"] {"));
        assert!(css.contains("min-height: 28rem"));
        assert!(css.contains("--cui-receive-sky"));
        assert!(css.contains("[data-slot=\"receive-button\"]:has(input:checked) > [data-slot=\"receive-button-dialog\"]"));
        assert!(css.contains("border-radius: 30px"));
        assert!(css.contains("border-radius: 24px"));
        assert!(css.contains("backdrop-filter: blur(4px)"));
        assert!(css.contains("var(--cronus-spring-soft)"));
        assert!(
            css.contains("[data-slot=\"receive-button\"] [data-slot=\"receive-button-cancel\"]")
        );
    }

    #[test]
    fn registered_as_dedicated() {
        let c = receive();
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), {
            reset_instance_ids();
            render(&c)
        });
        assert_eq!(
            dedicated_fn_name("receive-button"),
            Some("cronus_ui_receive_button::render")
        );
        assert_eq!(
            renderer_kind("receive-button"),
            RendererKind::Dedicated("cronus_ui_receive_button::render")
        );
    }
}
