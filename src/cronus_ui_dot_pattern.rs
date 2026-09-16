//! Dedicated DotPattern renderer. DOM matches React idle without SVG:
//! `<div data-slot="dot-pattern">` + `aria-hidden` field + relative children
//! div wrapping the label. Only the root has a `data-slot`, like React (Wave 1t
//! geometry parity). CSS radial-gradient dots (16px cell, r 1px) live in
//! COMPONENT_CHROME. Zero JS, no inline style. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"dot-pattern\"><div aria-hidden=\"true\"></div><div>{}</div></div>",
        label_of(comp)
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
        let html = render(&stub("dot-pattern", "Dots"));
        assert_eq!(
            html,
            "<div data-slot=\"dot-pattern\"><div aria-hidden=\"true\"></div><div>Dots</div></div>"
        );
        assert!(!html.contains("dot-pattern-field"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("dot-pattern", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"dot-pattern\"><div aria-hidden=\"true\"></div><div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("dot-pattern", "Dots");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("dot-pattern"),
            Some("cronus_ui_dot_pattern::render")
        );
        assert_eq!(
            renderer_kind("dot-pattern"),
            RendererKind::Dedicated("cronus_ui_dot_pattern::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("dot-pattern", "Dots"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"dot-pattern\""));
        });
    }

    #[test]
    fn chrome_dot_pattern_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        let start = css.find("[data-slot=\"dot-pattern\"] {").unwrap();
        let block = &css[start..start + css[start..].find('}').unwrap()];
        assert!(block.contains("width: var(--cui-dot-pattern-w, 100%); min-height: 8rem;"));
        assert!(css.contains("[data-slot=\"dot-pattern\"] > [aria-hidden] {"));
        assert!(css.contains("[data-slot=\"dot-pattern\"] > div:last-child {"));
        assert!(!css.contains("[data-slot=\"dot-pattern-field\"]"));
        assert!(css.contains(
            "radial-gradient(circle at 8px 8px, var(--cronus-fg) 1px, transparent 1.25px);"
        ));
        assert!(css.contains("background-size: 16px 16px;"));
        assert!(!css.contains("repeating-radial-gradient(circle at 8px 8px"));
        assert!(css.contains("var(--cronus-fg)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
