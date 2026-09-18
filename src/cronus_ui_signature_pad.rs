//! Dedicated SignaturePad renderer. DOM matches React idle (empty pad):
//! `<div data-slot="signature-pad" data-empty="true">` with
//! the `signature-pad-canvas` surface (an inner SVG for live.js polylines),
//! the `signature-pad-hint` (dashed rule + fixed "Sign here" caption) and the
//! two ghost `icon-sm` Buttons (Undo / Clear). Undo/Clear stay enabled unless
//! the author set `disabled:true`. `disabled:true` is React's `disabled`:
//! `data-disabled="true"` on the root (60% opacity), `aria-disabled` on the
//! canvas (not-allowed cursor). React's `<canvas>` is emitted as a `<div>`
//! (zero-JS renderers emit no canvas) holding `<svg data-slot="signature-pad-surface">`;
//! the chrome makes it an absolute 100%×100% block, so its box is the same.
//! Not interact `signature()` (SURF box + canvas without the canvas slot).

use crate::cronus_ui_kit::{attr_nonempty, esc, flag, label_of};
use crate::parser::ComponentNode;

/// React SignaturePad's built-in caption (not a prop).
const HINT: &str = "Sign here";

const UNDO_SVG: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M9 14 4 9l5-5\"></path><path d=\"M4 9h10.5a5.5 5.5 0 0 1 5.5 5.5a5.5 5.5 0 0 1-5.5 5.5H11\"></path></svg>";

const ERASER_SVG: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M21 21H8a2 2 0 0 1-1.42-.587l-3.994-3.999a2 2 0 0 1 0-2.828l10-10a2 2 0 0 1 2.829 0l5.999 6a2 2 0 0 1 0 2.828L12.834 21\"></path><path d=\"m5.082 11.09 8.828 8.828\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let label = aria_label(comp).unwrap_or_else(|| label_of(comp));
    let disabled = flag(comp, "disabled");
    let root = if disabled {
        " data-disabled=\"true\""
    } else {
        ""
    };
    let canvas = if disabled {
        " aria-disabled=\"true\""
    } else {
        ""
    };
    format!(
        "<div data-slot=\"signature-pad\" data-empty=\"true\"{root}><div role=\"img\" aria-label=\"{label}\"{canvas} data-slot=\"signature-pad-canvas\"><svg data-slot=\"signature-pad-surface\" width=\"100%\" height=\"100%\"></svg></div><div aria-hidden=\"true\" data-slot=\"signature-pad-hint\"><div></div><span>{HINT}</span></div><div>{undo}{clear}</div></div>",
        undo = button("signature-pad-undo", "Undo last stroke", UNDO_SVG, disabled),
        clear = button("signature-pad-clear", "Clear signature", ERASER_SVG, disabled),
    )
}

fn button(slot: &str, aria: &str, svg: &str, disabled: bool) -> String {
    let disabled_attr = if disabled { " disabled" } else { "" };
    format!(
        "<button data-slot=\"{slot}\" data-variant=\"ghost\" data-size=\"icon-sm\" type=\"button\" aria-label=\"{aria}\"{disabled_attr}>{svg}</button>"
    )
}

