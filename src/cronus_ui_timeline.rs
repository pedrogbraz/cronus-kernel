//! Dedicated Timeline renderer. DOM mirrors React:
//! `<ol role="list" data-slot="timeline" aria-label>` of
//! `<li data-slot="timeline-item">` each with `timeline-rail` (dot + connector,
//! none on the last item) and `timeline-body` > `timeline-content` >
//! `timeline-title`. Events are `text`/`item` lines; the `label` names the list.
//! Not interact `timeline()` (`<ol style=BASE>` bordered list) or catalog
//! `display()` SURF `<section>`.

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

const CONNECTOR: &str = "<div data-slot=\"timeline-connector\" aria-hidden=\"true\"></div>";

pub fn render(comp: &ComponentNode) -> String {
    let events = events_of(comp);
    let last = events.len().saturating_sub(1);
    let items = events
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let connector = if i < last { CONNECTOR } else { "" };
            format!(
                "<li data-slot=\"timeline-item\"><div data-slot=\"timeline-rail\"><span data-slot=\"timeline-dot\" data-tone=\"default\"></span>{connector}</div><div data-slot=\"timeline-body\"><div data-slot=\"timeline-content\"><div data-slot=\"timeline-title\">{t}</div></div></div></li>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let aria = comp
        .props
        .get("aria-label")
        .or_else(|| comp.items.iter().find_map(|i| i.config.get("aria-label")))
        .filter(|s| !s.is_empty())
        .map(|v| format!(" aria-label=\"{}\"", esc(v)))
        .unwrap_or_default();
    format!("<ol role=\"list\" data-slot=\"timeline\"{aria}>{items}</ol>")
}

fn events_of(comp: &ComponentNode) -> Vec<String> {
    let events: Vec<String> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "text" | "item") && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if events.is_empty() {
        vec![label_of(comp)]
    } else {
        events
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

    /// Audit fixture: `label "Order history"`, `text` per event, `aria-label:`
    /// on the last text. React: `<ol aria-label="Order history">` with only
    /// "Order placed" / "Order shipped".
    #[test]
    fn fixture_label_names_list_not_an_event() {
        let mut c = stub("timeline", "Order history");
        c.items.push(extra("text", "Order placed"));
        let mut shipped = extra("text", "Order shipped");
        shipped
            .config
            .insert("aria-label".into(), "Order history".into());
        c.items.push(shipped);
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<ol role=\"list\" data-slot=\"timeline\" aria-label=\"Order history\">{}{}</ol>",
                event("Order placed", true),
                event("Order shipped", false)
            )
        );
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
        reject_interact(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("timeline", "A <B> & \"C\""));
        assert!(html
            .contains("<div data-slot=\"timeline-title\">A &lt;B&gt; &amp; &quot;C&quot;</div>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_styled_ol_and_display_surf() {
        let c = feed(&["Shipped", "Delivered"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("timeline", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<ol data-slot=\"timeline\""));
        assert!(interact.contains("style="));
        assert!(!interact.contains("data-slot=\"timeline-item\""));
        assert!(!interact.contains("data-slot=\"timeline-content\""));
        assert!(html.starts_with("<ol role=\"list\" data-slot=\"timeline\">"));
        assert!(html.contains("data-slot=\"timeline-item\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&feed(&["Shipped"]));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"timeline-item\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
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
