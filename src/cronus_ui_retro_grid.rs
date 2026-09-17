//! Dedicated RetroGrid renderer. DOM mirrors React without the injected
//! `<style>`: `<div data-slot="retro-grid">` + aria-hidden mask `<div>` >
//! tilted floor `<div>` > scrolling grid `<div>`, then a relative content
//! `<div>` wrapping the label. Only the root carries a `data-slot`. Fixture
//! `w-72 min-h-32` is mirrored in COMPONENT_CHROME. Perspective floor +
//! `@keyframes cui-retro-grid` live in COMPONENT_CHROME. Zero JS, no inline
//! style. Not the catalog `fx()` title SURF box.
//!
//! Docs chrome from the `.cronus`: `card:true` (class `card`, the docs
//! `grid min-h-64 place-items-center rounded-2xl border bg-surface-raised`
//! wrapper) and `heading:2xl` (`<p class="h-2xl">` = `font-display text-2xl text-fg`).

use crate::cronus_ui_dot_pattern::HEADINGS;
use crate::cronus_ui_kit::{choice, flag, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let class = if flag(comp, "card") {
        " class=\"card\""
    } else {
        ""
    };
    let text = label_of(comp);
    let content = match choice(comp, "heading", HEADINGS) {
        Some(h) => format!("<p class=\"h-{h}\">{text}</p>"),
        None => text,
    };
    format!(
        "<div data-slot=\"retro-grid\"{class}><div aria-hidden=\"true\"><div><div></div></div></div><div>{content}</div></div>"
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

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("retro-grid", "Grid"));
        assert_eq!(
            html,
            "<div data-slot=\"retro-grid\"><div aria-hidden=\"true\"><div><div></div></div></div><div>Grid</div></div>"
        );
        assert_eq!(html.matches("data-slot=").count(), 1);
        assert!(!html.contains("retro-grid-field"));
        assert!(!html.contains("retro-grid-content"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("retro-grid", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"retro-grid\"><div aria-hidden=\"true\"><div><div></div></div></div><div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    /// Docs example: card wrapper (min-h-64) + `<p className="font-display text-2xl text-fg">Horizon</p>`.
    #[test]
    fn card_and_heading_render_the_docs_wrapper() {
        let mut c = stub("retro-grid", "Horizon");
        c.props.insert("card".into(), "true".into());
        c.props.insert("heading".into(), "2xl".into());
        assert_eq!(
            render(&c),
            "<div data-slot=\"retro-grid\" class=\"card\"><div aria-hidden=\"true\"><div><div></div></div></div><div><p class=\"h-2xl\">Horizon</p></div></div>"
        );
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("retro-grid", "Grid");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
        assert_eq!(
            dedicated_fn_name("retro-grid"),
            Some("cronus_ui_retro_grid::render")
        );
        assert_eq!(
            renderer_kind("retro-grid"),
            RendererKind::Dedicated("cronus_ui_retro_grid::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("retro-grid", "Grid"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"retro-grid\""));
        });
    }

    #[test]
    fn chrome_retro_grid_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"retro-grid\"] {\n  position: relative; overflow: hidden;\n  width: var(--cui-retro-grid-w, 100%); min-height: 8rem;"));
        assert!(css.contains("[data-slot=\"retro-grid\"] > [aria-hidden=\"true\"] > div > div {"));
        assert!(
            css.contains("[data-slot=\"retro-grid\"] > div:last-child {\n  position: relative;\n}")
        );
        assert!(!css.contains("[data-slot=\"retro-grid-field\"]"));
        assert!(css.contains("perspective: 240px"));
        assert!(css.contains("rotateX(60deg)"));
        assert!(css.contains("@keyframes cui-retro-grid"));
        assert!(css.contains("animation: cui-retro-grid 8s linear infinite"));
        assert!(css.contains("translateY(48px)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(css.contains("[data-slot=\"retro-grid\"].card {\n  display: grid; place-items: center; min-height: 16rem;"));
        assert!(css.contains("[data-slot=\"retro-grid\"] .h-2xl {"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
    }
}
