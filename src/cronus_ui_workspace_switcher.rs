//! Dedicated WorkspaceSwitcher renderer. React idle state is a closed menu:
//! `<button data-slot="workspace-switcher">` with
//! `<span data-slot="avatar"><span data-slot="avatar-fallback">` initials, the
//! current name `<span>` and a chevrons-up-down `<svg>`.
//! Zero JS: the trigger's `popovertarget` opens React's
//! `workspace-switcher-content` menu as a native `popover="auto"` anchored under
//! it (Esc / outside click dismiss), one `dropdown-menu-radio-item`
//! (`menuitemradio`, indicator dot + avatar + name) per workspace.
//!
//! Picking a workspace is native too: each menu row is a `<label>` with a
//! visually hidden radio (page-unique group) and React's decorative item
//! (`aria-hidden`); the trigger carries one `avatar` + name pair per
//! workspace and CSS shows the pair whose radio is checked, so the trigger
//! follows the selection (`value:"…"` / `item … selected:true` picks the
//! initial one, else the first). A slotless `<div>` wraps trigger + menu so
//! `:has()` can pair them; `width:56` (the docs' `w-56` box) is a class on it.
//! `item "Cronus" initials:"CR"` overrides the derived initials.
//! Docs "In a sidebar": `sidebar:"Cronus"` wraps the switcher in a
//! `Sidebar collapsible="none"` header (brand span + switcher) with a menu of
//! the `link` lines (`link "Home" icon:home active:true`).
//! Gaps: `aria-label` on the trigger keeps the initial name; no roving
//! arrow-key focus. The `label` names the widget; it is never a workspace.
//! Not interact `nav("workspace-switcher")` SURF `<nav>` and not `<details>`.

use crate::cronus_ui_kit::{attr_nonempty, esc, instance_id, label_of, safe_url, widget_id};
use crate::parser::{ComponentItemNode, ComponentNode};

const CHEVRONS: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m7 15 5 5 5-5\"/><path d=\"m7 9 5-5 5 5\"/></svg>";
const DOT: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"currentColor\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><circle cx=\"12\" cy=\"12\" r=\"10\"></circle></svg>";

/// Trigger rows beyond this index render but never become the shown current one.
pub const MAX_WORKSPACES: usize = 16;

struct Workspace {
    name: String,
    initials: String,
}

pub fn render(comp: &ComponentNode) -> String {
    let mut listed = workspaces(comp);
    if listed.is_empty() {
        let name = label_of(comp);
        listed.push(Workspace {
            initials: initials(&name),
            name,
        });
    }
    let selected = attr_nonempty(comp, "value")
        .map(esc)
        .and_then(|v| listed.iter().position(|w| w.name == v))
        .or_else(|| {
            comp.items.iter().filter(|i| is_workspace(i)).position(|i| {
                i.config
                    .get("selected")
                    .is_some_and(|v| crate::cronus_ui_kit::truthy(v))
            })
        })
        .unwrap_or(0);
    let current = &listed[selected];
    let trigger_id = widget_id(comp, "trigger");
    let pop_id = widget_id(comp, "menu");
    let group = instance_id(comp, "workspace");
    let pairs = listed
        .iter()
        .map(|w| {
            format!(
                "<span>{}<span>{}</span></span>",
                avatar(&w.initials),
                w.name
            )
        })
        .collect::<String>();
    let items = listed
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let checked = if i == selected { " checked" } else { "" };
            format!(
                "<label><input type=\"radio\" name=\"{group}\" aria-label=\"{}\"{checked}><div data-slot=\"dropdown-menu-radio-item\" role=\"menuitemradio\" aria-hidden=\"true\" tabindex=\"-1\"><span>{DOT}</span>{}<span>{}</span></div></label>",
                w.name,
                avatar(&w.initials),
                w.name
            )
        })
        .collect::<String>();
    let class = match attr_nonempty(comp, "width").map(str::trim) {
        Some(w @ ("48" | "56" | "64" | "72")) => format!(" class=\"w-{w}\""),
        _ => String::new(),
    };
    let switcher = format!(
        "<div{class}><button type=\"button\" id=\"{trigger_id}\" data-slot=\"workspace-switcher\" aria-label=\"Switch workspace, {}\" aria-haspopup=\"menu\" popovertarget=\"{pop_id}\">{pairs}{CHEVRONS}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"workspace-switcher-content\" role=\"menu\" aria-orientation=\"vertical\" anchor=\"{trigger_id}\">{items}</div></div>",
        current.name
    );
    match attr_nonempty(comp, "sidebar") {
        Some(brand) => in_sidebar(comp, &esc(brand), &switcher),
        None => switcher,
    }
}

