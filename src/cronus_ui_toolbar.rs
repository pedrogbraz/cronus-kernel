//! Dedicated Toolbar renderer. DOM matches React:
//! `<div data-slot="toolbar" role="toolbar" aria-orientation="horizontal" aria-label>`
//! plus each item as `<button type="button" data-slot="toolbar-button">`.
//! The `label` names the toolbar (it is never a button). Editor commands need JS,
//! so (wave 1t rule) buttons are native `disabled` buttons, not visually dimmed.
//! Docs formatting bar: `item "Bold" icon:bold type:toggle pressed:true
//! group:"marks"` lines. `group:` clusters buttons in `toolbar-group`s with a
//! `toolbar-separator` between groups; `icon:` renders the lucide glyph with
//! the text as `aria-label` (icon-only, like the docs). Pressed state is
//! native: `type:toggle` puts the button in a `<label>` with a visually hidden
//! checkbox (`pressed:true` checked) and `type:radio` uses a radio per group
//! (one pressed at a time), React's button staying decorative
//! (`aria-hidden`, `tabindex="-1"`); CSS paints `data-[state=on]` from
//! `:has(:checked)`.
//! Not interact `nav("toolbar")` (generic SURF `<nav>`).

use crate::cronus_ui_kit::{attr_nonempty, choice_texts, esc, instance_id, item_icon, label_of};
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render(comp: &ComponentNode) -> String {
    let aria = attr_nonempty(comp, "aria-label")
        .map(|a| format!(" aria-label=\"{}\"", esc(a)))
        .unwrap_or_default();
    let rich: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| {
            i.item_type == "item"
                && !i.text.is_empty()
                && ["icon", "type", "group", "pressed"]
                    .iter()
                    .any(|k| i.config.contains_key(*k))
        })
        .collect();
    if rich.is_empty() {
        let buttons = toolbar_buttons(comp)
            .into_iter()
            .map(|t| {
                format!(
                    "<button type=\"button\" data-slot=\"toolbar-button\" disabled>{t}</button>"
                )
            })
            .collect::<Vec<_>>()
            .join("");
        return format!(
            "<div data-slot=\"toolbar\" role=\"toolbar\" aria-orientation=\"horizontal\"{aria}>{buttons}</div>"
        );
    }
    let groups = rich
        .iter()
        .fold(Vec::<Option<String>>::new(), |mut acc, i| {
            let g = i
                .config
                .get("group")
                .filter(|g| !g.trim().is_empty())
                .cloned();
            if !acc.contains(&g) {
                acc.push(g);
            }
            acc
        });
    let id = instance_id(comp, "toolbar");
    let body = groups
        .iter()
        .enumerate()
        .map(|(n, g)| {
            let name = format!("{id}-g{}", n + 1);
            let buttons: String = rich
                .iter()
                .filter(|i| &i.config.get("group").filter(|g| !g.trim().is_empty()).cloned() == g)
                .map(|i| button(i, &name))
                .collect();
            let sep = if n > 0 {
                "<div data-slot=\"toolbar-separator\" role=\"separator\" aria-orientation=\"vertical\"></div>"
            } else {
                ""
            };
            match g {
                Some(_) => format!("{sep}<div data-slot=\"toolbar-group\" role=\"group\">{buttons}</div>"),
                None => format!("{sep}{buttons}"),
            }
        })
        .collect::<String>();
    format!(
        "<div data-slot=\"toolbar\" role=\"toolbar\" aria-orientation=\"horizontal\"{aria}>{body}</div>"
    )
}

fn button(i: &ComponentItemNode, group: &str) -> String {
    let text = esc(&i.text);
    let icon = item_icon(i);
    let (inner, label) = if icon.is_empty() {
        (text.clone(), String::new())
    } else {
        (icon, format!(" aria-label=\"{text}\""))
    };
    let pressed = i
        .config
        .get("pressed")
        .is_some_and(|v| crate::cronus_ui_kit::truthy(v));
    let input = match i.config.get("type").map(|t| t.trim()) {
        Some("toggle") | Some("checkbox") => Some("type=\"checkbox\"".to_string()),
        Some("radio") => Some(format!("type=\"radio\" name=\"{group}\"")),
        _ => None,
    };
    match input {
        Some(input) => {
            let checked = if pressed { " checked" } else { "" };
            let state = if pressed { " data-state=\"on\"" } else { "" };
            format!(
                "<label><input {input} aria-label=\"{text}\"{checked}><button type=\"button\" data-slot=\"toolbar-button\"{label} aria-pressed=\"{pressed}\"{state} tabindex=\"-1\" aria-hidden=\"true\">{inner}</button></label>"
            )
        }
        None => format!(
            "<button type=\"button\" data-slot=\"toolbar-button\"{label} disabled>{inner}</button>"
        ),
    }
}

