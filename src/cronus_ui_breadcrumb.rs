//! Dedicated Breadcrumb renderer. Native nav/ol. Chrome in CSS.

use crate::cronus_ui_kit::texts;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let crumbs = texts(comp);
    let n = crumbs.len();
    let items = crumbs
        .into_iter()
        .enumerate()
        .map(|(i, t)| {
            let sep = if i + 1 < n {
                "<li data-slot=\"breadcrumb-separator\" aria-hidden=\"true\">/</li>"
            } else {
                ""
            };
            let inner = if i + 1 == n {
                format!("<span data-slot=\"breadcrumb-page\" aria-current=\"page\">{t}</span>")
            } else {
                format!("<a data-slot=\"breadcrumb-link\" href=\"#\">{t}</a>")
            };
            format!("<li data-slot=\"breadcrumb-item\">{inner}</li>{sep}")
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<nav data-slot=\"breadcrumb\" aria-label=\"Breadcrumb\" data-animate><ol data-slot=\"breadcrumb-list\">{items}</ol></nav>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    #[test]
    fn nav_list() {
        let html = render(&stub("breadcrumb", "Home"));
        assert!(html.contains("data-slot=\"breadcrumb\""));
        assert!(html.contains("<ol"));
        assert!(html.contains("Home"));
        assert!(!html.contains("zinc-"));
        let interact =
            crate::cronus_ui_interact::render("breadcrumb", &stub("breadcrumb", "Home")).unwrap();
        assert_ne!(html, interact);
    }
}
