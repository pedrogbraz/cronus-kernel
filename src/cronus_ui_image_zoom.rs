//! Dedicated ImageZoom renderer. DOM matches React idle:
//! `<button type="button" data-slot="image-zoom" data-state="idle">` wrapping
//! `image-zoom-content` (the `src:` / `alt:` image, a URL text, or the label
//! text) and `image-zoom-indicator` (ZoomIn icon). The accessible name is
//! React's `"Zoom image"` (or `label:`, its `labels.zoom`), extended with
//! `: {alt}` when an image `alt` exists. `zoom:N` (2 by default, clamped to
//! 1..=6) is the magnification, carried as class `z-N`; `max-width:md` is the
//! docs' `max-w-md`. `caption:"zoom 4"` and `badge:"…"` add the docs' figure
//! copy under the image (a plain span, or a secondary Badge).
//!
//! Zero JS: the button sits in a `<label>` after a visually hidden checkbox
//! that carries the sticky zoom (tap / Enter toggles it, Esc cannot); hovering
//! the label zooms too, like React's fine-pointer path, but the pan does not
//! follow the cursor (transform-origin tracking needs script). The button
//! keeps React's slot and look but is decorative (`aria-hidden`,
//! `tabindex="-1"`, `pointer-events: none`). Width mirrors the audit
//! fixture's `w-72`. Not catalog `fx()` SURF box.

use crate::cronus_ui_kit::{attr_nonempty, esc, instance_id, label_of, safe_url, texts};
use crate::parser::ComponentNode;

const ZOOM_IN_SVG: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><circle cx=\"11\" cy=\"11\" r=\"8\"></circle><line x1=\"21\" x2=\"16.65\" y1=\"21\" y2=\"16.65\"></line><line x1=\"11\" x2=\"11\" y1=\"8\" y2=\"14\"></line><line x1=\"8\" x2=\"14\" y1=\"11\" y2=\"11\"></line></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let zoom_label = attr_nonempty(comp, "label")
        .map(esc)
        .unwrap_or_else(|| "Zoom image".to_string());
    let (inner, alt) = match attr_nonempty(comp, "src") {
        Some(src) => {
            let alt = attr_nonempty(comp, "alt").map(esc).unwrap_or_default();
            (
                format!("<img src=\"{}\" alt=\"{alt}\">", safe_url(src)),
                alt,
            )
        }
        None => {
            let labels = texts(comp);
            match labels.iter().find(|t| looks_like_src(t)).cloned() {
                Some(src) => {
                    let alt = labels
                        .iter()
                        .find(|t| !looks_like_src(t))
                        .cloned()
                        .unwrap_or_default();
                    (format!("<img src=\"{src}\" alt=\"{alt}\">"), alt)
                }
                None => (label_of(comp), String::new()),
            }
        }
    };
    let aria = if alt.is_empty() {
        zoom_label
    } else {
        format!("{zoom_label}: {alt}")
    };
    let mut classes: Vec<String> = vec!["cui-image-zoom".into()];
    if let Some(z) = attr_nonempty(comp, "zoom")
        .and_then(|z| z.trim().parse::<f64>().ok())
        .map(|z| z.round().clamp(1.0, 6.0) as u8)
        .filter(|z| *z != 2)
    {
        classes.push(format!("z-{z}"));
    }
    if let Some(mw) = crate::cronus_ui_card::max_width_class(comp) {
        classes.push(mw.into());
    }
    let class = classes.join(" ");
    let id = instance_id(comp, "zoom");
    let control = format!(
        "<label class=\"{class}\"><input type=\"checkbox\" id=\"{id}\" aria-label=\"{aria}\"><button type=\"button\" data-slot=\"image-zoom\" data-state=\"idle\" aria-pressed=\"false\" aria-label=\"{aria}\" tabindex=\"-1\" aria-hidden=\"true\"><span data-slot=\"image-zoom-content\">{inner}</span><span aria-hidden=\"true\" data-slot=\"image-zoom-indicator\">{ZOOM_IN_SVG}</span></button></label>"
    );
    let caption = attr_nonempty(comp, "caption")
        .map(|c| format!("<span class=\"cui-image-zoom-caption\">{}</span>", esc(c)));
    let badge = attr_nonempty(comp, "badge")
        .map(|b| crate::cronus_ui_badge::badge_html(&esc(b), "secondary"));
    match caption.or(badge) {
        Some(below) => format!("<div class=\"cui-image-zoom-figure\">{control}{below}</div>"),
        None => control,
    }
}

