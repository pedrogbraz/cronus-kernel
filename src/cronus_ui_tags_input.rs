//! Dedicated TagsInput renderer. DOM matches React:
//! `<div data-slot="tags-input">` chips from extra texts plus
//! `<input data-slot="tags-input-field">`.
//! Not interact `select("tags-input")` (`<label><select data-slot="tags-input-control">`).

use crate::cronus_ui_kit::{choice_texts, esc, item, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let tags = extra_texts(comp);
    let placeholder = placeholder_of(comp);
    let disabled = flag(comp, "disabled");
    let invalid = flag(comp, "invalid");
    let aria = aria_label_of(comp);

    let mut root = String::from("data-slot=\"tags-input\"");
    if disabled {
        root.push_str(" data-disabled=\"\"");
    }
    if invalid {
        root.push_str(" aria-invalid=\"true\"");
    }

    let chips = tags
        .iter()
        .map(|t| chip(t, disabled))
        .collect::<Vec<_>>()
        .join("");

    let mut field = String::from(
        "data-slot=\"tags-input-field\" type=\"text\" autocomplete=\"off\"",
    );
    if tags.is_empty() && !placeholder.is_empty() {
        field.push_str(&format!(" placeholder=\"{placeholder}\""));
    }
    if !aria.is_empty() {
        field.push_str(&format!(" aria-label=\"{aria}\""));
    }
    if disabled {
        field.push_str(" disabled");
    }
    if invalid {
        field.push_str(" aria-invalid=\"true\"");
    }

    format!("<div {root}>{chips}<input {field} /></div>")
}

fn chip(tag: &str, disabled: bool) -> String {
    if disabled {
        return format!("<span data-slot=\"tags-input-item\">{tag}</span>");
    }
    format!(
        "<span data-slot=\"tags-input-item\">{tag}<button type=\"button\" data-slot=\"tags-input-remove\" tabindex=\"-1\" aria-label=\"Remove {tag}\">×</button></span>"
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
    String::new()
}

fn aria_label_of(comp: &ComponentNode) -> String {
    if let Some(v) = attr(comp, "aria-label").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    placeholder_of(comp)
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

    fn extra(text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "item".into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn tags(placeholder: &str, items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("tags-input", placeholder);
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
    fn root_is_chips_and_field_not_native_select() {
        let html = render(&tags("Add a tag", &["react", "vue"]));
        assert!(html.starts_with("<div data-slot=\"tags-input\">"));
        assert!(html.contains("<span data-slot=\"tags-input-item\">react"));
        assert!(html.contains("<span data-slot=\"tags-input-item\">vue"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"tags-input-remove\" tabindex=\"-1\" aria-label=\"Remove react\">×</button>"
        ));
        assert!(html.contains("data-slot=\"tags-input-field\""));
        assert!(html.contains("type=\"text\""));
        assert!(!html.contains("placeholder="));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"tags-input\"><span data-slot=\"tags-input-item\">react<button type=\"button\" data-slot=\"tags-input-remove\" tabindex=\"-1\" aria-label=\"Remove react\">×</button></span><span data-slot=\"tags-input-item\">vue<button type=\"button\" data-slot=\"tags-input-remove\" tabindex=\"-1\" aria-label=\"Remove vue\">×</button></span><input data-slot=\"tags-input-field\" type=\"text\" autocomplete=\"off\" aria-label=\"Add a tag\" /></div>"
        );
    }

    #[test]
    fn label_is_placeholder_not_a_chip() {
        let html = render(&tags("Add a tag", &["react", "vue"]));
        assert_eq!(html.matches("data-slot=\"tags-input-item\"").count(), 2);
        for chunk in html.split("data-slot=\"tags-input-item\"").skip(1) {
            assert!(
                !chunk.contains(">Add a tag"),
                "placeholder leaked as chip: {html}"
            );
        }
        reject_interact(&html);
    }

    #[test]
    fn empty_tags_keep_field_and_placeholder() {
        let html = render(&stub("tags-input", "Add a tag"));
        assert!(html.contains("data-slot=\"tags-input-field\""));
        assert!(html.contains("placeholder=\"Add a tag\""));
        assert!(!html.contains("data-slot=\"tags-input-item\""));
        reject_interact(&html);
    }

    #[test]
    fn extra_text_items_become_chips() {
        let mut c = stub("tags-input", "Tags");
        c.items.push(ComponentItemNode {
            item_type: "text".into(),
            text: "alpha".into(),
            link: None,
            tone: None,
            config: Default::default(),
        });
        c.items.push(ComponentItemNode {
            item_type: "text".into(),
            text: "beta".into(),
            link: None,
            tone: None,
            config: Default::default(),
        });
        let html = render(&c);
        assert!(html.contains("<span data-slot=\"tags-input-item\">alpha"));
        assert!(html.contains("<span data-slot=\"tags-input-item\">beta"));
        assert_eq!(html.matches("data-slot=\"tags-input-item\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn disabled_and_invalid() {
        let mut c = tags("Add a tag", &["react"]);
        c.props.insert("disabled".into(), "true".into());
        c.props.insert("invalid".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(" data-disabled=\"\""));
        assert!(html.contains(" aria-invalid=\"true\""));
        assert!(html.contains(" disabled"));
        assert!(!html.contains("data-slot=\"tags-input-remove\""));
        assert!(html.contains("<span data-slot=\"tags-input-item\">react</span>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_native_select() {
        let c = tags("Add a tag", &["react", "vue"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("tags-input", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<label data-slot=\"tags-input\""));
        assert!(interact.contains("<select data-slot=\"tags-input-control\""));
        assert!(interact.contains("style="));
        assert!(!html.contains("tags-input-control"));
        assert!(!html.contains("<select"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&tags("Add a tag", &["react"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"tags-input\"]"));
        assert!(css.contains("[data-slot=\"tags-input-item\"]"));
        assert!(css.contains("[data-slot=\"tags-input-remove\"]"));
        assert!(css.contains("[data-slot=\"tags-input-field\"]"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("min-height: 2.5rem"));
        assert!(css.contains("min-width: 6rem"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
