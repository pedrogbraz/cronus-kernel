//! Dedicated Sources renderer (AI suite). DOM matches React/Radix `Sources` +
//! `SourcesTrigger` + `SourcesContent` + `Source`:
//! `<div data-state data-slot="sources">` > `<button data-slot="sources-trigger">`
//! (`<p>Used N sources</p>` + chevron) > `<div data-slot="sources-content">` >
//! `<a data-slot="source" href rel="noreferrer" target="_blank">` (book icon +
//! `<span>` title) per `link "Title" -> "url"` item. Links work without JS.
//! N is `count:` or the number of links; `usedSources:` overrides the label
//! (`{count}` placeholder).
//!
//! Zero JS: the trigger and content sit in a native `<details>` inside the
//! root, the button in its `<summary>` (decorative: `aria-hidden`,
//! `tabindex="-1"`, `pointer-events: none`), so the list toggles natively.
//! Radix `Collapsible` starts closed, like the docs example: `defaultOpen:true`
//! (or `open:true`) opens it; `data-state` / `aria-expanded` reflect that
//! initial state.

use crate::cronus_ui_kit::{attr, attr_nonempty, attr_num, esc, safe_url, truthy};
use crate::parser::ComponentNode;

const CHEVRON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m6 9 6 6 6-6\"></path></svg>";
const BOOK: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H19a1 1 0 0 1 1 1v18a1 1 0 0 1-1 1H6.5a1 1 0 0 1 0-5H20\"></path></svg>";
const USED_SOURCES: &str = "Used {count} sources";

pub fn render(comp: &ComponentNode) -> String {
    let open = ["open", "defaultOpen"]
        .iter()
        .find_map(|key| attr(comp, key))
        .is_some_and(truthy);
    let state = if open { "open" } else { "closed" };
    let links: Vec<(&str, &str)> = comp
        .items
        .iter()
        .filter_map(|i| i.link.as_deref().map(|href| (i.text.as_str(), href)))
        .collect();
    let count = attr_num::<u64>(comp, "count").unwrap_or(links.len() as u64);
    let label = attr_nonempty(comp, "usedSources")
        .unwrap_or(USED_SOURCES)
        .replacen("{count}", &count.to_string(), 1);
    let anchors: String = links
        .iter()
        .map(|(title, href)| {
            format!(
                "<a data-slot=\"source\" href=\"{}\" rel=\"noreferrer\" target=\"_blank\">{BOOK}<span>{}</span></a>",
                safe_url(href),
                esc(title)
            )
        })
        .collect();
    let details_open = if open { " open" } else { "" };
    format!(
        "<div data-state=\"{state}\" data-slot=\"sources\"><details{details_open}><summary><button type=\"button\" aria-expanded=\"{open}\" data-state=\"{state}\" data-slot=\"sources-trigger\" tabindex=\"-1\" aria-hidden=\"true\"><p>{}</p>{CHEVRON}</button></summary><div data-state=\"{state}\" data-slot=\"sources-content\">{anchors}</div></details></div>",
        esc(&label)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn link(title: &str, href: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "link".into(),
            text: title.into(),
            link: Some(href.into()),
            tone: None,
            config: Default::default(),
        }
    }

    /// Docs "Used sources": `count={2}`, two links, closed until toggled.
    #[test]
    fn docs_sources_match_react_dom_in_a_closed_details() {
        let mut c = stub("sources", "default");
        c.props.insert("count".into(), "2".into());
        c.items.push(link("Cronus UI", "https://aicronus.com"));
        c.items
            .push(link("Design", "https://aicronus.com/docs/design"));
        let html = render(&c);
        assert!(html.starts_with("<div data-state=\"closed\" data-slot=\"sources\"><details><summary><button type=\"button\" aria-expanded=\"false\" data-state=\"closed\" data-slot=\"sources-trigger\" tabindex=\"-1\" aria-hidden=\"true\"><p>Used 2 sources</p><svg"));
        assert!(html.contains("</button></summary><div data-state=\"closed\" data-slot=\"sources-content\"><a data-slot=\"source\" href=\"https://aicronus.com\" rel=\"noreferrer\" target=\"_blank\"><svg"));
        assert!(html.contains("<span>Cronus UI</span></a><a data-slot=\"source\" href=\"https://aicronus.com/docs/design\""));
        assert!(html.ends_with("<span>Design</span></a></div></details></div>"));
        assert!(!html.contains("default"));
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
        assert!(!html.contains(" disabled"));
    }

    #[test]
    fn default_open_opens_the_details() {
        let mut c = stub("sources", "x");
        c.props.insert("defaultOpen".into(), "true".into());
        c.items.push(link("Docs", "https://cronus.dev/docs"));
        let html = render(&c);
        assert!(html.starts_with("<div data-state=\"open\" data-slot=\"sources\"><details open><summary><button type=\"button\" aria-expanded=\"true\" data-state=\"open\""));
        assert!(html.contains("<p>Used 1 sources</p>"));
        assert!(html.contains("<div data-state=\"open\" data-slot=\"sources-content\">"));
    }

    #[test]
    fn unsafe_hrefs_are_neutralized_and_titles_escaped() {
        let mut c = stub("sources", "x");
        c.items.push(link("<b>x</b>", "javascript:alert(1)"));
        let html = render(&c);
        assert!(!html.contains("javascript:"));
        assert!(html.contains("&lt;b&gt;x&lt;/b&gt;"));
        assert!(html.contains("<p>Used 1 sources</p>"));
    }

    #[test]
    fn custom_label() {
        let mut c = stub("sources", "x");
        c.props.insert("usedSources".into(), "{count} refs".into());
        let html = render(&c);
        assert!(html.contains("<p>0 refs</p>"));
        assert!(html.contains("<div data-state=\"closed\" data-slot=\"sources-content\"></div>"));
    }

    #[test]
    fn chrome_toggles_on_details() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"sources\"] > details > summary {"));
        assert!(css.contains(
            "[data-slot=\"sources-trigger\"] {\n  display: flex; align-items: center; gap: 0.5rem;"
        ));
        assert!(css.contains("[data-slot=\"sources\"] > details[open] > [data-slot=\"sources-content\"] {\n  animation: cui-sources-in 220ms var(--ease-out-quart);\n}"));
        assert!(css.contains("[data-slot=\"source\"] {\n  display: flex; align-items: center; gap: 0.5rem; color: inherit; text-decoration: none;\n}"));
    }
}
