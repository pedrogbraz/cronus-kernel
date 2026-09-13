//! Dedicated CopyButton renderer. DOM matches React idle:
//! `<button type="button" data-slot="copy-button" aria-label="Copy">`.
//! Not interact clipboard `onclick` or inline BASE/SURF styles.

use crate::cronus_ui_kit::{esc, item, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let mut attrs = String::from("type=\"button\" data-slot=\"copy-button\" aria-label=\"Copy\"");
    if let Some(v) = copy_value(comp) {
        attrs.push_str(&format!(" data-copy=\"{}\"", esc(v)));
    }
    format!("<button {attrs}>{label}</button>")
}

fn copy_value(comp: &ComponentNode) -> Option<&str> {
    if let Some(v) = comp.props.get("value") {
        if !v.is_empty() {
            return Some(v.as_str());
        }
    }
    if let Some(t) = item(comp, "value") {
        if !t.is_empty() {
            return Some(t);
        }
    }
    for kind in ["label", "title", "text"] {
        if let Some(t) = item(comp, kind) {
            if !t.is_empty() {
                return Some(t);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn reject_interact(html: &str) {
        assert!(!html.contains("onclick="));
        assert!(!html.contains("navigator.clipboard"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<input"));
        assert!(!html.contains("<label"));
    }

    #[test]
    fn root_is_idle_copy_button() {
        let html = render(&stub("copy-button", "Copy"));
        assert!(html.starts_with("<button "));
        assert!(html.contains("type=\"button\""));
        assert!(html.contains("data-slot=\"copy-button\""));
        assert!(html.contains("aria-label=\"Copy\""));
        assert!(html.contains(">Copy</button>"));
        assert!(html.contains("data-copy=\"Copy\""));
        reject_interact(&html);
    }

    #[test]
    fn text_comes_from_label() {
        let html = render(&stub("copy-button", "Copy link"));
        assert!(html.contains(">Copy link</button>"));
        assert!(html.contains("aria-label=\"Copy\""));
        assert!(html.contains("data-copy=\"Copy link\""));
        reject_interact(&html);
    }

    #[test]
    fn data_copy_from_props_value() {
        let mut c = stub("copy-button", "Copy");
        c.props.insert("value".into(), "secret-token".into());
        let html = render(&c);
        assert!(html.contains("data-copy=\"secret-token\""));
        assert!(html.contains(">Copy</button>"));
        assert!(html.contains("aria-label=\"Copy\""));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_clipboard_onclick() {
        let c = stub("copy-button", "Copy");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("copy-button", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("onclick=\"navigator.clipboard.writeText"));
        assert!(interact.contains("style="));
        reject_interact(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"copy-button\"]"));
        assert!(css.contains("display: inline-flex"));
        assert!(css.contains("height: 2.25rem"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
