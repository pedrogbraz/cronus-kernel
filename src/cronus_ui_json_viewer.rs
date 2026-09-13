//! Dedicated JsonViewer renderer. DOM matches React slots:
//! `<div data-slot="json-viewer">` plus each text as
//! `<div data-slot="json-viewer-row">` with `json-viewer-key` /
//! `json-viewer-value`. Static tree from texts (`key: value` lines OK).
//! Not interact `codey()` (`<pre style=SURF>`) or catalog `display()` SURF
//! `<section>`.

use crate::cronus_ui_kit::esc;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let rows = pairs(comp)
        .into_iter()
        .map(|(k, v)| {
            format!(
                "<div data-slot=\"json-viewer-row\"><span data-slot=\"json-viewer-key\">{k}</span><span data-slot=\"json-viewer-value\">{v}</span></div>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"json-viewer\">{rows}</div>")
}

fn pairs(comp: &ComponentNode) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for i in &comp.items {
        if i.text.is_empty() {
            continue;
        }
        out.push(split_kv(&i.text));
    }
    if out.is_empty() {
        out.push(split_kv(&comp.name));
    }
    out
}

fn split_kv(text: &str) -> (String, String) {
    match text.split_once(':') {
        Some((k, v)) => (esc(k.trim()), esc(v.trim())),
        None => (esc(text.trim()), String::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn tree(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("json-viewer", items.first().copied().unwrap_or("name: Ada"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<pre"));
        assert!(!html.contains("<code"));
        assert!(!html.contains("<section"));
        assert!(!html.contains("<nav"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("padding:0.85rem 1rem;overflow:auto"));
        assert!(!html.contains("padding:1rem;display:flex;flex-direction:column;gap:0.5rem"));
        assert!(!html.contains("display("));
        assert!(!html.contains("codey("));
    }

    #[test]
    fn root_is_div_of_key_value_rows_not_pre_or_section() {
        let html = render(&tree(&["name: Ada", "role: Engineer"]));
        assert!(html.starts_with("<div data-slot=\"json-viewer\">"));
        assert_eq!(html.matches("data-slot=\"json-viewer-row\"").count(), 2);
        assert_eq!(html.matches("data-slot=\"json-viewer-key\"").count(), 2);
        assert_eq!(html.matches("data-slot=\"json-viewer-value\"").count(), 2);
        assert!(html.contains(
            "<div data-slot=\"json-viewer-row\"><span data-slot=\"json-viewer-key\">name</span><span data-slot=\"json-viewer-value\">Ada</span></div>"
        ));
        assert!(html.contains(">role</span>"));
        assert!(html.contains(">Engineer</span>"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"json-viewer\"><div data-slot=\"json-viewer-row\"><span data-slot=\"json-viewer-key\">name</span><span data-slot=\"json-viewer-value\">Ada</span></div><div data-slot=\"json-viewer-row\"><span data-slot=\"json-viewer-key\">role</span><span data-slot=\"json-viewer-value\">Engineer</span></div></div>"
        );
    }

    #[test]
    fn extra_text_items_become_rows() {
        let mut c = stub("json-viewer", "name: Ada");
        c.items.push(extra("text", "role: Engineer"));
        c.items.push(extra("text", "id: 7"));
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"json-viewer-row\"").count(), 3);
        assert!(html.contains(">name</span>"));
        assert!(html.contains(">Ada</span>"));
        assert!(html.contains(">role</span>"));
        assert!(html.contains(">id</span>"));
        assert!(html.contains(">7</span>"));
        reject_interact(&html);
    }

    #[test]
    fn label_without_colon_still_emits_key_slot() {
        let html = render(&stub("json-viewer", "Demo"));
        assert!(html.starts_with("<div data-slot=\"json-viewer\">"));
        assert_eq!(html.matches("data-slot=\"json-viewer-row\"").count(), 1);
        assert!(html.contains("data-slot=\"json-viewer-key\">Demo</span>"));
        assert!(html.contains("data-slot=\"json-viewer-value\"></span>"));
        reject_interact(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("json-viewer", "a <b>: c & \"d\""));
        assert!(html.contains("data-slot=\"json-viewer-key\">a &lt;b&gt;</span>"));
        assert!(html.contains("data-slot=\"json-viewer-value\">c &amp; &quot;d&quot;</span>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_pre_and_display_surf() {
        let c = tree(&["name: Ada"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("json-viewer", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<pre data-slot=\"json-viewer\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("<code>"));
        assert!(!interact.contains("data-slot=\"json-viewer-row\""));
        assert!(!interact.contains("data-slot=\"json-viewer-key\""));
        assert!(html.contains("data-slot=\"json-viewer-row\""));
        assert!(!html.contains("<pre"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&tree(&["name: Ada"]));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"json-viewer-key\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"json-viewer\"]"));
        assert!(css.contains("[data-slot=\"json-viewer-row\"]"));
        assert!(css.contains("[data-slot=\"json-viewer-key\"]"));
        assert!(css.contains("[data-slot=\"json-viewer-value\"]"));
        assert!(css.contains("var(--cronus-font-mono"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
