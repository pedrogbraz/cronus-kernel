#![allow(dead_code, unused_imports, unused_variables)]
//! Dashboard page renderers — full-page HTML for various dashboard types
//! (billing, payouts, security, settings, checkout, order detail, etc.)

use crate::parser::{SectionNode, ComponentNode, ComponentItemNode};
use super::util::{item_by_kind, items_by_kind};
use super::CRONUS_ANIMATIONS_CSS;
use super::CRONUS_ANIMATIONS_JS;
use super::section_extra::render_sidebar;

pub fn render_dashboard_page(
    app_name: &str,
    sections: &[SectionNode],
    components: &[ComponentNode],
    theme: &str,
    current_route: &str,
) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    let _ = theme;

    // ── Extract component data ──────────────────────

    // Sidebar component (layout:sidebar)
    let sidebar_comp = components.iter().find(|c| c.layout.as_deref() == Some("sidebar"));
    let topbar_comp = components.iter().find(|c| {
        c.layout.as_deref() == Some("inline") && c.style.as_deref().map(|s| s.contains("topbar")).unwrap_or(false)
    });

    // ── Extract section data by type ────────────────

    let page_header = sections.iter().find(|s| s.section_type == "page-header");
    let live_keys = sections.iter().find(|s| s.section_type == "live-keys");
    let test_keys = sections.iter().find(|s| s.section_type == "test-keys");
    let webhooks = sections.iter().find(|s| s.section_type == "webhooks");
    let promo = sections.iter().find(|s| s.section_type == "promo");
    let quick_links = sections.iter().find(|s| s.section_type == "quick-links" || s.section_type == "links");
    let status_card = sections.iter().find(|s| s.section_type == "status-card");
    // Also check for a sidebar section if no component
    let sidebar_section = sections.iter().find(|s| s.section_type == "sidebar");

    // ── Build sidebar HTML ──────────────────────────

    let sidebar_html = build_dashboard_sidebar(sidebar_comp, sidebar_section, current_route);

    // ── Build topbar HTML ───────────────────────────

    let topbar_html = build_dashboard_topbar(topbar_comp);

    // ── Build page header ───────────────────────────

    let header_html = build_dashboard_page_header(page_header);

    // ── Build left column cards ─────────────────────

    let live_keys_html = build_api_keys_card(live_keys, "live");
    let test_keys_html = build_api_keys_card(test_keys, "test");
    let webhooks_html = build_webhooks_card(webhooks);

    // ── Build right column panels ───────────────────

    let promo_html = build_promo_panel(promo);
    let quick_links_html = build_quick_links_panel(quick_links);
    let status_html = build_status_panel(status_card);

    // ── Assemble complete page ──────────────────────

    format!(
        r##"<!DOCTYPE html>
<html class="light" lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta name="view-transition" content="same-origin">
  <title>{app_name}</title>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap" rel="stylesheet">
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet">
  <style>
    body {{ font-family:'Inter',sans-serif; background:#f9f9f9; color:#1a1c1c; margin:0; }}
    * {{ box-sizing:border-box; }}
    .material-symbols-outlined {{ font-variation-settings:'FILL' 0,'wght' 400,'GRAD' 0,'opsz' 24; font-size:20px; display:inline-block; line-height:1; vertical-align:middle; }}
    ::selection {{ background:rgba(0,111,240,0.15); }}
    ::-webkit-scrollbar {{ width:4px; }}
    ::-webkit-scrollbar-thumb {{ background:rgba(0,0,0,0.1); border-radius:2px; }}
  </style>
  <style>{anim_css}</style>
</head>
<body>

{sidebar}

{topbar}

<main style="margin-left:256px;min-height:100vh;background:radial-gradient(circle at top right,rgba(0,111,240,0.08),transparent 40%),radial-gradient(circle at bottom left,rgba(0,56,129,0.05),transparent 40%);background-image:linear-gradient(to right,rgba(198,198,198,0.1) 1px,transparent 1px),linear-gradient(to bottom,rgba(198,198,198,0.1) 1px,transparent 1px);background-size:40px 40px">
  <div style="max-width:1280px;margin:0 auto;padding:48px 24px">

    {header}

    <div style="display:grid;grid-template-columns:2fr 1fr;gap:32px">

      <!-- Left column: API cards -->
      <div style="display:flex;flex-direction:column;gap:32px">
        {live_keys}
        {test_keys}
        {webhooks}
      </div>

      <!-- Right column: panels -->
      <div style="display:flex;flex-direction:column;gap:24px">
        {promo}
        {quick_links}
        {status}
      </div>

    </div>
  </div>
</main>

<script{script_nonce}>{runtime}</script>
<script{script_nonce}>{hmr}</script>
{anim_js}
</body>
</html>"##,
        app_name = app_name,
        sidebar = sidebar_html,
        topbar = topbar_html,
        header = header_html,
        live_keys = live_keys_html,
        test_keys = test_keys_html,
        webhooks = webhooks_html,
        promo = promo_html,
        quick_links = quick_links_html,
        status = status_html,
        anim_css = CRONUS_ANIMATIONS_CSS,
        anim_js = crate::security::mark_kernel_scripts(CRONUS_ANIMATIONS_JS),
        runtime = crate::render::CRONUS_RUNTIME_JS,
        hmr = crate::hmr::HMR_CLIENT_JS,
    )
}

// ── Generic dashboard wrapper (FIX 3) ──────────
// For pages that have a sidebar component but no specific dashboard section types
// (e.g. Overview page with hero + stats). Renders body content inside dashboard layout.

pub fn render_generic_dashboard(
    app_name: &str,
    body: &str,
    components: &[ComponentNode],
    theme: &str,
    current_route: &str,
) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    let is_dark = theme == "dark" || theme == "obsidian";
    let (html_class, bg_color, text_color, selection_bg, scrollbar_color) = if is_dark {
        ("dark", "#131313", "#e2e2e2", "rgba(173,198,255,0.2)", "rgba(255,255,255,0.1)")
    } else {
        ("light", "#f9f9f9", "#1a1c1c", "rgba(0,111,240,0.15)", "rgba(0,0,0,0.1)")
    };

    let sidebar_comp = components.iter().find(|c| c.layout.as_deref() == Some("sidebar"));
    let topbar_comp = components.iter().find(|c| {
        c.layout.as_deref() == Some("inline") && c.style.as_deref().map(|s| s.contains("topbar")).unwrap_or(false)
    });

    let sidebar_html = build_dashboard_sidebar(sidebar_comp, None, current_route);
    let topbar_html = build_dashboard_topbar(topbar_comp);

    format!(
        r##"<!DOCTYPE html>
<html class="{html_class}" lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta name="view-transition" content="same-origin">
  <title>{app_name}</title>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800;900&family=Inter+Display:wght@400;500;600;700;800&display=swap" rel="stylesheet">
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet">
  <style>
    body {{ font-family:'Inter',sans-serif; background:{bg_color}; color:{text_color}; margin:0; -webkit-font-smoothing:antialiased; }}
    * {{ box-sizing:border-box; }}
    .material-symbols-outlined {{ font-variation-settings:'FILL' 0,'wght' 400,'GRAD' 0,'opsz' 24; font-size:20px; display:inline-block; line-height:1; vertical-align:middle; }}
    .technical-border {{ border:0.5px solid rgba(76,69,70,0.15); }}
    .liquid-glass {{ background:linear-gradient(135deg,rgba(173,198,255,0.05) 0%,rgba(194,193,255,0.05) 100%); backdrop-filter:blur(32px); }}
    ::selection {{ background:{selection_bg}; }}
    ::-webkit-scrollbar {{ width:4px; }}
    ::-webkit-scrollbar-thumb {{ background:{scrollbar_color}; border-radius:2px; }}
    @keyframes fadeIn {{ from {{ opacity:0;transform:translateY(4px) }} to {{ opacity:1;transform:translateY(0) }} }}
    .anim {{ animation:fadeIn 0.4s ease-out both; }}
  </style>
  <style>{anim_css}</style>
</head>
<body>

{sidebar}

{topbar}

<main style="margin-left:256px;min-height:100vh">
  <div style="max-width:1280px;margin:0 auto;padding:48px 24px">
    {body}
  </div>
</main>

<script{script_nonce}>{runtime}</script>
<script{script_nonce}>{hmr}</script>
{anim_js}
</body>
</html>"##,
        app_name = app_name,
        sidebar = sidebar_html,
        topbar = topbar_html,
        body = body,
        anim_css = CRONUS_ANIMATIONS_CSS,
        anim_js = crate::security::mark_kernel_scripts(CRONUS_ANIMATIONS_JS),
        runtime = crate::render::CRONUS_RUNTIME_JS,
        hmr = crate::hmr::HMR_CLIENT_JS,
    )
}

// ── Sidebar builder ─────────────────────────────

pub(super) fn build_dashboard_sidebar(comp: Option<&ComponentNode>, section: Option<&SectionNode>, current_route: &str) -> String {
    let mut brand = "";
    let mut subtitle = "";
    let mut nav_items_html = String::new();
    let mut bottom_items_html = String::new();

    if let Some(c) = comp {
        // Extract brand from props or items
        if let Some(b) = c.props.get("brand") {
            brand = b.as_str();
        } else if let Some(b) = item_by_kind(&c.items, "brand") {
            brand = b;
        }
        if let Some(s) = c.props.get("subtitle") {
            subtitle = s.as_str();
        } else if let Some(s) = item_by_kind(&c.items, "subtitle") {
            subtitle = s;
        }
        // Nav items
        let items = items_by_kind(&c.items, "item");
        let bottom_types = ["contact_support", "menu_book", "support", "docs"];
        for item in &items {
            let title = item.text.as_str();
            let icon = item.config.get("icon").map(|s| s.as_str()).unwrap_or("");
            let href = item.link.as_deref().unwrap_or("");
            // FIX 2: Active state based on current route matching the item's link
            let is_active = (!href.is_empty() && href == current_route) ||
                item.config.get("active").map(|s| s == "true").unwrap_or(false);
            let is_bottom = bottom_types.contains(&icon) || title.eq_ignore_ascii_case("support") || title.eq_ignore_ascii_case("docs");

            let nav_idx = if is_bottom { bottom_items_html.matches("<a ").count() } else { nav_items_html.matches("<a ").count() };
            let delay_cls = format!("d{}", (nav_idx % 10) + 1);
            let link_html = if is_active {
                format!(
                    r#"<a href="{href}" class="nav-hover anim-slide-right {delay}" style="display:flex;align-items:center;gap:12px;padding:8px 12px;background:#f4f4f5;color:#000;border-radius:6px;font-size:14px;font-weight:500;text-decoration:none;transform:scale(0.97)">
  <span class="material-symbols-outlined" style="font-size:20px">{icon}</span>
  <span>{title}</span>
</a>"#,
                    href = href, icon = icon, title = title, delay = delay_cls
                )
            } else {
                format!(
                    r#"<a href="{href}" class="nav-hover anim-slide-right {delay}" style="display:flex;align-items:center;gap:12px;padding:8px 12px;color:#71717a;border-radius:6px;font-size:14px;font-weight:500;text-decoration:none;transition:all 0.15s" onmouseover="this.style.color='#000';this.style.background='#f4f4f5'" onmouseout="this.style.color='#71717a';this.style.background='transparent'">
  <span class="material-symbols-outlined" style="font-size:20px">{icon}</span>
  <span>{title}</span>
</a>"#,
                    href = href, icon = icon, title = title, delay = delay_cls
                )
            };

            if is_bottom {
                bottom_items_html.push_str(&link_html);
                bottom_items_html.push('\n');
            } else {
                nav_items_html.push_str(&link_html);
                nav_items_html.push('\n');
            }
        }
    } else if let Some(sec) = section {
        brand = sec.config.get("brand").map(|s| s.as_str())
            .or(sec.title.as_deref())
            .unwrap_or("");
        subtitle = sec.config.get("subtitle").map(|s| s.as_str())
            .or(sec.subtitle.as_deref())
            .unwrap_or("");
        let active = sec.config.get("active").map(|s| s.as_str()).unwrap_or("");
        for item in &sec.items {
            let title = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("");
            let href = item.get("href").map(|s| s.as_str()).unwrap_or("");
            let position = item.get("position").map(|s| s.as_str()).unwrap_or("top");
            // FIX 2: Active state based on current route matching the item's href
            let is_active = (!href.is_empty() && href == current_route)
                || (!active.is_empty() && title.eq_ignore_ascii_case(active))
                || item.get("active").map(|s| s == "true").unwrap_or(false);

            let nav_idx2 = if position == "bottom" { bottom_items_html.matches("<a ").count() } else { nav_items_html.matches("<a ").count() };
            let delay_cls2 = format!("d{}", (nav_idx2 % 10) + 1);
            let link_html = if is_active {
                format!(
                    r#"<a href="{href}" class="nav-hover anim-slide-right {delay}" style="display:flex;align-items:center;gap:12px;padding:8px 12px;background:#f4f4f5;color:#000;border-radius:6px;font-size:14px;font-weight:500;text-decoration:none;transform:scale(0.97)">
  <span class="material-symbols-outlined" style="font-size:20px">{icon}</span>
  <span>{title}</span>
</a>"#,
                    href = href, icon = icon, title = title, delay = delay_cls2
                )
            } else {
                format!(
                    r#"<a href="{href}" class="nav-hover anim-slide-right {delay}" style="display:flex;align-items:center;gap:12px;padding:8px 12px;color:#71717a;border-radius:6px;font-size:14px;font-weight:500;text-decoration:none;transition:all 0.15s" onmouseover="this.style.color='#000';this.style.background='#f4f4f5'" onmouseout="this.style.color='#71717a';this.style.background='transparent'">
  <span class="material-symbols-outlined" style="font-size:20px">{icon}</span>
  <span>{title}</span>
</a>"#,
                    href = href, icon = icon, title = title, delay = delay_cls2
                )
            };

            if position == "bottom" {
                bottom_items_html.push_str(&link_html);
                bottom_items_html.push('\n');
            } else {
                nav_items_html.push_str(&link_html);
                nav_items_html.push('\n');
            }
        }
    }

    format!(
        r##"<aside class="anim-slide-right" style="position:fixed;left:0;top:0;height:100%;width:256px;background:rgba(250,250,250,0.5);border-right:1px solid #e5e7eb;display:flex;flex-direction:column;z-index:50;padding:16px;gap:4px;font-family:'Inter',sans-serif;-webkit-font-smoothing:antialiased">
  <div style="display:flex;align-items:center;gap:12px;margin-bottom:32px;padding:0 8px">
    <div style="width:32px;height:32px;background:#000;border-radius:6px;display:flex;align-items:center;justify-content:center">
      <span style="color:#fff;font-weight:700;letter-spacing:-0.04em;font-size:14px">{brand_letter}</span>
    </div>
    <div>
      <h1 style="font-weight:700;letter-spacing:-0.04em;color:#000;font-size:16px;margin:0">{brand}</h1>
      <p style="font-size:10px;color:#71717a;font-weight:600;letter-spacing:0.15em;text-transform:uppercase;margin:0">{subtitle}</p>
    </div>
  </div>
  <nav style="flex:1;display:flex;flex-direction:column;gap:4px">
    {nav_items}
  </nav>
  <div style="margin-top:auto;border-top:1px solid #e5e7eb;padding-top:16px;display:flex;flex-direction:column;gap:4px">
    {bottom_items}
  </div>
</aside>"##,
        brand_letter = brand.chars().next().unwrap_or('D'),
        brand = brand,
        subtitle = subtitle,
        nav_items = nav_items_html,
        bottom_items = bottom_items_html,
    )
}

// ── Topbar builder ──────────────────────────────

pub(super) fn build_dashboard_topbar(comp: Option<&ComponentNode>) -> String {
    let search_placeholder = comp.and_then(|c| {
        c.items.iter().find(|i| {
            i.config.get("icon").map(|s| s == "search").unwrap_or(false)
        }).map(|i| i.text.as_str())
    }).unwrap_or("Search documentation...");

    let avatar_url = comp.and_then(|c| c.props.get("avatar").map(|s| s.as_str())).unwrap_or("");

    format!(
        r##"<header class="anim-slide-down" style="position:sticky;top:0;z-index:40;background:rgba(255,255,255,0.8);backdrop-filter:blur(20px);-webkit-backdrop-filter:blur(20px);border-bottom:1px solid #e5e7eb;margin-left:256px">
  <div style="display:flex;justify-content:space-between;align-items:center;height:64px;padding:0 24px;max-width:1280px;margin:0 auto">
    <div style="position:relative;width:100%;max-width:448px">
      <span class="material-symbols-outlined" style="position:absolute;left:12px;top:50%;transform:translateY(-50%);color:#a1a1aa;font-size:16px">search</span>
      <input type="text" placeholder="{placeholder}" style="width:100%;padding:6px 16px 6px 40px;border:none;background:#f3f3f3;border-radius:999px;font-size:14px;outline:none;font-family:'Inter',sans-serif">
    </div>
    <div style="display:flex;align-items:center;gap:16px">
      <span class="material-symbols-outlined" style="color:#71717a;cursor:pointer">notifications</span>
      <span class="material-symbols-outlined" style="color:#71717a;cursor:pointer">help</span>
      <div style="width:32px;height:32px;border-radius:50%;background:#e5e7eb;overflow:hidden;border:1px solid #d4d4d8">
        <img src="{avatar}" style="width:100%;height:100%;object-fit:cover" alt="User">
      </div>
    </div>
  </div>
</header>"##,
        placeholder = search_placeholder,
        avatar = avatar_url,
    )
}

// ── Page header builder ─────────────────────────

pub(super) fn build_dashboard_page_header(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("Page Title");
    let subtitle = sec.subtitle.as_deref().unwrap_or("");
    let badge = sec.config.get("badge").map(|s| s.as_str()).unwrap_or("");

    let badge_html = if !badge.is_empty() {
        format!(
            r#"<div style="display:flex;align-items:center;gap:8px;margin-bottom:8px">
        <span style="display:inline-flex;align-items:center;padding:2px 8px;border-radius:999px;font-size:10px;font-weight:700;letter-spacing:0.1em;text-transform:uppercase;background:#e2e2e2;color:#5e5e5e">{badge}</span>
        <span style="width:6px;height:6px;border-radius:50%;background:#006ff0"></span>
      </div>"#,
            badge = badge
        )
    } else {
        String::new()
    };

    let subtitle_html = if !subtitle.is_empty() {
        format!(
            r#"<p style="color:#5e5e5e;max-width:640px;line-height:1.6;font-size:15px;margin:0">{}</p>"#,
            subtitle
        )
    } else {
        String::new()
    };

    format!(
        r#"<div style="margin-bottom:48px">
      <div class="anim-fade d1">{badge}</div>
      <h2 class="anim-slide-up d1" style="font-size:36px;font-weight:800;letter-spacing:-0.04em;color:#1a1c1c;margin:0 0 8px">{title}</h2>
      <div class="anim-slide-up d2">{subtitle}</div>
    </div>"#,
        badge = badge_html,
        title = title,
        subtitle = subtitle_html,
    )
}

// ── API Keys card builder ───────────────────────

pub(super) fn build_api_keys_card(section: Option<&SectionNode>, mode: &str) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("");
    let subtitle = sec.subtitle.as_deref().unwrap_or("");
    let icon = sec.config.get("icon").map(|s| s.as_str()).unwrap_or("");

    // Build key fields from items
    let mut fields_html = String::new();
    for item in &sec.items {
        let item_title = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let description = item.get("description").map(|s| s.as_str()).unwrap_or("");
        let style_val = item.get("style").map(|s| s.as_str()).unwrap_or("");

        // Check if this has a "Reveal" or "Hide" action → masked secret key
        let has_reveal = sec.items.iter().any(|i| {
            i.get("title").map(|t| t == item_title).unwrap_or(false) &&
            description.contains('\u{2022}') // bullet dots = masked
        });
        let is_masked = description.contains('\u{2022}');

        // Find actions for this item
        // In the .cronus, actions are separate items with action text
        // But in our parsed structure, each item is a HashMap
        // The items have "action" keys for inline actions
        let action_text = item.get("action").map(|s| s.as_str()).unwrap_or("");
        let action_icon = item.get("action_icon").map(|s| s.as_str()).unwrap_or("");

        // Label
        fields_html.push_str(&format!(
            r#"<div>
              <label style="display:block;font-size:11px;font-weight:700;letter-spacing:0.15em;text-transform:uppercase;color:#a1a1aa;margin-bottom:8px">{label}</label>
              <div style="display:flex;align-items:center;gap:8px">"#,
            label = item_title,
        ));

        if is_masked {
            // Secret key with masked value + Reveal/Hide button
            let reveal_text = if description.starts_with("sk_test") { "Hide" } else { "Reveal" };
            fields_html.push_str(&format!(
                r#"<div style="flex:1;background:#f3f3f3;border:1px solid rgba(198,198,198,0.1);border-radius:8px;padding:12px 16px;font-family:'JetBrains Mono',monospace;font-size:14px;display:flex;justify-content:space-between;align-items:center">
                  <span style="letter-spacing:0.3em">{value}</span>
                  <button style="color:#0059c5;font-size:12px;font-weight:600;text-transform:uppercase;background:none;border:none;cursor:pointer;letter-spacing:-0.02em">{reveal}</button>
                </div>"#,
                value = description,
                reveal = reveal_text,
            ));
        } else {
            // Public key — plain display
            fields_html.push_str(&format!(
                r#"<div style="flex:1;background:#f3f3f3;border:1px solid rgba(198,198,198,0.1);border-radius:8px;padding:12px 16px;font-family:'JetBrains Mono',monospace;font-size:14px;overflow:hidden;white-space:nowrap">{value}</div>"#,
                value = description,
            ));
        }

        // Copy button
        fields_html.push_str(
            r#"<button style="padding:12px;color:#5e5e5e;background:none;border:none;border-radius:8px;cursor:pointer"><span class="material-symbols-outlined">content_copy</span></button>"#
        );

        fields_html.push_str("</div></div>\n");
    }

    format!(
        r##"<section class="anim-slide-up d2 card-hover" style="background:#fff;border:1px solid rgba(198,198,198,0.2);border-radius:12px;padding:32px;box-shadow:0 1px 3px rgba(0,0,0,0.04)">
          <div style="display:flex;justify-content:space-between;align-items:start;margin-bottom:24px">
            <div>
              <h3 style="font-size:20px;font-weight:700;letter-spacing:-0.02em;margin:0 0 4px">{title}</h3>
              <p style="font-size:14px;color:#5e5e5e;margin:0">{subtitle}</p>
            </div>
            <span class="material-symbols-outlined" style="color:#d4d4d8">{icon}</span>
          </div>
          <div style="display:flex;flex-direction:column;gap:24px">
            {fields}
          </div>
        </section>"##,
        title = title,
        subtitle = subtitle,
        icon = icon,
        fields = fields_html,
    )
}

// ── Webhooks card builder ───────────────────────

