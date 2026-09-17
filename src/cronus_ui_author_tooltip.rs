//! Dedicated AuthorTooltip renderer. React composes `Tooltip` (a Radix root,
//! which renders no element) with an `Avatar` trigger and a `TooltipContent`
//! card. The kernel keeps the same visible tree inside one anchoring
//! `<span data-slot="author-tooltip">`: the trigger
//! `<span data-slot="avatar" class="s-sm|md|lg|xl" data-state="closed">`
//! (`authorTooltipAvatarVariants`: 2 / 2.5 / 3 / 4rem, `cursor-help`, 2px
//! border) with `avatar-image` + `avatar-fallback` initials, then
//! `<div data-slot="tooltip-content" role="tooltip" data-side="top">` holding
//! the 2.5rem avatar, name + role, the social links (`linkedin`, `twitter`,
//! `github`; X keeps its glyph, the others use lucide `external-link` like
//! React's neutral defaults) and an optional bio line.
//!
//! Zero JS: the card opens on `:hover` / `:focus-within` of the wrapper (the
//! trigger is `tabindex="0"` so keyboard users reach it) with React's
//! `cronus-pop-in` 150ms entrance. Radix's `delayDuration={0}` means no
//! hover delay, which CSS matches.
//!
//! Inputs: `label "Aryan"` (name), `role:"Founder & CEO"`, `avatar:"https://…"`,
//! `github:` / `twitter:` / `linkedin:` URLs, `size:sm|md|lg|xl` (prop or style
//! segment `author-tooltip+lg`) and an optional `text "…"` bio.

use crate::cronus_ui_kit::{attr_nonempty, choice, esc, item, label_of, safe_url};
use crate::parser::ComponentNode;

const X_ICON: &str = "<svg viewBox=\"0 0 24 24\" aria-hidden=\"true\" fill=\"currentColor\" data-icon=\"x-brand\"><title>X</title><path d=\"M18.244 2H21.5l-7.5 8.57L22.5 22h-6.56l-5.14-6.71L5.5 22H2.24l8.02-9.16L1.5 2h6.72l4.64 6.15L18.244 2Zm-1.15 18.13h1.82L7.01 3.78H5.06l12.03 16.35Z\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let name = label_of(comp);
    let fallback = esc(&initials(&name));
    let size = choice(comp, "size", &["sm", "md", "lg", "xl"])
        .or_else(|| choice(comp, "avatar-size", &["sm", "md", "lg", "xl"]))
        .unwrap_or("sm");
    let image = attr_nonempty(comp, "avatar")
        .or_else(|| attr_nonempty(comp, "src"))
        .or_else(|| attr_nonempty(comp, "image"))
        .map(safe_url);
    let role = attr_nonempty(comp, "role").map(esc).unwrap_or_default();
    let mut links = String::new();
    for (key, label, glyph) in [
        ("linkedin", "LinkedIn", None),
        ("twitter", "X", Some(X_ICON)),
        ("github", "GitHub", None),
    ] {
        if let Some(href) = attr_nonempty(comp, key) {
            let icon = glyph
                .map(str::to_string)
                .unwrap_or_else(|| crate::cronus_ui_icons::svg_or_empty("external-link"));
            links.push_str(&format!(
                "<a href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\" aria-label=\"{label}\">{icon}</a>",
                safe_url(href)
            ));
        }
    }
    let links = if links.is_empty() {
        String::new()
    } else {
        format!("<div>{links}</div>")
    };
    let bio = item(comp, "text")
        .filter(|t| !t.is_empty())
        .map(|t| format!("<div>{}</div>", esc(t)))
        .unwrap_or_default();
    let content_id = crate::cronus_ui_kit::instance_id(comp, "author-tooltip");
    let trigger = format!(
        "<span data-slot=\"avatar\" class=\"s-{size}\" aria-label=\"{name}\" tabindex=\"0\" data-state=\"closed\" aria-describedby=\"{content_id}\">{}<span data-slot=\"avatar-fallback\">{fallback}</span></span>",
        image
            .as_deref()
            .map(|src| format!("<img data-slot=\"avatar-image\" src=\"{src}\" alt=\"{name}\">"))
            .unwrap_or_default()
    );
    let card_avatar = format!(
        "<span data-slot=\"avatar\">{}<span data-slot=\"avatar-fallback\">{fallback}</span></span>",
        image
            .as_deref()
            .map(|src| format!("<img data-slot=\"avatar-image\" src=\"{src}\" alt=\"\">"))
            .unwrap_or_default()
    );
    format!(
        "<span data-slot=\"author-tooltip\">{trigger}<div id=\"{content_id}\" data-slot=\"tooltip-content\" role=\"tooltip\" data-side=\"top\"><div><div>{card_avatar}<div><span>{name}</span><span>{role}</span></div>{links}</div>{bio}</div></div></span>"
    )
}

