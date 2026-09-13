//! Dedicated FileDropzone renderer. DOM matches React:
//! `<label data-slot="file-dropzone"><input type="file" class="sr-only" /><span>`.
//! Not interact `input("file-dropzone")` (`data-slot="file-dropzone-control"` + CTRL styles).

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let aria = match aria_label_of(comp) {
        Some(v) => esc(v),
        None => label.clone(),
    };
    let mut input = String::from("<input type=\"file\" class=\"sr-only\"");
    if let Some(accept) = attr(comp, "accept") {
        input.push_str(&format!(" accept=\"{}\"", esc(accept)));
    }
    if flag(comp, "multiple") {
        input.push_str(" multiple");
    }
    if flag(comp, "disabled") {
        input.push_str(" disabled");
    }
    input.push_str(&format!(" aria-label=\"{aria}\""));
    if let Some(described) = attr(comp, "aria-describedby") {
        input.push_str(&format!(" aria-describedby=\"{}\"", esc(described)));
    }
    input.push_str(" />");
    let mut label_attrs = String::from("data-slot=\"file-dropzone\"");
    if flag(comp, "disabled") {
        label_attrs.push_str(" aria-disabled=\"true\"");
    }
    format!("<label {label_attrs}>{input}<span>{label}</span></label>")
}

fn aria_label_of(comp: &ComponentNode) -> Option<&str> {
    attr(comp, "aria-label").filter(|s| !s.is_empty())
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
        assert!(!html.contains("data-slot=\"file-dropzone-control\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("height:2.5rem;padding:0 0.75rem"));
    }

    #[test]
    fn root_is_label_with_sr_only_file_input() {
        let html = render(&stub("file-dropzone", "Upload files"));
        assert!(html.starts_with("<label "));
        assert!(html.contains("data-slot=\"file-dropzone\""));
        assert!(html.contains("<input type=\"file\" class=\"sr-only\""));
        assert!(html.contains("aria-label=\"Upload files\""));
        assert!(html.contains("<span>Upload files</span>"));
        assert!(html.ends_with("</label>"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<label data-slot=\"file-dropzone\"><input type=\"file\" class=\"sr-only\" aria-label=\"Upload files\" /><span>Upload files</span></label>"
        );
    }

    #[test]
    fn disabled_multiple_accept() {
        let mut c = stub("file-dropzone", "Upload files");
        c.props.insert("disabled".into(), "true".into());
        c.props.insert("multiple".into(), "true".into());
        c.props.insert("accept".into(), "image/*".into());
        let html = render(&c);
        assert!(html.contains(" aria-disabled=\"true\""));
        assert!(html.contains(" multiple"));
        assert!(html.contains(" disabled"));
        assert!(html.contains(" accept=\"image/*\""));
        reject_interact(&html);
    }

    #[test]
    fn aria_label_from_props() {
        let mut c = stub("file-dropzone", "Drop");
        c.props.insert("aria-label".into(), "Choose a file".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Choose a file\""));
        assert!(html.contains("<span>Drop</span>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_control_slot() {
        let html = render(&stub("file-dropzone", "Upload files"));
        let interact = crate::cronus_ui_interact::render(
            "file-dropzone",
            &stub("file-dropzone", "Upload files"),
        )
        .unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("data-slot=\"file-dropzone-control\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("type=\"file\""));
        assert!(!html.contains("data-slot=\"file-dropzone-control\""));
        reject_interact(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"file-dropzone\"]"));
        assert!(css.contains("flex-direction: column"));
        assert!(css.contains("align-items: center"));
        assert!(css.contains("justify-content: center"));
        assert!(css.contains("gap: 0.5rem"));
        assert!(css.contains("border: 2px dashed var(--cronus-border)"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("padding: 2.5rem 1.5rem"));
        assert!(css.contains("text-align: center"));
        assert!(css.contains("font-size: 0.875rem"));
        assert!(css.contains("clip: rect(0, 0, 0, 0)"));
        assert!(!css.contains("zinc-"));
    }
}
