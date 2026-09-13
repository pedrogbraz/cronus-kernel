//! Dedicated Scheduler renderer. Static week table, no JS.
//! `<div data-slot="scheduler"><h2 data-slot="scheduler-title">` plus
//! `<table data-slot="scheduler-grid">` weekday headers.
//! Not interact `calendar("scheduler")` SURF CSS grid of day buttons.

use crate::cronus_ui_kit::{choice_texts, label_of, texts};
use crate::parser::ComponentNode;

const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

pub fn render(comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let heads = WEEKDAYS
        .iter()
        .map(|d| format!("<th scope=\"col\">{d}</th>"))
        .collect::<Vec<_>>()
        .join("");
    let events = events_of(comp);
    let mut cells = String::new();
    for day in 1..=7 {
        let event = events
            .get(day - 1)
            .map(|t| format!("<span data-slot=\"scheduler-event\">{t}</span>"))
            .unwrap_or_default();
        cells.push_str(&format!("<td><span>{day}</span>{event}</td>"));
    }
    format!(
        "<div data-slot=\"scheduler\"><h2 data-slot=\"scheduler-title\">{title}</h2><table data-slot=\"scheduler-grid\" aria-label=\"Event calendar\"><thead data-slot=\"scheduler-weekdays\"><tr>{heads}</tr></thead><tbody><tr>{cells}</tr></tbody></table></div>"
    )
}

fn events_of(comp: &ComponentNode) -> Vec<String> {
    let choices = choice_texts(comp);
    if !choices.is_empty() {
        return choices;
    }
    texts(comp).into_iter().skip(1).collect()
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

    fn week(title: &str, events: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("scheduler", title);
        for n in events {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("grid-template-columns:repeat(7,1fr)"));
        assert!(!html.contains("<button type=\"button\""));
        assert!(!html.contains("<input"));
        assert!(!html.contains("type=\"date\""));
        assert!(!html.contains("calendar("));
        assert!(html.contains("data-slot=\"scheduler-title\""));
        assert!(html.contains("data-slot=\"scheduler-grid\""));
    }

    #[test]
    fn root_is_scheduler_with_title_and_week_grid() {
        let html = render(&stub("scheduler", "March"));
        assert!(html.starts_with("<div data-slot=\"scheduler\">"));
        assert!(html.contains("<h2 data-slot=\"scheduler-title\">March</h2>"));
        assert!(html.contains(
            "<table data-slot=\"scheduler-grid\" aria-label=\"Event calendar\">"
        ));
        assert!(html.contains("<thead data-slot=\"scheduler-weekdays\">"));
        assert!(html.contains("<th scope=\"col\">Sun</th>"));
        assert!(html.contains("<th scope=\"col\">Sat</th>"));
        assert!(html.contains("<td><span>1</span></td>"));
        assert!(html.contains("<td><span>7</span></td>"));
        assert!(!html.contains("<td><span>8</span></td>"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"scheduler\"><h2 data-slot=\"scheduler-title\">March</h2><table data-slot=\"scheduler-grid\" aria-label=\"Event calendar\"><thead data-slot=\"scheduler-weekdays\"><tr><th scope=\"col\">Sun</th><th scope=\"col\">Mon</th><th scope=\"col\">Tue</th><th scope=\"col\">Wed</th><th scope=\"col\">Thu</th><th scope=\"col\">Fri</th><th scope=\"col\">Sat</th></tr></thead><tbody><tr><td><span>1</span></td><td><span>2</span></td><td><span>3</span></td><td><span>4</span></td><td><span>5</span></td><td><span>6</span></td><td><span>7</span></td></tr></tbody></table></div>"
        );
    }

    #[test]
    fn extra_items_become_events() {
        let html = render(&week("March", &["Standup", "Review"]));
        assert!(html.contains("<h2 data-slot=\"scheduler-title\">March</h2>"));
        assert!(html.contains(
            "<td><span>1</span><span data-slot=\"scheduler-event\">Standup</span></td>"
        ));
        assert!(html.contains(
            "<td><span>2</span><span data-slot=\"scheduler-event\">Review</span></td>"
        ));
        assert!(html.contains("<td><span>3</span></td>"));
        assert!(!html.contains("data-slot=\"scheduler-event\">March</span>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_calendar_surf() {
        let c = stub("scheduler", "March");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("scheduler", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("data-slot=\"scheduler\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("grid-template-columns:repeat(7,1fr)"));
        assert!(interact.contains("<button type=\"button\""));
        assert!(!interact.contains("data-slot=\"scheduler-title\""));
        assert!(!interact.contains("data-slot=\"scheduler-grid\""));
        assert!(html.contains("data-slot=\"scheduler-title\""));
        assert!(html.contains("data-slot=\"scheduler-grid\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("scheduler", "March"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"scheduler\"]"));
        assert!(css.contains("[data-slot=\"scheduler-title\"]"));
        assert!(css.contains("[data-slot=\"scheduler-grid\"]"));
        assert!(css.contains("[data-slot=\"scheduler-weekdays\"]"));
        assert!(css.contains("border-collapse: collapse"));
        assert!(css.contains("var(--cronus-fg-tertiary)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-surface-base)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains("showModal"));
    }
}
