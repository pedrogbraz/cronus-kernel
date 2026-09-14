//! Dedicated Orbit renderer. DOM mirrors React `Orbit` + `OrbitRing`:
//! `<div data-slot="orbit">` with the nucleus as bare text, then
//! `orbit-ring` > `orbit-positioner` > `orbit-holder` > `orbit-item`.
//! No `<style>`, no inline `--orbit-angle`: slot angles come from
//! `:nth-child(k):nth-last-child(n)` quantity queries in COMPONENT_CHROME,
//! which also holds `@keyframes cui-orbit-spin`. Zero JS. Not the catalog
//! `fx()` title SURF box.

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

const MAX_ITEMS: usize = 6;

pub fn render(comp: &ComponentNode) -> String {
    let nucleus = label_of(comp);
    let mut items: Vec<String> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "text" | "item") && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .take(MAX_ITEMS)
        .collect();
    if items.is_empty() {
        items = vec![nucleus.clone(); 3];
    }
    let ring = items
        .into_iter()
        .map(|t| {
            format!(
                "<div data-slot=\"orbit-positioner\"><div data-slot=\"orbit-holder\"><div data-slot=\"orbit-item\">{t}</div></div></div>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    // `aria-label:` after an item line lands in that item's config (the
    // tokenizer has no newlines), so look there as well as in props.
    let aria_label = comp
        .props
        .get("aria-label")
        .or_else(|| comp.items.iter().find_map(|i| i.config.get("aria-label")));
    let aria = match aria_label.filter(|s| !s.is_empty()) {
        Some(v) => format!(" aria-label=\"{}\"", esc(v)),
        None => String::new(),
    };
    format!("<div data-slot=\"orbit\"{aria}>{nucleus}<div data-slot=\"orbit-ring\">{ring}</div></div>")
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

    fn slot(t: &str) -> String {
        format!(
            "<div data-slot=\"orbit-positioner\"><div data-slot=\"orbit-holder\"><div data-slot=\"orbit-item\">{t}</div></div></div>"
        )
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
    fn label_only_is_nucleus_with_three_slots() {
        let html = render(&stub("orbit", "Core"));
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"orbit\">Core<div data-slot=\"orbit-ring\">{}{}{}</div></div>",
                slot("Core"),
                slot("Core"),
                slot("Core")
            )
        );
        assert_eq!(html.matches("data-slot=\"orbit-positioner\"").count(), 3);
        assert_eq!(html.matches("data-slot=\"orbit-holder\"").count(), 3);
        reject_fx(&html);
    }

    /// Audit fixture shape: emitter writes `label "Orbit"` (from aria-label),
    /// `text` per item and `aria-label:"Orbit"`, which the parser attaches to
    /// the last text item. React: nucleus "Orbit", ring A/B/C, root aria-label.
    #[test]
    fn fixture_items_orbit_label_is_nucleus_and_aria() {
        let mut c = stub("orbit", "Orbit");
        c.items.push(extra("text", "A"));
        c.items.push(extra("text", "B"));
        let mut last = extra("text", "C");
        last.config.insert("aria-label".into(), "Orbit".into());
        c.items.push(last);
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"orbit\" aria-label=\"Orbit\">Orbit<div data-slot=\"orbit-ring\">{}{}{}</div></div>",
                slot("A"),
                slot("B"),
                slot("C")
            )
        );
        assert!(!html.contains("aria-hidden"));
        assert!(!html.contains("orbit-nucleus"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("orbit", "A <B> & \"C\""));
        let esc_label = "A &lt;B&gt; &amp; &quot;C&quot;";
        assert!(html.starts_with(&format!("<div data-slot=\"orbit\">{esc_label}<div")));
        assert_eq!(html.matches(&slot(esc_label)).count(), 3);
        assert!(!html.contains("<B>"));
        reject_fx(&html);
    }

    #[test]
    fn extra_texts_become_orbit_items_not_nucleus() {
        let mut c = stub("orbit", "Nucleus");
        c.items.push(extra("text", "Alpha"));
        c.items.push(extra("text", "Beta"));
        c.items.push(extra("text", "Gamma"));
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"orbit\">Nucleus<div data-slot=\"orbit-ring\">"));
        assert_eq!(html.matches("data-slot=\"orbit-item\"").count(), 3);
        assert!(html.contains(&slot("Alpha")));
        assert!(html.contains(&slot("Beta")));
        assert!(html.contains(&slot("Gamma")));
        assert!(!html.contains(&slot("Nucleus")));
        reject_fx(&html);
    }

    #[test]
    fn orbit_items_cap_at_six() {
        let mut c = stub("orbit", "Nucleus");
        for name in ["A", "B", "C", "D", "E", "F", "G"] {
            c.items.push(extra("text", name));
        }
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"orbit-item\"").count(), 6);
        assert!(html.contains(&slot("F")));
        assert!(!html.contains(&slot("G")));
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
            assert!(html.contains("data-slot=\"orbit-ring\""));
            assert_eq!(html.matches("data-slot=\"orbit-item\"").count(), 3);
        });
    }

    #[test]
    fn chrome_orbit_mirrors_react_mechanism() {
        let css = crate::cronus_ui::component_chrome_css();
        for slot in ["orbit", "orbit-ring", "orbit-positioner", "orbit-holder", "orbit-item"] {
            assert!(css.contains(&format!("[data-slot=\"{slot}\"]")), "{slot}");
        }
        assert!(css.contains("@keyframes cui-orbit-spin"));
        assert!(css.contains("width: 18rem"));
        assert!(css.contains("width: 16rem"));
        // Tailwind preflight parity: 256px border-box ring, 1.5 line-height.
        assert!(css.contains("box-sizing: border-box"));
        assert!(css.contains("line-height: 1.5"));
        assert!(css.contains("color-mix(in oklch, var(--cronus-border) 40%, transparent)"));
        assert!(css.contains("rotate: var(--orbit-angle, 0deg)"));
        assert!(css.contains("rotate: calc(var(--orbit-angle, 0deg) * -1)"));
        assert!(css.contains("cui-orbit-spin 24s linear infinite reverse"));
        // 360/n distribution, e.g. 3 items at 0/120/240 like React.
        assert!(css.contains(
            "[data-slot=\"orbit-positioner\"]:nth-child(2):nth-last-child(2) { --orbit-angle: 120deg; }"
        ));
        assert!(css.contains(
            "[data-slot=\"orbit-positioner\"]:nth-child(3):nth-last-child(1) { --orbit-angle: 240deg; }"
        ));
        assert!(css.contains(
            "[data-slot=\"orbit-positioner\"]:nth-child(6):nth-last-child(1) { --orbit-angle: 300deg; }"
        ));
        assert!(css.contains("[data-slot=\"orbit\"]:hover [data-slot=\"orbit-positioner\"]"));
        assert!(css.contains("[data-slot=\"orbit\"]:focus-within [data-slot=\"orbit-item\"]"));
        assert!(css.contains("animation-play-state: paused"));
        assert!(css.contains("prefers-reduced-motion"));
        assert!(!css.contains("[data-slot=\"orbit-nucleus\"]"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
        assert!(!css.contains("<style"));
        assert!(!css.contains("style="));
    }
}
