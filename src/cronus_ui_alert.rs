//! Dedicated Alert renderer. DOM matches React:
//! `<div data-slot="alert" role="status">` (role=`alert` only if destructive)
//! with an optional leading lucide `<svg>` (`icon:` prop), then
//! `<div data-slot="alert-title">` and optional `alert-description`
//! (a `description:"…"` attribute first, then extra texts).
//! Variants (`style:alert+info`, or `variant:`) are React's `info`, `success`,
//! `warning`, `destructive`; the root has no data attribute for them in React,
//! so the kernel puts a `v-<variant>` class on the slot element.
//! Not the interact SURF box wrapping raw `<div>text</div>` with no title slot.

use crate::cronus_ui_kit::{attr_nonempty, choice, esc};
use crate::parser::{ComponentItemNode, ComponentNode};

const VARIANTS: &[&str] = &[
    "default",
    "info",
    "success",
    "warning",
    "destructive",
    "danger",
];

pub fn render(comp: &ComponentNode) -> String {
    let variant = variant_of(comp);
    let role = if variant == "destructive" {
        "alert"
    } else {
        "status"
    };
    let class = if variant == "default" {
        String::new()
    } else {
        format!(" class=\"v-{variant}\"")
    };
    let icon = attr_nonempty(comp, "icon")
        .map(crate::cronus_ui_icons::svg_or_empty)
        .unwrap_or_default();
    let (title, mut descs) = title_and_descriptions(comp);
    if let Some(d) = attr_nonempty(comp, "description") {
        descs.insert(0, esc(d));
    }
    let mut inner = format!("{icon}<div data-slot=\"alert-title\">{title}</div>");
    for d in descs {
        inner.push_str(&format!("<div data-slot=\"alert-description\">{d}</div>"));
    }
    format!("<div data-slot=\"alert\" role=\"{role}\"{class}>{inner}</div>")
}

/// `variant:` prop / item config, else the style segment; `danger` is the
/// legacy alias of `destructive`.
fn variant_of(comp: &ComponentNode) -> &'static str {
    match choice(comp, "variant", VARIANTS) {
        Some("info") => "info",
        Some("success") => "success",
        Some("warning") => "warning",
        Some("destructive") | Some("danger") => "destructive",
        _ => "default",
    }
}

fn title_and_descriptions(comp: &ComponentNode) -> (String, Vec<String>) {
    for kind in ["label", "title", "text", "value"] {
        if let Some((idx, t)) = comp
            .items
            .iter()
            .enumerate()
            .find(|(_, i)| i.item_type == kind && !i.text.is_empty())
        {
            return (esc(&t.text), extra_descs(comp, idx));
        }
    }
    (esc(&comp.name), extra_descs(comp, usize::MAX))
}

