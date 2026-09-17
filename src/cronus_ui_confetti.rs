//! Dedicated Confetti renderer. React draws the burst on an aria-hidden
//! `<canvas>` (48 pieces from the pointer, gravity, ~1s life, hue `i·29`);
//! the kernel emits no canvas and no JS: `<div data-slot="confetti">` + an
//! aria-hidden overflow layer of piece `<span>`s (launch vector, hue and
//! size per `nth-child` in COMPONENT_CHROME) + the relative content
//! `<div>`. The burst fires from the centre while the wrapper is `:active`.
//!
//! `action "…"` renders the docs' `<Button>` child; `count:` is React's
//! `count` (8–48 pieces, default 48; the stylesheet vectors stop at 48);
//! `card:true` is the docs wrapper (`grid min-h-40 place-items-center
//! rounded-2xl border bg-surface-raised`); `heading:<size>` wraps a text
//! label in `<p class="h-<size>">`.

use crate::cronus_ui_dot_pattern::HEADINGS;
use crate::cronus_ui_kit::{attr_num, choice, esc, flag, item, label_of};
use crate::parser::ComponentNode;

pub const MAX_PIECES: usize = 48;

pub fn render(comp: &ComponentNode) -> String {
    let count = attr_num::<usize>(comp, "count")
        .unwrap_or(MAX_PIECES)
        .clamp(8, MAX_PIECES);
    let class = if flag(comp, "card") {
        " class=\"card\""
    } else {
        ""
    };
    let content = match item(comp, "action") {
        Some(action) if !action.is_empty() => {
            crate::cronus_ui::button(&esc(action), "primary", "md", None)
        }
        _ => {
            let text = label_of(comp);
            match choice(comp, "heading", HEADINGS) {
                Some(h) => format!("<p class=\"h-{h}\">{text}</p>"),
                None => text,
            }
        }
    };
    let pieces = "<span></span>".repeat(count);
    format!(
        "<div data-slot=\"confetti\"{class}><div aria-hidden=\"true\">{pieces}</div><div>{content}</div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<canvas"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("onpointerdown="));
        assert!(!html.contains("zinc-"));
        assert_eq!(html.matches("data-slot=\"confetti\"").count(), 1);
    }

    #[test]
    fn root_has_piece_layer_then_content_div() {
        let html = render(&stub("confetti", "Party"));
        assert!(
            html.starts_with("<div data-slot=\"confetti\"><div aria-hidden=\"true\"><span></span>")
        );
        assert!(html.ends_with("</div><div>Party</div></div>"));
        assert_eq!(html.matches("<span></span>").count(), 48);
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("confetti", "A <B> & \"C\""));
        assert!(html.ends_with("<div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"));
        reject_fx(&html);
    }

    /// Docs example: `<Confetti className="grid min-h-40 …"><Button>Celebrate</Button>`.
    #[test]
    fn action_renders_the_docs_button_inside_a_card() {
        let mut c = stub("confetti", "Burst");
        c.items.push(ComponentItemNode {
            item_type: "action".into(),
            text: "Celebrate".into(),
            link: None,
            tone: None,
            config: Default::default(),
        });
        c.props.insert("card".into(), "true".into());
        let html = render(&c);
        assert!(html
            .starts_with("<div data-slot=\"confetti\" class=\"card\"><div aria-hidden=\"true\">"));
        assert!(html.ends_with("<div><button type=\"button\" data-slot=\"button\" data-variant=\"primary\" data-size=\"md\" class=\"cui-btn\">Celebrate</button></div></div>"));
        assert!(!html.contains("Burst"));
    }

    #[test]
    fn count_is_clamped_to_the_stylesheet_vectors() {
        let mut c = stub("confetti", "Party");
        c.props.insert("count".into(), "160".into());
        assert_eq!(render(&c).matches("<span></span>").count(), 48);
        c.props.insert("count".into(), "2".into());
        assert_eq!(render(&c).matches("<span></span>").count(), 8);
        c.props.insert("count".into(), "12".into());
        assert_eq!(render(&c).matches("<span></span>").count(), 12);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("confetti", "Party"));
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("confetti", "Party"));
            reject_fx(&html);
        });
    }

    #[test]
    fn chrome_confetti_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"confetti\"] {\n  position: relative;\n  width: var(--cui-confetti-w, 100%); min-height: 8rem;"));
        assert!(css.contains("[data-slot=\"confetti\"] > [aria-hidden] {\n  position: absolute; inset: 0; overflow: hidden; pointer-events: none;"));
        assert!(
            css.contains("[data-slot=\"confetti\"] > div:last-child {\n  position: relative;\n}")
        );
        // React: oklch(0.75 0.18 hue), size × 0.6 tall, 1s life, gravity 0.12 px/frame².
        assert!(css.contains("background: oklch(0.75 0.18 var(--cui-h));"));
        assert!(css.contains("height: calc(var(--cui-s) * 0.6);"));
        assert!(css.contains("[data-slot=\"confetti\"]:active > [aria-hidden] > span {\n  animation: cui-confetti 1s linear forwards;"));
        assert!(css.contains(
            "100% { translate: var(--cui-dx) calc(var(--cui-dy) + 231px); opacity: 0; }"
        ));
        for i in 1..=MAX_PIECES {
            assert!(
                css.contains(&format!(
                    "[data-slot=\"confetti\"] > [aria-hidden] > span:nth-child({i}) {{ --cui-dx: "
                )),
                "{i}"
            );
        }
        assert!(css.contains("> span:nth-child(2) { --cui-dx: -297px; --cui-dy: 148px; --cui-h: 29; --cui-s: 4.5px; }"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(css.contains("[data-slot=\"confetti\"].card {\n  display: grid; place-items: center; min-height: 10rem;"));
        assert!(!css.contains("<canvas"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
