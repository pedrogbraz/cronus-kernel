//! KPI and stat card section renderers
use crate::parser::SectionNode;

pub(super) fn render_stat_cards(section: &SectionNode, bound_data: &crate::binding::ResolvedData) -> String {
    let cols = section.config.get("cols")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(3);

    let t = crate::theme::get();
    let is_dark = t.surface.contains("0e0e0e") || t.surface.contains("000") || t.on_surface.contains("fff");

    // Build items from DB rows when section has no static items
    let db_items: Vec<std::collections::HashMap<String, String>> = if section.items.is_empty() {
        if let crate::binding::ResolvedData::Rows(rows) = bound_data {
            rows.iter().map(|row| {
                let mut map = std::collections::HashMap::new();
                if let Some(obj) = row.as_object() {
                    for (k, v) in obj {
                        let val = match v {
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::Bool(b) => b.to_string(),
                            serde_json::Value::Null => String::new(),
                            other => other.to_string(),
                        };
                        match k.as_str() {
                            "label" => { map.insert("title".to_string(), val); }
                            "change" => { map.insert("badge".to_string(), val); }
                            _ => { map.insert(k.clone(), val); }
                        }
                    }
                }
                map
            }).collect()
        } else { Vec::new() }
    } else { Vec::new() };

    let use_db = !db_items.is_empty();
    let item_count = if use_db { db_items.len() } else { section.items.len() };

    let bound_value: Option<String> = if !use_db {
        match bound_data {
            crate::binding::ResolvedData::Count(n) => Some(n.to_string()),
            crate::binding::ResolvedData::Rows(rows) if !rows.is_empty() => Some(rows.len().to_string()),
            _ => None,
        }
    } else { None };

    // Theme colors
    let (card_bg, card_border, label_color, value_color) = if is_dark {
        ("#1b1b1b", "0.5px solid rgba(76,69,70,0.15)", "rgba(226,226,226,0.5)", "#e2e2e2")
    } else {
        ("#fff", "1px solid rgba(198,198,198,0.2)", "#5e5e5e", "#1a1c1c")
    };

    let mut cards = Vec::new();
    for idx in 0..item_count {
        let (label, value_owned, icon, badge);
        if use_db {
            let db = &db_items[idx];
            label = db.get("title").map(|s| s.as_str()).unwrap_or("Metric");
            value_owned = db.get("value").cloned().unwrap_or_default();
            icon = db.get("icon").cloned().unwrap_or_default();
            badge = db.get("badge").cloned().unwrap_or_default();
        } else {
            let si = &section.items[idx];
            label = si.get("title").or_else(|| si.get("name")).map(|s| s.as_str()).unwrap_or("Metric");
            value_owned = if idx == 0 && bound_value.is_some() {
                bound_value.as_ref().unwrap().clone()
            } else {
                si.get("description").or_else(|| si.get("desc")).map(|s| s.to_string()).unwrap_or_default()
            };
            icon = si.get("icon").cloned().unwrap_or_default();
            badge = String::new();
        }
        let value = value_owned.as_str();
        let icon_html = if icon.is_empty() {
            String::new()
        } else {
            format!(r#"<span class="material-symbols-outlined" style="font-size:20px;color:{label_color};margin-bottom:4px">{icon}</span>"#)
        };
        let badge_html = if badge.is_empty() {
            String::new()
        } else {
            let positive = badge.starts_with('+') || (!badge.starts_with('-') && badge.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false));
            let (bg, fg) = if positive { ("rgba(16,185,129,0.1)", "#10b981") } else { ("rgba(239,68,68,0.1)", "#ef4444") };
            format!(r#"<span style="display:inline-flex;align-items:center;padding:2px 8px;border-radius:9999px;font-size:12px;font-weight:600;background:{bg};color:{fg}">{badge}</span>"#)
        };
        let delay = format!("d{}", (idx % 10) + 1);

        cards.push(format!(
            r##"<div class="anim-scale {delay}" style="background:{card_bg};border:{card_border};border-radius:12px;padding:24px;transition:background 0.2s" onmouseover="this.style.opacity='0.9'" onmouseout="this.style.opacity='1'">
  {icon_html}
  <div style="display:flex;align-items:center;gap:8px;margin-bottom:8px"><p style="font-size:12px;font-weight:600;text-transform:uppercase;letter-spacing:0.1em;color:{label_color};margin:0">{label}</p>{badge_html}</div>
  <p data-count-to="{value}" style="font-size:28px;font-weight:700;color:{value_color};margin:0;letter-spacing:-0.02em">0</p>
</div>"##
        ));
    }

    format!(
        r##"<div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:16px;font-family:'Inter',system-ui,-apple-system,sans-serif;width:100%">
  {items}
</div>"##,
        items = cards.join("\n  "),
    )
}


pub(super) fn render_kpi_section(section: &SectionNode, bound_data: &crate::binding::ResolvedData) -> String {
    let title = section.title.as_deref().unwrap_or("");
    let subtitle = section.subtitle.as_deref().unwrap_or("");
    let cols: usize = section.config.get("cols")
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);

    let title_html = if title.is_empty() {
        String::new()
    } else {
        let sub = if subtitle.is_empty() {
            String::new()
        } else {
            format!(r#"<p style="font-size:14px;color:#71717a;margin:4px 0 0">{}</p>"#, subtitle)
        };
        format!(r#"<div style="margin-bottom:24px"><h2 style="font-size:20px;font-weight:700;letter-spacing:-0.02em;margin:0">{}</h2>{}</div>"#, title, sub)
    };

    // Build items from DB rows when section has no static items
    let db_items: Vec<std::collections::HashMap<String, String>> = if section.items.is_empty() {
        if let crate::binding::ResolvedData::Rows(rows) = bound_data {
            rows.iter().map(|row| {
                let mut map = std::collections::HashMap::new();
                if let Some(obj) = row.as_object() {
                    for (k, v) in obj {
                        let val = match v {
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::Bool(b) => b.to_string(),
                            serde_json::Value::Null => String::new(),
                            other => other.to_string(),
                        };
                        match k.as_str() {
                            "label" => { map.insert("title".to_string(), val); }
                            "change" => { map.insert("badge".to_string(), val); }
                            _ => { map.insert(k.clone(), val); }
                        }
                    }
                }
                map
            }).collect()
        } else { Vec::new() }
    } else { Vec::new() };

    let use_db = !db_items.is_empty();
    let item_count = if use_db { db_items.len() } else { section.items.len() };

    let bound_value: Option<String> = if !use_db {
        match bound_data {
            crate::binding::ResolvedData::Count(n) => Some(n.to_string()),
            crate::binding::ResolvedData::Rows(rows) if !rows.is_empty() => Some(rows.len().to_string()),
            _ => None,
        }
    } else { None };

    // Empty state: show placeholder cards when no data (light theme)
    if item_count == 0 {
        let placeholder_count = cols;
        let mut placeholder_html = String::new();
        for _i in 0..placeholder_count {
            placeholder_html.push_str(
                r#"<div style="background:#fff;border:1px solid #e5e7eb;border-radius:12px;padding:20px 24px">
  <div style="margin-bottom:12px"><span style="font-size:13px;font-weight:500;color:#9ca3af">—</span></div>
  <div><span style="font-size:36px;font-weight:700;letter-spacing:-0.03em;color:#d1d5db">—</span></div>
  <p style="font-size:11px;color:#d1d5db;margin:8px 0 0">No data yet</p>
</div>"#
            );
        }
        return format!(
            r#"<section style="padding:32px 0">{title_html}<div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:16px">{cards}</div></section>"#,
            title_html = title_html, cards = placeholder_html,
        );
    }

    let mut cards_html = String::new();
    for i in 0..item_count {
        let (item_title, value_owned, icon, trend, badge, meta, item_subtitle);
        if use_db {
            let db = &db_items[i];
            item_title = db.get("title").map(|s| s.as_str()).unwrap_or("");
            value_owned = db.get("value").cloned().unwrap_or_default();
            icon = db.get("icon").map(|s| s.as_str()).unwrap_or("");
            trend = db.get("trend").or(db.get("badge")).map(|s| s.as_str()).unwrap_or("");
            badge = db.get("badge").map(|s| s.as_str()).unwrap_or("");
            meta = db.get("description").map(|s| s.as_str()).unwrap_or("");
            item_subtitle = db.get("subtitle").map(|s| s.as_str()).unwrap_or("");
        } else {
            let si = &section.items[i];
            item_title = si.get("title").map(|s| s.as_str()).unwrap_or("");
            value_owned = if i == 0 && bound_value.is_some() {
                bound_value.as_ref().unwrap().clone()
            } else {
                si.get("value").map(|s| s.to_string()).unwrap_or_default()
            };
            icon = si.get("icon").map(|s| s.as_str()).unwrap_or("");
            trend = si.get("trend").map(|s| s.as_str()).unwrap_or("");
            badge = si.get("badge").map(|s| s.as_str()).unwrap_or("");
            meta = si.get("description").map(|s| s.as_str()).unwrap_or("");
            item_subtitle = si.get("subtitle").map(|s| s.as_str()).unwrap_or("");
        }
        let value = value_owned.as_str();
        let delay_class = format!("d{}", (i % 10) + 1);

        let icon_html = if icon.is_empty() {
            String::new()
        } else {
            format!(r#"<span class="material-symbols-outlined" style="font-size:20px;color:#71717a;margin-bottom:8px">{}</span>"#, icon)
        };

        // Badge pill (same as dark theme)
        let badge_html = if badge.is_empty() {
            String::new()
        } else {
            let badge_positive = badge.starts_with('+') || (!badge.starts_with('-') && badge.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false));
            let (bg, fg) = if badge_positive {
                ("rgba(16,185,129,0.1)", "#10b981")
            } else {
                ("rgba(239,68,68,0.1)", "#ef4444")
            };
            format!(
                r#"<span style="display:inline-flex;align-items:center;padding:2px 8px;border-radius:9999px;font-size:12px;font-weight:600;background:{bg};color:{fg};letter-spacing:-0.01em">{badge}</span>"#,
                bg = bg, fg = fg, badge = badge
            )
        };

        let (trend_arrow, trend_color) = match trend {
            t if t.starts_with('+') || t == "up" => ("&#9650;", "#059669"),
            t if t.starts_with('-') || t == "down" => ("&#9660;", "#dc2626"),
            _ => ("", "#71717a"),
        };
        let trend_html = if trend.is_empty() {
            String::new()
        } else if trend == "up" || trend == "down" {
            format!(r#"<span style="font-size:12px;color:{};font-weight:600">{}</span>"#, trend_color, trend_arrow)
        } else {
            format!(r#"<span style="font-size:12px;color:{};font-weight:600">{} {}</span>"#, trend_color, trend_arrow, trend)
        };

        // Subtitle / description
        let sub_text = if !item_subtitle.is_empty() { item_subtitle } else { meta };
        let meta_html = if sub_text.is_empty() {
            String::new()
        } else {
            format!(r#"<div style="font-size:12px;color:#a1a1aa;margin-top:4px">{}</div>"#, sub_text)
        };

        cards_html.push_str(&format!(
            r##"<div class="anim-slide-up {delay}" style="background:#fff;border:1px solid #f4f4f5;border-radius:12px;padding:20px">
  {icon_html}
  <div style="display:flex;align-items:baseline;gap:8px">
    <span data-count-to="{value}" style="font-size:32px;font-weight:700;letter-spacing:-0.03em;line-height:1">0</span>
    {badge_html}
    {trend_html}
  </div>
  <div style="font-size:14px;color:#71717a;margin-top:6px;font-weight:500">{title}</div>
  {meta_html}
</div>"##,
            delay = delay_class, icon_html = icon_html, value = value,
            badge_html = badge_html, trend_html = trend_html, title = item_title, meta_html = meta_html,
        ));
    }

    format!(
        r##"<section style="padding:32px 0">
  {title_html}
  <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:16px">
    {cards}
  </div>
</section>"##,
        title_html = title_html, cards = cards_html,
    )
}

// ══════════════════════════════════════════════════
// KPI DASHBOARD DARK (Obsidian theme)
// ══════════════════════════════════════════════════

pub(super) fn render_kpi_dashboard_dark(section: &SectionNode, bound_data: &crate::binding::ResolvedData) -> String {
    let t = crate::theme::get();
    let title = section.title.as_deref().unwrap_or("");
    let subtitle = section.subtitle.as_deref().unwrap_or("");
    let cols: usize = section.config.get("cols")
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);

    let title_html = if title.is_empty() {
        String::new()
    } else {
        let sub = if subtitle.is_empty() {
            String::new()
        } else {
            format!(r#"<p style="font-size:13px;color:rgba(226,226,226,0.4);margin:4px 0 0">{}</p>"#, subtitle)
        };
        format!(r#"<div style="margin-bottom:24px"><h2 style="font-size:20px;font-weight:700;letter-spacing:-0.02em;margin:0;color:{}">{}</h2>{}</div>"#, t.on_surface, title, sub)
    };

    // Build items list: from DB rows (when section has no static items) or from .cronus items
    let db_items: Vec<std::collections::HashMap<String, String>> = if section.items.is_empty() {
        if let crate::binding::ResolvedData::Rows(rows) = bound_data {
            rows.iter().map(|row| {
                let mut map = std::collections::HashMap::new();
                if let Some(obj) = row.as_object() {
                    for (k, v) in obj {
                        let val = match v {
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::Bool(b) => b.to_string(),
                            serde_json::Value::Null => String::new(),
                            other => other.to_string(),
                        };
                        // Map entity fields to KPI card fields
                        match k.as_str() {
                            "label" => { map.insert("title".to_string(), val); }
                            "change" => { map.insert("badge".to_string(), val); }
                            _ => { map.insert(k.clone(), val); }
                        }
                    }
                }
                map
            }).collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Use DB items when available, otherwise fall back to static .cronus items
    let use_db_items = !db_items.is_empty();
    let item_count = if use_db_items { db_items.len() } else { section.items.len() };

    // Bound data override for first item (only when using static items)
    let bound_value: Option<String> = if !use_db_items {
        match bound_data {
            crate::binding::ResolvedData::Count(n) => Some(n.to_string()),
            crate::binding::ResolvedData::Rows(rows) if !rows.is_empty() => Some(rows.len().to_string()),
            _ => None,
        }
    } else {
        None
    };

    // Empty state: show placeholder cards when no data
    if item_count == 0 {
        let placeholder_count = cols;
        let mut placeholder_html = String::new();
        for i in 0..placeholder_count {
            placeholder_html.push_str(&format!(
                r##"<div class="anim-slide-up d{delay}" style="background:#1b1b1b;border:0.5px solid rgba(76,69,70,0.15);border-radius:12px;padding:20px 24px;cursor:default">
  <div style="display:flex;align-items:center;gap:6px;margin-bottom:12px">
    <span style="font-size:13px;font-weight:500;color:rgba(226,226,226,0.3);letter-spacing:0.02em">—</span>
  </div>
  <div style="display:flex;align-items:baseline;gap:10px">
    <span style="font-size:36px;font-weight:700;letter-spacing:-0.03em;line-height:1;color:rgba(226,226,226,0.15)">—</span>
  </div>
  <p style="font-size:11px;color:rgba(226,226,226,0.2);margin:8px 0 0">No data yet</p>
</div>"##,
                delay = (i % 10) + 1,
            ));
        }
        return format!(
            r##"<section style="padding:32px 0">
  {title_html}
  <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:16px">
    {cards}
  </div>
</section>"##,
            title_html = title_html, cards = placeholder_html,
        );
    }

    let mut cards_html = String::new();
    for i in 0..item_count {
        let (item_title, value_owned, icon, badge, item_subtitle, description, span);
        if use_db_items {
            let db_item = &db_items[i];
            item_title = db_item.get("title").map(|s| s.as_str()).unwrap_or("");
            value_owned = db_item.get("value").cloned().unwrap_or_default();
            icon = db_item.get("icon").map(|s| s.as_str()).unwrap_or("");
            badge = db_item.get("badge").map(|s| s.as_str()).unwrap_or("");
            item_subtitle = db_item.get("subtitle").map(|s| s.as_str()).unwrap_or("");
            description = db_item.get("description").map(|s| s.as_str()).unwrap_or("");
            span = db_item.get("span").and_then(|s| s.parse().ok()).unwrap_or(1usize);
        } else {
            let static_item = &section.items[i];
            item_title = static_item.get("title").map(|s| s.as_str()).unwrap_or("");
            value_owned = if i == 0 && bound_value.is_some() {
                bound_value.as_ref().unwrap().clone()
            } else {
                static_item.get("value").map(|s| s.to_string()).unwrap_or_default()
            };
            icon = static_item.get("icon").map(|s| s.as_str()).unwrap_or("");
            badge = static_item.get("badge").map(|s| s.as_str()).unwrap_or("");
            item_subtitle = static_item.get("subtitle").map(|s| s.as_str()).unwrap_or("");
            description = static_item.get("description").map(|s| s.as_str()).unwrap_or("");
            span = static_item.get("span").and_then(|s| s.parse().ok()).unwrap_or(1usize);
        }
        let value = value_owned.as_str();

        let col_span_style = if span > 1 {
            format!("grid-column:span {}", span)
        } else {
            String::new()
        };

        // Larger text for span:2+ cards — use clamp to prevent overflow
        let value_size = if span > 1 { "clamp(32px,5vw,56px)" } else { "30px" };
        let value_weight = "700";

        // Icon
        let icon_html = if icon.is_empty() {
            String::new()
        } else {
            format!(
                r#"<span class="material-symbols-outlined" style="font-size:20px;color:rgba(226,226,226,0.4);margin-bottom:2px">{}</span>"#,
                icon
            )
        };

        // Badge pill
        let badge_html = if badge.is_empty() {
            String::new()
        } else {
            let badge_positive = badge.starts_with('+') || (!badge.starts_with('-') && badge.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false));
            let (bg, fg) = if badge_positive {
                ("rgba(16,185,129,0.1)", "#10b981")
            } else {
                ("rgba(239,68,68,0.1)", "#ef4444")
            };
            format!(
                r#"<span style="display:inline-flex;align-items:center;padding:2px 8px;border-radius:9999px;font-size:12px;font-weight:600;background:{bg};color:{fg};letter-spacing:-0.01em">{badge}</span>"#,
                bg = bg, fg = fg, badge = badge
            )
        };

        // Subtitle / description
        let sub_text = if !item_subtitle.is_empty() { item_subtitle } else { description };
        let sub_html = if sub_text.is_empty() {
            String::new()
        } else {
            format!(r#"<div style="font-size:12px;color:rgba(226,226,226,0.4);margin-top:4px">{}</div>"#, sub_text)
        };

        // Mini bar chart for span:2 cards (decorative)
        let chart_html = if span > 1 {
            let bar_heights = [40, 65, 50, 80, 60, 90, 70];
            let mut bars = String::new();
            for h in &bar_heights {
                bars.push_str(&format!(
                    r#"<div style="flex:1;background:rgba(226,226,226,0.06);border-radius:3px;height:{}%;transition:background 0.2s" onmouseover="this.style.background='rgba(226,226,226,0.12)'" onmouseout="this.style.background='rgba(226,226,226,0.06)'"></div>"#,
                    h
                ));
            }
            format!(
                r#"<div style="display:flex;align-items:flex-end;gap:4px;height:48px;margin-top:16px">{}</div>"#,
                bars
            )
        } else {
            String::new()
        };

        // Label row: icon + title + badge
        let label_row = format!(
            r#"<div style="display:flex;align-items:center;gap:6px;margin-bottom:12px">{icon}{label}{badge}</div>"#,
            icon = icon_html,
            label = format!(r#"<span style="font-size:13px;font-weight:500;color:rgba(226,226,226,0.5);letter-spacing:0.02em">{}</span>"#, item_title),
            badge = badge_html,
        );

        cards_html.push_str(&format!(
            r##"<div class="anim-slide-up d{delay}" style="background:#1b1b1b;border:0.5px solid rgba(76,69,70,0.15);border-radius:12px;padding:20px 24px;transition:background 0.2s ease;cursor:default;{col_span}" onmouseover="this.style.background='#1f1f1f'" onmouseout="this.style.background='#1b1b1b'">
  {label_row}
  <div style="display:flex;align-items:baseline;gap:10px">
    <span data-count-to="{value}" style="font-size:{vsize};font-weight:{vweight};letter-spacing:-0.03em;line-height:1;color:#e2e2e2">0</span>
  </div>
  {sub_html}
  {chart_html}
</div>"##,
            delay = (i % 10) + 1,
            col_span = col_span_style,
            label_row = label_row,
            vsize = value_size,
            vweight = value_weight,
            value = value,
            sub_html = sub_html,
            chart_html = chart_html,
        ));
    }

    format!(
        r##"<section style="padding:32px 0">
  {title_html}
  <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:16px">
    {cards}
  </div>
</section>"##,
        title_html = title_html, cards = cards_html,
    )
}

// ══════════════════════════════════════════════════
// TIMELINE SECTION
// ══════════════════════════════════════════════════

