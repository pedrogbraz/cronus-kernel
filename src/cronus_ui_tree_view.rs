//! Dedicated TreeView renderer. DOM matches React slots:
//! `<div data-slot="tree-view">` plus each text as
//! `<div data-slot="tree-view-item"><div data-slot="tree-view-item-trigger">`.
//! Flat list is OK. Not interact `tree()` (`<ul style=SURF>` li list) or
//! catalog `display()` SURF `<section>`.

use crate::cronus_ui_kit::texts;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let items = texts(comp)
        .into_iter()
        .map(|t| {
            format!(
                "<div data-slot=\"tree-view-item\"><div data-slot=\"tree-view-item-trigger\">{t}</div></div>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<div data-slot=\"tree-view\">{items}</div>")
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
        let mut c = stub("tree-view", items.first().copied().unwrap_or("src"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<ul"));
        assert!(!html.contains("<li"));
        assert!(!html.contains("<section"));
        assert!(!html.contains("<nav"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("padding:0.75rem 1.25rem"));
        assert!(!html.contains("padding:1rem;display:flex;flex-direction:column;gap:0.5rem"));
        assert!(!html.contains("display("));
        assert!(!html.contains("tree("));
    }

    #[test]
    fn root_is_div_of_item_triggers_not_ul_or_section() {
        let html = render(&tree(&["src", "Cargo.toml"]));
        assert!(html.starts_with("<div data-slot=\"tree-view\">"));
        assert_eq!(html.matches("data-slot=\"tree-view-item\"").count(), 2);
        assert_eq!(html.matches("data-slot=\"tree-view-item-trigger\"").count(), 2);
        assert!(html.contains(
            "<div data-slot=\"tree-view-item\"><div data-slot=\"tree-view-item-trigger\">src</div></div>"
        ));
        assert!(html.contains(">Cargo.toml</div>"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"tree-view\"><div data-slot=\"tree-view-item\"><div data-slot=\"tree-view-item-trigger\">src</div></div><div data-slot=\"tree-view-item\"><div data-slot=\"tree-view-item-trigger\">Cargo.toml</div></div></div>"
        );
    }

    #[test]
    fn extra_text_items_become_nodes() {
        let mut c = stub("tree-view", "src");
        c.items.push(extra("text", "lib"));
        c.items.push(extra("text", "Cargo.toml"));
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"tree-view-item\"").count(), 3);
        assert!(html.contains(">src</div>"));
        assert!(html.contains(">lib</div>"));
        assert!(html.contains(">Cargo.toml</div>"));
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_one_item() {
        let html = render(&stub("tree-view", "src"));
        assert!(html.starts_with("<div data-slot=\"tree-view\">"));
        assert_eq!(html.matches("data-slot=\"tree-view-item\"").count(), 1);
        assert!(html.contains(
            "<div data-slot=\"tree-view-item\"><div data-slot=\"tree-view-item-trigger\">src</div></div>"
        ));
        reject_interact(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("tree-view", "A <B> & \"C\""));
        assert!(html.contains(
            "<div data-slot=\"tree-view-item-trigger\">A &lt;B&gt; &amp; &quot;C&quot;</div>"
        ));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_ul_and_display_surf() {
        let c = tree(&["src", "Cargo.toml"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("tree-view", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<ul data-slot=\"tree-view\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("<li"));
        assert!(!interact.contains("data-slot=\"tree-view-item\""));
        assert!(!interact.contains("data-slot=\"tree-view-item-trigger\""));
        assert!(html.contains("data-slot=\"tree-view-item-trigger\""));
        assert!(!html.contains("<ul"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&tree(&["src"]));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"tree-view-item-trigger\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"tree-view\"]"));
        assert!(css.contains("[data-slot=\"tree-view-item\"]"));
        assert!(css.contains("[data-slot=\"tree-view-item-trigger\"]"));
        assert!(css.contains("flex-direction: column"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
