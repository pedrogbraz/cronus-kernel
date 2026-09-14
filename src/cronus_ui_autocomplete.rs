//! Dedicated Autocomplete renderer. DOM matches React:
//! `<div data-slot="autocomplete"><input data-slot="autocomplete-input" role="combobox">`
//! and, while there is a query, the open Radix/cmdk content React shows on focus:
//! `autocomplete-content` > `autocomplete-command` > `command-list` (listbox) >
//! sizer `<div>` > `command-item` rows (`<span>` label, first row highlighted).
//! React portals the content to `<body>`; the kernel keeps it in place as the
//! wrapper's next sibling inside a plain anchor `<div>` (so `autocomplete` keeps
//! React's empty text and 40px box), absolutely positioned 4px under the input at
//! the input's width (sideOffset 4, align start).
//! Options are filtered statically with React's local `includes` match on the
//! initial value. Live re-filtering, keyboard highlight and picking need JS, so
//! rows are non-interactive `role="option"` divs. Empty query → closed (React).
//! Not interact `select("autocomplete")` (`<label><select data-slot="autocomplete-control">`).

use crate::cronus_ui_kit::{choice_texts, esc, item};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let placeholder = placeholder_of(comp);
    let options = options_of(comp, &placeholder);
    let value = attr(comp, "value").filter(|s| !s.is_empty()).map(esc).unwrap_or_default();
    let disabled = flag(comp, "disabled");
    let aria = aria_label_of(comp);
    let needle = value.to_lowercase();
    let matches: Vec<&String> = options
        .iter()
        .filter(|o| o.to_lowercase().contains(&needle))
        .collect();
    let open = !value.is_empty() && !disabled && !matches.is_empty();
    let list_id = crate::cronus_ui_kit::widget_id(comp, "list");

    let mut field = format!(
        "type=\"text\" role=\"combobox\" autocomplete=\"off\" autocorrect=\"off\" autocapitalize=\"none\" spellcheck=\"false\" aria-autocomplete=\"list\" aria-expanded=\"{open}\""
    );
    if !aria.is_empty() {
        field.push_str(&format!(" aria-label=\"{aria}\""));
    }
    field.push_str(" data-slot=\"autocomplete-input\"");
    if !placeholder.is_empty() {
        field.push_str(&format!(" placeholder=\"{placeholder}\""));
    }
    if !value.is_empty() {
        field.push_str(&format!(" value=\"{value}\""));
    }
    if disabled {
        field.push_str(" disabled");
    }
    if !open {
        return format!("<div data-slot=\"autocomplete\"><input {field} /></div>");
    }
    field.push_str(&format!(" aria-controls=\"{list_id}\""));

    let rows = matches
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let sel = if i == 0 { "true" } else { "false" };
            format!(
                "<div data-slot=\"command-item\" role=\"option\" aria-selected=\"{sel}\" data-selected=\"{sel}\"><span>{t}</span></div>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let list_label = if aria.is_empty() {
        String::new()
    } else {
        format!(" aria-label=\"{aria}\"")
    };
    format!(
        "<div><div data-slot=\"autocomplete\"><input {field} /></div><div data-side=\"bottom\" data-align=\"start\" data-state=\"open\" role=\"dialog\" data-slot=\"autocomplete-content\"><div data-slot=\"autocomplete-command\"><div data-slot=\"command-list\" role=\"listbox\"{list_label} id=\"{list_id}\"><div>{rows}</div></div></div></div></div>"
    )
}

