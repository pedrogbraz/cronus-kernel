//! Dedicated Accordion renderer. Native `<details>` / `<summary>`.

use crate::cronus_ui_kit::texts;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let items = texts(comp)
        .into_iter()
        .enumerate()
        .map(|(i, t)| {
            let open = if i == 0 { " open" } else { "" };
            format!(
                "<details{open} data-slot=\"accordion-item\"><summary data-slot=\"accordion-trigger\">{t}</summary><div data-slot=\"accordion-content\">{t}</div></details>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"accordion\" data-animate>{items}</div>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    #[test]
    fn details_summary() {
        let html = render(&stub("accordion", "What is Cronus"));
        assert!(html.contains("<details"));
        assert!(html.contains("<summary"));
        assert!(html.contains("What is Cronus"));
        assert!(!html.contains("zinc-"));
        let interact =
            crate::cronus_ui_interact::render("accordion", &stub("accordion", "What is Cronus"))
                .unwrap();
        assert_ne!(html, interact);
    }
}
