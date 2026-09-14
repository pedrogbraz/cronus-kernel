//! Dedicated FloatingLabelInput renderer. DOM matches React:
//! `<div data-slot="floating-label-input"><div>` (the `relative` positioning
//! box) holding a native `<input data-slot="input">` followed by its
//! `<label data-slot="floating-label-input-label">`. The float is pure CSS
//! (`:placeholder-shown` / `:focus`), exactly like React's `peer-*` variants.
//! Not interact `input("floating-label-input")` (`<label>` wrapping `*-control`).

use crate::cronus_ui_kit::{esc, item, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let id = id_of(comp);
    let value = value_of(comp);
    let helper = helper_of(comp);
    let disabled = flag(comp, "disabled");
    let invalid = flag(comp, "invalid");

    let mut root = String::from("data-slot=\"floating-label-input\"");
    if disabled {
        root.push_str(" data-disabled=\"\"");
    }
    if invalid {
        root.push_str(" data-invalid=\"\"");
    }

    let mut field = format!("data-slot=\"input\" id=\"{id}\" placeholder=\" \"");
    if !value.is_empty() {
        field.push_str(&format!(" value=\"{value}\""));
    }
    if disabled {
        field.push_str(" disabled");
    }
    if invalid {
        field.push_str(" aria-invalid=\"true\"");
    }
    if !helper.is_empty() {
        field.push_str(&format!(" aria-describedby=\"{id}-helper\""));
    }

    let mut html = format!(
        "<div {root}><div><input {field} /><label data-slot=\"floating-label-input-label\" for=\"{id}\">{label}</label></div>"
    );
    if !helper.is_empty() {
        let role = if invalid { " role=\"alert\"" } else { "" };
        html.push_str(&format!(
            "<p data-slot=\"floating-label-input-helper\" id=\"{id}-helper\"{role}>{helper}</p>"
        ));
    }
    html.push_str("</div>");
    html
}

fn id_of(comp: &ComponentNode) -> String {
    if let Some(v) = attr(comp, "id").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    "floating-label-input".into()
}

fn value_of(comp: &ComponentNode) -> String {
    attr(comp, "value")
        .or_else(|| item(comp, "value"))
        .filter(|s| !s.is_empty())
        .map(esc)
        .unwrap_or_default()
}

fn helper_of(comp: &ComponentNode) -> String {
    attr(comp, "helperText")
        .or_else(|| attr(comp, "helper-text"))
        .or_else(|| attr(comp, "helper"))
        .or_else(|| attr(comp, "description"))
        .or_else(|| item(comp, "helper"))
        .or_else(|| item(comp, "description"))
        .filter(|s| !s.is_empty())
        .map(esc)
        .unwrap_or_default()
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

    fn reject_interact(html: &str) {
        assert!(!html.contains("floating-label-input-control"));
        assert!(!html.contains("<label data-slot=\"floating-label-input\" style="));
        assert!(!html.contains("<label data-slot=\"floating-label-input\">"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("height:2.5rem;padding:0 0.75rem"));
    }

    #[test]
    fn root_is_div_with_relative_box_input_then_label() {
        let html = render(&stub("floating-label-input", "Email"));
        assert!(html.starts_with("<div data-slot=\"floating-label-input\"><div><input data-slot=\"input\""));
        assert!(html.contains("<label data-slot=\"floating-label-input-label\" for=\"floating-label-input\">Email</label>"));
        assert!(html.contains("placeholder=\" \""));
        // React names the field through <label for>, not aria-label.
        assert!(!html.contains("aria-label="));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"floating-label-input\"><div><input data-slot=\"input\" id=\"floating-label-input\" placeholder=\" \" /><label data-slot=\"floating-label-input-label\" for=\"floating-label-input\">Email</label></div></div>"
        );
    }

    #[test]
    fn helper_and_value() {
        let mut c = stub("floating-label-input", "Email");
        c.props.insert("value".into(), "a@b.co".into());
        c.props.insert("helperText".into(), "We'll never share this.".into());
        let html = render(&c);
        assert!(html.contains("value=\"a@b.co\""));
        assert!(html.contains("</label></div><p data-slot=\"floating-label-input-helper\""));
        assert!(html.contains("We'll never share this."));
        assert!(!html.contains("role=\"alert\""));
        reject_interact(&html);
    }

    /// Emitter writes `description:"…"` (attr-style, props or item config).
    #[test]
    fn description_attr_is_helper() {
        let mut c = stub("floating-label-input", "Email");
        c.items[0].config.insert("description".into(), "Work address".into());
        let html = render(&c);
        assert!(html.contains("aria-describedby=\"floating-label-input-helper\""));
        assert!(html.contains(">Work address</p>"));
        reject_interact(&html);
    }

    #[test]
    fn disabled_and_invalid() {
        let mut c = stub("floating-label-input", "Email");
        c.props.insert("disabled".into(), "true".into());
        c.props.insert("invalid".into(), "true".into());
        c.props.insert("helperText".into(), "Required".into());
        let html = render(&c);
        assert!(html.contains(" data-disabled=\"\""));
        assert!(html.contains(" data-invalid=\"\""));
        assert!(html.contains(" disabled"));
        assert!(html.contains("aria-invalid=\"true\""));
        assert!(html.contains("role=\"alert\""));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_label_wrapping_control() {
        let c = stub("floating-label-input", "Email");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("floating-label-input", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<label data-slot=\"floating-label-input\""));
        assert!(interact.contains("data-slot=\"floating-label-input-control\""));
        assert!(interact.contains("style="));
        assert!(!html.contains("floating-label-input-control"));
        assert!(html.contains("data-slot=\"floating-label-input-label\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("floating-label-input", "Email"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"floating-label-input\"]"));
        assert!(css.contains("[data-slot=\"floating-label-input-label\"]"));
        assert!(css.contains("height: 3.5rem"));
        assert!(css.contains("var(--cronus-fg-tertiary)"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }

    /// Wave 1t geometry: React label sits at `top-1/2` of the 56px box, lifted
    /// `-translate-y-[1.6rem] scale-[0.8]` (visual top 4.4px, 16px tall); both
    /// input and label use text-sm's 20px line-height, not the document's 1.5.
    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"floating-label-input\"] > div { position: relative; }"));
        assert!(css.contains(
            "[data-slot=\"floating-label-input\"] [data-slot=\"input\"] {\n  height: 3.5rem; padding-top: 1rem; line-height: 1.25rem;\n}"
        ));
        assert!(css.contains("top: 50%; z-index: 10;"));
        assert!(css.contains("transform: translateY(-1.6rem) scale(0.8);"));
        assert!(!css.contains("top: 1.15rem"));
    }
}
