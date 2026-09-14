//! Dedicated FlipCard renderer. DOM matches React:
//! `<div data-slot="flip-card">` plus `flip-card-front` (label) and
//! `flip-card-back` (extra text). Static CSS 3D in COMPONENT_CHROME; no JS.
//! Not the catalog `display()` SURF `<section>`.

use crate::cronus_ui_kit::{attr, esc, label_of};
use crate::parser::ComponentNode;

/// Faces mirror React `items[0]` / `items[1]`. With fewer than two `text`
/// lines the `label` stays the front and the single line is the back.
pub fn render(comp: &ComponentNode) -> String {
    let faces: Vec<String> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "text" | "item") && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    let (front, back) = match faces.as_slice() {
        [f, b, ..] => (f.clone(), b.clone()),
        [b] => (label_of(comp), b.clone()),
        [] => (label_of(comp), String::new()),
    };
    // `aria-label:` after an item line lands in that item's config (the
    // tokenizer has no newlines), so look there as well as in props.
    let aria_label = attr(comp, "aria-label");
    // React hover trigger: `role="group"` only when named, always `tabindex=0`
    // (CSS `:focus-within` flips it, so focus is a real, JS-free control).
    let aria = match aria_label.filter(|s| !s.is_empty()) {
        Some(v) => format!(" role=\"group\" tabindex=\"0\" aria-label=\"{}\"", esc(v)),
        None => " tabindex=\"0\"".to_string(),
    };
    // Inner `<div>` is React's preserve-3d stage that rotates on hover/focus.
    format!(
        "<div data-slot=\"flip-card\"{aria}><div><div data-slot=\"flip-card-front\">{front}</div><div data-slot=\"flip-card-back\">{back}</div></div></div>"
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
        assert!(!html.contains("onmouseenter"));
        assert!(!html.contains("addEventListener"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("display("));
    }

    #[test]
    fn root_is_flip_card_with_front_label_and_back() {
        let html = render(&stub("flip-card", "Front"));
        assert_eq!(
            html,
            "<div data-slot=\"flip-card\" tabindex=\"0\"><div><div data-slot=\"flip-card-front\">Front</div><div data-slot=\"flip-card-back\"></div></div></div>"
        );
        assert!(html.starts_with("<div data-slot=\"flip-card\""));
        assert!(html.contains("data-slot=\"flip-card-front\">Front</div>"));
        assert!(html.contains("data-slot=\"flip-card-back\"></div>"));
        reject_display(&html);
    }

    #[test]
    fn extra_text_becomes_back_face() {
        let mut c = stub("flip-card", "Front");
        c.items.push(extra("text", "Back copy"));
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"flip-card\" tabindex=\"0\"><div><div data-slot=\"flip-card-front\">Front</div><div data-slot=\"flip-card-back\">Back copy</div></div></div>"
        );
        reject_display(&html);
    }

    /// Audit fixture shape: emitter writes `label "Plan"` (from aria-label),
    /// `text "Front"`, `text "Back"`, `aria-label:"Plan"`. React faces are
    /// items[0] / items[1]; "Plan" only names the card.
    #[test]
    fn fixture_items_are_faces_label_is_aria() {
        let mut c = stub("flip-card", "Plan");
        c.items.push(extra("text", "Front"));
        let mut back = extra("text", "Back");
        back.config.insert("aria-label".into(), "Plan".into());
        c.items.push(back);
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"flip-card\" role=\"group\" tabindex=\"0\" aria-label=\"Plan\"><div><div data-slot=\"flip-card-front\">Front</div><div data-slot=\"flip-card-back\">Back</div></div></div>"
        );
        reject_display(&html);
    }

    #[test]
    fn label_is_escaped() {
        let mut c = stub("flip-card", "A <B> & \"C\"");
        c.items.push(extra("text", "D <E>"));
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"flip-card\" tabindex=\"0\"><div><div data-slot=\"flip-card-front\">A &lt;B&gt; &amp; &quot;C&quot;</div><div data-slot=\"flip-card-back\">D &lt;E&gt;</div></div></div>"
        );
        reject_display(&html);
    }

    #[test]
    fn skips_display_surf_section() {
        let html = render(&stub("flip-card", "Front"));
        assert!(!html.contains("<section"));
        reject_display(&html);
        assert_eq!(
            dedicated_fn_name("flip-card"),
            Some("cronus_ui_flip_card::render")
        );
        assert_eq!(
            renderer_kind("flip-card"),
            RendererKind::Dedicated("cronus_ui_flip_card::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("flip-card", "Front"));
            reject_display(&html);
            assert!(html.contains("data-slot=\"flip-card-front\""));
            assert!(html.contains("data-slot=\"flip-card-back\""));
        });
    }

    #[test]
    fn chrome_flip_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"flip-card\"]"));
        assert!(css.contains("[data-slot=\"flip-card-front\"]"));
        assert!(css.contains("[data-slot=\"flip-card-back\"]"));
        assert!(css.contains("perspective: 1600px"));
        assert!(css.contains("transform-style: preserve-3d"));
        assert!(css.contains("backface-visibility: hidden"));
        assert!(css.contains("rotateY(180deg)"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(css.contains("var(--cronus-surface-elevated"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(DISPLAY_SURF));
        assert!(!css.contains("onclick"));
    }

    /// Wave 1s geometry parity with the corrected React `FlipCard
    /// className="w-72"`: root 288×256 rounded-2xl (22px); the preserve-3d
    /// stage is `absolute inset-0` (was `size-full`, which resolved to 0px
    /// against a min-height-only card), so both faces measure 288×256.
    #[test]
    fn chrome_geometry_matches_react() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"flip-card\"] {\n  position: relative; isolation: isolate;\n  width: var(--cui-flip-card-w, 100%);\n  min-height: 16rem; border-radius: calc(var(--cronus-radius, 14px) + 8px);\n  perspective: 1600px;\n  line-height: 1.5;"
        ));
        assert!(css.contains(
            "[data-slot=\"flip-card\"] > div {\n  position: absolute; inset: 0;\n  transform-style: preserve-3d;"
        ));
        assert!(!css.contains(
            "position: relative; width: 100%; height: 100%;\n  transform-style: preserve-3d;"
        ));
        assert!(css.contains(
            "[data-slot=\"flip-card\"]:hover > div,\n[data-slot=\"flip-card\"]:focus-within > div {\n  transform: rotateY(180deg);\n}"
        ));
        assert!(css.contains(
            "overflow: hidden; border-radius: calc(var(--cronus-radius, 14px) + 8px);\n  border: 1px solid var(--cronus-border);"
        ));
        assert!(!css.contains("transform: rotateY(360deg)"));
        assert!(!css.contains("min-height: 16rem; border-radius: var(--cronus-radius-xl);"));
    }
}