fn extra_descs(comp: &ComponentNode, skip: usize) -> Vec<String> {
    comp.items
        .iter()
        .enumerate()
        .filter(|(i, item)| *i != skip && !item.text.is_empty())
        .map(|(_, item)| esc(&item.text))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn stub(style: &str, label: &str) -> ComponentNode {
        ComponentNode {
            name: "Alert".into(),
            layout: Some("stack".into()),
            style: Some(style.into()),
            items: vec![ComponentItemNode {
                item_type: "label".into(),
                text: label.into(),
                link: None,
                tone: None,
                config: HashMap::new(),
            }],
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

    fn reject_interact(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("padding:0.85rem 1rem"));
        assert!(!html.contains("<div>Demo</div>"));
        assert!(html.contains("data-slot=\"alert-title\""));
        assert!(!html.contains("v-data="));
    }

    #[test]
    fn root_is_div_with_title_slot_not_surf_box() {
        let html = render(&stub("alert", "Saved"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"alert\" role=\"status\"><div data-slot=\"alert-title\">Saved</div></div>"
        );
    }

    #[test]
    fn description_attribute_matches_react() {
        let mut c = stub("alert", "Heads up");
        c.items[0]
            .config
            .insert("description".into(), "Your trial ends soon.".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"alert\" role=\"status\"><div data-slot=\"alert-title\">Heads up</div><div data-slot=\"alert-description\">Your trial ends soon.</div></div>"
        );
        reject_interact(&html);
    }

    #[test]
    fn extra_text_is_description() {
        let mut c = stub("alert", "Payment failed");
        c.items.push(ComponentItemNode {
            item_type: "text".into(),
            text: "Your card was declined.".into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        });
        let html = render(&c);
        assert!(html.contains("data-slot=\"alert-title\">Payment failed<"));
        assert!(html.contains("data-slot=\"alert-description\">Your card was declined.<"));
        reject_interact(&html);
    }

    #[test]
    fn destructive_sets_role_alert() {
        let html = render(&stub("alert+destructive", "Outage"));
        assert!(html.contains("role=\"alert\" class=\"v-destructive\""));
        assert!(!html.contains("role=\"status\""));
        reject_interact(&html);
    }

    #[test]
    fn destructive_from_props() {
        let mut c = stub("alert", "Outage");
        c.props.insert("variant".into(), "destructive".into());
        assert!(render(&c).contains("role=\"alert\""));
        c.props.insert("variant".into(), "danger".into());
        assert!(render(&c).contains("role=\"alert\" class=\"v-destructive\""));
    }

    #[test]
    fn semantic_variants_are_classes_on_the_slot() {
        for v in ["info", "success", "warning"] {
            let html = render(&stub(&format!("alert+{v}"), "Note"));
            assert!(
                html.contains(&format!("role=\"status\" class=\"v-{v}\"")),
                "{html}"
            );
        }
        // React has no data attribute for the variant.
        assert!(!render(&stub("alert+info", "Note")).contains("data-variant"));
    }

    #[test]
    fn icon_prop_renders_leading_lucide_glyph() {
        let mut c = stub("alert+info", "New version available");
        c.props.insert("icon".into(), "info".into());
        c.props
            .insert("description".into(), "A new release is ready.".into());
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"alert\" role=\"status\" class=\"v-info\"><svg "));
        assert!(html.contains("data-icon=\"info\""));
        assert!(html.contains("</svg><div data-slot=\"alert-title\">New version available</div><div data-slot=\"alert-description\">A new release is ready.</div></div>"));
        let mut missing = stub("alert", "Note");
        missing.props.insert("icon".into(), "not-an-icon".into());
        assert!(!render(&missing).contains("<svg"));
    }

    #[test]
    fn chrome_is_token_only_and_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("grid-template-columns: 0 1fr; align-items: start; row-gap: 0.25rem;"));
        assert!(css.contains("padding: 0.75rem 1rem; font-size: 0.875rem; line-height: 1.25rem;"));
        assert!(css.contains("border-radius: var(--cronus-radius-lg)"));
        assert!(css
            .contains("[data-slot=\"alert-title\"] {\n  grid-column-start: 2; min-height: 1rem;"));
        assert!(css.contains("[data-slot=\"alert-description\"] {\n  grid-column-start: 2; display: grid; justify-items: start; gap: 0.25rem;"));
        assert!(css.contains("var(--cronus-surface-overlay)"));
        // Icon column (`has-[>svg]:grid-cols-[--spacing(6)_1fr] has-[>svg]:gap-x-3`) and tints.
        assert!(css.contains("[data-slot=\"alert\"]:has(> svg) {\n  grid-template-columns: 1.5rem 1fr; column-gap: 0.75rem;"));
        assert!(css.contains("[data-slot=\"alert\"] > svg {\n  width: 1.25rem; height: 1.25rem; flex-shrink: 0; translate: 0 0.125rem;"));
        assert!(css.contains("[data-slot=\"alert\"].v-info {\n  background: color-mix(in oklab, var(--cronus-info) 10%, transparent);"));
        assert!(
            css.contains("[data-slot=\"alert\"].v-success > svg { color: var(--cronus-success); }")
        );
        assert!(
            css.contains("[data-slot=\"alert\"].v-warning > svg { color: var(--cronus-warning); }")
        );
        assert!(css
            .contains("[data-slot=\"alert\"].v-destructive > svg { color: var(--cronus-error); }"));
        assert!(!css.contains("zinc-"));
    }
}
