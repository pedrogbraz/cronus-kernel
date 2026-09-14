//! Dedicated DescriptionList renderer. DOM matches React (stacked, md):
//! `<dl data-slot="description-list">` plus `description-item` wrapping
//! `description-term` / `description-details` from paired content texts
//! (odd=term, even=details). The `label` names the widget (it becomes the
//! `aria-label` fallback) and is never rendered as a term. Not interact styled
//! `<dl>` rows without term/details slots, not catalog `display()` `<section>`.

use crate::cronus_ui_kit::esc;
use crate::parser::ComponentNode;

/// Item kinds that name the widget rather than carry list content.
const NAME_KINDS: &[&str] = &["label", "title"];

pub fn render(comp: &ComponentNode) -> String {
    let rows = pairs(comp)
        .into_iter()
        .map(|(term, details)| {
            format!(
                "<div data-slot=\"description-item\"><dt data-slot=\"description-term\">{term}</dt><dd data-slot=\"description-details\">{details}</dd></div>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let aria = aria_label(comp)
        .map(|v| format!(" aria-label=\"{}\"", esc(v)))
        .unwrap_or_default();
    format!("<dl data-slot=\"description-list\"{aria}>{rows}</dl>")
}

fn aria_label(comp: &ComponentNode) -> Option<&str> {
    comp.props
        .get("aria-label")
        .or_else(|| comp.items.iter().find_map(|i| i.config.get("aria-label")))
        .map(String::as_str)
        .filter(|v| !v.is_empty())
}

fn content(comp: &ComponentNode) -> Vec<String> {
    let texts: Vec<String> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty() && !NAME_KINDS.contains(&i.item_type.as_str()))
        .map(|i| esc(&i.text))
        .collect();
    if !texts.is_empty() {
        return texts;
    }
    // Label-only list: the label is the single term.
    comp.items
        .iter()
        .find(|i| !i.text.is_empty())
        .map(|i| vec![esc(&i.text)])
        .unwrap_or_default()
}

fn pairs(comp: &ComponentNode) -> Vec<(String, String)> {
    content(comp)
        .chunks(2)
        .map(|pair| (pair[0].clone(), pair.get(1).cloned().unwrap_or_default()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const DISPLAY_SURF: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem";
    const INTERACT_ROW: &str =
        "display:flex;justify-content:space-between;gap:1rem;font-size:0.875rem";

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn list(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("description-list", items.first().copied().unwrap_or("Name"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_stub(html: &str) {
        assert!(!html.contains("<section"));
        assert!(!html.contains("<nav"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("v-show"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains(DISPLAY_SURF));
        assert!(!html.contains(INTERACT_ROW));
        assert!(!html.contains("display("));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_is_dl_with_paired_items() {
        let html = render(&list(&["Order", "#10245", "Status", "Paid"]));
        reject_stub(&html);
        assert_eq!(
            html,
            "<dl data-slot=\"description-list\"><div data-slot=\"description-item\"><dt data-slot=\"description-term\">Order</dt><dd data-slot=\"description-details\">#10245</dd></div><div data-slot=\"description-item\"><dt data-slot=\"description-term\">Status</dt><dd data-slot=\"description-details\">Paid</dd></div></dl>"
        );
    }

    /// Emitted audit source: `label "Details"` + four texts + `aria-label:"Details"`.
    /// The label names the widget; it must not shift the term/details pairing.
    #[test]
    fn label_names_the_list_and_is_not_a_term() {
        let mut c = stub("description-list", "Details");
        for t in ["Name", "Ada", "Status", "Paid"] {
            c.items.push(extra("text", t));
        }
        c.items
            .last_mut()
            .unwrap()
            .config
            .insert("aria-label".into(), "Details".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<dl data-slot=\"description-list\" aria-label=\"Details\"><div data-slot=\"description-item\"><dt data-slot=\"description-term\">Name</dt><dd data-slot=\"description-details\">Ada</dd></div><div data-slot=\"description-item\"><dt data-slot=\"description-term\">Status</dt><dd data-slot=\"description-details\">Paid</dd></div></dl>"
        );
        reject_stub(&html);
    }

    #[test]
    fn odd_leftover_term_has_empty_details() {
        let html = render(&list(&["Name", "Ada", "Role"]));
        assert_eq!(html.matches("data-slot=\"description-item\"").count(), 2);
        assert!(html.contains("data-slot=\"description-term\">Role</dt>"));
        assert!(html.contains("data-slot=\"description-details\"></dd>"));
        reject_stub(&html);
    }

    #[test]
    fn label_only_still_emits_one_item() {
        let html = render(&stub("description-list", "Order"));
        assert_eq!(html.matches("data-slot=\"description-item\"").count(), 1);
        assert!(html.contains("data-slot=\"description-term\">Order</dt>"));
        assert!(html.contains("data-slot=\"description-details\"></dd>"));
        reject_stub(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("description-list", "A <B> & \"C\""));
        assert!(
            html.contains("data-slot=\"description-term\">A &lt;B&gt; &amp; &quot;C&quot;</dt>")
        );
        reject_stub(&html);
    }

    #[test]
    fn skips_interact_styled_dl_and_display_surf() {
        let c = list(&["Order", "#10245"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("description-list", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains(INTERACT_ROW));
        assert!(!interact.contains("data-slot=\"description-term\""));
        assert!(html.contains("data-slot=\"description-term\""));
        reject_stub(&html);
        assert_eq!(
            dedicated_fn_name("description-list"),
            Some("cronus_ui_description_list::render")
        );
        assert_eq!(
            renderer_kind("description-list"),
            RendererKind::Dedicated("cronus_ui_description_list::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&list(&["Order", "#10245"]));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"description-term\""));
        });
    }

    /// Geometry parity (Wave 1t): React `text-sm` pairs 14px with a 20px line
    /// box, groups stack with `mt-4`, details sit `mt-1` below the term, and the
    /// audit harness sizes the list `w-72` (18rem).
    #[test]
    fn chrome_matches_react_stacked_md_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"description-list\"] {\n  display: block; width: 18rem; max-width: 100%; min-width: 0; margin: 0;\n  font-size: 0.875rem; line-height: 1.25rem;\n}"
        ));
        assert!(css.contains(
            "[data-slot=\"description-item\"] + [data-slot=\"description-item\"] { margin-top: 1rem; }"
        ));
        assert!(css.contains("[data-slot=\"description-term\"] { font-weight: 500; color: var(--cronus-fg-secondary); }"));
        assert!(css.contains("var(--cronus-fg)"));
        assert!(!css.contains("zinc-"));
    }
}
