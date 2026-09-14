//! Dedicated ClickSpark renderer. DOM matches React idle:
//! `<div data-slot="click-spark">` + `data-slot="click-spark-content"` wrapping
//! the label. Sparks are pointer-only in React — kernel idle emits none.
//! CSS (relative overflow) lives in COMPONENT_CHROME. Zero JS, no onclick,
//! no spark spans. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    format!(
        "<div data-slot=\"click-spark\"><div data-slot=\"click-spark-content\">{}</div></div>",
        label_of(comp)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("<style"));
        assert!(!html.contains("@keyframes"));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<span"));
        assert!(!html.contains("<canvas"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setTimeout"));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("requestAnimationFrame"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("click-spark", "Spark"));
        assert_eq!(
            html,
            "<div data-slot=\"click-spark\"><div data-slot=\"click-spark-content\">Spark</div></div>"
        );
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"click-spark\""));
        assert!(html.contains("data-slot=\"click-spark-content\""));
        assert!(html.contains(">Spark</div></div>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("click-spark", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"click-spark\"><div data-slot=\"click-spark-content\">A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("click-spark", "Spark");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.contains("<span>"));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        assert!(!html.contains("<span"));
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("click-spark"),
            Some("cronus_ui_click_spark::render")
        );
        assert_eq!(
            renderer_kind("click-spark"),
            RendererKind::Dedicated("cronus_ui_click_spark::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("click-spark", "Spark"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"click-spark\""));
            assert!(html.contains("data-slot=\"click-spark-content\""));
        });
    }

    #[test]
    fn chrome_click_spark_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"click-spark\"]"));
        assert!(css.contains("[data-slot=\"click-spark-content\"]"));
        assert!(css.contains("overflow: hidden"));
        assert!(css.contains("position: relative"));
        assert!(css.contains("var(--cronus-fg)"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
    }
}
