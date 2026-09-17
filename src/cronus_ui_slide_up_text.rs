//! Dedicated SlideUpText renderer. DOM matches React `SlideUpText` (Spell):
//! `<span data-slot="slide-up-text">` + a screen-reader `<span>` with the whole
//! phrase + one `aria-hidden` clip `<span>` per word (or line) whose pieces are
//! a clipping `<span>` around an inline-block `<span>` that slides up from 100%.
//!
//! React plays the reveal on mount with motion (JS). The kernel runs the same
//! 0.5s `cubic-bezier(0.625, 0.05, 0, 1)` tween as a CSS animation. Each piece's
//! delay (`data-delay`, 50 ms steps, capped at 2 s) is `delay` plus React's
//! stagger for that piece's index, computed here from `stagger` and `from`.
//! Reduced motion shows the text in place. `inView`, `autoStart:false`,
//! callbacks and custom `transition`s need JS and are not rendered;
//! `split:"characters"` splits by Unicode scalar value, not grapheme.
//! `split:lines` takes one line per `label` / `text` item; `size:2xl|3xl|4xl`
//! is the docs `font-display text-3xl text-fg` typography.

use crate::cronus_ui_kit::{attr, attr_num, esc, item};
use crate::parser::ComponentNode;

/// One `data-delay` step.
const STEP_SECONDS: f64 = 0.05;
/// Last step with a CSS rule (2 s).
const MAX_STEP: u32 = 40;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Split {
    Words,
    Characters,
    Lines,
}

#[derive(Clone, Copy)]
enum From {
    First,
    Last,
    Center,
}

struct Word {
    pieces: Vec<String>,
    needs_space: bool,
}

fn words(text: &str, split: Split) -> Vec<Word> {
    let sep = if split == Split::Lines { '\n' } else { ' ' };
    let parts: Vec<&str> = text.split(sep).collect();
    let last = parts.len() - 1;
    parts
        .iter()
        .enumerate()
        .map(|(i, part)| Word {
            pieces: if split == Split::Characters {
                part.chars().map(String::from).collect()
            } else {
                vec![part.to_string()]
            },
            needs_space: split != Split::Lines && i != last,
        })
        .collect()
}

/// `data-delay` step for a piece: `delay + getStaggerDelay(index)` in React.
fn step(seconds: f64) -> u32 {
    ((seconds.max(0.0) / STEP_SECONDS).round() as u32).min(MAX_STEP)
}

