//! Dedicated InviteDialog renderer — mirrors React `InviteDialog` (a
//! `DialogContent` with `data-slot="invite-dialog"`) in its default form state.
//!
//! Open by default, zero JS, no native `<dialog>` (React's portal renders plain
//! divs): the fixed `dialog-overlay` scrim and the fixed, centred
//! `invite-dialog` panel (`role="dialog"`):
//! `dialog-header` (`dialog-title` h2 + `dialog-description` p), a
//! `<form>` with two `field`s (Email `input`; Role
//! `select-trigger`), `dialog-footer` (outline Cancel + primary
//! `invite-dialog-send`) and the absolute `dialog-close` icon button.
//!
//! JS-only (Wave 1t rule: same native element, `disabled`, idle look): Send,
//! Cancel and Close (sending/closing need JS) and the Radix role select — its
//! trigger is a disabled `button` showing the default role, and a hidden native
//! `select` carries that value. The email input stays editable. Not reproduced either: focus
//! trap, async `onInvite` spinner/error, invite-link success view.
//!
//! Closed mode (`trigger:"…"` prop, or `open:false` / `defaultOpen:false`; see
//! `cronus_ui_kit::overlay_trigger`): an outline `button` trigger opens the panel
//! as a native `popover="auto"` (scrim on `::backdrop`, no overlay div); Cancel
//! and Close hide it (`popovertargetaction="hide"`). Send and the role select
//! still need JS and stay `disabled`. Gaps: a popover is not modal (no focus
//! trap, background not inert, `aria-expanded` not reflected).

use crate::cronus_ui_kit::{
    attr, esc, item, label_of, modal_close_attrs, modal_dialog_open, modal_open_button,
    overlay_trigger, widget_id,
};
use crate::parser::ComponentNode;

/// React `DEFAULT_ROLES`; the first is initially selected.
const ROLES: [(&str, &str); 2] = [("member", "Member"), ("admin", "Admin")];

const CHEVRON: &str = "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m6 9 6 6 6-6\"/></svg>";
const CROSS: &str = "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M18 6 6 18\"/><path d=\"m6 6 12 12\"/></svg>";

fn label(comp: &ComponentNode, kind: &str, fallback: &str) -> String {
    item(comp, kind)
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| fallback.into())
}

