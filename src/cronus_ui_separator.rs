//! Dedicated Separator renderer. DOM matches React/Radix Root (`div`).
//! Decorative by default (no role). Not the interact `<hr data-slot="separator">`.

use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let orientation = orientation_of(comp);
    let mut attrs = format!("data-slot=\"separator\" data-orientation=\"{orientation}\"");
    if !decorative(comp) {
        attrs.push_str(&format!(
            " role=\"separator\" aria-orientation=\"{orientation}\""
        ));
    }
    format!("<div {attrs}></div>")
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
    if let Some(v) = comp.props.get("decorative") {
        return v != "false";
    }
    if let Some(v) = comp
        .items
        .iter()
        .find_map(|i| i.config.get("decorative"))
    {
        return v != "false";
    }
    true
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

    #[test]
    fn chrome_horizontal_and_vertical() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"separator\"]"));
        assert!(css.contains("background: var(--cronus-border)"));
        assert!(css.contains("height: 1px; width: 100%"));
        assert!(css.contains("height: 100%; width: 1px"));
        assert!(!css.contains("zinc-"));
    }
}
