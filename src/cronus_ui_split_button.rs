//! Dedicated SplitButton renderer. DOM matches React:
//! `<div data-slot="split-button" role="group" data-variant aria-label>` plus primary
//! `<button data-slot="button">` from the label and a chevron `<button data-slot="button">`.
//! Opening the menu needs JS, so (wave 1t rule) the chevron is the same native
//! `<button>` marked `disabled`, without visual attenuation, and the menu items
//! (React portals them out of the canvas) are not emitted.
//! Not interact `buttonish()` (single primary + inline SURF).

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

const CHEVRON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" aria-hidden=\"true\"><path d=\"m6 9 6 6 6-6\"/></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let variant = variant_of(comp);
    let menu_label = menu_label_of(comp);
    let disabled = flag(comp, "disabled") || flag(comp, "loading");

    let mut root = format!("data-slot=\"split-button\" role=\"group\" data-variant=\"{variant}\"");
    if let Some(a) = attr(comp, "aria-label").filter(|s| !s.is_empty()) {
        root.push_str(&format!(" aria-label=\"{}\"", esc(a)));
    }
    if disabled {
        root.push_str(" data-disabled=\"\" aria-disabled=\"true\"");
    }

    let disabled_attr = if disabled { " disabled" } else { "" };
    let primary = format!("<button type=\"button\" data-slot=\"button\"{disabled_attr}>{label}</button>");
    let chevron = format!(
        "<button type=\"button\" data-slot=\"button\" aria-label=\"{menu_label}\" aria-haspopup=\"menu\" disabled>{CHEVRON}</button>"
    );
    format!("<div {root}>{primary}{chevron}</div>")
}

fn variant_of(comp: &ComponentNode) -> &'static str {
    match attr(comp, "variant").unwrap_or("") {
        "secondary" => "secondary",
        "outline" => "outline",
        "ghost" => "ghost",
        "destructive" | "danger" => "destructive",
        "link" => "link",
        "primary" => "primary",
        _ => {
            let style = comp.style.as_deref().unwrap_or("");
            let has = |name: &str| style.split('+').any(|p| p.trim() == name);
            if has("secondary") {
                "secondary"
            } else if has("outline") {
                "outline"
            } else if has("ghost") {
                "ghost"
            } else if has("destructive") || has("danger") {
                "destructive"
            } else {
                "primary"
            }
        }
    }
}

fn menu_label_of(comp: &ComponentNode) -> String {
    attr(comp, "menuLabel")
        .or_else(|| attr(comp, "menu-label"))
        .filter(|s| !s.is_empty())
        .map(esc)
        .unwrap_or_else(|| "More actions".into())
}

fn attr<'a>(comp: &'a ComponentNode, name: &str) -> Option<&'a str> {
    if let Some(v) = comp.props.get(name) {
        return Some(v.as_str());
    }
    comp.items
        .iter()
        .find_map(|i| i.config.get(name).map(String::as_str))
}

fn flag(comp: &ComponentNode, name: &str) -> bool {
    attr(comp, name).map(|s| s == "true").unwrap_or(false)
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
        assert!(!html.contains("display:inline-flex;gap:0.25rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("<nav"));
    }

    #[test]
    fn emitted_fixture_primary_and_disabled_chevron() {
        let mut c = stub("split-button", "Save");
        c.items.push(extra("Duplicate"));
        c.items.push(extra("Archive"));
        c.items[2].config.insert("aria-label".into(), "Save actions".into());
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"split-button\" role=\"group\" data-variant=\"primary\" aria-label=\"Save actions\"><button type=\"button\" data-slot=\"button\">Save</button><button type=\"button\" data-slot=\"button\" aria-label=\"More actions\" aria-haspopup=\"menu\" disabled>{CHEVRON}</button></div>"
            )
        );
        assert!(!html.contains("Duplicate"));
        reject_interact(&html);
    }

    #[test]
    fn variant_from_props() {
        let mut c = stub("split-button", "Export");
        c.props.insert("variant".into(), "outline".into());
        let html = render(&c);
        assert!(html.contains("data-variant=\"outline\""));
        assert!(html.contains(">Export</button>"));
        reject_interact(&html);
    }

    #[test]
    fn disabled_both_segments() {
        let mut c = stub("split-button", "Save");
        c.props.insert("disabled".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(" data-disabled=\"\" aria-disabled=\"true\""));
        assert_eq!(html.matches(" disabled>").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_buttonish() {
        let c = stub("split-button", "Save");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("split-button", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("display:inline-flex;gap:0.25rem"));
        assert_eq!(html.matches("<button").count(), 2);
        assert!(html.contains("role=\"group\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            reject_interact(&render(&stub("split-button", "Save")));
        });
    }

    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"split-button\"] > [data-slot=\"button\"] { position: relative; line-height: 1.25rem; }"
        ));
        assert!(css.contains(
            "[data-slot=\"split-button\"][data-variant=\"primary\"] > [data-slot=\"button\"] { border-width: 0; }"
        ));
        assert!(css.contains("[data-slot=\"split-button\"] > [data-slot=\"button\"]:last-of-type::before"));
        assert!(css.contains("aspect-ratio: 1"));
        assert!(!css.contains("zinc-"));
    }
}
