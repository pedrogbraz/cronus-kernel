//! Dedicated Terminal renderer. DOM matches React idle:
//! `<div data-slot="terminal"><div data-slot="terminal-screen">` plus each
//! text as a `terminal-prompt` line. Static (no JS cursor; optional CSS
//! blink in COMPONENT_CHROME). Not catalog `display()` SURF `<section>`
//! without `terminal-screen`, not interact `codey()` `<pre>`.

use crate::cronus_ui_kit::texts;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let lines = texts(comp)
        .into_iter()
        .map(|t| {
            format!(
                "<div data-slot=\"terminal-line\"><span data-slot=\"terminal-prompt\">$</span><span>{t}</span></div>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"terminal\"><div data-slot=\"terminal-screen\">{lines}</div></div>")
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

    fn session(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("terminal", items.first().copied().unwrap_or("npm install"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_stub(html: &str) {
        assert!(!html.contains("<pre"));
        assert!(!html.contains("<code"));
        assert!(!html.contains("<section"));
        assert!(!html.contains("<nav"));
        assert!(!html.contains("style="));
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
    fn root_is_div_with_screen_and_prompt_lines() {
        let html = render(&session(&["npm install", "ready"]));
        assert!(html.starts_with("<div data-slot=\"terminal\">"));
        assert!(html.contains("<div data-slot=\"terminal-screen\">"));
        assert_eq!(html.matches("data-slot=\"terminal-prompt\"").count(), 2);
        assert!(html.contains(
            "<div data-slot=\"terminal-line\"><span data-slot=\"terminal-prompt\">$</span><span>npm install</span></div>"
        ));
        assert!(html.contains(">ready</span>"));
        reject_stub(&html);
        assert_eq!(
            html,
            "<div data-slot=\"terminal\"><div data-slot=\"terminal-screen\"><div data-slot=\"terminal-line\"><span data-slot=\"terminal-prompt\">$</span><span>npm install</span></div><div data-slot=\"terminal-line\"><span data-slot=\"terminal-prompt\">$</span><span>ready</span></div></div></div>"
        );
    }

    #[test]
    fn extra_text_items_become_prompt_lines() {
        let mut c = stub("terminal", "npm install");
        c.items.push(extra("text", "ready"));
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"terminal-prompt\"").count(), 2);
        assert!(html.contains(">npm install</span>"));
        assert!(html.contains(">ready</span>"));
        reject_stub(&html);
    }

    #[test]
    fn label_only_still_emits_one_line() {
        let html = render(&stub("terminal", "npm install"));
        assert!(html.starts_with("<div data-slot=\"terminal\">"));
        assert!(html.contains("data-slot=\"terminal-screen\""));
        assert_eq!(html.matches("data-slot=\"terminal-prompt\"").count(), 1);
        assert!(html.contains(
            "<div data-slot=\"terminal-line\"><span data-slot=\"terminal-prompt\">$</span><span>npm install</span></div>"
        ));
        reject_stub(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("terminal", "A <B> & \"C\""));
        assert!(html.contains(
            "<span data-slot=\"terminal-prompt\">$</span><span>A &lt;B&gt; &amp; &quot;C&quot;</span>"
        ));
        reject_stub(&html);
    }

    #[test]
    fn skips_interact_pre_and_display_surf() {
        let c = session(&["npm install", "ready"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("terminal", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<pre data-slot=\"terminal\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("<code>"));
        assert!(!interact.contains("data-slot=\"terminal-screen\""));
        assert!(!interact.contains("data-slot=\"terminal-prompt\""));
        assert!(html.contains("data-slot=\"terminal-screen\""));
        assert!(html.contains("data-slot=\"terminal-prompt\""));
        assert!(!html.contains("<pre"));
        assert!(!html.contains("<section"));
        assert!(!html.contains(DISPLAY_SURF));
        reject_stub(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&session(&["npm install"]));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"terminal-screen\""));
            assert!(html.contains("data-slot=\"terminal-prompt\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"terminal\"]"));
        assert!(css.contains("[data-slot=\"terminal-screen\"]"));
        assert!(css.contains("[data-slot=\"terminal-prompt\"]"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("var(--cronus-font-mono"));
        assert!(css.contains("var(--cronus-fg-tertiary"));
        assert!(css.contains("@keyframes cui-terminal-caret"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains(DISPLAY_SURF));
    }
}
