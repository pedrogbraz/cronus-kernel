//! Dedicated CodeBlock renderer. DOM:
//! `<pre data-slot="code-block"><code>` with text from label/items.
//! Not interact `codey()` SURF `<pre style=…>` and not catalog `display()`
//! `<section>`. Not ai-code-block.

use crate::cronus_ui_kit::texts;
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let body = texts(comp).join("\n");
    format!("<pre data-slot=\"code-block\"><code>{body}</code></pre>")
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
        let mut c = stub("code-block", items.first().copied().unwrap_or("fn main() {}"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_stub(html: &str) {
        assert!(!html.contains("<section"));
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
    fn root_is_pre_code_not_section_or_codey() {
        let html = render(&snippet(&["fn main() {}"]));
        assert!(html.starts_with("<pre data-slot=\"code-block\">"));
        assert!(html.contains("<code>fn main() {}</code>"));
        assert!(!html.contains("data-slot=\"code-tabs\""));
        reject_stub(&html);
        assert_eq!(
            html,
            "<pre data-slot=\"code-block\"><code>fn main() {}</code></pre>"
        );
    }

    #[test]
    fn extra_text_items_join_as_code() {
        let mut c = stub("code-block", "let a = 1;");
        c.items.push(extra("text", "let b = 2;"));
        let html = render(&c);
        assert_eq!(
            html,
            "<pre data-slot=\"code-block\"><code>let a = 1;\nlet b = 2;</code></pre>"
        );
        reject_stub(&html);
    }

    #[test]
    fn label_only_still_emits_code() {
        let html = render(&stub("code-block", "SELECT 1"));
        assert!(html.starts_with("<pre data-slot=\"code-block\">"));
        assert!(html.contains("<code>SELECT 1</code>"));
        reject_stub(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("code-block", "a <b> & \"c\""));
        assert_eq!(
            html,
            "<pre data-slot=\"code-block\"><code>a &lt;b&gt; &amp; &quot;c&quot;</code></pre>"
        );
        reject_stub(&html);
    }

    #[test]
    fn skips_interact_codey_and_display_surf() {
        let c = snippet(&["fn main() {}"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("code-block", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<pre data-slot=\"code-block\""));
        assert!(interact.contains("style="));
        assert!(interact.contains(CODEY));
        assert!(interact.contains("<code>fn main() {}</code>"));
        assert!(!html.contains("style="));
        assert!(!html.contains("<section"));
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
            assert!(html.starts_with("<pre data-slot=\"code-block\">"));
            assert!(html.contains("<code>"));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"code-block\"]"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(css.contains("var(--cronus-font-mono"));
        assert!(css.contains("var(--cronus-fg)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains(DISPLAY_SURF));
        assert!(!css.contains(CODEY));
    }
}
