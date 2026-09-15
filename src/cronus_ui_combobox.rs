//! Dedicated Combobox renderer. DOM matches React trigger + open listbox:
//! `<button type="button" data-slot="combobox-trigger" role="combobox">` plus
//! `<div data-slot="combobox-content" role="listbox">` option buttons.
//! Not interact `select("combobox")` (`<label><select data-slot="combobox-control">`).

use crate::cronus_ui_kit::{choice_texts, esc, item};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let options = choice_texts(comp);
    let selected = selected_of(comp, &options);
    let trigger = selected.clone().unwrap_or_else(|| placeholder_of(comp));
    let items = options
        .iter()
        .map(|t| {
            let aria = if selected.as_deref() == Some(t.as_str()) {
                "true"
            } else {
                "false"
            };
            format!(
                "<button type=\"button\" data-slot=\"combobox-item\" role=\"option\" aria-selected=\"{aria}\">{t}</button>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let trigger_id = crate::cronus_ui_kit::widget_id(comp, "trigger");
    let pop_id = crate::cronus_ui_kit::widget_id(comp, "list");
    let mut trigger_attrs = format!(
        "type=\"button\" id=\"{trigger_id}\" data-slot=\"combobox-trigger\" role=\"combobox\" aria-expanded=\"false\" aria-haspopup=\"listbox\" popovertarget=\"{pop_id}\""
    );
    if disabled(comp) {
        trigger_attrs.push_str(" disabled");
    }
    format!(
        "<div data-slot=\"combobox\"><button {trigger_attrs}>{trigger}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"combobox-content\" role=\"listbox\" anchor=\"{trigger_id}\">{items}</div></div>"
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
    "Select an option…".into()
}

fn selected_of(comp: &ComponentNode, options: &[String]) -> Option<String> {
    if let Some(v) = comp.props.get("value").filter(|s| !s.is_empty()) {
        let e = esc(v);
        if options.iter().any(|o| o == &e) {
            return Some(e);
        }
    }
    for i in &comp.items {
        if i.text.is_empty() {
            continue;
        }
        if is_true(i.config.get("selected")) || is_true(i.config.get("checked")) {
            let e = esc(&i.text);
            if options.iter().any(|o| o == &e) {
                return Some(e);
            }
        }
    }
    None
}

fn disabled(comp: &ComponentNode) -> bool {
    comp.props.get("disabled").map(|s| s == "true").unwrap_or(false)
}

fn is_true(raw: Option<&String>) -> bool {
    matches!(raw.map(String::as_str), Some("true" | "on" | "1"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn extra(text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "item".into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn combo(placeholder: &str, items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("combobox", placeholder);
        for t in items {
            c.items.push(extra(t));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<select"));
        assert!(!html.contains("</select>"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("<label"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
    }

    #[test]
    fn root_is_trigger_and_open_listbox_not_native_select() {
        let html = render(&combo("Search", &["Ada", "Grace"]));
        assert!(html.starts_with("<div data-slot=\"combobox\">"));
        assert!(html.contains("data-slot=\"combobox-trigger\""));
        assert!(html.contains("role=\"combobox\""));
        assert!(html.contains("aria-haspopup=\"listbox\""));
        assert!(html.contains("popovertarget="));
        assert!(html.contains(">Search</button>"));
        assert!(html.contains("data-slot=\"combobox-content\" role=\"listbox\""));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"combobox-item\" role=\"option\" aria-selected=\"false\">Ada</button>"
        ));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"combobox-item\" role=\"option\" aria-selected=\"false\">Grace</button>"
        ));
        reject_interact(&html);
        assert!(html.contains("popover=\"auto\""));
    }

    #[test]
    fn label_is_placeholder_not_an_option() {
        let html = render(&combo("Search", &["Ada", "Grace"]));
        assert!(html.contains(">Search</button>"));
        assert_eq!(html.matches("role=\"option\"").count(), 2);
        for chunk in html.split("data-slot=\"combobox-item\"").skip(1) {
            assert!(
                !chunk.contains(">Search</button>"),
                "Search leaked as option: {html}"
            );
        }
        reject_interact(&html);
    }

    #[test]
    fn value_selects_matching_option() {
        let mut c = combo("Search", &["Ada", "Grace"]);
        c.props.insert("value".into(), "Grace".into());
        let html = render(&c);
        assert!(html.contains(">Grace</button>"));
        assert!(html.contains("aria-selected=\"false\">Ada</button>"));
        assert!(html.contains("aria-selected=\"true\">Grace</button>"));
        reject_interact(&html);
    }

    #[test]
    fn selected_item_config_marks_option() {
        let mut c = combo("Search", &["Ada", "Grace"]);
        c.items[1]
            .config
            .insert("selected".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(">Ada</button>"));
        assert!(html.contains("aria-selected=\"true\">Ada</button>"));
        assert!(html.contains("aria-selected=\"false\">Grace</button>"));
        reject_interact(&html);
    }

    #[test]
    fn disabled_trigger() {
        let mut c = combo("Search", &["Ada"]);
        c.props.insert("disabled".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(" disabled>Search</button>"));
        reject_interact(&html);
    }

    #[test]
    fn empty_choices_keep_open_listbox() {
        let html = render(&stub("combobox", "Search"));
        assert!(html.contains("data-slot=\"combobox-trigger\""));
        assert!(html.contains("role=\"listbox\""));
        assert!(!html.contains("data-slot=\"combobox-item\""));
        assert!(html.contains(">Search</button>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_native_select() {
        let c = combo("Search", &["Ada", "Grace"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("combobox", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<label data-slot=\"combobox\""));
        assert!(interact.contains("<select data-slot=\"combobox-control\""));
        assert!(interact.contains("style="));
        assert!(!html.contains("combobox-control"));
        assert!(!html.contains("<select"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&combo("Search", &["Ada", "Grace"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"combobox\"]"));
        assert!(css.contains("[data-slot=\"combobox-trigger\"]"));
        assert!(css.contains("[data-slot=\"combobox-content\"]"));
        assert!(css.contains("[data-slot=\"combobox-item\"]"));
        assert!(css.contains("justify-content: space-between"));
        assert!(css.contains("min-width: 8rem"));
        assert!(css.contains("var(--cronus-surface-floating"));
        assert!(css.contains("[data-slot=\"combobox-item\"][aria-selected=\"true\"]"));
        assert!(!css.contains("zinc-"));
    }
}
