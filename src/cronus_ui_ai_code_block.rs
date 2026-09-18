//! Dedicated AiCodeBlock renderer (AI suite). DOM matches React `AiCodeBlock`
//! as the docs compose it: `<div data-slot="ai-code-block">` >
//! `<div data-slot="ai-code-block-header">` (`<span>` filename +
//! `<button data-slot="copy-button" data-variant="ghost" aria-label="Copy">`)
//! > `<div data-slot="ai-code-block-body">` > `<div data-slot="ai-code-block-item">`
//! > `<pre data-slot="ai-code-block-content" data-language><code>`.
//!
//! Files are `item "main.ts" language:ts` lines; the `text` lines that follow
//! an item are that file's code (joined by newlines, trailing newline like the
//! docs data). `value:"main.ts"` picks the active file (React `defaultValue`),
//! else the first; like React only the active item is rendered.
//! `line-numbers:true` stamps `data-line-numbers` on the item. `copy:"…"` /
//! `select-file:"…"` rename the labels. The header copy control is React's
//! CopyButton slot (`data-slot="copy-button"` + `data-copy` of the active
//! file) so page runtime can write it to the clipboard; `disabled` only when
//! the author set it.

use crate::cronus_ui_kit::{attr_nonempty, esc, flag};
use crate::parser::ComponentNode;

const COPY: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" data-icon=\"copy\"><rect width=\"14\" height=\"14\" x=\"8\" y=\"8\" rx=\"2\" ry=\"2\"></rect><path d=\"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2\"></path></svg>";

struct File {
    name: String,
    language: Option<String>,
    lines: Vec<String>,
}

fn files(comp: &ComponentNode) -> Vec<File> {
    let mut out: Vec<File> = Vec::new();
    for i in &comp.items {
        match i.item_type.as_str() {
            "item" if !i.text.is_empty() => out.push(File {
                name: i.text.clone(),
                language: i.config.get("language").filter(|l| !l.is_empty()).cloned(),
                lines: Vec::new(),
            }),
            "text" | "value" => {
                if out.is_empty() {
                    let name = attr_nonempty(comp, "value")
                        .or_else(|| crate::cronus_ui_kit::item(comp, "label"))
                        .unwrap_or(&comp.name)
                        .to_string();
                    out.push(File {
                        name,
                        language: attr_nonempty(comp, "language").map(str::to_string),
                        lines: Vec::new(),
                    });
                }
                if let Some(f) = out.last_mut() {
                    f.lines.push(i.text.clone());
                }
            }
            _ => {}
        }
    }
    if out.is_empty() {
        out.push(File {
            name: crate::cronus_ui_kit::item(comp, "label")
                .filter(|l| !l.is_empty())
                .unwrap_or(&comp.name)
                .to_string(),
            language: attr_nonempty(comp, "language").map(str::to_string),
            lines: Vec::new(),
        });
    }
    out
}

