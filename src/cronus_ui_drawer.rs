//! Dedicated Drawer renderer. Always-open static: trigger button (label) plus
//! `<div data-slot="drawer-content">` with `drawer-title` / `drawer-description`.
//! React Root has `data-slot="drawer"` — wrapper is allowed.
//! Not interact `dialog("drawer")` native `<dialog>` + `showModal()` + SURF.

use crate::cronus_ui_kit::{esc, item, label_of, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let trigger = texts(comp)
        .first()
        .cloned()
        .unwrap_or_else(|| label_of(comp));
    let title = item(comp, "title")
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| trigger.clone());
    let desc = extras(comp, &trigger, &title);
    format!(
        "<div data-slot=\"drawer\"><button type=\"button\">{trigger}</button><div data-slot=\"drawer-content\"><div data-slot=\"drawer-title\">{title}</div><div data-slot=\"drawer-description\">{desc}</div></div></div>"
    )
}

fn extras(comp: &ComponentNode, trigger: &str, title: &str) -> String {
    texts(comp)
        .into_iter()
        .filter(|t| t != trigger && t != title)
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
    fn trigger_and_always_open_content() {
        let mut c = stub("drawer", "Menu");
        c.items.push(extra("text", "Slide-up panel."));
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"drawer\">"));
        assert!(html.contains("<button type=\"button\">Menu</button>"));
        assert!(html.contains("<div data-slot=\"drawer-content\">"));
        assert!(html.contains("<div data-slot=\"drawer-title\">Menu</div>"));
        assert!(html.contains("<div data-slot=\"drawer-description\">Slide-up panel.</div>"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"drawer\"><button type=\"button\">Menu</button><div data-slot=\"drawer-content\"><div data-slot=\"drawer-title\">Menu</div><div data-slot=\"drawer-description\">Slide-up panel.</div></div></div>"
        );
    }

    #[test]
    fn title_item_is_panel_title() {
        let mut c = stub("drawer", "Open");
        c.items.push(extra("title", "Filters"));
        c.items.push(extra("text", "Narrow the list."));
        let html = render(&c);
        assert!(html.contains("<button type=\"button\">Open</button>"));
        assert!(html.contains("<div data-slot=\"drawer-title\">Filters</div>"));
        assert!(html.contains("<div data-slot=\"drawer-description\">Narrow the list.</div>"));
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_opens_empty_description() {
        let html = render(&stub("drawer", "Menu"));
        assert!(html.contains("<button type=\"button\">Menu</button>"));
        assert!(html.contains("<div data-slot=\"drawer-title\">Menu</div>"));
        assert!(html.contains("<div data-slot=\"drawer-description\"></div>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_dialog_surf() {
        let mut c = stub("drawer", "Menu");
        c.items.push(extra("text", "Slide-up panel."));
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("drawer", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<dialog data-slot=\"drawer-content\""));
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
            let html = render(&stub("drawer", "Menu"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"drawer-content\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"drawer\"]"));
        assert!(css.contains("[data-slot=\"drawer-content\"]"));
        assert!(css.contains("[data-slot=\"drawer-title\"]"));
        assert!(css.contains("[data-slot=\"drawer-description\"]"));
        assert!(css.contains("z-index: 50"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains("showModal"));
    }
}
