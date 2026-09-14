//! Dedicated AuroraBackground renderer. DOM matches React idle:
//! `<div data-slot="aurora-background">` + `aria-hidden` layer with three
//! blob divs + relative children wrapper (label). Only the root carries a
//! `data-slot` — React's layer and blobs have none (Wave 1t geometry parity).
//! CSS blobs/keyframes live in COMPONENT_CHROME (structural selectors).
//! Zero JS, no inline style. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"aurora-background\"><div aria-hidden=\"true\"><div></div><div></div><div></div></div><div>{}</div></div>",
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
        // React has exactly one data-slot here.
        assert_eq!(html.matches("data-slot=").count(), 1);
    }

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("aurora-background", "Aurora"));
        assert_eq!(
            html,
            "<div data-slot=\"aurora-background\"><div aria-hidden=\"true\"><div></div><div></div><div></div></div><div>Aurora</div></div>"
        );
        assert!(!html.contains("aurora-blob"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("aurora-background", "A <B> & \"C\""));
        assert!(html.ends_with("<div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"));
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("aurora-background", "Aurora");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("aurora-background"),
            Some("cronus_ui_aurora_background::render")
        );
        assert_eq!(
            renderer_kind("aurora-background"),
            RendererKind::Dedicated("cronus_ui_aurora_background::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("aurora-background", "Aurora"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"aurora-background\""));
        });
    }

    #[test]
    fn chrome_aurora_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"aurora-background\"] > [aria-hidden] > div {"));
        assert!(!css.contains("[data-slot=\"aurora-blob\"]"));
        assert!(css.contains("@keyframes cui-aurora"));
        assert!(css.contains("animation: cui-aurora"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(css.contains("var(--cronus-accent)"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }

    /// Wave 1t geometry: fixture `w-72 min-h-32` → 288×128.
    #[test]
    fn chrome_mirrors_fixture_box() {
        let css = crate::cronus_ui::component_chrome_css();
        let start = css.find("[data-slot=\"aurora-background\"] {").unwrap();
        let block = &css[start..start + css[start..].find('}').unwrap()];
        assert!(block.contains("width: 18rem; min-height: 8rem;"));
        let layer = css.find("[data-slot=\"aurora-background\"] > [aria-hidden] {").unwrap();
        let layer = &css[layer..layer + css[layer..].find('}').unwrap()];
        // React `-z-10`: the blobs sit behind the ground, not over it.
        assert!(layer.contains("z-index: -10;"));
    }
}
