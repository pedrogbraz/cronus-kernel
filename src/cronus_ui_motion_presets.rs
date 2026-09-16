//! Dedicated motion-presets renderer. `motion-presets` in React is a module of
//! framer variants (`fadeIn` / `fadeInUp` / `scaleIn`), not a component; the
//! audit fixture renders a static demo list:
//! `<div data-slot="motion-presets">` + one `motion-preset` per preset name.
//! Kernel mirrors that DOM and its static look. Zero JS, no framer.
//! Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::esc;
use crate::parser::ComponentNode;

const DEFAULT_PRESETS: &[&str] = &["fade-in", "fade-in-up", "scale-in"];

pub fn render(comp: &ComponentNode) -> String {
    let mut presets: Vec<String> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "text" | "item") && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if presets.is_empty() {
        presets = DEFAULT_PRESETS.iter().map(|p| (*p).to_string()).collect();
    }
    let rows = presets
        .into_iter()
        .map(|p| format!("<div data-slot=\"motion-preset\" data-preset=\"{p}\">{p}</div>"))
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"motion-presets\">{rows}</div>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";
    const DEFAULT_HTML: &str = "<div data-slot=\"motion-presets\"><div data-slot=\"motion-preset\" data-preset=\"fade-in\">fade-in</div><div data-slot=\"motion-preset\" data-preset=\"fade-in-up\">fade-in-up</div><div data-slot=\"motion-preset\" data-preset=\"scale-in\">scale-in</div></div>";

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
        assert!(!html.contains("<style"));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<span"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
        assert!(!html.contains("framer"));
    }

    /// Audit fixture shape: no label prop, so the emitter writes
    /// `label "default"` (fixture id) plus one `text` per preset. React shows
    /// only the preset names; the id must not become a row.
    #[test]
    fn fixture_items_are_rows_label_is_not() {
        let mut c = stub("motion-presets", "default");
        for p in DEFAULT_PRESETS {
            c.items.push(extra("text", p));
        }
        let html = render(&c);
        assert_eq!(html, DEFAULT_HTML);
        assert!(!html.contains(">default<"));
        reject_fx(&html);
    }

    #[test]
    fn label_only_falls_back_to_default_presets() {
        let html = render(&stub("motion-presets", "Motion"));
        assert_eq!(html, DEFAULT_HTML);
        assert!(!html.contains("Motion"));
        reject_fx(&html);
    }

    #[test]
    fn custom_preset_is_escaped_in_text_and_attr() {
        let mut c = stub("motion-presets", "Motion");
        c.items.push(extra("item", "A <B> & \"C\""));
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"motion-presets\"><div data-slot=\"motion-preset\" data-preset=\"A &lt;B&gt; &amp; &quot;C&quot;\">A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("motion-presets", "Motion");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        assert!(!html.contains("<span"));
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("motion-presets"),
            Some("cronus_ui_motion_presets::render")
        );
        assert_eq!(
            renderer_kind("motion-presets"),
            RendererKind::Dedicated("cronus_ui_motion_presets::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("motion-presets", "Motion"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"motion-presets\""));
            assert_eq!(html.matches("data-slot=\"motion-preset\"").count(), 3);
        });
    }

    /// React fixture rows are unstyled blocks (16px/400, line-height 24px,
    /// no gap, no animation). Chrome must match that static look.
    #[test]
    fn chrome_matches_static_react_demo() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"motion-presets\"] {\n  display: block;\n  line-height: 1.5;\n  color: var(--cronus-fg);\n}"
        ));
        assert!(css.contains("[data-slot=\"motion-preset\"] { display: block; }"));
        assert!(!css.contains("[data-slot=\"motion-preset\"][data-preset="));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
        assert!(!css.contains("style="));
        assert!(!css.contains("framer"));
    }
}
