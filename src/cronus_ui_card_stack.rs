//! Dedicated CardStack renderer. DOM mirrors React idle:
//! `<section data-slot="card-stack" aria-label>` plus `card-stack-item`s (2–3
//! stacked); cards behind the front one are `aria-hidden`. Cards are
//! `text`/`item` lines; the `label` names the stack. CSS offset in
//! COMPONENT_CHROME, no JS (no "show next card" button role).
//! Not the interact/catalog `<section style=…>` SURF card.

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let items = items_of(comp)
        .into_iter()
        .enumerate()
        .map(|(i, t)| {
            let hidden = if i == 0 { "" } else { " aria-hidden=\"true\"" };
            format!("<div data-slot=\"card-stack-item\"{hidden}>{t}</div>")
        })
        .collect::<Vec<_>>()
        .join("");
    let aria = comp
        .props
        .get("aria-label")
        .or_else(|| comp.items.iter().find_map(|i| i.config.get("aria-label")))
        .filter(|s| !s.is_empty())
        .map(|v| format!(" aria-label=\"{}\"", esc(v)))
        .unwrap_or_default();
    // React's first child is a visually hidden region label (`sr-only`,
    // default "Card stack"); CSS hides it via `[data-slot="card-stack"] > span`.
    format!("<section data-slot=\"card-stack\"{aria}><span>Card stack</span>{items}</section>")
}

fn items_of(comp: &ComponentNode) -> Vec<String> {
    let mut out: Vec<String> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "text" | "item") && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .take(3)
        .collect();
    if out.is_empty() {
        out.push(label_of(comp));
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
    fn root_is_section_with_stacked_items_like_react() {
        let html = render(&stack(&["Alpha", "Beta"]));
        assert_eq!(
            html,
            "<section data-slot=\"card-stack\"><span>Card stack</span><div data-slot=\"card-stack-item\">Alpha</div><div data-slot=\"card-stack-item\" aria-hidden=\"true\">Beta</div></section>"
        );
        reject_display(&html);
    }

    /// Audit fixture: `label "Stack"`, `text "One"`, `text "Two"`,
    /// `aria-label:` on the last text. React: `<section aria-label="Stack">`
    /// with cards One/Two only.
    #[test]
    fn fixture_label_names_stack_not_a_card() {
        let mut c = stub("card-stack", "Stack");
        c.items.push(extra("text", "One"));
        let mut two = extra("text", "Two");
        two.config.insert("aria-label".into(), "Stack".into());
        c.items.push(two);
        let html = render(&c);
        assert_eq!(
            html,
            "<section data-slot=\"card-stack\" aria-label=\"Stack\"><span>Card stack</span><div data-slot=\"card-stack-item\">One</div><div data-slot=\"card-stack-item\" aria-hidden=\"true\">Two</div></section>"
        );
        reject_display(&html);
    }

    #[test]
    fn three_texts_stack() {
        let html = render(&stack(&["Alpha", "Beta", "Gamma"]));
        assert_eq!(html.matches("data-slot=\"card-stack-item\"").count(), 3);
        assert!(html.contains(">Gamma</div>"));
        assert_eq!(html.matches("aria-hidden=\"true\"").count(), 2);
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
        assert!(
            html.contains("<div data-slot=\"card-stack-item\" aria-hidden=\"true\">Card 2</div>")
        );
        reject_display(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stack(&["A <B> & \"C\"", "Back"]));
        assert!(html
            .contains("<div data-slot=\"card-stack-item\">A &lt;B&gt; &amp; &quot;C&quot;</div>"));
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
        assert!(html.starts_with("<section data-slot=\"card-stack\">"));
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
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(DISPLAY_SURF));
    }

    /// Wave 1s geometry parity with React `CardStack className="w-72"`:
    /// section 288×224; front 288×224 (`translate: 0px`, z 2); next card
    /// `translate: 10px 10px` + framer `scale(.96) rotate(-1.5deg)` from top
    /// center → bbox 282×222 at (16, 6.5); radius rounded-2xl (22px).
    #[test]
    fn chrome_geometry_matches_react() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("height: 14rem; width: 18rem; max-width: 24rem;\n  line-height: 1.5;"));
        assert!(css.contains(
            "[data-slot=\"card-stack\"] > span {\n  position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;"
        ));
        assert!(css.contains("border-radius: calc(var(--cronus-radius, 14px) + 8px);\n  border: 1px solid var(--cronus-border);\n  background: var(--cronus-surface-raised);\n  padding: 1.25rem;"));
        assert!(
            css.contains("[data-slot=\"card-stack-item\"]:nth-of-type(1) { translate: 0px 0px; }")
        );
        assert!(css.contains(
            "[data-slot=\"card-stack-item\"]:nth-of-type(2) {\n  translate: 10px 10px; transform: scale(0.96) rotate(-1.5deg);\n}"
        ));
        assert!(css.contains(
            "[data-slot=\"card-stack-item\"]:nth-of-type(3) {\n  translate: 20px 20px; transform: scale(0.92) rotate(-3deg);\n}"
        ));
        assert!(css.contains("[data-slot=\"card-stack-item\"]:nth-last-of-type(1) { z-index: 1; }"));
        assert!(css.contains("[data-slot=\"card-stack-item\"]:nth-last-of-type(2) { z-index: 2; }"));
        assert!(!css.contains("translate(10px, 10px)"));
        assert!(!css.contains("height: 14rem; width: 100%; max-width: 24rem;"));
    }
}
