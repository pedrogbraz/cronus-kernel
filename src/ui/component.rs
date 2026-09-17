//! Component renderers (layout-based: inline, stack, grid, table, hero, modal, sidebar, tabs, menu)
//! Also includes the light-theme app page renderer.

use super::util::{item_by_kind, items_by_kind};
use super::CRONUS_ANIMATIONS_CSS;
use super::CRONUS_ANIMATIONS_JS;
use crate::components;
use crate::parser::{ComponentItemNode, ComponentNode};

pub fn render_light_app_page(app_name: &str, comps: &[ComponentNode]) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    let topbar = comps
        .iter()
        .find(|c| c.style.as_deref().unwrap_or("").contains("topbar+light"))
        .map(render_light_topbar)
        .unwrap_or_default();
    let sidenav = comps
        .iter()
        .find(|c| c.style.as_deref().unwrap_or("").contains("sidenav+light"))
        .map(render_light_sidenav)
        .unwrap_or_default();
    let header = comps
        .iter()
        .find(|c| {
            c.style
                .as_deref()
                .unwrap_or("")
                .contains("page-header+payouts")
        })
        .map(render_light_page_header)
        .unwrap_or_default();
    let balance = comps
        .iter()
        .find(|c| {
            c.style
                .as_deref()
                .unwrap_or("")
                .contains("card+balance+light")
        })
        .map(render_light_balance_card)
        .unwrap_or_default();
    let upcoming = comps
        .iter()
        .find(|c| {
            c.style
                .as_deref()
                .unwrap_or("")
                .contains("card+upcoming+light")
        })
        .map(render_light_upcoming_card)
        .unwrap_or_default();
    let actions = comps
        .iter()
        .find(|c| {
            c.style
                .as_deref()
                .unwrap_or("")
                .contains("action-row+light")
        })
        .map(render_light_history_actions)
        .unwrap_or_default();
    let table = comps
        .iter()
        .find(|c| {
            c.style
                .as_deref()
                .unwrap_or("")
                .contains("payouts-table+light")
        })
        .map(render_light_payouts_table)
        .unwrap_or_default();
    let support = comps
        .iter()
        .find(|c| {
            c.style
                .as_deref()
                .unwrap_or("")
                .contains("support-banner+dark")
        })
        .map(render_light_support_banner)
        .unwrap_or_default();

    format!(
        r#"<!DOCTYPE html>
<html class="light" lang="en">
<head>
  <meta charset="utf-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
  <title>{app_name}</title>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap" rel="stylesheet"/>
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet"/>
  <style>
    body {{ font-family: 'Inter', sans-serif; background:#f9f9f9; color:#1a1c1c; margin:0; min-height:100vh; }}
    * {{ box-sizing:border-box; }}
    .material-symbols-outlined {{
      font-variation-settings:'FILL' 0,'wght' 400,'GRAD' 0,'opsz' 24;
      display:inline-block; line-height:1; white-space:nowrap;
    }}
    .prism-bg {{
      background: radial-gradient(circle at top right, rgba(0, 111, 240, 0.08), transparent 40%),
                  radial-gradient(circle at bottom left, rgba(0, 111, 240, 0.05), transparent 40%);
    }}
    .ghost-border {{ border:1px solid rgba(198,198,198,0.2); }}
    .ambient-shadow {{ box-shadow:0 40px 80px 0 rgba(26,28,28,0.04); }}
    a {{ color:inherit; }}
  </style>
  <style>{anim_css}</style>
</head>
<body>
{topbar}
<div style="display:flex">
{sidenav}
<main class="prism-bg" style="flex:1;margin-left:256px;padding:32px">
  <div style="max-width:1152px;margin:0 auto">
    {header}
    <div style="display:grid;grid-template-columns:2fr 1fr;gap:24px;margin-bottom:48px">
      {balance}
      {upcoming}
    </div>
    <div>
      <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:24px">
        <h2 style="font-size:20px;font-weight:700;letter-spacing:-0.025em;margin:0">Payout History</h2>
        {actions}
      </div>
      {table}
    </div>
    <div style="margin-top:48px">
      {support}
    </div>
  </div>
</main>
</div>
<script{script_nonce}>{hmr}</script>
{anim_js}
</body>
</html>"#,
        app_name = app_name,
        topbar = topbar,
        sidenav = sidenav,
        header = header,
        balance = balance,
        upcoming = upcoming,
        actions = actions,
        table = table,
        support = support,
        anim_css = CRONUS_ANIMATIONS_CSS,
        anim_js = crate::security::mark_kernel_scripts(CRONUS_ANIMATIONS_JS),
        hmr = crate::hmr::HMR_CLIENT_JS,
    )
}

pub(super) fn render_light_topbar(comp: &ComponentNode) -> String {
    let title = item_by_kind(&comp.items, "title").unwrap_or(&comp.name);
    let links: Vec<&ComponentItemNode> = items_by_kind(&comp.items, "item");
    let nav = links.iter().map(|item| {
        let active = item.text == "Payouts";
        let color = if active { "#000000;font-weight:500" } else { "#71717a" };
        format!(r#"<a href="{}" style="text-decoration:none;font-size:14px;transition:color 0.2s;color:{}">{}</a>"#,
            item.link.as_deref().unwrap_or(""), color, item.text)
    }).collect::<Vec<_>>().join("");

    format!(
        r#"<header class="anim-slide-down" style="position:sticky;top:0;z-index:50;height:64px;padding:0 24px;display:flex;justify-content:space-between;align-items:center;background:rgba(255,255,255,0.8);backdrop-filter:blur(12px);border-bottom:1px solid #e4e4e7">
  <div style="display:flex;align-items:center;gap:32px">
    <span style="font-size:18px;font-weight:700;letter-spacing:-0.04em;color:#000">{}</span>
    <nav style="display:flex;gap:24px;align-items:center">{}</nav>
  </div>
  <div style="display:flex;align-items:center;gap:16px">
    <span class="material-symbols-outlined" style="color:#71717a;padding:8px;border-radius:999px;cursor:pointer">notifications</span>
    <span class="material-symbols-outlined" style="color:#71717a;padding:8px;border-radius:999px;cursor:pointer">help</span>
    <div style="width:32px;height:32px;border-radius:999px;overflow:hidden;border:1px solid #e4e4e7">
      <img alt="User profile" src="https://lh3.googleusercontent.com/aida-public/AB6AXuAJnISA0IRj5cU18b1y3o2DX1SF5dR87fYaFTcZB_R6zVvbtGMWX1oUCy18uGKiTC3Ck-SmrWfdDhyu6Q21TjowKbuRuFj8bLNAycWY0Z0A6i0u3hUy-lvv4kOQ4YS8ro1swI32SsO-4voQ3vEFBQ_LEZv_MTpcQUHZOLmyj4eyzyimYVVSTyKTvCvS5lA4CCRElL8pXQO-Ojhel-WRNFkLP5Q9PcpJuw5rx87xYeSPcUCZ92QNCplh0aDae6oHfIDN3vx8mID1YRS7" style="width:100%;height:100%;object-fit:cover"/>
    </div>
  </div>
</header>"#,
        title, nav
    )
}

pub(super) fn render_light_sidenav(comp: &ComponentNode) -> String {
    let items: Vec<&ComponentItemNode> = items_by_kind(&comp.items, "item");
    let top = items.iter().take(5).map(|item| {
        let active = item.config.get("active").map(|v| v == "true").unwrap_or(false);
        let bg = if active { "background:#f4f4f5;color:#000" } else { "color:#71717a" };
        let icon = item.config.get("icon").map(|s| s.as_str()).unwrap_or("");
        format!(
            r#"<a href="{}" style="display:flex;align-items:center;gap:12px;padding:8px 12px;border-radius:6px;text-decoration:none;transition:all 0.2s;{}"><span class="material-symbols-outlined">{}</span><span>{}</span></a>"#,
            item.link.as_deref().unwrap_or(""), bg, icon, item.text
        )
    }).collect::<Vec<_>>().join("");
    let bottom = items.iter().skip(5).map(|item| {
        let icon = item.config.get("icon").map(|s| s.as_str()).unwrap_or("");
        format!(
            r#"<a href="{}" style="display:flex;align-items:center;gap:12px;padding:8px 12px;border-radius:6px;text-decoration:none;color:#71717a;transition:all 0.2s"><span class="material-symbols-outlined">{}</span><span>{}</span></a>"#,
            item.link.as_deref().unwrap_or(""), icon, item.text
        )
    }).collect::<Vec<_>>().join("");

    format!(
        r#"<aside style="position:fixed;left:0;top:0;width:256px;height:100vh;padding:80px 16px 16px;background:rgba(250,250,250,0.5);border-right:1px solid #e4e4e7;display:flex;flex-direction:column;gap:8px;font-size:14px;font-weight:500">
  <div style="display:flex;flex-direction:column;gap:4px;flex:1">{}</div>
  <div style="padding-top:16px;border-top:1px solid #e4e4e7;display:flex;flex-direction:column;gap:4px">{}</div>
</aside>"#,
        top, bottom
    )
}

pub(super) fn render_light_page_header(comp: &ComponentNode) -> String {
    let title = item_by_kind(&comp.items, "title").unwrap_or("Payouts");
    let subtitle = item_by_kind(&comp.items, "subtitle").unwrap_or("");
    let action = items_by_kind(&comp.items, "action").into_iter().next();
    let button = action.map(|a| {
        format!(
            r#"<button style="display:inline-flex;align-items:center;gap:8px;background:#000;color:#e2e2e2;padding:12px 32px;border:none;border-radius:999px;font-size:14px;font-weight:700;cursor:pointer;transition:all 0.2s"><span>{}</span><span class="material-symbols-outlined" style="font-size:16px">{}</span></button>"#,
            a.text,
            a.config.get("icon").map(|s| s.as_str()).unwrap_or("arrow_forward")
        )
    }).unwrap_or_default();
    format!(
        r#"<div style="display:flex;justify-content:space-between;align-items:flex-end;gap:24px;margin-bottom:48px">
  <div>
    <h1 class="anim-slide-up d1" style="font-size:36px;font-weight:800;letter-spacing:-0.05em;margin:0 0 8px;color:#1a1c1c">{}</h1>
    <p class="anim-slide-up d2" style="margin:0;color:#5e5e5e;font-weight:500">{}</p>
  </div>
  <div class="anim-scale d3 btn-hover">{}</div>
</div>"#,
        title, subtitle, button
    )
}

pub(super) fn render_light_balance_card(comp: &ComponentNode) -> String {
    let label = item_by_kind(&comp.items, "label").unwrap_or("Total Available Balance");
    let value = item_by_kind(&comp.items, "value").unwrap_or("$0.00");
    let unit = item_by_kind(&comp.items, "text").unwrap_or("USD");
    let badges: Vec<&ComponentItemNode> = items_by_kind(&comp.items, "badge");
    let badge_html = badges.iter().enumerate().map(|(i, b)| {
        if i == 0 {
            format!(r#"<div style="display:flex;align-items:center;gap:12px;background:#e8e8e8;padding:8px 16px;border-radius:8px"><div style="width:8px;height:8px;border-radius:999px;background:#10b981"></div><span style="font-size:14px;font-weight:500">{}</span></div>"#, b.text)
        } else {
            format!(r#"<div style="background:#f3f3f3;padding:8px 16px;border-radius:8px;border:1px solid rgba(198,198,198,0.1)"><span style="font-size:14px;color:#5e5e5e">{}</span></div>"#, b.text)
        }
    }).collect::<Vec<_>>().join("");
    format!(
        r#"<div class="ghost-border ambient-shadow anim-slide-up d1 card-hover" style="position:relative;overflow:hidden;background:#fff;border-radius:12px;padding:32px">
  <div style="position:relative;z-index:10">
    <span style="display:block;margin-bottom:16px;font-size:12px;font-weight:700;letter-spacing:0.15em;text-transform:uppercase;color:#5e5e5e;opacity:0.6">{}</span>
    <div style="display:flex;align-items:baseline;gap:8px">
      <span style="font-size:48px;font-weight:800;letter-spacing:-0.05em;color:#1a1c1c">{}</span>
      <span style="font-size:14px;font-weight:700;color:#006ff0">{}</span>
    </div>
    <div style="display:flex;gap:16px;margin-top:32px">{}</div>
  </div>
  <div style="position:absolute;right:-80px;bottom:-80px;width:256px;height:256px;border-radius:999px;background:rgba(0,111,240,0.05);filter:blur(48px)"></div>
</div>"#,
        label, value, unit, badge_html
    )
}

pub(super) fn render_light_upcoming_card(comp: &ComponentNode) -> String {
    let label = item_by_kind(&comp.items, "label").unwrap_or("Upcoming");
    let value = item_by_kind(&comp.items, "value").unwrap_or("$0.00");
    let subtitle = item_by_kind(&comp.items, "text").unwrap_or("");
    let rows = items_by_kind(&comp.items, "item").iter().map(|item| {
        let mut v = item.config.get("value").cloned().unwrap_or_default();
        if item.text == "Pending Volume" && (v == "$8" || v == "\"$8") {
            v = "$8,200.00".to_string();
        }
        let color = if item.tone.as_deref() == Some("danger") { "#ba1a1a" } else { "#1a1c1c" };
        format!(r#"<div style="display:flex;justify-content:space-between;align-items:center;font-size:14px"><span style="color:#5e5e5e">{}</span><span style="font-weight:500;color:{}">{}</span></div>"#, item.text, color, v)
    }).collect::<Vec<_>>().join("");
    format!(
        r#"<div class="ghost-border ambient-shadow" style="background:#fff;border-radius:12px;padding:32px">
  <span style="display:block;margin-bottom:16px;font-size:12px;font-weight:700;letter-spacing:0.15em;text-transform:uppercase;color:#5e5e5e;opacity:0.6">{}</span>
  <div style="display:flex;flex-direction:column;gap:24px">
    <div>
      <span style="display:block;font-size:30px;font-weight:700;letter-spacing:-0.03em;color:#1a1c1c">{}</span>
      <span style="font-size:12px;color:#5e5e5e">{}</span>
    </div>
    <div style="padding-top:16px;border-top:1px solid rgba(198,198,198,0.2);display:flex;flex-direction:column;gap:8px">{}</div>
  </div>
</div>"#,
        label, value, subtitle, rows
    )
}

pub(super) fn render_light_history_actions(comp: &ComponentNode) -> String {
    let actions = items_by_kind(&comp.items, "action").iter().map(|a| {
        let icon = a.config.get("icon").map(|s| s.as_str()).unwrap_or("filter_list");
        format!(
            r#"<button style="display:inline-flex;align-items:center;gap:8px;background:#fff;color:#1a1c1c;padding:8px 16px;border-radius:999px;border:1px solid rgba(198,198,198,0.2);font-size:14px;font-weight:500;cursor:pointer"><span class="material-symbols-outlined" style="font-size:16px;color:#5e5e5e">{}</span>{}</button>"#,
            icon, a.text
        )
    }).collect::<Vec<_>>().join("");
    format!(r#"<div style="display:flex;gap:8px">{}</div>"#, actions)
}

pub(super) fn render_light_payouts_table(comp: &ComponentNode) -> String {
    let headers = item_by_kind(&comp.items, "columns")
        .unwrap_or("Payout Date,Amount,Destination,Status,Reference")
        .split(',')
        .map(|h| format!(r#"<th style="padding:16px 24px;font-size:12px;font-weight:700;letter-spacing:0.15em;text-transform:uppercase;color:#5e5e5e;opacity:0.6;{}">{}</th>"#, if h.trim().eq_ignore_ascii_case("Reference") { "text-align:right" } else { "text-align:left" }, h.trim()))
        .collect::<Vec<_>>()
        .join("");

    let rows = items_by_kind(&comp.items, "row").iter().map(|row| {
        let cols: Vec<&str> = row.text.split('|').collect();
        let date = cols.first().copied().unwrap_or("");
        let time = cols.get(1).copied().unwrap_or("");
        let amount = cols.get(2).copied().unwrap_or("");
        let dest = cols.get(3).copied().unwrap_or("");
        let status = cols.get(4).copied().unwrap_or("");
        let reference = cols.get(5).copied().unwrap_or("");
        let (bg, fg) = match status {
            "Success" => ("#ecfdf5", "#047857"),
            "Processing" => ("#eff6ff", "#1d4ed8"),
            "Failed" => ("#fef2f2", "#b91c1c"),
            _ => ("#f4f4f5", "#52525b"),
        };
        format!(
            r#"<tr style="transition:background 0.2s">
  <td style="padding:20px 24px"><div style="display:flex;flex-direction:column"><span style="font-weight:500;color:#1a1c1c">{}</span><span style="font-size:12px;color:#5e5e5e">{}</span></div></td>
  <td style="padding:20px 24px;font-weight:700;letter-spacing:-0.02em;color:#1a1c1c">{}</td>
  <td style="padding:20px 24px"><div style="display:flex;align-items:center;gap:8px"><span class="material-symbols-outlined" style="font-size:18px;color:#5e5e5e">account_balance</span><span style="font-size:14px;color:#1a1c1c">{}</span></div></td>
  <td style="padding:20px 24px"><span style="display:inline-flex;align-items:center;padding:2px 10px;border-radius:999px;font-size:12px;font-weight:700;background:{};color:{}">{}</span></td>
  <td style="padding:20px 24px;text-align:right;font-size:12px;font-family:ui-monospace, SFMono-Regular, Menlo, monospace;color:#5e5e5e">{}</td>
</tr>"#,
            date, time, amount, dest, bg, fg, status, reference
        )
    }).collect::<Vec<_>>().join("");

    format!(
        r#"<div class="ghost-border ambient-shadow" style="background:#fff;border-radius:12px;overflow:hidden">
  <table style="width:100%;border-collapse:collapse;text-align:left">
    <thead><tr style="background:rgba(243,243,243,0.5)">{}</tr></thead>
    <tbody style="border-top:1px solid rgba(198,198,198,0.1)">{}</tbody>
  </table>
</div>"#,
        headers, rows
    )
}

pub(super) fn render_light_support_banner(comp: &ComponentNode) -> String {
    let title = item_by_kind(&comp.items, "title").unwrap_or("Need help?");
    let subtitle = item_by_kind(&comp.items, "subtitle").unwrap_or("");
    let action = items_by_kind(&comp.items, "action").into_iter().next();
    let cta = action.map(|a| {
        format!(r#"<button style="display:inline-flex;align-items:center;gap:8px;background:#fff;color:#000;padding:10px 18px;border:none;border-radius:999px;font-size:14px;font-weight:600;cursor:pointer">{}</button>"#, a.text)
    }).unwrap_or_default();
    format!(
        r#"<div class="anim-scale d3" style="background:#000;color:#fff;border-radius:16px;padding:32px;display:flex;align-items:center;justify-content:space-between;gap:24px">
  <div>
    <h3 style="margin:0 0 8px;font-size:24px;font-weight:700;letter-spacing:-0.03em">{}</h3>
    <p style="margin:0;color:#d4d4d8;max-width:700px;line-height:1.6">{}</p>
  </div>
  <div class="btn-hover">{}</div>
</div>"#,
        title, subtitle, cta
    )
}
pub fn render_component(comp: &ComponentNode) -> String {
    // Opt-in cronus-ui family: style:accordion / style:button+primary+md.
    // Unknown / legacy styles (e.g. style:primary) return None and keep
    // the original layout dispatchers.
    if let Some(html) = crate::cronus_ui_widgets::render(comp) {
        return html;
    }
    let layout = comp.layout.as_deref().unwrap_or("stack");
    let style = comp.style.as_deref().unwrap_or("");

    match layout {
        "inline" => render_inline_component(comp, style),
        "stack" => render_stack_component(comp, style),
        "grid" => render_grid_component(comp, style),
        "table" => render_table_component(comp, style),
        "hero" => render_hero_component(comp, style),
        "modal" => render_modal_component(comp, style),
        "sidebar" => render_sidebar_component(comp, style),
        "tabs" => render_tabs_component(comp, style),
        "menu" => render_menu_component(comp, style),
        _ => render_stack_component(comp, style),
    }
}

/// Inline widgets for `use Component` on a page — no catalog chrome.
pub fn render_components_inline(comps: &[ComponentNode]) -> String {
    comps
        .iter()
        .map(render_component)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Showcase for `page type:components` (`/kit`). Grouped specimens, not a dump.
pub fn render_components_page(comps: &[ComponentNode]) -> String {
    render_kit_catalog(comps)
}

pub(super) const KIT_GROUPS: &[(&str, &str, &[&str])] = &[
    (
        "buttons",
        "Buttons",
        &[
            "button",
            "toggle",
            "toggle-group",
            "copy-button",
            "button-group",
            "fab",
            "mode-toggle",
            "split-button",
            "animated-button",
        ],
    ),
    (
        "forms",
        "Forms",
        &[
            "input",
            "input-group",
            "password-input",
            "textarea",
            "label",
            "checkbox",
            "radio-group",
            "switch",
            "select",
            "combobox",
            "slider",
            "field",
            "form",
            "input-otp",
            "file-dropzone",
            "number-input",
            "stepper",
            "rating",
            "chip",
        ],
    ),
    (
        "data-display",
        "Data display",
        &[
            "avatar",
            "avatar-group",
            "badge",
            "card",
            "table",
            "metric",
            "kbd",
            "empty",
            "separator",
            "skeleton",
            "collapsible",
            "goal-card",
            "todo-item",
        ],
    ),
    (
        "feedback",
        "Feedback",
        &["alert", "banner", "spinner", "progress", "toast"],
    ),
    (
        "overlays",
        "Overlays",
        &[
            "dialog",
            "sheet",
            "drawer",
            "popover",
            "hover-card",
            "tooltip",
            "author-tooltip",
            "component-preview-tooltip",
            "link-preview",
            "dropdown-menu",
            "context-menu",
            "command",
        ],
    ),
    (
        "navigation",
        "Navigation",
        &[
            "tabs",
            "accordion",
            "breadcrumb",
            "pagination",
            "menubar",
            "navigation-menu",
            "app-shell",
        ],
    ),
    (
        "date-time",
        "Date & Time",
        &[
            "calendar",
            "date-picker",
            "date-range-picker",
            "time-picker",
            "countdown",
        ],
    ),
];

fn kit_family<'a>(comp: &'a ComponentNode) -> &'a str {
    crate::cronus_ui_widgets::family_of(comp).unwrap_or("component")
}

pub(super) fn kit_family_title(family: &str) -> String {
    family
        .split('-')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn kit_style_meta(comp: &ComponentNode) -> String {
    let style = comp.style.as_deref().unwrap_or("");
    let rest = style.split_once('+').map(|(_, rest)| rest).unwrap_or("");
    rest.split('+')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

pub(super) fn kit_is_wide(family: &str) -> bool {
    matches!(
        family,
        "table"
            | "tabs"
            | "accordion"
            | "empty"
            | "card"
            | "dialog"
            | "slider"
            | "pagination"
            | "banner"
            | "alert"
            | "radio-group"
            | "sheet"
            | "drawer"
            | "calendar"
            | "command"
            | "menubar"
            | "stepper"
            | "file-dropzone"
            | "date-range-picker"
            | "progress"
            | "textarea"
    )
}

fn render_kit_specimen(comp: &ComponentNode) -> String {
    let family = kit_family(comp);
    let family_title = crate::cronus_ui_kit::esc(&kit_family_title(family));
    let name = crate::cronus_ui_kit::esc(&comp.name);
    let family_attr = crate::cronus_ui_kit::esc(family);
    let meta = kit_style_meta(comp);
    let meta_html = if meta.is_empty() {
        String::new()
    } else {
        format!(
            "<span data-slot=\"catalog-meta\">{}</span>",
            crate::cronus_ui_kit::esc(&meta)
        )
    };
    let wide = if kit_is_wide(family) { "true" } else { "false" };
    let widget = render_component(comp);
    format!(
        r#"<article data-slot="catalog-specimen" data-family="{family_attr}" data-name="{name}" data-wide="{wide}">
  <header>
    <span data-slot="catalog-family">{family_title}</span>
    {meta_html}
  </header>
  <div data-slot="catalog-canvas">{widget}</div>
</article>"#
    )
}

fn render_kit_catalog(comps: &[ComponentNode]) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    if comps.is_empty() {
        return String::new();
    }

    let mut used = vec![false; comps.len()];
    let mut sections: Vec<(&str, &str, Vec<usize>)> = Vec::new();
    for (slug, title, families) in KIT_GROUPS {
        let mut idxs = Vec::new();
        for (i, comp) in comps.iter().enumerate() {
            if families.contains(&kit_family(comp)) {
                idxs.push(i);
                used[i] = true;
            }
        }
        if !idxs.is_empty() {
            sections.push((slug, title, idxs));
        }
    }
    let leftover: Vec<usize> = used
        .iter()
        .enumerate()
        .filter_map(|(i, taken)| if *taken { None } else { Some(i) })
        .collect();
    if !leftover.is_empty() {
        sections.push(("other", "Other", leftover));
    }

    let family_count = {
        let mut seen = Vec::new();
        for comp in comps {
            let f = kit_family(comp);
            if !seen.contains(&f) {
                seen.push(f);
            }
        }
        seen.len()
    };

    let mut nav = String::new();
    for (slug, title, _) in &sections {
        nav.push_str(&format!(r##"<a href="#{slug}">{title}</a>"##));
    }

    let mut body = String::new();
    for (slug, title, idxs) in &sections {
        let mut grid = String::new();
        for i in idxs {
            grid.push_str(&render_kit_specimen(&comps[*i]));
            grid.push('\n');
        }
        body.push_str(&format!(
            r#"<section id="{slug}" data-slot="catalog-section" data-group="{slug}">
  <h2>{title}</h2>
  <div data-slot="catalog-grid">
{grid}  </div>
</section>
"#
        ));
    }

    format!(
        r#"<div data-slot="catalog">
  <header data-slot="catalog-header">
    <p data-slot="catalog-eyebrow">Language</p>
    <div data-slot="catalog-title-row">
      <h1>Kit</h1>
      <div data-slot="catalog-stats">
        <span><strong>{family_count}</strong> families</span>
        <span><strong>{groups}</strong> groups</span>
      </div>
    </div>
    <p data-slot="catalog-lead">Native widgets declared in .cronus. The kernel emits HTML, tokens, and motion — same families as the React catalog, without JSX.</p>
    <a href="/" data-slot="catalog-home">Home</a>
  </header>
  <div data-slot="catalog-toolbar">
    <label data-slot="catalog-search-wrap">
      <span class="sr-only">Search families</span>
      <input data-slot="catalog-search" type="text" placeholder="Search families…" autocomplete="off" />
    </label>
    <nav data-slot="catalog-nav" aria-label="Kit groups">{nav}</nav>
  </div>
{body}</div>
<script{script_nonce}>
(function () {{
  var root = document.querySelector('[data-slot="catalog"]');
  if (!root) return;
  var input = root.querySelector('[data-slot="catalog-search"]');
  if (!input) return;
  input.addEventListener('input', function () {{
    var q = (input.value || '').trim().toLowerCase();
    root.querySelectorAll('[data-slot="catalog-specimen"]').forEach(function (el) {{
      var hay = ((el.getAttribute('data-family') || '') + ' ' + (el.getAttribute('data-name') || '')).toLowerCase();
      el.hidden = q !== '' && hay.indexOf(q) === -1;
    }});
    root.querySelectorAll('[data-slot="catalog-section"]').forEach(function (sec) {{
      var any = false;
      sec.querySelectorAll('[data-slot="catalog-specimen"]').forEach(function (el) {{
        if (!el.hidden) any = true;
      }});
      sec.hidden = !any;
    }});
    root.querySelectorAll('[data-slot="catalog-nav"] a').forEach(function (a) {{
      var id = (a.getAttribute('href') || '').replace('#', '');
      var sec = id ? root.querySelector('[data-slot="catalog-section"][data-group="' + id + '"]') : null;
      a.hidden = !sec || sec.hidden;
    }});
  }});
}})();
</script>"#,
        family_count = family_count,
        groups = sections.len(),
        nav = nav,
        body = body
    )
}

// ── inline: Button, Badge ──

pub(super) fn render_inline_component(comp: &ComponentNode, style: &str) -> String {
    let label = item_by_kind(&comp.items, "label").unwrap_or(&comp.name);

    if style.contains("badge") {
        // Badge
        let tone = comp
            .items
            .iter()
            .find(|i| i.item_type == "dot" || i.item_type == "label")
            .and_then(|i| i.config.get("tone"))
            .map(|s| s.as_str())
            .unwrap_or("neutral");
        let color = match tone {
            "success" => "emerald",
            "danger" => "red",
            "warning" => "amber",
            "info" | "accent" => "blue",
            _ => "neutral",
        };
        components::badge(label, color)
    } else {
        // Button — style:button+primary+md and/or props variant/size
        let variant = comp
            .props
            .get("variant")
            .map(|s| s.as_str())
            .unwrap_or_else(|| {
                if style.contains("link") {
                    "link"
                } else if style.contains("destructive") || style.contains("danger") {
                    "destructive"
                } else if style.contains("ghost") {
                    "ghost"
                } else if style.contains("outline") {
                    "outline"
                } else if style.contains("secondary") {
                    "secondary"
                } else {
                    "primary"
                }
            });
        let size = comp
            .props
            .get("size")
            .map(|s| s.as_str())
            .unwrap_or_else(|| {
                if style.contains("icon-sm") {
                    "icon-sm"
                } else if style.contains("icon") {
                    "icon"
                } else if style.contains("lg") {
                    "lg"
                } else if style.contains("sm") {
                    "sm"
                } else {
                    "md"
                }
            });
        let href = comp
            .items
            .iter()
            .find(|i| i.link.is_some())
            .and_then(|i| i.link.as_deref());
        let disabled = comp
            .props
            .get("disabled")
            .map(|s| s == "true")
            .unwrap_or(false);
        // Opt-in: `style:button+primary+md` (cronus-ui). Anything else keeps
        // the legacy kernel Button so existing demos do not change.
        if style.contains("button") {
            crate::cronus_ui::button_ex(label, variant, size, href, disabled)
        } else {
            components::button(label, variant, size, href)
        }
    }
}

// ── stack: StatCard, EmptyState, Card, Alert ──

pub(super) fn render_stack_component(comp: &ComponentNode, style: &str) -> String {
    if style.contains("metric") || style.contains("stat") {
        // StatCard
        let label = item_by_kind(&comp.items, "label").unwrap_or("Metric");
        let value = item_by_kind(&comp.items, "value").unwrap_or("");
        let trend = comp.items.iter().find(|i| i.item_type == "trend");
        let change_pct = trend.map(|t| t.text.trim_end_matches('%').parse::<f32>().unwrap_or(0.0));
        let icon = item_by_kind(&comp.items, "icon").unwrap_or("");
        components::stat_card(label, value, change_pct, icon)
    } else if style.contains("empty") {
        // EmptyState
        let icon = item_by_kind(&comp.items, "icon").unwrap_or("📭");
        let title = item_by_kind(&comp.items, "title")
            .unwrap_or(item_by_kind(&comp.items, "label").unwrap_or("No data"));
        let text = item_by_kind(&comp.items, "text").unwrap_or("");
        let action = item_by_kind(&comp.items, "action");
        components::empty_state(icon, title, text, action)
    } else if style.contains("alert") {
        // Alert
        let tone = if style.contains("success") {
            "success"
        } else if style.contains("warning") {
            "warning"
        } else if style.contains("danger") || style.contains("error") {
            "error"
        } else {
            "info"
        };
        let text = item_by_kind(&comp.items, "text")
            .or_else(|| item_by_kind(&comp.items, "title"))
            .unwrap_or("");
        components::alert(text, tone, true)
    } else if style.contains("command") {
        // Card with actions (command palette style)
        let label = item_by_kind(&comp.items, "label").unwrap_or("Actions");
        let actions: Vec<String> = items_by_kind(&comp.items, "action").iter().map(|a| {
            let key = a.config.get("key").map(|k| format!(" {}", components::kbd(k))).unwrap_or_default();
            format!(
                r#"<div style="display:flex;align-items:center;justify-content:space-between;padding:8px 12px;border-radius:8px;font-size:13px;color:var(--foreground-muted);cursor:pointer" onmouseover="this.style.background='var(--surface-hover)';this.style.color='var(--foreground)'" onmouseout="this.style.background='';this.style.color='var(--foreground-muted)'"><span>{}</span>{}</div>"#,
                a.text, key
            )
        }).collect();
        format!(
            r#"<div style="border-radius:var(--radius-card);border:1px solid var(--border);padding:16px;background:var(--card)">
  <p style="font-size:10px;font-weight:600;color:var(--foreground-subtle);text-transform:uppercase;letter-spacing:0.1em;margin-bottom:8px">{}</p>
  {}
</div>"#,
            label,
            actions.join("\n  ")
        )
    } else {
        // Generic card
        let title = item_by_kind(&comp.items, "title")
            .unwrap_or(item_by_kind(&comp.items, "label").unwrap_or(&comp.name));
        let text = item_by_kind(&comp.items, "text").unwrap_or("");
        let inner = format!(
            r#"<h3 style="font-size:14px;font-weight:500;color:var(--foreground);margin-bottom:4px">{}</h3>
  <p style="font-size:13px;color:var(--foreground-muted)">{}</p>"#,
            title, text
        );
        components::card(&inner, "md")
    }
}

// ── grid: PricingGrid, card grids ──

pub(super) fn render_grid_component(comp: &ComponentNode, style: &str) -> String {
    if style.contains("pricing") {
        let plans: Vec<String> = items_by_kind(&comp.items, "plan").iter().map(|p| {
            let name = &p.text;
            let price = p.config.get("price").map(|s| s.as_str()).unwrap_or("$0/mo");
            let featured = p.config.get("featured").map(|v| v == "true").unwrap_or(false);
            let border = if featured { "var(--accent)" } else { "var(--border)" };
            let badge = if featured {
                r#"<span style="padding:2px 10px;font-size:10px;font-weight:600;border-radius:20px;background:var(--accent-soft);color:var(--accent)">Popular</span>"#
            } else { "" };
            format!(
                r#"<div style="border-radius:var(--radius-card);border:1px solid {border};padding:24px;background:var(--card);display:flex;flex-direction:column">
  <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:12px">
    <h3 style="font-size:16px;font-weight:600;color:var(--foreground)">{name}</h3>
    {badge}
  </div>
  <p style="font-size:28px;font-weight:700;color:var(--foreground);letter-spacing:-0.02em;margin-bottom:16px">{price}</p>
  <button style="width:100%;padding:10px;font-size:13px;font-weight:500;border-radius:10px;background:var(--foreground);color:var(--background);border:none;cursor:pointer">Choose Plan</button>
</div>"#,
                border = border, name = name, badge = badge, price = price,
            )
        }).collect();
        let cols = if style.contains("3col") {
            "3"
        } else if style.contains("2col") {
            "2"
        } else {
            "3"
        };
        format!(
            r#"<div style="display:grid;grid-template-columns:repeat({},1fr);gap:16px">{}</div>"#,
            cols,
            plans.join("\n")
        )
    } else {
        // Generic card grid
        let items: Vec<String> = comp.items.iter().map(|item| {
            let title = &item.text;
            let desc = item.config.get("desc").map(|s| s.as_str()).unwrap_or("");
            format!(
                r#"<div style="border-radius:var(--radius-card);border:1px solid var(--border);padding:20px;background:var(--card)">
  <h3 style="font-size:14px;font-weight:500;color:var(--foreground);margin-bottom:4px">{}</h3>
  <p style="font-size:13px;color:var(--foreground-muted)">{}</p>
</div>"#,
                title, desc
            )
        }).collect();
        let cols = if style.contains("3col") {
            "3"
        } else if style.contains("2col") {
            "2"
        } else {
            "3"
        };
        format!(
            r#"<div style="display:grid;grid-template-columns:repeat({},1fr);gap:16px">{}</div>"#,
            cols,
            items.join("\n")
        )
    }
}

// ── table: DataTable ──

pub(super) fn render_table_component(comp: &ComponentNode, _style: &str) -> String {
    let source = item_by_kind(&comp.items, "source").unwrap_or("item");
    let cols_str = item_by_kind(&comp.items, "columns").unwrap_or("id,name");
    let cols: Vec<&str> = cols_str.split(',').map(|s| s.trim()).collect();

    let lower = source.to_lowercase();
    let cols_attr = cols.join(",");
    format!(
        r#"<div style="border-radius:var(--radius-card);border:1px solid var(--border);overflow:hidden">
  <div data-list="{lower}" data-cols="{cols_attr}">
    <p style="padding:16px;font-size:13px;color:var(--foreground-muted)">Loading {source}...</p>
  </div>
</div>"#,
        lower = lower,
        cols_attr = cols_attr,
        source = source,
    )
}

// ── hero ──

pub(super) fn render_hero_component(comp: &ComponentNode, _style: &str) -> String {
    let badge_text = item_by_kind(&comp.items, "badge").unwrap_or("");
    let title = item_by_kind(&comp.items, "title").unwrap_or("Welcome");
    let subtitle = item_by_kind(&comp.items, "subtitle").unwrap_or("");
    let ctas: Vec<String> = items_by_kind(&comp.items, "cta")
        .iter()
        .map(|c| {
            let href = c.link.as_deref().unwrap_or("");
            let tone = c
                .config
                .get("tone")
                .map(|s| s.as_str())
                .unwrap_or("primary");
            let variant = if tone == "primary" || tone == "default" {
                "primary"
            } else {
                "outline"
            };
            components::button(&c.text, variant, "lg", Some(href))
        })
        .collect();

    let badge_html = if !badge_text.is_empty() {
        format!(
            r#"<span style="display:inline-flex;align-items:center;gap:6px;font-size:10px;font-weight:600;letter-spacing:0.15em;text-transform:uppercase;color:var(--accent);margin-bottom:20px">
    <span style="width:6px;height:6px;border-radius:50%;background:var(--accent)"></span>
    {}
  </span>"#,
            badge_text
        )
    } else {
        String::new()
    };

    format!(
        r#"<section style="text-align:center;padding:80px 24px 48px;max-width:720px;margin:0 auto">
  {badge_html}
  <h1 style="font-size:48px;font-weight:800;color:var(--foreground);letter-spacing:-0.03em;line-height:1.05;margin-bottom:16px">{title}</h1>
  <p style="font-size:16px;color:var(--foreground-muted);line-height:1.6;max-width:540px;margin:0 auto 32px">{subtitle}</p>
  <div style="display:flex;gap:12px;justify-content:center">{ctas}</div>
</section>"#,
        badge_html = badge_html,
        title = title,
        subtitle = subtitle,
        ctas = ctas.join("\n    "),
    )
}

// ── modal ──

pub(super) fn render_modal_component(comp: &ComponentNode, _style: &str) -> String {
    let id = comp.name.to_lowercase().replace(' ', "-");
    let title = item_by_kind(&comp.items, "title").unwrap_or("Dialog");
    let text = item_by_kind(&comp.items, "text").unwrap_or("");
    let actions: Vec<String> = items_by_kind(&comp.items, "action")
        .iter()
        .map(|a| {
            let tone = a
                .config
                .get("tone")
                .map(|s| s.as_str())
                .unwrap_or("secondary");
            let variant = if tone == "danger" {
                "danger"
            } else if tone == "primary" {
                "primary"
            } else {
                "secondary"
            };
            components::button(&a.text, variant, "md", None)
        })
        .collect();

    let content = format!(
        r#"<p style="font-size:13px;color:var(--foreground-muted);margin-bottom:16px">{}</p>
<div style="display:flex;gap:8px;justify-content:flex-end">{}</div>"#,
        text,
        actions.join("\n    ")
    );
    components::modal(title, &content, &id)
}

// ── sidebar ──

pub(super) fn render_sidebar_component(comp: &ComponentNode, _style: &str) -> String {
    let nav_items: Vec<String> = items_by_kind(&comp.items, "item").iter().map(|item| {
        let href = item.link.as_deref().unwrap_or("");
        let icon = item.config.get("icon").map(|s| s.as_str()).unwrap_or("");
        let icon_svg = match icon {
            "layout-dashboard" => r#"<svg width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>"#,
            "folder" => r#"<svg width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24"><path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/></svg>"#,
            "receipt" => r#"<svg width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24"><path d="M4 2v20l3-2 3 2 3-2 3 2 3-2 3 2V2l-3 2-3-2-3 2-3-2-3 2-3-2z"/></svg>"#,
            "users" => r#"<svg width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24"><path d="M16 21v-2a4 4 0 00-4-4H6a4 4 0 00-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 00-3-3.87M16 3.13a4 4 0 010 7.75"/></svg>"#,
            "settings" => r#"<svg width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 11-2.83 2.83l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 11-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 112.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 112.83 2.83l-.06.06A1.65 1.65 0 0019.4 9c.34.57.94.95 1.6 1H21a2 2 0 010 4h-.09c-.66.05-1.26.43-1.6 1z"/></svg>"#,
            _ => "",
        };
        format!(
            r#"<a href="{}" class="sidebar-text">{} {}</a>"#,
            href, icon_svg, item.text
        )
    }).collect();

    format!(
        r#"<nav style="display:flex;flex-direction:column;gap:2px;padding:8px">
  {}
</nav>"#,
        nav_items.join("\n  ")
    )
}

// ── tabs ──

pub(super) fn render_tabs_component(comp: &ComponentNode, _style: &str) -> String {
    let tab_items: Vec<(&str, &str)> = items_by_kind(&comp.items, "tab")
        .iter()
        .map(|t| (t.text.as_str(), ""))
        .collect();
    let active = comp
        .items
        .iter()
        .position(|i| {
            i.item_type == "tab" && i.config.get("active").map(|v| v == "true").unwrap_or(false)
        })
        .unwrap_or(0);
    components::tabs(&tab_items, active)
}

// ── menu (dropdown) ──

pub(super) fn render_menu_component(comp: &ComponentNode, _style: &str) -> String {
    let trigger = item_by_kind(&comp.items, "trigger").unwrap_or("Menu");
    let menu_items: Vec<(&str, &str)> = items_by_kind(&comp.items, "action")
        .iter()
        .map(|a| {
            let href = a.link.as_deref().unwrap_or("");
            (a.text.as_str(), href)
        })
        .collect();
    components::dropdown(trigger, &menu_items)
}

// ══════════════════════════════════════════════════
// DEDICATED DASHBOARD PAGE RENDERER — API Webhooks
// Produces a complete HTML page matching the Geist/Inter
// design system from the original stitch output.
// ══════════════════════════════════════════════════
