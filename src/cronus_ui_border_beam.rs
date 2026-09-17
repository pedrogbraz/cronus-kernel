//! Dedicated BorderBeam renderer. DOM matches React idle:
//! `<div data-slot="border-beam">` + `aria-hidden` beam layer + relative
//! children div wrapping the content. Only the root has a `data-slot`, like
//! React (Wave 1t geometry parity). `@keyframes cui-border-beam` lives in the
//! family CSS — never a `<style>` tag. React's knobs ride CSS variables set
//! inline; the kernel sets them through classes on the root: `size:40…120`
//! (`sz-<px>`, head length and corner radius), `duration:1…20` (`dur-<s>`),
//! `reverse:true`, `from:fg|primary|accent`, `to:transparent|primary`.
//!
//! Content: the label, or the docs feature card (`cronus_ui_glass_card::
//! feature_content`, `card` class: `flex-col gap-4 rounded-2xl
//! bg-surface-raised p-6`); `style:border-beam+bar` lays it out as the
//! prompt bar (`items-center gap-3 px-4 py-3`); `gap:3` tightens the card
//! column. `width:xs|sm|md` is the docs `max-w-*`. Zero JS, no inline style. Not the catalog `fx()` title
//! SURF box.

use crate::cronus_ui_glass_card::feature_content;
use crate::cronus_ui_kit::{attr_nonempty, attr_num, flag, label_of, style_seg};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let content = feature_content(comp);
    let mut classes: Vec<String> = Vec::new();
    if content != label_of(comp) {
        classes.push(
            if style_seg(comp, &["bar"]).is_some() {
                "bar"
            } else {
                "card"
            }
            .into(),
        );
    }
    if let Some(w @ ("xs" | "sm" | "md" | "lg")) = attr_nonempty(comp, "width") {
        classes.push(format!("w-{w}"));
    }
    if attr_nonempty(comp, "gap") == Some("3") {
        classes.push("g-3".into());
    }
    if let Some(size) = attr_num::<f64>(comp, "size") {
        let step = ((size / 10.0).round() as u32).clamp(4, 12) * 10;
        if step != 60 {
            classes.push(format!("sz-{step}"));
        }
    }
    if let Some(d) = attr_num::<f64>(comp, "duration") {
        let s = (d.round() as u32).clamp(1, 20);
        if s != 8 {
            classes.push(format!("dur-{s}"));
        }
    }
    if flag(comp, "reverse") {
        classes.push("reverse".into());
    }
    if let Some(c @ ("primary" | "accent")) = attr_nonempty(comp, "from") {
        classes.push(format!("from-{c}"));
    }
    if let Some(c @ ("primary" | "accent" | "fg")) = attr_nonempty(comp, "to") {
        classes.push(format!("to-{c}"));
    }
    let class = if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    };
    format!(
        "<div data-slot=\"border-beam\"{class}><div aria-hidden=\"true\"></div><div>{content}</div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_glass_card::tests::item;
    use crate::cronus_ui_kit::stub;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";
    const CSS: &str = include_str!("cronus_ui_css/border-beam.css");

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
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("border-beam", "Beam"));
        assert_eq!(
            html,
            "<div data-slot=\"border-beam\"><div aria-hidden=\"true\"></div><div>Beam</div></div>"
        );
        assert!(!html.contains("border-beam-layer"));
        assert!(!html.contains("border-beam-content"));
        assert!(!html.contains("<span"));
        assert_eq!(html.matches("data-slot=").count(), 1);
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("border-beam", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"border-beam\"><div aria-hidden=\"true\"></div><div>A &lt;B&gt; &amp; &quot;C&quot;</div></div>"
        );
        reject_fx(&html);
    }

    /// Docs "Featured card": 6 s lap, `max-w-xs`, chip + Popular pill, plan
    /// copy, 3xl price and a full-width primary CTA.
    #[test]
    fn featured_card_knobs_and_content() {
        let mut c = stub("border-beam", "Pro");
        c.props.insert("duration".into(), "6".into());
        c.props.insert("width".into(), "xs".into());
        c.props.insert("icon".into(), "sparkles".into());
        c.items.push(item("badge", "Popular", &[]));
        c.items.push(item("title", "Pro", &[]));
        c.items.push(item(
            "text",
            "Tudo para escalar a sua loja.",
            &[("mt", "1")],
        ));
        c.items.push(item(
            "value",
            "R$ 79",
            &[("meta", "/ mês"), ("size", "3xl")],
        ));
        c.items.push(item(
            "action",
            "Assinar o Pro",
            &[("icon-end", "arrow-right")],
        ));
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"border-beam\" class=\"card w-xs dur-6\"><div aria-hidden=\"true\"></div><div><div class=\"row\"><span class=\"glyph\"><svg"));
        assert!(html.contains("<span class=\"pill\">Popular</span></div><div class=\"copy\"><h3>Pro</h3><p class=\"mt-1\">Tudo para escalar a sua loja.</p></div><div class=\"price v-3xl\"><span>R$ 79</span><span>/ mês</span></div><div class=\"cta\"><button type=\"button\" data-slot=\"button\" data-variant=\"primary\" data-size=\"md\" class=\"cui-btn\">Assinar o Pro<svg"));
        reject_fx(&html);
    }

    /// Docs "Prompt bar" (`style:border-beam+bar`, size 80, 5 s) and "Custom
    /// colours & reverse" (size 90 → 90, reverse, fg → transparent = defaults).
    #[test]
    fn prompt_bar_and_reverse_knobs() {
        let mut c = stub("border-beam", "Prompt");
        c.style = Some("border-beam+bar".into());
        c.props.insert("size".into(), "80".into());
        c.props.insert("duration".into(), "5".into());
        c.props.insert("width".into(), "md".into());
        c.props.insert("icon".into(), "sparkles".into());
        c.props.insert("icon-tone".into(), "primary".into());
        c.items
            .push(item("text", "Pergunte qualquer coisa ao Cronus…", &[]));
        c.items.push(item(
            "action",
            "Enviar",
            &[("size", "icon"), ("icon", "arrow-right")],
        ));
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"border-beam\" class=\"bar w-md sz-80 dur-5\"><div aria-hidden=\"true\"></div><div><span class=\"glyph primary\"><svg"));
        assert!(html.contains("</span><p>Pergunte qualquer coisa ao Cronus…</p><div class=\"cta\"><button type=\"button\" aria-label=\"Enviar\" data-slot=\"button\" data-variant=\"primary\" data-size=\"icon\" class=\"cui-btn\"><svg"));
        reject_fx(&html);
        let mut r = stub("border-beam", "Safe");
        r.props.insert("size".into(), "90".into());
        r.props.insert("duration".into(), "5".into());
        r.props.insert("reverse".into(), "true".into());
        r.props.insert("from".into(), "var(--cronus-fg)".into());
        r.props.insert("to".into(), "transparent".into());
        assert!(
            render(&r).starts_with("<div data-slot=\"border-beam\" class=\"sz-90 dur-5 reverse\">")
        );
        r.props.insert("gap".into(), "3".into());
        r.items.push(item("title", "Pagamentos protegidos", &[]));
        assert!(render(&r)
            .starts_with("<div data-slot=\"border-beam\" class=\"card g-3 sz-90 dur-5 reverse\">"));
        r.props.insert("from".into(), "primary".into());
        assert!(render(&r).contains("class=\"card g-3 sz-90 dur-5 reverse from-primary\""));
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("border-beam", "Beam");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("border-beam"),
            Some("cronus_ui_border_beam::render")
        );
        assert_eq!(
            renderer_kind("border-beam"),
            RendererKind::Dedicated("cronus_ui_border_beam::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("border-beam", "Beam"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"border-beam\""));
        });
    }

    #[test]
    fn chrome_border_beam_via_css() {
        assert!(CSS.contains("[data-slot=\"border-beam\"] > [aria-hidden] {"));
        assert!(CSS.contains("[data-slot=\"border-beam\"] > div:last-child {"));
        assert!(!CSS.contains("[data-slot=\"border-beam-layer\"]"));
        assert!(CSS.contains("@keyframes cui-border-beam"));
        assert!(CSS
            .contains("animation: cui-border-beam var(--cui-beam-duration, 8s) linear infinite;"));
        assert!(CSS.contains("offset-path: rect(0 auto auto 0 round var(--cui-beam-size, 60px));"));
        assert!(
            CSS.contains("width: var(--cui-beam-size, 60px); height: var(--cui-beam-size, 60px);")
        );
        assert!(CSS.contains("background: linear-gradient(to left, var(--cui-beam-from, var(--cronus-fg)), var(--cui-beam-to, transparent), transparent);"));
        assert!(CSS.contains("[data-slot=\"border-beam\"].reverse > [aria-hidden]::after { animation-direction: reverse; }"));
        assert!(CSS.contains("[data-slot=\"border-beam\"].sz-80 { --cui-beam-size: 80px; }"));
        assert!(CSS.contains("[data-slot=\"border-beam\"].dur-6 { --cui-beam-duration: 6s; }"));
        assert!(CSS.contains(
            "[data-slot=\"border-beam\"].from-primary { --cui-beam-from: var(--cronus-primary); }"
        ));
        assert!(CSS.contains("prefers-reduced-motion"));
        assert!(!CSS.contains("zinc-"));
        assert!(!CSS.contains(FX_BOX));
    }

    /// Wave 1t geometry: fixture `w-72 p-6` + React `rounded-2xl` (radius + 8px).
    #[test]
    fn chrome_mirrors_fixture_box_and_2xl_radius() {
        let start = CSS.find("[data-slot=\"border-beam\"] {").unwrap();
        let block = &CSS[start..start + CSS[start..].find('}').unwrap()];
        assert!(block.contains(
            "width: var(--cui-border-beam-w, 100%); box-sizing: border-box; padding: 1.5rem;"
        ));
        assert!(block.contains("border-radius: calc(var(--cronus-radius, 14px) + 8px);"));
        // Docs cards: the root is bare and the content div is the surface.
        assert!(CSS.contains(
            "[data-slot=\"border-beam\"].card, [data-slot=\"border-beam\"].bar { padding: 0; }"
        ));
        assert!(CSS.contains("[data-slot=\"border-beam\"].card > div:last-child {"));
        assert!(CSS.contains("display: flex; flex-direction: column; gap: 1rem; padding: 1.5rem;"));
        assert!(CSS.contains("[data-slot=\"border-beam\"].bar > div:last-child {"));
        assert!(CSS.contains("[data-slot=\"border-beam\"].w-xs { max-width: 20rem; }"));
    }
}