/// The docs' `SidebarHeader` (brand + switcher) over a menu of the `link` lines.
fn in_sidebar(comp: &ComponentNode, brand: &str, switcher: &str) -> String {
    let links: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "link" && !i.text.is_empty())
        .collect();
    let explicit = links.iter().any(|l| l.config.contains_key("active"));
    let menu = links
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let active = match l.config.get("active") {
                Some(v) => crate::cronus_ui_kit::truthy(v),
                None => !explicit && i == 0,
            };
            let state = if active {
                " data-active=\"true\" aria-current=\"page\""
            } else {
                " data-active=\"false\""
            };
            let icon = crate::cronus_ui_kit::item_icon(l);
            let inner = if icon.is_empty() {
                esc(&l.text)
            } else {
                format!("{icon}<span>{}</span>", esc(&l.text))
            };
            let button = match l.link.as_deref().filter(|h| !h.trim().is_empty()) {
                Some(h) => format!(
                    "<a data-slot=\"sidebar-menu-button\" href=\"{}\"{state}>{inner}</a>",
                    safe_url(h)
                ),
                None => format!(
                    "<button type=\"button\" data-slot=\"sidebar-menu-button\"{state} disabled>{inner}</button>"
                ),
            };
            format!("<li data-slot=\"sidebar-menu-item\">{button}</li>")
        })
        .collect::<String>();
    format!(
        "<div data-slot=\"sidebar-wrapper\" data-state=\"expanded\"><aside data-slot=\"sidebar\" data-state=\"expanded\" data-collapsible=\"\" data-variant=\"sidebar\" data-side=\"left\"><nav aria-label=\"Sidebar\"><div data-slot=\"sidebar-header\"><span>{brand}</span>{switcher}</div><div data-slot=\"sidebar-content\"><div data-slot=\"scroll-area\"><div><div data-slot=\"sidebar-group\"><div data-slot=\"sidebar-group-content\"><ul data-slot=\"sidebar-menu\">{menu}</ul></div></div></div></div></div></nav></aside></div>"
    )
}

