//! Dedicated ConfirmationDialog renderer — mirrors React `ConfirmationDialog`
//! (an `AlertDialogContent` with `data-slot="confirmation-dialog"`).
//!
//! Open by default, zero JS, no native `<dialog>` (React's portal renders plain
//! divs): the fixed `alert-dialog-overlay` scrim and the fixed, centred
//! `confirmation-dialog` panel (`role="alertdialog"`). Structure as in React:
//! `alert-dialog-header` > `alert-dialog-title` (`h2`) [+ `alert-dialog-description`],
//! then `alert-dialog-footer` > `alert-dialog-cancel` (outline) +
//! `confirmation-dialog-confirm` (primary, min-width 6rem). In this open specimen
//! nothing can close the panel, so both are the same native buttons with
//! `disabled` and React's idle look.
//!
//! Closed mode (`trigger:"…"` prop, or `open:false` / `defaultOpen:false`; see
//! `cronus_ui_kit::overlay_trigger`): a `button` trigger
//! (`cronus_ui_dialog::trigger_button`: `trigger-variant:`, `trigger-icon:`)
//! opens the panel as a native modal `<dialog>` (scrim on `::backdrop`, Esc
//! only, React's `cronus-pop-in` entrance). Cancel and Confirm both close it
//! (`command="close"`; React's `handleConfirm` closes when there is no async
//! `onConfirm`). `confirm:"…"` / `cancel:"…"` props (or `action` items) label
//! the buttons; `icon:` fills the header badge (`confirmation-dialog-icon`);
//! `destructive:true` (or `style:confirmation-dialog+destructive`) switches to
//! the red confirm button and the triangle-alert badge — React has no data
//! attribute for it, so both carry a `destructive` class.
//! Not reproduced (JS-only): async `onConfirm` spinner/error state.

use crate::cronus_ui_dialog::trigger_button;
use crate::cronus_ui_kit::{
    attr_nonempty, esc, item, label_of, modal_close_attrs, modal_dialog_open, overlay_trigger,
    widget_id,
};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let confirm = attr_nonempty(comp, "confirm")
        .or_else(|| item(comp, "confirm"))
        .or_else(|| item(comp, "action"))
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| "Confirm".into());
    let cancel = attr_nonempty(comp, "cancel")
        .or_else(|| item(comp, "cancel"))
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| "Cancel".into());
    let destructive = crate::cronus_ui_kit::flag(comp, "destructive")
        || crate::cronus_ui_kit::style_seg(comp, &["destructive"]).is_some();
    let badge_class = if destructive {
        " class=\"destructive\""
    } else {
        ""
    };
    let icon = match attr_nonempty(comp, "icon") {
        Some(name) => crate::cronus_ui_icons::svg_or_empty(name),
        None if destructive => crate::cronus_ui_icons::svg_or_empty("triangle-alert"),
        None => String::new(),
    };
    let icon = if icon.is_empty() {
        String::new()
    } else {
        format!(
            "<div aria-hidden=\"true\" data-slot=\"confirmation-dialog-icon\"{badge_class}>{icon}</div>"
        )
    };
    // `description:"…"` prop or attr-style item config, else free text items.
    let desc = attr_nonempty(comp, "description")
        .map(|d| esc(d))
        .unwrap_or_else(|| extras(comp, &title));
    let desc_html = if desc.is_empty() {
        String::new()
    } else {
        format!("<p data-slot=\"alert-dialog-description\">{desc}</p>")
    };
    let title_id = widget_id(comp, "confirmation-dialog-title");
    let trigger = overlay_trigger(comp, "Open");
    let pop_id = widget_id(comp, "confirmation-dialog");
    let closes = if trigger.is_some() {
        modal_close_attrs(&pop_id)
    } else {
        " disabled".to_string()
    };
    let body = format!(
        "<div data-slot=\"alert-dialog-header\">{icon}<h2 data-slot=\"alert-dialog-title\" id=\"{title_id}\">{title}</h2>{desc_html}</div><div data-slot=\"alert-dialog-footer\"><button type=\"button\" data-slot=\"alert-dialog-cancel\"{closes}>{cancel}</button><button type=\"button\" data-slot=\"confirmation-dialog-confirm\"{badge_class}{closes}>{confirm}</button></div>"
    );
    match trigger {
        None => format!(
            "<div data-slot=\"alert-dialog-overlay\" data-state=\"open\" aria-hidden=\"true\"></div><div data-slot=\"confirmation-dialog\" data-state=\"open\" role=\"alertdialog\" aria-labelledby=\"{title_id}\">{body}</div>"
        ),
        Some(trigger) => {
            let trigger_id = widget_id(comp, "confirmation-dialog-trigger");
            format!(
                "{}{}{body}</dialog>",
                trigger_button(comp, &trigger_id, &pop_id, &trigger),
                modal_dialog_open(
                    &pop_id,
                    "confirmation-dialog",
                    "alertdialog",
                    &title_id,
                    "",
                    false
                ),
            )
        }
    }
}

