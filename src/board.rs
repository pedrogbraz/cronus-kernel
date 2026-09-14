use std::collections::HashMap;
use crate::parser::SectionNode;

// ---------------------------------------------------------------------------
// Kanban Board
// ---------------------------------------------------------------------------

struct KanbanColumn {
    name: String,
    color: String,
    cards: Vec<KanbanCard>,
}

struct KanbanCard {
    name: String,
    assignee: String,
    priority: String,
    label: String,
}

/// Renders a kanban board from a "kanban" section.
///
/// Items with `column:true` are column headers; subsequent items
/// are cards belonging to that column until the next column header.
/// When bound_data contains Rows, generates cards from database rows.
/// DB rows are placed into the first column, or matched by a "status"/"column" field.
pub fn render_kanban(section: &SectionNode, bound_data: &crate::binding::ResolvedData) -> String {
    let title = section.title.as_deref().unwrap_or("");

    // Group items into columns (column headers always from section)
    let mut columns: Vec<KanbanColumn> = Vec::new();

    for item in &section.items {
        let is_column = item.get("column").map(|v| v == "true").unwrap_or(false);

        if is_column {
            columns.push(KanbanColumn {
                name: item.get("name").cloned().unwrap_or_default(),
                color: item.get("color").cloned().unwrap_or_else(|| "#94a3b8".to_string()),
                cards: Vec::new(),
            });
        } else if let Some(col) = columns.last_mut() {
            // Only add static cards if we have no bound data
            if matches!(bound_data, crate::binding::ResolvedData::None) {
                col.cards.push(KanbanCard {
                    name: item.get("name").cloned().unwrap_or_default(),
                    assignee: item.get("assignee").cloned().unwrap_or_default(),
                    priority: item.get("priority").cloned().unwrap_or_default(),
                    label: item.get("label").cloned().unwrap_or_default(),
                });
            }
        }
    }

    // If we have bound rows, distribute them into columns
    if let crate::binding::ResolvedData::Rows(rows) = bound_data {
        if !rows.is_empty() {
            for row in rows {
                let card = KanbanCard {
                    name: row.get("name").or_else(|| row.get("title"))
                        .and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    assignee: row.get("assignee")
                        .and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    priority: row.get("priority")
                        .and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    label: row.get("label")
                        .and_then(|v| v.as_str()).unwrap_or("").to_string(),
                };

                // Try to match row to column by status/column field
                let row_col = row.get("status")
                    .or_else(|| row.get("column"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let target_idx = columns.iter()
                    .position(|c| c.name.eq_ignore_ascii_case(row_col))
                    .or(if columns.is_empty() { None } else { Some(0) });

                if let Some(idx) = target_idx {
                    columns[idx].cards.push(card);
                }
            }
        }
    }

    // Build HTML
    let mut html = String::new();

    // Board title
    if !title.is_empty() {
        html.push_str(&format!(
            "<h2 class=\"text-lg font-semibold mb-4\">{}</h2>\n",
            title
        ));
    }

    // Board container
    html.push_str("<div class=\"flex gap-4 overflow-x-auto pb-4 -mx-2 px-2\">\n");

    for col in &columns {
        let card_count = col.cards.len();

        // Column
        html.push_str("  <div class=\"min-w-[280px] max-w-[320px] bg-neutral-50 rounded-xl p-3 flex-shrink-0\">\n");

        // Column header
        html.push_str("    <div class=\"flex items-center gap-2 mb-3 px-1\">\n");
        html.push_str(&format!(
            "      <span class=\"w-3 h-3 rounded-full inline-block\" style=\"background:{};\"></span>\n",
            col.color
        ));
        html.push_str(&format!(
            "      <span class=\"font-semibold text-sm\">{}</span>\n",
            crate::security::html_escape(&col.name)
        ));
        html.push_str(&format!(
            "      <span class=\"text-neutral-400 text-xs ml-auto\">{}</span>\n",
            card_count
        ));
        html.push_str("    </div>\n");

        // Cards container
        html.push_str("    <div class=\"flex flex-col gap-2\">\n");

        for card in &col.cards {
            html.push_str("      <div class=\"bg-white border border-neutral-200 rounded-lg p-3 shadow-sm hover:shadow-md transition-shadow\" style=\"cursor:grab;\">\n");

            // Card title
            html.push_str(&format!(
                "        <div class=\"text-sm font-medium mb-2\">{}</div>\n",
                crate::security::html_escape(&card.name)
            ));

            // Card footer
            let has_label = !card.label.is_empty();
            let has_priority = !card.priority.is_empty();
            let has_assignee = !card.assignee.is_empty();

            if has_label || has_priority || has_assignee {
                html.push_str("        <div class=\"flex items-center justify-between mt-2\">\n");

                // Left: label + priority
                html.push_str("          <div class=\"flex items-center gap-2\">\n");

                if has_label {
                    html.push_str(&format!(
                        "            <span class=\"text-xs px-2 py-0.5 rounded-full bg-blue-50 text-blue-700 border border-blue-200\">{}</span>\n",
                        crate::security::html_escape(&card.label)
                    ));
                }

                if has_priority {
                    let (bg, text, border) = priority_classes(&card.priority);
                    html.push_str(&format!(
                        "            <span class=\"text-xs px-2 py-0.5 rounded-full {} {} border {}\">{}</span>\n",
                        bg, text, border, crate::security::html_escape(&card.priority)
                    ));
                }

                html.push_str("          </div>\n");

                // Right: assignee avatar
                if has_assignee {
                    let initials = assignee_initials(&card.assignee);
                    html.push_str(&format!(
                        "          <div class=\"w-6 h-6 rounded-full bg-neutral-200 text-[10px] font-semibold flex items-center justify-center uppercase\" title=\"{}\">{}</div>\n",
                        crate::security::html_escape(&card.assignee), crate::security::html_escape(&initials)
                    ));
                }

                html.push_str("        </div>\n");
            }

            html.push_str("      </div>\n");
        }

        html.push_str("    </div>\n");
        html.push_str("  </div>\n");
    }

    html.push_str("</div>\n");
    html
}

/// Map priority level to Tailwind color classes.
fn priority_classes(priority: &str) -> (&'static str, &'static str, &'static str) {
    match priority {
        "critical" => ("bg-red-100", "text-red-700", "border-red-200"),
        "high" => ("bg-orange-100", "text-orange-700", "border-orange-200"),
        "medium" => ("bg-yellow-100", "text-yellow-700", "border-yellow-200"),
        "low" => ("bg-neutral-100", "text-neutral-600", "border-neutral-200"),
        _ => ("bg-neutral-100", "text-neutral-600", "border-neutral-200"),
    }
}

/// Extract first 2 uppercase letters from assignee name.
fn assignee_initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

// ---------------------------------------------------------------------------
// Dark Mode Toggle
// ---------------------------------------------------------------------------

/// Renders a dark mode toggle with CSS variables and localStorage persistence.
pub fn render_dark_mode_toggle(_section: &SectionNode) -> String {
    let mut html = String::new();

    // CSS variables
    html.push_str("<style>\n");
    html.push_str(":root {\n");
    html.push_str("  --bg-primary: #ffffff;\n");
    html.push_str("  --bg-secondary: #f9fafb;\n");
    html.push_str("  --bg-tertiary: #f3f4f6;\n");
    html.push_str("  --text-primary: #111827;\n");
    html.push_str("  --text-secondary: #6b7280;\n");
    html.push_str("  --border-color: #e5e7eb;\n");
    html.push_str("}\n");
    html.push_str("[data-theme=\"dark\"] {\n");
    html.push_str("  --bg-primary: #0a0a0a;\n");
    html.push_str("  --bg-secondary: #141414;\n");
    html.push_str("  --bg-tertiary: #1f1f1f;\n");
    html.push_str("  --text-primary: #f9fafb;\n");
    html.push_str("  --text-secondary: #9ca3af;\n");
    html.push_str("  --border-color: #2a2a2a;\n");
    html.push_str("}\n");
    html.push_str("#cronus-theme-toggle {\n");
    html.push_str("  position: relative;\n");
    html.push_str("  width: 2.75rem;\n");
    html.push_str("  height: 1.5rem;\n");
    html.push_str("  border-radius: 9999px;\n");
    html.push_str("  border: none;\n");
    html.push_str("  cursor: pointer;\n");
    html.push_str("  background: #d1d5db;\n");
    html.push_str("  transition: background 0.2s;\n");
    html.push_str("}\n");
    html.push_str("#cronus-theme-toggle[data-active=\"true\"] {\n");
    html.push_str("  background: #2563eb;\n");
    html.push_str("}\n");
    html.push_str("#cronus-theme-toggle .knob {\n");
    html.push_str("  position: absolute;\n");
    html.push_str("  top: 2px;\n");
    html.push_str("  left: 2px;\n");
    html.push_str("  width: 1.25rem;\n");
    html.push_str("  height: 1.25rem;\n");
    html.push_str("  border-radius: 9999px;\n");
    html.push_str("  background: #ffffff;\n");
    html.push_str("  box-shadow: 0 1px 2px rgba(0,0,0,0.1);\n");
    html.push_str("  transition: left 0.2s;\n");
    html.push_str("  display: flex;\n");
    html.push_str("  align-items: center;\n");
    html.push_str("  justify-content: center;\n");
    html.push_str("  font-size: 10px;\n");
    html.push_str("}\n");
    html.push_str("#cronus-theme-toggle[data-active=\"true\"] .knob {\n");
    html.push_str("  left: 22px;\n");
    html.push_str("}\n");
    html.push_str("</style>\n");

    // Toggle widget
    html.push_str("<div class=\"flex items-center gap-2\">\n");
    html.push_str("  <span class=\"text-xs text-neutral-500\">Theme</span>\n");
    html.push_str("  <button id=\"cronus-theme-toggle\" onclick=\"cronusToggleTheme()\" data-active=\"false\">\n");
    html.push_str("    <span class=\"knob\" id=\"cronus-theme-knob\">\u{2600}</span>\n");
    html.push_str("  </button>\n");
    html.push_str("</div>\n");

    // JavaScript
    html.push_str(&format!("<script{}>\n", crate::security::script_nonce_attr()));
    html.push_str("(function() {\n");
    html.push_str("  var saved = localStorage.getItem('cronus-theme');\n");
    html.push_str("  if (saved) { document.documentElement.dataset.theme = saved; }\n");
    html.push_str("  else if (window.matchMedia('(prefers-color-scheme: dark)').matches) {\n");
    html.push_str("    document.documentElement.dataset.theme = 'dark';\n");
    html.push_str("  }\n");
    html.push_str("  var isDark = document.documentElement.dataset.theme === 'dark';\n");
    html.push_str("  var toggle = document.getElementById('cronus-theme-toggle');\n");
    html.push_str("  var knob = document.getElementById('cronus-theme-knob');\n");
    html.push_str("  if (toggle) toggle.dataset.active = String(isDark);\n");
    html.push_str("  if (knob) knob.textContent = isDark ? '\\u{1F319}' : '\\u{2600}';\n");
    html.push_str("})();\n");
    html.push_str("function cronusToggleTheme() {\n");
    html.push_str("  var html = document.documentElement;\n");
    html.push_str("  var isDark = html.dataset.theme === 'dark';\n");
    html.push_str("  html.dataset.theme = isDark ? 'light' : 'dark';\n");
    html.push_str("  localStorage.setItem('cronus-theme', html.dataset.theme);\n");
    html.push_str("  var toggle = document.getElementById('cronus-theme-toggle');\n");
    html.push_str("  var knob = document.getElementById('cronus-theme-knob');\n");
    html.push_str("  if (toggle) toggle.dataset.active = String(!isDark);\n");
    html.push_str("  if (knob) knob.textContent = !isDark ? '\\u{1F319}' : '\\u{2600}';\n");
    html.push_str("}\n");
    html.push_str("</script>\n");

    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::ResolvedData;
    use serde_json::json;

    #[test]
    fn kanban_cards_from_rows_are_escaped() {
        let mut column = HashMap::new();
        column.insert("column".to_string(), "true".to_string());
        column.insert("name".to_string(), "Todo".to_string());
        let section = SectionNode {
            section_type: "kanban".into(),
            title: None,
            subtitle: None,
            config: HashMap::new(),
            items: vec![column],
            plans: vec![],
            binding: None,
            actions: vec![],
            visibility: None,
            template: None,
            style_block: None,
            doc: None,
        };
        let rows = ResolvedData::Rows(vec![json!({
            "name": "<script>card()</script>",
            "assignee": "\" onmouseover=\"alert(1)",
            "priority": "<i>p</i>",
            "label": "<u>l</u>",
            "status": "Todo",
        })]);
        let html = render_kanban(&section, &rows);
        assert!(html.contains("&lt;script&gt;card()"), "{html}");
        for raw in ["<script>card()", "\" onmouseover=\"alert", "<i>p</i>", "<u>l</u>"] {
            assert!(!html.contains(raw), "found {raw:?} in {html}");
        }
    }
}
