//! Dedicated Separator renderer. DOM matches React/Radix Root (`div`).
//! Decorative by default (no role). With `text` items the renderer emits the
//! docs' surrounding copy: a `flex-col gap-3` column (horizontal) or a
//! `flex h-10 items-center gap-3` row (vertical) of `text-sm text-fg-secondary`
//! spans with one separator between neighbours. Not the interact
//! `<hr data-slot="separator">`.

use crate::cronus_ui_kit::{attr, esc};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let orientation = orientation_of(comp);
    let mut attrs = format!("data-slot=\"separator\" data-orientation=\"{orientation}\"");
    if !decorative(comp) {
        attrs.push_str(&format!(
            " role=\"separator\" aria-orientation=\"{orientation}\""
        ));
    }
    let separator = format!("<div {attrs}></div>");
    let texts: Vec<String> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "text" | "item") && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if texts.is_empty() {
        return separator;
    }
    let class = if orientation == "vertical" {
        "cui-separator-row"
    } else {
        "cui-separator-stack"
    };
    let inner = texts
        .iter()
        .map(|t| format!("<span>{t}</span>"))
        .collect::<Vec<_>>()
        .join(&separator);
    format!("<div class=\"{class}\">{inner}</div>")
}

fn orientation_of(comp: &ComponentNode) -> &'static str {
    if let Some(v) = comp.props.get("orientation") {
        if v == "vertical" {
            return "vertical";
        }
        if v == "horizontal" {
            return "horizontal";
        }
    }
    if comp.items.iter().any(|i| {
        i.config
            .get("orientation")
            .map(|s| s == "vertical")
            .unwrap_or(false)
    }) {
        return "vertical";
    }
    let style = comp.style.as_deref().unwrap_or("");
    for part in style.split('+') {
        if part.trim() == "vertical" {
            return "vertical";
        }
    }
    "horizontal"
}

fn decorative(comp: &ComponentNode) -> bool {
    attr(comp, "decorative").is_none_or(|v| v != "false")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn stub(style: &str) -> ComponentNode {
        ComponentNode {
            name: "Separator".into(),
            layout: Some("inline".into()),
            style: Some(style.into()),
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

    fn text(t: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "text".into(),
            text: t.into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        }
    }

    #[test]
    fn root_is_div_not_hr() {
        let html = render(&stub("separator"));
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"separator\""));
        assert!(html.contains("data-orientation=\"horizontal\""));
        assert!(!html.contains("role="));
        assert!(!html.contains("<hr"));
        assert!(!html.contains("border-top"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn vertical_from_style() {
        let html = render(&stub("separator+vertical"));
        assert!(html.contains("data-orientation=\"vertical\""));
        assert!(!html.contains("data-orientation=\"horizontal\""));
    }

    #[test]
    fn vertical_from_props() {
        let mut c = stub("separator");
        c.props.insert("orientation".into(), "vertical".into());
        let html = render(&c);
        assert!(html.contains("data-orientation=\"vertical\""));
    }

    #[test]
    fn non_decorative_sets_role() {
        let mut c = stub("separator");
        c.props.insert("decorative".into(), "false".into());
        let html = render(&c);
        assert!(html.contains("role=\"separator\""));
        assert!(html.contains("aria-orientation=\"horizontal\""));
    }

    /// Docs "Horizontal & vertical": texts around the divider render the
    /// example's column (horizontal) and `h-10` row (vertical) of spans.
    #[test]
    fn texts_surround_the_separator_in_docs_layout() {
        let mut c = stub("separator");
        c.items.push(text("Section A"));
        c.items.push(text("Section B"));
        assert_eq!(
            render(&c),
            "<div class=\"cui-separator-stack\"><span>Section A</span><div data-slot=\"separator\" data-orientation=\"horizontal\"></div><span>Section B</span></div>"
        );
        let mut v = stub("separator+vertical");
        for t in ["Docs", "API", "Blog"] {
            v.items.push(text(t));
        }
        let html = render(&v);
        assert!(html.starts_with("<div class=\"cui-separator-row\"><span>Docs</span><div data-slot=\"separator\" data-orientation=\"vertical\"></div><span>API</span>"));
        assert_eq!(html.matches("data-slot=\"separator\"").count(), 2);
        let css = include_str!("cronus_ui_css/separator.css");
        assert!(css.contains(".cui-separator-row { display: flex; height: 2.5rem; align-items: center; gap: 0.75rem;"));
        assert!(css.contains(
            ".cui-separator-stack { display: flex; flex-direction: column; gap: 0.75rem;"
        ));
    }

    #[test]
    fn chrome_horizontal_and_vertical() {
        let css = include_str!("cronus_ui_css/separator.css");
        assert!(css.contains("[data-slot=\"separator\"]"));
        assert!(css.contains("background: var(--cronus-border)"));
        assert!(css.contains("height: 1px; width: 100%"));
        assert!(css.contains("height: 100%; width: 1px"));
        assert!(!css.contains("zinc-"));
    }
}
