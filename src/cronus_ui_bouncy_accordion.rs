//! Dedicated BouncyAccordion renderer. DOM mirrors React (`<div
//! data-slot="bouncy-accordion">` + hint `<p>` + `<ul>` of rows, each a
//! `bouncy-accordion-trigger` with a 45px title row and a description) with
//! zero JS: every row is a native `<details name="…">` (one open at a time,
//! click the open one to collapse — React's single-open toggle) whose
//! `<summary>` is the trigger. Rows are `item`/`text` lines; `description:`
//! is the body (the title when absent, like the audit fixture), `icon:` a
//! lucide glyph in React's 24px overlay tile. `value:"Title"` opens that
//! row (the first by default, like the fixture); `hint:` replaces React's
//! default hint, `hint:""` drops it. COMPONENT_CHROME keeps the shared 20px
//! outer radius, springs the open row into its own card with a 10px gap
//! (bounce 0.32 / 0.55s as a `linear()` easing) and grows the body with
//! `::details-content`.

use crate::cronus_ui_kit::{attr, esc, instance_id, item_icon, label_of};
use crate::parser::{ComponentItemNode, ComponentNode};

/// React BouncyAccordion `DEFAULT_LABELS.hint`.
const HINT: &str = "Click on items to expand & collapse";

struct Row {
    title: String,
    description: String,
    icon: String,
}

fn row_of(i: &ComponentItemNode) -> Row {
    let title = esc(&i.text);
    Row {
        description: i
            .config
            .get("description")
            .filter(|d| !d.is_empty())
            .map(|d| esc(d))
            .unwrap_or_else(|| title.clone()),
        icon: item_icon(i),
        title,
    }
}

