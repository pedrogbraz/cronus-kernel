//! Dedicated Artifact renderer (AI suite). DOM matches React `Artifact`:
//! `<div data-slot="artifact">` > `<div data-slot="artifact-header">` >
//! `<p data-slot="artifact-title">` (the `title` item; with a `description:`
//! prop the two `<p>`s share a slotless `<div>`, as the docs compose them) +
//! optional `<div data-slot="artifact-actions">` of
//! `<button data-slot="artifact-action" data-variant="ghost" aria-label>` per
//! `action "Copy" icon:copy` item (hint popover via `interestfor`, `tooltip:`
//! overrides the text, `tooltip:false` drops it) and an
//! `<button data-slot="artifact-close" aria-label="Close">` × when
//! `close:true` (`close-label:"…"` renames it). Then
//! `<div data-slot="artifact-content">` with one `<p>` per `text` line, or a
//! single `<pre>` of the lines joined by newlines when `code:true` (the docs'
//! generated-file canvas). Action buttons need `onClick`, so they are the same
//! native buttons `disabled` at React's idle look.

use crate::cronus_ui_kit::{attr_nonempty, esc, flag, instance_id, truthy};
use crate::parser::{ComponentItemNode, ComponentNode};

const X: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M18 6 6 18\"></path><path d=\"m6 6 12 12\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let title = title_of(comp);
    let description = attr_nonempty(comp, "description").map(esc);
    let heading = match description {
        Some(d) => format!(
            "<div><p data-slot=\"artifact-title\">{title}</p><p data-slot=\"artifact-description\">{d}</p></div>"
        ),
        None => format!("<p data-slot=\"artifact-title\">{title}</p>"),
    };
    let actions: String = comp
        .items
        .iter()
        .filter(|i| i.item_type == "action" && !i.text.is_empty())
        .map(|i| action(comp, i))
        .collect();
    let close = if flag(comp, "close") {
        let label = attr_nonempty(comp, "close-label")
            .map(esc)
            .unwrap_or_else(|| "Close".into());
        format!(
            "<button type=\"button\" data-slot=\"artifact-close\" data-variant=\"ghost\" aria-label=\"{label}\" disabled>{X}</button>"
        )
    } else {
        String::new()
    };
    let toolbar = if actions.is_empty() && close.is_empty() {
        String::new()
    } else {
        format!("<div data-slot=\"artifact-actions\">{actions}{close}</div>")
    };
    let lines = body_lines(comp);
    let content = if flag(comp, "code") {
        format!("<pre>{}</pre>", lines.join("\n"))
    } else {
        lines.iter().map(|t| format!("<p>{t}</p>")).collect()
    };
    format!(
        "<div data-slot=\"artifact\"><div data-slot=\"artifact-header\">{heading}{toolbar}</div><div data-slot=\"artifact-content\">{content}</div></div>"
    )
}

fn title_of(comp: &ComponentNode) -> String {
    for kind in ["title", "label"] {
        if let Some(t) = crate::cronus_ui_kit::item(comp, kind) {
            if !t.is_empty() {
                return esc(t);
            }
        }
    }
    esc(&comp.name)
}

/// `text` lines (and a `label` when a `title` names the artifact), escaped.
fn body_lines(comp: &ComponentNode) -> Vec<String> {
    let has_title = crate::cronus_ui_kit::item(comp, "title").is_some_and(|t| !t.is_empty());
    let mut lines = Vec::new();
    if has_title {
        if let Some(l) = crate::cronus_ui_kit::item(comp, "label").filter(|l| !l.is_empty()) {
            lines.push(esc(l));
        }
    }
    lines.extend(
        comp.items
            .iter()
            .filter(|i| !matches!(i.item_type.as_str(), "label" | "title" | "action"))
            .filter(|i| !i.text.is_empty())
            .map(|i| esc(&i.text)),
    );
    lines
}

