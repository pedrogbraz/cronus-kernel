//! Dedicated Masonry renderer. DOM matches React `Masonry` (audit harness:
//! `columns={2}`, `w-72`): `<div data-slot="masonry" aria-label=…>` whose
//! direct children are plain card `<div>`s (React gives them no slot). An
//! `item "Title" description:"…"` renders the docs' `Card` (header title +
//! content paragraph) instead of a plain cell. `columns:3` fixes the column
//! count and `columns:"1,2,3"` is React's responsive map (base, `sm`, `lg`),
//! carried as classes (`cols-1 sm-cols-2 lg-cols-3`; the default stays the
//! harness's two columns). CSS columns, zero JS. Not catalog `display()` SURF,
//! not interact `scroll()`.

use crate::cronus_ui_kit::{attr_nonempty, esc, label_of};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let mut cells: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type != "label" && !i.text.is_empty())
        .map(
            |i| match i.config.get("description").filter(|d| !d.is_empty()) {
                Some(body) => crate::cronus_ui_card::titled_card(&esc(&i.text), &esc(body)),
                None => format!("<div>{}</div>", esc(&i.text)),
            },
        )
        .collect();
    if cells.is_empty() {
        cells.push(format!("<div>{}</div>", label_of(comp)));
    }
    let aria = attr_nonempty(comp, "aria-label")
        .map(|a| format!(" aria-label=\"{}\"", esc(a)))
        .unwrap_or_default();
    let class = column_classes(comp)
        .map(|c| format!(" class=\"{c}\""))
        .unwrap_or_default();
    format!(
        "<div data-slot=\"masonry\"{class}{aria}>{}</div>",
        cells.concat()
    )
}

/// `columns:N` → `cols-N`; `columns:"base,sm,lg[,md,xl]"` → the responsive
/// classes, each count clamped to 1..=6 like React.
fn column_classes(comp: &ComponentNode) -> Option<String> {
    let raw = attr_nonempty(comp, "columns")?;
    let counts: Vec<u8> = raw
        .split(',')
        .filter_map(|p| p.trim().parse::<f64>().ok())
        .map(|n| n.round().clamp(1.0, 6.0) as u8)
        .collect();
    let prefixes = ["cols", "sm-cols", "lg-cols", "md-cols", "xl-cols"];
    let classes: Vec<String> = counts
        .iter()
        .zip(prefixes)
        .map(|(n, p)| format!("{p}-{n}"))
        .collect();
    if classes.is_empty() {
        None
    } else {
        Some(classes.join(" "))
    }
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

    /// Docs "Responsive cards": described items are Cards and the responsive
    /// `columns` map becomes base/sm/lg classes.
    #[test]
    fn described_items_are_cards_with_responsive_columns() {
        let mut c = stub("masonry", "Features");
        c.items.clear();
        let mut card = extra("item", "Onboarding");
        card.config
            .insert("description".into(), "Passwordless email code.".into());
        c.items.push(card);
        c.items.push(extra("item", "Plain"));
        c.props.insert("columns".into(), "1,2,3".into());
        assert_eq!(
            render(&c),
            "<div data-slot=\"masonry\" class=\"cols-1 sm-cols-2 lg-cols-3\"><div data-slot=\"card\"><div data-slot=\"card-header\"><div data-slot=\"card-title\">Onboarding</div></div><div data-slot=\"card-content\"><p>Passwordless email code.</p></div></div><div>Plain</div></div>"
        );
        c.props.insert("columns".into(), "9".into());
        assert!(render(&c).contains("class=\"cols-6\""));
        let css = include_str!("cronus_ui_css/masonry.css");
        assert!(css.contains("[data-slot=\"masonry\"].cols-1 { column-count: 1; }"));
        assert!(css.contains("@media (min-width: 1024px) {\n  [data-slot=\"masonry\"].lg-cols-3 { column-count: 3; }"));
        assert!(css.contains("[data-slot=\"masonry\"] > [data-slot=\"card\"] {\n  padding: 1.5rem 0; border-radius: var(--cronus-radius-xl); font-size: inherit; line-height: inherit;\n}"));
    }

    #[test]
    fn skips_interact_scroll_and_display_surf() {
        let c = wall("Gallery", &["Alpha", "Beta"]);
        let html = render(&c);
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
        let css = include_str!("cronus_ui_css/masonry.css");
        assert!(css.contains("[data-slot=\"masonry\"] {\n  box-sizing: border-box; width: var(--cui-masonry-w, 100%); max-width: 100%;\n  column-count: 2; column-gap: 1rem;"));
        assert!(css
            .contains("[data-slot=\"masonry\"] > * { margin-bottom: 1rem; break-inside: avoid; }"));
        assert!(css.contains("padding: 0.75rem; font-size: 0.875rem; line-height: 1.25rem;"));
        assert!(!css.contains("[data-slot=\"masonry-cell\"]"));
        assert!(!css.contains("zinc-"));
    }
}
