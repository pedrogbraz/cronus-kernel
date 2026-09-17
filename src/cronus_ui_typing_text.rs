//! Dedicated TypingText renderer. One phrase renders like React's static
//! mode (the audit fixture: `<span data-slot="typing-text">Shipping</span>`,
//! no caret). Two or more phrases (`text`/`item` lines) or `loop:true`
//! render React's typewriter with zero JS: the phrases are laid out glyph
//! by glyph (`<span aria-hidden>` per phrase, `<span><span>g</span></span>`
//! per glyph) and COMPONENT_CHROME types, pauses, deletes and moves on with
//! React's cadence (50ms per typed glyph, 1400ms rest, 30ms per deleted
//! glyph, 80ms before the next phrase), looping over the whole list.
//!
//! Every timing is a CSS animation on `font-size` (0 ↔ 1em): the outer glyph
//! span turns on at the glyph's typed instant, the inner span turns off at
//! its deleted instant; both loop over the full cycle, so the glyph is
//! visible exactly while both are on. The cycle length (`t3-…t0`, 10ms
//! digits on the root), each phrase's offset and length (`s3-…s0`, `n1-n0`
//! on the phrase span) and each glyph's index (`nth-child`) feed the delays;
//! the `f2/f4/f8/f16` class picks the keyframe window that fits the glyph's
//! visible span. A blinking caret (`animate-pulse`) follows the typed text.
//! An `sr-only` copy of the first phrase is what assistive tech reads.
//!
//! `heading:<size>` puts the docs' `font-display text-<size> text-fg` on the
//! root (class `h-<size>`).

use crate::cronus_ui_dot_pattern::HEADINGS;
use crate::cronus_ui_kit::{attr, choice, esc, truthy};
use crate::parser::ComponentNode;

/// Longest phrase the stylesheet can index (two `nth-child` digits).
pub const MAX_GLYPHS: usize = 99;
/// Most phrases in one cycle.
pub const MAX_PHRASES: usize = 8;

const TYPE_MS: u32 = 50;
const DELETE_MS: u32 = 30;
const PAUSE_MS: u32 = 1400;
const NEXT_MS: u32 = 80;

fn phrases(comp: &ComponentNode) -> Vec<String> {
    let out: Vec<String> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "text" | "item") && !i.text.is_empty())
        .map(|i| i.text.clone())
        .take(MAX_PHRASES)
        .collect();
    if out.is_empty() {
        vec![raw_label(comp)]
    } else {
        out
    }
}

fn raw_label(comp: &ComponentNode) -> String {
    for kind in ["label", "title", "text", "value"] {
        if let Some(t) = crate::cronus_ui_kit::item(comp, kind) {
            if !t.is_empty() {
                return t.to_string();
            }
        }
    }
    comp.items
        .iter()
        .find(|i| !i.text.is_empty())
        .map(|i| i.text.clone())
        .unwrap_or_else(|| comp.name.clone())
}

/// Phrase length in the cycle: type + rest + delete + hand-off.
fn phrase_ms(glyphs: u32) -> u32 {
    (TYPE_MS + DELETE_MS) * glyphs + PAUSE_MS + NEXT_MS
}

/// `prefix<d>-<digit>` classes for a 10ms count (thousands … units), zero
/// digits omitted (the stylesheet defaults them to 0).
fn digit_classes(prefix: &str, units: u32) -> String {
    let units = units.min(9999);
    let mut out = String::new();
    for (place, div) in [(3, 1000), (2, 100), (1, 10), (0, 1)] {
        let d = (units / div) % 10;
        if d != 0 {
            out.push_str(&format!(" {prefix}{place}-{d}"));
        }
    }
    out
}

/// Keyframe window class for a glyph visible `visible` ms of a `cycle` ms loop.
fn window_class(visible: u32, cycle: u32) -> &'static str {
    let ratio = visible as f64 / cycle as f64;
    if ratio <= 0.5 {
        "f2"
    } else if ratio <= 0.75 {
        "f4"
    } else if ratio <= 0.875 {
        "f8"
    } else {
        "f16"
    }
}

