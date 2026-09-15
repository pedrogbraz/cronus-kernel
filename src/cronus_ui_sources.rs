//! Dedicated Sources renderer (AI suite). DOM matches React/Radix `Sources` +
//! `SourcesTrigger` + `SourcesContent` + `Source`:
//! `<div data-state data-slot="sources">` > `<button data-slot="sources-trigger">`
//! (`<p>Used N sources</p>` + chevron) > `<div data-slot="sources-content">` >
//! `<a data-slot="source" href rel="noreferrer" target="_blank">` (book icon +
//! `<span>` title) per `link "Title" -> "url"` item. Links work without JS.
//! N is `count:` or the number of links; `usedSources:` overrides the label
//! (`{count}` placeholder). Toggling needs JS: the kernel renders `defaultOpen`
//! (open unless `defaultOpen:false`) with the trigger `disabled`.

use crate::cronus_ui_kit::{attr, attr_nonempty, attr_num, esc, safe_url, truthy};
use crate::parser::ComponentNode;

const CHEVRON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m6 9 6 6 6-6\"></path></svg>";
const BOOK: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H19a1 1 0 0 1 1 1v18a1 1 0 0 1-1 1H6.5a1 1 0 0 1 0-5H20\"></path></svg>";
const USED_SOURCES: &str = "Used {count} sources";

pub fn render(comp: &ComponentNode) -> String {
    let open = attr(comp, "defaultOpen").is_none_or(truthy);
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
    let content = if open {
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
        format!("<div data-state=\"open\" data-slot=\"sources-content\">{anchors}</div>")
    } else {
        String::new()
    };
    format!(
        "<div data-state=\"{state}\" data-slot=\"sources\"><button type=\"button\" aria-expanded=\"{open}\" data-state=\"{state}\" data-slot=\"sources-trigger\" disabled><p>{}</p>{CHEVRON}</button>{content}</div>",
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

    #[test]
    fn open_sources_match_react_dom() {
        let mut c = stub("sources", "default");
        c.props.insert("count".into(), "2".into());
        c.items.push(link("Docs", "https://cronus.dev/docs"));
        c.items.push(link("Notes", "https://cronus.dev/changelog"));
        let html = render(&c);
        assert!(html.starts_with("<div data-state=\"open\" data-slot=\"sources\"><button type=\"button\" aria-expanded=\"true\" data-state=\"open\" data-slot=\"sources-trigger\" disabled><p>Used 2 sources</p><svg"));
        assert!(html.contains("<div data-state=\"open\" data-slot=\"sources-content\"><a data-slot=\"source\" href=\"https://cronus.dev/docs\" rel=\"noreferrer\" target=\"_blank\"><svg"));
        assert!(html.contains("<span>Docs</span></a><a data-slot=\"source\""));
        assert!(!html.contains("default"));
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
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
    fn closed_and_custom_label() {
        let mut c = stub("sources", "x");
        c.props.insert("defaultOpen".into(), "false".into());
        c.props.insert("usedSources".into(), "{count} refs".into());
        let html = render(&c);
        assert!(html.contains("<p>0 refs</p>"));
        assert!(!html.contains("sources-content"));
    }
}
