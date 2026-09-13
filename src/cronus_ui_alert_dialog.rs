//! Dedicated AlertDialog renderer. Always-open static: trigger button (label)
//! plus `<div data-slot="alert-dialog-content">` with `alert-dialog-title`,
//! optional description, and action/cancel buttons. Wrapper
//! `data-slot="alert-dialog"` for the family slot. Not interact
//! `dialog("alert-dialog")` native `<dialog>` + `showModal()` + SURF.

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
    let action = item(comp, "action")
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| "Continue".into());
    let cancel = item(comp, "cancel")
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| "Cancel".into());
    let desc = extras(comp, &trigger, &title);
    let desc_html = if desc.is_empty() {
        String::new()
    } else {
        format!("<div data-slot=\"alert-dialog-description\">{desc}</div>")
    };
    format!(
        "<div data-slot=\"alert-dialog\"><button type=\"button\">{trigger}</button><div data-slot=\"alert-dialog-content\"><div data-slot=\"alert-dialog-title\">{title}</div>{desc_html}<button type=\"button\" data-slot=\"alert-dialog-cancel\">{cancel}</button><button type=\"button\" data-slot=\"alert-dialog-action\">{action}</button></div></div>"
    )
}

fn extras(comp: &ComponentNode, trigger: &str, title: &str) -> String {
    comp.items
        .iter()
        .filter(|i| !matches!(i.item_type.as_str(), "title" | "action" | "cancel"))
        .filter(|i| !i.text.is_empty())
        .map(|i| esc(&i.text))
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
        let mut c = stub("alert-dialog", "Delete");
        c.items.push(extra("text", "This cannot be undone."));
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"alert-dialog\">"));
        assert!(html.contains("<button type=\"button\">Delete</button>"));
        assert!(html.contains("<div data-slot=\"alert-dialog-content\">"));
        assert!(html.contains("<div data-slot=\"alert-dialog-title\">Delete</div>"));
        assert!(html.contains(
            "<div data-slot=\"alert-dialog-description\">This cannot be undone.</div>"
        ));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"alert-dialog-cancel\">Cancel</button>"
        ));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"alert-dialog-action\">Continue</button>"
        ));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"alert-dialog\"><button type=\"button\">Delete</button><div data-slot=\"alert-dialog-content\"><div data-slot=\"alert-dialog-title\">Delete</div><div data-slot=\"alert-dialog-description\">This cannot be undone.</div><button type=\"button\" data-slot=\"alert-dialog-cancel\">Cancel</button><button type=\"button\" data-slot=\"alert-dialog-action\">Continue</button></div></div>"
        );
    }

    #[test]
    fn title_item_is_panel_title() {
        let mut c = stub("alert-dialog", "Open");
        c.items.push(extra("title", "Delete project"));
        c.items.push(extra("text", "This cannot be undone."));
        let html = render(&c);
        assert!(html.contains("<button type=\"button\">Open</button>"));
        assert!(html.contains("<div data-slot=\"alert-dialog-title\">Delete project</div>"));
        assert!(html.contains(
            "<div data-slot=\"alert-dialog-description\">This cannot be undone.</div>"
        ));
        reject_interact(&html);
    }

    #[test]
    fn action_and_cancel_items() {
        let mut c = stub("alert-dialog", "Delete");
        c.items.push(extra("text", "This cannot be undone."));
        c.items.push(extra("cancel", "Keep"));
        c.items.push(extra("action", "Delete anyway"));
        let html = render(&c);
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"alert-dialog-cancel\">Keep</button>"
        ));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"alert-dialog-action\">Delete anyway</button>"
        ));
        assert!(html.contains(
            "<div data-slot=\"alert-dialog-description\">This cannot be undone.</div>"
        ));
        assert!(!html.contains(">Keep</div>"));
        assert!(!html.contains(">Delete anyway</div>"));
        reject_interact(&html);
    }

    #[test]
    fn description_is_optional() {
        let html = render(&stub("alert-dialog", "Delete"));
        assert!(html.contains("<button type=\"button\">Delete</button>"));
        assert!(html.contains("<div data-slot=\"alert-dialog-title\">Delete</div>"));
        assert!(!html.contains("data-slot=\"alert-dialog-description\""));
        assert!(html.contains("data-slot=\"alert-dialog-cancel\""));
        assert!(html.contains("data-slot=\"alert-dialog-action\""));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_dialog_surf() {
        let mut c = stub("alert-dialog", "Delete");
        c.items.push(extra("text", "This cannot be undone."));
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("alert-dialog", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<dialog data-slot=\"alert-dialog-content\""));
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
            let html = render(&stub("alert-dialog", "Delete"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"alert-dialog-content\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"alert-dialog\"]"));
        assert!(css.contains("[data-slot=\"alert-dialog-content\"]"));
        assert!(css.contains("[data-slot=\"alert-dialog-title\"]"));
        assert!(css.contains("[data-slot=\"alert-dialog-description\"]"));
        assert!(css.contains("[data-slot=\"alert-dialog-action\"]"));
        assert!(css.contains("[data-slot=\"alert-dialog-cancel\"]"));
        assert!(css.contains("z-index: 50"));
        assert!(css.contains("max-width: 32rem"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains("showModal"));
    }
}
