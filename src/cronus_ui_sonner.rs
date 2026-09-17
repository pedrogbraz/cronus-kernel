//! Dedicated Sonner renderer. Family name is `sonner`; React `<Toaster />`
//! renders `<div data-slot="toaster">` around Sonner's live-region
//! `<section aria-label="Notifications …" tabindex="-1" aria-live="polite">`,
//! which stays empty until `toast()` is called. The idle audit state has no
//! toasts, so the kernel emits the same empty toaster (0px tall).
//!
//! `action "…"` items are the docs' firing buttons (outline `button` slots, in
//! a row after the toaster). Each one's `description:"…"` is its toast message
//! and `type:success|error|info|warning` its Sonner `data-type`. Zero JS: the
//! toast is Sonner's `<li data-sonner-toast>` (icon + `data-content` >
//! `data-title`) rendered as a native `popover="manual"` inside the toaster's
//! `<ol data-sonner-toaster>`; the button's `popovertarget` shows it at
//! Sonner's bottom-right slot with its slide-up entrance, and a second click
//! (or Esc) hides it. Gaps: no 4s auto-dismiss, no stacking (one popover
//! shows at a time), no swipe.
//! Not interact `popover("sonner")`.

use crate::cronus_ui_kit::{attr_nonempty, esc, label_of, widget_id};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = aria_label(comp).unwrap_or_else(|| label_of(comp));
    let actions: Vec<_> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "action" && !i.text.is_empty())
        .collect();
    let mut toasts = String::new();
    let mut buttons = String::new();
    for (n, a) in actions.iter().enumerate() {
        let id = widget_id(comp, &format!("toast-{}", n + 1));
        let ty = match a.config.get("type").map(|s| s.trim()) {
            Some("success") => "success",
            Some("error") => "error",
            Some("info") => "info",
            Some("warning") => "warning",
            _ => "default",
        };
        let icon = match ty {
            "success" => crate::cronus_ui_icons::svg_or_empty("circle-check"),
            "error" => crate::cronus_ui_icons::svg_or_empty("circle-alert"),
            "info" => crate::cronus_ui_icons::svg_or_empty("info"),
            "warning" => crate::cronus_ui_icons::svg_or_empty("triangle-alert"),
            _ => String::new(),
        };
        let icon = if icon.is_empty() {
            String::new()
        } else {
            format!("<div data-icon=\"\">{icon}</div>")
        };
        let message = a
            .config
            .get("description")
            .filter(|d| !d.trim().is_empty())
            .map(|d| esc(d))
            .unwrap_or_else(|| esc(&a.text));
        toasts.push_str(&format!(
            "<li id=\"{id}\" popover=\"manual\" role=\"status\" data-sonner-toast=\"\" data-styled=\"true\" data-mounted=\"true\" data-type=\"{ty}\" data-y-position=\"bottom\" data-x-position=\"right\">{icon}<div data-content=\"\"><div data-title=\"\">{message}</div></div></li>"
        ));
        buttons.push_str(&format!(
            "<button type=\"button\" data-slot=\"button\" data-variant=\"outline\" popovertarget=\"{id}\">{}</button>",
            esc(&a.text)
        ));
    }
    let list = if toasts.is_empty() {
        String::new()
    } else {
        format!(
            "<ol dir=\"ltr\" tabindex=\"-1\" data-sonner-toaster=\"\" data-y-position=\"bottom\" data-x-position=\"right\">{toasts}</ol>"
        )
    };
    let row = if buttons.is_empty() {
        String::new()
    } else {
        format!("<div>{buttons}</div>")
    };
    format!(
        "<div data-slot=\"toaster\"><section aria-label=\"{label}\" tabindex=\"-1\" aria-live=\"polite\" aria-relevant=\"additions text\" aria-atomic=\"false\">{list}</section></div>{row}"
    )
}

