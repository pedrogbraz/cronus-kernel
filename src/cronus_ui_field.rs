//! Dedicated Field renderer. DOM matches React:
//! `<div data-slot="field"><label data-slot="field-label">` plus optional
//! `<p data-slot="field-description">` from extra text items.
//! Not interact `field_form()` (`<form>` of `<label>…<input style=CTRL>`).

use crate::cronus_ui_kit::{label_of, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let ts = texts(comp);
    let label = ts.first().cloned().unwrap_or_else(|| label_of(comp));
    let mut inner = format!("<label data-slot=\"field-label\">{label}</label>");
    for d in ts.iter().skip(1) {
        inner.push_str(&format!("<p data-slot=\"field-description\">{d}</p>"));
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
