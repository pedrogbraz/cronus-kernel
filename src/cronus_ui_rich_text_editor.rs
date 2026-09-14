//! Dedicated RichTextEditor renderer. DOM matches React's empty editor:
//! `rich-text-editor` > `rich-text-editor-toolbar` (14 icon `toggle` buttons,
//! 3 vertical `separator`s) + `rich-text-editor-content-wrapper` with the
//! `rich-text-editor-placeholder` paragraph over an empty textbox.
//!
//! Zero JS (no Tiptap): formatting toggles need the editor runtime, so every
//! toggle is rendered `disabled` (inert, announced as unavailable) with React's
//! idle look; only Undo/Redo carry `data-disabled` and dim, as they do in React.
//! The textbox is `aria-readonly`. Not interact `textarea("rich-text-editor")`.

use crate::cronus_ui_kit::{attr_nonempty, esc, label_of, widget_id};
use crate::parser::ComponentNode;

const SVG_OPEN: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\">";

/// `(aria-label, lucide icon body, idle-disabled)`; `None` is a separator.
const TOOLS: &[Option<(&str, &str, bool)>] = &[
    Some(("Bold", "<path d=\"M6 12h9a4 4 0 0 1 0 8H7a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1h7a4 4 0 0 1 0 8\"></path>", false)),
    Some(("Italic", "<line x1=\"19\" x2=\"10\" y1=\"4\" y2=\"4\"></line><line x1=\"14\" x2=\"5\" y1=\"20\" y2=\"20\"></line><line x1=\"15\" x2=\"9\" y1=\"4\" y2=\"20\"></line>", false)),
    Some(("Strikethrough", "<path d=\"M16 4H9a3 3 0 0 0-2.83 4\"></path><path d=\"M14 12a4 4 0 0 1 0 8H6\"></path><line x1=\"4\" x2=\"20\" y1=\"12\" y2=\"12\"></line>", false)),
    Some(("Inline code", "<path d=\"m16 18 6-6-6-6\"></path><path d=\"m8 6-6 6 6 6\"></path>", false)),
    None,
    Some(("Heading 1", "<path d=\"M4 12h8\"></path><path d=\"M4 18V6\"></path><path d=\"M12 18V6\"></path><path d=\"m17 12 3-2v8\"></path>", false)),
    Some(("Heading 2", "<path d=\"M4 12h8\"></path><path d=\"M4 18V6\"></path><path d=\"M12 18V6\"></path><path d=\"M21 18h-4c0-4 4-3 4-6 0-1.5-2-2.5-4-1\"></path>", false)),
    Some(("Heading 3", "<path d=\"M4 12h8\"></path><path d=\"M4 18V6\"></path><path d=\"M12 18V6\"></path><path d=\"M17.5 10.5c1.7-1 3.5 0 3.5 1.5a2 2 0 0 1-2 2\"></path><path d=\"M17 17.5c2 1.5 4 .3 4-1.5a2 2 0 0 0-2-2\"></path>", false)),
    None,
    Some(("Bullet list", "<path d=\"M3 5h.01\"></path><path d=\"M3 12h.01\"></path><path d=\"M3 19h.01\"></path><path d=\"M8 5h13\"></path><path d=\"M8 12h13\"></path><path d=\"M8 19h13\"></path>", false)),
    Some(("Ordered list", "<path d=\"M11 5h10\"></path><path d=\"M11 12h10\"></path><path d=\"M11 19h10\"></path><path d=\"M4 4h1v5\"></path><path d=\"M4 9h2\"></path><path d=\"M6.5 20H3.4c0-1 2.6-1.925 2.6-3.5a1.5 1.5 0 0 0-2.6-1.02\"></path>", false)),
    Some(("Blockquote", "<path d=\"M16 3a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2 1 1 0 0 1 1 1v1a2 2 0 0 1-2 2 1 1 0 0 0-1 1v2a1 1 0 0 0 1 1 6 6 0 0 0 6-6V5a2 2 0 0 0-2-2z\"></path><path d=\"M5 3a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2 1 1 0 0 1 1 1v1a2 2 0 0 1-2 2 1 1 0 0 0-1 1v2a1 1 0 0 0 1 1 6 6 0 0 0 6-6V5a2 2 0 0 0-2-2z\"></path>", false)),
    Some(("Code block", "<path d=\"m10 9-3 3 3 3\"></path><path d=\"m14 15 3-3-3-3\"></path><rect x=\"3\" y=\"3\" width=\"18\" height=\"18\" rx=\"2\"></rect>", false)),
    Some(("Horizontal rule", "<path d=\"M5 12h14\"></path>", false)),
    None,
    Some(("Undo", "<path d=\"M9 14 4 9l5-5\"></path><path d=\"M4 9h10.5a5.5 5.5 0 0 1 5.5 5.5a5.5 5.5 0 0 1-5.5 5.5H11\"></path>", true)),
    Some(("Redo", "<path d=\"m15 14 5-5-5-5\"></path><path d=\"M20 9H9.5A5.5 5.5 0 0 0 4 14.5A5.5 5.5 0 0 0 9.5 20H13\"></path>", true)),
];

