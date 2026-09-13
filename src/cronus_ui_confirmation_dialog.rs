//! Dedicated ConfirmationDialog renderer. Always-open static:
//! `<div data-slot="confirmation-dialog">` with title from the label plus
//! confirm button `confirmation-dialog-confirm`. Not interact
//! `dialog("confirmation-dialog")` native `<dialog>` + `showModal()` + SURF.

use crate::cronus_ui_kit::{esc, item, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let confirm = item(comp, "confirm")
        .or_else(|| item(comp, "action"))
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| "Confirm".into());
    let cancel = item(comp, "cancel")
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| "Cancel".into());
    let desc = extras(comp, &title);
    let desc_html = if desc.is_empty() {
        String::new()
    } else {
        format!("<div data-slot=\"confirmation-dialog-description\">{desc}</div>")
    };
    format!(
        "<div data-slot=\"confirmation-dialog\"><div data-slot=\"confirmation-dialog-title\">{title}</div>{desc_html}<button type=\"button\">{cancel}</button><button type=\"button\" data-slot=\"confirmation-dialog-confirm\">{confirm}</button></div>"
    )
}

fn extras(comp: &ComponentNode, title: &str) -> String {
    comp.items
        .iter()
        .filter(|i| !matches!(i.item_type.as_str(), "title" | "action" | "cancel" | "confirm"))
        .filter(|i| !i.text.is_empty())
        .map(|i| esc(&i.text))
        .filter(|t| t != title)
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
        assert!(!html.contains("-control"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("max-width:28rem"));
        assert!(!html.contains("role=\"dialog\""));
    }

    #[test]
    fn always_open_title_from_label_and_confirm() {
        let mut c = stub("confirmation-dialog", "Delete project");
        c.items.push(extra("text", "This cannot be undone."));
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"confirmation-dialog\">"));
        assert!(html.contains(
            "<div data-slot=\"confirmation-dialog-title\">Delete project</div>"
        ));
        assert!(html.contains(
            "<div data-slot=\"confirmation-dialog-description\">This cannot be undone.</div>"
        ));
        assert!(html.contains("<button type=\"button\">Cancel</button>"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"confirmation-dialog-confirm\">Confirm</button>"
        ));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"confirmation-dialog\"><div data-slot=\"confirmation-dialog-title\">Delete project</div><div data-slot=\"confirmation-dialog-description\">This cannot be undone.</div><button type=\"button\">Cancel</button><button type=\"button\" data-slot=\"confirmation-dialog-confirm\">Confirm</button></div>"
        );
    }

    #[test]
    fn confirm_and_cancel_items() {
        let mut c = stub("confirmation-dialog", "Delete project");
        c.items.push(extra("text", "This cannot be undone."));
        c.items.push(extra("cancel", "Keep"));
        c.items.push(extra("confirm", "Delete anyway"));
        let html = render(&c);
        assert!(html.contains("<button type=\"button\">Keep</button>"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"confirmation-dialog-confirm\">Delete anyway</button>"
        ));
        assert!(html.contains(
            "<div data-slot=\"confirmation-dialog-description\">This cannot be undone.</div>"
        ));
        assert!(!html.contains(">Keep</div>"));
        assert!(!html.contains(">Delete anyway</div>"));
        reject_interact(&html);
    }

    #[test]
    fn description_is_optional() {
        let html = render(&stub("confirmation-dialog", "Delete project"));
        assert!(html.contains(
            "<div data-slot=\"confirmation-dialog-title\">Delete project</div>"
        ));
        assert!(!html.contains("data-slot=\"confirmation-dialog-description\""));
        assert!(html.contains("data-slot=\"confirmation-dialog-confirm\">Confirm</button>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_dialog_surf() {
        let mut c = stub("confirmation-dialog", "Delete project");
        c.items.push(extra("text", "This cannot be undone."));
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("confirmation-dialog", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<dialog data-slot=\"confirmation-dialog-content\""));
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
            let html = render(&stub("confirmation-dialog", "Delete project"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"confirmation-dialog\""));
            assert!(html.contains("data-slot=\"confirmation-dialog-confirm\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"confirmation-dialog\"]"));
        assert!(css.contains("[data-slot=\"confirmation-dialog-title\"]"));
        assert!(css.contains("[data-slot=\"confirmation-dialog-description\"]"));
        assert!(css.contains("[data-slot=\"confirmation-dialog-confirm\"]"));
        assert!(css.contains("z-index: 50"));
        assert!(css.contains("max-width: 28rem"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains("showModal"));
    }
}
