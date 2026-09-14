//! Dedicated Lightbox renderer (wave1t geometry parity with the React Lightbox).
//!
//! React renders an open, full-viewport Radix Dialog portaled to `<body>`:
//! `dialog-overlay` + `dialog-content` (fixed inset-0, bg black/95, white text)
//! wrapping `lightbox` > header (`lightbox-counter` + icon-only `lightbox-close`),
//! stage (prev button, `img lightbox-image`, next button) and, with more than one
//! image, `lightbox-thumbnails` (one 56px button per image). No caption without
//! a caption prop.
//! The kernel renders the same tree at index 0, zero JS. Close / previous / next /
//! thumbnail selection need JS, so they are the same native `<button>`s with
//! `disabled`; only the ones React itself disables (`data-edge`: previous at the
//! first image, next at the last) are dimmed. Images are the `text` items (alts);
//! `.cronus` carries no image source, so each item gets a transparent 64x64 SVG
//! placeholder painted with a token (inset box-shadow, background stays clear).

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

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
    let current = if count > 0 { 1 } else { 0 };
    let image = images
        .first()
        .map(|alt| {
            format!("<img data-slot=\"lightbox-image\" src=\"{PLACEHOLDER}\" alt=\"{alt}\">")
        })
        .unwrap_or_default();
    let prev_edge = " data-edge";
    let next_edge = if count <= 1 { " data-edge" } else { "" };
    let thumbnails = if count > 1 {
        let buttons = (1..=count)
            .map(|i| {
                format!(
                    "<button type=\"button\" aria-label=\"View image {i}\" aria-current=\"{}\" disabled><img src=\"{PLACEHOLDER}\" alt=\"\"></button>",
                    i == 1
                )
            })
            .collect::<String>();
        format!("<div data-slot=\"lightbox-thumbnails\">{buttons}</div>")
    } else {
        String::new()
    };
    format!(
        "<div data-slot=\"dialog-overlay\" aria-hidden=\"true\"></div><div role=\"dialog\" aria-label=\"{gallery}\" data-slot=\"dialog-content\"><div data-slot=\"lightbox\"><div><span data-slot=\"lightbox-counter\">{current} / {count}</span><button type=\"button\" data-slot=\"lightbox-close\" aria-label=\"Close\" disabled></button></div><div><button type=\"button\" aria-label=\"Previous image\"{prev_edge} disabled></button>{image}<button type=\"button\" aria-label=\"Next image\"{next_edge} disabled></button></div>{thumbnails}</div></div>"
    )
}

fn images(comp: &ComponentNode) -> Vec<String> {
    comp.items
        .iter()
        .filter(|i| i.item_type == "text" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
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
    }

    #[test]
    fn react_dom_two_images() {
        let html = render(&gallery(&["First image", "Second image"]));
        let p = PLACEHOLDER;
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"dialog-overlay\" aria-hidden=\"true\"></div><div role=\"dialog\" aria-label=\"Image gallery\" data-slot=\"dialog-content\"><div data-slot=\"lightbox\"><div><span data-slot=\"lightbox-counter\">1 / 2</span><button type=\"button\" data-slot=\"lightbox-close\" aria-label=\"Close\" disabled></button></div><div><button type=\"button\" aria-label=\"Previous image\" data-edge disabled></button><img data-slot=\"lightbox-image\" src=\"{p}\" alt=\"First image\"><button type=\"button\" aria-label=\"Next image\" disabled></button></div><div data-slot=\"lightbox-thumbnails\"><button type=\"button\" aria-label=\"View image 1\" aria-current=\"true\" disabled><img src=\"{p}\" alt=\"\"></button><button type=\"button\" aria-label=\"View image 2\" aria-current=\"false\" disabled><img src=\"{p}\" alt=\"\"></button></div></div></div>"
            )
        );
        reject_interact(&html);
    }

    #[test]
    fn counter_counts_items_and_close_is_icon_only() {
        let html = render(&gallery(&["A", "B", "C"]));
        assert!(html.contains(">1 / 3</span>"));
        assert!(html.contains("aria-label=\"Close\" disabled></button>"));
        assert_eq!(html.matches("aria-label=\"View image").count(), 3);
    }

    #[test]
    fn single_image_has_no_thumbnails_and_both_edges() {
        let html = render(&gallery(&["Only"]));
        assert!(html.contains(">1 / 1</span>"));
        assert!(!html.contains("lightbox-thumbnails"));
        assert!(html.contains("aria-label=\"Next image\" data-edge disabled"));
    }

    #[test]
    fn no_items_matches_react_empty_gallery() {
        let html = render(&stub("lightbox", "Image gallery"));
        assert!(html.contains(">0 / 0</span>"));
        assert!(!html.contains("lightbox-image"));
        assert!(!html.contains("lightbox-thumbnails"));
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
        let interact = crate::cronus_ui_interact::render("lightbox", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<dialog data-slot=\"lightbox-content\""));
        assert!(interact.contains("showModal()"));
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
        let thumb = block("[data-slot=\"lightbox-thumbnails\"] > button");
        assert!(thumb.contains("width: 3.5rem; height: 3.5rem;"));
        assert!(css.contains("var(--cronus-fg-tertiary)"));
        assert!(!css.contains("[data-slot=\"lightbox-caption\"]"));
        assert!(!css.contains("zinc-"));
    }
}
