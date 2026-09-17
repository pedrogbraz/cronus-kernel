//! Dedicated ButtonGroup renderer. DOM matches React:
//! `<div data-slot="button-group" role="group" data-orientation aria-label>`
//! plus child `<button type="button" data-slot="button" data-variant="primary">`
//! per item. The `label` names the group (it is never a button).
//! Not interact `buttonish()` (wrapper with a SINGLE primary button and inline styles).

use crate::cronus_ui_kit::{attr_nonempty, choice, esc, item_icon, item_icon_end, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let orientation = orientation_of(comp);
    let variant = choice(
        comp,
        "variant",
        &[
            "primary",
            "secondary",
            "outline",
            "ghost",
            "destructive",
            "link",
        ],
    )
    .unwrap_or("primary");
    let size = choice(comp, "size", &["sm", "md", "lg", "icon", "icon-sm"]).unwrap_or("md");
    let buttons = buttons_of(comp)
        .into_iter()
        .map(|(text, icon, icon_end, disabled)| {
            let dis = if disabled { " disabled" } else { "" };
            format!(
                "<button type=\"button\" data-slot=\"button\" data-variant=\"{variant}\" data-size=\"{size}\"{dis}>{icon}{text}{icon_end}</button>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let aria = attr_nonempty(comp, "aria-label")
        .map(|a| format!(" aria-label=\"{}\"", esc(a)))
        .unwrap_or_default();
    format!(
        "<div data-slot=\"button-group\" role=\"group\" data-orientation=\"{orientation}\"{aria}>{buttons}</div>"
    )
}

/// (label, leading glyph, trailing glyph, disabled) per button. Choice items
/// (`item` / `tab`) win; else every non-label text; else the component label.
fn buttons_of(comp: &ComponentNode) -> Vec<(String, String, String, bool)> {
    let choices: Vec<&crate::parser::ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "item" | "tab" | "button") && !i.text.is_empty())
        .collect();
    let body: Vec<&crate::parser::ComponentItemNode> = if !choices.is_empty() {
        choices
    } else {
        comp.items
            .iter()
            .filter(|i| !matches!(i.item_type.as_str(), "label" | "title") && !i.text.is_empty())
            .collect()
    };
    if body.is_empty() {
        return vec![(label_of(comp), String::new(), String::new(), false)];
    }
    body.into_iter()
        .map(|i| {
            (
                esc(&i.text),
                item_icon(i),
                item_icon_end(i),
                i.config
                    .get("disabled")
                    .is_some_and(|v| crate::cronus_ui_kit::truthy(v)),
            )
        })
        .collect()
}

fn orientation_of(comp: &ComponentNode) -> &'static str {
    if let Some(v) = comp.props.get("orientation") {
        if v == "vertical" {
            return "vertical";
        }
        if v == "horizontal" {
            return "horizontal";
        }
    }
    if comp
        .style
        .as_deref()
        .unwrap_or("")
        .split('+')
        .any(|part| part.trim() == "vertical")
    {
        return "vertical";
    }
    "horizontal"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn group(label: &str, items: &[&str]) -> ComponentNode {
        let mut c = stub("button-group", label);
        for n in items {
            c.items.push(extra("text", n));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("display:inline-flex;gap:0.25rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
    }

    #[test]
    fn emitted_fixture_label_is_aria_not_a_button() {
        let mut c = group("Actions", &["Save", "Cancel"]);
        c.items[2]
            .config
            .insert("aria-label".into(), "Actions".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"button-group\" role=\"group\" data-orientation=\"horizontal\" aria-label=\"Actions\"><button type=\"button\" data-slot=\"button\" data-variant=\"primary\" data-size=\"md\">Save</button><button type=\"button\" data-slot=\"button\" data-variant=\"primary\" data-size=\"md\">Cancel</button></div>"
        );
        reject_interact(&html);
    }

    #[test]
    fn vertical_from_props_and_style() {
        let mut c = group("G", &["Top", "Bottom"]);
        c.props.insert("orientation".into(), "vertical".into());
        assert!(render(&c).contains("data-orientation=\"vertical\""));
        let mut s = group("G", &["Top", "Bottom"]);
        s.style = Some("button-group+vertical".into());
        assert!(render(&s).contains("data-orientation=\"vertical\""));
    }

    #[test]
    fn label_only_still_emits_one_button() {
        let html = render(&stub("button-group", "Edit"));
        assert_eq!(html.matches("data-slot=\"button\"").count(), 1);
        assert!(html.contains(">Edit</button>"));
    }

    #[test]
    fn skips_interact_buttonish() {
        let html = render(&group("G", &["Edit", "Share"]));
        assert_eq!(html.matches("<button").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            reject_interact(&render(&group("G", &["Edit", "Share"])));
        });
    }

    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"button-group\"] > [data-slot=\"button\"] { line-height: 1.25rem; }"
        ));
        assert!(css.contains(
            "[data-slot=\"button-group\"] > [data-slot=\"button\"][data-variant=\"primary\"] { border-width: 0; }"
        ));
        assert!(css.contains("margin-inline-start: -1px"));
        assert!(css.contains("margin-top: -1px"));
        assert!(!css.contains("zinc-"));
    }
}