pub fn render(comp: &ComponentNode) -> String {
    let list = phrases(comp);
    let loops = attr(comp, "loop").map(truthy).unwrap_or(list.len() > 1);
    let heading = choice(comp, "heading", HEADINGS)
        .map(|h| format!(" h-{h}"))
        .unwrap_or_default();
    if !loops {
        let class = if heading.is_empty() {
            String::new()
        } else {
            format!(" class=\"{}\"", heading.trim_start())
        };
        return format!(
            "<span data-slot=\"typing-text\"{class}>{}</span>",
            esc(&list[0])
        );
    }
    let glyphs: Vec<Vec<char>> = list
        .iter()
        .map(|p| p.chars().take(MAX_GLYPHS).collect())
        .collect();
    let cycle: u32 = glyphs.iter().map(|g| phrase_ms(g.len() as u32)).sum();
    let mut offset = 0u32;
    let mut body = String::new();
    for phrase in &glyphs {
        let n = phrase.len() as u32;
        let mut spans = String::new();
        for (j, ch) in phrase.iter().enumerate() {
            let j = j as u32;
            let typed = offset + TYPE_MS * (j + 1);
            let deleted = offset + TYPE_MS * n + PAUSE_MS + DELETE_MS * (n - j);
            let glyph = if *ch == ' ' {
                "\u{a0}".to_string()
            } else {
                esc(&ch.to_string())
            };
            spans.push_str(&format!(
                "<span class=\"{}\"><span>{glyph}</span></span>",
                window_class(deleted - typed, cycle)
            ));
        }
        body.push_str(&format!(
            "<span aria-hidden=\"true\" class=\"phrase{}{}\">{spans}</span>",
            digit_classes("s", offset / 10),
            digit_classes("n", n)
        ));
        offset += phrase_ms(n);
    }
    format!(
        "<span data-slot=\"typing-text\" class=\"loop{heading}{}\"><span>{}</span>{body}<span aria-hidden=\"true\" class=\"caret\"></span></span>",
        digit_classes("t", cycle / 10),
        esc(&list[0])
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn text(t: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "text".into(),
            text: t.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("<style"));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<div"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setTimeout"));
        assert!(!html.contains("zinc-"));
    }

    /// One phrase is React's static mode (the audit fixture): text only, no caret.
    #[test]
    fn root_is_span_with_full_label_not_fx_title_box() {
        let html = render(&stub("typing-text", "Ship faster"));
        assert_eq!(html, "<span data-slot=\"typing-text\">Ship faster</span>");
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("typing-text", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<span data-slot=\"typing-text\">A &lt;B&gt; &amp; &quot;C&quot;</span>"
        );
        reject_fx(&html);
    }

    /// Docs example: three phrases type, rest, delete and cycle. "Design systems."
    /// is 15 glyphs, so its slot is 80·15 + 1480 = 2680ms; the second phrase
    /// starts there (`s2-2 s1-6 s0-8` in 10ms units).
    #[test]
    fn phrases_render_glyph_spans_with_cycle_offsets_and_lengths() {
        let mut c = stub("typing-text", "Design systems.");
        c.items.push(text("Design systems."));
        c.items.push(text("Product surfaces."));
        c.items.push(text("Copy you can upgrade."));
        c.props.insert("heading".into(), "3xl".into());
        let html = render(&c);
        // Cycle: (15 + 17 + 21) · 80 + 3 · 1480 = 8680ms → 868 units.
        assert!(html.starts_with(
            "<span data-slot=\"typing-text\" class=\"loop h-3xl t2-8 t1-6 t0-8\"><span>Design systems.</span><span aria-hidden=\"true\" class=\"phrase n1-1 n0-5\"><span class=\"f2\"><span>D</span></span>"
        ));
        assert!(html.contains("<span aria-hidden=\"true\" class=\"phrase s2-2 s1-6 s0-8 n1-1 n0-7\"><span class=\"f2\"><span>P</span></span>"));
        // 2680 + 2840 = 5520ms → s2-5 s1-5 s0-2.
        assert!(
            html.contains("<span aria-hidden=\"true\" class=\"phrase s2-5 s1-5 s0-2 n1-2 n0-1\">")
        );
        assert!(html.contains("<span class=\"f2\"><span>\u{a0}</span></span>"));
        assert!(html.ends_with("<span aria-hidden=\"true\" class=\"caret\"></span></span>"));
        assert_eq!(html.matches("class=\"phrase").count(), 3);
        assert_eq!(html.matches("<span class=\"f").count(), 15 + 17 + 21);
        reject_fx(&html);
    }

    /// A glyph visible for more than half the cycle needs the wider keyframe window.
    #[test]
    fn window_class_widens_with_the_visible_share() {
        let mut c = stub("typing-text", "x");
        c.items.push(text("A rather long first phrase to type"));
        c.items.push(text("b"));
        let html = render(&c);
        assert!(html.contains("class=\"f4\"><span>A</span>"));
        assert!(html.contains("class=\"f2\"><span>b</span>"));
        assert_eq!(window_class(1, 10), "f2");
        assert_eq!(window_class(6, 10), "f4");
        assert_eq!(window_class(8, 10), "f8");
        assert_eq!(window_class(9, 10), "f16");
    }

    #[test]
    fn loop_prop_switches_modes() {
        let mut c = stub("typing-text", "Once");
        c.props.insert("loop".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("class=\"loop t2-1 t1-8\""));
        assert!(html.contains("class=\"caret\""));
        let mut d = stub("typing-text", "Design");
        d.items.push(text("Design"));
        d.items.push(text("System"));
        d.props.insert("loop".into(), "false".into());
        assert_eq!(render(&d), "<span data-slot=\"typing-text\">Design</span>");
    }

    #[test]
    fn digit_classes_skip_zero_places() {
        assert_eq!(digit_classes("t", 868), " t2-8 t1-6 t0-8");
        assert_eq!(digit_classes("s", 0), "");
        assert_eq!(digit_classes("n", 20), " n1-2");
        assert_eq!(digit_classes("t", 50000), " t3-9 t2-9 t1-9 t0-9");
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("typing-text", "Ship faster"));
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("typing-text", "Ship faster"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"typing-text\""));
        });
    }

    #[test]
    fn chrome_typing_cadence_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"typing-text\"] {\n  display: inline; white-space: pre-wrap;\n}"
        ));
        // React caret: ms-px inline-block h-[1em] w-px translate-y-[0.1em] animate-pulse bg-fg.
        assert!(css.contains("[data-slot=\"typing-text\"] > .caret {\n  display: inline-block; width: 1px; height: 1em; margin-inline-start: 1px;"));
        assert!(
            css.contains("animation: cui-typing-pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;")
        );
        assert!(css.contains("@keyframes cui-typing-pulse {\n  50% { opacity: 0.5; }\n}"));
        // Cadence: 50ms typed, 1400ms rest, 30ms deleted, 80ms hand-off.
        assert!(css.contains("--cui-a: calc(var(--cui-s) + 50ms * (var(--cui-j) + 1));"));
        assert!(css.contains("--cui-b: calc(var(--cui-s) + 50ms * var(--cui-n) + 1400ms + 30ms * (var(--cui-n) - var(--cui-j)));"));
        assert!(css.contains("animation: cui-type-2 var(--cui-t) steps(1, end) infinite both;"));
        assert!(css.contains("animation-delay: calc(var(--cui-a) - var(--cui-g) * var(--cui-t));"));
        assert!(css.contains("animation-delay: calc(var(--cui-b) - var(--cui-t));"));
        assert!(css.contains(
            "@keyframes cui-type-2 {\n  0% { font-size: 0; }\n  50%, 100% { font-size: 1em; }\n}"
        ));
        assert!(css.contains("@keyframes cui-type-16 {\n  0% { font-size: 0; }\n  6.25%, 100% { font-size: 1em; }\n}"));
        assert!(css.contains("[data-slot=\"typing-text\"].t3-9 { --cui-t3: 9; }"));
        assert!(css.contains("[data-slot=\"typing-text\"] > .s0-1 { --cui-s0: 1; }"));
        assert!(css.contains("[data-slot=\"typing-text\"] > .n1-9 { --cui-n1: 9; }"));
        assert!(css.contains(
            "[data-slot=\"typing-text\"] > .phrase > span:nth-child(n+91) { --cui-jt: 9; }"
        ));
        assert!(css.contains("[data-slot=\"typing-text\"] > .phrase > .f16 { --cui-g: 0.9375; animation-name: cui-type-16; }"));
        assert!(css.contains(
            "[data-slot=\"typing-text\"] > .phrase > .f16 > span { animation-name: cui-type-16; }"
        ));
        assert!(css.contains(
            "[data-slot=\"typing-text\"].h-3xl { font-size: 1.875rem; line-height: 2.25rem; }"
        ));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(!css.contains("[data-slot=\"typing-text\"]::after"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