fn looks_like_src(s: &str) -> bool {
    let s = s.trim();
    s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("data:")
        || s.starts_with('/')
        || s.starts_with("./")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::{reset_instance_ids, stub};
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
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("onpointer"));
        assert!(!html.contains("zinc-"));
    }

    /// wave1t: children text is content, not alt — React names it "Zoom image".
    /// The button is decorative behind the label's checkbox (zero-JS toggle).
    #[test]
    fn root_is_idle_button_with_content_and_icon() {
        reset_instance_ids();
        let html = render(&stub("image-zoom", "Zoom"));
        assert_eq!(
            html,
            format!("<label class=\"cui-image-zoom\"><input type=\"checkbox\" id=\"cui-image-zoom-zoom\" aria-label=\"Zoom image\"><button type=\"button\" data-slot=\"image-zoom\" data-state=\"idle\" aria-pressed=\"false\" aria-label=\"Zoom image\" tabindex=\"-1\" aria-hidden=\"true\"><span data-slot=\"image-zoom-content\">Zoom</span><span aria-hidden=\"true\" data-slot=\"image-zoom-indicator\">{ZOOM_IN_SVG}</span></button></label>")
        );
        assert!(!html.contains("<img"));
        reject_fx(&html);
    }

    #[test]
    fn extra_url_text_becomes_img() {
        let mut c = stub("image-zoom", "Hero");
        c.items.push(extra("text", "https://cdn.example/hero.png"));
        let html = render(&c);
        assert!(html.contains(
            "<span data-slot=\"image-zoom-content\"><img src=\"https://cdn.example/hero.png\" alt=\"Hero\"></span>"
        ));
        assert!(html.contains("aria-label=\"Zoom image: Hero\""));
        reject_fx(&html);
    }

    #[test]
    fn url_label_emits_img() {
        let html = render(&stub("image-zoom", "https://cdn.example/a.png"));
        assert!(html.contains("<img src=\"https://cdn.example/a.png\" alt=\"\">"));
        assert!(html.contains("aria-label=\"Zoom image\""));
        reject_fx(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("image-zoom", "A <B> & \"C\""));
        assert!(html.contains(
            "<span data-slot=\"image-zoom-content\">A &lt;B&gt; &amp; &quot;C&quot;</span>"
        ));
        reject_fx(&html);
    }

    #[test]
    fn url_attr_is_escaped() {
        let html = render(&stub("image-zoom", "https://cdn.example/a.png?x=1&y=\"z\""));
        assert!(html
            .contains("<img src=\"https://cdn.example/a.png?x=1&amp;y=&quot;z&quot;\" alt=\"\">"));
        reject_fx(&html);
    }

    /// Docs examples: `src:` + `alt:` image, `zoom:4` class, `label:` name,
    /// `max-width:md`, and the caption / badge figure copy.
    #[test]
    fn src_zoom_label_and_figure_copy() {
        reset_instance_ids();
        let mut c = stub("image-zoom", "");
        c.items.clear();
        c.props.insert(
            "src".into(),
            "https://picsum.photos/seed/cronus-loft/700/500".into(),
        );
        c.props.insert("zoom".into(), "4".into());
        c.props.insert("label".into(), "Zoom image (4×)".into());
        c.props.insert("caption".into(), "zoom 4".into());
        let html = render(&c);
        assert!(html.starts_with("<div class=\"cui-image-zoom-figure\"><label class=\"cui-image-zoom z-4\"><input type=\"checkbox\" id=\"cui-image-zoom-zoom\" aria-label=\"Zoom image (4×)\"><button"));
        assert!(
            html.contains("<img src=\"https://picsum.photos/seed/cronus-loft/700/500\" alt=\"\">")
        );
        assert!(html.ends_with(
            "</button></label><span class=\"cui-image-zoom-caption\">zoom 4</span></div>"
        ));
        let mut f = stub("image-zoom", "");
        f.items.clear();
        f.props.insert("src".into(), "/fabric.jpg".into());
        f.props.insert("alt".into(), "Linen".into());
        f.props.insert("zoom".into(), "3".into());
        f.props.insert("max-width".into(), "md".into());
        f.props
            .insert("badge".into(), "Hover, tap or press Enter to zoom".into());
        let html = render(&f);
        assert!(html.contains("<label class=\"cui-image-zoom z-3 mw-md\">"));
        assert!(html.contains("aria-label=\"Zoom image: Linen\""));
        assert!(html.ends_with("</label><span data-slot=\"badge\" data-variant=\"secondary\">Hover, tap or press Enter to zoom</span></div>"));
        reject_fx(&html);
        let css = include_str!("cronus_ui_css/image-zoom.css");
        assert!(css.contains(".cui-image-zoom:hover > [data-slot=\"image-zoom\"] [data-slot=\"image-zoom-content\"],\n.cui-image-zoom > input:checked + [data-slot=\"image-zoom\"] [data-slot=\"image-zoom-content\"] { transform: scale(2); }"));
        assert!(css.contains(".cui-image-zoom.z-4:hover > [data-slot=\"image-zoom\"] [data-slot=\"image-zoom-content\"],\n.cui-image-zoom.z-4 > input:checked + [data-slot=\"image-zoom\"] [data-slot=\"image-zoom-content\"] { transform: scale(4); }"));
        assert!(css.contains(".cui-image-zoom > input:checked + [data-slot=\"image-zoom\"] [data-slot=\"image-zoom-indicator\"] { opacity: 0; }"));
        assert!(css.contains(".cui-image-zoom-figure { display: flex; width: 100%; flex-direction: column; align-items: center; gap: 0.5rem; }"));
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("image-zoom", "Photo");
        reset_instance_ids();
        let html = render(&c);
        reset_instance_ids();
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), html);
        reject_fx(&html);
        assert_eq!(
            dedicated_fn_name("image-zoom"),
            Some("cronus_ui_image_zoom::render")
        );
        assert_eq!(
            renderer_kind("image-zoom"),
            RendererKind::Dedicated("cronus_ui_image_zoom::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("image-zoom", "Photo"));
            reject_fx(&html);
            assert!(html.contains("data-state=\"idle\""));
        });
    }

    #[test]
    fn chrome_mirrors_w72_and_translucent_indicator() {
        let css = include_str!("cronus_ui_css/image-zoom.css");
        assert!(css.contains(
            "[data-slot=\"image-zoom\"] {\n  position: relative; display: block; width: var(--cui-image-zoom-w, 100%); box-sizing: border-box; overflow: hidden;"
        ));
        assert!(css.contains(
            "background: color-mix(in oklab, var(--cronus-surface-overlay) 90%, transparent);"
        ));
        assert!(css.contains(
            ".cui-image-zoom > [data-slot=\"image-zoom\"] [data-slot=\"image-zoom-content\"] { transform-origin: var(--zoom-ox, 50%) var(--zoom-oy, 50%); }"
        ));
        assert!(
            css.contains("[data-slot=\"image-zoom-indicator\"] svg { width: 1rem; height: 1rem; }")
        );
        assert!(css.contains("transform: scale("));
        assert!(css.contains("prefers-reduced-motion: reduce"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
