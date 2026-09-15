//! Dedicated SplitButton renderer. DOM matches React:
//! `<div data-slot="split-button" role="group" data-variant aria-label>` plus primary
//! `<button data-slot="button">` from the label and a chevron `<button data-slot="button">`.
//! Zero JS: the chevron's `popovertarget` opens React's `DropdownMenuContent`
//! (`dropdown-menu-content` > `dropdown-menu-item`s from the extra texts) as a
//! native `popover="auto"` anchored under it, emitted right after the group
//! (React portals it out of the canvas). Esc / outside click dismiss it.
//! Gaps: items run no action (`onSelect` needs JS), no roving arrow-key focus,
//! `aria-expanded` is not reflected. `disabled` / `loading` disable both halves.
//! Not interact `buttonish()` (single primary + inline SURF).

use crate::cronus_ui_kit::{attr, attr_nonempty, content_texts, esc, flag, label_of, widget_id};
use crate::parser::ComponentNode;

const CHEVRON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" aria-hidden=\"true\"><path d=\"m6 9 6 6 6-6\"/></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let variant = variant_of(comp);
    let menu_label = menu_label_of(comp);
    let disabled = flag(comp, "disabled") || flag(comp, "loading");

    let mut root = format!("data-slot=\"split-button\" role=\"group\" data-variant=\"{variant}\"");
    if let Some(a) = attr_nonempty(comp, "aria-label") {
        root.push_str(&format!(" aria-label=\"{}\"", esc(a)));
    }
    if disabled {
        root.push_str(" data-disabled=\"\" aria-disabled=\"true\"");
    }

    let disabled_attr = if disabled { " disabled" } else { "" };
    let primary =
        format!("<button type=\"button\" data-slot=\"button\"{disabled_attr}>{label}</button>");
    let trigger_id = widget_id(comp, "menu-trigger");
    let pop_id = widget_id(comp, "menu");
    let chevron = format!(
        "<button type=\"button\" id=\"{trigger_id}\" data-slot=\"button\" aria-label=\"{menu_label}\" aria-haspopup=\"menu\" popovertarget=\"{pop_id}\"{disabled_attr}>{CHEVRON}</button>"
    );
    let items = content_texts(comp)
        .into_iter()
        .map(|t| format!("<div data-slot=\"dropdown-menu-item\" role=\"menuitem\">{t}</div>"))
        .collect::<String>();
    format!(
        "<div {root}>{primary}{chevron}</div><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"dropdown-menu-content\" role=\"menu\" aria-orientation=\"vertical\" anchor=\"{trigger_id}\">{items}</div>"
    )
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
    fn emitted_fixture_chevron_opens_native_popover_menu() {
        let mut c = stub("split-button", "Save");
        c.items.push(extra("Duplicate"));
        c.items.push(extra("Archive"));
        c.items[2]
            .config
            .insert("aria-label".into(), "Save actions".into());
        let html = render(&c);
        let tid = crate::cronus_ui_kit::widget_id(&c, "menu-trigger");
        let pid = crate::cronus_ui_kit::widget_id(&c, "menu");
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"split-button\" role=\"group\" data-variant=\"primary\" aria-label=\"Save actions\"><button type=\"button\" data-slot=\"button\">Save</button><button type=\"button\" id=\"{tid}\" data-slot=\"button\" aria-label=\"More actions\" aria-haspopup=\"menu\" popovertarget=\"{pid}\">{CHEVRON}</button></div><div id=\"{pid}\" popover=\"auto\" data-slot=\"dropdown-menu-content\" role=\"menu\" aria-orientation=\"vertical\" anchor=\"{tid}\"><div data-slot=\"dropdown-menu-item\" role=\"menuitem\">Duplicate</div><div data-slot=\"dropdown-menu-item\" role=\"menuitem\">Archive</div></div>"
            )
        );
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
        assert!(css.contains(
            "[data-slot=\"split-button\"] > [data-slot=\"button\"]:last-of-type::before"
        ));
        assert!(css.contains("aspect-ratio: 1"));
        assert!(!css.contains("zinc-"));
    }
}
