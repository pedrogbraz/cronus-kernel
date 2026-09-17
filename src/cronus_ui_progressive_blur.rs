//! Dedicated ProgressiveBlur renderer. React `ProgressiveBlur` is an
//! aria-hidden `absolute` band of three unslotted backdrop-blur layers; the
//! audit fixture wraps it in `relative min-h-32 w-72` after the text.
//! Kernel mirrors that: a `data-progressive-blur-host` wrapper (the fixture
//! wrapper — deliberately not a `data-slot`, React has none) holds the label
//! as bare text, then `<div data-slot="progressive-blur">` with
//! three plain `<div>` layers styled by `> div:nth-child(n)` in
//! COMPONENT_CHROME. No inline style, zero JS. Not the catalog `fx()` box.
//!
//! Docs example ("Edge fade"): `text` items become the scroll region
//! (`h-full overflow-y-auto p-4 pb-16` with `text-sm text-fg-secondary`
//! paragraphs) and `card:true` is the docs host
//! (`h-48 overflow-hidden rounded-2xl border bg-surface-raised`, class
//! `card`). `side:top` moves the band to the top edge (class `side-top`,
//! React's CVA `side`).

use crate::cronus_ui_kit::{choice, esc, flag, label_of};
use crate::parser::ComponentNode;

const LAYER: &str = "<div></div>";

pub fn render(comp: &ComponentNode) -> String {
    let paragraphs: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "text" && !i.text.is_empty())
        .map(|i| format!("<p>{}</p>", esc(&i.text)))
        .collect();
    let body = if paragraphs.is_empty() {
        label_of(comp)
    } else {
        format!("<div>{}</div>", paragraphs.concat())
    };
    let host_class = if flag(comp, "card") {
        " class=\"card\""
    } else {
        ""
    };
    let side_class = match choice(comp, "side", &["top", "bottom"]) {
        Some("top") => " class=\"side-top\"",
        _ => "",
    };
    format!(
        "<div data-progressive-blur-host=\"true\"{host_class}>{body}<div data-slot=\"progressive-blur\"{side_class} aria-hidden=\"true\">{LAYER}{LAYER}{LAYER}</div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
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
        assert!(!html.contains("@keyframes"));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<span"));
        assert!(!html.contains("<canvas"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("requestAnimationFrame"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    /// React fixture DOM: `<div class="relative min-h-32 w-72">Blur<div
    /// data-slot="progressive-blur" aria-hidden="true">` + 3 unslotted layers.
    #[test]
    fn text_precedes_aria_hidden_band_like_react_fixture() {
        let html = render(&stub("progressive-blur", "Blur"));
        assert_eq!(
            html,
            "<div data-progressive-blur-host=\"true\">Blur<div data-slot=\"progressive-blur\" aria-hidden=\"true\"><div></div><div></div><div></div></div></div>"
        );
        // Only one data-slot, like React: the host is the fixture wrapper.
        assert_eq!(html.matches("data-slot=").count(), 1);
        assert!(!html.contains("progressive-blur-host\""));
        assert!(!html.contains("progressive-blur-layer"));
        assert!(!html.contains("progressive-blur-content"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("progressive-blur", "A <B> & \"C\""));
        assert!(html.starts_with(
            "<div data-progressive-blur-host=\"true\">A &lt;B&gt; &amp; &quot;C&quot;<div"
        ));
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    /// Docs example: a card host with a scroll region of paragraphs under the band.
    #[test]
    fn text_items_become_the_scroll_region_and_card_is_the_docs_host() {
        let mut c = stub("progressive-blur", "Edge");
        c.items.push(text("Scroll under the blur."));
        c.items.push(text("Last <line>."));
        c.props.insert("card".into(), "true".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-progressive-blur-host=\"true\" class=\"card\"><div><p>Scroll under the blur.</p><p>Last &lt;line&gt;.</p></div><div data-slot=\"progressive-blur\" aria-hidden=\"true\"><div></div><div></div><div></div></div></div>"
        );
        assert!(!html.contains("Edge"));
        reject_fx(&html);
    }

    #[test]
    fn side_top_is_a_class_on_the_band() {
        let mut c = stub("progressive-blur", "Blur");
        c.props.insert("side".into(), "top".into());
        let html = render(&c);
        assert!(html.contains(
            "<div data-slot=\"progressive-blur\" class=\"side-top\" aria-hidden=\"true\">"
        ));
        c.props.insert("side".into(), "left".into());
        assert!(!render(&c).contains("class=\"side-"));
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("progressive-blur", "Blur");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        assert!(!html.contains("<span"));
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("progressive-blur"),
            Some("cronus_ui_progressive_blur::render")
        );
        assert_eq!(
            renderer_kind("progressive-blur"),
            RendererKind::Dedicated("cronus_ui_progressive_blur::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("progressive-blur", "Blur"));
            reject_fx(&html);
            assert!(html.contains("data-progressive-blur-host=\"true\""));
            assert!(html.contains("data-slot=\"progressive-blur\" aria-hidden=\"true\""));
            assert_eq!(html.matches("<div></div>").count(), 3);
        });
    }

    /// Fixture wrapper is `min-h-32 w-72` (288x128) with 24px line-height;
    /// band is 6rem at the bottom with 1/4/12px layers.
    #[test]
    fn chrome_matches_react_fixture_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-progressive-blur-host] {\n  position: relative;\n  width: var(--cui-progressive-blur-w, 100%);\n  min-height: 8rem;\n  line-height: 1.5;"
        ));
        assert!(css.contains("height: 6rem; z-index: 10; pointer-events: none;"));
        assert!(css.contains("[data-slot=\"progressive-blur\"] > div {"));
        for (n, px) in [(1, 1), (2, 4), (3, 12)] {
            let rule = format!(
                "[data-slot=\"progressive-blur\"] > div:nth-child({n}) {{\n  backdrop-filter: blur({px}px);"
            );
            assert!(css.contains(&rule), "{rule}");
        }
        assert!(css.contains("mask-image: linear-gradient(to top, black 30%, transparent 70%)"));
        assert!(css.contains("[data-slot=\"progressive-blur\"].side-top { top: 0; bottom: auto; }"));
        assert!(css.contains("mask-image: linear-gradient(to bottom, black 10%, transparent 45%)"));
        assert!(css.contains("[data-slot=\"progressive-blur\"] { display: none; }"));
        // Docs host: h-48 overflow-hidden rounded-2xl border bg-surface-raised + scroll region.
        assert!(
            css.contains("[data-progressive-blur-host].card {\n  height: 12rem; overflow: hidden;")
        );
        assert!(css.contains("[data-progressive-blur-host] > div:not([data-slot]) {\n  height: 100%; overflow-y: auto; padding: 1rem 1rem 4rem;"));
        assert!(css.contains(
            "[data-progressive-blur-host] > div:not([data-slot]) > p + p { margin-top: 1rem; }"
        ));
        assert!(!css.contains("progressive-blur-layer"));
        assert!(!css.contains("progressive-blur-content"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
    }
}
