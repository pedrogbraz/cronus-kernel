//! Dedicated DynamicIsland renderer. DOM matches React idle:
//! `<div data-slot="dynamic-island">` plus `dynamic-island-shell` and a
//! `role="tablist"` of `dynamic-island-trigger` buttons. Static first view.
//! No motion/framer. CSS in COMPONENT_CHROME. Not catalog `fx()` SURF box.

use crate::cronus_ui_kit::{label_of, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let labels = texts(comp);
    let first = labels.first().cloned().unwrap_or_else(|| label_of(comp));
    let triggers = labels
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let selected = if i == 0 { "true" } else { "false" };
            format!(
                "<button type=\"button\" role=\"tab\" aria-selected=\"{selected}\" aria-label=\"{t}\" data-slot=\"dynamic-island-trigger\"></button>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"dynamic-island\"><div data-slot=\"dynamic-island-shell\">{first}</div><div role=\"tablist\" aria-label=\"{first}\">{triggers}</div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn views(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("dynamic-island", items.first().copied().unwrap_or("Now"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("framer"));
        assert!(!html.contains("animate"));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_is_island_with_shell_and_triggers() {
        let html = render(&views(&["Now playing", "Timer"]));
        assert!(html.starts_with("<div data-slot=\"dynamic-island\">"));
        assert!(html.contains("<div data-slot=\"dynamic-island-shell\">Now playing</div>"));
        assert!(html.contains("role=\"tablist\""));
        assert!(html.contains("aria-label=\"Now playing\""));
        assert_eq!(html.matches("data-slot=\"dynamic-island-trigger\"").count(), 2);
        assert!(html.contains(
            "<button type=\"button\" role=\"tab\" aria-selected=\"true\" aria-label=\"Now playing\" data-slot=\"dynamic-island-trigger\"></button>"
        ));
        assert!(html.contains(
            "<button type=\"button\" role=\"tab\" aria-selected=\"false\" aria-label=\"Timer\" data-slot=\"dynamic-island-trigger\"></button>"
        ));
        assert!(!html.contains(">Timer</div>"));
        reject_fx(&html);
        assert_eq!(
            html,
            "<div data-slot=\"dynamic-island\"><div data-slot=\"dynamic-island-shell\">Now playing</div><div role=\"tablist\" aria-label=\"Now playing\"><button type=\"button\" role=\"tab\" aria-selected=\"true\" aria-label=\"Now playing\" data-slot=\"dynamic-island-trigger\"></button><button type=\"button\" role=\"tab\" aria-selected=\"false\" aria-label=\"Timer\" data-slot=\"dynamic-island-trigger\"></button></div></div>"
        );
    }

    #[test]
    fn extra_text_items_become_triggers() {
        let mut c = stub("dynamic-island", "Now playing");
        c.items.push(extra("text", "Timer"));
        c.items.push(extra("text", "Silent"));
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"dynamic-island-trigger\"").count(), 3);
        assert!(html.contains("aria-selected=\"true\" aria-label=\"Now playing\""));
        assert!(html.contains("aria-selected=\"false\" aria-label=\"Timer\""));
        assert!(html.contains("aria-selected=\"false\" aria-label=\"Silent\""));
        assert!(html.contains("data-slot=\"dynamic-island-shell\">Now playing</div>"));
        reject_fx(&html);
    }

    #[test]
    fn label_only_still_emits_one_trigger() {
        let html = render(&stub("dynamic-island", "Now playing"));
        assert!(html.starts_with("<div data-slot=\"dynamic-island\">"));
        assert_eq!(html.matches("data-slot=\"dynamic-island-trigger\"").count(), 1);
        assert!(html.contains("aria-selected=\"true\""));
        assert!(html.contains("data-slot=\"dynamic-island-shell\">Now playing</div>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("dynamic-island", "A <B> & \"C\""));
        assert!(html.contains(
            "data-slot=\"dynamic-island-shell\">A &lt;B&gt; &amp; &quot;C&quot;</div>"
        ));
        assert!(html.contains("aria-label=\"A &lt;B&gt; &amp; &quot;C&quot;\""));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("dynamic-island", "Now playing");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.contains("<span>"));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        assert!(html.contains("data-slot=\"dynamic-island-shell\""));
        assert!(html.contains("data-slot=\"dynamic-island-trigger\""));
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("dynamic-island"),
            Some("cronus_ui_dynamic_island::render")
        );
        assert_eq!(
            renderer_kind("dynamic-island"),
            RendererKind::Dedicated("cronus_ui_dynamic_island::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("dynamic-island", "Now playing"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"dynamic-island-shell\""));
            assert!(html.contains("data-slot=\"dynamic-island-trigger\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"dynamic-island\"]"));
        assert!(css.contains("[data-slot=\"dynamic-island-shell\"]"));
        assert!(css.contains("[data-slot=\"dynamic-island-trigger\"]"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-fg)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains(FX_BOX));
    }
}
