//! Dedicated Table renderer. Native `<table data-slot="table">`. Chrome in CSS.

use crate::cronus_ui_kit::{esc, texts};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let bound = crate::cronus_ui_data::rows();
    if !bound.is_empty() {
        let keys: Vec<String> = bound[0]
            .as_object()
            .map(|o| {
                o.keys()
                    .filter(|k| !k.starts_with('_'))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        let head = keys
            .iter()
            .map(|c| format!("<th>{}</th>", esc(c)))
            .collect::<Vec<_>>()
            .join("");
        let body = bound
            .iter()
            .map(|row| {
                let tds = keys
                    .iter()
                    .map(|k| {
                        let v = row
                            .get(k)
                            .map(|x| match x {
                                serde_json::Value::String(s) => esc(s),
                                other => esc(&other.to_string()),
                            })
                            .unwrap_or_default();
                        format!("<td>{v}</td>")
                    })
                    .collect::<Vec<_>>()
                    .join("");
                format!("<tr>{tds}</tr>")
            })
            .collect::<Vec<_>>()
            .join("");
        return format!(
            "<div data-slot=\"table\" data-animate><table><thead><tr>{head}</tr></thead><tbody>{body}</tbody></table></div>"
        );
    }
    let cols = texts(comp);
    let head = cols
        .iter()
        .map(|c| format!("<th>{c}</th>"))
        .collect::<Vec<_>>()
        .join("");
    let body = format!(
        "<tr>{}</tr>",
        cols.iter()
            .map(|_| "<td>—</td>".to_string())
            .collect::<Vec<_>>()
            .join("")
    );
    format!(
        "<div data-slot=\"table\" data-animate><table><thead><tr>{head}</tr></thead><tbody>{body}</tbody></table></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    #[test]
    fn native_table() {
        let html = render(&stub("table", "Name"));
        assert!(html.contains("<table"));
        assert!(html.contains("<th>"));
        assert!(html.contains("Name"));
        assert!(!html.contains("zinc-"));
        let interact = crate::cronus_ui_interact::render("table", &stub("table", "Name")).unwrap();
        assert_ne!(html, interact);
    }
}
