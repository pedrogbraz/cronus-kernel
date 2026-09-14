//! Dedicated Sonner renderer. Family name is `sonner`; React `<Toaster />`
//! renders `<div data-slot="toaster">` around Sonner's live-region
//! `<section aria-label="Notifications …" tabindex="-1" aria-live="polite">`,
//! which stays empty until `toast()` is called. The idle audit state has no
//! toasts, so the kernel emits the same empty toaster (0px tall). Queued toasts
//! need JS timers; they are not faked. Not interact `popover("sonner")`.

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = aria_label(comp).unwrap_or_else(|| label_of(comp));
    format!(
        "<div data-slot=\"toaster\"><section aria-label=\"{label}\" tabindex=\"-1\" aria-live=\"polite\" aria-relevant=\"additions text\" aria-atomic=\"false\"></section></div>"
    )
}

fn aria_label(comp: &ComponentNode) -> Option<String> {
    comp.props
        .get("aria-label")
        .or_else(|| comp.items.iter().find_map(|i| i.config.get("aria-label")))
        .filter(|s| !s.is_empty())
        .map(|s| esc(s))
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
        assert!(!html.contains("role=\"dialog\""));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
    }

    /// wave1t: React idle Toaster has no `sonner` slot and no toast; label
    /// text must not become visible content (it is the region's name).
    #[test]
    fn root_is_empty_toaster_region_like_react_idle() {
        let html = render(&stub("sonner", "Notifications"));
        assert_eq!(
            html,
            "<div data-slot=\"toaster\"><section aria-label=\"Notifications\" tabindex=\"-1\" aria-live=\"polite\" aria-relevant=\"additions text\" aria-atomic=\"false\"></section></div>"
        );
        assert!(!html.contains("data-slot=\"sonner\""));
        assert!(!html.contains("data-slot=\"toast\""));
        assert!(!html.contains(">Notifications<"));
        reject_interact(&html);
    }

    #[test]
    fn aria_label_from_item_config_wins_over_label() {
        let mut c = stub("sonner", "Saved");
        c.items[0]
            .config
            .insert("aria-label".into(), "Alerts & more".into());
        let html = render(&c);
        assert!(html.contains("<section aria-label=\"Alerts &amp; more\""));
        assert!(!html.contains(">Saved<"));
        reject_interact(&html);
    }

    #[test]
    fn extra_text_does_not_add_toasts() {
        let mut c = stub("sonner", "Saved");
        c.items.push(extra("text", "Undo"));
        let html = render(&c);
        assert!(!html.contains("Undo"));
        assert!(!html.contains("data-slot=\"toast\""));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_popover_surf() {
        let c = stub("sonner", "Saved");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("sonner", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<details data-slot=\"sonner\""));
        assert!(interact.contains("style="));
        assert!(!interact.contains("data-slot=\"toaster\""));
        assert!(html.contains("data-slot=\"toaster\""));
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
    fn chrome_toaster_is_in_flow_not_fixed() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"toaster\"] { display: block; }"));
        assert!(!css.contains("[data-slot=\"sonner\"] {"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
