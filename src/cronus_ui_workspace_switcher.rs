//! Dedicated WorkspaceSwitcher renderer. Static always-open:
//! `<button type="button" data-slot="workspace-switcher">` plus
//! `<div data-slot="workspace-switcher-content">` with each text item.
//! Family slot is on the trigger (React). Not interact `nav("workspace-switcher")`
//! SURF `<nav>` and not `popover()` `<details>`.

use crate::cronus_ui_kit::{choice_texts, label_of, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let workspaces = workspaces(comp);
    let current = workspaces
        .first()
        .cloned()
        .unwrap_or_else(|| label_of(comp));
    let items = workspaces
        .into_iter()
        .map(|t| format!("<div data-slot=\"workspace-switcher-item\">{t}</div>"))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<button type=\"button\" data-slot=\"workspace-switcher\" aria-label=\"Switch workspace, {current}\">{current}</button><div data-slot=\"workspace-switcher-content\">{items}</div>"
    )
}

fn workspaces(comp: &ComponentNode) -> Vec<String> {
    let choices = choice_texts(comp);
    if !choices.is_empty() {
        return choices;
    }
    texts(comp)
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

    fn bar(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub(
            "workspace-switcher",
            items.first().copied().unwrap_or("Acme"),
        );
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<nav"));
        assert!(!html.contains("<details"));
        assert!(!html.contains("<summary"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!html.contains("position:absolute;z-index:20"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("nav("));
        assert!(!html.contains("popover("));
    }

    #[test]
    fn root_is_trigger_with_always_open_content_not_nav() {
        let html = render(&bar(&["Acme", "Globex"]));
        assert!(html.starts_with(
            "<button type=\"button\" data-slot=\"workspace-switcher\" aria-label=\"Switch workspace, Acme\">Acme</button>"
        ));
        assert!(html.contains("<div data-slot=\"workspace-switcher-content\">"));
        assert!(html.contains("<div data-slot=\"workspace-switcher-item\">Acme</div>"));
        assert!(html.contains("<div data-slot=\"workspace-switcher-item\">Globex</div>"));
        assert_eq!(html.matches("data-slot=\"workspace-switcher-item\"").count(), 2);
        reject_interact(&html);
        assert_eq!(
            html,
            "<button type=\"button\" data-slot=\"workspace-switcher\" aria-label=\"Switch workspace, Acme\">Acme</button><div data-slot=\"workspace-switcher-content\"><div data-slot=\"workspace-switcher-item\">Acme</div><div data-slot=\"workspace-switcher-item\">Globex</div></div>"
        );
    }

    #[test]
    fn extra_text_items_become_workspaces() {
        let mut c = stub("workspace-switcher", "Acme");
        c.items.push(extra("text", "Globex"));
        c.items.push(extra("text", "Initech"));
        let html = render(&c);
        assert!(html.contains("aria-label=\"Switch workspace, Acme\">Acme</button>"));
        assert!(html.contains("data-slot=\"workspace-switcher-item\">Acme</div>"));
        assert!(html.contains("data-slot=\"workspace-switcher-item\">Globex</div>"));
        assert!(html.contains("data-slot=\"workspace-switcher-item\">Initech</div>"));
        assert_eq!(html.matches("data-slot=\"workspace-switcher-item\"").count(), 3);
        reject_interact(&html);
    }

    #[test]
    fn field_label_is_not_a_workspace_when_items_exist() {
        let mut c = stub("workspace-switcher", "Orgs");
        c.items.push(extra("item", "Acme"));
        c.items.push(extra("item", "Globex"));
        let html = render(&c);
        assert!(html.contains("aria-label=\"Switch workspace, Acme\">Acme</button>"));
        assert!(html.contains("data-slot=\"workspace-switcher-item\">Acme</div>"));
        assert!(html.contains("data-slot=\"workspace-switcher-item\">Globex</div>"));
        assert!(!html.contains(">Orgs</button>"));
        assert!(!html.contains(">Orgs</div>"));
        assert_eq!(html.matches("data-slot=\"workspace-switcher-item\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_opens_content() {
        let html = render(&stub("workspace-switcher", "Acme"));
        assert!(html.contains(
            "<button type=\"button\" data-slot=\"workspace-switcher\" aria-label=\"Switch workspace, Acme\">Acme</button>"
        ));
        assert!(html.contains("<div data-slot=\"workspace-switcher-content\">"));
        assert!(html.contains("<div data-slot=\"workspace-switcher-item\">Acme</div>"));
        assert_eq!(html.matches("data-slot=\"workspace-switcher-item\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_nav_surf() {
        let c = bar(&["Acme", "Globex"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("workspace-switcher", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<nav data-slot=\"workspace-switcher\""));
        assert!(interact.contains("style="));
        assert!(interact.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!interact.contains("data-slot=\"workspace-switcher-content\""));
        assert!(!interact.contains("data-slot=\"workspace-switcher-item\""));
        assert!(html.contains("data-slot=\"workspace-switcher-content\""));
        assert!(html.contains("data-slot=\"workspace-switcher-item\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&bar(&["Acme"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"workspace-switcher\"]"));
        assert!(css.contains("[data-slot=\"workspace-switcher-content\"]"));
        assert!(css.contains("[data-slot=\"workspace-switcher-item\"]"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("min-width: 14rem"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
