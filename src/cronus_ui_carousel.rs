//! Dedicated Carousel renderer. DOM matches React idle (first slide):
//! `carousel` (region, aria-label) > `carousel-content` (scroll-snap strip) >
//! `carousel-item` > card `<div>`, then a centred row with icon-only
//! `carousel-previous` / `carousel-next`.
//!
//! Zero JS: the strip scrolls/snaps natively; the prev/next buttons need the
//! Embla runtime, so both are rendered `disabled` (inert). `data-disabled`
//! marks the ones React itself disables at idle (previous; next when there is
//! a single slide) and only those dim. Not catalog `display()` SURF, not
//! interact flex-overflow slides without `carousel-item`.

use crate::cronus_ui_kit::{attr_nonempty, esc, label_of};
use crate::parser::ComponentNode;

const CHEVRON_LEFT: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m15 18-6-6 6-6\"></path></svg>";
const CHEVRON_RIGHT: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m9 18 6-6-6-6\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let mut slides: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type != "label" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if slides.is_empty() {
        slides.push(label_of(comp));
    }
    let aria = attr_nonempty(comp, "aria-label")
        .map(esc)
        .unwrap_or_else(|| label_of(comp));
    let next_idle = if slides.len() <= 1 {
        " data-disabled=\"\""
    } else {
        ""
    };
    let items = slides
        .iter()
        .map(|t| {
            format!(
                "<div data-slot=\"carousel-item\" role=\"group\" aria-roledescription=\"slide\"><div>{t}</div></div>"
            )
        })
        .collect::<String>();
    format!(
        "<div data-slot=\"carousel\" role=\"region\" aria-roledescription=\"carousel\" aria-label=\"{aria}\"><div data-slot=\"carousel-content\" tabindex=\"0\">{items}</div><div><button type=\"button\" data-slot=\"carousel-previous\" data-variant=\"outline\" aria-label=\"Previous slide\" data-disabled=\"\" disabled>{CHEVRON_LEFT}</button><button type=\"button\" data-slot=\"carousel-next\" data-variant=\"outline\" aria-label=\"Next slide\"{next_idle} disabled>{CHEVRON_RIGHT}</button></div></div>"
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

    /// Emitter shape: `label "Slides"`, `text` per slide, `aria-label:` on the last item.
    fn slides(label: &str, items: &[&str]) -> ComponentNode {
        let mut c = stub("carousel", label);
        for n in items {
            c.items.push(extra("text", n));
        }
        if let Some(last) = c.items.last_mut() {
            last.config.insert("aria-label".into(), label.into());
        }
        c
    }

    fn reject_stub(html: &str) {
        assert!(!html.contains("<section"));
        assert!(!html.contains("<nav"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-show"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("addEventListener"));
        assert!(!html.contains(DISPLAY_SURF));
        assert!(!html.contains(INTERACT_ROW));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn dom_matches_react_idle_carousel() {
        let html = render(&slides("Slides", &["One", "Two"]));
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"carousel\" role=\"region\" aria-roledescription=\"carousel\" aria-label=\"Slides\"><div data-slot=\"carousel-content\" tabindex=\"0\"><div data-slot=\"carousel-item\" role=\"group\" aria-roledescription=\"slide\"><div>One</div></div><div data-slot=\"carousel-item\" role=\"group\" aria-roledescription=\"slide\"><div>Two</div></div></div><div><button type=\"button\" data-slot=\"carousel-previous\" data-variant=\"outline\" aria-label=\"Previous slide\" data-disabled=\"\" disabled>{CHEVRON_LEFT}</button><button type=\"button\" data-slot=\"carousel-next\" data-variant=\"outline\" aria-label=\"Next slide\" disabled>{CHEVRON_RIGHT}</button></div></div>"
            )
        );
        assert!(!html.contains(">Slides<"));
        reject_stub(&html);
    }

    #[test]
    fn label_only_still_emits_one_item_and_idle_disables_next() {
        let html = render(&stub("carousel", "Slide"));
        assert_eq!(html.matches("data-slot=\"carousel-item\"").count(), 1);
        assert!(html.contains("<div>Slide</div>"));
        assert!(html.contains("aria-label=\"Next slide\" data-disabled=\"\" disabled>"));
        reject_stub(&html);
    }

    #[test]
    fn slide_text_is_escaped() {
        let html = render(&slides("S", &["A <B> & \"C\""]));
        assert!(html.contains("<div>A &lt;B&gt; &amp; &quot;C&quot;</div>"));
        reject_stub(&html);
    }

    #[test]
    fn skips_interact_flex_and_display_surf() {
        let c = slides("Slides", &["Alpha", "Beta"]);
        let html = render(&c);
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
            let html = render(&slides("S", &["Alpha"]));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"carousel-item\""));
        });
    }

    #[test]
    fn chrome_is_token_only_and_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"carousel\"] { position: relative; outline: none; width: 18rem;"
        ));
        assert!(css.contains("margin-left: -1rem; display: flex; overflow-x: auto;"));
        assert!(css.contains("min-width: 0; flex: 0 0 100%; padding-left: 1rem;"));
        assert!(css.contains("width: 2.25rem; height: 2.25rem;"));
        assert!(css.contains("[data-slot=\"carousel-previous\"][data-disabled], [data-slot=\"carousel-next\"][data-disabled] { opacity: 0.5; }"));
        assert!(css.contains("scroll-snap-type: x mandatory"));
        assert!(css.contains("scroll-snap-align: start"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(!css.contains("zinc-"));
    }
}
