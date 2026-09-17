//! Dedicated DynamicIsland renderer. DOM mirrors React (`<div
//! data-slot="dynamic-island">` + `dynamic-island-shell` + a dot tablist of
//! `dynamic-island-trigger`s) with zero JS: every view stays in the shell as
//! a `<div>` whose `grid-template-columns` cell COMPONENT_CHROME opens from
//! `0fr` to `1fr` (React's `layout` spring, bounce 0.18 / 0.45s →
//! `--cronus-spring-snappy`) while the content fades and un-blurs in
//! (`opacity 0 → 1`, `blur(6px) → 0`, 200ms); each dot is a `<label>` with
//! a visually hidden radio (page-unique `name`; `value:` picks the checked
//! view) and React's `<button role="tab">` as a decorative twin. Views are
//! `item`/`text` lines (the content); `aria-label:` on an item is React's
//! `label` (the dot's and tablist's accessible name), the text when absent.
//! The component `label` is never a view.

use crate::cronus_ui_kit::{attr, esc, instance_id, label_of};
use crate::parser::{ComponentItemNode, ComponentNode};

/// Most views one island can hold (the stylesheet indexes 8 states).
pub const MAX_VIEWS: usize = 8;

struct View {
    content: String,
    label: String,
}

fn view_of(i: &ComponentItemNode) -> View {
    let content = esc(&i.text);
    View {
        label: i
            .config
            .get("aria-label")
            .filter(|l| !l.is_empty())
            .map(|l| esc(l))
            .unwrap_or_else(|| content.clone()),
        content,
    }
}

