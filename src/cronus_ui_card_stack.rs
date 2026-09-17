//! Dedicated CardStack renderer. DOM mirrors React idle:
//! `<section data-slot="card-stack" aria-label>` + the `sr-only` region
//! name + the `card-stack-item`s (2–8 stacked, `text`/`item` lines; the
//! `label` names the stack). React cycles the front card to the back on
//! click / Space / ArrowRight (`motion.div` `layout` + spring 260/26); the
//! kernel does it with zero JS: one visually hidden radio per stack state
//! (`value` = which card is in front, page-unique `name`, ArrowRight moves
//! between them) and each card wrapped in a `display: contents` `<label
//! for>` that targets the *next* state, so clicking the front card sends it
//! to the back. COMPONENT_CHROME derives every card's depth from the checked
//! state (`:has()`), its index (`nth-of-type`) and the card count (class
//! `n-<count>`), keeps React's `10px` step, `scale 1 - 0.04·depth`,
//! `rotate -1.5deg·depth` and `z-index`, and eases the shuffle with
//! `--cronus-spring-soft`. The card `<div>`s stay decorative
//! (`aria-hidden`, `tabindex="-1"`); the radios carry each card's name.
//!
//! `heading:lg` is the docs' `<p className="font-display text-lg">` body
//! (`<p class="h-lg">` in every card).

use crate::cronus_ui_dot_pattern::HEADINGS;
use crate::cronus_ui_kit::{attr_nonempty, choice, esc, instance_id, label_of};
use crate::parser::ComponentNode;

/// Most cards one stack can hold (the stylesheet indexes 8 states).
pub const MAX_CARDS: usize = 8;

pub fn render(comp: &ComponentNode) -> String {
    let cards = items_of(comp);
    let n = cards.len();
    let name = instance_id(comp, "card-stack");
    let heading = choice(comp, "heading", HEADINGS);
    let radios = cards
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let checked = if i == 0 { " checked" } else { "" };
            format!(
                "<input type=\"radio\" name=\"{name}\" id=\"{name}-{i}\" value=\"{i}\" aria-label=\"{t}\"{checked}>"
            )
        })
        .collect::<String>();
    let items = cards
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let next = (i + 1) % n;
            let body = match heading {
                Some(h) => format!("<p class=\"h-{h}\">{t}</p>"),
                None => t.clone(),
            };
            format!(
                "<label for=\"{name}-{next}\"><div data-slot=\"card-stack-item\" aria-hidden=\"true\" tabindex=\"-1\">{body}</div></label>"
            )
        })
        .collect::<String>();
    let aria = attr_nonempty(comp, "aria-label")
        .map(|v| format!(" aria-label=\"{}\"", esc(v)))
        .unwrap_or_default();
    // React's first child is a visually hidden region label (`sr-only`,
    // default "Card stack"); CSS hides it via `[data-slot="card-stack"] > span`.
    format!(
        "<section data-slot=\"card-stack\" class=\"n-{n}\"{aria}><span>Card stack</span>{radios}{items}</section>"
    )
}

