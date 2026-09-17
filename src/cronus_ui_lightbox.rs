//! Dedicated Lightbox renderer (wave1t geometry parity with the React Lightbox).
//!
//! React renders an open, full-viewport Radix Dialog portaled to `<body>`:
//! `dialog-overlay` + `dialog-content` (fixed inset-0, bg black/95, white text)
//! wrapping `lightbox` > header (`lightbox-counter` + icon-only `lightbox-close`),
//! stage (prev button, `img lightbox-image`, next button) and, with more than one
//! image, `lightbox-thumbnails` (one 56px button per image). No caption without
//! a caption prop. Images are the `text` items (alts); `.cronus` carries no image
//! source, so each item gets a transparent 64x64 SVG placeholder painted with a
//! token (inset box-shadow, background stays clear).
//!
//! Zero JS, open by default like React's `open` fixture:
//! - **Image selection**: the stage holds, per image, a visually hidden radio
//!   (page-unique `name`, labelled by the alt) followed by its `lightbox-image`
//!   and unslotted `<label for>`s pointing at the previous/next image's radio.
//!   CSS shows the image and the prev/next labels after the checked radio, so
//!   only one image renders (React's DOM) and the labels sit exactly over
//!   React's prev/next buttons. Arrow keys on the focused radio move too.
//!   Thumbnails are wrapped in `<label for>` the image's radio. The counter holds
//!   one number per image; CSS shows the checked one (first `MAX_IMAGES`), and
//!   the edge dimming / thumbnail ring follow the checked radio.
//! - **Close**: the close button sits in a `<label>` with a visually hidden
//!   checkbox ("Close"); checking it hides the overlay and the content.
//!
//! React's buttons keep their slots and look but are decorative (`aria-hidden`,
//! `tabindex="-1"`, `pointer-events: none`).
//!
//! Gaps vs Radix: no focus trap, Escape or outside-click close; once closed
//! there is no trigger to reopen (React's component has none either — the
//! fixture is controlled `open`); `aria-current` on thumbnails and the
//! `data-edge` attributes stay at the initial state.
//!
//! Closed mode (`open:false` / `defaultOpen:false`, or `trigger:` — the docs'
//! gallery): the images render first as the docs' `grid grid-cols-3 gap-2` of
//! thumbnails, each a `<label for>` its image's stage radio (React's
//! `<button onClick>` + `<img>`), so clicking one opens the lightbox at that
//! image. No radio starts checked, and the panel stays hidden until one is;
//! the header's close control is a radio of the same group ("Close"), so it
//! unchecks the image and hides the panel again. React's `dialog-overlay` is
//! omitted here: the opaque full-viewport `dialog-content` covers it, and its
//! presence would pin the specimen inside the catalog frame. Images can carry
//! a `src:"…"` (through `safe_url`), else the placeholder.

use crate::cronus_ui_kit::{esc, instance_id, label_of, overlay_trigger, safe_url};
use crate::parser::ComponentNode;

/// Images beyond this index still switch, but the counter shows no number and
/// their thumbnail gets no ring.
pub const MAX_IMAGES: usize = 16;

const PLACEHOLDER: &str = "data:image/svg+xml,%3Csvg%20xmlns%3D%27http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%27%20width%3D%2764%27%20height%3D%2764%27%2F%3E";

