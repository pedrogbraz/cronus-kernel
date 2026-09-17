//! Dedicated WordRotate renderer. DOM mirrors React: `<span
//! data-slot="word-rotate">` + a polite `sr-only` live copy of the current
//! word + one invisible `data-word-rotate-sizer` per word (the box stays as
//! wide as the longest word) + the visible word. React swaps the visible
//! `motion.span` every 2200ms (exit `y:-40% opacity:0` 350ms, then enter
//! from `y:40% opacity:0` 350ms, `[0.22,1,0.36,1]`); the kernel keeps every
//! word in the DOM and COMPONENT_CHROME runs the same swap: a `mount` copy of
//! the first word (visible on load, exits at 2200ms, never returns) plus
//! one looping `word` span per entry whose keyframes (`cui-word-rotate-<n>`,
//! cycle `2200ms × n`) enter, hold, exit and wait, offset by
//! `2200ms · i + 350ms` (the first word rejoins one cycle later). Zero JS,
//! no inline style. A single word does not cycle (React starts no interval).
//!
//! Docs sentence from the `.cronus`: `prefix:"Ship "` / `suffix:"."` put
//! words around the rotator and `heading:3xl` is the docs
//! `<p className="font-display text-3xl text-fg">`; any of the three wraps
//! the slot in that `<p>` (class `h-<size>`).

use crate::cronus_ui_dot_pattern::HEADINGS;
use crate::cronus_ui_kit::{attr, choice, esc, label_of};
use crate::parser::ComponentNode;

/// Most words one cycle can hold (the stylesheet has keyframes for 2–8).
pub const MAX_WORDS: usize = 8;

pub fn render(comp: &ComponentNode) -> String {
    let list = words(comp);
    let current = list.first().cloned().unwrap_or_default();
    let sizers: String = list
        .iter()
        .map(|w| format!("<span aria-hidden=\"true\" data-word-rotate-sizer=\"\">{w}</span>"))
        .collect();
    let rotator = if list.len() < 2 {
        format!(
            "<span data-slot=\"word-rotate\"><span aria-live=\"polite\">{current}</span>{sizers}<span aria-hidden=\"true\">{current}</span></span>"
        )
    } else {
        let loop_words: String = list
            .iter()
            .map(|w| format!("<span aria-hidden=\"true\" class=\"word\">{w}</span>"))
            .collect();
        format!(
            "<span data-slot=\"word-rotate\" class=\"n-{}\"><span aria-live=\"polite\">{current}</span>{sizers}<span aria-hidden=\"true\" class=\"mount\">{current}</span>{loop_words}</span>",
            list.len()
        )
    };
    let prefix = attr(comp, "prefix").map(esc).unwrap_or_default();
    let suffix = attr(comp, "suffix").map(esc).unwrap_or_default();
    let heading = choice(comp, "heading", HEADINGS);
    if prefix.is_empty() && suffix.is_empty() && heading.is_none() {
        return rotator;
    }
    let class = heading
        .map(|h| format!(" class=\"h-{h}\""))
        .unwrap_or_default();
    format!("<p{class}>{prefix}{rotator}{suffix}</p>")
}

