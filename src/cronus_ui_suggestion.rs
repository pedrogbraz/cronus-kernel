//! Dedicated Suggestion renderer (AI suite). DOM matches React `Suggestions` +
//! `Suggestion` (a `Button variant="outline" size="sm"`):
//! `<section data-slot="suggestions" tabindex="0" aria-label>` > `<div>` >
//! `<button type="button" data-slot="suggestion" data-variant="outline">` per
//! text item (the label when there are none). Picking a suggestion calls a JS
//! callback, so the buttons are native `disabled` controls at React's idle
//! look. The region keeps `tabindex="0"`: it scrolls horizontally without JS.

use crate::cronus_ui_kit::{attr_nonempty, content_texts, esc, label_of};
use crate::parser::ComponentNode;

const REGION_LABEL: &str = "Suggestions";

pub fn render(comp: &ComponentNode) -> String {
    let mut texts = content_texts(comp);
    if texts.is_empty() {
        texts.push(label_of(comp));
    }
    let buttons: String = texts
        .iter()
        .map(|t| {
            format!(
                "<button type=\"button\" data-slot=\"suggestion\" data-variant=\"outline\" disabled>{t}</button>"
            )
        })
        .collect();
    let region = esc(attr_nonempty(comp, "aria-label").unwrap_or(REGION_LABEL));
    format!(
        "<section data-slot=\"suggestions\" tabindex=\"0\" aria-label=\"{region}\"><div>{buttons}</div></section>"
    )
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
    fn text_items_are_disabled_outline_buttons() {
        let mut c = stub("suggestion", "default");
        c.items.push(text("What is Cronus?"));
        c.items.push(text("Deploy?"));
        let html = render(&c);
        assert_eq!(
            html,
            "<section data-slot=\"suggestions\" tabindex=\"0\" aria-label=\"Suggestions\"><div><button type=\"button\" data-slot=\"suggestion\" data-variant=\"outline\" disabled>What is Cronus?</button><button type=\"button\" data-slot=\"suggestion\" data-variant=\"outline\" disabled>Deploy?</button></div></section>"
        );
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick"));
    }

    #[test]
    fn label_only_is_one_suggestion_and_text_is_escaped() {
        let mut c = stub("suggestion", "<i>Ask</i>");
        c.props.insert("aria-label".into(), "Ideas \"now\"".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Ideas &quot;now&quot;\""));
        assert!(html.contains("disabled>&lt;i&gt;Ask&lt;/i&gt;</button>"));
    }
}
