//! Dedicated Marquee renderer. DOM matches React (motion on):
//! `<div data-slot="marquee">` holding `repeat` (2) `marquee-group` copies —
//! the first named, the clones `aria-hidden` — each with one node per item.
//! Items are the `text` / `item` rows; the `label` row (emitted from
//! `aria-label`) names the region and is never an item.
//!
//! React measures one copy and derives the loop duration from `speed`
//! (px/s). The kernel estimates the copy length from the items (glyph
//! widths, margins, card widths) and rounds the duration to whole seconds
//! (`d-<s>` class, 1–90 s); the keyframe travels one copy plus the gap, so
//! the loop is seamless. `direction:right` reverses it, `pauseOnHover`
//! is `:hover` / `:focus-within`, `fade:false` drops the edge mask.
//!
//! Item looks: `style:marquee+ticker` renders each row as the docs brand
//! word (`mx-8 text-2xl font-semibold tracking-tight`); rows with `name:` /
//! `role:` / `initials:` config render the docs testimonial `Card`
//! (`mx-3 w-80`). Zero JS. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::{attr_num, esc, flag, label_of, style_seg, truthy};
use crate::parser::{ComponentItemNode, ComponentNode};

const DEFAULT_SPEED: f64 = 40.0;
const GAP_PX: f64 = 16.0;
pub const MAX_DURATION: u32 = 90;

fn is_row(i: &ComponentItemNode) -> bool {
    i.item_type != "label" && i.item_type != "title" && !i.text.is_empty()
}

fn cfg<'a>(i: &'a ComponentItemNode, key: &str) -> Option<&'a str> {
    i.config
        .get(key)
        .map(String::as_str)
        .filter(|v| !v.is_empty())
}

enum Look {
    Plain,
    Ticker,
}

/// Non-label rows; falls back to the label so a bare `label` still scrolls.
fn rows(comp: &ComponentNode) -> Vec<&ComponentItemNode> {
    comp.items.iter().filter(|i| is_row(i)).collect()
}

/// Docs testimonial card: quote + avatar initials + name / role.
fn card(i: &ComponentItemNode) -> String {
    let initials = cfg(i, "initials").map(str::to_string).unwrap_or_else(|| {
        cfg(i, "name")
            .unwrap_or("")
            .split_whitespace()
            .take(2)
            .filter_map(|w| w.chars().next())
            .flat_map(char::to_uppercase)
            .collect()
    });
    let role = cfg(i, "role")
        .map(|r| format!("<p>{}</p>", esc(r)))
        .unwrap_or_default();
    format!(
        "<div data-slot=\"card\"><div data-slot=\"card-content\"><p>\"{}\"</p><div><span>{}</span><div><p>{}</p>{role}</div></div></div></div>",
        esc(&i.text),
        esc(&initials),
        esc(cfg(i, "name").unwrap_or(""))
    )
}

fn node(i: &ComponentItemNode, look: &Look) -> String {
    if cfg(i, "name").is_some() || cfg(i, "initials").is_some() {
        return card(i);
    }
    match look {
        Look::Ticker => format!("<span class=\"brand\">{}</span>", esc(&i.text)),
        Look::Plain => format!("<span>{}</span>", esc(&i.text)),
    }
}

/// Estimated width of one node along the scroll axis, in px.
fn node_width(i: &ComponentItemNode, look: &Look) -> f64 {
    if cfg(i, "name").is_some() || cfg(i, "initials").is_some() {
        return 320.0 + 24.0;
    }
    let chars = i.text.chars().count() as f64;
    match look {
        Look::Ticker => chars * 24.0 * 0.58 + 64.0,
        Look::Plain => chars * 14.0 * 0.5 + 24.0,
    }
}

/// React: `duration = max((copy + gap) / speed, 1)` seconds, rounded to the
/// nearest whole second the family CSS has a class for.
fn duration_seconds(comp: &ComponentNode, rows: &[&ComponentItemNode], look: &Look) -> u32 {
    let speed = attr_num::<f64>(comp, "speed")
        .filter(|s| s.is_finite())
        .unwrap_or(DEFAULT_SPEED)
        .max(1.0);
    let copy: f64 = if rows.is_empty() {
        let raw = comp
            .items
            .iter()
            .find(|i| !i.text.is_empty())
            .map(|i| i.text.clone())
            .unwrap_or_else(|| comp.name.clone());
        node_width(
            &ComponentItemNode {
                item_type: "text".into(),
                text: raw,
                link: None,
                tone: None,
                config: Default::default(),
            },
            look,
        )
    } else {
        rows.iter().map(|r| node_width(r, look)).sum::<f64>() + (rows.len() as f64 - 1.0) * GAP_PX
    };
    let seconds = ((copy + GAP_PX) / speed).max(1.0).round() as u32;
    seconds.clamp(1, MAX_DURATION)
}

