//! Dedicated AlertDialog renderer — mirrors the React `AlertDialogContent`
//! fixture (packages/audit react-fixture-render: title + one action).
//!
//! Open by default, zero JS, no native `<dialog>` (React's portal renders plain
//! divs): a fixed `alert-dialog-overlay` scrim and the fixed, centred
//! `alert-dialog-content` panel carrying `role="alertdialog"`. In this open
//! specimen nothing can close the panel, so action/cancel are the same native
//! buttons with `disabled` and React's idle look (Wave 1t rule). No trigger: the
//! React fixture renders none.
//!
//! Closed mode (`trigger:"…"` prop, or `open:false` / `defaultOpen:false`; see
//! `cronus_ui_kit::overlay_trigger`): a `button` trigger
//! (`cronus_ui_dialog::trigger_button`: outline unless `trigger-variant:`,
//! `trigger-icon:` glyph) opens the content as a native `<dialog>`
//! (`command="show-modal"`: focus trap, inert background, React's
//! `cronus-pop-in` entrance). `closedby="closerequest"` so only Esc (not an
//! outside click) dismisses it, matching Radix AlertDialog. The docs' DOM is
//! used in this mode: `alert-dialog-header` (title + description) and
//! `alert-dialog-footer` (cancel + action), and both buttons close natively
//! (`command="close"`) like Radix `AlertDialogCancel` / `AlertDialogAction`.
//!
//! Content: `alert-dialog-title` (`h2`), optional `alert-dialog-description`
//! (`description:"…"` prop / item config / `description` item), optional
//! `alert-dialog-cancel` (`cancel:"…"` prop or `cancel` item) and the primary
//! `alert-dialog-action` (`action` item, else the first `text` item — the
//! fixture's `items[0]` — else "Confirm").

use crate::cronus_ui_dialog::trigger_button;
use crate::cronus_ui_kit::{
    attr, attr_nonempty, esc, item, label_of, modal_close_attrs, modal_dialog_open,
    overlay_trigger, widget_id,
};
use crate::parser::ComponentNode;

fn non_empty<'a>(comp: &'a ComponentNode, kind: &str) -> Option<&'a str> {
    item(comp, kind).filter(|t| !t.is_empty())
}

/// `description:"…"` prop, attr-style item config, or a `description` item.
fn description_of(comp: &ComponentNode) -> Option<String> {
    attr(comp, "description")
        .or_else(|| item(comp, "description"))
        .filter(|d| !d.is_empty())
        .map(esc)
}

