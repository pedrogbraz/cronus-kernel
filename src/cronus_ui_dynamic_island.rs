//! Dedicated DynamicIsland renderer. DOM matches React idle:
//! `<div data-slot="dynamic-island">` > `dynamic-island-shell` > `<div>`
//! (flex row, active view content) plus a `role="tablist"` of
//! `dynamic-island-trigger` dots. Views come from the `text` items (the label is
//! the fixture id, not a view). Static first view: the spring morph and view
//! switching need JS, so the dots keep React's `<button role="tab">` but are
//! `disabled` (not dimmed, React does not dim them at idle).
//! Not catalog `fx()` SURF box.

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let mut views: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type != "label" && i.item_type != "title" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if views.is_empty() {
        views.push(label_of(comp));
    }
    let first = views[0].clone();
    let triggers = views
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let selected = if i == 0 { "true" } else { "false" };
            format!(
                "<button type=\"button\" role=\"tab\" aria-selected=\"{selected}\" aria-label=\"{t}\" data-slot=\"dynamic-island-trigger\" disabled></button>"
            )
        })
        .collect::<String>();
    format!(
        "<div data-slot=\"dynamic-island\"><div data-slot=\"dynamic-island-shell\"><div>{first}</div></div><div role=\"tablist\" aria-label=\"{first}\">{triggers}</div></div>"
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

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("framer"));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("zinc-"));
    }

    /// wave1t: fixture emits `label "default"` + `text "Idle"` + `text "Active"`;
    /// the label is not a view.
    #[test]
    fn views_from_texts_not_label() {
        let mut c = stub("dynamic-island", "default");
        c.items.push(extra("text", "Idle"));
        c.items.push(extra("text", "Active"));
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"dynamic-island\"><div data-slot=\"dynamic-island-shell\"><div>Idle</div></div><div role=\"tablist\" aria-label=\"Idle\"><button type=\"button\" role=\"tab\" aria-selected=\"true\" aria-label=\"Idle\" data-slot=\"dynamic-island-trigger\" disabled></button><button type=\"button\" role=\"tab\" aria-selected=\"false\" aria-label=\"Active\" data-slot=\"dynamic-island-trigger\" disabled></button></div></div>"
        );
        assert!(!html.contains("default"));
        reject_fx(&html);
    }

    #[test]
    fn label_only_still_emits_one_trigger() {
        let html = render(&stub("dynamic-island", "Now playing"));
        assert_eq!(
            html.matches("data-slot=\"dynamic-island-trigger\"").count(),
            1
        );
        assert!(html.contains("data-slot=\"dynamic-island-shell\"><div>Now playing</div></div>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("dynamic-island", "A <B> & \"C\""));
        assert!(html.contains("<div>A &lt;B&gt; &amp; &quot;C&quot;</div>"));
        assert!(html.contains("aria-label=\"A &lt;B&gt; &amp; &quot;C&quot;\""));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("dynamic-island", "Now playing");
        let html = render(&c);
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), html);
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert_ne!(html, fx);
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("dynamic-island"),
            Some("cronus_ui_dynamic_island::render")
        );
        assert_eq!(
            renderer_kind("dynamic-island"),
            RendererKind::Dedicated("cronus_ui_dynamic_island::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("dynamic-island", "Now playing"));
            reject_fx(&html);
        });
    }

    #[test]
    fn chrome_shell_row_and_dots() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"dynamic-island-shell\"] > div { display: flex; align-items: center; gap: 0.75rem; }"
        ));
        assert!(css.contains("[data-slot=\"dynamic-island-trigger\"][aria-selected=\"true\"]"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
