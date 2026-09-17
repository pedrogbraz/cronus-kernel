//! Dedicated Timeline renderer. DOM mirrors React:
//! `<ol role="list" data-slot="timeline" aria-label>` of
//! `<li data-slot="timeline-item">` each with `timeline-rail` (dot + connector,
//! none on the last item) and `timeline-body` > `timeline-content` >
//! `timeline-title` (+ `<time data-slot="timeline-time">` and
//! `timeline-description`). Events are `text`/`item` lines; an item's config
//! carries React's props: `tone:primary|success|warning|error` (default:
//! muted disc), `icon:credit-card` (the dot becomes the ring-bordered `size-7`
//! chip holding the glyph), `time:"Jun 21 · 09:24"` + `datetime:"…"`, and
//! `description:"…"`. The `label` names the list; `max-width:md` is the docs'
//! `max-w-md`. Not interact `timeline()` (`<ol style=BASE>` bordered list) or
//! catalog `display()` SURF `<section>`.

use crate::cronus_ui_kit::{attr_nonempty, esc, item_icon, label_of};
use crate::parser::{ComponentItemNode, ComponentNode};

const CONNECTOR: &str = "<div data-slot=\"timeline-connector\" aria-hidden=\"true\"></div>";

struct Event {
    title: String,
    tone: String,
    icon: String,
    time: Option<(String, String)>,
    description: Option<String>,
}

