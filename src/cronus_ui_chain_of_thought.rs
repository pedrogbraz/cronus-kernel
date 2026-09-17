//! Dedicated ChainOfThought renderer (AI suite). DOM matches React/Radix
//! `ChainOfThought` (Collapsible root `<div data-state data-slot="chain-of-thought">`)
//! > `ChainOfThoughtHeader` (`<button data-slot="chain-of-thought-header">`
//! with the chevron/brain glyph pair and a `<span>` label — `label "…"`,
//! default "Chain of Thought") > `ChainOfThoughtContent`
//! (`<div data-slot="chain-of-thought-content">`) of one
//! `<div data-slot="chain-of-thought-step" data-status>` per
//! `item "Parse the question" status:complete|active|pending description:"…" icon:search`:
//! the lucide glyph (Dot by default) over its connector line, the label (a
//! `text-shimmer` while `active`) and a `<div data-slot="response">`
//! description.
//!
//! Zero JS: the header button is decorative inside a `<label>` after a
//! visually hidden checkbox that carries the open state (`open:true` — React's
//! `defaultOpen`, closed by default); CSS shows the content and rotates the
//! chevron after `:checked`. `data-state` reflects the initial state only.

use crate::cronus_ui_kit::{attr, attr_nonempty, esc, instance_id, truthy};
use crate::parser::{ComponentItemNode, ComponentNode};

const BRAIN: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" data-icon=\"brain\"><path d=\"M12 18V5\"></path><path d=\"M15 13a4.17 4.17 0 0 1-3-4 4.17 4.17 0 0 1-3 4\"></path><path d=\"M17.598 6.5A3 3 0 1 0 12 5a3 3 0 1 0-5.598 1.5\"></path><path d=\"M17.997 5.125a4 4 0 0 1 2.526 5.77\"></path><path d=\"M18 18a4 4 0 0 0 2-7.464\"></path><path d=\"M19.967 17.483A4 4 0 1 1 12 18a4 4 0 1 1-7.967-.517\"></path><path d=\"M6 18a4 4 0 0 1-2-7.464\"></path><path d=\"M6.003 5.125a4 4 0 0 0-2.526 5.77\"></path></svg>";
const CHEVRON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" data-icon=\"chevron-right\"><path d=\"m9 18 6-6-6-6\"></path></svg>";
const DOT: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" data-icon=\"dot\"><circle cx=\"12.1\" cy=\"12.1\" r=\"1\"></circle></svg>";
const HEADER: &str = "Chain of Thought";
const STATUSES: &[&str] = &["complete", "active", "pending"];
/// React `TextShimmer` default `spread` (px per UTF-16 unit).
const SPREAD_PER_CHAR: usize = 2;

pub fn render(comp: &ComponentNode) -> String {
    let open = ["open", "defaultOpen", "default-open"]
        .iter()
        .find_map(|k| attr(comp, k))
        .is_some_and(truthy);
    let state = if open { "open" } else { "closed" };
    let checked = if open { " checked" } else { "" };
    let header = attr_nonempty(comp, "header")
        .or_else(|| crate::cronus_ui_kit::item(comp, "label").filter(|l| !l.is_empty()))
        .map(esc)
        .unwrap_or_else(|| HEADER.into());
    let id = instance_id(comp, "cot");
    let steps: String = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty())
        .map(step)
        .collect();
    format!(
        "<div data-state=\"{state}\" data-slot=\"chain-of-thought\"><label><input type=\"checkbox\" id=\"{id}\" aria-label=\"{header}\" aria-controls=\"{id}-content\"{checked}><button type=\"button\" data-slot=\"chain-of-thought-header\" data-state=\"{state}\" aria-expanded=\"{open}\" tabindex=\"-1\" aria-hidden=\"true\"><div>{CHEVRON}{BRAIN}</div><span>{header}</span></button></label><div id=\"{id}-content\" data-state=\"{state}\" data-slot=\"chain-of-thought-content\">{steps}</div></div>"
    )
}

