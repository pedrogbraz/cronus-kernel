//! Dedicated Fab renderer. DOM matches React (no `actions`):
//! `<div data-slot="fab"><button type="button" aria-label><span><svg plus/></span></button></div>`.
//! The main button has no `data-slot` in React, so CSS targets
//! `[data-slot="fab"] > button`. The label is the accessible name only — never
//! visible text. Speed-dial actions need JS (toggle), so they are not rendered.
//! Not interact `buttonish()` (`data-slot="button"` + inline primary styles).

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

const PLUS_ICON: &str = "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" aria-hidden=\"true\"><path d=\"M12 5v14M5 12h14\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    format!(
        "<div data-slot=\"fab\"><button type=\"button\" aria-label=\"{label}\"><span>{PLUS_ICON}</span></button></div>"
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
    fn root_is_wrapper_with_icon_button_no_slot_no_text() {
        // Wave 1t: React's main button has no data-slot and no visible text.
        let html = render(&stub("fab", "Create"));
        assert_eq!(
            html,
            format!("<div data-slot=\"fab\"><button type=\"button\" aria-label=\"Create\"><span>{PLUS_ICON}</span></button></div>")
        );
        assert!(!html.contains("fab-button"));
        assert!(!html.contains(">Create<"));
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
        assert!(css.contains("[data-slot=\"fab\"] > button {"));
        assert!(css.contains("[data-slot=\"fab\"] > button svg {\n  width: 1.5rem; height: 1.5rem;"));
        assert!(!css.contains("[data-slot=\"fab-button\"]"));
        assert!(css.contains("display: inline-flex"));
        assert!(css.contains("width: 3.5rem; height: 3.5rem"));
        assert!(css.contains("border-radius: 9999px"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(css.contains("var(--cronus-primary-foreground)"));
        assert!(!css.contains("zinc-"));
    }
}
