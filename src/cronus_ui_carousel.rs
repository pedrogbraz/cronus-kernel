//! Dedicated Carousel renderer. DOM matches React idle:
//! `<div data-slot="carousel"><div data-slot="carousel-content">` plus
//! `carousel-item` from texts and static prev/next buttons.
//! No JS swipe. Not catalog `display()` SURF `<section>` and not interact
//! flex-overflow slides without `carousel-item`.

use crate::cronus_ui_kit::texts;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let labels = texts(comp);
    let n = labels.len();
    let items = labels
        .into_iter()
        .map(|t| {
            format!(
                "<div data-slot=\"carousel-item\" role=\"group\" aria-roledescription=\"slide\">{t}</div>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let next_disabled = if n <= 1 { " disabled" } else { "" };
    format!(
        "<div data-slot=\"carousel\" role=\"region\" aria-roledescription=\"carousel\"><div data-slot=\"carousel-content\" tabindex=\"0\">{items}</div><button type=\"button\" data-slot=\"carousel-previous\" aria-label=\"Previous slide\" disabled>Previous</button><button type=\"button\" data-slot=\"carousel-next\" aria-label=\"Next slide\"{next_disabled}>Next</button></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const DISPLAY_SURF: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem";
    const INTERACT_ROW: &str = "display:flex;gap:0.75rem;overflow:auto";

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn slides(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("carousel", items.first().copied().unwrap_or("Slide"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_stub(html: &str) {
        assert!(!html.contains("<section"));
        assert!(!html.contains("<nav"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("v-show"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("onscroll"));
        assert!(!html.contains("addEventListener"));
        assert!(!html.contains(DISPLAY_SURF));
        assert!(!html.contains(INTERACT_ROW));
        assert!(!html.contains("min-width:12rem"));
        assert!(!html.contains("display("));
        assert!(!html.contains("carousel("));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_is_region_with_content_items_and_static_buttons() {
        let html = render(&slides(&["Alpha", "Beta"]));
        assert!(html.starts_with(
            "<div data-slot=\"carousel\" role=\"region\" aria-roledescription=\"carousel\">"
        ));
        assert!(html.contains("<div data-slot=\"carousel-content\" tabindex=\"0\">"));
        assert!(html.contains(
            "<div data-slot=\"carousel-item\" role=\"group\" aria-roledescription=\"slide\">Alpha</div>"
        ));
        assert!(html.contains(">Beta</div>"));
        assert_eq!(html.matches("data-slot=\"carousel-item\"").count(), 2);
        assert!(html.contains("data-slot=\"carousel-previous\""));
        assert!(html.contains("data-slot=\"carousel-next\""));
        assert!(html.contains("aria-label=\"Previous slide\""));
        assert!(html.contains("aria-label=\"Next slide\""));
        assert!(html.contains("type=\"button\""));
        assert!(html.contains("disabled>Previous</button>"));
        assert!(!html.contains("disabled>Next</button>"));
        reject_stub(&html);
        assert_eq!(
            html,
            "<div data-slot=\"carousel\" role=\"region\" aria-roledescription=\"carousel\"><div data-slot=\"carousel-content\" tabindex=\"0\"><div data-slot=\"carousel-item\" role=\"group\" aria-roledescription=\"slide\">Alpha</div><div data-slot=\"carousel-item\" role=\"group\" aria-roledescription=\"slide\">Beta</div></div><button type=\"button\" data-slot=\"carousel-previous\" aria-label=\"Previous slide\" disabled>Previous</button><button type=\"button\" data-slot=\"carousel-next\" aria-label=\"Next slide\">Next</button></div>"
        );
    }

    #[test]
    fn extra_text_items_become_slides() {
        let mut c = stub("carousel", "Alpha");
        c.items.push(extra("text", "Beta"));
        c.items.push(extra("text", "Gamma"));
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"carousel-item\"").count(), 3);
        assert!(html.contains(">Alpha</div>"));
        assert!(html.contains(">Beta</div>"));
        assert!(html.contains(">Gamma</div>"));
        reject_stub(&html);
    }

    #[test]
    fn label_only_still_emits_one_item() {
        let html = render(&stub("carousel", "Slide"));
        assert!(html.starts_with("<div data-slot=\"carousel\""));
        assert!(html.contains("data-slot=\"carousel-content\""));
        assert_eq!(html.matches("data-slot=\"carousel-item\"").count(), 1);
        assert!(html.contains(">Slide</div>"));
        assert!(html.contains("disabled>Previous</button>"));
        assert!(html.contains("disabled>Next</button>"));
        reject_stub(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("carousel", "A <B> & \"C\""));
        assert!(html.contains(
            "aria-roledescription=\"slide\">A &lt;B&gt; &amp; &quot;C&quot;</div>"
        ));
        reject_stub(&html);
    }

    #[test]
    fn skips_interact_flex_and_display_surf() {
        let c = slides(&["Alpha", "Beta"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("carousel", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("data-slot=\"carousel\""));
        assert!(interact.contains("style="));
        assert!(interact.contains(INTERACT_ROW));
        assert!(!interact.contains("data-slot=\"carousel-content\""));
        assert!(!interact.contains("data-slot=\"carousel-item\""));
        assert!(!interact.contains("data-slot=\"carousel-previous\""));
        assert!(html.contains("data-slot=\"carousel-content\""));
        assert!(html.contains("data-slot=\"carousel-item\""));
        assert!(!html.contains("<section"));
        assert!(!html.contains(DISPLAY_SURF));
        reject_stub(&html);
        assert_eq!(
            dedicated_fn_name("carousel"),
            Some("cronus_ui_carousel::render")
        );
        assert_eq!(
            renderer_kind("carousel"),
            RendererKind::Dedicated("cronus_ui_carousel::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&slides(&["Alpha"]));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"carousel-content\""));
            assert!(html.contains("data-slot=\"carousel-item\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"carousel\"]"));
        assert!(css.contains("[data-slot=\"carousel-content\"]"));
        assert!(css.contains("[data-slot=\"carousel-item\"]"));
        assert!(css.contains("[data-slot=\"carousel-previous\"]"));
        assert!(css.contains("[data-slot=\"carousel-next\"]"));
        assert!(css.contains("scroll-snap-type: x mandatory"));
        assert!(css.contains("scroll-snap-align: start"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains(DISPLAY_SURF));
    }
}
