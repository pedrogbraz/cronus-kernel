//! Dedicated TiltCard renderer. DOM mirrors React:
//! `<div data-slot="tilt-card">` + optional `aria-hidden` glare layer
//! (`glare:true`) + a relative content `<div>`. React tilts with pointer JS
//! through `--tilt-rx` / `--tilt-ry`; the kernel keeps the same
//! `perspective(1000px) rotateX/rotateY` transform at its settled 0deg rest
//! and plays the hover state React reaches at the card's centre: `scale`
//! (`scale:1.02|1.03|1.05` → `sc-*` class), `shadow-lg`, the glare sheen
//! fading in and — `parallax:true` — the content lifting `translateZ(42px)`,
//! all on React's 150 ms ease-out / 300 ms transitions. `maxTilt` needs the
//! pointer and has no CSS equivalent.
//!
//! Content (`cronus_ui_glass_card::feature_content`): the default `flex-col
//! gap-3` stack; `style:tilt-card+row` is the docs list row (round avatar
//! glyph, name / email, trailing `icon-end:`); `style:tilt-card+payment` is
//! the inverted payment card (`title` brand + `icon-end:wifi`, chip, `value`
//! number, two `meta` lines). `width:xs|sm` is the docs `max-w-*`.
//! `rounded-2xl`, fixture `w-72` and colors live in the family CSS.
//! Not the catalog `display()` SURF `<section>`.

use crate::cronus_ui_glass_card::feature_content;
use crate::cronus_ui_kit::{attr_nonempty, attr_num, esc, flag, style_seg};
use crate::parser::ComponentNode;

