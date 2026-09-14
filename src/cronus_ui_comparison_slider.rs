//! Dedicated ComparisonSlider renderer. DOM matches React idle:
//! `<div data-slot="comparison-slider">` > `comparison-after` and
//! `comparison-before` layers, each wrapping the audit fixture's centered
//! `<div>` (text-sm, surface-raised / surface-overlay), then the `aria-hidden`
//! divider and the round `role="slider"` handle at 50%. Dragging and arrow keys
//! need JS, so the handle keeps React's element and ARIA value but is
//! `aria-disabled="true"` and not focusable (a `<div>` has no `disabled`); it is
//! not dimmed because React does not dim it at idle. The label is the handle's
//! accessible name, never a layer text. Not the catalog `fx()` SURF title box.

use crate::cronus_ui_kit::{esc, item, label_of};
use crate::parser::ComponentNode;

const CHEVRONS_SVG: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m9 7-5 5 5 5\"></path><path d=\"m15 7 5 5-5 5\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let texts = body_texts(comp);
    let before = side(comp, "before", texts.first(), "Before");
    let after = side(comp, "after", texts.get(1), "After");
    let label = aria_label(comp).unwrap_or_else(|| label_of(comp));
    format!(
        "<div data-slot=\"comparison-slider\"><div data-slot=\"comparison-after\"><div>{after}</div></div><div data-slot=\"comparison-before\"><div>{before}</div></div><div aria-hidden=\"true\"></div><div role=\"slider\" aria-label=\"{label}\" aria-valuemin=\"0\" aria-valuemax=\"100\" aria-valuenow=\"50\" aria-orientation=\"horizontal\" aria-disabled=\"true\">{CHEVRONS_SVG}</div></div>"
    )
}

/// Layer labels: items other than the field label/title (React `items`).
fn body_texts(comp: &ComponentNode) -> Vec<String> {
    comp.items
        .iter()
        .filter(|i| {
            !matches!(i.item_type.as_str(), "label" | "title" | "before" | "after")
                && !i.text.is_empty()
        })
        .map(|i| esc(&i.text))
        .collect()
}

fn side(comp: &ComponentNode, kind: &str, fallback: Option<&String>, default: &str) -> String {
    if let Some(t) = item(comp, kind).filter(|s| !s.is_empty()) {
        return esc(t);
    }
    if let Some(t) = comp.props.get(kind).filter(|s| !s.is_empty()) {
        return esc(t);
    }
    fallback.cloned().unwrap_or_else(|| default.into())
}

fn aria_label(comp: &ComponentNode) -> Option<String> {
    comp.props
        .get("aria-label")
        .or_else(|| comp.items.iter().find_map(|i| i.config.get("aria-label")))
        .filter(|s| !s.is_empty())
        .map(|s| esc(s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn extra(item_type: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: item_type.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_fx(html: &str) {
        assert!(!html.contains("padding:0.75rem 1rem;position:relative;overflow:hidden"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("tabindex="));
        assert!(!html.contains("zinc-"));
    }

    /// wave1t: fixture emits `label "Comparison"` + `text "Before"` +
    /// `text "After"`; the label names the disabled handle, not a layer.
    #[test]
    fn layers_come_from_texts_label_names_disabled_handle() {
        let mut c = stub("comparison-slider", "Comparison");
        c.items.push(extra("text", "Before"));
        c.items.push(extra("text", "After"));
        let html = render(&c);
        assert_eq!(
            html,
            format!("<div data-slot=\"comparison-slider\"><div data-slot=\"comparison-after\"><div>After</div></div><div data-slot=\"comparison-before\"><div>Before</div></div><div aria-hidden=\"true\"></div><div role=\"slider\" aria-label=\"Comparison\" aria-valuemin=\"0\" aria-valuemax=\"100\" aria-valuenow=\"50\" aria-orientation=\"horizontal\" aria-disabled=\"true\">{CHEVRONS_SVG}</div></div>")
        );
        assert!(!html.contains("<div>Comparison</div>"));
        reject_fx(&html);
    }

    #[test]
    fn label_only_uses_react_fallback_labels() {
        let html = render(&stub("comparison-slider", "Comparison"));
        assert!(html.contains("<div data-slot=\"comparison-after\"><div>After</div></div>"));
        assert!(html.contains("<div data-slot=\"comparison-before\"><div>Before</div></div>"));
        reject_fx(&html);
    }

    #[test]
    fn aria_label_config_names_handle_escaped() {
        let mut c = stub("comparison-slider", "Demo");
        c.items[0].config.insert("aria-label".into(), "A & B".into());
        let html = render(&c);
        assert!(html.contains("<div role=\"slider\" aria-label=\"A &amp; B\""));
        reject_fx(&html);
    }

    #[test]
    fn before_after_item_kinds() {
        let mut c = stub("comparison-slider", "Demo");
        c.items.push(extra("before", "Draft"));
        c.items.push(extra("after", "Shipped"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"comparison-before\"><div>Draft</div>"));
        assert!(html.contains("data-slot=\"comparison-after\"><div>Shipped</div>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("comparison-slider", "Before"));
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains("padding:0.75rem 1rem;position:relative;overflow:hidden"));
        assert_ne!(html, fx);
        reject_fx(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("comparison-slider", "Before"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"comparison-before\""));
        });
    }

    /// wave1t: layer slots are bare absolute boxes; the inner fixture div
    /// carries background and `text-sm` 14px/20px.
    #[test]
    fn chrome_puts_fill_and_type_on_inner_div() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"comparison-after\"], [data-slot=\"comparison-before\"] {\n  position: absolute; inset: 0; width: 100%; height: 100%;\n}"
        ));
        assert!(css.contains(
            "[data-slot=\"comparison-after\"] > div, [data-slot=\"comparison-before\"] > div {\n  display: flex; width: 100%; height: 100%; align-items: center; justify-content: center;\n  font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg);\n}"
        ));
        assert!(css.contains("[data-slot=\"comparison-after\"] > div { background: var(--cronus-surface-raised); }"));
        assert!(css.contains("[data-slot=\"comparison-before\"] > div { background: var(--cronus-surface-overlay); }"));
        assert!(css.contains("[data-slot=\"comparison-slider\"] > [role=\"slider\"] {"));
        assert!(css.contains("[data-slot=\"comparison-slider\"] > [aria-hidden] {"));
        assert!(css.contains("clip-path: inset(0 50% 0 0)"));
        assert!(css.contains("aspect-ratio: 16 / 9"));
        assert!(!css.contains("zinc-"));
    }
}
