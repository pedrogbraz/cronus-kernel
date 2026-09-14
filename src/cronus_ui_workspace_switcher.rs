//! Dedicated WorkspaceSwitcher renderer. React idle state is a closed menu:
//! `<button data-slot="workspace-switcher" disabled>` with
//! `<span data-slot="avatar"><span data-slot="avatar-fallback">` initials, the
//! current name `<span>` and a chevrons-up-down `<svg>`.
//! Opening the menu needs JS, so (wave 1t rule) the trigger is the same native
//! `<button>` marked `disabled`, without visual attenuation (React's idle trigger
//! is not dimmed) and no menu content is emitted.
//! The `label` names the widget ("Switch workspace"); it is never a workspace.
//! Not interact `nav("workspace-switcher")` SURF `<nav>` and not `<details>`.

use crate::cronus_ui_kit::{choice_texts, esc, label_of};
use crate::parser::ComponentNode;

const CHEVRONS: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m7 15 5 5 5-5\"/><path d=\"m7 9 5-5 5 5\"/></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let current = workspaces(comp)
        .into_iter()
        .next()
        .unwrap_or_else(|| label_of(comp));
    format!(
        "<button type=\"button\" data-slot=\"workspace-switcher\" aria-label=\"Switch workspace, {current}\" disabled><span data-slot=\"avatar\"><span data-slot=\"avatar-fallback\">{}</span></span><span>{current}</span>{CHEVRONS}</button>",
        esc(&initials(&current))
    )
}

/// React `workspaceInitials`: first letters of the first two words, else first two chars.
fn initials(escaped_name: &str) -> String {
    let name = escaped_name
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&");
    let parts: Vec<&str> = name.split_whitespace().collect();
    let raw: String = if parts.len() >= 2 {
        parts[..2].iter().filter_map(|p| p.chars().next()).collect()
    } else {
        name.trim().chars().take(2).collect()
    };
    raw.to_uppercase()
}

fn workspaces(comp: &ComponentNode) -> Vec<String> {
    let choices = choice_texts(comp);
    if !choices.is_empty() {
        return choices;
    }
    comp.items
        .iter()
        .filter(|i| !matches!(i.item_type.as_str(), "label" | "title") && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect()
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

    fn reject_interact(html: &str) {
        assert!(!html.contains("<nav"));
        assert!(!html.contains("<details"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn emitted_fixture_is_disabled_idle_trigger_with_avatar() {
        let mut c = stub("workspace-switcher", "Switch workspace");
        c.items.push(extra("text", "Cronus"));
        c.items.push(extra("text", "Northwind"));
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<button type=\"button\" data-slot=\"workspace-switcher\" aria-label=\"Switch workspace, Cronus\" disabled><span data-slot=\"avatar\"><span data-slot=\"avatar-fallback\">CR</span></span><span>Cronus</span>{CHEVRONS}</button>"
            )
        );
        assert!(!html.contains(">Switch workspace<"));
        assert!(!html.contains("workspace-switcher-content"));
        reject_interact(&html);
    }

    #[test]
    fn initials_follow_react() {
        assert_eq!(initials("Acme Corp"), "AC");
        assert_eq!(initials("Globex"), "GL");
        assert_eq!(initials("x"), "X");
    }

    #[test]
    fn label_only_uses_label_as_current() {
        let html = render(&stub("workspace-switcher", "Acme"));
        assert!(html.contains("aria-label=\"Switch workspace, Acme\" disabled>"));
        assert!(html.contains("<span>Acme</span>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_nav_surf_and_passes_stub_gate() {
        let mut c = stub("workspace-switcher", "Orgs");
        c.items.push(extra("item", "Acme"));
        let html = render(&c);
        assert!(!crate::cli::stub_renderer_gate::looks_like_interact_generic(&html));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            reject_interact(&render(&stub("workspace-switcher", "Acme")));
        });
    }

    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"workspace-switcher\"] [data-slot=\"avatar\"] {\n  width: 1.5rem; height: 1.5rem;"));
        assert!(css.contains("border-radius: var(--cronus-radius-lg); padding: 0.375rem 0.5rem;"));
        assert!(!css.contains("[data-slot=\"workspace-switcher-content\"]"));
        assert!(!css.contains("zinc-"));
    }
}
