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
//! `cronus_ui_kit::overlay_trigger`): a `button` trigger ([`trigger_button`]:
//! outline unless `trigger-variant:` says otherwise, `trigger-icon:` /
//! `trigger-icon-end:` glyphs) with `command="show-modal"` opens
//! `dialog-content` as a native `<dialog>` (focus trap, inert background, Esc
//! and backdrop dismiss; scrim is `::backdrop`, entrance is React's
//! `cronus-pop-in` / `cronus-overlay-in`). Everything the docs wrap in
//! `DialogClose` closes it natively (`command="close"`): `dialog-close`, the
//! `cancel:"…"` outline button and the `action` button.
//!
//! Content: title = label; description = `description:"…"` prop / item config /
//! `description` item; body = `field "Name" value:"…"` items ([`fields_html`]:
//! React's `Label` + `Input` pairs in the docs' `flex-col gap-4 py-2` block);
//! action = `action` item, else the first `text` item, else "Continue";
//! `cancel:"…"` adds the outline button before it; close label = `close` item,
//! else "Close".

use crate::cronus_ui_kit::{
    attr, attr_nonempty, esc, item, label_of, modal_close_attrs, modal_dialog_open,
    modal_open_button, overlay_trigger, widget_id,
};
use crate::parser::ComponentNode;

/// lucide `x` (React `size-4`).
const CROSS: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M18 6 6 18\"></path><path d=\"m6 6 12 12\"></path></svg>";

fn non_empty<'a>(comp: &'a ComponentNode, kind: &str) -> Option<&'a str> {
    item(comp, kind).filter(|t| !t.is_empty())
}

/// Modal trigger: `cronus_ui_kit::modal_open_button` (outline `button` slot,
/// `command="show-modal"`) styled by `trigger-variant:` (React Button
/// variants), `trigger-size:` and `trigger-icon:` / `trigger-icon-end:`
/// (lucide ids) so the docs' `<DialogTrigger asChild><Button …>` renders as
/// declared. Shared by every modal overlay family.
pub fn trigger_button(
    comp: &ComponentNode,
    trigger_id: &str,
    dialog_id: &str,
    label: &str,
) -> String {
    let icon = attr_nonempty(comp, "trigger-icon")
        .map(crate::cronus_ui_icons::svg_or_empty)
        .unwrap_or_default();
    let icon_end = attr_nonempty(comp, "trigger-icon-end")
        .map(crate::cronus_ui_icons::svg_or_empty)
        .unwrap_or_default();
    let mut html = modal_open_button(trigger_id, dialog_id, &format!("{icon}{label}{icon_end}"));
    if let Some(v) = attr_nonempty(comp, "trigger-variant") {
        let v = match v.trim() {
            "primary" => "primary",
            "secondary" => "secondary",
            "ghost" => "ghost",
            "link" => "link",
            "destructive" | "danger" => "destructive",
            _ => "outline",
        };
        html = html.replace("data-variant=\"outline\"", &format!("data-variant=\"{v}\""));
    }
    if let Some(s) = attr_nonempty(comp, "trigger-size") {
        if matches!(s.trim(), "sm" | "md" | "lg" | "icon" | "icon-sm") {
            html = html.replace(
                " commandfor=",
                &format!(" data-size=\"{}\" commandfor=", s.trim()),
            );
        }
    }
    html
}

