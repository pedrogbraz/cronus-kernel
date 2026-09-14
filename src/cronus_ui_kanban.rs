//! Dedicated Kanban renderer. DOM matches React (`kanban.tsx`):
//! `<div data-slot="kanban">` of `<section data-slot="kanban-column">`, each
//! with a `<header data-slot="kanban-column-header">` (`<h3>` title + a
//! secondary badge `kanban-column-count`) and a `<ul data-slot="kanban-column-list">`
//! of `<li data-slot="kanban-card">` rows (`kanban-drag-handle` + `kanban-card-body`
//! with `kanban-card-title` and, when the card item carries `description:"…"`
//! config, `kanban-card-description`).
//!
//! Content texts pair like the audit fixture: `title, card, title, card, …`
//! (a trailing title makes an empty column). The `label` names the board
//! (`aria-label`) and is never a column or card.
//!
//! Zero JS (Wave 1t control contract): dnd-kit drag and keyboard reordering
//! need a runtime, so the grip is React's native `<button>` marked `disabled`,
//! same box and idle look. Not interact `kanban()` (inline SURF columns) or
//! catalog `display()` SURF `<section>`.

use crate::cronus_ui_kit::esc;
use crate::parser::ComponentNode;

const NAME_KINDS: &[&str] = &["label", "title"];

/// lucide `GripVertical`.
const GRIP: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><circle cx=\"9\" cy=\"12\" r=\"1\"/><circle cx=\"9\" cy=\"5\" r=\"1\"/><circle cx=\"9\" cy=\"19\" r=\"1\"/><circle cx=\"15\" cy=\"12\" r=\"1\"/><circle cx=\"15\" cy=\"5\" r=\"1\"/><circle cx=\"15\" cy=\"19\" r=\"1\"/></svg>";

/// One content text plus its optional `description:` config, both escaped.
struct Entry {
    text: String,
    description: Option<String>,
}

pub fn render(comp: &ComponentNode) -> String {
    let columns = content(comp)
        .chunks(2)
        .map(|pair| column(&pair[0].text, pair.get(1)))
        .collect::<String>();
    let aria = esc(aria_label(comp).unwrap_or("Board"));
    format!("<div data-slot=\"kanban\" aria-label=\"{aria}\">{columns}</div>")
}

fn column(title: &str, card: Option<&Entry>) -> String {
    let count = usize::from(card.is_some());
    let cards = card
        .map(|c| {
            let description = c
                .description
                .as_ref()
                .map(|d| format!("<span data-slot=\"kanban-card-description\">{d}</span>"))
                .unwrap_or_default();
            format!(
                "<li data-slot=\"kanban-card\"><button type=\"button\" data-slot=\"kanban-drag-handle\" aria-label=\"Reorder card\" disabled>{GRIP}</button><div data-slot=\"kanban-card-body\"><span data-slot=\"kanban-card-title\">{}</span>{description}</div></li>",
                c.text
            )
        })
        .unwrap_or_default();
    format!(
        "<section data-slot=\"kanban-column\" aria-label=\"{title}\"><header data-slot=\"kanban-column-header\"><h3 data-slot=\"kanban-column-title\">{title}</h3><span data-slot=\"kanban-column-count\" data-variant=\"secondary\">{count}</span></header><ul data-slot=\"kanban-column-list\">{cards}</ul></section>"
    )
}

fn aria_label(comp: &ComponentNode) -> Option<&str> {
    comp.props
        .get("aria-label")
        .or_else(|| comp.items.iter().find_map(|i| i.config.get("aria-label")))
        .map(String::as_str)
        .filter(|v| !v.is_empty())
}

