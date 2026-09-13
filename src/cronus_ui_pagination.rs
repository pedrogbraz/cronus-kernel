//! Dedicated Pagination renderer. Native nav of labelled buttons. Chrome in CSS.

use crate::parser::ComponentNode;

pub fn render(_comp: &ComponentNode) -> String {
    "<nav data-slot=\"pagination\" aria-label=\"Pagination\" data-animate><a data-slot=\"pagination-previous\" href=\"#\" aria-label=\"Go to previous page\">Previous</a><a data-slot=\"pagination-link\" href=\"#\" aria-current=\"page\">1</a><a data-slot=\"pagination-link\" href=\"#\">2</a><a data-slot=\"pagination-next\" href=\"#\" aria-label=\"Go to next page\">Next</a></nav>".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    #[test]
    fn labelled_nav() {
        let html = render(&stub("pagination", "Pages"));
        assert!(html.contains("data-slot=\"pagination\""));
        assert!(html.contains("aria-label=\"Pagination\""));
        assert!(html.contains("Previous"));
        assert!(!html.contains("zinc-"));
        let interact =
            crate::cronus_ui_interact::render("pagination", &stub("pagination", "Pages")).unwrap();
        assert_ne!(html, interact);
    }
}
