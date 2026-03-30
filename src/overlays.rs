#![allow(dead_code, unused_imports)]
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use crate::parser::SectionNode;

static OVERLAY_COUNTER: AtomicU32 = AtomicU32::new(0);

fn next_id(prefix: &str) -> String {
    let n = OVERLAY_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}", prefix, n)
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

// ---------------------------------------------------------------------------
// 1. Dropdown Menu
// ---------------------------------------------------------------------------

/// Renders a dropdown menu from a "dropdown" section
pub fn render_dropdown(section: &SectionNode) -> String {
    let trigger = section.config.get("trigger").map(|s| s.as_str()).unwrap_or("Actions");
    let id = section.config.get("id").cloned().unwrap_or_else(|| next_id("dd"));

    let mut items_html = String::new();
    for item in &section.items {
        let name = item.get("title").or(item.get("name")).map(|s| s.as_str()).unwrap_or("");

        // Divider
        if name == "---" || name == "separator" {
            items_html.push_str(r#"<div style="border-top:1px solid #e5e7eb;margin:4px 0"></div>"#);
            continue;
        }

        let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("");
        let danger = item.get("danger").map(|s| s == "true").unwrap_or(false);
        let link = item.get("link").map(|s| s.as_str()).unwrap_or("#");

        let text_color = if danger { "color:#dc2626" } else { "color:#1a1c1c" };
        let hover_bg = if danger { "background:#fef2f2" } else { "background:#f5f5f5" };
        let icon_color = if danger { "color:#dc2626" } else { "color:#71717a" };

        let icon_html = if icon.is_empty() {
            String::new()
        } else {
            format!(r#"<span class="material-symbols-outlined" style="font-size:16px;{}">{}</span>"#, icon_color, escape_html(icon))
        };

        items_html.push_str(&format!(
            r#"<div style="padding:8px 12px;font-size:14px;display:flex;align-items:center;gap:10px;cursor:pointer;{text_color};font-family:Inter,sans-serif" onmouseenter="this.style.background='{hover_bg}'" onmouseleave="this.style.background='transparent'" onclick="if('{link}'!=='#')window.location='{link}'">{icon}<span>{name}</span></div>"#,
            text_color = text_color, hover_bg = hover_bg, icon = icon_html,
            name = escape_html(name), link = escape_html(link),
        ));
    }

    format!(
        r##"<div data-dropdown style="position:relative;display:inline-block">
  <button onclick="cronusToggleDropdown('{id}')" style="border:1px solid #e5e7eb;border-radius:8px;padding:8px 12px;font-size:14px;display:inline-flex;align-items:center;gap:8px;cursor:pointer;background:#fff;font-family:Inter,sans-serif;font-weight:500">
    {trigger}
    <span class="material-symbols-outlined" style="font-size:16px;color:#71717a">expand_more</span>
  </button>
  <div id="{id}-menu" data-dropdown-menu class="hidden" style="position:absolute;top:100%;left:0;margin-top:4px;background:#fff;border:1px solid #e5e7eb;border-radius:12px;box-shadow:0 8px 24px rgba(0,0,0,0.12);padding:4px 0;min-width:180px;z-index:50">
    {items}
  </div>
</div>"##,
        id = id, trigger = escape_html(trigger), items = items_html,
    )
}

// ---------------------------------------------------------------------------
// 2. Toast Notification
// ---------------------------------------------------------------------------

/// Renders a toast notification from a "toast" section
pub fn render_toast(section: &SectionNode) -> String {
    let message = section.title.as_deref().unwrap_or("Notification");
    let toast_type = section.config.get("type").map(|s| s.as_str()).unwrap_or("info");
    let duration = section.config.get("duration").map(|s| s.as_str()).unwrap_or("3000");
    let id = section.config.get("id").cloned().unwrap_or_else(|| next_id("toast"));

    let (bg, border_color, text_color, icon_color, icon_name) = match toast_type {
        "success" => ("#ecfdf5", "#a7f3d0", "#065f46", "#059669", "check_circle"),
        "error"   => ("#fef2f2", "#fecaca", "#991b1b", "#dc2626", "error"),
        "warning" => ("#fffbeb", "#fde68a", "#92400e", "#d97706", "warning"),
        _         => ("#eff6ff", "#bfdbfe", "#1e40af", "#2563eb", "info"),
    };

    format!(
        r##"<div id="{id}" data-toast-duration="{duration}" style="position:fixed;bottom:24px;right:24px;z-index:200;display:flex;align-items:center;gap:12px;padding:12px 16px;border-radius:12px;border:1px solid {border};background:{bg};box-shadow:0 8px 24px rgba(0,0,0,0.1);animation:cronusSlideIn 0.3s ease-out;font-family:Inter,sans-serif;max-width:400px">
  <span class="material-symbols-outlined" style="font-size:24px;color:{icon_color};flex-shrink:0">{icon}</span>
  <span style="font-size:14px;font-weight:500;color:{text};flex:1">{message}</span>
  <button onclick="cronusDismissToast(this.parentElement)" style="background:none;border:none;cursor:pointer;padding:2px;flex-shrink:0">
    <span class="material-symbols-outlined" style="font-size:16px;color:{text}">close</span>
  </button>
</div>"##,
        id = id, duration = duration, bg = bg, border = border_color,
        text = text_color, icon_color = icon_color, icon = icon_name,
        message = escape_html(message),
    )
}

// ---------------------------------------------------------------------------
// 3. Notification Center
// ---------------------------------------------------------------------------

/// Renders a notification center panel from a "notifications" section
pub fn render_notification_center(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("Notifications");
    let id = section.config.get("id").cloned().unwrap_or_else(|| next_id("notif"));

    let mut unread_count = 0u32;
    let mut items_html = String::new();

    for (i, item) in section.items.iter().enumerate() {
        let name = item.get("title").or(item.get("name")).map(|s| s.as_str()).unwrap_or("");
        let time = item.get("time").map(|s| s.as_str()).unwrap_or("");
        let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("notifications");
        let unread = item.get("unread").map(|s| s == "true").unwrap_or(false);

        if unread { unread_count += 1; }

        let font_weight = if unread { "font-weight:600" } else { "font-weight:500" };
        let dot_html = if unread {
            r#"<div style="width:8px;height:8px;border-radius:50%;background:#3b82f6;flex-shrink:0;margin-top:8px"></div>"#
        } else { "" };

        let border_bottom = if i < section.items.len() - 1 { "border-bottom:1px solid #f4f4f5;" } else { "" };

        items_html.push_str(&format!(
            r##"<div style="display:flex;align-items:flex-start;gap:12px;padding:12px 16px;cursor:pointer;{border}" onmouseenter="this.style.background='#fafafa'" onmouseleave="this.style.background='transparent'">
  <div style="width:32px;height:32px;border-radius:50%;background:#f5f5f5;display:flex;align-items:center;justify-content:center;flex-shrink:0">
    <span class="material-symbols-outlined" style="font-size:16px;color:#71717a">{icon}</span>
  </div>
  <div style="flex:1;min-width:0">
    <div style="font-size:14px;{font_weight};color:#1a1c1c;font-family:Inter,sans-serif">{name}</div>
    <div style="font-size:12px;color:#a3a3a3;margin-top:2px">{time}</div>
  </div>
  {dot}
</div>"##,
            border = border_bottom, icon = escape_html(icon),
            font_weight = font_weight, name = escape_html(name),
            time = escape_html(time), dot = dot_html,
        ));
    }

    let badge_html = if unread_count > 0 {
        format!(
            r#"<div style="background:#ef4444;color:#fff;font-size:11px;font-weight:700;border-radius:50%;width:20px;height:20px;display:flex;align-items:center;justify-content:center">{}</div>"#,
            unread_count
        )
    } else { String::new() };

    format!(
        r##"<div id="{id}" style="max-width:384px;width:100%;background:#fff;border:1px solid #e5e7eb;border-radius:12px;box-shadow:0 8px 32px rgba(0,0,0,0.12);overflow:hidden;font-family:Inter,sans-serif">
  <div style="display:flex;justify-content:space-between;align-items:center;padding:12px 16px;border-bottom:1px solid #f4f4f5">
    <div style="display:flex;align-items:center;gap:8px">
      <span style="font-size:14px;font-weight:600;color:#1a1c1c">{title}</span>
      {badge}
    </div>
    <button onclick="document.querySelectorAll('#{id} [style*=background\\:rgb]').forEach(d=>d.remove())" style="background:none;border:none;font-size:12px;color:#2563eb;cursor:pointer;font-family:Inter,sans-serif">Mark all read</button>
  </div>
  <div style="max-height:400px;overflow-y:auto">
    {items}
  </div>
</div>"##,
        id = id, title = escape_html(title), badge = badge_html, items = items_html,
    )
}
