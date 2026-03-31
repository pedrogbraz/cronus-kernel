// tabs.rs — Dedicated renderer for `section tabs { tab "Label" { ... } }`
// Renders a tab bar with switchable content panels.

use crate::parser::SectionNode;

/// Render a tabs section into HTML with tab buttons and content panels.
///
/// Expected items: `_type:"tab"` with `title` (label), optional `icon`,
/// optional `active`/`style:"active"` flag, and `description` (panel content).
pub fn render_tabs(section: &SectionNode) -> String {
    let mut html = String::new();

    // Collect only tab items
    let tabs: Vec<&std::collections::HashMap<String, String>> = section
        .items
        .iter()
        .filter(|item| item.get("_type").map(|t| t.as_str()) == Some("tab"))
        .collect();

    if tabs.is_empty() {
        return String::new();
    }

    // Generate a unique group id from the section title or fallback
    let group_id = section
        .title
        .as_deref()
        .unwrap_or("tabs")
        .replace(|c: char| !c.is_alphanumeric(), "_")
        .to_lowercase();

    // Determine which tab is active (first with active flag, or index 0)
    let active_index = tabs
        .iter()
        .position(|tab| {
            tab.get("active").map(|v| v == "true").unwrap_or(false)
                || tab.get("style").map(|v| v == "active").unwrap_or(false)
        })
        .unwrap_or(0);

    // ── Section wrapper ──
    html.push_str(&format!(
        r#"<section id="cronus-tabs-{}" style="padding:32px 24px;max-width:1280px;margin:0 auto;font-family:Inter,sans-serif">"#,
        group_id
    ));

    // ── Optional section title ──
    if let Some(ref title) = section.title {
        html.push_str(&format!(
            r#"<h2 style="font-size:24px;font-weight:700;color:#171717;margin:0 0 16px">{}</h2>"#,
            title
        ));
    }

    // ── Optional subtitle ──
    if let Some(ref subtitle) = section.subtitle {
        html.push_str(&format!(
            r#"<p style="font-size:14px;color:#6b7280;margin:0 0 16px">{}</p>"#,
            subtitle
        ));
    }

    // ── Tab bar ──
    html.push_str(
        r#"<div role="tablist" style="display:flex;gap:0;border-bottom:1px solid #e5e7eb">"#,
    );

    for (i, tab) in tabs.iter().enumerate() {
        let label = tab
            .get("title")
            .or_else(|| tab.get("name"))
            .map(|s| s.as_str())
            .unwrap_or("");
        let icon = tab.get("icon").map(|s| s.as_str()).unwrap_or("");
        let is_active = i == active_index;

        let (border_style, color) = if is_active {
            ("border-bottom:2px solid #171717;", "color:#171717;")
        } else {
            ("border-bottom:2px solid transparent;", "color:#6b7280;")
        };

        html.push_str(&format!(
            r#"<button role="tab" aria-selected="{}" data-tab-index="{}" onclick="cronusSwitchTab('{}',{})" style="padding:10px 16px;font-size:14px;font-weight:500;cursor:pointer;background:none;border:none;{}{}{}">"#,
            is_active,
            i,
            group_id,
            i,
            border_style,
            color,
            if is_active { "" } else { "" }
        ));

        // Icon (optional)
        if !icon.is_empty() {
            html.push_str(&format!(
                r#"<span class="material-symbols-outlined" style="font-size:16px;margin-right:6px;vertical-align:middle">{}</span>"#,
                icon
            ));
        }

        html.push_str(&format!(
            r#"<span style="vertical-align:middle">{}</span></button>"#,
            label
        ));
    }

    html.push_str("</div>");

    // ── Tab panels ──
    html.push_str(r#"<div style="padding-top:16px">"#);

    for (i, tab) in tabs.iter().enumerate() {
        let is_active = i == active_index;
        let display = if is_active { "block" } else { "none" };
        let content = tab.get("description").map(|s| s.as_str()).unwrap_or("");

        html.push_str(&format!(
            r#"<div role="tabpanel" data-panel-index="{}" style="display:{}">"#,
            i, display
        ));

        if !content.is_empty() {
            html.push_str(&format!(
                r#"<div style="font-size:14px;color:#171717;line-height:1.6">{}</div>"#,
                content
            ));
        }

        html.push_str("</div>");
    }

    html.push_str("</div>");

    // ── JavaScript ──
    html.push_str(&format!(
        r#"<script>
function cronusSwitchTab(groupId, tabIndex) {{
  var container = document.getElementById('cronus-tabs-' + groupId);
  if (!container) return;
  var buttons = container.querySelectorAll('[role="tab"]');
  var panels = container.querySelectorAll('[role="tabpanel"]');
  for (var i = 0; i < buttons.length; i++) {{
    var btn = buttons[i];
    var panel = panels[i];
    if (i === tabIndex) {{
      btn.setAttribute('aria-selected', 'true');
      btn.style.borderBottom = '2px solid #171717';
      btn.style.color = '#171717';
      if (panel) panel.style.display = 'block';
    }} else {{
      btn.setAttribute('aria-selected', 'false');
      btn.style.borderBottom = '2px solid transparent';
      btn.style.color = '#6b7280';
      if (panel) panel.style.display = 'none';
    }}
  }}
}}
</script>"#
    ));

    html.push_str("</section>");
    html
}
