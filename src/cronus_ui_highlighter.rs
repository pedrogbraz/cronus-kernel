//! Dedicated Highlighter renderer. DOM mirrors React without the injected
//! `<style>`: `<span data-slot="highlighter">` + an aria-hidden mark
//! `<span>` (draws in over 0.6s, `cubic-bezier(.22,1,.36,1)`) + the words.
//! The mark lives in COMPONENT_CHROME. Zero JS, no inline style.
//!
//! Docs sentence from the `.cronus`: `prefix:"Build the "` / `suffix:" first."`
//! put the words around the mark and `heading:3xl` is the docs
//! `<p className="font-display text-3xl text-fg">`; any of the three wraps
//! the slot in that `<p>` (class `h-<size>`).

use crate::cronus_ui_dot_pattern::HEADINGS;
use crate::cronus_ui_kit::{attr, choice, esc, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let mark = format!(
        "<span data-slot=\"highlighter\"><span aria-hidden=\"true\"></span>{}</span>",
        label_of(comp)
    );
    let prefix = attr(comp, "prefix").map(esc).unwrap_or_default();
    let suffix = attr(comp, "suffix").map(esc).unwrap_or_default();
    let heading = choice(comp, "heading", HEADINGS);
    if prefix.is_empty() && suffix.is_empty() && heading.is_none() {
        return mark;
    }
    let class = heading
        .map(|h| format!(" class=\"h-{h}\""))
        .unwrap_or_default();
    format!("<p{class}>{prefix}{mark}{suffix}</p>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("<style"));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<div"));
        assert!(!html.contains("<mark"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
        assert_eq!(html.matches("data-slot=").count(), 1);
    }

    #[test]
    fn root_is_span_with_mark_not_fx_title_box() {
        let html = render(&stub("highlighter", "Important"));
        assert_eq!(
            html,
            "<span data-slot=\"highlighter\"><span aria-hidden=\"true\"></span>Important</span>"
        );
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("highlighter", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<span data-slot=\"highlighter\"><span aria-hidden=\"true\"></span>A &lt;B&gt; &amp; &quot;C&quot;</span>"
        );
        reject_fx(&html);
    }

    /// Docs example: `<p className="font-display text-3xl text-fg">Build the
    /// <Highlighter>product surface</Highlighter> first.</p>`.
    #[test]
    fn prefix_suffix_and_heading_wrap_the_docs_sentence() {
        let mut c = stub("highlighter", "product surface");
        c.props.insert("prefix".into(), "Build the ".into());
        c.props.insert("suffix".into(), " <first>.".into());
        c.props.insert("heading".into(), "3xl".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<p class=\"h-3xl\">Build the <span data-slot=\"highlighter\"><span aria-hidden=\"true\"></span>product surface</span> &lt;first&gt;.</p>"
        );
        reject_fx(&html);
        c.props.remove("heading");
        assert!(render(&c).starts_with("<p>Build the <span"));
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("highlighter", "Important");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("highlighter"),
            Some("cronus_ui_highlighter::render")
        );
        assert_eq!(
            renderer_kind("highlighter"),
            RendererKind::Dedicated("cronus_ui_highlighter::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("highlighter", "Important"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"highlighter\""));
        });
    }

    #[test]
    fn chrome_highlighter_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"highlighter\"] {\n  position: relative;\n  display: inline;\n  white-space: nowrap;\n}"));
        assert!(css.contains("[data-slot=\"highlighter\"] > [aria-hidden] {"));
        assert!(css.contains("bottom: 0.08em;"));
        assert!(css.contains("height: 0.45em;"));
        assert!(css.contains("color-mix(in oklch, var(--cronus-primary) 25%, transparent)"));
        assert!(css.contains("animation: cui-highlighter 0.6s cubic-bezier(.22, 1, .36, 1) both;"));
        assert!(css.contains("@keyframes cui-highlighter {\n  from { transform: scaleX(0); }\n  to { transform: scaleX(1); }\n}"));
        assert!(css.contains("p:has(> [data-slot=\"highlighter\"]) {\n  margin: 0; isolation: isolate; color: var(--cronus-fg);\n}"));
        assert!(css.contains("p.h-3xl:has(> [data-slot=\"highlighter\"]) { font-size: 1.875rem; line-height: 2.25rem; }"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
