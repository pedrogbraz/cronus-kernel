//! Dedicated TextEffect renderer. DOM mirrors React:
//! `<p data-slot="text-effect">` (or `as:h1…h4|span`) with a visually hidden
//! `<span class="sr-only">` carrying the accessible text and an `aria-hidden`
//! span holding the visible units: one `inline-block` group per word, each
//! word split into `inline-block` pieces (`per:word` — the word itself,
//! `per:char` — its characters), whitespace kept as plain `white-space: pre`
//! spans. Every piece plays the preset (`fade` / `blur` (default) / `slide`;
//! 0.4 s ease-out) staggered by its index (`i-<n>` class; word 0.04 s, char
//! 0.02 s, or `stagger:`) after `delay:` — Motion's `staggerChildren` /
//! `delayChildren` as CSS animation delays. Off under reduced motion.
//! `size:sm|3xl` and `tone:secondary` are the docs typography; an `action`
//! item (React's Replay button) renders after it with `data-text-effect-replay`
//! so live.js can restart the animation.
//! Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::{attr, attr_num, esc, label_of};
use crate::parser::ComponentNode;

/// Piece indexes with a delay class in the family CSS; later pieces share the last.
pub const MAX_PIECES: usize = 96;

fn units(text: &str, per_char: bool) -> String {
    let mut out = String::new();
    let mut index = 0usize;
    let mut chars = text.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            let mut ws = String::new();
            while let Some(&c) = chars.peek() {
                if !c.is_whitespace() {
                    break;
                }
                ws.push(c);
                chars.next();
            }
            out.push_str(&format!("<span class=\"ws\">{ws}</span>"));
            continue;
        }
        let mut word = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                break;
            }
            word.push(c);
            chars.next();
        }
        out.push_str("<span>");
        if per_char {
            for ch in word.chars() {
                out.push_str(&format!(
                    "<span class=\"i-{}\">{}</span>",
                    index.min(MAX_PIECES - 1),
                    esc(&ch.to_string())
                ));
                index += 1;
            }
        } else {
            out.push_str(&format!(
                "<span class=\"i-{}\">{}</span>",
                index.min(MAX_PIECES - 1),
                esc(&word)
            ));
            index += 1;
        }
        out.push_str("</span>");
    }
    out
}

fn tenths(seconds: f64) -> u32 {
    (seconds * 100.0).round() as u32
}

