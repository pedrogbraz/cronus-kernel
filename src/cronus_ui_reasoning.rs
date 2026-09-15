//! Dedicated Reasoning renderer (AI suite). DOM matches React/Radix
//! `Reasoning`, `ReasoningTrigger` and `ReasoningContent`: the root
//! `<div data-state data-slot="reasoning">` holds the trigger
//! `<button aria-expanded data-state data-slot="reasoning-trigger">` (brain
//! icon, `<p>` status, chevron) and `<div data-slot="reasoning-content">`, which
//! wraps `<div data-slot="response">` (escaped text; several texts are `<p>`s).
//! Not `<details>`: React's slots are a `div` root and a `button` trigger, and
//! geometry pairs tags. Toggling needs JS, so the kernel renders React's
//! `defaultOpen` state (open unless `defaultOpen:false` / `open:false`) with the
//! trigger `disabled` at React's idle look. Status text follows React: a
//! `duration:` above 0 is "Thought for N seconds"; no duration, 0 or
//! `isStreaming:true` is the "Thinking..." `text-shimmer`.

use crate::cronus_ui_kit::{attr, attr_num, content_texts, esc, flag, item, truthy};
use crate::parser::ComponentNode;

const BRAIN: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M12 18V5\"></path><path d=\"M15 13a4.17 4.17 0 0 1-3-4 4.17 4.17 0 0 1-3 4\"></path><path d=\"M17.598 6.5A3 3 0 1 0 12 5a3 3 0 1 0-5.598 1.5\"></path><path d=\"M17.997 5.125a4 4 0 0 1 2.526 5.77\"></path><path d=\"M18 18a4 4 0 0 0 2-7.464\"></path><path d=\"M19.967 17.483A4 4 0 1 1 12 18a4 4 0 1 1-7.967-.517\"></path><path d=\"M6 18a4 4 0 0 1-2-7.464\"></path><path d=\"M6.003 5.125a4 4 0 0 0-2.526 5.77\"></path></svg>";
const CHEVRON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m6 9 6 6 6-6\"></path></svg>";
const THINKING: &str = "Thinking...";
/// React `TextShimmer` default `spread` (px per UTF-16 unit), as in `cronus_ui_text_shimmer`.
const SPREAD_PER_CHAR: usize = 2;

pub fn render(comp: &ComponentNode) -> String {
    let open = ["open", "defaultOpen"]
        .iter()
        .find_map(|key| attr(comp, key))
        .is_none_or(truthy);
    let state = if open { "open" } else { "closed" };
    let status = status_html(flag(comp, "isStreaming"), attr_num(comp, "duration"));
    let content = if open {
        format!(
            "<div data-state=\"open\" data-slot=\"reasoning-content\"><div data-slot=\"response\">{}</div></div>",
            body(comp)
        )
    } else {
        String::new()
    };
    format!(
        "<div data-state=\"{state}\" data-slot=\"reasoning\"><button type=\"button\" aria-expanded=\"{open}\" data-state=\"{state}\" data-slot=\"reasoning-trigger\" disabled>{BRAIN}{status}{CHEVRON}</button>{content}</div>"
    )
}

fn status_html(streaming: bool, duration: Option<u64>) -> String {
    match duration {
        Some(seconds) if !streaming && seconds > 0 => {
            format!("<p>Thought for {seconds} seconds</p>")
        }
        _ => format!(
            "<p data-slot=\"text-shimmer\" data-spread=\"{}\">{THINKING}</p>",
            THINKING.encode_utf16().count() * SPREAD_PER_CHAR
        ),
    }
}

fn body(comp: &ComponentNode) -> String {
    let mut texts: Vec<String> = item(comp, "label")
        .filter(|t| !t.is_empty())
        .map(esc)
        .into_iter()
        .collect();
    texts.extend(content_texts(comp));
    match texts.as_slice() {
        [one] => one.clone(),
        many => many.iter().map(|t| format!("<p>{t}</p>")).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn zero_js(html: &str) {
        assert!(!html.contains("<details"));
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick"));
    }

    #[test]
    fn duration_renders_open_react_dom() {
        let mut c = stub("reasoning", "List the biggest changes first.");
        c.props.insert("duration".into(), "3".into());
        let html = render(&c);
        assert!(html.starts_with("<div data-state=\"open\" data-slot=\"reasoning\"><button type=\"button\" aria-expanded=\"true\" data-state=\"open\" data-slot=\"reasoning-trigger\" disabled><svg"));
        assert!(html.contains("<p>Thought for 3 seconds</p><svg"));
        assert!(html.ends_with("<div data-state=\"open\" data-slot=\"reasoning-content\"><div data-slot=\"response\">List the biggest changes first.</div></div></div>"));
        zero_js(&html);
    }

    #[test]
    fn no_duration_or_streaming_is_thinking_shimmer() {
        let html = render(&stub("reasoning", "Hmm"));
        assert!(html.contains("<p data-slot=\"text-shimmer\" data-spread=\"22\">Thinking...</p>"));
        let mut c = stub("reasoning", "Hmm");
        c.props.insert("duration".into(), "9".into());
        c.props.insert("isStreaming".into(), "true".into());
        assert!(render(&c).contains(">Thinking...</p>"));
    }

    #[test]
    fn closed_state_has_no_content_and_escapes_text() {
        let mut c = stub("reasoning", "<script>x</script>");
        c.props.insert("defaultOpen".into(), "false".into());
        let html = render(&c);
        assert!(html.contains("aria-expanded=\"false\" data-state=\"closed\""));
        assert!(!html.contains("reasoning-content"));
        let open = render(&stub("reasoning", "<script>x</script>"));
        assert!(open.contains("&lt;script&gt;x&lt;/script&gt;"));
        zero_js(&open);
    }
}
