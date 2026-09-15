//! Dedicated Dialog renderer — mirrors React `DialogContent` (Radix) in the
//! audited open state (fixture `defaultOpen`: title, description, one action).
//!
//! Open by default, zero JS, no native `<dialog>` (React's portal renders plain
//! divs), same model as alert-dialog / invite-dialog: a fixed `dialog-overlay`
//! scrim and the fixed, centred `dialog-content` panel (`role="dialog"`) >
//! `dialog-header` (`dialog-title` h2 + optional `dialog-description` p),
//! `dialog-footer` with the primary action (`data-slot="button"`, the React
//! fixture's `Button`) and the absolute `dialog-close` icon button.
//! In this open specimen the panel is not a popover, so nothing can close it:
//! action and close are the same native buttons with `disabled` and React's
//! idle look.
//!
//! Closed mode (`trigger:"…"` prop, or `open:false` / `defaultOpen:false`; see
//! `cronus_ui_kit::overlay_trigger`): an outline `button` trigger with
//! `popovertarget` opens `dialog-content` as a native `popover="auto"` (Esc and
//! outside click dismiss it; the scrim is `::backdrop`, so no overlay div) and
//! `dialog-close` hides it (`popovertargetaction="hide"`). The action still
//! needs JS and stays `disabled`. Gaps (a popover is not modal): no focus trap,
//! the background is not inert, `aria-modal` is not claimed, and the trigger's
//! state is not reflected in `aria-expanded`.
//!
//! Content: title = label; description = `description:"…"` prop / item config /
//! `description` item; action = `action` item, else the first `text` item, else
//! "Continue"; close label = `close` item, else "Close".

use crate::cronus_ui_kit::{attr, esc, item, label_of, overlay_trigger, widget_id};
use crate::parser::ComponentNode;

/// lucide `x` (React `size-4`).
const CROSS: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M18 6 6 18\"></path><path d=\"m6 6 12 12\"></path></svg>";

fn non_empty<'a>(comp: &'a ComponentNode, kind: &str) -> Option<&'a str> {
    item(comp, kind).filter(|t| !t.is_empty())
}

