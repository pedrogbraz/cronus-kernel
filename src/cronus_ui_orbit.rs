//! Dedicated Orbit renderer. DOM mirrors React `Orbit` + `OrbitRing`:
//! `<div data-slot="orbit">` with the nucleus, then one `orbit-ring` per
//! ring > `orbit-positioner` > `orbit-holder` > `orbit-item`.
//! No `<style>`, no inline `--orbit-angle`: slot angles come from
//! `:nth-child(k):nth-last-child(n)` quantity queries in the family CSS,
//! which also holds `@keyframes cui-orbit-spin`. Zero JS.
//!
//! Nucleus: the `label` text, an `icon:` chip (`size-14 rounded-2xl`) or a
//! `value "04" meta:"on call"` metric. Rings: a `slot "ring"` line starts a
//! ring — `radius:<px>` (`r-*` class, sizes the track), `duration:<s>`
//! (`d-*`), `reverse:true`, `start:<deg>` (`a-*`, the `startAngle`),
//! `guide:false` — and the `item` lines after it are its slots: bare text,
//! an `icon:` tool chip (`role="img"`, named by the text) or `initials:`
//! avatar (`title` = the text). Without slots every item sits on one default
//! ring (radius 128, 24 s), the audit shape. `size:64|80` is the docs
//! `size-*`. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::{attr, attr_nonempty, esc, label_of, truthy};
use crate::parser::{ComponentItemNode, ComponentNode};

const MAX_ITEMS: usize = 8;
/// Ring radii with a size class in the family CSS (px).
pub const RADII: [u32; 15] = [
    48, 56, 64, 72, 80, 88, 96, 104, 112, 120, 128, 136, 144, 152, 160,
];

fn cfg<'a>(i: &'a ComponentItemNode, key: &str) -> Option<&'a str> {
    i.config
        .get(key)
        .map(String::as_str)
        .filter(|v| !v.is_empty())
}

fn slot_html(item: &ComponentItemNode) -> String {
    let inner = match (cfg(item, "icon"), cfg(item, "initials")) {
        (Some(icon), _) if crate::cronus_ui_icons::has(icon) => format!(
            "<span role=\"img\" aria-label=\"{}\" class=\"chip\">{}</span>",
            esc(&item.text),
            crate::cronus_ui_icons::svg_or_empty(icon)
        ),
        (_, Some(initials)) => format!(
            "<span title=\"{}\" class=\"avatar\">{}</span>",
            esc(&item.text),
            esc(initials)
        ),
        _ => esc(&item.text),
    };
    format!(
        "<div data-slot=\"orbit-positioner\"><div data-slot=\"orbit-holder\"><div data-slot=\"orbit-item\">{inner}</div></div></div>"
    )
}

fn ring_html(ring: Option<&ComponentItemNode>, slots: &[String]) -> String {
    let mut classes: Vec<String> = Vec::new();
    if let Some(r) = ring {
        if let Some(radius) = cfg(r, "radius").and_then(|v| v.trim().parse::<f64>().ok()) {
            let nearest = RADII
                .iter()
                .copied()
                .min_by_key(|c| (*c as f64 - radius).abs() as u32)
                .unwrap_or(128);
            if nearest != 128 {
                classes.push(format!("r-{nearest}"));
            }
        }
        if let Some(d) = cfg(r, "duration").and_then(|v| v.trim().parse::<f64>().ok()) {
            let s = (d.round() as u32).clamp(1, 60);
            if s != 24 {
                classes.push(format!("d-{s}"));
            }
        }
        if cfg(r, "reverse").is_some_and(truthy) {
            classes.push("reverse".into());
        }
        if let Some(a) = cfg(r, "start").and_then(|v| v.trim().parse::<f64>().ok()) {
            let a = ((a.round() as i64).rem_euclid(360)) as u32;
            if a != 0 {
                classes.push(format!("a-{a}"));
            }
        }
        if cfg(r, "guide").is_some_and(|g| !truthy(g)) {
            classes.push("no-guide".into());
        }
    }
    let class = if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    };
    format!(
        "<div data-slot=\"orbit-ring\"{class}>{}</div>",
        slots.concat()
    )
}

