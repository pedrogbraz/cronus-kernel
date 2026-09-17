//! Dedicated ScrambleText renderer. DOM mirrors React: `<span
//! data-slot="scramble-text">` + an `sr-only` copy of the phrase + an
//! aria-hidden display span. React re-renders the display every 40ms
//! (glyph `j` locks after `4·(j + 1)` ticks and, while unlocked, shows
//! `pool[(j·17 + tick·31) % 62]`, which alternates between two pool
//! glyphs); the kernel lays the display out glyph by glyph — `<span
//! data-g0 data-g1><span>c</span></span>` — and COMPONENT_CHROME runs the
//! same decrypt with CSS animations: `::before` flips between the two pool
//! glyphs every 40ms until the lock instant (`160ms·(j + 1)`, from
//! `nth-child`), when the real glyph appears. Whitespace is never
//! scrambled. Zero JS, no inline style.
//!
//! `heading:<size>` puts the docs' `font-display text-<size> text-fg` on the
//! root (class `h-<size>`).

use crate::cronus_ui_dot_pattern::HEADINGS;
use crate::cronus_ui_kit::{choice, esc, label_of};
use crate::parser::ComponentNode;

const POOL: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
/// Longest phrase the stylesheet can index (two `nth-child` digits).
pub const MAX_GLYPHS: usize = 99;

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

/// The two pool glyphs React alternates for glyph `j` (`tick` even / odd).
fn pool_pair(j: usize) -> (char, char) {
    let a = (j * 17) % POOL.len();
    let b = (j * 17 + 31) % POOL.len();
    (POOL[a] as char, POOL[b] as char)
}

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let class = choice(comp, "heading", HEADINGS)
        .map(|h| format!(" class=\"h-{h}\""))
        .unwrap_or_default();
    let display: String = raw_label(comp)
        .chars()
        .take(MAX_GLYPHS)
        .enumerate()
        .map(|(j, ch)| {
            if ch.is_whitespace() {
                "<span>\u{a0}</span>".to_string()
            } else {
                let (g0, g1) = pool_pair(j);
                format!(
                    "<span data-g0=\"{g0}\" data-g1=\"{g1}\"><span>{}</span></span>",
                    esc(&ch.to_string())
                )
            }
        })
        .collect();
    format!(
        "<span data-slot=\"scramble-text\"{class}><span>{label}</span><span aria-hidden=\"true\">{display}</span></span>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
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
        assert!(!html.contains("setTimeout"));
        assert!(!html.contains("zinc-"));
        assert_eq!(html.matches("data-slot=").count(), 1);
    }

    /// React: glyph 0 alternates pool[0]/pool[31] ("A"/"f"), glyph 1 pool[17]/pool[48] ("R"/"w").
    #[test]
    fn root_has_sr_copy_and_glyph_display_with_react_pool_pairs() {
        let html = render(&stub("scramble-text", "Ab c"));
        assert_eq!(
            html,
            "<span data-slot=\"scramble-text\"><span>Ab c</span><span aria-hidden=\"true\"><span data-g0=\"A\" data-g1=\"f\"><span>A</span></span><span data-g0=\"R\" data-g1=\"w\"><span>b</span></span><span>\u{a0}</span><span data-g0=\"z\" data-g1=\"U\"><span>c</span></span></span></span>"
        );
        assert_eq!(pool_pair(0), ('A', 'f'));
        assert_eq!(pool_pair(3), ('z', 'U'));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("scramble-text", "<&"));
        assert!(html.starts_with("<span data-slot=\"scramble-text\"><span>&lt;&amp;</span><span aria-hidden=\"true\"><span data-g0=\"A\" data-g1=\"f\"><span>&lt;</span></span>"));
        assert!(!html.contains("<&"));
        reject_fx(&html);
    }

    /// Docs example: `<ScrambleText className="font-display text-3xl text-fg">cronus-ui</ScrambleText>`.
    #[test]
    fn heading_class_goes_on_the_root() {
        let mut c = stub("scramble-text", "cronus-ui");
        c.props.insert("heading".into(), "3xl".into());
        let html = render(&c);
        assert!(html.starts_with(
            "<span data-slot=\"scramble-text\" class=\"h-3xl\"><span>cronus-ui</span>"
        ));
        assert_eq!(html.matches("data-g0=").count(), 9);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("scramble-text", "Decrypt");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
        assert_eq!(
            dedicated_fn_name("scramble-text"),
            Some("cronus_ui_scramble_text::render")
        );
        assert_eq!(
            renderer_kind("scramble-text"),
            RendererKind::Dedicated("cronus_ui_scramble_text::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("scramble-text", "Decrypt"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"scramble-text\""));
        });
    }

    #[test]
    fn chrome_scramble_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"scramble-text\"] {\n  display: inline;\n  font-family: var(--cronus-font-mono, ui-monospace, monospace);\n}"));
        assert!(css
            .contains("[data-slot=\"scramble-text\"] > span:first-child {\n  position: absolute;"));
        // React: 40ms ticks, lock after 4·(j + 1) ticks = 160ms·(j + 1).
        assert!(css.contains("[data-slot=\"scramble-text\"] > [aria-hidden] > span > span {\n  animation: cui-scramble-lock 1ms steps(1, end) both;\n  animation-delay: calc(160ms * (var(--cui-j) + 1));\n}"));
        assert!(css.contains("animation: cui-scramble 80ms steps(1, end) infinite, cui-scramble-off 1ms steps(1, end) forwards;"));
        assert!(css.contains("@keyframes cui-scramble {\n  0% { content: attr(data-g0); }\n  50% { content: attr(data-g1); }\n}"));
        assert!(css.contains(
            "[data-slot=\"scramble-text\"] > [aria-hidden] > span:nth-child(n+91) { --cui-jt: 9; }"
        ));
        assert!(css.contains(
            "[data-slot=\"scramble-text\"].h-3xl { font-size: 1.875rem; line-height: 2.25rem; }"
        ));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
