use std::collections::HashMap;
use crate::parser::SectionNode;

/// Renders a generic data table from a "table" section.
/// Items with `column:true` are column headers. All other items are data rows.
/// Row props map to column names (lowercase).
pub fn render_data_table(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("");
    let table_id = format!("cronus-tbl-{}", title.replace(' ', "-").to_lowercase());

    // Separate column headers from data rows
    let columns: Vec<&str> = section.items.iter()
        .filter(|item| item.get("column").map(|v| v == "true").unwrap_or(false))
        .map(|item| item.get("title").map(|s| s.as_str()).unwrap_or(""))
        .collect();

    let rows: Vec<&HashMap<String, String>> = section.items.iter()
        .filter(|item| !item.get("column").map(|v| v == "true").unwrap_or(false))
        .filter(|item| {
            let t = item.get("_type").map(|s| s.as_str()).unwrap_or("item");
            t == "item" || t.is_empty()
        })
        .collect();

    let row_count = rows.len();
    let mut html = String::new();

    // Sort + bulk select JS (once per page)
    html.push_str(&format!(
        r##"<script>
function cronusSortTable(tid,ci){{var t=document.getElementById(tid);if(!t)return;var tb=t.querySelector('tbody');var rs=Array.from(tb.rows);var d=t.dataset.sortDir==='asc'?'desc':'asc';t.dataset.sortDir=d;rs.sort(function(a,b){{var av=a.cells[ci].textContent.trim();var bv=b.cells[ci].textContent.trim();var r=av.localeCompare(bv,undefined,{{numeric:true}});return d==='asc'?r:-r;}});rs.forEach(function(r){{tb.appendChild(r);}});t.querySelectorAll('thead th[data-sort] .sa').forEach(function(s,i){{s.textContent=i+1===ci?(d==='asc'?'▲':'▼'):'';}});}}
function cronusBulkSelect(tid,cb){{var t=document.getElementById(tid);if(!t)return;t.querySelectorAll('tbody input[type=checkbox]').forEach(function(c){{c.checked=cb.checked;}});}}
</script>"##
    ));

    // Table wrapper
    html.push_str(&format!(
        r#"<div style="border:1px solid #e5e7eb;border-radius:12px;overflow:hidden;background:#fff" id="{table_id}">"#
    ));

    // Title bar
    if !title.is_empty() {
        html.push_str(&format!(
            r#"<div style="padding:16px 16px 0;font-size:16px;font-weight:600;color:#1a1c1c">{title}</div>"#
        ));
    }

    html.push_str(r#"<table style="width:100%;border-collapse:collapse;font-size:14px">"#);

    // ── Header ──
    html.push_str(r#"<thead><tr style="border-bottom:1px solid #e5e7eb">"#);
    html.push_str(&format!(
        r##"<th style="width:40px;padding:12px 16px;text-align:left"><input type="checkbox" onclick="cronusBulkSelect('{table_id}',this)" style="width:16px;height:16px;cursor:pointer;accent-color:#000"></th>"##
    ));
    for (i, col) in columns.iter().enumerate() {
        html.push_str(&format!(
            r##"<th data-sort="true" onclick="cronusSortTable('{table_id}',{})" style="padding:12px 16px;text-align:left;font-size:11px;font-weight:600;text-transform:uppercase;letter-spacing:0.05em;color:#71717a;cursor:pointer;user-select:none">{col} <span class="sa" style="font-size:10px"></span></th>"##,
            i + 1
        ));
    }
    html.push_str("</tr></thead>");

    // ── Body ──
    html.push_str("<tbody>");
    for (idx, row) in rows.iter().enumerate() {
        let row_title = row.get("title").map(|s| s.as_str()).unwrap_or("");
        let alt_bg = if idx % 2 == 1 { "rgba(250,250,250,0.3)" } else { "transparent" };

        html.push_str(&format!(
            r#"<tr style="border-bottom:1px solid #f3f4f6;background:{alt_bg};transition:background 0.1s" onmouseover="this.style.background='rgba(245,245,245,0.5)'" onmouseout="this.style.background='{alt_bg}'">"#
        ));
        html.push_str(r#"<td style="padding:12px 16px"><input type="checkbox" style="width:16px;height:16px;cursor:pointer;accent-color:#000"></td>"#);

        for (ci, col) in columns.iter().enumerate() {
            let col_key = col.to_lowercase();

            // First column uses row title as value
            let cell_value = if ci == 0 {
                row_title
            } else {
                row.get(&col_key).map(|s| s.as_str()).unwrap_or("")
            };

            // Check for badge color (on status column)
            let badge_color = if col_key == "status" {
                row.get("badge").map(|s| s.as_str()).unwrap_or("")
            } else {
                ""
            };

            html.push_str(r#"<td style="padding:12px 16px;color:#374151">"#);

            if !badge_color.is_empty() {
                let (dot_bg, txt_color) = match badge_color {
                    "green" => ("#10b981", "#047857"),
                    "yellow" => ("#f59e0b", "#b45309"),
                    "red" => ("#ef4444", "#b91c1c"),
                    "blue" => ("#3b82f6", "#1d4ed8"),
                    _ => ("#71717a", "#374151"),
                };
                html.push_str(&format!(
                    r#"<span style="display:inline-flex;align-items:center;gap:6px"><span style="width:8px;height:8px;border-radius:50%;background:{dot_bg};flex-shrink:0"></span><span style="color:{txt_color};font-weight:500">{cell_value}</span></span>"#
                ));
            } else {
                html.push_str(cell_value);
            }

            html.push_str("</td>");
        }
        html.push_str("</tr>");
    }
    html.push_str("</tbody>");

    // ── Footer ──
    html.push_str(&format!(
        r#"<tfoot><tr><td colspan="{}" style="padding:12px 16px;font-size:13px;color:#71717a;border-top:1px solid #e5e7eb">Showing {row_count} items</td></tr></tfoot>"#,
        columns.len() + 1
    ));

    html.push_str("</table></div>");
    html
}

/// Renders pagination controls.
/// Config keys: total, per_page, current
pub fn render_pagination(section: &SectionNode) -> String {
    let total: u32 = section.config.get("total").and_then(|v| v.parse().ok()).unwrap_or(0);
    let per_page: u32 = section.config.get("per_page").and_then(|v| v.parse().ok()).unwrap_or(10);
    let current: u32 = section.config.get("current").and_then(|v| v.parse().ok()).unwrap_or(1);
    let total_pages = if total == 0 { 1 } else { (total + per_page - 1) / per_page };

    let btn = "display:inline-flex;align-items:center;justify-content:center;min-width:36px;height:36px;padding:0 12px;font-size:14px;font-weight:500;border-radius:6px;text-decoration:none;transition:background 0.1s;cursor:pointer;";
    let btn_active = format!("{btn}background:#18181b;color:#fff;border:1px solid #18181b;");
    let btn_normal = format!("{btn}background:#fff;color:#374151;border:1px solid #e5e7eb;");
    let btn_off = format!("{btn}background:#fff;color:#d4d4d8;border:1px solid #e5e7eb;opacity:0.4;cursor:not-allowed;pointer-events:none;");

    let hover_on = r#"onmouseover="this.style.background='#f5f5f5'" onmouseout="this.style.background='#fff'""#;

    let mut html = String::new();
    html.push_str(r#"<nav style="display:flex;justify-content:space-between;align-items:center;padding-top:16px;border-top:1px solid #e5e7eb">"#);

    // Left: page info
    html.push_str(&format!(
        r#"<span style="font-size:13px;color:#71717a">Page {current} of {total_pages}</span>"#
    ));

    // Right: buttons
    html.push_str(r#"<div style="display:flex;align-items:center;gap:4px">"#);

    // Previous
    if current <= 1 {
        html.push_str(&format!(r#"<span style="{btn_off}">« Previous</span>"#));
    } else {
        html.push_str(&format!(r#"<a style="{btn_normal}" {hover_on}>« Previous</a>"#));
    }

    // Page numbers — max 7 visible
    let (start, end) = if total_pages <= 7 {
        (1, total_pages)
    } else if current <= 4 {
        (1, 5)
    } else if current >= total_pages - 3 {
        (total_pages - 4, total_pages)
    } else {
        (current - 2, current + 2)
    };

    if start > 1 {
        html.push_str(&format!(r#"<a style="{btn_normal}" {hover_on}>1</a>"#));
        if start > 2 {
            html.push_str(r#"<span style="padding:0 4px;color:#71717a">…</span>"#);
        }
    }

    for page in start..=end {
        if page == current {
            html.push_str(&format!(r#"<a style="{btn_active}">{page}</a>"#));
        } else {
            html.push_str(&format!(r#"<a style="{btn_normal}" {hover_on}>{page}</a>"#));
        }
    }

    if end < total_pages {
        if end < total_pages - 1 {
            html.push_str(r#"<span style="padding:0 4px;color:#71717a">…</span>"#);
        }
        html.push_str(&format!(r#"<a style="{btn_normal}" {hover_on}>{total_pages}</a>"#));
    }

    // Next
    if current >= total_pages {
        html.push_str(&format!(r#"<span style="{btn_off}">Next »</span>"#));
    } else {
        html.push_str(&format!(r#"<a style="{btn_normal}" {hover_on}>Next »</a>"#));
    }

    html.push_str("</div></nav>");
    html
}

/// Renders a filters toolbar.
/// Each item defines a filter: type:select|date|search, options:"A,B,C", placeholder:"..."
pub fn render_filters_toolbar(section: &SectionNode) -> String {
    let mut html = String::new();
    html.push_str(r#"<div style="display:flex;gap:12px;margin-bottom:16px;align-items:center">"#);

    let input_base = "padding:8px 12px;font-size:14px;border:1px solid #e5e7eb;border-radius:8px;background:#fff;color:#374151;outline:none;transition:border-color 0.15s;";
    let focus = r#"onfocus="this.style.borderColor='#000'" onblur="this.style.borderColor='#e5e7eb'""#;

    for item in &section.items {
        let label = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let filter_type = item.get("type").map(|s| s.as_str()).unwrap_or("text");
        let placeholder = item.get("placeholder").map(|s| s.as_str()).unwrap_or("");
        let options = item.get("options").map(|s| s.as_str()).unwrap_or("");

        match filter_type {
            "select" => {
                html.push_str(&format!(
                    r#"<select style="{input_base}cursor:pointer" {focus}>"#
                ));
                for opt in options.split(',') {
                    let opt = opt.trim();
                    html.push_str(&format!(r#"<option value="{opt}">{opt}</option>"#));
                }
                html.push_str("</select>");
            }
            "date" => {
                html.push_str(&format!(
                    r#"<input type="date" style="{input_base}cursor:pointer" {focus}>"#
                ));
            }
            "search" => {
                let ph = if placeholder.is_empty() { format!("Search {}...", label.to_lowercase()) } else { placeholder.to_string() };
                html.push_str(&format!(
                    r#"<div style="display:flex;align-items:center;gap:8px;{input_base}flex:1"><span class="material-symbols-outlined" style="font-size:18px;color:#71717a">search</span><input type="text" placeholder="{ph}" style="border:none;outline:none;background:transparent;flex:1;font-size:14px;color:#374151"></div>"#
                ));
            }
            _ => {
                let ph = if placeholder.is_empty() { label } else { placeholder };
                html.push_str(&format!(
                    r#"<input type="text" placeholder="{ph}" style="{input_base}" {focus}>"#
                ));
            }
        }
    }

    html.push_str("</div>");
    html
}
