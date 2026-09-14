//! Dedicated CodeTabs renderer. DOM matches React idle (`code-tabs.tsx`):
//! `<div data-slot="code-tabs">` → `code-tabs-header` (`role="tablist"`
//! `code-tabs-list` of `code-tabs-trigger` buttons + `code-tabs-indicator`,
//! then `code-tabs-language`) → `code-tabs-panels` with the snippet
//! `copy-button` and the active `code-tabs-panel` / `code-tabs-pre` /
//! `code-tabs-code`.
//!
//! Tabs are the content texts (the `label` names the widget and becomes the
//! `aria-label`). Code/language come from item config (`code:` / `language:`)
//! or a `code` item. With no code anywhere, each tab gets the React audit
//! harness placeholder (`"{label} install"`, language `bash`) — the emitter
//! cannot carry per-tab code, so both sides share one placeholder.
//!
//! Zero JS (Wave 1t control contract), matching React's default state: the
//! first tab is active; the other triggers and the copy button are the same
//! native `<button>`s marked `disabled` (tab switching and clipboard need a
//! runtime), un-dimmed like React's idle state. Hidden inactive panels are not
//! emitted (no way to reveal them). The sliding underline is positioned with CSS
//! anchor positioning instead of JS-measured offsets.

use crate::cronus_ui_kit::{esc, item};
use crate::parser::ComponentNode;

const NAME_KINDS: &[&str] = &["label", "title", "code"];

/// lucide `Copy`.
const COPY_ICON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><rect width=\"14\" height=\"14\" x=\"8\" y=\"8\" rx=\"2\" ry=\"2\"/><path d=\"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2\"/></svg>";

struct Tab {
    label: String,
    code: Option<String>,
    language: Option<String>,
}

pub fn render(comp: &ComponentNode) -> String {
    let tabs = tabs(comp);
    let demo = tabs.iter().all(|t| t.code.is_none()) && item(comp, "code").is_none();
    let triggers = tabs
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let label = &t.label;
            if i == 0 {
                format!(
                    "<button type=\"button\" role=\"tab\" aria-selected=\"true\" data-state=\"active\" data-slot=\"code-tabs-trigger\">{label}</button>"
                )
            } else {
                format!(
                    "<button type=\"button\" role=\"tab\" aria-selected=\"false\" data-state=\"inactive\" data-slot=\"code-tabs-trigger\" disabled>{label}</button>"
                )
            }
        })
        .collect::<String>();
    let first = tabs.first();
    let first_label = first.map(|t| t.label.as_str()).unwrap_or("");
    let code = match (first.and_then(|t| t.code.clone()), item(comp, "code")) {
        (Some(c), _) => c,
        (None, Some(c)) if !c.is_empty() => esc(c),
        _ if demo => format!("{first_label} install"),
        _ => String::new(),
    };
    let language = first
        .and_then(|t| t.language.clone())
        .or_else(|| demo.then(|| "bash".to_string()))
        .map(|l| format!("<span data-slot=\"code-tabs-language\">{l}</span>"))
        .unwrap_or_default();
    let aria = aria_label(comp)
        .map(|v| format!(" aria-label=\"{}\"", esc(v)))
        .unwrap_or_default();
    format!(
        "<div data-slot=\"code-tabs\" data-orientation=\"horizontal\"{aria}><div data-slot=\"code-tabs-header\"><div role=\"tablist\" aria-orientation=\"horizontal\" data-slot=\"code-tabs-list\">{triggers}<span aria-hidden=\"true\" data-slot=\"code-tabs-indicator\"></span></div>{language}</div><div data-slot=\"code-tabs-panels\"><button type=\"button\" data-slot=\"copy-button\" data-variant=\"ghost\" aria-label=\"Copy {first_label} snippet\" disabled>{COPY_ICON}</button><div role=\"tabpanel\" data-state=\"active\" tabindex=\"0\" data-slot=\"code-tabs-panel\"><pre data-slot=\"code-tabs-pre\"><code data-slot=\"code-tabs-code\">{code}</code></pre></div></div></div>"
    )
}

fn aria_label(comp: &ComponentNode) -> Option<&str> {
    comp.props
        .get("aria-label")
        .or_else(|| comp.items.iter().find_map(|i| i.config.get("aria-label")))
        .map(String::as_str)
        .filter(|v| !v.is_empty())
}

