//! Dedicated GeneratedImage renderer (docs slug `ai-image`). DOM matches React:
//! a single `<img data-slot="generated-image" alt src>`. `base64:"…"` with
//! `media-type:image/png` (the default) builds React's `data:` URL; only
//! base64 characters and a raster image media type (png, jpeg, gif, webp,
//! avif, bmp) are accepted, anything else drops the `src`. `src:"…"` is an
//! ordinary URL ([`safe_url`]) for an already hosted image. `alt:"…"` (or the
//! `label` item) is the accessible name, "Generated image" by default like
//! React; `width:` / `height:` are the `<img>` size attributes (the docs'
//! `size-24` is `width:96 height:96`).

use crate::cronus_ui_kit::{attr_nonempty, esc, item, safe_url};
use crate::parser::ComponentNode;

const DEFAULT_ALT: &str = "Generated image";
const MEDIA_TYPES: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/jpg",
    "image/gif",
    "image/webp",
    "image/avif",
    "image/bmp",
];

/// `data:<media>;base64,<payload>` when both parts are well formed.
fn data_url(media: &str, payload: &str) -> Option<String> {
    let media = media.trim().to_ascii_lowercase();
    if !MEDIA_TYPES.contains(&media.as_str()) {
        return None;
    }
    let payload: String = payload.split_whitespace().collect();
    if payload.is_empty()
        || !payload
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'/' || b == b'=')
    {
        return None;
    }
    Some(format!("data:{media};base64,{payload}"))
}

pub fn render(comp: &ComponentNode) -> String {
    let alt = attr_nonempty(comp, "alt")
        .or_else(|| item(comp, "label").filter(|l| !l.is_empty()))
        .map(esc)
        .unwrap_or_else(|| DEFAULT_ALT.into());
    let src = attr_nonempty(comp, "base64")
        .and_then(|b| {
            data_url(
                attr_nonempty(comp, "media-type")
                    .or_else(|| attr_nonempty(comp, "mediaType"))
                    .unwrap_or("image/png"),
                b,
            )
        })
        .or_else(|| attr_nonempty(comp, "src").map(safe_url))
        .map(|s| format!(" src=\"{s}\""))
        .unwrap_or_default();
    let size = ["width", "height"]
        .iter()
        .filter_map(|k| {
            crate::cronus_ui_kit::attr_num::<u32>(comp, k).map(|v| format!(" {k}=\"{v}\""))
        })
        .collect::<String>();
    format!("<img data-slot=\"generated-image\" alt=\"{alt}\"{src}{size}>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    const PIXEL: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";

    #[test]
    fn docs_example_is_a_data_url_image() {
        let mut c = stub("ai-image", "A 1×1 placeholder the model returned");
        c.props.insert("base64".into(), PIXEL.into());
        c.props.insert("media-type".into(), "image/png".into());
        c.props.insert("width".into(), "96".into());
        c.props.insert("height".into(), "96".into());
        assert_eq!(
            render(&c),
            format!("<img data-slot=\"generated-image\" alt=\"A 1×1 placeholder the model returned\" src=\"data:image/png;base64,{PIXEL}\" width=\"96\" height=\"96\">")
        );
    }

    #[test]
    fn rejects_non_base64_payloads_and_odd_media_types() {
        let mut c = stub("ai-image", "x");
        c.props
            .insert("base64".into(), "<script>alert(1)</script>".into());
        assert!(!render(&c).contains("src="));
        c.props.insert("base64".into(), PIXEL.into());
        c.props.insert("media-type".into(), "text/html".into());
        assert!(!render(&c).contains("src="));
        c.props.insert("media-type".into(), "image/svg+xml".into());
        assert!(!render(&c).contains("src="));
    }

    #[test]
    fn src_url_alt_default_and_escaping() {
        let mut c = stub("ai-image", "");
        c.props.insert("src".into(), "javascript:alert(1)".into());
        assert_eq!(
            render(&c),
            "<img data-slot=\"generated-image\" alt=\"Generated image\" src=\"#\">"
        );
        c.props
            .insert("src".into(), "https://example.com/a.png".into());
        c.props.insert("alt".into(), "<b>\"x\"</b>".into());
        let html = render(&c);
        assert!(html.contains(
            "alt=\"&lt;b&gt;&quot;x&quot;&lt;/b&gt;\" src=\"https://example.com/a.png\">"
        ));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/ai-image.css");
        assert!(css.contains("[data-slot=\"generated-image\"]"));
        assert!(css.contains("var(--cronus-radius-md)"));
        assert!(!css.contains("zinc-"));
    }

    #[test]
    fn registered_as_dedicated() {
        use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
        assert_eq!(
            dedicated_fn_name("ai-image"),
            Some("cronus_ui_ai_image::render")
        );
        assert_eq!(
            renderer_kind("ai-image"),
            RendererKind::Dedicated("cronus_ui_ai_image::render")
        );
        let c = stub("ai-image", "Generated image");
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), render(&c));
    }
}
