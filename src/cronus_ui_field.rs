//! Dedicated Field renderer. DOM matches React:
//! `<div data-slot="field"><label data-slot="field-label">` plus optional
//! `<p data-slot="field-description">` from the `description:` attr (props or
//! item config, as the audit emitter writes it) or extra text items.
//! Not interact `field_form()` (`<form>` of `<label>…<input style=CTRL>`).

use crate::cronus_ui_kit::{attr_nonempty, esc, label_of, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let ts = texts(comp);
    let label = ts.first().cloned().unwrap_or_else(|| label_of(comp));
    let mut inner = format!("<label data-slot=\"field-label\">{label}</label>");
    let described = attr_nonempty(comp, "description");
    match described {
        Some(d) => inner.push_str(&format!(
            "<p data-slot=\"field-description\">{}</p>",
            esc(d)
        )),
        None => {
            for d in ts.iter().skip(1) {
                inner.push_str(&format!("<p data-slot=\"field-description\">{d}</p>"));
            }
        }
    }
    format!("<div data-slot=\"field\">{inner}</div>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;
    use std::collections::HashMap;

    fn reject_interact(html: &str) {
        assert!(!html.contains("<form"));
        assert!(!html.contains("<input"));
        assert!(!html.contains("style="));
        assert!(!html.contains("data-slot=\"field-control\""));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("v-submit="));
    }

    #[test]
    fn root_is_div_with_field_label_not_form_stack() {
        let html = render(&stub("field", "Email"));
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"field\""));
        assert!(html.contains("<label data-slot=\"field-label\">Email</label>"));
        assert!(!html.contains("data-slot=\"field-description\""));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"field\"><label data-slot=\"field-label\">Email</label></div>"
        );
    }

    #[test]
    fn description_from_extra_text() {
        let mut c = stub("field", "Email");
        c.items.push(ComponentItemNode {
            item_type: "text".into(),
            text: "We'll never share this.".into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        });
        let html = render(&c);
        assert!(html.contains("data-slot=\"field-label\">Email</label>"));
        assert!(html.contains("data-slot=\"field-description\">We'll never share this.</p>"));
        reject_interact(&html);
    }

    /// Audit fixture: the emitter writes `description:"…"`, which the parser
    /// attaches to the label item's config. React renders it as FieldDescription.
    #[test]
    fn description_attr_from_props_or_item_config() {
        let mut c = stub("field", "Email");
        c.items[0]
            .config
            .insert("description".into(), "We'll never share it.".into());
        assert_eq!(
            render(&c),
            "<div data-slot=\"field\"><label data-slot=\"field-label\">Email</label><p data-slot=\"field-description\">We'll never share it.</p></div>"
        );
        let mut c = stub("field", "Email");
        c.props.insert("description".into(), "A <b>".into());
        assert!(render(&c).contains("<p data-slot=\"field-description\">A &lt;b&gt;</p>"));
        reject_interact(&render(&c));
    }

    #[test]
    fn skips_interact_field_form() {
        let html = render(&stub("field", "Email"));
        let interact = crate::cronus_ui_interact::render("field", &stub("field", "Email")).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<form data-slot=\"field\""));
        assert!(interact.contains("<input"));
        assert!(interact.contains("style="));
        assert!(!interact.contains("data-slot=\"field-label\""));
        assert!(html.contains("data-slot=\"field-label\""));
        reject_interact(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"field\"]"));
        assert!(css.contains("[data-slot=\"field-label\"]"));
        assert!(css.contains("[data-slot=\"field-description\"]"));
        assert!(css.contains("flex-direction: column"));
        assert!(css.contains("gap: 0.375rem"));
        assert!(css.contains("font-size: 0.875rem"));
        assert!(css.contains("font-size: 0.75rem"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(!css.contains("zinc-"));
    }
}
