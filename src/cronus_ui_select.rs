//! Dedicated Select renderer. Native `<select data-slot="select">`, chrome in CSS.
//! Not interact `select-control` + inline CTRL styles.

use crate::cronus_ui_kit::{label_of, texts};
use crate::parser::ComponentNode;
use crate::voodoo;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let opts = texts(comp)
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
        let interact = crate::cronus_ui_interact::render("select", &stub("select", "Plan")).unwrap();
        assert_ne!(html, interact);
    }
}
