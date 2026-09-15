//! Dedicated ScrollNav renderer. DOM matches React `ScrollNav` (Skiper 60):
//! `<div data-slot="scroll-nav">` + `<h1>` title + a row with the sticky
//! `<nav aria-label>` list (`<li><a href="#id"><p>`) and one
//! `<section id><h2><div>` per term.
//!
//! Terms are consecutive pairs of `text` / `item` lines: the term title, then
//! its body (`text "Privacy" -> "#privacy"` + `text "We store little."`). The
//! title line's link is the section id (default: slug of the title); ids keep
//! only `[A-Za-z0-9_-]`, so a fragment can never carry a scheme or markup.
//! Jumping is the native `#id` link. Scroll-spy needs JS, so the first term
//! stays active, as React renders before any scroll: `aria-current="location"`
//! and the indicator bar `<span>` on it.
//!
//! Viewport-driven styles (`md:`/`lg:`, `vh`) read `--cui-scroll-nav-*` first
//! (see `scroll-nav.css`); the audit canvas pins React's pane state there.

use crate::cronus_ui_kit::{attr_nonempty, esc, item};
use crate::parser::{ComponentItemNode, ComponentNode};

/// React `title` default.
const DEFAULT_TITLE: &str = "Terms & Conditions";

struct Term<'a> {
    id: String,
    title: &'a str,
    content: &'a str,
}

fn slug(text: &str) -> String {
    let mut out = String::new();
    for c in text.trim().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

/// Explicit id from the title line's link: `#` stripped, other characters
/// outside `[A-Za-z0-9_-]` become `-` (case kept, so React ids still match).
fn fragment_id(link: &str) -> String {
    link.trim()
        .trim_start_matches('#')
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

fn terms(comp: &ComponentNode) -> Vec<Term<'_>> {
    let lines: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "text" | "item") && !i.text.is_empty())
        .collect();
    lines
        .chunks(2)
        .map(|pair| {
            let head = pair[0];
            let id = head
                .link
                .as_deref()
                .map(fragment_id)
                .filter(|id| !id.is_empty())
                .unwrap_or_else(|| slug(&head.text));
            Term {
                id,
                title: &head.text,
                content: pair.get(1).map(|body| body.text.as_str()).unwrap_or(""),
            }
        })
        .collect()
}

