//! Dedicated AppShell renderer. DOM matches React `AppShell` (a composition
//! over the Sidebar system — React has no `app-shell` root slot):
//! `<div data-slot="sidebar-wrapper" data-app-shell>` + `<aside data-slot="sidebar">`
//! (text items become sidebar menu buttons) + `app-shell-content` with
//! `app-shell-header` (`<span>` title from `label`) and an empty `app-shell-body`.
//! `data-app-shell` is not a slot; it only scopes the shell CSS.
//! The sidebar takes the Sidebar family's docs props (`header:`, `item …
//! icon: active: group: footer:`); `collapsible:icon` puts the docs'
//! `SidebarTrigger` (native collapse checkbox, see `cronus_ui_sidebar`) before
//! the header title.
//! Not interact `nav("app-shell")` (generic SURF `<nav>` without header/body/content).

use crate::cronus_ui_kit::{attr_nonempty, esc, label_of};
use crate::cronus_ui_sidebar::{aria_label, aside_of, trigger};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let title = label_of(comp);
    let aria = aria_label(comp).unwrap_or_else(|| title.clone());
    let sidebar = aside_of(comp, &aria);
    let body = description(comp)
        .map(|d| format!("<div>{d}</div>"))
        .unwrap_or_default();
    let toggle = if attr_nonempty(comp, "collapsible").map(str::trim) == Some("icon") {
        trigger(comp)
    } else {
        String::new()
    };
    format!(
        "<div data-slot=\"sidebar-wrapper\" data-state=\"expanded\" data-app-shell=\"\">{sidebar}<div data-slot=\"app-shell-content\"><header data-slot=\"app-shell-header\">{toggle}<span>{title}</span></header><div data-slot=\"app-shell-body\">{body}</div></div></div>"
    )
}

/// Body content: `description` from props or item config (`key:value` attaches to the previous item).
fn description(comp: &ComponentNode) -> Option<String> {
    attr_nonempty(comp, "description").map(|s| esc(s))
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
        assert!(!html.contains("<nav data-slot=\"app-shell\""));
        assert!(!html.starts_with("<nav"));
        assert!(!html.contains("<details"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn emitted_fixture_composes_sidebar_header_and_body() {
        let mut c = stub("app-shell", "Acme");
        c.items.push(extra("text", "Home"));
        c.items.push(extra("text", "Inbox"));
        c.items[2].config.insert("aria-label".into(), "App".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"sidebar-wrapper\" data-state=\"expanded\" data-app-shell=\"\"><aside data-slot=\"sidebar\" data-state=\"expanded\" data-collapsible=\"\" data-variant=\"sidebar\" data-side=\"left\"><nav aria-label=\"App\"><div data-slot=\"sidebar-content\"><div data-slot=\"scroll-area\"><div><ul data-slot=\"sidebar-menu\"><li data-slot=\"sidebar-menu-item\"><button type=\"button\" data-slot=\"sidebar-menu-button\" data-active=\"true\" aria-current=\"page\" disabled>Home</button></li><li data-slot=\"sidebar-menu-item\"><button type=\"button\" data-slot=\"sidebar-menu-button\" data-active=\"false\" disabled>Inbox</button></li></ul></div></div></div></nav></aside><div data-slot=\"app-shell-content\"><header data-slot=\"app-shell-header\"><span>Acme</span></header><div data-slot=\"app-shell-body\"></div></div></div>"
        );
        assert!(!html.contains("data-slot=\"app-shell\""));
        reject_interact(&html);
    }

    #[test]
    fn description_attr_fills_body() {
        let mut c = stub("app-shell", "Acme");
        c.items.push(extra("text", "Home"));
        c.items[1]
            .config
            .insert("description".into(), "Inbox".into());
        let html = render(&c);
        assert!(html.contains("<div data-slot=\"app-shell-body\"><div>Inbox</div></div>"));
        assert!(!html.contains(">Inbox</button>"));
        let mut p = stub("app-shell", "Acme");
        p.props.insert("description".into(), "<b>".into());
        assert!(render(&p).contains("<div data-slot=\"app-shell-body\"><div>&lt;b&gt;</div></div>"));
    }

    #[test]
    fn docs_shell_has_trigger_sidebar_header_and_icon_items() {
        let mut c = stub("app-shell", "Dashboard");
        c.props.insert("collapsible".into(), "icon".into());
        c.props.insert("header".into(), "Acme Inc.".into());
        c.props
            .insert("description".into(), "Your page content goes here.".into());
        let mut home = extra("item", "Home");
        home.config.insert("icon".into(), "home".into());
        home.config.insert("active".into(), "true".into());
        c.items.push(home);
        let mut search = extra("item", "Search");
        search.config.insert("icon".into(), "search".into());
        c.items.push(search);
        let html = render(&c);
        assert!(html.contains("<nav aria-label=\"Dashboard\"><div data-slot=\"sidebar-header\"><span>Acme Inc.</span></div><div data-slot=\"sidebar-content\">"), "{html}");
        assert!(html.contains("data-active=\"true\" aria-current=\"page\" disabled><svg "));
        assert!(html.contains("</svg><span>Search</span></button>"));
        assert!(html.contains(
            "<header data-slot=\"app-shell-header\"><label><input type=\"checkbox\" id=\""
        ));
        assert!(html.contains("data-slot=\"sidebar-trigger\" data-variant=\"ghost\""));
        assert!(html.contains("</label><span>Dashboard</span></header><div data-slot=\"app-shell-body\"><div>Your page content goes here.</div></div>"));
        reject_interact(&html);
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"app-shell-header\"] > span {\n  font-size: 0.875rem; line-height: 1.25rem; font-weight: 500; color: var(--cronus-fg);"));
        assert!(css.contains("[data-slot=\"catalog-frame\"] > [data-slot=\"sidebar-wrapper\"][data-app-shell] { height: 20rem; min-height: 0; overflow: hidden; }"));
    }

    #[test]
    fn label_only_has_empty_menu_and_header_title() {
        let html = render(&stub("app-shell", "Dashboard"));
        assert!(
            html.contains("<header data-slot=\"app-shell-header\"><span>Dashboard</span></header>")
        );
        assert!(html.contains("<ul data-slot=\"sidebar-menu\"></ul>"));
        assert!(html.contains("<div data-slot=\"app-shell-body\"></div>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_nav_surf() {
        let c = stub("app-shell", "Dashboard");
        let html = render(&c);
        assert!(html.contains("data-slot=\"app-shell-content\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            reject_interact(&render(&stub("app-shell", "Dashboard")));
        });
    }

    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(!css.contains("[data-slot=\"app-shell\"] {"));
        assert!(
            css.contains("[data-audit-canvas] > [data-slot=\"sidebar-wrapper\"][data-app-shell]")
        );
        assert!(css.contains("width: 20rem; height: 14rem; min-height: 0; overflow: hidden;"));
        assert!(css.contains("[data-slot=\"app-shell-content\"]"));
        assert!(css.contains("min-height: 100svh"));
        assert!(css.contains("height: 3.5rem"));
        assert!(css.contains("color-mix(in oklab, var(--cronus-surface-base) 80%, transparent)"));
        assert!(css.contains("[data-slot=\"app-shell-body\"]"));
        assert!(!css.contains("zinc-"));
    }
}