fn action(comp: &ComponentNode, item: &ComponentItemNode) -> String {
    let label = esc(&item.text);
    let glyph = item
        .config
        .get("icon")
        .and_then(|i| crate::cronus_ui_icons::svg(i))
        .unwrap_or_else(|| label.clone());
    let tooltip = match item.config.get("tooltip").map(|t| t.trim()) {
        None | Some("") => Some(label.clone()),
        Some(t) if t.eq_ignore_ascii_case("false") || t == "0" => None,
        Some(t) if truthy(t) => Some(label.clone()),
        Some(t) => Some(esc(t)),
    };
    let (invoker, popover) = tooltip
        .map(|text| {
            let id = instance_id(comp, "artifact-tip");
            (
                format!(" interestfor=\"{id}\" aria-describedby=\"{id}\""),
                format!(
                    "<div id=\"{id}\" popover=\"hint\" data-slot=\"tooltip-content\" role=\"tooltip\">{text}</div>"
                ),
            )
        })
        .unwrap_or_default();
    format!(
        "<button type=\"button\" data-slot=\"artifact-action\" data-variant=\"ghost\" aria-label=\"{label}\"{invoker} disabled>{glyph}</button>{popover}"
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

    fn docs() -> ComponentNode {
        let mut c = stub("artifact", "landing.tsx");
        c.items[0].item_type = "title".into();
        c.props.insert("code".into(), "true".into());
        c.items.push(line("text", "export function Hero() {", &[]));
        c.items.push(line(
            "text",
            "  return <h1>Intelligence at scale.</h1>",
            &[],
        ));
        c.items.push(line("text", "}", &[]));
        c
    }

    #[test]
    fn docs_example_is_header_title_and_pre_content() {
        assert_eq!(
            render(&docs()),
            "<div data-slot=\"artifact\"><div data-slot=\"artifact-header\"><p data-slot=\"artifact-title\">landing.tsx</p></div><div data-slot=\"artifact-content\"><pre>export function Hero() {\n  return &lt;h1&gt;Intelligence at scale.&lt;/h1&gt;\n}</pre></div></div>"
        );
    }

    #[test]
    fn description_actions_close_and_paragraphs() {
        reset_instance_ids();
        let mut c = stub("artifact", "Report");
        c.items[0].item_type = "title".into();
        c.props
            .insert("description".into(), "Generated \"today\"".into());
        c.props.insert("close".into(), "true".into());
        c.items.push(line(
            "action",
            "Copy",
            &[("icon", "copy"), ("tooltip", "Copy file")],
        ));
        c.items
            .push(line("action", "Download", &[("icon", "download")]));
        c.items.push(line("text", "First <b>paragraph</b>", &[]));
        let html = render(&c);
        assert!(html.contains("<div data-slot=\"artifact-header\"><div><p data-slot=\"artifact-title\">Report</p><p data-slot=\"artifact-description\">Generated &quot;today&quot;</p></div><div data-slot=\"artifact-actions\"><button type=\"button\" data-slot=\"artifact-action\" data-variant=\"ghost\" aria-label=\"Copy\" interestfor=\"cui-artifact-artifact-tip\" aria-describedby=\"cui-artifact-artifact-tip\" disabled><svg"));
        assert!(html.contains("role=\"tooltip\">Copy file</div>"));
        assert!(html.contains("role=\"tooltip\">Download</div><button type=\"button\" data-slot=\"artifact-close\" data-variant=\"ghost\" aria-label=\"Close\" disabled><svg"));
        assert!(html.ends_with("<div data-slot=\"artifact-content\"><p>First &lt;b&gt;paragraph&lt;/b&gt;</p></div></div>"));
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn label_only_stub_titles_the_frame() {
        let html = render(&stub("artifact", "<b>\"x\"</b>"));
        assert!(
            html.contains("<p data-slot=\"artifact-title\">&lt;b&gt;&quot;x&quot;&lt;/b&gt;</p>")
        );
        assert!(html.contains("<div data-slot=\"artifact-content\"></div>"));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/artifact.css");
        assert!(css.contains("[data-slot=\"artifact\"]"));
        assert!(css.contains("[data-slot=\"artifact-header\"]"));
        assert!(css.contains("color-mix(in oklab, var(--cronus-surface-overlay) 50%, transparent)"));
        assert!(css.contains("[data-slot=\"artifact-content\"] > pre"));
        assert!(!css.contains("zinc-"));
    }

    #[test]
    fn registered_as_dedicated() {
        use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
        assert_eq!(
            dedicated_fn_name("artifact"),
            Some("cronus_ui_artifact::render")
        );
        assert_eq!(
            renderer_kind("artifact"),
            RendererKind::Dedicated("cronus_ui_artifact::render")
        );
        let c = stub("artifact", "landing.tsx");
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), render(&c));
    }
}
