//! Dedicated RichTextEditor renderer. DOM matches React:
//! `<div data-slot="rich-text-editor">` plus `rich-text-editor-toolbar` and a
//! contenteditable-looking `rich-text-editor-content-wrapper` with static text.
//! No Tiptap JS. Not interact `textarea("rich-text-editor")`
//! (`<label><textarea data-slot="rich-text-editor-control">`).

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

const TOOLBAR: &str = concat!(
    "<div data-slot=\"rich-text-editor-toolbar\" role=\"toolbar\" aria-label=\"Text formatting\">",
    "<button type=\"button\" aria-label=\"Bold\">Bold</button>",
    "<button type=\"button\" aria-label=\"Italic\">Italic</button>",
    "<button type=\"button\" aria-label=\"Strikethrough\">Strikethrough</button>",
    "<button type=\"button\" aria-label=\"Inline code\">Inline code</button>",
    "<div data-slot=\"separator\" data-orientation=\"vertical\"></div>",
    "<button type=\"button\" aria-label=\"Heading 1\">Heading 1</button>",
    "<button type=\"button\" aria-label=\"Heading 2\">Heading 2</button>",
    "<button type=\"button\" aria-label=\"Heading 3\">Heading 3</button>",
    "<div data-slot=\"separator\" data-orientation=\"vertical\"></div>",
    "<button type=\"button\" aria-label=\"Bullet list\">Bullet list</button>",
    "<button type=\"button\" aria-label=\"Ordered list\">Ordered list</button>",
    "<button type=\"button\" aria-label=\"Blockquote\">Blockquote</button>",
    "<button type=\"button\" aria-label=\"Code block\">Code block</button>",
    "<button type=\"button\" aria-label=\"Horizontal rule\">Horizontal rule</button>",
    "<div data-slot=\"separator\" data-orientation=\"vertical\"></div>",
    "<button type=\"button\" aria-label=\"Undo\">Undo</button>",
    "<button type=\"button\" aria-label=\"Redo\">Redo</button>",
    "</div>",
);

pub fn render(comp: &ComponentNode) -> String {
    let text = label_of(comp);
    format!(
        "<div data-slot=\"rich-text-editor\">{TOOLBAR}<div data-slot=\"rich-text-editor-content-wrapper\"><div role=\"textbox\" aria-multiline=\"true\" aria-label=\"{text}\">{text}</div></div></div>"
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

    fn reject_interact(html: &str) {
        assert!(!html.contains("<textarea"));
        assert!(!html.contains("<label"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("rich-text-editor-control"));
        assert!(!html.contains("<dialog"));
        assert!(!html.contains("showModal"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("tiptap"));
        assert!(!html.contains("contenteditable=\"true\""));
        assert!(!html.contains("ProseMirror"));
    }

    #[test]
    fn root_is_editor_with_toolbar_and_static_content() {
        let html = render(&stub("rich-text-editor", "Release notes"));
        assert!(html.starts_with("<div data-slot=\"rich-text-editor\">"));
        assert!(html.contains(
            "<div data-slot=\"rich-text-editor-toolbar\" role=\"toolbar\" aria-label=\"Text formatting\">"
        ));
        assert!(html.contains("<button type=\"button\" aria-label=\"Bold\">Bold</button>"));
        assert!(html.contains("<button type=\"button\" aria-label=\"Undo\">Undo</button>"));
        assert!(html.contains("<div data-slot=\"rich-text-editor-content-wrapper\">"));
        assert!(html.contains(
            "<div role=\"textbox\" aria-multiline=\"true\" aria-label=\"Release notes\">Release notes</div>"
        ));
        reject_interact(&html);
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"rich-text-editor\">{TOOLBAR}<div data-slot=\"rich-text-editor-content-wrapper\"><div role=\"textbox\" aria-multiline=\"true\" aria-label=\"Release notes\">Release notes</div></div></div>"
            )
        );
    }

    #[test]
    fn content_is_label_not_a_textarea() {
        let html = render(&stub("rich-text-editor", "Draft body"));
        assert!(html.contains(">Draft body</div>"));
        assert!(html.contains("data-slot=\"rich-text-editor-content-wrapper\""));
        assert!(!html.contains("<textarea"));
        assert!(!html.contains("data-slot=\"rich-text-editor-control\""));
        reject_interact(&html);
    }

    #[test]
    fn extra_text_does_not_become_a_control() {
        let mut c = stub("rich-text-editor", "Release notes");
        c.items.push(extra("text", "More copy"));
        let html = render(&c);
        assert!(html.contains("aria-label=\"Release notes\">Release notes</div>"));
        assert!(!html.contains("More copy"));
        assert!(!html.contains("-control"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_textarea_surf() {
        let c = stub("rich-text-editor", "Release notes");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("rich-text-editor", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<label data-slot=\"rich-text-editor\""));
        assert!(interact.contains("<textarea data-slot=\"rich-text-editor-control\""));
        assert!(interact.contains("style="));
        assert!(!html.contains("<label"));
        assert!(!html.contains("<textarea"));
        assert!(!html.contains("rich-text-editor-control"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("rich-text-editor", "Release notes"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"rich-text-editor\""));
            assert!(html.contains("data-slot=\"rich-text-editor-toolbar\""));
            assert!(html.contains("data-slot=\"rich-text-editor-content-wrapper\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"rich-text-editor\"]"));
        assert!(css.contains("[data-slot=\"rich-text-editor-toolbar\"]"));
        assert!(css.contains("[data-slot=\"rich-text-editor-content-wrapper\"]"));
        assert!(css.contains("min-height: 10rem"));
        assert!(css.contains("var(--cronus-surface-base)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(!css.contains("[data-slot=\"rich-text-editor\"] textarea"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains("showModal"));
        assert!(!css.contains("tiptap"));
    }
}
