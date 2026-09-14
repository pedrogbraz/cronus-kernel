//! Dedicated Avatar renderer. DOM matches React/Radix Root (`span`):
//! `<span data-slot="avatar"><span data-slot="avatar-fallback">XX</span></span>`.
//! Optional `<img data-slot="avatar-image" src>` when a url item/link exists.
//! Not interact `avatar()` (`<div data-slot="avatar" style="width:2.25rem;…">` single letter).

use crate::cronus_ui_kit::{esc, safe_url};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let fallback = esc(&initials(&label));
    let mut inner = String::new();
    if let Some(src) = image_src(comp) {
        inner.push_str(&format!(
            "<img data-slot=\"avatar-image\" src=\"{}\" alt=\"{}\">",
            safe_url(src),
            esc(&label)
        ));
    }
    inner.push_str(&format!(
        "<span data-slot=\"avatar-fallback\">{fallback}</span>"
    ));
    format!("<span data-slot=\"avatar\">{inner}</span>")
}

fn label_of(comp: &ComponentNode) -> String {
    for kind in ["label", "title", "text", "value"] {
        if let Some(t) = item(comp, kind) {
            if !t.is_empty() && !looks_like_url(t) {
                return t.to_string();
            }
        }
    }
    for i in &comp.items {
        if !i.text.is_empty() && !looks_like_url(&i.text) {
            return i.text.clone();
        }
    }
    if !comp.name.is_empty() && !looks_like_url(&comp.name) {
        return comp.name.clone();
    }
    String::new()
}

fn initials(label: &str) -> String {
    let words: Vec<&str> = label.split_whitespace().filter(|w| !w.is_empty()).collect();
    let mut out = String::new();
    if words.len() >= 2 {
        for w in words.iter().take(2) {
            if let Some(ch) = w.chars().next() {
                out.extend(ch.to_uppercase().take(1));
            }
        }
    } else if let Some(w) = words.first() {
        for ch in w.chars().take(2) {
            out.extend(ch.to_uppercase());
        }
        if out.chars().count() > 2 {
            out = out.chars().take(2).collect();
        }
    }
    if out.is_empty() {
        "A".into()
    } else {
        out
    }
}

fn image_src(comp: &ComponentNode) -> Option<&str> {
    for key in ["src", "image", "url"] {
        if let Some(v) = comp.props.get(key) {
            if !v.is_empty() {
                return Some(v.as_str());
            }
        }
    }
    for i in &comp.items {
        if let Some(link) = i.link.as_deref() {
            if !link.is_empty() {
                return Some(link);
            }
        }
        if matches!(i.item_type.as_str(), "source" | "url" | "image") && !i.text.is_empty() {
            return Some(i.text.as_str());
        }
        for key in ["src", "image", "url"] {
            if let Some(v) = i.config.get(key) {
                if !v.is_empty() {
                    return Some(v.as_str());
                }
            }
        }
        if looks_like_url(&i.text) {
            return Some(i.text.as_str());
        }
    }
    None
}

fn looks_like_url(s: &str) -> bool {
    let s = s.trim();
    s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("data:")
        || s.starts_with('/')
        || s.starts_with("./")
}

fn item<'a>(comp: &'a ComponentNode, kind: &str) -> Option<&'a str> {
    comp.items
        .iter()
        .find(|i| i.item_type == kind)
        .map(|i| i.text.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn stub(label: &str) -> ComponentNode {
        ComponentNode {
            name: "Avatar".into(),
            layout: Some("inline".into()),
            style: Some("avatar".into()),
            items: vec![ComponentItemNode {
                item_type: "label".into(),
                text: label.into(),
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
    fn root_is_span_with_fallback_not_div() {
        let html = render(&stub("Demo"));
        assert!(html.starts_with("<span "));
        assert!(html.contains("data-slot=\"avatar\""));
        assert!(html.contains("data-slot=\"avatar-fallback\""));
        assert!(html.contains(">DE</span>"));
        assert!(!html.contains("<div"));
        assert!(!html.contains("style="));
        assert!(!html.contains("width:2.25rem"));
        assert_eq!(
            html,
            "<span data-slot=\"avatar\"><span data-slot=\"avatar-fallback\">DE</span></span>"
        );
    }

    #[test]
    fn initials_from_two_words() {
        let html = render(&stub("Jane Doe"));
        assert!(html.contains(">JD</span>"));
        assert!(!html.contains(">J</span>"));
    }

    #[test]
    fn image_from_item_link() {
        let mut c = stub("Jane Doe");
        c.items[0].link = Some("https://cdn.example/j.png".into());
        let html = render(&c);
        assert!(html.contains("<img data-slot=\"avatar-image\""));
        assert!(html.contains("src=\"https://cdn.example/j.png\""));
        assert!(html.contains("alt=\"Jane Doe\""));
        assert!(html.contains("data-slot=\"avatar-fallback\""));
        assert!(html.contains(">JD</span>"));
        assert!(!html.contains("<div"));
        assert!(!html.contains("style="));
    }

    #[test]
    fn image_from_source_item() {
        let mut c = stub("Ada Lovelace");
        c.items.push(ComponentItemNode {
            item_type: "source".into(),
            text: "/ada.jpg".into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        });
        let html = render(&c);
        assert!(html.contains("<img data-slot=\"avatar-image\" src=\"/ada.jpg\""));
        assert!(html.contains(">AL</span>"));
    }

    #[test]
    fn skips_interact_div() {
        let html = render(&stub("Demo"));
        let interact = crate::cronus_ui_interact::render("avatar", &stub("Demo")).unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<div data-slot=\"avatar\""));
        assert!(interact.contains("width:2.25rem"));
        assert!(!interact.contains("data-slot=\"avatar-fallback\""));
        assert!(!html.contains("v-data="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"avatar\"]"));
        assert!(css.contains("[data-slot=\"avatar-fallback\"]"));
        assert!(css.contains("position: relative"));
        assert!(css.contains("width: 2.5rem"));
        assert!(css.contains("height: 2.5rem"));
        assert!(css.contains("overflow: hidden"));
        assert!(css.contains("border-radius: 9999px"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(css.contains("font-size: 0.875rem"));
        assert!(!css.contains("zinc-"));
    }

    #[test]
    fn fallback_line_height_is_text_sm() {
        // Wave 1t: React avatar-fallback lineHeight 20px (text-sm), was 21px.
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("background: var(--cronus-surface-overlay); color: var(--cronus-fg-secondary);\n  font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;"));
        assert_eq!(
            render(&stub("AL")),
            "<span data-slot=\"avatar\"><span data-slot=\"avatar-fallback\">AL</span></span>"
        );
    }
}
