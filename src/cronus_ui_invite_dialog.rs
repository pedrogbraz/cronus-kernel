//! Dedicated InviteDialog renderer. Always-open static:
//! `<div data-slot="invite-dialog">` plus an email field and send button
//! `invite-dialog-send`. Not interact `dialog("invite-dialog")` native
//! `<dialog>` + `showModal()` + SURF.

use crate::cronus_ui_kit::{esc, item, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let send = item(comp, "send")
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| "Send invite".into());
    let cancel = item(comp, "cancel")
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| "Cancel".into());
    let placeholder = item(comp, "placeholder")
        .or_else(|| comp.props.get("placeholder").map(String::as_str))
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| "name@example.com".into());
    let desc = extras(comp, &title);
    let desc_html = if desc.is_empty() {
        String::new()
    } else {
        format!("<div data-slot=\"invite-dialog-description\">{desc}</div>")
    };
    format!(
        "<div data-slot=\"invite-dialog\"><div data-slot=\"invite-dialog-title\">{title}</div>{desc_html}<label>Email<input type=\"email\" name=\"email\" placeholder=\"{placeholder}\" autocomplete=\"email\" required /></label><button type=\"button\">{cancel}</button><button type=\"button\" data-slot=\"invite-dialog-send\">{send}</button></div>"
    )
}

fn extras(comp: &ComponentNode, title: &str) -> String {
    comp.items
        .iter()
        .filter(|i| {
            !matches!(
                i.item_type.as_str(),
                "title" | "send" | "cancel" | "confirm" | "email" | "placeholder"
            )
        })
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
        assert!(!html.contains("<label data-slot=\"invite-dialog\""));
    }

    #[test]
    fn always_open_email_field_and_send() {
        let html = render(&stub("invite-dialog", "Invite member"));
        assert!(html.starts_with("<div data-slot=\"invite-dialog\">"));
        assert!(html.contains("<div data-slot=\"invite-dialog-title\">Invite member</div>"));
        assert!(html.contains("<label>Email<input type=\"email\" name=\"email\" placeholder=\"name@example.com\" autocomplete=\"email\" required /></label>"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"invite-dialog-send\">Send invite</button>"
        ));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"invite-dialog\"><div data-slot=\"invite-dialog-title\">Invite member</div><label>Email<input type=\"email\" name=\"email\" placeholder=\"name@example.com\" autocomplete=\"email\" required /></label><button type=\"button\">Cancel</button><button type=\"button\" data-slot=\"invite-dialog-send\">Send invite</button></div>"
        );
    }

    #[test]
    fn description_and_send_override() {
        let mut c = stub("invite-dialog", "Invite member");
        c.items
            .push(extra("text", "Send an invitation to join this workspace."));
        c.items.push(extra("send", "Invite"));
        c.items.push(extra("cancel", "Close"));
        let html = render(&c);
        assert!(html.contains(
            "<div data-slot=\"invite-dialog-description\">Send an invitation to join this workspace.</div>"
        ));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"invite-dialog-send\">Invite</button>"
        ));
        assert!(html.contains("<button type=\"button\">Close</button>"));
        assert!(!html.contains(">Invite</div>"));
        assert!(!html.contains(">Close</div>"));
        reject_interact(&html);
    }

    #[test]
    fn placeholder_from_item() {
        let mut c = stub("invite-dialog", "Invite member");
        c.items
            .push(extra("placeholder", "teammate@company.com"));
        let html = render(&c);
        assert!(html.contains("placeholder=\"teammate@company.com\""));
        assert!(!html.contains("teammate@company.com</div>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_dialog_surf() {
        let c = stub("invite-dialog", "Invite member");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("invite-dialog", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<dialog data-slot=\"invite-dialog-content\""));
        assert!(interact.contains("showModal()"));
        assert!(interact.contains("onclick="));
        assert!(interact.contains("style="));
        assert!(interact.contains("max-width:28rem"));
        assert!(!html.contains("<dialog"));
        assert!(!html.contains("showModal"));
        assert!(html.contains("type=\"email\""));
        assert!(html.contains("data-slot=\"invite-dialog-send\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("invite-dialog", "Invite member"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"invite-dialog\""));
            assert!(html.contains("data-slot=\"invite-dialog-send\""));
            assert!(html.contains("type=\"email\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"invite-dialog\"]"));
        assert!(css.contains("[data-slot=\"invite-dialog-title\"]"));
        assert!(css.contains("[data-slot=\"invite-dialog-description\"]"));
        assert!(css.contains("[data-slot=\"invite-dialog-send\"]"));
        assert!(css.contains("z-index: 50"));
        assert!(css.contains("max-width: 28rem"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains("showModal"));
    }
}