fn extras(comp: &ComponentNode, title: &str) -> String {
    comp.items
        .iter()
        .filter(|i| {
            !matches!(
                i.item_type.as_str(),
                "label" | "title" | "action" | "cancel" | "confirm" | "trigger"
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

    fn reject_js(html: &str) {
        assert!(!html.contains("showModal("));
        assert!(!html.contains("-control"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("popovertarget"));
    }

    #[test]
    fn fixture_matches_react_dom() {
        let c = stub("confirmation-dialog", "Delete project");
        let id = widget_id(&c, "");
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"alert-dialog-overlay\" data-state=\"open\" aria-hidden=\"true\"></div><div data-slot=\"confirmation-dialog\" data-state=\"open\" role=\"alertdialog\" aria-labelledby=\"{id}confirmation-dialog-title\"><div data-slot=\"alert-dialog-header\"><h2 data-slot=\"alert-dialog-title\" id=\"{id}confirmation-dialog-title\">Delete project</h2></div><div data-slot=\"alert-dialog-footer\"><button type=\"button\" data-slot=\"alert-dialog-cancel\" disabled>Cancel</button><button type=\"button\" data-slot=\"confirmation-dialog-confirm\" disabled>Confirm</button></div></div>"
            )
        );
        assert!(!html.contains("data-slot=\"confirmation-dialog-title\""));
        reject_js(&html);
    }

    #[test]
    fn description_and_label_items() {
        let mut c = stub("confirmation-dialog", "Delete project");
        c.items.push(extra("text", "This cannot be undone."));
        c.items.push(extra("cancel", "Keep"));
        c.items.push(extra("confirm", "Delete anyway"));
        let html = render(&c);
        assert!(html.contains(
            "Delete project</h2><p data-slot=\"alert-dialog-description\">This cannot be undone.</p></div>"
        ));
        assert!(html.contains("data-slot=\"alert-dialog-cancel\" disabled>Keep</button>"));
        assert!(html
            .contains("data-slot=\"confirmation-dialog-confirm\" disabled>Delete anyway</button>"));
        reject_js(&html);
    }

    #[test]
    fn description_prop_renders_react_description() {
        let mut c = stub("confirmation-dialog", "Delete project");
        c.props
            .insert("description".into(), "This cannot be undone.".into());
        assert!(render(&c).contains(
            "Delete project</h2><p data-slot=\"alert-dialog-description\">This cannot be undone.</p></div>"
        ));
    }

    #[test]
    fn trigger_item_renders_closed_modal_dialog_with_working_cancel() {
        let mut c = stub("confirmation-dialog", "Delete project");
        c.items.push(extra("trigger", "Delete…"));
        let html = render(&c);
        let tid = widget_id(&c, "confirmation-dialog-trigger");
        let pid = widget_id(&c, "confirmation-dialog");
        assert!(html.starts_with(&format!(
            "<button type=\"button\" id=\"{tid}\" data-slot=\"button\" data-variant=\"outline\" commandfor=\"{pid}\" command=\"show-modal\" aria-haspopup=\"dialog\">Delete…</button><dialog id=\"{pid}\" data-slot=\"confirmation-dialog\" role=\"alertdialog\" aria-modal=\"true\" aria-labelledby="
        )));
        assert!(html.contains("closedby=\"closerequest\""));
        assert!(!html.contains("alert-dialog-overlay"));
        assert!(!html.contains("popover"));
        assert!(
            !html.contains("alert-dialog-description"),
            "trigger is not description"
        );
        assert!(html.contains(&format!(
            "<button type=\"button\" data-slot=\"alert-dialog-cancel\" commandfor=\"{pid}\" command=\"close\">Cancel</button>"
        )));
        // React closes on confirm when there is no async `onConfirm`.
        assert!(html.contains(&format!(
            "data-slot=\"confirmation-dialog-confirm\" commandfor=\"{pid}\" command=\"close\">Confirm</button>"
        )));
        reject_js(&html);
    }

    #[test]
    fn docs_variants_icon_badge_and_destructive() {
        let mut c = stub("confirmation-dialog", "Publish this article?");
        c.props.insert("trigger".into(), "Publish article".into());
        c.props.insert("trigger-variant".into(), "primary".into());
        c.props.insert("icon".into(), "rocket".into());
        c.props.insert("confirm".into(), "Publish".into());
        c.props.insert(
            "description".into(),
            "It becomes visible to everyone.".into(),
        );
        let html = render(&c);
        let pid = widget_id(&c, "confirmation-dialog");
        assert!(html.contains(
            "data-variant=\"primary\" commandfor=\"{pid}\" command=\"show-modal\""
                .replace("{pid}", &pid)
                .as_str()
        ));
        assert!(html.contains("<div data-slot=\"alert-dialog-header\"><div aria-hidden=\"true\" data-slot=\"confirmation-dialog-icon\"><svg "), "{html}");
        assert!(html.contains("data-icon=\"rocket\""));
        assert!(html.contains("<button type=\"button\" data-slot=\"alert-dialog-cancel\" commandfor=\"{pid}\" command=\"close\">Cancel</button><button type=\"button\" data-slot=\"confirmation-dialog-confirm\" commandfor=\"{pid}\" command=\"close\">Publish</button>".replace("{pid}", &pid).as_str()));
        let mut d = stub("confirmation-dialog+destructive", "Delete your account?");
        d.props.insert("trigger".into(), "Delete account".into());
        d.props.insert("confirm".into(), "Delete account".into());
        d.props.insert("cancel".into(), "Cancel".into());
        let html = render(&d);
        assert!(html.contains("<div aria-hidden=\"true\" data-slot=\"confirmation-dialog-icon\" class=\"destructive\"><svg "));
        assert!(html.contains("data-icon=\"triangle-alert\""));
        assert!(html.contains(
            "data-slot=\"confirmation-dialog-confirm\" class=\"destructive\" commandfor="
        ));
        assert!(html.contains("data-variant=\"outline\" commandfor="));
        // No icon, no badge (the audit fixture).
        assert!(!render(&stub("confirmation-dialog", "Sure?")).contains("confirmation-dialog-icon"));
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"confirmation-dialog-icon\"] {\n  display: flex; width: 2.75rem; height: 2.75rem; align-items: center; justify-content: center; align-self: center;"));
        assert!(css.contains("[data-slot=\"confirmation-dialog-icon\"].destructive {\n  background: color-mix(in oklab, var(--cronus-error) 10%, transparent); color: var(--cronus-error);"));
        assert!(css.contains("[data-slot=\"confirmation-dialog-confirm\"].destructive {\n  background: color-mix(in oklch, var(--cronus-error), black 30%); color: #fff;"));
        assert!(css.contains("[data-slot=\"confirmation-dialog\"]:modal {\n  position: fixed; inset: 0; margin: auto; translate: none;\n  animation: cronus-pop-in 200ms var(--ease-out-quart) both;"));
    }

    #[test]
    fn chrome_closed_mode_is_native_modal_dialog() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css
            .contains("dialog[data-slot=\"confirmation-dialog\"]:not([open]) { display: none; }"));
        assert!(css.contains("[data-slot=\"confirmation-dialog\"]:modal {\n  position: fixed; inset: 0; margin: auto; translate: none;"));
        assert!(css.contains("[data-slot=\"confirmation-dialog\"]::backdrop {"));
        assert!(!css.contains("[data-slot=\"confirmation-dialog\"][popover]"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("confirmation-dialog", "Delete project"));
            reject_js(&html);
            assert!(html.contains("data-slot=\"confirmation-dialog-confirm\""));
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
        assert!(!css.contains("dialog:has(> [data-slot=\"confirmation-dialog\"])"));
        assert!(block("[data-slot=\"confirmation-dialog\"]").contains("max-width: 28rem"));
        let header = block("[data-slot=\"alert-dialog-header\"]");
        assert!(header.contains("flex-direction: column; gap: 0.375rem;"));
        let footer = block("[data-slot=\"alert-dialog-footer\"]");
        assert!(footer.contains("justify-content: flex-end; gap: 0.5rem;"));
        let cancel = block("[data-slot=\"alert-dialog-cancel\"]");
        assert!(cancel.contains("border: 1px solid var(--cronus-border)"));
        assert!(cancel.contains("background: transparent"));
        assert!(block("[data-slot=\"confirmation-dialog-confirm\"]").contains("min-width: 6rem"));
        assert!(!css.contains("[data-slot=\"confirmation-dialog-title\"]"));
        assert!(!css.contains("zinc-"));
    }
}