fn avatar(initials: &str) -> String {
    format!(
        "<span data-slot=\"avatar\"><span data-slot=\"avatar-fallback\">{initials}</span></span>"
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
    esc(&raw.to_uppercase())
}

fn is_workspace(i: &ComponentItemNode) -> bool {
    matches!(i.item_type.as_str(), "item" | "tab" | "columns") && !i.text.is_empty()
}

fn workspaces(comp: &ComponentNode) -> Vec<Workspace> {
    let picked: Vec<&ComponentItemNode> = {
        let choices: Vec<&ComponentItemNode> =
            comp.items.iter().filter(|i| is_workspace(i)).collect();
        if choices.is_empty() {
            comp.items
                .iter()
                .filter(|i| {
                    !matches!(i.item_type.as_str(), "label" | "title" | "link")
                        && !i.text.is_empty()
                })
                .collect()
        } else {
            choices
        }
    };
    picked
        .into_iter()
        .map(|i| {
            let name = esc(&i.text);
            Workspace {
                initials: i
                    .config
                    .get("initials")
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| esc(s.trim()))
                    .unwrap_or_else(|| initials(&name)),
                name,
            }
        })
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
        assert!(!html.contains("<details"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn emitted_fixture_trigger_opens_native_popover_menu() {
        let mut c = stub("workspace-switcher", "Switch workspace");
        c.items.push(extra("text", "Cronus"));
        c.items.push(extra("text", "Northwind"));
        let html = render(&c);
        let tid = crate::cronus_ui_kit::widget_id(&c, "trigger");
        let pid = crate::cronus_ui_kit::widget_id(&c, "menu");
        let g = crate::cronus_ui_kit::widget_id(&c, "workspace");
        assert_eq!(
            html,
            format!(
                "<div><button type=\"button\" id=\"{tid}\" data-slot=\"workspace-switcher\" aria-label=\"Switch workspace, Cronus\" aria-haspopup=\"menu\" popovertarget=\"{pid}\"><span><span data-slot=\"avatar\"><span data-slot=\"avatar-fallback\">CR</span></span><span>Cronus</span></span><span><span data-slot=\"avatar\"><span data-slot=\"avatar-fallback\">NO</span></span><span>Northwind</span></span>{CHEVRONS}</button><div id=\"{pid}\" popover=\"auto\" data-slot=\"workspace-switcher-content\" role=\"menu\" aria-orientation=\"vertical\" anchor=\"{tid}\"><label><input type=\"radio\" name=\"{g}\" aria-label=\"Cronus\" checked><div data-slot=\"dropdown-menu-radio-item\" role=\"menuitemradio\" aria-hidden=\"true\" tabindex=\"-1\"><span>{DOT}</span><span data-slot=\"avatar\"><span data-slot=\"avatar-fallback\">CR</span></span><span>Cronus</span></div></label><label><input type=\"radio\" name=\"{g}\" aria-label=\"Northwind\"><div data-slot=\"dropdown-menu-radio-item\" role=\"menuitemradio\" aria-hidden=\"true\" tabindex=\"-1\"><span>{DOT}</span><span data-slot=\"avatar\"><span data-slot=\"avatar-fallback\">NO</span></span><span>Northwind</span></div></label></div></div>"
            )
        );
        assert!(!html.contains(">Switch workspace<"));
        assert!(!html.contains(" disabled"));
        reject_interact(&html);
    }

    #[test]
    fn initials_follow_react_and_can_be_declared() {
        assert_eq!(initials("Acme Corp"), "AC");
        assert_eq!(initials("Globex"), "GL");
        assert_eq!(initials("x"), "X");
        let mut c = stub("workspace-switcher", "Switch workspace");
        let mut nw = extra("item", "Northwind");
        nw.config.insert("initials".into(), "NW".into());
        c.items.push(nw);
        let mut acme = extra("item", "Acme");
        acme.config.insert("selected".into(), "true".into());
        c.items.push(acme);
        let html = render(&c);
        assert!(html.contains("avatar-fallback\">NW</span>"));
        assert!(html.contains("aria-label=\"Switch workspace, Acme\""));
        assert!(html.contains("aria-label=\"Acme\" checked>"));
        assert!(!html.contains("aria-label=\"Northwind\" checked"));
    }

    #[test]
    fn label_only_uses_label_as_current() {
        let html = render(&stub("workspace-switcher", "Acme"));
        assert!(html.contains("aria-label=\"Switch workspace, Acme\" aria-haspopup=\"menu\""));
        assert!(html.contains("<span>Acme</span>"));
        reject_interact(&html);
    }

    #[test]
    fn docs_sidebar_variant_wraps_the_header_and_menu() {
        let mut c = stub("workspace-switcher", "Switch workspace");
        c.props.insert("sidebar".into(), "Cronus".into());
        c.items.push(extra("item", "Cronus"));
        let mut home = extra("link", "Home");
        home.config.insert("icon".into(), "home".into());
        home.config.insert("active".into(), "true".into());
        c.items.push(home);
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"sidebar-wrapper\" data-state=\"expanded\"><aside data-slot=\"sidebar\" data-state=\"expanded\" data-collapsible=\"\" data-variant=\"sidebar\" data-side=\"left\"><nav aria-label=\"Sidebar\"><div data-slot=\"sidebar-header\"><span>Cronus</span><div><button type=\"button\" id=\""), "{html}");
        assert!(html.contains("<ul data-slot=\"sidebar-menu\"><li data-slot=\"sidebar-menu-item\"><button type=\"button\" data-slot=\"sidebar-menu-button\" data-active=\"true\" aria-current=\"page\" disabled><svg "));
        assert!(html.contains("</svg><span>Home</span></button></li></ul>"));
        // Links are not workspaces.
        assert_eq!(html.matches("menuitemradio").count(), 1);
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
        assert!(css.contains("[data-slot=\"workspace-switcher-content\"]:popover-open {"));
        assert!(!css.contains("zinc-"));
        // The trigger shows the checked workspace's pair.
        assert!(css.contains(
            "[data-slot=\"workspace-switcher\"] > span:not([data-slot]) { display: none; }"
        ));
        for n in 1..=MAX_WORKSPACES {
            assert!(css.contains(&format!(
                "div:has(> [data-slot=\"workspace-switcher-content\"] > label:nth-child({n}) > input:checked) > [data-slot=\"workspace-switcher\"] > span:nth-child({n})"
            )), "{n}");
        }
        assert!(css.contains("[data-slot=\"workspace-switcher-content\"] > label:has(> input:checked) > [data-slot=\"dropdown-menu-radio-item\"] > span:first-child > svg { display: block; }"));
        assert!(css.contains("div.w-56:has(> [data-slot=\"workspace-switcher\"]) { min-width: 14rem; max-width: 14rem; }"));
    }
}
