//! Dedicated Metric renderer. DOM matches React:
//! `<div data-slot="metric"><div data-slot="metric-label">…</div><div data-slot="metric-value">…</div></div>`.
//! Label from label/title; value from item type value / extra text / props.value
//! (bound scalar as a last resort). Not interact `metric()` (`<section style=SURF>`
//! + v-data `{ value }` interp).

use crate::cronus_ui_kit::{esc, item};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = esc(label_raw(comp));
    let value = value_of(comp);
    format!(
        "<div data-slot=\"metric\"><div data-slot=\"metric-label\">{label}</div><div data-slot=\"metric-value\">{value}</div></div>"
    )
}

fn label_raw(comp: &ComponentNode) -> &str {
    for kind in ["label", "title"] {
        if let Some(t) = item(comp, kind) {
            if !t.is_empty() {
                return t;
            }
        }
    }
    &comp.name
}

fn value_of(comp: &ComponentNode) -> String {
    if let Some(t) = item(comp, "value") {
        if !t.is_empty() {
            return esc(t);
        }
    }
    if let Some(v) = comp.props.get("value") {
        if !v.is_empty() {
            return esc(v);
        }
    }
    // The tokenizer attaches `value:"1,240"` after `label "Users"` to the
    // label item's config, not to props.
    if let Some(v) = comp.items.iter().find_map(|i| i.config.get("value")) {
        if !v.is_empty() {
            return esc(v);
        }
    }
    let label = label_raw(comp);
    for i in &comp.items {
        if i.text.is_empty() {
            continue;
        }
        if matches!(i.item_type.as_str(), "label" | "title") {
            continue;
        }
        if i.text == label {
            continue;
        }
        return esc(&i.text);
    }
    if let Some(s) = crate::cronus_ui_data::scalar() {
        return esc(&s);
    }
    "0".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn extra(item_type: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: item_type.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<section"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("{ value }"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
    }

    #[test]
    fn root_is_div_with_label_and_value_not_section() {
        let html = render(&stub("metric", "Revenue"));
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"metric\""));
        assert!(html.contains("data-slot=\"metric-label\">Revenue</div>"));
        assert!(html.contains("data-slot=\"metric-value\">0</div>"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"metric\"><div data-slot=\"metric-label\">Revenue</div><div data-slot=\"metric-value\">0</div></div>"
        );
    }

    #[test]
    fn label_from_title_item() {
        let mut c = stub("metric", "");
        c.items[0].item_type = "title".into();
        c.items[0].text = "Active users".into();
        let html = render(&c);
        assert!(html.contains("data-slot=\"metric-label\">Active users</div>"));
        reject_interact(&html);
    }

    #[test]
    fn value_from_value_item() {
        let mut c = stub("metric", "MRR");
        c.items.push(extra("value", "$12.4k"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"metric-label\">MRR</div>"));
        assert!(html.contains("data-slot=\"metric-value\">$12.4k</div>"));
        reject_interact(&html);
    }

    #[test]
    fn value_from_extra_text() {
        let mut c = stub("metric", "MRR");
        c.items.push(extra("text", "12,400"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"metric-label\">MRR</div>"));
        assert!(html.contains("data-slot=\"metric-value\">12,400</div>"));
        reject_interact(&html);
    }

    #[test]
    fn value_from_props() {
        let mut c = stub("metric", "Churn");
        c.props.insert("value".into(), "2.1%".into());
        let html = render(&c);
        assert!(html.contains("data-slot=\"metric-value\">2.1%</div>"));
        reject_interact(&html);
    }

    #[test]
    fn value_from_label_item_config() {
        // Wave 1t: emitter writes `label "Users"` + `value:"1,240"` → item config.
        let mut c = stub("metric", "Users");
        c.items[0].config.insert("value".into(), "1,240".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"metric\"><div data-slot=\"metric-label\">Users</div><div data-slot=\"metric-value\">1,240</div></div>"
        );
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "font-size: 0.75rem; line-height: 1rem; font-weight: 500; text-transform: uppercase;"
        ));
        assert!(css.contains("font-size: 1.5rem; line-height: 2rem; font-weight: 600;"));
    }

    #[test]
    fn value_item_wins_over_extra_and_props() {
        let mut c = stub("metric", "MRR");
        c.props.insert("value".into(), "props".into());
        c.items.push(extra("text", "extra"));
        c.items.push(extra("value", "item"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"metric-value\">item</div>"));
        assert!(!html.contains(">props</div>"));
        assert!(!html.contains(">extra</div>"));
    }

    #[test]
    fn bound_scalar_when_no_value_sources() {
        use crate::binding::ResolvedData;
        crate::cronus_ui_data::with_binding("Lead", &ResolvedData::Count(12), || {
            let html = render(&stub("metric", "Leads"));
            assert!(html.contains("data-slot=\"metric-value\">12</div>"));
            reject_interact(&html);
        });
    }

    #[test]
    fn skips_interact_section() {
        let html = render(&stub("metric", "Revenue"));
        assert!(html.starts_with("<div data-slot=\"metric\">"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("metric", "Revenue"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"metric-value\">0</div>"));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"metric\"]"));
        assert!(css.contains("[data-slot=\"metric-label\"]"));
        assert!(css.contains("[data-slot=\"metric-value\"]"));
        assert!(css.contains("flex-direction: column"));
        assert!(css.contains("gap: 0.25rem"));
        assert!(css.contains("text-transform: uppercase"));
        assert!(css.contains("letter-spacing: 0.05em"));
        assert!(css.contains("var(--cronus-fg-tertiary)"));
        assert!(css.contains("var(--cronus-font-display"));
        assert!(css.contains("font-size: 1.5rem"));
        assert!(css.contains("font-variant-numeric: tabular-nums"));
        assert!(!css.contains("zinc-"));
    }
}