pub fn render(comp: &ComponentNode) -> String {
    let split = match attr(comp, "split").map(str::trim) {
        Some("characters") => Split::Characters,
        Some("lines") => Split::Lines,
        _ => Split::Words,
    };
    // `split:lines`: every `label` / `text` / `item` line is one line of the
    // phrase (a `.cronus` string has no newline escape).
    let joined;
    let text = if split == Split::Lines {
        joined = comp
            .items
            .iter()
            .filter(|i| {
                matches!(i.item_type.as_str(), "label" | "text" | "item") && !i.text.is_empty()
            })
            .map(|i| i.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        joined.as_str()
    } else {
        item(comp, "label").unwrap_or("")
    };
    let from = match attr(comp, "from").map(str::trim) {
        Some("last") => From::Last,
        Some("center") => From::Center,
        _ => From::First,
    };
    let finite = |key: &str, default: f64| {
        attr_num::<f64>(comp, key)
            .filter(|v| v.is_finite())
            .unwrap_or(default)
    };
    let delay = finite("delay", 0.0);
    let stagger = finite("stagger", 0.1);
    let words = words(text, split);
    let total = if split == Split::Characters {
        words
            .iter()
            .map(|w| w.pieces.len() + usize::from(w.needs_space))
            .sum()
    } else {
        words.len()
    } as f64;
    let delay_of = |index: usize| {
        let index = index as f64;
        let stagger_delay = match from {
            From::First => index * stagger,
            From::Last => (total - 1.0 - index) * stagger,
            From::Center => ((total / 2.0).floor() - index).abs() * stagger,
        };
        step(delay + stagger_delay)
    };

    let size = match attr(comp, "size") {
        Some(s @ ("2xl" | "3xl" | "4xl")) => format!(" class=\"t-{s}\""),
        _ => String::new(),
    };
    let mut html = format!(
        "<span data-slot=\"slide-up-text\"{}{size}><span>{}</span>",
        if split == Split::Lines {
            " data-split=\"lines\""
        } else {
            ""
        },
        esc(text)
    );
    // React's `custom` index: characters of previous words, spaces not counted.
    let mut previous = 0;
    for word in &words {
        html.push_str("<span aria-hidden=\"true\">");
        for (i, piece) in word.pieces.iter().enumerate() {
            html.push_str(&format!(
                "<span><span data-delay=\"{}\">{}</span></span>",
                delay_of(previous + i),
                esc(piece)
            ));
        }
        if word.needs_space {
            html.push_str(&format!(
                "<span><span data-delay=\"{}\"> </span></span>",
                delay_of(previous + word.pieces.len())
            ));
        }
        html.push_str("</span>");
        previous += word.pieces.len();
    }
    html.push_str("</span>");
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn delays(html: &str) -> Vec<u32> {
        html.split("data-delay=\"")
            .skip(1)
            .map(|rest| rest[..rest.find('"').unwrap()].parse().unwrap())
            .collect()
    }

    #[test]
    fn words_split_matches_react_dom() {
        let html = render(&stub("slide-up-text", "Ship faster"));
        assert_eq!(
            html,
            "<span data-slot=\"slide-up-text\"><span>Ship faster</span><span aria-hidden=\"true\"><span><span data-delay=\"0\">Ship</span></span><span><span data-delay=\"2\"> </span></span></span><span aria-hidden=\"true\"><span><span data-delay=\"2\">faster</span></span></span></span>"
        );
        for bad in ["<script", "<style", " style=", "v-data=", "onanimation"] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    #[test]
    fn stagger_from_last_and_delay() {
        let mut c = stub("slide-up-text", "a b c");
        c.props.insert("from".into(), "last".into());
        c.props.insert("delay".into(), "0.25".into());
        // Indices a0 space1 b1 space2 c2 (a space shares the next word's index);
        // total 3, so (2 - i) * 0.1 + 0.25 s = 0.45 / 0.35 / 0.25 s.
        assert_eq!(delays(&render(&c)), vec![9, 7, 7, 5, 5]);
    }

    #[test]
    fn characters_from_center_count_spaces_in_total() {
        let mut c = stub("slide-up-text", "ab c");
        c.props.insert("split".into(), "characters".into());
        c.props.insert("from".into(), "center".into());
        c.props.insert("stagger".into(), "0.05".into());
        let html = render(&c);
        // pieces a(0) b(1) space(2) c(2); total 4, center 2.
        assert_eq!(delays(&html), vec![2, 1, 0, 0]);
        assert!(html.contains("<span aria-hidden=\"true\"><span><span data-delay=\"2\">a</span></span><span><span data-delay=\"1\">b</span></span>"), "{html}");
    }

    #[test]
    fn lines_split_is_a_column_without_spaces() {
        let mut c = stub("slide-up-text", "one\ntwo");
        c.props.insert("split".into(), "lines".into());
        let html = render(&c);
        assert!(html.starts_with("<span data-slot=\"slide-up-text\" data-split=\"lines\">"));
        assert!(!html.contains("> </span>"), "{html}");
        assert_eq!(html.matches("aria-hidden=\"true\"").count(), 2);
    }

    #[test]
    fn delays_clamp_and_text_is_escaped() {
        let mut c = stub("slide-up-text", "<b> & x");
        c.props.insert("delay".into(), "9".into());
        let html = render(&c);
        assert!(delays(&html).iter().all(|d| *d == MAX_STEP), "{html}");
        assert!(html.contains("<span>&lt;b&gt; &amp; x</span>"), "{html}");
        assert!(!html.contains("<b>"));
    }

    #[test]
    fn emitted_fixture_parses_to_props() {
        let src = "app \"x\" { port 1 }\ncomponent SlideUpTextCharacters layout:inline style:slide-up-text {\n  stagger:0.05\n  split:\"characters\"\n  from:\"center\"\n  label \"ab c\"\n}\n";
        let comp = crate::parser::parse(src)
            .expect("parse")
            .into_iter()
            .find_map(|n| match n {
                crate::parser::AstNode::Component(c) => Some(c),
                _ => None,
            })
            .expect("component");
        let html = crate::cronus_ui_widgets::render(&comp).expect("family");
        assert_eq!(delays(&html), vec![2, 1, 0, 0], "{html}");
    }

    #[test]
    fn chrome_animates_on_mount_with_stepped_delays() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("animation-name: cui-slide-up-text;"));
        assert!(css.contains("animation-timing-function: cubic-bezier(0.625, 0.05, 0, 1);"));
        assert!(
            css.contains("@keyframes cui-slide-up-text { from { transform: translateY(100%); } }")
        );
        for n in 1..=MAX_STEP {
            let ms = n * 50;
            assert!(
                css.contains(&format!(
                    "[data-slot=\"slide-up-text\"] [data-delay=\"{n}\"] {{ animation-delay: {ms}ms; }}"
                )),
                "step {n}"
            );
        }
        assert_eq!(
            crate::cli::stub_renderer_gate::dedicated_fn_name("slide-up-text"),
            Some("cronus_ui_slide_up_text::render")
        );
    }

    /// Docs "By lines": one `text` line per item, stacked (`flex-col`) with
    /// the 3xl display typography; "From last" staggers back from the end.
    #[test]
    fn lines_from_items_and_display_size() {
        let mut c = stub("slide-up-text", "First line");
        c.props.insert("split".into(), "lines".into());
        c.props.insert("size".into(), "3xl".into());
        for l in ["Second line", "Third line"] {
            c.items.push(crate::parser::ComponentItemNode {
                item_type: "text".into(),
                text: l.into(),
                link: None,
                tone: None,
                config: Default::default(),
            });
        }
        let html = render(&c);
        assert!(html.starts_with("<span data-slot=\"slide-up-text\" data-split=\"lines\" class=\"t-3xl\"><span>First line\nSecond line\nThird line</span><span aria-hidden=\"true\"><span><span data-delay=\"0\">First line</span></span></span><span aria-hidden=\"true\"><span><span data-delay=\"2\">Second line</span></span></span>"));
        assert_eq!(delays(&html), vec![0, 2, 4]);
        let css = include_str!("cronus_ui_css/slide-up-text.css");
        assert!(css.contains("[data-slot=\"slide-up-text\"].t-3xl {"));
        assert!(css.contains("font-size: 1.875rem; line-height: 2.25rem;"));
    }
}
