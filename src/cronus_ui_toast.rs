//! Dedicated Toast renderer. Family name is `toast`. One visible toast:
//! `<div data-slot="toast" role="status">` with label text.
//! Not catalog `fx()` SURF title box. Not sonner's toaster wrapper
//! (`data-slot="sonner"` / `data-slot="toaster"`).

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
        assert!(html.starts_with("<div data-slot=\"toast\""));
        assert!(html.contains("role=\"status\""));
        assert!(html.contains("aria-live=\"polite\""));
        assert!(html.contains(">Saved</div>"));
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
        let interact = crate::cronus_ui_interact::render("toast", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("data-slot=\"toast\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("padding:0.75rem 1rem;font-size:0.875rem"));
        assert!(!interact.contains("data-slot=\"toaster\""));
        let sonner = crate::cronus_ui_sonner::render(&stub("sonner", "Saved"));
        assert!(sonner.contains("data-slot=\"sonner\""));
        assert!(sonner.contains("data-slot=\"toaster\""));
        assert!(sonner.contains("data-slot=\"toast\""));
        assert_ne!(html, sonner);
        assert!(!html.contains("data-slot=\"toaster\""));
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.contains("<span>"));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        reject_fx_and_sonner(&html);
        assert_eq!(dedicated_fn_name("toast"), Some("cronus_ui_toast::render"));
        assert_eq!(
            renderer_kind("toast"),
            RendererKind::Dedicated("cronus_ui_toast::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(
            dedicated_fn_name("sonner"),
            Some("cronus_ui_sonner::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("toast", "Saved"));
            reject_fx_and_sonner(&html);
            assert!(html.contains("role=\"status\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"toast\"]"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("padding: 0.75rem 1rem"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains(FX_BOX));
    }
}
