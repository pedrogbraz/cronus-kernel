//! Dedicated CopyButton renderer. DOM matches React idle state (Button ghost/icon):
//! `<button data-slot="copy-button" data-variant="ghost" type="button" aria-label>`
//! with the lucide `Copy` glyph and an empty `aria-live` sr-only span.
//! Divergence: zero-JS kernel, so the clipboard write and the "copied" state
//! (Check glyph + "Copied" announcement) cannot happen. The button is emitted
//! as the same native element with `disabled` (inert, announced unavailable)
//! and React's idle look — no dimming, since React doesn't dim it; authors dim
//! a deliberately disabled copy button with `data-disabled`.
//! Not interact clipboard `onclick` or inline BASE/SURF styles.

use crate::cronus_ui_kit::{attr_nonempty, choice, esc};
use crate::parser::ComponentNode;

const COPY_ICON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><rect width=\"14\" height=\"14\" x=\"8\" y=\"8\" rx=\"2\" ry=\"2\"></rect><path d=\"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let aria = esc(aria_label(comp)
        .or_else(|| attr_nonempty(comp, "copyLabel"))
        .or_else(|| attr_nonempty(comp, "copy-label"))
        .unwrap_or("Copy"));
    let variant = choice(
        comp,
        "variant",
        &[
            "primary",
            "secondary",
            "outline",
            "ghost",
            "destructive",
            "link",
        ],
    )
    .unwrap_or("ghost");
    let size = choice(comp, "size", &["icon", "icon-sm", "sm", "md", "lg"]).unwrap_or("icon");
    idle_button_styled(&aria, variant, size)
}

/// Idle, JS-less CopyButton; `aria` must already be escaped. Shared by
/// families that embed React's CopyButton (code-block header).
pub fn idle_button(aria: &str) -> String {
    idle_button_styled(aria, "ghost", "icon")
}

fn idle_button_styled(aria: &str, variant: &str, size: &str) -> String {
    let size_attr = if size == "icon" {
        String::new()
    } else {
        format!(" data-size=\"{}\"", esc(size))
    };
    format!(
        "<button data-slot=\"copy-button\" data-variant=\"{}\"{size_attr} type=\"button\" aria-label=\"{aria}\" disabled>{COPY_ICON}<span aria-live=\"polite\"></span></button>",
        esc(variant)
    )
}

fn aria_label(comp: &ComponentNode) -> Option<&str> {
    attr_nonempty(comp, "aria-label")
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
            format!("<button data-slot=\"copy-button\" data-variant=\"ghost\" type=\"button\" aria-label=\"Copy\" disabled>{COPY_ICON}<span aria-live=\"polite\"></span></button>")
        );
        assert!(!html.contains(">Copy<"));
        reject_interact(&html);
    }

    /// Wave 1t policy: a control that needs JS is the native element with
    /// `disabled`, not a live button that silently does nothing; the idle look
    /// is kept (dimming only via author `data-disabled`).
    #[test]
    fn js_only_copy_is_disabled_but_not_dimmed() {
        let html = render(&stub("copy-button", "Copy"));
        assert!(html.contains(" disabled>"));
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"copy-button\"][data-disabled] { opacity: 0.5; pointer-events: none; }"
        ));
        assert!(!css.contains("[data-slot=\"copy-button\"]:disabled { opacity: 0.5;"));
    }

    #[test]
    fn aria_label_from_item_config() {
        let mut c = stub("copy-button", "Copy link");
        c.items[0]
            .config
            .insert("aria-label".into(), "Copy link".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Copy link\""));
        assert!(!html.contains(">Copy link<"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_clipboard_onclick() {
        let c = stub("copy-button", "Copy");
        let html = render(&c);
        reject_interact(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"copy-button\"]"));
        assert!(css.contains("display: inline-flex"));
        assert!(css.contains("line-height: 1.25rem; text-decoration: none;\n  outline: none; border: 0 solid transparent;"));
        assert!(css.contains("width: 2.25rem; height: 2.25rem; padding: 0;"));
        assert!(css.contains("[data-slot=\"copy-button\"] svg {\n  width: 1rem; height: 1rem;"));
        assert!(css.contains("[data-slot=\"copy-button\"] > [aria-live] {"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
