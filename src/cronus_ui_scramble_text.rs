//! Dedicated ScrambleText renderer. DOM mirrors React's settled (resolved)
//! render: `<span data-slot="scramble-text">` + a visually hidden copy
//! (`sr-only` in React) + an aria-hidden `<span>` with the resolved phrase.
//! No scramble ticks, no random glyphs, no `setTimeout` — the zero-JS kernel
//! shows the end state React settles on. `font-mono` + the sr-only rule live
//! in COMPONENT_CHROME. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::label_of;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    format!(
        "<span data-slot=\"scramble-text\"><span>{label}</span><span aria-hidden=\"true\">{label}</span></span>"
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
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<div"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setTimeout"));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_is_span_with_hidden_copy_and_resolved_phrase() {
        let html = render(&stub("scramble-text", "Decrypt"));
        assert_eq!(
            html,
            "<span data-slot=\"scramble-text\"><span>Decrypt</span><span aria-hidden=\"true\">Decrypt</span></span>"
        );
        assert_eq!(html.matches("data-slot=").count(), 1);
        assert!(!html.contains("scramble-text-label"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("scramble-text", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<span data-slot=\"scramble-text\"><span>A &lt;B&gt; &amp; &quot;C&quot;</span><span aria-hidden=\"true\">A &lt;B&gt; &amp; &quot;C&quot;</span></span>"
        );
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("scramble-text", "Decrypt");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
        assert_eq!(
            dedicated_fn_name("scramble-text"),
            Some("cronus_ui_scramble_text::render")
        );
        assert_eq!(
            renderer_kind("scramble-text"),
            RendererKind::Dedicated("cronus_ui_scramble_text::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("scramble-text", "Decrypt"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"scramble-text\""));
        });
    }

    #[test]
    fn chrome_scramble_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"scramble-text\"] {\n  display: inline;\n  font-family: var(--cronus-font-mono, ui-monospace, monospace);\n}"));
        assert!(css
            .contains("[data-slot=\"scramble-text\"] > span:first-child {\n  position: absolute;"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("setTimeout"));
        assert!(!css.contains("<style"));
    }
}
