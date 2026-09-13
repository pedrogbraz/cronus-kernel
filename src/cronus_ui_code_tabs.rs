//! Dedicated CodeTabs renderer. DOM matches React idle:
//! `<div data-slot="code-tabs">` + `code-tabs-list` triggers from texts +
//! first `code-tabs-panel` / `code-tabs-pre`. Not interact `tabs()` generic
//! tablist without `code-tabs-pre`.

use crate::cronus_ui_kit::{esc, item, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let labels = trigger_labels(comp);
    let triggers = labels
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let (state, selected) = if i == 0 {
                ("active", "true")
            } else {
                ("inactive", "false")
            };
            format!(
                "<button type=\"button\" role=\"tab\" data-slot=\"code-tabs-trigger\" data-state=\"{state}\" aria-selected=\"{selected}\">{t}</button>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let code = first_code(comp);
    format!(
        "<div data-slot=\"code-tabs\"><div data-slot=\"code-tabs-list\" role=\"tablist\">{triggers}</div><div data-slot=\"code-tabs-panel\" role=\"tabpanel\"><pre data-slot=\"code-tabs-pre\"><code data-slot=\"code-tabs-code\">{code}</code></pre></div></div>"
    )
}

fn trigger_labels(comp: &ComponentNode) -> Vec<String> {
    let labels: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type != "code" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if labels.is_empty() {
        texts(comp)
    } else {
        labels
    }
}

fn first_code(comp: &ComponentNode) -> String {
    if let Some(t) = item(comp, "code").filter(|s| !s.is_empty()) {
        return esc(t);
    }
    for i in &comp.items {
        if let Some(c) = i.config.get("code").filter(|s| !s.is_empty()) {
            return esc(c);
        }
    }
    trigger_labels(comp)
        .into_iter()
        .next()
        .unwrap_or_default()
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

    fn extra_code(label: &str, code: &str) -> ComponentItemNode {
        let mut config = HashMap::new();
        config.insert("code".into(), code.into());
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
        assert!(!html.contains("v-model="));
        assert!(!html.contains("v-show"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("<section"));
        assert!(!html.contains("padding:1rem;display:flex;flex-direction:column;gap:0.5rem"));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_list_triggers_first_panel_pre() {
        let html = render(&stub("code-tabs", "bun"));
        assert!(html.starts_with("<div data-slot=\"code-tabs\">"));
        assert!(html.contains("data-slot=\"code-tabs-list\""));
        assert!(html.contains("role=\"tablist\""));
        assert!(html.contains("data-slot=\"code-tabs-trigger\""));
        assert!(html.contains(">bun</button>"));
        assert!(html.contains("data-slot=\"code-tabs-panel\""));
        assert!(html.contains("data-slot=\"code-tabs-pre\""));
        assert!(html.contains("data-slot=\"code-tabs-code\""));
        assert!(html.contains(">bun</code>"));
        assert_eq!(html.matches("data-slot=\"code-tabs-panel\"").count(), 1);
        assert_eq!(html.matches("data-slot=\"code-tabs-pre\"").count(), 1);
        reject_interact(&html);
    }

    #[test]
    fn triggers_from_texts() {
        let mut c = stub("code-tabs", "bun");
        c.items.push(extra("item", "npm"));
        c.items.push(extra("item", "pnpm"));
        let html = render(&c);
        assert!(html.contains(">bun</button>"));
        assert!(html.contains(">npm</button>"));
        assert!(html.contains(">pnpm</button>"));
        assert_eq!(html.matches("data-slot=\"code-tabs-trigger\"").count(), 3);
        assert!(html.contains("data-state=\"active\""));
        assert!(html.contains("aria-selected=\"true\""));
        assert_eq!(html.matches("data-state=\"inactive\"").count(), 2);
        assert_eq!(html.matches("data-slot=\"code-tabs-panel\"").count(), 1);
        assert!(html.contains("<pre data-slot=\"code-tabs-pre\">"));
        reject_interact(&html);
    }

    #[test]
    fn first_panel_code_from_item_config() {
        let mut c = stub("code-tabs", "bun");
        c.items = vec![
            extra_code("bun", "bunx cronus-ui add code-tabs"),
            extra("item", "npm"),
        ];
        let html = render(&c);
        assert!(html.contains(">bun</button>"));
        assert!(html.contains(">npm</button>"));
        assert!(html.contains(">bunx cronus-ui add code-tabs</code>"));
        assert!(!html.contains("data-slot=\"code-tabs-trigger\">bunx"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_tabs() {
        let mut c = stub("code-tabs", "bun");
        c.items.push(extra("item", "npm"));
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("code-tabs", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("role=\"tablist\""));
        assert!(interact.contains("onclick="));
        assert!(!interact.contains("data-slot=\"code-tabs-pre\""));
        assert!(!interact.contains("data-slot=\"code-tabs-list\""));
        assert!(html.contains("data-slot=\"code-tabs-pre\""));
        assert!(html.contains("data-slot=\"code-tabs-list\""));
        assert!(!html.contains("onclick="));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("code-tabs", "bun"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"code-tabs\""));
            assert!(html.contains("data-slot=\"code-tabs-pre\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"code-tabs\"]"));
        assert!(css.contains("[data-slot=\"code-tabs-list\"]"));
        assert!(css.contains("[data-slot=\"code-tabs-trigger\"]"));
        assert!(css.contains("[data-slot=\"code-tabs-pre\"]"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(css.contains("var(--cronus-font-mono"));
        assert!(css.contains("var(--cronus-fg-tertiary)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
