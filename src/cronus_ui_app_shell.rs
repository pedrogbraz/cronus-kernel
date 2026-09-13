//! Dedicated AppShell renderer. DOM matches React composition:
//! `<div data-slot="app-shell">` wrapping `app-shell-content` with
//! `app-shell-header` (title from label) and `app-shell-body`.
//! Not interact `nav("app-shell")` (generic SURF `<nav>` without header/body/content).

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let body = body_inner(comp);
    format!(
        "<div data-slot=\"app-shell\"><div data-slot=\"app-shell-content\"><header data-slot=\"app-shell-header\">{title}</header><div data-slot=\"app-shell-body\">{body}</div></div></div>"
    )
}

fn body_inner(comp: &ComponentNode) -> String {
    comp.items
        .iter()
        .filter(|i| {
            !matches!(i.item_type.as_str(), "label" | "title") && !i.text.is_empty()
        })
        .map(|i| esc(&i.text))
        .collect::<Vec<_>>()
        .join("")
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
        assert!(!html.contains("<nav data-slot=\"app-shell\""));
        assert!(!html.starts_with("<nav"));
        assert!(!html.contains("<details"));
        assert!(!html.contains("<summary"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("nav("));
    }

    #[test]
    fn root_is_wrapper_with_header_body_content_not_nav_surf() {
        let html = render(&stub("app-shell", "Dashboard"));
        assert!(html.starts_with("<div data-slot=\"app-shell\">"));
        assert!(html.contains("<div data-slot=\"app-shell-content\">"));
        assert!(html.contains("<header data-slot=\"app-shell-header\">Dashboard</header>"));
        assert!(html.contains("<div data-slot=\"app-shell-body\"></div>"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"app-shell\"><div data-slot=\"app-shell-content\"><header data-slot=\"app-shell-header\">Dashboard</header><div data-slot=\"app-shell-body\"></div></div></div>"
        );
    }

    #[test]
    fn extra_text_items_go_in_body() {
        let mut c = stub("app-shell", "Dashboard");
        c.items.push(extra("text", "Inbox"));
        c.items.push(extra("text", "Settings"));
        let html = render(&c);
        assert!(html.contains("<header data-slot=\"app-shell-header\">Dashboard</header>"));
        assert!(html.contains("<div data-slot=\"app-shell-body\">InboxSettings</div>"));
        reject_interact(&html);
    }

    #[test]
    fn field_label_stays_in_header_when_items_exist() {
        let mut c = stub("app-shell", "App");
        c.items.push(extra("item", "Main"));
        let html = render(&c);
        assert!(html.contains("<header data-slot=\"app-shell-header\">App</header>"));
        assert!(html.contains("<div data-slot=\"app-shell-body\">Main</div>"));
        assert!(!html.contains("<div data-slot=\"app-shell-body\">App"));
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_header() {
        let html = render(&stub("app-shell", "Dashboard"));
        assert!(html.contains("<header data-slot=\"app-shell-header\">Dashboard</header>"));
        assert!(html.contains("<div data-slot=\"app-shell-body\"></div>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_nav_surf() {
        let c = stub("app-shell", "Dashboard");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("app-shell", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<nav data-slot=\"app-shell\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!interact.contains("data-slot=\"app-shell-header\""));
        assert!(!interact.contains("data-slot=\"app-shell-body\""));
        assert!(!interact.contains("data-slot=\"app-shell-content\""));
        assert!(html.contains("data-slot=\"app-shell-header\""));
        assert!(html.contains("data-slot=\"app-shell-body\""));
        assert!(html.contains("data-slot=\"app-shell-content\""));
        assert!(html.starts_with("<div data-slot=\"app-shell\">"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("app-shell", "Dashboard"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"app-shell\"]"));
        assert!(css.contains("[data-slot=\"app-shell-header\"]"));
        assert!(css.contains("[data-slot=\"app-shell-body\"]"));
        assert!(css.contains("[data-slot=\"app-shell-content\"]"));
        assert!(css.contains("min-height: 100svh"));
        assert!(css.contains("height: 3.5rem"));
        assert!(css.contains("var(--cronus-surface-base)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
