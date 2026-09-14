//! Dedicated CodeBlock renderer. DOM mirrors React:
//! `<div data-slot="code-block">` with an optional `code-block-header`
//! (filename / language), then `<div><section data-slot="code-block-scroll">`
//! > `<pre data-slot="code-block-pre">` > `<code data-slot="code-block-code">`
//! with one `<span>` per line. Code comes from `text`/`item` lines (the audit
//! emitter repeats the code as `label`). The header carries React's CopyButton
//! as a `disabled` native button (clipboard needs JS; idle look kept).
//! Not interact `codey()` SURF `<pre style=…>`, not catalog `display()`
//! `<section>`, not ai-code-block.

use crate::cronus_ui_kit::{attr_nonempty, esc, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let code = code_lines(comp)
        .iter()
        .map(|l| format!("<span>{l}</span>"))
        .collect::<Vec<_>>()
        .join("\n");
    let filename = attr_nonempty(comp, "filename").map(esc);
    let language = attr_nonempty(comp, "language").map(esc);
    let header = if filename.is_some() || language.is_some() {
        let f = filename
            .as_deref()
            .map(|f| format!("<span data-slot=\"code-block-filename\">{f}</span>"))
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
    let label = match filename.as_deref() {
        Some(f) => format!("Code block, {f}"),
        None => "Code block".into(),
    };
    format!(
        "<div data-slot=\"code-block\">{header}<div><section data-slot=\"code-block-scroll\" tabindex=\"0\" aria-label=\"{label}\"><pre data-slot=\"code-block-pre\"><code data-slot=\"code-block-code\">{code}</code></pre></section></div></div>"
    )
}

fn code_lines(comp: &ComponentNode) -> Vec<String> {
    let lines: Vec<String> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "text" | "item") && !i.text.is_empty())
        .flat_map(|i| i.text.split('\n').map(esc).collect::<Vec<_>>())
        .collect();
    if lines.is_empty() {
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

    fn block(header: &str, label: &str, code: &str) -> String {
        format!(
            "<div data-slot=\"code-block\">{header}<div><section data-slot=\"code-block-scroll\" tabindex=\"0\" aria-label=\"{label}\"><pre data-slot=\"code-block-pre\"><code data-slot=\"code-block-code\">{code}</code></pre></section></div></div>"
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
        assert!(!html.contains("copy-button"));
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

    #[test]
    fn text_lines_and_newlines_become_line_spans() {
        let mut c = stub("code-block", "demo");
        c.items.push(extra("text", "let a = 1;\nlet b = 2;"));
        c.items.push(extra("text", "let c = 3;"));
        let html = render(&c);
        assert_eq!(
            html,
            block(
                "",
                "Code block",
                "<span>let a = 1;</span>\n<span>let b = 2;</span>\n<span>let c = 3;</span>"
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
            block(
                &format!(
                    "<div data-slot=\"code-block-header\"><div><span data-slot=\"code-block-filename\">index.ts</span><span data-slot=\"code-block-language\" data-variant=\"secondary\">ts</span></div>{}</div>",
                    crate::cronus_ui_copy_button::idle_button("Copy")
                ),
                "Code block, index.ts",
                "<span>const n = 1;</span>"
            )
        );
        reject_stub(&html);
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
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"code-block-header\"] > [data-slot=\"copy-button\"] { width: 2rem; height: 2rem; }"));
        reject_stub(&html);
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
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
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
        let css = crate::cronus_ui::component_chrome_css();
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
        let css = crate::cronus_ui::component_chrome_css();
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
