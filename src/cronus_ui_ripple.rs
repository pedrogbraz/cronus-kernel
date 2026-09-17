//! Dedicated Ripple renderer. DOM mirrors React:
//! `<div data-slot="ripple">` + aria-hidden field `<div>` of `count` (default
//! 8, max 16) empty ring `<span>`s + a relative content `<div>` wrapping the
//! label. Only the root carries a `data-slot` (React adds none to
//! field/rings/content). The per-ring stagger React writes as inline
//! `--ripple-delay` (`index × duration / count`) is `:nth-of-type` index
//! variables in the family CSS, with `count` / `duration` (`n-*` / `d-*`
//! classes) feeding the same formula; `@keyframes cui-ripple` lives there
//! too — never a `<style>` tag. `surface:raised` + `size:2xl` are the docs
//! stage (`grid min-h-56 place-items-center rounded-2xl border
//! bg-surface-raised`, display copy). Zero JS, no inline style.

use crate::cronus_ui_kit::{attr_nonempty, attr_num, label_of};
use crate::parser::ComponentNode;

/// React `Ripple` default `count`.
const RINGS: usize = 8;

pub fn render(comp: &ComponentNode) -> String {
    let count = attr_num::<f64>(comp, "count")
        .map(|c| c.round().clamp(1.0, 16.0) as usize)
        .unwrap_or(RINGS);
    let duration = attr_num::<f64>(comp, "duration")
        .map(|d| d.max(0.5))
        .unwrap_or(8.0);
    let mut classes: Vec<String> = Vec::new();
    if attr_nonempty(comp, "surface") == Some("raised") {
        classes.push("raised".into());
    }
    if count != RINGS {
        classes.push(format!("n-{count}"));
    }
    let seconds = duration.round() as u32;
    if seconds != 8 {
        classes.push(format!("d-{}", seconds.clamp(1, 20)));
    }
    let class = if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    };
    let content = match attr_nonempty(comp, "size") {
        Some("2xl") => format!("<p class=\"t-2xl\">{}</p>", label_of(comp)),
        _ => label_of(comp),
    };
    format!(
        "<div data-slot=\"ripple\"{class}><div aria-hidden=\"true\">{}</div><div>{content}</div></div>",
        "<span></span>".repeat(count)
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
        assert!(!html.contains("@keyframes"));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<canvas"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("requestAnimationFrame"));
        assert!(!html.contains("--ripple-delay"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("ripple", "Pulse"));
        assert_eq!(
            html,
            "<div data-slot=\"ripple\"><div aria-hidden=\"true\"><span></span><span></span><span></span><span></span><span></span><span></span><span></span><span></span></div><div>Pulse</div></div>"
        );
        assert_eq!(html.matches("data-slot=").count(), 1);
        assert_eq!(html.matches("<span></span>").count(), 8);
        assert!(!html.contains("ripple-ring"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("ripple", "A <B> & \"C\""));
        assert!(html.ends_with("<div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"));
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("ripple", "Pulse");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
        assert_eq!(
            dedicated_fn_name("ripple"),
            Some("cronus_ui_ripple::render")
        );
        assert_eq!(
            renderer_kind("ripple"),
            RendererKind::Dedicated("cronus_ui_ripple::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("ripple", "Pulse"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"ripple\""));
            assert_eq!(html.matches("<span></span>").count(), 8);
        });
    }

    #[test]
    fn chrome_ripple_via_css() {
        let css = include_str!("cronus_ui_css/ripple.css");
        assert!(css.contains("[data-slot=\"ripple\"] {\n  position: relative; overflow: hidden;\n  width: var(--cui-ripple-w, 100%); min-height: 8rem;"));
        assert!(css.contains("[data-slot=\"ripple\"] > [aria-hidden=\"true\"] > span {"));
        assert!(css.contains("[data-slot=\"ripple\"] > div:last-child {\n  position: relative;\n}"));
        assert!(css.contains("> span:nth-of-type(8) { --ripple-i: 7; }"));
        assert!(!css.contains("[data-slot=\"ripple-ring\"]"));
        assert!(css.contains("@keyframes cui-ripple"));
        assert!(css.contains("animation: cui-ripple"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(!css.contains("--ripple-delay"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
    }

    /// Docs "Pulse": the raised stage with display copy; `count` / `duration`
    /// ride classes that feed React's `index × duration / count` stagger.
    #[test]
    fn docs_stage_count_and_duration() {
        let mut c = stub("ripple", "Now live");
        c.props.insert("surface".into(), "raised".into());
        c.props.insert("size".into(), "2xl".into());
        let html = render(&c);
        assert!(html.starts_with(
            "<div data-slot=\"ripple\" class=\"raised\"><div aria-hidden=\"true\"><span></span>"
        ));
        assert!(html.ends_with("</div><div><p class=\"t-2xl\">Now live</p></div></div>"));
        assert_eq!(html.matches("<span></span>").count(), 8);
        c.props.insert("count".into(), "12".into());
        c.props.insert("duration".into(), "6".into());
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"ripple\" class=\"raised n-12 d-6\">"));
        assert_eq!(html.matches("<span></span>").count(), 12);
        let css = include_str!("cronus_ui_css/ripple.css");
        assert!(css.contains("animation-delay: calc(var(--ripple-i, 0) * var(--ripple-cycle, 8s) / var(--ripple-count, 8));"));
        assert!(css.contains("[data-slot=\"ripple\"] > [aria-hidden=\"true\"] > span:nth-of-type(16) { --ripple-i: 15; }"));
        assert!(css.contains("[data-slot=\"ripple\"].n-12 { --ripple-count: 12; }"));
        assert!(css.contains("[data-slot=\"ripple\"].d-6 { --ripple-cycle: 6s; }"));
        assert!(css.contains("[data-slot=\"ripple\"].raised {"));
        assert!(css.contains("min-height: 14rem;"));
        assert!(css.contains("[data-slot=\"ripple\"] .t-2xl {"));
    }
}
