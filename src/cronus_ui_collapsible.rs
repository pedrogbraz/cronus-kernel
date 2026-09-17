//! Dedicated Collapsible renderer. DOM matches React/Radix:
//! unslotted root (Radix Root carries no data-slot) > unslotted trigger
//! `<button data-state>` (label) > `<div data-slot="collapsible-content"
//! data-state>` (body texts). `open:false` starts closed (the docs' default).
//!
//! With an `icon:` prop the trigger row is the docs' composition: a bordered
//! row holding the label `<span>` and a ghost `icon-sm` Button
//! (`aria-label:` names it) as the `CollapsibleTrigger`; `item` lines render
//! as bordered rows in the content (`font:mono` for the docs' region list);
//! `max-width:sm` is the example's `max-w-sm`.
//!
//! Zero JS: the root is `<details>` and the trigger sits in its `<summary>`,
//! so the summary toggles the content natively (click, Enter, Space) and
//! exposes the expanded state. The button keeps React's element and look but
//! is decorative (`aria-hidden`, `tabindex="-1"`, `pointer-events: none`).
//! The content opens with React's `cronus-collapsible-down` (220ms) keyframes.
//! Gaps vs Radix: `data-state` reflects the initial state only (CSS hides
//! closed content via `<details>`), the focusable element is the `<summary>`,
//! not the `<button>`. Not interact `accordion("collapsible")` (SURF, no
//! collapsible-content).

