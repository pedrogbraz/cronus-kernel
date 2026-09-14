//! Isolated Cronus Audit document. Injected by the binary — never authored
//! in `.cronus`. Zero JS. No Voodoo, HMR, or animation runtime.

use crate::cronus_ui;

pub fn render_audit_document(widget_html: &str, preset: &str, mode: &str, dir: &str) -> String {
    let css = cronus_ui::audit_stylesheet();
    format!(
        r#"<!DOCTYPE html>
<html lang="en" data-cronus-theme="{preset}" data-cronus-mode="{mode}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Cronus Audit</title>
<style>
{css}
[data-audit-canvas] {{
  width: 480px;
  min-height: 240px;
  padding: 24px;
  box-sizing: border-box;
  line-height: 1.5;
  background: var(--cronus-surface-base);
  color: var(--cronus-fg);
}}
</style>
</head>
<body>
<div data-audit-canvas dir="{dir}">{widget_html}</div>
</body>
</html>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_is_480_and_english_with_zero_js() {
        let html = render_audit_document(
            r#"<button data-slot="button">Save</button>"#,
            "aurora",
            "light",
            "ltr",
        );
        assert!(html.contains("lang=\"en\""));
        assert!(html.contains("data-cronus-theme=\"aurora\""));
        assert!(html.contains("data-cronus-mode=\"light\""));
        assert!(html.contains("data-audit-canvas"));
        assert!(html.contains("width: 480px"));
        assert!(html.contains("min-height: 240px"));
        assert!(html.contains("padding: 24px"));
        // React's audit canvas inherits Tailwind preflight `line-height: 1.5`
        // (24px at 16px); without it every text box was `normal` and shorter.
        assert!(html.contains("line-height: 1.5"));
        assert!(html.contains("data-slot=\"button\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("voodoojs"));
        assert!(!html.contains("preview-frame"));
        assert!(!html.contains("innerHTML"));
        assert!(!html.contains("srcdoc"));
    }
}
