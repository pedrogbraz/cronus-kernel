//! Dedicated Skeleton renderer. DOM matches React:
//! `<div data-slot="skeleton" aria-hidden="true">` (empty).
//!
//! `item` lines compose the docs' loading card: each item is one skeleton
//! whose config maps the example's utilities to classes — `shape:circle`
//! (`rounded-full`), `size:12` (`size-12`), `height:3` (`h-3`),
//! `width:3/4` (`w-3/4`, class `w-3-4`), `radius:md`. Circles sit on the
//! `flex items-center gap-3` row; the lines that follow share the
//! `flex-1 flex-col gap-2` column. Neither wrapper has a slot in React.
//! Not the interact inline `height:0.9rem;width:8rem;…background:var(--cronus-border)`.

use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let shapes: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item")
        .collect();
    if shapes.is_empty() {
        return skeleton("");
    }
    if shapes.len() == 1 {
        return skeleton(&classes(shapes[0]));
    }
    let mut out = String::from("<div class=\"cui-skeleton-row\">");
    let mut lines = String::new();
    for s in &shapes {
        let class = classes(s);
        if s.config.get("shape").is_some_and(|v| v == "circle") {
            out.push_str(&skeleton(&class));
        } else {
            lines.push_str(&skeleton(&class));
        }
    }
    if !lines.is_empty() {
        out.push_str(&format!("<div class=\"cui-skeleton-lines\">{lines}</div>"));
    }
    out.push_str("</div>");
    out
}

fn skeleton(class: &str) -> String {
    if class.is_empty() {
        "<div data-slot=\"skeleton\" aria-hidden=\"true\"></div>".into()
    } else {
        format!("<div data-slot=\"skeleton\" aria-hidden=\"true\" class=\"{class}\"></div>")
    }
}

/// Utility classes for one shape; unknown values are dropped.
fn classes(item: &ComponentItemNode) -> String {
    let mut out: Vec<String> = Vec::new();
    if item.config.get("shape").is_some_and(|v| v == "circle") {
        out.push("rounded-full".into());
    }
    if let Some(r) = item.config.get("radius") {
        if matches!(r.as_str(), "sm" | "md" | "lg" | "xl") {
            out.push(format!("rounded-{r}"));
        }
    }
    for (key, prefix) in [("size", "size-"), ("height", "h-"), ("width", "w-")] {
        if let Some(v) = item.config.get(key) {
            let v = v.trim();
            let ok = v == "full"
                || v.split('/')
                    .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()));
            if ok {
                out.push(format!("{prefix}{}", v.replace('/', "-")));
            }
        }
    }
    out.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn stub() -> ComponentNode {
        ComponentNode {
            name: "Skeleton".into(),
            layout: Some("inline".into()),
            style: Some("skeleton".into()),
            items: vec![ComponentItemNode {
                item_type: "label".into(),
                text: "Demo".into(),
                link: None,
                tone: None,
                config: HashMap::new(),
            }],
            props: HashMap::new(),
            params: vec![],
            template: None,
            sections: vec![],
            state: vec![],
            tests: vec![],
            binding: None,
            span: Default::default(),
        }
    }

    fn shape(pairs: &[(&str, &str)]) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "item".into(),
            text: "shape".into(),
            link: None,
            tone: None,
            config: pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    #[test]
    fn root_is_empty_div_not_inline_size() {
        let html = render(&stub());
        assert_eq!(
            html,
            "<div data-slot=\"skeleton\" aria-hidden=\"true\"></div>"
        );
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"skeleton\""));
        assert!(html.contains("aria-hidden=\"true\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("height:0.9rem"));
        assert!(!html.contains("width:8rem"));
        assert!(!html.contains("background:var(--cronus-border)"));
        assert!(!html.contains("Demo"));
        assert!(!html.contains("v-data="));
    }

    /// Docs "Loading card": a `size-12 rounded-full` circle beside a column of
    /// two `h-3` lines (`w-3/4`, `w-1/2`) — utilities become classes, no
    /// inline sizes.
    #[test]
    fn items_compose_the_loading_card() {
        let mut c = stub();
        c.items.push(shape(&[("shape", "circle"), ("size", "12")]));
        c.items.push(shape(&[
            ("height", "3"),
            ("width", "3/4"),
            ("radius", "md"),
        ]));
        c.items.push(shape(&[("height", "3"), ("width", "1/2")]));
        assert_eq!(
            render(&c),
            "<div class=\"cui-skeleton-row\"><div data-slot=\"skeleton\" aria-hidden=\"true\" class=\"rounded-full size-12\"></div><div class=\"cui-skeleton-lines\"><div data-slot=\"skeleton\" aria-hidden=\"true\" class=\"rounded-md h-3 w-3-4\"></div><div data-slot=\"skeleton\" aria-hidden=\"true\" class=\"h-3 w-1-2\"></div></div></div>"
        );
        let mut one = stub();
        one.items
            .push(shape(&[("height", "4"), ("width", "bogus")]));
        assert_eq!(
            render(&one),
            "<div data-slot=\"skeleton\" aria-hidden=\"true\" class=\"h-4\"></div>"
        );
        let css = include_str!("cronus_ui_css/skeleton.css");
        assert!(css.contains("[data-slot=\"skeleton\"].size-12 { width: 3rem; height: 3rem; }"));
        assert!(css.contains("[data-slot=\"skeleton\"].w-3-4 { width: 75%; }"));
        assert!(css.contains(
            ".cui-skeleton-row { display: flex; align-items: center; gap: 0.75rem; width: 100%; }"
        ));
        assert!(css.contains(
            ".cui-skeleton-lines { display: flex; flex: 1; flex-direction: column; gap: 0.5rem; }"
        ));
    }

    #[test]
    fn skips_interact_inline_dump() {
        let html = render(&stub());
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_default_size_via_css() {
        let css = include_str!("cronus_ui_css/skeleton.css");
        assert!(css.contains("[data-slot=\"skeleton\"]"));
        assert!(css.contains("width: 8rem"));
        // Wave 1t: harness default className `h-4 w-32` → 16px, not 14.4px.
        assert!(css.contains("display: block; height: 1rem; width: var(--cui-skeleton-w, 100%);"));
        assert!(css.contains("border-radius: var(--cronus-radius-md)"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(css.contains("@keyframes cui-pulse"));
        assert!(css.contains("animation: cui-pulse"));
        assert!(!css.contains("zinc-"));
    }
}
