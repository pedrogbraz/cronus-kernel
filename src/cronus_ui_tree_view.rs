//! Dedicated TreeView renderer. DOM matches React (`tree-view.tsx`):
//! `<div role="tree" data-slot="tree-view">` of `tree-view-item` wrappers,
//! each with a `role="treeitem"` `tree-view-item-trigger` row (chevron
//! `<svg data-slot="tree-view-chevron">` for branches, a 16px spacer for
//! leaves, an optional `tree-view-icon` glyph, then the label) and, for an
//! open branch, a `role="group"` `tree-view-group` of child items.
//!
//! Nodes are `item "src" icon:folder level:2 open:true selected:true` lines:
//! `level:` (1 by default) nests each node under the nearest shallower one, a
//! node with children is a branch, `open:true` expands it (React
//! `defaultExpandedIds`; a collapsed branch keeps its group in the DOM, hidden
//! by CSS, so it can open without script) and `selected:true` is
//! `defaultValue`. Without levels the content texts map like
//! the audit fixture: with two or more texts the first is an expanded branch
//! whose children are the rest; a single text is a leaf. The `label` names the
//! tree (`aria-label`) and is never a node. `framed:true` is the docs'
//! `rounded-lg border p-1.5`, `max-width:xs` its `max-w-xs`.
//!
//! Zero JS: a branch row wraps its chevron and label in a `<label>` (laid out
//! with `display: contents`) around a visually hidden checkbox that carries the
//! expanded state; CSS shows the group and rotates the chevron while it is
//! checked. Selection and the roving tabindex need a runtime, so the tree is
//! static there (first row `tabindex="0"`, the rest `-1`, like React's idle
//! render). Not interact `tree()` (`<ul style=SURF>`) or catalog `display()`
//! SURF `<section>`.

use crate::cronus_ui_kit::{attr_nonempty, esc, flag, instance_id};
use crate::parser::{ComponentItemNode, ComponentNode};

const NAME_KINDS: &[&str] = &["label", "title"];

/// lucide `ChevronRight`.
const CHEVRON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" data-slot=\"tree-view-chevron\"><path d=\"m9 18 6-6-6-6\"/></svg>";

struct Node {
    label: String,
    icon: String,
    open: bool,
    selected: bool,
    children: Vec<Node>,
}

pub fn render(comp: &ComponentNode) -> String {
    let nodes = nodes_of(comp);
    let mut first = true;
    let body = nodes
        .iter()
        .map(|n| node(comp, n, 1, &mut first))
        .collect::<String>();
    let aria = esc(aria_label(comp).unwrap_or("Tree"));
    let mut classes: Vec<&str> = Vec::new();
    if let Some(mw) = crate::cronus_ui_card::max_width_class(comp) {
        classes.push(mw);
    }
    if flag(comp, "framed") {
        classes.push("framed");
    }
    let class = if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    };
    format!("<div role=\"tree\" aria-label=\"{aria}\" data-slot=\"tree-view\"{class}>{body}</div>")
}

fn node(comp: &ComponentNode, n: &Node, level: u8, first: &mut bool) -> String {
    let tabindex = if *first { "0" } else { "-1" };
    *first = false;
    let selected = if n.selected {
        " aria-selected=\"true\" data-selected=\"true\""
    } else {
        " aria-selected=\"false\""
    };
    let icon = if n.icon.is_empty() {
        String::new()
    } else {
        format!(
            "<span aria-hidden=\"true\" data-slot=\"tree-view-icon\">{}</span>",
            n.icon
        )
    };
    let label = &n.label;
    if n.children.is_empty() {
        return format!(
            "<div data-slot=\"tree-view-item\"><div role=\"treeitem\" aria-level=\"{level}\"{selected} data-slot=\"tree-view-item-trigger\" data-state=\"closed\" tabindex=\"{tabindex}\"><span aria-hidden=\"true\"></span>{icon}<span>{label}</span></div></div>"
        );
    }
    let (state, expanded, checked) = if n.open {
        ("open", "true", " checked")
    } else {
        ("closed", "false", "")
    };
    let id = instance_id(comp, "branch");
    let children = n
        .children
        .iter()
        .map(|c| node(comp, c, level + 1, first))
        .collect::<String>();
    format!(
        "<div data-slot=\"tree-view-item\"><div role=\"treeitem\" aria-level=\"{level}\"{selected} aria-expanded=\"{expanded}\" data-slot=\"tree-view-item-trigger\" data-state=\"{state}\" tabindex=\"{tabindex}\"><label><input type=\"checkbox\" id=\"{id}\" aria-label=\"Toggle {label}\"{checked}>{CHEVRON}{icon}<span>{label}</span></label></div><div role=\"group\" data-slot=\"tree-view-group\">{children}</div></div>"
    )
}

