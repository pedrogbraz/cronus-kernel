//! Dedicated SegmentedControl renderer. DOM mirrors React:
//! `<div data-slot="segmented-control" data-size="md" role="radiogroup">` plus
//! per option `<label>` > visually hidden `<input type="radio">` + React's
//! `<button type="button" data-slot="segmented-control-item" role="radio">`
//! holding the `segmented-control-thumb`.
//! Options are `item`/`text` lines; the `label` names the group.
//!
//! Zero JS: the radios share a page-unique `name`; clicking an option's label
//! checks it and Arrow keys move the selection. CSS shows the thumb and the
//! active text colour on the item after the checked radio (every item carries a
//! thumb; unchecked ones are `display: none`, so only one renders, like React).
//! The button keeps React's slot and look but is decorative (`aria-hidden`,
//! `tabindex="-1"`, `pointer-events: none`). Gaps vs React: the thumb jumps
//! instead of sliding, and `data-state` / `aria-checked` stay at the initial
//! state (the native radio carries the live state).
//! Not interact `radios()` (inline-styled `<input type="radio">` labels).

use crate::cronus_ui_kit::{attr, attr_nonempty, esc, instance_id};
use crate::parser::{ComponentItemNode, ComponentNode};

const THUMB: &str = "<div data-slot=\"segmented-control-thumb\" aria-hidden=\"true\"></div>";

pub fn render(comp: &ComponentNode) -> String {
    let name = instance_id(comp, "segmented-control");
    let buttons = options(comp)
        .iter()
        .enumerate()
        .map(|(i, (text, on))| {
            let (state, aria, checked) = if *on {
                ("active", "true", " checked")
            } else {
                ("inactive", "false", "")
            };
            let t = esc(text);
            format!(
                "<label><input type=\"radio\" name=\"{name}\" value=\"{i}\" aria-label=\"{t}\"{checked}><button type=\"button\" data-slot=\"segmented-control-item\" data-state=\"{state}\" role=\"radio\" aria-checked=\"{aria}\" tabindex=\"-1\" aria-hidden=\"true\">{THUMB}<span>{t}</span></button></label>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let aria = match attr_nonempty(comp, "aria-label") {
        Some(v) => format!(" aria-label=\"{}\"", esc(v)),
        None => String::new(),
    };
    format!(
        "<div data-slot=\"segmented-control\" data-size=\"md\" role=\"radiogroup\"{aria}>{buttons}</div>"
    )
}

fn options(comp: &ComponentNode) -> Vec<(String, bool)> {
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
        return vec![(label, true)];
    }
    let on_idx = selected_idx(comp, &items);
    items
        .iter()
        .enumerate()
        .map(|(idx, i)| (i.text.clone(), idx == on_idx))
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
            "<label><input type=\"radio\" name=\"cui-{name}-segmented-control\" value=\"{i}\" aria-label=\"{text}\"{checked}><button type=\"button\" data-slot=\"segmented-control-item\" data-state=\"{state}\" role=\"radio\" aria-checked=\"{aria}\" tabindex=\"-1\" aria-hidden=\"true\">{THUMB}<span>{text}</span></button></label>"
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
                "<div data-slot=\"segmented-control\" data-size=\"md\" role=\"radiogroup\">{}{}</div>",
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

    /// Only the checked item shows its thumb and the active colour.
    #[test]
    fn chrome_thumb_follows_checked_radio() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"segmented-control\"] > label > input:not(:checked) + [data-slot=\"segmented-control-item\"] > [data-slot=\"segmented-control-thumb\"] { display: none; }"));
        assert!(css.contains("[data-slot=\"segmented-control\"] > label > input:checked + [data-slot=\"segmented-control-item\"] { color: var(--cronus-fg); }"));
        assert!(css.contains("[data-slot=\"segmented-control\"] > label > input:not(:checked) + [data-slot=\"segmented-control-item\"][data-state=\"active\"] { color: var(--cronus-fg-secondary); }"));
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
                "<div data-slot=\"segmented-control\" data-size=\"md\" role=\"radiogroup\" aria-label=\"Range\">{}{}</div>",
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
