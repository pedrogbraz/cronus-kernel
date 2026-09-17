//! Dedicated Banner renderer. DOM matches React:
//! a slotless collapse wrapper `<div>` (React's `motion.div`) around
//! `<section data-slot="banner" aria-label="Announcement">` holding the optional
//! leading lucide `<svg>` (`icon:`), `banner-content` (`banner-title` +
//! `banner-description` from a `description:"…"` attribute or extra texts), an
//! optional `banner-action` (`action` items rendered as `secondary` `sm`
//! buttons, links when they carry `-> "/href"`) and, unless `dismissible:false`,
//! the `banner-dismiss` close button.
//! Variants (`style:banner+brand`, or `variant:`) are React's `brand`, `info`,
//! `success`, `warning`, `error`; `align:start` moves the message to the start.
//! The root has no data attribute for them in React, so they are `v-…` /
//! `align-start` classes on the slot element.
//!
//! Dismissal is native: the close button sits in a `<label>` with a visually
//! hidden checkbox ("Dismiss"); CSS collapses the wrapper (grid rows 1fr → 0fr
//! + fade, React's 200ms collapse) when it is checked. A `restore:"…"` prop adds
//! an outline `sm` button-styled `<label for>` the same checkbox after the
//! wrapper, shown only while dismissed, that brings the banner back.
//! Not the interact `alert("banner")` SURF box.

use crate::cronus_ui_kit::{attr_nonempty, choice, esc, item_icon, item_icon_end, widget_id};
use crate::parser::{ComponentItemNode, ComponentNode};

const VARIANTS: &[&str] = &[
    "default",
    "brand",
    "info",
    "success",
    "warning",
    "error",
    "destructive",
];

pub fn render(comp: &ComponentNode) -> String {
    let aria = aria_label(comp).unwrap_or("Announcement");
    let variant = variant_of(comp);
    let mut classes: Vec<String> = Vec::new();
    if variant != "default" {
        classes.push(format!("v-{variant}"));
    }
    if choice(comp, "align", &["start", "center"]) == Some("start") {
        classes.push("align-start".into());
    }
    let class = if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    };
    let icon = attr_nonempty(comp, "icon")
        .map(crate::cronus_ui_icons::svg_or_empty)
        .unwrap_or_default();
    let (title, mut descs) = title_and_descriptions(comp);
    if let Some(d) = attr_nonempty(comp, "description") {
        descs.insert(0, esc(d));
    }
    let mut content = format!("<span data-slot=\"banner-title\">{title}</span>");
    for d in descs {
        content.push_str(&format!(
            "<span data-slot=\"banner-description\">{d}</span>"
        ));
    }
    let actions: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "action" && !i.text.is_empty())
        .map(|i| {
            let inner = format!("{}{}{}", item_icon(i), esc(&i.text), item_icon_end(i));
            crate::cronus_ui::button_html(&inner, "secondary", "sm", i.link.as_deref(), false, None)
        })
        .collect();
    let action = if actions.is_empty() {
        String::new()
    } else {
        format!(
            "<div data-slot=\"banner-action\">{}</div>",
            actions.join("")
        )
    };
    let dismissible = !matches!(comp.props.get("dismissible").map(String::as_str), Some(v) if !crate::cronus_ui_kit::truthy(v));
    let dismiss_label = attr_nonempty(comp, "dismiss-label")
        .map(esc)
        .unwrap_or_else(|| "Dismiss".into());
    let id = widget_id(comp, "dismiss");
    let dismiss = if dismissible {
        format!(
            "<label><input type=\"checkbox\" id=\"{id}\" aria-label=\"{dismiss_label}\"><button type=\"button\" data-slot=\"banner-dismiss\" data-variant=\"ghost\" aria-label=\"{dismiss_label}\" tabindex=\"-1\" aria-hidden=\"true\">{}</button></label>",
            crate::cronus_ui_icons::svg_or_empty("x")
        )
    } else {
        String::new()
    };
    let restore = match attr_nonempty(comp, "restore") {
        Some(r) if dismissible => format!(
            "<label for=\"{id}\" data-slot=\"button\" data-variant=\"outline\" data-size=\"sm\" class=\"cui-btn\">{}</label>",
            esc(r)
        ),
        _ => String::new(),
    };
    format!(
        "<div><section data-slot=\"banner\" aria-label=\"{}\"{class}>{icon}<div data-slot=\"banner-content\">{content}</div>{action}{dismiss}</section></div>{restore}",
        esc(aria)
    )
}

/// `variant:` prop / item config, else the style segment; `destructive` is
/// accepted as an alias of `error`.
fn variant_of(comp: &ComponentNode) -> &'static str {
    match choice(comp, "variant", VARIANTS) {
        Some("brand") => "brand",
        Some("info") => "info",
        Some("success") => "success",
        Some("warning") => "warning",
        Some("error") | Some("destructive") => "error",
        _ => "default",
    }
}

