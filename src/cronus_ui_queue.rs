//! Dedicated Queue renderer (AI suite). DOM matches React `Queue` >
//! `QueueList` (`<div data-slot="queue-list">` > slotless `<div>` > `<ul>`) >
//! one `<li data-slot="queue-item">` per `item "Summarise the last three PRs"`
//! with `<span data-slot="queue-item-indicator">` and
//! `<span data-slot="queue-item-content">`, plus
//! `<div data-slot="queue-item-description">` for `description:"…"` and
//! `<div data-slot="queue-item-attachment">` of `<span data-slot="queue-item-file">`
//! chips for `file:"a.png,b.pdf"`. `completed:true` on an item stamps
//! `data-completed` on the `<li>`: CSS paints React's `completed` recipe
//! (dimmed indicator, struck-through text). The `label` item names the list.

use crate::cronus_ui_kit::{esc, label_of, truthy};
use crate::parser::{ComponentItemNode, ComponentNode};

const PAPERCLIP: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" data-icon=\"paperclip\"><path d=\"M13.234 20.252 21 12.3\"></path><path d=\"m16 6-8.414 8.586a2 2 0 0 0 0 2.828 2 2 0 0 0 2.828 0l8.414-8.586a4 4 0 0 0 0-5.656 4 4 0 0 0-5.656 0l-8.415 8.585a6 6 0 1 0 8.486 8.486\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let items: String = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty())
        .map(item)
        .collect();
    format!(
        "<div data-slot=\"queue\"><div data-slot=\"queue-list\"><div><ul aria-label=\"{}\">{items}</ul></div></div></div>",
        label_of(comp)
    )
}

fn item(i: &ComponentItemNode) -> String {
    let completed = i.config.get("completed").is_some_and(|v| truthy(v))
        || i.config
            .get("status")
            .is_some_and(|s| s.trim() == "completed");
    let done = if completed {
        " data-completed=\"\""
    } else {
        ""
    };
    let description = i
        .config
        .get("description")
        .filter(|d| !d.is_empty())
        .map(|d| format!("<div data-slot=\"queue-item-description\">{}</div>", esc(d)))
        .unwrap_or_default();
    let files: String = i
        .config
        .get("file")
        .map(|f| {
            f.split(',')
                .map(str::trim)
                .filter(|f| !f.is_empty())
                .map(|f| {
                    format!(
                        "<span data-slot=\"queue-item-file\">{PAPERCLIP}<span>{}</span></span>",
                        esc(f)
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let attachment = if files.is_empty() {
        String::new()
    } else {
        format!("<div data-slot=\"queue-item-attachment\">{files}</div>")
    };
    format!(
        "<li data-slot=\"queue-item\"{done}><span data-slot=\"queue-item-indicator\"></span><span data-slot=\"queue-item-content\">{}</span>{description}{attachment}</li>",
        esc(&i.text)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use std::collections::HashMap;

    fn todo(text: &str, extra: &[(&str, &str)]) -> ComponentItemNode {
        let mut config = HashMap::new();
        for (k, v) in extra {
            config.insert(k.to_string(), v.to_string());
        }
        ComponentItemNode {
            item_type: "item".into(),
            text: text.into(),
            link: None,
            tone: None,
            config,
        }
    }

    #[test]
    fn docs_example_is_two_pending_items() {
        let mut c = stub("queue", "Pending work");
        c.items.push(todo("Summarise the last three PRs", &[]));
        c.items.push(todo("Draft the changelog", &[]));
        assert_eq!(
            render(&c),
            "<div data-slot=\"queue\"><div data-slot=\"queue-list\"><div><ul aria-label=\"Pending work\"><li data-slot=\"queue-item\"><span data-slot=\"queue-item-indicator\"></span><span data-slot=\"queue-item-content\">Summarise the last three PRs</span></li><li data-slot=\"queue-item\"><span data-slot=\"queue-item-indicator\"></span><span data-slot=\"queue-item-content\">Draft the changelog</span></li></ul></div></div></div>"
        );
    }

    #[test]
    fn completed_description_files_and_escaping() {
        let mut c = stub("queue", "<b>\"q\"</b>");
        c.items.push(todo(
            "Ship <it>",
            &[
                ("completed", "true"),
                ("description", "Done \"now\""),
                ("file", "a.png, b.pdf"),
            ],
        ));
        let html = render(&c);
        assert!(html.contains("<ul aria-label=\"&lt;b&gt;&quot;q&quot;&lt;/b&gt;\">"));
        assert!(html.contains("<li data-slot=\"queue-item\" data-completed=\"\"><span data-slot=\"queue-item-indicator\"></span><span data-slot=\"queue-item-content\">Ship &lt;it&gt;</span><div data-slot=\"queue-item-description\">Done &quot;now&quot;</div><div data-slot=\"queue-item-attachment\"><span data-slot=\"queue-item-file\"><svg"));
        assert!(html.contains("<span>a.png</span></span><span data-slot=\"queue-item-file\"><svg"));
        assert!(html.contains("<span>b.pdf</span></span></div></li>"));
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
        assert!(render(&stub("queue", "empty")).contains("<ul aria-label=\"empty\"></ul>"));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/queue.css");
        assert!(css.contains(
            "[data-slot=\"queue-item\"][data-completed] > [data-slot=\"queue-item-content\"]"
        ));
        assert!(css.contains("text-decoration: line-through"));
        assert!(css.contains("max-height: 10rem"));
        assert!(!css.contains("zinc-"));
    }

    #[test]
    fn registered_as_dedicated() {
        use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
        assert_eq!(dedicated_fn_name("queue"), Some("cronus_ui_queue::render"));
        assert_eq!(
            renderer_kind("queue"),
            RendererKind::Dedicated("cronus_ui_queue::render")
        );
        let c = stub("queue", "Pending work");
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), render(&c));
    }
}
