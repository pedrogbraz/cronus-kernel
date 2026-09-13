//! Dedicated DescriptionList renderer. DOM matches React:
//! `<dl data-slot="description-list">` plus `description-item` wrapping
//! `description-term` / `description-details` from paired texts (odd=term,
//! even=details; label+texts). Not interact styled `<dl>` rows without
//! term/details slots, not catalog `display()` `<section>`.

use crate::cronus_ui_kit::texts;
use crate::parser::ComponentNode;

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
    format!("<dl data-slot=\"description-list\">{rows}</dl>")
}

fn pairs(comp: &ComponentNode) -> Vec<(String, String)> {
    let texts = texts(comp);
    let mut out = Vec::new();
    let mut i = 0;
    while i < texts.len() {
        let term = texts[i].clone();
        let details = if i + 1 < texts.len() {
            texts[i + 1].clone()
        } else {
            String::new()
        };
        out.push((term, details));
        i += 2;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const DISPLAY_SURF: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem";
    const INTERACT_ROW: &str = "display:flex;justify-content:space-between;gap:1rem;font-size:0.875rem";

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
        assert!(html.starts_with("<dl data-slot=\"description-list\">"));
        assert!(html.contains(
            "<div data-slot=\"description-item\"><dt data-slot=\"description-term\">Order</dt><dd data-slot=\"description-details\">#10245</dd></div>"
        ));
        assert!(html.contains(
            "<div data-slot=\"description-item\"><dt data-slot=\"description-term\">Status</dt><dd data-slot=\"description-details\">Paid</dd></div>"
        ));
        assert_eq!(html.matches("data-slot=\"description-item\"").count(), 2);
        assert_eq!(html.matches("data-slot=\"description-term\"").count(), 2);
        assert_eq!(html.matches("data-slot=\"description-details\"").count(), 2);
        reject_stub(&html);
        assert_eq!(
            html,
            "<dl data-slot=\"description-list\"><div data-slot=\"description-item\"><dt data-slot=\"description-term\">Order</dt><dd data-slot=\"description-details\">#10245</dd></div><div data-slot=\"description-item\"><dt data-slot=\"description-term\">Status</dt><dd data-slot=\"description-details\">Paid</dd></div></dl>"
        );
    }

    #[test]
    fn label_plus_texts_pair_as_term_and_details() {
        let mut c = stub("description-list", "Name");
        c.items.push(extra("text", "Ada"));
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"description-item\"").count(), 1);
        assert!(html.contains("data-slot=\"description-term\">Name</dt>"));
        assert!(html.contains("data-slot=\"description-details\">Ada</dd>"));
        reject_stub(&html);
    }

    #[test]
    fn odd_leftover_term_has_empty_details() {
        let html = render(&list(&["Name", "Ada", "Role"]));
        assert_eq!(html.matches("data-slot=\"description-item\"").count(), 2);
        assert!(html.contains("data-slot=\"description-term\">Name</dt>"));
        assert!(html.contains("data-slot=\"description-details\">Ada</dd>"));
        assert!(html.contains("data-slot=\"description-term\">Role</dt>"));
        assert!(html.contains("data-slot=\"description-details\"></dd>"));
        reject_stub(&html);
    }

    #[test]
    fn label_only_still_emits_one_item() {
        let html = render(&stub("description-list", "Order"));
        assert!(html.starts_with("<dl data-slot=\"description-list\">"));
        assert_eq!(html.matches("data-slot=\"description-item\"").count(), 1);
        assert!(html.contains("data-slot=\"description-term\">Order</dt>"));
        assert!(html.contains("data-slot=\"description-details\"></dd>"));
        reject_stub(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("description-list", "A <B> & \"C\""));
        assert!(html.contains(
            "data-slot=\"description-term\">A &lt;B&gt; &amp; &quot;C&quot;</dt>"
        ));
        reject_stub(&html);
    }

    #[test]
    fn skips_interact_styled_dl_and_display_surf() {
        let c = list(&["Order", "#10245"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("description-list", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<dl data-slot=\"description-list\""));
        assert!(interact.contains("style="));
        assert!(interact.contains(INTERACT_ROW));
        assert!(!interact.contains("data-slot=\"description-term\""));
        assert!(!interact.contains("data-slot=\"description-details\""));
        assert!(!interact.contains("data-slot=\"description-item\""));
        assert!(html.contains("data-slot=\"description-term\""));
        assert!(html.contains("data-slot=\"description-details\""));
        assert!(!html.contains("<section"));
        assert!(!html.contains(DISPLAY_SURF));
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
            assert!(html.contains("data-slot=\"description-details\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"description-list\"]"));
        assert!(css.contains("[data-slot=\"description-item\"]"));
        assert!(css.contains("[data-slot=\"description-term\"]"));
        assert!(css.contains("[data-slot=\"description-details\"]"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(css.contains("var(--cronus-fg)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains(DISPLAY_SURF));
    }
}
