//! Dedicated NotificationCenter renderer. DOM mirrors React
//! (`PopoverTrigger asChild` + ghost icon `Button`):
//! `<button data-slot="notification-trigger" data-variant="ghost">` (Bell icon +
//! `<span data-slot="notification-badge">` unread count) followed by a native
//! `popover="auto"` `<div data-slot="notification-center" role="dialog">` with a
//! title header and `notification-list > ul > li > notification-row`s.
//! The label is the title; extra texts are rows (native buttons whose action
//! needs JS, so they render `disabled`). Unread count: `unread` prop, or
//! the audit fixture's convention (first row unread). Zero JS: the panel is
//! closed until `popovertarget` opens it (React's fixture clicks the bell and
//! portals the open panel out of the canvas).
//! Not interact `popover("notification-center")` SURF `<details>`.

use crate::cronus_ui_kit::{choice_texts, label_of, texts, widget_id};
use crate::parser::ComponentNode;

const BELL: &str = concat!(
    "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" ",
    "stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" focusable=\"false\">",
    "<path d=\"M10.268 21a2 2 0 0 0 3.464 0\" />",
    "<path d=\"M3.262 15.326A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.673C19.41 13.956 18 12.499 18 8A6 6 0 0 0 6 8c0 4.499-1.411 5.956-2.738 7.326\" />",
    "</svg>",
);

pub fn render(comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let rows = row_texts(comp);
    let unread = unread_count(comp, rows.len());
    let trigger_id = widget_id(comp, "trigger");
    let pop_id = widget_id(comp, "inbox");
    let (trigger_label, badge) = if unread == 0 {
        ("Notifications".to_string(), String::new())
    } else {
        let shown = if unread > 9 { "9+".to_string() } else { unread.to_string() };
        (
            format!("Notifications, {unread} unread"),
            format!(
                "<span data-slot=\"notification-badge\" data-variant=\"primary\" aria-hidden=\"true\">{shown}</span>"
            ),
        )
    };
    let body = if rows.is_empty() {
        "<div data-slot=\"notification-empty\"><p>You're all caught up</p></div>".to_string()
    } else {
        let items = rows
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let is_unread = i < unread;
                let flag = if is_unread { " data-unread=\"\"" } else { "" };
                let dot = if is_unread {
                    "<span data-slot=\"notification-unread-dot\" aria-hidden=\"true\"></span>"
                } else {
                    ""
                };
                format!(
                    "<li><button type=\"button\" data-slot=\"notification-row\"{flag} disabled><span><span><span>{t}</span>{dot}</span></span></button></li>"
                )
            })
            .collect::<String>();
        format!("<div data-slot=\"notification-list\"><ul>{items}</ul></div>")
    };
    format!(
        "<button type=\"button\" id=\"{trigger_id}\" data-slot=\"notification-trigger\" data-variant=\"ghost\" aria-label=\"{trigger_label}\" aria-haspopup=\"dialog\" popovertarget=\"{pop_id}\">{BELL}{badge}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"notification-center\" role=\"dialog\" aria-label=\"{title}\" anchor=\"{trigger_id}\"><div><p>{title}</p></div><div>{body}</div></div>"
    )
}

fn row_texts(comp: &ComponentNode) -> Vec<String> {
    let choices = choice_texts(comp);
    if !choices.is_empty() {
        return choices;
    }
    texts(comp).into_iter().skip(1).collect()
}

fn unread_count(comp: &ComponentNode, rows: usize) -> usize {
    let explicit = comp
        .props
        .get("unread")
        .or_else(|| comp.items.iter().find_map(|i| i.config.get("unread")))
        .and_then(|v| v.parse::<usize>().ok());
    explicit.unwrap_or(usize::from(rows > 0)).min(rows)
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
        assert!(!html.contains("<dialog"));
        assert!(!html.contains("showModal"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("max-width:28rem"));
        assert!(!html.contains("position:absolute;z-index:20"));
    }

    #[test]
    fn emitted_fixture_is_bell_trigger_badge_and_closed_panel() {
        let mut c = stub("notification-center", "Notifications");
        c.items.push(extra("text", "New comment"));
        c.items.push(extra("text", "Payout sent"));
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<button type=\"button\" id=\"cui-notification-center-trigger\" data-slot=\"notification-trigger\" data-variant=\"ghost\" aria-label=\"Notifications, 1 unread\" aria-haspopup=\"dialog\" popovertarget=\"cui-notification-center-inbox\">{BELL}<span data-slot=\"notification-badge\" data-variant=\"primary\" aria-hidden=\"true\">1</span></button><div id=\"cui-notification-center-inbox\" popover=\"auto\" data-slot=\"notification-center\" role=\"dialog\" aria-label=\"Notifications\" anchor=\"cui-notification-center-trigger\"><div><p>Notifications</p></div><div><div data-slot=\"notification-list\"><ul><li><button type=\"button\" data-slot=\"notification-row\" data-unread=\"\" disabled><span><span><span>New comment</span><span data-slot=\"notification-unread-dot\" aria-hidden=\"true\"></span></span></span></button></li><li><button type=\"button\" data-slot=\"notification-row\" disabled><span><span><span>Payout sent</span></span></span></button></li></ul></div></div></div>"
            )
        );
        assert!(!html.contains(">Notifications</button>"));
        reject_interact(&html);
    }

    #[test]
    fn label_only_has_no_badge_and_empty_state() {
        let html = render(&stub("notification-center", "Notifications"));
        assert!(html.contains("aria-label=\"Notifications\" aria-haspopup=\"dialog\""));
        assert!(!html.contains("data-slot=\"notification-badge\""));
        assert!(html.contains("data-slot=\"notification-empty\""));
        assert!(!html.contains("data-slot=\"notification-row\""));
        reject_interact(&html);
    }

    #[test]
    fn unread_prop_caps_at_rows_and_nine_plus() {
        let mut c = stub("notification-center", "Inbox");
        for i in 0..12 {
            c.items.push(extra("item", &format!("Row {i}")));
        }
        c.props.insert("unread".into(), "11".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Notifications, 11 unread\""));
        assert!(html.contains("aria-hidden=\"true\">9+</span></button>"));
        assert_eq!(html.matches("data-unread=\"\"").count(), 11);
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_popover_surf() {
        let mut c = stub("notification-center", "Deployed to production");
        c.items.push(extra("text", "Invite accepted"));
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("notification-center", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<details data-slot=\"notification-center\""));
        assert!(interact.contains("<summary"));
        assert!(interact.contains("style="));
        assert!(!interact.contains("data-slot=\"notification-trigger\""));
        assert!(!html.contains("<details"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let mut c = stub("notification-center", "Notifications");
            c.items.push(extra("text", "New comment"));
            let html = render(&c);
            reject_interact(&html);
            assert!(html.contains("data-slot=\"notification-row\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"notification-trigger\"] {\n  position: relative; display: inline-flex;"));
        assert!(css.contains("[data-slot=\"notification-badge\"] {\n  position: absolute; top: -0.25rem; right: -0.25rem;"));
        assert!(css.contains("[data-slot=\"notification-center\"]:not(:popover-open) { display: none; }"));
        assert!(css.contains("[data-slot=\"notification-row\"]"));
        assert!(css.contains("z-index: 50"));
        assert!(css.contains("width: 20rem"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains("showModal"));
    }
}
