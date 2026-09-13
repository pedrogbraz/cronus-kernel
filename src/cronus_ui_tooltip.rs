//! Dedicated Tooltip renderer. Native `<details>` disclosure, chrome in CSS.
//! Not interact popover SURF overlay stub.

use crate::cronus_ui_kit::{label_of, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let body = texts(comp)
        .into_iter()
        .skip(1)
        .map(|t| format!("<p>{t}</p>"))
        .collect::<Vec<_>>()
        .join("");
    let hint = if body.is_empty() {
        format!("<p>{title}</p>")
    } else {
        body
    };
    format!(
        "<details data-slot=\"tooltip\" data-animate><summary data-slot=\"tooltip-trigger\">{title}</summary><div data-slot=\"tooltip-content\" role=\"tooltip\">{hint}</div></details>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    #[test]
    fn details_tooltip() {
        let html = render(&stub("tooltip", "Hint"));
        assert!(html.contains("data-slot=\"tooltip\""));
        assert!(html.contains("role=\"tooltip\""));
        assert!(html.contains("Hint"));
        assert!(!html.contains("zinc-"));
        let interact = crate::cronus_ui_interact::render("tooltip", &stub("tooltip", "Hint")).unwrap();
        assert_ne!(html, interact);
    }
}
