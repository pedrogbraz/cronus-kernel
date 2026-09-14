//! Dedicated Breadcrumb renderer — mirrors React `Breadcrumb` composed as
//! links separated by the default chevron, the last crumb as the page.
//!
//! DOM: `<nav data-slot="breadcrumb" aria-label="breadcrumb">` >
//! `<ol data-slot="breadcrumb-list">` > `<li data-slot="breadcrumb-item">`
//! (`<a data-slot="breadcrumb-link">` or, last, `<span data-slot="breadcrumb-page"
//! aria-current="page">`) with `<li data-slot="breadcrumb-separator"
//! role="presentation" aria-hidden="true">` (lucide chevron-right) between.
//!
//! Crumbs: the non-label items (`text "…" -> "/href"`), else the label alone.
//! Hrefs come from the item link through `safe_url`, else `#`.

use crate::cronus_ui_kit::{esc, safe_url};
use crate::parser::ComponentNode;

const CHEVRON_RIGHT: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m9 18 6-6-6-6\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let named: Vec<_> = comp.items.iter().filter(|i| !i.text.is_empty()).collect();
    let rest: Vec<_> = named
        .iter()
        .copied()
        .filter(|i| i.item_type != "label" && i.item_type != "title")
        .collect();
    let crumbs = if rest.is_empty() { named } else { rest };
    let n = crumbs.len();
    let items = crumbs
        .into_iter()
        .enumerate()
        .map(|(i, c)| {
            let text = esc(&c.text);
            if i + 1 == n {
                format!(
                    "<li data-slot=\"breadcrumb-item\"><span aria-current=\"page\" data-slot=\"breadcrumb-page\">{text}</span></li>"
                )
            } else {
                let href = c.link.as_deref().map(safe_url).unwrap_or_else(|| "#".into());
                format!(
                    "<li data-slot=\"breadcrumb-item\"><a data-slot=\"breadcrumb-link\" href=\"{href}\">{text}</a></li><li role=\"presentation\" aria-hidden=\"true\" data-slot=\"breadcrumb-separator\">{CHEVRON_RIGHT}</li>"
                )
            }
        })
        .collect::<String>();
    format!(
        "<nav aria-label=\"breadcrumb\" data-slot=\"breadcrumb\"><ol data-slot=\"breadcrumb-list\">{items}</ol></nav>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn crumb(text: &str, link: Option<&str>) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "text".into(),
            text: text.into(),
            link: link.map(Into::into),
            tone: None,
            config: Default::default(),
        }
    }

    #[test]
    fn fixture_matches_react_dom() {
        let mut c = stub("breadcrumb", "default");
        for t in ["Home", "Components", "Button"] {
            c.items.push(crumb(t, None));
        }
        assert_eq!(
            render(&c),
            format!(
                "<nav aria-label=\"breadcrumb\" data-slot=\"breadcrumb\"><ol data-slot=\"breadcrumb-list\"><li data-slot=\"breadcrumb-item\"><a data-slot=\"breadcrumb-link\" href=\"#\">Home</a></li><li role=\"presentation\" aria-hidden=\"true\" data-slot=\"breadcrumb-separator\">{CHEVRON_RIGHT}</li><li data-slot=\"breadcrumb-item\"><a data-slot=\"breadcrumb-link\" href=\"#\">Components</a></li><li role=\"presentation\" aria-hidden=\"true\" data-slot=\"breadcrumb-separator\">{CHEVRON_RIGHT}</li><li data-slot=\"breadcrumb-item\"><span aria-current=\"page\" data-slot=\"breadcrumb-page\">Button</span></li></ol></nav>"
            )
        );
    }

    #[test]
    fn links_are_safe_and_label_only_is_page() {
        let mut c = stub("breadcrumb", "ignored");
        c.items.push(crumb("Docs", Some("/docs")));
        c.items.push(crumb("Evil", Some("javascript:alert(1)")));
        c.items.push(crumb("Here", None));
        let html = render(&c);
        assert!(html.contains("href=\"/docs\">Docs</a>"));
        assert!(html.contains("href=\"#\">Evil</a>"));
        assert!(!html.contains("ignored"));
        let only = render(&stub("breadcrumb", "Home"));
        assert!(only.contains("data-slot=\"breadcrumb-page\">Home</span>"));
        assert!(!only.contains("breadcrumb-separator"));
    }

    #[test]
    fn chrome_is_text_sm_tertiary() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"breadcrumb-list\"] {\n  display: flex; flex-wrap: wrap; align-items: center; gap: 0.375rem;"));
        assert!(css.contains("[data-slot=\"breadcrumb-separator\"] > svg"));
    }
}
