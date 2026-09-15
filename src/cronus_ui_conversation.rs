//! Dedicated Conversation renderer (AI suite). DOM matches React
//! `Conversation` + `ConversationContent`: `<div data-slot="conversation">` >
//! `<div data-slot="conversation-content" role="log" aria-live="polite"
//! aria-relevant="additions">` > one `cronus_ui_message` turn per text item,
//! alternating user / assistant unless the item sets `from:`. Stick-to-bottom
//! and the scroll button need JS: React hides the button while at the bottom,
//! which is the idle state, so it is not rendered. With no messages the
//! content holds `ConversationEmptyState` (`title:` / `description:` props).

use crate::cronus_ui_kit::{attr_nonempty, esc};
use crate::cronus_ui_message::{from_of, message_html};
use crate::parser::ComponentNode;

const EMPTY_TITLE: &str = "No messages yet";
const EMPTY_DESCRIPTION: &str = "Start a conversation to see messages here";

pub fn render(comp: &ComponentNode) -> String {
    let messages: String = comp
        .items
        .iter()
        .filter(|i| !matches!(i.item_type.as_str(), "label" | "title") && !i.text.is_empty())
        .enumerate()
        .map(|(n, i)| {
            let from = match i.config.get("from") {
                Some(v) => from_of(Some(v)),
                None if n % 2 == 0 => "user",
                None => "assistant",
            };
            message_html(from, false, &[esc(&i.text)])
        })
        .collect();
    let inner = if messages.is_empty() {
        empty_state(comp)
    } else {
        messages
    };
    format!(
        "<div data-slot=\"conversation\"><div data-slot=\"conversation-content\" role=\"log\" aria-live=\"polite\" aria-relevant=\"additions\">{inner}</div></div>"
    )
}

fn empty_state(comp: &ComponentNode) -> String {
    let title = esc(attr_nonempty(comp, "title").unwrap_or(EMPTY_TITLE));
    let description = esc(attr_nonempty(comp, "description").unwrap_or(EMPTY_DESCRIPTION));
    format!(
        "<div data-slot=\"conversation-empty-state\"><div><h3>{title}</h3><p>{description}</p></div></div>"
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

    #[test]
    fn texts_alternate_user_and_assistant_turns() {
        let mut c = stub("conversation", "default");
        c.items.push(text("What is Cronus?"));
        c.items.push(text("A language."));
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"conversation\"><div data-slot=\"conversation-content\" role=\"log\" aria-live=\"polite\" aria-relevant=\"additions\"><div data-slot=\"message\" data-from=\"user\"><div data-slot=\"message-content\">What is Cronus?</div></div><div data-slot=\"message\" data-from=\"assistant\"><div data-slot=\"message-content\">A language.</div></div></div></div>"
        );
        assert!(!html.contains("default"), "the label is not a message");
        assert!(!html.contains("conversation-scroll-button"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn item_from_overrides_alternation_and_text_is_escaped() {
        let mut c = stub("conversation", "Chat");
        let mut first = text("<b>hi</b>");
        first.config.insert("from".into(), "assistant".into());
        c.items.push(first);
        let html = render(&c);
        assert!(html.contains("data-from=\"assistant\""));
        assert!(html.contains("&lt;b&gt;hi&lt;/b&gt;"));
    }

    #[test]
    fn no_messages_renders_react_empty_state_defaults() {
        let html = render(&stub("conversation", "Chat"));
        assert!(html.contains(
            "<div data-slot=\"conversation-empty-state\"><div><h3>No messages yet</h3><p>Start a conversation to see messages here</p></div></div>"
        ));
    }
}