fn step(item: &ComponentItemNode) -> String {
    let status = item
        .config
        .get("status")
        .map(|s| s.trim())
        .filter(|s| STATUSES.contains(s))
        .unwrap_or("complete");
    let icon = item
        .config
        .get("icon")
        .and_then(|i| crate::cronus_ui_icons::svg(i))
        .unwrap_or_else(|| DOT.into());
    let label = esc(&item.text);
    let label = if status == "active" {
        format!(
            "<p data-slot=\"text-shimmer\" data-spread=\"{}\">{label}</p>",
            item.text.encode_utf16().count() * SPREAD_PER_CHAR
        )
    } else {
        label
    };
    let description = item
        .config
        .get("description")
        .filter(|d| !d.is_empty())
        .map(|d| format!("<div data-slot=\"response\">{}</div>", esc(d)))
        .unwrap_or_default();
    format!(
        "<div data-slot=\"chain-of-thought-step\" data-status=\"{status}\"><div>{icon}<div></div></div><div><div>{label}</div>{description}</div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::{reset_instance_ids, stub};
    use std::collections::HashMap;

    fn item(text: &str, extra: &[(&str, &str)]) -> ComponentItemNode {
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
        let mut c = stub("chain-of-thought", "Chain of Thought");
        c.props.insert("open".into(), "true".into());
        c.items.push(item("Parse the question", &[]));
        c.items.push(item("Retrieve the docs", &[]));
        c.items.push(item("Draft the answer", &[]));
        c
    }

    #[test]
    fn docs_example_is_open_checkbox_header_and_three_complete_steps() {
        reset_instance_ids();
        let html = render(&docs());
        assert!(html.starts_with("<div data-state=\"open\" data-slot=\"chain-of-thought\"><label><input type=\"checkbox\" id=\"cui-chain-of-thought-cot\" aria-label=\"Chain of Thought\" aria-controls=\"cui-chain-of-thought-cot-content\" checked><button type=\"button\" data-slot=\"chain-of-thought-header\" data-state=\"open\" aria-expanded=\"true\" tabindex=\"-1\" aria-hidden=\"true\"><div><svg"));
        assert!(html.contains("data-icon=\"chevron-right\""));
        assert!(html.contains("data-icon=\"brain\""));
        assert!(html.contains("</div><span>Chain of Thought</span></button></label><div id=\"cui-chain-of-thought-cot-content\" data-state=\"open\" data-slot=\"chain-of-thought-content\"><div data-slot=\"chain-of-thought-step\" data-status=\"complete\"><div><svg"));
        assert!(html.contains("data-icon=\"dot\""));
        assert!(html.contains("<div></div></div><div><div>Parse the question</div></div></div>"));
        assert_eq!(
            html.matches("data-slot=\"chain-of-thought-step\"").count(),
            3
        );
        assert!(!html.contains("<details"));
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn closed_by_default_active_shimmer_description_and_icon() {
        reset_instance_ids();
        let mut c = stub("chain-of-thought", "Thinking");
        c.items.push(item(
            "Searching \"docs\"",
            &[
                ("status", "active"),
                ("icon", "search"),
                ("description", "<b>3</b> results"),
            ],
        ));
        c.items.push(item("Answer", &[("status", "pending")]));
        let html = render(&c);
        assert!(html.starts_with("<div data-state=\"closed\" data-slot=\"chain-of-thought\"><label><input type=\"checkbox\" id=\"cui-chain-of-thought-cot\" aria-label=\"Thinking\" aria-controls=\"cui-chain-of-thought-cot-content\"><button"));
        assert!(html.contains("aria-expanded=\"false\""));
        assert!(html.contains("data-status=\"active\"><div><svg"));
        assert!(html.contains("data-icon=\"search\""));
        assert!(html.contains("<div><p data-slot=\"text-shimmer\" data-spread=\"32\">Searching &quot;docs&quot;</p></div><div data-slot=\"response\">&lt;b&gt;3&lt;/b&gt; results</div>"));
        assert!(html.contains("data-status=\"pending\""));
    }

    #[test]
    fn chrome_toggles_content_by_checkbox() {
        let css = include_str!("cronus_ui_css/chain-of-thought.css");
        assert!(css.contains("[data-slot=\"chain-of-thought\"]:not(:has(> label > input:checked)) > [data-slot=\"chain-of-thought-content\"]"));
        assert!(css.contains("[data-status=\"pending\"]"));
        assert!(css.contains("rotate: 90deg"));
        assert!(!css.contains("zinc-"));
    }

    #[test]
    fn registered_as_dedicated() {
        use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
        assert_eq!(
            dedicated_fn_name("chain-of-thought"),
            Some("cronus_ui_chain_of_thought::render")
        );
        assert_eq!(
            renderer_kind("chain-of-thought"),
            RendererKind::Dedicated("cronus_ui_chain_of_thought::render")
        );
        reset_instance_ids();
        let c = stub("chain-of-thought", "Chain of Thought");
        let a = crate::cronus_ui_widgets::render(&c).unwrap();
        reset_instance_ids();
        assert_eq!(a, render(&c));
    }
}
