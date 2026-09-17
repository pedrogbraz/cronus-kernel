//! Dedicated Frame renderer. DOM matches React:
//! `<div data-slot="frame" data-variant="browser|window">` > `frame-chrome`
//! (traffic-light dots + `frame-address-bar` with the `url` value for the
//! browser variant, empty when no url, exactly like React's `{url}`) >
//! `frame-content`. The content is the label, or — with `title` / `text`
//! items — the docs mockup (`space-y-2 p-6`: display heading + copy).
//! Width mirrors the audit fixture's `w-72`. Not the catalog `display()` SURF.

use crate::cronus_ui_kit::{attr_nonempty, choice, esc, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let variant = choice(comp, "variant", &["browser", "window"]).unwrap_or("browser");
    let address = if variant == "browser" {
        format!(
            "<div data-slot=\"frame-address-bar\">{}</div>",
            url_of(comp).unwrap_or_default()
        )
    } else {
        String::new()
    };
    let lines: String = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "title" | "text") && !i.text.is_empty())
        .map(|i| {
            if i.item_type == "title" {
                format!("<h3>{}</h3>", esc(&i.text))
            } else {
                format!("<p>{}</p>", esc(&i.text))
            }
        })
        .collect();
    let content = if lines.is_empty() {
        label_of(comp)
    } else {
        format!("<div>{lines}</div>")
    };
    format!(
        "<div data-slot=\"frame\" data-variant=\"{variant}\"><div data-slot=\"frame-chrome\"><div aria-hidden=\"true\"><span></span><span></span><span></span></div>{address}</div><div data-slot=\"frame-content\">{content}</div></div>"
    )
}

/// `url:"…"` as a prop, or attached to an item's config (the tokenizer has no
/// newlines, so a trailing `key:value` lands on the preceding item).
fn url_of(comp: &ComponentNode) -> Option<String> {
    attr_nonempty(comp, "url").map(|s| esc(s))
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

    /// wave1t: React browser variant always renders the address bar.
    #[test]
    fn root_is_browser_frame_with_address_bar() {
        let html = render(&stub("frame", "Checkout"));
        assert_eq!(
            html,
            "<div data-slot=\"frame\" data-variant=\"browser\"><div data-slot=\"frame-chrome\"><div aria-hidden=\"true\"><span></span><span></span><span></span></div><div data-slot=\"frame-address-bar\"></div></div><div data-slot=\"frame-content\">Checkout</div></div>"
        );
        reject_display(&html);
    }

    #[test]
    fn url_config_fills_address_bar_escaped() {
        let mut c = stub("frame", "Checkout");
        c.items[0]
            .config
            .insert("url".into(), "cronus.dev/?a=1&b".into());
        let html = render(&c);
        assert!(html.contains("<div data-slot=\"frame-address-bar\">cronus.dev/?a=1&amp;b</div>"));
        let mut p = stub("frame", "Checkout");
        p.props.insert("url".into(), "cronus.dev".into());
        assert!(render(&p).contains("<div data-slot=\"frame-address-bar\">cronus.dev</div>"));
        reject_display(&html);
    }

    /// Docs "Browser chrome" / "Window chrome": `title` + `text` items are the
    /// `space-y-2 p-6` mockup; the window variant drops the address bar.
    #[test]
    fn title_and_text_items_are_the_mockup() {
        let mut c = stub("frame", "Preview");
        c.props.insert("url".into(), "cronus.app/dashboard".into());
        c.items.push(extra("title", "Faturamento"));
        c.items
            .push(extra("text", "R$ 128.940 nos últimos 30 dias."));
        c.items.push(extra("text", "+18% vs. o período anterior."));
        let html = render(&c);
        assert!(html.ends_with("<div data-slot=\"frame-address-bar\">cronus.app/dashboard</div></div><div data-slot=\"frame-content\"><div><h3>Faturamento</h3><p>R$ 128.940 nos últimos 30 dias.</p><p>+18% vs. o período anterior.</p></div></div></div>"));
        assert!(!html.contains("Preview"));
        reject_display(&html);
        c.props.remove("url");
        c.props.insert("variant".into(), "window".into());
        let html = render(&c);
        assert!(html.starts_with("<div data-slot=\"frame\" data-variant=\"window\"><div data-slot=\"frame-chrome\"><div aria-hidden=\"true\"><span></span><span></span><span></span></div></div><div data-slot=\"frame-content\"><div><h3>"));
        assert!(!html.contains("frame-address-bar"));
        c.style = Some("frame+window".into());
        c.props.remove("variant");
        assert!(render(&c).contains("data-variant=\"window\""));
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("frame", "A <B> & \"C\""));
        assert!(html.contains("data-slot=\"frame-content\">A &lt;B&gt; &amp; &quot;C&quot;</div>"));
        reject_display(&html);
    }

    #[test]
    fn skips_display_surf_section() {
        let html = render(&stub("frame", "Preview"));
        reject_display(&html);
        assert_eq!(dedicated_fn_name("frame"), Some("cronus_ui_frame::render"));
        assert_eq!(
            renderer_kind("frame"),
            RendererKind::Dedicated("cronus_ui_frame::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("frame", "Preview"));
            reject_display(&html);
        });
    }

    #[test]
    fn chrome_frame_w72_and_address_bar() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"frame\"] {\n  width: var(--cui-frame-w, 100%); box-sizing: border-box; overflow: hidden;"));
        assert!(css.contains(
            "[data-slot=\"frame-address-bar\"] {\n  margin: 0 auto; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;"
        ));
        assert!(css
            .contains("font-size: 0.75rem; line-height: 1rem; color: var(--cronus-fg-tertiary);"));
        assert!(css.contains("var(--cronus-error)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(DISPLAY_SURF));
        let file = include_str!("cronus_ui_css/frame.css");
        assert!(file.contains("[data-slot=\"frame-content\"] > div { padding: 1.5rem; }"));
        assert!(file.contains(
            "[data-slot=\"frame-content\"] > div > * + * { margin-block-start: 0.5rem; }"
        ));
        assert!(file.contains("[data-slot=\"frame-content\"] > div > h3 {"));
        assert!(file.contains("font-size: 1.125rem; line-height: 1.75rem; font-weight: 600;"));
    }
}
