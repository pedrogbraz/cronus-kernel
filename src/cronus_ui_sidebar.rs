//! Dedicated Sidebar renderer. Desktop DOM matches React `SidebarProvider` +
//! `Sidebar collapsible="none"`:
//! `<div data-slot="sidebar-wrapper"><aside data-slot="sidebar"><nav>`
//! `<div data-slot="sidebar-content"><div data-slot="scroll-area"><div>`
//! `<ul data-slot="sidebar-menu">` plus each item as
//! `<li data-slot="sidebar-menu-item"><button data-slot="sidebar-menu-button">`
//! (`<a>` when the item has a link). First item is active. A link-less button would
//! need JS to navigate, so (wave 1t rule) it is a native `disabled` button, not dimmed.
//! The `label` names the widget (nav `aria-label`); it is never a menu item.
//!
//! Docs composition: `header:"Acme Inc."` renders `sidebar-header` (a `px-2`
//! semibold span), `item "Home" icon:home active:true group:"Workspace"` lines
//! group into `sidebar-group` (+ `sidebar-group-label`) > `sidebar-group-content`
//! > menu, `footer:true` items go to `sidebar-footer`'s menu, `icon:` puts a
//! lucide glyph before the `<span>` label and `active:true` marks the current
//! item (else the first). `collapsible:icon` adds the docs' inset column
//! (`sidebar-trigger` ghost icon button + `description:"…"` paragraph) and
//! collapses the sidebar to icons natively: the trigger sits in a `<label>`
//! with a visually hidden checkbox ("Toggle sidebar") and CSS applies React's
//! `data-collapsible="icon"` geometry (3rem width, square buttons, hidden
//! labels, 200ms width transition) while it is checked.
//! Not interact `nav("sidebar")` (generic SURF `<nav>` without sidebar-content/menu).

use crate::cronus_ui_kit::{attr_nonempty, esc, item_icon, label_of, safe_url, widget_id};
use crate::parser::{ComponentItemNode, ComponentNode};

/// React `SidebarTriggerIcon` (panel-left glyph).
const TRIGGER_ICON: &str = "<svg aria-hidden=\"true\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><rect x=\"3\" y=\"4\" width=\"18\" height=\"16\" rx=\"2\"></rect><path d=\"M9 4v16\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let aria = aria_label(comp).unwrap_or_else(|| label_of(comp));
    let collapsible = attr_nonempty(comp, "collapsible").map(str::trim) == Some("icon");
    let inset = if collapsible {
        let description = attr_nonempty(comp, "description")
            .map(|d| format!("<p>{}</p>", esc(d)))
            .unwrap_or_default();
        format!("<div>{}{description}</div>", trigger(comp))
    } else {
        String::new()
    };
    let mut aside = aside_of(comp, &aria);
    if rich_entries(comp).is_empty() {
        // Legacy shape: a lone label is the single menu item.
        let only = vec![Entry {
            text: label_of(comp),
            href: None,
            icon: String::new(),
            active: None,
            group: None,
            footer: false,
        }];
        aside = aside_html(&aria, attr_nonempty(comp, "header").map(|h| esc(h)), &only);
    }
    format!("<div data-slot=\"sidebar-wrapper\" data-state=\"expanded\">{aside}{inset}</div>")
}

/// `SidebarTrigger` (ghost `icon-sm` Button) in a label with the collapse checkbox.
pub fn trigger(comp: &ComponentNode) -> String {
    let id = widget_id(comp, "collapse");
    format!(
        "<label><input type=\"checkbox\" id=\"{id}\" aria-label=\"Toggle sidebar\"><button type=\"button\" data-slot=\"sidebar-trigger\" data-variant=\"ghost\" aria-label=\"Toggle sidebar\" aria-expanded=\"true\" tabindex=\"-1\" aria-hidden=\"true\">{TRIGGER_ICON}</button></label>"
    )
}

/// One menu entry.
pub struct Entry {
    pub text: String,
    pub href: Option<String>,
    pub icon: String,
    pub active: Option<bool>,
    pub group: Option<String>,
    pub footer: bool,
}

