//! Dedicated TextShimmer renderer. DOM matches React:
//! `<p data-slot="text-shimmer">` with label text. CSS gradient animation
//! in COMPONENT_CHROME. Not the catalog `fx()` title SURF box.
//! React sets `--spread: {text.length * 2}px` inline; the kernel emits the
//! same number as `data-spread` and the chrome reads it with typed `attr()`.

use crate::cronus_ui_kit::{item, label_of};
use crate::parser::ComponentNode;

/// React `TextShimmer` default `spread` multiplier (px per UTF-16 unit).
const SPREAD_PER_CHAR: usize = 2;

pub fn render(comp: &ComponentNode) -> String {
    let spread = raw_label(comp).encode_utf16().count() * SPREAD_PER_CHAR;
    format!(
        "<p data-slot=\"text-shimmer\" data-spread=\"{spread}\">{}</p>",
        label_of(comp)
    )
}

/// Unescaped text `label_of` picks (React measures `children.length` raw).
fn raw_label(comp: &ComponentNode) -> &str {
    for kind in ["label", "title", "text", "value"] {
        if let Some(t) = item(comp, kind) {
            if !t.is_empty() {
                return t;
            }
        }
    }
    comp.items
        .iter()
        .map(|i| i.text.as_str())
        .find(|t| !t.is_empty())
        .unwrap_or(&comp.name)
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
        assert!(!html.contains("<div"));
        assert!(!html.contains("<span"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
    }

    /// React `TextShimmer` renders `<p data-slot="text-shimmer">`; audit e2e
    /// expects tag P.
    #[test]
    fn root_is_p_with_label_like_react() {
        let html = render(&stub("text-shimmer", "Thinking"));
        assert_eq!(
            html,
            "<p data-slot=\"text-shimmer\" data-spread=\"16\">Thinking</p>"
        );
        reject_fx(&html);
    }

    /// fa: React `--spread` = `children.length * 2`px (UTF-16 units, raw text),
    /// and the gradient stops are `calc(50% ∓ spread)`, not 40%/60%.
    #[test]
    fn spread_matches_react_children_length() {
        assert!(render(&stub("text-shimmer", "Loading")).contains("data-spread=\"14\""));
        // "é😀" = 1 + 2 UTF-16 units; `&` counts once even though it is escaped.
        assert!(render(&stub("text-shimmer", "é😀&")).contains("data-spread=\"8\""));
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("transparent calc(50% - attr(data-spread px, 0px)), var(--cronus-surface-base), transparent calc(50% + attr(data-spread px, 0px))),\n    linear-gradient(var(--cronus-fg-tertiary), var(--cronus-fg-tertiary));"));
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("text-shimmer", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<p data-slot=\"text-shimmer\" data-spread=\"22\">A &lt;B&gt; &amp; &quot;C&quot;</p>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("text-shimmer", "Thinking"));
        reject_fx(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("text-shimmer", "Thinking"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"text-shimmer\""));
        });
    }

    #[test]
    fn chrome_text_shimmer_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        // Wave 1s: React `<p>` inherits line-height 24px → root 57.5×24.
        assert!(css.contains(
            "[data-slot=\"text-shimmer\"] {\n  display: inline-block;\n  margin: 0;\n  position: relative;\n  line-height: 1.5;\n  color: transparent;"
        ));
        assert!(css.contains("background-size: 250% 100%, auto;"));
        assert!(css.contains("background-clip: text"));
        assert!(css.contains("@keyframes cui-text-shimmer"));
        assert!(css.contains("animation: cui-text-shimmer"));
        assert!(css.contains("var(--cronus-surface-base)"));
        assert!(css.contains("var(--cronus-fg-tertiary"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
