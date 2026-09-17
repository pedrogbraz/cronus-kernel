//! Dedicated Table renderer — mirrors React `Table` composed as caption +
//! header row + body rows + footer row.
//!
//! DOM: `<section data-slot="table-container" tabindex="0" aria-label="Table">`
//! (keyboard-scrollable region, like React) > `<table data-slot="table">` >
//! optional `<caption data-slot="table-caption">` (`caption:"…"`) >
//! `<thead data-slot="table-header">` / `<tbody data-slot="table-body">` /
//! optional `<tfoot data-slot="table-footer">` > `<tr data-slot="table-row">` >
//! `<th data-slot="table-head">` / `<td data-slot="table-cell">`.
//!
//! Data: bound rows (`crate::cronus_ui_data::rows()`, keys not starting with
//! `_` as columns) win. Otherwise `columns "…"` lines are the headers (config
//! `align:end`, and `font:medium|mono` for the cells, as classes) and the
//! `text` lines fill body rows of that width; `value` lines fill one footer
//! row (`span:N` sets `colspan`). Without `columns`, the choice texts apply:
//! with `columns:N` the first N are headers and the rest fill body rows of N;
//! without it every text is a header over one placeholder row.
use crate::cronus_ui_kit::{attr_nonempty, attr_num, content_texts, esc, texts};
use crate::parser::{ComponentItemNode, ComponentNode};

#[derive(Default, Clone)]
struct Col {
    head_class: Vec<&'static str>,
    cell_class: Vec<&'static str>,
}

fn class_attr(classes: &[&str]) -> String {
    if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    }
}

fn table(
    caption: Option<&str>,
    head: &[String],
    cols: &[Col],
    rows: &[Vec<String>],
    footer: &[(String, usize)],
) -> String {
    let col = |i: usize| cols.get(i).cloned().unwrap_or_default();
    let caption = caption
        .map(|c| format!("<caption data-slot=\"table-caption\">{c}</caption>"))
        .unwrap_or_default();
    let ths = head
        .iter()
        .enumerate()
        .map(|(i, h)| {
            format!(
                "<th data-slot=\"table-head\"{}>{h}</th>",
                class_attr(&col(i).head_class)
            )
        })
        .collect::<String>();
    let trs = rows
        .iter()
        .map(|r| {
            let tds = (0..head.len())
                .map(|i| {
                    let c = col(i);
                    let mut classes = c.head_class.clone();
                    classes.extend(c.cell_class.iter().copied());
                    format!(
                        "<td data-slot=\"table-cell\"{}>{}</td>",
                        class_attr(&classes),
                        r.get(i).map(String::as_str).unwrap_or("")
                    )
                })
                .collect::<String>();
            format!("<tr data-slot=\"table-row\">{tds}</tr>")
        })
        .collect::<String>();
    let tfoot = if footer.is_empty() {
        String::new()
    } else {
        let mut index = 0;
        let tds = footer
            .iter()
            .map(|(text, span)| {
                let c = col(index);
                index += span.max(&1);
                let colspan = if *span > 1 {
                    format!(" colspan=\"{span}\"")
                } else {
                    String::new()
                };
                let mut classes = c.head_class.clone();
                classes.extend(c.cell_class.iter().copied());
                format!(
                    "<td data-slot=\"table-cell\"{colspan}{}>{text}</td>",
                    class_attr(&classes)
                )
            })
            .collect::<String>();
        format!("<tfoot data-slot=\"table-footer\"><tr data-slot=\"table-row\">{tds}</tr></tfoot>")
    };
    format!(
        "<section data-slot=\"table-container\" tabindex=\"0\" aria-label=\"Table\"><table data-slot=\"table\">{caption}<thead data-slot=\"table-header\"><tr data-slot=\"table-row\">{ths}</tr></thead><tbody data-slot=\"table-body\">{trs}</tbody>{tfoot}</table></section>"
    )
}

