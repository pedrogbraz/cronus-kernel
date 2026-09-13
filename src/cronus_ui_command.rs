//! Dedicated Command renderer. Static palette:
//! `<div data-slot="command"><input data-slot="command-input" /><div data-slot="command-list">`
//! plus each extra text as `<div data-slot="command-item">`.
//! Not interact `popover("command")` (`<details>` SURF box).

use crate::cronus_ui_kit::{choice_texts, esc, item, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let placeholder = placeholder_of(comp);
    let items = command_items(comp)
        .into_iter()
        .map(|t| format!("<div data-slot=\"command-item\">{t}</div>"))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"command\"><input data-slot=\"command-input\" type=\"text\" placeholder=\"{placeholder}\" /><div data-slot=\"command-list\">{items}</div></div>"
    )
}

fn placeholder_of(comp: &ComponentNode) -> String {
    if let Some(v) = comp.props.get("placeholder").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    if let Some(t) = item(comp, "placeholder").filter(|s| !s.is_empty()) {
        return esc(t);
    }
    for kind in ["label", "title"] {
        if let Some(t) = item(comp, kind).filter(|s| !s.is_empty()) {
            return esc(t);
        }
    }
    "Search…".into()
}

fn command_items(comp: &ComponentNode) -> Vec<String> {
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
    fn root_is_input_and_list_not_popover_details() {
        let html = render(&palette("Search", &["Calendar", "Profile"]));
        assert!(html.starts_with("<div data-slot=\"command\">"));
        assert!(html.contains(
            "<input data-slot=\"command-input\" type=\"text\" placeholder=\"Search\" />"
        ));
        assert!(html.contains("<div data-slot=\"command-list\">"));
        assert!(html.contains("<div data-slot=\"command-item\">Calendar</div>"));
        assert!(html.contains("<div data-slot=\"command-item\">Profile</div>"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"command\"><input data-slot=\"command-input\" type=\"text\" placeholder=\"Search\" /><div data-slot=\"command-list\"><div data-slot=\"command-item\">Calendar</div><div data-slot=\"command-item\">Profile</div></div></div>"
        );
    }

    #[test]
    fn label_is_placeholder_not_an_item() {
        let html = render(&palette("Search", &["Calendar", "Profile"]));
        assert!(html.contains("placeholder=\"Search\""));
        assert_eq!(html.matches("data-slot=\"command-item\"").count(), 2);
        for chunk in html.split("data-slot=\"command-item\"").skip(1) {
            assert!(
                !chunk.contains(">Search</div>"),
                "Search leaked as item: {html}"
            );
        }
        reject_interact(&html);
    }

    #[test]
    fn extra_text_items_become_command_items() {
        let mut c = stub("command", "Search");
        c.items.push(extra("text", "Calendar"));
        c.items.push(extra("text", "Profile"));
        let html = render(&c);
        assert!(html.contains("placeholder=\"Search\""));
        assert!(html.contains("<div data-slot=\"command-item\">Calendar</div>"));
        assert!(html.contains("<div data-slot=\"command-item\">Profile</div>"));
        assert_eq!(html.matches("data-slot=\"command-item\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn label_only_keeps_empty_list() {
        let html = render(&stub("command", "Search"));
        assert!(html.contains(
            "<input data-slot=\"command-input\" type=\"text\" placeholder=\"Search\" />"
        ));
        assert!(html.contains("<div data-slot=\"command-list\"></div>"));
        assert!(!html.contains("data-slot=\"command-item\""));
        reject_interact(&html);
    }

    #[test]
    fn placeholder_from_props() {
        let mut c = palette("Search", &["Calendar"]);
        c.props.insert("placeholder".into(), "Type a command…".into());
        let html = render(&c);
        assert!(html.contains("placeholder=\"Type a command…\""));
        assert!(html.contains("<div data-slot=\"command-item\">Calendar</div>"));
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
        assert!(interact.contains("position:absolute;z-index:20"));
        assert!(!interact.contains("data-slot=\"command-input\""));
        assert!(!interact.contains("data-slot=\"command-list\""));
        assert!(html.contains("data-slot=\"command-input\""));
        assert!(html.contains("data-slot=\"command-list\""));
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
        assert!(css.contains("[data-slot=\"command\"]"));
        assert!(css.contains("[data-slot=\"command-input\"]"));
        assert!(css.contains("[data-slot=\"command-list\"]"));
        assert!(css.contains("[data-slot=\"command-item\"]"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("max-height: 20rem"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
