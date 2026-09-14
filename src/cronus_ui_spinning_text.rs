//! Dedicated SpinningText renderer. DOM mirrors React:
//! `<div data-slot="spinning-text">` + a visually hidden copy of the phrase
//! (`sr-only`) + an aria-hidden orbit `<div>` holding one `<span>` per glyph
//! (spaces become U+00A0). React writes each glyph's
//! `rotate(i/n·360deg) translateY(-radius)` inline; the kernel emits the angle
//! as `data-angle` and COMPONENT_CHROME reads it with typed
//! `attr(data-angle type(<angle>))` — no inline style. Glyphs are Unicode
//! scalar values (React segments graphemes; no segmenter dep here).
//! `@keyframes cui-spinning-text` lives in COMPONENT_CHROME. Zero JS.

use crate::cronus_ui_kit::{esc, fmt_coord, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let phrase = raw_label(comp);
    let glyphs: Vec<char> = phrase.chars().collect();
    let n = glyphs.len().max(1) as f64;
    let orbit: String = glyphs
        .iter()
        .enumerate()
        .map(|(i, ch)| {
            let angle = fmt_coord(i as f64 / n * 360.0);
            let glyph = if *ch == ' ' {
                "\u{a0}".to_string()
            } else {
                esc(&ch.to_string())
            };
            format!("<span data-angle=\"{angle}deg\">{glyph}</span>")
        })
        .collect();
    format!(
        "<div data-slot=\"spinning-text\"><span>{label}</span><div aria-hidden=\"true\">{orbit}</div></div>"
    )
}

/// Unescaped label text (glyphs are escaped one at a time).
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
        .unwrap_or_default()
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
        assert!(!html.contains("@keyframes"));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setTimeout"));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("translateY"));
        assert!(!html.contains("rotate("));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_is_div_with_hidden_copy_and_glyph_orbit() {
        let html = render(&stub("spinning-text", "SPIN"));
        assert_eq!(
            html,
            "<div data-slot=\"spinning-text\"><span>SPIN</span><div aria-hidden=\"true\"><span data-angle=\"0deg\">S</span><span data-angle=\"90deg\">P</span><span data-angle=\"180deg\">I</span><span data-angle=\"270deg\">N</span></div></div>"
        );
        assert_eq!(html.matches("data-slot=").count(), 1);
        assert!(!html.contains("spinning-text-label"));
        assert!(!html.contains("spinning-text-orbit"));
        reject_fx(&html);
    }

    #[test]
    fn spaces_become_nbsp_and_angles_are_fractional() {
        let html = render(&stub("spinning-text", "A B"));
        assert!(html.contains("<span data-angle=\"0deg\">A</span>"));
        assert!(html.contains("<span data-angle=\"120deg\">\u{a0}</span>"));
        assert!(html.contains("<span data-angle=\"240deg\">B</span>"));
        let seven = render(&stub("spinning-text", "SPINNER"));
        assert!(seven.contains("<span data-angle=\"51.43deg\">P</span>"));
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("spinning-text", "<&"));
        assert_eq!(
            html,
            "<div data-slot=\"spinning-text\"><span>&lt;&amp;</span><div aria-hidden=\"true\"><span data-angle=\"0deg\">&lt;</span><span data-angle=\"180deg\">&amp;</span></div></div>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("spinning-text", "Cronus");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        reject_fx(&html);
        assert_eq!(crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html), None);
        assert_eq!(
            dedicated_fn_name("spinning-text"),
            Some("cronus_ui_spinning_text::render")
        );
        assert_eq!(
            renderer_kind("spinning-text"),
            RendererKind::Dedicated("cronus_ui_spinning_text::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("spinning-text", "Cronus"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"spinning-text\""));
        });
    }

    #[test]
    fn chrome_spinning_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"spinning-text\"] > span:first-child {\n  position: absolute;"));
        assert!(css.contains("[data-slot=\"spinning-text\"] > [aria-hidden=\"true\"] > span {"));
        assert!(css.contains("transform: rotate(attr(data-angle type(<angle>), 0deg)) translateY(-3rem);"));
        assert!(css.contains("font-size: 0.75rem; line-height: 1rem; font-weight: 500;"));
        assert!(css.contains("letter-spacing: 0.1em;"));
        assert!(!css.contains("[data-slot=\"spinning-text-orbit\"]"));
        assert!(css.contains("@keyframes cui-spinning-text"));
        assert!(css.contains("animation: cui-spinning-text"));
        assert!(css.contains("rotate(360deg)"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
        assert!(!css.contains("style="));
    }
}