fn col_of(item: &ComponentItemNode) -> Col {
    let mut c = Col::default();
    if item.config.get("align").is_some_and(|a| a == "end") {
        c.head_class.push("text-end");
    }
    match item.config.get("font").map(String::as_str) {
        Some("medium") => c.cell_class.push("font-medium"),
        Some("mono") => c.cell_class.push("font-mono"),
        _ => {}
    }
    c
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
        return table(None, &head, &[], &rows, &[]);
    }
    let caption = attr_nonempty(comp, "caption").map(esc);
    // `columns "Name"` lines are headers; the `text` lines fill rows under them.
    let column_items: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "columns" && !i.text.is_empty())
        .collect();
    if !column_items.is_empty() {
        let headers: Vec<String> = column_items.iter().map(|i| esc(&i.text)).collect();
        let cols: Vec<Col> = column_items.iter().map(|i| col_of(i)).collect();
        let body: Vec<String> = comp
            .items
            .iter()
            .filter(|i| matches!(i.item_type.as_str(), "text" | "item") && !i.text.is_empty())
            .map(|i| esc(&i.text))
            .collect();
        let mut rows: Vec<Vec<String>> =
            body.chunks(headers.len()).map(<[String]>::to_vec).collect();
        if rows.is_empty() {
            rows.push(vec!["—".to_string(); headers.len()]);
        }
        let footer: Vec<(String, usize)> = comp
            .items
            .iter()
            .filter(|i| i.item_type == "value" && !i.text.is_empty())
            .map(|i| {
                (
                    esc(&i.text),
                    i.config
                        .get("span")
                        .and_then(|s| s.trim().parse().ok())
                        .unwrap_or(1),
                )
            })
            .collect();
        return table(caption.as_deref(), &headers, &cols, &rows, &footer);
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
            table(caption.as_deref(), &cells[..n], &[], &rows, &[])
        }
        None => {
            let placeholder = vec![vec!["—".to_string(); cells.len()]];
            table(caption.as_deref(), &cells, &[], &placeholder, &[])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

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

    fn item(kind: &str, text: &str, pairs: &[(&str, &str)]) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
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
        assert!(!html.contains("table-caption"));
        assert!(!html.contains("table-footer"));
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

    /// Docs "Basic": caption, four `columns` (align/font config as classes),
    /// body rows from the texts and a `value` footer with `colspan`.
    #[test]
    fn caption_column_config_and_footer_match_docs() {
        let mut c = stub("table", "Invoices");
        c.props
            .insert("caption".into(), "A list of your recent invoices.".into());
        c.items = vec![
            item("columns", "Invoice", &[("font", "medium")]),
            item("columns", "Status", &[]),
            item("columns", "Method", &[]),
            item("columns", "Amount", &[("align", "end"), ("font", "mono")]),
            item("text", "INV-001", &[]),
            item("text", "Paid", &[]),
            item("text", "Credit Card", &[]),
            item("text", "$250.00", &[]),
            item("value", "Total", &[("span", "3")]),
            item("value", "$1,200.00", &[]),
        ];
        let html = render(&c);
        assert!(html.starts_with("<section data-slot=\"table-container\" tabindex=\"0\" aria-label=\"Table\"><table data-slot=\"table\"><caption data-slot=\"table-caption\">A list of your recent invoices.</caption><thead data-slot=\"table-header\"><tr data-slot=\"table-row\"><th data-slot=\"table-head\">Invoice</th><th data-slot=\"table-head\">Status</th><th data-slot=\"table-head\">Method</th><th data-slot=\"table-head\" class=\"text-end\">Amount</th></tr></thead>"));
        assert!(html.contains("<tbody data-slot=\"table-body\"><tr data-slot=\"table-row\"><td data-slot=\"table-cell\" class=\"font-medium\">INV-001</td><td data-slot=\"table-cell\">Paid</td><td data-slot=\"table-cell\">Credit Card</td><td data-slot=\"table-cell\" class=\"text-end font-mono\">$250.00</td></tr></tbody>"));
        assert!(html.ends_with("<tfoot data-slot=\"table-footer\"><tr data-slot=\"table-row\"><td data-slot=\"table-cell\" colspan=\"3\" class=\"font-medium\">Total</td><td data-slot=\"table-cell\" class=\"text-end font-mono\">$1,200.00</td></tr></tfoot></table></section>"));
        assert!(!html.contains("style="));
        let css = include_str!("cronus_ui_css/table.css");
        assert!(css.contains("[data-slot=\"table-caption\"] { margin-top: 1rem; font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg-tertiary); caption-side: bottom; }"));
        assert!(css.contains("[data-slot=\"table-footer\"] {\n  border-top: 1px solid var(--cronus-border); font-weight: 500;\n  background: color-mix(in oklch, var(--cronus-surface-inset) 50%, transparent);\n}"));
        assert!(css.contains("[data-slot=\"table\"] .font-mono { font-family: var(--cronus-font-mono, ui-monospace, monospace); font-variant-numeric: tabular-nums; }"));
    }

    #[test]
    fn chrome_matches_react_table() {
        let css = include_str!("cronus_ui_css/table.css");
        assert!(css.contains("[data-slot=\"table-container\"] {\n  position: relative; display: block; width: 100%; overflow-x: auto;"));
        assert!(css.contains("[data-slot=\"table-head\"] {\n  height: 2.5rem;"));
        assert!(!css.contains("[data-slot=\"table\"] table"));
    }
}
