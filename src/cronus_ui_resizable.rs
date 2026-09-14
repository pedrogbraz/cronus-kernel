//! Dedicated Resizable renderer. Static two-panel split, no JS drag.
//! DOM matches React `ResizablePanelGroup` (react-resizable-panels):
//! `<div data-slot="resizable-panel-group" data-panel-group-direction aria-label>`
//! with two `<div data-panel data-panel-size="50.0">` panels (React panels carry no
//! data-slot) around `<div data-slot="resizable-handle" role="separator">`.
//! The handle is not focusable: without JS it cannot resize, so it only separates.
//! The `label` names the group; it is never a panel.
//! Not catalog `display()` SURF (`<section>` without panel-group).

use crate::cronus_ui_kit::{choice_texts, esc, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let (left, right) = panels(comp);
    let aria = comp
        .props
        .get("aria-label")
        .or_else(|| comp.items.iter().find_map(|i| i.config.get("aria-label")))
        .filter(|s| !s.is_empty())
        .map(|a| format!(" aria-label=\"{}\"", esc(a)))
        .unwrap_or_default();
    format!(
        "<div data-slot=\"resizable-panel-group\" data-panel-group-direction=\"horizontal\"{aria}><div data-panel=\"\" data-panel-size=\"50.0\">{left}</div><div data-slot=\"resizable-handle\" role=\"separator\" aria-valuenow=\"50\" aria-valuemin=\"0\" aria-valuemax=\"100\" data-panel-group-direction=\"horizontal\"></div><div data-panel=\"\" data-panel-size=\"50.0\">{right}</div></div>"
    )
}

fn panels(comp: &ComponentNode) -> (String, String) {
    let mut items = choice_texts(comp);
    if items.is_empty() {
        items = comp
            .items
            .iter()
            .filter(|i| !matches!(i.item_type.as_str(), "label" | "title") && !i.text.is_empty())
            .map(|i| esc(&i.text))
            .collect();
    }
    let left = items.first().cloned().unwrap_or_else(|| label_of(comp));
    let right = items.get(1).cloned().unwrap_or_default();
    (left, right)
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
        assert!(!html.starts_with("<section"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("tabindex"));
        assert!(!html.contains("max-height:12rem;overflow:auto"));
        assert!(html.contains("data-slot=\"resizable-panel-group\""));
        assert!(html.contains("data-slot=\"resizable-handle\""));
    }

    #[test]
    fn emitted_fixture_matches_react_dom_label_is_not_a_panel() {
        let mut c = stub("resizable", "Panels");
        c.items.push(extra("text", "One"));
        c.items.push(extra("text", "Two"));
        c.items[2].config.insert("aria-label".into(), "Panels".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"resizable-panel-group\" data-panel-group-direction=\"horizontal\" aria-label=\"Panels\"><div data-panel=\"\" data-panel-size=\"50.0\">One</div><div data-slot=\"resizable-handle\" role=\"separator\" aria-valuenow=\"50\" aria-valuemin=\"0\" aria-valuemax=\"100\" data-panel-group-direction=\"horizontal\"></div><div data-panel=\"\" data-panel-size=\"50.0\">Two</div></div>"
        );
        assert!(!html.contains("data-slot=\"resizable\""));
        reject_interact(&html);
    }

    #[test]
    fn label_only_uses_label_for_first_panel() {
        let html = render(&stub("resizable", "Sidebar"));
        assert!(html.contains("<div data-panel=\"\" data-panel-size=\"50.0\">Sidebar</div>"));
        assert_eq!(html.matches("data-panel=\"\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn skips_display_and_scroll_surf() {
        let mut c = stub("resizable", "Split");
        c.items.push(extra("item", "Sidebar"));
        c.items.push(extra("item", "Main"));
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("resizable", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("max-height:12rem;overflow:auto"));
        assert!(!crate::cli::stub_renderer_gate::looks_like_interact_generic(&html));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            reject_interact(&render(&stub("resizable", "Sidebar")));
        });
    }

    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(!css.contains("[data-slot=\"resizable\"] {"));
        assert!(css.contains("[data-slot=\"resizable-panel-group\"] > [data-panel]"));
        assert!(css.contains("flex: 50 1 0px; overflow: hidden;"));
        assert!(css.contains("[data-slot=\"resizable-handle\"]::after"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
    }
}
