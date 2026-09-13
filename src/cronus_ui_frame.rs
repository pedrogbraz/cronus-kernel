//! Dedicated Frame renderer. DOM matches React:
//! `<div data-slot="frame">` plus `frame-chrome` (traffic-light dots) and
//! `frame-content` with label. Not the catalog `display()` SURF `<section>`.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"frame\"><div data-slot=\"frame-chrome\"><div aria-hidden=\"true\"><span></span><span></span><span></span></div></div><div data-slot=\"frame-content\">{}</div></div>",
        label_of(comp)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const DISPLAY_SURF: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem;";

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_display(html: &str) {
        assert!(!html.contains(DISPLAY_SURF));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<section"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("display("));
    }

    #[test]
    fn root_is_frame_with_chrome_and_content_label() {
        let html = render(&stub("frame", "Preview"));
        assert_eq!(
            html,
            "<div data-slot=\"frame\"><div data-slot=\"frame-chrome\"><div aria-hidden=\"true\"><span></span><span></span><span></span></div></div><div data-slot=\"frame-content\">Preview</div></div>"
        );
        assert!(html.starts_with("<div data-slot=\"frame\">"));
        assert!(html.contains("data-slot=\"frame-chrome\""));
        assert!(html.contains("data-slot=\"frame-content\">Preview</div>"));
        assert_eq!(html.matches("<span></span>").count(), 3);
        reject_display(&html);
    }

    #[test]
    fn extra_text_does_not_add_nodes() {
        let mut c = stub("frame", "Preview");
        c.items.push(extra("text", "https://cronus.com"));
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"frame\"><div data-slot=\"frame-chrome\"><div aria-hidden=\"true\"><span></span><span></span><span></span></div></div><div data-slot=\"frame-content\">Preview</div></div>"
        );
        assert!(!html.contains("https://cronus.com"));
        reject_display(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("frame", "A <B> & \"C\""));
        assert!(html.contains("data-slot=\"frame-content\">A &lt;B&gt; &amp; &quot;C&quot;</div>"));
        reject_display(&html);
    }

    #[test]
    fn skips_display_surf_section() {
        let html = render(&stub("frame", "Preview"));
        let interact =
            crate::cronus_ui_interact::render("frame", &stub("frame", "Preview")).unwrap();
        assert!(interact.starts_with("<section data-slot=\"frame\""));
        assert!(interact.contains("style="));
        assert!(interact.contains(DISPLAY_SURF));
        assert!(!interact.contains("data-slot=\"frame-chrome\""));
        assert!(!interact.contains("data-slot=\"frame-content\""));
        assert_ne!(html, interact);
        assert!(!html.contains("<section"));
        reject_display(&html);
        assert_eq!(dedicated_fn_name("frame"), Some("cronus_ui_frame::render"));
        assert_eq!(
            renderer_kind("frame"),
            RendererKind::Dedicated("cronus_ui_frame::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("frame", "Preview"));
            reject_display(&html);
            assert!(html.contains("data-slot=\"frame-chrome\""));
            assert!(html.contains("data-slot=\"frame-content\""));
        });
    }

    #[test]
    fn chrome_frame_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"frame\"]"));
        assert!(css.contains("[data-slot=\"frame-chrome\"]"));
        assert!(css.contains("[data-slot=\"frame-content\"]"));
        assert!(css.contains("overflow: hidden"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-error)"));
        assert!(css.contains("var(--cronus-warning)"));
        assert!(css.contains("var(--cronus-success)"));
        assert!(css.contains("var(--cronus-fg)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(DISPLAY_SURF));
    }
}