pub fn render(comp: &ComponentNode) -> String {
    let title = label_of(comp);
    // `description:"…"` prop, attr-style item config, a `description` item, else React's default.
    let description = attr(comp, "description")
        .or_else(|| item(comp, "description"))
        .filter(|d| !d.is_empty())
        .map(esc)
        .unwrap_or_else(|| "Send an invitation to join this workspace.".into());
    let email = label(comp, "email", "Email");
    let placeholder = item(comp, "placeholder")
        .or_else(|| comp.props.get("placeholder").map(String::as_str))
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| "name@example.com".into());
    let role = label(comp, "role", "Role");
    let send = label(comp, "send", "Send invite");
    let cancel = label(comp, "cancel", "Cancel");
    let close = label(comp, "close", "Close");
    let role_text = ROLES[0].1;
    // Radix Select's visually hidden native <select> (form value; aria-hidden, tabindex -1).
    let options: String = ROLES
        .iter()
        .enumerate()
        .map(|(i, (value, text))| {
            let selected = if i == 0 { " selected" } else { "" };
            format!("<option value=\"{value}\"{selected}>{text}</option>")
        })
        .collect();

    let form = widget_id(comp, "invite-dialog-form");
    let title_id = widget_id(comp, "invite-dialog-title");
    let email_id = widget_id(comp, "invite-dialog-email");
    let role_id = widget_id(comp, "invite-dialog-role");

    let trigger = overlay_trigger(comp, "Open");
    let pop_id = widget_id(comp, "invite-dialog");
    // Cancel / Close: inert in the open specimen, native popover hide when closed.
    let dismiss = if trigger.is_some() {
        modal_close_attrs(&pop_id)
    } else {
        " disabled".to_string()
    };
    let body = format!(
        "<div data-slot=\"dialog-header\"><h2 data-slot=\"dialog-title\" id=\"{title_id}\">{title}</h2><p data-slot=\"dialog-description\">{description}</p></div><form id=\"{form}\"><div data-slot=\"field\"><label data-slot=\"field-label\" for=\"{email_id}\">{email}</label><input data-slot=\"input\" id=\"{email_id}\" type=\"email\" name=\"email\" autocomplete=\"email\" required placeholder=\"{placeholder}\"></div><div data-slot=\"field\"><label data-slot=\"field-label\" for=\"{role_id}\">{role}</label><button type=\"button\" data-slot=\"select-trigger\" id=\"{role_id}\" role=\"combobox\" aria-expanded=\"false\" disabled><span>{role_text}</span>{CHEVRON}</button><select aria-hidden=\"true\" tabindex=\"-1\" name=\"role\">{options}</select></div><div data-slot=\"dialog-footer\"><button type=\"button\" data-slot=\"button\" data-variant=\"outline\"{dismiss}>{cancel}</button><button type=\"button\" data-slot=\"invite-dialog-send\" data-variant=\"primary\" disabled>{send}</button></div></form><button type=\"button\" data-slot=\"dialog-close\"{dismiss}>{CROSS}<span>{close}</span></button>"
    );
    match trigger {
        None => format!(
            "<div data-slot=\"dialog-overlay\" data-state=\"open\" aria-hidden=\"true\"></div><div data-slot=\"invite-dialog\" data-state=\"open\" role=\"dialog\" aria-labelledby=\"{title_id}\">{body}</div>"
        ),
        Some(trigger) => {
            let trigger_id = widget_id(comp, "invite-dialog-trigger");
            format!(
                "{}{}{body}</dialog>",
                modal_open_button(&trigger_id, &pop_id, &trigger),
                modal_dialog_open(&pop_id, "invite-dialog", "dialog", &title_id, "", true),
            )
        }
    }
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

    fn reject_js(html: &str) {
        assert!(!html.contains("showModal("));
        assert!(!html.contains("-control"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("popovertarget"));
        assert!(!html.contains("<label data-slot=\"invite-dialog\""));
    }

    #[test]
    fn fixture_matches_react_dom() {
        let c = stub("invite-dialog", "Invite member");
        let id = widget_id(&c, "invite-dialog-");
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"dialog-overlay\" data-state=\"open\" aria-hidden=\"true\"></div><div data-slot=\"invite-dialog\" data-state=\"open\" role=\"dialog\" aria-labelledby=\"{id}title\"><div data-slot=\"dialog-header\"><h2 data-slot=\"dialog-title\" id=\"{id}title\">Invite member</h2><p data-slot=\"dialog-description\">Send an invitation to join this workspace.</p></div><form id=\"{id}form\"><div data-slot=\"field\"><label data-slot=\"field-label\" for=\"{id}email\">Email</label><input data-slot=\"input\" id=\"{id}email\" type=\"email\" name=\"email\" autocomplete=\"email\" required placeholder=\"name@example.com\"></div><div data-slot=\"field\"><label data-slot=\"field-label\" for=\"{id}role\">Role</label><button type=\"button\" data-slot=\"select-trigger\" id=\"{id}role\" role=\"combobox\" aria-expanded=\"false\" disabled><span>Member</span>{CHEVRON}</button><select aria-hidden=\"true\" tabindex=\"-1\" name=\"role\"><option value=\"member\" selected>Member</option><option value=\"admin\">Admin</option></select></div><div data-slot=\"dialog-footer\"><button type=\"button\" data-slot=\"button\" data-variant=\"outline\" disabled>Cancel</button><button type=\"button\" data-slot=\"invite-dialog-send\" data-variant=\"primary\" disabled>Send invite</button></div></form><button type=\"button\" data-slot=\"dialog-close\" disabled>{CROSS}<span>Close</span></button></div>"
            )
        );
        assert!(!html.contains("data-slot=\"invite-dialog-title\""));
        reject_js(&html);
    }

    #[test]
    fn label_items_override_defaults() {
        let mut c = stub("invite-dialog", "Invite member");
        c.items.push(extra("placeholder", "teammate@company.com"));
        c.items.push(extra("send", "Invite"));
        c.items.push(extra("cancel", "Dismiss"));
        c.props
            .insert("description".into(), "Add a teammate.".into());
        let html = render(&c);
        assert!(html.contains("<p data-slot=\"dialog-description\">Add a teammate.</p>"));
        assert!(html.contains("placeholder=\"teammate@company.com\""));
        assert!(html.contains(
            "data-slot=\"invite-dialog-send\" data-variant=\"primary\" disabled>Invite</button>"
        ));
        assert!(html.contains("data-variant=\"outline\" disabled>Dismiss</button>"));
        let mut c = stub("invite-dialog", "Invite member");
        c.items[0]
            .config
            .insert("description".into(), "From config.".into());
        assert!(render(&c).contains("<p data-slot=\"dialog-description\">From config.</p>"));
        reject_js(&html);
    }

    #[test]
    fn trigger_item_renders_closed_modal_dialog_with_working_cancel_and_close() {
        let mut c = stub("invite-dialog", "Invite member");
        c.items.push(extra("trigger", "Invite"));
        let html = render(&c);
        let tid = widget_id(&c, "invite-dialog-trigger");
        let pid = widget_id(&c, "invite-dialog");
        assert!(html.starts_with(&format!(
            "<button type=\"button\" id=\"{tid}\" data-slot=\"button\" data-variant=\"outline\" commandfor=\"{pid}\" command=\"show-modal\" aria-haspopup=\"dialog\">Invite</button><dialog id=\"{pid}\" data-slot=\"invite-dialog\" role=\"dialog\" aria-modal=\"true\" aria-labelledby="
        )));
        assert!(html.contains("closedby=\"any\""));
        assert!(!html.contains("dialog-overlay"));
        assert!(!html.contains("popovertarget"));
        assert!(html.contains(&format!(
            "data-variant=\"outline\" commandfor=\"{pid}\" command=\"close\">Cancel</button>"
        )));
        assert!(html.contains(&format!(
            "<button type=\"button\" data-slot=\"dialog-close\" commandfor=\"{pid}\" command=\"close\">{CROSS}<span>Close</span></button></dialog>"
        )));
        assert!(html.contains("data-variant=\"primary\" disabled>Send invite</button>"));
        reject_js(&html);
    }

    #[test]
    fn chrome_closed_mode_is_native_modal_dialog() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"invite-dialog\"]:modal {\n  position: fixed; inset: 0; margin: auto; translate: none;"));
        assert!(css.contains("[data-slot=\"invite-dialog\"]::backdrop {"));
        assert!(!css.contains("[data-slot=\"invite-dialog\"][popover]"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("invite-dialog", "Invite member"));
            reject_js(&html);
            assert!(html.contains("data-slot=\"invite-dialog-send\""));
            assert!(html.contains("type=\"email\""));
        });
    }

    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        let block = |sel: &str| {
            let start = css
                .find(&format!("{sel} {{"))
                .unwrap_or_else(|| panic!("{sel}"));
            let end = css[start..].find('}').unwrap() + start;
            css[start..end].to_string()
        };
        assert!(!css.contains("dialog:has(> [data-slot=\"invite-dialog\"])"));
        let overlay = block("[data-slot=\"dialog-overlay\"]:has(+ [data-slot=\"invite-dialog\"])");
        assert!(overlay.contains("position: fixed; inset: 0; z-index: 50;"));
        let content = block("[data-slot=\"invite-dialog\"]");
        assert!(content.contains("position: fixed; inset: 0; z-index: 50; margin: auto;"));
        assert!(content.contains("max-width: 28rem"));
        assert!(content.contains("border: 1px solid var(--cronus-border)"));
        assert!(content.contains("border-radius: var(--cronus-radius-xl)"));
        let close = block("[data-slot=\"invite-dialog\"] [data-slot=\"dialog-close\"]");
        assert!(close.contains("position: absolute; top: 1rem; inset-inline-end: 1rem;"));
        let trigger = block("[data-slot=\"invite-dialog\"] [data-slot=\"select-trigger\"]");
        assert!(trigger.contains("height: 2.5rem"));
        assert!(trigger.contains("background: var(--cronus-surface-inset)"));
        let send = block(
            "[data-slot=\"invite-dialog\"] [data-slot=\"dialog-footer\"] > [data-slot=\"invite-dialog-send\"]",
        );
        assert!(send.contains("background: var(--cronus-primary)"));
        assert!(send.contains("min-width: 6rem"));
        assert!(!css.contains("[data-slot=\"invite-dialog-title\"]"));
        assert!(!css.contains("zinc-"));
    }
}
