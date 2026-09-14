//! Dedicated TreeView renderer. DOM matches React (`tree-view.tsx`):
//! `<div role="tree" data-slot="tree-view">` of `tree-view-item` wrappers,
//! each with a `role="treeitem"` `tree-view-item-trigger` row (chevron
//! `<svg data-slot="tree-view-chevron">` for branches, a 16px spacer for
//! leaves, then the label) and, for an open branch, a `role="group"`
//! `tree-view-group` of child items.
//!
//! Content texts map exactly like the audit fixture: with two or more texts the
//! first is a branch whose children are the rest, rendered expanded (React's
//! `defaultExpandedIds` idle state); a single text is a leaf. The `label` names
//! the tree (`aria-label`) and is never a node.
//!
//! Zero JS: expand/collapse, selection and the roving tabindex need a runtime,
//! so the tree is static in its default-expanded state (no tabindex, no
//! `aria-selected="true"`). Not interact `tree()` (`<ul style=SURF>`) or catalog
//! `display()` SURF `<section>`.

use crate::cronus_ui_kit::esc;
use crate::parser::ComponentNode;

const NAME_KINDS: &[&str] = &["label", "title"];

/// lucide `ChevronRight`.
const CHEVRON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" data-slot=\"tree-view-chevron\"><path d=\"m9 18 6-6-6-6\"/></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let labels = content(comp);
    let body = match labels.split_first() {
        Some((root, rest)) if !rest.is_empty() => {
            let children = rest.iter().map(|l| leaf(l, 2)).collect::<String>();
            format!(
                "<div data-slot=\"tree-view-item\"><div role=\"treeitem\" aria-level=\"1\" aria-selected=\"false\" aria-expanded=\"true\" data-slot=\"tree-view-item-trigger\" data-state=\"open\">{CHEVRON}<span>{root}</span></div><div role=\"group\" data-slot=\"tree-view-group\">{children}</div></div>"
            )
        }
        _ => labels.iter().map(|l| leaf(l, 1)).collect(),
    };
    let aria = esc(aria_label(comp).unwrap_or("Tree"));
    format!("<div role=\"tree\" aria-label=\"{aria}\" data-slot=\"tree-view\">{body}</div>")
}

fn leaf(label: &str, level: u8) -> String {
    format!(
        "<div data-slot=\"tree-view-item\"><div role=\"treeitem\" aria-level=\"{level}\" aria-selected=\"false\" data-slot=\"tree-view-item-trigger\" data-state=\"closed\"><span aria-hidden=\"true\"></span><span>{label}</span></div></div>"
    )
}

fn aria_label(comp: &ComponentNode) -> Option<&str> {
    comp.props
        .get("aria-label")
        .or_else(|| comp.items.iter().find_map(|i| i.config.get("aria-label")))
        .map(String::as_str)
        .filter(|v| !v.is_empty())
}

fn content(comp: &ComponentNode) -> Vec<String> {
    let texts: Vec<String> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty() && !NAME_KINDS.contains(&i.item_type.as_str()))
        .map(|i| esc(&i.text))
        .collect();
    if !texts.is_empty() {
        return texts;
    }
    comp.items
        .iter()
        .find(|i| !i.text.is_empty())
        .map(|i| vec![esc(&i.text)])
        .unwrap_or_default()
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

    fn reject_interact(html: &str) {
        assert!(!html.contains("<ul"));
        assert!(!html.contains("<li"));
        assert!(!html.contains("<section"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("tabindex"));
    }

    /// Emitted audit source: `label "Files"` + texts `src`, `README.md` +
    /// `aria-label:"Files"`. The label must not become a node.
    #[test]
    fn audit_source_renders_open_branch_with_child_leaf() {
        let mut c = stub("tree-view", "Files");
        c.items.push(extra("text", "src"));
        c.items.push(extra("text", "README.md"));
        c.items
            .last_mut()
            .unwrap()
            .config
            .insert("aria-label".into(), "Files".into());
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<div role=\"tree\" aria-label=\"Files\" data-slot=\"tree-view\"><div data-slot=\"tree-view-item\"><div role=\"treeitem\" aria-level=\"1\" aria-selected=\"false\" aria-expanded=\"true\" data-slot=\"tree-view-item-trigger\" data-state=\"open\">{CHEVRON}<span>src</span></div><div role=\"group\" data-slot=\"tree-view-group\"><div data-slot=\"tree-view-item\"><div role=\"treeitem\" aria-level=\"2\" aria-selected=\"false\" data-slot=\"tree-view-item-trigger\" data-state=\"closed\"><span aria-hidden=\"true\"></span><span>README.md</span></div></div></div></div></div>"
            )
        );
        assert!(!html.contains(">Files<"));
        reject_interact(&html);
    }

    #[test]
    fn single_text_is_a_leaf_without_chevron() {
        let mut c = stub("tree-view", "Files");
        c.items.push(extra("text", "Cargo.toml"));
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"tree-view-item\"").count(), 1);
        assert!(!html.contains("tree-view-chevron"));
        assert!(!html.contains("tree-view-group"));
        assert!(html.contains("aria-level=\"1\""));
        assert!(html.contains("<span>Cargo.toml</span>"));
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_one_item() {
        let html = render(&stub("tree-view", "src"));
        assert_eq!(html.matches("data-slot=\"tree-view-item\"").count(), 1);
        assert!(html.contains("aria-label=\"Tree\""));
        assert!(html.contains("<span>src</span>"));
        reject_interact(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("tree-view", "A <B> & \"C\""));
        assert!(html.contains("<span>A &lt;B&gt; &amp; &quot;C&quot;</span>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_ul_and_display_surf() {
        let mut c = stub("tree-view", "Files");
        c.items.push(extra("item", "src"));
        c.items.push(extra("item", "Cargo.toml"));
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("tree-view", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<ul data-slot=\"tree-view\""));
        assert!(html.contains("data-slot=\"tree-view-item-trigger\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("tree-view", "src"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"tree-view-item-trigger\""));
        });
    }

    /// Geometry parity (Wave 1t): `text-sm` 14px/20px, 32px rows, 16px chevron
    /// rotated open, level-2 rows indented to 1.5rem.
    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"tree-view\"] {\n  display: flex; flex-direction: column; width: 100%; min-width: 0;\n  font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg); user-select: none;\n}"
        ));
        assert!(css.contains(
            "[data-slot=\"tree-view-item-trigger\"][aria-level=\"2\"] { padding-inline-start: 1.5rem; }"
        ));
        assert!(css.contains("[data-slot=\"tree-view-item-trigger\"][data-state=\"open\"] > [data-slot=\"tree-view-chevron\"] { transform: rotate(90deg); }"));
        assert!(css.contains("var(--cronus-fg-tertiary)"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(!css.contains("zinc-"));
    }
}
