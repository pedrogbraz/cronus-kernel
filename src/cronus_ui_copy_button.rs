//! Dedicated CopyButton renderer. DOM matches React idle state (Button ghost/icon):
//! `<button data-slot="copy-button" data-variant="ghost" type="button" aria-label>`
//! with the lucide `Copy` glyph and an empty `aria-live` sr-only span.
//! Divergence: zero-JS kernel, so clicking does not write to the clipboard and
//! the "copied" state (Check glyph + "Copied" announcement) never appears. The
//! idle render is geometry-identical; no text label (React has none either).
//! Not interact clipboard `onclick` or inline BASE/SURF styles.

use crate::cronus_ui_kit::esc;
use crate::parser::ComponentNode;

const COPY_ICON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><rect width=\"14\" height=\"14\" x=\"8\" y=\"8\" rx=\"2\" ry=\"2\"></rect><path d=\"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let aria = esc(aria_label(comp).unwrap_or("Copy"));
    format!(
        "<button data-slot=\"copy-button\" data-variant=\"ghost\" type=\"button\" aria-label=\"{aria}\">{COPY_ICON}<span aria-live=\"polite\"></span></button>"
    )
}

fn aria_label(comp: &ComponentNode) -> Option<&str> {
    comp.props
        .get("aria-label")
        .map(String::as_str)
        .or_else(|| {
            comp.items
                .iter()
                .find_map(|i| i.config.get("aria-label").map(String::as_str))
        })
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn reject_interact(html: &str) {
        assert!(!html.contains("onclick="));
        assert!(!html.contains("navigator.clipboard"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<input"));
        assert!(!html.contains("<label"));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn root_is_idle_icon_button_without_text() {
        // Wave 1t: React renders an icon-only ghost button (36x36, no text);
        // the label ("Copy") must not leak as visible content.
        let html = render(&stub("copy-button", "Copy"));
        assert_eq!(
            html,
            format!("<button data-slot=\"copy-button\" data-variant=\"ghost\" type=\"button\" aria-label=\"Copy\">{COPY_ICON}<span aria-live=\"polite\"></span></button>")
        );
        assert!(!html.contains(">Copy<"));
        reject_interact(&html);
    }

    #[test]
    fn aria_label_from_item_config() {
        let mut c = stub("copy-button", "Copy link");
        c.items[0].config.insert("aria-label".into(), "Copy link".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Copy link\""));
        assert!(!html.contains(">Copy link<"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_clipboard_onclick() {
        let c = stub("copy-button", "Copy");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("copy-button", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("onclick=\"navigator.clipboard.writeText"));
        assert!(interact.contains("style="));
        reject_interact(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"copy-button\"]"));
        assert!(css.contains("display: inline-flex"));
        assert!(css.contains("line-height: 1.25rem; cursor: pointer; text-decoration: none;\n  outline: none; border: 0;"));
        assert!(css.contains("width: 2.25rem; height: 2.25rem; padding: 0;"));
        assert!(css.contains("[data-slot=\"copy-button\"] svg {\n  width: 1rem; height: 1rem;"));
        assert!(css.contains("[data-slot=\"copy-button\"] > [aria-live] {"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