fn aria_label(comp: &ComponentNode) -> Option<&str> {
    attr_nonempty(comp, "aria-label").or_else(|| {
        comp.props
            .get("label")
            .map(String::as_str)
            .filter(|s| !s.is_empty())
    })
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
        .filter(|(i, item)| *i != skip && !item.text.is_empty() && item.item_type != "action")
        .map(|(_, item)| esc(&item.text))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn stub(style: &str, label: &str) -> ComponentNode {
        let mut c = ComponentNode {
            name: "Banner".into(),
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
        };
        // The audit fixture renders `dismissible={false}`.
        c.props.insert("dismissible".into(), "false".into());
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("motion.div"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("role=\"status\""));
    }

    #[test]
    fn root_is_section_with_title_not_surf_box() {
        let html = render(&stub("banner", "New pricing"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div><section data-slot=\"banner\" aria-label=\"Announcement\"><div data-slot=\"banner-content\"><span data-slot=\"banner-title\">New pricing</span></div></section></div>"
        );
    }

    #[test]
    fn description_attribute_and_extra_text() {
        let mut c = stub("banner", "New billing");
        c.items[0]
            .config
            .insert("description".into(), "Usage-based".into());
        assert!(render(&c).contains(
            "<span data-slot=\"banner-title\">New billing</span><span data-slot=\"banner-description\">Usage-based</span>"
        ));
        let mut t = stub("banner", "New billing");
        t.items.push(ComponentItemNode {
            item_type: "text".into(),
            text: "Usage-based invoices are live.".into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        });
        let html = render(&t);
        assert!(html.contains("data-slot=\"banner-description\">Usage-based invoices are live.<"));
        reject_interact(&html);
    }

    #[test]
    fn aria_label_from_props_or_item_config() {
        let mut c = stub("banner", "Sale");
        c.props.insert("aria-label".into(), "Promo".into());
        assert!(render(&c).contains("aria-label=\"Promo\""));
        let mut i = stub("banner", "Sale");
        i.items[0]
            .config
            .insert("aria-label".into(), "Promo".into());
        assert!(render(&i).contains("aria-label=\"Promo\""));
    }

    #[test]
    fn skips_interact_surf_box() {
        let html = render(&stub("banner", "New billing"));
        assert!(html.starts_with("<div><section "));
    }

    #[test]
    fn variant_icon_and_action_match_react_promo() {
        let mut c = stub("banner+brand", "Cronus UI 1.0 is here.");
        c.props.remove("dismissible");
        c.props.insert("icon".into(), "sparkles".into());
        c.props
            .insert("description".into(), "60+ themeable components.".into());
        let mut action = ComponentItemNode {
            item_type: "action".into(),
            text: "Read the announcement".into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        };
        action
            .config
            .insert("icon-end".into(), "arrow-right".into());
        c.items.push(action);
        let html = render(&c);
        assert!(html.contains(
            "<section data-slot=\"banner\" aria-label=\"Announcement\" class=\"v-brand\"><svg "
        ));
        assert!(html.contains("data-icon=\"sparkles\""));
        assert!(html.contains("<div data-slot=\"banner-action\"><button type=\"button\" data-slot=\"button\" data-variant=\"secondary\" data-size=\"sm\" class=\"cui-btn\">Read the announcement<svg "));
        assert!(html.contains("data-icon=\"arrow-right\""));
        // The action is not a description.
        assert!(!html.contains("banner-description\">Read the announcement"));
        // Dismissible by default: a native checkbox in a label wraps React's close button.
        let id = widget_id(&c, "dismiss");
        assert!(html.contains(&format!("<label><input type=\"checkbox\" id=\"{id}\" aria-label=\"Dismiss\"><button type=\"button\" data-slot=\"banner-dismiss\" data-variant=\"ghost\" aria-label=\"Dismiss\" tabindex=\"-1\" aria-hidden=\"true\"><svg ")));
        assert!(html.contains("data-icon=\"x\""));
        assert!(html.ends_with("</svg></button></label></section></div>"));
        reject_interact(&html);
    }

    #[test]
    fn restore_label_targets_the_dismiss_checkbox() {
        let mut c = stub("banner+info", "Scheduled maintenance");
        c.props.remove("dismissible");
        c.props.insert("restore".into(), "Show banner again".into());
        let html = render(&c);
        let id = widget_id(&c, "dismiss");
        assert!(html.ends_with(&format!(
            "</section></div><label for=\"{id}\" data-slot=\"button\" data-variant=\"outline\" data-size=\"sm\" class=\"cui-btn\">Show banner again</label>"
        )));
        assert!(html.contains("class=\"v-info\""));
        // Not dismissible: no checkbox, no restore.
        c.props.insert("dismissible".into(), "false".into());
        let html = render(&c);
        assert!(!html.contains("checkbox"));
        assert!(!html.contains("Show banner again"));
    }

    #[test]
    fn chrome_is_token_only_and_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(
            css.contains("display: flex; width: 100%; align-items: center; gap: 0.25rem 0.75rem;")
        );
        assert!(css.contains("padding: 0.625rem 1rem; font-size: 0.875rem; line-height: 1.25rem; text-align: center;"));
        assert!(css.contains("border-bottom: 1px solid var(--cronus-border)"));
        assert!(css.contains("align-items: center; gap: 0.125rem 0.5rem;"));
        assert!(css.contains("[data-slot=\"banner\"].v-brand {\n  background: var(--cronus-primary); color: var(--cronus-primary-foreground); border-color: transparent;"));
        assert!(css.contains("[data-slot=\"banner\"].v-brand > [data-slot=\"banner-content\"] > [data-slot=\"banner-description\"] {\n  color: color-mix(in oklab, var(--cronus-primary-foreground) 85%, transparent);"));
        assert!(css.contains("[data-slot=\"banner-dismiss\"] {\n  display: inline-flex; align-items: center; justify-content: center;\n  width: 2rem; height: 2rem;"));
        // Native dismissal: the wrapper collapses when the checkbox is checked.
        assert!(css.contains("div:has(> [data-slot=\"banner\"] > label > input:checked) {\n  grid-template-rows: 0fr; opacity: 0;"));
        assert!(css.contains("transition: grid-template-rows 200ms var(--ease-out-quart), opacity 200ms var(--ease-out-quart);"));
        assert!(!css.contains("zinc-"));
    }
}
