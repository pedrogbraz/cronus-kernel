//! Dedicated Terminal renderer. DOM matches React `Terminal`:
//! `terminal` > `terminal-chrome` (traffic dots, `terminal-title`, copy button;
//! `chrome:false` drops it and floats the copy button over the body) +
//! `terminal-body` (sr-only `terminal-transcript`, invisible `terminal-sizer`
//! and absolutely positioned `terminal-screen`, both holding `terminal-line`s).
//!
//! Lines: `item` lines are commands (behind the `prompt`, default `$`) and
//! `text` lines their output; with only `text` lines they alternate input /
//! output (even index = command), the audit fixture's mapping. The copy
//! button needs the clipboard API, so it is rendered `disabled` (inert,
//! announced as unavailable) at React's idle geometry.
//!
//! Motion: by default the finished transcript (`data-state="static"`, React's
//! `motionPreference="never"`). `motion:respect|always` plays React's
//! session as CSS: each screen line carries its start time (`s-<s> t-<10ms>`
//! classes → `--cui-terminal-t`); a command shows its prompt and blinking
//! block cursor at its start, types after 400 ms at 30 ms per grapheme
//! (`ch-<n>`: a `steps(n)` width tween over `n ch`; the cursor follows the
//! typed span as a sibling so the clip never hides it), an output fades in
//! 150 ms after its start (`cronus-terminal-fade`, 240 ms). `loop` needs a
//! timer and is not replayed. Not catalog `display()` SURF
//! `<section data-slot="terminal">`, not interact `codey()` `<pre>`.

use crate::cronus_ui_kit::{attr, attr_nonempty, esc, flag, label_of};
use crate::parser::ComponentNode;

const COPY_ICON: &str = concat!(
    "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\">",
    "<rect width=\"14\" height=\"14\" x=\"8\" y=\"8\" rx=\"2\" ry=\"2\"></rect>",
    "<path d=\"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2\"></path></svg>",
);

/// React `DEFAULT_INPUT_DELAY` / `DEFAULT_OUTPUT_DELAY` / `DEFAULT_TYPING_SPEED` (ms).
const INPUT_DELAY: u32 = 400;
const OUTPUT_DELAY: u32 = 150;
const TYPING_SPEED: u32 = 30;
/// Longest command with a typing class (graphemes).
pub const MAX_CHARS: usize = 80;

struct Line {
    input: bool,
    text: String,
    /// Start of the line's turn in the session, in ms.
    start: u32,
    chars: usize,
}

fn lines(comp: &ComponentNode) -> Vec<Line> {
    let has_items = comp.items.iter().any(|i| i.item_type == "item");
    let mut t = 0u32;
    let mut out: Vec<Line> = Vec::new();
    for (n, i) in comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "text" | "item") && !i.text.is_empty())
        .enumerate()
    {
        let input = if has_items {
            i.item_type == "item"
        } else {
            n % 2 == 0
        };
        let chars = i.text.chars().count().min(MAX_CHARS);
        let delay = i
            .config
            .get("delay")
            .and_then(|d| d.trim().parse::<u32>().ok())
            .unwrap_or(if input { INPUT_DELAY } else { OUTPUT_DELAY });
        out.push(Line {
            input,
            text: esc(&i.text),
            start: t,
            chars,
        });
        t += delay
            + if input {
                chars as u32 * TYPING_SPEED
            } else {
                0
            };
    }
    out
}

fn timing_class(ms: u32) -> String {
    let ms = ms.min(30_990);
    format!("s-{} t-{}", ms / 1000, (ms % 1000) / 10)
}

fn row(line: &Line, prompt: &str, animated: bool) -> String {
    let class = if animated {
        let mut c = timing_class(line.start);
        if line.input {
            c.push_str(&format!(" ch-{}", line.chars.max(1)));
        }
        format!(" class=\"{c}\"")
    } else {
        String::new()
    };
    if line.input {
        let cursor = if animated {
            "<span data-slot=\"terminal-cursor\"></span>"
        } else {
            ""
        };
        // The cursor follows the typed span as a sibling (React nests it) so
        // the `steps()` width tween never clips it.
        format!(
            "<div data-slot=\"terminal-line\" data-line-type=\"input\"{class}><span data-slot=\"terminal-prompt\">{prompt}</span><span>{}</span>{cursor}</div>",
            line.text
        )
    } else {
        format!(
            "<div data-slot=\"terminal-line\" data-line-type=\"output\"{class}><span>{}</span></div>",
            line.text
        )
    }
}

