//! Dedicated Shimmer renderer. DOM mirrors React:
//! `<div data-slot="shimmer" aria-hidden="true">` + an empty sweep `<div>`
//! (React: `absolute inset-0 animate-shimmer` gradient, `shimmer 2s linear
//! infinite` from rest to `translateX(100%)`). Fixture default `h-8 w-48`,
//! `rounded-md bg-surface-overlay` and the sweep live in the family CSS.
//!
//! Sizing follows the docs' Tailwind classes: `h:4|8|10|12`, `w:32|48|64|
//! full|3/4|1/2`, `radius:md|lg|xl|full` as props on one shimmer, or one
//! `item` line per block (`item h:4 w:full radius:md`) for the docs' stack
//! (`flex flex-col gap-3`). Not the catalog `fx()` title SURF box.

use crate::parser::{ComponentItemNode, ComponentNode};
use std::collections::HashMap;

fn class_of(get: impl Fn(&str) -> Option<String>) -> String {
    let mut classes = Vec::new();
    if let Some(h) = get("h").filter(|h| matches!(h.as_str(), "4" | "6" | "8" | "10" | "12")) {
        classes.push(format!("h-{h}"));
    }
    if let Some(w) = get("w") {
        match w.as_str() {
            "32" | "48" | "64" | "full" => classes.push(format!("w-{w}")),
            "3/4" => classes.push("w-3-4".into()),
            "1/2" => classes.push("w-1-2".into()),
            _ => {}
        }
    }
    if let Some(r) = get("radius").filter(|r| matches!(r.as_str(), "lg" | "xl" | "full")) {
        classes.push(format!("r-{r}"));
    }
    if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    }
}

fn block(class: &str) -> String {
    format!("<div data-slot=\"shimmer\"{class} aria-hidden=\"true\"><div></div></div>")
}

pub fn render(comp: &ComponentNode) -> String {
    let items: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item")
        .collect();
    if items.is_empty() {
        let props: &HashMap<String, String> = &comp.props;
        return block(&class_of(|k| props.get(k).cloned()));
    }
    let blocks: String = items
        .iter()
        .map(|i| block(&class_of(|k| i.config.get(k).cloned())))
        .collect();
    format!("<div class=\"cui-shimmer-stack\">{blocks}</div>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";
    const CSS: &str = include_str!("cronus_ui_css/shimmer.css");

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<span"));
        assert!(!html.contains("Demo"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_is_block_with_sweep_layer_not_fx_title_box() {
        let html = render(&stub("shimmer", "Demo"));
        assert_eq!(
            html,
            "<div data-slot=\"shimmer\" aria-hidden=\"true\"><div></div></div>"
        );
        reject_fx(&html);
    }

    /// Docs "Loading sheen": four sized blocks in a `gap-3` column.
    #[test]
    fn item_lines_stack_sized_blocks() {
        let mut c = stub("shimmer", "Demo");
        c.items.clear();
        for cfg in [
            &[("h", "8"), ("w", "48"), ("radius", "lg")][..],
            &[("h", "4"), ("w", "full"), ("radius", "md")],
            &[("h", "4"), ("w", "3/4"), ("radius", "md")],
            &[("h", "10"), ("w", "32"), ("radius", "lg")],
        ] {
            c.items.push(ComponentItemNode {
                item_type: "item".into(),
                text: String::new(),
                link: None,
                tone: None,
                config: cfg
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            });
        }
        let html = render(&c);
        assert_eq!(
            html,
            "<div class=\"cui-shimmer-stack\"><div data-slot=\"shimmer\" class=\"h-8 w-48 r-lg\" aria-hidden=\"true\"><div></div></div><div data-slot=\"shimmer\" class=\"h-4 w-full\" aria-hidden=\"true\"><div></div></div><div data-slot=\"shimmer\" class=\"h-4 w-3-4\" aria-hidden=\"true\"><div></div></div><div data-slot=\"shimmer\" class=\"h-10 w-32 r-lg\" aria-hidden=\"true\"><div></div></div></div>"
        );
        reject_fx(&html);
    }

    #[test]
    fn size_props_on_a_single_block() {
        let mut c = stub("shimmer", "Demo");
        c.props.insert("h".into(), "10".into());
        c.props.insert("w".into(), "1/2".into());
        c.props.insert("radius".into(), "full".into());
        assert_eq!(
            render(&c),
            "<div data-slot=\"shimmer\" class=\"h-10 w-1-2 r-full\" aria-hidden=\"true\"><div></div></div>"
        );
        c.props.insert("h".into(), "999".into());
        assert!(render(&c).contains("class=\"w-1-2 r-full\""));
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("shimmer", "Demo"));
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("shimmer", "Demo"));
            reject_fx(&html);
        });
    }

    /// React `animate-shimmer`: `shimmer 2s linear infinite`, a single
    /// `100% { translateX(100%) }` keyframe from the layer's rest position.
    #[test]
    fn chrome_shimmer_via_css() {
        assert!(CSS.contains("[data-slot=\"shimmer\"] {\n  display: block; position: relative; overflow: hidden;\n  height: 2rem; width: var(--cui-shimmer-w, 100%);\n  border-radius: var(--cronus-radius-md);"));
        assert!(CSS.contains("[data-slot=\"shimmer\"] > div {\n  position: absolute; inset: 0;"));
        assert!(!CSS.contains("transform: translateX(-100%);"));
        assert!(CSS.contains("var(--cronus-surface-overlay)"));
        assert!(CSS.contains("@keyframes cui-shimmer { 100% { transform: translateX(100%); } }"));
        assert!(CSS.contains("animation: cui-shimmer 2s linear infinite;"));
        assert!(CSS.contains("[data-slot=\"shimmer\"].h-4 { height: 1rem; }"));
        assert!(CSS.contains("[data-slot=\"shimmer\"].w-3-4 { width: 75%; }"));
        assert!(CSS
            .contains("[data-slot=\"shimmer\"].r-lg { border-radius: var(--cronus-radius-lg); }"));
        assert!(CSS.contains(".cui-shimmer-stack { display: flex; flex-direction: column; gap: 0.75rem; width: 100%; }"));
        assert!(CSS.contains("prefers-reduced-motion"));
        assert!(!CSS.contains("zinc-"));
        assert!(!CSS.contains(FX_BOX));
    }
}
