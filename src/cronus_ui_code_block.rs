//! Dedicated CodeBlock renderer. DOM mirrors React:
//! `<div data-slot="code-block">` with an optional `code-block-header`
//! (`filename:` span with an id the scroll region is described by, and the
//! `language:` secondary Badge, plus React's CopyButton), then
//! `<div><section data-slot="code-block-scroll">` > `<pre data-slot="code-block-pre">`
//! > `<code data-slot="code-block-code">` holding one `<span>` with the code
//! (newlines kept) — or, with `line-numbers:true`, one `code-block-line` row
//! per line with its `code-block-line-number` gutter cell. Without a header the
//! copy button floats over the top-end corner like React's. Code comes from
//! `text` / `item` lines (an empty `text ""` is a blank line; `\"` in a line is
//! a quote, since `.cronus` strings keep the backslash) — the audit emitter
//! repeats the code as `label`. The copy button is a `disabled` native button
//! (clipboard needs JS; idle look kept).
//! Not interact `codey()` SURF `<pre style=…>`, not catalog `display()`
//! `<section>`, not ai-code-block.

use crate::cronus_ui_kit::{attr_nonempty, esc, flag, label_of, widget_id};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let lines = code_lines(comp);
    let filename = attr_nonempty(comp, "filename").map(esc);
    let language = attr_nonempty(comp, "language").map(esc);
    let heading_id = widget_id(comp, "filename");
    let header = if filename.is_some() || language.is_some() {
        let f = filename
            .as_deref()
            .map(|f| {
                format!("<span id=\"{heading_id}\" data-slot=\"code-block-filename\">{f}</span>")
            })
            .unwrap_or_default();
        let l = language
            .as_deref()
            .map(|l| {
                format!(
                    "<span data-slot=\"code-block-language\" data-variant=\"secondary\">{l}</span>"
                )
            })
            .unwrap_or_default();
        format!(
            "<div data-slot=\"code-block-header\"><div>{f}{l}</div>{}</div>",
            crate::cronus_ui_copy_button::idle_button("Copy")
        )
    } else {
        String::new()
    };
    let floating = if header.is_empty() {
        crate::cronus_ui_copy_button::idle_button("Copy")
    } else {
        String::new()
    };
    let label = match (filename.as_deref(), language.as_deref()) {
        (Some(f), _) => format!("Code block, {f}"),
        (None, Some(l)) => format!("Code block, {l}"),
        (None, None) => "Code block".into(),
    };
    let described = if filename.is_some() {
        format!(" aria-describedby=\"{heading_id}\"")
    } else {
        String::new()
    };
    let code = if flag(comp, "line-numbers") || flag(comp, "show-line-numbers") {
        lines
            .iter()
            .enumerate()
            .map(|(n, line)| {
                let text = if line.is_empty() { " " } else { line.as_str() };
                format!(
                    "<span data-slot=\"code-block-line\"><span aria-hidden=\"true\" data-slot=\"code-block-line-number\">{}</span><span>{text}</span></span>",
                    n + 1
                )
            })
            .collect::<String>()
    } else {
        format!("<span>{}</span>", lines.join("\n"))
    };
    format!(
        "<div data-slot=\"code-block\">{header}<div>{floating}<section data-slot=\"code-block-scroll\" tabindex=\"0\" aria-label=\"{label}\"{described}><pre data-slot=\"code-block-pre\"><code data-slot=\"code-block-code\">{code}</code></pre></section></div></div>"
    )
}

/// `\"` and `\\` written in a `.cronus` string come through verbatim; code
/// lines read them as the quote / backslash they stand for.
pub fn unescape(s: &str) -> String {
    s.replace("\\\"", "\"").replace("\\\\", "\\")
}

