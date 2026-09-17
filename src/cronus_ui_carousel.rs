//! Dedicated Carousel renderer. DOM matches React idle (first slide):
//! `carousel` (region, aria-label) > `carousel-content` (scroll-snap strip) >
//! `carousel-item` > card `<div>`, then a centred row with icon-only
//! `carousel-previous` / `carousel-next` (and, with `dots:true`, React's
//! `carousel-dots` between them).
//!
//! Zero JS navigation: every slide has a page-unique fragment id
//! (`{id}-slide-N`) and carries unslotted `<a href="#{id}-slide-N±1">` links for
//! its own previous/next. The links are absolutely positioned over React's
//! prev/next buttons (anchor-positioned to them; their containing block is the
//! carousel root, outside the scroll strip, so they are not clipped and add no
//! scroll overflow). Only the `:target` slide's pair is displayed (the first
//! slide's pair when no slide is targeted), so a click follows the fragment and
//! the strip scrolls/snaps to the slide natively. The buttons keep React's slot
//! and idle look but are decorative (`aria-hidden`, `tabindex="-1"`); the links
//! are the keyboard and screen-reader controls. `data-disabled` marks what
//! React disables at idle (previous; next with one slide); CSS re-derives the
//! edges from `:target`. Dots are real `<a href="#…">` controls in React's
//! `carousel-dot` slot; the active dot follows `:target` too.
//!
//! Docs slide: `item "1" description:"Onboarding"` renders the `h-40` tile
//! (display number over a caption); `width:sm|md|lg` is the docs `max-w-*`;
//! `align:center` snaps slides to the centre.
//!
//! Gaps vs Embla: fragment navigation also scrolls the page so the slide is in
//! view and adds a history entry; swiping/scrolling the strip by hand does not
//! move `:target`, so prev/next then act relative to the last linked slide; no
//! Arrow-key handling on the region; `loop` cannot wrap. Not catalog
//! `display()` SURF, not interact flex-overflow slides without `carousel-item`.

use crate::cronus_ui_kit::{attr_nonempty, esc, flag, instance_id, label_of};
use crate::parser::ComponentNode;