/// One `<div class="field">` per `field "Label" …` item: React `Label` +
/// `Input` (`value:`, `placeholder:`, `type:`), ids derived from `base_id`.
pub fn fields_html(comp: &ComponentNode, base_id: &str) -> String {
    comp.items
        .iter()
        .filter(|i| i.item_type == "field" && !i.text.is_empty())
        .enumerate()
        .map(|(n, f)| {
            let id = format!("{base_id}-f{}", n + 1);
            let ty = f
                .config
                .get("type")
                .map(String::as_str)
                .filter(|t| !t.trim().is_empty())
                .unwrap_or("text");
            let value = f
                .config
                .get("value")
                .filter(|v| !v.is_empty())
                .map(|v| format!(" value=\"{}\"", esc(v)))
                .unwrap_or_default();
            let placeholder = f
                .config
                .get("placeholder")
                .filter(|v| !v.is_empty())
                .map(|v| format!(" placeholder=\"{}\"", esc(v)))
                .unwrap_or_default();
            format!(
                "<div class=\"field\"><label data-slot=\"label\" for=\"{id}\">{}</label><input data-slot=\"input\" id=\"{id}\" type=\"{}\"{value}{placeholder} /></div>",
                esc(&f.text),
                esc(ty)
            )
        })
        .collect()
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
    let cancel = attr_nonempty(comp, "cancel")
        .or_else(|| non_empty(comp, "cancel"))
        .map(esc);
    let close = non_empty(comp, "close")
        .map(esc)
        .unwrap_or_else(|| "Close".into());
    let title_id = widget_id(comp, "dialog-title");
    let desc_id = widget_id(comp, "dialog-description");
    let pop_id = widget_id(comp, "dialog");
    let (described, description) = match description {
        Some(d) => (
            format!(" aria-describedby=\"{desc_id}\""),
            format!("<p data-slot=\"dialog-description\" id=\"{desc_id}\">{d}</p>"),
        ),
        None => (String::new(), String::new()),
    };
    let fields = fields_html(comp, &pop_id);
    let body_fields = if fields.is_empty() {
        String::new()
    } else {
        format!("<div class=\"fields\">{fields}</div>")
    };
    let trigger = overlay_trigger(comp, "Open");
    // DialogClose semantics: in closed mode the footer buttons close natively.
    let closes = match &trigger {
        Some(_) => modal_close_attrs(&pop_id),
        None => " disabled".to_string(),
    };
    let cancel = cancel
        .map(|c| {
            format!(
                "<button type=\"button\" data-slot=\"button\" data-variant=\"outline\"{closes}>{c}</button>"
            )
        })
        .unwrap_or_default();
    let body = format!(
        "<div data-slot=\"dialog-header\"><h2 data-slot=\"dialog-title\" id=\"{title_id}\">{title}</h2>{description}</div>{body_fields}<div data-slot=\"dialog-footer\">{cancel}<button type=\"button\" data-slot=\"button\" data-variant=\"primary\"{closes}>{action}</button></div>"
    );
    match trigger {
        None => format!(
            "<div data-slot=\"dialog-overlay\" data-state=\"open\" aria-hidden=\"true\"></div><div data-slot=\"dialog-content\" data-state=\"open\" role=\"dialog\" aria-labelledby=\"{title_id}\"{described}>{body}<button type=\"button\" data-slot=\"dialog-close\" disabled>{CROSS}<span>{close}</span></button></div>"
        ),
        Some(trigger) => {
            let trigger_id = widget_id(comp, "dialog-trigger");
            format!(
                "{}{}{body}<button type=\"button\" data-slot=\"dialog-close\"{}>{CROSS}<span>{close}</span></button></dialog>",
                trigger_button(comp, &trigger_id, &pop_id, &trigger),
                modal_dialog_open(&pop_id, "dialog-content", "dialog", &title_id, &described, true),
                modal_close_attrs(&pop_id),
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
            "showModal(",
            "popovertarget",
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
    fn trigger_item_renders_closed_modal_dialog_with_working_close() {
        let mut c = fixture();
        c.items.push(extra("trigger", "Edit"));
        let html = render(&c);
        let tid = widget_id(&c, "dialog-trigger");
        let pid = widget_id(&c, "dialog");
        assert!(html.starts_with(&format!(
            "<button type=\"button\" id=\"{tid}\" data-slot=\"button\" data-variant=\"outline\" commandfor=\"{pid}\" command=\"show-modal\" aria-haspopup=\"dialog\">Edit</button><dialog id=\"{pid}\" data-slot=\"dialog-content\" role=\"dialog\" aria-modal=\"true\" aria-labelledby="
        )));
        assert!(html.contains("closedby=\"any\""));
        assert!(!html.contains("dialog-overlay"), "scrim is ::backdrop");
        assert!(!html.contains("data-state=\"open\""));
        assert!(!html.contains("popover"));
        assert!(html.contains(&format!(
            "<button type=\"button\" data-slot=\"dialog-close\" commandfor=\"{pid}\" command=\"close\">{CROSS}<span>Close</span></button></dialog>"
        )));
        // The docs wrap the footer buttons in DialogClose: they close natively.
        assert!(html.contains(&format!(
            "data-variant=\"primary\" commandfor=\"{pid}\" command=\"close\">Save changes</button>"
        )));
        reject_js(&html);
    }

    #[test]
    fn docs_dialog_has_styled_trigger_fields_and_cancel() {
        let mut c = stub("dialog", "Edit profile");
        c.props.insert("trigger".into(), "Edit profile".into());
        c.props.insert("trigger-variant".into(), "primary".into());
        c.props.insert("cancel".into(), "Cancel".into());
        c.props.insert(
            "description".into(),
            "Update your display name. Changes are saved when you confirm.".into(),
        );
        let mut name = extra("field", "Name");
        name.config.insert("value".into(), "Ada Lovelace".into());
        let mut user = extra("field", "Username");
        user.config.insert("value".into(), "ada".into());
        c.items.push(name);
        c.items.push(user);
        c.items.push(extra("action", "Save changes"));
        let html = render(&c);
        let pid = widget_id(&c, "dialog");
        assert!(html.contains("data-slot=\"button\" data-variant=\"primary\" commandfor=\"{pid}\" command=\"show-modal\" aria-haspopup=\"dialog\">Edit profile</button>".replace("{pid}", &pid).as_str()));
        assert!(html.contains(&format!(
            "</div><div class=\"fields\"><div class=\"field\"><label data-slot=\"label\" for=\"{pid}-f1\">Name</label><input data-slot=\"input\" id=\"{pid}-f1\" type=\"text\" value=\"Ada Lovelace\" /></div><div class=\"field\"><label data-slot=\"label\" for=\"{pid}-f2\">Username</label><input data-slot=\"input\" id=\"{pid}-f2\" type=\"text\" value=\"ada\" /></div></div><div data-slot=\"dialog-footer\"><button type=\"button\" data-slot=\"button\" data-variant=\"outline\" commandfor=\"{pid}\" command=\"close\">Cancel</button><button type=\"button\" data-slot=\"button\" data-variant=\"primary\" commandfor=\"{pid}\" command=\"close\">Save changes</button></div>"
        )), "{html}");
        reject_js(&html);
    }

    #[test]
    fn trigger_button_takes_icons_and_sizes() {
        let mut c = stub("dialog", "Delete");
        c.props
            .insert("trigger-variant".into(), "destructive".into());
        c.props.insert("trigger-icon".into(), "trash-2".into());
        c.props.insert("trigger-size".into(), "sm".into());
        let html = trigger_button(&c, "t", "d", "Delete account");
        assert_eq!(
            html,
            format!(
                "<button type=\"button\" id=\"t\" data-slot=\"button\" data-variant=\"destructive\" data-size=\"sm\" commandfor=\"d\" command=\"show-modal\" aria-haspopup=\"dialog\">{}Delete account</button>",
                crate::cronus_ui_icons::svg_or_empty("trash-2")
            )
        );
        c.props.insert("trigger-size".into(), "huge".into());
        assert!(!trigger_button(&c, "t", "d", "x").contains("data-size"));
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
    fn chrome_closed_mode_is_native_modal_dialog() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"dialog-content\"]:modal {\n  position: fixed; inset: 0; margin: auto; translate: none;"));
        assert!(css.contains("[data-slot=\"dialog-content\"]::backdrop {"));
        assert!(!css.contains("[data-slot=\"dialog-content\"][popover]"));
        // React's enter animations (data-[state=open]) on the native dialog + backdrop.
        assert!(css.contains("animation: cronus-pop-in 200ms var(--ease-out-quart) both;"));
        assert!(css.contains("[data-slot=\"dialog-content\"]:modal::backdrop {\n  animation: cronus-overlay-in 200ms var(--ease-out-quart);"));
        assert!(css.contains("[data-slot=\"dialog-content\"] > .fields {\n  display: flex; flex-direction: column; gap: 1rem; padding: 0.5rem 0;"));
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
        assert!(!css.contains("dialog[data-slot=\"dialog-content\"] {"));
        // The panel rule sets `display`, which would override the UA's hidden closed dialog.
        assert!(css.contains("dialog[data-slot=\"dialog-content\"]:not([open]) { display: none; }"));
    }
}
