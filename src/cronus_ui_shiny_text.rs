//! Dedicated ShinyText renderer. DOM mirrors React without the injected
//! `<style>`: a `display: contents` `<span data-slot="shiny-text">` wrapping
//! the painted `<span>` (gradient `background-clip: text`, 3s sweep). The
//! sheen lives in COMPONENT_CHROME. Zero JS, no inline style.
//!
//! `heading:<size>` puts the docs' `font-display text-<size>` on the painted
//! span (class `h-<size>`).

use crate::cronus_ui_dot_pattern::HEADINGS;
use crate::cronus_ui_kit::{choice, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let class = choice(comp, "heading", HEADINGS)
        .map(|h| format!(" class=\"h-{h}\""))
        .unwrap_or_default();
    format!(
        "<span data-slot=\"shiny-text\"><span{class}>{}</span></span>",
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
        assert!(!html.contains("<div"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
        assert_eq!(html.matches("data-slot=").count(), 1);
    }

    #[test]
    fn contents_slot_wraps_painted_span() {
        let html = render(&stub("shiny-text", "Sheen"));
        assert_eq!(
            html,
            "<span data-slot=\"shiny-text\"><span>Sheen</span></span>"
        );
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("shiny-text", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<span data-slot=\"shiny-text\"><span>A &lt;B&gt; &amp; &quot;C&quot;</span></span>"
        );
        reject_fx(&html);
    }

    /// Docs example: `<ShinyText className="font-display text-4xl">Ship the surface</ShinyText>`.
    #[test]
    fn heading_class_goes_on_the_painted_span() {
        let mut c = stub("shiny-text", "Ship the surface");
        c.props.insert("heading".into(), "4xl".into());
        assert_eq!(
            render(&c),
            "<span data-slot=\"shiny-text\"><span class=\"h-4xl\">Ship the surface</span></span>"
        );
        c.props.insert("heading".into(), "giant".into());
        assert!(!render(&c).contains("class="));
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("shiny-text", "Shimmer");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("shiny-text"),
            Some("cronus_ui_shiny_text::render")
        );
        assert_eq!(
            renderer_kind("shiny-text"),
            RendererKind::Dedicated("cronus_ui_shiny_text::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("shiny-text", "Shimmer"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"shiny-text\""));
        });
    }

    #[test]
    fn chrome_paints_text_transparent_with_clipped_gradient() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"shiny-text\"] { display: contents; }"));
        let start = css.find("[data-slot=\"shiny-text\"] > span {").unwrap();
        let end = start + css[start..].find('}').unwrap();
        let painted = &css[start..end];
        assert!(painted.contains("display: inline; color: transparent;"));
        assert!(painted.contains("background-size: 200% 100%;"));
        assert!(painted.contains("background-clip: text;"));
        assert!(painted.contains("animation: cui-shiny-text 3s linear infinite;"));
        assert!(painted.contains("var(--cronus-fg) 50%"));
        assert!(css.contains("@keyframes cui-shiny-text {\n  0% { background-position: 100% 0; }\n  100% { background-position: -100% 0; }\n}"));
        assert!(css.contains(
            "[data-slot=\"shiny-text\"] > .h-4xl { font-size: 2.25rem; line-height: 2.5rem; }"
        ));
        assert!(css.contains("[data-slot=\"shiny-text\"] > span[class*=\"h-\"] { font-family: var(--cronus-font-display, inherit); }"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
