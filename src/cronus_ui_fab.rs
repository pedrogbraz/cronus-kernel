//! Dedicated Fab renderer. DOM matches React wrapper + main control:
//! `<div data-slot="fab"><button type="button" data-slot="fab-button" aria-label>`.
//! Not interact `buttonish()` (`data-slot="button"` + inline primary styles).

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    format!(
        "<div data-slot=\"fab\"><button type=\"button\" data-slot=\"fab-button\" aria-label=\"{label}\">{label}</button></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn reject_interact(html: &str) {
        assert!(!html.contains("data-slot=\"button\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("animate"));
        assert!(!html.contains("<input"));
        assert!(!html.contains("<label"));
    }

    #[test]
    fn root_is_wrapper_with_labelled_button() {
        let html = render(&stub("fab", "Compose"));
        assert!(html.starts_with("<div data-slot=\"fab\">"));
        assert!(html
            .contains("<button type=\"button\" data-slot=\"fab-button\" aria-label=\"Compose\">"));
        assert!(html.contains(">Compose</button>"));
        reject_interact(&html);
    }

    #[test]
    fn label_is_aria_label() {
        let html = render(&stub("fab", "New item"));
        assert!(html.contains("aria-label=\"New item\""));
        assert!(!html.contains("aria-expanded"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_buttonish() {
        let c = stub("fab", "Compose");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("fab", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("data-slot=\"fab\""));
        assert!(interact.contains("data-slot=\"button\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("background:var(--cronus-primary)"));
        assert!(!html.contains("data-slot=\"button\""));
        reject_interact(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"fab\"]"));
        assert!(css.contains("[data-slot=\"fab-button\"]"));
        assert!(css.contains("display: inline-flex"));
        assert!(css.contains("width: 3.5rem; height: 3.5rem"));
        assert!(css.contains("border-radius: 9999px"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(css.contains("var(--cronus-primary-foreground)"));
        assert!(!css.contains("zinc-"));
    }
}