pub fn render(comp: &ComponentNode) -> String {
    let placeholder = comp
        .items
        .iter()
        .find(|i| i.item_type == "text" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .unwrap_or_else(|| label_of(comp));
    let aria = attr_nonempty(comp, "aria-label")
        .map(esc)
        .unwrap_or_else(|| placeholder.clone());
    let id = widget_id(comp, "content");
    let tools = TOOLS
        .iter()
        .map(|tool| match tool {
            Some((label, icon, idle_disabled)) => {
                let dd = if *idle_disabled { " data-disabled=\"\"" } else { "" };
                format!(
                    "<button type=\"button\" aria-pressed=\"false\" data-state=\"off\"{dd} data-slot=\"toggle\" aria-label=\"{label}\" disabled>{SVG_OPEN}{icon}</svg></button>"
                )
            }
            None => "<div data-orientation=\"vertical\" role=\"none\" data-slot=\"separator\"></div>".to_string(),
        })
        .collect::<String>();
    format!(
        "<div data-slot=\"rich-text-editor\"><div data-slot=\"rich-text-editor-toolbar\" role=\"toolbar\" aria-label=\"Text formatting\" aria-controls=\"{id}\">{tools}</div><div data-slot=\"rich-text-editor-content-wrapper\"><p aria-hidden=\"true\" data-slot=\"rich-text-editor-placeholder\">{placeholder}</p><div><div role=\"textbox\" id=\"{id}\" aria-multiline=\"true\" aria-readonly=\"true\" aria-label=\"{aria}\"><p><br></p></div></div></div></div>"
    )
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

    fn fixture() -> ComponentNode {
        let mut c = stub("rich-text-editor", "Write something…");
        let mut t = extra("text", "Write something…");
        t.config.insert("aria-label".into(), "Post body".into());
        c.items.push(t);
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<textarea"));
        assert!(!html.contains("<label"));
        assert!(!html.contains("data-slot=\"rich-text-editor-control\""));
        assert!(!html.contains("<dialog"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("tiptap"));
        assert!(!html.contains("contenteditable"));
        assert!(!html.contains("ProseMirror"));
    }

    #[test]
    fn toolbar_matches_react_toggle_and_separator_slots() {
        let html = render(&fixture());
        assert!(html.starts_with("<div data-slot=\"rich-text-editor\"><div data-slot=\"rich-text-editor-toolbar\" role=\"toolbar\" aria-label=\"Text formatting\" aria-controls=\""));
        assert_eq!(html.matches("data-slot=\"toggle\"").count(), 14);
        assert_eq!(html.matches("data-slot=\"separator\"").count(), 3);
        assert_eq!(html.matches(" disabled>").count(), 14);
        assert_eq!(html.matches("data-disabled=\"\"").count(), 2);
        assert!(html.contains(&format!(
            "<button type=\"button\" aria-pressed=\"false\" data-state=\"off\" data-slot=\"toggle\" aria-label=\"Bold\" disabled>{SVG_OPEN}<path d=\"M6 12h9a4 4 0 0 1 0 8H7a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1h7a4 4 0 0 1 0 8\"></path></svg></button>"
        )));
        assert!(
            html.contains("data-disabled=\"\" data-slot=\"toggle\" aria-label=\"Undo\" disabled>")
        );
        // Icon buttons: no visible label text.
        assert!(!html.contains(">Bold</button>"));
        reject_interact(&html);
    }

    #[test]
    fn content_is_placeholder_over_empty_readonly_textbox() {
        let html = render(&fixture());
        let id = widget_id(&fixture(), "content");
        assert!(html.ends_with(&format!(
            "<div data-slot=\"rich-text-editor-content-wrapper\"><p aria-hidden=\"true\" data-slot=\"rich-text-editor-placeholder\">Write something…</p><div><div role=\"textbox\" id=\"{id}\" aria-multiline=\"true\" aria-readonly=\"true\" aria-label=\"Post body\"><p><br></p></div></div></div></div>"
        )));
        assert!(html.contains(&format!("aria-controls=\"{id}\"")));
        assert_eq!(html.matches("Write something…").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn label_only_uses_label_as_placeholder_and_name() {
        let html = render(&stub("rich-text-editor", "Release notes"));
        assert!(html.contains("data-slot=\"rich-text-editor-placeholder\">Release notes</p>"));
        assert!(html.contains("aria-label=\"Release notes\"><p><br></p>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_textarea_surf() {
        let c = stub("rich-text-editor", "Release notes");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("rich-text-editor", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<textarea data-slot=\"rich-text-editor-control\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&fixture());
            reject_interact(&html);
            assert!(html.contains("data-slot=\"rich-text-editor-content-wrapper\""));
        });
    }

    #[test]
    fn chrome_is_token_only_and_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"rich-text-editor-toolbar\"] [data-slot=\"toggle\"] {"));
        assert!(css.contains("width: 2rem; height: 2rem; padding: 0; border: 0;"));
        assert!(css.contains("[data-slot=\"rich-text-editor-toolbar\"] [data-slot=\"toggle\"][data-disabled] { opacity: 0.5; }"));
        assert!(css.contains("[data-slot=\"rich-text-editor-placeholder\"] {"));
        assert!(css.contains("min-height: 10rem"));
        assert!(css.contains("var(--cronus-surface-base)"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(!css.contains("[data-slot=\"rich-text-editor-toolbar\"] button {"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("tiptap"));
    }
}
