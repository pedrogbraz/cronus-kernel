//! Dedicated SparklesText renderer. DOM mirrors React without the injected
//! `<style>`: `<span data-slot="sparkles-text">` + an aria-hidden layer of
//! `count` sparkle `<span>`s (each an 8px four-point star `<svg>`) + a
//! relative `<span>` with the words. React writes each sparkle's slot,
//! delay (`(i·0.35) % 1.6s`) and duration (`1.4 + (i % 3)·0.25s`) inline;
//! COMPONENT_CHROME derives them from `nth-child`. Zero JS, no inline style.
//!
//! `count:` is React's `count` (2–8, default 4); `heading:<size>` puts the
//! docs' `font-display text-<size> text-fg` on the root (class `h-<size>`).

use crate::cronus_ui_dot_pattern::HEADINGS;
use crate::cronus_ui_kit::{attr_num, choice, label_of};
use crate::parser::ComponentNode;

const STAR: &str = "<svg viewBox=\"0 0 16 16\" aria-hidden=\"true\"><path d=\"M8 0 L9.2 6.8 L16 8 L9.2 9.2 L8 16 L6.8 9.2 L0 8 L6.8 6.8 Z\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let count = attr_num::<usize>(comp, "count").unwrap_or(4).clamp(2, 8);
    let class = choice(comp, "heading", HEADINGS)
        .map(|h| format!(" class=\"h-{h}\""))
        .unwrap_or_default();
    let sparkles = format!("<span>{STAR}</span>").repeat(count);
    format!(
        "<span data-slot=\"sparkles-text\"{class}><span aria-hidden=\"true\">{sparkles}</span><span>{}</span></span>",
        label_of(comp)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

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
        assert!(!html.contains("zinc-"));
        assert_eq!(html.matches("data-slot=").count(), 1);
    }

    #[test]
    fn root_has_four_sparkles_then_the_words() {
        let html = render(&stub("sparkles-text", "Magic"));
        assert!(html.starts_with("<span data-slot=\"sparkles-text\"><span aria-hidden=\"true\"><span><svg viewBox=\"0 0 16 16\" aria-hidden=\"true\"><path d=\"M8 0 L9.2 6.8 L16 8 L9.2 9.2 L8 16 L6.8 9.2 L0 8 L6.8 6.8 Z\"></path></svg></span>"));
        assert!(html.ends_with("</span><span>Magic</span></span>"));
        assert_eq!(html.matches("<svg").count(), 4);
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("sparkles-text", "A <B> & \"C\""));
        assert!(html.ends_with("<span>A &lt;B&gt; &amp; &quot;C&quot;</span></span>"));
        reject_fx(&html);
    }

    /// Docs example: `<SparklesText className="font-display text-4xl text-fg">Launch</SparklesText>`.
    #[test]
    fn heading_and_count_follow_the_props() {
        let mut c = stub("sparkles-text", "Launch");
        c.props.insert("heading".into(), "4xl".into());
        c.props.insert("count".into(), "6".into());
        let html = render(&c);
        assert!(html.starts_with(
            "<span data-slot=\"sparkles-text\" class=\"h-4xl\"><span aria-hidden=\"true\">"
        ));
        assert_eq!(html.matches("<svg").count(), 6);
        c.props.insert("count".into(), "40".into());
        assert_eq!(render(&c).matches("<svg").count(), 8);
        c.props.insert("count".into(), "0".into());
        assert_eq!(render(&c).matches("<svg").count(), 2);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("sparkles-text", "Magic"));
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("sparkles-text", "Magic"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"sparkles-text\""));
        });
    }

    #[test]
    fn chrome_sparkles_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"sparkles-text\"] {\n  position: relative; display: inline-block;\n}"
        ));
        assert!(css.contains("[data-slot=\"sparkles-text\"] > [aria-hidden] {\n  position: absolute; inset: 0; overflow: visible; pointer-events: none;\n}"));
        // React: size-2 text-primary, fill-current svg, keyframes cronus-sparkle, backwards fill.
        assert!(css.contains("[data-slot=\"sparkles-text\"] > [aria-hidden] > span {\n  position: absolute; width: 0.5rem; height: 0.5rem;\n  color: var(--cronus-primary);"));
        assert!(css.contains("animation: cui-sparkle 1.4s ease infinite backwards;"));
        assert!(css.contains("> span:nth-child(1) { top: 0%; inset-inline-start: 8%; animation-delay: 0s; animation-duration: 1.4s; }"));
        assert!(css.contains("> span:nth-child(2) { top: 10%; inset-inline-start: 88%; animation-delay: 0.35s; animation-duration: 1.65s; }"));
        assert!(css.contains("> span:nth-child(5) { top: -8%; inset-inline-start: 48%; animation-delay: 1.4s; animation-duration: 1.65s; }"));
        assert!(css.contains("> span:nth-child(8) { top: 100%; inset-inline-start: 52%; animation-delay: 0.85s; animation-duration: 1.65s; }"));
        assert!(css.contains("@keyframes cui-sparkle {\n  0%, 100% { transform: scale(0); opacity: 0; }\n  40% { transform: scale(1); opacity: 1; }\n  70% { transform: scale(0.6); opacity: 0.6; }\n}"));
        assert!(css.contains(
            "[data-slot=\"sparkles-text\"].h-4xl { font-size: 2.25rem; line-height: 2.5rem; }"
        ));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(!css.contains("[data-slot=\"sparkles-text\"]::before"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
