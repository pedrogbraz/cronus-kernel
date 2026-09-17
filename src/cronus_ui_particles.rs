//! Dedicated Particles renderer. React draws `count` specks on an
//! aria-hidden `<canvas>` every frame; the kernel emits no canvas and no
//! JS: `<div data-slot="particles">` + an aria-hidden field of speck
//! `<span>`s (React's spawn grid `(i·97, i·53)`, radius `1 + (i % 3)·0.4`
//! and velocity `((i % 5) - 2)·0.12 / ((i % 3) - 1)·0.08 px/frame` derived
//! from `nth-child` in COMPONENT_CHROME, drifting on a 20s alternate loop) +
//! the relative content `<div>`. Only the root has a `data-slot`.
//!
//! `count:` is React's `count` (8–40 specks, default 40; the stylesheet
//! vectors stop at 40); `card:true` is the docs wrapper (`grid min-h-56
//! place-items-center rounded-2xl border bg-surface-raised`); `heading:2xl`
//! wraps the label in `<p class="h-2xl">`.

use crate::cronus_ui_dot_pattern::HEADINGS;
use crate::cronus_ui_kit::{attr_num, choice, flag, label_of};
use crate::parser::ComponentNode;

pub const MAX_SPECKS: usize = 40;

pub fn render(comp: &ComponentNode) -> String {
    let count = attr_num::<usize>(comp, "count")
        .unwrap_or(MAX_SPECKS)
        .clamp(8, MAX_SPECKS);
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
    let specks = "<span></span>".repeat(count);
    format!(
        "<div data-slot=\"particles\"{class}><div aria-hidden=\"true\">{specks}</div><div>{content}</div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<canvas"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("requestAnimationFrame"));
        assert!(!html.contains("zinc-"));
        assert_eq!(html.matches("data-slot=").count(), 1);
    }

    #[test]
    fn root_has_speck_field_and_content_div() {
        let html = render(&stub("particles", "Drift"));
        assert!(html
            .starts_with("<div data-slot=\"particles\"><div aria-hidden=\"true\"><span></span>"));
        assert!(html.ends_with("</div><div>Drift</div></div>"));
        assert_eq!(html.matches("<span></span>").count(), 40);
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("particles", "A <B> & \"C\""));
        assert!(html.ends_with("<div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"));
        reject_fx(&html);
    }

    /// Docs example: card wrapper + `<p className="font-display text-2xl text-fg">Atmosphere</p>`.
    #[test]
    fn card_and_heading_render_the_docs_wrapper() {
        let mut c = stub("particles", "Atmosphere");
        c.props.insert("card".into(), "true".into());
        c.props.insert("heading".into(), "2xl".into());
        c.props.insert("count".into(), "12".into());
        let html = render(&c);
        assert!(html
            .starts_with("<div data-slot=\"particles\" class=\"card\"><div aria-hidden=\"true\">"));
        assert!(html.ends_with("<div><p class=\"h-2xl\">Atmosphere</p></div></div>"));
        assert_eq!(html.matches("<span></span>").count(), 12);
        c.props.insert("count".into(), "500".into());
        assert_eq!(render(&c).matches("<span></span>").count(), 40);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("particles", "Drift"));
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("particles", "Drift"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"particles\""));
        });
    }

    #[test]
    fn chrome_particles_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"particles\"] {\n  position: relative; overflow: hidden;\n  width: var(--cui-particles-w, 100%); height: 8rem;\n}"));
        assert!(css.contains("[data-slot=\"particles\"] > [aria-hidden] {"));
        assert!(
            css.contains("[data-slot=\"particles\"] > div:last-child {\n  position: relative;\n}")
        );
        assert!(css.contains("@keyframes cui-particles"));
        assert!(css.contains("animation: cui-particles 20s linear infinite alternate;"));
        assert!(css.contains("color-mix(in oklch, var(--cronus-fg) 35%, transparent)"));
        // React speck 1: x = 97 % w, y = 53 % h, vx = -0.12 px/frame, vy = 0, r = 1.4.
        assert!(css.contains("> span:nth-child(2) { --cui-x: 97%; --cui-y: 53%; --cui-vx: -144px; --cui-vy: 0px; --cui-r: 1.4px; }"));
        for i in 1..=MAX_SPECKS {
            assert!(
                css.contains(&format!(
                    "[data-slot=\"particles\"] > [aria-hidden] > span:nth-child({i}) {{ --cui-x: "
                )),
                "{i}"
            );
        }
        assert!(css.contains("prefers-reduced-motion"));
        assert!(css.contains("[data-slot=\"particles\"].card {\n  display: grid; place-items: center; height: auto; min-height: 14rem;"));
        assert!(!css.contains("<canvas"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
