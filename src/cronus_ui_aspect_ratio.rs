//! Dedicated AspectRatio renderer. DOM matches React:
//! `<div data-slot="aspect-ratio">` wrapping children/label. CSS
//! `aspect-ratio: 16 / 9` lives in COMPONENT_CHROME (default). Not the
//! catalog `display()` SURF `<section>`.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!("<div data-slot=\"aspect-ratio\">{}</div>", label_of(comp))
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
    fn root_wraps_label_text_not_display_surf_section() {
        let html = render(&stub("aspect-ratio", "Cover"));
        assert_eq!(html, "<div data-slot=\"aspect-ratio\">Cover</div>");
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"aspect-ratio\""));
        assert!(html.contains(">Cover</div>"));
        reject_display(&html);
    }

    #[test]
    fn extra_text_does_not_add_nodes() {
        let mut c = stub("aspect-ratio", "Cover");
        c.items.push(extra("text", "More"));
        let html = render(&c);
        assert_eq!(html, "<div data-slot=\"aspect-ratio\">Cover</div>");
        assert!(!html.contains("More"));
        reject_display(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("aspect-ratio", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"aspect-ratio\">A &lt;B&gt; &amp; &quot;C&quot;</div>"
        );
        reject_display(&html);
    }

    #[test]
    fn skips_display_surf_section() {
        let html = render(&stub("aspect-ratio", "Cover"));
        let interact =
            crate::cronus_ui_interact::render("aspect-ratio", &stub("aspect-ratio", "Cover"))
                .unwrap();
        assert!(interact.starts_with("<section data-slot=\"aspect-ratio\""));
        assert!(interact.contains("style="));
        assert!(interact.contains(DISPLAY_SURF));
        assert_ne!(html, interact);
        assert!(!html.contains("<section"));
        reject_display(&html);
        assert_eq!(
            dedicated_fn_name("aspect-ratio"),
            Some("cronus_ui_aspect_ratio::render")
        );
        assert_eq!(
            renderer_kind("aspect-ratio"),
            RendererKind::Dedicated("cronus_ui_aspect_ratio::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("aspect-ratio", "Cover"));
            reject_display(&html);
            assert!(html.contains("data-slot=\"aspect-ratio\""));
        });
    }

    #[test]
    fn chrome_aspect_ratio_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"aspect-ratio\"]"));
        assert!(css.contains("aspect-ratio: 16 / 9"));
        assert!(css.contains("position: relative"));
        assert!(css.contains("width: 100%"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(DISPLAY_SURF));
    }
}