fn aria_label(comp: &ComponentNode) -> Option<&str> {
    attr_nonempty(comp, "aria-label")
}

fn nodes_of(comp: &ComponentNode) -> Vec<Node> {
    let items: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty() && !NAME_KINDS.contains(&i.item_type.as_str()))
        .collect();
    let levelled = items.iter().any(|i| i.config.contains_key("level"))
        || items
            .iter()
            .any(|i| i.config.contains_key("icon") || i.config.contains_key("open"));
    if levelled {
        return build(&items);
    }
    let labels: Vec<String> = match items.is_empty() {
        true => comp
            .items
            .iter()
            .find(|i| !i.text.is_empty())
            .map(|i| vec![esc(&i.text)])
            .unwrap_or_default(),
        false => items.iter().map(|i| esc(&i.text)).collect(),
    };
    let leaf = |l: &String| Node {
        label: l.clone(),
        icon: String::new(),
        open: false,
        selected: false,
        children: Vec::new(),
    };
    match labels.split_first() {
        Some((root, rest)) if !rest.is_empty() => vec![Node {
            label: root.clone(),
            icon: String::new(),
            open: true,
            selected: false,
            children: rest.iter().map(leaf).collect(),
        }],
        _ => labels.iter().map(leaf).collect(),
    }
}

