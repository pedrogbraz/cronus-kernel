//! Dedicated Sonner renderer. Family name is `sonner`; React Toaster uses
//! `data-slot="toaster"`. Kernel emits both plus one sample toast from the
//! label: `<div data-slot="sonner"><div data-slot="toaster"><div data-slot="toast">`.
//! Not interact `popover("sonner")` (`<details>` SURF box).

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    format!(
        "<div data-slot=\"sonner\"><div data-slot=\"toaster\"><div data-slot=\"toast\" role=\"status\" aria-live=\"polite\">{label}</div></div></div>"
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
        assert!(!html.contains("<details"));
        assert!(!html.contains("<summary"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("position:absolute;z-index:20"));
        assert!(!html.contains("role=\"dialog\""));
        assert!(!html.contains("max-width:28rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("popover("));
    }

    #[test]
    fn root_is_sonner_toaster_toast_not_popover_details() {
        let html = render(&stub("sonner", "Saved"));
        assert!(html.starts_with("<div data-slot=\"sonner\">"));
        assert!(html.contains("<div data-slot=\"toaster\">"));
        assert!(html.contains(
            "<div data-slot=\"toast\" role=\"status\" aria-live=\"polite\">Saved</div>"
        ));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"sonner\"><div data-slot=\"toaster\"><div data-slot=\"toast\" role=\"status\" aria-live=\"polite\">Saved</div></div></div>"
        );
    }

    #[test]
    fn toast_text_from_label() {
        let html = render(&stub("sonner", "Copied to clipboard"));
        assert!(html.contains("data-slot=\"sonner\""));
        assert!(html.contains("data-slot=\"toaster\""));
        assert!(html.contains(">Copied to clipboard</div>"));
        assert_eq!(html.matches("data-slot=\"toast\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn extra_text_does_not_add_toasts() {
        let mut c = stub("sonner", "Saved");
        c.items.push(extra("text", "Undo"));
        let html = render(&c);
        assert!(html.contains(">Saved</div>"));
        assert!(!html.contains(">Undo</div>"));
        assert_eq!(html.matches("data-slot=\"toast\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_popover_surf() {
        let c = stub("sonner", "Saved");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("sonner", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<details data-slot=\"sonner\""));
        assert!(interact.contains("<summary"));
        assert!(interact.contains("style="));
        assert!(interact.contains("position:absolute;z-index:20"));
        assert!(!interact.contains("data-slot=\"toaster\""));
        assert!(!interact.contains("data-slot=\"toast\""));
        assert!(html.contains("data-slot=\"sonner\""));
        assert!(html.contains("data-slot=\"toaster\""));
        assert!(html.contains("data-slot=\"toast\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("sonner", "Saved"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"sonner\"]"));
        assert!(css.contains("[data-slot=\"toaster\"]"));
        assert!(css.contains("[data-slot=\"toast\"]"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("z-index: 50"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