pub fn render(comp: &ComponentNode) -> String {
    let look = match style_seg(comp, &["ticker"]) {
        Some(_) => Look::Ticker,
        None => Look::Plain,
    };
    let rows = rows(comp);
    let copy = if rows.is_empty() {
        format!("<span>{}</span>", label_of(comp))
    } else {
        rows.iter().map(|r| node(r, &look)).collect::<String>()
    };
    let label = aria_label(comp);
    let repeat = attr_num::<f64>(comp, "repeat")
        .map(|r| r.round().max(2.0) as usize)
        .unwrap_or(2)
        .min(6);
    let mut class = vec![format!("d-{}", duration_seconds(comp, &rows, &look))];
    if let Look::Ticker = look {
        class.push("ticker".into());
    }
    if comp.props.get("direction").is_some_and(|d| d == "right") {
        class.push("reverse".into());
    }
    if comp.props.get("fade").is_some_and(|v| !truthy(v)) {
        class.push("no-fade".into());
    }
    if comp
        .props
        .get("pauseOnHover")
        .or(comp.props.get("pause"))
        .is_some_and(|v| !truthy(v))
    {
        class.push("no-pause".into());
    }
    let vertical = if flag(comp, "vertical") {
        " data-vertical=\"true\""
    } else {
        ""
    };
    let groups = (0..repeat)
        .map(|n| {
            let hidden = if n == 0 { "" } else { " aria-hidden=\"true\"" };
            format!("<div data-slot=\"marquee-group\"{hidden}>{copy}</div>")
        })
        .collect::<String>();
    format!(
        "<div data-slot=\"marquee\" class=\"{}\"{vertical} aria-label=\"{label}\">{groups}</div>",
        class.join(" ")
    )
}