fn items_of(comp: &ComponentNode) -> Vec<String> {
    let mut out: Vec<String> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "text" | "item") && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .take(MAX_CARDS)
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
        assert!(!html.contains("onkeydown="));
        assert!(!html.contains("role=\"button\""));
        assert!(!html.contains("display("));
        assert!(!html.contains("zinc-"));
    }

    /// The front card's label targets the next state's radio; the last card's
    /// label wraps back to the first state.
    #[test]
    fn root_is_section_with_state_radios_and_stacked_items_like_react() {
        crate::cronus_ui_kit::reset_instance_ids();
        let html = render(&stack(&["Alpha", "Beta"]));
        assert_eq!(
            html,
            "<section data-slot=\"card-stack\" class=\"n-2\"><span>Card stack</span><input type=\"radio\" name=\"cui-card-stack-card-stack\" id=\"cui-card-stack-card-stack-0\" value=\"0\" aria-label=\"Alpha\" checked><input type=\"radio\" name=\"cui-card-stack-card-stack\" id=\"cui-card-stack-card-stack-1\" value=\"1\" aria-label=\"Beta\"><label for=\"cui-card-stack-card-stack-1\"><div data-slot=\"card-stack-item\" aria-hidden=\"true\" tabindex=\"-1\">Alpha</div></label><label for=\"cui-card-stack-card-stack-0\"><div data-slot=\"card-stack-item\" aria-hidden=\"true\" tabindex=\"-1\">Beta</div></label></section>"
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
        assert!(html.starts_with("<section data-slot=\"card-stack\" class=\"n-2\" aria-label=\"Stack\"><span>Card stack</span>"));
        assert_eq!(html.matches("data-slot=\"card-stack-item\"").count(), 2);
        assert!(html.contains(">One</div>"));
        assert!(html.contains(">Two</div>"));
        assert!(!html.contains(">Stack</div>"));
        reject_display(&html);
    }

    /// Docs example: three cards with `<p className="font-display text-lg">` bodies.
    #[test]
    fn three_texts_stack_with_heading_bodies() {
        let mut c = stack(&["Northwind", "Contoso", "Adventure Works"]);
        c.props.insert("heading".into(), "lg".into());
        let html = render(&c);
        assert!(html.contains("class=\"n-3\""));
        assert_eq!(html.matches("data-slot=\"card-stack-item\"").count(), 3);
        assert_eq!(html.matches("<input type=\"radio\"").count(), 3);
        assert!(html.contains("<div data-slot=\"card-stack-item\" aria-hidden=\"true\" tabindex=\"-1\"><p class=\"h-lg\">Adventure Works</p></div>"));
        reject_display(&html);
    }

    #[test]
    fn extra_texts_capped_at_max_cards() {
        let names: Vec<String> = (0..12).map(|i| format!("C{i}")).collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        let html = render(&stack(&refs));
        assert_eq!(
            html.matches("data-slot=\"card-stack-item\"").count(),
            MAX_CARDS
        );
        assert!(!html.contains(">C8</div>"));
        reject_display(&html);
    }

    #[test]
    fn label_only_still_emits_two_items() {
        let html = render(&stub("card-stack", "Front"));
        assert_eq!(html.matches("data-slot=\"card-stack-item\"").count(), 2);
        assert!(html.contains(">Front</div>"));
        assert!(html.contains(">Card 2</div>"));
        reject_display(&html);
    }

    #[test]
    fn two_stacks_on_one_page_get_distinct_radio_names() {
        crate::cronus_ui_kit::reset_instance_ids();
        let a = render(&stub("card-stack", "Front"));
        let b = render(&stub("card-stack", "Front"));
        assert!(a.contains("name=\"cui-card-stack-card-stack\" "));
        assert!(b.contains("name=\"cui-card-stack-card-stack-2\" "));
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stack(&["A <B> & \"C\"", "Back"]));
        assert!(html.contains("tabindex=\"-1\">A &lt;B&gt; &amp; &quot;C&quot;</div>"));
        assert!(html.contains("aria-label=\"A &lt;B&gt; &amp; &quot;C&quot;\""));
        reject_display(&html);
    }

    #[test]
    fn skips_display_surf_and_interact_card() {
        let c = stub("card-stack", "Front");
        let html = render(&c);
        crate::cronus_ui_kit::reset_instance_ids();
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        crate::cronus_ui_kit::reset_instance_ids();
        assert_eq!(via, render(&c));
        assert!(html.starts_with("<section data-slot=\"card-stack\""));
        reject_display(&html);
        assert_eq!(
            dedicated_fn_name("card-stack"),
            Some("cronus_ui_card_stack::render")
        );
        assert_eq!(
            renderer_kind("card-stack"),
            RendererKind::Dedicated("cronus_ui_card_stack::render")
        );
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
    /// center → bbox 282×222 at (16, 6.5); radius rounded-2xl (22px). The
    /// depth `d` of card `i` in state `k` is `(i - k) mod n`, capped at 3.
    #[test]
    fn chrome_geometry_matches_react() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("height: 14rem; width: var(--cui-card-stack-w, 100%); max-width: 24rem;\n  line-height: 1.5;"));
        assert!(css.contains(
            "[data-slot=\"card-stack\"] > span,\n[data-slot=\"card-stack\"] > input {\n  position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;"
        ));
        assert!(
            css.contains("[data-slot=\"card-stack\"] > label { display: contents; --cui-i: 0; }")
        );
        assert!(css.contains("border-radius: calc(var(--cronus-radius, 14px) + 8px);\n  border: 1px solid var(--cronus-border);\n  background: var(--cronus-surface-raised);\n  padding: 1.25rem;"));
        assert!(css.contains(
            "--cui-d: mod(calc(var(--cui-i) - var(--cui-k) + var(--cui-n)), var(--cui-n));"
        ));
        assert!(css.contains("--cui-dd: min(var(--cui-d), 3);"));
        assert!(css.contains("translate: calc(var(--cui-dd) * 10px) calc(var(--cui-dd) * 10px);"));
        assert!(css.contains("transform: scale(calc(1 - var(--cui-dd) * 0.04)) rotate(calc(var(--cui-dd) * -1.5deg));"));
        assert!(css.contains("z-index: calc(var(--cui-n) - var(--cui-d));"));
        assert!(css.contains(
            "transition: translate var(--cronus-spring-soft), transform var(--cronus-spring-soft);"
        ));
        for k in 0..MAX_CARDS {
            assert!(css.contains(&format!("[data-slot=\"card-stack\"]:has(> input[value=\"{k}\"]:checked) {{ --cui-k: {k}; }}")), "{k}");
            assert!(css.contains(&format!("[data-slot=\"card-stack\"]:has(> input[value=\"{k}\"]:checked) > label:nth-of-type({}) > [data-slot=\"card-stack-item\"] {{ pointer-events: auto; cursor: pointer; }}", k + 1)), "{k}");
            assert!(css.contains(&format!("[data-slot=\"card-stack\"]:has(> input[value=\"{k}\"]:focus-visible) > label:nth-of-type({}) > [data-slot=\"card-stack-item\"]", k + 1)), "{k}");
            assert!(
                css.contains(&format!(
                    "[data-slot=\"card-stack\"] > label:nth-of-type({}) {{ --cui-i: {k}; }}",
                    k + 1
                )),
                "{k}"
            );
        }
        assert!(css.contains("[data-slot=\"card-stack\"].n-3 { --cui-n: 3; }"));
        assert!(css.contains("prefers-reduced-motion"));
    }
}