fn aria_label(comp: &ComponentNode) -> Option<String> {
    attr_nonempty(comp, "aria-label").map(|s| esc(s))
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
        assert!(!html.contains("<details"));
        assert!(!html.contains("<summary"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("role=\"dialog\""));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
    }

    /// wave1t: React idle Toaster has no `sonner` slot and no toast; label
    /// text must not become visible content (it is the region's name).
    #[test]
    fn root_is_empty_toaster_region_like_react_idle() {
        let html = render(&stub("sonner", "Notifications"));
        assert_eq!(
            html,
            "<div data-slot=\"toaster\"><section aria-label=\"Notifications\" tabindex=\"-1\" aria-live=\"polite\" aria-relevant=\"additions text\" aria-atomic=\"false\"></section></div>"
        );
        assert!(!html.contains("data-slot=\"sonner\""));
        assert!(!html.contains("data-slot=\"toast\""));
        assert!(!html.contains(">Notifications<"));
        reject_interact(&html);
    }

    #[test]
    fn aria_label_from_item_config_wins_over_label() {
        let mut c = stub("sonner", "Saved");
        c.items[0]
            .config
            .insert("aria-label".into(), "Alerts & more".into());
        let html = render(&c);
        assert!(html.contains("<section aria-label=\"Alerts &amp; more\""));
        assert!(!html.contains(">Saved<"));
        reject_interact(&html);
    }

    #[test]
    fn extra_text_does_not_add_toasts() {
        let mut c = stub("sonner", "Saved");
        c.items.push(extra("text", "Undo"));
        let html = render(&c);
        assert!(!html.contains("Undo"));
        assert!(!html.contains("data-slot=\"toast\""));
        assert!(!html.contains("data-sonner-toast"));
        reject_interact(&html);
    }

    #[test]
    fn action_items_fire_native_popover_toasts() {
        let mut c = stub("sonner", "Notifications");
        let mut a = extra("action", "Default");
        a.config
            .insert("description".into(), "Event has been created.".into());
        c.items.push(a);
        let mut b = extra("action", "Success");
        b.config.insert("type".into(), "success".into());
        b.config
            .insert("description".into(), "Changes saved successfully.".into());
        c.items.push(b);
        let html = render(&c);
        let t1 = widget_id(&c, "toast-1");
        let t2 = widget_id(&c, "toast-2");
        assert!(html.contains(&format!(
            "<ol dir=\"ltr\" tabindex=\"-1\" data-sonner-toaster=\"\" data-y-position=\"bottom\" data-x-position=\"right\"><li id=\"{t1}\" popover=\"manual\" role=\"status\" data-sonner-toast=\"\" data-styled=\"true\" data-mounted=\"true\" data-type=\"default\" data-y-position=\"bottom\" data-x-position=\"right\"><div data-content=\"\"><div data-title=\"\">Event has been created.</div></div></li><li id=\"{t2}\" popover=\"manual\" role=\"status\" data-sonner-toast=\"\" data-styled=\"true\" data-mounted=\"true\" data-type=\"success\" data-y-position=\"bottom\" data-x-position=\"right\"><div data-icon=\"\"><svg "
        )), "{html}");
        assert!(html.contains("data-icon=\"circle-check\""));
        assert!(html.ends_with(&format!(
            "</section></div><div><button type=\"button\" data-slot=\"button\" data-variant=\"outline\" popovertarget=\"{t1}\">Default</button><button type=\"button\" data-slot=\"button\" data-variant=\"outline\" popovertarget=\"{t2}\">Success</button></div>"
        )));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_popover_surf() {
        let c = stub("sonner", "Saved");
        let html = render(&c);
        assert!(html.contains("data-slot=\"toaster\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("sonner", "Saved"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_toaster_is_in_flow_not_fixed() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"toaster\"] { display: block; }"));
        assert!(!css.contains("[data-slot=\"sonner\"] {"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        // Sonner toast geometry (width 356px, padding 16px, 13px text), bottom-right.
        assert!(css.contains("[data-sonner-toast][popover]:popover-open {\n  position: fixed; inset: auto 24px 24px auto;"));
        assert!(css.contains(
            "min-width: 356px; max-width: 356px; padding: 16px; box-sizing: border-box;"
        ));
        assert!(css.contains("animation: cui-sonner-in 400ms var(--ease-out-quart) both;"));
    }
}
