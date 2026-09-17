//! Dedicated Chip renderer. DOM matches React `Chip`: a static
//! `<span data-slot="chip">` (no `data-size`, the audit checks that); the
//! `chipVariants` recipe — `variant` solid|soft|outline, `color`
//! neutral|primary|success|warning|error|info, `size` sm|md|lg — travels as
//! classes (`v-soft c-primary s-md`) from the style (`chip+solid+primary+sm`)
//! or props. `icon:` puts a lucide glyph in `chip-icon`; `removable:true` adds
//! the `chip-remove` × affordance (JS-only removal: the same native button,
//! `disabled`, idle look).
//!
//! Several `item`s render a `<div data-slot="chip-group">` of chips (React
//! `ChipGroup`, `aria-label` from the label). A chip with `selected:` (true
//! or false) is React's filter chip: a `<button aria-pressed>` with a leading
//! `chip-check` while on. Zero JS: that button is decorative inside a `<label>`
//! after a visually hidden checkbox that carries the pressed state; CSS shows
//! the check and the selected emphasis after `:checked`.

use crate::cronus_ui_kit::{attr_nonempty, esc, flag, instance_id, own_choice, truthy};
use crate::parser::{ComponentItemNode, ComponentNode};

const CHECK: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" data-slot=\"chip-check\"><path d=\"M20 6 9 17l-5-5\"></path></svg>";
const X: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M18 6 6 18\"></path><path d=\"m6 6 12 12\"></path></svg>";

const VARIANTS: &[&str] = &["solid", "soft", "outline"];
const COLORS: &[&str] = &["neutral", "primary", "success", "warning", "error", "info"];
const SIZES: &[&str] = &["sm", "md", "lg"];

struct Look {
    variant: String,
    color: String,
    size: String,
}

fn look(comp: &ComponentNode) -> Look {
    Look {
        variant: own_choice(comp, "variant", VARIANTS)
            .unwrap_or("soft")
            .into(),
        color: own_choice(comp, "color", COLORS)
            .unwrap_or("neutral")
            .into(),
        size: own_choice(comp, "size", SIZES).unwrap_or("md").into(),
    }
}

pub fn render(comp: &ComponentNode) -> String {
    let base = look(comp);
    let items: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty())
        .collect();
    if items.is_empty() {
        let text = label_of(comp);
        let selected = attr_nonempty(comp, "selected").map(truthy);
        let icon = attr_nonempty(comp, "icon");
        return chip(
            comp,
            &text,
            &base,
            selected,
            icon,
            flag(comp, "removable"),
            flag(comp, "disabled"),
        );
    }
    let name = attr_nonempty(comp, "aria-label")
        .map(esc)
        .unwrap_or_else(|| label_of(comp));
    let chips: String = items
        .iter()
        .map(|i| {
            let per = Look {
                variant: i
                    .config
                    .get("variant")
                    .filter(|v| VARIANTS.contains(&v.as_str()))
                    .cloned()
                    .unwrap_or_else(|| base.variant.clone()),
                color: i
                    .config
                    .get("color")
                    .filter(|v| COLORS.contains(&v.as_str()))
                    .cloned()
                    .unwrap_or_else(|| base.color.clone()),
                size: i
                    .config
                    .get("size")
                    .filter(|v| SIZES.contains(&v.as_str()))
                    .cloned()
                    .unwrap_or_else(|| base.size.clone()),
            };
            let selected = i
                .config
                .get("selected")
                .map(|v| truthy(v))
                .or_else(|| attr_nonempty(comp, "selected").map(truthy));
            chip(
                comp,
                &esc(&i.text),
                &per,
                selected,
                i.config.get("icon").map(String::as_str),
                flag(comp, "removable") || i.config.get("removable").is_some_and(|v| truthy(v)),
                i.config.get("disabled").is_some_and(|v| truthy(v)),
            )
        })
        .collect();
    format!("<div data-slot=\"chip-group\" role=\"group\" aria-label=\"{name}\">{chips}</div>")
}

