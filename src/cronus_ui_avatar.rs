//! Dedicated Avatar renderer. DOM matches React/Radix Root (`span`):
//! `<span data-slot="avatar"><span data-slot="avatar-fallback">XX</span></span>`,
//! with `<img data-slot="avatar-image" src alt>` first when `src:` (or a url
//! item/link) exists. `alt:` names the image (default: the label);
//! `fallback:` overrides the initials computed from the label.
//!
//! Two or more `item` lines render the docs' overlapping stack: one avatar per
//! item inside the example's plain `<div>` (`flex -space-x-3`, no slot).
//! `ring:true` adds React's `ring-2 ring-surface-raised` as class `ring`; an
//! item's `size:xs` puts `text-xs` on its fallback (class `s-xs`).
//! Not interact `avatar()` (`<div data-slot="avatar" style="width:2.25rem;…">` single letter).

use crate::cronus_ui_kit::{attr_nonempty, esc, flag, safe_url};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let members: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty())
        .collect();
    if members.len() >= 2 {
        let ring = if flag(comp, "ring") { "ring" } else { "" };
        let inner: String = members
            .iter()
            .map(|m| {
                let fallback = m
                    .config
                    .get("fallback")
                    .filter(|f| !f.is_empty())
                    .map(|f| esc(f))
                    .unwrap_or_else(|| esc(&initials(&m.text)));
                let src = m.config.get("src").filter(|s| !s.is_empty());
                let alt = m
                    .config
                    .get("alt")
                    .map(String::as_str)
                    .unwrap_or(m.text.as_str());
                let size = match m.config.get("size").map(String::as_str) {
                    Some("xs") => "s-xs",
                    _ => "",
                };
                avatar_html(&fallback, src.map(String::as_str), &esc(alt), ring, size)
            })
            .collect();
        return format!("<div class=\"cui-avatar-stack\">{inner}</div>");
    }
    let label = label_of(comp);
    let fallback = attr_nonempty(comp, "fallback")
        .map(esc)
        .unwrap_or_else(|| esc(&initials(&label)));
    let alt = attr_nonempty(comp, "alt").unwrap_or(&label);
    let ring = if flag(comp, "ring") { "ring" } else { "" };
    avatar_html(&fallback, image_src(comp), &esc(alt), ring, "")
}

/// One React `Avatar`: optional image, then the fallback. `fallback` and `alt`
/// must already be escaped; `class` / `fallback_class` are extra classes (or
/// empty). Shared with status-dot (avatar overlay) and avatar-group.
pub fn avatar_html(
    fallback: &str,
    src: Option<&str>,
    alt: &str,
    class: &str,
    fallback_class: &str,
) -> String {
    let mut inner = String::new();
    if let Some(src) = src {
        inner.push_str(&format!(
            "<img data-slot=\"avatar-image\" src=\"{}\" alt=\"{alt}\">",
            safe_url(src)
        ));
    }
    let fb_class = if fallback_class.is_empty() {
        String::new()
    } else {
        format!(" class=\"{fallback_class}\"")
    };
    inner.push_str(&format!(
        "<span data-slot=\"avatar-fallback\"{fb_class}>{fallback}</span>"
    ));
    let class = if class.is_empty() {
        String::new()
    } else {
        format!(" class=\"{class}\"")
    };
    format!("<span data-slot=\"avatar\"{class}>{inner}</span>")
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

/// Initials of a name: first letter of the first two words, or the first two
/// characters of a single word (already-short fallbacks like `CN` pass through).
pub fn initials(label: &str) -> String {
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
            span: Default::default(),
        }
    }

    fn member(text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "item".into(),
            text: text.into(),
            link: None,
            tone: None,
            config: HashMap::new(),
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

    /// Docs "Image & fallback": `src:` + `alt:"@shadcn"` name the image and the
    /// two-letter label passes through as the fallback.
    #[test]
    fn src_and_alt_props_render_react_image_then_fallback() {
        let mut c = stub("CN");
        c.props
            .insert("src".into(), "https://github.com/shadcn.png".into());
        c.props.insert("alt".into(), "@shadcn".into());
        assert_eq!(
            render(&c),
            "<span data-slot=\"avatar\"><img data-slot=\"avatar-image\" src=\"https://github.com/shadcn.png\" alt=\"@shadcn\"><span data-slot=\"avatar-fallback\">CN</span></span>"
        );
        c.props.insert("fallback".into(), "SC".into());
        assert!(render(&c).contains("<span data-slot=\"avatar-fallback\">SC</span>"));
    }

    /// Docs "Group": items become the `flex -space-x-3` stack, `ring:true`
    /// puts React's `ring-2 ring-surface-raised` class on every avatar and the
    /// `+5` item's `size:xs` lands on its fallback (`text-xs`).
    #[test]
    fn items_render_overlapping_stack_with_ring_and_xs_fallback() {
        let mut c = stub("Team");
        c.items.clear();
        for n in ["CN", "AL", "JL", "MK"] {
            c.items.push(member(n));
        }
        let mut more = member("+5");
        more.config.insert("size".into(), "xs".into());
        c.items.push(more);
        c.props.insert("ring".into(), "true".into());
        let html = render(&c);
        assert!(html.starts_with("<div class=\"cui-avatar-stack\"><span data-slot=\"avatar\" class=\"ring\"><span data-slot=\"avatar-fallback\">CN</span></span>"));
        assert_eq!(html.matches("data-slot=\"avatar\"").count(), 5);
        assert!(html.ends_with("<span data-slot=\"avatar\" class=\"ring\"><span data-slot=\"avatar-fallback\" class=\"s-xs\">+5</span></span></div>"));
        assert!(!html.contains("style="));
        let css = include_str!("cronus_ui_css/avatar.css");
        assert!(css.contains(".cui-avatar-stack > * + * { margin-inline-start: -0.75rem; }"));
        assert!(css.contains(
            "[data-slot=\"avatar\"].ring { box-shadow: 0 0 0 2px var(--cronus-surface-raised); }"
        ));
        assert!(css.contains(
            "[data-slot=\"avatar-fallback\"].s-xs { font-size: 0.75rem; line-height: 1rem; }"
        ));
    }

    #[test]
    fn skips_interact_div() {
        let html = render(&stub("Demo"));
        assert!(!html.contains("v-data="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/avatar.css");
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
        let css = include_str!("cronus_ui_css/avatar.css");
        assert!(css.contains("background: var(--cronus-surface-overlay); color: var(--cronus-fg-secondary);\n  font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;"));
        assert_eq!(
            render(&stub("AL")),
            "<span data-slot=\"avatar\"><span data-slot=\"avatar-fallback\">AL</span></span>"
        );
    }
}