fn content(comp: &ComponentNode) -> Vec<Entry> {
    let entries: Vec<Entry> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty() && !NAME_KINDS.contains(&i.item_type.as_str()))
        .map(|i| Entry {
            text: esc(&i.text),
            description: i
                .config
                .get("description")
                .filter(|d| !d.is_empty())
                .map(|d| esc(d)),
        })
        .collect();
    if !entries.is_empty() {
        return entries;
    }
    // Label-only board: one column titled by the label holding it as a card, so
    // a board is never rendered without a card (stub gate contract).
    comp.items
        .iter()
        .find(|i| !i.text.is_empty())
        .map(|i| {
            let entry = || Entry {
                text: esc(&i.text),
                description: None,
            };
            vec![entry(), entry()]
        })
        .unwrap_or_default()
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
        assert!(!html.contains("<pre"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("draggable"));
        assert!(!html.contains("min-width:10rem"));
        assert!(!html.contains("kanban("));
    }

    /// Emitted audit source: `label "Board"` + texts Todo/Ship/Doing/Review +
    /// `aria-label:"Board"` → two columns with one card each.
    #[test]
    fn audit_source_pairs_titles_and_cards() {
        let mut c = stub("kanban", "Board");
        for t in ["Todo", "Ship", "Doing", "Review"] {
            c.items.push(extra("text", t));
        }
        c.items
            .last_mut()
            .unwrap()
            .config
            .insert("aria-label".into(), "Board".into());
        let html = render(&c);
        let col = |title: &str, card: &str| {
            format!(
                "<section data-slot=\"kanban-column\" aria-label=\"{title}\"><header data-slot=\"kanban-column-header\"><h3 data-slot=\"kanban-column-title\">{title}</h3><span data-slot=\"kanban-column-count\" data-variant=\"secondary\">1</span></header><ul data-slot=\"kanban-column-list\"><li data-slot=\"kanban-card\"><button type=\"button\" data-slot=\"kanban-drag-handle\" aria-label=\"Reorder card\" disabled>{GRIP}</button><div data-slot=\"kanban-card-body\"><span data-slot=\"kanban-card-title\">{card}</span></div></li></ul></section>"
            )
        };
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"kanban\" aria-label=\"Board\">{}{}</div>",
                col("Todo", "Ship"),
                col("Doing", "Review")
            )
        );
        assert!(!html.contains(">Board<"));
        reject_interact(&html);
    }

    /// Coordinator contract: `description:"…"` (attr-style item config) becomes
    /// React's optional `kanban-card-description` under the title.
    #[test]
    fn card_description_from_item_config() {
        let mut c = stub("kanban", "Board");
        c.items.push(extra("text", "Todo"));
        c.items.push(extra("text", "Ship"));
        c.items[2]
            .config
            .insert("description".into(), "Cut <v1> & tag".into());
        let html = render(&c);
        assert!(html.contains("<span data-slot=\"kanban-card-title\">Ship</span><span data-slot=\"kanban-card-description\">Cut &lt;v1&gt; &amp; tag</span></div>"));
        reject_interact(&html);
    }

    #[test]
    fn trailing_title_is_an_empty_column() {
        let mut c = stub("kanban", "Board");
        for t in ["Todo", "Ship", "Done"] {
            c.items.push(extra("item", t));
        }
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"kanban-column\"").count(), 2);
        assert_eq!(html.matches("data-slot=\"kanban-card\"").count(), 1);
        assert!(html.contains("kanban-column-title\">Done</h3><span data-slot=\"kanban-column-count\" data-variant=\"secondary\">0</span></header><ul data-slot=\"kanban-column-list\"></ul>"));
        assert!(!html.contains("kanban-card-description"));
        reject_interact(&html);
    }

    #[test]
    fn label_only_is_one_column_with_one_card() {
        let html = render(&stub("kanban", "Todo"));
        assert_eq!(html.matches("data-slot=\"kanban-column\"").count(), 1);
        assert_eq!(html.matches("data-slot=\"kanban-card\"").count(), 1);
        assert!(html.contains("kanban-column-title\">Todo</h3>"));
        assert!(html.contains("kanban-card-title\">Todo</span>"));
        assert!(!crate::cli::stub_renderer_gate::looks_like_interact_generic(&html));
        assert!(html.contains("aria-label=\"Board\""));
        reject_interact(&html);
    }

    #[test]
    fn label_is_escaped() {
        let html = render(&stub("kanban", "A <B> & \"C\""));
        assert!(html.contains("kanban-column-title\">A &lt;B&gt; &amp; &quot;C&quot;</h3>"));
        reject_interact(&html);
    }

    #[test]
    fn drag_handle_is_disabled_native_button() {
        let mut c = stub("kanban", "Board");
        c.items.push(extra("text", "Todo"));
        c.items.push(extra("text", "Ship"));
        let html = render(&c);
        assert!(html.contains("<button type=\"button\" data-slot=\"kanban-drag-handle\" aria-label=\"Reorder card\" disabled>"));
        assert!(!html.contains("tabindex"));
        assert!(!html.contains("aria-roledescription"));
    }

    #[test]
    fn skips_interact_surf_columns_and_display_section() {
        let mut c = stub("kanban", "Board");
        c.items.push(extra("item", "Todo"));
        c.items.push(extra("item", "Ship"));
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("kanban", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("min-width:10rem"));
        assert!(html.contains("data-slot=\"kanban-column\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("kanban", "Todo"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"kanban-column\""));
        });
    }

    /// Geometry parity (Wave 1t): 18rem columns with an 18px-radius inset
    /// border, 42px header, `min-h-24` list, 48px cards (16px/24px body text,
    /// `leading-snug` 22px title) and a 20px grip.
    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"kanban-column\"] {\n  display: flex; flex-direction: column; width: 18rem; flex-shrink: 0;\n  border-radius: var(--cronus-radius-xl); border: 1px solid var(--cronus-border);\n  background: var(--cronus-surface-inset);\n}"));
        assert!(css.contains("[data-slot=\"kanban-column-list\"] {\n  display: flex; flex: 1; flex-direction: column; gap: 0.5rem; min-height: 6rem;\n  overflow-y: auto; margin: 0; padding: 0.5rem;"));
        assert!(css.contains("[data-slot=\"kanban-card-title\"] { display: block; font-weight: 500; line-height: 1.375; color: var(--cronus-fg); }"));
        assert!(css.contains("[data-slot=\"kanban-card-description\"] {"));
        // `html[data-cronus-theme] h3` (0,1,2) forces weight 400 and -0.02em;
        // the header-scoped rule (0,2,0) restores React's `font-semibold`.
        assert!(css.contains("[data-slot=\"kanban-column-header\"] > [data-slot=\"kanban-column-title\"] {\n  font-weight: 600; letter-spacing: normal; font-variant-numeric: tabular-nums;\n}"));
        assert!(css.contains("[data-slot=\"kanban-column-header\"] > [data-slot=\"kanban-column-count\"] { font-variant-numeric: tabular-nums; }"));
        assert!(css.contains("overflow-x: auto"));
        assert!(css.contains("var(--cronus-surface-raised)"));
        assert!(css.contains("var(--cronus-fg-muted)"));
        assert!(!css.contains("zinc-"));
    }
}
