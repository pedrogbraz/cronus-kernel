//! Dedicated Form renderer. DOM matches React rhf wrappers:
//! `<form data-slot="form"><div data-slot="form-item"><label data-slot="form-label">`
//! plus `form-control` wrapping an input and optional `form-description`.
//! Fields come from `item` lines; without them the form has one field named by
//! the `label`, and the first `text` line is its placeholder (the audit
//! emitter writes `placeholder` as `text`).
//! Not interact `field_form()` (SURF `<form>` dump + Submit button styles).

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

struct Field {
    label: String,
    description: Option<String>,
    placeholder: Option<String>,
}

pub fn render(comp: &ComponentNode) -> String {
    let items = field_entries(comp)
        .iter()
        .map(field_html)
        .collect::<Vec<_>>()
        .join("");
    format!("<form data-slot=\"form\">{items}</form>")
}

fn field_html(field: &Field) -> String {
    let label = &field.label;
    let desc = field
        .description
        .as_deref()
        .map(|d| format!("<p data-slot=\"form-description\">{d}</p>"))
        .unwrap_or_default();
    let placeholder = field
        .placeholder
        .as_deref()
        .map(|p| format!(" placeholder=\"{p}\""))
        .unwrap_or_default();
    format!(
        "<div data-slot=\"form-item\"><label data-slot=\"form-label\">{label}</label><div data-slot=\"form-control\"><input type=\"text\" name=\"{label}\"{placeholder} /></div>{desc}</div>"
    )
}

fn field_entries(comp: &ComponentNode) -> Vec<Field> {
    let descriptions = descriptions_of(comp);
    let choice: Vec<String> = comp
        .items
        .iter()
        .filter(|i| {
            matches!(i.item_type.as_str(), "item" | "tab" | "columns") && !i.text.is_empty()
        })
        .map(|i| esc(&i.text))
        .collect();
    if !choice.is_empty() {
        return attach_descriptions(choice, descriptions);
    }
    let other: Vec<String> = comp
        .items
        .iter()
        .filter(|i| {
            !matches!(
                i.item_type.as_str(),
                "label" | "title" | "text" | "value" | "description"
            ) && !i.text.is_empty()
        })
        .map(|i| esc(&i.text))
        .collect();
    if !other.is_empty() {
        return attach_descriptions(other, descriptions);
    }
    let label = label_of(comp);
    let placeholder = comp
        .items
        .iter()
        .filter(|i| i.item_type == "text" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .find(|t| *t != label);
    let mut fields = attach_descriptions(vec![label], descriptions);
    fields[0].placeholder = placeholder;
    fields
}

fn descriptions_of(comp: &ComponentNode) -> Vec<String> {
    comp.items
        .iter()
        .filter(|i| i.item_type == "description" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect()
}

fn attach_descriptions(labels: Vec<String>, descriptions: Vec<String>) -> Vec<Field> {
    labels
        .into_iter()
        .enumerate()
        .map(|(i, label)| Field {
            label,
            description: descriptions.get(i).cloned(),
            placeholder: None,
        })
        .collect()
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

    fn reject_interact(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("v-submit="));
        assert!(!html.contains(">Submit</button>"));
        assert!(!html.contains("font-weight:500"));
        assert!(!html.contains("padding:1rem;display:flex;flex-direction:column;gap:0.75rem"));
        assert!(!html.contains("<input data-slot="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("field_form("));
    }

    #[test]
    fn root_is_form_item_label_and_input_not_surf() {
        let html = render(&stub("form", "Email"));
        assert!(html.starts_with("<form data-slot=\"form\">"));
        assert!(html.contains("<div data-slot=\"form-item\">"));
        assert!(html.contains("<label data-slot=\"form-label\">Email</label>"));
        assert!(html.contains("<div data-slot=\"form-control\">"));
        assert!(html.contains("<input type=\"text\" name=\"Email\" />"));
        assert!(!html.contains("data-slot=\"form-description\""));
        reject_interact(&html);
        assert_eq!(
            html,
            "<form data-slot=\"form\"><div data-slot=\"form-item\"><label data-slot=\"form-label\">Email</label><div data-slot=\"form-control\"><input type=\"text\" name=\"Email\" /></div></div></form>"
        );
    }

    /// Audit fixture: `label "Email"` + `text "ada@cronus.dev"` (the
    /// placeholder). React renders ONE form-item; the text is not a field.
    #[test]
    fn text_line_is_placeholder_not_second_field() {
        let mut c = stub("form", "Email");
        c.items.push(extra("text", "ada@cronus.dev"));
        let html = render(&c);
        assert_eq!(
            html,
            "<form data-slot=\"form\"><div data-slot=\"form-item\"><label data-slot=\"form-label\">Email</label><div data-slot=\"form-control\"><input type=\"text\" name=\"Email\" placeholder=\"ada@cronus.dev\" /></div></div></form>"
        );
        assert_eq!(html.matches("data-slot=\"form-item\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn placeholder_is_escaped() {
        let mut c = stub("form", "Email");
        c.items.push(extra("text", "a \"b\" <c>"));
        let html = render(&c);
        assert!(html.contains(" placeholder=\"a &quot;b&quot; &lt;c&gt;\" />"));
        reject_interact(&html);
    }

    #[test]
    fn field_label_is_not_a_field_when_items_exist() {
        let mut c = stub("form", "Contact");
        c.items.push(extra("item", "Email"));
        c.items.push(extra("item", "Name"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"form-label\">Email</label>"));
        assert!(html.contains("data-slot=\"form-label\">Name</label>"));
        assert!(!html.contains("data-slot=\"form-label\">Contact</label>"));
        assert_eq!(html.matches("data-slot=\"form-item\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn description_from_description_item() {
        let mut c = stub("form", "Email");
        c.items
            .push(extra("description", "We'll never share this."));
        let html = render(&c);
        assert!(html.contains("data-slot=\"form-label\">Email</label>"));
        assert!(html.contains("data-slot=\"form-description\">We'll never share this.</p>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_field_form() {
        let c = stub("form", "Email");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("form", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<form data-slot=\"form\""));
        assert!(interact.contains("style="));
        assert!(interact.contains(">Submit</button>"));
        assert!(interact.contains("data-slot=\"button\""));
        assert!(!interact.contains("data-slot=\"form-item\""));
        assert!(!interact.contains("data-slot=\"form-label\""));
        assert!(!interact.contains("data-slot=\"form-control\""));
        assert!(html.contains("data-slot=\"form-item\""));
        assert!(html.contains("data-slot=\"form-label\""));
        assert!(html.contains("data-slot=\"form-control\""));
        assert!(!html.contains(">Submit</button>"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("form", "Email"));
            reject_interact(&html);
            assert!(!html.contains("v-submit="));
            assert!(!html.contains("v-method="));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"form\"]"));
        assert!(css.contains("[data-slot=\"form-item\"]"));
        assert!(css.contains("[data-slot=\"form-label\"]"));
        assert!(css.contains("[data-slot=\"form-control\"]"));
        assert!(css.contains("[data-slot=\"form-description\"]"));
        assert!(css.contains("flex-direction: column"));
        assert!(css.contains("gap: 0.375rem"));
        assert!(css.contains("font-size: 0.75rem"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
