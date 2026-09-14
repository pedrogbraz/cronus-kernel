//! Dedicated PasswordInput renderer. Native password field, chrome in CSS.

use crate::cronus_ui_kit::{esc, item, label_of};
use crate::parser::ComponentNode;
use crate::voodoo;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let ph = esc(item(comp, "text").unwrap_or(""));
    format!(
        "<label data-slot=\"password-input\"{data}><span data-slot=\"label\">{label}</span><input data-slot=\"password-input\" type=\"password\" placeholder=\"{ph}\" autocomplete=\"current-password\"{model} /></label>",
        data = voodoo::data("{ value: '' }"),
        model = voodoo::model("value"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    #[test]
    fn password_type() {
        let html = render(&stub("password-input", "Secret"));
        assert!(html.contains("type=\"password\""));
        assert!(html.contains("data-slot=\"password-input\""));
        assert!(!html.contains("zinc-"));
        let interact =
            crate::cronus_ui_interact::render("password-input", &stub("password-input", "Secret"))
                .unwrap();
        assert_ne!(html, interact);
    }
}
