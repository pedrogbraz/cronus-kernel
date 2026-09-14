//! Dedicated LogoCarousel renderer. DOM matches React idle (page 0):
//! `<ul data-slot="logo-carousel">` plus up to 3 `logo-carousel-item`
//! `<li aria-label>` each holding `div > span[aria-hidden] > span` with the
//! label initials (React `LogoMark` without icon). No setInterval / paging.
//! CSS in COMPONENT_CHROME. Not interact flex-overflow
//! `<div data-slot="logo-carousel" style=...>` without items.

use crate::cronus_ui_kit::{attr, esc, item, label_of};
use crate::parser::ComponentNode;

/// Logos come from `text` / `item` lines. The `label` names the list
/// (aria-label) and is only a logo when no other line exists.
/// Returns `(raw, escaped)` so initials are computed on unescaped text.
fn logos(comp: &ComponentNode) -> Vec<(String, String)> {
    let out: Vec<(String, String)> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "text" | "item") && !i.text.is_empty())
        .map(|i| (i.text.clone(), esc(&i.text)))
        .collect();
    if !out.is_empty() {
        return out;
    }
    // Same lookup order as `label_of`, but unescaped for the initials.
    let raw = ["label", "title", "text", "value"]
        .iter()
        .find_map(|k| item(comp, *k).filter(|t| !t.is_empty()))
        .map(str::to_string)
        .or_else(|| {
            comp.items
                .iter()
                .find(|i| !i.text.is_empty())
                .map(|i| i.text.clone())
        })
        .unwrap_or_else(|| comp.name.clone());
    vec![(raw, label_of(comp))]
}

/// React's default `columns` (3) capped by `MAX_COLUMNS` (8). The idle page
/// shows exactly one logo per column; later logos only appear when React's
/// timer pages, which a zero-JS render cannot do.
const COLUMNS: usize = 3;

/// React `getInitials`: first letter of up to two whitespace words, uppercased.
/// Works on the raw text; the result is escaped by the caller.
fn initials(label: &str) -> String {
    label
        .split_whitespace()
        .take(2)
        .filter_map(|w| w.chars().next())
        .flat_map(char::to_uppercase)
        .collect()
}