pub fn render(comp: &ComponentNode) -> String {
    let gallery = comp
        .props
        .get("aria-label")
        .filter(|s| !s.is_empty())
        .map(|s| esc(s))
        .unwrap_or_else(|| label_of(comp));
    let images = images(comp);
    let count = images.len();
    let id = instance_id(comp, "lightbox");
    let closed = overlay_trigger(comp, "").is_some();
    let counter = if count == 0 {
        "0".to_string()
    } else {
        (1..=count).map(|i| format!("<span>{i}</span>")).collect()
    };
    let stage_images = images
        .iter()
        .enumerate()
        .map(|(i, (alt, src))| {
            let checked = if i == 0 && !closed { " checked" } else { "" };
            let prev = if i > 0 {
                format!("<label for=\"{id}-i{}\" data-nav=\"previous\"></label>", i - 1)
            } else {
                String::new()
            };
            let next = if i + 1 < count {
                format!("<label for=\"{id}-i{}\" data-nav=\"next\"></label>", i + 1)
            } else {
                String::new()
            };
            format!(
                "<input type=\"radio\" name=\"{id}\" id=\"{id}-i{i}\" aria-label=\"{alt}\"{checked}><img data-slot=\"lightbox-image\" src=\"{src}\" alt=\"{alt}\">{prev}{next}"
            )
        })
        .collect::<String>();
    let prev_edge = " data-edge";
    let next_edge = if count <= 1 { " data-edge" } else { "" };
    let thumbnails = if count > 1 {
        let buttons = images
            .iter()
            .enumerate()
            .map(|(i, (_, src))| {
                format!(
                    "<label for=\"{id}-i{i}\"><button type=\"button\" aria-label=\"View image {}\" aria-current=\"{}\" tabindex=\"-1\" aria-hidden=\"true\"><img src=\"{src}\" alt=\"\"></button></label>",
                    i + 1,
                    i == 0
                )
            })
            .collect::<String>();
        format!("<div data-slot=\"lightbox-thumbnails\">{buttons}</div>")
    } else {
        String::new()
    };
    let (grid, overlay, close_input) = if closed {
        let cells = images
            .iter()
            .enumerate()
            .map(|(i, (alt, src))| {
                format!("<label for=\"{id}-i{i}\"><img src=\"{src}\" alt=\"{alt}\"></label>")
            })
            .collect::<String>();
        (
            format!("<div class=\"grid\">{cells}</div>"),
            String::new(),
            format!("<input type=\"radio\" name=\"{id}\" id=\"{id}-none\" aria-label=\"Close\">"),
        )
    } else {
        (
            String::new(),
            "<div data-slot=\"dialog-overlay\" aria-hidden=\"true\"></div>".to_string(),
            "<input type=\"checkbox\" aria-label=\"Close\">".to_string(),
        )
    };
    format!(
        "{grid}{overlay}<div role=\"dialog\" aria-label=\"{gallery}\" data-slot=\"dialog-content\"><div data-slot=\"lightbox\"><div><span data-slot=\"lightbox-counter\">{counter} / {count}</span><label>{close_input}<button type=\"button\" data-slot=\"lightbox-close\" aria-label=\"Close\" tabindex=\"-1\" aria-hidden=\"true\"></button></label></div><div><button type=\"button\" aria-label=\"Previous image\" tabindex=\"-1\" aria-hidden=\"true\"{prev_edge}></button>{stage_images}<button type=\"button\" aria-label=\"Next image\" tabindex=\"-1\" aria-hidden=\"true\"{next_edge}></button></div>{thumbnails}</div></div>"
    )
}

