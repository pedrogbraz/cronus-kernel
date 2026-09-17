//! Dedicated StarBorder renderer. DOM mirrors React without the injected
//! `<style>`: `<div data-slot="star-border">` + an aria-hidden overflow
//! layer holding the two sparkle `<span>`s (phase 0 and 50, i.e. a `-3s`
//! delay on the second) + a relative content `<div>` wrapping the label.
//! Only the root has a `data-slot`. `offset-path` lap + `@keyframes
//! cui-star-border` live in COMPONENT_CHROME. Zero JS, no inline style.
//!
//! Docs chrome from the `.cronus`: `card:true` (class `card`, the docs
//! `border bg-surface-raised p-8` wrapper) and `heading:xl`
//! (`<p class="h-xl">` = `font-display text-xl text-fg`).

use crate::cronus_ui_dot_pattern::HEADINGS;
use crate::cronus_ui_kit::{choice, flag, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let class = if flag(comp, "card") {
        " class=\"card\""
    } else {
        ""
    };
    let text = label_of(comp);
    let content = match choice(comp, "heading", HEADINGS) {
        Some(h) => format!("<p class=\"h-{h}\">{text}</p>"),
        None => text,
    };
    format!(
        "<div data-slot=\"star-border\"{class}><div aria-hidden=\"true\"><span></span><span></span></div><div>{content}</div></div>"
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

    /// Docs example: `<StarBorder className="rounded-2xl border … p-8">
    /// <p className="font-display text-xl text-fg">Featured</p>`.
    #[test]
    fn card_and_heading_render_the_docs_wrapper() {
        let mut c = stub("star-border", "Featured");
        c.props.insert("card".into(), "true".into());
        c.props.insert("heading".into(), "xl".into());
        assert_eq!(
            render(&c),
            "<div data-slot=\"star-border\" class=\"card\"><div aria-hidden=\"true\"><span></span><span></span></div><div><p class=\"h-xl\">Featured</p></div></div>"
        );
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("star-border", "Twinkle"));
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
        assert!(css.contains("[data-slot=\"star-border\"] {\n  position: relative; width: var(--cui-star-border-w, 100%);\n  border-radius: calc(var(--cronus-radius, 14px) + 8px);\n}"));
        assert!(css.contains("[data-slot=\"star-border\"] > [aria-hidden=\"true\"] > span {"));
        assert!(css.contains("[data-slot=\"star-border\"] > [aria-hidden=\"true\"] > span:nth-child(2) { animation-delay: -3s; }"));
        assert!(!css.contains("[data-slot=\"star-border\"]::before"));
        assert!(css.contains("@keyframes cui-star-border"));
        assert!(css.contains("animation: cui-star-border 6s linear infinite"));
        assert!(css.contains("offset-path: rect(0 auto auto 0 round 12px)"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(css.contains("[data-slot=\"star-border\"].card {\n  padding: 2rem;"));
        assert!(css.contains("[data-slot=\"star-border\"] .h-xl {"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
