//! Dedicated GridPattern renderer. DOM matches React idle without SVG:
//! `<div data-slot="grid-pattern">` + `aria-hidden` field + relative content
//! wrapping the label. CSS repeating-linear-gradient lines live in
//! COMPONENT_CHROME. Zero JS, no inline style. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"grid-pattern\"><div data-slot=\"grid-pattern-field\" aria-hidden=\"true\"></div><div data-slot=\"grid-pattern-content\">{}</div></div>",
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
    }

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("grid-pattern", "Grid"));
        assert_eq!(
            html,
            "<div data-slot=\"grid-pattern\"><div data-slot=\"grid-pattern-field\" aria-hidden=\"true\"></div><div data-slot=\"grid-pattern-content\">Grid</div></div>"
        );
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"grid-pattern\""));
        assert!(html.contains("data-slot=\"grid-pattern-field\""));
        assert!(html.contains("data-slot=\"grid-pattern-content\""));
        assert!(html.contains("aria-hidden=\"true\""));
        assert!(html.contains(">Grid</div></div>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("grid-pattern", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"grid-pattern\"><div data-slot=\"grid-pattern-field\" aria-hidden=\"true\"></div><div data-slot=\"grid-pattern-content\">A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("grid-pattern", "Grid");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.contains("<span>"));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        assert!(!html.contains("<span"));
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("grid-pattern"),
            Some("cronus_ui_grid_pattern::render")
        );
        assert_eq!(
            renderer_kind("grid-pattern"),
            RendererKind::Dedicated("cronus_ui_grid_pattern::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("grid-pattern", "Grid"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"grid-pattern\""));
            assert!(html.contains("data-slot=\"grid-pattern-field\""));
            assert!(html.contains("data-slot=\"grid-pattern-content\""));
        });
    }

    #[test]
    fn chrome_grid_pattern_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"grid-pattern\"]"));
        assert!(css.contains("[data-slot=\"grid-pattern-field\"]"));
        assert!(css.contains("[data-slot=\"grid-pattern-content\"]"));
        assert!(css.contains("repeating-linear-gradient"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("<canvas"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
    }
}
