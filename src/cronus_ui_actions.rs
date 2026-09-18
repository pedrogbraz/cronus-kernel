//! Dedicated Actions renderer (AI suite). DOM matches React `Actions` +
//! `Action`: `<div data-slot="actions">` holding one
//! `<button type="button" data-slot="action" data-variant="ghost" aria-label>`
//! per `action "Copy" icon:copy` item (React's Button keeps `data-variant`;
//! the Action slot replaces the Button slot). `tooltip:"…"` on an item (or
//! the item text when the docs pass `tooltip` = `label`) wraps the button in
//! React's Tooltip: the kernel renders the hint as a sibling
//! `<div popover="hint" data-slot="tooltip-content" role="tooltip">` opened by
//! the button's `interestfor` (no JS); `tooltip:false` drops it. An item with
//! a link (`action "Docs" -> "/docs"`) is an `<a data-slot="action">`; a plain
//! action is a live `<button type="button">`. Copy-like items (`icon:copy`,
//! label "Copy", or a `copy:` payload) emit `data-slot="copy-button"` and
//! `data-copy` so page runtime can write the clipboard. `disabled` only when
//! the author set it. `variant:` (ghost, outline, secondary…) and `size:`
//! (icon-sm, icon, sm, md) on the component or an item follow React's Button
//! recipe (`data-variant` + `s-*` class).

use crate::cronus_ui_kit::{choice, esc, instance_id, label_of, safe_url, truthy};
use crate::parser::{ComponentItemNode, ComponentNode};

const VARIANTS: &[&str] = &[
    "ghost",
    "outline",
    "primary",
    "secondary",
    "destructive",
    "link",
];
const SIZES: &[&str] = &["icon-sm", "icon", "sm", "md", "lg"];

pub fn render(comp: &ComponentNode) -> String {
    let variant = choice(comp, "variant", VARIANTS).unwrap_or("ghost");
    let size = choice(comp, "size", SIZES).unwrap_or("icon-sm");
    let items: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "action" && !i.text.is_empty())
        .collect();
    let name = label_of(comp);
    let buttons: String = if items.is_empty() {
        action(
            comp,
            &name,
            None,
            None,
            variant,
            size,
            None,
            copy_payload_for(&name, None, None, None),
            author_disabled(comp, None),
        )
    } else {
        items
            .iter()
            .map(|i| {
                let v = i
                    .config
                    .get("variant")
                    .map(String::as_str)
                    .filter(|v| VARIANTS.contains(v))
                    .unwrap_or(variant);
                let s = i
                    .config
                    .get("size")
                    .map(String::as_str)
                    .filter(|s| SIZES.contains(s))
                    .unwrap_or(size);
                let tooltip = match i.config.get("tooltip").map(|t| t.trim()) {
                    None | Some("") => Some(esc(&i.text)),
                    Some(t) if t.eq_ignore_ascii_case("false") || t == "0" => None,
                    Some(t) if truthy(t) => Some(esc(&i.text)),
                    Some(t) => Some(esc(t)),
                };
                action(
                    comp,
                    &esc(&i.text),
                    i.config.get("icon").map(String::as_str),
                    tooltip,
                    v,
                    s,
                    i.link.as_deref(),
                    copy_payload_for(
                        &i.text,
                        i.config.get("icon").map(String::as_str),
                        i.config.get("copy").map(String::as_str),
                        i.config.get("value").map(String::as_str),
                    ),
                    author_disabled(comp, Some(i)),
                )
            })
            .collect()
    };
    format!("<div data-slot=\"actions\">{buttons}</div>")
}

fn author_disabled(comp: &ComponentNode, item: Option<&ComponentItemNode>) -> bool {
    comp.props.get("disabled").is_some_and(|v| truthy(v))
        || item
            .and_then(|i| i.config.get("disabled"))
            .is_some_and(|v| truthy(v))
}

fn is_copy_like(text: &str, icon: Option<&str>, copy: Option<&str>) -> bool {
    text.eq_ignore_ascii_case("copy")
        || icon.is_some_and(|i| i.eq_ignore_ascii_case("copy"))
        || copy.is_some_and(|c| !c.is_empty())
}

fn copy_payload_for(
    text: &str,
    icon: Option<&str>,
    copy: Option<&str>,
    value: Option<&str>,
) -> Option<String> {
    if !is_copy_like(text, icon, copy) {
        return None;
    }
    Some(esc(copy
        .filter(|c| !c.is_empty())
        .or_else(|| value.filter(|v| !v.is_empty()))
        .unwrap_or(text)))
}

