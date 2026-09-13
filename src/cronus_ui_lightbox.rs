//! Dedicated Lightbox renderer. Always-open static: `<div data-slot="lightbox">`
//! with caption from the label plus optional counter/close. No JS. Not
//! interact `dialog("lightbox")` native `<dialog>` + `showModal()` + SURF.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let caption = label_of(comp);
    format!(
        "<div data-slot=\"lightbox\"><span data-slot=\"lightbox-counter\">1 / 1</span><button type=\"button\" data-slot=\"lightbox-close\">Close</button><p data-slot=\"lightbox-caption\">{caption}</p></div>"
    )
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

    fn reject_interact(html: &str) {
        assert!(!html.contains("<dialog"));
        assert!(!html.contains("showModal"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("max-width:28rem"));
        assert!(!html.contains("role=\"dialog\""));
    }

    #[test]
    fn always_open_caption_from_label() {
        let html = render(&stub("lightbox", "Sunset over the bay"));
        assert!(html.starts_with("<div data-slot=\"lightbox\">"));
        assert!(html.contains("<p data-slot=\"lightbox-caption\">Sunset over the bay</p>"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"lightbox\"><span data-slot=\"lightbox-counter\">1 / 1</span><button type=\"button\" data-slot=\"lightbox-close\">Close</button><p data-slot=\"lightbox-caption\">Sunset over the bay</p></div>"
        );
    }

    #[test]
    fn optional_counter_and_close() {
        let html = render(&stub("lightbox", "Sunset over the bay"));
        assert!(html.contains("<span data-slot=\"lightbox-counter\">1 / 1</span>"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"lightbox-close\">Close</button>"
        ));
        assert!(!html.contains("lightbox-image"));
        reject_interact(&html);
    }

    #[test]
    fn extra_text_does_not_change_caption() {
        let mut c = stub("lightbox", "Sunset over the bay");
        c.items.push(extra("text", "Next slide"));
        let html = render(&c);
        assert!(html.contains("<p data-slot=\"lightbox-caption\">Sunset over the bay</p>"));
        assert!(!html.contains("Next slide"));
        assert_eq!(html.matches("data-slot=\"lightbox-caption\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_dialog_surf() {
        let c = stub("lightbox", "Sunset over the bay");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("lightbox", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<dialog data-slot=\"lightbox-content\""));
        assert!(interact.contains("showModal()"));
        assert!(interact.contains("onclick="));
        assert!(interact.contains("style="));
        assert!(interact.contains("max-width:28rem"));
        assert!(!html.contains("<dialog"));
        assert!(!html.contains("showModal"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("lightbox", "Sunset over the bay"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"lightbox\""));
            assert!(html.contains("data-slot=\"lightbox-caption\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"lightbox\"]"));
        assert!(css.contains("[data-slot=\"lightbox-caption\"]"));
        assert!(css.contains("[data-slot=\"lightbox-counter\"]"));
        assert!(css.contains("[data-slot=\"lightbox-close\"]"));
        assert!(css.contains("z-index: 50"));
        assert!(css.contains("var(--cronus-surface-base)"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains("showModal"));
    }
}