pub fn render(comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let description = attr(comp, "description")
        .or_else(|| item(comp, "description"))
        .filter(|d| !d.is_empty())
        .map(esc);
    let action = non_empty(comp, "action")
        .or_else(|| {
            comp.items
                .iter()
                .find(|i| i.item_type == "text" && !i.text.is_empty())
                .map(|i| i.text.as_str())
        })
        .map(esc)
        .unwrap_or_else(|| "Continue".into());
    let close = non_empty(comp, "close")
        .map(esc)
        .unwrap_or_else(|| "Close".into());
    let title_id = widget_id(comp, "dialog-title");
    let desc_id = widget_id(comp, "dialog-description");
    let (described, description) = match description {
        Some(d) => (
            format!(" aria-describedby=\"{desc_id}\""),
            format!("<p data-slot=\"dialog-description\" id=\"{desc_id}\">{d}</p>"),
        ),
        None => (String::new(), String::new()),
    };
    let body = format!(
        "<div data-slot=\"dialog-header\"><h2 data-slot=\"dialog-title\" id=\"{title_id}\">{title}</h2>{description}</div><div data-slot=\"dialog-footer\"><button type=\"button\" data-slot=\"button\" data-variant=\"primary\" disabled>{action}</button></div>"
    );
    match overlay_trigger(comp, "Open") {
        None => format!(
            "<div data-slot=\"dialog-overlay\" data-state=\"open\" aria-hidden=\"true\"></div><div data-slot=\"dialog-content\" data-state=\"open\" role=\"dialog\" aria-labelledby=\"{title_id}\"{described}>{body}<button type=\"button\" data-slot=\"dialog-close\" disabled>{CROSS}<span>{close}</span></button></div>"
        ),
        Some(trigger) => {
            let trigger_id = widget_id(comp, "dialog-trigger");
            let pop_id = widget_id(comp, "dialog");
            format!(
                "<button type=\"button\" id=\"{trigger_id}\" data-slot=\"button\" data-variant=\"outline\" popovertarget=\"{pop_id}\" aria-haspopup=\"dialog\">{trigger}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"dialog-content\" role=\"dialog\" aria-labelledby=\"{title_id}\"{described}>{body}<button type=\"button\" data-slot=\"dialog-close\" popovertarget=\"{pop_id}\" popovertargetaction=\"hide\">{CROSS}<span>{close}</span></button></div>"
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

    /// Emitted audit fixture: `description:"…"`, `label "Edit profile"`, `text "Save changes"`.
    fn fixture() -> ComponentNode {
        let mut c = stub("dialog", "Edit profile");
        c.props
            .insert("description".into(), "Update your display name.".into());
        c.items.push(extra("text", "Save changes"));
        c
    }

    fn reject_js(html: &str) {
        for bad in [
            "onclick",
            "showModal",
            "<dialog",
            "<form",
            "<script",
            "style=",
            "v-data",
        ] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    #[test]
    fn fixture_matches_react_open_dom() {
        let html = render(&fixture());
        let id = widget_id(&fixture(), "");
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"dialog-overlay\" data-state=\"open\" aria-hidden=\"true\"></div><div data-slot=\"dialog-content\" data-state=\"open\" role=\"dialog\" aria-labelledby=\"{id}dialog-title\" aria-describedby=\"{id}dialog-description\"><div data-slot=\"dialog-header\"><h2 data-slot=\"dialog-title\" id=\"{id}dialog-title\">Edit profile</h2><p data-slot=\"dialog-description\" id=\"{id}dialog-description\">Update your display name.</p></div><div data-slot=\"dialog-footer\"><button type=\"button\" data-slot=\"button\" data-variant=\"primary\" disabled>Save changes</button></div><button type=\"button\" data-slot=\"dialog-close\" disabled>{CROSS}<span>Close</span></button></div>"
            )
        );
        reject_js(&html);
    }

    #[test]
    fn defaults_without_description_or_action() {
        let html = render(&stub("dialog", "Hello"));
        assert!(!html.contains("dialog-description"));
        assert!(!html.contains("aria-describedby"));
        assert!(html.contains("disabled>Continue</button>"));
        reject_js(&html);
    }

    #[test]
    fn trigger_item_renders_closed_popover_with_working_close() {
        let mut c = fixture();
        c.items.push(extra("trigger", "Edit"));
        let html = render(&c);
        let tid = widget_id(&c, "dialog-trigger");
        let pid = widget_id(&c, "dialog");
        assert!(html.starts_with(&format!(
            "<button type=\"button\" id=\"{tid}\" data-slot=\"button\" data-variant=\"outline\" popovertarget=\"{pid}\" aria-haspopup=\"dialog\">Edit</button><div id=\"{pid}\" popover=\"auto\" data-slot=\"dialog-content\" role=\"dialog\" aria-labelledby="
        )));
        assert!(!html.contains("dialog-overlay"), "scrim is ::backdrop");
        assert!(!html.contains("data-state=\"open\""));
        assert!(html.contains(&format!(
            "<button type=\"button\" data-slot=\"dialog-close\" popovertarget=\"{pid}\" popovertargetaction=\"hide\">{CROSS}<span>Close</span></button></div>"
        )));
        // Saving needs JS: the action keeps the native disabled button.
        assert!(html.contains("data-variant=\"primary\" disabled>Save changes</button>"));
        reject_js(&html);
    }

    #[test]
    fn open_props_choose_the_mode() {
        let mut c = fixture();
        c.props.insert("open".into(), "false".into());
        assert!(render(&c).contains("aria-haspopup=\"dialog\">Open</button>"));
        c.items.push(extra("trigger", "Edit"));
        c.props.insert("open".into(), "true".into());
        assert_eq!(render(&c), render(&fixture()));
    }

    #[test]
    fn chrome_closed_mode_hides_until_open_with_backdrop() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"dialog-content\"][popover]:not(:popover-open) { display: none; }"
        ));
        assert!(css.contains("[data-slot=\"dialog-content\"][popover]:popover-open {\n  position: fixed; inset: 0; margin: auto; translate: none;"));
        assert!(css.contains("[data-slot=\"dialog-content\"][popover]::backdrop {"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || reject_js(&render(&fixture())));
    }

    #[test]
    fn chrome_is_fixed_overlay_and_centred_panel() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"dialog-overlay\"] {\n  position: fixed; inset: 0; z-index: 50;"
        ));
        assert!(css.contains(
            "[data-slot=\"dialog-content\"]:not(:has(> [data-slot=\"lightbox\"])) {\n  position: fixed; inset: 0; z-index: 50; margin: auto;"
        ));
        // The open dialog stays inside its catalog specimen instead of covering the page.
        assert!(css.contains(
            "[data-slot=\"catalog-canvas\"]:has([data-slot=\"dialog-overlay\"]) {\n  position: relative; contain: layout paint; min-height: 16rem;"
        ));
        assert!(!css.contains("[data-slot=\"dialog-trigger\"]"));
        assert!(!css.contains("dialog[data-slot=\"dialog-content\"]"));
    }
}
