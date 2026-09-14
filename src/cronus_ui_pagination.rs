//! Dedicated Pagination renderer — data-driven, mirrors React `Pagination`
//! composed as Previous · page links (with ellipses) · Next.
//!
//! DOM: `<nav data-slot="pagination" aria-label="Pagination">` >
//! `<ul data-slot="pagination-content">` > `<li data-slot="pagination-item">`
//! each holding `pagination-previous` / `pagination-link` / `pagination-ellipsis`
//! / `pagination-next`. Links are real `<a href>` — pagination works without JS.
//!
//! Props: `total` (pages, default 1), `current` (1-based, clamped),
//! `href-template` (default `?page={n}`; `{n}` is replaced, the result goes
//! through `safe_url`). The window is [`page_window`].

use crate::cronus_ui_kit::{attr_nonempty, attr_num, safe_url};
use crate::parser::ComponentNode;

const CHEVRON_LEFT: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m15 18-6-6 6-6\"></path></svg>";
const CHEVRON_RIGHT: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m9 18 6-6-6-6\"></path></svg>";
const ELLIPSIS: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><circle cx=\"12\" cy=\"12\" r=\"1\"></circle><circle cx=\"19\" cy=\"12\" r=\"1\"></circle><circle cx=\"5\" cy=\"12\" r=\"1\"></circle></svg>";

pub const DEFAULT_HREF_TEMPLATE: &str = "?page={n}";
/// Upper bound on `total` so a hostile value cannot blow up the loop.
const MAX_PAGES: u32 = 100_000;

/// `None` = ellipsis. Every page up to 7 pages; otherwise first, last,
/// current ± 1 and one ellipsis per gap. Mirrored by the audit React fixture.
pub fn page_window(total: u32, current: u32) -> Vec<Option<u32>> {
    let total = total.clamp(1, MAX_PAGES);
    let current = current.clamp(1, total);
    if total <= 7 {
        return (1..=total).map(Some).collect();
    }
    let start = (current.saturating_sub(1)).max(2);
    let end = (current + 1).min(total - 1);
    let mut out = vec![Some(1)];
    if start > 2 {
        out.push(None);
    }
    out.extend((start..=end).map(Some));
    if end < total - 1 {
        out.push(None);
    }
    out.push(Some(total));
    out
}

pub fn render(comp: &ComponentNode) -> String {
    let total = attr_num::<u32>(comp, "total")
        .unwrap_or(1)
        .clamp(1, MAX_PAGES);
    let current = attr_num::<u32>(comp, "current")
        .unwrap_or(1)
        .clamp(1, total);
    let template = attr_nonempty(comp, "href-template").unwrap_or(DEFAULT_HREF_TEMPLATE);
    let href = |n: u32| safe_url(&template.replace("{n}", &n.to_string()));
    let li = |inner: String| format!("<li data-slot=\"pagination-item\">{inner}</li>");
    let mut items = vec![li(format!(
        "<a aria-label=\"Go to previous page\" data-slot=\"pagination-previous\" href=\"{}\">{CHEVRON_LEFT}<span>Previous</span></a>",
        href(current.saturating_sub(1).max(1))
    ))];
    for entry in page_window(total, current) {
        items.push(li(match entry {
            Some(n) if n == current => format!(
                "<a aria-current=\"page\" data-slot=\"pagination-link\" data-active=\"true\" href=\"{}\">{n}</a>",
                href(n)
            ),
            Some(n) => format!(
                "<a data-slot=\"pagination-link\" data-active=\"false\" href=\"{}\">{n}</a>",
                href(n)
            ),
            None => format!(
                "<span aria-hidden=\"true\" data-slot=\"pagination-ellipsis\">{ELLIPSIS}<span>More pages</span></span>"
            ),
        }));
    }
    items.push(li(format!(
        "<a aria-label=\"Go to next page\" data-slot=\"pagination-next\" href=\"{}\"><span>Next</span>{CHEVRON_RIGHT}</a>",
        href((current + 1).min(total))
    )));
    format!(
        "<nav aria-label=\"Pagination\" data-slot=\"pagination\"><ul data-slot=\"pagination-content\">{}</ul></nav>",
        items.join("")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn pages(total: &str, current: &str) -> ComponentNode {
        let mut c = stub("pagination", "Pages");
        c.props.insert("total".into(), total.into());
        c.props.insert("current".into(), current.into());
        c
    }

    #[test]
    fn window_small_and_large() {
        assert_eq!(page_window(5, 2), (1..=5).map(Some).collect::<Vec<_>>());
        assert_eq!(
            page_window(10, 5),
            vec![Some(1), None, Some(4), Some(5), Some(6), None, Some(10)]
        );
        assert_eq!(page_window(10, 1), vec![Some(1), Some(2), None, Some(10)]);
        assert_eq!(page_window(10, 10), vec![Some(1), None, Some(9), Some(10)]);
        assert_eq!(
            page_window(10, 3),
            vec![Some(1), Some(2), Some(3), Some(4), None, Some(10)]
        );
        assert_eq!(page_window(0, 9), vec![Some(1)]);
    }

    #[test]
    fn data_driven_links() {
        let html = render(&pages("5", "2"));
        assert_eq!(html.matches("data-slot=\"pagination-link\"").count(), 5);
        assert_eq!(html.matches("data-slot=\"pagination-item\"").count(), 7);
        assert!(html.contains("<a aria-current=\"page\" data-slot=\"pagination-link\" data-active=\"true\" href=\"?page=2\">2</a>"));
        assert!(html.contains(
            "<a data-slot=\"pagination-link\" data-active=\"false\" href=\"?page=5\">5</a>"
        ));
        assert!(html.contains("data-slot=\"pagination-previous\" href=\"?page=1\""));
        assert!(html.contains("data-slot=\"pagination-next\" href=\"?page=3\""));
        assert_eq!(html.matches("aria-current").count(), 1);
        assert!(!html.contains("pagination-ellipsis"));
        let many = render(&pages("10", "5"));
        assert_eq!(many.matches("data-slot=\"pagination-ellipsis\"").count(), 2);
    }

    #[test]
    fn clamps_and_defaults() {
        let html = render(&stub("pagination", "Pages"));
        assert_eq!(html.matches("data-slot=\"pagination-link\"").count(), 1);
        assert!(html.contains("data-slot=\"pagination-previous\" href=\"?page=1\""));
        assert!(html.contains("data-slot=\"pagination-next\" href=\"?page=1\""));
        let html = render(&pages("3", "99"));
        assert!(html.contains("data-active=\"true\" href=\"?page=3\">3</a>"));
        let html = render(&pages("-4", "x"));
        assert_eq!(html.matches("data-slot=\"pagination-link\"").count(), 1);
    }

    #[test]
    fn href_template_goes_through_safe_url() {
        let mut c = pages("3", "1");
        c.props
            .insert("href-template".into(), "/posts/{n}?q=\"x\"".into());
        let html = render(&c);
        assert!(html.contains("href=\"/posts/2?q=&quot;x&quot;\">2</a>"));
        c.props
            .insert("href-template".into(), "javascript:alert({n})".into());
        let html = render(&c);
        assert!(!html.contains("javascript:"));
        assert!(html.contains("href=\"#\">2</a>"));
    }

    #[test]
    fn chrome_matches_button_variants() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"pagination-content\"] {\n  display: flex; align-items: center; gap: 0.25rem;"));
        assert!(css.contains("[data-slot=\"pagination-link\"][data-active=\"true\"]"));
        assert!(!css.contains("[data-slot=\"pagination-link\"][aria-current=\"page\"]"));
    }
}
