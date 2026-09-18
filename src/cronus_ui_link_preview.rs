//! Dedicated LinkPreview renderer. React wraps an `<a>` trigger
//! (`linkPreviewVariants`: primary/20 tint, rounded-sm, `text-primary-strong`)
//! and a 300px `TooltipContent` that unfurls metadata fetched from an
//! endpoint (microlink.io) on hover. The kernel cannot fetch, so the unfurl
//! comes from the `.cronus`: `site:"GitHub"`, `title "…"`, `text "…"`
//! (description), `image:"…"` (16:9 thumbnail) and `favicon:"…"` (falls back
//! to lucide `globe`); the display URL is `href` without its scheme. With no
//! metadata the card shows `No preview available`; a non-http(s) `href` shows
//! `Invalid URL` in the error tone, both like React.
//!
//! DOM, inside one anchoring `<span data-slot="link-preview">`:
//! `<a href target="_blank" rel="noopener noreferrer" data-state="closed" interestfor>`
//! + `<span popover="hint" data-slot="tooltip-content" role="tooltip">` > unfurl column
//! (spans styled as blocks: a `<div>` would close the docs' `<p>`).
//! `prefix:"Check out the"` / `suffix:"to browse the catalog."` wrap the
//! whole thing in the docs' `<p>` paragraph (`max-w-prose text-fg-secondary`).
//! Zero JS: the anchor's `interestfor` opens a `popover="hint"` unfurl card
//! (live.js `showPopover` after 200ms). CSS `:popover-open` runs `cronus-pop-in`.
//!
//! Inputs: `label "Cronus UI repository"` (link text), `href:"https://…"`,
//! `site:`, `title`, `text`, `image:`, `favicon:`, `prefix:`, `suffix:`.

use crate::cronus_ui_kit::{attr_nonempty, esc, item, label_of, safe_url};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let raw_href = attr_nonempty(comp, "href")
        .or_else(|| attr_nonempty(comp, "link"))
        .or_else(|| attr_nonempty(comp, "url"))
        .or_else(|| comp.items.iter().find_map(|i| i.link.as_deref()))
        .unwrap_or("");
    let href = safe_url(raw_href);
    let valid = is_http_url(raw_href);
    let content_id = crate::cronus_ui_kit::instance_id(comp, "link-preview");
    let globe = crate::cronus_ui_icons::svg_or_empty("globe");
    let body = if !valid {
        format!("<span data-error=\"true\">{globe}<span>Invalid URL</span></span>")
    } else {
        let site = attr_nonempty(comp, "site")
            .or_else(|| attr_nonempty(comp, "website"))
            .map(esc);
        let title = item(comp, "title").filter(|t| !t.is_empty()).map(esc);
        let description = item(comp, "text").filter(|t| !t.is_empty()).map(esc);
        let image = attr_nonempty(comp, "image").map(safe_url);
        let favicon = attr_nonempty(comp, "favicon").map(safe_url);
        if site.is_none()
            && title.is_none()
            && description.is_none()
            && image.is_none()
            && favicon.is_none()
        {
            format!("<span data-empty=\"true\">{globe}<span>No preview available</span></span>")
        } else {
            let mut out = String::from("<span>");
            if let Some(src) = &image {
                out.push_str(&format!(
                    "<span><img src=\"{src}\" alt=\"Website preview\"></span>"
                ));
            }
            if site.is_some() || favicon.is_some() {
                out.push_str("<span>");
                match &favicon {
                    Some(src) => out.push_str(&format!(
                        "<img width=\"20\" height=\"20\" alt=\"Favicon\" src=\"{src}\">"
                    )),
                    None => out.push_str(&globe),
                }
                if let Some(s) = &site {
                    out.push_str(&format!("<span>{s}</span>"));
                }
                out.push_str("</span>");
            }
            if let Some(t) = &title {
                out.push_str(&format!("<span class=\"lp-title\">{t}</span>"));
            }
            if let Some(d) = &description {
                out.push_str(&format!("<span class=\"lp-description\">{d}</span>"));
            }
            out.push_str(&format!(
                "<span class=\"lp-url\">{}</span></span>",
                esc(display_url(raw_href))
            ));
            out
        }
    };
    let trigger_id = format!("{content_id}-trigger");
    let inner = format!(
        "<span data-slot=\"link-preview\"><a id=\"{trigger_id}\" href=\"{href}\" target=\"_blank\" rel=\"noopener noreferrer\" data-state=\"closed\" interestfor=\"{content_id}\" aria-describedby=\"{content_id}\">{label}</a><span id=\"{content_id}\" popover=\"hint\" anchor=\"{trigger_id}\" data-slot=\"tooltip-content\" role=\"tooltip\">{body}</span></span>"
    );
    let prefix = attr_nonempty(comp, "prefix").map(esc);
    let suffix = attr_nonempty(comp, "suffix").map(esc);
    if prefix.is_none() && suffix.is_none() {
        return inner;
    }
    format!(
        "<p class=\"link-preview-prose\">{}{inner}{}</p>",
        prefix.map(|p| format!("{p} ")).unwrap_or_default(),
        suffix.map(|s| format!(" {s}")).unwrap_or_default()
    )
}

fn is_http_url(raw: &str) -> bool {
    let t = raw.trim().to_ascii_lowercase();
    (t.starts_with("http://") || t.starts_with("https://"))
        && t.len() > 8
        && !t.contains('@')
        && !t.chars().any(char::is_control)
}

