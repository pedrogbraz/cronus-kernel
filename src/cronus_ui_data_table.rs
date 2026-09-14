//! Dedicated DataTable renderer. DOM matches React (`data-table.tsx` over
//! `table.tsx`, no toolbar/pagination):
//! `<div data-slot="data-table">` → `data-table-container` (rounded border) →
//! `<section data-slot="table-container">` → `<table data-slot="table">` with
//! `table-header` / `table-body`, `table-row`, `table-head` and `table-cell`.
//!
//! Content: bound rows (keys become headers) → `columns` items as headers with
//! the content texts wrapped into rows → otherwise the first two content texts
//! are headers and the rest fill two-column rows. The `label` names the table
//! (never a header or cell); label-only tables show a Name/Value dummy row.
//!
//! Zero JS: sorting, filtering, column visibility and pagination need a
//! runtime; the table renders React's unsorted idle state (`aria-sort="none"`
//! headers, no sort buttons). Not interact `table("data-table")`.

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

/// Item kinds that name the widget rather than carry table content.
const NAME_KINDS: &[&str] = &["label", "title", "columns"];

pub fn render(comp: &ComponentNode) -> String {
    let bound = crate::cronus_ui_data::rows();
    if !bound.is_empty() {
        return from_bound(&bound);
    }
    let (headers, rows) = headers_and_rows(comp);
    wrap(&headers, &rows)
}

fn wrap(headers: &[String], rows: &[Vec<String>]) -> String {
    let head = headers
        .iter()
        .map(|c| {
            format!("<th data-slot=\"table-head\" colspan=\"1\" scope=\"col\" aria-sort=\"none\">{c}</th>")
        })
        .collect::<String>();
    let body = rows
        .iter()
        .map(|row| {
            let tds = row
                .iter()
                .map(|c| format!("<td data-slot=\"table-cell\">{c}</td>"))
                .collect::<String>();
            format!("<tr data-slot=\"table-row\">{tds}</tr>")
        })
        .collect::<String>();
    format!(
        "<div data-slot=\"data-table\"><div data-slot=\"data-table-container\"><section data-slot=\"table-container\" tabindex=\"0\" aria-label=\"Table\"><table data-slot=\"table\"><thead data-slot=\"table-header\"><tr data-slot=\"table-row\">{head}</tr></thead><tbody data-slot=\"table-body\">{body}</tbody></table></section></div></div>"
    )
}

fn from_bound(bound: &[serde_json::Value]) -> String {
    let keys: Vec<String> = bound
        .first()
        .and_then(|row| row.as_object())
        .map(|o| o.keys().filter(|k| !k.starts_with('_')).cloned().collect())
        .unwrap_or_default();
    let headers: Vec<String> = keys.iter().map(|k| esc(k)).collect();
    let rows: Vec<Vec<String>> = bound
        .iter()
        .map(|row| {
            keys.iter()
                .map(|k| {
                    row.get(k)
                        .map(|x| match x {
                            serde_json::Value::String(s) => esc(s),
                            other => esc(&other.to_string()),
                        })
                        .unwrap_or_default()
                })
                .collect()
        })
        .collect();
    wrap(&headers, &rows)
}

fn headers_and_rows(comp: &ComponentNode) -> (Vec<String>, Vec<Vec<String>>) {
    let headers: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "columns" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    let cells: Vec<String> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty() && !NAME_KINDS.contains(&i.item_type.as_str()))
        .map(|i| esc(&i.text))
        .collect();
    if !headers.is_empty() {
        let n = headers.len();
        return (headers, chunk_or_dummy(&cells, n));
    }
    match cells.len() {
        0 => (
            vec!["Name".into(), "Value".into()],
            vec![vec![label_of(comp), "—".into()]],
        ),
        1 => (
            vec!["Name".into(), "Value".into()],
            vec![vec![cells[0].clone(), "—".into()]],
        ),
        _ => {
            let (head, rest) = cells.split_at(2);
            (head.to_vec(), chunk_or_dummy(rest, 2))
        }
    }
}

