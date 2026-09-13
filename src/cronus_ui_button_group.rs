//! Dedicated ButtonGroup renderer. DOM matches React:
//! `<div data-slot="button-group" role="group" data-orientation="horizontal">`
//! plus child `<button type="button" data-slot="button">` from text items.
//! Not interact `buttonish()` (wrapper with a SINGLE primary button and inline styles).

use crate::cronus_ui_kit::texts;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let orientation = orientation_of(comp);
    let buttons = texts(comp)
        .into_iter()
        .map(|t| format!("<button type=\"button\" data-slot=\"button\">{t}</button>"))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"button-group\" role=\"group\" data-orientation=\"{orientation}\">{buttons}</div>"
    )
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

    fn extra(text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "item".into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn group(labels: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("button-group", labels[0]);
        for n in labels.iter().skip(1) {
            c.items.push(extra(n));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("display:inline-flex;gap:0.25rem"));
        assert!(!html.contains("height:2.5rem;padding:0 1rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("{ value }"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
    }

    #[test]
    fn root_is_group_of_buttons_not_single_buttonish() {
        let html = render(&group(&["Edit", "Share", "Delete"]));
        assert!(html.starts_with("<div data-slot=\"button-group\""));
        assert!(html.contains("role=\"group\""));
        assert!(html.contains("data-orientation=\"horizontal\""));
        assert_eq!(html.matches("<button type=\"button\" data-slot=\"button\">").count(), 3);
        assert!(html.contains(">Edit</button>"));
        assert!(html.contains(">Share</button>"));
        assert!(html.contains(">Delete</button>"));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"button-group\" role=\"group\" data-orientation=\"horizontal\"><button type=\"button\" data-slot=\"button\">Edit</button><button type=\"button\" data-slot=\"button\">Share</button><button type=\"button\" data-slot=\"button\">Delete</button></div>"
        );
    }

    #[test]
    fn vertical_from_props() {
        let mut c = group(&["Top", "Bottom"]);
        c.props.insert("orientation".into(), "vertical".into());
        let html = render(&c);
        assert!(html.contains("data-orientation=\"vertical\""));
        assert!(!html.contains("data-orientation=\"horizontal\""));
        assert_eq!(html.matches("data-slot=\"button\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn vertical_from_style() {
        let mut c = group(&["Top", "Bottom"]);
        c.style = Some("button-group+vertical".into());
        let html = render(&c);
        assert!(html.contains("data-orientation=\"vertical\""));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_buttonish() {
        let html = render(&group(&["Edit", "Share"]));
        let interact =
            crate::cronus_ui_interact::render("button-group", &stub("button-group", "Edit")).unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<div data-slot=\"button-group\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("display:inline-flex;gap:0.25rem"));
        assert_eq!(interact.matches("<button").count(), 1);
        assert_eq!(html.matches("<button").count(), 2);
        assert!(html.contains("role=\"group\""));
        assert!(!interact.contains("role=\"group\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&group(&["Edit", "Share"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"button-group\"]"));
        assert!(css.contains("display: inline-flex"));
        assert!(css.contains("flex-direction: row"));
        assert!(css.contains("flex-direction: column"));
        assert!(css.contains("margin-left: -1px"));
        assert!(css.contains("margin-top: -1px"));
        assert!(css.contains("border-top-left-radius: 0"));
        assert!(css.contains("[data-slot=\"button-group\"] > *:focus-visible"));
        assert!(!css.contains("zinc-"));
    }
}
