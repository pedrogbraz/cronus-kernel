//! Dedicated Task renderer (AI suite). DOM matches React/Radix `Task`
//! (Collapsible root `<div data-state data-slot="task">`) > `TaskTrigger`
//! (`<button data-slot="task-trigger">` > slotless row `<div>` with the
//! chevron/glyph pair — `icon:search` by default — and `<p>` title from the
//! `title` item) > `TaskContent` (`<div data-slot="task-content">` > slotless
//! `<div>` with the start border) of one `<div data-slot="task-item">` per
//! `item "Read the contract"`; `file:"button.tsx"` on an item appends a
//! `<div data-slot="task-item-file">` chip after the text.
//!
//! Zero JS: the trigger button is decorative inside a `<label>` after a
//! visually hidden checkbox that carries the open state (`open:false` closes
//! it; React's `defaultOpen` is true); CSS hides the content until `:checked`
//! and swaps the glyph for the rotated chevron.

use crate::cronus_ui_kit::{attr, attr_nonempty, esc, instance_id, item, truthy};
use crate::parser::{ComponentItemNode, ComponentNode};

const CHEVRON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" data-icon=\"chevron-right\"><path d=\"m9 18 6-6-6-6\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let open = ["open", "defaultOpen", "default-open"]
        .iter()
        .find_map(|k| attr(comp, k))
        .is_none_or(truthy);
    let state = if open { "open" } else { "closed" };
    let checked = if open { " checked" } else { "" };
    let title = item(comp, "title")
        .or_else(|| item(comp, "label"))
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| esc(&comp.name));
    let glyph = attr_nonempty(comp, "icon")
        .and_then(crate::cronus_ui_icons::svg)
        .unwrap_or_else(|| crate::cronus_ui_icons::svg_or_empty("search"));
    let id = instance_id(comp, "task");
    let items: String = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty())
        .map(task_item)
        .collect();
    format!(
        "<div data-state=\"{state}\" data-slot=\"task\"><label><input type=\"checkbox\" id=\"{id}\" aria-label=\"{title}\" aria-controls=\"{id}-content\"{checked}><button type=\"button\" data-slot=\"task-trigger\" data-state=\"{state}\" aria-expanded=\"{open}\" tabindex=\"-1\" aria-hidden=\"true\"><div><div>{CHEVRON}{glyph}</div><p>{title}</p></div></button></label><div id=\"{id}-content\" data-state=\"{state}\" data-slot=\"task-content\"><div>{items}</div></div></div>"
    )
}

fn task_item(i: &ComponentItemNode) -> String {
    let file = i
        .config
        .get("file")
        .filter(|f| !f.is_empty())
        .map(|f| format!(" <div data-slot=\"task-item-file\">{}</div>", esc(f)))
        .unwrap_or_default();
    format!("<div data-slot=\"task-item\">{}{file}</div>", esc(&i.text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::{reset_instance_ids, stub};
    use std::collections::HashMap;

    fn it(text: &str, extra: &[(&str, &str)]) -> ComponentItemNode {
        let mut config = HashMap::new();
        for (k, v) in extra {
            config.insert(k.to_string(), v.to_string());
        }
        ComponentItemNode {
            item_type: "item".into(),
            text: text.into(),
            link: None,
            tone: None,
            config,
        }
    }

    fn docs() -> ComponentNode {
        let mut c = stub("task", "Preparing the PR");
        c.items[0].item_type = "title".into();
        c.items.push(it("Read the contract", &[]));
        c.items.push(it("Port the component", &[]));
        c.items.push(it("Verify in the browser", &[]));
        c
    }

    #[test]
    fn docs_example_is_open_trigger_and_three_items() {
        reset_instance_ids();
        let html = render(&docs());
        assert!(html.starts_with("<div data-state=\"open\" data-slot=\"task\"><label><input type=\"checkbox\" id=\"cui-task-task\" aria-label=\"Preparing the PR\" aria-controls=\"cui-task-task-content\" checked><button type=\"button\" data-slot=\"task-trigger\" data-state=\"open\" aria-expanded=\"true\" tabindex=\"-1\" aria-hidden=\"true\"><div><div><svg"));
        assert!(html.contains("data-icon=\"chevron-right\""));
        assert!(html.contains("data-icon=\"search\""));
        assert!(html.contains("</div><p>Preparing the PR</p></div></button></label><div id=\"cui-task-task-content\" data-state=\"open\" data-slot=\"task-content\"><div><div data-slot=\"task-item\">Read the contract</div><div data-slot=\"task-item\">Port the component</div><div data-slot=\"task-item\">Verify in the browser</div></div></div></div>"));
        assert!(!html.contains("<details"));
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn closed_custom_icon_file_chip_and_escaping() {
        reset_instance_ids();
        let mut c = stub("task", "<b>\"Task\"</b>");
        c.props.insert("open".into(), "false".into());
        c.props.insert("icon".into(), "terminal".into());
        c.items.push(it("Edit <file>", &[("file", "button.tsx")]));
        let html = render(&c);
        assert!(html.starts_with("<div data-state=\"closed\" data-slot=\"task\"><label><input type=\"checkbox\" id=\"cui-task-task\" aria-label=\"&lt;b&gt;&quot;Task&quot;&lt;/b&gt;\" aria-controls=\"cui-task-task-content\"><button"));
        assert!(html.contains("aria-expanded=\"false\""));
        assert!(html.contains("data-icon=\"terminal\""));
        assert!(html.contains("<div data-slot=\"task-item\">Edit &lt;file&gt; <div data-slot=\"task-item-file\">button.tsx</div></div>"));
    }

    #[test]
    fn chrome_toggles_content_by_checkbox() {
        let css = include_str!("cronus_ui_css/task.css");
        assert!(css.contains("[data-slot=\"task\"]:not(:has(> label > input:checked)) > [data-slot=\"task-content\"]"));
        assert!(css.contains("border-inline-start: 2px solid var(--cronus-border)"));
        assert!(css.contains("rotate: 90deg"));
        assert!(!css.contains("zinc-"));
    }

    #[test]
    fn registered_as_dedicated() {
        use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
        assert_eq!(dedicated_fn_name("task"), Some("cronus_ui_task::render"));
        assert_eq!(
            renderer_kind("task"),
            RendererKind::Dedicated("cronus_ui_task::render")
        );
        reset_instance_ids();
        let c = stub("task", "Task");
        let a = crate::cronus_ui_widgets::render(&c).unwrap();
        reset_instance_ids();
        assert_eq!(a, render(&c));
    }
}