const CHEVRON_LEFT: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m15 18-6-6 6-6\"></path></svg>";
const CHEVRON_RIGHT: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m9 18 6-6-6-6\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let mut slides: Vec<(String, Option<String>)> = comp
        .items
        .iter()
        .filter(|i| i.item_type != "label" && !i.text.is_empty())
        .map(|i| {
            (
                esc(&i.text),
                i.config
                    .get("description")
                    .filter(|d| !d.is_empty())
                    .map(|d| esc(d)),
            )
        })
        .collect();
    if slides.is_empty() {
        slides.push((label_of(comp), None));
    }
    let aria = attr_nonempty(comp, "aria-label")
        .map(esc)
        .unwrap_or_else(|| label_of(comp));
    let id = instance_id(comp, "carousel");
    let n = slides.len();
    let next_idle = if n <= 1 { " data-disabled=\"\"" } else { "" };
    let mut classes: Vec<String> = Vec::new();
    if let Some(w @ ("xs" | "sm" | "md" | "lg")) = attr_nonempty(comp, "width") {
        classes.push(format!("w-{w}"));
    }
    if attr_nonempty(comp, "align") == Some("center") {
        classes.push("center".into());
    }
    let class = if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    };
    let items = slides
        .iter()
        .enumerate()
        .map(|(i, (t, caption))| {
            let prev = if i > 0 {
                format!(
                    "<a href=\"#{id}-slide-{}\" data-nav=\"previous\" aria-label=\"Previous slide\"></a>",
                    i - 1
                )
            } else {
                String::new()
            };
            let next = if i + 1 < n {
                format!(
                    "<a href=\"#{id}-slide-{}\" data-nav=\"next\" aria-label=\"Next slide\"></a>",
                    i + 1
                )
            } else {
                String::new()
            };
            let card = match caption {
                Some(c) => format!("<div class=\"tile\"><span>{t}</span><span>{c}</span></div>"),
                None => format!("<div>{t}</div>"),
            };
            format!(
                "<div data-slot=\"carousel-item\" id=\"{id}-slide-{i}\" role=\"group\" aria-roledescription=\"slide\">{card}{prev}{next}</div>"
            )
        })
        .collect::<String>();
    let dots = if flag(comp, "dots") {
        let links: String = (0..n)
            .map(|i| {
                let active = if i == 0 {
                    " data-active=\"\" aria-current=\"true\""
                } else {
                    ""
                };
                format!(
                    "<a href=\"#{id}-slide-{i}\" data-slot=\"carousel-dot\"{active} aria-label=\"Go to slide {}\"></a>",
                    i + 1
                )
            })
            .collect();
        format!("<div data-slot=\"carousel-dots\">{links}</div>")
    } else {
        String::new()
    };
    format!(
        "<div data-slot=\"carousel\"{class} role=\"region\" aria-roledescription=\"carousel\" aria-label=\"{aria}\"><div data-slot=\"carousel-content\" tabindex=\"0\">{items}</div><div><button type=\"button\" data-slot=\"carousel-previous\" data-variant=\"outline\" aria-label=\"Previous slide\" tabindex=\"-1\" aria-hidden=\"true\" data-disabled=\"\">{CHEVRON_LEFT}</button>{dots}<button type=\"button\" data-slot=\"carousel-next\" data-variant=\"outline\" aria-label=\"Next slide\" tabindex=\"-1\" aria-hidden=\"true\"{next_idle}>{CHEVRON_RIGHT}</button></div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::{reset_instance_ids, stub};
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
        assert!(!html.contains(" disabled"));
    }

    #[test]
    fn dom_matches_react_idle_carousel() {
        let html = render(&slides("Slides", &["One", "Two"]));
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"carousel\" role=\"region\" aria-roledescription=\"carousel\" aria-label=\"Slides\"><div data-slot=\"carousel-content\" tabindex=\"0\"><div data-slot=\"carousel-item\" id=\"cui-carousel-carousel-slide-0\" role=\"group\" aria-roledescription=\"slide\"><div>One</div><a href=\"#cui-carousel-carousel-slide-1\" data-nav=\"next\" aria-label=\"Next slide\"></a></div><div data-slot=\"carousel-item\" id=\"cui-carousel-carousel-slide-1\" role=\"group\" aria-roledescription=\"slide\"><div>Two</div><a href=\"#cui-carousel-carousel-slide-0\" data-nav=\"previous\" aria-label=\"Previous slide\"></a></div></div><div><button type=\"button\" data-slot=\"carousel-previous\" data-variant=\"outline\" aria-label=\"Previous slide\" tabindex=\"-1\" aria-hidden=\"true\" data-disabled=\"\">{CHEVRON_LEFT}</button><button type=\"button\" data-slot=\"carousel-next\" data-variant=\"outline\" aria-label=\"Next slide\" tabindex=\"-1\" aria-hidden=\"true\">{CHEVRON_RIGHT}</button></div></div>"
            )
        );
        assert!(!html.contains(">Slides<"));
        reject_stub(&html);
    }

    /// Each slide links to its neighbours only; the edges have no link.
    #[test]
    fn slides_link_to_their_neighbours() {
        let html = render(&slides("S", &["A", "B", "C"]));
        let item = |i: usize| {
            let start = html
                .find(&format!("id=\"cui-carousel-carousel-slide-{i}\""))
                .unwrap();
            let end = start + html[start..].find("</a></div>").unwrap_or(0);
            html[start..end].to_string()
        };
        assert!(!item(0).contains("data-nav=\"previous\""));
        assert!(item(0).contains("href=\"#cui-carousel-carousel-slide-1\" data-nav=\"next\""));
        assert!(item(1).contains("href=\"#cui-carousel-carousel-slide-0\" data-nav=\"previous\""));
        assert!(item(1).contains("href=\"#cui-carousel-carousel-slide-2\" data-nav=\"next\""));
        assert_eq!(html.matches("data-nav=\"next\"").count(), 2);
        assert_eq!(html.matches("data-nav=\"previous\"").count(), 2);
    }

    #[test]
    fn two_carousels_on_one_page_have_distinct_slide_ids() {
        reset_instance_ids();
        let a = render(&slides("S", &["A", "B"]));
        let b = render(&slides("S", &["A", "B"]));
        assert!(a.contains("id=\"cui-carousel-carousel-slide-1\""));
        assert!(b.contains("id=\"cui-carousel-carousel-2-slide-1\""));
        assert!(b.contains("href=\"#cui-carousel-carousel-2-slide-1\""));
    }

    #[test]
    fn label_only_still_emits_one_item_and_idle_disables_next() {
        let html = render(&stub("carousel", "Slide"));
        assert_eq!(html.matches("data-slot=\"carousel-item\"").count(), 1);
        assert!(html.contains("<div>Slide</div></div>"));
        assert!(!html.contains("<a "));
        assert!(html.contains(
            "aria-label=\"Next slide\" tabindex=\"-1\" aria-hidden=\"true\" data-disabled=\"\">"
        ));
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
            "[data-slot=\"carousel\"] { position: relative; outline: none; width: var(--cui-carousel-w, 100%);"
        ));
        assert!(css.contains("margin-inline-start: -1rem; display: flex; overflow-x: auto;"));
        assert!(css.contains("min-width: 0; flex: 0 0 100%; padding-inline-start: 1rem;"));
        assert!(css.contains("width: 2.25rem; height: 2.25rem;"));
        assert!(css.contains("[data-slot=\"carousel-previous\"][data-disabled], [data-slot=\"carousel-next\"][data-disabled] { opacity: 0.5; }"));
        assert!(css.contains("scroll-snap-type: x mandatory"));
        assert!(css.contains("scroll-snap-align: start"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(!css.contains("zinc-"));
    }

    /// Links sit over the buttons (row of two 2.25rem buttons, 0.5rem gap,
    /// centred), only the live slide's pair shows, and edges follow `:target`.
    #[test]
    fn chrome_links_follow_target_slide() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"carousel-item\"] > a[data-nav=\"previous\"] { inset-inline-start: calc(50% - 2.5rem); }"));
        assert!(css.contains("[data-slot=\"carousel-item\"] > a[data-nav=\"next\"] { inset-inline-start: calc(50% + 0.25rem); }"));
        assert!(css.contains("[data-slot=\"carousel-item\"]:target > a,\n[data-slot=\"carousel-content\"]:not(:has(> :target)) > [data-slot=\"carousel-item\"]:first-child > a {\n  display: block;\n}"));
        assert!(css.contains(
            ":target:not(:first-child)) [data-slot=\"carousel-previous\"] { opacity: 1; }"
        ));
        assert!(css.contains(":target:last-child) [data-slot=\"carousel-next\"] { opacity: 0.5; }"));
    }

    /// Docs "Slides": five numbered tiles with captions inside a `max-w-sm`
    /// carousel, prev / dots / next in a `mt-4 gap-3` row. Dots are real
    /// fragment links in React's `carousel-dot` slot; the first is active.
    #[test]
    fn docs_tiles_dots_and_width() {
        reset_instance_ids();
        let mut c = stub("carousel", "Slides");
        c.items.clear();
        for (n, label) in [("1", "Onboarding"), ("2", "Checkout"), ("3", "Repasse")] {
            let mut i = extra("item", n);
            i.config.insert("description".into(), label.into());
            c.items.push(i);
        }
        c.props.insert("width".into(), "sm".into());
        c.props.insert("dots".into(), "true".into());
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"carousel\" class=\"w-sm\" role=\"region\""));
        assert!(html.contains("aria-roledescription=\"slide\"><div class=\"tile\"><span>1</span><span>Onboarding</span></div><a href=\"#cui-carousel-carousel-slide-1\" data-nav=\"next\""));
        assert!(html.contains("</button><div data-slot=\"carousel-dots\"><a href=\"#cui-carousel-carousel-slide-0\" data-slot=\"carousel-dot\" data-active=\"\" aria-current=\"true\" aria-label=\"Go to slide 1\"></a><a href=\"#cui-carousel-carousel-slide-1\" data-slot=\"carousel-dot\" aria-label=\"Go to slide 2\"></a><a href=\"#cui-carousel-carousel-slide-2\" data-slot=\"carousel-dot\" aria-label=\"Go to slide 3\"></a></div><button type=\"button\" data-slot=\"carousel-next\""));
        reject_stub(&html);
        c.props.insert("align".into(), "center".into());
        assert!(render(&c).contains("class=\"w-sm center\""));
    }

    #[test]
    fn chrome_docs_tile_dots_and_anchored_links() {
        let css = include_str!("cronus_ui_css/carousel.css");
        assert!(css.contains("[data-slot=\"carousel\"].w-sm { max-width: 24rem; }"));
        assert!(css.contains("[data-slot=\"carousel\"].center [data-slot=\"carousel-item\"] { scroll-snap-align: center; }"));
        assert!(css.contains("[data-slot=\"carousel-item\"] > .tile {"));
        assert!(css.contains("height: 10rem; padding: 0; gap: 0.25rem;"));
        assert!(css.contains("[data-slot=\"carousel-dots\"] { display: flex; align-items: center; justify-content: center; gap: 0.5rem; }"));
        assert!(css.contains("[data-slot=\"carousel-dot\"] {"));
        assert!(css.contains("width: 0.5rem; height: 0.5rem; border-radius: 9999px;"));
        assert!(css.contains(
            "[data-slot=\"carousel-dot\"][data-active] { background: var(--cronus-fg); }"
        ));
        assert!(css.contains("transition: background-color 150ms var(--ease-out-quart);"));
        for k in 1..=12 {
            assert!(css.contains(&format!(":nth-child({k}):target) [data-slot=\"carousel-dot\"]:nth-child({k}) {{ background: var(--cronus-fg); }}")), "{k}");
        }
        assert!(css.contains("anchor-name: --cui-carousel-previous;"));
        assert!(css.contains("anchor-scope: --cui-carousel-previous, --cui-carousel-next;"));
        assert!(css.contains("inset-inline-start: anchor(--cui-carousel-next start); top: anchor(--cui-carousel-next top);"));
    }
}