pub fn render(comp: &ComponentNode) -> String {
    let mut rows: Vec<Row> = comp
        .items
        .iter()
        .filter(|i| i.item_type != "label" && i.item_type != "title" && !i.text.is_empty())
        .map(row_of)
        .collect();
    if rows.is_empty() {
        let title = label_of(comp);
        rows.push(Row {
            description: title.clone(),
            icon: String::new(),
            title,
        });
    }
    let open = attr(comp, "value")
        .map(esc)
        .and_then(|v| rows.iter().position(|r| r.title == v))
        .unwrap_or(0);
    let name = instance_id(comp, "bouncy-accordion");
    let hint = match attr(comp, "hint") {
        Some(h) if h.is_empty() => String::new(),
        Some(h) => format!("<p>{}</p>", esc(h)),
        None => format!("<p>{}</p>", esc(HINT)),
    };
    let list = rows
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let open_attr = if i == open { " open" } else { "" };
            let icon = if r.icon.is_empty() {
                String::new()
            } else {
                format!("<span>{}</span>", r.icon)
            };
            format!(
                "<li><details name=\"{name}\"{open_attr}><summary data-slot=\"bouncy-accordion-trigger\"><span>{icon}<span>{}</span></span></summary><span>{}</span></details></li>",
                r.title, r.description
            )
        })
        .collect::<String>();
    format!("<div data-slot=\"bouncy-accordion\">{hint}<ul>{list}</ul></div>")
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

    fn reject_interact(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains(" disabled"));
        assert!(!html.contains("aria-expanded"));
        assert!(!html.contains("zinc-"));
    }

    /// wave1t: fixture emits `label "default"` (fixture id) + texts; the label
    /// is not an item, the description is the title, the first row is open.
    #[test]
    fn rows_from_texts_first_open_matches_react_dom() {
        crate::cronus_ui_kit::reset_instance_ids();
        let mut c = stub("bouncy-accordion", "default");
        c.items.push(extra("text", "Type"));
        c.items.push(extra("text", "Schedule"));
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"bouncy-accordion\"><p>Click on items to expand &amp; collapse</p><ul><li><details name=\"cui-bouncy-accordion-bouncy-accordion\" open><summary data-slot=\"bouncy-accordion-trigger\"><span><span>Type</span></span></summary><span>Type</span></details></li><li><details name=\"cui-bouncy-accordion-bouncy-accordion\"><summary data-slot=\"bouncy-accordion-trigger\"><span><span>Schedule</span></span></summary><span>Schedule</span></details></li></ul></div>"
        );
        assert!(!html.contains("default"));
        assert!(!html.contains("bouncy-accordion-content"));
        reject_interact(&html);
    }

    /// Docs example: icons, descriptions, `defaultValue="schedule"`.
    #[test]
    fn docs_items_open_the_valued_row_with_icon_and_description() {
        let mut c = stub("bouncy-accordion", "Stack");
        c.items.clear();
        let mut t = extra("item", "Type Shit");
        t.config.insert("icon".into(), "book-open".into());
        t.config
            .insert("description".into(), "Fast, accurate typing.".into());
        let mut s = extra("item", "Schedule");
        s.config.insert("icon".into(), "calendar".into());
        s.config
            .insert("description".into(), "Plan tasks with <timelines>.".into());
        c.items.push(t);
        c.items.push(s);
        c.props.insert("value".into(), "Schedule".into());
        let html = render(&c);
        assert_eq!(html.matches("<details").count(), 2);
        assert_eq!(html.matches(" open>").count(), 1);
        assert!(html.contains("<summary data-slot=\"bouncy-accordion-trigger\"><span><span><svg"));
        assert!(html.contains("</svg></span><span>Type Shit</span></span></summary><span>Fast, accurate typing.</span></details></li><li><details name=\"cui-bouncy-accordion-bouncy-accordion"));
        assert!(
            html.contains(" open><summary data-slot=\"bouncy-accordion-trigger\"><span><span><svg")
        );
        assert!(html.contains("<span>Plan tasks with &lt;timelines&gt;.</span>"));
        reject_interact(&html);
    }

    #[test]
    fn hint_prop_replaces_or_drops_the_default_hint() {
        let mut c = stub("bouncy-accordion", "Type");
        c.props.insert("hint".into(), "Tap a <row>".into());
        assert!(render(&c).contains("<p>Tap a &lt;row&gt;</p>"));
        c.props.insert("hint".into(), "".into());
        assert!(!render(&c).contains("<p>"));
    }

    #[test]
    fn label_only_opens_single_row() {
        let html = render(&stub("bouncy-accordion", "Type"));
        assert_eq!(
            html.matches("data-slot=\"bouncy-accordion-trigger\"")
                .count(),
            1
        );
        assert!(html.contains(" open><summary"));
        reject_interact(&html);
    }

    #[test]
    fn two_stacks_on_one_page_get_distinct_group_names() {
        crate::cronus_ui_kit::reset_instance_ids();
        let a = render(&stub("bouncy-accordion", "Type"));
        let b = render(&stub("bouncy-accordion", "Type"));
        assert!(a.contains("name=\"cui-bouncy-accordion-bouncy-accordion\""));
        assert!(b.contains("name=\"cui-bouncy-accordion-bouncy-accordion-2\""));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("bouncy-accordion", "Type"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_matches_react_stack_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"bouncy-accordion\"] > ul > li {\n  position: relative; overflow: hidden;"
        ));
        assert!(css.contains("[data-slot=\"bouncy-accordion\"] > ul > li:has(> details[open]) {\n  margin-block: 10px; border-radius: 20px;\n}"));
        assert!(css.contains("margin: 0 0 5rem; max-width: 18ch;"));
        assert!(css.contains("font-size: 0.75rem; line-height: 1.25; text-transform: uppercase;"));
        assert!(
            css.contains("[data-slot=\"bouncy-accordion\"] > ul > li:has(+ li > details[open])")
        );
        assert!(
            css.contains("[data-slot=\"bouncy-accordion\"] > ul > li:has(> details[open]) + li")
        );
        // Spring bounce 0.32 / 0.55s (motion) as a linear() easing.
        assert!(css.contains("--cui-bouncy-spring: 550ms linear(0, 0.067, 0.2203, 0.4048, 0.5847, 0.7398, 0.8616, 0.9491, 1.006, 1.0382, 1.0521, 1.0538, 1.0482, 1.039, 1.0289, 1.0195, 1.0117, 1.0057, 1.0015, 0.999, 0.9976, 0.9971, 0.9972, 0.9975, 0.9981, 1);"));
        assert!(css.contains("transition: margin-block var(--cui-bouncy-spring), border-radius var(--cui-bouncy-spring), background-color 150ms var(--cronus-ease);"));
        assert!(css.contains("[data-slot=\"bouncy-accordion\"] details::details-content {\n  display: block; height: 0; overflow: hidden;"));
        assert!(css.contains(
            "[data-slot=\"bouncy-accordion\"] details[open]::details-content { height: auto; }"
        ));
        assert!(css.contains(
            "[data-slot=\"bouncy-accordion-trigger\"] {\n  display: flex; width: 100%; padding: 0 0.5rem;"
        ));
        assert!(css.contains(
            "[data-slot=\"bouncy-accordion-trigger\"]::-webkit-details-marker { display: none; }"
        ));
        assert!(css.contains("[data-slot=\"bouncy-accordion-trigger\"] > span > span:first-child:not(:last-child) {\n  display: grid; width: 1.5rem; height: 1.5rem;"));
        assert!(!css.contains("[data-slot=\"bouncy-accordion-content\"]"));
        assert!(!css.contains("[data-slot=\"bouncy-accordion\"] > ul > li[data-state"));
        assert!(!css.contains("zinc-"));
    }
}