fn code_lines(comp: &ComponentNode) -> Vec<String> {
    let lines: Vec<String> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "text" | "item"))
        .flat_map(|i| unescape(&i.text).split('\n').map(esc).collect::<Vec<_>>())
        .collect();
    if lines.iter().all(String::is_empty) {
        vec![label_of(comp)]
    } else {
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const DISPLAY_SURF: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem";
    const CODEY: &str = "padding:0.85rem 1rem;overflow:auto;font-size:0.8125rem";

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn snippet(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub(
            "code-block",
            items.first().copied().unwrap_or("fn main() {}"),
        );
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn copy() -> String {
        crate::cronus_ui_copy_button::idle_button("Copy")
    }

    fn block(header: &str, label: &str, code: &str) -> String {
        let floating = if header.is_empty() {
            copy()
        } else {
            String::new()
        };
        format!(
            "<div data-slot=\"code-block\">{header}<div>{floating}<section data-slot=\"code-block-scroll\" tabindex=\"0\" aria-label=\"{label}\"><pre data-slot=\"code-block-pre\"><code data-slot=\"code-block-code\">{code}</code></pre></section></div></div>"
        )
    }

    fn reject_stub(html: &str) {
        assert!(!html.contains("<section data-slot=\"code-block\""));
        assert!(!html.contains("<pre data-slot=\"code-block\""));
        assert!(!html.contains("<nav"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("v-show"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("navigator.clipboard"));
        assert!(!html.contains(DISPLAY_SURF));
        assert!(!html.contains(CODEY));
        assert!(!html.contains("display("));
        assert!(!html.contains("codey("));
        assert!(!html.contains("ai-code-block"));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_is_div_with_scroll_pre_code_like_react() {
        let html = render(&snippet(&["fn main() {}"]));
        assert_eq!(html, block("", "Code block", "<span>fn main() {}</span>"));
        assert!(!html.contains("data-slot=\"code-tabs\""));
        // No header: React floats the copy button over the code's top-end corner.
        assert_eq!(html.matches("data-slot=\"copy-button\"").count(), 1);
        reject_stub(&html);
    }

    /// Audit fixture: emitter writes the code as both `label` and `text`.
    /// React shows it once.
    #[test]
    fn fixture_code_is_not_duplicated() {
        let mut c = stub("code-block", "const n = 1;");
        c.items.push(extra("text", "const n = 1;"));
        let html = render(&c);
        assert_eq!(html, block("", "Code block", "<span>const n = 1;</span>"));
        reject_stub(&html);
    }

    /// React renders the whole snippet in one `whitespace-pre` span; text
    /// lines and embedded newlines join with `\n`.
    #[test]
    fn text_lines_and_newlines_join_in_one_span() {
        let mut c = stub("code-block", "demo");
        c.items.push(extra("text", "let a = 1;\nlet b = 2;"));
        c.items.push(extra("text", "let c = 3;"));
        let html = render(&c);
        assert_eq!(
            html,
            block(
                "",
                "Code block",
                "<span>let a = 1;\nlet b = 2;\nlet c = 3;</span>"
            )
        );
        reject_stub(&html);
    }

    #[test]
    fn filename_and_language_render_header() {
        let mut c = snippet(&["const n = 1;"]);
        c.props.insert("filename".into(), "index.ts".into());
        c.props.insert("language".into(), "ts".into());
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"code-block\"><div data-slot=\"code-block-header\"><div><span id=\"cui-code-block-filename\" data-slot=\"code-block-filename\">index.ts</span><span data-slot=\"code-block-language\" data-variant=\"secondary\">ts</span></div>{}</div><div><section data-slot=\"code-block-scroll\" tabindex=\"0\" aria-label=\"Code block, index.ts\" aria-describedby=\"cui-code-block-filename\"><pre data-slot=\"code-block-pre\"><code data-slot=\"code-block-code\"><span>const n = 1;</span></code></pre></section></div></div>",
                copy()
            )
        );
        reject_stub(&html);
        c.props.remove("filename");
        assert!(render(&c).contains("aria-label=\"Code block, ts\"><pre"));
    }

    /// React's code-block header renders CopyButton; the geometry spec compares
    /// every React slot, so the kernel emits it — disabled, since copying needs JS.
    #[test]
    fn header_copy_button_is_disabled_native_button() {
        let mut c = snippet(&["const n = 1;"]);
        c.props.insert("filename".into(), "index.ts".into());
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"copy-button\"").count(), 1);
        assert!(html.contains("aria-label=\"Copy\" disabled>"));
        let css = include_str!("cronus_ui_css/code-block.css");
        assert!(css.contains("[data-slot=\"code-block-header\"] > [data-slot=\"copy-button\"] { width: 2rem; height: 2rem; }"));
        assert!(css.contains("[data-slot=\"code-block\"] > div > [data-slot=\"copy-button\"] {\n  position: absolute; inset-inline-end: 0.5rem; top: 0.5rem; z-index: 10;"));
        reject_stub(&html);
    }

    /// Docs "Line numbers": each line is a table row with a non-selectable
    /// gutter; blank lines keep their height; `\"` in a line is a quote.
    #[test]
    fn line_numbers_render_gutter_rows() {
        let mut c = stub("code-block", "demo");
        c.items.clear();
        c.items.push(extra(
            "text",
            "import { Button } from \\\"@cronus-ui/ui\\\";",
        ));
        c.items.push(extra("text", ""));
        c.items.push(extra("text", "export function Save() {}"));
        c.props.insert("language".into(), "tsx".into());
        c.props.insert("line-numbers".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("<code data-slot=\"code-block-code\"><span data-slot=\"code-block-line\"><span aria-hidden=\"true\" data-slot=\"code-block-line-number\">1</span><span>import { Button } from &quot;@cronus-ui/ui&quot;;</span></span><span data-slot=\"code-block-line\"><span aria-hidden=\"true\" data-slot=\"code-block-line-number\">2</span><span> </span></span><span data-slot=\"code-block-line\"><span aria-hidden=\"true\" data-slot=\"code-block-line-number\">3</span><span>export function Save() {}</span></span></code>"));
        assert!(!html.contains("\\&quot;"));
        reject_stub(&html);
        let css = include_str!("cronus_ui_css/code-block.css");
        assert!(css.contains("[data-slot=\"code-block-line\"] { display: table-row; }"));
        assert!(css.contains("[data-slot=\"code-block-line-number\"] {\n  display: table-cell; user-select: none; padding-inline-end: 1rem; text-align: end;\n  color: var(--cronus-fg-muted); font-variant-numeric: tabular-nums;\n}"));
    }

    #[test]
    fn label_only_is_escaped_code() {
        let html = render(&stub("code-block", "a <b> & \"c\""));
        assert_eq!(
            html,
            block(
                "",
                "Code block",
                "<span>a &lt;b&gt; &amp; &quot;c&quot;</span>"
            )
        );
        reject_stub(&html);
    }

    #[test]
    fn skips_interact_codey_and_display_surf() {
        let c = snippet(&["fn main() {}"]);
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"code-block\">"));
        assert!(!html.contains(DISPLAY_SURF));
        reject_stub(&html);
        assert_eq!(
            dedicated_fn_name("code-block"),
            Some("cronus_ui_code_block::render")
        );
        assert_eq!(
            renderer_kind("code-block"),
            RendererKind::Dedicated("cronus_ui_code_block::render")
        );
        assert_eq!(
            dedicated_fn_name("code-tabs"),
            Some("cronus_ui_code_tabs::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&snippet(&["fn main() {}"]));
            reject_stub(&html);
            assert!(html.starts_with("<div data-slot=\"code-block\">"));
            assert!(html.contains("<code data-slot=\"code-block-code\">"));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/code-block.css");
        for slot in [
            "code-block",
            "code-block-header",
            "code-block-scroll",
            "code-block-pre",
            "code-block-code",
        ] {
            assert!(css.contains(&format!("[data-slot=\"{slot}\"]")), "{slot}");
        }
        assert!(css.contains("[data-slot=\"code-block-pre\"] {\n  margin: 0;"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(css.contains("var(--cronus-font-mono"));
        assert!(css.contains("var(--cronus-fg)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains(DISPLAY_SURF));
        assert!(!css.contains(CODEY));
    }

    /// Wave 1s geometry parity with React `CodeBlock className="w-72"`:
    /// root 288×106, header 286×49 (copy-button slot 2rem tall), filename
    /// mono 12/16, language Badge secondary 29×22, pre padding 16, 14/22.75.
    #[test]
    fn chrome_geometry_matches_react() {
        let css = include_str!("cronus_ui_css/code-block.css");
        assert!(css.contains(
            "[data-slot=\"code-block\"] {\n  box-sizing: border-box; width: var(--cui-code-block-w, 100%); max-width: 100%;\n  overflow: hidden;\n  border-radius: var(--cronus-radius-xl);\n  border: 1px solid var(--cronus-border);\n  background: var(--cronus-surface-raised);\n  color: var(--cronus-fg);\n  line-height: 1.5;\n}"
        ));
        assert!(css.contains(
            "display: flex; align-items: center; justify-content: space-between; gap: 0.75rem;\n  padding-block: 0.5rem; padding-inline: 1rem 0.625rem; border: 0 solid var(--cronus-border); border-bottom-width: 1px;\n  background: var(--cronus-surface-overlay);"
        ));
        assert!(css.contains(
            "[data-slot=\"code-block-header\"] > div { display: flex; align-items: center; gap: 0.5rem; min-width: 0; min-height: 2rem; }"
        ));
        assert!(css
            .contains("font-size: 0.75rem; line-height: 1rem; color: var(--cronus-fg-secondary);"));
        assert!(css.contains("padding: 0.125rem 0.5rem; border-radius: var(--cronus-radius-md);"));
        assert!(css.contains(
            "font-size: 0.75rem; line-height: 1rem; font-weight: 500; white-space: nowrap;"
        ));
        assert!(!css.contains(
            "padding: 0.5rem 0.75rem; border-bottom: 1px solid var(--cronus-border);\n  font-size: 0.75rem; color: var(--cronus-fg-secondary);"
        ));
    }
}
