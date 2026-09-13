//! Dedicated LogoCarousel renderer. DOM matches React idle:
//! `<ul data-slot="logo-carousel">` plus `logo-carousel-item` from texts.
//! No setInterval. CSS in COMPONENT_CHROME. Not interact flex-overflow
//! `<div data-slot="logo-carousel" style=...>` without items.

use crate::cronus_ui_kit::{esc, item, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let items = texts(comp)
        .into_iter()
        .map(|t| format!("<li data-slot=\"logo-carousel-item\">{t}</li>"))
        .collect::<Vec<_>>()
        .join("");
    let aria = match item(comp, "label").filter(|t| !t.is_empty()) {
        Some(label) => esc(label),
        None => "Logo carousel".into(),
    };
    format!("<ul data-slot=\"logo-carousel\" aria-label=\"{aria}\" aria-live=\"off\">{items}</ul>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const INTERACT_ROW: &str = "display:flex;gap:0.75rem;overflow:auto";
    const DISPLAY_SURF: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem";
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

    fn logos(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("logo-carousel", items.first().copied().unwrap_or("Acme"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_stub(html: &str) {
        assert!(!html.contains("<section"));
        assert!(!html.contains("<div"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("onscroll"));
        assert!(!html.contains("addEventListener"));
        assert!(!html.contains(INTERACT_ROW));
        assert!(!html.contains("min-width:12rem"));
        assert!(!html.contains(DISPLAY_SURF));
        assert!(!html.contains("display("));
        assert!(!html.contains("carousel("));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_is_ul_with_items_not_interact_flex() {
        let html = render(&logos(&["Acme", "Stripe"]));
        assert!(html.starts_with("<ul data-slot=\"logo-carousel\""));
        assert!(html.contains("aria-live=\"off\""));
        assert!(html.contains("aria-label=\"Logo carousel\""));
        assert_eq!(html.matches("data-slot=\"logo-carousel-item\"").count(), 2);
        assert!(html.contains("<li data-slot=\"logo-carousel-item\">Acme</li>"));
        assert!(html.contains("<li data-slot=\"logo-carousel-item\">Stripe</li>"));
        reject_stub(&html);
        assert_eq!(
            html,
            "<ul data-slot=\"logo-carousel\" aria-label=\"Logo carousel\" aria-live=\"off\"><li data-slot=\"logo-carousel-item\">Acme</li><li data-slot=\"logo-carousel-item\">Stripe</li></ul>"
        );
    }

    #[test]
    fn extra_text_items_become_logos() {
        let mut c = stub("logo-carousel", "Acme");
        c.items.push(extra("text", "Stripe"));
        c.items.push(extra("text", "Vercel"));
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"logo-carousel-item\"").count(), 3);
        assert!(html.contains(">Acme</li>"));
        assert!(html.contains(">Stripe</li>"));
        assert!(html.contains(">Vercel</li>"));
        assert!(html.contains("aria-label=\"Acme\""));
        reject_stub(&html);
    }

    #[test]
    fn label_only_still_emits_one_item() {
        let html = render(&stub("logo-carousel", "Acme"));
        assert!(html.starts_with("<ul data-slot=\"logo-carousel\""));
        assert_eq!(html.matches("data-slot=\"logo-carousel-item\"").count(), 1);
        assert!(html.contains(">Acme</li>"));
        assert!(html.contains("aria-label=\"Acme\""));
        assert!(html.contains("aria-live=\"off\""));
        reject_stub(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("logo-carousel", "A <B> & \"C\""));
        assert!(html.contains(
            "<li data-slot=\"logo-carousel-item\">A &lt;B&gt; &amp; &quot;C&quot;</li>"
        ));
        assert!(html.contains("aria-label=\"A &lt;B&gt; &amp; &quot;C&quot;\""));
        reject_stub(&html);
    }

    #[test]
    fn skips_interact_flex_and_display_surf() {
        let c = logos(&["Acme", "Stripe"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("logo-carousel", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("data-slot=\"logo-carousel\""));
        assert!(interact.contains("<div data-slot=\"logo-carousel\""));
        assert!(interact.contains("style="));
        assert!(interact.contains(INTERACT_ROW));
        assert!(!interact.contains("data-slot=\"logo-carousel-item\""));
        assert!(!interact.contains("<ul"));
        assert!(html.contains("data-slot=\"logo-carousel-item\""));
        assert!(html.starts_with("<ul"));
        assert!(!html.contains("<section"));
        assert!(!html.contains(DISPLAY_SURF));
        reject_stub(&html);
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert_ne!(html, fx);
        assert_eq!(
            dedicated_fn_name("logo-carousel"),
            Some("cronus_ui_logo_carousel::render")
        );
        assert_eq!(
            renderer_kind("logo-carousel"),
            RendererKind::Dedicated("cronus_ui_logo_carousel::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&logos(&["Acme"]));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"logo-carousel-item\""));
            assert!(html.contains("aria-live=\"off\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"logo-carousel\"]"));
        assert!(css.contains("[data-slot=\"logo-carousel-item\"]"));
        assert!(css.contains("list-style: none"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains("setInterval"));
        assert!(!css.contains(INTERACT_ROW));
        assert!(!css.contains(DISPLAY_SURF));
    }
}