fn chunk_or_dummy(cells: &[String], n: usize) -> Vec<Vec<String>> {
    if n == 0 {
        return Vec::new();
    }
    if cells.is_empty() {
        return vec![vec!["—".into(); n]];
    }
    cells
        .chunks(n)
        .map(|chunk| {
            (0..n)
                .map(|i| chunk.get(i).cloned().unwrap_or_else(|| "—".into()))
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn extra(item_type: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: item_type.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("text-align: start;padding:0.5rem 0.75rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("zinc-"));
    }

    fn th(t: &str) -> String {
        format!(
            "<th data-slot=\"table-head\" colspan=\"1\" scope=\"col\" aria-sort=\"none\">{t}</th>"
        )
    }

    #[test]
    fn label_only_is_name_value_dummy_in_react_table_dom() {
        let html = render(&stub("data-table", "People"));
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"data-table\"><div data-slot=\"data-table-container\"><section data-slot=\"table-container\" tabindex=\"0\" aria-label=\"Table\"><table data-slot=\"table\"><thead data-slot=\"table-header\"><tr data-slot=\"table-row\">{}{}</tr></thead><tbody data-slot=\"table-body\"><tr data-slot=\"table-row\"><td data-slot=\"table-cell\">People</td><td data-slot=\"table-cell\">—</td></tr></tbody></table></section></div></div>",
                th("Name"),
                th("Value")
            )
        );
        reject_interact(&html);
    }

    #[test]
    fn columns_are_headers_remaining_are_cells() {
        let mut c = stub("data-table", "People");
        c.items = vec![
            extra("columns", "Name"),
            extra("columns", "Role"),
            extra("item", "Ada"),
            extra("item", "Engineer"),
            extra("item", "Grace"),
            extra("item", "Admiral"),
        ];
        let html = render(&c);
        assert!(html.contains(&format!("{}{}", th("Name"), th("Role"))));
        assert!(html.contains("<tr data-slot=\"table-row\"><td data-slot=\"table-cell\">Ada</td><td data-slot=\"table-cell\">Engineer</td></tr>"));
        assert!(html.contains(
            "<td data-slot=\"table-cell\">Grace</td><td data-slot=\"table-cell\">Admiral</td>"
        ));
        reject_interact(&html);
    }

    /// `label` + `text` items (the emitter's shape): the label names the table
    /// and the texts fill it — texts used to be dropped as "field" kinds.
    #[test]
    fn text_items_fill_table_and_label_is_not_content() {
        let mut c = stub("data-table", "Members");
        for t in ["Name", "Role", "Ada", "Admin", "Linus", "Editor"] {
            c.items.push(extra("text", t));
        }
        let html = render(&c);
        assert!(html.contains(&format!(
            "<tr data-slot=\"table-row\">{}{}</tr>",
            th("Name"),
            th("Role")
        )));
        assert!(html.contains(
            "<td data-slot=\"table-cell\">Ada</td><td data-slot=\"table-cell\">Admin</td>"
        ));
        assert!(html.contains(
            "<td data-slot=\"table-cell\">Linus</td><td data-slot=\"table-cell\">Editor</td>"
        ));
        assert!(!html.contains("Members"));
        assert_eq!(html.matches("data-slot=\"table-row\"").count(), 3);
        reject_interact(&html);
    }

    #[test]
    fn bound_rows_fill_cells_without_voodoo() {
        use crate::binding::ResolvedData;
        let rows = vec![serde_json::json!({"name": "Ada", "role": "Eng"})];
        crate::cronus_ui_data::with_binding("Lead", &ResolvedData::Rows(rows), || {
            crate::voodoo::with_enabled(true, || {
                let html = render(&stub("data-table", "People"));
                assert!(html.contains(&th("name")));
                assert!(html.contains(&th("role")));
                assert!(html.contains("<td data-slot=\"table-cell\">Ada</td>"));
                reject_interact(&html);
            });
        });
    }

    #[test]
    fn skips_interact_inline_th_styles() {
        let c = stub("data-table", "People");
        let html = render(&c);
        assert!(!html.contains("style="));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("data-table", "People"));
            reject_interact(&html);
            assert!(html.contains("<table data-slot=\"table\">"));
        });
    }

    /// Geometry parity (Wave 1t): rounded bordered container, `text-sm`
    /// 14px/20px table, 40px `font-medium` heads, `p-3` cells, row dividers
    /// with none under the last body row.
    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"data-table-container\"] {\n  overflow: hidden; border-radius: var(--cronus-radius-xl); border: 1px solid var(--cronus-border);\n}"));
        assert!(css.contains("[data-slot=\"data-table\"] [data-slot=\"table-head\"] {\n  height: 2.5rem; padding: 0 0.75rem; text-align: start; vertical-align: middle;\n  font-size: inherit; font-weight: 500; color: var(--cronus-fg-secondary); white-space: nowrap; border: 0;\n}"));
        assert!(css.contains("[data-slot=\"data-table\"] [data-slot=\"table-body\"] > [data-slot=\"table-row\"]:last-child { border-bottom: 0; }"));
        assert!(!css.contains("zinc-"));
    }
}
