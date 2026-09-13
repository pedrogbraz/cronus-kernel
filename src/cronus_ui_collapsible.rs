//! Dedicated Collapsible renderer. Open static: trigger button (label) +
//! `<div data-slot="collapsible-content" data-state="open">` extra text.
//! Not interact `accordion("collapsible")` (`<details>` SURF, no collapsible-content).

use crate::cronus_ui_kit::{label_of, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let extra = texts(comp)
        .into_iter()
        .skip(1)
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"collapsible\"><button type=\"button\">{label}</button><div data-slot=\"collapsible-content\" data-state=\"open\">{extra}</div></div>"
    )
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
        assert!(!html.contains("<details"));
        assert!(!html.contains("<summary"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("<nav "));
    }

    #[test]
    fn root_is_open_content_not_accordion_details() {
        let mut c = stub("collapsible", "Show more");
        c.items.push(extra("Hidden details here."));
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"collapsible\">"));
        assert!(html.contains("<button type=\"button\">Show more</button>"));
        assert!(html.contains(
            "<div data-slot=\"collapsible-content\" data-state=\"open\">Hidden details here.</div>"
        ));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"collapsible\"><button type=\"button\">Show more</button><div data-slot=\"collapsible-content\" data-state=\"open\">Hidden details here.</div></div>"
        );
    }

    #[test]
    fn label_only_still_opens_empty_content() {
        let html = render(&stub("collapsible", "Show more"));
        assert!(html.contains("<button type=\"button\">Show more</button>"));
        assert!(html.contains("<div data-slot=\"collapsible-content\" data-state=\"open\"></div>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_accordion_surf() {
        let mut c = stub("collapsible", "Show more");
        c.items.push(extra("Hidden details here."));
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("collapsible", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("<div data-slot=\"collapsible\""));
        assert!(interact.contains("<details"));
        assert!(interact.contains("<summary"));
        assert!(interact.contains("style="));
        assert!(!interact.contains("data-slot=\"collapsible-content\""));
        assert!(html.contains("data-slot=\"collapsible-content\""));
        assert!(html.contains("data-state=\"open\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("collapsible", "Show more"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"collapsible-content\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"collapsible\"]"));
        assert!(css.contains("[data-slot=\"collapsible-content\"]"));
        assert!(css.contains("overflow: hidden"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
