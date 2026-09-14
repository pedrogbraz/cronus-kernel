//! Dedicated StarBorder renderer. DOM mirrors React without the injected
//! `<style>`: `<div data-slot="star-border">` + aria-hidden clip `<div>`
//! holding two sparkle `<span>`s + a relative content `<div>` wrapping the
//! label. Sparkles ride `offset-path: rect(… round 12px)` with
//! `@keyframes cui-star-border` (second one delayed -3s, as React's -50%
//! phase) in COMPONENT_CHROME, with `rounded-2xl` and fixture `w-72`. Zero
//! JS, no inline style. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"star-border\"><div aria-hidden=\"true\"><span></span><span></span></div><div>{}</div></div>",
        label_of(comp)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("<style"));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_has_sparkle_layer_and_content_div() {
        let html = render(&stub("star-border", "Twinkle"));
        assert_eq!(
            html,
            "<div data-slot=\"star-border\"><div aria-hidden=\"true\"><span></span><span></span></div><div>Twinkle</div></div>"
        );
        assert_eq!(html.matches("data-slot=").count(), 1);
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("star-border", "A <B> & \"C\""));
        assert!(html.ends_with("<div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("star-border", "Twinkle"));
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("star-border", "Twinkle"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"star-border\""));
        });
    }

    #[test]
    fn chrome_star_border_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"star-border\"] {\n  position: relative; width: 18rem;\n  border-radius: calc(var(--cronus-radius, 14px) + 8px);\n}"));
        assert!(css.contains("[data-slot=\"star-border\"] > [aria-hidden=\"true\"] > span {"));
        assert!(css.contains("[data-slot=\"star-border\"] > [aria-hidden=\"true\"] > span:nth-child(2) { animation-delay: -3s; }"));
        assert!(!css.contains("[data-slot=\"star-border\"]::before"));
        assert!(css.contains("@keyframes cui-star-border"));
        assert!(css.contains("animation: cui-star-border"));
        assert!(css.contains("offset-path"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
