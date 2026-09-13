//! Dedicated NumberInput renderer. Native number field, chrome in CSS.

use crate::cronus_ui_kit::{esc, item, label_of};
use crate::parser::ComponentNode;
use crate::voodoo;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let ph = esc(item(comp, "text").unwrap_or(""));
    format!(
        "<label data-slot=\"number-input\"{data}><span data-slot=\"label\">{label}</span><input data-slot=\"number-input\" type=\"number\" inputmode=\"numeric\" placeholder=\"{ph}\"{model} /></label>",
        data = voodoo::data("{ value: 0 }"),
        model = voodoo::model("value"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    #[test]
    fn number_type() {
        let html = render(&stub("number-input", "Quantity"));
        assert!(html.contains("type=\"number\""));
        assert!(html.contains("data-slot=\"number-input\""));
        assert!(!html.contains("zinc-"));
        let interact =
            crate::cronus_ui_interact::render("number-input", &stub("number-input", "Quantity"))
                .unwrap();
        assert_ne!(html, interact);
    }
}
