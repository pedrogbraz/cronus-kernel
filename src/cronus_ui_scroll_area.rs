//! Dedicated ScrollArea renderer. DOM matches React/Radix:
//! `<div data-slot="scroll-area">` plus a viewport of text-item children
//! and `<div data-slot="scroll-bar">`.
//! Not interact `scroll()` / catalog `display()` (SURF box without viewport).

use crate::cronus_ui_kit::texts;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let children = texts(comp)
        .into_iter()
        .map(|t| format!("<div>{t}</div>"))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"scroll-area\"><div data-slot=\"scroll-area-viewport\" tabindex=\"0\">{children}</div><div data-slot=\"scroll-bar\"></div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn area(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("scroll-area", items.first().copied().unwrap_or("Notes"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.starts_with("<section"));
        assert!(!html.starts_with("<nav"));
        assert!(!html.contains("<section data-slot=\"scroll-area\""));
        assert!(!html.contains("<nav data-slot=\"scroll-area\""));
        assert!(!html.contains("<details"));
        assert!(!html.contains("<summary"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("padding:1rem;display:flex;flex-direction:column;gap:0.5rem"));
        assert!(!html.contains("max-height:12rem;overflow:auto"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("display("));
        assert!(!html.contains("nav("));
    }

    #[test]
    fn root_is_scroll_area_with_viewport_not_display_surf() {
        let html = render(&area(&["Alpha", "Beta"]));
        assert!(html.starts_with("<div data-slot=\"scroll-area\">"));
        assert!(html.contains("<div data-slot=\"scroll-area-viewport\" tabindex=\"0\">"));
        assert!(html.contains("<div>Alpha</div>"));
        assert!(html.contains("<div>Beta</div>"));
        assert!(html.contains("<div data-slot=\"scroll-bar\"></div>"));
        assert_eq!(html.matches("data-slot=\"scroll-area-viewport\"").count(), 1);
        assert_eq!(html.matches("data-slot=\"scroll-bar\"").count(), 1);
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"scroll-area\"><div data-slot=\"scroll-area-viewport\" tabindex=\"0\"><div>Alpha</div><div>Beta</div></div><div data-slot=\"scroll-bar\"></div></div>"
        );
    }

    #[test]
    fn extra_text_items_become_viewport_children() {
        let mut c = stub("scroll-area", "Notes");
        c.items.push(extra("text", "Line two"));
        c.items.push(extra("text", "Line three"));
        let html = render(&c);
        assert!(html.contains("<div>Notes</div>"));
        assert!(html.contains("<div>Line two</div>"));
        assert!(html.contains("<div>Line three</div>"));
        assert_eq!(
            html.matches("data-slot=\"scroll-area-viewport\"").count(),
            1
        );
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_viewport_child() {
        let html = render(&stub("scroll-area", "Release notes"));
        assert!(html.contains("<div data-slot=\"scroll-area\">"));
        assert!(html.contains("<div data-slot=\"scroll-area-viewport\" tabindex=\"0\">"));
        assert!(html.contains("<div>Release notes</div>"));
        assert!(html.contains("<div data-slot=\"scroll-bar\"></div>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_scroll_surf() {
        let c = area(&["Alpha", "Beta"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("scroll-area", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<div data-slot=\"scroll-area\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("max-height:12rem;overflow:auto"));
        assert!(!interact.contains("data-slot=\"scroll-area-viewport\""));
        assert!(!interact.contains("data-slot=\"scroll-bar\""));
        assert!(html.contains("data-slot=\"scroll-area-viewport\""));
        assert!(html.contains("data-slot=\"scroll-bar\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&area(&["Alpha"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"scroll-area\"]"));
        assert!(css.contains("[data-slot=\"scroll-area-viewport\"]"));
        assert!(css.contains("[data-slot=\"scroll-bar\"]"));
        assert!(css.contains("position: relative"));
        assert!(css.contains("overflow: hidden"));
        assert!(css.contains("overflow: auto"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
