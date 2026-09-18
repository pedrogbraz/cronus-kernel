//! Dedicated WebPreview renderer (AI suite). DOM matches React `WebPreview`:
//! `<div data-slot="web-preview">` > `<div data-slot="web-preview-navigation">`
//! (optional `<button data-slot="web-preview-navigation-button"
//! data-variant="ghost" aria-label>` per `action "Back" icon:arrow-left`
//! item with its hint popover, then the `<input data-slot="web-preview-url"
//! type="url">` showing `url:"…"`, placeholder "Enter URL...") >
//! `<div data-slot="web-preview-body">` > the frame box (`title:"…"`,
//! "Preview" by default). The body is a sandboxed `<iframe
//! data-slot="web-preview-iframe">` whose `src` is [`safe_url`] of `url:` /
//! `href:` (or `about:blank`). Rejected schemes (`javascript:`) never land
//! in `src`. The URL bar shows the same address (dropped when [`safe_url`]
//! rejects it). `console:true` adds `<div data-slot="web-preview-console">`
//! — React's Collapsible with a ghost `<button data-slot="button">` trigger
//! (label, count `badge`, chevron) and a `collapsible-content` of
//! `item "message" level:log|warn|error time:"…"` rows (level `badge`, time,
//! text) or "No console output".
//!
//! Zero JS in the renderer: the URL box is a real input; Back / Forward
//! carry `data-web-preview-nav` so page runtime can walk history. The
//! console opens through a visually hidden checkbox in a `<label>` around
//! the decorative trigger (closed by default, like React's `consoleOpen`).

use crate::cronus_ui_kit::{attr_nonempty, esc, flag, instance_id, safe_url};
use crate::parser::{ComponentItemNode, ComponentNode};

const PLACEHOLDER: &str = "Enter URL...";
const PREVIEW: &str = "Preview";
const CONSOLE: &str = "Console";
const CONSOLE_EMPTY: &str = "No console output";
const CHEVRON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" data-icon=\"chevron-down\"><path d=\"m6 9 6 6 6-6\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let placeholder = attr_nonempty(comp, "placeholder")
        .map(esc)
        .unwrap_or_else(|| PLACEHOLDER.into());
    // A rejected URL (`#`) is not an address to show: drop it.
    let url = attr_nonempty(comp, "url")
        .or_else(|| attr_nonempty(comp, "href"))
        .map(safe_url)
        .filter(|u| u != "#");
    let value = url
        .as_ref()
        .map(|u| format!(" value=\"{u}\""))
        .unwrap_or_default();
    let frame_src = url.as_deref().unwrap_or("about:blank");
    let title = attr_nonempty(comp, "title")
        .map(esc)
        .unwrap_or_else(|| PREVIEW.into());
    let buttons: String = comp
        .items
        .iter()
        .filter(|i| i.item_type == "action" && !i.text.is_empty())
        .map(|i| nav_button(comp, i))
        .collect();
    let console = if flag(comp, "console") {
        console_html(comp)
    } else {
        String::new()
    };
    format!(
        "<div data-slot=\"web-preview\"><div data-slot=\"web-preview-navigation\">{buttons}<input data-slot=\"web-preview-url\" type=\"url\" placeholder=\"{placeholder}\" aria-label=\"{placeholder}\"{value}></div><div data-slot=\"web-preview-body\"><iframe data-slot=\"web-preview-iframe\" title=\"{title}\" sandbox=\"allow-scripts allow-same-origin\" src=\"{frame_src}\"></iframe></div>{console}</div>"
    )
}

fn nav_kind(item: &ComponentItemNode) -> Option<&'static str> {
    let text = item.text.to_ascii_lowercase();
    let icon = item
        .config
        .get("icon")
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    if text.contains("back") || icon == "arrow-left" {
        Some("back")
    } else if text.contains("forward") || icon == "arrow-right" {
        Some("forward")
    } else {
        None
    }
}

fn nav_button(comp: &ComponentNode, item: &ComponentItemNode) -> String {
    let label = esc(&item.text);
    let glyph = item
        .config
        .get("icon")
        .and_then(|i| crate::cronus_ui_icons::svg(i))
        .unwrap_or_else(|| label.clone());
    let id = instance_id(comp, "web-preview-tip");
    let nav = nav_kind(item)
        .map(|k| format!(" data-web-preview-nav=\"{k}\""))
        .unwrap_or_default();
    format!(
        "<button type=\"button\" data-slot=\"web-preview-navigation-button\" data-variant=\"ghost\" aria-label=\"{label}\" interestfor=\"{id}\" aria-describedby=\"{id}\"{nav}>{glyph}</button><div id=\"{id}\" popover=\"hint\" data-slot=\"tooltip-content\" role=\"tooltip\">{label}</div>"
    )
}