/// `(alt, src)` per `text` item; `src:` config through `safe_url`, else the placeholder.
fn images(comp: &ComponentNode) -> Vec<(String, String)> {
    comp.items
        .iter()
        .filter(|i| i.item_type == "text" && !i.text.is_empty())
        .map(|i| {
            let src = i
                .config
                .get("src")
                .filter(|s| !s.trim().is_empty())
                .map(|s| safe_url(s))
                .unwrap_or_else(|| PLACEHOLDER.to_string());
            (esc(&i.text), src)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::{reset_instance_ids, stub};
    use crate::parser::ComponentItemNode;

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn gallery(items: &[&str]) -> ComponentNode {
        let mut c = stub("lightbox", "Image gallery");
        for i in items {
            c.items.push(extra("text", i));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<dialog"));
        assert!(!html.contains("showModal"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("max-width:28rem"));
        assert!(!html.contains("lightbox-caption"));
        assert!(!html.contains(" disabled"));
    }

    #[test]
    fn react_dom_two_images() {
        let html = render(&gallery(&["First image", "Second image"]));
        let p = PLACEHOLDER;
        let n = "cui-lightbox-lightbox";
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"dialog-overlay\" aria-hidden=\"true\"></div><div role=\"dialog\" aria-label=\"Image gallery\" data-slot=\"dialog-content\"><div data-slot=\"lightbox\"><div><span data-slot=\"lightbox-counter\"><span>1</span><span>2</span> / 2</span><label><input type=\"checkbox\" aria-label=\"Close\"><button type=\"button\" data-slot=\"lightbox-close\" aria-label=\"Close\" tabindex=\"-1\" aria-hidden=\"true\"></button></label></div><div><button type=\"button\" aria-label=\"Previous image\" tabindex=\"-1\" aria-hidden=\"true\" data-edge></button><input type=\"radio\" name=\"{n}\" id=\"{n}-i0\" aria-label=\"First image\" checked><img data-slot=\"lightbox-image\" src=\"{p}\" alt=\"First image\"><label for=\"{n}-i1\" data-nav=\"next\"></label><input type=\"radio\" name=\"{n}\" id=\"{n}-i1\" aria-label=\"Second image\"><img data-slot=\"lightbox-image\" src=\"{p}\" alt=\"Second image\"><label for=\"{n}-i0\" data-nav=\"previous\"></label><button type=\"button\" aria-label=\"Next image\" tabindex=\"-1\" aria-hidden=\"true\"></button></div><div data-slot=\"lightbox-thumbnails\"><label for=\"{n}-i0\"><button type=\"button\" aria-label=\"View image 1\" aria-current=\"true\" tabindex=\"-1\" aria-hidden=\"true\"><img src=\"{p}\" alt=\"\"></button></label><label for=\"{n}-i1\"><button type=\"button\" aria-label=\"View image 2\" aria-current=\"false\" tabindex=\"-1\" aria-hidden=\"true\"><img src=\"{p}\" alt=\"\"></button></label></div></div></div>"
            )
        );
        reject_interact(&html);
    }

    #[test]
    fn counter_counts_items_and_close_is_icon_only() {
        let html = render(&gallery(&["A", "B", "C"]));
        assert!(html.contains("<span>1</span><span>2</span><span>3</span> / 3</span>"));
        assert!(html.contains("data-slot=\"lightbox-close\" aria-label=\"Close\" tabindex=\"-1\" aria-hidden=\"true\"></button>"));
        assert_eq!(html.matches("aria-label=\"View image").count(), 3);
        assert_eq!(html.matches("type=\"radio\"").count(), 3);
        assert_eq!(html.matches(" checked").count(), 1);
    }

    /// Each image links to its neighbours only; edges have no link.
    #[test]
    fn stage_labels_point_at_neighbour_radios() {
        let html = render(&gallery(&["A", "B", "C"]));
        let n = "cui-lightbox-lightbox";
        assert!(html.contains(&format!(
            "alt=\"A\"><label for=\"{n}-i1\" data-nav=\"next\"></label><input"
        )));
        assert!(html.contains(&format!("alt=\"B\"><label for=\"{n}-i0\" data-nav=\"previous\"></label><label for=\"{n}-i2\" data-nav=\"next\"></label><input")));
        assert!(html.contains(&format!(
            "alt=\"C\"><label for=\"{n}-i1\" data-nav=\"previous\"></label><button"
        )));
    }

    #[test]
    fn two_lightboxes_on_one_page_get_distinct_groups() {
        reset_instance_ids();
        let a = render(&gallery(&["A", "B"]));
        let b = render(&gallery(&["A", "B"]));
        assert!(a.contains("name=\"cui-lightbox-lightbox\" "));
        assert!(b.contains("name=\"cui-lightbox-lightbox-2\" "));
        assert!(b.contains("<label for=\"cui-lightbox-lightbox-2-i1\""));
    }

    #[test]
    fn single_image_has_no_thumbnails_and_both_edges() {
        let html = render(&gallery(&["Only"]));
        assert!(html.contains("<span>1</span> / 1</span>"));
        assert!(!html.contains("lightbox-thumbnails"));
        assert!(!html.contains("data-nav"));
        assert!(html
            .contains("aria-label=\"Next image\" tabindex=\"-1\" aria-hidden=\"true\" data-edge"));
    }

    #[test]
    fn closed_mode_grid_of_thumbnails_opens_the_gallery_at_that_image() {
        let mut c = gallery(&["Mountain ridge at dawn", "Coastline from above"]);
        c.props.insert("open".into(), "false".into());
        c.items[1]
            .config
            .insert("src".into(), "/photos/1.jpg".into());
        let html = render(&c);
        let n = "cui-lightbox-lightbox";
        let p = PLACEHOLDER;
        assert!(html.starts_with(&format!(
            "<div class=\"grid\"><label for=\"{n}-i0\"><img src=\"/photos/1.jpg\" alt=\"Mountain ridge at dawn\"></label><label for=\"{n}-i1\"><img src=\"{p}\" alt=\"Coastline from above\"></label></div><div role=\"dialog\" aria-label=\"Image gallery\" data-slot=\"dialog-content\">"
        )), "{html}");
        assert!(!html.contains("dialog-overlay"));
        // No image starts selected; the close control is a radio of the same group.
        assert!(!html.contains(" checked"));
        assert!(html.contains(&format!(
            "<label><input type=\"radio\" name=\"{n}\" id=\"{n}-none\" aria-label=\"Close\"><button type=\"button\" data-slot=\"lightbox-close\""
        )));
        assert!(html.contains(&format!("<input type=\"radio\" name=\"{n}\" id=\"{n}-i0\" aria-label=\"Mountain ridge at dawn\"><img data-slot=\"lightbox-image\" src=\"/photos/1.jpg\"")));
        assert!(!html.contains("javascript:"));
        reject_interact(&html);
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"dialog-content\"]:has(> [data-slot=\"lightbox\"] > div:nth-child(2) > input):not(:has(> [data-slot=\"lightbox\"] > div:nth-child(2) > input:checked)) { display: none; }"));
        assert!(css.contains("div.grid:has(+ [data-slot=\"dialog-content\"] > [data-slot=\"lightbox\"]) {\n  display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 0.5rem;"));
        assert!(css.contains("[data-slot=\"catalog-frame\"]:has([data-slot=\"lightbox\"] > div:nth-child(2) > input:checked) { z-index: 30; overflow: visible; }"));
    }

    #[test]
    fn no_items_matches_react_empty_gallery() {
        let html = render(&stub("lightbox", "Image gallery"));
        assert!(html.contains(">0 / 0</span>"));
        assert!(!html.contains("lightbox-image"));
        assert!(!html.contains("lightbox-thumbnails"));
        assert!(!html.contains("type=\"radio\""));
        reject_interact(&html);
    }

    #[test]
    fn aria_label_prop_names_the_dialog() {
        let mut c = gallery(&["A"]);
        c.props.insert("aria-label".into(), "Photos".into());
        assert!(render(&c).contains("role=\"dialog\" aria-label=\"Photos\""));
    }

    #[test]
    fn skips_interact_dialog_surf() {
        let c = stub("lightbox", "Sunset over the bay");
        let html = render(&c);
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&gallery(&["A", "B"]));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"lightbox\""));
        });
    }

    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        let block = |sel: &str| {
            let start = css
                .find(&format!("{sel} {{"))
                .unwrap_or_else(|| panic!("{sel}"));
            let end = start + css[start..].find('}').unwrap();
            css[start..end].to_string()
        };
        let content = block("[data-slot=\"dialog-content\"]:has(> [data-slot=\"lightbox\"])");
        assert!(content.contains("position: fixed; inset: 0;"));
        assert!(content.contains("border: 0; border-radius: 0;"));
        assert!(content.contains("color-mix(in srgb, black 95%, transparent)"));
        let counter = block("[data-slot=\"lightbox-counter\"]");
        assert!(counter.contains("font-size: 0.875rem; line-height: 1.25rem;"));
        assert!(counter.contains("color-mix(in srgb, white 70%, transparent)"));
        let close = block("[data-slot=\"lightbox-close\"]");
        assert!(close.contains("width: 2rem; height: 2rem; padding: 0.375rem;"));
        let thumbs = block("[data-slot=\"lightbox-thumbnails\"]");
        assert!(thumbs.contains("gap: 0.5rem; overflow-x: auto; padding: 1rem;"));
        let thumb = block("[data-slot=\"lightbox-thumbnails\"] > label > button");
        assert!(thumb.contains("width: 3.5rem; height: 3.5rem;"));
        assert!(css.contains("var(--cronus-fg-tertiary)"));
        assert!(!css.contains("[data-slot=\"lightbox-caption\"]"));
        assert!(!css.contains("zinc-"));
    }

    /// The checked radio picks the image, the nav labels, the counter number
    /// and the thumbnail ring; the close checkbox hides overlay and content.
    #[test]
    fn chrome_follows_checked_state() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"lightbox\"] > div:nth-child(2) > input:not(:checked) + [data-slot=\"lightbox-image\"] { display: none; }"));
        assert!(css.contains("[data-slot=\"lightbox\"] > div:nth-child(2) > input:checked + [data-slot=\"lightbox-image\"] + label[data-nav],\n[data-slot=\"lightbox\"] > div:nth-child(2) > input:checked + [data-slot=\"lightbox-image\"] + label + label[data-nav] {\n  display: block;\n}"));
        assert!(css.contains("[data-slot=\"lightbox\"] > div:nth-child(2):has(> input:checked:not(:first-of-type)) > button:first-child { opacity: 1; }"));
        assert!(css.contains("[data-slot=\"lightbox\"] > div:nth-child(2):has(> input:checked:last-of-type) > button:last-child { opacity: 0.4; }"));
        assert!(css.contains("[data-slot=\"dialog-content\"]:has(> [data-slot=\"lightbox\"] > div:first-child > label > input:checked) { display: none; }"));
        assert!(css.contains("[data-slot=\"dialog-overlay\"]:has(+ [data-slot=\"dialog-content\"] > [data-slot=\"lightbox\"] > div:first-child > label > input:checked) { display: none; }"));
        for i in 1..=MAX_IMAGES {
            let has = format!(
                "[data-slot=\"lightbox\"]:has(> div:nth-child(2) > input:nth-of-type({i}):checked)"
            );
            assert!(
                css.contains(&format!(
                    "{has} [data-slot=\"lightbox-counter\"] > span:nth-child({i})"
                )),
                "counter {i}"
            );
            assert!(
                css.contains(&format!(
                    "{has} [data-slot=\"lightbox-thumbnails\"] > label:nth-child({i}) > button"
                )),
                "thumb {i}"
            );
        }
    }
}
