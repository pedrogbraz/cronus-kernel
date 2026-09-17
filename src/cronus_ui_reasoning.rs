//! Dedicated Reasoning renderer (AI suite). DOM matches React/Radix
//! `Reasoning`, `ReasoningTrigger` and `ReasoningContent`: the root
//! `<div data-state data-slot="reasoning">` holds the trigger
//! `<button aria-expanded data-state data-slot="reasoning-trigger">` (brain
//! icon, `<p>` status, chevron) and `<div data-slot="reasoning-content">`, which
//! wraps `<div data-slot="response">` (escaped text; several texts are `<p>`s).
//!
//! Zero JS: the trigger and content sit in a native `<details>` inside the
//! root, the button in its `<summary>`. The summary toggles the content
//! natively (click, Enter, Space) and exposes the expanded state; the button
//! keeps React's element and idle look but is decorative (`aria-hidden`,
//! `tabindex="-1"`, `pointer-events: none`). The chevron turns and the content
//! slides in on `details[open]`. `data-state` / `aria-expanded` reflect the
//! initial state (React's `defaultOpen`, true unless `defaultOpen:false` /
//! `open:false`). Status text follows React: a `duration:` above 0 is
//! "Thought for N seconds"; no duration, 0 or `isStreaming:true` is the
//! "Thinking..." `text-shimmer` (1s sweep). Labels localize with `thinking:`
//! and `thoughtFor:"…{duration}…"`.

use crate::cronus_ui_kit::{attr, attr_nonempty, attr_num, content_texts, esc, flag, item, truthy};
use crate::parser::ComponentNode;

const BRAIN: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M12 18V5\"></path><path d=\"M15 13a4.17 4.17 0 0 1-3-4 4.17 4.17 0 0 1-3 4\"></path><path d=\"M17.598 6.5A3 3 0 1 0 12 5a3 3 0 1 0-5.598 1.5\"></path><path d=\"M17.997 5.125a4 4 0 0 1 2.526 5.77\"></path><path d=\"M18 18a4 4 0 0 0 2-7.464\"></path><path d=\"M19.967 17.483A4 4 0 1 1 12 18a4 4 0 1 1-7.967-.517\"></path><path d=\"M6 18a4 4 0 0 1-2-7.464\"></path><path d=\"M6.003 5.125a4 4 0 0 0-2.526 5.77\"></path></svg>";
const CHEVRON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m6 9 6 6 6-6\"></path></svg>";
const THINKING: &str = "Thinking...";
const THOUGHT_FOR: &str = "Thought for {duration} seconds";
/// React `TextShimmer` default `spread` (px per UTF-16 unit), as in `cronus_ui_text_shimmer`.
const SPREAD_PER_CHAR: usize = 2;

pub fn render(comp: &ComponentNode) -> String {
    let open = ["open", "defaultOpen"]
        .iter()
        .find_map(|key| attr(comp, key))
        .is_none_or(truthy);
    let state = if open { "open" } else { "closed" };
    let status = status_html(comp, flag(comp, "isStreaming"), attr_num(comp, "duration"));
    let details_open = if open { " open" } else { "" };
    format!(
        "<div data-state=\"{state}\" data-slot=\"reasoning\"><details{details_open}><summary><button type=\"button\" aria-expanded=\"{open}\" data-state=\"{state}\" data-slot=\"reasoning-trigger\" tabindex=\"-1\" aria-hidden=\"true\">{BRAIN}{status}{CHEVRON}</button></summary><div data-state=\"{state}\" data-slot=\"reasoning-content\"><div data-slot=\"response\">{}</div></div></details></div>",
        body(comp)
    )
}