pub fn render(comp: &ComponentNode) -> String {
    let raw = comp
        .items
        .iter()
        .find(|i| !i.text.is_empty() && !matches!(i.item_type.as_str(), "action" | "button"))
        .map(|i| i.text.clone())
        .unwrap_or_else(|| comp.name.clone());
    let text = esc(&raw);
    let tag = match attr(comp, "as") {
        Some(t @ ("h1" | "h2" | "h3" | "h4" | "span" | "div")) => t,
        _ => "p",
    };
    let per_char = attr(comp, "per") == Some("char");
    let mut classes: Vec<String> = Vec::new();
    match attr(comp, "preset") {
        Some(p @ ("fade" | "slide")) => classes.push(format!("p-{p}")),
        _ => {}
    }
    let stagger = attr_num::<f64>(comp, "stagger").unwrap_or(if per_char { 0.02 } else { 0.04 });
    classes.push(format!("s-{}", tenths(stagger).clamp(0, 50)));
    if let Some(d) = attr_num::<f64>(comp, "delay").filter(|d| *d > 0.0) {
        classes.push(format!("d-{}", ((d * 100.0).round() as u32).clamp(1, 200)));
    }
    if let Some(s @ ("sm" | "3xl")) = attr(comp, "size") {
        classes.push(format!("t-{s}"));
    }
    if attr(comp, "tone") == Some("secondary") {
        classes.push("secondary".into());
    }
    let class = format!(" class=\"{}\"", classes.join(" "));
    let effect = format!(
        "<{tag} data-slot=\"text-effect\"{class}><span class=\"sr-only\">{text}</span><span aria-hidden=\"true\">{}</span></{tag}>",
        units(&raw, per_char)
    );
    match comp
        .items
        .iter()
        .find(|i| matches!(i.item_type.as_str(), "action" | "button") && !i.text.is_empty())
    {
        Some(action) => {
            let btn = crate::cronus_ui_glass_card::action_button(action).replacen(
                "<button ",
                "<button data-text-effect-replay ",
                1,
            );
            format!("<div class=\"cui-text-effect-demo\">{effect}{btn}</div>")
        }
        None => effect,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";
    const CSS: &str = include_str!("cronus_ui_css/text-effect.css");

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
    }

    /// React `TextEffect`: `<p data-slot="text-effect">` > `span.sr-only` +
    /// `span[aria-hidden]` > word groups > staggered pieces. The audit
    /// geometry spec reads the paragraph text as "Ship faster Ship faster".
    #[test]
    fn root_is_p_with_sr_only_and_visible_text_like_react() {
        let html = render(&stub("text-effect", "Ship faster"));
        assert_eq!(
            html,
            "<p data-slot=\"text-effect\" class=\"s-4\"><span class=\"sr-only\">Ship faster</span><span aria-hidden=\"true\"><span><span class=\"i-0\">Ship</span></span><span class=\"ws\"> </span><span><span class=\"i-1\">faster</span></span></span></p>"
        );
        reject_fx(&html);
    }

    /// Docs headline: `as="h3" per="char" preset="blur"` — one piece per
    /// character, 0.02 s apart; the sub-line slides per word after 0.35 s.
    #[test]
    fn per_char_blur_headline_and_delayed_slide_paragraph() {
        let mut c = stub("text-effect", "Ship it");
        c.props.insert("as".into(), "h3".into());
        c.props.insert("per".into(), "char".into());
        c.props.insert("preset".into(), "blur".into());
        c.props.insert("size".into(), "3xl".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<h3 data-slot=\"text-effect\" class=\"s-2 t-3xl\"><span class=\"sr-only\">Ship it</span><span aria-hidden=\"true\"><span><span class=\"i-0\">S</span><span class=\"i-1\">h</span><span class=\"i-2\">i</span><span class=\"i-3\">p</span></span><span class=\"ws\"> </span><span><span class=\"i-4\">i</span><span class=\"i-5\">t</span></span></span></h3>"
        );
        let mut p = stub("text-effect", "Every surface arrives.");
        p.props.insert("preset".into(), "slide".into());
        p.props.insert("delay".into(), "0.35".into());
        p.props.insert("size".into(), "sm".into());
        p.props.insert("tone".into(), "secondary".into());
        let mut replay = c.items[0].clone();
        replay.item_type = "action".into();
        replay.text = "Replay".into();
        replay.config.insert("variant".into(), "outline".into());
        replay.config.insert("size".into(), "sm".into());
        replay.config.insert("icon".into(), "rotate-cw".into());
        p.items.push(replay);
        let html = render(&p);
        assert!(html.starts_with("<div class=\"cui-text-effect-demo\"><p data-slot=\"text-effect\" class=\"p-slide s-4 d-35 t-sm secondary\"><span class=\"sr-only\">Every surface arrives.</span>"));
        assert!(html.contains("<span class=\"i-2\">arrives.</span></span></span></p><button data-text-effect-replay type=\"button\" data-slot=\"button\" data-variant=\"outline\" data-size=\"sm\" class=\"cui-btn\"><svg"));
        assert!(!html.contains(" disabled"));
        assert!(html.ends_with("</svg>Replay</button></div>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("text-effect", "A <B> & \"C\""));
        assert!(html.contains("<span class=\"sr-only\">A &lt;B&gt; &amp; &quot;C&quot;</span>"));
        assert!(html.contains("<span class=\"i-1\">&lt;B&gt;</span>"));
        assert!(html.contains("<span class=\"i-3\">&quot;C&quot;</span>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("text-effect", "Ship faster"));
        reject_fx(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("text-effect", "Ship faster"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"text-effect\""));
        });
    }

    /// Pieces animate individually (0.4 s ease-out, `both`), delayed by
    /// index × stagger (+ delay); the presets are Motion's variants.
    #[test]
    fn chrome_text_effect_via_css() {
        assert!(CSS.contains(
            "[data-slot=\"text-effect\"] {\n  display: block;\n  margin: 0;\n  line-height: 1.5;\n}"
        ));
        assert!(CSS.contains(
            "[data-slot=\"text-effect\"] > .sr-only {\n  position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;"
        ));
        assert!(CSS.contains("[data-slot=\"text-effect\"] > [aria-hidden] > span > span {\n  display: inline-block;\n  animation: cui-text-effect-blur 400ms ease-out both;\n  animation-delay: calc(var(--cui-text-effect-delay, 0s) + var(--cui-text-effect-i, 0) * var(--cui-text-effect-stagger, 0.04s));\n}"));
        assert!(CSS.contains("@keyframes cui-text-effect-blur {\n  from { opacity: 0; filter: blur(8px); }\n  to { opacity: 1; filter: none; }\n}"));
        assert!(CSS.contains(
            "@keyframes cui-text-effect-fade {\n  from { opacity: 0; }\n  to { opacity: 1; }\n}"
        ));
        assert!(CSS.contains("@keyframes cui-text-effect-slide {\n  from { opacity: 0; transform: translateY(8px); }\n  to { opacity: 1; transform: none; }\n}"));
        assert!(CSS.contains("[data-slot=\"text-effect\"].p-slide > [aria-hidden] > span > span { animation-name: cui-text-effect-slide; }"));
        assert!(
            CSS.contains("[data-slot=\"text-effect\"] > [aria-hidden] > .ws { white-space: pre; }")
        );
        assert!(
            CSS.contains("[data-slot=\"text-effect\"].s-2 { --cui-text-effect-stagger: 0.02s; }")
        );
        assert!(
            CSS.contains("[data-slot=\"text-effect\"].d-35 { --cui-text-effect-delay: 0.35s; }")
        );
        assert!(CSS.contains(&format!(
            "[data-slot=\"text-effect\"] .i-{} {{ --cui-text-effect-i: {}; }}",
            MAX_PIECES - 1,
            MAX_PIECES - 1
        )));
        assert!(CSS.contains("@media (prefers-reduced-motion: reduce) {\n  [data-slot=\"text-effect\"] > [aria-hidden] > span > span { animation: none; }\n}"));
        assert!(CSS.contains("[data-slot=\"text-effect\"].t-3xl {"));
        assert!(CSS.contains(".cui-text-effect-demo {"));
        assert!(!CSS.contains("zinc-"));
        assert!(!CSS.contains(FX_BOX));
    }
}
