use std::collections::HashMap;
use serde_json::Value as JsonValue;
use crate::parser::SectionNode;

/// Renders a generic data table from a "table" section.
/// Items with `column:true` are column headers. All other items are data rows.
/// Row props map to column names (lowercase).
/// When bound_data contains Rows, generates table body from database rows instead of static items.
pub fn render_data_table(section: &SectionNode, bound_data: &crate::binding::ResolvedData) -> String {
    let title = section.title.as_deref().unwrap_or("");
    let table_id = format!("cronus-tbl-{}", title.replace(' ', "-").to_lowercase());

    // Column headers: from items with column:true, or from config "columns" string
    let columns_from_items: Vec<&str> = section.items.iter()
        .filter(|item| item.get("column").map(|v| v == "true").unwrap_or(false))
        .map(|item| item.get("title").map(|s| s.as_str()).unwrap_or(""))
        .collect();

    // Parse columns from config string (pipe-separated or comma-separated)
    let config_columns_owned: Vec<String> = if columns_from_items.is_empty() {
        if let Some(cols_str) = section.config.get("columns") {
            let separator = if cols_str.contains('|') { '|' } else { ',' };
            cols_str.split(separator)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let columns: Vec<&str> = if !columns_from_items.is_empty() {
        columns_from_items
    } else {
        config_columns_owned.iter().map(|s| s.as_str()).collect()
    };

    // Check if we have bound database rows
    let use_bound = matches!(bound_data, crate::binding::ResolvedData::Rows(r) if !r.is_empty());

    let static_rows: Vec<&HashMap<String, String>> = if use_bound {
        Vec::new()
    } else {
        section.items.iter()
            .filter(|item| !item.get("column").map(|v| v == "true").unwrap_or(false))
            .filter(|item| {
                let t = item.get("_type").map(|s| s.as_str()).unwrap_or("item");
                t == "item" || t == "row" || t.is_empty()
            })
            .collect()
    };

    let row_count = if let crate::binding::ResolvedData::Rows(rows) = bound_data {
        if !rows.is_empty() { rows.len() } else { static_rows.len() }
    } else {
        static_rows.len()
    };
    let mut html = String::new();

    // Search and pagination config
    let search_fields = section.config.get("search").cloned().unwrap_or_default();
    let has_search = section.config.contains_key("search");
    let paginate_per: usize = section.config.get("paginate")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let has_paginate = paginate_per > 0;

    // Sort + bulk select + search + pagination JS (once per page)
    html.push_str(&format!(
        r##"<script>
function cronusSortTable(tid,ci){{var t=document.getElementById(tid);if(!t)return;var tb=t.querySelector('tbody');var rs=Array.from(tb.rows);var d=t.dataset.sortDir==='asc'?'desc':'asc';t.dataset.sortDir=d;rs.sort(function(a,b){{var av=a.cells[ci].textContent.trim();var bv=b.cells[ci].textContent.trim();var r=av.localeCompare(bv,undefined,{{numeric:true}});return d==='asc'?r:-r;}});rs.forEach(function(r){{tb.appendChild(r);}});t.querySelectorAll('thead th[data-sort] .sa').forEach(function(s,i){{s.textContent=i+1===ci?(d==='asc'?'▲':'▼'):'';}});}}
function cronusBulkSelect(tid,cb){{var t=document.getElementById(tid);if(!t)return;t.querySelectorAll('tbody input[type=checkbox]').forEach(function(c){{c.checked=cb.checked;}});}}
function cronusSearch(tid){{var w=document.getElementById(tid);if(!w)return;var input=w.querySelector('[data-cronus-search]');var tbl=w.querySelector('table');if(!input||!tbl)return;input.addEventListener('input',function(){{var term=this.value.toLowerCase();tbl.querySelectorAll('tbody tr').forEach(function(tr){{var txt=Array.from(tr.cells).map(function(c){{return c.textContent.toLowerCase();}}).join(' ');tr.style.display=txt.indexOf(term)>=0?'':'none';}});if(w._cronusPaginate)w._cronusPaginate(0);}});}}
function cronusPaginate(tid,perPage){{var w=document.getElementById(tid);if(!w)return;var tbl=w.querySelector('table');if(!tbl)return;var info=w.querySelector('[data-page-info]');var prevBtn=w.querySelector('[data-cronus-page="prev"]');var nextBtn=w.querySelector('[data-cronus-page="next"]');var page=0;function show(p){{var rows=Array.from(tbl.querySelectorAll('tbody tr')).filter(function(r){{return r.style.display!=='none'||!r.style.display;}});var visible=Array.from(tbl.querySelectorAll('tbody tr')).filter(function(r){{return r.style.display!=='none';}});var total=visible.length;var tp=Math.max(1,Math.ceil(total/perPage));if(p<0)p=0;if(p>=tp)p=tp-1;page=p;visible.forEach(function(r,i){{r.style.display=(i>=p*perPage&&i<(p+1)*perPage)?'':'none';}});if(info)info.textContent='Showing '+(total===0?'0':((p*perPage+1)+'-'+Math.min((p+1)*perPage,total)))+' of '+total;if(prevBtn)prevBtn.disabled=p<=0;if(nextBtn)nextBtn.disabled=p>=tp-1;}}w._cronusPaginate=show;show(0);if(prevBtn)prevBtn.addEventListener('click',function(){{show(page-1);}});if(nextBtn)nextBtn.addEventListener('click',function(){{show(page+1);}});}}
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

    // Search bar
    if has_search {
        html.push_str(&format!(
            r#"<div style="padding:12px 16px 0"><input type="text" placeholder="Search..." data-cronus-search="{table_id}" data-search-fields="{search_fields}" style="padding:8px 16px;background:#f9fafb;border:1px solid #e5e7eb;border-radius:8px;color:#374151;width:300px;font-size:14px;outline:none;transition:border-color 0.15s" onfocus="this.style.borderColor='#000'" onblur="this.style.borderColor='#e5e7eb'"></div>"#
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

    if let crate::binding::ResolvedData::Rows(bound_rows) = bound_data {
        if !bound_rows.is_empty() {
            // === Render from database rows ===
            for (idx, row) in bound_rows.iter().enumerate() {
                let alt_bg = if idx % 2 == 1 { "rgba(250,250,250,0.3)" } else { "transparent" };

                html.push_str(&format!(
                    r#"<tr style="border-bottom:1px solid #f3f4f6;background:{alt_bg};transition:background 0.1s" onmouseover="this.style.background='rgba(245,245,245,0.5)'" onmouseout="this.style.background='{alt_bg}'">"#
                ));
                html.push_str(r#"<td style="padding:12px 16px"><input type="checkbox" style="width:16px;height:16px;cursor:pointer;accent-color:#000"></td>"#);

                for col in &columns {
                    let col_key = col.to_lowercase();
                    let cell_value = row.get(&col_key)
                        .and_then(|v| v.as_str())
                        .or_else(|| row.get(&col_key).and_then(|v| if v.is_number() { None } else { None }))
                        .unwrap_or("");
                    // For numeric values, convert to string
                    let cell_display = if cell_value.is_empty() {
                        if let Some(v) = row.get(&col_key) {
                            match v {
                                JsonValue::Number(n) => n.to_string(),
                                JsonValue::Bool(b) => b.to_string(),
                                _ => String::new(),
                            }
                        } else {
                            String::new()
                        }
                    } else {
                        cell_value.to_string()
                    };

                    html.push_str(r#"<td style="padding:12px 16px;color:#374151">"#);
                    html.push_str(&crate::security::html_escape(&cell_display));
                    html.push_str("</td>");
                }
                html.push_str("</tr>");
            }
        }
    }

    if !use_bound {
        // === Fallback: render from static section items ===
        for (idx, row) in static_rows.iter().enumerate() {
            let row_title = row.get("title").map(|s| s.as_str()).unwrap_or("");
            let alt_bg = if idx % 2 == 1 { "rgba(250,250,250,0.3)" } else { "transparent" };

            html.push_str(&format!(
                r#"<tr style="border-bottom:1px solid #f3f4f6;background:{alt_bg};transition:background 0.1s" onmouseover="this.style.background='rgba(245,245,245,0.5)'" onmouseout="this.style.background='{alt_bg}'">"#
            ));
            html.push_str(r#"<td style="padding:12px 16px"><input type="checkbox" style="width:16px;height:16px;cursor:pointer;accent-color:#000"></td>"#);

            for (_ci, col) in columns.iter().enumerate() {
                let col_key = col.to_lowercase();

                // Look up cell value by column key; fall back to row title for first match
                let cell_value = row.get(&col_key).map(|s| s.as_str())
                    .unwrap_or_else(|| {
                        // If row title is non-empty and no explicit column value, use title
                        if !row_title.is_empty() && _ci == 0 { row_title } else { "" }
                    });

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
                        r#"<span style="display:inline-flex;align-items:center;gap:6px"><span style="width:8px;height:8px;border-radius:50%;background:{dot_bg};flex-shrink:0"></span><span style="color:{txt_color};font-weight:500">{}</span></span>"#,
                        crate::security::html_escape(cell_value)
                    ));
                } else {
                    html.push_str(&crate::security::html_escape(cell_value));
                }

                html.push_str("</td>");
            }
            html.push_str("</tr>");
        }
    }

    html.push_str("</tbody>");

    // ── Footer ──
    if has_paginate {
        html.push_str("</table>");
        // Pagination controls
        html.push_str(&format!(
            r#"<div style="display:flex;justify-content:space-between;align-items:center;padding:12px 16px;border-top:1px solid #e5e7eb"><span data-page-info style="font-size:13px;color:#71717a">Showing 1-{paginate_per} of {row_count}</span><div style="display:flex;gap:4px"><button data-cronus-page="prev" style="padding:6px 14px;font-size:13px;border:1px solid #e5e7eb;border-radius:6px;background:#fff;color:#374151;cursor:pointer" onmouseover="this.style.background='#f5f5f5'" onmouseout="this.style.background='#fff'">Previous</button><button data-cronus-page="next" style="padding:6px 14px;font-size:13px;border:1px solid #e5e7eb;border-radius:6px;background:#fff;color:#374151;cursor:pointer" onmouseover="this.style.background='#f5f5f5'" onmouseout="this.style.background='#fff'">Next</button></div></div>"#
        ));
    } else {
        html.push_str(&format!(
            r#"<tfoot><tr><td colspan="{}" style="padding:12px 16px;font-size:13px;color:#71717a;border-top:1px solid #e5e7eb">Showing {row_count} items</td></tr></tfoot>"#,
            columns.len() + 1
        ));
        html.push_str("</table>");
    }

    // Initialization script for search + pagination
    let mut init_js = String::new();
    if has_search {
        init_js.push_str(&format!("cronusSearch('{table_id}');"));
    }
    if has_paginate {
        init_js.push_str(&format!("cronusPaginate('{table_id}',{paginate_per});"));
    }
    if !init_js.is_empty() {
        html.push_str(&format!(r#"<script>document.addEventListener('DOMContentLoaded',function(){{{init_js}}});</script>"#));
    }

    html.push_str("</div>");
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

/// Renders a dark-themed data table with static rows from .cronus config.
/// Used when `style:dark` is set and items contain row data (title starting with "#" or config keys like client, value, status).
pub fn render_data_table_dark(section: &SectionNode, bound_data: &crate::binding::ResolvedData) -> String {
    let title = section.title.as_deref().unwrap_or("");
    let search_placeholder = section.config.get("search").map(|s| s.as_str()).unwrap_or("Search...");
    let footer_link = section.config.get("footer_link").map(|s| s.as_str());

    // Parse columns from config
    let columns: Vec<String> = if let Some(cols_str) = section.config.get("columns") {
        let separator = if cols_str.contains('|') { '|' } else { ',' };
        cols_str.split(separator)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        Vec::new()
    };

    // Collect static rows: items that have row-like data
    let static_rows: Vec<&HashMap<String, String>> = section.items.iter()
        .filter(|item| {
            let t = item.get("_type").map(|s| s.as_str()).unwrap_or("item");
            (t == "item" || t == "row") && (
                item.get("title").map(|s| s.starts_with('#')).unwrap_or(false)
                || item.get("client").is_some()
                || item.get("value").is_some()
                || item.get("status").is_some()
            )
        })
        .collect();

    let mut html = String::new();

    // Container
    html.push_str(r#"<div class="anim-slide-up" style="background:#1b1b1b;border:0.5px solid rgba(76,69,70,0.15);border-radius:12px;overflow:hidden">"#);

    // Header: title + search
    html.push_str(r#"<div style="display:flex;justify-content:space-between;align-items:center;padding:24px 32px">"#);
    if !title.is_empty() {
        html.push_str(&format!(
            r#"<h3 style="font-size:18px;font-weight:600;margin:0;color:#e2e2e2">{title}</h3>"#
        ));
    }
    html.push_str(&format!(
        r#"<input type="text" placeholder="{search_placeholder}" style="padding:8px 16px;background:#0e0e0e;border:none;border-radius:8px;font-size:12px;color:rgba(226,226,226,0.6);outline:none;width:220px">"#
    ));
    html.push_str("</div>");

    // Table
    html.push_str(r#"<table style="width:100%;border-collapse:collapse">"#);

    // Column headers
    html.push_str(r#"<thead><tr style="background:rgba(14,14,14,0.5)">"#);
    for col in &columns {
        html.push_str(&format!(
            r#"<th style="padding:12px 32px;text-align:left;font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:0.15em;color:rgba(226,226,226,0.4)">{col}</th>"#
        ));
    }
    html.push_str("</tr></thead>");

    // Body
    html.push_str("<tbody>");
    let avatar_colors = ["#3b82f6", "#8b5cf6", "#ef4444", "#10b981", "#f59e0b", "#ec4899"];

    // Render from bound DB rows when available and no static rows
    if static_rows.is_empty() {
        if let crate::binding::ResolvedData::Rows(bound_rows) = bound_data {
            for (idx, row) in bound_rows.iter().enumerate() {
                let delay = format!("{:.2}", 0.05 + idx as f64 * 0.06);
                html.push_str(&format!(
                    r#"<tr class="anim-row" style="border-bottom:1px solid rgba(76,69,70,0.05);transition:background 0.15s;animation-delay:{delay}s" onmouseover="this.style.background='#1f1f1f'" onmouseout="this.style.background='transparent'">"#,
                    delay = delay
                ));
                for (ci, col) in columns.iter().enumerate() {
                    let col_key = col.to_lowercase().replace(' ', "_");
                    let cell_value = row.get(&col_key)
                        .or_else(|| row.get(&col.to_lowercase()))
                        .map(|v| match v {
                            JsonValue::String(s) => s.clone(),
                            JsonValue::Number(n) => n.to_string(),
                            JsonValue::Bool(b) => b.to_string(),
                            _ => String::new(),
                        })
                        .unwrap_or_default();

                    html.push_str(r#"<td style="padding:16px 32px">"#);

                    if col_key == "status" {
                        let status_lower = cell_value.to_lowercase();
                        let (badge_bg, badge_color, dot_color) = match status_lower.as_str() {
                            "live" | "fulfilled" | "completed" | "active" | "success" => ("rgba(16,185,129,0.12)", "#10b981", "#10b981"),
                            "rolling" | "processing" | "in_progress" | "in progress" => ("rgba(59,130,246,0.12)", "#3b82f6", "#3b82f6"),
                            "pending" | "waiting" | "draft" => ("rgba(113,113,122,0.12)", "#71717a", "#71717a"),
                            "failed" | "cancelled" | "rejected" => ("rgba(239,68,68,0.12)", "#ef4444", "#ef4444"),
                            "blocked" => ("rgba(239,68,68,0.12)", "#ef4444", "#ef4444"),
                            "throttled" | "flagged" => ("rgba(245,158,11,0.12)", "#f59e0b", "#f59e0b"),
                            "critical" => ("rgba(239,68,68,0.15)", "#ef4444", "#ef4444"),
                            "high" => ("rgba(249,115,22,0.12)", "#f97316", "#f97316"),
                            "medium" | "warning" => ("rgba(245,158,11,0.12)", "#f59e0b", "#f59e0b"),
                            "low" | "info" => ("rgba(59,130,246,0.12)", "#3b82f6", "#3b82f6"),
                            _ => ("rgba(113,113,122,0.12)", "#71717a", "#71717a"),
                        };
                        let is_active = matches!(status_lower.as_str(), "live" | "rolling" | "processing" | "active");
                        let dot_anim = if is_active { "animation:pulse 2s ease-in-out infinite;" } else { "" };
                        let escaped_val = crate::security::html_escape(&cell_value);
                        html.push_str(&format!(
                            r#"<span style="display:inline-flex;align-items:center;gap:6px;padding:4px 12px;font-size:11px;font-weight:600;border-radius:999px;background:{badge_bg};color:{badge_color}"><span style="width:6px;height:6px;border-radius:50%;background:{dot_color};flex-shrink:0;{dot_anim}"></span>{escaped_val}</span>"#
                        ));
                    } else if ci == 0 {
                        let initials: String = cell_value.chars()
                            .filter(|c| c.is_alphabetic())
                            .take(2)
                            .collect::<String>()
                            .to_uppercase();
                        let avatar_bg = avatar_colors[idx % avatar_colors.len()];
                        let escaped_val = crate::security::html_escape(&cell_value);
                        html.push_str(&format!(
                            r#"<div style="display:flex;align-items:center;gap:12px"><div style="width:32px;height:32px;border-radius:50%;background:{avatar_bg};display:flex;align-items:center;justify-content:center;font-size:11px;font-weight:700;color:#fff;flex-shrink:0">{initials}</div><span style="font-size:13px;color:#e2e2e2">{escaped_val}</span></div>"#
                        ));
                    } else {
                        html.push_str(&format!(
                            r#"<span style="font-size:13px;color:rgba(226,226,226,0.8)">{}</span>"#,
                            crate::security::html_escape(&cell_value)
                        ));
                    }

                    html.push_str("</td>");
                }
                html.push_str("</tr>");
            }
        }
    }

    for (idx, row) in static_rows.iter().enumerate() {
        let row_title = row.get("title").map(|s| s.as_str()).unwrap_or("");
        let delay = format!("{:.2}", 0.05 + idx as f64 * 0.06);

        html.push_str(&format!(
            r#"<tr class="anim-row" style="border-bottom:1px solid rgba(76,69,70,0.05);transition:background 0.15s;animation-delay:{delay}s" onmouseover="this.style.background='#1f1f1f'" onmouseout="this.style.background='transparent'">"#,
            delay = delay
        ));

        for (ci, col) in columns.iter().enumerate() {
            let col_key = col.to_lowercase().replace(' ', "_");

            let cell_value = row.get(&col_key)
                .or_else(|| row.get(&col.to_lowercase()))
                .map(|s| s.as_str())
                .unwrap_or_else(|| {
                    if ci == 0 && !row_title.is_empty() { row_title } else { "" }
                });

            html.push_str(r#"<td style="padding:16px 32px">"#);

            if col_key == "client" {
                let initials: String = cell_value.chars()
                    .filter(|c| c.is_alphabetic())
                    .take(2)
                    .collect::<String>()
                    .to_uppercase();
                let avatar_bg = avatar_colors[idx % avatar_colors.len()];
                let escaped_val = crate::security::html_escape(cell_value);
                html.push_str(&format!(
                    r#"<div style="display:flex;align-items:center;gap:12px"><div style="width:32px;height:32px;border-radius:50%;background:{avatar_bg};display:flex;align-items:center;justify-content:center;font-size:11px;font-weight:700;color:#fff;flex-shrink:0">{initials}</div><span style="font-size:13px;color:#e2e2e2">{escaped_val}</span></div>"#
                ));
            } else if col_key == "status" {
                let status_lower = cell_value.to_lowercase();
                let (badge_bg, badge_color, dot_color) = match status_lower.as_str() {
                    "live" | "fulfilled" | "completed" | "active" | "success" => ("rgba(16,185,129,0.12)", "#10b981", "#10b981"),
                    "rolling" | "processing" | "in_progress" | "in progress" => ("rgba(59,130,246,0.12)", "#3b82f6", "#3b82f6"),
                    "pending" | "waiting" | "draft" => ("rgba(113,113,122,0.12)", "#71717a", "#71717a"),
                    "failed" | "cancelled" | "rejected" | "blocked" => ("rgba(239,68,68,0.12)", "#ef4444", "#ef4444"),
                    "throttled" | "flagged" | "warning" => ("rgba(245,158,11,0.12)", "#f59e0b", "#f59e0b"),
                    "critical" => ("rgba(239,68,68,0.15)", "#ef4444", "#ef4444"),
                    "high" => ("rgba(249,115,22,0.12)", "#f97316", "#f97316"),
                    "medium" => ("rgba(245,158,11,0.12)", "#f59e0b", "#f59e0b"),
                    "info" | "low" => ("rgba(59,130,246,0.12)", "#3b82f6", "#3b82f6"),
                    _ => ("rgba(113,113,122,0.12)", "#71717a", "#71717a"),
                };
                let is_active = matches!(status_lower.as_str(), "live" | "rolling" | "processing" | "active");
                let dot_anim = if is_active { "animation:pulse 2s ease-in-out infinite;" } else { "" };
                html.push_str(&format!(
                    r#"<span style="display:inline-flex;align-items:center;gap:6px;padding:4px 12px;font-size:11px;font-weight:600;border-radius:999px;background:{badge_bg};color:{badge_color}"><span style="width:6px;height:6px;border-radius:50%;background:{dot_color};flex-shrink:0;{dot_anim}"></span>{}</span>"#,
                    crate::security::html_escape(cell_value)
                ));
            } else if col_key == "action" {
                html.push_str(
                    r#"<button style="background:none;border:none;color:rgba(226,226,226,0.4);font-size:18px;cursor:pointer;padding:4px 8px;border-radius:6px;transition:background 0.15s" onmouseover="this.style.background='rgba(226,226,226,0.08)'" onmouseout="this.style.background='none'">&#x2026;</button>"#
                );
            } else {
                html.push_str(&format!(
                    r#"<span style="font-size:13px;color:rgba(226,226,226,0.8)">{}</span>"#,
                    crate::security::html_escape(cell_value)
                ));
            }

            html.push_str("</td>");
        }
        html.push_str("</tr>");
    }
    html.push_str("</tbody>");
    html.push_str("</table>");

    // Footer link
    if let Some(link_text) = footer_link {
        html.push_str(&format!(
            r##"<div style="padding:20px 32px;text-align:center;border-top:1px solid rgba(76,69,70,0.08)"><a href="#" style="font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:0.15em;color:rgba(226,226,226,0.4);text-decoration:none;transition:color 0.15s" onmouseover="this.style.color='rgba(226,226,226,0.7)'" onmouseout="this.style.color='rgba(226,226,226,0.4)'">{link_text}</a></div>"##
        ));
    }

    html.push_str("</div>");
    html
}