/// Nest `level:` items: each node hangs under the nearest preceding shallower one.
fn build(items: &[&ComponentItemNode]) -> Vec<Node> {
    let mut roots: Vec<Node> = Vec::new();
    // Path of indices from the roots down to the current open branch.
    let mut path: Vec<usize> = Vec::new();
    let mut levels: Vec<u8> = Vec::new();
    for i in items {
        let level: u8 = i
            .config
            .get("level")
            .and_then(|l| l.trim().parse().ok())
            .unwrap_or(1)
            .max(1);
        while let Some(last) = levels.last() {
            if *last >= level {
                levels.pop();
                path.pop();
            } else {
                break;
            }
        }
        let node = Node {
            label: esc(&i.text),
            icon: crate::cronus_ui_kit::item_icon(i),
            open: i
                .config
                .get("open")
                .is_some_and(|o| crate::cronus_ui_kit::truthy(o)),
            selected: i
                .config
                .get("selected")
                .is_some_and(|s| crate::cronus_ui_kit::truthy(s)),
            children: Vec::new(),
        };
        let mut siblings = &mut roots;
        for &index in &path {
            siblings = &mut siblings[index].children;
        }
        siblings.push(node);
        path.push(siblings.len() - 1);
        levels.push(level);
    }
    roots
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::{reset_instance_ids, stub};

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
    }

    /// Emitted audit source: `label "Files"` + texts `src`, `README.md` +
    /// `aria-label:"Files"`. The label must not become a node; the branch row
    /// carries the expanded state in a hidden checkbox (zero JS).
    #[test]
    fn audit_source_renders_open_branch_with_child_leaf() {
        reset_instance_ids();
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
                "<div role=\"tree\" aria-label=\"Files\" data-slot=\"tree-view\"><div data-slot=\"tree-view-item\"><div role=\"treeitem\" aria-level=\"1\" aria-selected=\"false\" aria-expanded=\"true\" data-slot=\"tree-view-item-trigger\" data-state=\"open\" tabindex=\"0\"><label><input type=\"checkbox\" id=\"cui-tree-view-branch\" aria-label=\"Toggle src\" checked>{CHEVRON}<span>src</span></label></div><div role=\"group\" data-slot=\"tree-view-group\"><div data-slot=\"tree-view-item\"><div role=\"treeitem\" aria-level=\"2\" aria-selected=\"false\" data-slot=\"tree-view-item-trigger\" data-state=\"closed\" tabindex=\"-1\"><span aria-hidden=\"true\"></span><span>README.md</span></div></div></div></div></div>"
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

    /// Docs "File tree": `level:` nesting, folder/file glyphs, `open` branches,
    /// a selected leaf, a collapsed branch keeping its children unrendered.
    #[test]
    fn levelled_items_build_the_docs_file_tree() {
        reset_instance_ids();
        let mut c = stub("tree-view", "Project files");
        c.items.clear();
        let mk = |text: &str, pairs: &[(&str, &str)]| {
            let mut i = extra("item", text);
            for (k, v) in pairs {
                i.config.insert(k.to_string(), v.to_string());
            }
            i
        };
        c.items
            .push(mk("src", &[("icon", "folder"), ("open", "true")]));
        c.items.push(mk(
            "components",
            &[("icon", "folder"), ("level", "2"), ("open", "true")],
        ));
        c.items.push(mk(
            "button.tsx",
            &[("icon", "file-text"), ("level", "3"), ("selected", "true")],
        ));
        c.items
            .push(mk("card.tsx", &[("icon", "file-text"), ("level", "3")]));
        c.items
            .push(mk("index.ts", &[("icon", "file-text"), ("level", "2")]));
        c.items.push(mk("package.json", &[("icon", "file-text")]));
        c.items.push(mk("dist", &[("icon", "folder")]));
        c.items
            .push(mk("bundle.js", &[("icon", "file-text"), ("level", "2")]));
        c.props.insert("aria-label".into(), "Project files".into());
        c.props.insert("max-width".into(), "xs".into());
        c.props.insert("framed".into(), "true".into());
        let html = render(&c);
        assert!(html.starts_with("<div role=\"tree\" aria-label=\"Project files\" data-slot=\"tree-view\" class=\"mw-xs framed\"><div data-slot=\"tree-view-item\"><div role=\"treeitem\" aria-level=\"1\" aria-selected=\"false\" aria-expanded=\"true\" data-slot=\"tree-view-item-trigger\" data-state=\"open\" tabindex=\"0\"><label><input type=\"checkbox\" id=\"cui-tree-view-branch\" aria-label=\"Toggle src\" checked>"));
        assert!(html.contains("<span aria-hidden=\"true\" data-slot=\"tree-view-icon\"><svg "));
        assert!(html.contains("data-icon=\"folder\""));
        assert!(html.contains("<div role=\"treeitem\" aria-level=\"3\" aria-selected=\"true\" data-selected=\"true\" data-slot=\"tree-view-item-trigger\" data-state=\"closed\" tabindex=\"-1\"><span aria-hidden=\"true\"></span><span aria-hidden=\"true\" data-slot=\"tree-view-icon\">"));
        assert!(html.contains("<span>button.tsx</span>"));
        assert!(html.contains("aria-level=\"2\" aria-selected=\"false\" data-slot=\"tree-view-item-trigger\" data-state=\"closed\" tabindex=\"-1\"><span aria-hidden=\"true\"></span><span aria-hidden=\"true\" data-slot=\"tree-view-icon\">"));
        assert!(html
            .contains("id=\"cui-tree-view-branch-2\" aria-label=\"Toggle components\" checked>"));
        // `dist` is a collapsed branch: unchecked, group present, child hidden by CSS.
        assert!(html.contains("aria-expanded=\"false\" data-slot=\"tree-view-item-trigger\" data-state=\"closed\" tabindex=\"-1\"><label><input type=\"checkbox\" id=\"cui-tree-view-branch-3\" aria-label=\"Toggle dist\">"));
        assert!(html.contains("<span>bundle.js</span>"));
        assert_eq!(html.matches("tabindex=\"0\"").count(), 1);
        assert_eq!(html.matches("data-slot=\"tree-view-item\"").count(), 8);
        reject_interact(&html);
        let css = include_str!("cronus_ui_css/tree-view.css");
        assert!(
            css.contains("[data-slot=\"tree-view-item-trigger\"] > label { display: contents; }")
        );
        assert!(css.contains("[data-slot=\"tree-view-item\"]:has(> [data-slot=\"tree-view-item-trigger\"] > label > input:not(:checked)) > [data-slot=\"tree-view-group\"] { display: none; }"));
        assert!(css.contains("[data-slot=\"tree-view-item-trigger\"]:has(> label > input:checked) > label > [data-slot=\"tree-view-chevron\"] { transform: rotate(90deg); }"));
        assert!(css.contains("[data-slot=\"tree-view-item-trigger\"][aria-selected=\"true\"] {\n  background: var(--cronus-surface-overlay); font-weight: 500; color: var(--cronus-fg);\n}"));
        assert!(css.contains("[data-slot=\"tree-view\"].framed { border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border); padding: 0.375rem; box-sizing: border-box; }"));
    }

    #[test]
    fn skips_interact_ul_and_display_surf() {
        let mut c = stub("tree-view", "Files");
        c.items.push(extra("item", "src"));
        c.items.push(extra("item", "Cargo.toml"));
        let html = render(&c);
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
        let css = include_str!("cronus_ui_css/tree-view.css");
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
