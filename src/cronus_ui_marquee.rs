//! Dedicated Marquee renderer. DOM matches React:
//! `<div data-slot="marquee"><div data-slot="marquee-group">` with items
//! from texts. CSS scroll in COMPONENT_CHROME. Zero JS.
//! Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::texts;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let items = texts(comp)
        .into_iter()
        .map(|t| format!("<span>{t}</span>"))
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"marquee\"><div data-slot=\"marquee-group\">{items}</div></div>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn list(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("marquee", items.first().copied().unwrap_or("Acme"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_is_marquee_group_of_texts_not_fx_title_box() {
        let html = render(&list(&["Acme", "Stripe"]));
        assert_eq!(
            html,
            "<div data-slot=\"marquee\"><div data-slot=\"marquee-group\"><span>Acme</span><span>Stripe</span></div></div>"
        );
        assert!(html.starts_with("<div data-slot=\"marquee\">"));
        assert!(html.contains("data-slot=\"marquee-group\""));
        assert_eq!(html.matches("data-slot=\"marquee-group\"").count(), 1);
        reject_fx(&html);
    }

    #[test]
    fn extra_text_items_become_spans() {
        let mut c = stub("marquee", "Acme");
        c.items.push(extra("text", "Stripe"));
        c.items.push(extra("text", "Vercel"));
        let html = render(&c);
        assert_eq!(html.matches("<span>").count(), 3);
        assert!(html.contains("<span>Acme</span>"));
        assert!(html.contains("<span>Stripe</span>"));
        assert!(html.contains("<span>Vercel</span>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("marquee", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"marquee\"><div data-slot=\"marquee-group\"><span>A &lt;B&gt; &amp; &quot;C&quot;</span></div></div>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("marquee", "Acme");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.contains("<span>"));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        assert!(!html.contains("<span>Demo</span></div>"));
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("marquee"),
            Some("cronus_ui_marquee::render")
        );
        assert_eq!(
            renderer_kind("marquee"),
            RendererKind::Dedicated("cronus_ui_marquee::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("marquee", "Acme"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"marquee-group\""));
        });
    }

    #[test]
    fn chrome_marquee_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"marquee\"]"));
        assert!(css.contains("[data-slot=\"marquee-group\"]"));
        assert!(css.contains("@keyframes cui-marquee"));
        assert!(css.contains("animation: cui-marquee"));
        assert!(css.contains("overflow: hidden"));
        assert!(css.contains("var(--cronus-fg)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
