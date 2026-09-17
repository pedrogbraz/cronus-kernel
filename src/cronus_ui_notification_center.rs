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
//! Docs inbox: `item "Ada mentioned you" description:"…" time:"2m ago"
//! icon:message-square` rows (an `avatar:"GR"` fallback replaces the icon
//! badge; `read:true` rows have no dot and do not count as unread), the
//! header's "Mark all read" link button (`mark-all-read:"…"`, shown while
//! something is unread; JS-only, so `disabled` with React's idle look) and a
//! `footer:"View all notifications"` link button in `notification-footer`.
//! Not interact `popover("notification-center")` SURF `<details>`.

use crate::cronus_ui_kit::{
    attr_nonempty, attr_num, choice_texts, esc, label_of, texts, widget_id,
};
use crate::parser::{ComponentItemNode, ComponentNode};

const BELL: &str = concat!(
    "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" ",
    "stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" focusable=\"false\">",
    "<path d=\"M10.268 21a2 2 0 0 0 3.464 0\" />",
    "<path d=\"M3.262 15.326A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.673C19.41 13.956 18 12.499 18 8A6 6 0 0 0 6 8c0 4.499-1.411 5.956-2.738 7.326\" />",
    "</svg>",
);

struct Row {
    title: String,
    description: Option<String>,
    time: Option<String>,
    icon: String,
    avatar: Option<String>,
    unread: bool,
}

/// Rich `item` rows (docs) — `None` when the items carry no row config.
fn rich_rows(comp: &ComponentNode) -> Option<Vec<Row>> {
    let items: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty())
        .collect();
    let rich = items.iter().any(|i| {
        ["description", "time", "icon", "avatar", "read"]
            .iter()
            .any(|k| i.config.contains_key(*k))
    });
    if !rich {
        return None;
    }
    let get = |i: &ComponentItemNode, k: &str| {
        i.config
            .get(k)
            .filter(|v| !v.trim().is_empty())
            .map(|v| esc(v))
    };
    Some(
        items
            .iter()
            .map(|i| Row {
                title: esc(&i.text),
                description: get(i, "description"),
                time: get(i, "time"),
                icon: crate::cronus_ui_kit::item_icon(i),
                avatar: get(i, "avatar"),
                unread: !i
                    .config
                    .get("read")
                    .is_some_and(|v| crate::cronus_ui_kit::truthy(v)),
            })
            .collect(),
    )
}

