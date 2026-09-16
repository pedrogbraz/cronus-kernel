//! Dedicated Marquee renderer. DOM matches React:
//! `<div data-slot="marquee" aria-label=…><div data-slot="marquee-group">`
//! with one `<span>` per item. Items are the fixture `text` rows; the
//! `label` row (emitted from `aria-label`) names the region and is never an
//! item. CSS scroll in COMPONENT_CHROME (React's audit fixture renders the
//! static reduced-motion copy; the spec freezes the kernel animation). Zero JS.
//! Not the catalog `fx()` title SURF box.

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let items = entries(comp)
        .into_iter()
        .map(|t| format!("<span>{t}</span>"))
        .collect::<Vec<_>>()
        .join("");
    let label = aria_label(comp);
    format!(
        "<div data-slot=\"marquee\" aria-label=\"{label}\"><div data-slot=\"marquee-group\">{items}</div></div>"
    )
}

/// Non-label rows; falls back to the label so a bare `label` still scrolls.
fn entries(comp: &ComponentNode) -> Vec<String> {
    let out: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type != "label" && i.item_type != "title" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if out.is_empty() {
        vec![label_of(comp)]
    } else {
        out
    }
}

/// `aria-label` from props or any item config (`key:value` after a row
/// attaches to that row), else the label.
fn aria_label(comp: &ComponentNode) -> String {
    if let Some(v) = comp.props.get("aria-label").filter(|v| !v.is_empty()) {
        return esc(v);
    }
    for i in &comp.items {
        if let Some(v) = i.config.get("aria-label").filter(|v| !v.is_empty()) {
            return esc(v);
        }
    }
    label_of(comp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
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
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("fx("));
    }

    /// Shape emitted for the audit fixture: label + texts, aria-label on the last row.
    fn fixture() -> ComponentNode {
        let mut c = stub("marquee", "Logos");
        c.items.push(extra("text", "Acme"));
        let mut last = extra("text", "Globex");
        last.config.insert("aria-label".into(), "Logos".into());
        c.items.push(last);
        c
    }

    #[test]
    fn label_names_region_and_is_not_an_item() {
        let html = render(&fixture());
        assert_eq!(
            html,
            "<div data-slot=\"marquee\" aria-label=\"Logos\"><div data-slot=\"marquee-group\"><span>Acme</span><span>Globex</span></div></div>"
        );
        assert!(!html.contains("<span>Logos</span>"));
        assert_eq!(html.matches("data-slot=\"marquee-group\"").count(), 1);
        reject_fx(&html);
    }

    #[test]
    fn aria_label_from_props_wins() {
        let mut c = fixture();
        c.props.insert("aria-label".into(), "Partners".into());
        assert!(render(&c).contains("aria-label=\"Partners\""));
    }

    #[test]
    fn bare_label_is_single_item() {
        let html = render(&stub("marquee", "A <B> & \"C\""));
        assert_eq!(
            html,
            "<div data-slot=\"marquee\" aria-label=\"A &lt;B&gt; &amp; &quot;C&quot;\"><div data-slot=\"marquee-group\"><span>A &lt;B&gt; &amp; &quot;C&quot;</span></div></div>"
        );
        reject_fx(&html);
    }

    #[test]
    fn skips_fx_surf_title_box() {
        let c = stub("marquee", "Acme");
        let html = render(&c);
        let via = crate::cronus_ui_widgets::render(&c).unwrap();
        assert_eq!(via, html);
        reject_fx(&html);
        assert_eq!(
            crate::cli::stub_renderer_gate::looks_like_stub_fingerprint(&html),
            None
        );
        assert_eq!(
            dedicated_fn_name("marquee"),
            Some("cronus_ui_marquee::render")
        );
        assert_eq!(
            renderer_kind("marquee"),
            RendererKind::Dedicated("cronus_ui_marquee::render")
        );
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("marquee", "Acme"));
            reject_fx(&html);
            assert!(html.contains("data-slot=\"marquee-group\""));
        });
    }

    #[test]
    fn chrome_marquee_via_css() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"marquee\"] {\n  position: relative; display: flex; overflow: hidden;\n  width: var(--cui-marquee-w, 100%);"));
        assert!(css.contains("[data-slot=\"marquee-group\"] > span {\n  padding-inline: 0.75rem;\n  font-size: 0.875rem; line-height: 1.25rem;"));
        assert!(css.contains("@keyframes cui-marquee"));
        assert!(css.contains("animation: cui-marquee"));
        assert!(css.contains("var(--cronus-fg)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains(FX_BOX));
    }
}