fn words(comp: &ComponentNode) -> Vec<String> {
    let out: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type != "label" && i.item_type != "title" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .take(MAX_WORDS)
        .collect();
    if out.is_empty() {
        vec![label_of(comp)]
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn extra(text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "text".into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<div"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("zinc-"));
    }

    /// Audit fixture: label + two words. The mount copy shows "Design" on load;
    /// the two looping copies take over from the first swap.
    #[test]
    fn fixture_label_plus_words_locks_width_to_all_words() {
        let mut c = stub("word-rotate", "Design");
        c.items.push(extra("Design"));
        c.items.push(extra("System"));
        let html = render(&c);
        assert_eq!(
            html,
            "<span data-slot=\"word-rotate\" class=\"n-2\"><span aria-live=\"polite\">Design</span><span aria-hidden=\"true\" data-word-rotate-sizer=\"\">Design</span><span aria-hidden=\"true\" data-word-rotate-sizer=\"\">System</span><span aria-hidden=\"true\" class=\"mount\">Design</span><span aria-hidden=\"true\" class=\"word\">Design</span><span aria-hidden=\"true\" class=\"word\">System</span></span>"
        );
        assert_eq!(html.matches("data-slot=").count(), 1);
        reject_fx(&html);
    }

    /// One word never cycles (React starts no interval): no mount/loop copies.
    #[test]
    fn bare_label_is_the_only_word() {
        let html = render(&stub("word-rotate", "A <B>"));
        assert_eq!(
            html,
            "<span data-slot=\"word-rotate\"><span aria-live=\"polite\">A &lt;B&gt;</span><span aria-hidden=\"true\" data-word-rotate-sizer=\"\">A &lt;B&gt;</span><span aria-hidden=\"true\">A &lt;B&gt;</span></span>"
        );
        reject_fx(&html);
    }

    /// Docs example: `<p className="font-display text-3xl text-fg">Ship <WordRotate
    /// words={["faster", "calmer", "on-brand"]} />.</p>`.
    #[test]
    fn prefix_suffix_and_heading_wrap_the_docs_sentence() {
        let mut c = stub("word-rotate", "Ship");
        c.items.push(extra("faster"));
        c.items.push(extra("calmer"));
        c.items.push(extra("on-brand"));
        c.props.insert("prefix".into(), "Ship ".into());
        c.props.insert("suffix".into(), ".".into());
        c.props.insert("heading".into(), "3xl".into());
        let html = render(&c);
        assert!(html.starts_with("<p class=\"h-3xl\">Ship <span data-slot=\"word-rotate\" class=\"n-3\"><span aria-live=\"polite\">faster</span>"));
        assert!(
            html.ends_with("<span aria-hidden=\"true\" class=\"word\">on-brand</span></span>.</p>")
        );
        assert_eq!(html.matches("class=\"word\"").count(), 3);
        assert!(!html.contains(">Ship</span>"));
        reject_fx(&html);
    }

    #[test]
    fn words_are_capped_to_the_stylesheet_cycles() {
        let mut c = stub("word-rotate", "w");
        for i in 0..12 {
            c.items.push(extra(&format!("w{i}")));
        }
        let html = render(&c);
        assert!(html.contains("class=\"n-8\""));
        assert_eq!(html.matches("class=\"word\"").count(), 8);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("word-rotate", "Ship"));
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("word-rotate", "Ship"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"word-rotate\""));
        });
    }

    #[test]
    fn chrome_word_rotate_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"word-rotate\"] {\n  position: relative; display: inline-grid;\n  height: 1.2em; overflow: hidden;\n  vertical-align: baseline;\n}"));
        assert!(
            css.contains("[data-slot=\"word-rotate\"] > span:first-child {\n  position: absolute;")
        );
        assert!(css.contains("[data-slot=\"word-rotate\"] > [data-word-rotate-sizer] {\n  visibility: hidden; grid-area: 1 / 1; white-space: nowrap;\n}"));
        // React: 2200ms interval, 350ms exit then 350ms enter, y ±40%, ease [0.22, 1, 0.36, 1].
        assert!(css.contains("[data-slot=\"word-rotate\"] > .word {\n  opacity: 0;\n  animation: cui-word-rotate-2 4400ms linear infinite both;\n  animation-delay: calc(2200ms * var(--cui-i) + 350ms);\n}"));
        assert!(css.contains("[data-slot=\"word-rotate\"] > .mount {\n  animation: cui-word-rotate-mount-2 4400ms linear 1 forwards;\n}"));
        assert!(css.contains("[data-slot=\"word-rotate\"].n-3 > .word { animation-name: cui-word-rotate-3; animation-duration: 6600ms; }"));
        assert!(css.contains("[data-slot=\"word-rotate\"].n-3 > .mount { animation-name: cui-word-rotate-mount-3; animation-duration: 6600ms; }"));
        assert!(css.contains("[data-slot=\"word-rotate\"].n-3 > .word:nth-last-child(3) { animation-delay: 6950ms; }"));
        assert!(css.contains("@keyframes cui-word-rotate-2 {\n  0% { translate: 0 40%; opacity: 0; animation-timing-function: cubic-bezier(.22, 1, .36, 1); }\n  7.9545% { translate: 0 0; opacity: 1; animation-timing-function: linear; }\n  42.0455% { translate: 0 0; opacity: 1; animation-timing-function: cubic-bezier(.22, 1, .36, 1); }\n  50%, 100% { translate: 0 -40%; opacity: 0; }\n}"));
        assert!(css.contains("@keyframes cui-word-rotate-mount-2 {\n  0%, 50% { translate: 0 0; opacity: 1; animation-timing-function: cubic-bezier(.22, 1, .36, 1); }\n  57.9545%, 100% { translate: 0 -40%; opacity: 0; }\n}"));
        for n in 2..=MAX_WORDS {
            assert!(
                css.contains(&format!("@keyframes cui-word-rotate-{n} {{")),
                "{n}"
            );
            assert!(
                css.contains(&format!("@keyframes cui-word-rotate-mount-{n} {{")),
                "{n}"
            );
        }
        assert!(css.contains(
            "p:has(> [data-slot=\"word-rotate\"]) {\n  margin: 0; color: var(--cronus-fg);\n}"
        ));
        assert!(css.contains("p.h-3xl:has(> [data-slot=\"word-rotate\"]) { font-size: 1.875rem; line-height: 2.25rem; }"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
