//! Dedicated Shimmer renderer. DOM matches React:
//! `<div data-slot="shimmer" aria-hidden="true">` (empty skeleton-like block).
//! Sweep lives in COMPONENT_CHROME (`::after` + `@keyframes cui-shimmer`).
//! Not the catalog `fx()` title SURF box.

use crate::parser::ComponentNode;

pub fn render(_comp: &ComponentNode) -> String {
    "<div data-slot=\"shimmer\" aria-hidden=\"true\"></div>".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<span"));
        assert!(!html.contains("Demo"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_is_empty_div_not_fx_title_box() {
        let html = render(&stub("shimmer", "Demo"));
        assert_eq!(
            html,
            "<div data-slot=\"shimmer\" aria-hidden=\"true\"></div>"
        );
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"shimmer\""));
        assert!(html.contains("aria-hidden=\"true\""));
        assert!(!html.contains("height:0.9rem"));
        assert!(!html.contains("width:8rem"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("shimmer", "Demo"));
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.contains("<span>"));
        assert_ne!(html, fx);
        assert!(!html.contains("<span"));
        reject_fx(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("shimmer", "Demo"));
            reject_fx(&html);
        });
    }

    #[test]
    fn chrome_shimmer_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"shimmer\"]"));
        assert!(css.contains("width: 8rem"));
        assert!(css.contains("height: 0.9rem"));
        assert!(css.contains("border-radius: var(--cronus-radius-md)"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(css.contains("@keyframes cui-shimmer"));
        assert!(css.contains("animation: cui-shimmer"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
