//! Dedicated GradientBorder renderer. DOM matches React idle:
//! `<div data-slot="gradient-border">` wrapping an inner surface div (no
//! `data-slot`, like React — Wave 1t geometry parity) with the label.
//! Gradient ring, `rounded-2xl` and the inner `surface-raised` fill live in
//! COMPONENT_CHROME. Zero JS, no glow, no inline style. Not the catalog `fx()`
//! title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"gradient-border\"><div>{}</div></div>",
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
        assert_eq!(html.matches("data-slot=").count(), 1);
    }

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("gradient-border", "Glow"));
        assert_eq!(
            html,
            "<div data-slot=\"gradient-border\"><div>Glow</div></div>"
        );
        assert!(!html.contains("gradient-border-inner"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("gradient-border", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"gradient-border\"><div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("gradient-border", "Glow");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("gradient-border"),
            Some("cronus_ui_gradient_border::render")
        );
        assert_eq!(
            renderer_kind("gradient-border"),
            RendererKind::Dedicated("cronus_ui_gradient_border::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("gradient-border", "Glow"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"gradient-border\""));
        });
    }

    #[test]
    fn chrome_gradient_border_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"gradient-border\"] > div {"));
        assert!(!css.contains("[data-slot=\"gradient-border-inner\"]"));
        assert!(css.contains("linear-gradient"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(css.contains("var(--cronus-accent)"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }

    fn chrome_block(selector: &str) -> String {
        let css = crate::cronus_ui::component_chrome_css();
        let start = css.find(selector).unwrap();
        css[start..start + css[start..].find('}').unwrap()].to_string()
    }

    /// Geometry: React `rounded-2xl bg-gradient-primary p-px` + fixture
    /// `className="w-72"` → 288px block with a 1px gradient ring, r22.
    #[test]
    fn chrome_mirrors_fixture_box_and_2xl_radius() {
        let block = chrome_block("[data-slot=\"gradient-border\"] {");
        assert!(block.contains("display: block;"));
        assert!(!block.contains("inline-block"));
        assert!(
            block.contains("width: var(--cui-gradient-border-w, 100%); box-sizing: border-box;")
        );
        assert!(block.contains("border-radius: calc(var(--cronus-radius, 14px) + 8px);"));
    }

    /// Regression: the ring is `p-px` (1px), not the 24px fixture padding that
    /// used to override it. Content padding (`innerClassName="p-6"`) lives on
    /// the inner surface.
    #[test]
    fn ring_is_1px_and_padding_lives_on_inner_surface() {
        let outer = chrome_block("[data-slot=\"gradient-border\"] {");
        assert!(outer.contains("padding: 1px;"));
        assert!(!outer.contains("padding: 1.5rem"));
        let inner = chrome_block("[data-slot=\"gradient-border\"] > div {");
        assert!(inner.contains("padding: 1.5rem;"));
        assert!(inner.contains("border-radius: inherit;"));
        assert!(inner.contains("background: var(--cronus-surface-raised);"));
    }
}