fn toolbar_buttons(comp: &ComponentNode) -> Vec<String> {
    let choices = choice_texts(comp);
    if !choices.is_empty() {
        return choices;
    }
    let body: Vec<String> = comp
        .items
        .iter()
        .filter(|i| !matches!(i.item_type.as_str(), "label" | "title") && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if body.is_empty() {
        vec![label_of(comp)]
    } else {
        body
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_interact(html: &str) {
        assert!(!html.starts_with("<nav"));
        assert!(!html.contains("<details"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("flex-wrap:wrap;gap:0.25rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn emitted_fixture_label_is_aria_not_a_button() {
        let mut c = stub("toolbar", "Formatting");
        c.items.push(extra("text", "Bold"));
        c.items.push(extra("text", "Italic"));
        c.items[2]
            .config
            .insert("aria-label".into(), "Formatting".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"toolbar\" role=\"toolbar\" aria-orientation=\"horizontal\" aria-label=\"Formatting\"><button type=\"button\" data-slot=\"toolbar-button\" disabled>Bold</button><button type=\"button\" data-slot=\"toolbar-button\" disabled>Italic</button></div>"
        );
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_one_button() {
        let html = render(&stub("toolbar", "Bold"));
        assert_eq!(html.matches("data-slot=\"toolbar-button\"").count(), 1);
        assert!(html.contains(">Bold</button>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_nav_surf() {
        let mut c = stub("toolbar", "Format");
        c.items.push(extra("item", "Bold"));
        let html = render(&c);
        assert!(html.contains("role=\"toolbar\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            reject_interact(&render(&stub("toolbar", "Bold")));
        });
    }

    #[test]
    fn docs_formatting_bar_has_native_toggles_groups_and_separator() {
        let mut c = stub("toolbar", "Formatting");
        c.props.insert("aria-label".into(), "Formatting".into());
        let mut bold = extra("item", "Bold");
        bold.config.insert("icon".into(), "bold".into());
        bold.config.insert("type".into(), "toggle".into());
        bold.config.insert("pressed".into(), "true".into());
        bold.config.insert("group".into(), "marks".into());
        c.items.push(bold);
        let mut italic = extra("item", "Italic");
        italic.config.insert("icon".into(), "italic".into());
        italic.config.insert("type".into(), "toggle".into());
        italic.config.insert("group".into(), "marks".into());
        c.items.push(italic);
        let mut left = extra("item", "Align left");
        left.config.insert("icon".into(), "align-left".into());
        left.config.insert("type".into(), "radio".into());
        left.config.insert("pressed".into(), "true".into());
        left.config.insert("group".into(), "align".into());
        c.items.push(left);
        let mut center = extra("item", "Align center");
        center.config.insert("icon".into(), "align-center".into());
        center.config.insert("type".into(), "radio".into());
        center.config.insert("group".into(), "align".into());
        c.items.push(center);
        let html = render(&c);
        let id = crate::cronus_ui_kit::widget_id(&c, "toolbar");
        assert!(html.starts_with("<div data-slot=\"toolbar\" role=\"toolbar\" aria-orientation=\"horizontal\" aria-label=\"Formatting\"><div data-slot=\"toolbar-group\" role=\"group\"><label><input type=\"checkbox\" aria-label=\"Bold\" checked><button type=\"button\" data-slot=\"toolbar-button\" aria-label=\"Bold\" aria-pressed=\"true\" data-state=\"on\" tabindex=\"-1\" aria-hidden=\"true\"><svg "), "{html}");
        assert!(html.contains("data-icon=\"bold\""));
        assert!(html.contains("<label><input type=\"checkbox\" aria-label=\"Italic\"><button type=\"button\" data-slot=\"toolbar-button\" aria-label=\"Italic\" aria-pressed=\"false\" tabindex=\"-1\" aria-hidden=\"true\"><svg "));
        assert!(html.contains(&format!(
            "</label></div><div data-slot=\"toolbar-separator\" role=\"separator\" aria-orientation=\"vertical\"></div><div data-slot=\"toolbar-group\" role=\"group\"><label><input type=\"radio\" name=\"{id}-g2\" aria-label=\"Align left\" checked>"
        )), "{html}");
        assert!(html.contains(&format!(
            "<label><input type=\"radio\" name=\"{id}-g2\" aria-label=\"Align center\"><button"
        )));
        assert!(!html.contains("disabled"));
        reject_interact(&html);
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"toolbar\"] label:has(> input:checked) > [data-slot=\"toolbar-button\"] {\n  background: var(--cronus-surface-overlay); color: var(--cronus-fg);"));
        assert!(css.contains("[data-slot=\"toolbar-separator\"] {\n  margin: 0 0.25rem; height: 1.25rem; width: 1px;"));
        assert!(css.contains("[data-slot=\"toolbar-group\"] {\n  display: inline-flex; align-items: center; gap: 0.25rem;"));
    }

    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"toolbar\"]"));
        assert!(css.contains(
            "font: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 500; cursor: pointer;\n  background: transparent; color: var(--cronus-fg-secondary);"
        ));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(!css.contains("zinc-"));
    }
}