pub fn render(comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let has_title = comp
        .items
        .iter()
        .any(|i| matches!(i.item_type.as_str(), "label" | "title") && !i.text.is_empty());
    let prompt = attr_nonempty(comp, "prompt")
        .map(esc)
        .unwrap_or_else(|| "$".into());
    let mut lines = lines(comp);
    if lines.is_empty() {
        lines.push(Line {
            input: true,
            text: title.clone(),
            start: 0,
            chars: title.chars().count(),
        });
    }
    let animated = matches!(attr(comp, "motion"), Some("respect" | "always"));
    let sizer: String = lines.iter().map(|l| row(l, &prompt, false)).collect();
    let screen: String = lines.iter().map(|l| row(l, &prompt, animated)).collect();
    let transcript = lines
        .iter()
        .map(|l| {
            if l.input {
                format!("{prompt} {}", l.text)
            } else {
                l.text.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let copy = |class: &str| {
        format!(
            "<button type=\"button\" data-slot=\"copy-button\" data-variant=\"ghost\"{class} aria-label=\"Copy commands\" disabled>{COPY_ICON}</button>"
        )
    };
    let chrome = flag(comp, "chrome") || attr(comp, "chrome").is_none();
    let (chrome_html, floating) = if chrome {
        (
            format!(
                "<div data-slot=\"terminal-chrome\"><div aria-hidden=\"true\"><span></span><span></span><span></span></div><span data-slot=\"terminal-title\">{}</span>{}</div>",
                if has_title { title.as_str() } else { "" },
                copy("")
            ),
            String::new(),
        )
    } else {
        (String::new(), copy(" class=\"floating\""))
    };
    let region = if has_title {
        format!("Terminal, {title}")
    } else {
        "Terminal".into()
    };
    let state = if animated { "animating" } else { "static" };
    format!(
        "<div data-slot=\"terminal\" data-state=\"{state}\">{chrome_html}<div>{floating}<section data-slot=\"terminal-body\" tabindex=\"0\" aria-label=\"{region}\"><pre data-slot=\"terminal-transcript\" class=\"sr-only\">{transcript}</pre><div aria-hidden=\"true\"><div data-slot=\"terminal-sizer\">{sizer}</div><div data-slot=\"terminal-screen\">{screen}</div></div></section></div></div>"
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
        assert!(css.contains("[data-slot=\"terminal\"] {\n  box-sizing: border-box; width: var(--cui-terminal-w, 100%);"));
        assert!(css.contains(
            "[data-slot=\"terminal-chrome\"] {\n  display: flex; align-items: center; gap: 0.5rem;"
        ));
        assert!(css.contains("font-size: 0.75rem; line-height: 1rem; font-weight: 500;"));
        assert!(css.contains("[data-slot=\"terminal-sizer\"] { visibility: hidden; }"));
        assert!(css.contains("[data-slot=\"terminal-screen\"] { position: absolute; inset: 0;"));
        assert!(css.contains("[data-slot=\"terminal-body\"] > .sr-only"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("var(--cronus-font-mono"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(DISPLAY_SURF));
    }

    /// Docs "Install session" (`motion:respect`): `item` commands type after
    /// 400 ms at 30 ms/char behind a blinking cursor, `text` outputs fade in
    /// 150 ms after their turn; starts: 0, 1180, 1330, 1480, 2210 ms.
    #[test]
    fn typed_session_carries_timing_classes() {
        let mut c = stub("terminal", "zsh");
        c.props.insert("motion".into(), "respect".into());
        c.items.push(extra("item", "npx cronus-ui add terminal"));
        c.items.push(extra("text", "✔ 1 component installed"));
        c.items
            .push(extra("text", "  src/components/ui/terminal.tsx"));
        c.items.push(extra("item", "bun run dev"));
        c.items.push(extra("text", "ready in 312 ms"));
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"terminal\" data-state=\"animating\"><div data-slot=\"terminal-chrome\">"));
        let screen = html.split("data-slot=\"terminal-screen\">").nth(1).unwrap();
        assert!(screen.starts_with("<div data-slot=\"terminal-line\" data-line-type=\"input\" class=\"s-0 t-0 ch-26\"><span data-slot=\"terminal-prompt\">$</span><span>npx cronus-ui add terminal</span><span data-slot=\"terminal-cursor\"></span></div><div data-slot=\"terminal-line\" data-line-type=\"output\" class=\"s-1 t-18\"><span>✔ 1 component installed</span></div><div data-slot=\"terminal-line\" data-line-type=\"output\" class=\"s-1 t-33\">"));
        assert!(screen.contains("data-line-type=\"input\" class=\"s-1 t-48 ch-11\"><span data-slot=\"terminal-prompt\">$</span><span>bun run dev</span><span data-slot=\"terminal-cursor\"></span></div><div data-slot=\"terminal-line\" data-line-type=\"output\" class=\"s-2 t-21\">"));
        // The sizer keeps the finished transcript, untimed and without cursors.
        let sizer = html
            .split("data-slot=\"terminal-sizer\">")
            .nth(1)
            .unwrap()
            .split("data-slot=\"terminal-screen\"")
            .next()
            .unwrap();
        assert!(!sizer.contains("class="));
        assert!(!sizer.contains("terminal-cursor"));
        assert!(html.contains("$ npx cronus-ui add terminal\n✔ 1 component installed\n  src/components/ui/terminal.tsx\n$ bun run dev\nready in 312 ms</pre>"));
        reject_stub(&html);
    }

    /// Docs "Static, chrome-less log": no chrome bar, the copy button floats
    /// over the body corner, the transcript is finished (`static`).
    #[test]
    fn chromeless_static_log_floats_the_copy_button() {
        let mut c = stub("terminal", "log");
        c.items.clear();
        c.props.insert("chrome".into(), "false".into());
        c.props.insert("motion".into(), "never".into());
        c.items.push(extra("item", "cronus deploy --prod"));
        c.items.push(extra("text", "Build completed in 8.2s"));
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"terminal\" data-state=\"static\"><div><button type=\"button\" data-slot=\"copy-button\" data-variant=\"ghost\" class=\"floating\" aria-label=\"Copy commands\" disabled>"));
        assert!(!html.contains("terminal-chrome"));
        assert!(html.contains("aria-label=\"Terminal\">"));
        assert!(!html.contains("terminal-cursor"));
        assert!(html.contains("$ cronus deploy --prod\nBuild completed in 8.2s</pre>"));
        reject_stub(&html);
    }

    #[test]
    fn chrome_session_timing_via_css() {
        let css = include_str!("cronus_ui_css/terminal.css");
        assert!(css.contains("@keyframes cui-terminal-type { from { width: 0; } to { width: var(--cui-terminal-ch, 0ch); } }"));
        assert!(css.contains("@keyframes cui-terminal-fade {\n  from { opacity: 0; transform: translateY(2px); }\n  to { opacity: 1; transform: none; }\n}"));
        assert!(css.contains("@keyframes cui-terminal-blink {"));
        assert!(css.contains("animation: cui-terminal-type var(--cui-terminal-type, 0ms) steps(var(--cui-terminal-steps, 1), end) calc(var(--cui-terminal-t, 0ms) + 400ms) both;"));
        assert!(css.contains("animation: cui-terminal-fade 240ms ease-out calc(var(--cui-terminal-t, 0ms) + 150ms) both;"));
        assert!(css.contains("[data-slot=\"terminal-line\"].ch-26 { --cui-terminal-ch: 26ch; --cui-terminal-steps: 26; --cui-terminal-type: 780ms; }"));
        assert!(css.contains("[data-slot=\"terminal-line\"].s-2 { --cui-terminal-s: 2000ms; }"));
        assert!(css.contains("[data-slot=\"terminal-line\"].t-48 { --cui-terminal-t: calc(var(--cui-terminal-s, 0ms) + 480ms); }"));
        assert!(css.contains("[data-slot=\"terminal-cursor\"] {"));
        assert!(css.contains("[data-slot=\"terminal-chrome\"] [data-slot=\"copy-button\"], [data-slot=\"terminal\"] > div > [data-slot=\"copy-button\"].floating {"));
        assert!(css.contains("[data-slot=\"terminal\"] > div > [data-slot=\"copy-button\"].floating {\n  position: absolute; inset-inline-end: 0.5rem; top: 0.5rem; z-index: 10;"));
        assert!(css.contains("prefers-reduced-motion"));
    }
}
