//! Dedicated Autocomplete renderer. DOM matches React:
//! `<div data-slot="autocomplete"><input data-slot="autocomplete-input">`
//! plus always-open `<div data-slot="autocomplete-content">` options from extra texts.
//! Not interact `select("autocomplete")` (`<label><select data-slot="autocomplete-control">`).

use crate::cronus_ui_kit::{choice_texts, esc, item, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let options = extra_texts(comp);
    let selected = selected_of(comp, &options);
    let placeholder = placeholder_of(comp);
    let value = selected
        .clone()
        .or_else(|| attr(comp, "value").filter(|s| !s.is_empty()).map(esc))
        .unwrap_or_default();
    let disabled = flag(comp, "disabled");
    let aria = aria_label_of(comp);

    let items = options
        .iter()
        .map(|t| {
            let aria_sel = if selected.as_deref() == Some(t.as_str()) {
                "true"
            } else {
                "false"
            };
            format!(
                "<button type=\"button\" data-slot=\"autocomplete-item\" role=\"option\" aria-selected=\"{aria_sel}\">{t}</button>"
            )
        })
        .collect::<Vec<_>>()
        .join("");

    let mut field = String::from(
        "data-slot=\"autocomplete-input\" type=\"text\" role=\"combobox\" aria-autocomplete=\"list\" aria-expanded=\"true\" autocomplete=\"off\"",
    );
    if !placeholder.is_empty() {
        field.push_str(&format!(" placeholder=\"{placeholder}\""));
    }
    if !aria.is_empty() {
        field.push_str(&format!(" aria-label=\"{aria}\""));
    }
    if !value.is_empty() {
        field.push_str(&format!(" value=\"{value}\""));
    }
    if disabled {
        field.push_str(" disabled");
    }

    format!(
        "<div data-slot=\"autocomplete\"><input {field} /><div data-slot=\"autocomplete-content\" role=\"listbox\">{items}</div></div>"
    )
}

fn extra_texts(comp: &ComponentNode) -> Vec<String> {
    let choices = choice_texts(comp);
    if !choices.is_empty() {
        return choices;
    }
    texts(comp).into_iter().skip(1).collect()
}

fn placeholder_of(comp: &ComponentNode) -> String {
    if let Some(v) = attr(comp, "placeholder").filter(|s| !s.is_empty()) {
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
    "Type to search…".into()
}

fn aria_label_of(comp: &ComponentNode) -> String {
    if let Some(v) = attr(comp, "aria-label").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    String::new()
}

fn selected_of(comp: &ComponentNode, options: &[String]) -> Option<String> {
    if let Some(v) = attr(comp, "value").filter(|s| !s.is_empty()) {
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

fn attr<'a>(comp: &'a ComponentNode, name: &str) -> Option<&'a str> {
    if let Some(v) = comp.props.get(name) {
        return Some(v.as_str());
    }
    comp.items
        .iter()
        .find_map(|i| i.config.get(name).map(String::as_str))
}

fn flag(comp: &ComponentNode, name: &str) -> bool {
    attr(comp, name).map(|s| s == "true").unwrap_or(false)
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

    fn auto(placeholder: &str, items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("autocomplete", placeholder);
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
    fn root_is_input_and_open_content_not_native_select() {
        let html = render(&auto("Search", &["Ada", "Grace"]));
        assert!(html.starts_with("<div data-slot=\"autocomplete\">"));
        assert!(html.contains(
            "<input data-slot=\"autocomplete-input\" type=\"text\" role=\"combobox\" aria-autocomplete=\"list\" aria-expanded=\"true\" autocomplete=\"off\" placeholder=\"Search\" />"
        ));
        assert!(html.contains("<div data-slot=\"autocomplete-content\" role=\"listbox\">"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"autocomplete-item\" role=\"option\" aria-selected=\"false\">Ada</button>"
        ));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"autocomplete-item\" role=\"option\" aria-selected=\"false\">Grace</button>"
        ));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"autocomplete\"><input data-slot=\"autocomplete-input\" type=\"text\" role=\"combobox\" aria-autocomplete=\"list\" aria-expanded=\"true\" autocomplete=\"off\" placeholder=\"Search\" /><div data-slot=\"autocomplete-content\" role=\"listbox\"><button type=\"button\" data-slot=\"autocomplete-item\" role=\"option\" aria-selected=\"false\">Ada</button><button type=\"button\" data-slot=\"autocomplete-item\" role=\"option\" aria-selected=\"false\">Grace</button></div></div>"
        );
    }

    #[test]
    fn label_is_placeholder_not_an_option() {
        let html = render(&auto("Search", &["Ada", "Grace"]));
        assert!(html.contains("placeholder=\"Search\""));
        assert_eq!(html.matches("role=\"option\"").count(), 2);
        for chunk in html.split("data-slot=\"autocomplete-item\"").skip(1) {
            assert!(
                !chunk.contains(">Search</button>"),
                "Search leaked as option: {html}"
            );
        }
        reject_interact(&html);
    }

    #[test]
    fn extra_text_items_become_options() {
        let mut c = stub("autocomplete", "Search");
        c.items.push(ComponentItemNode {
            item_type: "text".into(),
            text: "Ada".into(),
            link: None,
            tone: None,
            config: Default::default(),
        });
        c.items.push(ComponentItemNode {
            item_type: "text".into(),
            text: "Grace".into(),
            link: None,
            tone: None,
            config: Default::default(),
        });
        let html = render(&c);
        assert!(html.contains("placeholder=\"Search\""));
        assert!(html.contains("aria-selected=\"false\">Ada</button>"));
        assert!(html.contains("aria-selected=\"false\">Grace</button>"));
        assert_eq!(html.matches("data-slot=\"autocomplete-item\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn value_selects_matching_option() {
        let mut c = auto("Search", &["Ada", "Grace"]);
        c.props.insert("value".into(), "Grace".into());
        let html = render(&c);
        assert!(html.contains("value=\"Grace\""));
        assert!(html.contains("aria-selected=\"false\">Ada</button>"));
        assert!(html.contains("aria-selected=\"true\">Grace</button>"));
        reject_interact(&html);
    }

    #[test]
    fn empty_choices_keep_open_content() {
        let html = render(&stub("autocomplete", "Search"));
        assert!(html.contains("data-slot=\"autocomplete-input\""));
        assert!(html.contains("role=\"listbox\""));
        assert!(!html.contains("data-slot=\"autocomplete-item\""));
        assert!(html.contains("placeholder=\"Search\""));
        reject_interact(&html);
    }

    #[test]
    fn disabled_input() {
        let mut c = auto("Search", &["Ada"]);
        c.props.insert("disabled".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(" disabled />"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_native_select() {
        let c = auto("Search", &["Ada", "Grace"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("autocomplete", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<label data-slot=\"autocomplete\""));
        assert!(interact.contains("<select data-slot=\"autocomplete-control\""));
        assert!(interact.contains("style="));
        assert!(!html.contains("autocomplete-control"));
        assert!(!html.contains("<select"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&auto("Search", &["Ada", "Grace"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"autocomplete\"]"));
        assert!(css.contains("[data-slot=\"autocomplete-input\"]"));
        assert!(css.contains("[data-slot=\"autocomplete-content\"]"));
        assert!(css.contains("[data-slot=\"autocomplete-item\"]"));
        assert!(css.contains("min-width: 8rem"));
        assert!(css.contains("var(--cronus-surface-floating"));
        assert!(css.contains("[data-slot=\"autocomplete-item\"][aria-selected=\"true\"]"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