fn console_html(comp: &ComponentNode) -> String {
    let logs: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty())
        .collect();
    let label = attr_nonempty(comp, "console-label")
        .map(esc)
        .unwrap_or_else(|| CONSOLE.into());
    let empty = attr_nonempty(comp, "console-empty")
        .map(esc)
        .unwrap_or_else(|| CONSOLE_EMPTY.into());
    let id = instance_id(comp, "web-preview-console");
    let rows: String = if logs.is_empty() {
        format!("<p>{empty}</p>")
    } else {
        logs.iter()
            .map(|l| {
                let level = match l.config.get("level").map(|s| s.trim()) {
                    Some("error") => "error",
                    Some("warn") => "warn",
                    _ => "log",
                };
                let variant = match level {
                    "error" => "destructive",
                    "warn" => "warning",
                    _ => "secondary",
                };
                let time = l
                    .config
                    .get("time")
                    .map(|t| esc(t))
                    .unwrap_or_default();
                format!(
                    "<div data-level=\"{level}\"><span data-slot=\"badge\" data-variant=\"{variant}\">{level}</span><span>{time}</span><span>{}</span></div>",
                    esc(&l.text)
                )
            })
            .collect()
    };
    format!(
        "<div data-slot=\"web-preview-console\"><div data-state=\"closed\"><label><input type=\"checkbox\" id=\"{id}\" aria-label=\"{label}\" aria-controls=\"{id}-content\"><button type=\"button\" data-slot=\"button\" data-variant=\"ghost\" data-state=\"closed\" aria-expanded=\"false\" tabindex=\"-1\" aria-hidden=\"true\"><span>{label}<span data-slot=\"badge\" data-variant=\"secondary\">{}</span></span>{CHEVRON}</button></label><div id=\"{id}-content\" data-state=\"closed\" data-slot=\"collapsible-content\"><div>{rows}</div></div></div></div>",
        logs.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::{reset_instance_ids, stub};
    use std::collections::HashMap;

    fn line(kind: &str, text: &str, extra: &[(&str, &str)]) -> ComponentItemNode {
        let mut config = HashMap::new();
        for (k, v) in extra {
            config.insert(k.to_string(), v.to_string());
        }
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config,
        }
    }

    #[test]
    fn docs_example_is_url_bar_and_sandboxed_iframe() {
        let mut c = stub("web-preview", "Preview");
        c.props.insert("url".into(), "https://aicronus.com".into());
        assert_eq!(
            render(&c),
            "<div data-slot=\"web-preview\"><div data-slot=\"web-preview-navigation\"><input data-slot=\"web-preview-url\" type=\"url\" placeholder=\"Enter URL...\" aria-label=\"Enter URL...\" value=\"https://aicronus.com\"></div><div data-slot=\"web-preview-body\"><iframe data-slot=\"web-preview-iframe\" title=\"Preview\" sandbox=\"allow-scripts allow-same-origin\" src=\"https://aicronus.com\"></iframe></div></div>"
        );
    }

    #[test]
    fn hostile_url_nav_buttons_and_console_logs() {
        reset_instance_ids();
        let mut c = stub("web-preview", "x");
        c.props.insert("url".into(), "javascript:alert(1)".into());
        c.props.insert("title".into(), "<b>\"Frame\"</b>".into());
        c.props.insert("console".into(), "true".into());
        c.items
            .push(line("action", "Back", &[("icon", "arrow-left")]));
        c.items.push(line(
            "item",
            "Boot <ok>",
            &[("level", "log"), ("time", "10:24:03 AM")],
        ));
        c.items.push(line("item", "Failed", &[("level", "error")]));
        let html = render(&c);
        assert!(!html.contains("value="));
        assert!(!html.contains("javascript:"));
        assert!(html.contains("<div data-slot=\"web-preview-body\"><iframe data-slot=\"web-preview-iframe\" title=\"&lt;b&gt;&quot;Frame&quot;&lt;/b&gt;\" sandbox=\"allow-scripts allow-same-origin\" src=\"about:blank\"></iframe></div>"));
        assert!(html.contains("<button type=\"button\" data-slot=\"web-preview-navigation-button\" data-variant=\"ghost\" aria-label=\"Back\" interestfor=\"cui-web-preview-web-preview-tip\" aria-describedby=\"cui-web-preview-web-preview-tip\" data-web-preview-nav=\"back\"><svg"));
        assert!(!html.contains("navigation-button\" data-variant=\"ghost\" aria-label=\"Back\" interestfor=\"cui-web-preview-web-preview-tip\" aria-describedby=\"cui-web-preview-web-preview-tip\" disabled"));
        assert!(html.contains("role=\"tooltip\">Back</div><input"));
        assert!(html.contains("<div data-slot=\"web-preview-console\"><div data-state=\"closed\"><label><input type=\"checkbox\" id=\"cui-web-preview-web-preview-console\" aria-label=\"Console\" aria-controls=\"cui-web-preview-web-preview-console-content\"><button type=\"button\" data-slot=\"button\" data-variant=\"ghost\" data-state=\"closed\" aria-expanded=\"false\" tabindex=\"-1\" aria-hidden=\"true\"><span>Console<span data-slot=\"badge\" data-variant=\"secondary\">2</span></span><svg"));
        assert!(html.contains("<div data-level=\"log\"><span data-slot=\"badge\" data-variant=\"secondary\">log</span><span>10:24:03 AM</span><span>Boot &lt;ok&gt;</span></div><div data-level=\"error\"><span data-slot=\"badge\" data-variant=\"destructive\">error</span><span></span><span>Failed</span></div>"));
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn no_url_leaves_the_frame_empty_and_console_reports_nothing() {
        let mut c = stub("web-preview", "x");
        c.props.insert("console".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Enter URL...\"></div>"));
        assert!(!html.contains(" value="));
        assert!(html.contains("src=\"about:blank\""));
        assert!(html.contains("<div><p>No console output</p></div>"));
        assert!(html.contains("data-variant=\"secondary\">0</span>"));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/web-preview.css");
        assert!(css.contains("[data-slot=\"web-preview-url\"]"));
        assert!(css.contains("[data-slot=\"web-preview-iframe\"]"));
        assert!(css.contains("height: 14rem"));
        assert!(!css.contains("zinc-"));
    }

    #[test]
    fn registered_as_dedicated() {
        use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
        assert_eq!(
            dedicated_fn_name("web-preview"),
            Some("cronus_ui_web_preview::render")
        );
        assert_eq!(
            renderer_kind("web-preview"),
            RendererKind::Dedicated("cronus_ui_web_preview::render")
        );
        let c = stub("web-preview", "Preview");
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), render(&c));
    }
}