fn status_html(comp: &ComponentNode, streaming: bool, duration: Option<u64>) -> String {
    match duration {
        Some(seconds) if !streaming && seconds > 0 => {
            let label = attr_nonempty(comp, "thoughtFor")
                .unwrap_or(THOUGHT_FOR)
                .replacen("{duration}", &seconds.to_string(), 1);
            format!("<p>{}</p>", esc(&label))
        }
        _ => {
            let thinking = attr_nonempty(comp, "thinking").unwrap_or(THINKING);
            format!(
                "<p data-slot=\"text-shimmer\" data-spread=\"{}\" data-duration=\"1\">{}</p>",
                thinking.encode_utf16().count() * SPREAD_PER_CHAR,
                esc(thinking)
            )
        }
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
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick"));
        assert!(!html.contains(" disabled"));
    }

    /// Docs "Thinking": `defaultOpen duration={4}`.
    #[test]
    fn duration_renders_open_react_dom_in_a_native_details() {
        let mut c = stub(
            "reasoning",
            "The user wants a product UI system, not a bag of parts.",
        );
        c.props.insert("duration".into(), "4".into());
        let html = render(&c);
        assert!(html.starts_with("<div data-state=\"open\" data-slot=\"reasoning\"><details open><summary><button type=\"button\" aria-expanded=\"true\" data-state=\"open\" data-slot=\"reasoning-trigger\" tabindex=\"-1\" aria-hidden=\"true\"><svg"));
        assert!(html.contains("<p>Thought for 4 seconds</p><svg"));
        assert!(html.ends_with("</svg></button></summary><div data-state=\"open\" data-slot=\"reasoning-content\"><div data-slot=\"response\">The user wants a product UI system, not a bag of parts.</div></div></details></div>"));
        zero_js(&html);
    }

    #[test]
    fn no_duration_or_streaming_is_thinking_shimmer() {
        let html = render(&stub("reasoning", "Hmm"));
        assert!(html.contains(
            "<p data-slot=\"text-shimmer\" data-spread=\"22\" data-duration=\"1\">Thinking...</p>"
        ));
        let mut c = stub("reasoning", "Hmm");
        c.props.insert("duration".into(), "9".into());
        c.props.insert("isStreaming".into(), "true".into());
        assert!(render(&c).contains(">Thinking...</p>"));
    }

    #[test]
    fn labels_localize_status() {
        let mut c = stub("reasoning", "Hmm");
        c.props.insert("thinking".into(), "Pensando…".into());
        assert!(render(&c).contains("data-spread=\"18\" data-duration=\"1\">Pensando…</p>"));
        c.props.insert("duration".into(), "2".into());
        c.props
            .insert("thoughtFor".into(), "Pensou por {duration} s <b>".into());
        assert!(render(&c).contains("<p>Pensou por 2 s &lt;b&gt;</p>"));
    }

    #[test]
    fn closed_state_keeps_content_in_the_closed_details_and_escapes_text() {
        let mut c = stub("reasoning", "<script>x</script>");
        c.props.insert("defaultOpen".into(), "false".into());
        let html = render(&c);
        assert!(html.contains("data-state=\"closed\" data-slot=\"reasoning\"><details><summary>"));
        assert!(html.contains(
            "aria-expanded=\"false\" data-state=\"closed\" data-slot=\"reasoning-trigger\""
        ));
        assert!(html.contains("<div data-state=\"closed\" data-slot=\"reasoning-content\">"));
        assert!(html.contains("&lt;script&gt;x&lt;/script&gt;"));
        assert!(!html.contains("<script>"));
        zero_js(&html);
    }

    #[test]
    fn chrome_toggles_on_details() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"reasoning\"] > details > summary {"));
        assert!(css.contains("[data-slot=\"reasoning-trigger\"] {\n  display: flex; width: 100%; align-items: center; gap: 0.5rem;"));
        assert!(css.contains("pointer-events: none;"));
        assert!(css.contains("[data-slot=\"reasoning\"] > details[open] > summary [data-slot=\"reasoning-trigger\"] > svg:last-child {\n  rotate: 180deg;\n}"));
        assert!(css.contains("[data-slot=\"reasoning\"] > details[open] > [data-slot=\"reasoning-content\"] {\n  animation: cui-reasoning-in 150ms var(--ease-out-quart);\n}"));
        assert!(css.contains("[data-slot=\"reasoning-trigger\"] > [data-slot=\"text-shimmer\"] { animation-duration: 1s; }"));
    }
}