fn nucleus(comp: &ComponentNode) -> String {
    if let Some(icon) = comp.props.get("icon") {
        if crate::cronus_ui_icons::has(icon) {
            return format!(
                "<span class=\"core\">{}</span>",
                crate::cronus_ui_icons::svg_or_empty(icon)
            );
        }
    }
    if let Some(v) = comp
        .items
        .iter()
        .find(|i| i.item_type == "value" && !i.text.is_empty())
    {
        let meta = cfg(v, "meta")
            .map(|m| format!("<span>{}</span>", esc(m)))
            .unwrap_or_default();
        return format!(
            "<div class=\"metric\"><span>{}</span>{meta}</div>",
            esc(&v.text)
        );
    }
    label_of(comp)
}

pub fn render(comp: &ComponentNode) -> String {
    let nucleus = nucleus(comp);
    let mut rings: Vec<String> = Vec::new();
    let mut current: Option<&ComponentItemNode> = None;
    let mut slots: Vec<String> = Vec::new();
    let mut any_ring = false;
    for i in &comp.items {
        if i.item_type == "slot" {
            if any_ring {
                rings.push(ring_html(current, &slots));
                slots.clear();
            }
            any_ring = true;
            current = Some(i);
            continue;
        }
        if matches!(i.item_type.as_str(), "text" | "item")
            && !i.text.is_empty()
            && slots.len() < MAX_ITEMS
        {
            slots.push(slot_html(i));
        }
    }
    if slots.is_empty() && !any_ring {
        let plain = format!(
            "<div data-slot=\"orbit-positioner\"><div data-slot=\"orbit-holder\"><div data-slot=\"orbit-item\">{nucleus}</div></div></div>"
        );
        slots = vec![plain.clone(), plain.clone(), plain];
    }
    rings.push(ring_html(current, &slots));
    // `aria-label:` after an item line lands in that item's config (the
    // tokenizer has no newlines), so look there as well as in props.
    let aria_label = attr(comp, "aria-label");
    let aria = match aria_label.filter(|s| !s.is_empty()) {
        Some(v) => format!(" aria-label=\"{}\"", esc(v)),
        None => String::new(),
    };
    let class = match attr_nonempty(comp, "size") {
        Some(s @ ("64" | "72" | "80" | "96")) => format!(" class=\"sz-{s}\""),
        _ => String::new(),
    };
    format!(
        "<div data-slot=\"orbit\"{class}{aria}>{nucleus}{}</div>",
        rings.concat()
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
    fn orbit_items_cap_at_eight() {
        let mut c = stub("orbit", "Nucleus");
        for name in ["A", "B", "C", "D", "E", "F", "G", "H", "I"] {
            c.items.push(extra("text", name));
        }
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"orbit-item\"").count(), 8);
        assert!(html.contains(&slot("H")));
        assert!(!html.contains(&slot("I")));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("orbit", "Core");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        assert!(!html.contains("<span"));
        reject_fx(&html);
        assert_eq!(dedicated_fn_name("orbit"), Some("cronus_ui_orbit::render"));
        assert_eq!(
            renderer_kind("orbit"),
            RendererKind::Dedicated("cronus_ui_orbit::render")
        );
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

    /// Docs "Integration constellation": zap core, two rings (72px / 22s and
    /// 128px / 36s reversed from 36deg) of icon chips; "Team halo": a metric
    /// core with a guide-less 96px ring of initials avatars from 45deg.
    #[test]
    fn rings_from_slots_with_chips_avatars_and_a_core() {
        let mut c = stub("orbit", "Core");
        c.props.insert(
            "aria-label".into(),
            "Tools orbiting the product core".into(),
        );
        c.props.insert("size".into(), "80".into());
        c.props.insert("icon".into(), "zap".into());
        let ring = |cfg: &[(&str, &str)]| {
            let mut s = extra("slot", "ring");
            for (k, v) in cfg {
                s.config.insert(k.to_string(), v.to_string());
            }
            s
        };
        let tool = |label: &str, icon: &str| {
            let mut i = extra("item", label);
            i.config.insert("icon".into(), icon.into());
            i
        };
        c.items.push(ring(&[("radius", "72"), ("duration", "22")]));
        c.items.push(tool("Search", "search"));
        c.items.push(tool("Alerts", "bell"));
        c.items.push(ring(&[
            ("radius", "128"),
            ("duration", "36"),
            ("reverse", "true"),
            ("start", "36"),
        ]));
        c.items.push(tool("GitHub", "github"));
        c.items.push(tool("Security", "shield-check"));
        c.items.push(tool("Chat", "message-square-plus"));
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"orbit\" class=\"sz-80\" aria-label=\"Tools orbiting the product core\"><span class=\"core\"><svg"));
        assert!(html.contains("</span><div data-slot=\"orbit-ring\" class=\"r-72 d-22\"><div data-slot=\"orbit-positioner\"><div data-slot=\"orbit-holder\"><div data-slot=\"orbit-item\"><span role=\"img\" aria-label=\"Search\" class=\"chip\"><svg"));
        assert!(html.contains("</div></div></div></div><div data-slot=\"orbit-ring\" class=\"d-36 reverse a-36\"><div data-slot=\"orbit-positioner\">"));
        assert_eq!(html.matches("data-slot=\"orbit-ring\"").count(), 2);
        assert_eq!(html.matches("data-slot=\"orbit-item\"").count(), 5);
        assert!(!html.contains(">Core<"));
        reject_fx(&html);
        let mut t = stub("orbit", "Team");
        t.props.insert("size".into(), "64".into());
        let mut v = extra("value", "04");
        v.config.insert("meta".into(), "on call".into());
        t.items.push(v);
        t.items.push(ring(&[
            ("radius", "96"),
            ("duration", "30"),
            ("guide", "false"),
            ("start", "45"),
        ]));
        let mut ana = extra("item", "Ana Ribeiro");
        ana.config.insert("initials".into(), "AR".into());
        t.items.push(ana);
        let html = render(&t);
        assert!(html.starts_with("<div data-slot=\"orbit\" class=\"sz-64\"><div class=\"metric\"><span>04</span><span>on call</span></div><div data-slot=\"orbit-ring\" class=\"r-96 d-30 a-45 no-guide\"><div data-slot=\"orbit-positioner\"><div data-slot=\"orbit-holder\"><div data-slot=\"orbit-item\"><span title=\"Ana Ribeiro\" class=\"avatar\">AR</span></div>"));
        reject_fx(&html);
    }

    #[test]
    fn chrome_orbit_mirrors_react_mechanism() {
        let css = include_str!("cronus_ui_css/orbit.css");
        for slot in [
            "orbit",
            "orbit-ring",
            "orbit-positioner",
            "orbit-holder",
            "orbit-item",
        ] {
            assert!(css.contains(&format!("[data-slot=\"{slot}\"]")), "{slot}");
        }
        assert!(css.contains("@keyframes cui-orbit-spin"));
        assert!(css.contains("width: var(--cui-orbit-size, 18rem)"));
        assert!(css.contains("width: 16rem"));
        // Tailwind preflight parity: 256px border-box ring, 1.5 line-height.
        assert!(css.contains("box-sizing: border-box"));
        assert!(css.contains("line-height: 1.5"));
        assert!(css.contains("color-mix(in oklch, var(--cronus-border) 40%, transparent)"));
        assert!(css.contains("rotate: calc(var(--orbit-start, 0deg) + var(--orbit-angle, 0deg))"));
        assert!(css
            .contains("rotate: calc((var(--orbit-start, 0deg) + var(--orbit-angle, 0deg)) * -1)"));
        assert!(css.contains("animation: cui-orbit-spin var(--orbit-duration, 24s) linear infinite var(--orbit-counter-direction, reverse);"));
        assert!(css.contains("[data-slot=\"orbit-ring\"].reverse { --orbit-spin-direction: reverse; --orbit-counter-direction: normal; }"));
        assert!(css.contains("[data-slot=\"orbit-ring\"].r-72 { --orbit-radius: 72px; width: calc(var(--orbit-radius) * 2); height: calc(var(--orbit-radius) * 2); }"));
        assert!(css.contains("[data-slot=\"orbit-ring\"].d-22 { --orbit-duration: 22s; }"));
        assert!(css.contains("[data-slot=\"orbit-ring\"].a-36 { --orbit-start: 36deg; }"));
        assert!(css.contains("[data-slot=\"orbit-ring\"].no-guide { border: 0; }"));
        assert!(css.contains("[data-slot=\"orbit\"].sz-80 { --cui-orbit-size: 20rem; }"));
        assert!(css.contains("[data-slot=\"orbit-positioner\"]:nth-child(2):nth-last-child(7) { --orbit-angle: 45deg; }"));
        assert!(css.contains("[data-slot=\"orbit\"] .core {"));
        assert!(css.contains("[data-slot=\"orbit\"] .chip {"));
        assert!(css.contains("[data-slot=\"orbit\"] .avatar {"));
        assert!(css.contains("[data-slot=\"orbit\"] .metric {"));
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
