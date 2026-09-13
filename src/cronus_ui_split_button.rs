//! Dedicated SplitButton renderer. DOM matches React:
//! `<div data-slot="split-button" role="group">` plus primary
//! `<button data-slot="button">` from the label and a chevron button.
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
    if disabled {
        root.push_str(" data-disabled=\"\"");
        root.push_str(" aria-disabled=\"true\"");
    }

    let disabled_attr = if disabled { " disabled" } else { "" };
    let primary = format!(
        "<button type=\"button\" data-slot=\"button\"{disabled_attr}>{label}</button>"
    );
    let chevron = format!(
        "<button type=\"button\" data-slot=\"button\" aria-label=\"{menu_label}\" aria-haspopup=\"menu\"{disabled_attr}>{CHEVRON}</button>"
    );

    format!("<div {root}>{primary}{chevron}</div>")
}

fn variant_of(comp: &ComponentNode) -> &'static str {
    let raw = attr(comp, "variant").unwrap_or("");
    match raw {
        "secondary" => "secondary",
        "outline" => "outline",
        "ghost" => "ghost",
        "destructive" | "danger" => "destructive",
        "link" => "link",
        "primary" => "primary",
        _ => {
            let style = comp.style.as_deref().unwrap_or("");
            if style.split('+').any(|p| p.trim() == "secondary") {
                "secondary"
            } else if style.split('+').any(|p| p.trim() == "outline") {
                "outline"
            } else if style.split('+').any(|p| p.trim() == "ghost") {
                "ghost"
            } else if style.split('+').any(|p| p.trim() == "destructive" || p.trim() == "danger") {
                "destructive"
            } else {
                "primary"
            }
        }
    }
}

fn menu_label_of(comp: &ComponentNode) -> String {
    if let Some(v) = attr(comp, "menuLabel")
        .or_else(|| attr(comp, "menu-label"))
        .filter(|s| !s.is_empty())
    {
        return esc(v);
    }
    "More actions".into()
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

    fn reject_interact(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("display:inline-flex;gap:0.25rem"));
        assert!(!html.contains("height:2.5rem;padding:0 1rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("<nav"));
    }

    #[test]
    fn root_is_group_with_primary_and_chevron() {
        let html = render(&stub("split-button", "Save"));
        assert!(html.starts_with("<div data-slot=\"split-button\" role=\"group\""));
        assert!(html.contains("data-variant=\"primary\""));
        assert_eq!(html.matches("<button type=\"button\" data-slot=\"button\"").count(), 2);
        assert!(html.contains(">Save</button>"));
        assert!(html.contains("aria-label=\"More actions\""));
        assert!(html.contains("aria-haspopup=\"menu\""));
        assert!(html.contains("<svg "));
        assert!(html.contains("aria-hidden=\"true\""));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"split-button\" role=\"group\" data-variant=\"primary\"><button type=\"button\" data-slot=\"button\">Save</button><button type=\"button\" data-slot=\"button\" aria-label=\"More actions\" aria-haspopup=\"menu\"><svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" aria-hidden=\"true\"><path d=\"m6 9 6 6 6-6\"/></svg></button></div>"
        );
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
        assert!(html.contains(" data-disabled=\"\""));
        assert!(html.contains(" aria-disabled=\"true\""));
        assert_eq!(html.matches(" disabled").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_buttonish() {
        let c = stub("split-button", "Save");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("split-button", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<div data-slot=\"split-button\""));
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
            let html = render(&stub("split-button", "Save"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"split-button\"]"));
        assert!(css.contains("display: inline-flex"));
        assert!(css.contains("border-top-right-radius: 0"));
        assert!(css.contains("border-top-left-radius: 0"));
        assert!(css.contains("aspect-ratio: 1"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