pub fn render(comp: &ComponentNode) -> String {
    let items = logos(comp)
        .into_iter()
        .take(COLUMNS)
        .map(|(raw, t)| {
            format!(
                "<li data-slot=\"logo-carousel-item\" aria-label=\"{t}\"><div><span aria-hidden=\"true\"><span>{}</span></span></div></li>",
                esc(&initials(&raw))
            )
        })
        .collect::<Vec<_>>()
        .join("");
    // `aria-label:` after an item line lands in that item's config (the
    // tokenizer has no newlines), so look there as well as in props.
    let aria = match attr(comp, "aria-label")
        .or_else(|| item(comp, "label"))
        .filter(|t| !t.is_empty())
    {
        Some(label) => esc(label),
        None => "Logo carousel".into(),
    };
    format!("<ul data-slot=\"logo-carousel\" aria-label=\"{aria}\" aria-live=\"off\">{items}</ul>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    const INTERACT_ROW: &str = "display:flex;gap:0.75rem;overflow:auto";
    const DISPLAY_SURF: &str = "padding:1rem;display:flex;flex-direction:column;gap:0.5rem";
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

    fn logos(items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("logo-carousel", items.first().copied().unwrap_or("Acme"));
        c.items.clear();
        for n in items {
            c.items.push(extra("item", n));
        }
        c
    }

    fn reject_stub(html: &str) {
        assert!(!html.contains("<section"));
        assert!(!html.contains("<div data-slot=\"logo-carousel\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("setInterval"));
        assert!(!html.contains("onscroll"));
        assert!(!html.contains("addEventListener"));
        assert!(!html.contains(INTERACT_ROW));
        assert!(!html.contains("min-width:12rem"));
        assert!(!html.contains(DISPLAY_SURF));
        assert!(!html.contains("display("));
        assert!(!html.contains("carousel("));
        assert!(!html.contains("zinc-"));
    }

    /// React `LogoMark` without icon/node: motion wrapper `div` >
    /// `span[aria-hidden]` > `span` holding the initials; name on the `li`.
    fn li(label: &str, initials: &str) -> String {
        format!(
            "<li data-slot=\"logo-carousel-item\" aria-label=\"{label}\"><div><span aria-hidden=\"true\"><span>{initials}</span></span></div></li>"
        )
    }

    #[test]
    fn root_is_ul_with_items_not_interact_flex() {
        let html = render(&logos(&["Acme", "Stripe"]));
        assert!(html.starts_with("<ul data-slot=\"logo-carousel\""));
        assert!(html.contains("aria-live=\"off\""));
        assert!(html.contains("aria-label=\"Logo carousel\""));
        assert_eq!(html.matches("data-slot=\"logo-carousel-item\"").count(), 2);
        reject_stub(&html);
        assert_eq!(
            html,
            format!(
                "<ul data-slot=\"logo-carousel\" aria-label=\"Logo carousel\" aria-live=\"off\">{}{}</ul>",
                li("Acme", "A"),
                li("Stripe", "S")
            )
        );
    }

    /// React shows initials, never the full name, when a logo has no icon.
    #[test]
    fn items_show_initials_not_names() {
        let html = render(&logos(&["Acme", "Globex Corp", "big  blue  sky"]));
        assert!(html.contains(&li("Acme", "A")));
        assert!(html.contains(&li("Globex Corp", "GC")));
        assert!(html.contains(&li("big  blue  sky", "BB")));
        assert!(!html.contains(">Acme<"));
        assert!(!html.contains(">Globex Corp<"));
        reject_stub(&html);
    }

    /// Idle page 0 renders one logo per column (default 3); the rest only
    /// appear when React's timer pages.
    #[test]
    fn idle_page_caps_at_three_columns() {
        let html = render(&logos(&["Acme", "Globex", "Initech", "Umbrella"]));
        assert_eq!(html.matches("data-slot=\"logo-carousel-item\"").count(), 3);
        assert!(!html.contains("Umbrella"));
        reject_stub(&html);
    }

    #[test]
    fn extra_text_items_become_logos_label_names_list() {
        let mut c = stub("logo-carousel", "Acme");
        c.items.push(extra("text", "Stripe"));
        c.items.push(extra("text", "Vercel"));
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"logo-carousel-item\"").count(), 2);
        assert!(!html.contains("aria-label=\"Acme\"><div>"));
        assert!(html.contains(&li("Stripe", "S")));
        assert!(html.contains(&li("Vercel", "V")));
        assert!(html.starts_with("<ul data-slot=\"logo-carousel\" aria-label=\"Acme\""));
        reject_stub(&html);
    }

    /// Audit fixture shape: emitter writes `label "Logos"` (from aria-label),
    /// `text` per item and `aria-label:"Logos"`. React renders only Acme/Globex.
    #[test]
    fn fixture_aria_label_is_not_a_logo() {
        // Label differs from aria-label to prove the item-config value wins.
        let mut c = stub("logo-carousel", "Brands");
        c.items.push(extra("text", "Acme"));
        let mut globex = extra("text", "Globex");
        globex.config.insert("aria-label".into(), "Logos".into());
        c.items.push(globex);
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<ul data-slot=\"logo-carousel\" aria-label=\"Logos\" aria-live=\"off\">{}{}</ul>",
                li("Acme", "A"),
                li("Globex", "G")
            )
        );
        reject_stub(&html);
    }

    #[test]
    fn label_only_still_emits_one_item() {
        let html = render(&stub("logo-carousel", "Acme"));
        assert!(html.starts_with("<ul data-slot=\"logo-carousel\""));
        assert_eq!(html.matches("data-slot=\"logo-carousel-item\"").count(), 1);
        assert!(html.contains(&li("Acme", "A")));
        assert!(html.contains("aria-live=\"off\""));
        reject_stub(&html);
    }

    /// Initials come from raw text (`<B>` → `<`), then get escaped once.
    #[test]
    fn label_is_escaped() {
        let html = render(&stub("logo-carousel", "A <B> & \"C\""));
        assert!(html.contains(&li("A &lt;B&gt; &amp; &quot;C&quot;", "A&lt;")));
        assert!(!html.contains("&amp;lt;"));
        reject_stub(&html);
    }

    #[test]
    fn skips_interact_flex_and_display_surf() {
        let c = logos(&["Acme", "Stripe"]);
        let html = render(&c);
        assert!(html.contains("data-slot=\"logo-carousel-item\""));
        assert!(html.starts_with("<ul"));
        assert!(!html.contains("<section"));
        assert!(!html.contains(DISPLAY_SURF));
        reject_stub(&html);
        let fx = crate::cronus_ui_widgets::render(&crate::cronus_ui_widgets::test_stub("meteors"))
            .unwrap();
        assert!(fx.contains(FX_BOX));
        assert_ne!(html, fx);
        assert_eq!(
            dedicated_fn_name("logo-carousel"),
            Some("cronus_ui_logo_carousel::render")
        );
        assert_eq!(
            renderer_kind("logo-carousel"),
            RendererKind::Dedicated("cronus_ui_logo_carousel::render")
        );
        assert_eq!(renderer_kind("meteors"), RendererKind::Stub("fx"));
        assert_eq!(renderer_kind("sankey-chart"), RendererKind::Stub("chart"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&logos(&["Acme"]));
            reject_stub(&html);
            assert!(html.contains("data-slot=\"logo-carousel-item\""));
            assert!(html.contains("aria-live=\"off\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"logo-carousel\"]"));
        assert!(css.contains("[data-slot=\"logo-carousel-item\"]"));
        assert!(css.contains("list-style: none"));
        // Wave 1s geometry parity (React measured: ul 288x96, li 138x96, 60px initials).
        assert!(css.contains(
            "display: grid; grid-auto-flow: column; grid-auto-columns: minmax(0, 1fr);\n  gap: 0.75rem; width: 18rem;"
        ));
        assert!(css.contains("position: relative; box-sizing: border-box;"));
        assert!(css.contains("overflow: hidden; height: 5rem; padding: 0 0.75rem;"));
        assert!(css.contains("border: 1px solid transparent;\n  background: transparent;"));
        assert!(css
            .contains("color: color-mix(in oklab, var(--cronus-fg-secondary) 70%, transparent);"));
        assert!(css.contains(
            "font-size: 2.25rem; font-weight: 600; line-height: 1; color: currentColor;"
        ));
        assert!(css.contains(
            "[data-slot=\"logo-carousel-item\"] { height: 6rem; }\n  [data-slot=\"logo-carousel-item\"] > div > span { font-size: 3.75rem; }"
        ));
        assert!(!css.contains("repeat(auto-fit, minmax(6rem, 1fr))"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains("setInterval"));
        assert!(!css.contains(INTERACT_ROW));
        assert!(!css.contains(DISPLAY_SURF));
    }
}
