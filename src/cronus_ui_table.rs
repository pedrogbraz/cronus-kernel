//! Dedicated Table renderer — mirrors React `Table` composed as header row +
//! body rows.
//!
//! DOM: `<section data-slot="table-container" tabindex="0" aria-label="Table">`
//! (keyboard-scrollable region, like React) > `<table data-slot="table">` >
//! `<thead data-slot="table-header">` / `<tbody data-slot="table-body">` >
//! `<tr data-slot="table-row">` > `<th data-slot="table-head">` /
//! `<td data-slot="table-cell">`.
//!
//! Data: bound rows (`crate::cronus_ui_data::rows()`, keys not starting with
//! `_` as columns) win. Otherwise the choice texts: with `columns:N` the first N
//! are headers and the rest fill body rows of N; without it every text is a
//! header over one placeholder row.

use crate::cronus_ui_kit::{attr_num, content_texts, esc, texts};
use crate::parser::ComponentNode;

fn table(head: &[String], rows: &[Vec<String>]) -> String {
    let ths = head
        .iter()
        .map(|h| format!("<th data-slot=\"table-head\">{h}</th>"))
        .collect::<String>();
    let trs = rows
        .iter()
        .map(|r| {
            let tds = (0..head.len())
                .map(|i| {
                    format!(
                        "<td data-slot=\"table-cell\">{}</td>",
                        r.get(i).map(String::as_str).unwrap_or("")
                    )
                })
                .collect::<String>();
            format!("<tr data-slot=\"table-row\">{tds}</tr>")
        })
        .collect::<String>();
    format!(
        "<section data-slot=\"table-container\" tabindex=\"0\" aria-label=\"Table\"><table data-slot=\"table\"><thead data-slot=\"table-header\"><tr data-slot=\"table-row\">{ths}</tr></thead><tbody data-slot=\"table-body\">{trs}</tbody></table></section>"
    )
}

pub fn render(comp: &ComponentNode) -> String {
    let bound = crate::cronus_ui_data::rows();
    if !bound.is_empty() {
        let keys: Vec<String> = bound[0]
            .as_object()
            .map(|o| o.keys().filter(|k| !k.starts_with('_')).cloned().collect())
            .unwrap_or_default();
        let rows = bound
            .iter()
            .map(|row| {
                keys.iter()
                    .map(|k| match row.get(k) {
                        Some(serde_json::Value::String(s)) => esc(s),
                        Some(other) => esc(&other.to_string()),
                        None => String::new(),
                    })
                    .collect()
            })
            .collect::<Vec<Vec<String>>>();
        let head = keys.iter().map(|k| esc(k)).collect::<Vec<_>>();
        return table(&head, &rows);
    }
    // `columns "Name"` lines are headers; the `text` lines fill rows under them.
    let headers: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "columns" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .collect();
    if !headers.is_empty() {
        let body: Vec<String> = comp
            .items
            .iter()
            .filter(|i| i.item_type == "text" && !i.text.is_empty())
            .map(|i| esc(&i.text))
            .collect();
        let mut rows: Vec<Vec<String>> =
            body.chunks(headers.len()).map(<[String]>::to_vec).collect();
        if rows.is_empty() {
            rows.push(vec!["—".to_string(); headers.len()]);
        }
        return table(&headers, &rows);
    }
    let cells = {
        let c = content_texts(comp);
        if c.is_empty() {
            texts(comp)
        } else {
            c
        }
    };
    match attr_num::<usize>(comp, "columns").filter(|n| *n > 0 && *n <= cells.len()) {
        Some(n) => {
            let rows = cells[n..]
                .chunks(n)
                .map(<[String]>::to_vec)
                .collect::<Vec<_>>();
            table(&cells[..n], &rows)
        }
        None => {
            let placeholder = vec![vec!["—".to_string(); cells.len()]];
            table(&cells, &placeholder)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn fixture() -> ComponentNode {
        let mut c = stub("table", "default");
        c.props.insert("columns".into(), "3".into());
        for t in [
            "Invoice", "Status", "Amount", "INV-001", "Paid", "$250.00", "INV-002", "Pending",
        ] {
            c.items.push(ComponentItemNode {
                item_type: "text".into(),
                text: t.into(),
                link: None,
                tone: None,
                config: Default::default(),
            });
        }
        c
    }

    #[test]
    fn columns_split_head_and_rows_like_react() {
        let html = render(&fixture());
        assert!(html.starts_with("<section data-slot=\"table-container\" tabindex=\"0\" aria-label=\"Table\"><table data-slot=\"table\"><thead data-slot=\"table-header\"><tr data-slot=\"table-row\"><th data-slot=\"table-head\">Invoice</th>"));
        assert_eq!(html.matches("data-slot=\"table-head\"").count(), 3);
        assert_eq!(html.matches("data-slot=\"table-row\"").count(), 3);
        assert_eq!(html.matches("data-slot=\"table-cell\"").count(), 6);
        assert!(html.contains("<td data-slot=\"table-cell\">Pending</td><td data-slot=\"table-cell\"></td></tr></tbody>"));
        assert!(!html.contains(">default<"));
        assert!(!html.contains("data-animate"));
    }

    #[test]
    fn without_columns_all_texts_are_headers() {
        let html = render(&stub("table", "Name"));
        assert!(html.contains("<th data-slot=\"table-head\">Name</th>"));
        assert!(html.contains("<td data-slot=\"table-cell\">—</td>"));
        let mut c = fixture();
        c.props.insert("columns".into(), "99".into());
        assert_eq!(render(&c).matches("data-slot=\"table-head\"").count(), 8);
    }

    #[test]
    fn chrome_matches_react_table() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"table-container\"] {\n  position: relative; display: block; width: 100%; overflow-x: auto;"));
        assert!(css.contains("[data-slot=\"table-head\"] {\n  height: 2.5rem;"));
        assert!(!css.contains("[data-slot=\"table\"] table"));
    }
}
