//! Dedicated Form renderer. DOM matches the React audit fixture:
//! `<form data-slot="form"><div data-slot="form-item"><label data-slot="label" for>`
//! + `<input data-slot="input" id>` (shared Label / Input chrome) and optional
//! `form-description`. Fields come from `item` lines; without them the form
//! has one field named by the `label`, and the first `text` line is its
//! placeholder (the audit emitter writes `placeholder` as `text`).
//! Not interact `field_form()` (SURF `<form>` dump + Submit button styles).

use crate::cronus_ui_kit::{esc, label_of, widget_id};
use crate::parser::ComponentNode;

struct Field {
    label: String,
    description: Option<String>,
    placeholder: Option<String>,
    textarea: bool,
    kind: Option<String>,
}

pub fn render(comp: &ComponentNode) -> String {
    let base = widget_id(comp, "input");
    let items = field_entries(comp)
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let id = if i == 0 {
                base.clone()
            } else {
                format!("{base}-{}", i + 1)
            };
            field_html(f, &id)
        })
        .collect::<Vec<_>>()
        .join("");
    // `action "Save"` renders the docs' submit Button at the end of the form.
    let submit = comp
        .items
        .iter()
        .find(|i| i.item_type == "action" && !i.text.is_empty())
        .map(|i| {
            format!(
                "<button type=\"submit\" data-slot=\"button\" data-variant=\"primary\" data-size=\"md\">{}</button>",
                esc(&i.text)
            )
        })
        .unwrap_or_default();
    format!("<form data-slot=\"form\">{items}{submit}</form>")
}

fn field_html(field: &Field, id: &str) -> String {
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
    let control = if field.textarea {
        format!("<textarea data-slot=\"textarea\" id=\"{id}\" name=\"{label}\" rows=\"3\"{placeholder}></textarea>")
    } else {
        let ty = field
            .kind
            .as_deref()
            .map(|t| format!(" type=\"{t}\""))
            .unwrap_or_default();
        format!("<input data-slot=\"input\"{ty} id=\"{id}\" name=\"{label}\"{placeholder} />")
    };
    format!(
        "<div data-slot=\"form-item\"><label data-slot=\"label\" for=\"{id}\">{label}</label>{control}{desc}</div>"
    )
}

fn field_entries(comp: &ComponentNode) -> Vec<Field> {
    let descriptions = descriptions_of(comp);
    let choice: Vec<&crate::parser::ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| {
            matches!(i.item_type.as_str(), "item" | "tab" | "columns" | "field")
                && !i.text.is_empty()
        })
        .collect();
    if !choice.is_empty() {
        return choice
            .iter()
            .enumerate()
            .map(|(i, it)| Field {
                label: esc(&it.text),
                description: it
                    .config
                    .get("description")
                    .map(|d| esc(d))
                    .or_else(|| descriptions.get(i).cloned()),
                placeholder: it.config.get("placeholder").map(|p| esc(p)),
                textarea: it
                    .config
                    .get("textarea")
                    .is_some_and(|v| crate::cronus_ui_kit::truthy(v)),
                kind: it.config.get("type").map(|t| esc(t)),
            })
            .collect();
    }
    let other: Vec<String> = comp
        .items
        .iter()
        .filter(|i| {
            !matches!(
                i.item_type.as_str(),
                "label" | "title" | "text" | "value" | "description" | "action"
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
            textarea: false,
            kind: None,
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
        assert!(!html.contains("data-slot=\"form-control\""));
        assert!(!html.contains("data-slot=\"form-label\""));
        // stub_renderer_gate "field" fingerprint = `<input data-slot=` + `-control"`.
        assert!(!html.contains("-control\""));
        assert!(crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(html).is_none());
        assert!(!html.contains("<script"));
        assert!(!html.contains("field_form("));
    }

    /// React fixture: `form > form-item > label[data-slot=label] + input[data-slot=input]`
    /// — no form-label / form-control wrapper (wave1s geometry parity).
    #[test]
    fn root_is_form_item_label_and_input_not_surf() {
        let html = render(&stub("form", "Email"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<form data-slot=\"form\"><div data-slot=\"form-item\"><label data-slot=\"label\" for=\"cui-form-input\">Email</label><input data-slot=\"input\" id=\"cui-form-input\" name=\"Email\" /></div></form>"
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
            "<form data-slot=\"form\"><div data-slot=\"form-item\"><label data-slot=\"label\" for=\"cui-form-input\">Email</label><input data-slot=\"input\" id=\"cui-form-input\" name=\"Email\" placeholder=\"ada@cronus.dev\" /></div></form>"
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
        assert!(html.contains("<label data-slot=\"label\" for=\"cui-form-input\">Email</label>"));
        assert!(html.contains("<label data-slot=\"label\" for=\"cui-form-input-2\">Name</label>"));
        assert!(
            html.contains("<input data-slot=\"input\" id=\"cui-form-input-2\" name=\"Name\" />")
        );
        assert!(!html.contains(">Contact</label>"));
        assert_eq!(html.matches("data-slot=\"form-item\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn description_from_description_item() {
        let mut c = stub("form", "Email");
        c.items
            .push(extra("description", "We'll never share this."));
        let html = render(&c);
        assert!(html.contains("for=\"cui-form-input\">Email</label>"));
        assert!(html.contains("data-slot=\"form-description\">We'll never share this.</p>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_field_form() {
        let c = stub("form", "Email");
        let html = render(&c);
        assert!(html.contains("data-slot=\"form-item\""));
        assert!(html.contains("<label data-slot=\"label\""));
        assert!(html.contains("<input data-slot=\"input\""));
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
        assert!(css.contains("[data-slot=\"form\"] {\n  display: block; line-height: 1.5;\n}"));
        assert!(css.contains(
            "[data-slot=\"form-item\"] {\n  display: flex; flex-direction: column; gap: 0.375rem;\n}"
        ));
        assert!(css.contains(
            "[data-slot=\"form-item\"] > [data-slot=\"input\"], [data-slot=\"form-item\"] > [data-slot=\"textarea\"] {\n  line-height: 1.25rem;\n}"
        ));
        assert!(css.contains(
            "[data-slot=\"form-item\"] > [data-slot=\"input\"]::placeholder {\n  color: var(--cronus-fg-tertiary);\n}"
        ));
        assert!(!css.contains("[data-slot=\"form-label\"] {"));
        assert!(!css.contains("[data-slot=\"form-control\"] {"));
        assert!(css.contains("[data-slot=\"form-description\"]"));
        assert!(css.contains("font-size: 0.75rem"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
