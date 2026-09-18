//! Dedicated GlareHover renderer. DOM mirrors React: `<div data-slot="glare-hover">`
//! + an aria-hidden glare layer + a relative content `<div>` wrapping the
//! label. Only the root has a `data-slot`. React tracks the pointer with
//! `requestAnimationFrame` and writes `--glare-x/--glare-y`; the kernel is
//! zero JS, so `:hover` fades the glare in (300ms, like React's
//! `data-glare-hovered`) and sweeps it across the surface with a CSS
//! keyframe as the pointer stand-in. No inline style.
//!
//! Docs chrome from the `.cronus`: `card:true` (class `card`, the docs
//! `rounded-2xl border bg-surface-raised p-8` wrapper) and `heading:xl`
//! (`<p class="h-xl">` = `font-display text-xl text-fg`).

use crate::cronus_ui_dot_pattern::HEADINGS;
use crate::cronus_ui_kit::{choice, flag, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let class = if flag(comp, "card") {
        " class=\"card\""
    } else {
        ""
    };
    let text = label_of(comp);
    let content = match choice(comp, "heading", HEADINGS) {
        Some(h) => format!("<p class=\"h-{h}\">{text}</p>"),
        None => text,
    };
    format!(
        "<div data-slot=\"glare-hover\"{class}><div aria-hidden=\"true\"></div><div>{content}</div></div>"
    )
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
        assert!(!html.contains("<span"));
        assert!(!html.contains("<canvas"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("onmousemove="));
        assert!(!html.contains("requestAnimationFrame"));
        assert!(!html.contains("--glare-x"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
        assert_eq!(html.matches("data-slot=").count(), 1);
    }

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("glare-hover", "Glare"));
        assert_eq!(
            html,
            "<div data-slot=\"glare-hover\"><div aria-hidden=\"true\"></div><div>Glare</div></div>"
        );
        assert!(!html.contains("glare-hover-layer"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("glare-hover", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"glare-hover\"><div aria-hidden=\"true\"></div><div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
        reject_fx(&html);
    }

    /// Docs example: `<GlareHover className="rounded-2xl border … p-8">
    /// <p className="font-display text-xl text-fg">Hover the surface</p>`.
    #[test]
    fn card_and_heading_render_the_docs_wrapper() {
        let mut c = stub("glare-hover", "Hover the surface");
        c.props.insert("card".into(), "true".into());
        c.props.insert("heading".into(), "xl".into());
        assert_eq!(
            render(&c),
            "<div data-slot=\"glare-hover\" class=\"card\"><div aria-hidden=\"true\"></div><div><p class=\"h-xl\">Hover the surface</p></div></div>"
        );
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("glare-hover", "Glare");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("glare-hover"),
            Some("cronus_ui_glare_hover::render")
        );
        assert_eq!(
            renderer_kind("glare-hover"),
            RendererKind::Dedicated("cronus_ui_glare_hover::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("glare-hover", "Glare"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"glare-hover\""));
        });
    }

    #[test]
    fn chrome_glare_hover_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        let start = css.find("[data-slot=\"glare-hover\"] {").unwrap();
        let block = &css[start..start + css[start..].find('}').unwrap()];
        assert!(block.contains("width: var(--cui-glare-hover-w, 100%);"));
        assert!(css.contains("[data-slot=\"glare-hover\"] > [aria-hidden] {"));
        assert!(css.contains("[data-slot=\"glare-hover\"] > div:last-child {"));
        assert!(!css.contains("[data-slot=\"glare-hover-layer\"]"));
        assert!(css.contains("linear-gradient(115deg, transparent 32%, color-mix(in oklch, var(--cronus-fg) 18%, transparent) 50%, transparent 68%)"));
        assert!(css.contains("background-size: 220% 220%;"));
        assert!(css.contains("transition: opacity 300ms var(--cronus-ease);"));
        assert!(css.contains("[data-slot=\"glare-hover\"]:hover > [aria-hidden]"));
        assert!(css.contains("[data-slot=\"glare-hover\"]:focus-within > [aria-hidden]"));
        assert!(css.contains("background-position: var(--glare-x, 50%) var(--glare-y, 50%);"));
        // Pointer stand-in: the glare sweeps the diagonal while hovered; live
        // mode pins the sheen to `--glare-x/--glare-y`.
        assert!(css.contains("animation: cui-glare-sweep 2.4s ease-in-out infinite alternate;"));
        assert!(css.contains("[data-slot=\"glare-hover\"][data-glare-live]:hover > [aria-hidden]"));
        assert!(css.contains("animation: none;"));
        assert!(css.contains("@keyframes cui-glare-sweep"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(css.contains("[data-slot=\"glare-hover\"].card {\n  padding: 2rem;"));
        assert!(css.contains("[data-slot=\"glare-hover\"] .h-xl {"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
