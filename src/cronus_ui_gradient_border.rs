//! Dedicated GradientBorder renderer. DOM matches React idle:
//! `<div data-slot="gradient-border">` wrapping an inner surface div (no
//! `data-slot`, like React — Wave 1t geometry parity) with the content.
//! Gradient ring, `rounded-2xl` and the inner `surface-raised` fill live in
//! the family CSS. `glow:true` adds React's `shadow-glow` as a class on the
//! root. Zero JS, no inline style. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_glass_card::feature_content;
use crate::cronus_ui_kit::flag;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let glow = if flag(comp, "glow") {
        " class=\"glow\""
    } else {
        ""
    };
    format!(
        "<div data-slot=\"gradient-border\"{glow}><div>{}</div></div>",
        feature_content(comp)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_glass_card::tests::item;
    use crate::cronus_ui_kit::stub;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";
    const CSS: &str = include_str!("cronus_ui_css/gradient-border.css");

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("<style"));
        assert!(!html.contains("@keyframes"));
        assert!(!html.contains("SURF"));
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
        assert!(!html.contains("<span"));
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

    /// Docs "With glow" / "Flat": `glow` is React's `shadow-glow` class; the
    /// inner surface stacks heading, copy and the `$29 / month` price row.
    #[test]
    fn glow_flag_and_plan_content() {
        let mut c = stub("gradient-border", "Pro plan");
        c.props.insert("glow".into(), "true".into());
        c.items.push(item("title", "Pro plan", &[]));
        c.items.push(item("text", "The hairline ring.", &[]));
        c.items.push(item("value", "$29", &[("meta", "/ month")]));
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"gradient-border\" class=\"glow\"><div><h3>Pro plan</h3><p>The hairline ring.</p><div class=\"price\"><span>$29</span><span>/ month</span></div></div></div>"
        );
        reject_fx(&html);
        c.props.insert("glow".into(), "false".into());
        assert!(render(&c).starts_with("<div data-slot=\"gradient-border\"><div>"));
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
        assert!(CSS.contains("[data-slot=\"gradient-border\"] > div {"));
        assert!(!CSS.contains("[data-slot=\"gradient-border-inner\"]"));
        assert!(CSS.contains("linear-gradient"));
        assert!(CSS.contains("var(--cronus-primary)"));
        assert!(CSS.contains("var(--cronus-accent)"));
        assert!(CSS.contains("var(--cronus-surface-raised)"));
        assert!(CSS.contains(
            "[data-slot=\"gradient-border\"].glow { box-shadow: var(--cronus-shadow-glow"
        ));
        assert!(!CSS.contains("zinc-"));
        assert!(!CSS.contains(FX_BOX));
    }

    fn chrome_block(selector: &str) -> String {
        let start = CSS.find(selector).unwrap();
        CSS[start..start + CSS[start..].find('}').unwrap()].to_string()
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

    /// The ring is `p-px` (1px); the docs `innerClassName="flex flex-col gap-3
    /// bg-surface-raised p-5"` padding and gap live on the inner surface.
    #[test]
    fn ring_is_1px_and_padding_lives_on_inner_surface() {
        let outer = chrome_block("[data-slot=\"gradient-border\"] {");
        assert!(outer.contains("padding: 1px;"));
        assert!(!outer.contains("padding: 1.5rem"));
        let inner = chrome_block("[data-slot=\"gradient-border\"] > div {");
        assert!(inner.contains("display: flex; flex-direction: column; gap: 0.75rem;"));
        assert!(inner.contains("padding: 1.25rem;"));
        assert!(inner.contains("border-radius: inherit;"));
        assert!(inner.contains("background: var(--cronus-surface-raised);"));
    }
}
