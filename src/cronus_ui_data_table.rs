//! Dedicated DataTable renderer. DOM matches React:
//! `<div data-slot="data-table"><table><thead><tbody>`.
//! Columns from first text items as headers, remaining as cells
//! (or 2-col Name/Value dummy). Token CSS — no inline SURF on `th`.
//! Not interact `table("data-table")`.

use crate::cronus_ui_kit::{esc, label_of};
use crate::parser::ComponentNode;

const FIELD_KINDS: &[&str] = &["label", "title", "text", "value"];

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
        .map(|c| format!("<th>{c}</th>"))
        .collect::<Vec<_>>()
        .join("");
    let body = rows
        .iter()
        .map(|row| {
            let tds = row
                .iter()
                .map(|c| format!("<td>{c}</td>"))
                .collect::<Vec<_>>()
                .join("");
            format!("<tr>{tds}</tr>")
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<div data-slot=\"data-table\"><table><thead><tr>{head}</tr></thead><tbody>{body}</tbody></table></div>"
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
        .filter(|i| {
            !i.text.is_empty()
                && i.item_type != "columns"
                && !FIELD_KINDS.contains(&i.item_type.as_str())
        })
        .map(|i| esc(&i.text))
        .collect();
    if !headers.is_empty() {
        let n = headers.len();
        return (headers, chunk_or_dummy(&cells, n));
    }
    if !cells.is_empty() {
        // First text items as headers; leftover values wrap into rows.
        // A single leftover group with no remaining cells still gets a dummy row.
        if cells.len() == 1 {
            return (
                vec!["Name".into(), "Value".into()],
                vec![vec![cells[0].clone(), "—".into()]],
            );
        }
        let width = 2;
        let (head, rest) = cells.split_at(width.min(cells.len()));
        if rest.is_empty() {
            return (head.to_vec(), dummy_row(head.len()));
        }
        return (head.to_vec(), chunk_or_dummy(rest, head.len()));
    }
    (
        vec!["Name".into(), "Value".into()],
        vec![vec![label_of(comp), "—".into()]],
    )
}

fn chunk_or_dummy(cells: &[String], n: usize) -> Vec<Vec<String>> {
    if n == 0 {
        return Vec::new();
    }
    if cells.is_empty() {
        return dummy_row(n);
    }
    let mut rows = Vec::new();
    let mut i = 0;
    while i < cells.len() {
        let mut row = Vec::with_capacity(n);
        for _ in 0..n {
            row.push(cells.get(i).cloned().unwrap_or_else(|| "—".into()));
            i += 1;
        }
        rows.push(row);
    }
    rows
}

fn dummy_row(n: usize) -> Vec<Vec<String>> {
    vec![vec!["—".into(); n]]
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
        assert!(!html.contains("text-align:left;padding:0.5rem 0.75rem"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("v-resource="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("zinc-"));
    }

    #[test]
    fn root_is_data_table_with_native_table() {
        let html = render(&stub("data-table", "People"));
        assert!(html.starts_with("<div data-slot=\"data-table\">"));
        assert!(html.contains("<table"));
        assert!(html.contains("<thead"));
        assert!(html.contains("<tbody"));
        assert!(html.contains("<th>Name</th>"));
        assert!(html.contains("<th>Value</th>"));
        assert!(html.contains("<td>People</td>"));
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
        assert!(html.contains("<th>Name</th><th>Role</th>"));
        assert!(html.contains("<td>Ada</td><td>Engineer</td>"));
        assert!(html.contains("<td>Grace</td><td>Admiral</td>"));
        reject_interact(&html);
    }

    #[test]
    fn first_text_items_become_headers() {
        let mut c = stub("data-table", "People");
        c.items = vec![
            extra("item", "Name"),
            extra("item", "Role"),
            extra("item", "Ada"),
            extra("item", "Engineer"),
        ];
        let html = render(&c);
        assert!(html.contains("<th>Name</th><th>Role</th>"));
        assert!(html.contains("<td>Ada</td><td>Engineer</td>"));
        reject_interact(&html);
    }

    #[test]
    fn bound_rows_fill_cells_without_voodoo() {
        use crate::binding::ResolvedData;
        let rows = vec![serde_json::json!({"name": "Ada", "role": "Eng"})];
        crate::cronus_ui_data::with_binding("Lead", &ResolvedData::Rows(rows), || {
            crate::voodoo::with_enabled(true, || {
                let html = render(&stub("data-table", "People"));
                assert!(html.contains("data-slot=\"data-table\""));
                assert!(html.contains("<th>name</th>"));
                assert!(html.contains("<th>role</th>"));
                assert!(html.contains("<td>Ada</td>"));
                assert!(html.contains("<td>Eng</td>"));
                reject_interact(&html);
            });
        });
    }

    #[test]
    fn skips_interact_inline_th_styles() {
        let c = stub("data-table", "People");
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("data-table", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.contains("data-slot=\"data-table\""));
        assert!(interact.contains("text-align:left;padding:0.5rem 0.75rem"));
        assert!(interact.contains("style="));
        assert!(!html.contains("style="));
        assert!(html.contains("<table"));
        assert!(html.contains("data-slot=\"data-table\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("data-table", "People"));
            reject_interact(&html);
            assert!(html.contains("<table"));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"data-table\"]"));
        assert!(css.contains("[data-slot=\"data-table\"] th"));
        assert!(css.contains("var(--cronus-fg-secondary)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
        assert!(!css.contains("onclick"));
        assert!(!css.contains("text-align:left;padding:0.5rem 0.75rem"));
    }
}
