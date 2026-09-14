//! Dedicated FileDropzone renderer. DOM matches React:
//! `<label data-slot="file-dropzone" data-dragging="false">` wrapping the
//! visually hidden `<input type="file" class="sr-only">`, the lucide
//! `cloud-upload` glyph and `<span>Drag &amp; drop or <span>browse</span></span>`.
//! The whole surface is the file input's label, so clicking it opens the native
//! picker with zero JS. Drag-and-drop highlighting (`data-dragging="true"`) needs
//! JS and is not emitted. `label` names the input (aria-label), never the copy.
//! Not interact `input("file-dropzone")` (`data-slot="file-dropzone-control"` + CTRL styles).

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

/// lucide `cloud-upload` (React `<UploadCloud />`).
const CLOUD_UPLOAD: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M12 13v8\"></path><path d=\"M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242\"></path><path d=\"m8 17 4-4 4 4\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let aria = match aria_label_of(comp) {
        Some(v) => esc(v),
        None => label_of(comp),
    };
    let drop = attr(comp, "drop")
        .filter(|s| !s.is_empty())
        .map(esc)
        .unwrap_or_else(|| "Drag &amp; drop or".into());
    let browse = attr(comp, "browse")
        .filter(|s| !s.is_empty())
        .map(esc)
        .unwrap_or_else(|| "browse".into());
    let mut input = String::from("<input type=\"file\"");
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
    input.push_str(" class=\"sr-only\" />");
    let mut label_attrs = String::from("data-slot=\"file-dropzone\"");
    if flag(comp, "disabled") {
        label_attrs.push_str(" aria-disabled=\"true\"");
    }
    label_attrs.push_str(" data-dragging=\"false\"");
    format!("<label {label_attrs}>{input}{CLOUD_UPLOAD}<span>{drop} <span>{browse}</span></span></label>")
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
        assert!(!html.contains("<script"));
        assert!(!html.contains("height:2.5rem;padding:0 0.75rem"));
    }

    #[test]
    fn root_is_label_with_sr_only_input_glyph_and_copy() {
        let html = render(&stub("file-dropzone", "Upload files"));
        assert!(html.starts_with("<label "));
        assert!(html.contains("data-slot=\"file-dropzone\""));
        assert!(
            html.contains("<input type=\"file\" aria-label=\"Upload files\" class=\"sr-only\" />")
        );
        assert!(html.ends_with("</label>"));
        reject_interact(&html);
        assert_eq!(
            html,
            format!("<label data-slot=\"file-dropzone\" data-dragging=\"false\"><input type=\"file\" aria-label=\"Upload files\" class=\"sr-only\" />{CLOUD_UPLOAD}<span>Drag &amp; drop or <span>browse</span></span></label>")
        );
    }

    /// The fixture label ("Upload files") is the input's accessible name; the
    /// visible copy is React's default `Drag & drop or browse`.
    #[test]
    fn label_names_input_not_visible_copy() {
        let html = render(&stub("file-dropzone", "Upload files"));
        assert!(!html.contains("<span>Upload files</span>"));
        assert!(html.contains("<span>Drag &amp; drop or <span>browse</span></span>"));
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
    fn aria_label_and_copy_from_props() {
        let mut c = stub("file-dropzone", "Drop");
        c.props.insert("aria-label".into(), "Choose a file".into());
        c.props.insert("drop".into(), "Solte ou".into());
        c.props.insert("browse".into(), "procure".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Choose a file\""));
        assert!(html.contains("<span>Solte ou <span>procure</span></span>"));
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

    /// Wave 1t geometry: 136px tall = py-10 + 2px borders + 24px glyph + gap-2 +
    /// one 20px text-sm line (the document's 1.5 line-height made it 21px).
    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg-secondary);"
        ));
        assert!(css.contains(
            "[data-slot=\"file-dropzone\"] > svg { width: 1.5rem; height: 1.5rem; color: var(--cronus-fg-muted); }"
        ));
        assert!(css.contains(
            "[data-slot=\"file-dropzone\"] > span > span { font-weight: 500; color: var(--cronus-fg); }"
        ));
    }
}
