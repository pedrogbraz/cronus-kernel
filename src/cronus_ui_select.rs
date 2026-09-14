//! Dedicated Select renderer. Native `<select data-slot="select">`, chrome in CSS.
//! Not interact `select-control` + inline CTRL styles.

use crate::cronus_ui_kit::{choice_texts, label_of};
use crate::parser::ComponentNode;
use crate::voodoo;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let opts = choice_texts(comp)
        .into_iter()
        .map(|t| format!("<option value=\"{t}\">{t}</option>"))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<label data-slot=\"select\"{data}><span data-slot=\"label\">{label}</span><select data-slot=\"select\"{model}>{opts}</select></label>",
        data = voodoo::data("{ value: '' }"),
        model = voodoo::model("value"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    #[test]
    fn native_select_without_control_slot() {
        let html = render(&stub("select", "Plan"));
        assert!(html.contains("<select data-slot=\"select\""));
        assert!(html.contains("Plan"));
        assert!(!html.contains("select-control"));
        assert!(!html.contains("style="));
        assert!(!html.contains("zinc-"));
        let interact =
            crate::cronus_ui_interact::render("select", &stub("select", "Plan")).unwrap();
        assert_ne!(html, interact);
    }

    #[test]
    fn items_are_options_label_is_not() {
        use crate::parser::ComponentItemNode;
        let mut c = stub("select", "Plan");
        for t in ["Free", "Pro"] {
            c.items.push(ComponentItemNode {
                item_type: "item".into(),
                text: t.into(),
                link: None,
                tone: None,
                config: Default::default(),
            });
        }
        let html = render(&c);
        assert!(html.contains("data-slot=\"label\">Plan</span>"));
        assert!(html.contains("<option value=\"Free\">Free</option>"));
        assert!(html.contains("<option value=\"Pro\">Pro</option>"));
        assert!(!html.contains("<option value=\"Plan\">"));
        assert_eq!(html.matches("<option ").count(), 2);
    }
}