pub fn render(comp: &ComponentNode) -> String {
    let events = events_of(comp);
    let last = events.len().saturating_sub(1);
    let items = events
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let connector = if i < last { CONNECTOR } else { "" };
            let time = e
                .time
                .as_ref()
                .map(|(text, datetime)| {
                    let dt = if datetime.is_empty() {
                        String::new()
                    } else {
                        format!(" datetime=\"{datetime}\"")
                    };
                    format!("<time data-slot=\"timeline-time\"{dt}>{text}</time>")
                })
                .unwrap_or_default();
            let description = e
                .description
                .as_ref()
                .map(|d| format!("<div data-slot=\"timeline-description\">{d}</div>"))
                .unwrap_or_default();
            format!(
                "<li data-slot=\"timeline-item\"><div data-slot=\"timeline-rail\"><span data-slot=\"timeline-dot\" data-tone=\"{}\">{}</span>{connector}</div><div data-slot=\"timeline-body\"><div data-slot=\"timeline-content\"><div data-slot=\"timeline-title\">{}</div>{time}{description}</div></div></li>",
                e.tone, e.icon, e.title
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let aria = attr_nonempty(comp, "aria-label")
        .map(|v| format!(" aria-label=\"{}\"", esc(v)))
        .unwrap_or_default();
    let class = crate::cronus_ui_card::max_width_class(comp)
        .map(|c| format!(" class=\"{c}\""))
        .unwrap_or_default();
    format!("<ol role=\"list\" data-slot=\"timeline\"{class}{aria}>{items}</ol>")
}

fn event_of(i: &ComponentItemNode) -> Event {
    let tone = match i.tone.as_deref().map(str::trim) {
        Some(t @ ("primary" | "success" | "warning" | "error")) => t.to_string(),
        _ => "default".to_string(),
    };
    let time = i.config.get("time").filter(|t| !t.is_empty()).map(|t| {
        (
            esc(t),
            i.config.get("datetime").map(|d| esc(d)).unwrap_or_default(),
        )
    });
    Event {
        title: esc(&i.text),
        tone,
        icon: item_icon(i),
        time,
        description: i
            .config
            .get("description")
            .filter(|d| !d.is_empty())
            .map(|d| esc(d)),
    }
}

fn events_of(comp: &ComponentNode) -> Vec<Event> {
    let events: Vec<Event> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "text" | "item") && !i.text.is_empty())
        .map(event_of)
        .collect();
    if events.is_empty() {
        vec![Event {
            title: label_of(comp),
            tone: "default".into(),
            icon: String::new(),
            time: None,
            description: None,
        }]
    } else {
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn feed(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("timeline", items.first().copied().unwrap_or("Shipped"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn event(t: &str, connector: bool) -> String {
        let c = if connector { CONNECTOR } else { "" };
        format!(
            "<li data-slot=\"timeline-item\"><div data-slot=\"timeline-rail\"><span data-slot=\"timeline-dot\" data-tone=\"default\"></span>{c}</div><div data-slot=\"timeline-body\"><div data-slot=\"timeline-content\"><div data-slot=\"timeline-title\">{t}</div></div></div></li>"
        )
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<section"));
        assert!(!html.contains("<nav"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("border-left:2px solid"));
        assert!(!html.contains("padding:1rem;display:flex;flex-direction:column;gap:0.5rem"));
        assert!(!html.contains("display("));
    }

    #[test]
    fn root_is_ol_of_rail_and_body_items_like_react() {
        let html = render(&feed(&["Shipped", "Delivered"]));
        assert_eq!(
            html,
            format!(
                "<ol role=\"list\" data-slot=\"timeline\">{}{}</ol>",
                event("Shipped", true),
                event("Delivered", false)
            )
        );
        assert_eq!(html.matches("data-slot=\"timeline-connector\"").count(), 1);
        reject_interact(&html);
    }

    /// Emitted audit source: `label "History"` + texts + `aria-label:"History"`.
    #[test]
    fn fixture_label_names_list_not_an_event() {
        let mut c = stub("timeline", "History");
        c.items.push(extra("text", "Shipped"));
        c.items.push(extra("text", "Delivered"));
        c.items
            .last_mut()
            .unwrap()
            .config
            .insert("aria-label".into(), "History".into());
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<ol role=\"list\" data-slot=\"timeline\" aria-label=\"History\">{}{}</ol>",
                event("Shipped", true),
                event("Delivered", false)
            )
        );
        assert!(!html.contains(">History</div>"));
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_one_item() {
        let html = render(&stub("timeline", "Shipped"));
        assert_eq!(
            html,
            format!(
                "<ol role=\"list\" data-slot=\"timeline\">{}</ol>",
                event("Shipped", false)
            )
        );
        assert!(!html.contains("timeline-connector"));
        reject_interact(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("timeline", "A <B> & \"C\""));
        assert!(html.contains("data-slot=\"timeline-title\">A &lt;B&gt; &amp; &quot;C&quot;</div>"));
        reject_interact(&html);
    }

    /// Docs "Activity": tone + icon dots become the ring-bordered chip, and
    /// each event carries its `<time>` and description.
    #[test]
    fn tone_icon_time_and_description_render_react_slots() {
        let mut c = stub("timeline", "Activity");
        c.items.clear();
        let mut placed = extra("item", "Order placed");
        placed.tone = Some("primary".into());
        placed.config.insert("icon".into(), "credit-card".into());
        placed.config.insert("time".into(), "Jun 21 · 09:24".into());
        placed
            .config
            .insert("datetime".into(), "2026-06-21T09:24".into());
        placed
            .config
            .insert("description".into(), "Order #4827 created.".into());
        c.items.push(placed);
        let mut packed = extra("item", "Packed");
        packed.config.insert("icon".into(), "package".into());
        c.items.push(packed);
        c.props.insert("max-width".into(), "md".into());
        let html = render(&c);
        assert!(html.starts_with("<ol role=\"list\" data-slot=\"timeline\" class=\"mw-md\"><li data-slot=\"timeline-item\"><div data-slot=\"timeline-rail\"><span data-slot=\"timeline-dot\" data-tone=\"primary\"><svg "));
        assert!(html.contains("data-icon=\"credit-card\""));
        assert!(html.contains("<div data-slot=\"timeline-title\">Order placed</div><time data-slot=\"timeline-time\" datetime=\"2026-06-21T09:24\">Jun 21 · 09:24</time><div data-slot=\"timeline-description\">Order #4827 created.</div></div>"));
        assert!(html.contains("<span data-slot=\"timeline-dot\" data-tone=\"default\"><svg "));
        assert!(html.contains("<div data-slot=\"timeline-title\">Packed</div></div>"));
        reject_interact(&html);
        let css = include_str!("cronus_ui_css/timeline.css");
        assert!(css.contains("[data-slot=\"timeline-dot\"]:has(> svg) {\n  width: 1.75rem; height: 1.75rem; border: 1px solid var(--cronus-border); background: var(--cronus-surface-raised); color: var(--cronus-fg-secondary);\n}"));
        assert!(css.contains("[data-slot=\"timeline-dot\"][data-tone=\"primary\"]:has(> svg) { border-color: color-mix(in oklab, var(--cronus-primary) 30%, transparent); color: var(--cronus-primary); }"));
        assert!(css.contains("[data-slot=\"timeline-time\"] { font-size: 0.75rem; line-height: 1; color: var(--cronus-fg-tertiary); }"));
        assert!(css.contains("[data-slot=\"timeline-description\"] { font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg-secondary); }"));
    }

    #[test]
    fn skips_interact_styled_ol_and_display_surf() {
        let c = feed(&["Shipped"]);
        let html = render(&c);
        assert!(html.starts_with("<ol role=\"list\" data-slot=\"timeline\">"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&feed(&["Shipped"]));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"timeline-title\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/timeline.css");
        for slot in [
            "timeline",
            "timeline-item",
            "timeline-rail",
            "timeline-dot",
            "timeline-connector",
            "timeline-body",
            "timeline-content",
            "timeline-title",
        ] {
            assert!(css.contains(&format!("[data-slot=\"{slot}\"]")), "{slot}");
        }
        assert!(css.contains("list-style: none; margin: 0; padding: 0;"));
        assert!(!css.contains("[data-slot=\"timeline-item\"]::before"));
        assert!(css.contains("flex-direction: column"));
        assert!(css.contains("var(--cronus-fg-tertiary)"));
        // Wave 1s geometry parity (React measured: item 432x40 pb24, dot 10 at y0,
        // connector 1x2 at y14, title 14px/14px at y2).
        assert!(css.contains(
            "position: relative; display: grid; grid-template-columns: auto minmax(0, 1fr);\n  column-gap: 0.75rem; padding-bottom: 1.5rem;"
        ));
        assert!(css.contains("[data-slot=\"timeline-item\"]:last-child { padding-bottom: 0; }"));
        assert!(css.contains(
            "width: 0.625rem; height: 0.625rem;\n  border-radius: calc(infinity * 1px);"
        ));
        assert!(css.contains("flex: 1 1 0%; width: 1px; margin-top: 0.25rem;"));
        assert!(
            css.contains("[data-slot=\"timeline-body\"] { min-width: 0; padding-top: 0.125rem; }")
        );
        assert!(css.contains(
            "font-size: 0.875rem; font-weight: 500; line-height: 1; color: var(--cronus-fg);"
        ));
        assert!(!css.contains("margin-top: 0.375rem;\n  border-radius: 999px"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