/// `aria-label` from props or any item config (`key:value` after a row
/// attaches to that row), else the label.
fn aria_label(comp: &ComponentNode) -> String {
    if let Some(v) = comp.props.get("aria-label").filter(|v| !v.is_empty()) {
        return esc(v);
    }
    for i in &comp.items {
        if let Some(v) = i.config.get("aria-label").filter(|v| !v.is_empty()) {
            return esc(v);
        }
    }
    label_of(comp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";
    const CSS: &str = include_str!("cronus_ui_css/marquee.css");

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
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    /// Shape emitted for the audit fixture: label + texts, aria-label on the last row.
    fn fixture() -> ComponentNode {
        let mut c = stub("marquee", "Logos");
        c.items.push(extra("text", "Acme"));
        let mut last = extra("text", "Globex");
        last.config.insert("aria-label".into(), "Logos".into());
        c.items.push(last);
        c
    }

    /// Two copies (React `repeat` default), the clone `aria-hidden`; the
    /// label names the region. 4 + 6 glyphs at 7px + paddings + gap → 134px
    /// + 16px gap at 40px/s ≈ 4s.
    #[test]
    fn label_names_region_and_is_not_an_item() {
        let html = render(&fixture());
        assert_eq!(
            html,
            "<div data-slot=\"marquee\" class=\"d-4\" aria-label=\"Logos\"><div data-slot=\"marquee-group\"><span>Acme</span><span>Globex</span></div><div data-slot=\"marquee-group\" aria-hidden=\"true\"><span>Acme</span><span>Globex</span></div></div>"
        );
        assert!(!html.contains("<span>Logos</span>"));
        assert_eq!(html.matches("data-slot=\"marquee-group\"").count(), 2);
        reject_fx(&html);
    }

    #[test]
    fn aria_label_from_props_wins() {
        let mut c = fixture();
        c.props.insert("aria-label".into(), "Partners".into());
        assert!(render(&c).contains("aria-label=\"Partners\""));
    }

    #[test]
    fn bare_label_is_single_item() {
        let html = render(&stub("marquee", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"marquee\" class=\"d-3\" aria-label=\"A &lt;B&gt; &amp; &quot;C&quot;\"><div data-slot=\"marquee-group\"><span>A &lt;B&gt; &amp; &quot;C&quot;</span></div><div data-slot=\"marquee-group\" aria-hidden=\"true\"><span>A &lt;B&gt; &amp; &quot;C&quot;</span></div></div>"
        );
        reject_fx(&html);
    }

    /// Docs "Logo ticker": six brand words at 32px/s → about 32s per loop.
    #[test]
    fn ticker_brand_words_and_speed() {
        let mut c = stub("marquee", "Brands");
        c.style = Some("marquee+ticker".into());
        c.props.insert("speed".into(), "32".into());
        for b in [
            "stripe", "vercel", "linear", "notion", "supabase", "raycast",
        ] {
            c.items.push(extra("item", b));
        }
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"marquee\" class=\"d-32 ticker\" aria-label=\"Brands\"><div data-slot=\"marquee-group\"><span class=\"brand\">stripe</span><span class=\"brand\">vercel</span>"));
        assert_eq!(html.matches("<span class=\"brand\">").count(), 12);
        reject_fx(&html);
    }

    /// Docs "Testimonial wall": `name:` / `role:` / `initials:` rows are the
    /// `Card` with quote, avatar and byline; `direction:right` reverses the
    /// loop. Four 344px cards + gaps at 24px/s → 60s.
    #[test]
    fn testimonial_cards_and_reverse_direction() {
        let mut c = stub("marquee", "Testimonials");
        c.props.insert("speed".into(), "24".into());
        c.props.insert("direction".into(), "right".into());
        for (q, n, r, i) in [
            (
                "We shipped fast.",
                "Ana Ribeiro",
                "Head of Design, Northwind",
                "AR",
            ),
            ("Accessible.", "Marcus Lee", "Staff Engineer, Atlas", "ML"),
            ("Tasteful.", "Priya Nair", "Product Lead, Lumen", "PN"),
            ("Zero lock-in.", "Tomás Costa", "Founder, Brava", "TC"),
        ] {
            let mut row = extra("item", q);
            row.config.insert("name".into(), n.into());
            row.config.insert("role".into(), r.into());
            row.config.insert("initials".into(), i.into());
            c.items.push(row);
        }
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"marquee\" class=\"d-60 reverse\" aria-label=\"Testimonials\"><div data-slot=\"marquee-group\"><div data-slot=\"card\"><div data-slot=\"card-content\"><p>\"We shipped fast.\"</p><div><span>AR</span><div><p>Ana Ribeiro</p><p>Head of Design, Northwind</p></div></div></div></div><div data-slot=\"card\">"));
        assert_eq!(html.matches("data-slot=\"card\"").count(), 8);
        assert!(html.contains("<p>Tomás Costa</p>"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("marquee", "Acme");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
        assert_eq!(
            dedicated_fn_name("marquee"),
            Some("cronus_ui_marquee::render")
        );
        assert_eq!(
            renderer_kind("marquee"),
            RendererKind::Dedicated("cronus_ui_marquee::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("marquee", "Acme"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"marquee-group\""));
        });
    }

    /// The track slides one copy plus the inter-copy gap (seamless seam),
    /// linear, at the `d-<s>` duration; hover / focus pause it; `reverse`
    /// swaps the endpoints; reduced motion stops it and hides the clone.
    #[test]
    fn chrome_marquee_via_css() {
        assert!(CSS.contains("[data-slot=\"marquee\"] {\n  position: relative; display: flex; overflow: hidden;\n  width: var(--cui-marquee-w, 100%);"));
        assert!(CSS.contains("gap: var(--cui-marquee-gap, 1rem);"));
        assert!(CSS.contains("[data-slot=\"marquee-group\"] > span {\n  padding-inline: 0.75rem;\n  font-size: 0.875rem; line-height: 1.25rem;"));
        assert!(CSS.contains("@keyframes cui-marquee {\n  from { transform: translateX(0); }\n  to { transform: translateX(calc(-100% - var(--cui-marquee-gap, 1rem))); }\n}"));
        assert!(CSS
            .contains("animation: cui-marquee var(--cui-marquee-duration, 20s) linear infinite;"));
        assert!(CSS.contains("[data-slot=\"marquee\"].reverse > [data-slot=\"marquee-group\"] { animation-direction: reverse; }"));
        for s in [1, 3, 30, 60, MAX_DURATION] {
            assert!(
                CSS.contains(&format!(
                    "[data-slot=\"marquee\"].d-{s} {{ --cui-marquee-duration: {s}s; }}"
                )),
                "{s}"
            );
        }
        assert!(CSS
            .contains("[data-slot=\"marquee\"].ticker > [data-slot=\"marquee-group\"] > .brand {"));
        assert!(CSS.contains("margin-inline: 2rem; font-size: 1.5rem; line-height: 2rem; font-weight: 600; letter-spacing: -0.025em;"));
        assert!(CSS.contains("[data-slot=\"marquee-group\"] > [data-slot=\"card\"] { margin-inline: 0.75rem; flex: 0 0 20rem; }"));
        assert!(CSS.contains("[data-slot=\"marquee\"][data-vertical] {"));
        assert!(CSS.contains("[data-slot=\"marquee-group\"][aria-hidden] { display: none; }"));
        assert!(CSS.contains("var(--cronus-fg)"));
        assert!(!CSS.contains("zinc-"));
        assert!(!CSS.contains(FX_BOX));
    }
}
