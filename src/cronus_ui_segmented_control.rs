//! Dedicated SegmentedControl renderer. DOM mirrors React:
//! `<div data-slot="segmented-control" data-size role="radiogroup">` plus
//! per option `<label>` > visually hidden `<input type="radio">` + React's
//! `<button type="button" data-slot="segmented-control-item" role="radio">`,
//! then one `segmented-control-thumb`.
//! Options are `item`/`text` lines; the `label` names the group; `size:sm`
//! is React's small size; `disabled:true` on an item disables it.
//!
//! Zero JS: the radios share a page-unique `name`; clicking an option's label
//! checks it and Arrow keys move the selection. The thumb is anchored to the
//! checked option's label (`anchor-name` per label, `position-anchor` picked
//! with `:has(input:checked)`) and its insets transition on React's 0.4 s
//! bounce-0.15 spring, so it slides between options like the Motion
//! `layoutId`. React keeps the thumb inside the active item; the kernel keeps
//! it as the root's last child so it can be anchored to any option (same
//! geometry). Without anchor positioning the checked item paints the thumb's
//! surface itself. The button keeps React's slot and look but is decorative
//! (`aria-hidden`, `tabindex="-1"`, `pointer-events: none`). `data-state` /
//! `aria-checked` stay at the initial state (the native radio carries the
//! live state). Not interact `radios()` (inline-styled `<input type="radio">`
//! labels).

use crate::cronus_ui_kit::{attr, attr_nonempty, esc, instance_id};
use crate::parser::{ComponentItemNode, ComponentNode};

const THUMB: &str = "<div data-slot=\"segmented-control-thumb\" aria-hidden=\"true\"></div>";

