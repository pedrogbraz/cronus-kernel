//! Dedicated Toast renderer. Family name is `toast`. One visible toast:
//! `<div data-slot="toast" role="status">` with label text, exactly the
//! audit `ToastFixture` box (block, `px-4 py-3 text-sm rounded-lg border
//! bg-surface-floating shadow-lg`). Not catalog `fx()` SURF title box. Not
//! sonner's toaster wrapper (`data-slot="toaster"`).

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"toast\" role=\"status\" aria-live=\"polite\">{}</div>",
        label_of(comp)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    /// Every CSS rule body whose selector list is exactly `selector`.
    fn blocks(css: &str, selector: &str) -> Vec<String> {
        let open = format!("{selector} {{");
        css.match_indices(&open)
            .filter(|(i, _)| *i == 0 || css.as_bytes()[i - 1] == b'\n')
            .map(|(i, _)| {
                let body = &css[i + open.len()..];
                body[..body.find('}').unwrap_or(body.len())].to_string()
            })
            .collect()
    }

    fn reject_fx_and_sonner(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("<span"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("data-slot=\"sonner\""));
        assert!(!html.contains("data-slot=\"toaster\""));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_is_one_toast_not_fx_or_sonner_toaster() {
        let html = render(&stub("toast", "Saved"));
        assert_eq!(
            html,
            "<div data-slot=\"toast\" role=\"status\" aria-live=\"polite\">Saved</div>"
        );
        assert_eq!(html.matches("data-slot=\"toast\"").count(), 1);
        reject_fx_and_sonner(&html);
    }

    #[test]
    fn extra_text_does_not_add_toasts() {
        let mut c = stub("toast", "Saved");
        c.items.push(extra("text", "Undo"));
        let html = render(&c);
        assert!(html.contains(">Saved</div>"));
        assert!(!html.contains(">Undo</div>"));
        assert_eq!(html.matches("data-slot=\"toast\"").count(), 1);
        reject_fx_and_sonner(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("toast", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"toast\" role=\"status\" aria-live=\"polite\">A &lt;B&gt; &amp; &quot;C&quot;</div>"
        );
        reject_fx_and_sonner(&html);
    }

    #[test]
    fn skips_fx_surf_interact_and_sonner_wrapper() {
        let c = stub("toast", "Saved");
        let html = render(&c);
        let sonner = crate::cronus_ui_sonner::render(&stub("sonner", "Saved"));
        assert!(sonner.contains("data-slot=\"toaster\""));
        assert_ne!(html, sonner);
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert_ne!(html, fx);
        reject_fx_and_sonner(&html);
        assert_eq!(dedicated_fn_name("toast"), Some("cronus_ui_toast::render"));
        assert_eq!(
            renderer_kind("toast"),
            RendererKind::Dedicated("cronus_ui_toast::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("toast", "Saved"));
            reject_fx_and_sonner(&html);
            assert!(html.contains("role=\"status\""));
        });
    }

    /// wave1t: React ToastFixture is a block box (432px in the audit canvas),
    /// weight 400, `text-sm` 14px/20px, no leading dot. One toast rule only.
    #[test]
    fn chrome_matches_react_toast_fixture_box() {
        let css = crate::cronus_ui::component_chrome_css();
        let toast = blocks(css, "[data-slot=\"toast\"]");
        assert_eq!(toast.len(), 1, "{toast:?}");
        let b = &toast[0];
        assert!(b.contains("display: block"), "{b}");
        assert!(b.contains("padding: 0.75rem 1rem"), "{b}");
        assert!(
            b.contains("font-size: 0.875rem; line-height: 1.25rem"),
            "{b}"
        );
        assert!(b.contains("var(--cronus-surface-floating)"), "{b}");
        assert!(b.contains("var(--cronus-border)"), "{b}");
        assert!(!b.contains("inline-flex"), "{b}");
        assert!(!b.contains("min-width"), "{b}");
        assert!(!b.contains("font-weight"), "{b}");
        assert!(!css.contains("[data-slot=\"toast\"]::before"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