/// `<aside data-slot="sidebar">` subtree. Shared with the AppShell renderer,
/// which composes the same Sidebar like React `AppShell` does.
pub fn aside(aria: &str, entries: &[(String, Option<String>)]) -> String {
    let entries: Vec<Entry> = entries
        .iter()
        .map(|(text, href)| Entry {
            text: text.clone(),
            href: href.clone(),
            icon: String::new(),
            active: None,
            group: None,
            footer: false,
        })
        .collect();
    aside_html(aria, None, &entries)
}

/// The aside of a component: header, grouped content, footer (no fallback item).
pub fn aside_of(comp: &ComponentNode, aria: &str) -> String {
    let entries = rich_entries(comp);
    let header = attr_nonempty(comp, "header").map(|h| esc(h));
    aside_html(aria, header, &entries)
}

fn menu_html(entries: &[&Entry], first_active: bool) -> String {
    let explicit = entries.iter().any(|e| e.active.is_some());
    let items = entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let active = match e.active {
                Some(a) => a,
                None => !explicit && first_active && i == 0,
            };
            let state = if active {
                " data-active=\"true\" aria-current=\"page\""
            } else {
                " data-active=\"false\""
            };
            let inner = if e.icon.is_empty() {
                e.text.clone()
            } else {
                format!("{}<span>{}</span>", e.icon, e.text)
            };
            let button = match &e.href {
                Some(h) => format!("<a data-slot=\"sidebar-menu-button\" href=\"{h}\"{state}>{inner}</a>"),
                None => format!(
                    "<button type=\"button\" data-slot=\"sidebar-menu-button\"{state} disabled>{inner}</button>"
                ),
            };
            format!("<li data-slot=\"sidebar-menu-item\">{button}</li>")
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<ul data-slot=\"sidebar-menu\">{items}</ul>")
}

fn aside_html(aria: &str, header: Option<String>, entries: &[Entry]) -> String {
    let body: Vec<&Entry> = entries.iter().filter(|e| !e.footer).collect();
    let footer: Vec<&Entry> = entries.iter().filter(|e| e.footer).collect();
    let grouped = body.iter().any(|e| e.group.is_some());
    let content = if grouped {
        let groups = body
            .iter()
            .fold(Vec::<Option<String>>::new(), |mut acc, e| {
                if !acc.contains(&e.group) {
                    acc.push(e.group.clone());
                }
                acc
            });
        groups
            .iter()
            .enumerate()
            .map(|(gi, g)| {
                let members: Vec<&Entry> = body.iter().copied().filter(|e| &e.group == g).collect();
                let label = g
                    .as_ref()
                    .map(|l| format!("<div data-slot=\"sidebar-group-label\">{}</div>", esc(l)))
                    .unwrap_or_default();
                format!(
                    "<div data-slot=\"sidebar-group\">{label}<div data-slot=\"sidebar-group-content\">{}</div></div>",
                    menu_html(&members, gi == 0)
                )
            })
            .collect::<String>()
    } else {
        menu_html(&body, true)
    };
    let header = header
        .map(|h| format!("<div data-slot=\"sidebar-header\"><span>{h}</span></div>"))
        .unwrap_or_default();
    let footer = if footer.is_empty() {
        String::new()
    } else {
        format!(
            "<div data-slot=\"sidebar-footer\">{}</div>",
            menu_html(&footer, false)
        )
    };
    // React: `data-collapsible` is "" while expanded, whatever the mode; the
    // native collapse checkbox (not this attribute) drives the icon geometry.
    format!(
        "<aside data-slot=\"sidebar\" data-state=\"expanded\" data-collapsible=\"\" data-variant=\"sidebar\" data-side=\"left\"><nav aria-label=\"{aria}\">{header}<div data-slot=\"sidebar-content\"><div data-slot=\"scroll-area\"><div>{content}</div></div></div>{footer}</nav></aside>"
    )
}

fn rich_entries(comp: &ComponentNode) -> Vec<Entry> {
    let choices: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| {
            matches!(i.item_type.as_str(), "item" | "tab" | "columns") && !i.text.is_empty()
        })
        .collect();
    let picked: Vec<&ComponentItemNode> = if choices.is_empty() {
        comp.items
            .iter()
            .filter(|i| !matches!(i.item_type.as_str(), "label" | "title") && !i.text.is_empty())
            .collect()
    } else {
        choices
    };
    picked
        .into_iter()
        .map(|i| Entry {
            text: esc(&i.text),
            href: i.link.as_deref().filter(|s| !s.is_empty()).map(safe_url),
            icon: item_icon(i),
            active: i
                .config
                .get("active")
                .map(|v| crate::cronus_ui_kit::truthy(v)),
            group: i
                .config
                .get("group")
                .filter(|g| !g.trim().is_empty())
                .cloned(),
            footer: i
                .config
                .get("footer")
                .is_some_and(|v| crate::cronus_ui_kit::truthy(v)),
        })
        .collect()
}