pub fn render(comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let action = non_empty(comp, "action")
        .or_else(|| {
            comp.items
                .iter()
                .find(|i| i.item_type == "text" && !i.text.is_empty())
                .map(|i| i.text.as_str())
        })
        .map(esc)
        .unwrap_or_else(|| "Confirm".into());
    let title_id = widget_id(comp, "alert-dialog-title");
    let desc_id = widget_id(comp, "alert-dialog-description");
    let description = description_of(comp);
    let trigger = overlay_trigger(comp, "Open");
    let pop_id = widget_id(comp, "alert-dialog");
    let cancel_text = attr_nonempty(comp, "cancel")
        .or_else(|| non_empty(comp, "cancel"))
        .map(esc);
    match trigger {
        None => {
            let description = description
                .map(|d| format!("<p data-slot=\"alert-dialog-description\">{d}</p>"))
                .unwrap_or_default();
            let cancel = cancel_text
                .map(|c| {
                    format!(
                        "<button type=\"button\" data-slot=\"alert-dialog-cancel\" disabled>{c}</button>"
                    )
                })
                .unwrap_or_default();
            format!(
                "<div data-slot=\"alert-dialog-overlay\" data-state=\"open\" aria-hidden=\"true\"></div><div data-slot=\"alert-dialog-content\" data-state=\"open\" role=\"alertdialog\" aria-labelledby=\"{title_id}\"><h2 data-slot=\"alert-dialog-title\" id=\"{title_id}\">{title}</h2>{description}{cancel}<button type=\"button\" data-slot=\"alert-dialog-action\" disabled>{action}</button></div>"
            )
        }
        Some(trigger) => {
            let trigger_id = widget_id(comp, "alert-dialog-trigger");
            let closes = modal_close_attrs(&pop_id);
            let (described, description) = match description {
                Some(d) => (
                    format!(" aria-describedby=\"{desc_id}\""),
                    format!("<p data-slot=\"alert-dialog-description\" id=\"{desc_id}\">{d}</p>"),
                ),
                None => (String::new(), String::new()),
            };
            let cancel = cancel_text
                .map(|c| {
                    format!(
                        "<button type=\"button\" data-slot=\"alert-dialog-cancel\"{closes}>{c}</button>"
                    )
                })
                .unwrap_or_default();
            format!(
                "{}{}<div data-slot=\"alert-dialog-header\"><h2 data-slot=\"alert-dialog-title\" id=\"{title_id}\">{title}</h2>{description}</div><div data-slot=\"alert-dialog-footer\">{cancel}<button type=\"button\" data-slot=\"alert-dialog-action\"{closes}>{action}</button></div></dialog>",
                trigger_button(comp, &trigger_id, &pop_id, &trigger),
                modal_dialog_open(
                    &pop_id,
                    "alert-dialog-content",
                    "alertdialog",
                    &title_id,
                    &described,
                    false
                ),
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

    /// Emitted audit fixture: `label "Delete account"` + `text "Confirm"`.
    fn fixture() -> ComponentNode {
        let mut c = stub("alert-dialog", "Delete account");
        c.items.push(extra("text", "Confirm"));
        c
    }

    fn reject_js(html: &str) {
        assert!(!html.contains("showModal("));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("popovertarget"));
    }

    #[test]
    fn fixture_matches_react_dom() {
        let html = render(&fixture());
        let id = widget_id(&fixture(), "");
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"alert-dialog-overlay\" data-state=\"open\" aria-hidden=\"true\"></div><div data-slot=\"alert-dialog-content\" data-state=\"open\" role=\"alertdialog\" aria-labelledby=\"{id}alert-dialog-title\"><h2 data-slot=\"alert-dialog-title\" id=\"{id}alert-dialog-title\">Delete account</h2><button type=\"button\" data-slot=\"alert-dialog-action\" disabled>Confirm</button></div>"
            )
        );
        reject_js(&html);
    }

    #[test]
    fn no_wrapper_trigger_description_or_cancel_by_default() {
        let html = render(&fixture());
        assert!(!html.contains("data-slot=\"alert-dialog\""));
        assert_eq!(html.matches("<button").count(), 1);
        assert!(!html.contains("alert-dialog-description"));
        assert!(!html.contains("alert-dialog-cancel"));
        assert!(!html.contains("Continue"));
    }

    #[test]
    fn action_defaults_to_confirm_and_items_override() {
        let html = render(&stub("alert-dialog", "Delete"));
        assert!(html.contains("data-slot=\"alert-dialog-action\" disabled>Confirm</button>"));
        let mut c = fixture();
        c.items.push(extra("action", "Delete anyway"));
        c.items.push(extra("cancel", "Keep"));
        c.items.push(extra("description", "This cannot be undone."));
        let html = render(&c);
        assert!(html.contains("data-slot=\"alert-dialog-action\" disabled>Delete anyway</button>"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"alert-dialog-cancel\" disabled>Keep</button>"
        ));
        assert!(
            html.contains("<p data-slot=\"alert-dialog-description\">This cannot be undone.</p>")
        );
        reject_js(&html);
    }

    #[test]
    fn description_from_prop_or_item_config() {
        let mut c = fixture();
        c.props
            .insert("description".into(), "Gone for good.".into());
        assert!(render(&c).contains("<p data-slot=\"alert-dialog-description\">Gone for good.</p>"));
        let mut c = fixture();
        c.items[0]
            .config
            .insert("description".into(), "From config.".into());
        assert!(render(&c).contains("<p data-slot=\"alert-dialog-description\">From config.</p>"));
    }

    #[test]
    fn trigger_item_renders_closed_modal_dialog_with_working_cancel() {
        let mut c = fixture();
        c.items.push(extra("trigger", "Delete"));
        c.items.push(extra("cancel", "Keep"));
        let html = render(&c);
        let tid = widget_id(&c, "alert-dialog-trigger");
        let pid = widget_id(&c, "alert-dialog");
        assert!(html.starts_with(&format!(
            "<button type=\"button\" id=\"{tid}\" data-slot=\"button\" data-variant=\"outline\" commandfor=\"{pid}\" command=\"show-modal\" aria-haspopup=\"dialog\">Delete</button><dialog id=\"{pid}\" data-slot=\"alert-dialog-content\" role=\"alertdialog\" aria-modal=\"true\" aria-labelledby="
        )));
        assert!(html.contains("closedby=\"closerequest\""));
        assert!(!html.contains("alert-dialog-overlay"));
        assert!(!html.contains("popover"));
        // Docs DOM: header + footer wrappers; cancel and action both close (Radix Cancel/Action).
        assert!(html.contains(&format!(
            "<div data-slot=\"alert-dialog-footer\"><button type=\"button\" data-slot=\"alert-dialog-cancel\" commandfor=\"{pid}\" command=\"close\">Keep</button><button type=\"button\" data-slot=\"alert-dialog-action\" commandfor=\"{pid}\" command=\"close\">Confirm</button></div></dialog>"
        )), "{html}");
        assert!(html.contains(
            "<div data-slot=\"alert-dialog-header\"><h2 data-slot=\"alert-dialog-title\""
        ));
        reject_js(&html);
        let mut open = c.clone();
        open.props.insert("open".into(), "true".into());
        let open_html = render(&open);
        assert!(open_html.starts_with("<div data-slot=\"alert-dialog-overlay\""));
        assert!(!open_html.contains("alert-dialog-header"));
    }

    #[test]
    fn docs_confirm_has_destructive_icon_trigger_and_description() {
        let mut c = stub("alert-dialog", "Are you absolutely sure?");
        c.props.insert("trigger".into(), "Delete account".into());
        c.props
            .insert("trigger-variant".into(), "destructive".into());
        c.props.insert("trigger-icon".into(), "trash-2".into());
        c.props.insert("cancel".into(), "Cancel".into());
        c.props
            .insert("description".into(), "This cannot be undone.".into());
        c.items.push(extra("action", "Yes, delete it"));
        let html = render(&c);
        let pid = widget_id(&c, "alert-dialog");
        let did = widget_id(&c, "alert-dialog-description");
        assert!(html.starts_with(&format!(
            "<button type=\"button\" id=\"{}\" data-slot=\"button\" data-variant=\"destructive\" commandfor=\"{pid}\" command=\"show-modal\" aria-haspopup=\"dialog\"><svg ",
            widget_id(&c, "alert-dialog-trigger")
        )), "{html}");
        assert!(html.contains("data-icon=\"trash-2\""));
        assert!(html.contains(&format!(
            "aria-describedby=\"{did}\" closedby=\"closerequest\">"
        )));
        assert!(html.contains(&format!(
            "<p data-slot=\"alert-dialog-description\" id=\"{did}\">This cannot be undone.</p></div>"
        )));
        assert!(html.contains("command=\"close\">Yes, delete it</button>"));
        reject_js(&html);
    }

    #[test]
    fn chrome_closed_mode_is_native_modal_dialog() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css
            .contains("dialog[data-slot=\"alert-dialog-content\"]:not([open]) { display: none; }"));
        assert!(css.contains("[data-slot=\"alert-dialog-content\"]:modal {\n  position: fixed; inset: 0; margin: auto; translate: none;"));
        assert!(css.contains("[data-slot=\"alert-dialog-content\"]::backdrop {"));
        assert!(!css.contains("[data-slot=\"alert-dialog-content\"][popover]"));
        assert!(css.contains("[data-slot=\"alert-dialog-content\"]:modal {\n  position: fixed; inset: 0; margin: auto; translate: none;\n  animation: cronus-pop-in 200ms var(--ease-out-quart) both;"));
        assert!(css.contains("[data-slot=\"alert-dialog-content\"]:modal::backdrop {\n  animation: cronus-overlay-in 200ms var(--ease-out-quart);"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&fixture());
            reject_js(&html);
            assert!(html.contains("data-slot=\"alert-dialog-content\""));
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
        assert!(!css.contains("dialog:has(> [data-slot=\"alert-dialog-content\"])"));
        let overlay = block("[data-slot=\"alert-dialog-overlay\"]");
        assert!(overlay.contains("position: fixed; inset: 0; z-index: 50;"));
        assert!(overlay.contains("backdrop-filter: blur(8px)"));
        let content =
            block("[data-slot=\"confirmation-dialog\"], [data-slot=\"alert-dialog-content\"]");
        assert!(content.contains("position: fixed; inset: 0; z-index: 50; margin: auto;"));
        assert!(content.contains("height: fit-content"));
        assert!(content.contains("max-width: 32rem"));
        assert!(content.contains("border: 1px solid var(--cronus-border)"));
        assert!(content.contains("border-radius: var(--cronus-radius-xl)"));
        assert!(content.contains("background: var(--cronus-surface-floating)"));
        let title = block("[data-slot=\"alert-dialog-title\"]");
        assert!(title.contains("font-size: 1.125rem; line-height: 1.75rem; font-weight: 600;"));
        let action = block("[data-slot=\"alert-dialog-action\"]");
        assert!(action.contains("height: 2.5rem"));
        assert!(action.contains("line-height: 1.25rem"));
        assert!(action.contains("border-radius: var(--cronus-radius-lg)"));
        assert!(action.contains("background: var(--cronus-primary)"));
        assert!(!css.contains("[data-slot=\"alert-dialog\"] {"));
        assert!(!css.contains("zinc-"));
    }
}
