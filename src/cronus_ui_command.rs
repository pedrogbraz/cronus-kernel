//! Dedicated Command renderer. DOM mirrors React/cmdk:
//! `<div data-slot="command">` > visually-hidden `<label>` (accessible name) >
//! `<div data-slot="command-input-wrapper">` (search icon + `command-input`) >
//! `<div data-slot="command-list" role="listbox">` > sizer `<div>` >
//! `command-item[role=option]`. The first item carries cmdk's initial highlight
//! (`data-selected="true"`). Zero JS: cmdk filtering needs JS, so the input is
//! the same native `<input>` rendered `disabled`, with React's undimmed idle look.
//!
//! Emitted source order is `label` (widget name), then the placeholder `text`
//! when the fixture has one, then one `text` per item. A first text equal to the
//! label or ending in an ellipsis is the placeholder, not an item.
//! Not interact `popover("command")` (`<details>` SURF box).

use crate::cronus_ui_kit::{choice_texts, esc, item, widget_id};
use crate::parser::ComponentNode;

const SEARCH_ICON: &str = concat!(
    "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" ",
    "stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" focusable=\"false\">",
    "<circle cx=\"11\" cy=\"11\" r=\"8\" /><path d=\"m21 21-4.3-4.3\" /></svg>",
);

pub fn render(comp: &ComponentNode) -> String {
    let parts = parts_of(comp);
    let input_id = widget_id(comp, "input");
    let items = parts
        .items
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let selected = if i == 0 { "true" } else { "false" };
            format!(
                "<div data-slot=\"command-item\" role=\"option\" aria-selected=\"{selected}\" data-selected=\"{selected}\">{t}</div>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"command\"><label for=\"{input_id}\">{label}</label><div data-slot=\"command-input-wrapper\">{SEARCH_ICON}<input data-slot=\"command-input\" id=\"{input_id}\" type=\"text\" placeholder=\"{placeholder}\" role=\"combobox\" aria-autocomplete=\"list\" aria-expanded=\"true\" autocomplete=\"off\" spellcheck=\"false\" disabled /></div><div data-slot=\"command-list\" role=\"listbox\" aria-label=\"Suggestions\"><div>{items}</div></div></div>",
        label = parts.label,
        placeholder = parts.placeholder,
    )
}

struct Parts {
    label: String,
    placeholder: String,
    items: Vec<String>,
}

fn parts_of(comp: &ComponentNode) -> Parts {
    let label = ["label", "title"]
        .iter()
        .find_map(|k| item(comp, k).filter(|s| !s.is_empty()))
        .map(esc)
        .unwrap_or_else(|| "Search…".into());
    let choices = choice_texts(comp);
    let mut rest: Vec<String> = if choices.is_empty() {
        comp.items
            .iter()
            .filter(|i| i.item_type == "text" && !i.text.is_empty())
            .map(|i| esc(&i.text))
            .collect()
    } else {
        Vec::new()
    };
    let explicit = comp
        .props
        .get("placeholder")
        .map(String::as_str)
        .or_else(|| item(comp, "placeholder"))
        .filter(|s| !s.is_empty())
        .map(esc);
    let placeholder = match explicit {
        Some(p) => p,
        None => match rest.first() {
            Some(first) if *first == label || first.ends_with('…') || first.ends_with("...") => {
                rest.remove(0)
            }
            _ => label.clone(),
        },
    };
    let items = if choices.is_empty() { rest } else { choices };
    Parts {
        label,
        placeholder,
        items,
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

    fn palette(label: &str, items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("command", label);
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<details"));
        assert!(!html.contains("<summary"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("position:absolute;z-index:20"));
        assert!(!html.contains("role=\"dialog\""));
        assert!(!html.contains("max-width:28rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("popover("));
    }

    #[test]
    fn emitted_fixture_splits_label_placeholder_and_items() {
        let mut c = stub("command", "Command menu");
        c.items.push(extra("text", "Type a command…"));
        c.items.push(extra("text", "Calendar"));
        c.items.push(extra("text", "Search"));
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"command\"><label for=\"cui-command-input\">Command menu</label><div data-slot=\"command-input-wrapper\">{SEARCH_ICON}<input data-slot=\"command-input\" id=\"cui-command-input\" type=\"text\" placeholder=\"Type a command…\" role=\"combobox\" aria-autocomplete=\"list\" aria-expanded=\"true\" autocomplete=\"off\" spellcheck=\"false\" disabled /></div><div data-slot=\"command-list\" role=\"listbox\" aria-label=\"Suggestions\"><div><div data-slot=\"command-item\" role=\"option\" aria-selected=\"true\" data-selected=\"true\">Calendar</div><div data-slot=\"command-item\" role=\"option\" aria-selected=\"false\" data-selected=\"false\">Search</div></div></div></div>"
            )
        );
        assert!(!html.contains("command-item\" role=\"option\" aria-selected=\"true\" data-selected=\"true\">Type a command"));
        reject_interact(&html);
    }

    #[test]
    fn item_kinds_are_items_and_label_is_placeholder() {
        let html = render(&palette("Search", &["Calendar", "Profile"]));
        assert!(html.contains("placeholder=\"Search\""));
        assert!(html.contains("<label for=\"cui-command-input\">Search</label>"));
        assert_eq!(html.matches("data-slot=\"command-item\"").count(), 2);
        assert!(html.contains("data-selected=\"true\">Calendar</div>"));
        assert!(html.contains("data-selected=\"false\">Profile</div>"));
        reject_interact(&html);
    }

    #[test]
    fn plain_texts_without_placeholder_are_items() {
        let mut c = stub("command", "Search");
        c.items.push(extra("text", "Calendar"));
        c.items.push(extra("text", "Profile"));
        let html = render(&c);
        assert!(html.contains("placeholder=\"Search\""));
        assert_eq!(html.matches("data-slot=\"command-item\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn label_only_keeps_empty_list() {
        let html = render(&stub("command", "Search"));
        assert!(html.contains("placeholder=\"Search\""));
        assert!(html.contains("aria-label=\"Suggestions\"><div></div></div>"));
        assert!(!html.contains("data-slot=\"command-item\""));
        reject_interact(&html);
    }

    #[test]
    fn placeholder_from_props() {
        let mut c = palette("Search", &["Calendar"]);
        c.props.insert("placeholder".into(), "Type a command…".into());
        let html = render(&c);
        assert!(html.contains("placeholder=\"Type a command…\""));
        assert!(html.contains("data-selected=\"true\">Calendar</div>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_popover_surf() {
        let c = palette("Search", &["Calendar"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("command", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<details data-slot=\"command\""));
        assert!(interact.contains("<summary"));
        assert!(interact.contains("style="));
        assert!(!interact.contains("data-slot=\"command-input\""));
        assert!(html.contains("data-slot=\"command-input\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&palette("Search", &["Calendar"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"command\"] {\n  display: flex; flex-direction: column; overflow: hidden;\n  width: 18rem; height: 12rem;"));
        assert!(css.contains("[data-slot=\"command-input-wrapper\"]"));
        assert!(css.contains("[data-slot=\"command\"] > label {"));
        assert!(css.contains("[data-slot=\"command-list\"]"));
        assert!(css.contains("[data-slot=\"command-item\"][data-selected=\"true\"]"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("max-height: 20rem"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