pub fn render(comp: &ComponentNode) -> String {
    let title = item(comp, "label")
        .filter(|t| !t.is_empty())
        .or_else(|| attr_nonempty(comp, "title"))
        .unwrap_or(DEFAULT_TITLE);
    let nav = attr_nonempty(comp, "nav").unwrap_or("Sections");
    let terms = terms(comp);
    let links: String = terms
        .iter()
        .enumerate()
        .map(|(index, term)| {
            let (current, indicator) = if index == 0 {
                (" aria-current=\"location\"", "<span></span>")
            } else {
                ("", "")
            };
            format!(
                "<li><a href=\"#{}\"{current}>{indicator}<p>{}</p></a></li>",
                term.id,
                esc(term.title)
            )
        })
        .collect();
    let sections: String = terms
        .iter()
        .map(|term| {
            format!(
                "<section id=\"{}\"><h2>{}</h2><div>{}</div></section>",
                term.id,
                esc(term.title),
                esc(term.content)
            )
        })
        .collect();
    format!(
        "<div data-slot=\"scroll-nav\"><h1>{}</h1><div><nav aria-label=\"{}\"><ul>{links}</ul></nav><div>{sections}</div></div></div>",
        esc(title),
        esc(nav)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn text(body: &str, link: Option<&str>) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "text".into(),
            text: body.into(),
            link: link.map(String::from),
            tone: None,
            config: Default::default(),
        }
    }

    #[test]
    fn nav_links_and_sections_match_react_dom() {
        let mut c = stub("scroll-nav", "Terms");
        c.items.push(text("Acceptance", Some("#acceptance")));
        c.items.push(text("By using it.", None));
        c.items.push(text("Privacy data", None));
        c.items.push(text("We keep little.", None));
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"scroll-nav\"><h1>Terms</h1><div><nav aria-label=\"Sections\"><ul><li><a href=\"#acceptance\" aria-current=\"location\"><span></span><p>Acceptance</p></a></li><li><a href=\"#privacy-data\"><p>Privacy data</p></a></li></ul></nav><div><section id=\"acceptance\"><h2>Acceptance</h2><div>By using it.</div></section><section id=\"privacy-data\"><h2>Privacy data</h2><div>We keep little.</div></section></div></div></div>"
        );
        for bad in ["<script", "<style", " style=", " onclick=", "disabled"] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    #[test]
    fn defaults_title_nav_label_and_escaping() {
        let mut c = stub("scroll-nav", "");
        c.items[0].text.clear();
        c.props.insert("nav".into(), "On <this> page".into());
        c.items
            .push(text("A \"quoted\" & term", Some("x\" onmouseover=\"y")));
        c.items.push(text("<b>", None));
        c.items.push(text("Last without body", None));
        let html = render(&c);
        assert!(html.contains("<h1>Terms &amp; Conditions</h1>"), "{html}");
        assert!(
            html.contains("aria-label=\"On &lt;this&gt; page\""),
            "{html}"
        );
        assert!(
            html.contains("<a href=\"#x--onmouseover--y\" aria-current=\"location\">"),
            "{html}"
        );
        assert!(
            html.contains("<p>A &quot;quoted&quot; &amp; term</p>"),
            "{html}"
        );
        assert!(html.contains("<div>&lt;b&gt;</div>"), "{html}");
        assert!(
            html.contains(
                "<section id=\"last-without-body\"><h2>Last without body</h2><div></div></section>"
            ),
            "{html}"
        );
    }

    #[test]
    fn link_ids_never_carry_a_scheme() {
        let mut c = stub("scroll-nav", "T");
        c.items.push(text("Evil", Some("javascript:alert(1)")));
        c.items.push(text("Body", None));
        let html = render(&c);
        assert!(!html.contains("javascript:"), "{html}");
        assert!(html.contains("href=\"#javascript-alert-1-\""), "{html}");
        assert!(html.contains("id=\"javascript-alert-1-\""), "{html}");
    }

    #[test]
    fn emitted_fixture_pairs_become_terms() {
        let src = "app \"x\" { port 1 }\ncomponent ScrollNavDefault layout:inline style:scroll-nav {\n  label \"Terms & Conditions\"\n  text \"Acceptance\" -> \"#acceptance\"\n  text \"By using the service you agree.\"\n  text \"Privacy\" -> \"#privacy\"\n  text \"We store little.\"\n}\n";
        let comp = crate::parser::parse(src)
            .expect("parse")
            .into_iter()
            .find_map(|n| match n {
                crate::parser::AstNode::Component(c) => Some(c),
                _ => None,
            })
            .expect("component");
        let html = crate::cronus_ui_widgets::render(&comp).expect("family");
        assert!(html.contains("<h1>Terms &amp; Conditions</h1>"), "{html}");
        assert!(
            html.contains("<section id=\"acceptance\"><h2>Acceptance</h2><div>By using the service you agree.</div></section>"),
            "{html}"
        );
        assert!(
            html.contains("<a href=\"#privacy\"><p>Privacy</p></a>"),
            "{html}"
        );
    }

    #[test]
    fn chrome_breakpoints_read_overridable_properties() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("min-height: calc(var(--cui-scroll-nav-vh, 1vh) * 100);"));
        assert!(css.contains("flex-direction: var(--cui-scroll-nav-dir, row);"));
        assert!(css.contains("@media (min-width: 64rem)"));
        assert!(crate::cronus_ui_css::AUDIT_CSS.contains("--cui-scroll-nav-dir: row;"));
        assert_eq!(
            crate::cli::stub_renderer_gate::dedicated_fn_name("scroll-nav"),
            Some("cronus_ui_scroll_nav::render")
        );
    }
}