/// Menu entries: choice items, else every non-label text item. No fallback.
pub fn menu_entries(comp: &ComponentNode) -> Vec<(String, Option<String>)> {
    rich_entries(comp)
        .into_iter()
        .map(|e| (e.text, e.href))
        .collect()
}

/// `aria-label` from props or item config (`key:value` lines attach to the previous item).
pub fn aria_label(comp: &ComponentNode) -> Option<String> {
    attr_nonempty(comp, "aria-label").map(|s| esc(s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    /// Shape the audit emitter writes: `label`, `text` per item, trailing `aria-label:`.
    fn emitted(label: &str, items: &[&str]) -> ComponentNode {
        let mut c = stub("sidebar", label);
        for n in items {
            c.items.push(extra("text", n));
        }
        if let Some(last) = c.items.last_mut() {
            last.config.insert("aria-label".into(), label.into());
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<nav data-slot=\"sidebar\""));
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
    fn emitted_fixture_matches_react_dom_and_label_is_not_an_item() {
        let html = render(&emitted("Sidebar", &["Home", "Inbox"]));
        assert_eq!(
            html,
            "<div data-slot=\"sidebar-wrapper\" data-state=\"expanded\"><aside data-slot=\"sidebar\" data-state=\"expanded\" data-collapsible=\"\" data-variant=\"sidebar\" data-side=\"left\"><nav aria-label=\"Sidebar\"><div data-slot=\"sidebar-content\"><div data-slot=\"scroll-area\"><div><ul data-slot=\"sidebar-menu\"><li data-slot=\"sidebar-menu-item\"><button type=\"button\" data-slot=\"sidebar-menu-button\" data-active=\"true\" aria-current=\"page\" disabled>Home</button></li><li data-slot=\"sidebar-menu-item\"><button type=\"button\" data-slot=\"sidebar-menu-button\" data-active=\"false\" disabled>Inbox</button></li></ul></div></div></div></nav></aside></div>"
        );
        assert!(!html.contains(">Sidebar</button>"));
        reject_interact(&html);
    }

    #[test]
    fn linked_item_is_anchor() {
        let mut c = stub("sidebar", "Nav");
        let mut home = extra("item", "Home");
        home.link = Some("/".into());
        c.items.push(home);
        let html = render(&c);
        assert!(html.contains(
            "<a data-slot=\"sidebar-menu-button\" href=\"/\" data-active=\"true\" aria-current=\"page\">Home</a>"
        ));
        assert!(html.contains("<nav aria-label=\"Nav\">"));
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_one_item() {
        let html = render(&stub("sidebar", "Home"));
        assert_eq!(html.matches("data-slot=\"sidebar-menu-item\"").count(), 1);
        assert!(html.contains("data-active=\"true\" aria-current=\"page\" disabled>Home</button>"));
        reject_interact(&html);
    }

    #[test]
    fn docs_collapsible_sidebar_with_header_groups_footer_and_trigger() {
        let mut c = stub("sidebar", "Sidebar");
        c.props.insert("collapsible".into(), "icon".into());
        c.props.insert("header".into(), "Acme Inc.".into());
        c.props.insert(
            "description".into(),
            "Toggle the sidebar with the button above.".into(),
        );
        let mut home = extra("item", "Home");
        home.config.insert("icon".into(), "home".into());
        home.config.insert("active".into(), "true".into());
        home.config.insert("group".into(), "Workspace".into());
        c.items.push(home);
        let mut inbox = extra("item", "Inbox");
        inbox.config.insert("icon".into(), "inbox".into());
        inbox.config.insert("group".into(), "Workspace".into());
        c.items.push(inbox);
        let mut settings = extra("item", "Settings");
        settings.config.insert("icon".into(), "settings".into());
        settings.config.insert("footer".into(), "true".into());
        c.items.push(settings);
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"sidebar-wrapper\" data-state=\"expanded\"><aside data-slot=\"sidebar\" data-state=\"expanded\" data-collapsible=\"\" data-variant=\"sidebar\" data-side=\"left\"><nav aria-label=\"Sidebar\"><div data-slot=\"sidebar-header\"><span>Acme Inc.</span></div><div data-slot=\"sidebar-content\"><div data-slot=\"scroll-area\"><div><div data-slot=\"sidebar-group\"><div data-slot=\"sidebar-group-label\">Workspace</div><div data-slot=\"sidebar-group-content\"><ul data-slot=\"sidebar-menu\"><li data-slot=\"sidebar-menu-item\"><button type=\"button\" data-slot=\"sidebar-menu-button\" data-active=\"true\" aria-current=\"page\" disabled><svg "), "{html}");
        assert!(html.contains("data-icon=\"home\""));
        assert!(html.contains("</svg><span>Home</span></button></li><li data-slot=\"sidebar-menu-item\"><button type=\"button\" data-slot=\"sidebar-menu-button\" data-active=\"false\" disabled><svg "));
        assert!(html.contains("</div></div></div></div><div data-slot=\"sidebar-footer\"><ul data-slot=\"sidebar-menu\"><li data-slot=\"sidebar-menu-item\"><button type=\"button\" data-slot=\"sidebar-menu-button\" data-active=\"false\" disabled><svg "));
        let id = widget_id(&c, "collapse");
        assert!(html.ends_with(&format!(
            "</nav></aside><div><label><input type=\"checkbox\" id=\"{id}\" aria-label=\"Toggle sidebar\"><button type=\"button\" data-slot=\"sidebar-trigger\" data-variant=\"ghost\" aria-label=\"Toggle sidebar\" aria-expanded=\"true\" tabindex=\"-1\" aria-hidden=\"true\">{TRIGGER_ICON}</button></label><p>Toggle the sidebar with the button above.</p></div></div>"
        )), "{html}");
        // Explicit `active:` wins over "first item".
        let mut later = stub("sidebar", "Nav");
        later.items.push(extra("item", "Home"));
        let mut search = extra("item", "Search");
        search.config.insert("active".into(), "true".into());
        later.items.push(search);
        let html = render(&later);
        assert!(html.contains("data-active=\"false\" disabled>Home</button>"));
        assert!(
            html.contains("data-active=\"true\" aria-current=\"page\" disabled>Search</button>")
        );
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_nav_surf() {
        let c = emitted("Sidebar", &["Home", "Settings"]);
        let html = render(&c);
        assert!(html.contains("<aside data-slot=\"sidebar\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            reject_interact(&render(&emitted("Sidebar", &["Home"])));
        });
    }

    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"sidebar-wrapper\"]"));
        assert!(css.contains(
            "[data-audit-canvas] > [data-slot=\"sidebar-wrapper\"]:not([data-app-shell])"
        ));
        assert!(css.contains("width: 13rem; height: 14rem; min-height: 0;"));
        assert!(css.contains("[data-slot=\"sidebar-content\"] > [data-slot=\"scroll-area\"]"));
        assert!(css.contains("[data-slot=\"sidebar-menu-button\"][data-active=\"true\"]"));
        assert!(css.contains("font-size: 0.875rem; line-height: 1.25rem;"));
        assert!(css.contains("width: 16rem"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(!css.contains("zinc-"));
        // Docs parts and the native icon collapse.
        assert!(css.contains("[data-slot=\"sidebar-header\"], [data-slot=\"sidebar-footer\"] {\n  display: flex; flex-direction: column; gap: 0.5rem; padding: 0.5rem;"));
        assert!(css.contains("[data-slot=\"sidebar-group-label\"] {\n  display: flex; height: 1.75rem; flex-shrink: 0; align-items: center; padding: 0 0.5rem;"));
        assert!(css.contains("transition: width 200ms var(--ease-out-quart);"));
        assert!(css.contains("[data-slot=\"sidebar-wrapper\"]:has(> div > label > input:checked) > [data-slot=\"sidebar\"],\n[data-slot=\"sidebar-wrapper\"]:has(> [data-slot=\"app-shell-content\"] label > input:checked) > [data-slot=\"sidebar\"] {\n  width: 3rem;"));
        assert!(css.contains("[data-slot=\"sidebar-trigger\"] {\n  display: inline-flex; align-items: center; justify-content: center; width: 2rem; height: 2rem;"));
    }
}