fn display_url(raw: &str) -> &str {
    let t = raw.trim();
    t.strip_prefix("https://")
        .or_else(|| t.strip_prefix("http://"))
        .unwrap_or(t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::{reset_instance_ids, stub};
    use crate::parser::ComponentItemNode;

    fn link(label: &str) -> ComponentNode {
        reset_instance_ids();
        let mut c = stub("link-preview", label);
        c.props.insert(
            "href".into(),
            "https://github.com/pedrogbraz/cronus-ui".into(),
        );
        c
    }

    fn push(c: &mut ComponentNode, kind: &str, text: &str) {
        c.items.push(ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        });
    }

    fn reject_js(html: &str) {
        for bad in ["<script", "style=", "onclick", "onmouse", "<canvas"] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    #[test]
    fn docs_unfurl_example_dom() {
        let mut c = link("Cronus UI repository");
        c.props.insert("prefix".into(), "Check out the".into());
        c.props
            .insert("suffix".into(), "to browse the catalog.".into());
        c.props.insert("site".into(), "GitHub".into());
        c.props
            .insert("favicon".into(), "https://github.com/favicon.ico".into());
        c.props.insert(
            "image".into(),
            "https://opengraph.githubassets.com/1/pedrogbraz/cronus-ui".into(),
        );
        push(&mut c, "title", "pedrogbraz/cronus-ui");
        push(&mut c, "text", "Design system for Cronus.");
        let html = render(&c);
        assert!(html.starts_with("<p class=\"link-preview-prose\">Check out the <span data-slot=\"link-preview\"><a id=\"cui-link-preview-link-preview-trigger\" href=\"https://github.com/pedrogbraz/cronus-ui\" target=\"_blank\" rel=\"noopener noreferrer\" data-state=\"closed\" interestfor=\"cui-link-preview-link-preview\" aria-describedby=\"cui-link-preview-link-preview\">Cronus UI repository</a><span id=\"cui-link-preview-link-preview\" popover=\"hint\" anchor=\"cui-link-preview-link-preview-trigger\" data-slot=\"tooltip-content\" role=\"tooltip\"><span><span><img src=\"https://opengraph.githubassets.com/1/pedrogbraz/cronus-ui\" alt=\"Website preview\"></span><span><img width=\"20\" height=\"20\" alt=\"Favicon\" src=\"https://github.com/favicon.ico\"><span>GitHub</span></span><span class=\"lp-title\">pedrogbraz/cronus-ui</span><span class=\"lp-description\">Design system for Cronus.</span><span class=\"lp-url\">github.com/pedrogbraz/cronus-ui</span></span></span></span> to browse the catalog.</p>"));
        reject_js(&html);
    }

    #[test]
    fn no_metadata_and_favicon_fallback() {
        let html = render(&link("Repo"));
        assert!(html.starts_with("<span data-slot=\"link-preview\">"));
        assert!(html.contains("<span data-empty=\"true\"><svg"));
        assert!(html.contains("data-icon=\"globe\""));
        assert!(html.contains("</svg><span>No preview available</span></span>"));
        let mut c = link("Repo");
        c.props.insert("site".into(), "GitHub".into());
        let html = render(&c);
        assert!(html.contains("<span><span><svg"));
        assert!(html.contains("</svg><span>GitHub</span></span><span class=\"lp-url\">"));
    }

    #[test]
    fn invalid_or_hostile_urls() {
        let mut c = link("Bad");
        c.props.insert("href".into(), "javascript:alert(1)".into());
        let html = render(&c);
        assert!(html.contains("href=\"#\""));
        assert!(html.contains("<span data-error=\"true\"><svg"));
        assert!(html.contains("<span>Invalid URL</span>"));
        let mut c = link("Mail");
        c.props.insert("href".into(), "mailto:x@y.z".into());
        assert!(render(&c).contains("Invalid URL"));
        let mut c = link("<b>x</b> \"q\"");
        c.props.insert("image".into(), "javascript:alert(1)".into());
        push(&mut c, "title", "<i>t</i>");
        let html = render(&c);
        assert!(html.contains(">&lt;b&gt;x&lt;/b&gt; &quot;q&quot;</a>"));
        assert!(html.contains("<img src=\"#\""));
        assert!(html.contains("<span class=\"lp-title\">&lt;i&gt;t&lt;/i&gt;</span>"));
        reject_js(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/link-preview.css");
        assert!(css.contains("[data-slot=\"link-preview\"] > a {"));
        assert!(css.contains("color-mix(in oklab, var(--cronus-primary) 20%, transparent)"));
        assert!(css.contains("[data-slot=\"link-preview\"]:is(:hover, :focus-within) > [data-slot=\"tooltip-content\"]"));
        assert!(css.contains(
            "[data-slot=\"link-preview\"] > [data-slot=\"tooltip-content\"]:popover-open"
        ));
        assert!(css.contains("min-width: 18.75rem; max-width: 18.75rem;"));
        assert!(css.contains("aspect-ratio: 16 / 9"));
        assert!(css.contains("-webkit-line-clamp: 3"));
        assert!(!css.contains("#"));
    }

    #[test]
    fn registered_as_dedicated() {
        let c = link("Repo");
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), {
            reset_instance_ids();
            render(&c)
        });
        assert_eq!(
            dedicated_fn_name("link-preview"),
            Some("cronus_ui_link_preview::render")
        );
        assert_eq!(
            renderer_kind("link-preview"),
            RendererKind::Dedicated("cronus_ui_link_preview::render")
        );
    }
}
