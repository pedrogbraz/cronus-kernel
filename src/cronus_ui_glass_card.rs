//! Dedicated GlassCard renderer. DOM matches React:
//! `<div data-slot="glass-card">` + `aria-hidden` top highlight line +
//! relative children div wrapping the label. Frost (`backdrop-filter`),
//! `rounded-2xl`, `border-soft` hairline and `p-6` live in COMPONENT_CHROME.
//! Not the catalog `display()` SURF `<section>`.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"glass-card\"><div aria-hidden=\"true\"></div><div>{}</div></div>",
        label_of(comp)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    const DISPLAY_SURF: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem;";

    fn reject_display(html: &str) {
        assert!(!html.contains(DISPLAY_SURF));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<section"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
        assert_eq!(html.matches("data-slot=").count(), 1);
    }

    #[test]
    fn root_wraps_highlight_and_label_not_display_surf_section() {
        let html = render(&stub("glass-card", "Frost"));
        assert_eq!(
            html,
            "<div data-slot=\"glass-card\"><div aria-hidden=\"true\"></div><div>Frost</div></div>"
        );
        reject_display(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("glass-card", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"glass-card\"><div aria-hidden=\"true\"></div><div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
        reject_display(&html);
    }

    #[test]
    fn skips_display_surf_section() {
        let html = render(&stub("glass-card", "Frost"));
        reject_display(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("glass-card", "Frost"));
            reject_display(&html);
            assert!(html.contains("data-slot=\"glass-card\""));
        });
    }

    #[test]
    fn chrome_glass_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("backdrop-filter: blur(24px)"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(css.contains("var(--cronus-shadow-lg"));
        assert!(css.contains("[data-slot=\"glass-card\"] > [aria-hidden] {"));
        assert!(css.contains("[data-slot=\"glass-card\"] > div:last-child {"));
        assert!(!css.contains("[data-slot=\"glass-card\"]::before"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(DISPLAY_SURF));
    }

    /// Wave 1t geometry: React `relative rounded-2xl border border-border-soft p-6`
    /// + fixture `w-72` → 288×74, radius 22px, 1px border.
    #[test]
    fn chrome_matches_react_box() {
        let css = crate::cronus_ui::component_chrome_css();
        let start = css.find("[data-slot=\"glass-card\"] {").unwrap();
        let block = &css[start..start + css[start..].find('}').unwrap()];
        assert!(block.contains("position: relative;"));
        assert!(block.contains("width: 18rem; padding: 1.5rem;"));
        assert!(block.contains("border-radius: calc(var(--cronus-radius, 14px) + 8px);"));
        assert!(
            block.contains("border: 1px solid var(--cronus-border-soft, var(--cronus-border));")
        );
    }
}