pub(super) fn build_webhooks_card(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("");
    let subtitle = sec.subtitle.as_deref().unwrap_or("");

    // Action button (e.g. "Add Endpoint")
    let action_text = sec.config.get("action_text")
        .or_else(|| sec.config.get("action"))
        .map(|s| s.as_str())
        .unwrap_or("");
    let action_btn = if !action_text.is_empty() {
        format!(
            r#"<button style="background:#000;color:#fff;padding:10px 24px;border-radius:999px;font-size:14px;font-weight:700;border:none;cursor:pointer;white-space:nowrap">{}</button>"#,
            action_text
        )
    } else {
        String::new()
    };

    // Webhook entries from items
    let mut entries_html = String::new();
    for item in &sec.items {
        let url = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let status = item.get("status").map(|s| s.as_str()).unwrap_or("");
        let events = item.get("description").map(|s| s.as_str()).unwrap_or("");

        let (badge_bg, badge_color, badge_text) = match status.to_lowercase().as_str() {
            "active" => ("#dcfce7", "#15803d", "Active"),
            "testing" => ("#f4f4f5", "#71717a", "Testing"),
            "error" | "failed" => ("#fef2f2", "#dc2626", "Error"),
            _ => ("#f4f4f5", "#71717a", status),
        };
        let badge_text_display = if status.is_empty() { "" } else { badge_text };

        entries_html.push_str(&format!(
            r##"<div style="padding:32px;border-bottom:1px solid #f3f3f3;transition:background 0.15s" onmouseover="this.style.background='rgba(243,243,243,0.5)'" onmouseout="this.style.background='transparent'">
              <div style="display:flex;align-items:center;gap:16px">
                <div style="flex:1">
                  <div style="display:flex;align-items:center;gap:12px;margin-bottom:4px">
                    <span style="font-size:14px;font-family:'JetBrains Mono',monospace;font-weight:700">{url}</span>
                    <span style="padding:2px 8px;background:{badge_bg};color:{badge_color};font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:0.05em;border-radius:4px">{badge_text}</span>
                  </div>
                  <p style="font-size:12px;color:#5e5e5e;margin:0">{events}</p>
                </div>
                <div style="display:flex;align-items:center;gap:4px;opacity:0;transition:opacity 0.15s" class="webhook-actions">
                  <button style="padding:8px;color:#a1a1aa;background:none;border:none;cursor:pointer;border-radius:4px" onmouseover="this.style.color='#000'" onmouseout="this.style.color='#a1a1aa'"><span class="material-symbols-outlined">edit</span></button>
                  <button style="padding:8px;color:#a1a1aa;background:none;border:none;cursor:pointer;border-radius:4px" onmouseover="this.style.color='#dc2626'" onmouseout="this.style.color='#a1a1aa'"><span class="material-symbols-outlined">delete</span></button>
                </div>
              </div>
            </div>"##,
            url = url,
            badge_bg = badge_bg,
            badge_color = badge_color,
            badge_text = badge_text_display,
            events = events,
        ));
    }

    format!(
        r##"<section class="anim-slide-up d3 card-hover" style="background:#fff;border:1px solid rgba(198,198,198,0.2);border-radius:12px;overflow:hidden;box-shadow:0 1px 3px rgba(0,0,0,0.04)">
          <div style="padding:32px;border-bottom:1px solid #f3f3f3;display:flex;justify-content:space-between;align-items:center">
            <div>
              <h3 style="font-size:20px;font-weight:700;letter-spacing:-0.02em;margin:0 0 4px">{title}</h3>
              <p style="font-size:14px;color:#5e5e5e;margin:0">{subtitle}</p>
            </div>
            <div class="btn-hover">{action_btn}</div>
          </div>
          <div>
            {entries}
          </div>
        </section>
        <style>
          section:hover .webhook-actions {{ opacity:1 !important; }}
          div:hover > div > .webhook-actions {{ opacity:1 !important; }}
        </style>"##,
        title = title,
        subtitle = subtitle,
        action_btn = action_btn,
        entries = entries_html,
    )
}

// ── Promo panel builder ─────────────────────────

pub(super) fn build_promo_panel(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("");
    let subtitle = sec.subtitle.as_deref().unwrap_or("");
    let cta_text = sec.config.get("cta_text")
        .or_else(|| sec.config.get("cta"))
        .map(|s| s.as_str())
        .unwrap_or("");

    let cta_html = if !cta_text.is_empty() {
        format!(
            r##"<a href="#" style="color:#fff;font-size:14px;font-weight:700;text-decoration:none;display:inline-flex;align-items:center;gap:8px">{cta} <span class="material-symbols-outlined" style="font-size:16px">arrow_forward</span></a>"##,
            cta = cta_text
        )
    } else {
        String::new()
    };

    format!(
        r##"<section class="anim-scale d3" style="background:#000;color:#fff;border-radius:12px;padding:32px;position:relative;overflow:hidden;box-shadow:0 20px 40px rgba(0,0,0,0.15)">
          <div style="position:relative;z-index:1">
            <h4 style="font-size:24px;font-weight:800;letter-spacing:-0.04em;margin:0 0 16px">{title}</h4>
            <p style="color:#a1a1aa;font-size:14px;line-height:1.6;margin:0 0 24px">{subtitle}</p>
            {cta}
          </div>
          <div style="position:absolute;right:-48px;bottom:-48px;width:192px;height:192px;background:rgba(0,111,240,0.2);border-radius:50%;filter:blur(48px)"></div>
        </section>"##,
        title = title,
        subtitle = subtitle,
        cta = cta_html,
    )
}

// ── Quick Links panel builder ───────────────────

pub(super) fn build_quick_links_panel(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("");

    let mut links_html = String::new();
    for item in &sec.items {
        let link_title = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let href = item.get("href").map(|s| s.as_str()).unwrap_or("");
        links_html.push_str(&format!(
            r#"<li><a href="{href}" class="link-hover" style="display:flex;justify-content:space-between;align-items:center;text-decoration:none;transition:opacity 0.15s" onmouseover="this.style.opacity='0.7'" onmouseout="this.style.opacity='1'">
              <span style="font-size:14px;font-weight:500;color:#1a1c1c">{title}</span>
              <span class="material-symbols-outlined" style="font-size:16px;color:#d4d4d8">open_in_new</span>
            </a></li>"#,
            href = href,
            title = link_title,
        ));
    }

    format!(
        r##"<section class="anim-slide-up d4 card-hover" style="background:#fff;border:1px solid rgba(198,198,198,0.2);border-radius:12px;padding:32px;box-shadow:0 1px 3px rgba(0,0,0,0.04)">
          <h4 style="font-size:12px;font-weight:700;letter-spacing:0.15em;text-transform:uppercase;color:#a1a1aa;margin:0 0 24px">{title}</h4>
          <ul style="list-style:none;padding:0;margin:0;display:flex;flex-direction:column;gap:16px">
            {links}
          </ul>
        </section>"##,
        title = title.to_uppercase(),
        links = links_html,
    )
}

// ── Status panel builder ────────────────────────

pub(super) fn build_status_panel(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("");

    let mut status_html = String::new();
    for item in &sec.items {
        let item_title = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let status = item.get("status").map(|s| s.as_str()).unwrap_or("default");
        let description = item.get("description").map(|s| s.as_str()).unwrap_or("");

        let dot_color = match status.to_lowercase().as_str() {
            "success" | "ok" | "operational" => "#22c55e",
            "warning" => "#eab308",
            "error" | "danger" => "#ef4444",
            _ => "#22c55e",
        };

        status_html.push_str(&format!(
            r#"<div style="display:flex;align-items:center;gap:12px;margin-bottom:16px">
              <span style="width:8px;height:8px;border-radius:50%;background:{dot_color}"></span>
              <span style="font-size:14px;font-weight:500">{title}</span>
            </div>"#,
            dot_color = dot_color,
            title = item_title,
        ));

        if !description.is_empty() {
            status_html.push_str(&format!(
                r#"<p style="font-size:12px;color:#71717a;line-height:1.4;margin:0">{}</p>"#,
                description
            ));
        }
    }

    format!(
        r##"<section style="background:rgba(232,232,232,0.5);border:1px solid rgba(198,198,198,0.1);border-radius:12px;padding:32px">
          <h4 style="font-size:12px;font-weight:700;letter-spacing:0.15em;text-transform:uppercase;color:#71717a;margin:0 0 16px">{title}</h4>
          {status}
        </section>"##,
        title = title.to_uppercase(),
        status = status_html,
    )
}

// ══════════════════════════════════════════════════
// DEDICATED DASHBOARD PAGE RENDERER — Billing
// Produces a complete HTML page for the Billing dashboard.
// ══════════════════════════════════════════════════

pub fn render_billing_dashboard(
    app_name: &str,
    sections: &[SectionNode],
    components: &[crate::parser::ComponentNode],
    theme: &str,
    current_route: &str,
) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    let is_dark = theme == "dark";

    // ── Extract component data ──────────────────────

    let sidebar_comp = components.iter().find(|c| c.layout.as_deref() == Some("sidebar"));
    let topbar_comp = components.iter().find(|c| {
        c.layout.as_deref() == Some("inline") && c.style.as_deref().map(|s| s.contains("topbar")).unwrap_or(false)
    });
    let sidebar_section = sections.iter().find(|s| s.section_type == "sidebar");

    // ── Extract section data by type ────────────────

    let page_header = sections.iter().find(|s| s.section_type == "page-header");
    let current_plan = sections.iter().find(|s| s.section_type == "current-plan");
    let usage_status = sections.iter().find(|s| s.section_type == "usage-status");
    let billing_stats = sections.iter().find(|s| s.section_type == "billing-stats");
    let payment_methods = sections.iter().find(|s| s.section_type == "payment-methods");
    let recent_invoices = sections.iter().find(|s| s.section_type == "recent-invoices");

    // ── Build sidebar + topbar (reuse) ─────────────

    let sidebar_html = build_dashboard_sidebar(sidebar_comp, sidebar_section, current_route);
    let topbar_html = build_dashboard_topbar(topbar_comp);

    // ── Build page header ───────────────────────────

    let header_html = build_billing_page_header(page_header);

    // ── Build bento grid cards ──────────────────────

    let current_plan_html = build_billing_current_plan(current_plan);
    let usage_status_html = build_billing_usage_status(usage_status);
    let billing_stats_html = build_billing_stats(billing_stats);
    let payment_methods_html = build_billing_payment_methods(payment_methods);
    let recent_invoices_html = build_billing_recent_invoices(recent_invoices);

    // ── Assemble complete page ──────────────────────

    let (html_class, bg, fg, card_bg, card_border, muted, badge_bg, badge_fg, btn_bg, btn_fg, row_border, main_gradient) = if is_dark {
        ("dark", "#0a0a0a", "#fafafa", "rgba(255,255,255,0.04)", "rgba(255,255,255,0.08)", "#a3a3a3",
         "rgba(255,255,255,0.1)", "#fafafa", "#fafafa", "#0a0a0a", "rgba(255,255,255,0.06)",
         "radial-gradient(circle at top right,rgba(255,255,255,0.03),transparent 40%),radial-gradient(circle at bottom left,rgba(255,255,255,0.02),transparent 40%)")
    } else {
        ("light", "#f9f9f9", "#1a1c1c", "#fff", "rgba(198,198,198,0.2)", "#5e5e5e",
         "#000", "#fff", "#000", "#fff", "#f3f3f3",
         "radial-gradient(circle at top right,rgba(0,111,240,0.08),transparent 40%),radial-gradient(circle at bottom left,rgba(0,56,129,0.05),transparent 40%)")
    };

    format!(
        r##"<!DOCTYPE html>
<html class="{html_class}" lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta name="view-transition" content="same-origin">
  <title>{app_name}</title>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap" rel="stylesheet">
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet">
  <style>
    :root {{ --bg:{bg}; --fg:{fg}; --card-bg:{card_bg}; --card-border:{card_border}; --muted:{muted}; --badge-bg:{badge_bg}; --badge-fg:{badge_fg}; --btn-bg:{btn_bg}; --btn-fg:{btn_fg}; --row-border:{row_border}; }}
    body {{ font-family:'Inter',sans-serif; background:var(--bg); color:var(--fg); margin:0; }}
    * {{ box-sizing:border-box; }}
    .material-symbols-outlined {{ font-variation-settings:'FILL' 0,'wght' 400,'GRAD' 0,'opsz' 24; font-size:20px; display:inline-block; line-height:1; vertical-align:middle; }}
    ::selection {{ background:rgba(255,255,255,0.15); }}
    ::-webkit-scrollbar {{ width:4px; }}
    ::-webkit-scrollbar-thumb {{ background:rgba(128,128,128,0.3); border-radius:2px; }}
    .ghost-border {{ border:1px solid var(--card-border); background:var(--card-bg); }}
    .payment-row:hover .payment-hover-actions {{ opacity:1 !important; }}
    html.dark section {{ background:var(--card-bg) !important; border-color:var(--card-border) !important; color:var(--fg); }}
    html.dark h2, html.dark h3, html.dark h4, html.dark span {{ color:inherit; }}
    html.dark section .anim-fade {{ background:var(--badge-bg) !important; color:var(--badge-fg) !important; }}
    html.dark button.btn-hover {{ background:var(--btn-bg) !important; color:var(--btn-fg) !important; }}
    html.dark [style*="color:#5e5e5e"], html.dark [style*="color:#1a1c1c"] {{ color:var(--muted) !important; }}
    html.dark [style*="background:#fff"], html.dark [style*="background:#f3f3f3"], html.dark [style*="background:#f4f4f5"] {{ background:var(--card-bg) !important; }}
    html.dark [style*="background:rgba(250"] {{ background:rgba(20,20,20,0.9) !important; }}
    html.dark aside {{ background:rgba(20,20,20,0.95) !important; border-color:rgba(255,255,255,0.06) !important; }}
    html.dark aside a {{ color:var(--muted) !important; }}
    html.dark aside a[style*="background:#fff"], html.dark aside a[style*="background:rgb(255"] {{ background:rgba(255,255,255,0.1) !important; color:var(--fg) !important; }}
    html.dark nav[style*="background:#fff"], html.dark nav[style*="background:rgba(255,255,255,0.95"] {{ background:rgba(20,20,20,0.95) !important; border-color:rgba(255,255,255,0.06) !important; }}
    html.dark header[style*="background:rgba(255,255,255"] {{ background:rgba(10,10,10,0.85) !important; border-color:rgba(255,255,255,0.06) !important; backdrop-filter:blur(20px); }}
    html.dark input {{ background:rgba(255,255,255,0.06) !important; border-color:rgba(255,255,255,0.1) !important; color:var(--fg) !important; }}
    html.dark [style*="border-bottom:1px solid #f3f3f3"], html.dark [style*="border-bottom: 1px solid #f3f3f3"] {{ border-color:var(--row-border) !important; }}
    html.dark [style*="border:1px solid rgba(198"] {{ border-color:var(--card-border) !important; }}
    html.dark [style*="border-right:1px solid #e5e7eb"] {{ border-color:rgba(255,255,255,0.06) !important; }}
  </style>
  <style>{anim_css}</style>
</head>
<body>

{sidebar}

{topbar}

<main style="margin-left:256px;min-height:100vh;background:{main_gradient}">
  <div style="max-width:1152px;margin:0 auto;padding:32px">

    {header}

    <div style="display:grid;grid-template-columns:repeat(12,1fr);gap:24px">

      <!-- Current Plan (col-span 7) -->
      <div style="grid-column:span 7">
        {current_plan}
      </div>

      <!-- Usage Status (col-span 5) -->
      <div style="grid-column:span 5">
        {usage_status}
      </div>

      <!-- Stats (col-span 12) -->
      <div style="grid-column:span 12">
        {billing_stats}
      </div>

      <!-- Payment Methods (col-span 8) -->
      <div style="grid-column:span 8">
        {payment_methods}
      </div>

      <!-- Recent Invoices (col-span 4) -->
      <div style="grid-column:span 4">
        {recent_invoices}
      </div>

    </div>
    <div style="height:96px"></div>
  </div>
</main>

<script{script_nonce}>{runtime}</script>
<script{script_nonce}>{hmr}</script>
{anim_js}
</body>
</html>"##,
        html_class = html_class,
        bg = bg, fg = fg, card_bg = card_bg, card_border = card_border,
        muted = muted, badge_bg = badge_bg, badge_fg = badge_fg,
        btn_bg = btn_bg, btn_fg = btn_fg, row_border = row_border,
        main_gradient = main_gradient,
        app_name = app_name,
        sidebar = sidebar_html,
        topbar = topbar_html,
        header = header_html,
        current_plan = current_plan_html,
        usage_status = usage_status_html,
        billing_stats = billing_stats_html,
        payment_methods = payment_methods_html,
        recent_invoices = recent_invoices_html,
        anim_css = CRONUS_ANIMATIONS_CSS,
        anim_js = crate::security::mark_kernel_scripts(CRONUS_ANIMATIONS_JS),
        runtime = crate::render::CRONUS_RUNTIME_JS,
        hmr = crate::hmr::HMR_CLIENT_JS,
    )
}

// ── Billing page header ────────────────────────

pub(super) fn build_billing_page_header(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("");
    let subtitle = sec.subtitle.as_deref().unwrap_or("");

    let subtitle_html = if !subtitle.is_empty() {
        format!(
            r#"<p style="color:var(--muted);font-size:18px;margin:8px 0 0;line-height:1.5">{}</p>"#,
            subtitle
        )
    } else {
        String::new()
    };

    format!(
        r#"<div style="margin-bottom:48px">
      <h2 class="anim-slide-up d1" style="font-size:56px;font-weight:800;letter-spacing:-0.05em;line-height:1.25;color:var(--fg);margin:0 0 8px">{title}</h2>
      <div class="anim-slide-up d2">{subtitle}</div>
    </div>"#,
        title = title,
        subtitle = subtitle_html,
    )
}

// ── Current Plan card ──────────────────────────

pub(super) fn build_billing_current_plan(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let badge_text = sec.config.get("badge").map(|s| s.as_str()).unwrap_or("");
    let plan_name = sec.title.as_deref().unwrap_or("");
    // Extract action text from items with _type=action, or fallback to config
    let action_text = sec.items.iter()
        .find(|i| i.get("_type").map(|s| s.as_str()) == Some("action"))
        .and_then(|i| i.get("title"))
        .map(|s| s.as_str())
        .or_else(|| sec.config.get("action_text").map(|s| s.as_str()))
        .or_else(|| sec.config.get("action").map(|s| s.as_str()))
        .unwrap_or("");

    // Build detail rows from items — skip action items
    let mut rows_html = String::new();
    for item in &sec.items {
        // Skip action items — they are rendered as the button above
        if item.get("_type").map(|s| s.as_str()) == Some("action") {
            continue;
        }
        let label = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let value = item.get("value").map(|s| s.as_str()).unwrap_or("");
        let suffix = item.get("suffix").map(|s| s.as_str()).unwrap_or("");

        let value_html = if !suffix.is_empty() {
            format!(
                r#"<span style="font-weight:600">{value}</span> <span style="font-size:14px;color:var(--muted)">{suffix}</span>"#,
                value = value, suffix = suffix
            )
        } else {
            format!(r#"<span style="font-weight:600">{value}</span>"#, value = value)
        };

        rows_html.push_str(&format!(
            r#"<div style="display:flex;justify-content:space-between;align-items:flex-end;border-bottom:1px solid var(--row-border);padding-bottom:16px;margin-bottom:24px">
              <span style="color:var(--muted)">{label}</span>
              <span>{value}</span>
            </div>"#,
            label = label,
            value = value_html,
        ));
    }

    format!(
        r##"<section class="ghost-border anim-slide-up d1 card-hover" style="border-radius:12px;padding:32px;position:relative;overflow:hidden;box-shadow:0 1px 3px rgba(0,0,0,0.08)">
          <div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:32px">
            <div>
              <span class="anim-fade d1" style="background:var(--badge-bg);color:var(--badge-fg);font-size:10px;font-weight:700;letter-spacing:0.1em;text-transform:uppercase;border-radius:4px;padding:2px 8px;display:inline-block;margin-bottom:16px">{badge}</span>
              <h3 style="font-size:30px;font-weight:700;letter-spacing:-0.02em;margin:0">{plan_name}</h3>
            </div>
            <button class="btn-hover" style="background:var(--btn-bg);color:var(--btn-fg);padding:10px 24px;border-radius:999px;font-size:14px;font-weight:500;border:none;cursor:pointer">{action}</button>
          </div>
          {rows}
          <div style="position:absolute;right:-80px;bottom:-80px;width:240px;height:240px;border-radius:50%;background:var(--muted);opacity:0.15;filter:blur(48px);pointer-events:none"></div>
        </section>"##,
        badge = badge_text,
        plan_name = plan_name,
        action = action_text,
        rows = rows_html,
    )
}

// ── Usage Status card ──────────────────────────

pub(super) fn build_billing_usage_status(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("");
    // Extract link from action items, or fallback to config
    let action_item = sec.items.iter()
        .find(|i| i.get("_type").map(|s| s.as_str()) == Some("action"));
    let link_text = action_item.and_then(|i| i.get("title")).map(|s| s.as_str())
        .or_else(|| sec.config.get("link_text").map(|s| s.as_str()))
        .unwrap_or("");
    let link_href = action_item.and_then(|i| i.get("link")).map(|s| s.as_str())
        .or_else(|| sec.config.get("link_href").map(|s| s.as_str()))
        .unwrap_or("");

    let mut bars_html = String::new();
    for item in &sec.items {
        // Skip action items — they are rendered as the link below
        if item.get("_type").map(|s| s.as_str()) == Some("action") {
            continue;
        }
        let label = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let usage = item.get("value").map(|s| s.as_str()).unwrap_or("");
        let percent_str = item.get("progress")
            .or_else(|| item.get("percent"))
            .map(|s| s.as_str())
            .unwrap_or("");
        let percent: u32 = percent_str.parse().unwrap_or(0);

        bars_html.push_str(&format!(
            r#"<div style="margin-bottom:32px">
              <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:8px">
                <span style="font-size:14px;font-weight:500">{label}</span>
                <span style="font-size:12px;font-weight:700">{usage}</span>
              </div>
              <div style="height:6px;background:#eeeeee;border-radius:999px;overflow:hidden">
                <div class="progress-fill" style="height:100%;width:0;background:#000;border-radius:999px;--target-width:{percent}%"></div>
              </div>
            </div>"#,
            label = label,
            usage = usage,
            percent = percent,
        ));
    }

    let link_html = if !link_text.is_empty() {
        format!(
            r#"<div style="padding-top:16px"><a href="{href}" style="font-size:14px;font-weight:700;color:#000;text-decoration:underline;text-underline-offset:4px">{text}</a></div>"#,
            href = link_href, text = link_text
        )
    } else {
        String::new()
    };

    format!(
        r##"<section class="ghost-border anim-slide-up d2 card-hover" style="background:#fff;border-radius:12px;padding:32px;box-shadow:0 1px 3px rgba(0,0,0,0.04)">
          <h4 style="font-size:12px;font-weight:700;letter-spacing:0.1em;text-transform:uppercase;color:#5e5e5e;margin:0 0 32px">{title}</h4>
          {bars}
          {link}
        </section>"##,
        title = title.to_uppercase(),
        bars = bars_html,
        link = link_html,
    )
}

// ── Billing Stats (3-col) ──────────────────────

pub(super) fn build_billing_stats(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };

    let mut cards_html = String::new();
    let mut billing_stat_idx = 0u32;
    for item in &sec.items {
        let label = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let value = item.get("value")
            .or_else(|| item.get("meta"))
            .or_else(|| item.get("description"))
            .map(|s| s.as_str())
            .unwrap_or("");
        let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("");
        billing_stat_idx += 1;
        let delay = format!("d{}", ((billing_stat_idx - 1) % 10) + 1);

        cards_html.push_str(&format!(
            r##"<div class="ghost-border anim-scale {delay} card-hover" style="border-radius:12px;padding:24px;box-shadow:0 1px 3px rgba(0,0,0,0.08)">
              <span class="material-symbols-outlined" style="color:var(--muted);margin-bottom:16px">{icon}</span>
              <div style="font-size:24px;font-weight:700;letter-spacing:-0.02em;margin-bottom:4px">{value}</div>
              <div style="font-size:12px;color:var(--muted);font-weight:500;text-transform:uppercase;letter-spacing:-0.05em">{label}</div>
            </div>"##,
            icon = icon,
            value = value,
            label = label,
        ));
    }

    let cols = sec.config.get("cols").map(|s| s.as_str()).unwrap_or("3");

    format!(
        r#"<div style="display:grid;grid-template-columns:repeat({cols},1fr);gap:24px">
          {cards}
        </div>"#,
        cols = cols,
        cards = cards_html,
    )
}

