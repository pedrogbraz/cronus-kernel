//! Dedicated ScrollArea renderer. DOM matches React/Radix idle:
//! `<div data-slot="scroll-area">` > viewport `<div role="region"
//! aria-label tabindex="0">` > `<div>` column of rows. Radix mounts its
//! scrollbar only while hovering/scrolling (`type="hover"`), so the idle DOM
//! has no scrollbar slot and neither does the kernel; the native scrollbar is
//! hidden like Radix hides it. The label names the region; rows come from the
//! `text` items. Size mirrors the audit harness default `h-32 w-48`.
//! Not interact `scroll()` / catalog `display()` (SURF box without viewport).

use crate::cronus_ui_kit::{attr_nonempty, esc, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let label = aria_label(comp).unwrap_or_else(|| label_of(comp));
    let mut rows: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type != "label" && i.item_type != "title" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if rows.is_empty() {
        rows.push(label.clone());
    }
    let rows = rows
        .into_iter()
        .map(|t| format!("<div>{t}</div>"))
        .collect::<String>();
    format!(
        "<div data-slot=\"scroll-area\"><div role=\"region\" aria-label=\"{label}\" tabindex=\"0\"><div>{rows}</div></div></div>"
    )
}

fn aria_label(comp: &ComponentNode) -> Option<String> {
    attr_nonempty(comp, "aria-label").map(|s| esc(s))
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

    #[test]
    fn skips_interact_scroll_surf() {
        let mut c = stub("scroll-area", "Notes");
        c.items.push(extra("text", "Alpha"));
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("scroll-area", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("max-height:12rem;overflow:auto"));
        assert!(crate::cli::stub_renderer_gate::looks_like_interact_generic(
            &interact
        ));
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
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"scroll-area\"] {\n  position: relative; overflow: hidden;\n  width: 12rem; height: 8rem;\n}"
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