use crate::cronus_ui_kit::{attr, attr_nonempty, esc, label_of, truthy};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let open = attr(comp, "open")
        .or_else(|| attr(comp, "defaultOpen"))
        .or_else(|| attr(comp, "default-open"))
        .is_none_or(truthy);
    let (open_attr, state) = if open {
        (" open", "open")
    } else {
        ("", "closed")
    };
    let rows: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty())
        .map(|i| format!("<div>{}</div>", esc(&i.text)))
        .collect();
    let body = if rows.is_empty() {
        comp.items
            .iter()
            .filter(|i| i.item_type != "label" && i.item_type != "title" && !i.text.is_empty())
            .map(|i| esc(&i.text))
            .collect::<Vec<_>>()
            .join("")
    } else {
        rows.concat()
    };
    let mut content_classes: Vec<&str> = Vec::new();
    if !rows.is_empty() {
        content_classes.push("rows");
    }
    if attr_nonempty(comp, "font").is_some_and(|f| f.trim() == "mono") {
        content_classes.push("font-mono");
    }
    let content_class = if content_classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", content_classes.join(" "))
    };
    let mut root_classes: Vec<&str> = Vec::new();
    let trigger = match attr_nonempty(comp, "icon") {
        Some(icon) => {
            root_classes.push("cui-collapsible-row");
            let name = attr_nonempty(comp, "aria-label")
                .map(esc)
                .unwrap_or_else(|| label.clone());
            format!(
                "<span>{label}</span><button type=\"button\" data-slot=\"button\" data-variant=\"ghost\" data-size=\"icon-sm\" class=\"cui-btn\" aria-label=\"{name}\" data-state=\"{state}\" tabindex=\"-1\" aria-hidden=\"true\">{}</button>",
                crate::cronus_ui_icons::svg_or_empty(icon)
            )
        }
        None => format!(
            "<button type=\"button\" data-state=\"{state}\" tabindex=\"-1\" aria-hidden=\"true\">{label}</button>"
        ),
    };
    if let Some(mw) = crate::cronus_ui_card::max_width_class(comp) {
        root_classes.push(mw);
    }
    let root_class = if root_classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", root_classes.join(" "))
    };
    format!(
        "<details{open_attr}{root_class} data-state=\"{state}\"><summary>{trigger}</summary><div data-state=\"{state}\" data-slot=\"collapsible-content\"{content_class}>{body}</div></details>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn extra(text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "text".into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains(" disabled"));
        assert!(!crate::cli::stub_renderer_gate::looks_like_interact_generic(html));
    }

    /// wave1t: Radix Root has no `data-slot`; an extra kernel slot would be
    /// an unpaired geometry node. The native disclosure is `<details open>`.
    #[test]
    fn root_is_unslotted_native_disclosure_open_by_default() {
        let mut c = stub("collapsible", "Toggle");
        c.items.push(extra("Hidden body"));
        let html = render(&c);
        assert_eq!(
            html,
            "<details open data-state=\"open\"><summary><button type=\"button\" data-state=\"open\" tabindex=\"-1\" aria-hidden=\"true\">Toggle</button></summary><div data-state=\"open\" data-slot=\"collapsible-content\">Hidden body</div></details>"
        );
        assert!(!html.contains("data-slot=\"collapsible\""));
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_opens_empty_content() {
        let html = render(&stub("collapsible", "Show more"));
        assert!(html.contains(">Show more</button></summary>"));
        assert!(html.contains("data-slot=\"collapsible-content\"></div>"));
        reject_interact(&html);
    }

    /// Docs "Default": closed, the bordered row with the label and a ghost
    /// icon-sm trigger, mono region rows in the content, `max-w-sm`.
    #[test]
    fn docs_row_trigger_with_icon_and_item_rows_starts_closed() {
        let mut c = stub("collapsible", "Deployment regions");
        c.props.insert("open".into(), "false".into());
        c.props.insert("icon".into(), "chevrons-up-down".into());
        c.props.insert("aria-label".into(), "Toggle regions".into());
        c.props.insert("max-width".into(), "sm".into());
        c.props.insert("font".into(), "mono".into());
        for r in ["us-east-1", "eu-west-1"] {
            let mut i = extra(r);
            i.item_type = "item".into();
            c.items.push(i);
        }
        let html = render(&c);
        assert!(html.starts_with("<details class=\"cui-collapsible-row mw-sm\" data-state=\"closed\"><summary><span>Deployment regions</span><button type=\"button\" data-slot=\"button\" data-variant=\"ghost\" data-size=\"icon-sm\" class=\"cui-btn\" aria-label=\"Toggle regions\" data-state=\"closed\" tabindex=\"-1\" aria-hidden=\"true\"><svg "));
        assert!(html.contains("data-icon=\"chevrons-up-down\""));
        assert!(html.ends_with("</svg></button></summary><div data-state=\"closed\" data-slot=\"collapsible-content\" class=\"rows font-mono\"><div>us-east-1</div><div>eu-west-1</div></div></details>"));
        assert!(!html.starts_with("<details open"));
        reject_interact(&html);
        let css = include_str!("cronus_ui_css/collapsible.css");
        assert!(css.contains("details.cui-collapsible-row > summary {\n  display: flex; align-items: center; justify-content: space-between; gap: 1rem;\n  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border); padding: 0.5rem 1rem;\n}"));
        assert!(css.contains("[data-slot=\"collapsible-content\"].rows > div {\n  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border); padding: 0.5rem 1rem;\n  font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg);\n}"));
        assert!(css.contains("@keyframes cui-collapsible-down"));
        assert!(css.contains("details[open] > [data-slot=\"collapsible-content\"] { animation: cui-collapsible-down 220ms var(--ease-out-quart); }"));
    }

    #[test]
    fn skips_interact_accordion_surf() {
        let mut c = stub("collapsible", "Show more");
        c.items.push(extra("Hidden details here."));
        let html = render(&c);
        assert!(html.contains("data-slot=\"collapsible-content\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("collapsible", "Show more"));
            reject_interact(&html);
        });
    }

    /// wave1t: `text-sm` is 14px/20px (not the canvas 1.5 line-height).
    #[test]
    fn chrome_content_is_text_sm_pair() {
        let css = include_str!("cronus_ui_css/collapsible.css");
        assert!(css.contains(
            "[data-slot=\"collapsible-content\"] {\n  overflow: hidden; font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg-secondary);\n}"
        ));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }

    /// The summary lays out like React's root > button (block line box, no
    /// marker) and hands clicks through the decorative button.
    #[test]
    fn chrome_summary_is_markerless_block_trigger() {
        let css = include_str!("cronus_ui_css/collapsible.css");
        assert!(css.contains("details:has(> [data-slot=\"collapsible-content\"]) > summary {\n  display: block; list-style: none; cursor: pointer;"));
        assert!(css.contains("details:has(> [data-slot=\"collapsible-content\"]) > summary::-webkit-details-marker { display: none; }"));
        assert!(css.contains("details:has(> [data-slot=\"collapsible-content\"]) > summary > button { pointer-events: none; }"));
    }
}
