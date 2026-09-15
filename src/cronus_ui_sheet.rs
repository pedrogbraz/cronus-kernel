//! Dedicated Sheet renderer. Always-open static: trigger button (label) plus
//! `<div data-slot="sheet-content">` with `sheet-title` and optional
//! `sheet-description`. Not interact `dialog("sheet")` native `<dialog>` +
//! `showModal()` + SURF.

use crate::cronus_ui_kit::{esc, item, label_of, texts, widget_id};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let trigger = texts(comp)
        .first()
        .cloned()
        .unwrap_or_else(|| label_of(comp));
    let title = item(comp, "title")
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| trigger.clone());
    let desc = extras(comp, &trigger, &title);
    let desc_html = if desc.is_empty() {
        String::new()
    } else {
        format!("<div data-slot=\"sheet-description\">{desc}</div>")
    };
    let trigger_id = widget_id(comp, "trigger");
    let pop_id = widget_id(comp, "sheet");
    format!(
        "<button type=\"button\" id=\"{trigger_id}\" popovertarget=\"{pop_id}\">{trigger}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"sheet-content\" anchor=\"{trigger_id}\"><div data-slot=\"sheet-title\">{title}</div>{desc_html}</div>"
    )
}

fn extras(comp: &ComponentNode, trigger: &str, title: &str) -> String {
    texts(comp)
        .into_iter()
        .filter(|t| t != trigger && t != title)
        .collect::<Vec<_>>()
        .join("")
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
        assert!(!html.contains("<dialog"));
        assert!(!html.contains("showModal"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("max-width:28rem"));
        assert!(!html.contains("role=\"dialog\""));
        assert!(!html.contains("data-slot=\"sheet\">"));
    }

    #[test]
    fn trigger_and_always_open_content() {
        let mut c = stub("sheet", "Filters");
        c.items.push(extra("text", "Narrow the list."));
        let html = render(&c);
        assert!(html.contains("<button type=\"button\""));
        assert!(html.contains("popovertarget="));
        assert!(html.contains(">Filters</button>"));
        assert!(html.contains("data-slot=\"sheet-content\""));
        assert!(html.contains("popover=\"auto\""));
        assert!(html.contains("<div data-slot=\"sheet-title\">Filters</div>"));
        assert!(html.contains("<div data-slot=\"sheet-description\">Narrow the list.</div>"));
        reject_interact(&html);
    }

    #[test]
    fn title_item_is_panel_title() {
        let mut c = stub("sheet", "Open");
        c.items.push(extra("title", "Filters"));
        c.items.push(extra("text", "Narrow the list."));
        let html = render(&c);
        assert!(html.contains(">Open</button>"));
        assert!(html.contains("<div data-slot=\"sheet-title\">Filters</div>"));
        assert!(html.contains("<div data-slot=\"sheet-description\">Narrow the list.</div>"));
        reject_interact(&html);
    }

    #[test]
    fn description_is_optional() {
        let html = render(&stub("sheet", "Filters"));
        assert!(html.contains(">Filters</button>"));
        assert!(html.contains("<div data-slot=\"sheet-title\">Filters</div>"));
        assert!(!html.contains("data-slot=\"sheet-description\""));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_dialog_surf() {
        let mut c = stub("sheet", "Filters");
        c.items.push(extra("text", "Narrow the list."));
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("sheet", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<dialog data-slot=\"sheet-content\""));
        assert!(interact.contains("showModal()"));
        assert!(interact.contains("onclick="));
        assert!(interact.contains("style="));
        assert!(interact.contains("max-width:28rem"));
        assert!(!html.contains("<dialog"));
        assert!(!html.contains("showModal"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("sheet", "Filters"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"sheet-content\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"sheet-content\"]"));
        assert!(css.contains("[data-slot=\"sheet-title\"]"));
        assert!(css.contains("[data-slot=\"sheet-description\"]"));
        assert!(css.contains("z-index: 50"));
        assert!(css.contains("max-width: 24rem"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains("showModal"));
    }
}
