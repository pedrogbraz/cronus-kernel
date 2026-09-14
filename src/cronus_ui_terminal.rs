//! Dedicated Terminal renderer. DOM matches React `Terminal` with
//! `motionPreference="never"` (the finished transcript, `data-state="static"`):
//! `terminal` > `terminal-chrome` (traffic dots, `terminal-title`, copy button)
//! + `terminal-body` (sr-only `terminal-transcript`, invisible `terminal-sizer`
//! and absolutely positioned `terminal-screen`, both holding `terminal-line`s).
//!
//! Lines alternate input / output (even index = command behind a `$` prompt,
//! odd index = its output), the same mapping the audit fixture uses for plain
//! string items. Zero JS: no typing animation, no caret. The copy button needs
//! the clipboard API, so it is rendered `disabled` (inert, announced as
//! unavailable) at React's idle geometry. Not catalog `display()` SURF
//! `<section data-slot="terminal">`, not interact `codey()` `<pre>`.

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

const COPY_ICON: &str = concat!(
    "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\">",
    "<rect width=\"14\" height=\"14\" x=\"8\" y=\"8\" rx=\"2\" ry=\"2\"></rect>",
    "<path d=\"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2\"></path></svg>",
);

pub fn render(comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let mut lines: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type != "label" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if lines.is_empty() {
        lines.push(title.clone());
    }
    let rows = lines
        .iter()
        .enumerate()
        .map(|(i, t)| {
            if i % 2 == 0 {
                format!(
                    "<div data-slot=\"terminal-line\" data-line-type=\"input\"><span data-slot=\"terminal-prompt\">$</span><span>{t}</span></div>"
                )
            } else {
                format!("<div data-slot=\"terminal-line\" data-line-type=\"output\"><span>{t}</span></div>")
            }
        })
        .collect::<String>();
    let transcript = lines
        .iter()
        .enumerate()
        .map(|(i, t)| {
            if i % 2 == 0 {
                format!("$ {t}")
            } else {
                t.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "<div data-slot=\"terminal\" data-state=\"static\"><div data-slot=\"terminal-chrome\"><div aria-hidden=\"true\"><span></span><span></span><span></span></div><span data-slot=\"terminal-title\">{title}</span><button type=\"button\" data-slot=\"copy-button\" data-variant=\"ghost\" aria-label=\"Copy commands\" disabled>{COPY_ICON}</button></div><div><section data-slot=\"terminal-body\" tabindex=\"0\" aria-label=\"Terminal, {title}\"><pre data-slot=\"terminal-transcript\" class=\"sr-only\">{transcript}</pre><div aria-hidden=\"true\"><div data-slot=\"terminal-sizer\">{rows}</div><div data-slot=\"terminal-screen\">{rows}</div></div></section></div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const DISPLAY_SURF: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem";

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn session(title: &str, items: &[&str]) -> ComponentNode {
        let mut c = stub("terminal", title);
        for n in items {
            c.items.push(extra("text", n));
        }
        c
    }

    fn reject_stub(html: &str) {
        assert!(!html.contains("<pre data-slot=\"terminal\""));
        assert!(!html.contains("<section data-slot=\"terminal\""));
        assert!(!html.contains("<code"));
        assert!(!html.contains("<nav"));
        assert!(!html.contains("style="));
        assert!(!html.contains("<style"));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setTimeout"));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains(DISPLAY_SURF));
        assert!(!html.contains("display("));
        assert!(!html.contains("codey("));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn dom_matches_react_static_terminal() {
        let html = render(&session(
            "zsh",
            &["bunx cronus-ui add button", "added button"],
        ));
        let rows = "<div data-slot=\"terminal-line\" data-line-type=\"input\"><span data-slot=\"terminal-prompt\">$</span><span>bunx cronus-ui add button</span></div><div data-slot=\"terminal-line\" data-line-type=\"output\"><span>added button</span></div>";
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"terminal\" data-state=\"static\"><div data-slot=\"terminal-chrome\"><div aria-hidden=\"true\"><span></span><span></span><span></span></div><span data-slot=\"terminal-title\">zsh</span><button type=\"button\" data-slot=\"copy-button\" data-variant=\"ghost\" aria-label=\"Copy commands\" disabled>{COPY_ICON}</button></div><div><section data-slot=\"terminal-body\" tabindex=\"0\" aria-label=\"Terminal, zsh\"><pre data-slot=\"terminal-transcript\" class=\"sr-only\">$ bunx cronus-ui add button\nadded button</pre><div aria-hidden=\"true\"><div data-slot=\"terminal-sizer\">{rows}</div><div data-slot=\"terminal-screen\">{rows}</div></div></section></div></div>"
            )
        );
        reject_stub(&html);
    }

    #[test]
    fn title_is_chrome_not_a_line() {
        let html = render(&session("zsh", &["ls", "a.txt", "pwd"]));
        assert!(html.contains("<span data-slot=\"terminal-title\">zsh</span>"));
        assert!(!html.contains("<span>zsh</span>"));
        // 2 inputs, rendered in sizer + screen.
        assert_eq!(html.matches("data-slot=\"terminal-prompt\"").count(), 4);
        assert_eq!(html.matches("data-line-type=\"output\"").count(), 2);
        assert!(html.contains("$ ls\na.txt\n$ pwd</pre>"));
        reject_stub(&html);
    }

    #[test]
    fn label_only_still_emits_one_input_line() {
        let html = render(&stub("terminal", "npm install"));
        assert!(html.contains("<span data-slot=\"terminal-title\">npm install</span>"));
        assert_eq!(html.matches("data-slot=\"terminal-prompt\"").count(), 2);
        assert!(html.contains("<span>npm install</span></div>"));
        reject_stub(&html);
    }

    #[test]
    fn copy_button_is_inert_not_a_fake_action() {
        let html = render(&session("zsh", &["ls"]));
        assert!(html.contains("aria-label=\"Copy commands\" disabled>"));
        assert!(!html.contains("clipboard"));
        reject_stub(&html);
    }

    #[test]
    fn text_is_escaped() {
        let html = render(&session("A <B>", &["echo \"C\" & D"]));
        assert!(html.contains("<span data-slot=\"terminal-title\">A &lt;B&gt;</span>"));
        assert!(html.contains("<span>echo &quot;C&quot; &amp; D</span>"));
        reject_stub(&html);
    }

    #[test]
    fn skips_interact_pre_and_display_surf() {
        let c = session("zsh", &["npm install", "ready"]);
        let html = render(&c);
        assert!(html.contains("data-slot=\"terminal-screen\""));
        assert!(html.contains("data-slot=\"terminal-prompt\""));
        reject_stub(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&session("zsh", &["npm install"]));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"terminal-screen\""));
        });
    }

    #[test]
    fn chrome_is_token_only_and_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"terminal\"] {\n  box-sizing: border-box; width: 18rem;"));
        assert!(css.contains(
            "[data-slot=\"terminal-chrome\"] {\n  display: flex; align-items: center; gap: 0.5rem;"
        ));
        assert!(css.contains("font-size: 0.75rem; line-height: 1rem; font-weight: 500;"));
        assert!(css.contains("[data-slot=\"terminal-sizer\"] { visibility: hidden; }"));
        assert!(css.contains("[data-slot=\"terminal-screen\"] { position: absolute; inset: 0;"));
        assert!(css.contains("[data-slot=\"terminal-body\"] > .sr-only"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("var(--cronus-font-mono"));
        assert!(!css.contains("cui-terminal-caret"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(DISPLAY_SURF));
    }
}
