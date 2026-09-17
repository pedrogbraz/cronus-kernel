//! Dedicated AuroraBackground renderer. DOM matches React idle:
//! `<div data-slot="aurora-background">` + `aria-hidden` layer with three
//! blob divs + relative children wrapper. Only the root carries a
//! `data-slot` — React's layer and blobs have none (Wave 1t geometry parity).
//! CSS blobs/keyframes (`aurora-drift` 18s, delays -6s / -12s) live in the
//! family CSS (structural selectors). Zero JS, no inline style.
//!
//! With feature content (`title` / `text` / `action` items, see
//! `cronus_ui_glass_card::feature_content`) the root takes the docs hero
//! recipe (`hero` class: `min-h-48 rounded-2xl` centered, `max-w-lg` stack).
//! Not the catalog `fx()` title SURF box.

use crate::cronus_ui_glass_card::feature_content;
use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let content = feature_content(comp);
    let hero = if content != label_of(comp) {
        " class=\"hero\""
    } else {
        ""
    };
    format!(
        "<div data-slot=\"aurora-background\"{hero}><div aria-hidden=\"true\"><div></div><div></div><div></div></div><div>{content}</div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_glass_card::tests::item;
    use crate::cronus_ui_kit::stub;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";
    const CSS: &str = include_str!("cronus_ui_css/aurora-background.css");

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
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
    }

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("aurora-background", "Aurora"));
        assert_eq!(
            html,
            "<div data-slot=\"aurora-background\"><div aria-hidden=\"true\"><div></div><div></div><div></div></div><div>Aurora</div></div>"
        );
        assert!(!html.contains("aurora-blob"));
        assert!(!html.contains("<span"));
        // React has exactly one data-slot here.
        assert_eq!(html.matches("data-slot=").count(), 1);
        reject_fx(&html);
    }

    /// Docs "Animated backdrop": centered hero stack with a 5xl display
    /// heading, balanced copy and a large primary CTA.
    #[test]
    fn hero_content_takes_docs_recipe() {
        let mut c = stub("aurora-background", "Aurora");
        c.items.push(item(
            "title",
            "Ship something beautiful",
            &[("size", "5xl")],
        ));
        c.items.push(item("text", "Accessible, token-driven.", &[]));
        c.items.push(item(
            "action",
            "Get started",
            &[("size", "lg"), ("icon-end", "arrow-right")],
        ));
        let html = render(&c);
        assert!(html.starts_with(
            "<div data-slot=\"aurora-background\" class=\"hero\"><div aria-hidden=\"true\"><div></div><div></div><div></div></div><div><h3 class=\"t-5xl\">Ship something beautiful</h3><p>Accessible, token-driven.</p><div class=\"cta\"><button type=\"button\" data-slot=\"button\" data-variant=\"primary\" data-size=\"lg\" class=\"cui-btn\">Get started<svg"
        ));
        assert!(html.contains("data-icon=\"arrow-right\""));
        assert!(!html.contains("Aurora"));
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
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("aurora-background"),
            Some("cronus_ui_aurora_background::render")
        );
        assert_eq!(
            renderer_kind("aurora-background"),
            RendererKind::Dedicated("cronus_ui_aurora_background::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("aurora-background", "Aurora"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"aurora-background\""));
        });
    }

    /// React `bg-gradient-aurora` (three theme stops), `blur-3xl` (64px),
    /// `aurora-drift 18s ease-in-out infinite alternate` with -6s / -12s delays.
    #[test]
    fn chrome_aurora_via_css() {
        assert!(CSS.contains("[data-slot=\"aurora-background\"] > [aria-hidden] > div {"));
        assert!(!CSS.contains("[data-slot=\"aurora-blob\"]"));
        assert!(CSS.contains("@keyframes cui-aurora"));
        assert!(CSS.contains("animation: cui-aurora 18s ease-in-out infinite alternate;"));
        assert!(CSS.contains("animation-delay: -6s;"));
        assert!(CSS.contains("animation-delay: -12s;"));
        assert!(CSS.contains("background: linear-gradient(135deg, var(--cronus-primary), var(--cronus-accent), color-mix(in oklch, var(--cronus-accent) 65%, var(--cronus-primary)));"));
        assert!(CSS.contains("filter: blur(64px);"));
        assert!(CSS.contains("prefers-reduced-motion"));
        assert!(!CSS.contains("zinc-"));
        assert!(!CSS.contains(FX_BOX));
    }

    /// Wave 1t geometry: fixture `w-72 min-h-32` → 288×128.
    #[test]
    fn chrome_mirrors_fixture_box() {
        let start = CSS.find("[data-slot=\"aurora-background\"] {").unwrap();
        let block = &CSS[start..start + CSS[start..].find('}').unwrap()];
        assert!(block.contains("width: var(--cui-aurora-background-w, 100%); min-height: 8rem;"));
        let layer = CSS
            .find("[data-slot=\"aurora-background\"] > [aria-hidden] {")
            .unwrap();
        let layer = &CSS[layer..layer + CSS[layer..].find('}').unwrap()];
        // React `-z-10`: the blobs sit behind the ground, not over it.
        assert!(layer.contains("z-index: -10;"));
    }

    #[test]
    fn chrome_hero_recipe() {
        assert!(CSS.contains("[data-slot=\"aurora-background\"].hero {"));
        assert!(CSS
            .contains("min-height: 12rem; border-radius: calc(var(--cronus-radius, 14px) + 8px);"));
        assert!(CSS.contains("[data-slot=\"aurora-background\"].hero > div:last-child {"));
        assert!(CSS.contains("max-width: 32rem; padding: 3rem 1.5rem; text-align: center;"));
        assert!(CSS.contains("[data-slot=\"aurora-background\"] h3.t-5xl {"));
        assert!(CSS.contains("font-size: 3rem; line-height: 1.05;"));
        assert!(CSS.contains("text-wrap: balance;"));
    }
}
