//! Dedicated Meteors renderer. DOM mirrors React:
//! `<div data-slot="meteors">` + an aria-hidden layer of `count` (default 20,
//! max 60) empty `<span>` streaks + a relative content `<div>` wrapping the
//! label. React derives each meteor's position, delay, duration, travel and
//! angle from its index; the family CSS carries the same formulas as
//! `:nth-child` rules, so server and kernel paint the same field. Motion is
//! `@keyframes cui-meteor` (zero JS, no inline style, no canvas).
//! `surface:raised` + `size:2xl` are the docs stage.

use crate::cronus_ui_kit::{attr_nonempty, attr_num, label_of};
use crate::parser::ComponentNode;

/// React `Meteors` default `count`.
const METEORS: usize = 20;
/// React `Meteors` max `count`; the family CSS positions this many.
pub const MAX_METEORS: usize = 60;

pub fn render(comp: &ComponentNode) -> String {
    let count = attr_num::<f64>(comp, "count")
        .map(|c| c.round().clamp(1.0, MAX_METEORS as f64) as usize)
        .unwrap_or(METEORS);
    let class = if attr_nonempty(comp, "surface") == Some("raised") {
        " class=\"raised\""
    } else {
        ""
    };
    let content = match attr_nonempty(comp, "size") {
        Some("2xl") => format!("<p class=\"t-2xl\">{}</p>", label_of(comp)),
        _ => label_of(comp),
    };
    format!(
        "<div data-slot=\"meteors\"{class}><div aria-hidden=\"true\">{}</div><div>{content}</div></div>",
        "<span></span>".repeat(count)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;

    const CSS: &str = include_str!("cronus_ui_css/meteors.css");

    fn reject_fx(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<canvas"));
        assert!(!html.contains("SURF"));
    }

    #[test]
    fn root_wraps_label_and_twenty_streaks() {
        let html = render(&stub("meteors", "Night"));
        assert!(
            html.starts_with("<div data-slot=\"meteors\"><div aria-hidden=\"true\"><span></span>")
        );
        assert_eq!(html.matches("<span></span>").count(), 20);
        assert!(html.ends_with("</div><div>Night</div></div>"));
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("meteors"),
            Some("cronus_ui_meteors::render")
        );
        assert_eq!(
            renderer_kind("meteors"),
            RendererKind::Dedicated("cronus_ui_meteors::render")
        );
        assert_eq!(
            crate::cronus_ui_widgets::render(&stub("meteors", "Night")).unwrap(),
            html
        );
    }

    /// Docs "Shooting stars": the raised stage with display copy; `count`
    /// clamps to React's 1..60.
    #[test]
    fn docs_stage_and_count() {
        let mut c = stub("meteors", "Launch window");
        c.props.insert("surface".into(), "raised".into());
        c.props.insert("size".into(), "2xl".into());
        let html = render(&c);
        assert!(html
            .starts_with("<div data-slot=\"meteors\" class=\"raised\"><div aria-hidden=\"true\">"));
        assert!(html.ends_with("</div><div><p class=\"t-2xl\">Launch window</p></div></div>"));
        c.props.insert("count".into(), "99".into());
        assert_eq!(render(&c).matches("<span></span>").count(), 60);
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("meteors", "A <B>"));
        assert!(html.contains("A &lt;B&gt;"));
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    /// React's index formulas: top `-14 + (i×17 % 32)%`, start `8 + (i×43 % 92)%`,
    /// delay `(i×13 % 80)/10 s`, duration `1.8 + (i×7 % 22)/10 s`, travel
    /// `340 + (i×29 % 220)px`, angle `122 + (i×11 % 16)deg`; the same keyframes.
    #[test]
    fn chrome_meteors_via_css() {
        assert!(CSS.contains(
            "[data-slot=\"meteors\"] {\n  position: relative; overflow: hidden;\n  width: var(--cui-meteors-w, 100%); min-height: 8rem;\n}"
        ));
        assert!(CSS.contains("@keyframes cui-meteor {"));
        assert!(CSS.contains("animation: cui-meteor var(--meteor-duration, 1.8s) linear var(--meteor-delay, 0s) infinite backwards;"));
        assert!(CSS.contains("78% { opacity: 1; transform: rotate(var(--meteor-angle)) translate3d(calc(var(--meteor-travel) * 0.86), 0, 0); }"));
        assert!(CSS.contains("88% { opacity: 0.55; transform: rotate(var(--meteor-angle)) translate3d(var(--meteor-travel), 0, 0) scale(1.85); }"));
        // i = 0 and i = 1.
        assert!(CSS.contains("> span:nth-child(1) { inset-block-start: -14%; inset-inline-start: 8%; --meteor-delay: 0s; --meteor-duration: 1.8s; --meteor-travel: 340px; --meteor-angle: 122deg; }"));
        assert!(CSS.contains("> span:nth-child(2) { inset-block-start: 3%; inset-inline-start: 51%; --meteor-delay: 1.3s; --meteor-duration: 2.5s; --meteor-travel: 369px; --meteor-angle: 133deg; }"));
        assert!(CSS.contains(&format!("> span:nth-child({MAX_METEORS}) {{")));
        assert!(CSS.contains("width: 4rem; height: 1px; transform-origin: left center;"));
        assert!(CSS.contains("color-mix(in oklch, var(--cronus-fg) 85%, transparent)"));
        assert!(CSS.contains("[data-slot=\"meteors\"].raised {"));
        assert!(!CSS.contains("zinc-"));
        assert!(!CSS.contains("style="));
    }
}