pub fn render(comp: &ComponentNode) -> String {
    let files = files(comp);
    let active = attr_nonempty(comp, "value")
        .and_then(|v| files.iter().position(|f| f.name == v))
        .unwrap_or(0);
    let file = &files[active];
    let copy = attr_nonempty(comp, "copy")
        .map(esc)
        .unwrap_or_else(|| "Copy".into());
    let language = file
        .language
        .as_deref()
        .map(|l| format!(" data-language=\"{}\"", esc(l)))
        .unwrap_or_default();
    let line_numbers = if flag(comp, "line-numbers") || flag(comp, "lineNumbers") {
        " data-line-numbers=\"true\""
    } else {
        ""
    };
    let code: String = file.lines.iter().map(|l| format!("{}\n", esc(l))).collect();
    let disabled = if flag(comp, "disabled") {
        " disabled data-disabled"
    } else {
        ""
    };
    format!(
        "<div data-slot=\"ai-code-block\"><div data-slot=\"ai-code-block-header\"><span>{}</span><button data-slot=\"copy-button\" data-variant=\"ghost\" data-size=\"icon-sm\" type=\"button\" aria-label=\"{copy}\" data-copy=\"{code}\"{disabled}>{COPY}<span aria-live=\"polite\"></span></button></div><div data-slot=\"ai-code-block-body\"><div data-slot=\"ai-code-block-item\"{line_numbers}><pre data-slot=\"ai-code-block-content\"{language}><code>{code}</code></pre></div></div></div>",
        esc(&file.name)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;
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
        let mut c = stub("ai-code-block", "Tool output");
        c.items.clear();
        c.props.insert("value".into(), "main.ts".into());
        c.items.push(line("item", "main.ts", &[("language", "ts")]));
        c.items.push(line("text", "export const n = 1", &[]));
        c
    }

    #[test]
    fn docs_example_is_header_copy_and_one_pre() {
        assert_eq!(
            render(&docs()),
            format!("<div data-slot=\"ai-code-block\"><div data-slot=\"ai-code-block-header\"><span>main.ts</span><button data-slot=\"copy-button\" data-variant=\"ghost\" data-size=\"icon-sm\" type=\"button\" aria-label=\"Copy\" data-copy=\"export const n = 1\n\">{COPY}<span aria-live=\"polite\"></span></button></div><div data-slot=\"ai-code-block-body\"><div data-slot=\"ai-code-block-item\"><pre data-slot=\"ai-code-block-content\" data-language=\"ts\"><code>export const n = 1\n</code></pre></div></div></div>")
        );
        assert!(!render(&docs()).contains(" disabled"));
    }

    #[test]
    fn author_disabled_keeps_the_copy_control_inert() {
        let mut c = docs();
        c.props.insert("disabled".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(" disabled data-disabled>"));
        assert!(html.contains("data-slot=\"copy-button\""));
    }

    #[test]
    fn value_picks_the_active_file_and_code_is_escaped() {
        let mut c = docs();
        c.props.insert("value".into(), "b.html".into());
        c.props.insert("line-numbers".into(), "true".into());
        c.items
            .push(line("item", "b.html", &[("language", "html")]));
        c.items.push(line("text", "<b class=\"x\">&</b>", &[]));
        c.items.push(line("text", "second", &[]));
        let html = render(&c);
        assert!(html.contains("<span>b.html</span>"));
        assert!(!html.contains("main.ts"));
        assert!(html.contains("<div data-slot=\"ai-code-block-item\" data-line-numbers=\"true\"><pre data-slot=\"ai-code-block-content\" data-language=\"html\"><code>&lt;b class=&quot;x&quot;&gt;&amp;&lt;/b&gt;\nsecond\n</code></pre>"));
        assert!(
            html.contains("data-copy=\"&lt;b class=&quot;x&quot;&gt;&amp;&lt;/b&gt;\nsecond\n\"")
        );
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn label_only_stub_names_an_empty_file() {
        let html = render(&stub("ai-code-block", "<b>\"x\"</b>"));
        assert!(html.contains("<span>&lt;b&gt;&quot;x&quot;&lt;/b&gt;</span>"));
        assert!(html.contains("<code></code>"));
        assert!(!html.contains("data-language"));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/ai-code-block.css");
        assert!(css.contains("[data-slot=\"ai-code-block\"]"));
        assert!(css.contains("[data-slot=\"ai-code-block-content\"]"));
        assert!(css.contains("var(--cronus-font-mono)"));
        assert!(!css.contains("zinc-"));
    }

    #[test]
    fn registered_as_dedicated() {
        use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
        assert_eq!(
            dedicated_fn_name("ai-code-block"),
            Some("cronus_ui_ai_code_block::render")
        );
        assert_eq!(
            renderer_kind("ai-code-block"),
            RendererKind::Dedicated("cronus_ui_ai_code_block::render")
        );
        let c = stub("ai-code-block", "main.ts");
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), render(&c));
    }
}
