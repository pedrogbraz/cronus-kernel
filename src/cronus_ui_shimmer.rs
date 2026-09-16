//! Dedicated Shimmer renderer. DOM mirrors React:
//! `<div data-slot="shimmer" aria-hidden="true">` + an empty sweep `<div>`
//! (React: `absolute inset-0 animate-shimmer` gradient). Fixture default
//! `h-8 w-48`, `rounded-md bg-surface-overlay` and the sweep
//! (`@keyframes cui-shimmer`) live in COMPONENT_CHROME. Not the catalog
//! `fx()` title SURF box.

use crate::parser::ComponentNode;

pub fn render(_comp: &ComponentNode) -> String {
    "<div data-slot=\"shimmer\" aria-hidden=\"true\"><div></div></div>".into()
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
    fn root_is_block_with_sweep_layer_not_fx_title_box() {
        let html = render(&stub("shimmer", "Demo"));
        assert_eq!(
            html,
            "<div data-slot=\"shimmer\" aria-hidden=\"true\"><div></div></div>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("shimmer", "Demo"));
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
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
        assert!(css.contains("[data-slot=\"shimmer\"] {\n  display: block; position: relative; overflow: hidden;\n  height: 2rem; width: var(--cui-shimmer-w, 100%);\n  border-radius: var(--cronus-radius-md);"));
        assert!(css.contains("[data-slot=\"shimmer\"] > div {\n  position: absolute; inset: 0;"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(css.contains("@keyframes cui-shimmer"));
        assert!(css.contains("animation: cui-shimmer"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
