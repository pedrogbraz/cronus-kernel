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
//! `cronus_ui_kit::overlay_trigger`): an outline `button` trigger opens the
//! content as a native `popover="auto"` (scrim on `::backdrop`, no overlay div)
//! and `alert-dialog-cancel` hides it (`popovertargetaction="hide"`). The action
//! still needs JS and stays `disabled`. Gaps: a popover is not modal (no focus
//! trap, background not inert, `aria-expanded` not reflected), and unlike
//! Radix AlertDialog an outside click also dismisses it (light dismiss).
//!
//! Content: `alert-dialog-title` (`h2`), optional `alert-dialog-description`
//! (`description` item), optional `alert-dialog-cancel` (`cancel` item) and the
//! primary `alert-dialog-action` (`action` item, else the first `text` item —
//! the fixture's `items[0]` — else "Confirm").

use crate::cronus_ui_kit::{attr, esc, item, label_of, overlay_trigger, widget_id};
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
    let description = description_of(comp)
        .map(|d| format!("<p data-slot=\"alert-dialog-description\">{d}</p>"))
        .unwrap_or_default();
    let trigger = overlay_trigger(comp, "Open");
    let pop_id = widget_id(comp, "alert-dialog");
    let cancel_state = if trigger.is_some() {
        format!(" popovertarget=\"{pop_id}\" popovertargetaction=\"hide\"")
    } else {
        " disabled".to_string()
    };
    let cancel = non_empty(comp, "cancel")
        .map(|c| {
            format!(
                "<button type=\"button\" data-slot=\"alert-dialog-cancel\"{cancel_state}>{}</button>",
                esc(c)
            )
        })
        .unwrap_or_default();
    let body = format!(
        "<h2 data-slot=\"alert-dialog-title\" id=\"{title_id}\">{title}</h2>{description}{cancel}<button type=\"button\" data-slot=\"alert-dialog-action\" disabled>{action}</button>"
    );
    match trigger {
        None => format!(
            "<div data-slot=\"alert-dialog-overlay\" data-state=\"open\" aria-hidden=\"true\"></div><div data-slot=\"alert-dialog-content\" data-state=\"open\" role=\"alertdialog\" aria-labelledby=\"{title_id}\">{body}</div>"
        ),
        Some(trigger) => {
            let trigger_id = widget_id(comp, "alert-dialog-trigger");
            format!(
                "<button type=\"button\" id=\"{trigger_id}\" data-slot=\"button\" data-variant=\"outline\" popovertarget=\"{pop_id}\" aria-haspopup=\"dialog\">{trigger}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"alert-dialog-content\" role=\"alertdialog\" aria-labelledby=\"{title_id}\">{body}</div>"
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
        assert!(!html.contains("showModal"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("<dialog"));
        assert!(!html.contains("<form"));
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
    fn trigger_item_renders_closed_popover_with_working_cancel() {
        let mut c = fixture();
        c.items.push(extra("trigger", "Delete"));
        c.items.push(extra("cancel", "Keep"));
        let html = render(&c);
        let tid = widget_id(&c, "alert-dialog-trigger");
        let pid = widget_id(&c, "alert-dialog");
        assert!(html.starts_with(&format!(
            "<button type=\"button\" id=\"{tid}\" data-slot=\"button\" data-variant=\"outline\" popovertarget=\"{pid}\" aria-haspopup=\"dialog\">Delete</button><div id=\"{pid}\" popover=\"auto\" data-slot=\"alert-dialog-content\" role=\"alertdialog\" aria-labelledby="
        )));
        assert!(!html.contains("alert-dialog-overlay"));
        assert!(html.contains(&format!(
            "<button type=\"button\" data-slot=\"alert-dialog-cancel\" popovertarget=\"{pid}\" popovertargetaction=\"hide\">Keep</button>"
        )));
        assert!(html.contains("data-slot=\"alert-dialog-action\" disabled>Confirm</button>"));
        reject_js(&html);
        let mut open = c.clone();
        open.props.insert("open".into(), "true".into());
        assert!(render(&open).starts_with("<div data-slot=\"alert-dialog-overlay\""));
    }

    #[test]
    fn chrome_closed_mode_hides_until_open_with_backdrop() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"alert-dialog-content\"][popover]:not(:popover-open) { display: none; }"
        ));
        assert!(css.contains("[data-slot=\"alert-dialog-content\"][popover]:popover-open {\n  position: fixed; inset: 0; margin: auto; translate: none;"));
        assert!(css.contains("[data-slot=\"alert-dialog-content\"][popover]::backdrop {"));
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
