//! Dedicated ClickSpark renderer. DOM mirrors React without the injected
//! `<style>`: `<div data-slot="click-spark">` + relative content `<div>` +
//! aria-hidden overflow layer. React mounts `sparkCount` spark `<span>`s at
//! the pointer on each click and removes them after 520ms; the kernel keeps
//! the ray `<span>`s in the layer and fires the same 500ms
//! `cubic-bezier(.22,1,.36,1)` burst from the centre while the wrapper is
//! `:active` (zero JS). Angles/distances (`i / count · 2π`,
//! `18 + (i % 3) · 6 px`) are derived from `nth-child` in COMPONENT_CHROME.
//!
//! `action "…"` renders the docs' `<Button>` child; `count:` is React's
//! `sparkCount` (3–24, default 8); `card:true` is the docs wrapper
//! (`grid min-h-40 place-items-center rounded-2xl border bg-surface-raised`);
//! `heading:<size>` wraps a text label in `<p class="h-<size>">`.

use crate::cronus_ui_dot_pattern::HEADINGS;
use crate::cronus_ui_kit::{attr_num, choice, esc, flag, item, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let count = attr_num::<usize>(comp, "count").unwrap_or(8).clamp(3, 24);
    let class = if flag(comp, "card") {
        " class=\"card\""
    } else {
        ""
    };
    let content = match item(comp, "action") {
        Some(action) if !action.is_empty() => {
            crate::cronus_ui::button(&esc(action), "primary", "md", None)
        }
        _ => {
            let text = label_of(comp);
            match choice(comp, "heading", HEADINGS) {
                Some(h) => format!("<p class=\"h-{h}\">{text}</p>"),
                None => text,
            }
        }
    };
    let sparks = "<span></span>".repeat(count);
    format!(
        "<div data-slot=\"click-spark\"{class}><div>{content}</div><div aria-hidden=\"true\" class=\"n-{count}\">{sparks}</div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("<style"));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<canvas"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("onpointerdown="));
        assert!(!html.contains("--dx"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_wraps_label_then_spark_layer() {
        let html = render(&stub("click-spark", "Spark"));
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"click-spark\"><div>Spark</div><div aria-hidden=\"true\" class=\"n-8\">{}</div></div>",
                "<span></span>".repeat(8)
            )
        );
        assert_eq!(html.matches("data-slot=").count(), 1);
        assert!(!html.contains("click-spark-layer"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("click-spark", "A <B> & \"C\""));
        assert!(html.starts_with(
            "<div data-slot=\"click-spark\"><div>A &lt;B&gt; &amp; &quot;C&quot;</div>"
        ));
        reject_fx(&html);
    }

    /// Docs example: `<ClickSpark className="grid min-h-40 …"><Button>Click me</Button>`.
    #[test]
    fn action_renders_the_docs_button_inside_a_card() {
        let mut c = stub("click-spark", "Burst");
        c.items.push(ComponentItemNode {
            item_type: "action".into(),
            text: "Click <me>".into(),
            link: None,
            tone: None,
            config: Default::default(),
        });
        c.props.insert("card".into(), "true".into());
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"click-spark\" class=\"card\"><div><button type=\"button\" data-slot=\"button\" data-variant=\"primary\" data-size=\"md\" class=\"cui-btn\">Click &lt;me&gt;</button></div>"));
        assert!(!html.contains("Burst"));
        assert_eq!(html.matches("data-slot=").count(), 2);
    }

    #[test]
    fn count_is_clamped_like_react_spark_count() {
        let mut c = stub("click-spark", "Spark");
        c.props.insert("count".into(), "99".into());
        assert_eq!(render(&c).matches("<span></span>").count(), 24);
        assert!(render(&c).contains("class=\"n-24\""));
        c.props.insert("count".into(), "1".into());
        assert_eq!(render(&c).matches("<span></span>").count(), 3);
        c.props.insert("count".into(), "nope".into());
        assert_eq!(render(&c).matches("<span></span>").count(), 8);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("click-spark", "Spark");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("click-spark"),
            Some("cronus_ui_click_spark::render")
        );
        assert_eq!(
            renderer_kind("click-spark"),
            RendererKind::Dedicated("cronus_ui_click_spark::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("click-spark", "Spark"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"click-spark\""));
        });
    }

    #[test]
    fn chrome_click_spark_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        let start = css.find("[data-slot=\"click-spark\"] {").unwrap();
        let block = &css[start..start + css[start..].find('}').unwrap()];
        assert!(block.contains("width: var(--cui-click-spark-w, 100%);"));
        assert!(css.contains("[data-slot=\"click-spark\"] > div:first-child {"));
        assert!(css.contains("[data-slot=\"click-spark\"] > [aria-hidden] {"));
        assert!(css.contains("[data-slot=\"click-spark\"] > [aria-hidden] > span {"));
        // React: size-1 rounded-full bg-primary, 500ms cubic-bezier(.22,1,.36,1) forwards.
        assert!(css.contains("width: 0.25rem; height: 0.25rem; border-radius: 9999px;"));
        assert!(css.contains("inset-inline-start: var(--spark-x, 50%); top: var(--spark-y, 50%);"));
        assert!(css.contains("[data-slot=\"click-spark\"]:active > [aria-hidden] > span,\n[data-slot=\"click-spark\"][data-sparking] > [aria-hidden] > span {\n  animation: cui-click-spark 500ms cubic-bezier(.22, 1, .36, 1) forwards;"));
        assert!(css.contains("--cui-a: calc(360deg * var(--cui-k) / var(--cui-n));"));
        assert!(css.contains("--cui-d: calc(18px + mod(var(--cui-k), 3) * 6px);"));
        assert!(css.contains("@keyframes cui-click-spark"));
        assert!(css.contains("translate(var(--cui-dx), var(--cui-dy)) scale(0)"));
        for k in 2..=24 {
            assert!(css.contains(&format!("> span:nth-child({k}) {{ --cui-k: {}; }}", k - 1)));
        }
        assert!(css.contains(".n-8 { --cui-n: 8; }"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(css.contains("[data-slot=\"click-spark\"].card {\n  display: grid; place-items: center; min-height: 10rem;"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
