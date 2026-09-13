//! Dedicated NotificationCenter renderer. Always-open static:
//! `<button data-slot="notification-trigger">` plus
//! `<div data-slot="notification-center">` listing `data-slot="notification-row"`
//! from texts. Not interact `popover("notification-center")` SURF `<details>`.

use crate::cronus_ui_kit::texts;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let rows = texts(comp)
        .into_iter()
        .map(|t| format!("<button type=\"button\" data-slot=\"notification-row\">{t}</button>"))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<button type=\"button\" data-slot=\"notification-trigger\">Notifications</button><div data-slot=\"notification-center\">{rows}</div>"
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
        assert!(!html.contains("<dialog"));
        assert!(!html.contains("showModal"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("max-width:28rem"));
        assert!(!html.contains("role=\"dialog\""));
        assert!(!html.contains("position:absolute;z-index:20"));
    }

    #[test]
    fn trigger_and_always_open_rows() {
        let mut c = stub("notification-center", "Deployed to production");
        c.items.push(extra("text", "Invite accepted"));
        let html = render(&c);
        assert!(html.starts_with(
            "<button type=\"button\" data-slot=\"notification-trigger\">Notifications</button>"
        ));
        assert!(html.contains("<div data-slot=\"notification-center\">"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"notification-row\">Deployed to production</button>"
        ));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"notification-row\">Invite accepted</button>"
        ));
        reject_interact(&html);
        assert_eq!(
            html,
            "<button type=\"button\" data-slot=\"notification-trigger\">Notifications</button><div data-slot=\"notification-center\"><button type=\"button\" data-slot=\"notification-row\">Deployed to production</button><button type=\"button\" data-slot=\"notification-row\">Invite accepted</button></div>"
        );
    }

    #[test]
    fn label_only_still_emits_one_row() {
        let html = render(&stub("notification-center", "Deployed to production"));
        assert!(html.contains("data-slot=\"notification-trigger\""));
        assert!(html.contains("data-slot=\"notification-center\""));
        assert_eq!(html.matches("data-slot=\"notification-row\"").count(), 1);
        assert!(html.contains(">Deployed to production</button>"));
        reject_interact(&html);
    }

    #[test]
    fn extra_texts_become_rows() {
        let mut c = stub("notification-center", "Deployed to production");
        c.items.push(extra("text", "Invite accepted"));
        c.items.push(extra("text", "Payment failed"));
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"notification-row\"").count(), 3);
        assert!(html.contains(">Payment failed</button>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_popover_surf() {
        let mut c = stub("notification-center", "Deployed to production");
        c.items.push(extra("text", "Invite accepted"));
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("notification-center", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<details data-slot=\"notification-center\""));
        assert!(interact.contains("<summary"));
        assert!(interact.contains("style="));
        assert!(interact.contains("position:absolute;z-index:20"));
        assert!(!interact.contains("data-slot=\"notification-trigger\""));
        assert!(!interact.contains("data-slot=\"notification-row\""));
        assert!(!html.contains("<details"));
        assert!(!html.contains("showModal"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("notification-center", "Deployed to production"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"notification-center\""));
            assert!(html.contains("data-slot=\"notification-row\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"notification-trigger\"]"));
        assert!(css.contains("[data-slot=\"notification-center\"]"));
        assert!(css.contains("[data-slot=\"notification-row\"]"));
        assert!(css.contains("z-index: 50"));
        assert!(css.contains("width: 20rem"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains("showModal"));
    }
}