fn chip(
    comp: &ComponentNode,
    text: &str,
    look: &Look,
    selected: Option<bool>,
    icon: Option<&str>,
    removable: bool,
    disabled: bool,
) -> String {
    let mut class = format!("v-{} c-{} s-{}", look.variant, look.color, look.size);
    let icon_html = icon
        .and_then(crate::cronus_ui_icons::svg)
        .map(|g| format!("<span data-slot=\"chip-icon\" aria-hidden=\"true\">{g}</span>"))
        .unwrap_or_default();
    let dis = if disabled { " data-disabled=\"\"" } else { "" };
    match selected {
        None => {
            let remove = if removable {
                format!("<button type=\"button\" data-slot=\"chip-remove\" aria-label=\"Remove\" disabled>{X}</button>")
            } else {
                String::new()
            };
            format!(
                "<span data-slot=\"chip\" class=\"{class}\"{dis}>{icon_html}{text}{remove}</span>"
            )
        }
        Some(on) => {
            class.push_str(" interactive");
            if on {
                class.push_str(" selected");
            }
            let pressed = if on { "true" } else { "false" };
            let sel = if on { " data-selected=\"\"" } else { "" };
            let checked = if on { " checked" } else { "" };
            let input_dis = if disabled { " disabled" } else { "" };
            let remove = if removable {
                format!("<span data-slot=\"chip-remove\" aria-hidden=\"true\">{X}</span>")
            } else {
                String::new()
            };
            let id = instance_id(comp, "chip");
            format!(
                "<label data-control=\"chip\"><input type=\"checkbox\" id=\"{id}\" aria-label=\"{text}\"{checked}{input_dis}><button type=\"button\" data-slot=\"chip\" class=\"{class}\"{sel}{dis} aria-pressed=\"{pressed}\" tabindex=\"-1\" aria-hidden=\"true\">{CHECK}{icon_html}{text}{remove}</button></label>"
            )
        }
    }
}

fn label_of(comp: &ComponentNode) -> String {
    for kind in ["label", "title", "text", "value"] {
        if let Some(t) = item(comp, kind) {
            if !t.is_empty() {
                return esc(t);
            }
        }
    }
    esc(&comp.name)
}

fn item<'a>(comp: &'a ComponentNode, kind: &str) -> Option<&'a str> {
    comp.items
        .iter()
        .find(|i| i.item_type == kind)
        .map(|i| i.text.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::{reset_instance_ids, stub};
    use std::collections::HashMap;

    #[test]
    fn static_chip_is_span_with_recipe_classes() {
        let html = render(&stub("chip", "Beta"));
        assert_eq!(
            html,
            "<span data-slot=\"chip\" class=\"v-soft c-neutral s-md\">Beta</span>"
        );
        assert!(!html.contains("data-size"));
        assert!(!html.contains("chip-control"));
    }

    #[test]
    fn style_segments_pick_variant_color_size() {
        let mut c = stub("chip", "Error");
        c.style = Some("chip+solid+error+sm".into());
        assert!(render(&c).contains("class=\"v-solid c-error s-sm\""));
        c.props.insert("removable".into(), "true".into());
        c.props.insert("icon".into(), "star".into());
        let html = render(&c);
        assert!(html.contains("<span data-slot=\"chip-icon\" aria-hidden=\"true\"><svg"));
        assert!(html.contains("data-slot=\"chip-remove\" aria-label=\"Remove\" disabled>"));
    }

    #[test]
    fn items_render_a_group_and_selected_is_a_checkbox_button() {
        reset_instance_ids();
        let mut c = stub("chip", "Filter articles by topic");
        for (t, sel) in [("Design", "true"), ("React", "false")] {
            let mut config = HashMap::new();
            config.insert("selected".to_string(), sel.to_string());
            c.items.push(ComponentItemNode {
                item_type: "item".into(),
                text: t.into(),
                link: None,
                tone: None,
                config,
            });
        }
        let html = render(&c);
        assert!(html.starts_with(
            "<div data-slot=\"chip-group\" role=\"group\" aria-label=\"Filter articles by topic\">"
        ));
        assert!(html.contains("<label data-control=\"chip\"><input type=\"checkbox\" id=\"cui-chip-chip\" aria-label=\"Design\" checked><button type=\"button\" data-slot=\"chip\" class=\"v-soft c-neutral s-md interactive selected\" data-selected=\"\" aria-pressed=\"true\""));
        assert!(html.contains("aria-label=\"React\"><button type=\"button\" data-slot=\"chip\" class=\"v-soft c-neutral s-md interactive\" aria-pressed=\"false\""));
        assert_eq!(html.matches("data-slot=\"chip-check\"").count(), 2);
        assert!(!html.contains("style="));
    }

    #[test]
    fn item_looks_do_not_leak_into_the_group() {
        let mut c = stub("chip", "Chip variants");
        let mut config = HashMap::new();
        config.insert("size".to_string(), "sm".to_string());
        config.insert("color".to_string(), "primary".to_string());
        c.items.push(ComponentItemNode {
            item_type: "item".into(),
            text: "Small".into(),
            link: None,
            tone: None,
            config,
        });
        c.items.push(ComponentItemNode {
            item_type: "item".into(),
            text: "Plain".into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        });
        let html = render(&c);
        assert!(html.contains("class=\"v-soft c-primary s-sm\">Small"));
        assert!(html.contains("class=\"v-soft c-neutral s-md\">Plain"));
    }

    #[test]
    fn registered_as_dedicated() {
        use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
        assert_eq!(dedicated_fn_name("chip"), Some("cronus_ui_chip::render"));
        assert_eq!(
            renderer_kind("chip"),
            RendererKind::Dedicated("cronus_ui_chip::render")
        );
        let c = stub("chip", "Beta");
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), render(&c));
    }
}
