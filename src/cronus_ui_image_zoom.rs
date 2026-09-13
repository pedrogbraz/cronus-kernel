//! Dedicated ImageZoom renderer. DOM matches React idle:
//! `<button type="button" data-slot="image-zoom" data-state="idle">` wrapping
//! `image-zoom-content` and optional `image-zoom-indicator`. Idle only — no
//! JS zoom. CSS hover scale in COMPONENT_CHROME. Not catalog `fx()` SURF box.

use crate::cronus_ui_kit::{label_of, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let labels = texts(comp);
    let src = labels.iter().find(|t| looks_like_src(t)).cloned();
    let alt = labels
        .iter()
        .find(|t| !looks_like_src(t))
        .cloned()
        .unwrap_or_default();
    let inner = if let Some(src) = src {
        format!("<img src=\"{src}\" alt=\"{alt}\">")
    } else {
        label_of(comp)
    };
    let aria = if alt.is_empty() {
        "Zoom image".into()
    } else {
        format!("Zoom image: {alt}")
    };
    format!(
        "<button type=\"button\" data-slot=\"image-zoom\" data-state=\"idle\" aria-pressed=\"false\" aria-label=\"{aria}\"><span data-slot=\"image-zoom-content\">{inner}</span><span aria-hidden=\"true\" data-slot=\"image-zoom-indicator\"></span></button>"
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
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("onpointer"));
        assert!(!html.contains("addEventListener"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_is_idle_button_with_content() {
        let html = render(&stub("image-zoom", "Photo"));
        assert!(html.starts_with(
            "<button type=\"button\" data-slot=\"image-zoom\" data-state=\"idle\""
        ));
        assert!(html.contains("aria-pressed=\"false\""));
        assert!(html.contains("aria-label=\"Zoom image: Photo\""));
        assert!(html.contains("<span data-slot=\"image-zoom-content\">Photo</span>"));
        assert!(html.contains("<span aria-hidden=\"true\" data-slot=\"image-zoom-indicator\">"));
        assert!(!html.contains("<img"));
        reject_fx(&html);
        assert_eq!(
            html,
            "<button type=\"button\" data-slot=\"image-zoom\" data-state=\"idle\" aria-pressed=\"false\" aria-label=\"Zoom image: Photo\"><span data-slot=\"image-zoom-content\">Photo</span><span aria-hidden=\"true\" data-slot=\"image-zoom-indicator\"></span></button>"
        );
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
        assert!(html.contains("data-state=\"idle\""));
        reject_fx(&html);
    }

    #[test]
    fn url_label_emits_img() {
        let html = render(&stub("image-zoom", "https://cdn.example/a.png"));
        assert!(html.contains(
            "<img src=\"https://cdn.example/a.png\" alt=\"\">"
        ));
        assert!(html.contains("aria-label=\"Zoom image\""));
        assert!(!html.contains(">https://cdn.example/a.png</span>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("image-zoom", "A <B> & \"C\""));
        assert!(html.contains(
            "<span data-slot=\"image-zoom-content\">A &lt;B&gt; &amp; &quot;C&quot;</span>"
        ));
        assert!(html.contains("aria-label=\"Zoom image: A &lt;B&gt; &amp; &quot;C&quot;\""));
        reject_fx(&html);
    }

    #[test]
    fn url_attr_is_escaped() {
        let html = render(&stub(
            "image-zoom",
            "https://cdn.example/a.png?x=1&y=\"z\"",
        ));
        assert!(html.contains(
            "<img src=\"https://cdn.example/a.png?x=1&amp;y=&quot;z&quot;\" alt=\"\">"
        ));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("image-zoom", "Photo");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.contains("<span>"));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        assert!(html.contains("data-slot=\"image-zoom-content\""));
        assert!(html.starts_with("<button"));
        assert!(!html.contains("<div"));
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("image-zoom"),
            Some("cronus_ui_image_zoom::render")
        );
        assert_eq!(
            renderer_kind("image-zoom"),
            RendererKind::Dedicated("cronus_ui_image_zoom::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("image-zoom", "Photo"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"image-zoom-content\""));
            assert!(html.contains("data-state=\"idle\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"image-zoom\"]"));
        assert!(css.contains("[data-slot=\"image-zoom-content\"]"));
        assert!(css.contains("[data-slot=\"image-zoom-indicator\"]"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("transform: scale("));
        assert!(css.contains("prefers-reduced-motion: reduce"));
        assert!(css.contains("transform: none"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains(FX_BOX));
    }
}
