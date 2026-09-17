//! Dedicated ImagesBadge renderer. DOM matches React:
//! `<div data-slot="images-badge" tabindex="0">` (an `<a href target rel>` when
//! `href:` is set) holding the 32×24 folder — back panel with its tab, up to
//! three peeking photo cards (`<img>`), the front flap with its crease — and
//! the `<span>` label. React's `motion` values (teaser 20×14 at y −4/−5/−6
//! with ±3° tilt; hover 48×32 at y −35/−38/−41, ±20px spread, ±15° fan; the
//! flap rotateX −25° → −45° and scaleY 1 → 0.8; spring stiffness 400 /
//! damping 25 with a 30ms stagger) are CSS transforms keyed on
//! `data-photos="1|2|3"` and `:nth-child`, transitioned with
//! `var(--cronus-spring-snappy)` on `:hover` / `:focus-visible`. Folder
//! sizes are the React defaults; the size props are not read.
//!
//! Inputs: `label "…"` (text), `item "Preview 1" -> "https://…"` images (the
//! item text is the alt; `src:` config works too), `href:` / `target:`.

use crate::cronus_ui_kit::{attr_nonempty, esc, label_of, safe_url};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let text = label_of(comp);
    let images: Vec<(String, String)> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" || i.item_type == "link")
        .filter_map(|i| {
            let src = i
                .link
                .as_deref()
                .or_else(|| i.config.get("src").map(String::as_str))
                .or_else(|| i.config.get("image").map(String::as_str))?;
            Some((safe_url(src), esc(&i.text)))
        })
        .take(3)
        .collect();
    let count = images.len();
    let photos: String = images
        .iter()
        .enumerate()
        .map(|(n, (src, alt))| {
            let alt = if alt.is_empty() {
                format!("Preview {}", n + 1)
            } else {
                alt.clone()
            };
            format!("<div class=\"photo\"><img src=\"{src}\" alt=\"{alt}\"></div>")
        })
        .collect();
    let folder = format!(
        "<div class=\"folder\" data-photos=\"{count}\"><div class=\"back\"><div class=\"tab\"></div></div>{photos}<div class=\"front\"><div class=\"crease\"></div></div></div>"
    );
    match attr_nonempty(comp, "href") {
        Some(href) => {
            let target = attr_nonempty(comp, "target").map(esc);
            let target_attrs = match target.as_deref() {
                Some("_blank") => " target=\"_blank\" rel=\"noopener noreferrer\"".to_string(),
                Some(t) => format!(" target=\"{t}\""),
                None => String::new(),
            };
            format!(
                "<a href=\"{}\"{target_attrs} data-slot=\"images-badge\">{folder}<span>{text}</span></a>",
                safe_url(href)
            )
        }
        None => format!(
            "<div data-slot=\"images-badge\" tabindex=\"0\">{folder}<span>{text}</span></div>"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn badge(text: &str, images: &[&str]) -> ComponentNode {
        let mut c = stub("images-badge", text);
        for (n, src) in images.iter().enumerate() {
            c.items.push(ComponentItemNode {
                item_type: "item".into(),
                text: format!("Preview {}", n + 1),
                link: Some((*src).into()),
                tone: None,
                config: Default::default(),
            });
        }
        c
    }

    fn reject_js(html: &str) {
        for bad in ["<script", "style=", "onclick", "onmouse", "<canvas"] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    #[test]
    fn docs_folder_example_dom() {
        let html = render(&badge(
            "Introducing Agenforce Marketing Template",
            &[
                "https://assets.aceternity.com/pro/agenforce-1.webp",
                "https://assets.aceternity.com/pro/agenforce-2.webp",
                "https://assets.aceternity.com/pro/agenforce-3.webp",
            ],
        ));
        assert_eq!(
            html,
            "<div data-slot=\"images-badge\" tabindex=\"0\"><div class=\"folder\" data-photos=\"3\"><div class=\"back\"><div class=\"tab\"></div></div><div class=\"photo\"><img src=\"https://assets.aceternity.com/pro/agenforce-1.webp\" alt=\"Preview 1\"></div><div class=\"photo\"><img src=\"https://assets.aceternity.com/pro/agenforce-2.webp\" alt=\"Preview 2\"></div><div class=\"photo\"><img src=\"https://assets.aceternity.com/pro/agenforce-3.webp\" alt=\"Preview 3\"></div><div class=\"front\"><div class=\"crease\"></div></div></div><span>Introducing Agenforce Marketing Template</span></div>"
        );
        reject_js(&html);
    }

    #[test]
    fn caps_at_three_and_counts() {
        let html = render(&badge("Four", &["/1.png", "/2.png", "/3.png", "/4.png"]));
        assert!(html.contains("data-photos=\"3\""));
        assert_eq!(html.matches("<div class=\"photo\">").count(), 3);
        let html = render(&badge("None", &[]));
        assert!(html.contains("data-photos=\"0\""));
        assert!(!html.contains("<img"));
    }

    #[test]
    fn href_renders_an_anchor() {
        let mut c = badge("Link", &["/1.png"]);
        c.props.insert("href".into(), "/templates".into());
        let html = render(&c);
        assert!(html.starts_with("<a href=\"/templates\" data-slot=\"images-badge\">"));
        assert!(!html.contains("tabindex"));
        c.props.insert("target".into(), "_blank".into());
        assert!(render(&c).contains("<a href=\"/templates\" target=\"_blank\" rel=\"noopener noreferrer\" data-slot=\"images-badge\">"));
        c.props.insert("href".into(), "javascript:alert(1)".into());
        assert!(render(&c).contains("<a href=\"#\""));
    }

    #[test]
    fn escapes_hostile_input() {
        let mut c = badge("<b>x</b> \"q\"", &["javascript:alert(1)"]);
        c.items[1].text = "<img onerror=x>".into();
        let html = render(&c);
        assert!(html.contains("<span>&lt;b&gt;x&lt;/b&gt; &quot;q&quot;</span>"));
        assert!(html.contains("<img src=\"#\" alt=\"&lt;img onerror=x&gt;\">"));
        reject_js(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/images-badge.css");
        assert!(css.contains("[data-slot=\"images-badge\"] > .folder { position: relative; width: 2rem; height: 1.5rem;"));
        assert!(css.contains("var(--cronus-spring-snappy)"));
        assert!(css.contains("rotateX(-25deg)"));
        assert!(css.contains("rotateX(-45deg) scaleY(0.8)"));
        assert!(css.contains("[data-photos=\"3\"] > .photo:nth-child(2)"));
        assert!(css.contains(":is(:hover, :focus-visible)"));
        assert!(!css.contains("#"));
        assert!(!css.contains("amber-"));
    }

    #[test]
    fn registered_as_dedicated() {
        let c = badge("Folder", &["/1.png"]);
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), render(&c));
        assert_eq!(
            dedicated_fn_name("images-badge"),
            Some("cronus_ui_images_badge::render")
        );
        assert_eq!(
            renderer_kind("images-badge"),
            RendererKind::Dedicated("cronus_ui_images_badge::render")
        );
    }
}
