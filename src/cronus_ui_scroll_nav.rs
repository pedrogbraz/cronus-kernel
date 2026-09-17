//! Dedicated ScrollNav renderer. DOM matches React `ScrollNav` (Skiper 60):
//! `<div data-slot="scroll-nav">` + `<h1>` title + a row with the sticky
//! `<nav aria-label>` list (`<li><a href="#id"><p>`) and one
//! `<section id><h2><div><p>…</p></div>` per term.
//!
//! Terms: an `item "Title" -> "#id"` line starts a term and the `text` lines
//! after it are its paragraphs. Without `item` lines, consecutive pairs of
//! `text` lines are (title, body) — the audit emitter's shape. The title
//! line's link is the section id (default: slug of the title); ids keep only
//! `[A-Za-z0-9_-]`, so a fragment can never carry a scheme or markup.
//! Jumping is the native `#id` link.
//!
//! `viewport:"…"` wraps the whole thing in the docs' named, keyboard-focusable
//! scroll region (`h-[28rem] rounded-3xl bg-surface-inset`). React spies the
//! section in view with an IntersectionObserver (`rootMargin
//! -20% 0px -65% 0px`); inside a viewport the kernel does the same with named
//! CSS view timelines (`view-timeline-inset: 20% 65%` on each section,
//! `timeline-scope` on the row) driving each row's opacity and indicator bar —
//! zero JS. Without a viewport the first term stays active, as React renders
//! before any scroll: `aria-current="location"` and the indicator `<span>`.
//!
//! Viewport-driven styles (`md:`/`lg:`, `vh`) read `--cui-scroll-nav-*` first
//! (see `scroll-nav.css`); the audit canvas pins React's pane state there.

use crate::cronus_ui_kit::{attr_nonempty, esc, item};
use crate::parser::{ComponentItemNode, ComponentNode};

/// React `title` default.
const DEFAULT_TITLE: &str = "Terms & Conditions";

/// Sections a viewport can spy on (one named view timeline each).
pub const SPY_TIMELINES: usize = 8;

struct Term<'a> {
    id: String,
    title: &'a str,
    content: Vec<&'a str>,
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

fn term_id(head: &ComponentItemNode) -> String {
    head.link
        .as_deref()
        .map(fragment_id)
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| slug(&head.text))
}

fn terms(comp: &ComponentNode) -> Vec<Term<'_>> {
    let lines: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "text" | "item") && !i.text.is_empty())
        .collect();
    if lines.iter().any(|i| i.item_type == "item") {
        let mut out: Vec<Term<'_>> = Vec::new();
        for line in lines {
            if line.item_type == "item" {
                out.push(Term {
                    id: term_id(line),
                    title: &line.text,
                    content: Vec::new(),
                });
            } else if let Some(term) = out.last_mut() {
                term.content.push(&line.text);
            }
        }
        return out;
    }
    lines
        .chunks(2)
        .map(|pair| Term {
            id: term_id(pair[0]),
            title: &pair[0].text,
            content: pair
                .get(1)
                .map(|body| vec![body.text.as_str()])
                .unwrap_or_default(),
        })
        .collect()
}