pub fn render(comp: &ComponentNode) -> String {
    let name = instance_id(comp, "segmented-control");
    let buttons = options(comp)
        .iter()
        .enumerate()
        .map(|(i, (text, on, disabled))| {
            let (state, aria, checked) = if *on {
                ("active", "true", " checked")
            } else {
                ("inactive", "false", "")
            };
            let off = if *disabled { " disabled" } else { "" };
            let t = esc(text);
            format!(
                "<label><input type=\"radio\" name=\"{name}\" value=\"{i}\" aria-label=\"{t}\"{checked}{off}><button type=\"button\" data-slot=\"segmented-control-item\" data-state=\"{state}\" role=\"radio\" aria-checked=\"{aria}\" tabindex=\"-1\" aria-hidden=\"true\"{off}><span>{t}</span></button></label>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let aria = match attr_nonempty(comp, "aria-label") {
        Some(v) => format!(" aria-label=\"{}\"", esc(v)),
        None => String::new(),
    };
    let size = match crate::cronus_ui_kit::choice(comp, "size", &["sm", "md"]) {
        Some("sm") => "sm",
        _ => "md",
    };
    format!(
        "<div data-slot=\"segmented-control\" data-size=\"{size}\" role=\"radiogroup\"{aria}>{buttons}{THUMB}</div>"
    )
}

fn options(comp: &ComponentNode) -> Vec<(String, bool, bool)> {
    let items: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| {
            matches!(i.item_type.as_str(), "item" | "text" | "tab" | "columns")
                && !i.text.is_empty()
        })
        .collect();
    if items.is_empty() {
        let label = comp
            .items
            .iter()
            .find(|i| !i.text.is_empty())
            .map(|i| i.text.clone())
            .unwrap_or_else(|| {
                if comp.name.is_empty() {
                    "Option".into()
                } else {
                    comp.name.clone()
                }
            });
        return vec![(label, true, false)];
    }
    let on_idx = selected_idx(comp, &items);
    items
        .iter()
        .enumerate()
        .map(|(idx, i)| {
            (
                i.text.clone(),
                idx == on_idx,
                i.config
                    .get("disabled")
                    .is_some_and(|v| crate::cronus_ui_kit::truthy(v)),
            )
        })
        .collect()
}

fn selected_idx(comp: &ComponentNode, items: &[&ComponentItemNode]) -> usize {
    if let Some(v) = attr(comp, "value") {
        if let Some(idx) = items.iter().position(|i| i.text == v) {
            return idx;
        }
    }
    items
        .iter()
        .position(|i| {
            is_true(i.config.get("selected"))
                || is_true(i.config.get("pressed"))
                || is_true(i.config.get("on"))
                || is_true(i.config.get("checked"))
        })
        .unwrap_or(0)
}

fn is_true(raw: Option<&String>) -> bool {
    matches!(raw.map(String::as_str), Some("true" | "on" | "1"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use std::collections::HashMap;

    fn stub_options(opts: &[&str]) -> ComponentNode {
        ComponentNode {
            name: "View".into(),
            layout: Some("inline".into()),
            style: Some("segmented-control".into()),
            items: opts
                .iter()
                .map(|t| ComponentItemNode {
                    item_type: "item".into(),
                    text: (*t).into(),
                    link: None,
                    tone: None,
                    config: HashMap::new(),
                })
                .collect(),
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

    /// Item `i` of the group named `cui-{name}-segmented-control`.
    fn seg_in(name: &str, i: usize, text: &str, on: bool) -> String {
        let (state, aria, checked) = if on {
            ("active", "true", " checked")
        } else {
            ("inactive", "false", "")
        };
        format!(
            "<label><input type=\"radio\" name=\"cui-{name}-segmented-control\" value=\"{i}\" aria-label=\"{text}\"{checked}><button type=\"button\" data-slot=\"segmented-control-item\" data-state=\"{state}\" role=\"radio\" aria-checked=\"{aria}\" tabindex=\"-1\" aria-hidden=\"true\"><span>{text}</span></button></label>"
        )
    }

    fn seg(i: usize, text: &str, on: bool) -> String {
        seg_in("view", i, text, on)
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains(" disabled"));
        assert!(!crate::cli::stub_renderer_gate::looks_like_interact_generic(html));
        assert!(!html.contains("role=\"tablist\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("radios("));
    }

    #[test]
    fn root_is_radiogroup_of_segment_radios() {
        let html = render(&stub_options(&["Day", "Week"]));
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"segmented-control\" data-size=\"md\" role=\"radiogroup\">{}{}{THUMB}</div>",
                seg(0, "Day", true),
                seg(1, "Week", false)
            )
        );
        assert_eq!(html.matches(" checked").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn two_controls_on_one_page_get_distinct_names() {
        crate::cronus_ui_kit::reset_instance_ids();
        let a = render(&stub_options(&["Day", "Week"]));
        let b = render(&stub_options(&["Day", "Week"]));
        assert!(a.contains("name=\"cui-view-segmented-control\" "));
        assert!(b.contains("name=\"cui-view-segmented-control-2\" "));
    }

    /// The thumb is anchored to the checked option's label and its insets
    /// slide on the 0.4 s bounce spring; without anchor positioning the
    /// checked item paints the surface itself.
    #[test]
    fn chrome_thumb_follows_checked_radio() {
        const CSS: &str = include_str!("cronus_ui_css/segmented-control.css");
        assert!(CSS.contains("[data-slot=\"segmented-control\"] > label > input:checked + [data-slot=\"segmented-control-item\"] { color: var(--cronus-fg); }"));
        assert!(CSS.contains("[data-slot=\"segmented-control\"] > label > input:not(:checked) + [data-slot=\"segmented-control-item\"][data-state=\"active\"] { color: var(--cronus-fg-secondary); }"));
        assert!(CSS.contains("--cui-segmented-spring: 400ms linear("));
        assert!(CSS.contains("inset-block-start: anchor(top); inset-block-end: anchor(bottom);\n  inset-inline-start: anchor(start); inset-inline-end: anchor(end);\n  transition: inset var(--cui-segmented-spring);"));
        for i in 1..=8 {
            assert!(CSS.contains(&format!("[data-slot=\"segmented-control\"] > label:nth-child({i}) {{ anchor-name: --cui-segmented-{i}; }}")));
            assert!(CSS.contains(&format!("[data-slot=\"segmented-control\"]:has(> label:nth-child({i}) > input:checked) > [data-slot=\"segmented-control-thumb\"] {{ position-anchor: --cui-segmented-{i}; }}")));
        }
        assert!(CSS.contains("@supports not (anchor-name: --a)"));
        assert!(CSS.contains("[data-slot=\"segmented-control\"][data-size=\"sm\"] [data-slot=\"segmented-control-item\"] { padding: 0.25rem 0.625rem; font-size: 0.75rem; line-height: 1rem; }"));
    }

    /// Docs "Single select": three periods, `30 dias` selected by `value:`,
    /// the group named by `aria-label`; the thumb is the root's last child.
    #[test]
    fn docs_period_filter() {
        let mut c = stub_options(&["7 dias", "30 dias", "12 meses"]);
        c.props.insert("value".into(), "30 dias".into());
        c.props.insert("aria-label".into(), "Período".into());
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"segmented-control\" data-size=\"md\" role=\"radiogroup\" aria-label=\"Período\">{}{}{}{THUMB}</div>",
                seg(0, "7 dias", false),
                seg(1, "30 dias", true),
                seg(2, "12 meses", false)
            )
        );
        reject_interact(&html);
        c.props.insert("size".into(), "sm".into());
        assert!(render(&c).contains("data-size=\"sm\""));
        c.items[2].config.insert("disabled".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"12 meses\" disabled><button type=\"button\" data-slot=\"segmented-control-item\" data-state=\"inactive\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-hidden=\"true\" disabled>"));
    }

    /// Audit fixture: `label "Range"`, `text "Day"`, `text "Week"`, then
    /// `value:` / `aria-label:` attached to the last text. React:
    /// radiogroup labelled "Range" with Day (active) and Week — no "Range" radio.
    #[test]
    fn fixture_label_names_group_value_selects() {
        let mut c = stub("segmented-control", "Range");
        let text = |t: &str| ComponentItemNode {
            item_type: "text".into(),
            text: t.into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        };
        c.items.push(text("Day"));
        let mut week = text("Week");
        week.config.insert("value".into(), "Day".into());
        week.config.insert("aria-label".into(), "Range".into());
        c.items.push(week);
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"segmented-control\" data-size=\"md\" role=\"radiogroup\" aria-label=\"Range\">{}{}{THUMB}</div>",
                seg_in("segmented-control", 0, "Day", true),
                seg_in("segmented-control", 1, "Week", false)
            )
        );
        reject_interact(&html);
    }

    #[test]
    fn first_item_is_selected_by_default() {
        let html = render(&stub_options(&["Day", "Week", "Month"]));
        assert!(html.contains(&seg(0, "Day", true)));
        assert!(html.contains(&seg(1, "Week", false)));
        assert!(html.contains(&seg(2, "Month", false)));
        reject_interact(&html);
    }

    #[test]
    fn selected_true_item_is_active() {
        let mut c = stub_options(&["Day", "Week", "Month"]);
        c.items[1].config.insert("selected".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(&seg(0, "Day", false)));
        assert!(html.contains(&seg(1, "Week", true)));
        assert!(html.contains(&seg(2, "Month", false)));
        reject_interact(&html);
    }

    #[test]
    fn value_prop_selects_matching_item() {
        let mut c = stub_options(&["Day", "Week", "Month"]);
        c.props.insert("value".into(), "Month".into());
        let html = render(&c);
        assert!(html.contains(&seg(2, "Month", true)));
        assert!(html.contains(&seg(0, "Day", false)));
        reject_interact(&html);
    }

    #[test]
    fn options_from_items_and_escaped() {
        let html = render(&stub_options(&["A", "B", "<C>"]));
        assert_eq!(
            html.matches("data-slot=\"segmented-control-item\"").count(),
            3
        );
        assert!(html.contains("<span>A</span></button>"));
        assert!(html.contains("<span>&lt;C&gt;</span></button>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_native_radios() {
        let c = stub_options(&["Day", "Week"]);
        let html = render(&c);
        assert!(html.contains("data-slot=\"segmented-control-item\""));
        assert!(html.contains("role=\"radiogroup\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub_options(&["Day"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"segmented-control\"]"));
        assert!(css.contains("[data-slot=\"segmented-control-item\"]"));
        assert!(css.contains("[data-slot=\"segmented-control-item\"][data-state=\"active\"]"));
        assert!(css.contains("[data-slot=\"segmented-control-thumb\"]"));
        assert!(css.contains("[data-slot=\"segmented-control-item\"] > span"));
        assert!(css.contains("display: inline-flex"));
        assert!(css.contains("gap: 0.25rem"));
        // Wave 1s geometry parity (React measured: root 124x42, items 32px tall, 14px/20px).
        assert!(css.contains("position: relative; isolation: isolate; vertical-align: middle;"));
        assert!(css.contains(
            "font: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;"
        ));
        assert!(css.contains("position: relative; z-index: 1; display: inline-flex; align-items: center; gap: 0.375rem;"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
