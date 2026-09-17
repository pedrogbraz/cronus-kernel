//! Dedicated ScrollArea renderer. DOM matches React/Radix idle:
//! `<div data-slot="scroll-area">` > viewport `<div tabindex="0">` (a named
//! `role="region"` only when a label exists, like React) > `<div>` column of
//! rows. Radix mounts its scrollbar only while hovering/scrolling
//! (`type="hover"`), so the idle DOM has no scrollbar slot and neither does the
//! kernel; the native scrollbar is hidden like Radix hides it and a thin,
//! token-coloured one appears on hover/focus. Rows come from the `item` /
//! `text` lines. Size mirrors the audit harness default `h-32 w-48`; the docs'
//! utilities travel as classes: `height:48` (`h-48`), `max-width:xs`
//! (`mw-xs`), `framed:true` (rounded border on surface-inset) and `font:mono`
//! (the `p-4` list of mono `text-fg-secondary` rows).
//! Not interact `scroll()` / catalog `display()` (SURF box without viewport).

use crate::cronus_ui_kit::{attr_nonempty, esc, flag, item};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = attr_nonempty(comp, "aria-label")
        .or_else(|| item(comp, "label").filter(|l| !l.is_empty()))
        .or_else(|| item(comp, "title").filter(|l| !l.is_empty()))
        .map(esc);
    let mut rows: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type != "label" && i.item_type != "title" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if rows.is_empty() {
        rows.push(label.clone().unwrap_or_else(|| esc(&comp.name)));
    }
    let rows = rows
        .into_iter()
        .map(|t| format!("<div>{t}</div>"))
        .collect::<String>();
    let region = label
        .map(|l| format!(" role=\"region\" aria-label=\"{l}\""))
        .unwrap_or_default();
    let mut classes: Vec<&str> = Vec::new();
    match attr_nonempty(comp, "height").map(str::trim) {
        Some("32") => classes.push("h-32"),
        Some("40") => classes.push("h-40"),
        Some("48") => classes.push("h-48"),
        Some("64") => classes.push("h-64"),
        Some("72") => classes.push("h-72"),
        _ => {}
    }
    if let Some(mw) = crate::cronus_ui_card::max_width_class(comp) {
        classes.push(mw);
    }
    if flag(comp, "framed") {
        classes.push("framed");
    }
    let class = if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    };
    let list = if attr_nonempty(comp, "font").is_some_and(|f| f.trim() == "mono") {
        " class=\"font-mono\""
    } else {
        ""
    };
    format!(
        "<div data-slot=\"scroll-area\"{class}><div{region} tabindex=\"0\"><div{list}>{rows}</div></div></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_interact(html: &str) {
        assert!(!html.starts_with("<section"));
        assert!(!html.contains("<details"));
        assert!(!html.contains("-control"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("max-height:12rem;overflow:auto"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
    }

    /// wave1t: React idle has only the root slot; the label is the region's
    /// name, never a row.
    #[test]
    fn root_matches_radix_idle_dom() {
        let mut c = stub("scroll-area", "Release notes");
        c.items.push(extra("text", "v1.2.0"));
        c.items.push(extra("text", "v1.1.0"));
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"scroll-area\"><div role=\"region\" aria-label=\"Release notes\" tabindex=\"0\"><div><div>v1.2.0</div><div>v1.1.0</div></div></div></div>"
        );
        assert!(!html.contains("data-slot=\"scroll-area-viewport\""));
        assert!(!html.contains("data-slot=\"scroll-bar\""));
        assert!(!html.contains("<div>Release notes</div>"));
        reject_interact(&html);
    }

    #[test]
    fn aria_label_from_trailing_item_config() {
        let mut c = stub("scroll-area", "Notes");
        let mut last = extra("text", "Row");
        last.config.insert("aria-label".into(), "Changelog".into());
        c.items.push(last);
        let html = render(&c);
        assert!(html.contains("role=\"region\" aria-label=\"Changelog\""));
        assert!(html.contains("<div><div>Row</div></div>"));
        reject_interact(&html);
    }

    #[test]
    fn label_only_still_emits_one_row() {
        let html = render(&stub("scroll-area", "Release notes"));
        assert!(html.contains("<div><div>Release notes</div></div>"));
        reject_interact(&html);
    }

    /// Docs "Scrollable list": no label means no region role (React only
    /// names the viewport when asked); the utilities become classes.
    #[test]
    fn docs_list_has_no_region_role_and_utility_classes() {
        let mut c = stub("scroll-area", "");
        c.items.clear();
        c.items.push(extra("item", "v1.2.0-beta.20"));
        c.items.push(extra("item", "v1.2.0-beta.19"));
        c.props.insert("height".into(), "48".into());
        c.props.insert("max-width".into(), "xs".into());
        c.props.insert("framed".into(), "true".into());
        c.props.insert("font".into(), "mono".into());
        assert_eq!(
            render(&c),
            "<div data-slot=\"scroll-area\" class=\"h-48 mw-xs framed\"><div tabindex=\"0\"><div class=\"font-mono\"><div>v1.2.0-beta.20</div><div>v1.2.0-beta.19</div></div></div></div>"
        );
        let css = include_str!("cronus_ui_css/scroll-area.css");
        assert!(css.contains("[data-slot=\"scroll-area\"].h-48 { height: 12rem; }"));
        assert!(css.contains("[data-slot=\"scroll-area\"].framed {\n  border-radius: var(--cronus-radius-xl); border: 1px solid var(--cronus-border-soft, var(--cronus-border)); background: var(--cronus-surface-inset); box-sizing: border-box;\n}"));
        assert!(css.contains("[data-slot=\"scroll-area\"] > div > div.font-mono > div {\n  border-radius: var(--cronus-radius-md); padding: 0.375rem 0.5rem;\n  font-family: var(--cronus-font-mono, ui-monospace, monospace); font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg-secondary);\n}"));
        assert!(css
            .contains("scrollbar-width: thin; scrollbar-color: var(--cronus-border) transparent;"));
    }

    #[test]
    fn skips_interact_scroll_surf() {
        let mut c = stub("scroll-area", "Notes");
        c.items.push(extra("text", "Alpha"));
        let html = render(&c);
        assert!(!crate::cli::stub_renderer_gate::looks_like_interact_generic(&html));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("scroll-area", "Alpha"));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_mirrors_h32_w48_and_hides_native_scrollbar() {
        let css = include_str!("cronus_ui_css/scroll-area.css");
        assert!(css.contains(
            "[data-slot=\"scroll-area\"] {\n  position: relative; overflow: hidden;\n  width: var(--cui-scroll-area-w, 100%); height: 8rem;\n}"
        ));
        assert!(css.contains("overflow-x: hidden; overflow-y: scroll; scrollbar-width: none;"));
        assert!(css.contains(
            "[data-slot=\"scroll-area\"] > [role=\"region\"] > div {\n  display: flex; flex-direction: column; gap: 0.25rem; padding: 0.5rem;\n}"
        ));
        assert!(!css.contains("[data-slot=\"scroll-bar\"]"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
    }
}