// ── Payment Methods card ───────────────────────

pub(super) fn build_billing_payment_methods(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("");
    let action_text = sec.config.get("action_text")
        .or_else(|| sec.config.get("action"))
        .map(|s| s.as_str())
        .unwrap_or("");

    let mut items_html = String::new();
    let mut card_index = 0usize;
    for item in &sec.items {
        // Skip action-type items (already rendered as header button)
        let item_type = item.get("_type").map(|s| s.as_str()).unwrap_or("");
        if item_type == "action" {
            continue;
        }

        let name = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let description = item.get("description").map(|s| s.as_str()).unwrap_or("");
        let meta = item.get("meta").map(|s| s.as_str()).unwrap_or("");
        let status = item.get("status").map(|s| s.as_str()).unwrap_or("");
        let action_icon = item.get("action_icon").map(|s| s.as_str()).unwrap_or("");

        let is_first = card_index == 0;

        // Badge: read from item config badge_type or badge, then icon
        let badge_type = item.get("badge").map(|s| s.as_str()).unwrap_or("");
        let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("");
        let badge_html = if badge_type == "dark" || (!badge_type.is_empty() && badge_type != "light") {
            // Dark badge with text (e.g. VISA)
            format!(
                r#"<div style="width:48px;height:32px;background:#171717;border-radius:4px;display:flex;align-items:center;justify-content:center;color:#fff;font-size:10px;font-weight:700;letter-spacing:-0.04em">{}</div>"#,
                badge_type.to_uppercase()
            )
        } else if !icon.is_empty() {
            // Icon badge
            format!(
                r#"<div style="width:48px;height:32px;border:1px solid #e5e7eb;border-radius:4px;display:flex;align-items:center;justify-content:center"><span class="material-symbols-outlined" style="font-size:18px">{}</span></div>"#,
                icon
            )
        } else {
            String::new()
        };

        // Description line with optional bold "Default" from meta
        let desc_html = if !meta.is_empty() && (status == "default" || meta.to_lowercase().contains("default")) {
            format!(
                r#"<div style="font-size:12px;color:#5e5e5e">{} · <span style="color:#000;font-weight:700">{}</span></div>"#,
                description, meta
            )
        } else if !description.is_empty() {
            format!(r#"<div style="font-size:12px;color:#5e5e5e">{}</div>"#, description)
        } else {
            String::new()
        };

        let bg = if is_first { "background:#f9f9f9;" } else { "" };
        let border = if is_first { "border:1px solid rgba(198,198,198,0.2);" } else { "" };

        // Action buttons: first card gets edit + more_vert, others get delete on hover
        let actions_html = if is_first {
            format!(
                r#"<div style="display:flex;align-items:center;gap:8px">
                <button style="padding:8px;border-radius:50%;background:none;border:none;cursor:pointer"><span class="material-symbols-outlined" style="font-size:16px;color:#a1a1aa">{}</span></button>
                <button style="padding:8px;border-radius:50%;background:none;border:none;cursor:pointer"><span class="material-symbols-outlined" style="font-size:16px;color:#a1a1aa">more_vert</span></button>
              </div>"#,
                action_icon
            )
        } else {
            r#"<div class="payment-hover-actions" style="display:flex;align-items:center;gap:8px;opacity:0;transition:opacity 0.15s">
                <button style="padding:8px;border-radius:50%;background:none;border:none;cursor:pointer" onmouseover="this.querySelector('span').style.color='#dc2626'" onmouseout="this.querySelector('span').style.color='#a1a1aa'"><span class="material-symbols-outlined" style="font-size:16px;color:#a1a1aa">delete</span></button>
              </div>"#.to_string()
        };

        items_html.push_str(&format!(
            r##"<div class="payment-row" style="display:flex;align-items:center;justify-content:space-between;padding:16px;border-radius:8px;{bg}{border}transition:background 0.15s" onmouseover="this.style.background='#f9f9f9';var h=this.querySelector('.payment-hover-actions');if(h)h.style.opacity='1'" onmouseout="var f={is_first};if(!f)this.style.background='';var h=this.querySelector('.payment-hover-actions');if(h)h.style.opacity='0'">
              <div style="display:flex;align-items:center;gap:16px">
                {badge}
                <div>
                  <div style="font-size:14px;font-weight:700">{name}</div>
                  {desc}
                </div>
              </div>
              {actions}
            </div>"##,
            bg = bg,
            border = border,
            is_first = is_first,
            badge = badge_html,
            name = name,
            desc = desc_html,
            actions = actions_html,
        ));
        card_index += 1;
    }

    format!(
        r##"<section class="ghost-border" style="background:#fff;border-radius:12px;padding:32px;box-shadow:0 1px 3px rgba(0,0,0,0.04)">
          <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:32px">
            <h4 style="font-size:20px;font-weight:700;margin:0">{title}</h4>
            <button style="background:transparent;color:#000;padding:8px 16px;border-radius:999px;font-size:14px;font-weight:700;border:1px solid #e5e7eb;cursor:pointer;display:flex;align-items:center;gap:8px"><span class="material-symbols-outlined" style="font-size:14px">add</span> {action}</button>
          </div>
          <div style="display:flex;flex-direction:column;gap:16px">
            {items}
          </div>
        </section>"##,
        title = title,
        action = action_text,
        items = items_html,
    )
}

// ── Recent Invoices card ───────────────────────

pub(super) fn build_billing_recent_invoices(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("");
    let action_text = sec.config.get("action_text")
        .or_else(|| sec.config.get("action"))
        .map(|s| s.as_str())
        .unwrap_or("");

    let mut rows_html = String::new();
    for item in &sec.items {
        let invoice_id = item.get("title").map(|s| s.as_str()).unwrap_or("");
        // Skip "Download All" items — rendered as the footer button only
        if invoice_id.to_lowercase().contains("download all") {
            continue;
        }
        let date = item.get("date")
            .or_else(|| item.get("description"))
            .map(|s| s.as_str())
            .unwrap_or("");
        let amount = item.get("amount")
            .or_else(|| item.get("value"))
            .map(|s| s.as_str())
            .unwrap_or("");
        let action_icon = item.get("action_icon").map(|s| s.as_str()).unwrap_or("");

        rows_html.push_str(&format!(
            r##"<div style="display:flex;justify-content:space-between;align-items:center;cursor:pointer;transition:background 0.15s" onmouseover="this.style.background='rgba(243,243,243,0.5)'" onmouseout="this.style.background='transparent'">
              <div>
                <div style="font-size:14px;font-weight:700">{invoice_id}</div>
                <div style="font-size:12px;color:#5e5e5e">{date}</div>
              </div>
              <div style="display:flex;align-items:center;gap:12px">
                <div style="font-size:14px;font-weight:700">{amount}</div>
                <span class="material-symbols-outlined" style="font-size:20px;color:#d4d4d8">{action_icon}</span>
              </div>
            </div>"##,
            invoice_id = invoice_id,
            date = date,
            amount = amount,
            action_icon = action_icon,
        ));
    }

    format!(
        r##"<section class="ghost-border" style="background:#fff;border-radius:12px;padding:32px;box-shadow:0 1px 3px rgba(0,0,0,0.04)">
          <h4 style="font-size:12px;font-weight:700;letter-spacing:0.1em;text-transform:uppercase;color:#5e5e5e;margin:0 0 24px">{title}</h4>
          <div style="display:flex;flex-direction:column;gap:24px">
            {rows}
          </div>
          <button style="width:100%;text-align:center;font-size:12px;font-weight:700;letter-spacing:0.1em;text-transform:uppercase;background:none;border:none;border-top:1px solid #f3f3f3;padding:12px 0;margin-top:16px;cursor:pointer;color:#000">{action}</button>
        </section>"##,
        title = title.to_uppercase(),
        rows = rows_html,
        action = action_text,
    )
}

// ════════════════════════════════════════════════
// ██  PAYOUTS DASHBOARD  ████████████████████████
// ════════════════════════════════════════════════

pub fn render_payouts_dashboard(
    app_name: &str,
    sections: &[SectionNode],
    components: &[crate::parser::ComponentNode],
    theme: &str,
    current_route: &str,
) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    let _ = theme;

    // ── Extract component data ──────────────────────

    let sidebar_comp = components.iter().find(|c| c.layout.as_deref() == Some("sidebar"));
    let topbar_comp = components.iter().find(|c| {
        c.layout.as_deref() == Some("inline") && c.style.as_deref().map(|s| s.contains("topbar")).unwrap_or(false)
    });
    let sidebar_section = sections.iter().find(|s| s.section_type == "sidebar");

    // ── Extract section data by type ────────────────

    let page_header = sections.iter().find(|s| s.section_type == "page-header");
    let balance_card = sections.iter().find(|s| s.section_type == "balance-card");
    let upcoming_card = sections.iter().find(|s| s.section_type == "upcoming-card");
    let payout_history = sections.iter().find(|s| s.section_type == "payout-history");
    let support_banner = sections.iter().find(|s| s.section_type == "support-banner");

    // ── Build sidebar + topbar (reuse) ─────────────

    let sidebar_html = build_dashboard_sidebar(sidebar_comp, sidebar_section, current_route);
    let topbar_html = build_dashboard_topbar(topbar_comp);

    // ── Build sections ──────────────────────────────

    let header_html = build_payouts_page_header(page_header);
    let balance_html = build_payouts_balance_card(balance_card);
    let upcoming_html = build_payouts_upcoming_card(upcoming_card);
    let history_html = build_payouts_history(payout_history);
    let support_html = build_payouts_support_banner(support_banner);

    // ── Assemble complete page ──────────────────────

    format!(
        r##"<!DOCTYPE html>
<html class="light" lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta name="view-transition" content="same-origin">
  <title>{app_name}</title>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap" rel="stylesheet">
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet">
  <style>
    body {{ font-family:'Inter',sans-serif; background:#f9f9f9; color:#1a1c1c; margin:0; }}
    * {{ box-sizing:border-box; }}
    .material-symbols-outlined {{ font-variation-settings:'FILL' 0,'wght' 400,'GRAD' 0,'opsz' 24; font-size:20px; display:inline-block; line-height:1; vertical-align:middle; }}
    ::selection {{ background:rgba(0,111,240,0.15); }}
    ::-webkit-scrollbar {{ width:4px; }}
    ::-webkit-scrollbar-thumb {{ background:rgba(0,0,0,0.1); border-radius:2px; }}
    .ghost-border {{ border:1px solid rgba(198,198,198,0.2); }}
    .payout-row:hover {{ background:rgba(243,243,243,0.5); }}
  </style>
  <style>{anim_css}</style>
</head>
<body>

{sidebar}

{topbar}

<main style="margin-left:256px;min-height:100vh;background:radial-gradient(circle at top right,rgba(0,111,240,0.08),transparent 40%),radial-gradient(circle at bottom left,rgba(0,56,129,0.05),transparent 40%)">
  <div style="max-width:1152px;margin:0 auto;padding:32px">

    {header}

    <div style="display:grid;grid-template-columns:repeat(12,1fr);gap:24px">

      <!-- Balance Card (col-span 8) -->
      <div style="grid-column:span 8">
        {balance}
      </div>

      <!-- Upcoming Card (col-span 4) -->
      <div style="grid-column:span 4">
        {upcoming}
      </div>

      <!-- history-section -->
      <div style="grid-column:span 12">
        {history}
      </div>

      <!-- Support Banner (col-span 12) -->
      <div style="grid-column:span 12">
        {support}
      </div>

    </div>
    <div style="height:96px"></div>
  </div>
</main>

<script{script_nonce}>{runtime}</script>
<script{script_nonce}>{hmr}</script>
{anim_js}
</body>
</html>"##,
        app_name = app_name,
        sidebar = sidebar_html,
        topbar = topbar_html,
        header = header_html,
        balance = balance_html,
        upcoming = upcoming_html,
        history = history_html,
        support = support_html,
        anim_css = CRONUS_ANIMATIONS_CSS,
        anim_js = crate::security::mark_kernel_scripts(CRONUS_ANIMATIONS_JS),
        runtime = crate::render::CRONUS_RUNTIME_JS,
        hmr = crate::hmr::HMR_CLIENT_JS,
    )
}

// ── Payouts page header ────────────────────────

pub(super) fn build_payouts_page_header(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("");
    let subtitle = sec.subtitle.as_deref().unwrap_or("");

    // Extract action from items
    let action_item = sec.items.iter()
        .find(|i| i.get("_type").map(|s| s.as_str()) == Some("action"));
    let action_text = action_item.and_then(|i| i.get("title")).map(|s| s.as_str()).unwrap_or("");
    let action_icon = action_item.and_then(|i| i.get("icon")).map(|s| s.as_str()).unwrap_or("");

    let subtitle_html = if !subtitle.is_empty() {
        format!(
            r#"<p style="color:#5e5e5e;font-size:18px;margin:8px 0 0;line-height:1.5">{}</p>"#,
            subtitle
        )
    } else {
        String::new()
    };

    let action_html = if !action_text.is_empty() {
        let icon_html = if !action_icon.is_empty() {
            format!(r#"<span class="material-symbols-outlined" style="font-size:16px">{}</span>"#, action_icon)
        } else {
            String::new()
        };
        format!(
            r#"<button style="background:#000;color:#fff;padding:10px 24px;border-radius:999px;font-size:14px;font-weight:500;border:none;cursor:pointer;display:flex;align-items:center;gap:8px">{icon} {text}</button>"#,
            icon = icon_html,
            text = action_text
        )
    } else {
        String::new()
    };

    format!(
        r#"<div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:48px">
      <div>
        <h2 class="anim-slide-up d1" style="font-size:36px;font-weight:800;letter-spacing:-0.05em;line-height:1.25;color:#1a1c1c;margin:0 0 8px">{title}</h2>
        <div class="anim-slide-up d2">{subtitle}</div>
      </div>
      <div class="anim-scale d3 btn-hover">{action}</div>
    </div>"#,
        title = title,
        subtitle = subtitle_html,
        action = action_html,
    )
}

// ── Balance Card ───────────────────────────────

pub(super) fn build_payouts_balance_card(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let label = sec.config.get("badge").map(|s| s.as_str()).unwrap_or("");
    let value = sec.title.as_deref().unwrap_or("");
    let unit = sec.subtitle.as_deref().unwrap_or("");

    // Build info badges from non-action items
    let mut badges_html = String::new();
    for item in &sec.items {
        if item.get("_type").map(|s| s.as_str()) == Some("action") {
            continue;
        }
        let text = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("");
        if !text.is_empty() {
            let icon_html = if !icon.is_empty() {
                format!(r#"<span class="material-symbols-outlined" style="font-size:14px;color:#5e5e5e">{}</span>"#, icon)
            } else {
                String::new()
            };
            badges_html.push_str(&format!(
                r#"<span style="display:inline-flex;align-items:center;gap:6px;background:#f3f3f3;padding:6px 12px;border-radius:999px;font-size:12px;font-weight:500;color:#5e5e5e">{icon} {text}</span>"#,
                icon = icon_html,
                text = text
            ));
        }
    }

    format!(
        r##"<section class="ghost-border anim-slide-up d1 card-hover" style="background:#fff;border-radius:12px;padding:32px;position:relative;overflow:hidden;box-shadow:0 1px 3px rgba(0,0,0,0.04)">
          <div style="font-size:10px;font-weight:700;letter-spacing:0.1em;text-transform:uppercase;color:#5e5e5e;margin-bottom:16px">{label}</div>
          <div style="display:flex;align-items:baseline;gap:16px;margin-bottom:24px">
            <span style="font-size:48px;font-weight:800;letter-spacing:-0.04em;line-height:1">{value}</span>
            <span style="background:rgba(0,111,240,0.1);color:#006ff0;font-size:12px;font-weight:700;padding:4px 12px;border-radius:999px">{unit}</span>
          </div>
          <div style="display:flex;gap:12px;flex-wrap:wrap">
            {badges}
          </div>
          <div style="position:absolute;right:-80px;bottom:-80px;width:240px;height:240px;border-radius:50%;background:rgba(0,111,240,0.08);filter:blur(48px);pointer-events:none"></div>
        </section>"##,
        label = label,
        value = value,
        unit = unit,
        badges = badges_html,
    )
}

// ── Upcoming Card ──────────────────────────────

pub(super) fn build_payouts_upcoming_card(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let label = sec.title.as_deref().unwrap_or("");
    let value = sec.config.get("badge").map(|s| s.as_str()).unwrap_or("");
    let date = sec.subtitle.as_deref().unwrap_or("");

    // Build detail rows from row items
    let mut rows_html = String::new();
    for item in &sec.items {
        if item.get("_type").map(|s| s.as_str()) == Some("action") {
            continue;
        }
        let row_label = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let row_value = item.get("value").map(|s| s.as_str()).unwrap_or("");
        let is_negative = row_value.starts_with('-');
        let value_color = if is_negative { "color:#dc2626;" } else { "" };

        rows_html.push_str(&format!(
            r#"<div style="display:flex;justify-content:space-between;align-items:center">
              <span style="font-size:13px;color:#5e5e5e">{label}</span>
              <span style="font-size:13px;font-weight:600;{color}">{value}</span>
            </div>"#,
            label = row_label,
            value = row_value,
            color = value_color,
        ));
    }

    format!(
        r##"<section class="ghost-border anim-slide-up d2 card-hover" style="background:#fff;border-radius:12px;padding:32px;box-shadow:0 1px 3px rgba(0,0,0,0.04)">
          <div style="font-size:12px;font-weight:700;letter-spacing:0.1em;text-transform:uppercase;color:#5e5e5e;margin-bottom:16px">{label}</div>
          <div style="font-size:24px;font-weight:700;letter-spacing:-0.02em;margin-bottom:4px">{value}</div>
          <div style="font-size:12px;color:#5e5e5e;margin-bottom:24px">{date}</div>
          <div style="border-top:1px solid #f3f3f3;padding-top:16px;display:flex;flex-direction:column;gap:12px">
            {rows}
          </div>
        </section>"##,
        label = label,
        value = value,
        date = date,
        rows = rows_html,
    )
}

// ── Payout History table ───────────────────────

pub(super) fn build_payouts_history(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("");
    let columns_raw = sec.config.get("columns").map(|s| s.as_str()).unwrap_or("");
    let columns: Vec<&str> = columns_raw.split(',').map(|c| c.trim()).filter(|c| !c.is_empty()).collect();
    let footnote = sec.config.get("footnote").map(|s| s.as_str()).unwrap_or("");

    // Extract action buttons
    let actions: Vec<&std::collections::HashMap<String, String>> = sec.items.iter()
        .filter(|i| i.get("_type").map(|s| s.as_str()) == Some("action"))
        .collect();

    let mut action_buttons_html = String::new();
    for action in &actions {
        let text = action.get("title").map(|s| s.as_str()).unwrap_or("");
        let icon = action.get("icon").map(|s| s.as_str()).unwrap_or("");
        let icon_html = if !icon.is_empty() {
            format!(r#"<span class="material-symbols-outlined" style="font-size:14px">{}</span>"#, icon)
        } else {
            String::new()
        };
        action_buttons_html.push_str(&format!(
            r#"<button class="btn-hover" style="background:transparent;color:#000;padding:8px 16px;border-radius:999px;font-size:13px;font-weight:600;border:1px solid #e5e7eb;cursor:pointer;display:flex;align-items:center;gap:6px">{icon} {text}</button>"#,
            icon = icon_html,
            text = text
        ));
    }

    // Build table header
    let mut thead_html = String::new();
    for col in &columns {
        thead_html.push_str(&format!(
            r#"<th style="text-align:left;font-size:11px;font-weight:600;letter-spacing:0.05em;text-transform:uppercase;color:#a1a1aa;padding:12px 16px;border-bottom:1px solid #f3f3f3">{}</th>"#,
            col
        ));
    }

    // Build table rows from non-action items
    let mut tbody_html = String::new();
    for item in &sec.items {
        if item.get("_type").map(|s| s.as_str()) == Some("action") {
            continue;
        }
        let date = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let time = item.get("time").map(|s| s.as_str()).unwrap_or("");
        let amount = item.get("amount").map(|s| s.as_str()).unwrap_or("");
        let destination = item.get("destination").map(|s| s.as_str()).unwrap_or("");
        let status = item.get("status").map(|s| s.as_str()).unwrap_or("");
        let reference = item.get("reference").map(|s| s.as_str()).unwrap_or("");

        // Colors from status value — label comes from .cronus status_label or capitalized status
        let status_label = item.get("status_label").map(|s| s.as_str()).unwrap_or(status);
        let (status_color, status_bg) = match status.to_lowercase().as_str() {
            "success" => ("#16a34a", "rgba(22,163,74,0.1)"),
            "processing" => ("#006ff0", "rgba(0,111,240,0.1)"),
            "failed" => ("#dc2626", "rgba(220,38,38,0.1)"),
            _ => ("#5e5e5e", "#f3f3f3"),
        };

        let row_delay = format!("d{}", (tbody_html.matches("<tr").count() % 10) + 1);
        tbody_html.push_str(&format!(
            r##"<tr class="payout-row anim-fade {row_delay}" style="transition:background 0.15s;cursor:pointer">
              <td style="padding:16px;border-bottom:1px solid #f9f9f9">
                <div style="font-size:14px;font-weight:600">{date}</div>
                <div style="font-size:11px;color:#a1a1aa">{time}</div>
              </td>
              <td style="padding:16px;border-bottom:1px solid #f9f9f9;font-size:14px;font-weight:700">{amount}</td>
              <td style="padding:16px;border-bottom:1px solid #f9f9f9;font-size:13px;color:#5e5e5e">{destination}</td>
              <td style="padding:16px;border-bottom:1px solid #f9f9f9">
                <span style="display:inline-block;padding:4px 12px;border-radius:999px;font-size:11px;font-weight:700;color:{status_color};background:{status_bg}">{status_label}</span>
              </td>
              <td style="padding:16px;border-bottom:1px solid #f9f9f9;font-family:'SF Mono','Fira Code',monospace;font-size:12px;color:#5e5e5e">{reference}</td>
            </tr>"##,
            row_delay = row_delay,
            date = date,
            time = time,
            amount = amount,
            destination = destination,
            status_color = status_color,
            status_bg = status_bg,
            status_label = status_label,
            reference = reference,
        ));
    }

    let footnote_html = if !footnote.is_empty() {
        format!(
            r#"<div style="padding:16px;font-size:12px;color:#a1a1aa;text-align:center;border-top:1px solid #f3f3f3">{}</div>"#,
            footnote
        )
    } else {
        String::new()
    };

    format!(
        r##"<section class="ghost-border anim-slide-up d3 card-hover" style="background:#fff;border-radius:12px;box-shadow:0 1px 3px rgba(0,0,0,0.04);overflow:hidden">
          <div style="display:flex;justify-content:space-between;align-items:center;padding:24px 24px 0">
            <h4 style="font-size:20px;font-weight:700;margin:0">{title}</h4>
            <div style="display:flex;gap:8px">
              {action_buttons}
            </div>
          </div>
          <div style="padding:16px 0 0;overflow-x:auto">
            <table style="width:100%;border-collapse:collapse">
              <thead><tr>{thead}</tr></thead>
              <tbody>{tbody}</tbody>
            </table>
          </div>
          {footnote}
        </section>"##,
        title = title,
        action_buttons = action_buttons_html,
        thead = thead_html,
        tbody = tbody_html,
        footnote = footnote_html,
    )
}

// ── Support Banner ─────────────────────────────

pub(super) fn build_payouts_support_banner(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("");
    let subtitle = sec.subtitle.as_deref().unwrap_or("");

    let action_item = sec.items.iter()
        .find(|i| i.get("_type").map(|s| s.as_str()) == Some("action"));
    let action_text = action_item.and_then(|i| i.get("title")).map(|s| s.as_str()).unwrap_or("");

    format!(
        r##"<section class="anim-scale d3" style="background:#0a0a0a;border-radius:12px;padding:40px;position:relative;overflow:hidden">
          <div style="position:absolute;right:-20%;top:-40%;width:60%;height:180%;background:rgba(255,255,255,0.05);transform:skewX(-12deg);pointer-events:none"></div>
          <div style="position:relative;z-index:1">
            <h4 style="font-size:20px;font-weight:700;color:#fff;margin:0 0 8px">{title}</h4>
            <p style="font-size:14px;color:#a1a1aa;margin:0 0 24px;max-width:560px;line-height:1.6">{subtitle}</p>
            <button class="btn-hover" style="background:#fff;color:#000;padding:10px 24px;border-radius:999px;font-size:14px;font-weight:600;border:none;cursor:pointer">{action}</button>
          </div>
        </section>"##,
        title = title,
        subtitle = subtitle,
        action = action_text,
    )
}

// ── Unified: Usage & Plan card (merged current-plan into usage-status) ──

pub(super) fn build_unified_usage_plan(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };

    // Badge = plan name from config, e.g. "Enterprise"
    let plan_name = sec.config.get("plan").map(|s| s.as_str()).unwrap_or("Enterprise");

    // Progress bars for items that are NOT action / row type
    let mut bars_html = String::new();
    let mut cost_html = String::new();
    let mut action_link = String::new();

    for item in &sec.items {
        let item_type = item.get("_type").map(|s| s.as_str()).unwrap_or("");
        if item_type == "action" {
            let text = item.get("title").map(|s| s.as_str()).unwrap_or("Manage plan");
            let href = item.get("link").map(|s| s.as_str()).unwrap_or("#");
            action_link = format!(
                r#"<div style="padding-top:12px"><a href="{href}" style="font-size:14px;font-weight:700;color:#1a1c1c;text-decoration:underline;text-underline-offset:4px">{text}</a></div>"#,
                href = href, text = text
            );
            continue;
        }
        if item_type == "row" {
            let label = item.get("title").map(|s| s.as_str()).unwrap_or("Monthly cost");
            let value = item.get("value").map(|s| s.as_str()).unwrap_or("");
            cost_html = format!(
                r#"<div style="display:flex;justify-content:space-between;align-items:center;padding:12px 0;border-top:1px solid #f3f3f3">
                  <span style="font-size:14px;color:#5e5e5e">{label}</span>
                  <span style="font-size:14px;font-weight:600">{value}</span>
                </div>"#,
                label = label, value = value
            );
            continue;
        }
        // Progress bar items
        let label = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let usage = item.get("value").map(|s| s.as_str()).unwrap_or("");
        let percent_str = item.get("progress")
            .or_else(|| item.get("percent"))
            .map(|s| s.as_str())
            .unwrap_or("");
        let percent: u32 = percent_str.parse().unwrap_or(0);

        bars_html.push_str(&format!(
            r#"<div style="margin-bottom:20px">
              <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:6px">
                <span style="font-size:14px;font-weight:500">{label}</span>
                <span style="font-size:12px;font-weight:700">{usage}</span>
              </div>
              <div style="height:6px;background:#eeeeee;border-radius:999px;overflow:hidden">
                <div class="progress-fill" style="height:100%;width:0;background:#000;border-radius:999px;--target-width:{percent}%"></div>
              </div>
            </div>"#,
            label = label, usage = usage, percent = percent
        ));
    }

    // If no explicit action link found, provide default
    if action_link.is_empty() {
        let link_text = sec.config.get("link_text").map(|s| s.as_str()).unwrap_or("Manage plan");
        let link_href = sec.config.get("link_href").map(|s| s.as_str()).unwrap_or("#");
        if !link_text.is_empty() {
            action_link = format!(
                r#"<div style="padding-top:12px"><a href="{href}" style="font-size:14px;font-weight:700;color:#1a1c1c;text-decoration:underline;text-underline-offset:4px">{text}</a></div>"#,
                href = link_href, text = link_text
            );
        }
    }

    format!(
        r##"<section class="ghost-border" style="background:#fff;border-radius:12px;padding:32px;box-shadow:0 1px 3px rgba(0,0,0,0.04)">
          <div style="display:flex;align-items:center;gap:12px;margin-bottom:24px">
            <h4 style="font-size:12px;font-weight:700;letter-spacing:0.1em;text-transform:uppercase;color:#5e5e5e;margin:0">USAGE &amp; PLAN</h4>
            <span style="font-size:11px;font-weight:700;letter-spacing:0.05em;text-transform:uppercase;background:#000;color:#fff;padding:3px 10px;border-radius:999px">{plan}</span>
          </div>
          {bars}
          {cost}
          {action}
        </section>"##,
        plan = plan_name,
        bars = bars_html,
        cost = cost_html,
        action = action_link,
    )
}

// ── Unified: Billing & Payments (merged payment-methods + recent-invoices) ──

pub(super) fn build_unified_billing_payments(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };

    // Separate payment method cards vs invoice rows
    let mut methods_html = String::new();
    let mut invoices_html = String::new();

    for item in &sec.items {
        let item_type = item.get("_type").map(|s| s.as_str()).unwrap_or("");
        if item_type == "row" {
            // Invoice row: title (period) + date + amount + download icon
            let title = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let date = item.get("date")
                .or_else(|| item.get("description"))
                .map(|s| s.as_str())
                .unwrap_or("");
            let amount = item.get("amount")
                .or_else(|| item.get("value"))
                .map(|s| s.as_str())
                .unwrap_or("");

            invoices_html.push_str(&format!(
                r##"<div style="display:flex;justify-content:space-between;align-items:center;padding:8px 0">
                  <div>
                    <div style="font-size:14px;font-weight:700">{title}</div>
                    <div style="font-size:12px;color:#5e5e5e">{date}</div>
                  </div>
                  <div style="display:flex;align-items:center;gap:8px">
                    <span style="font-size:14px;font-weight:700">{amount}</span>
                    <span class="material-symbols-outlined" style="font-size:16px;color:#d4d4d8">download</span>
                  </div>
                </div>"##,
                title = title, date = date, amount = amount
            ));
        } else {
            // Payment method card: VISA badge or icon + name + desc
            let name = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let desc = item.get("description").map(|s| s.as_str()).unwrap_or("");
            let badge = item.get("badge").map(|s| s.as_str()).unwrap_or("");
            let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("");

            if !name.is_empty() {
                let badge_html = if !badge.is_empty() {
                    format!(
                        r#"<div style="width:48px;height:32px;background:#171717;border-radius:4px;display:flex;align-items:center;justify-content:center;color:#fff;font-size:10px;font-weight:700">{}</div>"#,
                        badge.to_uppercase()
                    )
                } else if !icon.is_empty() {
                    format!(
                        r#"<div style="width:48px;height:32px;background:#171717;border-radius:4px;display:flex;align-items:center;justify-content:center;color:#fff"><span class="material-symbols-outlined" style="font-size:18px">{}</span></div>"#,
                        icon
                    )
                } else {
                    String::new()
                };

                methods_html.push_str(&format!(
                    r##"<div style="display:flex;align-items:center;gap:16px;padding:12px;background:#f9f9f9;border-radius:8px;margin-bottom:8px">
                      {badge_html}
                      <div>
                        <div style="font-size:14px;font-weight:700">{name}</div>
                        <div style="font-size:12px;color:#5e5e5e">{desc}</div>
                      </div>
                    </div>"##,
                    badge_html = badge_html, name = name, desc = desc
                ));
            }
        }
    }

    let has_methods = !methods_html.is_empty();
    let has_invoices = !invoices_html.is_empty();

    let divider = if has_methods && has_invoices {
        r#"<div style="border-top:1px solid #f3f3f3;margin:16px 0"></div>"#
    } else {
        ""
    };

    format!(
        r##"<section class="ghost-border" style="background:#fff;border-radius:12px;padding:32px;box-shadow:0 1px 3px rgba(0,0,0,0.04)">
          <h4 style="font-size:12px;font-weight:700;letter-spacing:0.1em;text-transform:uppercase;color:#5e5e5e;margin:0 0 20px">BILLING &amp; PAYMENTS</h4>
          {methods}
          {divider}
          {invoices}
        </section>"##,
        methods = methods_html,
        divider = divider,
        invoices = invoices_html,
    )
}

// ════════════════════════════════════════════════
// ██  UNIFIED DASHBOARD  ████████████████████████
// ════════════════════════════════════════════════

pub fn render_unified_dashboard(
    app_name: &str,
    sections: &[SectionNode],
    components: &[crate::parser::ComponentNode],
    theme: &str,
    current_route: &str,
) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    let _ = theme;

    // ── Extract component data ──────────────────────

    let sidebar_comp = components.iter().find(|c| c.layout.as_deref() == Some("sidebar"));
    let topbar_comp = components.iter().find(|c| {
        c.layout.as_deref() == Some("inline") && c.style.as_deref().map(|s| s.contains("topbar")).unwrap_or(false)
    });
    let sidebar_section = sections.iter().find(|s| s.section_type == "sidebar");

    // ── Find sections by type (no current-plan / payment-methods) ──

    let page_header = sections.iter().find(|s| s.section_type == "page-header");
    let balance = sections.iter().find(|s| s.section_type == "balance-card");
    let usage = sections.iter().find(|s| s.section_type == "usage-status");
    let stats = sections.iter().find(|s| s.section_type == "billing-stats");
    let history = sections.iter().find(|s| s.section_type == "payout-history");
    let invoices = sections.iter().find(|s| s.section_type == "recent-invoices");
    let support = sections.iter().find(|s| s.section_type == "support-banner");

    // ── Build sidebar + topbar (reuse) ─────────────

    let sidebar_html = build_dashboard_sidebar(sidebar_comp, sidebar_section, current_route);
    let topbar_html = build_dashboard_topbar(topbar_comp);

    // ── Build page header (same pattern as payouts) ─

    let header_html = build_payouts_page_header(page_header);

    // ── Build each section ─────────────────────────

    let balance_cols = balance.and_then(|s| s.config.get("cols")).map(|s| s.as_str()).unwrap_or("8");
    let balance_html = build_payouts_balance_card(balance);

    let usage_cols = usage.and_then(|s| s.config.get("cols")).map(|s| s.as_str()).unwrap_or("4");
    let usage_html = build_unified_usage_plan(usage);

    let stats_html = build_billing_stats(stats);

    let history_cols = history.and_then(|s| s.config.get("cols")).map(|s| s.as_str()).unwrap_or("8");
    let history_html = build_payouts_history(history);

    let invoices_cols = invoices.and_then(|s| s.config.get("cols")).map(|s| s.as_str()).unwrap_or("4");
    let invoices_html = build_unified_billing_payments(invoices);

    let support_html = build_payouts_support_banner(support);

    // ── Assemble complete page ──────────────────────

    format!(
        r##"<!DOCTYPE html>
<html class="light" lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta name="view-transition" content="same-origin">
  <title>{app_name}</title>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap" rel="stylesheet">
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet">
  <style>
    body {{ font-family:'Inter',sans-serif; background:#f9f9f9; color:#1a1c1c; margin:0; }}
    * {{ box-sizing:border-box; }}
    .material-symbols-outlined {{ font-variation-settings:'FILL' 0,'wght' 400,'GRAD' 0,'opsz' 24; font-size:20px; display:inline-block; line-height:1; vertical-align:middle; }}
    ::selection {{ background:rgba(0,111,240,0.15); }}
    ::-webkit-scrollbar {{ width:4px; }}
    ::-webkit-scrollbar-thumb {{ background:rgba(0,0,0,0.1); border-radius:2px; }}
    .ghost-border {{ border:1px solid rgba(198,198,198,0.2); }}
    .payment-row:hover .payment-hover-actions {{ opacity:1 !important; }}
    .payout-row:hover {{ background:rgba(243,243,243,0.5); }}
  </style>
  <style>{anim_css}</style>
</head>
<body>

{sidebar}

{topbar}

<main style="margin-left:256px;min-height:100vh;background:radial-gradient(circle at top right,rgba(0,111,240,0.08),transparent 40%),radial-gradient(circle at bottom left,rgba(0,56,129,0.05),transparent 40%)">
  <div style="max-width:1280px;margin:0 auto;padding:32px">

    {header}

    <div style="display:grid;grid-template-columns:repeat(12,1fr);gap:24px">

      <!-- Row 1: balance(8) + usage-status(4) -->
      <div style="grid-column:span {balance_cols}">
        {balance}
      </div>
      <div style="grid-column:span {usage_cols}">
        {usage}
      </div>

      <!-- Row 2: billing-stats(12) — 4 stat cards sub-grid -->
      <div style="grid-column:span 12">
        {stats}
      </div>

      <!-- Row 3: payout-history(8) + recent-invoices(4) -->
      <div style="grid-column:span {history_cols}">
        {history}
      </div>
      <div style="grid-column:span {invoices_cols}">
        {invoices}
      </div>

      <!-- Row 4: support-banner(12) -->
      <div style="grid-column:span 12">
        {support}
      </div>

    </div>
    <div style="height:96px"></div>
  </div>
</main>

<script{script_nonce}>{runtime}</script>
<script{script_nonce}>{hmr}</script>
{anim_js}
</body>
</html>"##,
        app_name = app_name,
        sidebar = sidebar_html,
        topbar = topbar_html,
        header = header_html,
        balance_cols = balance_cols,
        balance = balance_html,
        usage_cols = usage_cols,
        usage = usage_html,
        stats = stats_html,
        history_cols = history_cols,
        history = history_html,
        invoices_cols = invoices_cols,
        invoices = invoices_html,
        support = support_html,
        anim_css = CRONUS_ANIMATIONS_CSS,
        anim_js = crate::security::mark_kernel_scripts(CRONUS_ANIMATIONS_JS),
        runtime = crate::render::CRONUS_RUNTIME_JS,
        hmr = crate::hmr::HMR_CLIENT_JS,
    )
}

// ══════════════════════════════════════════════════
// PAYMENT LINKS DASHBOARD — GeistPay
// ══════════════════════════════════════════════════

pub fn render_payment_links_dashboard(
    app_name: &str,
    sections: &[SectionNode],
    components: &[crate::parser::ComponentNode],
    theme: &str,
    current_route: &str,
) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    let _ = theme;

    // ── Extract component data ──────────────────────

    let sidebar_comp = components.iter().find(|c| c.layout.as_deref() == Some("sidebar"));
    let topbar_comp = components.iter().find(|c| {
        c.layout.as_deref() == Some("inline") && c.style.as_deref().map(|s| s.contains("topbar")).unwrap_or(false)
    });
    let sidebar_section = sections.iter().find(|s| s.section_type == "sidebar");

    // ── Extract section data by type ────────────────

    let page_header = sections.iter().find(|s| s.section_type == "page-header");
    let stat_cards = sections.iter().find(|s| s.section_type == "stat-cards");
    let promo = sections.iter().find(|s| s.section_type == "promo");
    let info_bar = sections.iter().find(|s| s.section_type == "info-bar");

    // Product grids — may be multiple (main grid + draft card)
    let product_grids: Vec<&SectionNode> = sections.iter()
        .filter(|s| s.section_type == "product-grid")
        .collect();

    // ── Build sidebar + topbar (reuse) ─────────────

    let sidebar_html = build_dashboard_sidebar(sidebar_comp, sidebar_section, current_route);
    let topbar_html = build_dashboard_topbar(topbar_comp);

    // ── Build page header ───────────────────────────

    let header_html = build_payment_links_page_header(page_header);

    // ── Build stat cards ────────────────────────────

    let stat_cards_html = build_payment_links_stat_cards(stat_cards);

    // ── Build product grid(s) ───────────────────────

    let mut product_grid_html = String::new();
    for grid in &product_grids {
        product_grid_html.push_str(&build_payment_links_product_grid(grid));
    }

    // ── Build promo banner ──────────────────────────

    let promo_html = build_payment_links_promo(promo);

    // ── Build info bar (footer) ─────────────────────

    let info_bar_html = build_payment_links_info_bar(info_bar);

    // ── Assemble complete page ──────────────────────

    format!(
        r##"<!DOCTYPE html>
<html class="light" lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta name="view-transition" content="same-origin">
  <title>{app_name}</title>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap" rel="stylesheet">
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet">
  <style>
    body {{ font-family:'Inter',sans-serif; background:#f9f9f9; color:#1a1c1c; margin:0; }}
    * {{ box-sizing:border-box; }}
    .material-symbols-outlined {{ font-variation-settings:'FILL' 0,'wght' 400,'GRAD' 0,'opsz' 24; font-size:20px; display:inline-block; line-height:1; vertical-align:middle; }}
    ::selection {{ background:rgba(0,111,240,0.15); }}
    ::-webkit-scrollbar {{ width:4px; }}
    ::-webkit-scrollbar-thumb {{ background:rgba(0,0,0,0.1); border-radius:2px; }}
    .ghost-border {{ border:1px solid rgba(198,198,198,0.2); }}
    .prism-bg {{
      background: radial-gradient(circle at 50% -20%, rgba(0,111,240,0.06) 0%, transparent 50%),
                  radial-gradient(circle at 0% 100%, rgba(0,56,129,0.04) 0%, transparent 40%);
    }}
    .engineering-grid {{
      background-size: 40px 40px;
      background-image: linear-gradient(to right, rgba(0,0,0,0.03) 1px, transparent 1px),
                        linear-gradient(to bottom, rgba(0,0,0,0.03) 1px, transparent 1px);
    }}
  </style>
  <style>{anim_css}</style>
</head>
<body>

{sidebar}

{topbar}

<main style="margin-left:256px;min-height:100vh;display:flex;flex-direction:column">
  <section class="prism-bg engineering-grid" style="flex:1;padding:32px">
    <div style="max-width:1152px;margin:0 auto">

      {header}

      {stat_cards}

      <div style="display:grid;grid-template-columns:repeat(3,1fr);gap:24px">
        {product_grid}

        {promo}
      </div>

    </div>
  </section>

  {info_bar}
</main>

<script{script_nonce}>{runtime}</script>
<script{script_nonce}>{hmr}</script>
{anim_js}
</body>
</html>"##,
        app_name = app_name,
        sidebar = sidebar_html,
        topbar = topbar_html,
        header = header_html,
        stat_cards = stat_cards_html,
        product_grid = product_grid_html,
        promo = promo_html,
        info_bar = info_bar_html,
        anim_css = CRONUS_ANIMATIONS_CSS,
        anim_js = crate::security::mark_kernel_scripts(CRONUS_ANIMATIONS_JS),
        runtime = crate::render::CRONUS_RUNTIME_JS,
        hmr = crate::hmr::HMR_CLIENT_JS,
    )
}

// ── Payment Links: page header ─────────────────

pub(super) fn build_payment_links_page_header(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("");
    let subtitle = sec.subtitle.as_deref().unwrap_or("");

    // Extract action from items with _type=action
    let action_item = sec.items.iter()
        .find(|i| i.get("_type").map(|s| s.as_str()) == Some("action"));
    let action_text = action_item.and_then(|i| i.get("title")).map(|s| s.as_str()).unwrap_or("");
    let action_icon = action_item.and_then(|i| i.get("icon")).map(|s| s.as_str()).unwrap_or("add");

    let subtitle_html = if !subtitle.is_empty() {
        format!(r#"<p style="color:#5e5e5e;font-size:16px;font-weight:500;margin:0">{}</p>"#, subtitle)
    } else {
        String::new()
    };

    let action_html = if !action_text.is_empty() {
        format!(
            r##"<button style="display:flex;align-items:center;gap:8px;background:#000;color:#fff;border:none;padding:12px 24px;border-radius:999px;font-size:14px;font-weight:700;cursor:pointer;letter-spacing:-0.02em;box-shadow:0 20px 40px rgba(0,0,0,0.05);transition:all 0.15s" onmouseover="this.style.transform='scale(1.02)'" onmouseout="this.style.transform='scale(1)'">
  <span class="material-symbols-outlined" style="font-size:20px">{icon}</span>
  {text}
</button>"##,
            icon = action_icon, text = action_text
        )
    } else {
        String::new()
    };

    format!(
        r##"<div style="display:flex;flex-wrap:wrap;align-items:flex-end;justify-content:space-between;gap:24px;margin-bottom:48px">
  <div>
    <h1 class="anim-slide-up d1" style="font-size:36px;font-weight:800;letter-spacing:-0.04em;color:#000;margin:0 0 8px">{title}</h1>
    <div class="anim-slide-up d2">{subtitle}</div>
  </div>
  <div class="anim-scale d3 btn-hover">{action}</div>
</div>"##,
        title = title, subtitle = subtitle_html, action = action_html,
    )
}

// ── Payment Links: stat cards ──────────────────

pub(super) fn build_payment_links_stat_cards(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };

    let cols = sec.config.get("cols")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(3);

    let cards: Vec<String> = sec.items.iter()
        .filter(|i| i.get("_type").map(|s| s.as_str()) != Some("action"))
        .map(|item| {
            let label = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let value = item.get("description").map(|s| s.as_str()).unwrap_or("");

            format!(
                r##"<div class="anim-scale card-hover" style="background:#fff;border:1px solid rgba(198,198,198,0.2);border-radius:12px;padding:24px">
  <p style="font-size:12px;font-weight:600;text-transform:uppercase;letter-spacing:0.1em;color:#5e5e5e;margin:0 0 4px">{label}</p>
  <p style="font-size:24px;font-weight:700;letter-spacing:-0.02em;color:#1a1c1c;margin:0">{value}</p>
</div>"##,
                label = label, value = value,
            )
        })
        .collect();

    format!(
        r##"<div style="display:grid;grid-template-columns:repeat({cols},1fr);gap:24px;margin-bottom:48px">
  {items}
</div>"##,
        cols = cols, items = cards.join("\n  "),
    )
}

// ── Payment Links: product grid ────────────────

pub(super) fn build_payment_links_product_grid(section: &SectionNode) -> String {
    let cards: Vec<String> = section.items.iter()
        .filter(|i| i.get("_type").map(|s| s.as_str()) != Some("action"))
        .map(|item| {
            let name = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let desc = item.get("description").map(|s| s.as_str()).unwrap_or("");
            let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("");
            let status = item.get("status").map(|s| s.as_str());
            let price = item.get("price").map(|s| s.as_str());
            let price_interval = item.get("price_interval").map(|s| s.as_str());
            let price_unit = item.get("price_unit").map(|s| s.as_str());
            let action_text = item.get("action").map(|s| s.as_str()).unwrap_or("View");
            let action_icon = item.get("action_icon").map(|s| s.as_str()).unwrap_or("");

            // Icon box
            let icon_html = if icon.is_empty() {
                r#"<div style="width:48px;height:48px;background:#e8e8e8;border-radius:12px;flex-shrink:0"></div>"#.to_string()
            } else {
                format!(r#"<div style="width:48px;height:48px;background:#e8e8e8;border-radius:12px;flex-shrink:0;display:flex;align-items:center;justify-content:center"><span class="material-symbols-outlined" style="color:#1a1c1c">{icon}</span></div>"#, icon = icon)
            };

            // Status badge
            let status_html = match status {
                Some(s) => {
                    let (bg, tx) = if s.eq_ignore_ascii_case("active") {
                        ("rgba(0,111,240,0.1)", "#006ff0")
                    } else {
                        ("rgba(161,161,170,0.15)", "#71717a")
                    };
                    let label = s;
                    format!(r#"<span style="font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:0.1em;padding:4px 12px;border-radius:999px;background:{bg};color:{tx}">{label}</span>"#, bg = bg, tx = tx, label = label)
                }
                None => String::new(),
            };

            // Price line
            let price_html = match price {
                Some(p) => {
                    let suffix = price_interval.map(|i| format!(r#" <span style="font-size:12px;font-weight:500;color:#5e5e5e">/ {i}</span>"#, i = i))
                        .or_else(|| price_unit.map(|u| format!(r#" <span style="font-size:12px;font-weight:500;color:#5e5e5e">{u}</span>"#, u = u)))
                        .unwrap_or_default();
                    format!(r#"<div style="display:flex;align-items:baseline;gap:4px;margin-bottom:24px"><span style="font-size:24px;font-weight:700;letter-spacing:-0.03em;color:#1a1c1c">{p}</span>{suffix}</div>"#, p = p, suffix = suffix)
                }
                None => String::new(),
            };

            // Action button with icon
            let act_icon_html = if !action_icon.is_empty() {
                format!(r#"<span class="material-symbols-outlined" style="font-size:16px">{}</span>"#, action_icon)
            } else {
                String::new()
            };

            format!(
                r##"<div class="anim-slide-up card-hover" style="background:#fff;border:1px solid rgba(198,198,198,0.2);border-radius:12px;padding:24px;display:flex;flex-direction:column;transition:border-color 0.2s" onmouseover="this.style.borderColor='rgba(0,0,0,0.1)'" onmouseout="this.style.borderColor='rgba(198,198,198,0.2)'">
  <div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:24px">{icon_html}{status_html}</div>
  <h3 style="font-size:18px;font-weight:700;letter-spacing:-0.02em;color:#1a1c1c;margin:0 0 4px">{name}</h3>
  <p style="font-size:14px;color:#5e5e5e;margin:0 0 16px">{desc}</p>
  <div style="margin-top:auto">{price_html}<div style="display:flex;gap:8px">
    <button class="btn-hover" style="flex:1;display:flex;align-items:center;justify-content:center;gap:8px;background:#f3f3f3;border:none;border-radius:999px;padding:10px 0;font-size:12px;font-weight:700;color:#1a1c1c;cursor:pointer;transition:background 0.15s;font-family:inherit" onmouseover="this.style.background='#e8e8e8'" onmouseout="this.style.background='#f3f3f3'">{act_icon} {action_text}</button>
    <button style="width:40px;height:40px;border:1px solid rgba(198,198,198,0.3);border-radius:999px;display:flex;align-items:center;justify-content:center;background:transparent;cursor:pointer;transition:background 0.15s" onmouseover="this.style.background='#f3f3f3'" onmouseout="this.style.background='transparent'"><span class="material-symbols-outlined" style="font-size:20px;color:#71717a">more_horiz</span></button>
  </div></div>
</div>"##,
                icon_html = icon_html, status_html = status_html,
                name = name, desc = desc, price_html = price_html,
                act_icon = act_icon_html, action_text = action_text,
            )
        })
        .collect();

    cards.join("\n")
}

// ── Payment Links: promo banner ────────────────

pub(super) fn build_payment_links_promo(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };

    let title = sec.title.as_deref().unwrap_or("");
    let subtitle = sec.subtitle.as_deref().unwrap_or("");
    let badge_text = sec.config.get("badge").map(|s| s.as_str()).unwrap_or("");
    let cta_text = sec.config.get("cta_text").map(|s| s.as_str()).unwrap_or("");
    let cta_link = sec.config.get("cta_link").map(|s| s.as_str()).unwrap_or("#");

    // Extract image URL from items with _type=image
    let image_url = sec.items.iter()
        .find(|i| i.get("_type").map(|s| s.as_str()) == Some("image"))
        .and_then(|i| i.get("title"))
        .map(|s| s.as_str())
        .unwrap_or("");

    let badge_html = if !badge_text.is_empty() {
        format!(r#"<span style="display:inline-block;font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:0.1em;padding:4px 12px;border-radius:999px;background:rgba(255,255,255,0.2);color:#fff;margin-bottom:16px">{}</span>"#, badge_text)
    } else {
        String::new()
    };

    let subtitle_html = if !subtitle.is_empty() {
        format!(r#"<p style="font-size:14px;color:#a1a1aa;max-width:480px;line-height:1.6;margin:0 0 32px">{}</p>"#, subtitle)
    } else {
        String::new()
    };

    let cta_html = if !cta_text.is_empty() {
        format!(
            r##"<a href="{link}" style="display:inline-flex;align-items:center;padding:10px 24px;border-radius:999px;background:#fff;color:#000;font-weight:700;font-size:14px;text-decoration:none;transition:background 0.15s" onmouseover="this.style.background='#e5e7eb'" onmouseout="this.style.background='#fff'">{text}</a>"##,
            link = cta_link, text = cta_text
        )
    } else {
        String::new()
    };

    // Right side: image or gradient placeholder
    let right_html = if !image_url.is_empty() {
        format!(r#"<div style="flex:1;position:relative;min-height:200px"><img src="{}" style="position:absolute;inset:0;width:100%;height:100%;object-fit:cover;border-radius:8px;opacity:0.6" alt=""></div>"#, image_url)
    } else {
        r#"<div style="flex:1;position:relative;min-height:200px;border-radius:8px;overflow:hidden;background:radial-gradient(ellipse at 80% 50%,rgba(0,111,240,0.2),transparent 70%)"></div>"#.to_string()
    };

    // Span 2 columns in the parent 3-col grid
    format!(
        r##"<div class="anim-scale d3" style="grid-column:span 2;background:#000;color:#fff;border-radius:12px;padding:32px;position:relative;overflow:hidden;display:flex;gap:32px">
  <div style="flex:1;position:relative;z-index:1;display:flex;flex-direction:column">
    {badge}
    <h2 style="font-size:30px;font-weight:700;letter-spacing:-0.03em;line-height:1;color:#fff;margin:0 0 16px">{title}</h2>
    {subtitle}
    {cta}
  </div>
  {right}
</div>"##,
        badge = badge_html, title = title, subtitle = subtitle_html, cta = cta_html, right = right_html,
    )
}

// ── Payment Links: info bar (footer) ───────────

pub(super) fn build_payment_links_info_bar(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };

    let title = sec.title.as_deref().unwrap_or("");
    let subtitle = sec.subtitle.as_deref().unwrap_or("");
    let icon_name = sec.config.get("icon").map(|s| s.as_str()).unwrap_or("shield");

    let links: Vec<String> = sec.items.iter()
        .filter(|i| i.get("_type").map(|s| s.as_str()) != Some("action") || i.get("_type").is_none())
        .map(|item| {
            let name = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let href = item.get("link").or(item.get("href")).map(|s| s.as_str()).unwrap_or("#");
            format!(
                r##"<a href="{href}" style="font-size:12px;font-weight:700;color:#5e5e5e;text-transform:uppercase;letter-spacing:0.1em;text-decoration:none;transition:color 0.15s" onmouseover="this.style.color='#000'" onmouseout="this.style.color='#5e5e5e'">{name}</a>"##,
                href = href, name = name,
            )
        })
        .collect();

    let title_html = if !title.is_empty() {
        format!(r#"<p style="font-size:14px;font-weight:700;letter-spacing:-0.02em;color:#1a1c1c;margin:0">{}</p>"#, title)
    } else {
        String::new()
    };

    let subtitle_html = if !subtitle.is_empty() {
        format!(r#"<p style="font-size:12px;color:#5e5e5e;margin:0">{}</p>"#, subtitle)
    } else {
        String::new()
    };

    format!(
        r##"<footer style="padding:32px;background:#fafafa;border-top:1px solid rgba(198,198,198,0.2)">
  <div style="max-width:1024px;margin:0 auto;display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:24px">
    <div style="display:flex;align-items:center;gap:16px">
      <div style="width:48px;height:48px;background:#fff;border:1px solid rgba(198,198,198,0.2);border-radius:999px;display:flex;align-items:center;justify-content:center">
        <span class="material-symbols-outlined" style="color:#a1a1aa">{icon}</span>
      </div>
      <div>
        {title}
        {subtitle}
      </div>
    </div>
    <div style="display:flex;align-items:center;gap:32px">
      {links}
    </div>
  </div>
</footer>"##,
        icon = icon_name, title = title_html, subtitle = subtitle_html,
        links = links.join("\n      "),
    )
}

// ════════════════════════════════════════════════
// ██  CHECKOUT DASHBOARD  ████████████████████████
// ════════════════════════════════════════════════

pub fn render_checkout_dashboard(
    app_name: &str,
    sections: &[SectionNode],
    _components: &[crate::parser::ComponentNode],
    theme: &str,
) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    let _ = theme;

    let topbar_sec = sections.iter().find(|s| s.section_type == "topbar");
    let checkout_form = sections.iter().find(|s| s.section_type == "checkout-form");
    let product_summary = sections.iter().find(|s| s.section_type == "product-summary");
    let trust_indicators = sections.iter().find(|s| s.section_type == "trust-indicators");
    let testimonial = sections.iter().find(|s| s.section_type == "testimonial");
    let footer_sec = sections.iter().find(|s| s.section_type == "footer");

    let topbar_html = build_checkout_topbar(topbar_sec);
    let form_html = build_checkout_form(checkout_form);
    let summary_html = build_checkout_product_summary(product_summary);
    let trust_html = build_checkout_trust_indicators(trust_indicators);
    let testimonial_html = build_checkout_testimonial(testimonial);
    let footer_html = build_checkout_footer(footer_sec);

    format!(
        r##"<!DOCTYPE html>
<html class="light" lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta name="view-transition" content="same-origin">
  <title>{app_name}</title>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap" rel="stylesheet">
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet">
  <style>
    body {{ font-family:'Inter',sans-serif; background:#f9f9f9; color:#1a1c1c; margin:0; min-height:100vh; }}
    * {{ box-sizing:border-box; }}
    .material-symbols-outlined {{ font-variation-settings:'FILL' 0,'wght' 400,'GRAD' 0,'opsz' 24; font-size:20px; display:inline-block; line-height:1; vertical-align:middle; }}
    ::selection {{ background:rgba(0,111,240,0.15); }}
    .ghost-border {{ border:1px solid rgba(198,198,198,0.2); }}
    .prism-glow {{ background:radial-gradient(circle at top right,rgba(0,111,240,0.08),transparent 40%),radial-gradient(circle at bottom left,rgba(216,226,255,0.1),transparent 40%); }}
  </style>
  <style>{anim_css}</style>
</head>
<body class="prism-glow">

{topbar}

<main style="max-width:1152px;margin:0 auto;padding:48px 24px 80px">
  <div style="display:grid;grid-template-columns:repeat(12,1fr);gap:96px">
    <div style="grid-column:span 7">
      {form}
    </div>
    <div style="grid-column:span 5">
      <div style="position:sticky;top:96px;display:flex;flex-direction:column;gap:32px">
        {summary}
        {trust}
        {testimonial}
      </div>
    </div>
  </div>
</main>

{footer}

<script{script_nonce}>{runtime}</script>
<script{script_nonce}>{hmr}</script>
{anim_js}
</body>
</html>"##,
        app_name = app_name,
        topbar = topbar_html,
        form = form_html,
        summary = summary_html,
        trust = trust_html,
        testimonial = testimonial_html,
        footer = footer_html,
        anim_css = CRONUS_ANIMATIONS_CSS,
        anim_js = crate::security::mark_kernel_scripts(CRONUS_ANIMATIONS_JS),
        runtime = crate::render::CRONUS_RUNTIME_JS,
        hmr = crate::hmr::HMR_CLIENT_JS,
    )
}

pub(super) fn build_checkout_topbar(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let brand = sec.config.get("brand").map(|s| s.as_str()).unwrap_or("");
    let action_text = sec.items.iter()
        .find(|i| i.get("_type").map(|s| s.as_str()) == Some("action"))
        .and_then(|i| i.get("title"))
        .map(|s| s.as_str())
        .unwrap_or("");

    format!(
        r##"<header style="width:100%;border-bottom:1px solid #e5e7eb;position:sticky;top:0;z-index:50;background:rgba(255,255,255,0.8);backdrop-filter:blur(20px);display:flex;justify-content:space-between;align-items:center;height:64px;padding:0 48px">
  <div style="font-size:18px;font-weight:700;letter-spacing:-0.05em;color:#000;display:flex;align-items:center;gap:8px">
    <span style="width:24px;height:24px;background:#000;border-radius:4px;display:flex;align-items:center;justify-content:center;color:#fff;font-size:10px">GP</span>
    {brand}
  </div>
  <button style="display:flex;align-items:center;gap:8px;color:#71717a;font-size:14px;font-weight:500;background:none;border:none;cursor:pointer">
    <span class="material-symbols-outlined" style="font-size:14px">close</span>
    {action}
  </button>
</header>"##,
        brand = brand,
        action = action_text,
    )
}

pub(super) fn build_checkout_form(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("");
    let cta_text = sec.config.get("cta_text").map(|s| s.as_str()).unwrap_or("");
    let footnote = sec.config.get("footnote").map(|s| s.as_str()).unwrap_or("");

    let mut express_buttons: Vec<String> = Vec::new();
    let mut divider_text = String::new();
    let mut fields: Vec<(String, String, String)> = Vec::new();
    let mut checkbox_text = String::new();

    for item in &sec.items {
        let item_type = item.get("_type").map(|s| s.as_str()).unwrap_or("");
        if item_type == "action" { continue; }
        let style = item.get("style").map(|s| s.as_str()).unwrap_or("");
        let item_title = item.get("title").map(|s| s.as_str()).unwrap_or("");

        if style == "express" {
            express_buttons.push(item_title.to_string());
        } else if style == "divider" {
            divider_text = item_title.to_string();
        } else if style == "checkbox" {
            checkbox_text = item_title.to_string();
        } else if style == "field" {
            let meta = item.get("meta").map(|s| s.as_str()).unwrap_or("");
            let action = item.get("action").map(|s| s.as_str()).unwrap_or("");
            fields.push((item_title.to_string(), meta.to_string(), action.to_string()));
        }
    }

    let mut express_html = String::new();
    if !express_buttons.is_empty() {
        express_html.push_str(r#"<div style="display:grid;grid-template-columns:1fr 1fr;gap:16px;margin-bottom:40px">"#);
        for (i, name) in express_buttons.iter().enumerate() {
            if i == 0 {
                express_html.push_str(&format!(
                    r#"<button style="background:#000;color:#fff;height:48px;border-radius:999px;display:flex;align-items:center;justify-content:center;gap:8px;border:none;cursor:pointer;font-size:14px"><span style="font-weight:500">Pay with</span> <span style="font-weight:700">{name}</span></button>"#,
                    name = name
                ));
            } else {
                express_html.push_str(&format!(
                    r#"<button style="background:#fff;color:#000;height:48px;border-radius:999px;border:1px solid #e5e7eb;display:flex;align-items:center;justify-content:center;gap:8px;cursor:pointer;font-size:14px"><span style="font-weight:500">Pay with</span> <span style="font-weight:700">{name}</span></button>"#,
                    name = name
                ));
            }
        }
        express_html.push_str("</div>");
    }

    let divider_html = if !divider_text.is_empty() {
        format!(
            r#"<div style="display:flex;align-items:center;padding:16px 0;margin-bottom:24px">
              <div style="flex:1;border-top:1px solid #e5e7eb"></div>
              <span style="margin:0 16px;color:#a1a1aa;font-size:12px;font-weight:500;text-transform:uppercase;letter-spacing:0.1em">{text}</span>
              <div style="flex:1;border-top:1px solid #e5e7eb"></div>
            </div>"#,
            text = divider_text
        )
    } else {
        String::new()
    };

    let mut fields_html = String::new();
    for (label, placeholder, extra) in &fields {
        if label == "Card information" {
            fields_html.push_str(&format!(
                r#"<div style="margin-bottom:24px">
                  <label style="display:block;font-size:12px;font-weight:600;letter-spacing:0.05em;text-transform:uppercase;color:#71717a;margin-bottom:8px">{label}</label>
                  <div style="position:relative">
                    <input type="text" placeholder="{placeholder}" style="width:100%;height:48px;padding:0 16px;border-radius:8px 8px 0 0;background:#fff;border:1px solid rgba(198,198,198,0.2);outline:none;font-size:14px;color:#1a1c1c">
                    <span class="material-symbols-outlined" style="position:absolute;right:16px;top:50%;transform:translateY(-50%);color:#d4d4d8">credit_card</span>
                  </div>
                  <div style="display:grid;grid-template-columns:1fr 1fr">"#,
                label = label, placeholder = placeholder,
            ));
            let parts: Vec<&str> = extra.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            if parts.len() >= 2 {
                fields_html.push_str(&format!(
                    r#"<input type="text" placeholder="{}" style="width:100%;height:48px;padding:0 16px;border-radius:0 0 0 8px;background:#fff;border:1px solid rgba(198,198,198,0.2);border-top:none;outline:none;font-size:14px">
                    <input type="text" placeholder="{}" style="width:100%;height:48px;padding:0 16px;border-radius:0 0 8px 0;background:#fff;border:1px solid rgba(198,198,198,0.2);border-top:none;border-left:none;outline:none;font-size:14px">"#,
                    parts[0], parts[1]
                ));
            }
            fields_html.push_str("</div></div>");
        } else if label == "Billing address" {
            fields_html.push_str(&format!(
                r#"<div style="margin-bottom:24px">
                  <label style="display:block;font-size:12px;font-weight:600;letter-spacing:0.05em;text-transform:uppercase;color:#71717a;margin-bottom:8px">{label}</label>
                  <select style="width:100%;height:48px;padding:0 16px;border-radius:8px 8px 0 0;background:#fff;border:1px solid rgba(198,198,198,0.2);outline:none;font-size:14px;color:#1a1c1c;appearance:none">
                    <option>{placeholder}</option>
                  </select>
                  <input type="text" placeholder="{extra}" style="width:100%;height:48px;padding:0 16px;border-radius:0 0 8px 8px;background:#fff;border:1px solid rgba(198,198,198,0.2);border-top:none;outline:none;font-size:14px">
                </div>"#,
                label = label, placeholder = placeholder, extra = extra,
            ));
        } else {
            fields_html.push_str(&format!(
                r#"<div style="margin-bottom:24px">
                  <label style="display:block;font-size:12px;font-weight:600;letter-spacing:0.05em;text-transform:uppercase;color:#71717a;margin-bottom:8px">{label}</label>
                  <input type="text" placeholder="{placeholder}" style="width:100%;height:48px;padding:0 16px;border-radius:8px;background:#fff;border:1px solid rgba(198,198,198,0.2);outline:none;font-size:14px;color:#1a1c1c">
                </div>"#,
                label = label, placeholder = placeholder,
            ));
        }
    }

    let checkbox_html = if !checkbox_text.is_empty() {
        format!(
            r#"<div style="display:flex;align-items:center;gap:12px;padding-top:8px;margin-bottom:32px">
              <input type="checkbox" style="width:16px;height:16px;border-radius:4px;border:1px solid #d4d4d8;accent-color:#000">
              <label style="font-size:14px;color:#52525b">{text}</label>
            </div>"#,
            text = checkbox_text
        )
    } else {
        String::new()
    };

    let cta_html = if !cta_text.is_empty() {
        format!(
            r#"<button style="width:100%;background:#000;color:#fff;height:56px;border-radius:999px;font-weight:700;letter-spacing:-0.02em;font-size:18px;border:none;cursor:pointer;display:flex;align-items:center;justify-content:center;gap:8px;margin-top:32px">
              {cta}
              <span class="material-symbols-outlined" style="color:rgba(255,255,255,0.5);font-variation-settings:'FILL' 1">lock</span>
            </button>"#,
            cta = cta_text
        )
    } else {
        String::new()
    };

    let footnote_html = if !footnote.is_empty() {
        format!(
            r#"<p style="text-align:center;font-size:12px;color:#a1a1aa;margin-top:16px;padding:0 32px;line-height:1.6">{text}</p>"#,
            text = footnote
        )
    } else {
        String::new()
    };

    format!(
        r##"<section>
          <h1 style="font-size:30px;font-weight:700;letter-spacing:-0.02em;margin:0 0 32px">{title}</h1>
          {express}
          {divider}
          <form>
            {fields}
            {checkbox}
            {cta}
            {footnote}
          </form>
        </section>"##,
        title = title,
        express = express_html,
        divider = divider_html,
        fields = fields_html,
        checkbox = checkbox_html,
        cta = cta_html,
        footnote = footnote_html,
    )
}

pub(super) fn build_checkout_product_summary(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };

    let mut product_html = String::new();
    let mut rows_html = String::new();
    let mut total_label = String::new();
    let mut total_value = String::new();

    for item in &sec.items {
        let item_type = item.get("_type").map(|s| s.as_str()).unwrap_or("");
        if item_type == "action" { continue; }

        if item_type == "row" {
            let label = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let value = item.get("value").map(|s| s.as_str()).unwrap_or("");
            if label == "Total" {
                total_label = label.to_string();
                total_value = value.to_string();
            } else {
                rows_html.push_str(&format!(
                    r#"<div style="display:flex;justify-content:space-between;font-size:14px">
                      <span style="color:#71717a">{label}</span>
                      <span style="font-weight:500">{value}</span>
                    </div>"#,
                    label = label, value = value,
                ));
            }
        } else {
            let name = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let desc = item.get("description").map(|s| s.as_str()).unwrap_or("");
            let qty = item.get("meta").map(|s| s.as_str()).unwrap_or("");
            let img = item.get("icon").map(|s| s.as_str()).unwrap_or("");

            let img_html = if !img.is_empty() && img.starts_with("http") {
                format!(
                    r#"<div style="width:96px;height:96px;border-radius:8px;overflow:hidden;flex-shrink:0;border:1px solid rgba(198,198,198,0.2)"><img src="{img}" alt="{name}" style="width:100%;height:100%;object-fit:cover"></div>"#,
                    img = img, name = name
                )
            } else {
                r#"<div style="width:96px;height:96px;border-radius:8px;background:#f4f4f5;flex-shrink:0;border:1px solid rgba(198,198,198,0.2)"></div>"#.to_string()
            };

            product_html.push_str(&format!(
                r#"<div style="display:flex;gap:24px">
                  {img}
                  <div style="display:flex;flex-direction:column;justify-content:center">
                    <h3 style="font-size:18px;font-weight:700;letter-spacing:-0.02em;margin:0">{name}</h3>
                    <p style="font-size:14px;color:#71717a;margin:4px 0 0">{desc}</p>
                    <p style="font-size:14px;font-weight:500;margin:8px 0 0">{qty}</p>
                  </div>
                </div>"#,
                img = img_html, name = name, desc = desc, qty = qty,
            ));
        }
    }

    let total_html = if !total_label.is_empty() {
        format!(
            r#"<div style="display:flex;justify-content:space-between;font-size:20px;font-weight:700;letter-spacing:-0.02em;padding-top:16px;border-top:1px solid #f4f4f5">
              <span>{label}</span>
              <span>{value}</span>
            </div>"#,
            label = total_label, value = total_value,
        )
    } else {
        String::new()
    };

    format!(
        r##"<div class="ghost-border" style="background:#fff;border-radius:12px;padding:32px;display:flex;flex-direction:column;gap:32px">
          {product}
          <div style="display:flex;flex-direction:column;gap:16px;padding-top:16px;border-top:1px solid #f4f4f5">
            {rows}
            {total}
          </div>
        </div>"##,
        product = product_html, rows = rows_html, total = total_html,
    )
}

pub(super) fn build_checkout_trust_indicators(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };

    let mut items_html = String::new();
    for item in &sec.items {
        let item_type = item.get("_type").map(|s| s.as_str()).unwrap_or("");
        if item_type == "action" { continue; }
        let label = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("");

        items_html.push_str(&format!(
            r#"<div style="padding:16px;border-radius:8px;background:#f4f4f5;display:flex;flex-direction:column;align-items:center;text-align:center;gap:8px">
              <span class="material-symbols-outlined" style="color:#a1a1aa">{icon}</span>
              <span style="font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:0.1em;color:#71717a">{label}</span>
            </div>"#,
            icon = icon, label = label,
        ));
    }

    format!(
        r#"<div style="display:grid;grid-template-columns:1fr 1fr;gap:16px">{items}</div>"#,
        items = items_html,
    )
}

pub(super) fn build_checkout_testimonial(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let source = sec.title.as_deref().unwrap_or("");
    let quote = sec.subtitle.as_deref().unwrap_or("");

    format!(
        r##"<div style="position:relative;padding:24px;background:#000;color:#fff;border-radius:12px;overflow:hidden">
          <div style="position:relative;z-index:1">
            <p style="font-size:14px;font-weight:500;font-style:italic;line-height:1.6;opacity:0.9">"{quote}"</p>
            <p style="font-size:12px;font-weight:700;margin-top:16px;letter-spacing:0.05em;text-transform:uppercase">{source}</p>
          </div>
          <div style="position:absolute;inset:0;background:linear-gradient(to top right,rgba(0,111,240,0.2),transparent);opacity:0.5"></div>
        </div>"##,
        quote = quote, source = source,
    )
}

pub(super) fn build_checkout_footer(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let copyright = sec.config.get("copyright").map(|s| s.as_str()).unwrap_or("");
    let nav_items = sec.config.get("nav").map(|s| s.as_str()).unwrap_or("");

    let mut nav_html = String::new();
    for link in nav_items.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
        nav_html.push_str(&format!(
            r##"<a href="#" style="font-size:12px;font-weight:500;color:#71717a;text-decoration:none">{link}</a>"##,
            link = link
        ));
    }

    format!(
        r##"<footer style="margin-top:80px;padding:48px 0;border-top:1px solid #e5e7eb">
  <div style="max-width:1152px;margin:0 auto;padding:0 24px;display:flex;justify-content:space-between;align-items:center">
    <div style="display:flex;align-items:center;gap:24px">
      <span style="font-size:12px;font-weight:700;text-transform:uppercase;letter-spacing:0.1em;color:#a1a1aa">{copyright}</span>
      <div style="display:flex;gap:16px;opacity:0.3">
        <span class="material-symbols-outlined">payments</span>
        <span class="material-symbols-outlined">account_balance</span>
        <span class="material-symbols-outlined">shield</span>
      </div>
    </div>
    <div style="display:flex;gap:32px">{nav}</div>
  </div>
</footer>"##,
        copyright = copyright, nav = nav_html,
    )
}

// ════════════════════════════════════════════════
// ██  SECURITY TEAM DASHBOARD  ███████████████████
// ════════════════════════════════════════════════

pub fn render_security_dashboard(
    app_name: &str,
    sections: &[SectionNode],
    components: &[crate::parser::ComponentNode],
    theme: &str,
    current_route: &str,
) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    let _ = theme;

    let sidebar_comp = components.iter().find(|c| c.layout.as_deref() == Some("sidebar"));
    let topbar_comp = components.iter().find(|c| {
        c.layout.as_deref() == Some("inline") && c.style.as_deref().map(|s| s.contains("topbar")).unwrap_or(false)
    });
    let sidebar_section = sections.iter().find(|s| s.section_type == "sidebar");

    let page_header = sections.iter().find(|s| s.section_type == "page-header");
    let team_members = sections.iter().find(|s| s.section_type == "team-members");
    let security_status = sections.iter().find(|s| s.section_type == "security-status");
    let security_policies = sections.iter().find(|s| s.section_type == "security-policies");
    let login_activity = sections.iter().find(|s| s.section_type == "login-activity");

    let topbar_html = build_dashboard_topbar(topbar_comp);
    let sidebar_html = build_dashboard_sidebar(sidebar_comp, sidebar_section, current_route);

    let header_html = build_security_page_header(page_header);
    let team_html = build_security_team_members(team_members);
    let status_html = build_security_2fa_status(security_status);
    let policies_html = build_security_policies(security_policies);
    let activity_html = build_security_login_activity(login_activity);

    format!(
        r##"<!DOCTYPE html>
<html class="light" lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta name="view-transition" content="same-origin">
  <title>{app_name}</title>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap" rel="stylesheet">
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet">
  <style>
    body {{ font-family:'Inter',sans-serif; background:#f9f9f9; color:#1a1c1c; margin:0; }}
    * {{ box-sizing:border-box; }}
    .material-symbols-outlined {{ font-variation-settings:'FILL' 0,'wght' 400,'GRAD' 0,'opsz' 24; font-size:20px; display:inline-block; line-height:1; text-transform:none; letter-spacing:normal; word-wrap:normal; white-space:nowrap; direction:ltr; vertical-align:middle; }}
    ::selection {{ background:rgba(0,111,240,0.15); }}
    ::-webkit-scrollbar {{ width:4px; }}
    ::-webkit-scrollbar-thumb {{ background:rgba(0,0,0,0.1); border-radius:2px; }}
    .ghost-border {{ border:1px solid rgba(198,198,198,0.2); }}
    .prism-bg {{ background:radial-gradient(circle at top right,rgba(0,111,240,0.08),transparent 40%),radial-gradient(circle at bottom left,rgba(0,111,240,0.05),transparent 40%); }}
  </style>
  <style>{anim_css}</style>
</head>
<body class="prism-bg">

{topbar}

<div style="display:flex">

{sidebar}

<main style="flex:1;margin-left:256px;padding:32px">
  <div style="max-width:1152px;margin:0 auto">

    {header}

    <div style="display:grid;grid-template-columns:repeat(3,1fr);gap:24px">
      <div style="grid-column:span 2">
        {team}
      </div>
      <div style="display:flex;flex-direction:column;gap:24px">
        {status}
        {policies}
      </div>
      <div style="grid-column:span 3">
        {activity}
      </div>
    </div>
  </div>
</main>

</div>

<script{script_nonce}>{runtime}</script>
<script{script_nonce}>{hmr}</script>
{anim_js}
</body>
</html>"##,
        app_name = app_name,
        topbar = topbar_html,
        sidebar = sidebar_html,
        header = header_html,
        team = team_html,
        status = status_html,
        policies = policies_html,
        activity = activity_html,
        anim_css = CRONUS_ANIMATIONS_CSS,
        anim_js = crate::security::mark_kernel_scripts(CRONUS_ANIMATIONS_JS),
        runtime = crate::render::CRONUS_RUNTIME_JS,
        hmr = crate::hmr::HMR_CLIENT_JS,
    )
}

pub(super) fn build_security_topbar(comp: Option<&ComponentNode>) -> String {
    let search_placeholder = comp.and_then(|c| {
        c.items.iter().find(|i| {
            i.config.get("icon").map(|s| s == "search").unwrap_or(false)
        }).map(|i| i.text.as_str())
    }).unwrap_or("Search team or logs...");

    let avatar_url = comp.and_then(|c| c.props.get("avatar")).map(|s| s.as_str()).unwrap_or(
        "https://lh3.googleusercontent.com/aida-public/AB6AXuBIbgWddrG9yJJh3szTkHfHc24QiMUNrdmbevowt4H4pw-Sn3AdqIA63n5rQf4TwrFkjVxOgIwZ9vnMKqXg6AkvOnIGMOLL2PfMXo5wJHLsI4tCpLMq8c3bpiAa5zTM1vCgkbtU_MH41WSmUxmB4-P2AAWg1R5gZ--tClrCs3yPUQwVWTUJfJxRrXas6pXdrZxwZY9_Y-qfXZlFsSTTaLEEBoH7WiKE-vfIy6vzFuvW-pIZhe-aAUX_y4_uFxKULJPG18x1leIvlmd8"
    );

    format!(
        r##"<header style="width:100%;border-bottom:1px solid #e5e7eb;position:sticky;top:0;z-index:50;background:rgba(255,255,255,0.8);backdrop-filter:blur(20px);-webkit-backdrop-filter:blur(20px);display:flex;justify-content:space-between;align-items:center;height:64px;padding:0 24px;font-family:'Inter',sans-serif;-webkit-font-smoothing:antialiased;letter-spacing:-0.02em">
  <div style="display:flex;align-items:center;gap:32px">
    <span style="font-size:18px;font-weight:700;letter-spacing:-0.05em;color:#000">GeistPay</span>
    <div style="position:relative">
      <span class="material-symbols-outlined" style="position:absolute;left:12px;top:50%;transform:translateY(-50%);color:#a1a1aa;font-size:14px">search</span>
      <input type="text" placeholder="{placeholder}" style="background:#f3f3f3;border:none;border-radius:999px;padding:6px 16px 6px 40px;font-size:14px;width:256px;outline:none;font-family:'Inter',sans-serif">
    </div>
  </div>
  <div style="display:flex;align-items:center;gap:16px">
    <button style="padding:8px;color:#71717a;background:none;border:none;cursor:pointer"><span class="material-symbols-outlined">notifications</span></button>
    <button style="padding:8px;color:#71717a;background:none;border:none;cursor:pointer"><span class="material-symbols-outlined">help</span></button>
    <div style="width:32px;height:32px;border-radius:50%;overflow:hidden;background:#e5e7eb">
      <img src="{avatar}" style="width:100%;height:100%;object-fit:cover" alt="User profile">
    </div>
  </div>
</header>"##,
        placeholder = search_placeholder,
        avatar = avatar_url,
    )
}

pub(super) fn build_security_sidebar(comp: Option<&ComponentNode>, section: Option<&SectionNode>) -> String {
    let mut brand = "GeistPay";
    let mut subtitle = "";
    let mut nav_items_html = String::new();
    let mut bottom_items_html = String::new();

    if let Some(c) = comp {
        if let Some(b) = c.props.get("brand") {
            brand = b.as_str();
        } else if let Some(b) = item_by_kind(&c.items, "brand") {
            brand = b;
        }
        if let Some(s) = c.props.get("subtitle") {
            subtitle = s.as_str();
        } else if let Some(s) = item_by_kind(&c.items, "subtitle") {
            subtitle = s;
        }
        let items = items_by_kind(&c.items, "item");
        let bottom_types = ["contact_support", "menu_book", "support", "docs"];
        for item in &items {
            let title = item.text.as_str();
            let icon = item.config.get("icon").map(|s| s.as_str()).unwrap_or("");
            let href = item.link.as_deref().unwrap_or("#");
            let is_active = item.config.get("active").map(|s| s == "true").unwrap_or(false);
            let is_bottom = bottom_types.contains(&icon) || title.eq_ignore_ascii_case("support") || title.eq_ignore_ascii_case("docs");

            let link_html = if is_active {
                format!(
                    r#"<a href="{href}" style="display:flex;align-items:center;gap:12px;padding:8px 12px;background:#f4f4f5;color:#000;border-radius:6px;font-size:14px;font-weight:500;text-decoration:none">
  <span class="material-symbols-outlined" style="font-size:20px">{icon}</span> {title}
</a>"#,
                    href = href, icon = icon, title = title
                )
            } else {
                format!(
                    r#"<a href="{href}" style="display:flex;align-items:center;gap:12px;padding:8px 12px;color:#71717a;border-radius:6px;font-size:14px;font-weight:500;text-decoration:none;transition:all 0.2s" onmouseover="this.style.color='#000';this.style.background='#f4f4f5'" onmouseout="this.style.color='#71717a';this.style.background='transparent'">
  <span class="material-symbols-outlined" style="font-size:20px">{icon}</span> {title}
</a>"#,
                    href = href, icon = icon, title = title
                )
            };

            if is_bottom {
                bottom_items_html.push_str(&link_html);
                bottom_items_html.push('\n');
            } else {
                nav_items_html.push_str(&link_html);
                nav_items_html.push('\n');
            }
        }
    } else if let Some(sec) = section {
        brand = sec.config.get("brand").map(|s| s.as_str())
            .or(sec.title.as_deref())
            .unwrap_or("GeistPay");
        subtitle = sec.config.get("subtitle").map(|s| s.as_str())
            .or(sec.subtitle.as_deref())
            .unwrap_or("Enterprise");
    }

    format!(
        r##"<aside style="height:100vh;width:256px;border-right:1px solid #e5e7eb;position:fixed;left:0;top:64px;padding:16px;display:flex;flex-direction:column;gap:8px;background:rgba(250,250,250,0.5);font-size:14px;font-weight:500;letter-spacing:-0.02em;font-family:'Inter',sans-serif">
  <div style="margin-bottom:24px;padding:0 8px">
    <p style="font-weight:700;letter-spacing:-0.05em;color:#000;font-size:16px;margin:0">{brand}</p>
    <p style="font-size:12px;color:#71717a;font-weight:400;margin:0">{subtitle}</p>
  </div>
  <nav style="flex:1;display:flex;flex-direction:column;gap:4px">
    {nav_items}
  </nav>
  <div style="margin-top:auto;border-top:1px solid #e5e7eb;padding-top:16px;display:flex;flex-direction:column;gap:4px">
    {bottom_items}
    <button style="margin-top:16px;width:100%;background:#000;color:#fff;padding:8px;border-radius:999px;font-size:12px;font-weight:700;letter-spacing:-0.02em;border:none;cursor:pointer">Create Payment</button>
  </div>
</aside>"##,
        brand = brand,
        subtitle = subtitle,
        nav_items = nav_items_html,
        bottom_items = bottom_items_html,
    )
}

pub(super) fn build_security_page_header(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("");
    let subtitle = sec.subtitle.as_deref().unwrap_or("");
    let action_text = sec.items.iter()
        .find(|i| i.get("_type").map(|s| s.as_str()) == Some("action"))
        .and_then(|i| i.get("title"))
        .map(|s| s.as_str())
        .unwrap_or("");
    let action_icon = sec.items.iter()
        .find(|i| i.get("_type").map(|s| s.as_str()) == Some("action"))
        .and_then(|i| i.get("icon"))
        .map(|s| s.as_str())
        .unwrap_or("");

    let subtitle_html = if !subtitle.is_empty() {
        format!(r#"<p style="color:#71717a;font-size:14px;margin:0">{}</p>"#, subtitle)
    } else {
        String::new()
    };

    let action_html = if !action_text.is_empty() {
        format!(
            r#"<button onclick="cronusOpenCreate('TeamMember')" style="background:#000;color:#fff;padding:10px 24px;border-radius:999px;font-size:14px;font-weight:700;border:none;cursor:pointer;display:flex;align-items:center;gap:8px">
              <span class="material-symbols-outlined" style="font-size:14px">{icon}</span> {text}
            </button>"#,
            icon = action_icon, text = action_text
        )
    } else {
        String::new()
    };

    format!(
        r#"<div style="display:flex;justify-content:space-between;align-items:flex-end;margin-bottom:48px">
          <div>
            <h1 style="font-size:36px;font-weight:800;letter-spacing:-0.05em;color:#000;margin:0 0 8px">{title}</h1>
            {subtitle}
          </div>
          {action}
        </div>"#,
        title = title, subtitle = subtitle_html, action = action_html,
    )
}

pub(super) fn build_security_team_members(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("");
    let badge = sec.config.get("badge").map(|s| s.as_str()).unwrap_or("");

    let mut members_html = String::new();
    let mut footer_action = String::new();

    for item in &sec.items {
        let item_type = item.get("_type").map(|s| s.as_str()).unwrap_or("");
        if item_type == "action" {
            footer_action = item.get("title").map(|s| s.as_str()).unwrap_or("").to_string();
            continue;
        }

        let name = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let email = item.get("description").map(|s| s.as_str()).unwrap_or("");
        let role = item.get("meta").map(|s| s.as_str()).unwrap_or("");
        let avatar = item.get("icon").map(|s| s.as_str()).unwrap_or("");
        let status = item.get("status").map(|s| s.as_str()).unwrap_or("");

        let avatar_html = if !avatar.is_empty() && avatar.starts_with("http") {
            format!(
                r#"<div style="width:40px;height:40px;border-radius:50%;overflow:hidden;border:1px solid #e5e7eb;flex-shrink:0"><img src="{}" alt="{}" style="width:100%;height:100%;object-fit:cover"></div>"#,
                avatar, name
            )
        } else {
            format!(
                r#"<div style="width:40px;height:40px;border-radius:50%;background:#f4f4f5;border:1px solid #e5e7eb;display:flex;align-items:center;justify-content:center;flex-shrink:0"><span style="font-size:14px;font-weight:600;color:#71717a">{}</span></div>"#,
                name.chars().next().unwrap_or(' ')
            )
        };

        let role_style = if status == "admin" {
            "font-size:12px;font-weight:500;color:#000;background:#f4f4f5;padding:4px 12px;border-radius:999px;border:1px solid #e5e7eb"
        } else {
            "font-size:12px;font-weight:500;color:#71717a;padding:4px 12px;border-radius:999px"
        };

        members_html.push_str(&format!(
            r#"<div style="display:flex;align-items:center;justify-content:space-between">
              <div style="display:flex;align-items:center;gap:16px">
                {avatar}
                <div>
                  <p style="font-size:14px;font-weight:700;letter-spacing:-0.02em;margin:0">{name}</p>
                  <p style="font-size:12px;color:#a1a1aa;margin:0">{email}</p>
                </div>
              </div>
              <div style="display:flex;align-items:center;gap:24px">
                <span style="{role_style}">{role}</span>
                <button style="color:#a1a1aa;background:none;border:none;cursor:pointer"><span class="material-symbols-outlined" style="font-size:20px">more_vert</span></button>
              </div>
            </div>"#,
            avatar = avatar_html, name = name, email = email,
            role_style = role_style, role = role,
        ));
    }

    let badge_html = if !badge.is_empty() {
        format!(r#"<span style="background:#e8e8e8;font-size:12px;padding:4px 10px;border-radius:999px;font-weight:500">{}</span>"#, badge)
    } else {
        String::new()
    };

    let footer_html = if !footer_action.is_empty() {
        format!(
            r#"<div style="margin-top:40px;padding-top:24px;border-top:1px solid #f4f4f5">
              <button style="font-size:12px;font-weight:700;color:#a1a1aa;background:none;border:none;cursor:pointer;display:flex;align-items:center;gap:4px">{action} <span class="material-symbols-outlined" style="font-size:12px">arrow_forward</span></button>
            </div>"#,
            action = footer_action
        )
    } else {
        String::new()
    };

    format!(
        r##"<section class="ghost-border" style="background:#fff;border-radius:12px;padding:24px;box-shadow:0 1px 3px rgba(0,0,0,0.04)">
          <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:32px">
            <h2 style="font-size:20px;font-weight:700;letter-spacing:-0.02em;margin:0">{title}</h2>
            {badge}
          </div>
          <div data-live="teammembers" style="display:flex;flex-direction:column;gap:24px">{members}</div>
          {footer}
        </section>"##,
        title = title, badge = badge_html, members = members_html, footer = footer_html,
    )
}

pub(super) fn build_security_2fa_status(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("");
    let subtitle = sec.subtitle.as_deref().unwrap_or("");
    let icon = sec.config.get("icon").map(|s| s.as_str()).unwrap_or("");

    let compliance_item = sec.items.iter()
        .find(|i| i.get("_type").map(|s| s.as_str()) != Some("action"));
    let compliance_label = compliance_item.and_then(|i| i.get("title")).map(|s| s.as_str()).unwrap_or("");
    let compliance_value = compliance_item.and_then(|i| i.get("value")).map(|s| s.as_str()).unwrap_or("");

    format!(
        r##"<section class="ghost-border" style="background:#fff;border-radius:12px;padding:24px;box-shadow:0 1px 3px rgba(0,0,0,0.04)">
          <div style="display:flex;align-items:center;gap:12px;margin-bottom:16px">
            <div style="width:40px;height:40px;background:#000;color:#fff;border-radius:50%;display:flex;align-items:center;justify-content:center">
              <span class="material-symbols-outlined" style="font-size:18px;font-variation-settings:'FILL' 1">{icon}</span>
            </div>
            <h2 style="font-size:16px;font-weight:700;letter-spacing:-0.02em;margin:0">{title}</h2>
          </div>
          <p style="font-size:12px;color:#71717a;line-height:1.6;margin:0 0 24px">{subtitle}</p>
          <div style="display:flex;align-items:center;justify-content:space-between;background:#fafafa;padding:12px;border-radius:8px;border:1px solid #f4f4f5">
            <span style="font-size:12px;font-weight:700;text-transform:uppercase;letter-spacing:0.05em">{label}</span>
            <span style="font-size:12px;font-weight:700;color:#059669;display:flex;align-items:center;gap:4px">
              <span class="material-symbols-outlined" style="font-size:12px">check_circle</span> {value}
            </span>
          </div>
        </section>"##,
        icon = icon, title = title, subtitle = subtitle,
        label = compliance_label, value = compliance_value,
    )
}

pub(super) fn build_security_policies(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("");

    let mut policies_html = String::new();
    for item in &sec.items {
        let item_type = item.get("_type").map(|s| s.as_str()).unwrap_or("");
        if item_type != "policy" { continue; }

        let label = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let toggle = item.get("toggle").map(|s| s.as_str()).unwrap_or("");
        let value = item.get("value").map(|s| s.as_str()).unwrap_or("");

        let control_html = if !toggle.is_empty() {
            let is_on = toggle == "on";
            let bg = if is_on { "#000" } else { "#d4d4d8" };
            let pos = if is_on { "left:18px" } else { "left:4px" };
            format!(
                r#"<div style="width:32px;height:16px;background:{bg};border-radius:999px;position:relative;cursor:pointer"><div style="position:absolute;{pos};top:4px;width:8px;height:8px;background:#fff;border-radius:50%"></div></div>"#,
                bg = bg, pos = pos
            )
        } else if !value.is_empty() {
            format!(
                r#"<span style="font-size:10px;font-weight:700;background:#f4f4f5;padding:2px 8px;border-radius:4px;text-transform:uppercase">{}</span>"#,
                value
            )
        } else {
            String::new()
        };

        policies_html.push_str(&format!(
            r#"<li style="display:flex;align-items:center;justify-content:space-between">
              <span style="font-size:12px;color:#52525b">{label}</span>
              {control}
            </li>"#,
            label = label, control = control_html,
        ));
    }

    format!(
        r##"<section class="ghost-border" style="background:#fff;border-radius:12px;padding:24px;box-shadow:0 1px 3px rgba(0,0,0,0.04)">
          <h2 style="font-size:16px;font-weight:700;letter-spacing:-0.02em;margin:0 0 16px">{title}</h2>
          <ul style="list-style:none;padding:0;margin:0;display:flex;flex-direction:column;gap:12px">{policies}</ul>
        </section>"##,
        title = title, policies = policies_html,
    )
}

pub(super) fn build_security_login_activity(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("");
    let columns_str = sec.config.get("columns").map(|s| s.as_str()).unwrap_or("");
    let columns: Vec<&str> = columns_str.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();

    let action_text = sec.items.iter()
        .find(|i| i.get("_type").map(|s| s.as_str()) == Some("action"))
        .and_then(|i| i.get("title"))
        .map(|s| s.as_str())
        .unwrap_or("");

    let mut thead_html = String::new();
    for col in &columns {
        thead_html.push_str(&format!(
            r#"<th style="padding-bottom:16px;font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:0.1em;color:#a1a1aa">{}</th>"#,
            col
        ));
    }

    let mut tbody_html = String::new();
    for item in &sec.items {
        let item_type = item.get("_type").map(|s| s.as_str()).unwrap_or("");
        if item_type != "row" { continue; }

        let event = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let user = item.get("user").map(|s| s.as_str()).unwrap_or("");
        let location = item.get("location").map(|s| s.as_str()).unwrap_or("");
        let ip = item.get("ip").map(|s| s.as_str()).unwrap_or("");
        let time = item.get("time").map(|s| s.as_str()).unwrap_or("");
        let status = item.get("status").map(|s| s.as_str()).unwrap_or("");

        let (dot_color, status_text, status_color) = if status == "blocked" {
            ("#ba1a1a", "Blocked", "color:#ba1a1a;")
        } else {
            ("#059669", "Success", "")
        };

        tbody_html.push_str(&format!(
            r#"<tr style="border-bottom:1px solid #fafafa">
              <td style="padding:16px 0;font-weight:500;font-size:14px">{event}</td>
              <td style="padding:16px 0;font-size:14px"><div style="display:flex;align-items:center;gap:8px"><div style="width:24px;height:24px;border-radius:50%;background:#e5e7eb;flex-shrink:0"></div> {user}</div></td>
              <td style="padding:16px 0;font-size:14px;color:#71717a">{location}</td>
              <td style="padding:16px 0;font-family:monospace;font-size:12px;color:#a1a1aa">{ip}</td>
              <td style="padding:16px 0;font-size:14px;color:#71717a">{time}</td>
              <td style="padding:16px 0"><span style="display:inline-block;width:8px;height:8px;border-radius:50%;background:{dot};margin-right:8px"></span><span style="font-size:12px;font-weight:500;{status_color}">{status_text}</span></td>
            </tr>"#,
            event = event, user = user, location = location,
            ip = ip, time = time, dot = dot_color,
            status_text = status_text, status_color = status_color,
        ));
    }

    let action_html = if !action_text.is_empty() {
        format!(
            r#"<button style="font-size:12px;font-weight:700;color:#000;border:1px solid rgba(0,0,0,0.1);padding:8px 16px;border-radius:999px;background:none;cursor:pointer">{}</button>"#,
            action_text
        )
    } else {
        String::new()
    };

    format!(
        r##"<section class="ghost-border" style="background:#fff;border-radius:12px;padding:24px;box-shadow:0 1px 3px rgba(0,0,0,0.04)">
          <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:32px">
            <h2 style="font-size:20px;font-weight:700;letter-spacing:-0.02em;margin:0">{title}</h2>
            {action}
          </div>
          <div style="overflow-x:auto">
            <table style="width:100%;text-align:left;border-collapse:collapse">
              <thead><tr style="border-bottom:1px solid #f4f4f5">{thead}</tr></thead>
              <tbody style="font-size:14px">{tbody}</tbody>
            </table>
          </div>
        </section>"##,
        title = title, action = action_html, thead = thead_html, tbody = tbody_html,
    )
}

// ══════════════════════════════════════════════════
// SKELETON / LOADING SECTION
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

pub fn render_settings_dashboard(
    app_name: &str,
    sections: &[SectionNode],
    _components: &[crate::parser::ComponentNode],
    theme: &str,
    _current_route: &str,
) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    let _ = theme;
    let sidebar_section = sections.iter().find(|s| s.section_type == "sidebar");
    let topbar_section = sections.iter().find(|s| s.section_type == "topbar");
    let page_header = sections.iter().find(|s| s.section_type == "page-header");
    let profile = sections.iter().find(|s| s.section_type == "settings-profile");
    let api_keys = sections.iter().find(|s| s.section_type == "api-keys");
    let security = sections.iter().find(|s| s.section_type == "security-grid");
    let subscription = sections.iter().find(|s| s.section_type == "subscription-card");
    let invoices = sections.iter().find(|s| s.section_type == "invoices-list");
    let support = sections.iter().find(|s| s.section_type == "support-card");
    let danger = sections.iter().find(|s| s.section_type == "danger-zone");

    let sidebar_html = if let Some(sec) = sidebar_section { render_sidebar(sec) } else { String::new() };
    let topbar_html = if let Some(sec) = topbar_section { build_settings_topbar(sec) } else { String::new() };

    let header_html = if let Some(sec) = page_header {
        let title = sec.title.as_deref().unwrap_or("");
        let subtitle = sec.subtitle.as_deref().unwrap_or("");
        format!(r#"<header style="margin-bottom:48px">
  <h1 style="font-size:clamp(32px,5vw,48px);font-family:'Inter Display','Inter',sans-serif;font-weight:700;letter-spacing:-0.04em;color:#e2e2e2;margin:0">{title}</h1>
  <p style="font-size:14px;color:rgba(207,196,197,1);margin:8px 0 0;line-height:1.6">{subtitle}</p>
</header>"#, title = title, subtitle = subtitle)
    } else { String::new() };

    // ── Profile Form ──
    let profile_html = if let Some(sec) = profile {
        let sec_title = sec.title.as_deref().unwrap_or("");
        let note = sec.config.get("note").map(|s| s.as_str()).unwrap_or("");
        let action_label = sec.config.get("action_label").map(|s| s.as_str()).unwrap_or("");
        let mut fields_html = String::new();
        for item in &sec.items {
            let label = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let value = item.get("value").map(|s| s.as_str()).unwrap_or("");
            let input_type = item.get("type").map(|s| s.as_str()).unwrap_or("text");
            fields_html.push_str(&format!(
                r#"<div style="display:flex;flex-direction:column;gap:6px">
  <label style="font-size:10px;text-transform:uppercase;letter-spacing:0.15em;color:rgba(207,196,197,0.6);font-weight:500">{label}</label>
  <input type="{input_type}" value="{value}" style="width:100%;background:#0e0e0e;border:0.5px solid rgba(76,69,70,0.3);border-radius:8px;padding:10px 16px;color:#e2e2e2;font-size:14px;font-family:inherit;outline:none;transition:box-shadow 0.2s" onfocus="this.style.boxShadow='0 0 0 2px rgba(173,198,255,0.1)'" onblur="this.style.boxShadow='none'">
</div>"#, label = label, input_type = input_type, value = value));
        }
        let note_html = if note.is_empty() { String::new() } else {
            format!(r#"<p style="font-size:12px;color:rgba(226,226,226,0.7);font-style:italic;margin:0">{}</p>"#, note)
        };
        format!(r#"<section style="background:#1f1f1f;border-radius:12px;overflow:hidden;border:0.5px solid rgba(76,69,70,0.15)">
  <div style="padding:32px">
    <h3 style="font-size:20px;font-family:'Inter Display','Inter',sans-serif;font-weight:600;margin:0 0 24px;color:#e2e2e2">{title}</h3>
    <div style="display:grid;grid-template-columns:repeat(2,1fr);gap:24px">{fields}</div>
  </div>
  <div style="background:#1b1b1b;padding:16px 32px;display:flex;justify-content:space-between;align-items:center">
    {note}
    <button style="background:linear-gradient(135deg,#adc6ff 0%,#c2c1ff 50%,#e9b3ff 100%);color:#0071ec;font-weight:600;padding:8px 24px;border-radius:8px;border:none;font-size:14px;cursor:pointer;transition:transform 0.15s;font-family:inherit;box-shadow:0 4px 16px rgba(173,198,255,0.1)" onmousedown="this.style.transform='scale(0.95)'" onmouseup="this.style.transform='scale(1)'">{action}</button>
  </div>
</section>"#, title = sec_title, fields = fields_html, note = note_html, action = action_label)
    } else { String::new() };

    // ── API Keys ──
    let api_keys_html = if let Some(sec) = api_keys {
        let sec_title = sec.title.as_deref().unwrap_or("");
        let sec_subtitle = sec.subtitle.as_deref().unwrap_or("");
        let action_label = sec.config.get("action_label").map(|s| s.as_str()).unwrap_or("");
        let mut keys_html = String::new();
        for (i, item) in sec.items.iter().enumerate() {
            let name = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("");
            let note = item.get("subtitle").map(|s| s.as_str()).unwrap_or("");
            let divider = if i > 0 { r#"<div style="height:0.5px;background:rgba(76,69,70,0.1)"></div>"# } else { "" };
            keys_html.push_str(&format!(
                r#"{divider}<div style="padding:24px;display:flex;align-items:center;justify-content:space-between;transition:background 0.15s" onmouseover="this.style.background='#1b1b1b';this.querySelector('.key-actions').style.opacity='1'" onmouseout="this.style.background='transparent';this.querySelector('.key-actions').style.opacity='0'">
  <div style="display:flex;align-items:center;gap:16px">
    <div style="width:40px;height:40px;background:#1f1f1f;border-radius:6px;display:flex;align-items:center;justify-content:center;border:1px solid rgba(76,69,70,0.2)">
      <span class="material-symbols-outlined" style="color:rgba(207,196,197,1);font-size:20px">{icon}</span>
    </div>
    <div>
      <p style="font-size:14px;font-weight:500;font-family:'Courier New',monospace;margin:0;color:#e2e2e2">{name}</p>
      <p style="font-size:10px;color:rgba(207,196,197,1);text-transform:uppercase;letter-spacing:-0.02em;margin:2px 0 0">{note}</p>
    </div>
  </div>
  <div class="key-actions" style="display:flex;gap:8px;opacity:0;transition:opacity 0.15s">
    <button style="padding:8px;background:none;border:none;cursor:pointer;color:rgba(207,196,197,1)"><span class="material-symbols-outlined" style="font-size:16px">content_copy</span></button>
    <button style="padding:8px;background:none;border:none;cursor:pointer;color:rgba(207,196,197,1)" onmouseover="this.style.color='#ffb4ab'" onmouseout="this.style.color='rgba(207,196,197,1)'"><span class="material-symbols-outlined" style="font-size:16px">delete</span></button>
  </div>
</div>"#, divider = divider, icon = icon, name = name, note = note));
        }
        format!(r#"<section style="margin-top:48px">
  <div style="display:flex;justify-content:space-between;align-items:flex-end;margin-bottom:16px">
    <div>
      <h3 style="font-size:20px;font-family:'Inter Display','Inter',sans-serif;font-weight:600;margin:0;color:#e2e2e2">{title}</h3>
      <p style="font-size:14px;color:rgba(207,196,197,1);margin:4px 0 0">{subtitle}</p>
    </div>
    <button style="font-size:11px;font-family:'Courier New',monospace;color:#adc6ff;background:rgba(173,198,255,0.1);padding:6px 12px;border-radius:999px;border:1px solid rgba(173,198,255,0.2);cursor:pointer;transition:background 0.15s" onmouseover="this.style.background='rgba(173,198,255,0.2)'" onmouseout="this.style.background='rgba(173,198,255,0.1)'">{api_action}</button>
  </div>
  <div style="background:#0e0e0e;border:0.5px solid rgba(76,69,70,0.2);border-radius:12px;overflow:hidden">{keys}</div>
</section>"#, title = sec_title, subtitle = sec_subtitle, keys = keys_html, api_action = action_label)
    } else { String::new() };

    // ── Security Cards ──
    let security_html = if let Some(sec) = security {
        let mut cards_html = String::new();
        let card_accents = ["#adc6ff", "#c2c1ff", "#e9b3ff"];
        for (i, item) in sec.items.iter().enumerate() {
            let title = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("");
            let subtitle = item.get("subtitle").map(|s| s.as_str()).unwrap_or("");
            let action = item.get("action").map(|s| s.as_str()).unwrap_or("");
            let badge = item.get("badge").map(|s| s.as_str()).unwrap_or("");
            let accent = card_accents.get(i).copied().unwrap_or("#adc6ff");
            cards_html.push_str(&format!(
                r#"<div style="background:#2a2a2a;border:0.5px solid rgba(76,69,70,0.2);padding:24px;border-radius:12px;display:flex;flex-direction:column;justify-content:space-between">
  <div>
    <div style="display:flex;align-items:center;gap:8px;color:{accent};margin-bottom:16px">
      <span class="material-symbols-outlined">{icon}</span>
      <span style="font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:0.15em">{badge}</span>
    </div>
    <h4 style="font-size:18px;font-family:'Inter Display','Inter',sans-serif;font-weight:500;margin:0 0 8px;color:#e2e2e2">{title}</h4>
    <p style="font-size:14px;color:rgba(207,196,197,1);margin:0 0 24px;line-height:1.5">{subtitle}</p>
  </div>
  <button style="width:100%;padding:10px;border-radius:8px;border:1px solid rgba(76,69,70,1);background:transparent;color:#e2e2e2;font-size:14px;font-weight:500;cursor:pointer;transition:background 0.15s;font-family:inherit" onmouseover="this.style.background='#353535'" onmouseout="this.style.background='transparent'">{action}</button>
</div>"#, accent = accent, icon = icon, badge = badge, title = title, subtitle = subtitle, action = action));
        }
        format!(r#"<section style="display:grid;grid-template-columns:repeat(2,1fr);gap:24px;margin-top:48px">{cards}</section>"#, cards = cards_html)
    } else { String::new() };

    // ── Subscription Card ──
    let subscription_html = if let Some(sec) = subscription {
        let badge = sec.title.as_deref().unwrap_or("");
        let sub_label = sec.subtitle.as_deref().unwrap_or("");
        let plan_id = sec.config.get("plan_id").map(|s| s.as_str()).unwrap_or("");
        let manage_label = sec.config.get("action_label").map(|s| s.as_str()).unwrap_or("");
        let mut price = "";
        let mut billing_note = "";
        let mut features_html = String::new();
        for item in &sec.items {
            let t = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("");
            if item.get("badge").is_some() { price = t; billing_note = item.get("subtitle").map(|s| s.as_str()).unwrap_or(""); }
            else if icon == "check_circle" {
                features_html.push_str(&format!(r#"<li style="display:flex;align-items:center;gap:12px;font-size:14px;color:rgba(207,196,197,1)">
  <span class="material-symbols-outlined" style="color:#adc6ff;font-size:18px">check_circle</span>
  {title}
</li>"#, title = t));
            }
        }
        format!(r#"<section style="background:rgba(31,31,31,0.4);backdrop-filter:blur(32px);border:0.5px solid rgba(76,69,70,0.2);padding:32px;border-radius:12px;position:relative;overflow:hidden">
  <div style="position:absolute;inset:0;background:linear-gradient(135deg,rgba(173,198,255,0.03) 0%,rgba(194,193,255,0.03) 50%,rgba(233,179,255,0.03) 100%);pointer-events:none;z-index:0"></div>
  <div style="position:relative;z-index:1">
    <div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:40px">
      <div style="background:rgba(173,198,255,0.2);color:#adc6ff;font-size:10px;font-weight:900;padding:4px 8px;border-radius:4px;letter-spacing:-0.02em;text-transform:uppercase">{badge}</div>
      <span style="font-size:10px;color:rgba(207,196,197,1);font-family:monospace">{plan_id}</span>
    </div>
    <div style="margin-bottom:40px">
      <p style="font-size:12px;color:rgba(207,196,197,1);margin:0 0 4px">{sub_label}</p>
      <h2 style="font-size:clamp(40px,5vw,48px);font-family:'Inter Display','Inter',sans-serif;font-weight:900;letter-spacing:-0.04em;margin:0;color:#e2e2e2">{price}</h2>
      <p style="font-size:12px;color:rgba(226,226,226,0.6);margin:8px 0 0;font-style:italic">{note}</p>
    </div>
    <ul style="list-style:none;padding:0;margin:0 0 40px;display:flex;flex-direction:column;gap:12px">{features}</ul>
    <button style="width:100%;padding:12px;border-radius:8px;background:#353535;border:1px solid rgba(76,69,70,0.3);color:#e2e2e2;font-size:14px;font-weight:700;cursor:pointer;transition:background 0.15s;font-family:inherit" onmouseover="this.style.background='#393939'" onmouseout="this.style.background='#353535'">{manage_label}</button>
  </div>
</section>"#, badge = badge, sub_label = sub_label, price = price, note = billing_note, features = features_html, plan_id = plan_id, manage_label = manage_label)
    } else { String::new() };

    // ── Invoices ──
    let invoices_html = if let Some(sec) = invoices {
        let inv_title = sec.title.as_deref().unwrap_or("");
        let footer_link = sec.config.get("footer_link").map(|s| s.as_str()).unwrap_or("");
        let mut rows_html = String::new();
        for item in &sec.items {
            let name = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let value = item.get("value").map(|s| s.as_str()).unwrap_or("");
            rows_html.push_str(&format!(r#"<div style="padding:16px 24px;display:flex;justify-content:space-between;align-items:center;font-size:14px">
  <span style="font-family:monospace;font-size:12px;color:#e2e2e2">{name}</span>
  <span style="color:rgba(207,196,197,1)">{value}</span>
  <button style="background:none;border:none;cursor:pointer;color:rgba(207,196,197,0.4);transition:color 0.15s" onmouseover="this.style.color='#e2e2e2'" onmouseout="this.style.color='rgba(207,196,197,0.4)'"><span class="material-symbols-outlined" style="font-size:18px">download</span></button>
</div>"#, name = name, value = value));
        }
        format!(r#"<section style="background:#0e0e0e;border:0.5px solid rgba(76,69,70,0.15);border-radius:12px;overflow:hidden;margin-top:32px">
  <div style="padding:16px 24px;border-bottom:1px solid rgba(76,69,70,0.1);display:flex;justify-content:space-between;align-items:center">
    <h4 style="font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:0.15em;color:rgba(207,196,197,1);margin:0">{inv_title}</h4>
    <span class="material-symbols-outlined" style="color:rgba(207,196,197,1);font-size:16px">receipt_long</span>
  </div>
  <div>{rows}</div>
  <button style="width:100%;padding:12px;font-size:10px;text-transform:uppercase;font-weight:700;letter-spacing:0.15em;color:rgba(207,196,197,1);background:#1b1b1b;border:none;cursor:pointer;transition:color 0.15s;font-family:inherit" onmouseover="this.style.color='#e2e2e2'" onmouseout="this.style.color='rgba(207,196,197,1)'">{inv_footer}</button>
</section>"#, rows = rows_html, inv_title = inv_title, inv_footer = footer_link)
    } else { String::new() };

    // ── Support Card ──
    let support_html = if let Some(sec) = support {
        let title = sec.title.as_deref().unwrap_or("");
        let subtitle = sec.subtitle.as_deref().unwrap_or("");
        let action_label = sec.config.get("action_label").map(|s| s.as_str()).unwrap_or("");
        let action_html = if action_label.is_empty() { String::new() } else {
            format!(r##"<a href="#" style="font-size:12px;color:#adc6ff;font-weight:700;text-decoration:none;display:flex;align-items:center;gap:4px" onmouseover="this.style.textDecoration='underline'" onmouseout="this.style.textDecoration='none'">
    {label}
    <span class="material-symbols-outlined" style="font-size:14px">arrow_outward</span>
  </a>"##, label = action_label)
        };
        format!(r#"<section style="background:#1b1b1b;padding:24px;border-radius:12px;border-left:4px solid rgba(173,198,255,0.4);margin-top:32px">
  <h5 style="font-size:14px;font-weight:700;margin:0 0 8px;color:#e2e2e2">{title}</h5>
  <p style="font-size:12px;color:rgba(207,196,197,1);line-height:1.6;margin:0 0 16px">{subtitle}</p>
  {action}
</section>"#, title = title, subtitle = subtitle, action = action_html)
    } else { String::new() };

    // ── Danger Zone ──
    let danger_html = if let Some(sec) = danger {
        let sec_title = sec.title.as_deref().unwrap_or("");
        let item_title = sec.subtitle.as_deref().unwrap_or("");
        let action_label = sec.config.get("action_label").map(|s| s.as_str()).unwrap_or("");
        let desc = sec.items.first().and_then(|i| i.get("title")).map(|s| s.as_str()).unwrap_or("");
        format!(r#"<div style="margin-top:96px;border-top:1px solid rgba(255,180,171,0.2);padding-top:48px">
  <h3 style="font-size:20px;font-family:'Inter Display','Inter',sans-serif;font-weight:600;color:#ffb4ab;margin:0 0 16px">{sec_title}</h3>
  <div style="background:#1f1f1f;border:0.5px solid rgba(255,180,171,0.3);padding:32px;border-radius:12px;display:flex;justify-content:space-between;align-items:center;gap:24px;flex-wrap:wrap">
    <div style="flex:1;min-width:200px">
      <h4 style="font-weight:700;margin:0 0 4px;color:#e2e2e2">{item_title}</h4>
      <p style="font-size:14px;color:rgba(207,196,197,1);margin:0;line-height:1.5">{desc}</p>
    </div>
    <button style="flex-shrink:0;padding:12px 32px;background:#93000a;color:#ffdad6;font-size:14px;font-weight:700;border-radius:8px;border:1px solid rgba(255,180,171,0.5);cursor:pointer;transition:all 0.15s;font-family:inherit" onmouseover="this.style.background='#ffb4ab';this.style.color='#690005'" onmouseout="this.style.background='#93000a';this.style.color='#ffdad6'" onmousedown="this.style.transform='scale(0.95)'" onmouseup="this.style.transform='scale(1)'">{danger_action}</button>
  </div>
</div>"#, sec_title = sec_title, item_title = item_title, desc = desc, danger_action = action_label)
    } else { String::new() };

    // ── Full Page ──
    format!(r##"<!DOCTYPE html>
<html class="dark" lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta name="view-transition" content="same-origin">
  <title>{app_name}</title>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800;900&family=Inter+Display:wght@400;500;600;700;800&display=swap" rel="stylesheet">
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet">
  <style>
    body {{ font-family:'Inter',sans-serif; background:#131313; color:#e2e2e2; margin:0; -webkit-font-smoothing:antialiased; }}
    * {{ box-sizing:border-box; }}
    .material-symbols-outlined {{ font-variation-settings:'FILL' 0,'wght' 400,'GRAD' 0,'opsz' 24; font-size:20px; display:inline-block; line-height:1; vertical-align:middle; }}
    ::selection {{ background:rgba(173,198,255,0.2); }}
    ::-webkit-scrollbar {{ width:4px; }}
    ::-webkit-scrollbar-thumb {{ background:rgba(255,255,255,0.1); border-radius:2px; }}
    input:focus {{ outline:none; box-shadow:0 0 0 2px rgba(173,198,255,0.1); }}
    @keyframes fadeIn {{ from {{ opacity:0;transform:translateY(4px) }} to {{ opacity:1;transform:translateY(0) }} }}
    @keyframes slideUp {{ from {{ opacity:0; transform:translateY(24px) }} to {{ opacity:1; transform:translateY(0) }} }}
    .anim-fade {{ animation:fadeIn 0.6s ease-out both }}
    .anim-slide-up {{ animation:slideUp 0.6s cubic-bezier(0.16,1,0.3,1) both }}
    .d1 {{ animation-delay:0.05s }} .d2 {{ animation-delay:0.1s }} .d3 {{ animation-delay:0.15s }}
    .d4 {{ animation-delay:0.2s }} .d5 {{ animation-delay:0.25s }} .d6 {{ animation-delay:0.3s }}
  </style>
</head>
<body>
{topbar}
{sidebar}
<main style="margin-left:256px;padding:96px 32px 48px;max-width:calc(100% - 256px)">
  <div style="max-width:1152px;margin:0 auto">
    {header}
    <div style="display:grid;grid-template-columns:1fr 1fr 1fr;gap:32px">
      <div style="grid-column:span 2;display:flex;flex-direction:column">
        <div class="anim-slide-up d1">{profile}</div>
        <div class="anim-slide-up d2">{api_keys}</div>
        <div class="anim-slide-up d3">{security}</div>
      </div>
      <div style="display:flex;flex-direction:column">
        <div class="anim-slide-up d2">{subscription}</div>
        <div class="anim-slide-up d3">{invoices}</div>
        <div class="anim-slide-up d4">{support}</div>
      </div>
    </div>
    <div class="anim-slide-up d5">{danger}</div>
  </div>
</main>
<script{script_nonce}>{runtime}</script>
<script{script_nonce}>{hmr}</script>
</body>
</html>"##,
        app_name = app_name, topbar = topbar_html, sidebar = sidebar_html, header = header_html,
        profile = profile_html, api_keys = api_keys_html, security = security_html,
        subscription = subscription_html, invoices = invoices_html, support = support_html,
        danger = danger_html, runtime = crate::render::CRONUS_RUNTIME_JS, hmr = crate::hmr::HMR_CLIENT_JS,
    )
}

pub(super) fn build_settings_topbar(section: &SectionNode) -> String {
    let brand = section.config.get("brand").map(|s| s.as_str()).unwrap_or("");
    let nav_str = section.config.get("nav").map(|s| s.as_str()).unwrap_or("");
    let mut nav_html = String::new();
    for item in nav_str.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
        nav_html.push_str(&format!(
            r##"<a href="#" style="color:rgba(226,226,226,0.6);text-decoration:none;transition:color 0.3s" onmouseover="this.style.color='#e2e2e2'" onmouseout="this.style.color='rgba(226,226,226,0.6)'">{}</a>"##, item));
    }
    let mut actions_html = String::new();
    for item in &section.items {
        let item_type = item.get("_type").map(|s| s.as_str()).unwrap_or("item");
        if item_type == "action" {
            let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("");
            let style = item.get("style").map(|s| s.as_str()).unwrap_or("");
            if style == "active" {
                actions_html.push_str(&format!(r#"<button style="background:none;border:none;cursor:pointer;color:#adc6ff;border-bottom:2px solid #adc6ff;padding-bottom:4px"><span class="material-symbols-outlined">{}</span></button>"#, icon));
            } else {
                actions_html.push_str(&format!(r#"<button style="background:none;border:none;cursor:pointer;color:rgba(226,226,226,0.6);transition:color 0.15s" onmouseover="this.style.color='#e2e2e2'" onmouseout="this.style.color='rgba(226,226,226,0.6)'"><span class="material-symbols-outlined">{}</span></button>"#, icon));
            }
        }
    }
    let avatar_item = section.items.iter().find(|i| i.get("_type").map(|s| s.as_str()) == Some("image"));
    let avatar_src = avatar_item.and_then(|i| i.get("src")).map(|s| s.as_str()).unwrap_or("");
    let avatar_alt = avatar_item.and_then(|i| i.get("title")).map(|s| s.as_str()).unwrap_or("");
    let avatar_html = if !avatar_src.is_empty() {
        format!(r#"<div style="height:32px;width:32px;border-radius:50%;background:#2a2a2a;border:0.5px solid rgba(76,69,70,0.2);overflow:hidden">
  <img src="{src}" alt="{alt}" style="width:100%;height:100%;object-fit:cover">
</div>"#, src = avatar_src, alt = avatar_alt)
    } else { String::new() };
    format!(r##"<nav style="position:fixed;top:0;width:100%;z-index:50;background:rgba(19,19,19,0.8);backdrop-filter:blur(24px);border-bottom:0.5px solid rgba(76,69,70,0.2);box-shadow:0 8px 32px rgba(0,0,0,0.36);display:flex;align-items:center;justify-content:space-between;padding:0 32px;height:64px;font-family:'Inter Display','Inter',sans-serif;letter-spacing:-0.02em">
  <div style="display:flex;align-items:center;gap:32px">
    <span style="font-size:20px;font-weight:700;letter-spacing:-0.05em;color:#e2e2e2">{brand}</span>
    <div style="display:flex;gap:24px;align-items:center">{nav}</div>
  </div>
  <div style="display:flex;align-items:center;gap:16px">{actions}{avatar}</div>
</nav>"##, brand = brand, nav = nav_html, actions = actions_html, avatar = avatar_html)
}

// ══════════════════════════════════════════════════
// ORDER DETAIL DASHBOARD (dark Obsidian — full page renderer)
// ══════════════════════════════════════════════════

pub fn render_order_detail_dashboard(
    app_name: &str,
    sections: &[SectionNode],
    _components: &[crate::parser::ComponentNode],
    theme: &str,
    _current_route: &str,
) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    let _ = theme;
    let sidebar_section = sections.iter().find(|s| s.section_type == "sidebar");
    let topbar_section = sections.iter().find(|s| s.section_type == "topbar");
    let order_header = sections.iter().find(|s| s.section_type == "order-header");
    let line_items = sections.iter().find(|s| s.section_type == "line-items");
    let price_breakdown = sections.iter().find(|s| s.section_type == "price-breakdown");
    let payment_info = sections.iter().find(|s| s.section_type == "payment-info");
    let customer_profile = sections.iter().find(|s| s.section_type == "customer-profile");
    let shipping_timeline = sections.iter().find(|s| s.section_type == "shipping-timeline");
    let staff_notes = sections.iter().find(|s| s.section_type == "staff-notes");

    let sidebar_html = if let Some(sec) = sidebar_section { render_sidebar(sec) } else { String::new() };
    let topbar_html = if let Some(sec) = topbar_section { build_settings_topbar(sec) } else { String::new() };

    // ── Order Header ──
    let header_html = if let Some(sec) = order_header {
        let title = sec.title.as_deref().unwrap_or("");
        let subtitle = sec.subtitle.as_deref().unwrap_or("");
        let back_link = sec.config.get("back_link").map(|s| s.as_str()).unwrap_or("");
        let badge = sec.config.get("badge").map(|s| s.as_str()).unwrap_or("");
        let action_primary = sec.config.get("action_primary").map(|s| s.as_str()).unwrap_or("");
        let action_secondary = sec.config.get("action_secondary").map(|s| s.as_str()).unwrap_or("");
        let back_html = if back_link.is_empty() { String::new() } else {
            format!(r##"<a href="#" style="color:#adc6ff;font-size:14px;font-weight:500;text-decoration:none;display:flex;align-items:center;gap:4px"><span class="material-symbols-outlined" style="font-size:14px">arrow_back</span> {}</a>"##, back_link)
        };
        let badge_html = if badge.is_empty() { String::new() } else {
            format!(r#"<span style="padding:2px 8px;border-radius:4px;font-size:10px;font-weight:700;background:#3630bf;color:#e2dfff;letter-spacing:0.1em;text-transform:uppercase">{}</span>"#, badge)
        };
        let sec_btn = if action_secondary.is_empty() { String::new() } else {
            format!(r#"<button style="padding:10px 24px;background:#2a2a2a;color:#e2e2e2;font-weight:600;border-radius:8px;border:0.5px solid rgba(76,69,70,0.2);cursor:pointer;font-family:inherit;font-size:14px;transition:background 0.15s" onmouseover="this.style.background='#353535'" onmouseout="this.style.background='#2a2a2a'">{}</button>"#, action_secondary)
        };
        let pri_btn = if action_primary.is_empty() { String::new() } else {
            format!(r#"<button style="padding:10px 24px;background:linear-gradient(135deg,#adc6ff 0%,#c2c1ff 50%,#e9b3ff 100%);color:#000;font-weight:800;border-radius:8px;border:none;cursor:pointer;font-family:inherit;font-size:14px;transition:all 0.15s;box-shadow:0 4px 16px rgba(173,198,255,0.2)" onmousedown="this.style.transform='scale(0.95)'" onmouseup="this.style.transform='scale(1)'">{}</button>"#, action_primary)
        };
        format!(r#"<section style="display:flex;justify-content:space-between;align-items:flex-end">
  <div>
    <div style="display:flex;align-items:center;gap:12px;margin-bottom:8px">{back}{badge}</div>
    <h2 style="font-size:clamp(36px,5vw,48px);font-weight:800;letter-spacing:-0.04em;margin:0;color:#e2e2e2">{title}</h2>
    <p style="font-size:14px;color:rgba(226,226,226,0.6);margin:4px 0 0;letter-spacing:0.02em">{subtitle}</p>
  </div>
  <div style="display:flex;gap:16px">{sec_btn}{pri_btn}</div>
</section>"#, back = back_html, badge = badge_html, title = title, subtitle = subtitle, sec_btn = sec_btn, pri_btn = pri_btn)
    } else { String::new() };

    // ── Line Items ──
    let items_html = if let Some(sec) = line_items {
        let sec_title = sec.title.as_deref().unwrap_or("");
        let mut rows = String::new();
        for item in &sec.items {
            let name = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let sku = item.get("sku").map(|s| s.as_str()).unwrap_or("");
            let price = item.get("price").map(|s| s.as_str()).unwrap_or("");
            let qty = item.get("qty").map(|s| s.as_str()).unwrap_or("");
            let variant = item.get("variant").map(|s| s.as_str()).unwrap_or("");
            let image = item.get("image").map(|s| s.as_str()).unwrap_or("");
            let img_html = if image.is_empty() { String::new() } else {
                format!(r#"<div style="width:96px;height:96px;border-radius:8px;overflow:hidden;background:#0e0e0e;border:0.5px solid rgba(76,69,70,0.2);flex-shrink:0">
  <img src="{}" alt="{}" style="width:100%;height:100%;object-fit:cover;filter:grayscale(0.2)">
</div>"#, image, name)
            };
            rows.push_str(&format!(r#"<div style="display:flex;align-items:center;gap:24px;padding:16px 0">
  {img}
  <div style="flex:1">
    <p style="font-size:12px;color:#c2c1ff;font-family:monospace;letter-spacing:-0.02em;opacity:0.7;margin:0">SKU: {sku}</p>
    <h4 style="font-size:20px;font-weight:700;letter-spacing:-0.02em;margin:4px 0;color:#e2e2e2">{name}</h4>
    <p style="font-size:14px;color:rgba(207,196,197,1);margin:0">{variant}</p>
  </div>
  <div style="text-align:right">
    <p style="font-size:18px;font-weight:700;letter-spacing:-0.02em;margin:0;color:#e2e2e2">{price}</p>
    <p style="font-size:12px;color:rgba(207,196,197,1);margin:2px 0 0">Qty: {qty}</p>
  </div>
</div>"#, img = img_html, sku = sku, name = name, variant = variant, price = price, qty = qty));
        }
        format!(r#"<div style="background:#1f1f1f;border-radius:12px;padding:32px;border:0.5px solid rgba(76,69,70,0.1)">
  <h3 style="font-size:12px;font-weight:700;text-transform:uppercase;letter-spacing:0.2em;color:#adc6ff;margin:0 0 32px">{title}</h3>
  <div style="display:flex;flex-direction:column;gap:16px">{rows}</div>"#, title = sec_title, rows = rows)
    } else { String::new() };

    // ── Price Breakdown (inside items card) ──
    let breakdown_html = if let Some(sec) = price_breakdown {
        let mut rows = String::new();
        let mut total_html = String::new();
        for item in &sec.items {
            let label = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let value = item.get("value").map(|s| s.as_str()).unwrap_or("");
            let is_total = item.get("style").map(|s| s == "total").unwrap_or(false);
            if is_total {
                total_html = format!(r#"<div style="display:flex;justify-content:space-between;align-items:center;padding-top:16px">
  <span style="font-size:20px;font-weight:700;letter-spacing:-0.02em;color:#e2e2e2">{label}</span>
  <span style="font-size:28px;font-weight:900;letter-spacing:-0.02em;color:#adc6ff">{value}</span>
</div>"#, label = label, value = value);
            } else {
                rows.push_str(&format!(r#"<div style="display:flex;justify-content:space-between;align-items:center;font-size:14px;color:rgba(207,196,197,1)">
  <span>{label}</span><span style="font-family:monospace">{value}</span>
</div>"#, label = label, value = value));
            }
        }
        format!(r#"  <div style="margin-top:48px;padding-top:32px;border-top:1px solid rgba(76,69,70,0.1);display:flex;flex-direction:column;gap:16px">
    {rows}
    {total}
  </div>
</div>"#, rows = rows, total = total_html)
    } else {
        // Close the items card div even without breakdown
        "</div>".to_string()
    };

    // ── Payment Info (2-col grid) ──
    let payment_html = if let Some(sec) = payment_info {
        let mut cards = String::new();
        let accents = [("primary", "#adc6ff"), ("secondary", "#c2c1ff")];
        for (i, item) in sec.items.iter().enumerate() {
            let heading = item.get("title").map(|s| s.as_str()).unwrap_or("");
            // The actual heading is in the first field; inside {} block we have title/subtitle
            // But our parser stores it differently: the item title is the section heading
            // and "title" inside {} is the detail title
            let section_heading = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let detail_title = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let detail_subtitle = item.get("subtitle").map(|s| s.as_str()).unwrap_or("");
            let label_text = item.get("label").map(|s| s.as_str()).unwrap_or("");
            let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("");
            let (_, accent_color) = accents.get(i).copied().unwrap_or(("primary", "#adc6ff"));

            let icon_html = if !label_text.is_empty() {
                format!(r#"<div style="width:48px;height:32px;border-radius:4px;background:#0e0e0e;border:1px solid rgba(76,69,70,0.3);display:flex;align-items:center;justify-content:center">
  <span style="font-size:10px;font-weight:900;letter-spacing:0.1em;color:#e2e2e2">{}</span>
</div>"#, label_text)
            } else if !icon.is_empty() {
                format!(r#"<span class="material-symbols-outlined" style="color:#a944dc;font-size:24px">{}</span>"#, icon)
            } else { String::new() };

            cards.push_str(&format!(r#"<div style="background:#1f1f1f;padding:24px;border-radius:12px;border:0.5px solid rgba(76,69,70,0.1);position:relative;overflow:hidden">
  <h3 style="font-size:10px;font-weight:900;text-transform:uppercase;letter-spacing:0.2em;color:{accent}99;margin:0 0 16px">{heading}</h3>
  <div style="display:flex;align-items:center;gap:16px">
    {icon_html}
    <div>
      <p style="font-weight:700;letter-spacing:-0.02em;margin:0;color:#e2e2e2">{detail_title}</p>
      <p style="font-size:12px;color:rgba(207,196,197,1);margin:2px 0 0">{detail_sub}</p>
    </div>
  </div>
</div>"#, accent = accent_color, heading = heading, icon_html = icon_html,
                detail_title = detail_title, detail_sub = detail_subtitle));
        }
        format!(r#"<div style="display:grid;grid-template-columns:1fr 1fr;gap:24px">{}</div>"#, cards)
    } else { String::new() };

    // ── Customer Profile ──
    let customer_html = if let Some(sec) = customer_profile {
        let sec_title = sec.title.as_deref().unwrap_or("");
        let name = sec.config.get("customer_name").map(|s| s.as_str()).unwrap_or("");
        let tier = sec.config.get("customer_tier").map(|s| s.as_str()).unwrap_or("");
        let email = sec.config.get("customer_email").map(|s| s.as_str()).unwrap_or("");
        let email_label = sec.config.get("email_label").map(|s| s.as_str()).unwrap_or("");
        let address_label = sec.config.get("address_label").map(|s| s.as_str()).unwrap_or("");
        let address = sec.config.get("customer_address").map(|s| s.as_str()).unwrap_or("").replace("\\n", "<br/>");
        let avatar = sec.config.get("customer_avatar").map(|s| s.as_str()).unwrap_or("");
        let avatar_html = if avatar.is_empty() { String::new() } else {
            format!(r#"<div style="height:56px;width:56px;border-radius:50%;background:#2a2a2a;border:1px solid rgba(76,69,70,0.3);overflow:hidden;flex-shrink:0">
  <img src="{}" alt="{}" style="width:100%;height:100%;object-fit:cover">
</div>"#, avatar, name)
        };
        format!(r#"<div style="background:#1f1f1f;border-radius:12px;padding:24px;border:0.5px solid rgba(76,69,70,0.1)">
  <h3 style="font-size:12px;font-weight:700;text-transform:uppercase;letter-spacing:0.2em;color:#adc6ff;margin:0 0 24px">{sec_title}</h3>
  <div style="display:flex;align-items:center;gap:16px;margin-bottom:24px">
    {avatar}
    <div>
      <h4 style="font-size:18px;font-weight:800;letter-spacing:-0.02em;margin:0;color:#e2e2e2">{name}</h4>
      <p style="font-size:12px;color:#adc6ff;font-weight:500;margin:2px 0 0">{tier}</p>
    </div>
  </div>
  <div style="display:flex;flex-direction:column;gap:16px">
    <div>
      <p style="font-size:10px;text-transform:uppercase;color:rgba(207,196,197,1);font-weight:700;letter-spacing:0.15em;margin:0 0 4px">{email_label}</p>
      <p style="font-size:14px;font-weight:500;margin:0;color:#e2e2e2">{email}</p>
    </div>
    <div>
      <p style="font-size:10px;text-transform:uppercase;color:rgba(207,196,197,1);font-weight:700;letter-spacing:0.15em;margin:0 0 4px">{address_label}</p>
      <p style="font-size:14px;font-weight:500;margin:0;color:#e2e2e2;line-height:1.6">{address}</p>
    </div>
  </div>
</div>"#, sec_title = sec_title, avatar = avatar_html, name = name, tier = tier, email = email, email_label = email_label, address_label = address_label, address = address)
    } else { String::new() };

    // ── Shipping Timeline ──
    let timeline_html = if let Some(sec) = shipping_timeline {
        let sec_title = sec.title.as_deref().unwrap_or("");
        let mut steps = String::new();
        for item in &sec.items {
            let title = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let subtitle = item.get("subtitle").map(|s| s.as_str()).unwrap_or("");
            let style = item.get("style").map(|s| s.as_str()).unwrap_or("completed");
            let detail = item.get("detail").map(|s| s.as_str()).unwrap_or("");
            let (dot_style, title_color) = match style {
                "future" => ("background:#0e0e0e;border:1px solid rgba(76,69,70,0.5)", "color:rgba(226,226,226,0.4)"),
                "active" => ("background:#1f1f1f;border:2px solid #adc6ff;animation:pulse 2s infinite", "color:#adc6ff"),
                _ => ("background:#adc6ff", "color:#e2e2e2"),
            };
            let check_html = if style == "completed" {
                r#"<span class="material-symbols-outlined" style="font-size:10px;color:#000;font-weight:900">check</span>"#
            } else { "" };
            let detail_html = if detail.is_empty() { String::new() } else {
                format!(r#"<p style="font-size:11px;margin:4px 0 0;color:rgba(207,196,197,1);font-style:italic">{}</p>"#, detail)
            };
            let sub_color = if style == "future" { "color:rgba(226,226,226,0.4)" } else { "color:rgba(207,196,197,1)" };
            steps.push_str(&format!(r#"<div style="position:relative;padding-left:32px;padding-bottom:24px">
  <div style="position:absolute;left:0;top:4px;width:14px;height:14px;border-radius:50%;{dot_style};display:flex;align-items:center;justify-content:center">{check}</div>
  <div>
    <p style="font-size:14px;font-weight:700;margin:0;{title_color}">{title}</p>
    <p style="font-size:10px;{sub_color};margin:2px 0 0">{subtitle}</p>
    {detail}
  </div>
</div>"#, dot_style = dot_style, check = check_html, title_color = title_color, title = title, sub_color = sub_color, subtitle = subtitle, detail = detail_html));
        }
        format!(r#"<div style="background:#1f1f1f;border-radius:12px;padding:24px;border:0.5px solid rgba(76,69,70,0.1)">
  <h3 style="font-size:12px;font-weight:700;text-transform:uppercase;letter-spacing:0.2em;color:#adc6ff;margin:0 0 32px">{sec_title}</h3>
  <div style="position:relative;padding-left:8px">
    <div style="position:absolute;left:14px;top:8px;bottom:8px;width:1px;background:rgba(76,69,70,0.2)"></div>
    {steps}
  </div>
</div>"#, sec_title = sec_title, steps = steps)
    } else { String::new() };

    // ── Staff Notes ──
    let notes_html = if let Some(sec) = staff_notes {
        let sec_title = sec.title.as_deref().unwrap_or("");
        let action_label = sec.config.get("action_label").map(|s| s.as_str()).unwrap_or("");
        let mut notes = String::new();
        for item in &sec.items {
            let text = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let meta = item.get("meta").map(|s| s.as_str()).unwrap_or("");
            notes.push_str(&format!(r#"<div style="padding:12px;background:#1f1f1f;border-radius:8px">
  <p style="font-size:12px;line-height:1.6;color:rgba(207,196,197,1);font-style:italic;margin:0">"{text}"</p>
  <p style="font-size:10px;margin:8px 0 0;font-weight:700;color:#c2c1ff">{meta}</p>
</div>"#, text = text, meta = meta));
        }
        let action_html = if action_label.is_empty() { String::new() } else {
            format!(r#"<button style="width:100%;padding:8px;background:#2a2a2a;border:none;border-radius:6px;color:#e2e2e2;font-size:11px;font-weight:700;letter-spacing:0.1em;text-transform:uppercase;cursor:pointer;display:flex;align-items:center;justify-content:center;gap:8px;font-family:inherit;transition:background 0.15s" onmouseover="this.style.background='#353535'" onmouseout="this.style.background='#2a2a2a'"><span class="material-symbols-outlined" style="font-size:14px">add_comment</span> {}</button>"#, action_label)
        };
        format!(r#"<div style="background:#0e0e0e;border-radius:12px;padding:24px;border:0.5px dashed rgba(76,69,70,0.15)">
  <h3 style="font-size:12px;font-weight:700;text-transform:uppercase;letter-spacing:0.2em;color:rgba(207,196,197,1);margin:0 0 16px">{sec_title}</h3>
  <div style="display:flex;flex-direction:column;gap:16px">
    {notes}
    {action}
  </div>
</div>"#, sec_title = sec_title, notes = notes, action = action_html)
    } else { String::new() };

    // ── Full Page ──
    format!(r##"<!DOCTYPE html>
<html class="dark" lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta name="view-transition" content="same-origin">
  <title>{app_name}</title>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@100..900&display=swap" rel="stylesheet">
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet">
  <style>
    body {{ font-family:'Inter',sans-serif; background:#131313; color:#e2e2e2; margin:0; -webkit-font-smoothing:antialiased; }}
    * {{ box-sizing:border-box; }}
    .material-symbols-outlined {{ font-variation-settings:'FILL' 0,'wght' 400,'GRAD' 0,'opsz' 24; font-size:20px; display:inline-block; line-height:1; vertical-align:middle; }}
    ::selection {{ background:rgba(173,198,255,0.2); }}
    ::-webkit-scrollbar {{ width:4px; }}
    ::-webkit-scrollbar-thumb {{ background:rgba(255,255,255,0.1); border-radius:2px; }}
    @keyframes pulse {{ 0%,100% {{ opacity:1 }} 50% {{ opacity:0.5 }} }}
    @keyframes slideUp {{ from {{ opacity:0; transform:translateY(24px) }} to {{ opacity:1; transform:translateY(0) }} }}
    .anim-slide-up {{ animation:slideUp 0.6s cubic-bezier(0.16,1,0.3,1) both }}
    .d1 {{ animation-delay:0.05s }} .d2 {{ animation-delay:0.1s }} .d3 {{ animation-delay:0.15s }}
    .d4 {{ animation-delay:0.2s }} .d5 {{ animation-delay:0.25s }} .d6 {{ animation-delay:0.3s }}
    .d7 {{ animation-delay:0.35s }}
  </style>
</head>
<body>
{topbar}
{sidebar}
<main style="margin-left:256px;padding:80px 48px 48px">
  <div style="max-width:1200px;margin:0 auto;display:flex;flex-direction:column;gap:48px">
    <div class="anim-slide-up d1">{header}</div>
    <div style="display:grid;grid-template-columns:2fr 1fr;gap:24px">
      <div style="display:flex;flex-direction:column;gap:24px">
        <div class="anim-slide-up d2">{items}{breakdown}</div>
        <div class="anim-slide-up d4">{payment}</div>
      </div>
      <div style="display:flex;flex-direction:column;gap:24px">
        <div class="anim-slide-up d3">{customer}</div>
        <div class="anim-slide-up d5">{timeline}</div>
        <div class="anim-slide-up d6">{notes}</div>
      </div>
    </div>
  </div>
</main>
<script{script_nonce}>{runtime}</script>
<script{script_nonce}>{hmr}</script>
</body>
</html>"##,
        app_name = app_name, topbar = topbar_html, sidebar = sidebar_html,
        header = header_html, items = items_html, breakdown = breakdown_html,
        payment = payment_html, customer = customer_html, timeline = timeline_html,
        notes = notes_html, runtime = crate::render::CRONUS_RUNTIME_JS, hmr = crate::hmr::HMR_CLIENT_JS,
    )
}