/// Options: `item` choices, else free `text` lines. The emitter also writes the
/// placeholder as the first `text`; it names the field, it is not an option.
fn options_of(comp: &ComponentNode, placeholder: &str) -> Vec<String> {
    let choices = choice_texts(comp);
    if !choices.is_empty() {
        return choices;
    }
    let mut out: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "text" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if out.first().is_some_and(|t| t == placeholder) {
        out.remove(0);
    }
    out
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn node(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn auto(placeholder: &str, items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("autocomplete", placeholder);
        for t in items {
            c.items.push(node("item", t));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<select"));
        assert!(!html.contains("</select>"));
        assert!(!html.contains("data-slot=\"autocomplete-control\""));
        assert!(!html.contains("<label"));
        assert!(!html.contains("<button"));
        assert!(!html.contains(" style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
    }

    #[test]
    fn empty_query_renders_closed_input_only() {
        let html = render(&auto("Search", &["Ada", "Grace"]));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"autocomplete\"><input type=\"text\" role=\"combobox\" autocomplete=\"off\" autocorrect=\"off\" autocapitalize=\"none\" spellcheck=\"false\" aria-autocomplete=\"list\" aria-expanded=\"false\" data-slot=\"autocomplete-input\" placeholder=\"Search\" /></div>"
        );
    }

    /// Emitted fixture: `label/text "Type a city"`, texts Lisbon/Lima/London,
    /// `value:"L"`, `aria-label:"City"` → open cmdk tree, first row highlighted.
    #[test]
    fn query_opens_cmdk_tree_like_react() {
        let mut c = stub("autocomplete", "Type a city");
        for t in ["Type a city", "Lisbon", "Lima", "London"] {
            c.items.push(node("text", t));
        }
        c.props.insert("value".into(), "L".into());
        c.props.insert("aria-label".into(), "City".into());
        let html = render(&c);
        reject_interact(&html);
        assert_eq!(
            html,
            "<div><div data-slot=\"autocomplete\"><input type=\"text\" role=\"combobox\" autocomplete=\"off\" autocorrect=\"off\" autocapitalize=\"none\" spellcheck=\"false\" aria-autocomplete=\"list\" aria-expanded=\"true\" aria-label=\"City\" data-slot=\"autocomplete-input\" placeholder=\"Type a city\" value=\"L\" aria-controls=\"cui-autocomplete-list\" /></div><div data-side=\"bottom\" data-align=\"start\" data-state=\"open\" role=\"dialog\" data-slot=\"autocomplete-content\"><div data-slot=\"autocomplete-command\"><div data-slot=\"command-list\" role=\"listbox\" aria-label=\"City\" id=\"cui-autocomplete-list\"><div><div data-slot=\"command-item\" role=\"option\" aria-selected=\"true\" data-selected=\"true\"><span>Lisbon</span></div><div data-slot=\"command-item\" role=\"option\" aria-selected=\"false\" data-selected=\"false\"><span>Lima</span></div><div data-slot=\"command-item\" role=\"option\" aria-selected=\"false\" data-selected=\"false\"><span>London</span></div></div></div></div></div></div>"
        );
    }

    #[test]
    fn label_is_placeholder_not_an_option() {
        let mut c = auto("Search", &["Ada", "Grace"]);
        c.props.insert("value".into(), "a".into());
        let html = render(&c);
        assert!(html.contains("placeholder=\"Search\""));
        assert_eq!(html.matches("role=\"option\"").count(), 2);
        assert!(!html.contains("<span>Search</span>"), "Search leaked as option: {html}");
        reject_interact(&html);
    }

    #[test]
    fn query_filters_with_case_insensitive_includes() {
        let mut c = auto("Search", &["Ada", "Grace", "Linus"]);
        c.props.insert("value".into(), "GR".into());
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"command-item\"").count(), 1);
        assert!(html.contains("data-selected=\"true\"><span>Grace</span>"));
        c.props.insert("value".into(), "zzz".into());
        let closed = render(&c);
        assert!(closed.contains("aria-expanded=\"false\""));
        assert!(!closed.contains("autocomplete-content"));
    }

    #[test]
    fn disabled_input_stays_closed() {
        let mut c = auto("Search", &["Ada"]);
        c.props.insert("disabled".into(), "true".into());
        c.props.insert("value".into(), "A".into());
        let html = render(&c);
        assert!(html.contains(" disabled />"));
        assert!(!html.contains("autocomplete-content"));
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
        assert!(css.contains("[data-slot=\"autocomplete-command\"]"));
        assert!(css.contains("min-width: 8rem"));
        assert!(css.contains("var(--cronus-surface-floating"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }

    /// Wave 1t geometry: the wrapper stays the input's 40px (content is absolute,
    /// 4px below, full width, rounded-lg bordered floating surface); input is the
    /// Input chrome at text-sm/20px; rows px-2 py-1.5 text-sm/20px in a p-1 list.
    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"autocomplete-content\"] {\n  position: absolute; top: calc(100% + 4px); left: 0; z-index: 50;\n  width: 100%; min-width: 8rem; box-sizing: border-box; padding: 0; outline: none;"
        ));
        assert!(css.contains("padding: 0 0.75rem; font-family: inherit; font-size: 0.875rem; line-height: 1.25rem; outline: none;"));
        assert!(css.contains(
            "[data-slot=\"autocomplete-content\"] [data-slot=\"command-item\"][data-selected=\"true\"] {\n  background: var(--cronus-surface-overlay); color: var(--cronus-fg);\n}"
        ));
    }
}
