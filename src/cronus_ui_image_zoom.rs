//! Dedicated ImageZoom renderer. DOM matches React idle:
//! `<button type="button" data-slot="image-zoom" data-state="idle">` wrapping
//! `image-zoom-content` and `image-zoom-indicator` (ZoomIn icon). Idle only —
//! pointer-tracked zoom needs JS; CSS hover scale in COMPONENT_CHROME. The
//! accessible name is React's `"Zoom image"`, extended with `: {alt}` only when
//! an image `alt` exists (URL text + another text). Width mirrors the audit
//! fixture's `w-72`. Not catalog `fx()` SURF box.

use crate::cronus_ui_kit::{label_of, texts};
use crate::parser::ComponentNode;

const ZOOM_IN_SVG: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><circle cx=\"11\" cy=\"11\" r=\"8\"></circle><line x1=\"21\" x2=\"16.65\" y1=\"21\" y2=\"16.65\"></line><line x1=\"11\" x2=\"11\" y1=\"8\" y2=\"14\"></line><line x1=\"8\" x2=\"14\" y1=\"11\" y2=\"11\"></line></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let labels = texts(comp);
    let src = labels.iter().find(|t| looks_like_src(t)).cloned();
    let (inner, aria) = match src {
        Some(src) => {
            let alt = labels
                .iter()
                .find(|t| !looks_like_src(t))
                .cloned()
                .unwrap_or_default();
            let aria = if alt.is_empty() {
                "Zoom image".to_string()
            } else {
                format!("Zoom image: {alt}")
            };
            (format!("<img src=\"{src}\" alt=\"{alt}\">"), aria)
        }
        None => (label_of(comp), "Zoom image".to_string()),
    };
    format!(
        "<button type=\"button\" data-slot=\"image-zoom\" data-state=\"idle\" aria-pressed=\"false\" aria-label=\"{aria}\"><span data-slot=\"image-zoom-content\">{inner}</span><span aria-hidden=\"true\" data-slot=\"image-zoom-indicator\">{ZOOM_IN_SVG}</span></button>"
    )
}

fn looks_like_src(s: &str) -> bool {
    let s = s.trim();
    s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("data:")
        || s.starts_with('/')
        || s.starts_with("./")
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

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("onpointer"));
        assert!(!html.contains("zinc-"));
    }

    /// wave1t: children text is content, not alt — React names it "Zoom image".
    #[test]
    fn root_is_idle_button_with_content_and_icon() {
        let html = render(&stub("image-zoom", "Zoom"));
        assert_eq!(
            html,
            format!("<button type=\"button\" data-slot=\"image-zoom\" data-state=\"idle\" aria-pressed=\"false\" aria-label=\"Zoom image\"><span data-slot=\"image-zoom-content\">Zoom</span><span aria-hidden=\"true\" data-slot=\"image-zoom-indicator\">{ZOOM_IN_SVG}</span></button>")
        );
        assert!(!html.contains("<img"));
        reject_fx(&html);
    }

    #[test]
    fn extra_url_text_becomes_img() {
        let mut c = stub("image-zoom", "Hero");
        c.items.push(extra("text", "https://cdn.example/hero.png"));
        let html = render(&c);
        assert!(html.contains(
            "<span data-slot=\"image-zoom-content\"><img src=\"https://cdn.example/hero.png\" alt=\"Hero\"></span>"
        ));
        assert!(html.contains("aria-label=\"Zoom image: Hero\""));
        reject_fx(&html);
    }

    #[test]
    fn url_label_emits_img() {
        let html = render(&stub("image-zoom", "https://cdn.example/a.png"));
        assert!(html.contains("<img src=\"https://cdn.example/a.png\" alt=\"\">"));
        assert!(html.contains("aria-label=\"Zoom image\""));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("image-zoom", "A <B> & \"C\""));
        assert!(html.contains(
            "<span data-slot=\"image-zoom-content\">A &lt;B&gt; &amp; &quot;C&quot;</span>"
        ));
        reject_fx(&html);
    }

    #[test]
    fn url_attr_is_escaped() {
        let html = render(&stub("image-zoom", "https://cdn.example/a.png?x=1&y=\"z\""));
        assert!(html.contains("<img src=\"https://cdn.example/a.png?x=1&amp;y=&quot;z&quot;\" alt=\"\">"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("image-zoom", "Photo");
        let html = render(&c);
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), html);
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert_ne!(html, fx);
        reject_fx(&html);
        assert_eq!(dedicated_fn_name("image-zoom"), Some("cronus_ui_image_zoom::render"));
        assert_eq!(
            renderer_kind("image-zoom"),
            RendererKind::Dedicated("cronus_ui_image_zoom::render")
        );
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("image-zoom", "Photo"));
            reject_fx(&html);
            assert!(html.contains("data-state=\"idle\""));
        });
    }

    #[test]
    fn chrome_mirrors_w72_and_translucent_indicator() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"image-zoom\"] {\n  position: relative; display: block; width: 18rem; overflow: hidden;"
        ));
        assert!(css.contains("background: color-mix(in oklab, var(--cronus-surface-overlay) 90%, transparent);"));
        assert!(css.contains("[data-slot=\"image-zoom-indicator\"] svg { width: 1rem; height: 1rem; }"));
        assert!(css.contains("transform: scale("));
        assert!(css.contains("prefers-reduced-motion: reduce"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