fn aria_label(comp: &ComponentNode) -> Option<String> {
    attr_nonempty(comp, "aria-label").map(|s| esc(s))
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
        assert!(!html.contains("<canvas"));
        assert!(!html.contains("padding:0.75rem;"));
        assert!(!html.contains("signature()"));
        assert!(html.contains("data-slot=\"signature-pad-canvas\""));
    }

    /// wave1t: hint caption is React's fixed "Sign here" (the label names the
    /// canvas). Undo/Clear are live unless the author disabled the pad.
    #[test]
    fn root_matches_react_idle_pad() {
        let html = render(&stub("signature-pad", "Signature pad"));
        let expected = format!(
            "<div data-slot=\"signature-pad\" data-empty=\"true\"><div role=\"img\" aria-label=\"Signature pad\" data-slot=\"signature-pad-canvas\"><svg data-slot=\"signature-pad-surface\" width=\"100%\" height=\"100%\"></svg></div><div aria-hidden=\"true\" data-slot=\"signature-pad-hint\"><div></div><span>Sign here</span></div><div><button data-slot=\"signature-pad-undo\" data-variant=\"ghost\" data-size=\"icon-sm\" type=\"button\" aria-label=\"Undo last stroke\">{UNDO_SVG}</button><button data-slot=\"signature-pad-clear\" data-variant=\"ghost\" data-size=\"icon-sm\" type=\"button\" aria-label=\"Clear signature\">{ERASER_SVG}</button></div></div>"
        );
        assert_eq!(html, expected);
        assert!(!html.contains("<span>Signature pad</span>"));
        assert!(!html.contains(" disabled>"));
        assert!(html.contains("data-slot=\"signature-pad-surface\""));
        reject_interact(&html);
    }

    #[test]
    fn aria_label_names_canvas_hint_stays_fixed() {
        let mut c = stub("signature-pad", "Contract");
        c.items[0]
            .config
            .insert("aria-label".into(), "Sign \"here\"".into());
        let html = render(&c);
        assert!(html
            .contains("aria-label=\"Sign &quot;here&quot;\" data-slot=\"signature-pad-canvas\""));
        assert!(html.contains("data-slot=\"signature-pad-hint\"><div></div><span>Sign here</span>"));
        reject_interact(&html);
    }

    /// Docs "Disabled": the frozen pad keeps its idle DOM, dimmed and marked.
    #[test]
    fn disabled_marks_root_and_canvas() {
        let mut c = stub("signature-pad", "Signature");
        c.props.insert("disabled".into(), "true".into());
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"signature-pad\" data-empty=\"true\" data-disabled=\"true\"><div role=\"img\" aria-label=\"Signature\" aria-disabled=\"true\" data-slot=\"signature-pad-canvas\"><svg data-slot=\"signature-pad-surface\" width=\"100%\" height=\"100%\"></svg></div>"));
        assert_eq!(html.matches(" disabled>").count(), 2);
        assert!(html.contains("data-slot=\"signature-pad-undo\""));
        assert!(html.contains("data-slot=\"signature-pad-clear\""));
        reject_interact(&html);
        let css = crate::cronus_ui::component_chrome_css();
        assert!(
            css.contains("[data-slot=\"signature-pad\"][data-disabled=\"true\"] { opacity: 0.6; }")
        );
        assert!(css.contains(
            "[data-slot=\"signature-pad-canvas\"][aria-disabled=\"true\"] { cursor: not-allowed; }"
        ));
    }

    #[test]
    fn skips_interact_signature_surf() {
        let c = stub("signature-pad", "Sign here");
        let html = render(&c);
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("signature-pad", "Sign here"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_matches_react_pad_hint_and_buttons() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "background: var(--cronus-surface-inset); color: var(--cronus-fg);\n  box-shadow: var(--cronus-shadow-xs, none);"
        ));
        assert!(css.contains(
            "[data-slot=\"signature-pad-hint\"] > div { border-top: 1px dashed var(--cronus-border-strong); }"
        ));
        assert!(css.contains(
            "display: block; margin-top: 0.375rem; font-size: 0.75rem; line-height: 1rem; color: var(--cronus-fg-muted);"
        ));
        assert!(css.contains("[data-slot=\"signature-pad\"] [data-slot=\"button\"] {"));
        assert!(css.contains("border-width: 0; font-size: 0.875rem; line-height: 1.25rem;"));
        assert!(css.contains("[data-slot=\"signature-pad\"] [data-slot=\"button\"]:disabled"));
        assert!(css.contains("[data-slot=\"signature-pad-surface\"]"));
        assert!(css.contains("[data-slot=\"signature-pad-undo\"]"));
        assert!(css.contains("[data-slot=\"signature-pad-clear\"]"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("showModal"));
    }

    #[test]
    fn surface_is_svg_not_canvas_and_actions_are_live() {
        let html = render(&stub("signature-pad", "Sign here"));
        assert!(html.contains(
            "<svg data-slot=\"signature-pad-surface\" width=\"100%\" height=\"100%\"></svg>"
        ));
        assert!(!html.contains("<canvas"));
        assert!(html.contains("data-slot=\"signature-pad-undo\""));
        assert!(html.contains("data-slot=\"signature-pad-clear\""));
        assert!(!html.contains(" disabled"));
        reject_interact(&html);
    }
}