fn tabs(comp: &ComponentNode) -> Vec<Tab> {
    let config = |i: &crate::parser::ComponentItemNode, key: &str| {
        i.config.get(key).filter(|s| !s.is_empty()).map(|s| esc(s))
    };
    let tabs: Vec<Tab> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty() && !NAME_KINDS.contains(&i.item_type.as_str()))
        .map(|i| Tab {
            label: esc(&i.text),
            code: config(i, "code"),
            language: config(i, "language"),
        })
        .collect();
    if !tabs.is_empty() {
        return tabs;
    }
    comp.items
        .iter()
        .find(|i| !i.text.is_empty() && i.item_type != "code")
        .map(|i| Tab {
            label: esc(&i.text),
            code: None,
            language: None,
        })
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;
    use std::collections::HashMap;

    fn extra(item_type: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: item_type.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn with_config(label: &str, pairs: &[(&str, &str)]) -> ComponentItemNode {
        let mut config = HashMap::new();
        for (k, v) in pairs {
            config.insert((*k).into(), (*v).into());
        }
        ComponentItemNode {
            item_type: "item".into(),
            text: label.into(),
            link: None,
            tone: None,
            config,
        }
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("onclick="));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("<section"));
        assert!(!html.contains("navigator.clipboard"));
        assert!(!html.contains("zinc-"));
    }

    /// Emitted audit source: `label "Install"` + texts bun/npm +
    /// `aria-label:"Install"`. The label must not become a tab or the code.
    #[test]
    fn audit_source_renders_react_idle_dom() {
        let mut c = stub("code-tabs", "Install");
        c.items.push(extra("text", "bun"));
        c.items.push(extra("text", "npm"));
        c.items
            .last_mut()
            .unwrap()
            .config
            .insert("aria-label".into(), "Install".into());
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"code-tabs\" data-orientation=\"horizontal\" aria-label=\"Install\"><div data-slot=\"code-tabs-header\"><div role=\"tablist\" aria-orientation=\"horizontal\" data-slot=\"code-tabs-list\"><button type=\"button\" role=\"tab\" aria-selected=\"true\" data-state=\"active\" data-slot=\"code-tabs-trigger\">bun</button><button type=\"button\" role=\"tab\" aria-selected=\"false\" data-state=\"inactive\" data-slot=\"code-tabs-trigger\" disabled>npm</button><span aria-hidden=\"true\" data-slot=\"code-tabs-indicator\"></span></div><span data-slot=\"code-tabs-language\">bash</span></div><div data-slot=\"code-tabs-panels\"><button type=\"button\" data-slot=\"copy-button\" data-variant=\"ghost\" aria-label=\"Copy bun snippet\" disabled>{COPY_ICON}</button><div role=\"tabpanel\" data-state=\"active\" tabindex=\"0\" data-slot=\"code-tabs-panel\"><pre data-slot=\"code-tabs-pre\"><code data-slot=\"code-tabs-code\">bun install</code></pre></div></div></div>"
            )
        );
        assert!(!crate::cli::stub_renderer_gate::looks_like_interact_generic(&html));
        reject_interact(&html);
    }

    #[test]
    fn first_panel_code_and_language_from_item_config() {
        let mut c = stub("code-tabs", "Install");
        c.items = vec![
            with_config(
                "bun",
                &[("code", "bunx cronus-ui add code-tabs"), ("language", "sh")],
            ),
            extra("item", "npm"),
        ];
        let html = render(&c);
        assert!(html.contains(">bunx cronus-ui add code-tabs</code>"));
        assert!(html.contains("<span data-slot=\"code-tabs-language\">sh</span>"));
        assert!(!html.contains(">bash<"));
        assert!(!html.contains(" install</code>"));
        reject_interact(&html);
    }

    #[test]
    fn code_without_language_omits_language_slot() {
        let mut c = stub("code-tabs", "Install");
        c.items = vec![with_config("bun", &[("code", "bun i")])];
        let html = render(&c);
        assert!(!html.contains("code-tabs-language"));
        assert!(html.contains(">bun i</code>"));
    }

    #[test]
    fn label_only_is_one_active_tab() {
        let html = render(&stub("code-tabs", "bun"));
        assert_eq!(html.matches("data-slot=\"code-tabs-trigger\"").count(), 1);
        assert!(!html.contains("data-state=\"inactive\""));
        assert!(html.contains("aria-label=\"Copy bun snippet\" disabled>"));
        assert!(html.contains(">bun install</code>"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_tabs() {
        let mut c = stub("code-tabs", "bun");
        c.items.push(extra("item", "npm"));
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("code-tabs", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("onclick="));
        assert!(!interact.contains("data-slot=\"code-tabs-pre\""));
        assert!(html.contains("data-slot=\"code-tabs-pre\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("code-tabs", "bun"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"code-tabs-pre\""));
        });
    }

    /// Geometry parity (Wave 1t): 41px header on `surface-overlay`, 40px
    /// `text-sm font-medium` triggers, a 2px anchor-positioned underline under
    /// the active trigger, `text-xs` mono language, 22.75px code line, and the
    /// 32px copy button pinned `top-2 right-2` without the global disabled dim.
    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"code-tabs-trigger\"] {\n  display: inline-flex; align-items: center; justify-content: center; gap: 0.375rem;\n  white-space: nowrap; padding: 0.625rem 0.875rem; border: 0; border-radius: 0;\n  background: transparent; color: var(--cronus-fg-tertiary);\n  font-size: 0.875rem; line-height: 1.25rem; font-weight: 500; cursor: default;\n}"));
        assert!(css.contains("[data-slot=\"code-tabs-trigger\"][data-state=\"active\"] { color: var(--cronus-fg); anchor-name: --code-tabs-active; }"));
        assert!(css.contains(
            "left: anchor(--code-tabs-active left); width: anchor-size(--code-tabs-active width);"
        ));
        assert!(css.contains("[data-slot=\"code-tabs-panels\"] > [data-slot=\"copy-button\"] {\n  position: absolute; top: 0.5rem; right: 0.5rem; z-index: 10;\n  width: 2rem; height: 2rem; padding: 0; border: 0; border-radius: var(--cronus-radius-lg);"));
        assert!(css.contains("opacity: 1; cursor: default;"));
        assert!(!css.contains("zinc-"));
    }
}
