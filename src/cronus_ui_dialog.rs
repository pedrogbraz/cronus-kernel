//! Dedicated Dialog renderer. Native `<dialog>` + showModal. Chrome in CSS.
//! Not interact inline SURF / max-width:28rem overlay stub.

use crate::cronus_ui_kit::{label_of, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let body = texts(comp)
        .into_iter()
        .skip(1)
        .map(|t| format!("<p data-slot=\"dialog-description\">{t}</p>"))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"dialog\"><button type=\"button\" data-slot=\"button\" data-variant=\"primary\" onclick=\"this.nextElementSibling.showModal()\">{title}</button><dialog data-slot=\"dialog-content\"><form method=\"dialog\"><div data-slot=\"dialog-title\">{title}</div>{body}<button type=\"submit\" value=\"close\" data-slot=\"button\" data-variant=\"outline\">Close</button></form></dialog></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    #[test]
    fn native_dialog_element() {
        let html = render(&stub("dialog", "Delete project"));
        assert!(html.contains("<dialog data-slot=\"dialog-content\""));
        assert!(html.contains("data-slot=\"dialog-title\""));
        assert!(html.contains("Delete project"));
        assert!(html.contains("showModal()"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("max-width:28rem"));
        let interact = crate::cronus_ui_interact::render("dialog", &stub("dialog", "Delete project")).unwrap();
        assert_ne!(html, interact);
    }
}
