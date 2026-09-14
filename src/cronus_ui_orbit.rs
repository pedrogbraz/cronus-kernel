//! Dedicated Orbit renderer. DOM is a CSS-only stage:
//! `<div data-slot="orbit">` + `orbit-nucleus` + aria-hidden `orbit-ring`
//! of `orbit-item`s. No `<style>`, no inline `--orbit-angle`. Placement is
//! nth-child in COMPONENT_CHROME; `@keyframes cui-orbit-spin` lives there
//! too. Zero JS. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::{label_of, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let all = texts(comp);
    let (nucleus, items) = if all.len() <= 1 {
        let label = all.first().cloned().unwrap_or_else(|| label_of(comp));
        (label.clone(), vec![label.clone(), label.clone(), label])
    } else {
        let mut iter = all.into_iter();
        let nucleus = iter.next().unwrap_or_else(|| label_of(comp));
        (nucleus, iter.take(6).collect::<Vec<_>>())
    };
    let ring = items
        .into_iter()
        .map(|t| format!("<div data-slot=\"orbit-item\">{t}</div>"))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"orbit\"><div data-slot=\"orbit-nucleus\">{nucleus}</div><div data-slot=\"orbit-ring\" aria-hidden=\"true\">{ring}</div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("<style"));
        assert!(!html.contains("@keyframes"));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<span"));
        assert!(!html.contains("<canvas"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("requestAnimationFrame"));
        assert!(!html.contains("--orbit-angle"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("orbit", "Core"));
        assert_eq!(
            html,
            "<div data-slot=\"orbit\"><div data-slot=\"orbit-nucleus\">Core</div><div data-slot=\"orbit-ring\" aria-hidden=\"true\"><div data-slot=\"orbit-item\">Core</div><div data-slot=\"orbit-item\">Core</div><div data-slot=\"orbit-item\">Core</div></div></div>"
        );
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"orbit\""));
        assert!(html.contains("data-slot=\"orbit-nucleus\""));
        assert!(html.contains("data-slot=\"orbit-ring\""));
        assert!(html.contains("data-slot=\"orbit-item\""));
        assert!(html.contains("aria-hidden=\"true\""));
        assert_eq!(html.matches("data-slot=\"orbit-item\"").count(), 3);
        assert!(html.contains(">Core</div></div></div>"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("orbit", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"orbit\"><div data-slot=\"orbit-nucleus\">A &lt;B&gt; &amp; &quot;C&quot;</div><div data-slot=\"orbit-ring\" aria-hidden=\"true\"><div data-slot=\"orbit-item\">A &lt;B&gt; &amp; &quot;C&quot;</div><div data-slot=\"orbit-item\">A &lt;B&gt; &amp; &quot;C&quot;</div><div data-slot=\"orbit-item\">A &lt;B&gt; &amp; &quot;C&quot;</div></div></div>"
        );
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    #[test]
    fn extra_texts_become_orbit_items() {
        let mut c = stub("orbit", "Nucleus");
        c.items.push(extra("text", "Alpha"));
        c.items.push(extra("text", "Beta"));
        c.items.push(extra("text", "Gamma"));
        let html = render(&c);
        assert!(html.contains("<div data-slot=\"orbit-nucleus\">Nucleus</div>"));
        assert_eq!(html.matches("data-slot=\"orbit-item\"").count(), 3);
        assert!(html.contains("<div data-slot=\"orbit-item\">Alpha</div>"));
        assert!(html.contains("<div data-slot=\"orbit-item\">Beta</div>"));
        assert!(html.contains("<div data-slot=\"orbit-item\">Gamma</div>"));
        assert!(!html.contains("<div data-slot=\"orbit-item\">Nucleus</div>"));
        reject_fx(&html);
    }

    #[test]
    fn orbit_items_cap_at_six() {
        let mut c = stub("orbit", "Nucleus");
        for name in ["A", "B", "C", "D", "E", "F", "G"] {
            c.items.push(extra("text", name));
        }
        let html = render(&c);
        assert!(html.contains("<div data-slot=\"orbit-nucleus\">Nucleus</div>"));
        assert_eq!(html.matches("data-slot=\"orbit-item\"").count(), 6);
        assert!(html.contains("<div data-slot=\"orbit-item\">F</div>"));
        assert!(!html.contains("<div data-slot=\"orbit-item\">G</div>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("orbit", "Core");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert!(fx.contains("<span>"));
        assert!(fx.starts_with("<div data-slot=\"meteors\""));
        assert_ne!(html, fx);
        assert!(!html.contains("<span"));
        reject_fx(&html);
        assert_eq!(dedicated_fn_name("orbit"), Some("cronus_ui_orbit::render"));
        assert_eq!(
            renderer_kind("orbit"),
            RendererKind::Dedicated("cronus_ui_orbit::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("orbit", "Core"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"orbit\""));
            assert!(html.contains("data-slot=\"orbit-nucleus\""));
            assert!(html.contains("data-slot=\"orbit-ring\""));
            assert_eq!(html.matches("data-slot=\"orbit-item\"").count(), 3);
        });
    }

    #[test]
    fn chrome_orbit_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"orbit\"]"));
        assert!(css.contains("[data-slot=\"orbit-nucleus\"]"));
        assert!(css.contains("[data-slot=\"orbit-ring\"]"));
        assert!(css.contains("[data-slot=\"orbit-item\"]"));
        assert!(css.contains("@keyframes cui-orbit-spin"));
        assert!(css.contains("animation: cui-orbit-spin"));
        assert!(css.contains("nth-child(1)"));
        assert!(css.contains("nth-child(2)"));
        assert!(css.contains("rotate(60deg)"));
        assert!(css.contains("rotate(120deg)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("[data-slot=\"orbit\"]:hover"));
        assert!(css.contains("[data-slot=\"orbit\"]:focus-within"));
        assert!(css.contains("animation-play-state: paused"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
        assert!(!css.contains("style="));
    }
}