fn payment(comp: &ComponentNode) -> String {
    let brand = comp
        .items
        .iter()
        .find(|i| i.item_type == "title" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .unwrap_or_default();
    let wifi = attr_nonempty(comp, "icon-end")
        .map(|i| crate::cronus_ui_icons::svg_or_empty(i))
        .unwrap_or_default();
    let number = comp
        .items
        .iter()
        .find(|i| i.item_type == "value" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .unwrap_or_default();
    let metas: String = comp
        .items
        .iter()
        .filter(|i| i.item_type == "meta" && !i.text.is_empty())
        .map(|i| format!("<span>{}</span>", esc(&i.text)))
        .collect();
    format!(
        "<div class=\"head\"><span>{brand}</span>{wifi}</div><div class=\"chip\" aria-hidden=\"true\"></div><div class=\"number\"><p>{number}</p><div class=\"row\">{metas}</div></div>"
    )
}

fn row(comp: &ComponentNode) -> String {
    let glyph = crate::cronus_ui_glass_card::glyph(comp).unwrap_or_default();
    let mut clone = comp.clone();
    clone.props.remove("icon");
    let end = attr_nonempty(comp, "icon-end")
        .map(|i| crate::cronus_ui_icons::svg_or_empty(i))
        .unwrap_or_default();
    format!(
        "{glyph}<div class=\"copy\">{}</div>{end}",
        feature_content(&clone)
    )
}

pub fn render(comp: &ComponentNode) -> String {
    let look = style_seg(comp, &["row", "payment"]);
    let content = match look {
        Some("payment") => payment(comp),
        Some("row") => row(comp),
        _ => feature_content(comp),
    };
    let mut classes: Vec<String> = Vec::new();
    if let Some(l) = look {
        classes.push(l.into());
    }
    if flag(comp, "parallax") {
        classes.push("parallax".into());
    }
    if let Some(s) = attr_num::<f64>(comp, "scale") {
        let pct = (s * 100.0).round() as u32;
        if pct != 103 && (100..=110).contains(&pct) {
            classes.push(format!("sc-{pct}"));
        }
    }
    if let Some(w @ ("xs" | "sm" | "md")) = attr_nonempty(comp, "width") {
        classes.push(format!("w-{w}"));
    }
    let class = if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    };
    let glare = if flag(comp, "glare") {
        "<div aria-hidden=\"true\"></div>"
    } else {
        ""
    };
    format!("<div data-slot=\"tilt-card\"{class}>{glare}<div>{content}</div></div>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_glass_card::tests::item;
    use crate::cronus_ui_kit::stub;

    const DISPLAY_SURF: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem;";
    const CSS: &str = include_str!("cronus_ui_css/tilt-card.css");

    fn reject_display(html: &str) {
        assert!(!html.contains(DISPLAY_SURF));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<section"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_wraps_content_div_not_display_surf_section() {
        let html = render(&stub("tilt-card", "Tilt"));
        assert_eq!(html, "<div data-slot=\"tilt-card\"><div>Tilt</div></div>");
        assert_eq!(html.matches("data-slot=").count(), 1);
        reject_display(&html);
    }

    /// Docs "Glare & parallax": glare layer before the content, parallax
    /// class, `max-w-xs`, chip + lg heading + copy.
    #[test]
    fn glare_parallax_and_feature_content() {
        let mut c = stub("tilt-card", "Repasses");
        c.props.insert("glare".into(), "true".into());
        c.props.insert("parallax".into(), "true".into());
        c.props.insert("width".into(), "xs".into());
        c.props.insert("icon".into(), "zap".into());
        c.props.insert("icon-size".into(), "lg".into());
        c.items
            .push(item("title", "Repasses instantâneos", &[("size", "lg")]));
        c.items
            .push(item("text", "O saldo entra no mesmo instante.", &[]));
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"tilt-card\" class=\"parallax w-xs\"><div aria-hidden=\"true\"></div><div><span class=\"glyph lg\"><svg"));
        assert!(html.ends_with("</span><h3 class=\"t-lg\">Repasses instantâneos</h3><p>O saldo entra no mesmo instante.</p></div></div>"));
        reject_display(&html);
    }

    /// Docs "Payment card" (`style:tilt-card+payment`, scale 1.05, `max-w-sm`)
    /// and "Subtle" (`style:tilt-card+row`, scale 1.02, no glare).
    #[test]
    fn payment_and_row_looks() {
        let mut c = stub("tilt-card", "Card");
        c.style = Some("tilt-card+payment".into());
        c.props.insert("glare".into(), "true".into());
        c.props.insert("parallax".into(), "true".into());
        c.props.insert("scale".into(), "1.05".into());
        c.props.insert("width".into(), "sm".into());
        c.props.insert("icon-end".into(), "wifi".into());
        c.items.push(item("title", "Cronus", &[]));
        c.items.push(item("value", "4242 4242 4242 4242", &[]));
        c.items.push(item("meta", "Pedro Gontijo", &[]));
        c.items.push(item("meta", "12/29", &[]));
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"tilt-card\" class=\"payment parallax sc-105 w-sm\"><div aria-hidden=\"true\"></div><div><div class=\"head\"><span>Cronus</span><svg"));
        assert!(html.contains("data-icon=\"wifi\""));
        assert!(html.ends_with("</svg></div><div class=\"chip\" aria-hidden=\"true\"></div><div class=\"number\"><p>4242 4242 4242 4242</p><div class=\"row\"><span>Pedro Gontijo</span><span>12/29</span></div></div></div></div>"));
        reject_display(&html);
        let mut r = stub("tilt-card", "Ana");
        r.style = Some("tilt-card+row".into());
        r.props.insert("scale".into(), "1.02".into());
        r.props.insert("width".into(), "xs".into());
        r.props.insert("icon".into(), "user".into());
        r.props.insert("icon-size".into(), "lg".into());
        r.props.insert("icon-shape".into(), "round".into());
        r.props.insert("icon-tone".into(), "secondary".into());
        r.props.insert("icon-end".into(), "arrow-right".into());
        r.items.push(item("title", "Ana Ribeiro", &[]));
        r.items.push(item("text", "ana@cronus.app", &[]));
        let html = render(&r);
        assert!(html.starts_with("<div data-slot=\"tilt-card\" class=\"row sc-102 w-xs\"><div><span class=\"glyph lg round secondary\"><svg"));
        assert!(html.contains(
            "</span><div class=\"copy\"><h3>Ana Ribeiro</h3><p>ana@cronus.app</p></div><svg"
        ));
        assert!(html.contains("data-icon=\"arrow-right\""));
        assert!(!html.contains("aria-hidden=\"true\"></div>"));
        reject_display(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("tilt-card", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"tilt-card\"><div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
        reject_display(&html);
    }

    #[test]
    fn skips_display_surf_section() {
        let html = render(&stub("tilt-card", "Tilt"));
        reject_display(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("tilt-card", "Tilt"));
            reject_display(&html);
            assert!(html.contains("data-slot=\"tilt-card\""));
        });
    }

    #[test]
    fn chrome_tilt_via_css() {
        assert!(CSS.contains("[data-slot=\"tilt-card\"] {\n  position: relative;\n  width: var(--cui-tilt-card-w, 100%);\n  border-radius: calc(var(--cronus-radius, 14px) + 8px);"));
        assert!(
            CSS.contains("[data-slot=\"tilt-card\"] > div:last-child {\n  position: relative;\n}")
        );
        assert!(CSS.contains("transform-style: preserve-3d"));
        assert!(CSS.contains("perspective(1000px)"));
        assert!(CSS.contains("var(--cronus-surface-raised)"));
        assert!(CSS.contains("var(--cronus-border)"));
        assert!(!CSS.contains("zinc-"));
        assert!(!CSS.contains(DISPLAY_SURF));
    }

    /// Hover reaches React's `data-tilting` state at the card's centre:
    /// scale, `shadow-lg`, glare sheen (300 ms) and parallax lift (150 ms).
    #[test]
    fn chrome_hover_state_glare_parallax_and_looks() {
        assert!(CSS.contains("transition: transform 150ms ease-out, box-shadow 150ms ease-out;"));
        assert!(CSS.contains("[data-slot=\"tilt-card\"]:hover { --tilt-scale: 1.03; box-shadow: var(--cronus-shadow-lg, var(--cronus-shadow-md, none)); }"));
        assert!(CSS.contains("[data-slot=\"tilt-card\"].sc-105:hover { --tilt-scale: 1.05; }"));
        assert!(CSS.contains("[data-slot=\"tilt-card\"] > [aria-hidden] {"));
        assert!(CSS.contains("background: radial-gradient(600px circle at var(--tilt-gx, 50%) var(--tilt-gy, 50%), color-mix(in oklch, var(--cronus-fg) 16%, transparent), transparent 60%);"));
        assert!(CSS.contains("[data-slot=\"tilt-card\"]:hover > [aria-hidden] { opacity: 1; }"));
        assert!(CSS.contains("[data-slot=\"tilt-card\"].parallax:hover > div:last-child { transform: translateZ(42px); }"));
        assert!(CSS.contains("[data-slot=\"tilt-card\"].payment { background: var(--cronus-fg); color: var(--cronus-fg-inverse); }"));
        assert!(CSS.contains("[data-slot=\"tilt-card\"].row > div:last-child { display: flex; flex-direction: row; align-items: center; gap: 1rem; }"));
        assert!(CSS.contains("prefers-reduced-motion"));
    }
}