/// First letter of the first two words, upper-cased (React `initials`).
fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .flat_map(|c| c.to_uppercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::{reset_instance_ids, stub};

    fn author() -> ComponentNode {
        reset_instance_ids();
        let mut c = stub("author-tooltip", "Aryan");
        c.props.insert("role".into(), "Founder & CEO".into());
        c.props.insert(
            "avatar".into(),
            "https://github.com/aryanranderiya.png".into(),
        );
        c
    }

    fn reject_js(html: &str) {
        for bad in [
            "<script", "style=", "onclick", "onmouse", "<canvas", "popover",
        ] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    #[test]
    fn docs_profile_example_dom() {
        let mut c = author();
        c.props
            .insert("github".into(), "https://github.com/aryanranderiya".into());
        c.props.insert(
            "twitter".into(),
            "https://twitter.com/aryanranderiya".into(),
        );
        c.props.insert(
            "linkedin".into(),
            "https://linkedin.com/in/aryanranderiya".into(),
        );
        let html = render(&c);
        assert!(html.starts_with("<span data-slot=\"author-tooltip\"><span data-slot=\"avatar\" class=\"s-sm\" aria-label=\"Aryan\" tabindex=\"0\" data-state=\"closed\" aria-describedby=\"cui-author-tooltip-author-tooltip\"><img data-slot=\"avatar-image\" src=\"https://github.com/aryanranderiya.png\" alt=\"Aryan\"><span data-slot=\"avatar-fallback\">A</span></span><div id=\"cui-author-tooltip-author-tooltip\" data-slot=\"tooltip-content\" role=\"tooltip\" data-side=\"top\"><div><div><span data-slot=\"avatar\"><img data-slot=\"avatar-image\" src=\"https://github.com/aryanranderiya.png\" alt=\"\"><span data-slot=\"avatar-fallback\">A</span></span><div><span>Aryan</span><span>Founder &amp; CEO</span></div><div><a href=\"https://linkedin.com/in/aryanranderiya\" target=\"_blank\" rel=\"noopener noreferrer\" aria-label=\"LinkedIn\"><svg"));
        assert!(html.contains("data-icon=\"external-link\""));
        assert!(html.contains("aria-label=\"X\"><svg viewBox=\"0 0 24 24\" aria-hidden=\"true\" fill=\"currentColor\" data-icon=\"x-brand\"><title>X</title>"));
        assert!(html.contains("aria-label=\"GitHub\"><svg"));
        assert!(html.ends_with("</a></div></div></div></div></span>"));
        reject_js(&html);
    }

    #[test]
    fn sizes_and_style_segment() {
        for size in ["sm", "md", "lg", "xl"] {
            let mut c = author();
            c.props.insert("size".into(), size.into());
            assert!(render(&c).contains(&format!("class=\"s-{size}\"")));
            let mut c = author();
            c.style = Some(format!("author-tooltip+{size}"));
            assert!(render(&c).contains(&format!("class=\"s-{size}\"")));
        }
        assert!(render(&author()).contains("class=\"s-sm\""));
    }

    #[test]
    fn no_links_no_link_row_and_bio_line() {
        let mut c = author();
        c.items.push(crate::parser::ComponentItemNode {
            item_type: "text".into(),
            text: "Builds Cronus.".into(),
            link: None,
            tone: None,
            config: Default::default(),
        });
        let html = render(&c);
        assert!(html.contains("<span>Founder &amp; CEO</span></div></div><div>Builds Cronus.</div></div></div></span>"));
        assert!(!html.contains("<a "));
    }

    #[test]
    fn initials_and_fallback_without_image() {
        assert_eq!(initials("ada lovelace byron"), "AL");
        assert_eq!(initials("Aryan"), "A");
        reset_instance_ids();
        let html = render(&stub("author-tooltip", "Ada Lovelace"));
        assert!(!html.contains("<img"));
        assert!(html.contains("<span data-slot=\"avatar-fallback\">AL</span>"));
    }

    #[test]
    fn escapes_hostile_input_and_urls() {
        let mut c = author();
        c.items[0].text = "<b>x</b> \"q\"".into();
        c.props
            .insert("avatar".into(), "javascript:alert(1)".into());
        c.props
            .insert("github".into(), "javascript:alert(1)".into());
        c.props.insert("role".into(), "<i>r</i>".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"&lt;b&gt;x&lt;/b&gt; &quot;q&quot;\""));
        assert!(html.contains("src=\"#\""));
        assert!(html.contains("href=\"#\""));
        assert!(html.contains("<span>&lt;i&gt;r&lt;/i&gt;</span>"));
        reject_js(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = include_str!("cronus_ui_css/author-tooltip.css");
        assert!(css.contains("[data-slot=\"author-tooltip\"] > [data-slot=\"avatar\"].s-xl { width: 4rem; height: 4rem; }"));
        assert!(css.contains("cursor: help"));
        assert!(css.contains("[data-slot=\"author-tooltip\"]:is(:hover, :focus-within) > [data-slot=\"tooltip-content\"]"));
        assert!(css.contains("cronus-pop-in 150ms var(--ease-out-quart) both"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(!css.contains("#"));
    }

    #[test]
    fn registered_as_dedicated() {
        let c = author();
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), {
            reset_instance_ids();
            render(&c)
        });
        assert_eq!(
            dedicated_fn_name("author-tooltip"),
            Some("cronus_ui_author_tooltip::render")
        );
        assert_eq!(
            renderer_kind("author-tooltip"),
            RendererKind::Dedicated("cronus_ui_author_tooltip::render")
        );
    }
}
