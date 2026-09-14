//! Dedicated SpotlightCard renderer. DOM mirrors React:
//! `<div data-slot="spotlight-card">` + aria-hidden spotlight `<div>` + a
//! relative content `<div>` wrapping the label. The radial spotlight sits at
//! `--spot-x` / `--spot-y` (default 50% 50%); React moves it with pointer JS,
//! the kernel fades it in on `:hover` only. `rounded-2xl`, fixture `w-72`
//! and colors live in COMPONENT_CHROME. Not catalog `display()` SURF
//! `<section>`. Not interact `card()`.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"spotlight-card\"><div aria-hidden=\"true\"></div><div>{}</div></div>",
        label_of(comp)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const DISPLAY_BOX: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem";

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

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

    #[test]
    fn extra_text_does_not_add_nodes() {
        let mut c = stub("spotlight-card", "Hover me");
        c.items.push(extra("text", "More"));
        let html = render(&c);
        assert!(!html.contains("More"));
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
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"spotlight-card\"] {\n  position: relative; overflow: hidden;\n  width: var(--cui-spotlight-card-w, 100%); box-sizing: border-box; padding: 1.5rem; color: var(--cronus-fg);\n  border-radius: calc(var(--cronus-radius, 14px) + 8px); box-shadow: none;\n}"));
        assert!(css.contains("[data-slot=\"spotlight-card\"] > [aria-hidden=\"true\"] {"));
        assert!(css.contains(
            "[data-slot=\"spotlight-card\"]:hover > [aria-hidden=\"true\"] { opacity: 1; }"
        ));
        assert!(css.contains(
            "[data-slot=\"spotlight-card\"] > div:last-child {\n  position: relative;\n}"
        ));
        assert!(!css.contains("[data-slot=\"spotlight-card\"]::after"));
        assert!(css.contains("--spot-x"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(DISPLAY_BOX));
    }
}
