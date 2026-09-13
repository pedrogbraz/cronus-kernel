//! Dedicated SignaturePad renderer. Static canvas, no JS drawing.
//! `<div data-slot="signature-pad"><canvas data-slot="signature-pad-canvas">`
//! plus an optional hint. Not interact `signature()` (SURF box + canvas
//! without the signature-pad-canvas slot).

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let hint = if label.is_empty() {
        String::new()
    } else {
        format!(
            "<div aria-hidden=\"true\" data-slot=\"signature-pad-hint\"><span>{label}</span></div>"
        )
    };
    format!(
        "<div data-slot=\"signature-pad\" data-empty=\"true\"><canvas role=\"img\" aria-label=\"{label}\" data-slot=\"signature-pad-canvas\" width=\"320\" height=\"160\"></canvas>{hint}</div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn reject_interact(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("onpointer"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("padding:0.75rem;"));
        assert!(!html.contains("height:120px;background:var(--cronus-surface-inset"));
        assert!(!html.contains("signature()"));
        assert!(html.contains("data-slot=\"signature-pad-canvas\""));
    }

    #[test]
    fn root_is_pad_with_canvas_slot_not_interact_surf() {
        let html = render(&stub("signature-pad", "Sign here"));
        assert!(html.starts_with("<div data-slot=\"signature-pad\""));
        assert!(html.contains("data-empty=\"true\""));
        assert!(html.contains(
            "<canvas role=\"img\" aria-label=\"Sign here\" data-slot=\"signature-pad-canvas\" width=\"320\" height=\"160\"></canvas>"
        ));
        assert!(html.contains(
            "<div aria-hidden=\"true\" data-slot=\"signature-pad-hint\"><span>Sign here</span></div>"
        ));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"signature-pad\" data-empty=\"true\"><canvas role=\"img\" aria-label=\"Sign here\" data-slot=\"signature-pad-canvas\" width=\"320\" height=\"160\"></canvas><div aria-hidden=\"true\" data-slot=\"signature-pad-hint\"><span>Sign here</span></div></div>"
        );
    }

    #[test]
    fn hint_uses_label() {
        let html = render(&stub("signature-pad", "Contract"));
        assert!(html.contains("aria-label=\"Contract\""));
        assert!(html.contains("data-slot=\"signature-pad-hint\"><span>Contract</span>"));
        assert!(html.contains("data-slot=\"signature-pad-canvas\""));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_signature_surf() {
        let c = stub("signature-pad", "Sign here");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("signature-pad", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("data-slot=\"signature-pad\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("<canvas"));
        assert!(!interact.contains("data-slot=\"signature-pad-canvas\""));
        assert!(interact.contains("padding:0.75rem;"));
        assert!(html.contains("data-slot=\"signature-pad-canvas\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("signature-pad", "Sign here"));
            reject_interact(&html);
            assert!(!html.contains("v-data="));
            assert!(!html.contains("v-model="));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"signature-pad\"]"));
        assert!(css.contains("[data-slot=\"signature-pad-canvas\"]"));
        assert!(css.contains("[data-slot=\"signature-pad-hint\"]"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-fg-muted)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains("showModal"));
    }
}