/// One action; `label` is already escaped. `copy` is already escaped when set.
fn action(
    comp: &ComponentNode,
    label: &str,
    icon: Option<&str>,
    tooltip: Option<String>,
    variant: &str,
    size: &str,
    href: Option<&str>,
    copy: Option<String>,
    disabled: bool,
) -> String {
    let glyph = icon.and_then(crate::cronus_ui_icons::svg);
    let inner = glyph.unwrap_or_else(|| label.to_string());
    let class = format!(" class=\"s-{size}\"");
    let hint = tooltip.map(|text| {
        let id = instance_id(comp, "action-tip");
        (
            format!(" interestfor=\"{id}\" aria-describedby=\"{id}\""),
            format!(
                "<div id=\"{id}\" popover=\"hint\" data-slot=\"tooltip-content\" role=\"tooltip\">{text}</div>"
            ),
        )
    });
    let (invoker, popover) = hint.unwrap_or_default();
    let disabled_attr = if disabled {
        " disabled data-disabled"
    } else {
        ""
    };
    match href {
        Some(href) => format!(
            "<a data-slot=\"action\" data-variant=\"{variant}\"{class} href=\"{}\" aria-label=\"{label}\"{invoker}>{inner}</a>{popover}",
            safe_url(href)
        ),
        None => {
            if let Some(payload) = copy {
                let size_attr = if size == "icon" {
                    String::new()
                } else {
                    format!(" data-size=\"{size}\"")
                };
                format!(
                    "<button type=\"button\" data-slot=\"copy-button\" data-variant=\"{variant}\"{size_attr} aria-label=\"{label}\" data-copy=\"{payload}\"{invoker}{disabled_attr}>{inner}<span aria-live=\"polite\"></span></button>{popover}"
                )
            } else {
                format!(
                    "<button type=\"button\" data-slot=\"action\" data-variant=\"{variant}\"{class} aria-label=\"{label}\"{invoker}{disabled_attr}>{inner}</button>{popover}"
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::{reset_instance_ids, stub};
    use std::collections::HashMap;

    fn act(text: &str, icon: &str, extra: &[(&str, &str)]) -> ComponentItemNode {
        let mut config = HashMap::new();
        config.insert("icon".to_string(), icon.to_string());
        for (k, v) in extra {
            config.insert(k.to_string(), v.to_string());
        }
        ComponentItemNode {
            item_type: "action".into(),
            text: text.into(),
            link: None,
            tone: None,
            config,
        }
    }

    fn docs() -> ComponentNode {
        let mut c = stub("actions", "Message actions");
        c.items.push(act("Copy", "copy", &[]));
        c.items.push(act("Retry", "refresh-cw", &[]));
        c.items.push(act("Share", "share", &[]));
        c
    }

    fn zero_js(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick"));
    }

    #[test]
    fn docs_example_is_three_ghost_icon_buttons_with_hints() {
        reset_instance_ids();
        let html = render(&docs());
        assert!(html.starts_with("<div data-slot=\"actions\"><button type=\"button\" data-slot=\"copy-button\" data-variant=\"ghost\" data-size=\"icon-sm\" aria-label=\"Copy\" data-copy=\"Copy\" interestfor=\"cui-actions-action-tip\" aria-describedby=\"cui-actions-action-tip\"><svg"));
        assert!(html.contains("data-icon=\"copy\""));
        assert!(html.contains("data-icon=\"refresh-cw\""));
        assert!(html.contains("data-icon=\"share\""));
        assert_eq!(html.matches("data-slot=\"copy-button\"").count(), 1);
        assert_eq!(html.matches("data-slot=\"action\"").count(), 2);
        assert!(html.contains("<div id=\"cui-actions-action-tip\" popover=\"hint\" data-slot=\"tooltip-content\" role=\"tooltip\">Copy</div>"));
        assert!(html.contains("id=\"cui-actions-action-tip-3\" popover=\"hint\" data-slot=\"tooltip-content\" role=\"tooltip\">Share</div></div>"));
        assert!(!html.contains("data-slot=\"button\""));
        assert!(!html.contains(" disabled"));
        zero_js(&html);
    }

    #[test]
    fn tooltip_text_off_and_link_actions() {
        reset_instance_ids();
        let mut c = stub("actions", "Actions");
        c.items
            .push(act("Copy", "copy", &[("tooltip", "Copy to clipboard")]));
        c.items
            .push(act("Retry", "refresh-cw", &[("tooltip", "false")]));
        let mut docs = act("Docs", "", &[("variant", "outline"), ("size", "sm")]);
        docs.link = Some("javascript:alert(1)".into());
        c.items.push(docs);
        let html = render(&c);
        assert!(html.contains("role=\"tooltip\">Copy to clipboard</div>"));
        assert!(html.contains("data-slot=\"copy-button\""));
        assert!(html.contains("data-copy=\"Copy\""));
        assert!(html.contains("aria-label=\"Retry\">"));
        assert!(!html.contains(" disabled"));
        assert!(!html.contains("role=\"tooltip\">Retry</div>"));
        assert!(html.contains("<a data-slot=\"action\" data-variant=\"outline\" class=\"s-sm\" href=\"#\" aria-label=\"Docs\" interestfor="));
        assert!(html.contains(">Docs</a>"));
        zero_js(&html);
    }

    #[test]
    fn copy_payload_and_author_disabled() {
        reset_instance_ids();
        let mut c = stub("actions", "Actions");
        c.items
            .push(act("Copy", "copy", &[("copy", "hello <world>")]));
        c.items.push(act(
            "Retry",
            "refresh-cw",
            &[("disabled", "true"), ("tooltip", "false")],
        ));
        let html = render(&c);
        assert!(html.contains("data-copy=\"hello &lt;world&gt;\""));
        assert!(html.contains("aria-label=\"Retry\" disabled data-disabled>"));
        zero_js(&html);
    }

    #[test]
    fn escapes_hostile_text_and_label_only_falls_back_to_one_action() {
        reset_instance_ids();
        let mut c = stub("actions", "<b>\"x\"</b>");
        c.props.insert("size".into(), "icon".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"&lt;b&gt;&quot;x&quot;&lt;/b&gt;\""));
        assert!(html.contains("class=\"s-icon\""));
        assert!(!html.contains("<b>"));
        assert!(!html.contains("tooltip-content"));
        zero_js(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/actions.css");
        assert!(css.contains("[data-slot=\"action\"]"));
        assert!(css.contains("var(--cronus-fg-tertiary)"));
        assert!(css.contains("width: 2rem; height: 2rem"));
        assert!(!css.contains("zinc-"));
    }

    #[test]
    fn registered_as_dedicated() {
        use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
        assert_eq!(
            dedicated_fn_name("actions"),
            Some("cronus_ui_actions::render")
        );
        assert_eq!(
            renderer_kind("actions"),
            RendererKind::Dedicated("cronus_ui_actions::render")
        );
        let c = stub("actions", "Copy");
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), render(&c));
    }
}
