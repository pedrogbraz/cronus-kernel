//! Dedicated GridPattern renderer. DOM matches React idle without SVG:
//! `<div data-slot="grid-pattern">` + `aria-hidden` field + relative children
//! div wrapping the label. Only the root has a `data-slot`, like React (Wave 1t
//! geometry parity). CSS repeating-linear-gradient lines live in
//! COMPONENT_CHROME. Zero JS, no inline style. Not the catalog `fx()` title SURF box.
//!
//! Docs chrome from the `.cronus`: `card:true` (class `card`, the docs
//! `grid min-h-56 place-items-center rounded-2xl border bg-surface-raised`
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
        "<div data-slot=\"grid-pattern\"{class}><div aria-hidden=\"true\"></div><div>{content}</div></div>"
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
        assert!(!html.contains("<span"));
        assert!(!html.contains("<canvas"));
        assert!(!html.contains("<svg"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("requestAnimationFrame"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
        assert_eq!(html.matches("data-slot=").count(), 1);
    }

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("grid-pattern", "Grid"));
        assert_eq!(
            html,
            "<div data-slot=\"grid-pattern\"><div aria-hidden=\"true\"></div><div>Grid</div></div>"
        );
        assert!(!html.contains("grid-pattern-field"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("grid-pattern", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"grid-pattern\"><div aria-hidden=\"true\"></div><div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
        reject_fx(&html);
    }

    /// Docs example: card wrapper + `<p className="font-display text-2xl text-fg">Blueprint</p>`.
    #[test]
    fn card_and_heading_render_the_docs_wrapper() {
        let mut c = stub("grid-pattern", "Blueprint");
        c.props.insert("card".into(), "true".into());
        c.props.insert("heading".into(), "2xl".into());
        assert_eq!(
            render(&c),
            "<div data-slot=\"grid-pattern\" class=\"card\"><div aria-hidden=\"true\"></div><div><p class=\"h-2xl\">Blueprint</p></div></div>"
        );
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("grid-pattern", "Grid");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("grid-pattern"),
            Some("cronus_ui_grid_pattern::render")
        );
        assert_eq!(
            renderer_kind("grid-pattern"),
            RendererKind::Dedicated("cronus_ui_grid_pattern::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("grid-pattern", "Grid"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"grid-pattern\""));
        });
    }

    #[test]
    fn chrome_grid_pattern_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        let start = css.find("[data-slot=\"grid-pattern\"] {").unwrap();
        let block = &css[start..start + css[start..].find('}').unwrap()];
        assert!(block.contains("width: var(--cui-grid-pattern-w, 100%); min-height: 8rem;"));
        assert!(css.contains("[data-slot=\"grid-pattern\"] > [aria-hidden] {"));
        assert!(css.contains("[data-slot=\"grid-pattern\"] > div:last-child {"));
        assert!(!css.contains("[data-slot=\"grid-pattern-field\"]"));
        assert!(css.contains("repeating-linear-gradient"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("[data-slot=\"grid-pattern\"].card {\n  display: grid; place-items: center; min-height: 14rem;"));
        assert!(css.contains("[data-slot=\"grid-pattern\"] .h-2xl {"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
