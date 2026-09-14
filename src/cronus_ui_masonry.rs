//! Dedicated Masonry renderer. DOM matches React `Masonry` (audit harness:
//! `columns={2}`, `w-72`): `<div data-slot="masonry" aria-label=…>` whose
//! direct children are plain card `<div>`s (React gives them no slot).
//! CSS columns, zero JS. Not catalog `display()` SURF, not interact `scroll()`.

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let mut cells: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type != "label" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if cells.is_empty() {
        cells.push(label_of(comp));
    }
    let aria = attr(comp, "aria-label")
        .map(|a| format!(" aria-label=\"{}\"", esc(a)))
        .unwrap_or_default();
    let body = cells.iter().map(|t| format!("<div>{t}</div>")).collect::<String>();
    format!("<div data-slot=\"masonry\"{aria}>{body}</div>")
}

fn attr<'a>(comp: &'a ComponentNode, key: &str) -> Option<&'a str> {
    comp.props
        .get(key)
        .map(String::as_str)
        .or_else(|| comp.items.iter().find_map(|i| i.config.get(key).map(String::as_str)))
        .filter(|s| !s.is_empty())
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

    /// Emitter shape: `label "Gallery"`, `text` per card, `aria-label:` on the last item.
    fn wall(label: &str, items: &[&str]) -> ComponentNode {
        let mut c = stub("masonry", label);
        for n in items {
            c.items.push(extra("text", n));
        }
        if let Some(last) = c.items.last_mut() {
            last.config.insert("aria-label".into(), label.into());
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<section"));
        assert!(!html.contains("<details"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("max-height:12rem;overflow:auto"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("masonry-cell"));
    }

    #[test]
    fn dom_matches_react_masonry() {
        let html = render(&wall("Gallery", &["Alpha", "Beta", "Gamma", "Delta"]));
        assert_eq!(
            html,
            "<div data-slot=\"masonry\" aria-label=\"Gallery\"><div>Alpha</div><div>Beta</div><div>Gamma</div><div>Delta</div></div>"
        );
        reject_interact(&html);
        assert!(!crate::cli::stub_renderer_gate::looks_like_interact_generic(&html));
    }

    #[test]
    fn label_only_still_emits_one_cell_without_aria() {
        let html = render(&stub("masonry", "Card"));
        assert_eq!(html, "<div data-slot=\"masonry\"><div>Card</div></div>");
        reject_interact(&html);
    }

    #[test]
    fn text_is_escaped() {
        let html = render(&wall("G & \"H\"", &["<b>"]));
        assert!(html.contains("aria-label=\"G &amp; &quot;H&quot;\""));
        assert!(html.contains("<div>&lt;b&gt;</div>"));
    }

    #[test]
    fn skips_interact_scroll_and_display_surf() {
        let c = wall("Gallery", &["Alpha", "Beta"]);
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("masonry", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("max-height:12rem;overflow:auto"));
        assert!(crate::cli::stub_renderer_gate::looks_like_interact_generic(&interact));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&wall("Gallery", &["Alpha"]));
            reject_interact(&html);
            assert!(html.contains("<div>Alpha</div>"));
        });
    }

    #[test]
    fn chrome_is_token_only_and_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"masonry\"] {\n  box-sizing: border-box; width: 18rem; max-width: 100%;\n  column-count: 2; column-gap: 1rem;"));
        assert!(css.contains("[data-slot=\"masonry\"] > * { margin-bottom: 1rem; break-inside: avoid; }"));
        assert!(css.contains("padding: 0.75rem; font-size: 0.875rem; line-height: 1.25rem;"));
        assert!(!css.contains("[data-slot=\"masonry-cell\"]"));
        assert!(!css.contains("zinc-"));
    }
}
