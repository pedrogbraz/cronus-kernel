//! Dedicated SpotlightCard renderer. DOM mirrors React:
//! `<div data-slot="spotlight-card">` + aria-hidden spotlight `<div>` + a
//! relative content `<div>` wrapping the feature content (icon chip, heading,
//! copy — see `cronus_ui_glass_card::feature_content`). The radial spotlight
//! sits at `--spot-x` / `--spot-y` (default 50% 50%); React moves it with
//! pointer JS, the kernel fades it in on `:hover` only. `rounded-2xl`,
//! fixture `w-72` and colors live in the family CSS. Not catalog `display()`
//! SURF `<section>`. Not interact `card()`.

use crate::cronus_ui_glass_card::feature_content;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"spotlight-card\"><div aria-hidden=\"true\"></div><div>{}</div></div>",
        feature_content(comp)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_glass_card::tests::item;
    use crate::cronus_ui_kit::stub;

    const DISPLAY_BOX: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem";
    const CSS: &str = include_str!("cronus_ui_css/spotlight-card.css");

    fn reject_display(html: &str) {
        assert!(!html.contains(DISPLAY_BOX));
        assert!(!html.contains("<section"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("font-weight:500"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("display("));
    }

    #[test]
    fn root_has_spotlight_layer_and_content_div() {
        let html = render(&stub("spotlight-card", "Hover me"));
        assert_eq!(
            html,
            "<div data-slot=\"spotlight-card\"><div aria-hidden=\"true\"></div><div>Hover me</div></div>"
        );
        assert_eq!(html.matches("data-slot=").count(), 1);
        reject_display(&html);
    }

    /// Docs "Hover spotlight": a `text-primary` icon chip, heading and copy
    /// inside the relative content div.
    #[test]
    fn docs_content_with_primary_glyph() {
        let mut c = stub("spotlight-card", "Accessible core");
        c.props.insert("icon".into(), "gauge".into());
        c.props.insert("icon-tone".into(), "primary".into());
        c.items.push(item("title", "Accessible core", &[]));
        c.items.push(item("text", "Radix primitives.", &[]));
        let html = render(&c);
        assert!(html.starts_with(
            "<div data-slot=\"spotlight-card\"><div aria-hidden=\"true\"></div><div><span class=\"glyph primary\"><svg"
        ));
        assert!(html.contains("data-icon=\"gauge\""));
        assert!(
            html.ends_with("</span><h3>Accessible core</h3><p>Radix primitives.</p></div></div>")
        );
        assert_eq!(html.matches("Accessible core").count(), 1);
        reject_display(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("spotlight-card", "A <B> & \"C\""));
        assert!(html.ends_with("<div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"));
        reject_display(&html);
    }

    #[test]
    fn skips_display_surf_and_interact_card() {
        let c = stub("spotlight-card", "Hover me");
        let html = render(&c);
        reject_display(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("spotlight-card", "Hover me"));
            reject_display(&html);
            assert!(html.contains("data-slot=\"spotlight-card\""));
        });
    }

    #[test]
    fn chrome_spotlight_via_css() {
        assert!(CSS.contains("[data-slot=\"spotlight-card\"] {\n  position: relative; overflow: hidden;\n  width: var(--cui-spotlight-card-w, 100%); box-sizing: border-box; padding: 1.5rem; color: var(--cronus-fg);\n  border-radius: calc(var(--cronus-radius, 14px) + 8px); box-shadow: none;\n}"));
        assert!(CSS.contains("[data-slot=\"spotlight-card\"] > [aria-hidden=\"true\"] {"));
        assert!(CSS.contains(
            "[data-slot=\"spotlight-card\"]:hover > [aria-hidden=\"true\"] { opacity: 1; }"
        ));
        assert!(CSS.contains(
            "[data-slot=\"spotlight-card\"] > div:last-child {\n  position: relative;\n}"
        ));
        assert!(!CSS.contains("[data-slot=\"spotlight-card\"]::after"));
        assert!(CSS.contains("--spot-x"));
        assert!(CSS.contains("var(--cronus-primary)"));
        assert!(CSS.contains(
            "[data-slot=\"spotlight-card\"] .glyph.primary { color: var(--cronus-primary); }"
        ));
        assert!(!CSS.contains("zinc-"));
        assert!(!CSS.contains(DISPLAY_BOX));
    }
}
