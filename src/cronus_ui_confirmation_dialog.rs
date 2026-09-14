//! Dedicated ConfirmationDialog renderer — mirrors React `ConfirmationDialog`
//! (an `AlertDialogContent` with `data-slot="confirmation-dialog"`).
//!
//! Open by default, zero JS: a non-modal native `<dialog open>` (chrome gives it
//! `display: contents`) wraps the fixed `alert-dialog-overlay` scrim and the
//! fixed, centred `confirmation-dialog` panel. Structure as in React:
//! `alert-dialog-header` > `alert-dialog-title` (`h2`) [+ `alert-dialog-description`],
//! then `alert-dialog-footer` > `alert-dialog-cancel` (outline) +
//! `confirmation-dialog-confirm` (primary, min-width 6rem). Both buttons submit a
//! hidden `<form method="dialog">`, so they close the dialog without script.
//! Not reproduced (JS-only): focus trap, async `onConfirm` spinner/error state.

use crate::cronus_ui_kit::{esc, item, label_of, widget_id};
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
    // `description:"…"` prop or attr-style item config, else free text items.
    let desc = comp
        .props
        .get("description")
        .or_else(|| comp.items.iter().find_map(|i| i.config.get("description")))
        .filter(|d| !d.is_empty())
        .map(|d| esc(d))
        .unwrap_or_else(|| extras(comp, &title));
    let desc_html = if desc.is_empty() {
        String::new()
    } else {
        format!("<p data-slot=\"alert-dialog-description\">{desc}</p>")
    };
    let form = widget_id(comp, "confirmation-dialog-form");
    let title_id = widget_id(comp, "confirmation-dialog-title");
    format!(
        "<dialog open role=\"alertdialog\" aria-labelledby=\"{title_id}\"><form method=\"dialog\" id=\"{form}\" hidden></form><div data-slot=\"alert-dialog-overlay\" data-state=\"open\" aria-hidden=\"true\"></div><div data-slot=\"confirmation-dialog\" data-state=\"open\"><div data-slot=\"alert-dialog-header\"><h2 data-slot=\"alert-dialog-title\" id=\"{title_id}\">{title}</h2>{desc_html}</div><div data-slot=\"alert-dialog-footer\"><button type=\"submit\" form=\"{form}\" value=\"cancel\" data-slot=\"alert-dialog-cancel\">{cancel}</button><button type=\"submit\" form=\"{form}\" value=\"confirm\" data-slot=\"confirmation-dialog-confirm\">{confirm}</button></div></div></dialog>"
    )
}

fn extras(comp: &ComponentNode, title: &str) -> String {
    comp.items
        .iter()
        .filter(|i| !matches!(i.item_type.as_str(), "label" | "title" | "action" | "cancel" | "confirm"))
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
        assert!(!html.contains("showModal"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("<dialog data-slot="));
    }

    #[test]
    fn fixture_matches_react_dom() {
        let c = stub("confirmation-dialog", "Delete project");
        let id = widget_id(&c, "");
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<dialog open role=\"alertdialog\" aria-labelledby=\"{id}confirmation-dialog-title\"><form method=\"dialog\" id=\"{id}confirmation-dialog-form\" hidden></form><div data-slot=\"alert-dialog-overlay\" data-state=\"open\" aria-hidden=\"true\"></div><div data-slot=\"confirmation-dialog\" data-state=\"open\"><div data-slot=\"alert-dialog-header\"><h2 data-slot=\"alert-dialog-title\" id=\"{id}confirmation-dialog-title\">Delete project</h2></div><div data-slot=\"alert-dialog-footer\"><button type=\"submit\" form=\"{id}confirmation-dialog-form\" value=\"cancel\" data-slot=\"alert-dialog-cancel\">Cancel</button><button type=\"submit\" form=\"{id}confirmation-dialog-form\" value=\"confirm\" data-slot=\"confirmation-dialog-confirm\">Confirm</button></div></div></dialog>"
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
        assert!(html.contains("data-slot=\"alert-dialog-cancel\">Keep</button>"));
        assert!(html.contains("data-slot=\"confirmation-dialog-confirm\">Delete anyway</button>"));
        reject_js(&html);
    }

    #[test]
    fn description_prop_renders_react_description() {
        let mut c = stub("confirmation-dialog", "Delete project");
        c.props.insert("description".into(), "This cannot be undone.".into());
        assert!(render(&c).contains(
            "Delete project</h2><p data-slot=\"alert-dialog-description\">This cannot be undone.</p></div>"
        ));
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
            let start = css.find(&format!("{sel} {{")).unwrap_or_else(|| panic!("{sel}"));
            let end = css[start..].find('}').unwrap() + start;
            css[start..end].to_string()
        };
        assert!(css.contains("dialog:has(> [data-slot=\"confirmation-dialog\"])"));
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
