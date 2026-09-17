//! Dedicated AspectRatio renderer. DOM matches React:
//! `<div data-slot="aspect-ratio">` wrapping the child. React writes the
//! `ratio` as an inline `aspect-ratio`; the kernel emits no inline style, so a
//! `ratio:` prop (`16/9`, `1`, `4/3`, `3/2`, `21/9`, `9/16`, `2/3`) becomes a
//! class (`r-16-9`) and the stylesheet carries each value — 16 / 9 is the
//! default. `src:` (+ `alt:`) renders the docs' `<img class="h-full w-full
//! object-cover">`; `framed:true` is the docs' `overflow-hidden rounded-xl
//! border` wrapper. Width mirrors the audit fixture's `className="w-72"`
//! (18rem) unless the page overrides `--cui-aspect-ratio-w`.
//! Not the catalog `display()` SURF `<section>`.

use crate::cronus_ui_kit::{attr_nonempty, esc, flag, label_of, safe_url};
use crate::parser::ComponentNode;

const RATIOS: &[(&str, &str)] = &[
    ("16/9", "r-16-9"),
    ("1", "r-1-1"),
    ("1/1", "r-1-1"),
    ("4/3", "r-4-3"),
    ("3/2", "r-3-2"),
    ("21/9", "r-21-9"),
    ("9/16", "r-9-16"),
    ("2/3", "r-2-3"),
    ("3/4", "r-3-4"),
];

pub fn render(comp: &ComponentNode) -> String {
    let mut classes: Vec<&str> = Vec::new();
    if let Some(ratio) = attr_nonempty(comp, "ratio") {
        let key: String = ratio.chars().filter(|c| !c.is_whitespace()).collect();
        if let Some((_, class)) = RATIOS.iter().find(|(r, _)| *r == key) {
            classes.push(class);
        }
    }
    if flag(comp, "framed") {
        classes.push("framed");
    }
    let class = if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    };
    let inner = match attr_nonempty(comp, "src") {
        Some(src) => {
            let alt = attr_nonempty(comp, "alt").map(esc).unwrap_or_default();
            format!("<img src=\"{}\" alt=\"{alt}\">", safe_url(src))
        }
        None => label_of(comp),
    };
    format!("<div data-slot=\"aspect-ratio\"{class}>{inner}</div>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const DISPLAY_SURF: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem;";

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_display(html: &str) {
        assert!(!html.contains(DISPLAY_SURF));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("<section"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_wraps_label_text_not_display_surf_section() {
        let html = render(&stub("aspect-ratio", "Cover"));
        assert_eq!(html, "<div data-slot=\"aspect-ratio\">Cover</div>");
        reject_display(&html);
    }

    #[test]
    fn extra_text_does_not_add_nodes() {
        let mut c = stub("aspect-ratio", "Cover");
        c.items.push(extra("text", "More"));
        let html = render(&c);
        assert_eq!(html, "<div data-slot=\"aspect-ratio\">Cover</div>");
        reject_display(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("aspect-ratio", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"aspect-ratio\">A &lt;B&gt; &amp; &quot;C&quot;</div>"
        );
        reject_display(&html);
    }

    /// Docs "Default" / "Square": `ratio:` is a class (no inline style),
    /// `src:` + `alt:` the stretched image, `framed:true` the rounded border.
    #[test]
    fn ratio_src_and_frame_render_docs_examples() {
        let mut c = stub("aspect-ratio", "Cover");
        c.props.insert("ratio".into(), "16/9".into());
        c.props.insert("src".into(), "/og.png".into());
        c.props.insert("alt".into(), "Cover".into());
        c.props.insert("framed".into(), "true".into());
        assert_eq!(
            render(&c),
            "<div data-slot=\"aspect-ratio\" class=\"r-16-9 framed\"><img src=\"/og.png\" alt=\"Cover\"></div>"
        );
        c.props.insert("ratio".into(), "1".into());
        assert!(render(&c).contains("class=\"r-1-1 framed\""));
        c.props.insert("ratio".into(), "7/5".into());
        assert!(render(&c).contains("class=\"framed\""));
        reject_display(&render(&c));
        let css = include_str!("cronus_ui_css/aspect-ratio.css");
        assert!(css.contains("[data-slot=\"aspect-ratio\"].r-1-1 { aspect-ratio: 1 / 1; }"));
        assert!(css.contains("[data-slot=\"aspect-ratio\"].framed { overflow: hidden; border-radius: var(--cronus-radius-xl); border: 1px solid var(--cronus-border); }"));
        assert!(css.contains("[data-slot=\"aspect-ratio\"] > img { display: block; width: 100%; height: 100%; object-fit: cover; }"));
    }

    #[test]
    fn skips_display_surf_section() {
        let html = render(&stub("aspect-ratio", "Cover"));
        reject_display(&html);
        assert_eq!(
            dedicated_fn_name("aspect-ratio"),
            Some("cronus_ui_aspect_ratio::render")
        );
        assert_eq!(
            renderer_kind("aspect-ratio"),
            RendererKind::Dedicated("cronus_ui_aspect_ratio::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("aspect-ratio", "Cover"));
            reject_display(&html);
        });
    }

    /// wave1t: React fixture box is w-72 (288x162), not the 432px canvas width.
    #[test]
    fn chrome_mirrors_w72_at_16_9() {
        let css = include_str!("cronus_ui_css/aspect-ratio.css");
        assert!(css.contains(
            "[data-slot=\"aspect-ratio\"] {\n  position: relative; width: var(--cui-aspect-ratio-w, 100%); box-sizing: border-box;\n  aspect-ratio: 16 / 9;\n}"
        ));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(DISPLAY_SURF));
    }
}