pub fn render(comp: &ComponentNode) -> String {
    let mut views: Vec<View> = comp
        .items
        .iter()
        .filter(|i| i.item_type != "label" && i.item_type != "title" && !i.text.is_empty())
        .map(view_of)
        .take(MAX_VIEWS)
        .collect();
    if views.is_empty() {
        let content = label_of(comp);
        views.push(View {
            label: content.clone(),
            content,
        });
    }
    let selected = attr(comp, "value")
        .map(esc)
        .and_then(|v| views.iter().position(|w| w.content == v || w.label == v))
        .unwrap_or(0);
    let name = instance_id(comp, "dynamic-island");
    let panels = views
        .iter()
        .map(|v| format!("<div><div>{}</div></div>", v.content))
        .collect::<String>();
    let triggers = views
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let (aria_selected, checked) = if i == selected {
                ("true", " checked")
            } else {
                ("false", "")
            };
            format!(
                "<label><input type=\"radio\" name=\"{name}\" value=\"{i}\" aria-label=\"{label}\"{checked}><button type=\"button\" role=\"tab\" aria-selected=\"{aria_selected}\" aria-label=\"{label}\" data-slot=\"dynamic-island-trigger\" tabindex=\"-1\" aria-hidden=\"true\"></button></label>",
                label = v.label
            )
        })
        .collect::<String>();
    format!(
        "<div data-slot=\"dynamic-island\"><div data-slot=\"dynamic-island-shell\">{panels}</div><div role=\"radiogroup\" aria-label=\"{}\">{triggers}</div></div>",
        views[selected].label
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;

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
        assert!(!html.contains(" disabled"));
        assert!(!html.contains("zinc-"));
    }

    /// wave1t: fixture emits `label "default"` + `text "Idle"` + `text "Active"`;
    /// the label is not a view. Every view stays in the shell; the checked radio picks one.
    #[test]
    fn views_from_texts_not_label() {
        crate::cronus_ui_kit::reset_instance_ids();
        let mut c = stub("dynamic-island", "default");
        c.items.push(extra("text", "Idle"));
        c.items.push(extra("text", "Active"));
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"dynamic-island\"><div data-slot=\"dynamic-island-shell\"><div><div>Idle</div></div><div><div>Active</div></div></div><div role=\"radiogroup\" aria-label=\"Idle\"><label><input type=\"radio\" name=\"cui-dynamic-island-dynamic-island\" value=\"0\" aria-label=\"Idle\" checked><button type=\"button\" role=\"tab\" aria-selected=\"true\" aria-label=\"Idle\" data-slot=\"dynamic-island-trigger\" tabindex=\"-1\" aria-hidden=\"true\"></button></label><label><input type=\"radio\" name=\"cui-dynamic-island-dynamic-island\" value=\"1\" aria-label=\"Active\"><button type=\"button\" role=\"tab\" aria-selected=\"false\" aria-label=\"Active\" data-slot=\"dynamic-island-trigger\" tabindex=\"-1\" aria-hidden=\"true\"></button></label></div></div>"
        );
        assert!(!html.contains("default"));
        reject_fx(&html);
    }

    /// Docs example: views with their own accessible names; `value:` picks the view.
    #[test]
    fn item_aria_label_names_the_dot_and_value_selects() {
        let mut c = stub("dynamic-island", "Live");
        c.items.clear();
        let mut idle = extra("item", "Cronus");
        idle.config.insert("aria-label".into(), "Idle".into());
        let mut now = extra("item", "Shipping the surface");
        now.config.insert("aria-label".into(), "Now playing".into());
        c.items.push(idle);
        c.items.push(now);
        c.props.insert("value".into(), "Now playing".into());
        let html = render(&c);
        assert!(html.contains("<div data-slot=\"dynamic-island-shell\"><div><div>Cronus</div></div><div><div>Shipping the surface</div></div></div>"));
        assert!(html.contains("<div role=\"radiogroup\" aria-label=\"Now playing\">"));
        assert!(html.contains("value=\"1\" aria-label=\"Now playing\" checked>"));
        assert!(html.contains("aria-selected=\"true\" aria-label=\"Now playing\""));
        assert!(html.contains("aria-selected=\"false\" aria-label=\"Idle\""));
        reject_fx(&html);
    }

    #[test]
    fn label_only_still_emits_one_view() {
        let html = render(&stub("dynamic-island", "Idle"));
        assert_eq!(
            html.matches("data-slot=\"dynamic-island-trigger\"").count(),
            1
        );
        assert!(html.contains("<div><div>Idle</div></div>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("dynamic-island", "A <B> & \"C\""));
        assert!(html.contains("<div><div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"));
        assert!(html.contains("aria-label=\"A &lt;B&gt; &amp; &quot;C&quot;\""));
        reject_fx(&html);
    }

    #[test]
    fn two_islands_on_one_page_get_distinct_names() {
        crate::cronus_ui_kit::reset_instance_ids();
        let a = render(&stub("dynamic-island", "Idle"));
        let b = render(&stub("dynamic-island", "Idle"));
        assert!(a.contains("name=\"cui-dynamic-island-dynamic-island\" "));
        assert!(b.contains("name=\"cui-dynamic-island-dynamic-island-2\" "));
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("dynamic-island", "Idle");
        let html = render(&c);
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
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
            let html = render(&stub("dynamic-island", "Idle"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"dynamic-island\""));
        });
    }

    #[test]
    fn chrome_matches_react_geometry_and_morph() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"dynamic-island\"] {\n  display: inline-flex; flex-direction: column; align-items: center; gap: 0.75rem;\n}"));
        assert!(css.contains("[data-slot=\"dynamic-island-shell\"] {\n  display: flex; overflow: hidden; border-radius: 9999px;"));
        assert!(css.contains("padding: 0.5rem 1rem;"));
        assert!(css.contains("box-shadow: var(--cronus-shadow-md, none);"));
        // Morph: every view is a 0fr cell; the checked one opens to 1fr with the spring
        // while its content fades and un-blurs in over 200ms.
        assert!(css.contains("[data-slot=\"dynamic-island-shell\"] > div {\n  display: grid; grid-template-columns: 0fr; opacity: 0; filter: blur(6px);\n  transition: grid-template-columns var(--cronus-spring-snappy), opacity 0s, filter 0s;\n}"));
        assert!(css.contains("[data-slot=\"dynamic-island-shell\"] > div > div {\n  display: flex; align-items: center; gap: 0.75rem; min-width: 0; overflow: hidden; white-space: nowrap;"));
        for k in 0..MAX_VIEWS {
            assert!(css.contains(&format!("[data-slot=\"dynamic-island\"]:has(> [role=\"radiogroup\"] > label:nth-child({}) > input:checked) > [data-slot=\"dynamic-island-shell\"] > div:nth-child({}) {{\n  grid-template-columns: 1fr; opacity: 1; filter: blur(0);\n  transition: grid-template-columns var(--cronus-spring-snappy), opacity 200ms var(--cronus-ease), filter 200ms var(--cronus-ease);\n}}", k + 1, k + 1)), "{k}");
        }
        assert!(css.contains("[data-slot=\"dynamic-island-trigger\"] {\n  width: 0.5rem; height: 0.5rem; padding: 0; border: 0;"));
        assert!(css.contains("[data-slot=\"dynamic-island\"] > [role=\"radiogroup\"] > label > input:checked + [data-slot=\"dynamic-island-trigger\"] {\n  background: var(--cronus-fg);\n}"));
        assert!(css.contains("[data-slot=\"dynamic-island\"] > [role=\"radiogroup\"] > label:hover > input:not(:checked) + [data-slot=\"dynamic-island-trigger\"] { background: var(--cronus-border-strong); }"));
        assert!(!css.contains("[data-slot=\"dynamic-island-trigger\"][aria-selected=\"true\"]"));
        assert!(!css.contains("zinc-"));
    }
}
