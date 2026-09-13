//! Dedicated ComparisonSlider renderer. DOM matches React idle:
//! `<div data-slot="comparison-slider">` + `comparison-before` / `comparison-after`.
//! Static 50% split — no JS drag. Not the catalog `fx()` SURF title box.

use crate::cronus_ui_kit::{esc, item, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let before = before_of(comp);
    let after = after_of(comp);
    format!(
        "<div data-slot=\"comparison-slider\"><div data-slot=\"comparison-after\">{after}</div><div data-slot=\"comparison-before\">{before}</div></div>"
    )
}

fn before_of(comp: &ComponentNode) -> String {
    if let Some(t) = item(comp, "before").filter(|s| !s.is_empty()) {
        return esc(t);
    }
    if let Some(t) = comp.props.get("before").filter(|s| !s.is_empty()) {
        return esc(t);
    }
    texts(comp)
        .into_iter()
        .next()
        .unwrap_or_else(|| "Before".into())
}

fn after_of(comp: &ComponentNode) -> String {
    if let Some(t) = item(comp, "after").filter(|s| !s.is_empty()) {
        return esc(t);
    }
    if let Some(t) = comp.props.get("after").filter(|s| !s.is_empty()) {
        return esc(t);
    }
    let t = texts(comp);
    if t.len() >= 2 {
        t[1].clone()
    } else {
        "After".into()
    }
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
        assert!(!html.contains("<span>"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_is_static_split_not_fx_box() {
        let html = render(&stub("comparison-slider", "Before"));
        assert!(html.starts_with("<div data-slot=\"comparison-slider\">"));
        assert!(html.contains("<div data-slot=\"comparison-after\">After</div>"));
        assert!(html.contains("<div data-slot=\"comparison-before\">Before</div>"));
        assert!(!html.contains("<figure"));
        assert_eq!(
            html,
            "<div data-slot=\"comparison-slider\"><div data-slot=\"comparison-after\">After</div><div data-slot=\"comparison-before\">Before</div></div>"
        );
        reject_fx(&html);
    }

    #[test]
    fn before_after_from_texts() {
        let mut c = stub("comparison-slider", "Old");
        c.items.push(extra("text", "New"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"comparison-before\">Old</div>"));
        assert!(html.contains("data-slot=\"comparison-after\">New</div>"));
        reject_fx(&html);
    }

    #[test]
    fn before_after_item_kinds() {
        let mut c = stub("comparison-slider", "Demo");
        c.items.push(extra("before", "Draft"));
        c.items.push(extra("after", "Shipped"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"comparison-before\">Draft</div>"));
        assert!(html.contains("data-slot=\"comparison-after\">Shipped</div>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("comparison-slider", "Before"));
        let fx = crate::cronus_ui_widgets::render(
            &crate::cronus_ui_widgets::test_stub("meteors"),
        )
        .unwrap();
        assert!(fx.contains("padding:0.75rem 1rem;position:relative;overflow:hidden"));
        assert!(fx.contains("<span>"));
        assert_ne!(html, fx);
        assert!(!html.contains("<span>"));
        reject_fx(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("comparison-slider", "Before"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"comparison-slider\""));
            assert!(html.contains("data-slot=\"comparison-before\""));
            assert!(html.contains("data-slot=\"comparison-after\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"comparison-slider\"]"));
        assert!(css.contains("[data-slot=\"comparison-before\"]"));
        assert!(css.contains("[data-slot=\"comparison-after\"]"));
        assert!(css.contains("aspect-ratio: 16 / 9"));
        assert!(css.contains("clip-path: inset(0 50% 0 0)"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
