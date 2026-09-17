//! Dedicated Reveal renderer. DOM matches React: `<div data-slot="reveal">`
//! wrapping the content. React plays `fadeInUp` (opacity 0 / y 16 → 1 / 0,
//! 0.5 s ease-out-quart) once the block is in view; the kernel plays the same
//! keyframe on load (the specimen is in view), `delay:<s>` (0.1 s steps,
//! `d-<n>` class) offsets it like React's `delay` prop.
//!
//! Content: the label, or — `card:true` — the docs Card: `icon:` chip,
//! `title` (CardTitle), first `text` (CardDescription), further `text`
//! paragraphs and `item "Fade" description:"opacity 0 → 1"` stat tiles in a
//! three-column grid. Not the catalog `fx()` title SURF box.

use crate::cronus_ui_glass_card::{feature_content, glyph};
use crate::cronus_ui_kit::{attr_num, esc, flag};
use crate::parser::ComponentNode;

fn card(comp: &ComponentNode) -> String {
    let glyph = glyph(comp).unwrap_or_default();
    let title = comp
        .items
        .iter()
        .find(|i| i.item_type == "title" && !i.text.is_empty())
        .map(|i| format!("<div data-slot=\"card-title\">{}</div>", esc(&i.text)))
        .unwrap_or_default();
    let texts: Vec<&str> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "text" && !i.text.is_empty())
        .map(|i| i.text.as_str())
        .collect();
    let description = texts
        .first()
        .map(|t| format!("<div data-slot=\"card-description\">{}</div>", esc(t)))
        .unwrap_or_default();
    let paragraphs: String = texts
        .iter()
        .skip(1)
        .map(|t| format!("<p>{}</p>", esc(t)))
        .collect();
    let tiles: String = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty())
        .map(|i| {
            let hint = i
                .config
                .get("description")
                .filter(|d| !d.is_empty())
                .map(|d| format!("<p>{}</p>", esc(d)))
                .unwrap_or_default();
            format!("<div><p>{}</p>{hint}</div>", esc(&i.text))
        })
        .collect();
    let grid = if tiles.is_empty() {
        String::new()
    } else {
        format!("<div class=\"tiles\">{tiles}</div>")
    };
    let content = if paragraphs.is_empty() && grid.is_empty() {
        String::new()
    } else {
        format!("<div data-slot=\"card-content\">{paragraphs}{grid}</div>")
    };
    format!(
        "<div data-slot=\"card\"><div data-slot=\"card-header\">{glyph}{title}{description}</div>{content}</div>"
    )
}

pub fn render(comp: &ComponentNode) -> String {
    let class = attr_num::<f64>(comp, "delay")
        .filter(|d| *d > 0.0)
        .map(|d| format!(" class=\"d-{}\"", ((d * 10.0).round() as u32).clamp(1, 20)))
        .unwrap_or_default();
    let inner = if flag(comp, "card") {
        card(comp)
    } else {
        feature_content(comp)
    };
    format!("<div data-slot=\"reveal\"{class}>{inner}</div>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_glass_card::tests::item;
    use crate::cronus_ui_kit::stub;

    const FX_BOX: &str = "padding:0.75rem 1rem;position:relative;overflow:hidden";
    const CSS: &str = include_str!("cronus_ui_css/reveal.css");

    fn reject_fx(html: &str) {
        assert!(!html.contains(FX_BOX));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_wraps_label_text_not_fx_title_box() {
        let html = render(&stub("reveal", "Headline"));
        assert_eq!(html, "<div data-slot=\"reveal\">Headline</div>");
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"reveal\""));
        assert!(html.contains(">Headline</div>"));
        assert!(!html.contains("<span"));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("reveal", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"reveal\">A &lt;B&gt; &amp; &quot;C&quot;</div>"
        );
        reject_fx(&html);
    }

    /// Docs "Scroll reveal": the Card with an `size-11 rounded-xl` chip,
    /// 2xl title, base description, a paragraph and three stat tiles.
    #[test]
    fn card_mode_renders_docs_card() {
        let mut c = stub("reveal", "Reveal");
        c.props.insert("card".into(), "true".into());
        c.props.insert("icon".into(), "sparkles".into());
        c.props.insert("icon-size".into(), "lg".into());
        c.props.insert("delay".into(), "0.2".into());
        c.items.push(item("title", "Reveal as you scroll", &[]));
        c.items
            .push(item("text", "This card fades and slides.", &[]));
        c.items.push(item("text", "Wrap any block.", &[]));
        c.items
            .push(item("item", "Fade", &[("description", "opacity 0 → 1")]));
        c.items
            .push(item("item", "Slide", &[("description", "y 24 → 0")]));
        c.items
            .push(item("item", "Once", &[("description", "no replay")]));
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"reveal\" class=\"d-2\"><div data-slot=\"card\"><div data-slot=\"card-header\"><span class=\"glyph lg\"><svg"));
        assert!(html.contains("</span><div data-slot=\"card-title\">Reveal as you scroll</div><div data-slot=\"card-description\">This card fades and slides.</div></div><div data-slot=\"card-content\"><p>Wrap any block.</p><div class=\"tiles\"><div><p>Fade</p><p>opacity 0 → 1</p></div><div><p>Slide</p><p>y 24 → 0</p></div><div><p>Once</p><p>no replay</p></div></div></div></div></div>"));
        assert!(!html.contains(">Reveal<"));
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let html = render(&stub("reveal", "Headline"));
        assert!(!html.contains("<span"));
        reject_fx(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("reveal", "Headline"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"reveal\""));
        });
    }

    /// `fadeInUp`: 500 ms ease-out-quart from opacity 0 / 16px below.
    #[test]
    fn chrome_reveal_via_css() {
        assert!(CSS.contains("[data-slot=\"reveal\"]"));
        assert!(CSS.contains("@keyframes cui-reveal"));
        assert!(CSS.contains("animation: cui-reveal 500ms var(--ease-out-quart) both;"));
        assert!(CSS.contains("translateY(16px)"));
        assert!(CSS.contains("var(--ease-out-quart)"));
        assert!(CSS.contains("[data-slot=\"reveal\"].d-2 { animation-delay: 0.2s; }"));
        assert!(CSS.contains("[data-slot=\"reveal\"] > [data-slot=\"card\"] > [data-slot=\"card-header\"] > [data-slot=\"card-title\"] { font-size: 1.5rem; line-height: 2rem; }"));
        assert!(CSS.contains("[data-slot=\"reveal\"] .tiles { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 0.75rem; }"));
        assert!(!CSS.contains("zinc-"));
        assert!(!CSS.contains(FX_BOX));
    }
}
