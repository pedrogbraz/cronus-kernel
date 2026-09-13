//! Dedicated CardStack renderer. DOM matches React idle:
//! `<div data-slot="card-stack">` plus `card-stack-item` from texts (2–3
//! stacked). CSS offset in COMPONENT_CHROME. Not catalog `display()` SURF
//! `<section>`. Not interact `card()`.

use crate::cronus_ui_kit::texts;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let items = items_of(comp)
        .into_iter()
        .map(|t| format!("<div data-slot=\"card-stack-item\">{t}</div>"))
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"card-stack\">{items}</div>")
}

fn items_of(comp: &ComponentNode) -> Vec<String> {
    let mut out = texts(comp);
    if out.len() > 3 {
        out.truncate(3);
    }
    while out.len() < 2 {
        out.push(format!("Card {}", out.len() + 1));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const DISPLAY_SURF: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem";

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn stack(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("card-stack", items.first().copied().unwrap_or("Front"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_display(html: &str) {
        assert!(!html.contains(DISPLAY_SURF));
        assert!(!html.contains("<section"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("display("));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_is_div_with_stacked_items_not_display_section() {
        let html = render(&stack(&["Alpha", "Beta"]));
        assert!(html.starts_with("<div data-slot=\"card-stack\">"));
        assert_eq!(html.matches("data-slot=\"card-stack-item\"").count(), 2);
        assert!(html.contains("<div data-slot=\"card-stack-item\">Alpha</div>"));
        assert!(html.contains("<div data-slot=\"card-stack-item\">Beta</div>"));
        assert_eq!(
            html,
            "<div data-slot=\"card-stack\"><div data-slot=\"card-stack-item\">Alpha</div><div data-slot=\"card-stack-item\">Beta</div></div>"
        );
        reject_display(&html);
    }

    #[test]
    fn three_texts_stack() {
        let html = render(&stack(&["Alpha", "Beta", "Gamma"]));
        assert_eq!(html.matches("data-slot=\"card-stack-item\"").count(), 3);
        assert!(html.contains(">Gamma</div>"));
        reject_display(&html);
    }

    #[test]
    fn extra_texts_capped_at_three() {
        let html = render(&stack(&["A", "B", "C", "D"]));
        assert_eq!(html.matches("data-slot=\"card-stack-item\"").count(), 3);
        assert!(!html.contains(">D</div>"));
        reject_display(&html);
    }

    #[test]
    fn label_only_still_emits_two_items() {
        let html = render(&stub("card-stack", "Front"));
        assert_eq!(html.matches("data-slot=\"card-stack-item\"").count(), 2);
        assert!(html.contains("<div data-slot=\"card-stack-item\">Front</div>"));
        assert!(html.contains("<div data-slot=\"card-stack-item\">Card 2</div>"));
        reject_display(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stack(&["A <B> & \"C\"", "Back"]));
        assert!(html.contains(
            "<div data-slot=\"card-stack-item\">A &lt;B&gt; &amp; &quot;C&quot;</div>"
        ));
        reject_display(&html);
    }

    #[test]
    fn skips_display_surf_and_interact_card() {
        let c = stub("card-stack", "Front");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        let interact = crate::cronus_ui_interact::render("card-stack", &c).unwrap();
        assert!(interact.starts_with("<section data-slot=\"card-stack\""));
        assert!(interact.contains("style="));
        assert_ne!(html, interact);
        let display =
            crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("flip-card"))
                .unwrap();
        assert!(display.contains("<section data-slot=\"flip-card\""));
        assert!(display.contains(DISPLAY_SURF));
        assert!(!html.contains("<section"));
        reject_display(&html);
        assert_eq!(
            dedicated_fn_name("card-stack"),
            Some("cronus_ui_card_stack::render")
        );
        assert_eq!(
            renderer_kind("card-stack"),
            RendererKind::Dedicated("cronus_ui_card_stack::render")
        );
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stack(&["Alpha", "Beta"]));
            reject_display(&html);
            assert!(html.contains("data-slot=\"card-stack-item\""));
        });
    }

    #[test]
    fn chrome_offset_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"card-stack\"]"));
        assert!(css.contains("[data-slot=\"card-stack-item\"]"));
        assert!(css.contains("position: relative"));
        assert!(css.contains("position: absolute"));
        assert!(css.contains("translate(10px, 10px)"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(DISPLAY_SURF));
    }
}
