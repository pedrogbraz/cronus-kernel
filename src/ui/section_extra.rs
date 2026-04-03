#![allow(dead_code, unused_imports, unused_variables)]
//! Additional section renderers — product-grid, bento, team-list, modal, sheet,
//! tabs, accordion, breadcrumb, generic, alert, skeleton, empty, error, not-found,
//! timeline, progress, sidebar, card, links, etc.

use crate::parser::SectionNode;

pub(super) fn render_product_grid_section(section: &SectionNode) -> String {
    let cols = section.config.get("cols")
        .and_then(|c| c.parse::<u32>().ok())
        .unwrap_or(3);

    let cards: Vec<String> = section.items.iter().map(|item| {
        let name = item.get("title").or_else(|| item.get("name")).map(|s| s.as_str()).unwrap_or("Untitled");
        let desc = item.get("description").or_else(|| item.get("desc")).map(|s| s.as_str()).unwrap_or("");
        let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("");
        let status = item.get("status").map(|s| s.as_str());
        let price = item.get("price").map(|s| s.as_str());
        let interval = item.get("interval").map(|s| s.as_str());
        let unit = item.get("unit").map(|s| s.as_str());
        let action_text = item.get("action_text").map(|s| s.as_str()).unwrap_or("View");
        let action_icon = item.get("action_icon").map(|s| s.as_str());

        // Icon box (48x48, bg #e8e8e8, rounded 8px)
        let icon_html = if icon.is_empty() {
            r#"<div style="width:48px;height:48px;background:#e8e8e8;border-radius:8px;flex-shrink:0"></div>"#.to_string()
        } else {
            format!(r#"<div style="width:48px;height:48px;background:#e8e8e8;border-radius:8px;flex-shrink:0;display:flex;align-items:center;justify-content:center;font-size:20px">{icon}</div>"#, icon=icon)
        };

        // Status badge (Active=blue, Draft=gray, pill shape, 10px uppercase tracking-widest)
        let status_html = match status {
            Some(s) => {
                let (bg, tx) = if s.eq_ignore_ascii_case("active") {
                    ("rgba(59,130,246,0.15)", "#3b82f6")
                } else {
                    ("rgba(161,161,170,0.15)", "#a1a1aa")
                };
                let label = if s.eq_ignore_ascii_case("active") { "Active" } else { "Draft" };
                format!(r#"<span style="font-size:10px;font-weight:600;text-transform:uppercase;letter-spacing:0.1em;padding:4px 10px;border-radius:999px;background:{bg};color:{tx}">{label}</span>"#, bg=bg, tx=tx, label=label)
            }
            None => String::new(),
        };

        // Price line (24px bold + interval in 12px #5e5e5e)
        let price_html = match price {
            Some(p) => {
                let suffix = interval.or(unit).map(|i| format!(r#" <span style="font-size:12px;font-weight:400;color:#5e5e5e">/{i}</span>"#, i=i)).unwrap_or_default();
                format!(r#"<div style="font-size:24px;font-weight:700;color:#1a1c1c;margin-top:8px">{p}{suffix}</div>"#, p=p, suffix=suffix)
            }
            None => String::new(),
        };

        // Action button (full-width, bg #f3f3f3, rounded pill, 12px font-weight 700)
        let act_icon = action_icon.map(|ai| format!(r#" <span style="margin-left:4px">{ai}</span>"#, ai=ai)).unwrap_or_default();
        let action_html = format!(r#"<button style="width:100%;background:#f3f3f3;border:none;border-radius:999px;padding:10px 0;font-size:12px;font-weight:700;color:#1a1c1c;cursor:pointer;margin-top:16px;transition:background 0.15s" onmouseover="this.style.background='#e8e8e8'" onmouseout="this.style.background='#f3f3f3'">{action_text}{act_icon}</button>"#, action_text=action_text, act_icon=act_icon);

        format!(
            r##"<div style="background:white;border:1px solid rgba(198,198,198,0.2);border-radius:12px;padding:24px;display:flex;flex-direction:column;transition:border-color 0.2s" onmouseover="this.style.borderColor='rgba(0,0,0,0.1)'" onmouseout="this.style.borderColor='rgba(198,198,198,0.2)'"><div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:16px">{icon_html}{status_html}</div><div style="font-size:18px;font-weight:700;color:#1a1c1c">{name}</div><div style="font-size:14px;color:#5e5e5e;margin-top:4px">{desc}</div>{price_html}<div style="flex:1"></div>{action_html}</div>"##,
            icon_html=icon_html, status_html=status_html, name=name, desc=desc,
            price_html=price_html, action_html=action_html,
        )
    }).collect();

    let title_html = section.title.as_deref().map(|t| {
        format!(r#"<h2 style="font-size:24px;font-weight:700;color:#1a1c1c;margin-bottom:24px">{t}</h2>"#, t=t)
    }).unwrap_or_default();

    format!(
        r##"<section style="padding:48px 24px"><div style="max-width:var(--cronus-max-w,1120px);margin:0 auto">{title_html}<div style="display:grid;grid-template-columns:repeat({cols},1fr);gap:24px">{cards}</div></div></section>"##,
        title_html=title_html, cols=cols, cards=cards.join(""),
    )
}

pub(super) fn render_bento(section: &SectionNode, accent: &str) -> String {
    let cols = section.config.get("cols").and_then(|c| c.parse::<u32>().ok()).unwrap_or(3);
    let title_html = section.title.as_deref().map(|t| format!(
        r#"<h2 style="font-size:24px;font-weight:700;letter-spacing:-0.02em;color:#1a1c1c;margin-bottom:8px">{}</h2>"#, t
    )).unwrap_or_default();
    let subtitle_html = section.subtitle.as_deref().map(|s| format!(
        r#"<p style="font-size:14px;color:#5e5e5e;margin-bottom:32px;line-height:1.6">{}</p>"#, s
    )).unwrap_or_default();

    let cards: Vec<String> = section.items.iter().map(|item| {
        let name = item.get("title").or_else(|| item.get("name")).map(|s| s.as_str()).unwrap_or("Card");
        let desc = item.get("description").or_else(|| item.get("desc")).map(|s| s.as_str()).unwrap_or("");
        let span = item.get("span").and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);
        let style_hint = item.get("style").map(|s| s.as_str()).unwrap_or("");

        let (bg, text_color, border) = if style_hint == "dark" {
            ("#1a1a1a", "white", "1px solid rgba(255,255,255,0.1)")
        } else {
            ("white", "#1a1c1c", "1px solid rgba(198,198,198,0.2)")
        };

        let desc_color = if style_hint == "dark" { "#a1a1aa" } else { "#5e5e5e" };

        let badges_html = item.get("badges").map(|b| {
            let pills: Vec<String> = b.split(',').map(|badge| format!(
                r#"<span style="display:inline-block;padding:3px 10px;border-radius:999px;font-size:11px;font-weight:500;background:rgba(198,198,198,0.15);color:#5e5e5e">{}</span>"#,
                badge.trim()
            )).collect();
            format!(r#"<div style="display:flex;flex-wrap:wrap;gap:6px;margin-top:12px">{}</div>"#, pills.join(""))
        }).unwrap_or_default();

        let span_style = if span > 1 {
            format!("grid-column:span {};", span)
        } else {
            String::new()
        };

        format!(
            r#"<div class="reveal card-hover" style="background:{bg};border:{border};border-radius:12px;padding:24px;{span_style}">
  <h3 style="font-size:20px;font-weight:700;color:{text_color};margin-bottom:8px">{name}</h3>
  <p style="font-size:14px;color:{desc_color};line-height:1.6">{desc}</p>
  {badges_html}
</div>"#,
            bg = bg, border = border, span_style = span_style,
            text_color = text_color, name = name, desc_color = desc_color,
            desc = desc, badges_html = badges_html,
        )
    }).collect();

    format!(
        r#"<section style="max-width:1280px;margin:0 auto;padding:48px 24px">
  {title_html}
  {subtitle_html}
  <div class="stagger" style="display:grid;grid-template-columns:repeat({cols},1fr);gap:24px">
    {cards}
  </div>
</section>"#,
        title_html = title_html, subtitle_html = subtitle_html,
        cols = cols, cards = cards.join("\n    "),
    )
}

pub(super) fn render_team_list(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("Team");
    let badge = section.config.get("badge").map(|s| s.as_str());
    let footer_link = section.config.get("footer_link").map(|s| s.as_str());

    let badge_html = badge.map(|b| format!(
        r#" <span style="display:inline-block;padding:3px 10px;border-radius:999px;font-size:11px;font-weight:500;background:rgba(198,198,198,0.15);color:#5e5e5e;margin-left:8px">{}</span>"#, b
    )).unwrap_or_default();

    let rows: Vec<String> = section.items.iter().map(|item| {
        let name = item.get("title").or_else(|| item.get("name")).map(|s| s.as_str()).unwrap_or("Member");
        let email = item.get("email").map(|s| s.as_str()).unwrap_or("");
        let role = item.get("role").map(|s| s.as_str()).unwrap_or("Viewer");
        let initial = name.chars().next().unwrap_or('?').to_uppercase().to_string();

        let (role_bg, role_color) = match role.to_lowercase().as_str() {
            "admin" => ("#1a1a1a", "white"),
            "developer" | "dev" => ("#dbeafe", "#1d4ed8"),
            _ => ("#f3f3f3", "#5e5e5e"),
        };

        format!(
            r#"<div style="display:flex;align-items:center;gap:12px;padding:12px 0;border-bottom:1px solid #f3f3f3">
  <div style="width:40px;height:40px;border-radius:50%;background:#e8e8e8;display:flex;align-items:center;justify-content:center;flex-shrink:0">
    <span style="font-size:14px;font-weight:600;color:#5e5e5e">{initial}</span>
  </div>
  <span style="font-size:14px;font-weight:600;color:#1a1c1c;flex:1">{name}</span>
  <span style="font-size:14px;color:#5e5e5e;flex:1">{email}</span>
  <span style="display:inline-block;padding:3px 10px;border-radius:999px;font-size:11px;font-weight:500;background:{role_bg};color:{role_color}">{role}</span>
</div>"#,
            initial = initial, name = name, email = email,
            role = role, role_bg = role_bg, role_color = role_color,
        )
    }).collect();

    let footer_html = footer_link.map(|l| format!(
        r##"<div style="padding-top:16px;margin-top:8px">
  <a href="#" style="font-size:13px;color:#5e5e5e;text-decoration:none">{}</a>
</div>"##, l
    )).unwrap_or_default();

    format!(
        r#"<section style="max-width:1280px;margin:0 auto;padding:48px 24px">
  <div style="background:white;border:1px solid rgba(198,198,198,0.2);border-radius:12px;padding:24px">
    <h3 style="font-size:16px;font-weight:700;color:#1a1c1c;margin-bottom:16px">{title}{badge_html}</h3>
    {rows}
    {footer_html}
  </div>
</section>"#,
        title = title, badge_html = badge_html,
        rows = rows.join("\n    "),
        footer_html = footer_html,
    )
}

pub(super) fn render_policies(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("Policies");

    let rows: Vec<String> = section.items.iter().map(|item| {
        let name = item.get("title").or_else(|| item.get("name")).map(|s| s.as_str()).unwrap_or("Policy");
        let value = item.get("description").or_else(|| item.get("desc")).map(|s| s.as_str()).unwrap_or("");

        let value_html = match value.to_lowercase().as_str() {
            "on" => r#"<div style="width:36px;height:20px;border-radius:10px;background:#047857;position:relative"><div style="width:16px;height:16px;border-radius:50%;background:white;position:absolute;top:2px;right:2px"></div></div>"#.to_string(),
            "off" => r#"<div style="width:36px;height:20px;border-radius:10px;background:#d1d5db;position:relative"><div style="width:16px;height:16px;border-radius:50%;background:white;position:absolute;top:2px;left:2px"></div></div>"#.to_string(),
            _ => format!(r#"<span style="font-size:14px;font-weight:600;color:#1a1c1c">{}</span>"#, value),
        };

        format!(
            r#"<div style="display:flex;align-items:center;justify-content:space-between;padding:12px 0;border-bottom:1px solid #f3f3f3">
  <span style="font-size:14px;color:#1a1c1c">{name}</span>
  {value_html}
</div>"#,
            name = name, value_html = value_html,
        )
    }).collect();

    format!(
        r#"<section style="max-width:1280px;margin:0 auto;padding:48px 24px">
  <div style="background:white;border:1px solid rgba(198,198,198,0.2);border-radius:12px;padding:24px">
    <h3 style="font-size:16px;font-weight:700;color:#1a1c1c;margin-bottom:16px">{title}</h3>
    {rows}
  </div>
</section>"#,
        title = title, rows = rows.join("\n    "),
    )
}

pub(super) fn render_activity_table(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("Activity Log");
    let action_btn = section.config.get("action_text").map(|s| s.as_str());

    let action_html = action_btn.map(|t| format!(
        r#"<button style="padding:6px 16px;border-radius:8px;border:1px solid rgba(198,198,198,0.2);background:white;font-size:13px;font-weight:500;color:#1a1c1c;cursor:pointer">{}</button>"#, t
    )).unwrap_or_default();

    let default_headers = vec!["Event", "User", "Location", "IP Address", "Time", "Status"];
    let headers: Vec<&str> = section.config.get("columns").map(|c| {
        c.split(',').map(|s| s.trim()).collect::<Vec<&str>>()
    }).unwrap_or(default_headers);

    let header_cells: Vec<String> = headers.iter().map(|h| format!(
        r#"<th style="padding:12px 24px;text-align:left;font-size:12px;font-weight:500;text-transform:uppercase;letter-spacing:0.05em;color:#5e5e5e;border-bottom:1px solid #f3f3f3">{}</th>"#, h
    )).collect();

    let rows: Vec<String> = section.items.iter().map(|item| {
        let event = item.get("title").or_else(|| item.get("name")).map(|s| s.as_str()).unwrap_or("Event");
        let user = item.get("user").map(|s| s.as_str()).unwrap_or("-");
        let location = item.get("location").map(|s| s.as_str()).unwrap_or("-");
        let ip = item.get("ip").map(|s| s.as_str()).unwrap_or("-");
        let time = item.get("time").map(|s| s.as_str()).unwrap_or("-");
        let status = item.get("status").map(|s| s.as_str()).unwrap_or("");

        let (status_bg, status_color) = match status.to_lowercase().as_str() {
            "success" | "ok" => ("#ecfdf5", "#047857"),
            "blocked" | "failed" | "error" => ("#fef2f2", "#b91c1c"),
            _ => ("#f3f3f3", "#5e5e5e"),
        };

        let status_html = if status.is_empty() {
            String::new()
        } else {
            format!(
                r#"<span style="display:inline-block;padding:3px 10px;border-radius:999px;font-size:11px;font-weight:500;background:{};color:{}">{}</span>"#,
                status_bg, status_color, status
            )
        };

        format!(
            r#"<tr style="border-bottom:1px solid #f3f3f3">
  <td style="padding:16px 24px;font-size:14px;color:#1a1c1c;font-weight:500">{event}</td>
  <td style="padding:16px 24px;font-size:14px;color:#5e5e5e">{user}</td>
  <td style="padding:16px 24px;font-size:14px;color:#5e5e5e">{location}</td>
  <td style="padding:16px 24px;font-size:14px;color:#5e5e5e;font-family:monospace">{ip}</td>
  <td style="padding:16px 24px;font-size:14px;color:#5e5e5e">{time}</td>
  <td style="padding:16px 24px">{status_html}</td>
</tr>"#,
            event = event, user = user, location = location,
            ip = ip, time = time, status_html = status_html,
        )
    }).collect();

    format!(
        r#"<section style="max-width:1280px;margin:0 auto;padding:48px 24px">
  <div style="background:white;border:1px solid rgba(198,198,198,0.2);border-radius:12px;overflow:hidden">
    <div style="display:flex;align-items:center;justify-content:space-between;padding:24px">
      <h3 style="font-size:16px;font-weight:700;color:#1a1c1c">{title}</h3>
      {action_html}
    </div>
    <table style="width:100%;border-collapse:collapse">
      <thead>
        <tr>{header_cells}</tr>
      </thead>
      <tbody>
        {rows}
      </tbody>
    </table>
  </div>
</section>"#,
        title = title, action_html = action_html,
        header_cells = header_cells.join(""),
        rows = rows.join("\n        "),
    )
}

pub(super) fn render_status_card(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("Status");
    let description = section.subtitle.as_deref().unwrap_or("");
    let icon_type = section.config.get("icon").map(|s| s.as_str()).unwrap_or("shield");

    let icon_svg = match icon_type {
        "shield" | "security" => r##"<svg width="24" height="24" fill="none" stroke="#047857" stroke-width="1.5" viewBox="0 0 24 24"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>"##,
        "check" => r##"<svg width="24" height="24" fill="none" stroke="#047857" stroke-width="2" viewBox="0 0 24 24"><path d="M20 6L9 17l-5-5"/></svg>"##,
        "globe" => r##"<svg width="24" height="24" fill="none" stroke="#047857" stroke-width="1.5" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/><path d="M2 12h20M12 2a15.3 15.3 0 014 10 15.3 15.3 0 01-4 10 15.3 15.3 0 01-4-10 15.3 15.3 0 014-10z"/></svg>"##,
        _ => r##"<svg width="24" height="24" fill="none" stroke="#047857" stroke-width="1.5" viewBox="0 0 24 24"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>"##,
    };

    let meters: Vec<String> = section.items.iter().map(|item| {
        let label = item.get("title").or_else(|| item.get("name")).map(|s| s.as_str()).unwrap_or("Metric");
        let value = item.get("description").or_else(|| item.get("desc")).or_else(|| item.get("value")).map(|s| s.as_str()).unwrap_or("");
        let status = item.get("status").map(|s| s.as_str()).unwrap_or("default");
        let bar_color = match status.to_lowercase().as_str() {
            "success" | "ok" | "good" => "#047857",
            "warning" => "#d97706",
            "danger" | "error" => "#b91c1c",
            _ => "#047857",
        };
        let pct: u32 = value.trim_end_matches('%').parse().unwrap_or(0);

        format!(
            r#"<div style="margin-top:16px">
  <div style="display:flex;justify-content:space-between;margin-bottom:6px">
    <span style="font-size:13px;color:#5e5e5e">{label}</span>
    <span style="font-size:13px;font-weight:600;color:#1a1c1c">{value}%</span>
  </div>
  <div style="width:100%;height:6px;border-radius:3px;background:#f3f3f3">
    <div style="width:{pct}%;height:100%;border-radius:3px;background:{bar_color}"></div>
  </div>
</div>"#,
            label = label, value = pct, pct = pct, bar_color = bar_color,
        )
    }).collect();

    format!(
        r#"<section style="max-width:1280px;margin:0 auto;padding:48px 24px">
  <div style="background:white;border:1px solid rgba(198,198,198,0.2);border-radius:12px;padding:24px">
    <div style="width:48px;height:48px;border-radius:12px;background:#ecfdf5;display:flex;align-items:center;justify-content:center;margin-bottom:16px">
      {icon_svg}
    </div>
    <h3 style="font-size:16px;font-weight:700;color:#1a1c1c;margin-bottom:4px">{title}</h3>
    <p style="font-size:14px;color:#5e5e5e;line-height:1.6">{description}</p>
    {meters}
  </div>
</section>"#,
        icon_svg = icon_svg, title = title, description = description,
        meters = meters.join("\n    "),
    )
}

pub(super) fn render_edge(section: &SectionNode, accent: &str) -> String {
    let title_html = section.title.as_deref().map(|t| format!(
        r#"<h2 style="font-size:24px;font-weight:700;letter-spacing:-0.02em;color:#1a1c1c;margin-bottom:32px">{}</h2>"#, t
    )).unwrap_or_default();

    let feature_items: Vec<String> = section.items.iter().map(|item| {
        let name = item.get("title").or_else(|| item.get("name")).map(|s| s.as_str()).unwrap_or("Feature");
        let desc = item.get("description").or_else(|| item.get("desc")).map(|s| s.as_str()).unwrap_or("");
        let icon_letter = name.chars().next().unwrap_or('E');

        format!(
            r#"<div style="display:flex;gap:16px;align-items:flex-start">
  <div style="width:36px;height:36px;border-radius:50%;background:{accent};display:flex;align-items:center;justify-content:center;flex-shrink:0">
    <span style="color:white;font-size:14px;font-weight:700">{icon_letter}</span>
  </div>
  <div>
    <h4 style="font-size:16px;font-weight:600;color:#1a1c1c;margin-bottom:4px">{name}</h4>
    <p style="font-size:14px;color:#5e5e5e;line-height:1.6">{desc}</p>
  </div>
</div>"#,
            accent = accent, icon_letter = icon_letter, name = name, desc = desc,
        )
    }).collect();

    format!(
        r#"<section style="max-width:1280px;margin:0 auto;padding:48px 24px">
  {title_html}
  <div style="display:flex;gap:48px;align-items:flex-start">
    <div style="flex:1;display:flex;flex-direction:column;gap:24px">
      {features}
    </div>
    <div style="flex:1;min-height:300px;border-radius:12px;background:#f5f5f5;border:1px solid rgba(198,198,198,0.2);display:flex;align-items:center;justify-content:center">
      <span style="font-size:14px;color:#5e5e5e">Edge Network Map</span>
    </div>
  </div>
</section>"#,
        title_html = title_html,
        features = feature_items.join("\n      "),
    )
}

// ══════════════════════════════════════════════════
// DASHBOARD SECTION RENDERERS — Geist light design
// ══════════════════════════════════════════════════

pub(super) fn render_sidebar(section: &SectionNode) -> String {
    let style_variant = section.config.get("style").map(|s| s.as_str()).unwrap_or("");
    let is_dark = style_variant == "dark";

    let brand = section.config.get("brand")
        .map(|s| s.as_str())
        .or(section.title.as_deref())
        .unwrap_or("");
    let subtitle = section.config.get("subtitle")
        .map(|s| s.as_str())
        .or(section.subtitle.as_deref())
        .unwrap_or("");

    // Active page from config (matches against item title)
    let active_cfg = section.config.get("active").map(|s| s.as_str()).unwrap_or("");

    // Palette: use theme tokens for dark, hardcoded for light
    let t = crate::theme::get();
    let bg = if is_dark { &t.surface_container_lowest } else { "rgba(250,250,250,0.5)" };
    let border_color = if is_dark { format!("rgba(76,69,70,0.15)") } else { "rgba(228,228,231,1)".to_string() };
    let brand_color = if is_dark { &t.on_surface } else { "#000" };
    let subtitle_color = if is_dark { &t.primary } else { "#71717a" };
    let inactive_color_str = if is_dark { format!("{}66", t.on_surface) } else { "#71717a".to_string() };
    let inactive_color = inactive_color_str.as_str();
    let active_bg = if is_dark { &t.surface_container } else { "rgba(0,0,0,0.04)" };
    let active_text = if is_dark { &t.primary } else { "#000" };
    let hover_bg = if is_dark { &t.surface_container_low } else { "rgba(0,0,0,0.04)" };
    let hover_text = if is_dark { &t.on_surface } else { "#000" };

    // Resolve which item is active: explicit config > item marked active > first nav item
    let mut first_nav_title = String::new();
    let mut item_marked_active = String::new();
    for item in &section.items {
        let item_type = item.get("_type").map(|s| s.as_str()).unwrap_or("item");
        if item_type == "action" { continue; }
        let t = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let position = item.get("position").map(|s| s.as_str()).unwrap_or("top");
        if position != "bottom" && first_nav_title.is_empty() {
            first_nav_title = t.to_string();
        }
        if item.get("active").map(|s| s.as_str()) == Some("true") {
            item_marked_active = t.to_string();
        }
    }
    let resolved_active = if !active_cfg.is_empty() {
        active_cfg.to_string()
    } else if !item_marked_active.is_empty() {
        item_marked_active
    } else {
        first_nav_title
    };

    // Build nav links, action buttons, and bottom items
    let mut nav_items = String::new();
    let mut action_items = String::new();
    let mut bottom_items = String::new();
    for item in &section.items {
        let item_type = item.get("_type").map(|s| s.as_str()).unwrap_or("item");
        let title = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("");
        let href = item.get("href").map(|s| s.as_str()).unwrap_or("");
        let position = item.get("position").map(|s| s.as_str()).unwrap_or("top");
        let item_style = item.get("style").map(|s| s.as_str()).unwrap_or("");

        // Action / CTA button
        if item_type == "action" {
            let btn_bg = if item_style == "gradient" {
                "background:linear-gradient(135deg,#adc6ff 0%,#c4b5fd 100%)"
            } else if is_dark {
                "background:#adc6ff"
            } else {
                "background:#1a1c1c"
            };
            let btn_text = if is_dark || item_style == "gradient" { "#0e0e0e" } else { "#fff" };
            action_items.push_str(&format!(
                r#"<button style="width:100%;{btn_bg};color:{btn_text};border:none;border-radius:8px;padding:10px 16px;font-size:13px;font-weight:700;letter-spacing:0.05em;text-transform:uppercase;cursor:pointer;transition:opacity 0.15s;font-family:inherit;margin-top:8px" onmouseover="this.style.opacity='0.85'" onmouseout="this.style.opacity='1'">{title}</button>"#,
                btn_bg = btn_bg, btn_text = btn_text, title = title
            ));
            action_items.push('\n');
            continue;
        }

        let is_active = !resolved_active.is_empty() && title.eq_ignore_ascii_case(&resolved_active);

        let link_html = if is_active {
            format!(
                r#"<a href="{href}" style="display:flex;align-items:center;gap:12px;padding:8px 12px;background:{active_bg};color:{active_text};border-radius:8px;font-size:14px;font-weight:500;text-decoration:none">
  <span class="material-symbols-outlined" style="font-size:20px">{icon}</span>
  <span>{title}</span>
</a>"#,
                href = href, active_bg = active_bg, active_text = active_text,
                icon = icon, title = title
            )
        } else {
            format!(
                r#"<a href="{href}" style="display:flex;align-items:center;gap:12px;padding:8px 12px;color:{inactive};border-radius:8px;font-size:14px;font-weight:500;text-decoration:none;transition:all 0.15s" onmouseover="this.style.color='{h_text}';this.style.background='{h_bg}'" onmouseout="this.style.color='{inactive}';this.style.background='transparent'">
  <span class="material-symbols-outlined" style="font-size:20px">{icon}</span>
  <span>{title}</span>
</a>"#,
                href = href, inactive = inactive_color, h_text = hover_text,
                h_bg = hover_bg, icon = icon, title = title
            )
        };

        if position == "bottom" {
            bottom_items.push_str(&link_html);
            bottom_items.push('\n');
        } else {
            nav_items.push_str(&link_html);
            nav_items.push('\n');
        }
    }

    // Brand styling: dark uses uppercase tracking-widest font-black
    let brand_style = if is_dark {
        format!(
            "font-weight:900;letter-spacing:0.1em;text-transform:uppercase;color:{};font-size:16px;margin:0",
            brand_color
        )
    } else {
        format!(
            "font-weight:700;letter-spacing:-0.03em;color:{};font-size:20px;margin:0",
            brand_color
        )
    };

    let subtitle_style = format!(
        "font-size:11px;color:{};font-weight:500;letter-spacing:0.08em;text-transform:uppercase;margin:2px 0 0",
        subtitle_color
    );

    // Bottom section: only render if there are bottom items
    let bottom_html = if bottom_items.is_empty() {
        String::new()
    } else {
        format!(
            r#"<div style="margin-top:auto;border-top:0.5px solid {border};padding-top:16px;display:flex;flex-direction:column;gap:2px">
    {bottom}
  </div>"#,
            border = border_color, bottom = bottom_items
        )
    };

    format!(
        r##"<aside style="position:fixed;left:0;top:0;height:100vh;width:256px;background:{bg};border-right:0.5px solid {border};display:flex;flex-direction:column;z-index:50;padding:16px;gap:4px;font-family:'Inter',sans-serif;-webkit-font-smoothing:antialiased">
  <div style="margin-bottom:32px;padding:0 8px">
    <h1 style="{brand_style}">{brand}</h1>
    <p style="{subtitle_style}">{subtitle}</p>
  </div>
  <nav style="flex:1;display:flex;flex-direction:column;gap:2px">
    {nav_items}
    {action_items}
  </nav>
  {bottom_html}
</aside>"##,
        bg = bg,
        border = border_color,
        brand_style = brand_style,
        brand = brand,
        subtitle_style = subtitle_style,
        subtitle = subtitle,
        nav_items = nav_items,
        action_items = action_items,
        bottom_html = bottom_html,
    )
}

pub(super) fn render_card_section(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("");
    let subtitle = section.subtitle.as_deref().unwrap_or("");
    let icon = section.config.get("icon").map(|s| s.as_str()).unwrap_or("");

    // Header
    let mut header_html = String::new();
    if !icon.is_empty() || !title.is_empty() || !subtitle.is_empty() {
        header_html.push_str(r#"<div style="display:flex;align-items:flex-start;gap:12px;margin-bottom:24px">"#);
        if !icon.is_empty() {
            header_html.push_str(&format!(
                r#"<span class="material-symbols-outlined" style="font-size:24px;color:#474747">{}</span>"#,
                icon
            ));
        }
        header_html.push_str("<div>");
        if !title.is_empty() {
            header_html.push_str(&format!(
                r#"<h2 style="font-size:18px;font-weight:700;color:#1a1c1c;margin:0;letter-spacing:-0.02em">{}</h2>"#,
                title
            ));
        }
        if !subtitle.is_empty() {
            header_html.push_str(&format!(
                r#"<p style="font-size:14px;color:#71717a;margin:4px 0 0">{}</p>"#,
                subtitle
            ));
        }
        header_html.push_str("</div></div>");
    }

    // Items
    let mut items_html = String::new();
    for item in &section.items {
        let item_type = item.get("_type").map(|s| s.as_str()).unwrap_or("item");
        let item_title = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let desc = item.get("description").map(|s| s.as_str()).unwrap_or("");
        let value = item.get("value").map(|s| s.as_str()).unwrap_or("");
        let status = item.get("status").map(|s| s.as_str()).unwrap_or("");
        let href = item.get("href").map(|s| s.as_str()).unwrap_or("");

        match item_type {
            "label" => {
                items_html.push_str(&format!(
                    r#"<div style="margin-bottom:12px">
  <span style="font-size:11px;font-weight:600;letter-spacing:0.08em;text-transform:uppercase;color:#71717a;font-family:'JetBrains Mono',monospace">{}</span>
</div>"#,
                    item_title
                ));
            }
            "code" => {
                items_html.push_str(&format!(
                    r#"<div style="margin-bottom:12px">
  <label style="font-size:12px;font-weight:500;color:#474747;display:block;margin-bottom:4px">{title}</label>
  <div style="background:#f3f3f3;border:1px solid rgba(198,198,198,0.2);border-radius:8px;padding:10px 14px;font-family:'JetBrains Mono',monospace;font-size:13px;color:#1a1c1c;user-select:all">{value}</div>
</div>"#,
                    title = item_title,
                    value = if !value.is_empty() { value } else { desc },
                ));
            }
            "action" => {
                let on_click = item.get("on_click");
                if let Some(action_json) = on_click {
                    // Action button with data attributes for the runtime action system
                    let entity_attr = section.config.get("entity")
                        .or_else(|| section.binding.as_ref().map(|b| &b.entity))
                        .map(|e| format!(r#" data-cronus-entity="{}""#, e))
                        .unwrap_or_default();
                    let section_attr = format!(r#" data-cronus-section="{}""#, section.section_type);
                    let confirm_attr = item.get("confirm")
                        .map(|c| format!(r#" data-cronus-confirm="{}""#, c))
                        .unwrap_or_default();
                    let escaped_json = action_json.replace('"', "&quot;");
                    items_html.push_str(&format!(
                        r#"<div style="margin-top:8px">
  <button type="button" data-cronus-action="{action_json}"{entity_attr}{section_attr}{confirm_attr} style="display:inline-flex;align-items:center;gap:6px;padding:8px 20px;font-size:14px;font-weight:600;color:#fff;background:#1a1c1c;border-radius:8px;border:none;cursor:pointer;transition:opacity 0.15s;font-family:inherit" onmouseover="this.style.opacity='0.85'" onmouseout="this.style.opacity='1'">{title}</button>
</div>"#,
                        action_json = escaped_json,
                        entity_attr = entity_attr,
                        section_attr = section_attr,
                        confirm_attr = confirm_attr,
                        title = item_title,
                    ));
                } else {
                    // Regular link action
                    items_html.push_str(&format!(
                        r#"<div style="margin-top:8px">
  <a href="{href}" style="display:inline-flex;align-items:center;gap:6px;padding:8px 20px;font-size:14px;font-weight:600;color:#fff;background:#1a1c1c;border-radius:8px;text-decoration:none;transition:opacity 0.15s" onmouseover="this.style.opacity='0.85'" onmouseout="this.style.opacity='1'">{title}</a>
</div>"#,
                        href = href, title = item_title,
                    ));
                }
            }
            "row" => {
                let status_badge = if !status.is_empty() {
                    let (bg, fg) = match status {
                        "active" | "enabled" | "200" => ("rgba(22,163,74,0.1)", "#16a34a"),
                        "error" | "failed" | "500" => ("rgba(220,38,38,0.1)", "#dc2626"),
                        "pending" | "disabled" => ("rgba(234,179,8,0.1)", "#ca8a04"),
                        _ => ("rgba(0,0,0,0.05)", "#71717a"),
                    };
                    format!(
                        r#"<span style="padding:2px 10px;font-size:12px;font-weight:500;border-radius:999px;background:{};color:{}">{}</span>"#,
                        bg, fg, status
                    )
                } else {
                    String::new()
                };
                items_html.push_str(&format!(
                    r#"<div style="display:flex;justify-content:space-between;align-items:center;padding:12px 0;border-bottom:1px solid rgba(198,198,198,0.12)">
  <div>
    <span style="font-size:14px;font-weight:500;color:#1a1c1c">{title}</span>
    {desc_html}
  </div>
  {status_badge}
</div>"#,
                    title = item_title,
                    desc_html = if !desc.is_empty() {
                        format!(r#"<span style="font-size:13px;color:#71717a;margin-left:12px">{}</span>"#, desc)
                    } else {
                        String::new()
                    },
                    status_badge = status_badge,
                ));
            }
            _ => {
                // Default item rendering
                if !item_title.is_empty() {
                    items_html.push_str(&format!(
                        r#"<div style="padding:8px 0"><span style="font-size:14px;color:#1a1c1c">{}</span></div>"#,
                        item_title
                    ));
                }
            }
        }
    }

    format!(
        r#"<div style="background:#fff;border:1px solid rgba(198,198,198,0.2);border-radius:16px;padding:32px">
  {header}
  {items}
</div>"#,
        header = header_html,
        items = items_html,
    )
}

pub(super) fn render_links_section(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("");

    let mut links_html = String::new();
    for item in &section.items {
        let link_title = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let href = item.get("href").map(|s| s.as_str()).unwrap_or("");
        let desc = item.get("description").map(|s| s.as_str()).unwrap_or("");

        links_html.push_str(&format!(
            r#"<a href="{href}" target="_blank" rel="noopener" style="display:flex;align-items:center;justify-content:space-between;padding:12px 0;border-bottom:1px solid rgba(198,198,198,0.12);text-decoration:none;transition:opacity 0.15s" onmouseover="this.style.opacity='0.7'" onmouseout="this.style.opacity='1'">
  <div>
    <span style="font-size:14px;font-weight:500;color:#1a1c1c">{title}</span>
    {desc_html}
  </div>
  <span class="material-symbols-outlined" style="font-size:18px;color:#71717a">open_in_new</span>
</a>"#,
            href = href,
            title = link_title,
            desc_html = if !desc.is_empty() {
                format!(r#"<p style="font-size:13px;color:#71717a;margin:2px 0 0">{}</p>"#, desc)
            } else {
                String::new()
            },
        ));
    }

    format!(
        r#"<div style="background:#fff;border:1px solid rgba(198,198,198,0.2);border-radius:16px;padding:32px">
  {title_html}
  <div>{links}</div>
</div>"#,
        title_html = if !title.is_empty() {
            format!(r#"<h2 style="font-size:18px;font-weight:700;color:#1a1c1c;margin:0 0 20px;letter-spacing:-0.02em">{}</h2>"#, title)
        } else {
            String::new()
        },
        links = links_html,
    )
}

pub(super) fn render_modal_section(section: &SectionNode) -> String {
    let t = crate::theme::get();
    let title = section.title.as_deref().unwrap_or("Dialog");
    let subtitle = section.subtitle.as_deref().unwrap_or("");
    let icon = section.config.get("icon").map(|s| s.as_str()).unwrap_or("");
    let modal_id = section.config.get("id").cloned()
        .unwrap_or_else(|| format!("modal-{}", title.to_lowercase().replace(' ', "-")));
    let trigger = section.config.get("trigger").map(|s| s.as_str()).unwrap_or("");
    let is_dark = t.surface.contains("0e0e0e") || t.surface.contains("000") || t.on_surface.contains("fff");

    // Theme tokens
    let (backdrop_bg, panel_bg, panel_border, panel_shadow,
         label_color, input_bg, input_border, input_focus,
         text_color, text_muted, summary_bg,
         btn_secondary_bg, btn_secondary_border, btn_secondary_text,
         btn_primary_bg, btn_primary_text) = if is_dark {
        (
            "rgba(0,0,0,0.6)", // backdrop
            "rgba(25,25,25,0.85)", // panel
            "0.5px solid rgba(72,72,72,0.15)", // panel border
            "0 0 60px rgba(135,173,255,0.08)", // shadow glow
            "rgba(226,226,226,0.4)", // label
            "#000000", // input bg
            "rgba(72,72,72,0.3)", // input border
            "rgba(135,173,255,0.5)", // input focus
            "#ffffff", // text
            "rgba(226,226,226,0.4)", // muted
            "rgba(31,31,31,0.5)", // summary bg
            "#262626", // btn secondary bg
            "0.5px solid rgba(72,72,72,0.2)", // btn secondary border
            "#ffffff", // btn secondary text
            "linear-gradient(135deg,#87adff,#d277ff)", // btn primary bg (gradient)
            "#ffffff", // btn primary text
        )
    } else {
        (
            "rgba(0,0,0,0.4)",
            "#ffffff",
            "1px solid #e5e7eb",
            "0 24px 48px rgba(0,0,0,0.15)",
            "#71717a",
            "#f9fafb",
            "#e5e7eb",
            "#2563eb",
            "#1a1a1a",
            "#71717a",
            "#f3f4f6",
            "#ffffff",
            "1px solid #e5e7eb",
            "#1a1a1a",
            "#000000",
            "#ffffff",
        )
    };

    let label_style = format!("display:block;font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:0.15em;color:{};margin-bottom:8px;margin-left:2px", label_color);
    let input_style = format!("width:100%;padding:14px 16px;background:{};border:1px solid {};border-radius:8px;font-size:14px;outline:none;font-family:Inter,sans-serif;transition:all 0.2s;box-sizing:border-box;color:{}", input_bg, input_border, text_color);

    let mut fields_html = String::new();
    let mut actions_html = String::new();
    let mut summary_html = String::new();

    for item in &section.items {
        let itype = item.get("_type").map(|s| s.as_str()).unwrap_or("");
        let item_title = item.get("title").map(|s| s.as_str()).unwrap_or("");

        if itype == "field" {
            let ftype = item.get("type").map(|s| s.as_str()).unwrap_or("text");
            let name_lower = item_title.to_lowercase().replace(' ', "_");
            let placeholder = item.get("placeholder").map(|s| s.as_str()).unwrap_or("");
            let value = item.get("value").map(|s| s.as_str()).unwrap_or("");
            let required = if item.get("required").map(|s| s == "true").unwrap_or(false) { "required" } else { "" };
            let readonly = if item.get("readonly").map(|s| s == "true").unwrap_or(false) || !value.is_empty() { "readonly" } else { "" };
            let span = item.get("span").map(|s| s.as_str()).unwrap_or("1");

            let grid_col = if span != "1" { format!("grid-column:span {}", span) } else { String::new() };

            match ftype {
                "select" => {
                    let options_raw = item.get("options").map(|s| s.as_str()).unwrap_or("");
                    let options: Vec<&str> = if options_raw.is_empty() { vec![] } else { options_raw.split("||").collect() };
                    let mut opts_html = String::new();
                    for opt in &options {
                        opts_html.push_str(&format!(r#"<option value="{v}">{v}</option>"#, v = opt));
                    }
                    fields_html.push_str(&format!(
                        r#"<div style="{gc}"><label style="{ls}">{label}</label><select name="{name}" style="{is};appearance:none;cursor:pointer;background-image:url('data:image/svg+xml,<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"12\" height=\"12\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"%23888\" stroke-width=\"2\"><path d=\"M6 9l6 6 6-6\"/></svg>');background-repeat:no-repeat;background-position:right 14px center" onfocus="this.style.borderColor='{fc}';this.style.boxShadow='0 0 0 3px {fc}33'" onblur="this.style.borderColor='{ib}';this.style.boxShadow='none'">{opts}</select></div>"#,
                        gc = grid_col, ls = label_style, label = item_title, name = name_lower,
                        is = input_style, opts = opts_html, fc = input_focus, ib = input_border,
                    ));
                }
                "textarea" => {
                    let rows = item.get("rows").map(|s| s.as_str()).unwrap_or("3");
                    fields_html.push_str(&format!(
                        r#"<div style="{gc}"><label style="{ls}">{label}</label><textarea name="{name}" rows="{rows}" placeholder="{ph}" {req} style="{is};resize:none" onfocus="this.style.borderColor='{fc}';this.style.boxShadow='0 0 0 3px {fc}33'" onblur="this.style.borderColor='{ib}';this.style.boxShadow='none'"></textarea></div>"#,
                        gc = grid_col, ls = label_style, label = item_title, name = name_lower,
                        rows = rows, ph = placeholder, req = required, is = input_style,
                        fc = input_focus, ib = input_border,
                    ));
                }
                "checkbox" => {
                    fields_html.push_str(&format!(
                        r#"<label style="display:flex;align-items:center;gap:12px;cursor:pointer;{gc}"><input type="checkbox" name="{name}" style="width:18px;height:18px;accent-color:{fc}"><span style="font-size:14px;color:{tc}">{label}</span></label>"#,
                        gc = grid_col, name = name_lower, label = item_title, fc = input_focus, tc = text_color,
                    ));
                }
                _ => {
                    let val_attr = if !value.is_empty() { format!(r#"value="{}""#, value) } else { String::new() };
                    let copy_icon = if !readonly.is_empty() && !value.is_empty() {
                        format!(r#"<span class="material-symbols-outlined" style="position:absolute;right:14px;top:50%;transform:translateY(-50%);font-size:16px;color:{};cursor:pointer">content_copy</span>"#, label_color)
                    } else { String::new() };
                    let wrapper = if !copy_icon.is_empty() { "position:relative" } else { "" };
                    fields_html.push_str(&format!(
                        r#"<div style="{gc};{wr}"><label style="{ls}">{label}</label><div style="position:relative"><input type="{ftype}" name="{name}" placeholder="{ph}" {val} {req} {ro} style="{is}" onfocus="this.style.borderColor='{fc}';this.style.boxShadow='0 0 0 3px {fc}33'" onblur="this.style.borderColor='{ib}';this.style.boxShadow='none'">{copy}</div></div>"#,
                        gc = grid_col, wr = wrapper, ls = label_style, label = item_title, ftype = ftype,
                        name = name_lower, ph = placeholder, val = val_attr, req = required, ro = readonly,
                        is = input_style, fc = input_focus, ib = input_border, copy = copy_icon,
                    ));
                }
            }
        } else if itype == "action" {
            let variant = item.get("variant").map(|s| s.as_str())
                .or_else(|| item.get("style").map(|s| s.as_str()))
                .unwrap_or("primary");

            if variant == "secondary" || variant == "outline" || variant == "cancel" {
                let onclick = format!(r#"onclick="cronusModal.close('{}')" type="button""#, modal_id);
                actions_html.push_str(&format!(
                    r#"<button {onclick} style="flex:1;padding:14px;border:{bdr};border-radius:8px;background:{bg};color:{clr};font-size:12px;font-weight:700;cursor:pointer;font-family:'Space Grotesk',sans-serif;letter-spacing:0.1em;text-transform:uppercase;transition:all 0.15s" onmouseover="this.style.opacity='0.8'" onmouseout="this.style.opacity='1'">{label}</button>"#,
                    onclick = onclick, bdr = btn_secondary_border, bg = btn_secondary_bg,
                    clr = btn_secondary_text, label = item_title,
                ));
            } else {
                let glow = if is_dark { "box-shadow:0 0 20px rgba(135,173,255,0.25);" } else { "" };
                actions_html.push_str(&format!(
                    r#"<button type="submit" style="flex:1.5;padding:14px;border:none;border-radius:8px;background:{bg};color:{clr};font-size:12px;font-weight:700;cursor:pointer;font-family:'Space Grotesk',sans-serif;letter-spacing:0.12em;text-transform:uppercase;transition:all 0.15s;{glow}" onmouseover="this.style.filter='brightness(1.1)'" onmouseout="this.style.filter='none'">{label}</button>"#,
                    bg = btn_primary_bg, clr = btn_primary_text, glow = glow, label = item_title,
                ));
            }
        } else if itype == "summary" || itype == "row" {
            let val = item.get("value").map(|s| s.as_str()).unwrap_or("");
            let val_color = item.get("color").map(|s| s.as_str()).unwrap_or(text_muted);
            summary_html.push_str(&format!(
                r#"<div style="display:flex;justify-content:space-between;font-size:12px"><span style="color:{tm}">{label}</span><span style="color:{vc};font-weight:500">{val}</span></div>"#,
                tm = text_muted, label = item_title, vc = val_color, val = val,
            ));
        }
    }

    // Wrap fields in grid if any have span
    let has_grid = section.items.iter().any(|i| i.get("span").is_some());
    let fields_wrapper = if has_grid {
        format!(r#"<div style="display:grid;grid-template-columns:repeat(2,1fr);gap:20px">{}</div>"#, fields_html)
    } else {
        format!(r#"<div style="display:flex;flex-direction:column;gap:20px">{}</div>"#, fields_html)
    };

    let summary_block = if !summary_html.is_empty() {
        format!(r#"<div style="background:{};border-radius:8px;padding:16px;display:flex;flex-direction:column;gap:8px;margin-top:8px">{}</div>"#, summary_bg, summary_html)
    } else { String::new() };

    let subtitle_html = if subtitle.is_empty() {
        String::new()
    } else {
        format!(r#"<p style="font-size:13px;color:{};margin:4px 0 0">{}</p>"#, text_muted, subtitle)
    };

    let icon_html = if icon.is_empty() {
        String::new()
    } else {
        let (icon_bg, icon_color, icon_border) = if is_dark {
            ("rgba(135,173,255,0.1)", "#87adff", "1px solid rgba(135,173,255,0.2)")
        } else {
            ("rgba(37,99,235,0.1)", "#2563eb", "1px solid rgba(37,99,235,0.2)")
        };
        format!(r#"<div style="width:48px;height:48px;border-radius:50%;display:flex;align-items:center;justify-content:center;background:{};border:{};flex-shrink:0"><span class="material-symbols-outlined" style="font-size:24px;color:{}">{}</span></div>"#, icon_bg, icon_border, icon_color, icon)
    };

    // Gradient bar at bottom of modal
    let gradient_bar = if is_dark {
        r#"<div style="height:2px;width:100%;background:linear-gradient(90deg,rgba(135,173,255,0.3),rgba(210,119,255,0.3),rgba(135,173,255,0.3))"></div>"#
    } else { "" };

    // Entity binding — determines API endpoint
    let entity = section.config.get("entity").map(|s| s.as_str()).unwrap_or("");
    let entity_lower = entity.to_lowercase();
    let api_endpoint = if !entity.is_empty() {
        format!("/api/{}s", entity_lower)
    } else {
        String::new()
    };

    // Form submit JS
    let form_submit = if !api_endpoint.is_empty() {
        format!(
            r#"onsubmit="return cronusModalSubmit(event,'{id}','{api}')" "#,
            id = modal_id, api = api_endpoint
        )
    } else {
        String::new()
    };

    // Auto-open trigger
    let auto_open = if trigger == "auto" || trigger == "open" {
        format!(r#"<script>document.getElementById('{}').style.display='flex'</script>"#, modal_id)
    } else { String::new() };

    format!(
        r##"<div id="{id}" style="display:none;position:fixed;inset:0;z-index:100;background:{backdrop};align-items:center;justify-content:center;backdrop-filter:blur(12px);-webkit-backdrop-filter:blur(12px);padding:24px" onclick="if(event.target===this)cronusModal.close('{id}')">
  <div style="background:{panel};backdrop-filter:blur(40px);-webkit-backdrop-filter:blur(40px);border-radius:16px;width:100%;max-width:540px;{border};box-shadow:{shadow};animation:scaleIn 0.3s cubic-bezier(0.16,1,0.3,1);overflow:hidden;background-image:linear-gradient(135deg,rgba(135,173,255,0.03),rgba(210,119,255,0.06))">
    <div style="padding:32px 40px 24px;border-bottom:1px solid rgba(72,72,72,0.08)">
      <div style="display:flex;justify-content:space-between;align-items:flex-start">
        <div>
          <h3 style="font-size:22px;font-weight:700;margin:0;color:{text};letter-spacing:-0.02em;font-family:'Space Grotesk',sans-serif">{title}</h3>
          {subtitle_html}
        </div>
        {icon_html}
      </div>
    </div>
    <form {form_submit}style="padding:32px 40px;display:flex;flex-direction:column;gap:24px">
      {fields}
      {summary}
      <div style="display:flex;gap:12px;padding-top:8px">
        {actions}
      </div>
    </form>
    {gradient_bar}
  </div>
</div>{auto_open}"##,
        id = modal_id, backdrop = backdrop_bg, panel = panel_bg, border = format!("border:{}", panel_border),
        shadow = panel_shadow, text = text_color,
        title = title, subtitle_html = subtitle_html, icon_html = icon_html,
        fields = fields_wrapper, summary = summary_block,
        actions = actions_html, gradient_bar = gradient_bar, auto_open = auto_open,
    )
}

pub(super) fn render_sheet_section(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("Details");
    let sheet_id = section.config.get("id").cloned()
        .unwrap_or_else(|| format!("sheet-{}", title.to_lowercase().replace(' ', "-")));
    let side = section.config.get("side").map(|s| s.as_str()).unwrap_or("right");
    let width = section.config.get("width").map(|s| s.as_str()).unwrap_or("400px");

    let (position_style, animation) = match side {
        "left" => ("left:0;top:0;bottom:0", "slideFromLeft"),
        _ => ("right:0;top:0;bottom:0", "slideFromRight"),
    };

    let mut content_html = String::new();
    let mut actions_html = String::new();

    for item in &section.items {
        let itype = item.get("_type").map(|s| s.as_str()).unwrap_or("");
        let item_title = item.get("title").map(|s| s.as_str()).unwrap_or("");

        if itype == "action" {
            let variant = item.get("variant").map(|s| s.as_str())
                .or_else(|| item.get("style").map(|s| s.as_str()))
                .unwrap_or("primary");
            let (bg, color, border) = match variant {
                "secondary" | "outline" => ("#fff", "#000", "1px solid #e5e7eb"),
                "danger" => ("#dc2626", "#fff", "none"),
                _ => ("#000", "#fff", "none"),
            };
            actions_html.push_str(&format!(
                r#"<button style="flex:1;padding:12px;border:{border};border-radius:999px;background:{bg};color:{color};font-size:14px;font-weight:700;cursor:pointer;font-family:Inter,sans-serif">{label}</button>"#,
                border = border, bg = bg, color = color, label = item_title,
            ));
        } else if itype == "field" {
            let ftype = item.get("type").map(|s| s.as_str()).unwrap_or("text");
            let name_lower = item_title.to_lowercase().replace(' ', "_");
            let placeholder = item.get("placeholder").map(|s| s.as_str()).unwrap_or("");
            let required = if item.get("required").map(|s| s == "true").unwrap_or(false) { "required" } else { "" };
            let label_style = "display:block;font-size:12px;font-weight:600;text-transform:uppercase;letter-spacing:0.1em;color:#71717a;margin-bottom:8px";
            let input_style = "width:100%;padding:12px 16px;border:1px solid #e5e7eb;border-radius:8px;font-size:14px;outline:none;font-family:Inter,sans-serif;box-sizing:border-box";
            content_html.push_str(&format!(
                r#"<div><label style="{ls}">{label}</label><input type="{ftype}" name="{name}" placeholder="{ph}" {req} style="{is}" onfocus="this.style.borderColor='#000'" onblur="this.style.borderColor='#e5e7eb'"></div>"#,
                ls = label_style, label = item_title, ftype = ftype, name = name_lower,
                ph = placeholder, req = required, is = input_style,
            ));
        } else {
            // Regular items rendered as key-value rows
            let desc = item.get("description").map(|s| s.as_str()).unwrap_or("");
            if !item_title.is_empty() {
                content_html.push_str(&format!(
                    r#"<div style="display:flex;justify-content:space-between;align-items:center;padding:12px 0;border-bottom:1px solid #f4f4f5">
  <span style="font-size:13px;color:#71717a;font-weight:500">{label}</span>
  <span style="font-size:14px;font-weight:600">{value}</span>
</div>"#,
                    label = item_title, value = desc,
                ));
            }
        }
    }

    let actions_block = if actions_html.is_empty() {
        String::new()
    } else {
        format!(r#"<div style="display:flex;gap:12px;margin-top:24px">{}</div>"#, actions_html)
    };

    format!(
        r##"<div id="{id}" style="display:none;position:fixed;inset:0;z-index:100;background:rgba(0,0,0,0.3)" onclick="if(event.target===this)cronusModal.close('{id}')">
  <div style="position:absolute;{pos};width:{width};background:#fff;box-shadow:-8px 0 24px rgba(0,0,0,0.1);padding:32px;animation:{anim} 0.3s cubic-bezier(0.16,1,0.3,1);overflow-y:auto">
    <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:24px">
      <h3 style="font-size:20px;font-weight:700;margin:0">{title}</h3>
      <button onclick="cronusModal.close('{id}')" style="background:none;border:none;cursor:pointer;padding:4px">
        <span class="material-symbols-outlined">close</span>
      </button>
    </div>
    <div style="display:flex;flex-direction:column;gap:4px">
      {content}
    </div>
    {actions}
  </div>
</div>"##,
        id = sheet_id, pos = position_style, width = width, anim = animation,
        title = title, content = content_html, actions = actions_block,
    )
}

pub(super) fn render_tabs_section(section: &SectionNode) -> String {
    // Group items into tabs: each "tab" _type starts a new group, subsequent "item" types belong to it
    let mut tabs: Vec<(String, Vec<&std::collections::HashMap<String, String>>)> = Vec::new();

    for item in &section.items {
        let item_type = item.get("_type").map(|s| s.as_str()).unwrap_or("item");
        let title = item.get("title").map(|s| s.as_str()).unwrap_or("");

        if item_type == "tab" {
            tabs.push((title.to_string(), Vec::new()));
        } else if let Some(last) = tabs.last_mut() {
            last.1.push(item);
        } else {
            // Items before any tab — create an implicit tab
            tabs.push(("Tab".to_string(), vec![item]));
        }
    }

    let mut buttons_html = String::new();
    let mut panels_html = String::new();

    for (i, (tab_title, tab_items)) in tabs.iter().enumerate() {
        let is_active = i == 0;
        let (color, border_color, weight) = if is_active {
            ("#000", "#000", "600")
        } else {
            ("#71717a", "transparent", "500")
        };

        buttons_html.push_str(&format!(
            r#"<button class="cronus-tab" data-tab="{i}" style="padding:12px 24px;font-size:14px;font-weight:{weight};color:{color};border-bottom:2px solid {border_color};background:none;border-top:none;border-left:none;border-right:none;cursor:pointer;font-family:Inter,sans-serif;transition:all 0.2s">{title}</button>"#,
            i = i, weight = weight, color = color, border_color = border_color, title = tab_title,
        ));

        let display = if is_active { "block" } else { "none" };
        let anim = if is_active { " style=\"animation:fadeIn 0.3s ease-out\"" } else { "" };

        let mut content_html = String::new();
        for ti in tab_items {
            let ti_title = ti.get("title").map(|s| s.as_str()).unwrap_or("");
            let ti_desc = ti.get("description").map(|s| s.as_str()).unwrap_or("");
            let ti_icon = ti.get("icon").map(|s| s.as_str()).unwrap_or("");

            content_html.push_str(r#"<div style="background:#fff;border:1px solid rgba(198,198,198,0.2);border-radius:12px;padding:24px">"#);
            if !ti_icon.is_empty() {
                content_html.push_str(&format!(
                    r#"<span style="font-size:20px;margin-bottom:8px;display:block">{}</span>"#, ti_icon
                ));
            }
            if !ti_title.is_empty() {
                content_html.push_str(&format!(
                    r#"<h3 style="font-size:16px;font-weight:600;color:#1a1c1c;margin:0 0 6px">{}</h3>"#, ti_title
                ));
            }
            if !ti_desc.is_empty() {
                content_html.push_str(&format!(
                    r#"<p style="font-size:14px;color:#6e6e6e;margin:0;line-height:1.5">{}</p>"#, ti_desc
                ));
            }
            content_html.push_str("</div>");
        }

        panels_html.push_str(&format!(
            r#"<div class="cronus-tab-panel" data-panel="{i}" style="display:{display}"{anim}><div style="display:flex;flex-direction:column;gap:16px">{content}</div></div>"#,
            i = i, display = display, anim = anim, content = content_html,
        ));
    }

    format!(
        r#"<div style="margin:24px 0"><div style="display:flex;gap:0;border-bottom:1px solid #e5e7eb;margin-bottom:24px">{buttons}</div>{panels}</div>"#,
        buttons = buttons_html,
        panels = panels_html,
    )
}

pub(super) fn render_accordion_section(section: &SectionNode) -> String {
    let mut items_html = String::new();

    for item in &section.items {
        let title = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let description = item.get("description").map(|s| s.as_str()).unwrap_or("");

        items_html.push_str(&format!(
            r#"<div class="cronus-accordion"><button class="cronus-accordion-trigger" style="width:100%;display:flex;justify-content:space-between;align-items:center;padding:16px 20px;background:none;border:none;border-bottom:1px solid #e5e7eb;cursor:pointer;font-size:14px;font-weight:600;text-align:left;font-family:Inter,sans-serif"><span>{title}</span><span class="material-symbols-outlined" style="font-size:18px;transition:transform 0.2s">expand_more</span></button><div class="cronus-accordion-content" style="display:none;padding:16px 20px;font-size:14px;color:#5e5e5e;line-height:1.6;border-bottom:1px solid #e5e7eb">{desc}</div></div>"#,
            title = title,
            desc = description,
        ));
    }

    format!(
        r#"<div style="display:flex;flex-direction:column;border:1px solid #e5e7eb;border-radius:12px;overflow:hidden">{items}</div>"#,
        items = items_html,
    )
}

pub(super) fn render_breadcrumb_section(section: &SectionNode) -> String {
    let mut parts: Vec<String> = Vec::new();
    let total = section.items.len();

    for (i, item) in section.items.iter().enumerate() {
        let title = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let link = item.get("link").map(|s| s.as_str()).unwrap_or("");
        let is_last = i == total - 1;

        if i > 0 {
            parts.push(r#"<span style="color:#d4d4d8">/</span>"#.to_string());
        }

        if is_last || link.is_empty() {
            parts.push(format!(
                r#"<span style="color:#000;font-weight:500">{}</span>"#,
                title
            ));
        } else {
            parts.push(format!(
                r#"<a href="{}" style="color:#71717a;text-decoration:none;transition:color 0.2s" onmouseover="this.style.color='#000'" onmouseout="this.style.color='#71717a'">{}</a>"#,
                link, title
            ));
        }
    }

    format!(
        r#"<nav style="display:flex;align-items:center;gap:8px;padding:12px 0;font-size:14px" class="anim-fade">{}</nav>"#,
        parts.join("")
    )
}

// ── Route State Sections ──

pub(super) fn render_generic_section(section: &SectionNode, _accent: &str) -> String {
    let mut html = String::new();

    // --- Section container ---
    html.push_str(r#"<section style="padding:32px 24px;max-width:1280px;margin:0 auto">"#);

    // --- Badge (from config) ---
    if let Some(badge) = section.config.get("badge") {
        html.push_str(&format!(
            r#"<span style="display:inline-block;padding:4px 14px;font-size:12px;font-weight:600;border-radius:999px;background:#f3f3f3;color:#1a1c1c;margin-bottom:12px">{}</span>"#,
            badge
        ));
    }

    // --- Title ---
    if let Some(ref title) = section.title {
        html.push_str(&format!(
            r#"<h2 style="font-size:24px;font-weight:700;color:#1a1c1c;margin:0 0 4px">{}</h2>"#,
            title
        ));
    }

    // --- Subtitle ---
    if let Some(ref subtitle) = section.subtitle {
        html.push_str(&format!(
            r#"<p style="font-size:14px;color:#6e6e6e;margin:0 0 24px">{}</p>"#,
            subtitle
        ));
    }

    // --- Items container ---
    html.push_str(r#"<div style="display:flex;flex-direction:column;gap:16px">"#);

    for item in &section.items {
        let item_type = item.get("_type").map(|s| s.as_str()).unwrap_or("item");
        let title = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let description = item.get("description").map(|s| s.as_str()).unwrap_or("");
        let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("");
        let style_val = item.get("style").map(|s| s.as_str()).unwrap_or("");

        match item_type {
            // --- Card (default for "item" or empty) ---
            "item" | "" => {
                html.push_str(r#"<div style="background:#fff;border:1px solid rgba(198,198,198,0.2);border-radius:12px;padding:24px">"#);
                if !icon.is_empty() {
                    html.push_str(&format!(
                        r#"<span style="font-size:20px;margin-bottom:8px;display:block">{}</span>"#,
                        icon
                    ));
                }
                if !title.is_empty() {
                    html.push_str(&format!(
                        r#"<h3 style="font-size:16px;font-weight:600;color:#1a1c1c;margin:0 0 6px">{}</h3>"#,
                        title
                    ));
                }
                if !description.is_empty() {
                    html.push_str(&format!(
                        r#"<p style="font-size:14px;color:#6e6e6e;margin:0;line-height:1.5">{}</p>"#,
                        description
                    ));
                }
                html.push_str("</div>");
            }

            // --- Code / monospace label ---
            "code" | "label" if style_val == "mono" => {
                let text = if !title.is_empty() { title } else { description };
                html.push_str(&format!(
                    r#"<div style="background:#f3f3f3;font-family:'JetBrains Mono',monospace;font-size:13px;border-radius:8px;padding:12px 16px;color:#1a1c1c;white-space:pre-wrap">{}</div>"#,
                    text
                ));
            }
            "code" => {
                let text = if !title.is_empty() { title } else { description };
                html.push_str(&format!(
                    r#"<div style="background:#f3f3f3;font-family:'JetBrains Mono',monospace;font-size:13px;border-radius:8px;padding:12px 16px;color:#1a1c1c;white-space:pre-wrap">{}</div>"#,
                    text
                ));
            }
            "label" => {
                let text = if !title.is_empty() { title } else { description };
                html.push_str(&format!(
                    r#"<span style="font-size:13px;color:#6e6e6e">{}</span>"#,
                    text
                ));
            }

            // --- Chip (pill badge) ---
            "chip" => {
                html.push_str(&format!(
                    r#"<span style="display:inline-block;padding:4px 14px;font-size:12px;font-weight:500;border-radius:999px;background:#f3f3f3;color:#1a1c1c">{}</span>"#,
                    title
                ));
            }

            // --- Image ---
            "image" => {
                let src = item.get("src").map(|s| s.as_str()).unwrap_or("");
                let alt = if !title.is_empty() { title } else { "image" };
                if !src.is_empty() {
                    html.push_str(&format!(
                        r#"<img src="{}" alt="{}" style="max-width:100%;border-radius:12px;border:1px solid rgba(198,198,198,0.2)" />"#,
                        src, alt
                    ));
                }
            }

            // --- Action (button) ---
            "action" => {
                let href = item.get("href").map(|s| s.as_str()).unwrap_or("");
                html.push_str(&format!(
                    r#"<a href="{}" style="display:inline-block;padding:10px 24px;font-size:14px;font-weight:600;color:#fff;background:#1a1c1c;border-radius:999px;text-decoration:none;text-align:center">{}</a>"#,
                    href, title
                ));
            }

            // --- Row (table-like) ---
            "row" => {
                html.push_str(r#"<div style="display:flex;justify-content:space-between;align-items:center;padding:12px 16px;background:#fff;border:1px solid rgba(198,198,198,0.2);border-radius:8px">"#);
                html.push_str(&format!(
                    r#"<span style="font-size:14px;color:#1a1c1c;font-weight:500">{}</span>"#,
                    title
                ));
                if !description.is_empty() {
                    html.push_str(&format!(
                        r#"<span style="font-size:13px;color:#6e6e6e">{}</span>"#,
                        description
                    ));
                }
                html.push_str("</div>");
            }

            // --- Policy (key-value) ---
            "policy" => {
                let key = item.get("key").map(|s| s.as_str()).unwrap_or(title);
                let value = item.get("value").map(|s| s.as_str()).unwrap_or(description);
                html.push_str(r#"<div style="display:flex;justify-content:space-between;align-items:baseline;padding:10px 0;border-bottom:1px solid rgba(198,198,198,0.12)">"#);
                html.push_str(&format!(
                    r#"<span style="font-size:14px;color:#1a1c1c">{}</span>"#,
                    key
                ));
                html.push_str(&format!(
                    r#"<span style="font-size:14px;color:#6e6e6e">{}</span>"#,
                    value
                ));
                html.push_str("</div>");
            }

            // --- Link ---
            "link" => {
                let href = item.get("href").map(|s| s.as_str()).unwrap_or("");
                html.push_str(&format!(
                    r#"<a href="{}" style="font-size:14px;color:#1a1c1c;text-decoration:underline;text-underline-offset:3px">{}</a>"#,
                    href, if !title.is_empty() { title } else { href }
                ));
            }

            // --- Default: simple text ---
            _ => {
                let text = if !title.is_empty() { title } else { description };
                if !text.is_empty() {
                    html.push_str(&format!(
                        r#"<p style="font-size:14px;color:#1a1c1c;margin:0">{}</p>"#,
                        text
                    ));
                }
            }
        }
    }

    html.push_str("</div>");
    html.push_str("</section>");
    html
}

// ══════════════════════════════════════════════════
// ALERT SECTION
// ══════════════════════════════════════════════════

pub(super) fn render_alert_section(section: &SectionNode) -> String {
    let style = section.config.get("style").map(|s| s.as_str()).unwrap_or("info");
    let title = section.title.as_deref().unwrap_or("Alert");
    let subtitle = section.subtitle.as_deref().unwrap_or("");

    let (bg_color, border_color, text_color, icon) = match style {
        "success" => ("#f0fdf4", "#bbf7d0", "#16a34a", "check_circle"),
        "error"   => ("#fef2f2", "#fecaca", "#dc2626", "error"),
        "warning" => ("#fffbeb", "#fde68a", "#d97706", "warning"),
        _         => ("#eff6ff", "#bfdbfe", "#2563eb", "info"),
    };

    format!(
        r##"<div class="anim-slide-up" style="display:flex;align-items:flex-start;gap:12px;padding:16px 20px;border-radius:12px;border:1px solid {border_color};background:{bg_color}">
  <span class="material-symbols-outlined" style="font-size:20px;color:{text_color};flex-shrink:0;margin-top:1px">{icon}</span>
  <div>
    <h4 style="font-size:14px;font-weight:600;color:{text_color};margin:0 0 4px">{title}</h4>
    <p style="font-size:13px;color:{text_color};opacity:0.8;margin:0;line-height:1.5">{subtitle}</p>
  </div>
  <button onclick="this.parentElement.style.display='none'" style="margin-left:auto;background:none;border:none;cursor:pointer;color:{text_color};opacity:0.5;padding:4px">
    <span class="material-symbols-outlined" style="font-size:16px">close</span>
  </button>
</div>"##,
        bg_color = bg_color,
        border_color = border_color,
        text_color = text_color,
        icon = icon,
        title = title,
        subtitle = subtitle,
    )
}

// ══════════════════════════════════════════════════
// CHART SECTION
// ══════════════════════════════════════════════════

pub(super) fn render_skeleton_section(section: &SectionNode) -> String {
    let cols: usize = section.config.get("cols")
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);
    let rows: usize = section.config.get("rows")
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);
    let height = section.config.get("height").map(|s| s.as_str()).unwrap_or("120px");
    let title = section.title.as_deref().unwrap_or("");
    let subtitle = section.subtitle.as_deref().unwrap_or("");

    let title_html = if title.is_empty() {
        String::new()
    } else {
        let sub = if subtitle.is_empty() {
            String::new()
        } else {
            format!(r#"<p style="font-size:14px;color:#a1a1aa;margin:4px 0 0">{}</p>"#, subtitle)
        };
        format!(r#"<div style="margin-bottom:24px"><h2 style="font-size:20px;font-weight:700;letter-spacing:-0.02em;margin:0">{}</h2>{}</div>"#, title, sub)
    };

    let total = cols * rows;
    let mut blocks = String::new();
    for _ in 0..total {
        blocks.push_str(&format!(
            r#"<div style="background:#f3f3f3;border-radius:8px;height:{};animation:pulse 2s cubic-bezier(0.4,0,0.6,1) infinite"></div>"#,
            height
        ));
    }

    format!(
        r##"<section style="padding:32px 0">
  {title_html}
  <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:16px">
    {blocks}
  </div>
</section>"##,
        title_html = title_html, blocks = blocks,
    )
}

// ══════════════════════════════════════════════════
// EMPTY STATE SECTION
// ══════════════════════════════════════════════════

pub(super) fn render_empty_section(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("Nothing here yet");
    let subtitle = section.subtitle.as_deref().unwrap_or("");
    let icon = section.config.get("icon").map(|s| s.as_str()).unwrap_or("inbox");
    let cta_text = section.config.get("cta_text")
        .or(section.config.get("cta"))
        .map(|s| s.as_str())
        .unwrap_or("");
    let cta_link = section.config.get("cta_link").map(|s| s.as_str()).unwrap_or("#");

    let subtitle_html = if subtitle.is_empty() {
        String::new()
    } else {
        format!(r#"<p style="font-size:14px;color:#a1a1aa;margin:8px 0 0;max-width:360px">{}</p>"#, subtitle)
    };

    let cta_html = if cta_text.is_empty() {
        String::new()
    } else {
        format!(
            r#"<a href="{link}" style="display:inline-block;margin-top:24px;padding:12px 28px;background:#000;color:#fff;border-radius:999px;font-size:14px;font-weight:700;text-decoration:none;font-family:Inter,sans-serif">{text}</a>"#,
            link = cta_link, text = cta_text,
        )
    };

    format!(
        r##"<section style="padding:80px 0;display:flex;flex-direction:column;align-items:center;justify-content:center;text-align:center" class="anim-fade">
  <span class="material-symbols-outlined" style="font-size:64px;color:#d4d4d8;margin-bottom:16px">{icon}</span>
  <h2 style="font-size:20px;font-weight:700;letter-spacing:-0.02em;margin:0;color:#52525b">{title}</h2>
  {subtitle_html}
  {cta_html}
</section>"##,
        icon = icon, title = title, subtitle_html = subtitle_html, cta_html = cta_html,
    )
}

// ══════════════════════════════════════════════════
// ERROR SECTION
// ══════════════════════════════════════════════════

pub(super) fn render_error_section(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("Something went wrong");
    let subtitle = section.subtitle.as_deref().unwrap_or("");
    let icon = section.config.get("icon").map(|s| s.as_str()).unwrap_or("error");
    let retry_text = section.config.get("retry_text").map(|s| s.as_str()).unwrap_or("Try again");
    let retry_link = section.config.get("retry_link").map(|s| s.as_str()).unwrap_or("");

    let subtitle_html = if subtitle.is_empty() {
        String::new()
    } else {
        format!(r#"<p style="font-size:14px;color:#a1a1aa;margin:12px 0 0;max-width:400px">{}</p>"#, subtitle)
    };

    let retry_onclick = if retry_link.is_empty() {
        r#"onclick="if(window.CRONUS&&window.CRONUS.reload)window.CRONUS.reload();else location.reload()""#.to_string()
    } else {
        format!(r#"onclick="location.href='{}'"#, retry_link)
    };

    format!(
        r##"<section style="padding:80px 0;display:flex;flex-direction:column;align-items:center;justify-content:center;text-align:center" class="anim-fade">
  <div style="width:80px;height:80px;border-radius:50%;background:#fef2f2;display:flex;align-items:center;justify-content:center;margin-bottom:20px">
    <span class="material-symbols-outlined" style="font-size:36px;color:#dc2626">{icon}</span>
  </div>
  <h2 style="font-size:20px;font-weight:700;letter-spacing:-0.02em;margin:0;color:#18181b">{title}</h2>
  {subtitle_html}
  <button {retry_onclick} style="margin-top:24px;padding:12px 28px;background:#000;color:#fff;border:none;border-radius:999px;font-size:14px;font-weight:700;cursor:pointer;font-family:Inter,sans-serif">{retry_text}</button>
</section>"##,
        icon = icon, title = title, subtitle_html = subtitle_html,
        retry_onclick = retry_onclick, retry_text = retry_text,
    )
}

// ══════════════════════════════════════════════════
// NOT FOUND / 404 SECTION
// ══════════════════════════════════════════════════

pub(super) fn render_not_found_section(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("Page not found");
    let subtitle = section.subtitle.as_deref().unwrap_or("The page you're looking for doesn't exist or has been moved.");
    let home_text = section.config.get("home_text").map(|s| s.as_str()).unwrap_or("Go home");
    let home_link = section.config.get("home_link").map(|s| s.as_str()).unwrap_or("/");

    format!(
        r##"<section style="padding:80px 0;display:flex;flex-direction:column;align-items:center;justify-content:center;text-align:center" class="anim-fade">
  <div style="font-size:120px;font-weight:900;letter-spacing:-0.05em;color:#e4e4e7;line-height:1;margin-bottom:16px">404</div>
  <h2 style="font-size:22px;font-weight:700;letter-spacing:-0.02em;margin:0;color:#18181b">{title}</h2>
  <p style="font-size:14px;color:#a1a1aa;margin:12px 0 0;max-width:400px">{subtitle}</p>
  <a href="{home_link}" style="display:inline-block;margin-top:24px;padding:12px 28px;background:#000;color:#fff;border-radius:999px;font-size:14px;font-weight:700;text-decoration:none;font-family:Inter,sans-serif">{home_text}</a>
</section>"##,
        title = title, subtitle = subtitle, home_link = home_link, home_text = home_text,
    )
}

// ══════════════════════════════════════════════════
// KPI SECTION
// ══════════════════════════════════════════════════
// SETTINGS DASHBOARD (dark Obsidian — full page renderer)
// ══════════════════════════════════════════════════

pub(super) fn render_timeline_section(section: &SectionNode, bound_data: &crate::binding::ResolvedData) -> String {
    let title = section.title.as_deref().unwrap_or("");
    let subtitle = section.subtitle.as_deref().unwrap_or("");

    let title_html = if title.is_empty() {
        String::new()
    } else {
        let sub = if subtitle.is_empty() {
            String::new()
        } else {
            format!(r#"<p style="font-size:14px;color:#71717a;margin:4px 0 0">{}</p>"#, subtitle)
        };
        format!(r#"<div style="margin-bottom:32px"><h2 style="font-size:20px;font-weight:700;letter-spacing:-0.02em;margin:0">{}</h2>{}</div>"#, title, sub)
    };

    // Build timeline items from bound data or static items
    struct TimelineItem {
        title: String,
        description: String,
        time: String,
        icon: String,
        status: String,
    }

    let timeline_items: Vec<TimelineItem> = if let crate::binding::ResolvedData::Rows(rows) = bound_data {
        if !rows.is_empty() {
            rows.iter().map(|row| {
                TimelineItem {
                    title: row.get("title").or_else(|| row.get("name"))
                        .and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    description: row.get("description").or_else(|| row.get("desc"))
                        .and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    time: row.get("time").or_else(|| row.get("date")).or_else(|| row.get("created_at"))
                        .and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    icon: row.get("icon")
                        .and_then(|v| v.as_str()).unwrap_or("circle").to_string(),
                    status: row.get("status")
                        .and_then(|v| v.as_str()).unwrap_or("info").to_string(),
                }
            }).collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Use bound items if available, otherwise fall back to static section items
    let use_bound = !timeline_items.is_empty();

    let mut items_html = String::new();

    if use_bound {
        let item_count = timeline_items.len();
        for (i, tl) in timeline_items.iter().enumerate() {
            let delay_class = format!("d{}", (i % 10) + 1);
            let dot_color = match tl.status.as_str() {
                "success" | "done" | "completed" => "#059669",
                "error" | "failed" | "danger" => "#dc2626",
                "warning" => "#d97706",
                _ => "#3b82f6",
            };
            let is_last = i == item_count - 1;
            let line_html = if is_last {
                String::new()
            } else {
                r#"<div style="position:absolute;left:17px;top:40px;bottom:-12px;width:2px;background:#e4e4e7"></div>"#.to_string()
            };
            let desc_html = if tl.description.is_empty() {
                String::new()
            } else {
                format!(r#"<p style="font-size:13px;color:#a1a1aa;margin:4px 0 0">{}</p>"#, tl.description)
            };
            let time_html = if tl.time.is_empty() {
                String::new()
            } else {
                format!(r#"<span style="font-size:12px;color:#a1a1aa;margin-left:auto;white-space:nowrap">{}</span>"#, tl.time)
            };

            items_html.push_str(&format!(
                r##"<div class="anim-slide-up {delay}" style="position:relative;padding-left:48px;padding-bottom:28px">
  {line}
  <div style="position:absolute;left:0;top:0;width:36px;height:36px;border-radius:50%;background:{dot_bg};display:flex;align-items:center;justify-content:center">
    <span class="material-symbols-outlined" style="font-size:18px;color:#fff">{icon}</span>
  </div>
  <div style="display:flex;align-items:baseline;gap:12px">
    <h4 style="font-size:14px;font-weight:600;margin:0;padding-top:7px">{title}</h4>
    {time_html}
  </div>
  {desc_html}
</div>"##,
                delay = delay_class, line = line_html, dot_bg = dot_color,
                icon = tl.icon, title = tl.title, time_html = time_html, desc_html = desc_html,
            ));
        }
    } else {
        // Fallback: static items from section
        let item_count = section.items.len();
        for (i, item) in section.items.iter().enumerate() {
            let item_title = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let desc = item.get("description").map(|s| s.as_str()).unwrap_or("");
            let time = item.get("time").map(|s| s.as_str()).unwrap_or("");
            let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("circle");
            let status = item.get("status").map(|s| s.as_str()).unwrap_or("info");
            let delay_class = format!("d{}", (i % 10) + 1);

            let dot_color = match status {
                "success" | "done" | "completed" => "#059669",
                "error" | "failed" | "danger" => "#dc2626",
                "warning" => "#d97706",
                _ => "#3b82f6",
            };

            let is_last = i == item_count - 1;
            let line_html = if is_last {
                String::new()
            } else {
                r#"<div style="position:absolute;left:17px;top:40px;bottom:-12px;width:2px;background:#e4e4e7"></div>"#.to_string()
            };

            let desc_html = if desc.is_empty() {
                String::new()
            } else {
                format!(r#"<p style="font-size:13px;color:#a1a1aa;margin:4px 0 0">{}</p>"#, desc)
            };

            let time_html = if time.is_empty() {
                String::new()
            } else {
                format!(r#"<span style="font-size:12px;color:#a1a1aa;margin-left:auto;white-space:nowrap">{}</span>"#, time)
            };

            items_html.push_str(&format!(
                r##"<div class="anim-slide-up {delay}" style="position:relative;padding-left:48px;padding-bottom:28px">
  {line}
  <div style="position:absolute;left:0;top:0;width:36px;height:36px;border-radius:50%;background:{dot_bg};display:flex;align-items:center;justify-content:center">
    <span class="material-symbols-outlined" style="font-size:18px;color:#fff">{icon}</span>
  </div>
  <div style="display:flex;align-items:baseline;gap:12px">
    <h4 style="font-size:14px;font-weight:600;margin:0;padding-top:7px">{title}</h4>
    {time_html}
  </div>
  {desc_html}
</div>"##,
                delay = delay_class, line = line_html, dot_bg = dot_color,
                icon = icon, title = item_title, time_html = time_html, desc_html = desc_html,
            ));
        }
    }

    format!(
        r##"<section style="padding:32px 0">
  {title_html}
  <div style="position:relative">
    {items}
  </div>
</section>"##,
        title_html = title_html, items = items_html,
    )
}

// ══════════════════════════════════════════════════
// PROGRESS / STEPS SECTION
// ══════════════════════════════════════════════════

pub(super) fn render_progress_section(section: &SectionNode, bound_data: &crate::binding::ResolvedData) -> String {
    let title = section.title.as_deref().unwrap_or("");
    let subtitle = section.subtitle.as_deref().unwrap_or("");

    let title_html = if title.is_empty() {
        String::new()
    } else {
        let sub = if subtitle.is_empty() {
            String::new()
        } else {
            format!(r#"<p style="font-size:14px;color:#71717a;margin:4px 0 0">{}</p>"#, subtitle)
        };
        format!(r#"<div style="margin-bottom:32px"><h2 style="font-size:20px;font-weight:700;letter-spacing:-0.02em;margin:0">{}</h2>{}</div>"#, title, sub)
    };

    // Convert bound data rows into items format
    let items_from_data: Vec<std::collections::HashMap<String, String>> = match bound_data {
        crate::binding::ResolvedData::Rows(rows) if !rows.is_empty() => {
            rows.iter().filter_map(|row| {
                let obj = row.as_object()?;
                let mut map = std::collections::HashMap::new();
                if let Some(t) = obj.get("title").or(obj.get("name")).and_then(|v| v.as_str()) {
                    map.insert("title".to_string(), t.to_string());
                }
                if let Some(s) = obj.get("status").and_then(|v| v.as_str()) {
                    map.insert("status".to_string(), s.to_string());
                }
                if let Some(d) = obj.get("description").and_then(|v| v.as_str()) {
                    map.insert("description".to_string(), d.to_string());
                }
                if let Some(v) = obj.get("value").and_then(|v| v.as_str()) {
                    map.insert("value".to_string(), v.to_string());
                }
                if let Some(ic) = obj.get("icon").and_then(|v| v.as_str()) {
                    map.insert("icon".to_string(), ic.to_string());
                }
                Some(map)
            }).collect()
        }
        _ => Vec::new(),
    };

    let items: &Vec<std::collections::HashMap<String, String>> = if items_from_data.is_empty() {
        &section.items
    } else {
        &items_from_data
    };

    let mut steps_html = String::new();
    for (i, item) in items.iter().enumerate() {
        let item_title = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let desc = item.get("description").map(|s| s.as_str()).unwrap_or("");
        let status = item.get("status").map(|s| s.as_str()).unwrap_or("pending");
        let value = item.get("value").map(|s| s.as_str()).unwrap_or("");
        let delay_class = format!("d{}", (i % 10) + 1);

        let (indicator, label_color) = match status {
            "done" | "completed" | "success" => (
                r#"<div style="width:28px;height:28px;border-radius:50%;background:#059669;display:flex;align-items:center;justify-content:center;flex-shrink:0"><span class="material-symbols-outlined" style="font-size:16px;color:#fff">check</span></div>"#.to_string(),
                "#18181b",
            ),
            "active" | "current" | "in-progress" => (
                r#"<div style="width:28px;height:28px;border-radius:50%;background:#3b82f6;display:flex;align-items:center;justify-content:center;flex-shrink:0"><div style="width:10px;height:10px;border-radius:50%;background:#fff"></div></div>"#.to_string(),
                "#18181b",
            ),
            _ => (
                r#"<div style="width:28px;height:28px;border-radius:50%;background:#e4e4e7;flex-shrink:0"></div>"#.to_string(),
                "#a1a1aa",
            ),
        };

        let desc_html = if desc.is_empty() {
            String::new()
        } else {
            format!(r#"<p style="font-size:13px;color:#a1a1aa;margin:4px 0 0">{}</p>"#, desc)
        };

        let progress_bar = if status == "active" || status == "current" || status == "in-progress" {
            let pct = if value.is_empty() { "50" } else { value.trim_end_matches('%') };
            format!(
                r#"<div style="margin-top:8px;height:4px;background:#e4e4e7;border-radius:999px;overflow:hidden"><div style="height:100%;background:#3b82f6;border-radius:999px;width:{}%;transition:width 0.6s cubic-bezier(0.16,1,0.3,1)"></div></div>"#,
                pct
            )
        } else {
            String::new()
        };

        steps_html.push_str(&format!(
            r##"<div class="anim-slide-up {delay}" style="display:flex;gap:16px;padding:16px 0">
  {indicator}
  <div style="flex:1;min-width:0">
    <h4 style="font-size:14px;font-weight:600;margin:0;color:{label_color}">{title}</h4>
    {desc_html}
    {progress_bar}
  </div>
</div>"##,
            delay = delay_class, indicator = indicator, label_color = label_color,
            title = item_title, desc_html = desc_html, progress_bar = progress_bar,
        ));
    }

    format!(
        r##"<section style="padding:32px 0">
  {title_html}
  <div style="display:flex;flex-direction:column">
    {steps}
  </div>
</section>"##,
        title_html = title_html, steps = steps_html,
    )
}
