//! Dedicated TagsInput renderer. DOM matches React:
//! `<div data-slot="tags-input">` with one secondary `<span data-slot="badge">`
//! per committed tag (`<span>tag</span>` + `<button data-slot="tags-input-remove">`
//! holding a lucide `x`), then `<input data-slot="tags-input-field">`.
//! The field placeholder is only shown while there are no tags (React).
//! Adding / removing tags needs JS: the remove buttons are React's native
//! `<button tabindex="-1">` rendered `disabled`, with React's idle look (not dimmed).
//! Not interact `select("tags-input")` (`<label><select data-slot="tags-input-control">`).

use crate::cronus_ui_kit::{choice_texts, esc, item};
use crate::parser::ComponentNode;

/// lucide `x` (React `<X className="size-3" />`).
const X_ICON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M18 6 6 18\"></path><path d=\"m6 6 12 12\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let placeholder = placeholder_of(comp);
    let tags = tags_of(comp, &placeholder);
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

    let mut field = String::from("type=\"text\" autocomplete=\"off\"");
    if !aria.is_empty() {
        field.push_str(&format!(" aria-label=\"{aria}\""));
    }
    field.push_str(" data-slot=\"tags-input-field\"");
    if tags.is_empty() && !placeholder.is_empty() {
        field.push_str(&format!(" placeholder=\"{placeholder}\""));
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
        return format!(
            "<span data-slot=\"badge\" data-variant=\"secondary\"><span>{tag}</span></span>"
        );
    }
    format!(
        "<span data-slot=\"badge\" data-variant=\"secondary\"><span>{tag}</span><button type=\"button\" tabindex=\"-1\" aria-label=\"Remove {tag}\" data-slot=\"tags-input-remove\" disabled>{X_ICON}</button></span>"
    )
}

/// Committed tags: `item` choices, else the free `text` lines. The emitter also
/// writes the placeholder as the first `text`; that names the field, it is not a tag.
fn tags_of(comp: &ComponentNode, placeholder: &str) -> Vec<String> {
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

    fn node(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn tags(placeholder: &str, items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("tags-input", placeholder);
        for t in items {
            c.items.push(node("item", t));
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
    fn root_is_badges_and_field_not_native_select() {
        let html = render(&tags("Add a tag", &["react", "vue"]));
        assert!(html.starts_with("<div data-slot=\"tags-input\">"));
        assert!(!html.contains("placeholder="));
        assert!(!html.contains("tags-input-item"));
        reject_interact(&html);
        assert_eq!(
            html,
            format!("<div data-slot=\"tags-input\"><span data-slot=\"badge\" data-variant=\"secondary\"><span>react</span><button type=\"button\" tabindex=\"-1\" aria-label=\"Remove react\" data-slot=\"tags-input-remove\" disabled>{X_ICON}</button></span><span data-slot=\"badge\" data-variant=\"secondary\"><span>vue</span><button type=\"button\" tabindex=\"-1\" aria-label=\"Remove vue\" data-slot=\"tags-input-remove\" disabled>{X_ICON}</button></span><input type=\"text\" autocomplete=\"off\" aria-label=\"Add a tag\" data-slot=\"tags-input-field\" /></div>")
        );
    }

    #[test]
    fn label_is_placeholder_not_a_chip() {
        let html = render(&tags("Add a tag", &["react", "vue"]));
        assert_eq!(html.matches("data-slot=\"badge\"").count(), 2);
        assert!(
            !html.contains("<span>Add a tag</span>"),
            "placeholder leaked as chip: {html}"
        );
        reject_interact(&html);
    }

    /// Emitted fixture shape: `label "Add a tag"`, `text "Add a tag"`,
    /// `text "Design"`, `text "System"` — only Design and System are tags.
    #[test]
    fn emitted_placeholder_text_is_not_a_chip() {
        let mut c = stub("tags-input", "Add a tag");
        for t in ["Add a tag", "Design", "System"] {
            c.items.push(node("text", t));
        }
        c.props.insert("aria-label".into(), "Tags".into());
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"badge\"").count(), 2);
        assert!(html.contains("<span>Design</span>"));
        assert!(html.contains("<span>System</span>"));
        assert!(!html.contains("<span>Add a tag</span>"));
        assert!(html.contains("aria-label=\"Tags\" data-slot=\"tags-input-field\" />"));
    }

    #[test]
    fn empty_tags_keep_field_and_placeholder() {
        let html = render(&stub("tags-input", "Add a tag"));
        assert!(html.contains("data-slot=\"tags-input-field\""));
        assert!(html.contains("placeholder=\"Add a tag\""));
        assert!(!html.contains("data-slot=\"badge\""));
        reject_interact(&html);
    }

    #[test]
    fn extra_text_items_become_chips() {
        let mut c = stub("tags-input", "Tags");
        c.items.push(node("text", "alpha"));
        c.items.push(node("text", "beta"));
        let html = render(&c);
        assert!(html.contains("<span>alpha</span>"));
        assert!(html.contains("<span>beta</span>"));
        assert_eq!(html.matches("data-slot=\"badge\"").count(), 2);
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
        assert!(html.contains(
            "<span data-slot=\"badge\" data-variant=\"secondary\"><span>react</span></span>"
        ));
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
        assert!(css.contains("[data-slot=\"tags-input-remove\"]"));
        assert!(css.contains("[data-slot=\"tags-input-field\"]"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("min-height: 2.5rem"));
        assert!(css.contains("min-width: 6rem"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }

    /// Wave 1t geometry: text-sm root/field (20px lines), 22px secondary badges
    /// (text-xs/1rem, pr-1, gap-1), 14px rounded-sm remove with a 12px x glyph.
    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(
            css.contains("padding: 0.375rem 0.75rem; font-size: 0.875rem; line-height: 1.25rem;")
        );
        assert!(css.contains(
            "[data-slot=\"tags-input\"] > [data-slot=\"badge\"] {\n  gap: 0.25rem; padding-right: 0.25rem; line-height: 1rem;\n}"
        ));
        assert!(css.contains(
            "[data-slot=\"tags-input-remove\"] svg { width: 0.75rem; height: 0.75rem; }"
        ));
        assert!(css
            .contains("border-radius: var(--cronus-radius-sm); color: var(--cronus-fg-tertiary);"));
    }
}