pub fn render(comp: &ComponentNode) -> String {
    let title = item(comp, "label")
        .filter(|t| !t.is_empty())
        .or_else(|| attr_nonempty(comp, "title"))
        .unwrap_or(DEFAULT_TITLE);
    let nav = attr_nonempty(comp, "nav").unwrap_or("Sections");
    let viewport = attr_nonempty(comp, "viewport");
    let terms = terms(comp);
    let links: String = terms
        .iter()
        .enumerate()
        .map(|(index, term)| {
            let (current, indicator) = if index == 0 && viewport.is_none() {
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
            let body: String = term
                .content
                .iter()
                .map(|p| format!("<p>{}</p>", esc(p)))
                .collect();
            format!(
                "<section id=\"{}\"><h2>{}</h2><div>{body}</div></section>",
                term.id,
                esc(term.title)
            )
        })
        .collect();
    let html = format!(
        "<div data-slot=\"scroll-nav\"><h1>{}</h1><div><nav aria-label=\"{}\"><ul>{links}</ul></nav><div>{sections}</div></div></div>",
        esc(title),
        esc(nav)
    );
    match viewport {
        Some(label) => format!(
            "<section class=\"cui-scroll-nav-viewport\" tabindex=\"0\" aria-label=\"{}\">{html}</section>",
            esc(label)
        ),
        None => html,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    const CSS: &str = include_str!("cronus_ui_css/scroll-nav.css");

    fn line(kind: &str, body: &str, link: Option<&str>) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: body.into(),
            link: link.map(String::from),
            tone: None,
            config: Default::default(),
        }
    }

    fn text(body: &str, link: Option<&str>) -> ComponentItemNode {
        line("text", body, link)
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
            "<div data-slot=\"scroll-nav\"><h1>Terms</h1><div><nav aria-label=\"Sections\"><ul><li><a href=\"#acceptance\" aria-current=\"location\"><span></span><p>Acceptance</p></a></li><li><a href=\"#privacy-data\"><p>Privacy data</p></a></li></ul></nav><div><section id=\"acceptance\"><h2>Acceptance</h2><div><p>By using it.</p></div></section><section id=\"privacy-data\"><h2>Privacy data</h2><div><p>We keep little.</p></div></section></div></div></div>"
        );
        for bad in ["<script", "<style", " style=", " onclick=", "disabled"] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    /// Docs "Terms": `item` heads with several `text` paragraphs, inside a
    /// named scroll viewport; the spy is CSS-driven, so no row is pinned
    /// active in the markup.
    #[test]
    fn item_heads_paragraphs_and_viewport() {
        let mut c = stub("scroll-nav", "Terms & Conditions");
        c.props
            .insert("viewport".into(), "Terms, scrollable".into());
        c.items.push(line("item", "Acceptance of Terms", None));
        c.items.push(text("By accessing you agree.", None));
        c.items.push(text("Continued use means you accept.", None));
        c.items
            .push(line("item", "License Agreement", Some("#license")));
        c.items.push(text("Licensed, not sold.", None));
        let html = render(&c);
        assert!(html.starts_with(
            "<section class=\"cui-scroll-nav-viewport\" tabindex=\"0\" aria-label=\"Terms, scrollable\"><div data-slot=\"scroll-nav\"><h1>Terms &amp; Conditions</h1>"
        ));
        assert!(html.contains(
            "<ul><li><a href=\"#acceptance-of-terms\"><p>Acceptance of Terms</p></a></li><li><a href=\"#license\"><p>License Agreement</p></a></li></ul>"
        ));
        assert!(html.contains(
            "<section id=\"acceptance-of-terms\"><h2>Acceptance of Terms</h2><div><p>By accessing you agree.</p><p>Continued use means you accept.</p></div></section><section id=\"license\"><h2>License Agreement</h2><div><p>Licensed, not sold.</p></div></section>"
        ));
        assert!(html.ends_with("</div></div></section>"));
        assert!(!html.contains("aria-current"));
        assert!(!html.contains("<span>"));
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
        assert!(html.contains("<div><p>&lt;b&gt;</p></div>"), "{html}");
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
            html.contains("<section id=\"acceptance\"><h2>Acceptance</h2><div><p>By using the service you agree.</p></div></section>"),
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

    /// The viewport is the docs' `h-[28rem] rounded-3xl bg-surface-inset`
    /// scroller; inside it every section publishes a view timeline with
    /// React's observer band (`rootMargin: -20% 0px -65% 0px`) and each nav
    /// row reads its own.
    #[test]
    fn chrome_viewport_spy_uses_view_timelines() {
        assert!(CSS.contains(".cui-scroll-nav-viewport {"));
        assert!(CSS.contains("height: 28rem; width: 100%; overflow-y: auto;"));
        assert!(CSS.contains("background: var(--cronus-surface-inset);"));
        assert!(CSS.contains("view-timeline-inset: 20% 65%;"));
        assert!(CSS.contains("animation-range: cover 0% cover 100%;"));
        for n in 1..=SPY_TIMELINES {
            assert!(
                CSS.contains(&format!(
                    "section:nth-child({n}) {{ view-timeline-name: --cui-scroll-nav-{n}; }}"
                )),
                "{n}"
            );
            assert!(
                CSS.contains(&format!(
                    "li:nth-child({n}) > a > p {{ animation-timeline: --cui-scroll-nav-{n}; }}"
                )),
                "{n}"
            );
        }
        assert!(CSS.contains("timeline-scope: --cui-scroll-nav-1, --cui-scroll-nav-2"));
        assert!(CSS.contains("@keyframes cui-scroll-nav-active"));
        assert!(CSS.contains("section > div > p + p { margin-block-start: 0.75rem; }"));
    }
}