pub fn render(comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let rows: Vec<Row> = match rich_rows(comp) {
        Some(rows) => rows,
        None => {
            let texts = row_texts(comp);
            let unread = unread_count(comp, texts.len());
            texts
                .into_iter()
                .enumerate()
                .map(|(i, t)| Row {
                    title: t,
                    description: None,
                    time: None,
                    icon: String::new(),
                    avatar: None,
                    unread: i < unread,
                })
                .collect()
        }
    };
    let unread = rows.iter().filter(|r| r.unread).count();
    let trigger_id = widget_id(comp, "trigger");
    let pop_id = widget_id(comp, "inbox");
    let (trigger_label, badge) = if unread == 0 {
        ("Notifications".to_string(), String::new())
    } else {
        let shown = if unread > 9 {
            "9+".to_string()
        } else {
            unread.to_string()
        };
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
            .map(|r| {
                let flag = if r.unread { " data-unread=\"\"" } else { "" };
                let dot = if r.unread {
                    "<span data-slot=\"notification-unread-dot\" aria-hidden=\"true\"></span>"
                } else {
                    ""
                };
                let lead = match (&r.avatar, r.icon.is_empty()) {
                    (Some(a), _) => format!(
                        "<span data-slot=\"avatar\"><span data-slot=\"avatar-fallback\">{a}</span></span>"
                    ),
                    (None, false) => format!(
                        "<span data-slot=\"notification-icon\" aria-hidden=\"true\">{}</span>",
                        r.icon
                    ),
                    (None, true) => String::new(),
                };
                let description = r
                    .description
                    .as_ref()
                    .map(|d| format!("<span>{d}</span>"))
                    .unwrap_or_default();
                let time = r
                    .time
                    .as_ref()
                    .map(|t| format!("<span>{t}</span>"))
                    .unwrap_or_default();
                format!(
                    "<li><button type=\"button\" data-slot=\"notification-row\"{flag} disabled>{lead}<span><span><span>{}</span>{dot}</span>{description}{time}</span></button></li>",
                    r.title
                )
            })
            .collect::<String>();
        format!("<div data-slot=\"notification-list\"><ul>{items}</ul></div>")
    };
    let mark_all = match attr_nonempty(comp, "mark-all-read") {
        Some(m) if unread > 0 => format!(
            "<button type=\"button\" data-slot=\"button\" data-variant=\"link\" data-size=\"sm\" disabled>{}</button>",
            esc(m)
        ),
        _ => String::new(),
    };
    let footer = attr_nonempty(comp, "footer")
        .map(|f| {
            format!(
                "<div data-slot=\"notification-footer\"><button type=\"button\" data-slot=\"button\" data-variant=\"link\" data-size=\"sm\">{}</button></div>",
                esc(f)
            )
        })
        .unwrap_or_default();
    format!(
        "<button type=\"button\" id=\"{trigger_id}\" data-slot=\"notification-trigger\" data-variant=\"ghost\" aria-label=\"{trigger_label}\" aria-haspopup=\"dialog\" popovertarget=\"{pop_id}\">{BELL}{badge}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"notification-center\" role=\"dialog\" aria-label=\"{title}\" anchor=\"{trigger_id}\"><div><p>{title}</p>{mark_all}</div><div>{body}</div>{footer}</div>"
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
    let explicit = attr_num::<usize>(comp, "unread");
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
    fn docs_inbox_rows_footer_and_mark_all_read() {
        let mut c = stub("notification-center", "Notifications");
        c.props
            .insert("mark-all-read".into(), "Mark all read".into());
        c.props
            .insert("footer".into(), "View all notifications".into());
        let mut mention = extra("item", "Ada mentioned you");
        mention
            .config
            .insert("description".into(), "can you review the PR?".into());
        mention.config.insert("time".into(), "2m ago".into());
        mention
            .config
            .insert("icon".into(), "message-square".into());
        c.items.push(mention);
        let mut joined = extra("item", "New teammate joined");
        joined
            .config
            .insert("description".into(), "Grace accepted your invite.".into());
        joined.config.insert("time".into(), "Yesterday".into());
        joined.config.insert("avatar".into(), "GR".into());
        joined.config.insert("read".into(), "true".into());
        c.items.push(joined);
        let html = render(&c);
        assert!(html.contains("aria-label=\"Notifications, 1 unread\""));
        assert!(html.contains("<span data-slot=\"notification-badge\" data-variant=\"primary\" aria-hidden=\"true\">1</span>"));
        assert!(html.contains("<div><p>Notifications</p><button type=\"button\" data-slot=\"button\" data-variant=\"link\" data-size=\"sm\" disabled>Mark all read</button></div>"));
        assert!(html.contains("<li><button type=\"button\" data-slot=\"notification-row\" data-unread=\"\" disabled><span data-slot=\"notification-icon\" aria-hidden=\"true\"><svg "), "{html}");
        assert!(html.contains("data-icon=\"message-square\""));
        assert!(html.contains("</span><span><span><span>Ada mentioned you</span><span data-slot=\"notification-unread-dot\" aria-hidden=\"true\"></span></span><span>can you review the PR?</span><span>2m ago</span></span></button></li>"));
        assert!(html.contains("<li><button type=\"button\" data-slot=\"notification-row\" disabled><span data-slot=\"avatar\"><span data-slot=\"avatar-fallback\">GR</span></span><span><span><span>New teammate joined</span></span><span>Grace accepted your invite.</span><span>Yesterday</span></span></button></li>"));
        assert!(html.ends_with("<div data-slot=\"notification-footer\"><button type=\"button\" data-slot=\"button\" data-variant=\"link\" data-size=\"sm\">View all notifications</button></div></div>"));
        reject_interact(&html);
        // All read: no badge, no mark-all button.
        c.items[1].config.insert("read".into(), "true".into());
        let html = render(&c);
        assert!(!html.contains("notification-badge"));
        assert!(!html.contains("Mark all read"));
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"notification-icon\"] {\n  display: flex; width: 2.25rem; height: 2.25rem;"));
        assert!(css.contains("[data-slot=\"notification-footer\"] {\n  padding: 0.625rem 0.75rem; border-top: 1px solid var(--cronus-border);"));
        assert!(css.contains("[data-slot=\"notification-row\"] > span > span + span {\n  min-width: 0; overflow-wrap: break-word; font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg-secondary);"));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"notification-trigger\"] {\n  position: relative; display: inline-flex;"
        ));
        assert!(css.contains("[data-slot=\"notification-badge\"] {\n  position: absolute; top: -0.25rem; right: -0.25rem;"));
        assert!(css
            .contains("[data-slot=\"notification-center\"]:not(:popover-open) { display: none; }"));
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
