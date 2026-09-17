//! Dedicated Response renderer (AI suite). DOM matches React `Response`: a
//! single `<div data-slot="response">` (size-full text-sm text-fg) holding the
//! assistant text — one `text` line is written inline, several become `<p>`s
//! (the first/last child margins collapse like React's
//! `[&>*:first-child]:mt-0 [&>*:last-child]:mb-0`). The `label` item counts
//! as the first line. Streaming needs JS; the kernel renders the full text.

use crate::cronus_ui_kit::{content_texts, esc, item};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let mut texts: Vec<String> = item(comp, "label")
        .filter(|t| !t.is_empty())
        .map(esc)
        .into_iter()
        .collect();
    texts.extend(content_texts(comp));
    let body = match texts.as_slice() {
        [] => esc(&comp.name),
        [one] => one.clone(),
        many => many.iter().map(|t| format!("<p>{t}</p>")).collect(),
    };
    format!("<div data-slot=\"response\">{body}</div>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn text(t: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "text".into(),
            text: t.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    #[test]
    fn docs_example_is_inline_text() {
        let mut c = stub("response", "");
        c.items[0] = text("Cronus tokens re-theme live. Compose the app; do not fork the kit.");
        assert_eq!(
            render(&c),
            "<div data-slot=\"response\">Cronus tokens re-theme live. Compose the app; do not fork the kit.</div>"
        );
    }

    #[test]
    fn several_lines_become_paragraphs_and_escape() {
        let mut c = stub("response", "<b>\"one\"</b>");
        c.items.push(text("two"));
        assert_eq!(
            render(&c),
            "<div data-slot=\"response\"><p>&lt;b&gt;&quot;one&quot;&lt;/b&gt;</p><p>two</p></div>"
        );
        let html = render(&c);
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/response.css");
        assert!(css.contains("[data-slot=\"response\"]"));
        assert!(css.contains("var(--cronus-fg)"));
        assert!(!css.contains("zinc-"));
    }

    #[test]
    fn registered_as_dedicated() {
        use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
        assert_eq!(
            dedicated_fn_name("response"),
            Some("cronus_ui_response::render")
        );
        assert_eq!(
            renderer_kind("response"),
            RendererKind::Dedicated("cronus_ui_response::render")
        );
        let c = stub("response", "Hello");
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), render(&c));
    }
}
