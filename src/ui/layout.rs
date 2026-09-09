//! Layout rendering functions — wraps page content in full HTML documents.

use crate::parser::{LayoutNode, PageNode, StyleNode};
use super::{CRONUS_ANIMATIONS_CSS, CRONUS_ANIMATIONS_JS};

// ══════════════════════════════════════════════════
// LAYOUT (wraps every page)
// ══════════════════════════════════════════════════

pub fn render_layout(app_name: &str, _pages: &[PageNode], _accent: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="pt-BR" data-cronus-theme="aurora" data-cronus-mode="dark">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{app_name}</title>
  <style>{tailwind_css}</style>
  <style>{animations_css}</style>
  <style>{anim_css}</style>
  <style>{cronus_ui_css}</style>
  <style>
    :root {{
      --background: oklch(0.11 0 0);
      --card: oklch(0.14 0 0);
      --card-soft: oklch(0.16 0 0);
      --card-strong: oklch(0.18 0 0);
      --foreground: oklch(0.93 0 0);
      --foreground-muted: oklch(0.5 0 0);
      --foreground-subtle: oklch(0.4 0 0);
      --border: oklch(1 0 0 / 6%);
      --border-strong: oklch(1 0 0 / 8%);
      --surface-hover: oklch(0.18 0 0);
      --secondary: oklch(0.18 0 0);
      --accent: var(--primary);
      --accent-soft: oklch(0.488 0.243 264 / 12%);
      --success: var(--success, #10b981);
      --success-soft: oklch(0.696 0.17 162 / 12%);
      --warning: oklch(0.769 0.188 70);
      --danger: var(--error, #ef4444);
      --danger-soft: oklch(0.704 0.191 22 / 12%);
      --shadow-sm: 0 1px 3px rgba(0,0,0,0.1);
      --radius-card: 22px;
      --radius-button: 10px;
      --radius-badge: 999px;
      --radius: 0.875rem;
    }}
    body {{ background: var(--background); color: var(--foreground); font-family: -apple-system, 'SF Pro Display', 'SF Pro Text', system-ui, sans-serif; -webkit-font-smoothing: antialiased; margin: 0; }}
    ::selection {{ background: oklch(0.3 0 0); }}
    ::-webkit-scrollbar {{ width: 4px; }}
    ::-webkit-scrollbar-thumb {{ background: var(--border); border-radius: 2px; }}
    @keyframes fadeIn {{ from {{ opacity: 0; }} to {{ opacity: 1; }} }}
    .animate-fade-in {{ animation: fadeIn 0.3s ease-out; }}
    .sidebar-icon {{ width:36px;height:36px;border-radius:var(--radius-button);display:flex;align-items:center;justify-content:center;color:var(--foreground-muted);text-decoration:none;transition:all 0.15s }}
    .sidebar-icon:hover {{ background:var(--secondary);color:var(--foreground);transform:scale(1.04) }}
    .sidebar-text {{ display:flex;align-items:center;gap:8px;padding:8px 12px;border-radius:8px;font-size:13px;color:var(--foreground-muted);text-decoration:none;transition:all 0.15s }}
    .sidebar-text:hover {{ background:var(--secondary);color:var(--foreground) }}
  </style>
</head>
<body>
  <div style="display:flex;min-height:100vh">
    <!-- Sidebar (icon-only, 48px) -->
    <aside style="width:48px;flex-shrink:0;display:flex;flex-direction:column;align-items:center;gap:4px;padding:8px 0;border-right:1px solid var(--border);position:fixed;top:0;left:0;bottom:0">
      <a href="/" class="sidebar-icon" style="margin-bottom:8px;background:var(--secondary)" title="{app_name}">
        <span style="font-size:14px;font-weight:700;color:var(--foreground)">C</span>
      </a>
      <div style="width:20px;height:1px;background:var(--border);margin:4px 0"></div>
      <a href="/dashboard" title="Visao Geral" class="sidebar-icon">
        <svg width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>
      </a>
      <a href="/courses" title="Produtos" class="sidebar-icon">
        <svg width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24"><path d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"/></svg>
      </a>
      <a href="/orders" title="Transacoes" class="sidebar-icon">
        <svg width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24"><path d="M12 2v20m5-17H9.5a3.5 3.5 0 000 7h5a3.5 3.5 0 010 7H7"/></svg>
      </a>
      <a href="/analytics" title="Analise" class="sidebar-icon">
        <svg width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24"><path d="M3 3v18h18"/><path d="M7 16l4-4 4 4 6-6"/></svg>
      </a>
      <a href="/affiliates" title="Afiliados" class="sidebar-icon">
        <svg width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24"><path d="M16 21v-2a4 4 0 00-4-4H6a4 4 0 00-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 00-3-3.87M16 3.13a4 4 0 010 7.75"/></svg>
      </a>
      <a href="/webhooks" title="Integracoes" class="sidebar-icon">
        <svg width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/></svg>
      </a>
      <div style="flex:1"></div>
      <div style="width:20px;height:1px;background:var(--border);margin:4px 0"></div>
      <a href="/settings" title="Configuracoes" class="sidebar-icon">
        <svg width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 11-2.83-2.83l-.06.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09a1.65 1.65 0 00-1-1.51 1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 11-2.83-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09a1.65 1.65 0 001.51-1 1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 112.83-2.83l.06.06a1.65 1.65 0 001.82.33 1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 112.83 2.83l-.06.06a1.65 1.65 0 00-.33 1.82 1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/></svg>
      </a>
      <!-- Avatar -->
      <div title="Perfil" style="width:28px;height:28px;border-radius:50%;background:var(--secondary);display:flex;align-items:center;justify-content:center;font-size:11px;font-weight:600;color:var(--foreground-muted);margin:4px 0 8px;cursor:pointer">Z</div>
    </aside>
    <!-- Main -->
    <main style="flex:1;margin-left:48px;padding:6px;min-height:100vh" class="animate-fade-in">
      <div style="border-radius:var(--radius-card);border:1px solid var(--border);background:var(--background);padding:20px;min-height:calc(100vh - 12px);box-shadow:var(--shadow-sm)">
      {body}
      </div>
    </main>
  </div>
  <script>{runtime}</script>
  <script>{animate_js}</script>
  <script>{hmr}</script>
  <script>
    // Active nav state
    var p=window.location.pathname;
    document.querySelectorAll('aside a.sidebar-icon').forEach(function(a){{
      if(a.getAttribute('href')===p||(p==='/'&&a.getAttribute('href')==='/dashboard')){{
        a.style.background='var(--secondary)';
        a.style.color='var(--foreground)';
      }}
    }});
    document.querySelectorAll('a.sidebar-text').forEach(function(a){{
      if(a.getAttribute('href')===p){{
        a.style.background='var(--secondary)';
        a.style.color='var(--foreground)';
      }}
    }});
  </script>
  {anim_js}
  {action_js}
</body>
</html>"#,
        app_name = app_name,
        body = body,
        cronus_ui_css = crate::cronus_ui::token_css("aurora", "dark"),
        tailwind_css = crate::tailwind::CRONUS_TAILWIND,
        animations_css = crate::animations::CRONUS_ANIMATIONS,
        anim_css = CRONUS_ANIMATIONS_CSS,
        anim_js = CRONUS_ANIMATIONS_JS,
        runtime = crate::render::CRONUS_RUNTIME_JS,
        animate_js = crate::animations::CRONUS_ANIMATE_JS,
        hmr = crate::hmr::HMR_CLIENT_JS,
        action_js = crate::runtime_js::CRONUS_ACTION_JS,
    )
}

// ══════════════════════════════════════════════════
// DECLARATIVE LAYOUT (from `layout` block in .cronus)
// ══════════════════════════════════════════════════

pub fn render_layout_declarative(app_name: &str, layout: &LayoutNode, current_route: &str, body: &str) -> String {
    let brand = layout.sidebar_config.get("brand").map(|s| s.as_str()).unwrap_or(app_name);

    // Build nav items HTML with modern design
    let mut nav_html = String::new();
    for item in &layout.sidebar_items {
        if item.is_divider {
            nav_html.push_str(r#"<div class="sb-divider"></div>"#);
            continue;
        }
        let is_active = current_route == item.route
            || (current_route == "/" && item.route == "/dashboard")
            || (item.route != "/" && !item.route.is_empty() && current_route.starts_with(&item.route) && item.route.len() > 1);
        let active_class = if is_active { " active" } else { "" };
        let icon_html = if let Some(ref icon) = item.icon {
            format!(r#"<span class="material-symbols-outlined">{}</span>"#, icon)
        } else {
            String::new()
        };
        nav_html.push_str(&format!(
            r#"<a href="{route}" data-spa="true" class="sb-link{active_class}">{icon}{label}</a>"#,
            route = item.route,
            active_class = active_class,
            icon = icon_html,
            label = item.label,
        ));
    }

    // Topbar
    let search_placeholder = layout.topbar_config.get("search_placeholder").map(|s| s.as_str()).unwrap_or("");
    let has_topbar = !layout.topbar_config.is_empty();
    let topbar_html = if has_topbar {
        let search_html = if !search_placeholder.is_empty() {
            format!(
                r#"<div style="flex:1;max-width:400px">
  <input type="text" placeholder="{}" style="width:100%;padding:8px 12px;border-radius:8px;border:1px solid oklch(1 0 0 / 6%);background:oklch(0.14 0 0);color:oklch(0.93 0 0);font-size:13px;outline:none" onfocus="this.style.borderColor='var(--primary)'" onblur="this.style.borderColor='oklch(1 0 0 / 6%)'">
</div>"#,
                search_placeholder
            )
        } else {
            r#"<div style="flex:1"></div>"#.to_string()
        };
        format!(
            r#"<header style="height:52px;border-bottom:1px solid oklch(1 0 0 / 6%);display:flex;align-items:center;padding:0 24px;flex-shrink:0">
  {search}
</header>"#,
            search = search_html
        )
    } else {
        String::new()
    };

    format!(
        r##"<!DOCTYPE html>
<html lang="en" class="dark" data-cronus-theme="aurora" data-cronus-mode="dark">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta name="view-transition" content="same-origin">
  <title>{app_name}</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap" rel="stylesheet">
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200&display=swap" rel="stylesheet">
  <script src="https://cdn.tailwindcss.com"></script>
  <style>{cronus_ui_css}</style>
  <style>
    *,*::before,*::after{{box-sizing:border-box;margin:0;padding:0}}
    html,body{{background:#000;color:#fff;font-family:'Inter',system-ui,-apple-system,sans-serif;-webkit-font-smoothing:antialiased;min-height:100vh}}
    a{{color:inherit;text-decoration:none}}
    button{{font-family:inherit;cursor:pointer}}
    ::selection{{background:rgba(59,130,246,0.3);color:#fff}}
    ::-webkit-scrollbar{{width:6px;height:6px}}
    ::-webkit-scrollbar-track{{background:transparent}}
    ::-webkit-scrollbar-thumb{{background:rgba(255,255,255,0.08);border-radius:3px}}
    ::-webkit-scrollbar-thumb:hover{{background:rgba(255,255,255,0.15)}}
    .material-symbols-outlined{{font-variation-settings:'FILL' 0,'wght' 400,'GRAD' 0,'opsz' 24;font-size:19px;display:inline-block;line-height:1;vertical-align:middle;user-select:none}}

    /* ── Sidebar ── */
    #cronus-sidebar{{position:fixed;top:0;left:0;width:240px;height:100vh;background:linear-gradient(180deg,#0a0a0a 0%,#050505 100%);border-right:1px solid rgba(255,255,255,0.06);padding:20px;z-index:40;overflow-y:auto;display:flex;flex-direction:column;transition:transform 0.3s cubic-bezier(0.4,0,0.2,1)}}
    .sb-brand{{display:flex;align-items:center;gap:10px;margin-bottom:28px;padding:0 4px}}
    .sb-logo{{width:32px;height:32px;border-radius:8px;background:linear-gradient(135deg,#3b82f6,#a855f7);display:flex;align-items:center;justify-content:center;font-weight:700;color:#fff;font-size:14px;box-shadow:0 4px 20px rgba(59,130,246,0.25)}}
    .sb-name{{font-size:17px;font-weight:700;color:#fff;letter-spacing:-0.02em}}
    .sb-nav{{display:flex;flex-direction:column;gap:2px;flex:1}}
    .sb-link,.sb-link:link,.sb-link:visited,.sb-link:focus,.sb-link:active{{display:flex;align-items:center;gap:12px;padding:10px 12px;border-radius:8px;color:#9ca3af!important;font-size:14px;font-weight:500;text-decoration:none!important;transition:all 0.15s;white-space:nowrap;background:transparent;border:0!important;outline:0!important;-webkit-tap-highlight-color:transparent}}
    .sb-link:hover{{background:rgba(255,255,255,0.03)!important;color:#fff!important}}
    .sb-link:active{{color:#fff!important}}
    .sb-link.active,.sb-link.active:link,.sb-link.active:visited,.sb-link.active:focus,.sb-link.active:active{{background:linear-gradient(90deg,rgba(59,130,246,0.15),rgba(59,130,246,0.03))!important;color:#fff!important;box-shadow:inset 2px 0 0 #3b82f6!important}}
    .sb-link .material-symbols-outlined{{font-size:19px;color:#6b7280}}
    .sb-link:hover .material-symbols-outlined{{color:#9ca3af}}
    .sb-link.active .material-symbols-outlined{{color:#60a5fa}}
    .sb-divider{{height:1px;background:rgba(255,255,255,0.05);margin:12px 0}}
    .sb-footer{{padding-top:16px;border-top:1px solid rgba(255,255,255,0.05);margin-top:16px}}
    .sb-signout{{width:100%;padding:10px 12px;display:flex;align-items:center;justify-content:center;gap:8px;font-size:13px;font-weight:500;color:#9ca3af;background:transparent;border:1px solid rgba(255,255,255,0.08);border-radius:8px;transition:all 0.15s}}
    .sb-signout:hover{{background:rgba(239,68,68,0.08);border-color:rgba(239,68,68,0.3);color:#f87171}}
    .sb-signout .material-symbols-outlined{{font-size:16px}}

    /* ── Hamburger (mobile) ── */
    #sb-toggle{{display:none;position:fixed;top:16px;left:16px;width:40px;height:40px;background:rgba(10,10,10,0.9);backdrop-filter:blur(12px);border:1px solid rgba(255,255,255,0.1);border-radius:10px;color:#fff;z-index:50;align-items:center;justify-content:center}}

    /* ── Main shell ── */
    .cronus-decl-main{{margin-left:240px;min-height:100vh;background:#000;position:relative}}
    .cronus-decl-main::before{{content:'';position:fixed;top:0;left:240px;right:0;height:400px;background:radial-gradient(ellipse 80% 60% at 50% 0%,rgba(59,130,246,0.06),transparent 70%);pointer-events:none;z-index:0}}
    .cronus-decl-main > main{{position:relative;z-index:1;max-width:1400px;margin:0 auto;padding:32px 40px;animation:fadeIn 0.3s ease-out}}

    @keyframes fadeIn{{from{{opacity:0;transform:translateY(4px)}}to{{opacity:1;transform:translateY(0)}}}}

    /* ── Auto-style dev content for visual consistency ── */
    .cronus-decl-main > main h1{{font-size:32px;font-weight:700;color:#fff;letter-spacing:-0.03em;margin-bottom:6px;background:linear-gradient(180deg,#fff 0%,#a1a1aa 100%);-webkit-background-clip:text;-webkit-text-fill-color:transparent;background-clip:text}}
    .cronus-decl-main > main h2{{font-size:20px;font-weight:600;color:#fff;letter-spacing:-0.02em;margin-bottom:16px}}
    .cronus-decl-main > main h3{{font-size:16px;font-weight:600;color:#fff;margin-bottom:12px}}
    .cronus-decl-main > main p{{color:#9ca3af;font-size:14px;line-height:1.6}}
    .cronus-decl-main > main > div:first-child{{margin-bottom:32px}}

    /* ── Tables ── */
    .cronus-decl-main table{{width:100%;border-collapse:collapse;background:rgba(255,255,255,0.02);border:1px solid rgba(255,255,255,0.06);border-radius:14px;overflow:hidden}}
    .cronus-decl-main table thead{{background:rgba(255,255,255,0.02)}}
    .cronus-decl-main table th{{padding:14px 20px;text-align:left;font-size:11px;font-weight:600;color:#6b7280;text-transform:uppercase;letter-spacing:0.08em;border-bottom:1px solid rgba(255,255,255,0.05)}}
    .cronus-decl-main table td{{padding:14px 20px;font-size:14px;color:#d1d5db;border-bottom:1px solid rgba(255,255,255,0.03)}}
    .cronus-decl-main table tr:last-child td{{border-bottom:none}}
    .cronus-decl-main table tr:hover td{{background:rgba(255,255,255,0.02)}}

    /* ── Forms ── */
    .cronus-decl-main form{{background:rgba(255,255,255,0.02);border:1px solid rgba(255,255,255,0.06);border-radius:14px;padding:32px;max-width:560px}}
    .cronus-decl-main form label{{display:block;font-size:13px;font-weight:500;color:#d1d5db;margin-bottom:8px}}
    .cronus-decl-main form input,.cronus-decl-main form select,.cronus-decl-main form textarea{{width:100%;padding:10px 14px;font-size:14px;color:#fff;background:rgba(255,255,255,0.03);border:1px solid rgba(255,255,255,0.08);border-radius:10px;outline:none;margin-bottom:16px;font-family:inherit;transition:border-color 0.15s}}
    .cronus-decl-main form input:focus,.cronus-decl-main form select:focus,.cronus-decl-main form textarea:focus{{border-color:#3b82f6;background:rgba(255,255,255,0.04)}}
    .cronus-decl-main form button[type=submit]{{display:inline-flex;align-items:center;gap:8px;padding:11px 20px;font-size:14px;font-weight:600;color:#fff;background:linear-gradient(135deg,#3b82f6,#2563eb);border:1px solid rgba(59,130,246,0.5);border-radius:10px;box-shadow:0 4px 20px rgba(59,130,246,0.25);transition:all 0.15s}}
    .cronus-decl-main form button[type=submit]:hover{{transform:translateY(-1px);box-shadow:0 6px 28px rgba(59,130,246,0.35)}}

    /* ── Cards / generic sections ── */
    .cronus-decl-main section{{background:rgba(255,255,255,0.02);border:1px solid rgba(255,255,255,0.06);border-radius:14px;padding:24px;margin-bottom:20px}}
    .cronus-decl-main section:has(table){{padding:0;overflow:hidden}}
    .cronus-decl-main section:has(> section){{background:transparent;border:0;padding:0;margin:0}}

    /* ── Buttons ── */
    .cronus-decl-main a[href^="/"]:not([class]){{display:inline-flex;align-items:center;gap:6px;padding:8px 14px;font-size:13px;font-weight:500;color:#fff;background:rgba(255,255,255,0.04);border:1px solid rgba(255,255,255,0.08);border-radius:8px;transition:all 0.15s}}
    .cronus-decl-main a[href^="/"]:not([class]):hover{{background:rgba(255,255,255,0.06);border-color:rgba(255,255,255,0.12)}}

    /* ── KPI cards from built-in renderer ── */
    .cronus-decl-main [data-section="kpi"],.cronus-decl-main .kpi-card-wrapper{{background:linear-gradient(135deg,rgba(255,255,255,0.03),rgba(255,255,255,0.01))!important;border:1px solid rgba(255,255,255,0.06)!important;border-radius:14px!important}}

    /* ── Responsive ── */
    @media (max-width: 1024px){{
      .cronus-decl-main > main{{padding:24px}}
    }}
    @media (max-width: 768px){{
      #cronus-sidebar{{transform:translateX(-100%);width:260px;box-shadow:0 0 40px rgba(0,0,0,0.8)}}
      #cronus-sidebar.open{{transform:translateX(0)}}
      #sb-toggle{{display:flex}}
      .cronus-decl-main{{margin-left:0}}
      .cronus-decl-main::before{{left:0}}
      .cronus-decl-main > main{{padding:70px 16px 24px}}
    }}
    @media print{{#cronus-sidebar,#sb-toggle,.cronus-overlay{{display:none!important}}.cronus-decl-main{{margin-left:0!important}}}}
  </style>
  <style>{anim_css}</style>
</head>
<body>
  <aside id="cronus-sidebar">
    <div class="sb-brand">
      <div class="sb-logo">{brand_initial}</div>
      <span class="sb-name">{brand}</span>
    </div>
    <nav class="sb-nav">
      {nav_items}
    </nav>
    <div class="sb-footer">
      <button onclick="localStorage.clear();location.href='/login'" class="sb-signout">
        <span class="material-symbols-outlined">logout</span>Sign Out
      </button>
    </div>
  </aside>

  <button id="sb-toggle" onclick="document.getElementById('cronus-sidebar').classList.toggle('open')">
    <span class="material-symbols-outlined">menu</span>
  </button>

  <div class="cronus-decl-main" id="cronus-main">
    <main id="cronus-content">
      {body}
    </main>
  </div>

  <script>
    // Update sidebar active state on SPA navigation
    // Uses multiple strategies to ensure reliability:
    // 1. Initial page load
    // 2. History pushState/replaceState override
    // 3. popstate event
    // 4. Interval-based URL polling as fallback
    // 5. Click handler on sidebar links
    (function(){{
      var lastPath = location.pathname;
      function updateActive(){{
        var path = location.pathname;
        var sidebar = document.getElementById('cronus-sidebar');
        if (!sidebar) return;
        var links = sidebar.querySelectorAll('.sb-link');
        var bestMatch = null;
        var bestLen = 0;
        // Find the most specific match
        links.forEach(function(a){{
          var href = a.getAttribute('href');
          if (!href) return;
          if (path === href) {{
            bestMatch = a;
            bestLen = 9999;
          }} else if (href !== '/' && path.indexOf(href) === 0 && href.length > bestLen) {{
            bestMatch = a;
            bestLen = href.length;
          }}
        }});
        // Apply active class
        links.forEach(function(a){{
          if (a === bestMatch) {{
            a.classList.add('active');
          }} else {{
            a.classList.remove('active');
          }}
        }});
      }}
      updateActive();
      // Strategy 1: Override pushState
      ['pushState','replaceState'].forEach(function(m){{
        var orig = history[m];
        history[m] = function(){{
          var r = orig.apply(this, arguments);
          setTimeout(updateActive, 0);
          return r;
        }};
      }});
      // Strategy 2: popstate listener
      window.addEventListener('popstate', updateActive);
      // Strategy 3: interval fallback (polls every 200ms)
      setInterval(function(){{
        if (location.pathname !== lastPath) {{
          lastPath = location.pathname;
          updateActive();
        }}
      }}, 200);
      // Strategy 4: click handler on sidebar links (instant feedback)
      document.addEventListener('click', function(e){{
        var link = e.target.closest('#cronus-sidebar .sb-link');
        if (link) {{
          setTimeout(updateActive, 50);
        }}
      }});
    }})();
  </script>
  <script>{runtime}</script>
  <script>{animate_js}</script>
  <script>{hmr}</script>
  {anim_js}
  {action_js}
</body>
</html>"##,
        app_name = app_name,
        brand = brand,
        brand_initial = brand.chars().next().unwrap_or('K').to_uppercase().to_string(),
        nav_items = nav_html,
        body = body,
        anim_css = CRONUS_ANIMATIONS_CSS,
        anim_js = CRONUS_ANIMATIONS_JS,
        runtime = crate::render::CRONUS_RUNTIME_JS,
        animate_js = crate::animations::CRONUS_ANIMATE_JS,
        hmr = crate::hmr::HMR_CLIENT_JS,
        action_js = crate::runtime_js::CRONUS_ACTION_JS,
        cronus_ui_css = crate::cronus_ui::token_css("aurora", "dark"),
    )
}

/// Convert accent color name to hex (or pass through if already hex)
fn accent_to_hex(accent: &str) -> &str {
    // If already a hex color, return as-is
    if accent.starts_with('#') && accent.len() >= 4 {
        return accent;
    }
    match accent {
        "blue" => "#2563eb",
        "indigo" => "#6366f1",
        "amber" => "#f59e0b",
        "emerald" => "#10b981",
        "rose" => "#f43f5e",
        "violet" => "#8b5cf6",
        "sky" => "#0ea5e9",
        "orange" => "#f97316",
        "red" => "#ef4444",
        "green" => "#22c55e",
        "purple" => "#a855f7",
        "pink" => "#ec4899",
        "cyan" => "#06b6d4",
        "teal" => "#14b8a6",
        "black" => "#000000",
        "white" => "#ffffff",
        _ => "#2563eb",
    }
}

/// Convert hex color to "r,g,b" string for rgba() usage
fn hex_to_rgb(hex: &str) -> String {
    let h = hex.trim_start_matches('#');
    if h.len() >= 6 {
        let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(0);
        format!("{},{},{}", r, g, b)
    } else {
        "135,173,255".to_string()
    }
}

/// Generate CSS custom properties from the StyleNode for theming.
/// All renderers should use var(--cronus-*) instead of hardcoded colors.
fn generate_css_vars(style: &Option<&StyleNode>, theme: &str) -> String {
    let is_light = theme == "light";

    // Determine accent hex — from style config accent-hex, or named accent, or default
    let accent_hex = style
        .and_then(|s| s.config.get("accent-hex").map(|v| v.as_str()))
        .unwrap_or_else(|| {
            let name = style.and_then(|s| s.accent.as_deref()).unwrap_or("blue");
            accent_to_hex(name)
        });

    // Defaults based on theme
    let (def_bg, def_surface, def_text, def_text_muted, def_border) = if is_light {
        ("#fafafa", "#ffffff", "#1a1a1a", "#6b6b6b", "#e5e5e5")
    } else {
        ("#000000", "#0a0a0a", "#ffffff", "#9ca3af", "rgba(255,255,255,0.1)")
    };

    let cfg = |key: &str, default: &str| -> String {
        style
            .and_then(|s| s.config.get(key))
            .map(|v| v.to_string())
            .unwrap_or_else(|| default.to_string())
    };

    let bg = cfg("background", def_bg);
    let surface = cfg("surface", def_surface);
    let text = cfg("text", def_text);
    let text_muted = cfg("text-muted", def_text_muted);
    let border = cfg("border", def_border);
    let max_width = cfg("max-width", "1120px");

    let radius = style
        .and_then(|s| s.radius.as_deref())
        .unwrap_or("8px");
    let radius_px = match radius {
        "sm" => "4px", "md" => "6px", "lg" => "8px", "xl" => "12px", "2xl" => "16px", "full" => "999px",
        other => other,
    };

    let font = style.and_then(|s| s.font.as_deref()).unwrap_or("Inter");

    // Compute accent-hover: use explicit value or darken accent
    let accent_hover = cfg("accent-hover", accent_hex);

    // Glow orbs — from style config or computed from accent
    let glow_1 = cfg("glow-1", &format!("rgba({},0.07)", hex_to_rgb(accent_hex)));
    let glow_2 = cfg("glow-2", &if is_light { "rgba(0,0,0,0.02)".to_string() } else { "rgba(210,119,255,0.04)".to_string() });
    let glow_3 = cfg("glow-3", &if is_light { "rgba(0,0,0,0.01)".to_string() } else { "rgba(129,236,255,0.03)".to_string() });

    let preset = style
        .and_then(|s| s.config.get("preset"))
        .map(|s| s.as_str())
        .unwrap_or("aurora");
    let cronus_ui = crate::cronus_ui::token_css(preset, theme);

    format!(
        r#":root {{
  --cronus-bg: {bg};
  --cronus-surface: {surface};
  --cronus-text: {text};
  --cronus-text-muted: {text_muted};
  --cronus-accent: {accent_hex};
  --cronus-accent-hover: {accent_hover};
  --cronus-border: {border};
  --cronus-radius: {radius_px};
  --cronus-max-w: {max_width};
  --cronus-font: '{font}', system-ui, -apple-system, sans-serif;
  --cronus-glow-1: {glow_1};
  --cronus-glow-2: {glow_2};
  --cronus-glow-3: {glow_3};
}}
body {{ font-family: var(--cronus-font); background: var(--cronus-bg); color: var(--cronus-text); margin: 0; }}
a {{ text-decoration: none; color: inherit; }}
* {{ box-sizing: border-box; }}
{cronus_ui}"#,
        bg = bg, surface = surface, text = text, text_muted = text_muted,
        accent_hex = accent_hex, accent_hover = accent_hover,
        border = border, radius_px = radius_px, max_width = max_width, font = font,
        glow_1 = glow_1, glow_2 = glow_2, glow_3 = glow_3,
        cronus_ui = cronus_ui
    )
}

/// Full-width layout for landing pages (no sidebar)
pub fn render_layout_landing(app_name: &str, body: &str, theme: &str, style_node: Option<&StyleNode>) -> String {
    render_layout_landing_ex(app_name, body, theme, style_node, None)
}

/// Full-width layout for landing pages, with optional Tailwind config injection
pub fn render_layout_landing_ex(app_name: &str, body: &str, theme: &str, style_node: Option<&StyleNode>, tailwind_config_js: Option<&str>) -> String {
    let is_light = theme == "light";
    let html_class = if is_light { "light" } else { "dark" };
    let sel_bg = if is_light { "rgba(0,0,0,0.08)" } else { "rgba(0,111,240,0.3)" };
    let scroll_thumb = if is_light { "rgba(0,0,0,0.1)" } else { "rgba(255,255,255,0.1)" };
    let grid_line = if is_light { "rgba(0,0,0,0.05)" } else { "rgba(255,255,255,0.03)" };
    let css_vars = generate_css_vars(&style_node, theme);

    // Extract inline <style> blocks from the body (injected by render_template)
    // and move them to the <head> so they apply globally
    let mut head_styles = String::new();
    let mut clean_body = body.to_string();
    loop {
        if let Some(start) = clean_body.find("<style>") {
            if let Some(end) = clean_body[start..].find("</style>") {
                let style_content = &clean_body[start..start + end + 8]; // includes </style>
                head_styles.push_str(style_content);
                head_styles.push('\n');
                clean_body = format!("{}{}", &clean_body[..start], &clean_body[start + end + 8..]);
                continue;
            }
        }
        break;
    }
    // Generate Tailwind config script block if provided
    let tw_config_script = if let Some(cfg) = tailwind_config_js {
        // Unescape quotes that were escaped for .cronus string storage
        let unescaped = cfg.replace("\\\"", "\"").replace("\\'", "'");
        format!("<script>\n    {}\n  </script>", unescaped)
    } else {
        String::new()
    };

    // If body already contains a topbar section (rendered <header or <nav with data-topbar),
    // skip the built-in navbar to avoid duplication
    let has_topbar = clean_body.contains("data-cronus-topbar") || clean_body.contains("<nav ") || clean_body.contains("<nav\n") || clean_body.contains("<header ") || clean_body.contains("MONOLITH") || clean_body.contains("topbar") || clean_body.contains("min-h-[90vh]") || clean_body.contains("min-height:90vh") || clean_body.contains("hero") || clean_body.contains("fadeInUp") || clean_body.contains("animate-on-scroll") || clean_body.contains("animate") || !head_styles.is_empty();
    let nav_html = if has_topbar {
        String::new()
    } else {
        let nav_bg = if is_light { "rgba(255,255,255,0.8)" } else { "rgba(0,0,0,0.8)" };
        let nav_border = if is_light { "rgba(229,229,229,0.5)" } else { "rgba(255,255,255,0.05)" };
        let nav_text = if is_light { "black" } else { "white" };
        let nav_muted = if is_light { "#71717a" } else { "#9ca3af" };
        let btn_bg = if is_light { "black" } else { "white" };
        let btn_fg = if is_light { "white" } else { "black" };
        format!(r##"
  <!-- Fixed Navbar (fallback) -->
  <nav style="position:fixed;top:0;width:100%;z-index:50;background:{nav_bg};backdrop-filter:blur(12px);border-bottom:1px solid {nav_border}">
    <div style="display:flex;justify-content:space-between;align-items:center;padding:0 24px;height:64px;max-width:1280px;margin:0 auto">
      <div style="display:flex;align-items:center;gap:32px">
        <div style="font-size:20px;font-weight:700;letter-spacing:-0.03em;color:{nav_text};display:flex;align-items:center;gap:8px">
          <svg width="24" height="24" viewBox="0 0 76 65" fill="{nav_text}"><path d="M37.5274 0L75.0548 65L0 65L37.5274 0Z"/></svg>
          {app_name}
        </div>
        <div style="display:flex;align-items:center;gap:24px">
          <a href="#" style="color:{nav_text};font-size:14px;font-weight:600;letter-spacing:-0.025em;text-decoration:none;transition:color 0.2s">Solutions</a>
          <a href="#" style="color:{nav_muted};font-size:14px;font-weight:500;letter-spacing:-0.025em;text-decoration:none;transition:color 0.2s" onmouseover="this.style.color='{nav_text}'" onmouseout="this.style.color='{nav_muted}'">Resources</a>
          <a href="#" style="color:{nav_muted};font-size:14px;font-weight:500;letter-spacing:-0.025em;text-decoration:none;transition:color 0.2s" onmouseover="this.style.color='{nav_text}'" onmouseout="this.style.color='{nav_muted}'">Docs</a>
          <a href="#" style="color:{nav_muted};font-size:14px;font-weight:500;letter-spacing:-0.025em;text-decoration:none;transition:color 0.2s" onmouseover="this.style.color='{nav_text}'" onmouseout="this.style.color='{nav_muted}'">Pricing</a>
        </div>
      </div>
      <div style="display:flex;align-items:center;gap:12px">
        <a href="#" style="color:{nav_muted};font-size:14px;font-weight:500;padding:6px 16px;text-decoration:none;transition:color 0.2s" onmouseover="this.style.color='{nav_text}'" onmouseout="this.style.color='{nav_muted}'">Contact</a>
        <a href="/signup" style="display:inline-flex;align-items:center;padding:6px 20px;border-radius:999px;background:{btn_bg};color:{btn_fg};font-weight:700;font-size:14px;text-decoration:none;transition:transform 0.15s" onmouseover="this.style.transform='scale(1.05)'" onmouseout="this.style.transform='scale(1)'">Deploy</a>
      </div>
    </div>
  </nav>"##,
            nav_bg=nav_bg, nav_border=nav_border, nav_text=nav_text,
            nav_muted=nav_muted, btn_bg=btn_bg, btn_fg=btn_fg,
            app_name=app_name)
    };

    // Mobile bottom nav — auto-built from sidebar links via JS
    let bottom_nav_html = if clean_body.contains("<aside") {
        r##"<nav class="cronus-bottom-nav" aria-label="Mobile navigation"></nav>
<script>
!function(){
  var aside=document.querySelector('aside');
  var bnav=document.querySelector('.cronus-bottom-nav');
  if(!aside||!bnav)return;
  var links=aside.querySelectorAll('a[data-nav],a[href^="/"]');
  if(links.length===0)return;
  bnav.innerHTML='';
  var p=location.pathname;
  var added=0;
  links.forEach(function(a){
    if(added>=5)return;
    var href=a.getAttribute('href');
    if(!href||href==='#')return;
    var label=a.querySelector('span:last-child');
    var icon=a.querySelector('.material-symbols-outlined');
    if(!label||!icon)return;
    var li=document.createElement('a');
    li.href=href;
    li.innerHTML='<span class="material-symbols-outlined">'+icon.textContent+'</span>'+label.textContent;
    if(href===p)li.classList.add('active');
    bnav.appendChild(li);
    added++;
  });
}();
</script>"##.to_string()
    } else {
        String::new()
    };

    format!(
        r##"<!DOCTYPE html>
<html class="{html_class}" lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{app_name}</title>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800;900&family=JetBrains+Mono:wght@400;500&family=Space+Grotesk:wght@400;500;600;700&display=swap" rel="stylesheet">
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet">
  <script src="https://cdn.tailwindcss.com?plugins=forms,container-queries"></script>
  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
  {tw_config_script}
  {head_styles}
  <style>
    {css_vars}
    *, *::before, *::after {{ margin: 0; padding: 0; box-sizing: border-box; }}
    html {{ overflow-x: hidden; }}
    body {{ background: transparent; color: var(--cronus-text); font-family: var(--cronus-font); -webkit-font-smoothing: antialiased; position: relative; overflow-x: hidden; max-width: 100vw; }}
    html {{ background: var(--cronus-bg); }}
    img, svg, video, canvas {{ max-width: 100%; height: auto; }}
    p, h1, h2, h3, h4, h5, h6, span, a, li {{ overflow-wrap: break-word; word-break: break-word; }}
    #cronus-main > * {{ max-width: 100%; overflow-x: hidden; }}
    section, [style*="border-radius"] {{ max-width: 100%; }}
    html {{ scroll-behavior: smooth; }}
    [id] {{ scroll-margin-top: 4.5rem; }}
    body::before {{
      content: '';
      position: fixed;
      top: 0; left: 0; width: 100%; height: 100%;
      pointer-events: none;
      z-index: -1;
      background:
        radial-gradient(ellipse 60% 50% at 20% 10%, var(--cronus-glow-1, rgba(135,173,255,0.06)) 0%, transparent 70%),
        radial-gradient(ellipse 50% 60% at 80% 80%, var(--cronus-glow-2, rgba(210,119,255,0.04)) 0%, transparent 70%),
        radial-gradient(ellipse 40% 40% at 50% 50%, var(--cronus-glow-3, rgba(129,236,255,0.03)) 0%, transparent 70%);
    }}
    ::selection {{ background: {sel_bg}; }}
    ::-webkit-scrollbar {{ width: 4px; }}
    ::-webkit-scrollbar-thumb {{ background: {scroll_thumb}; border-radius: 2px; }}
    @keyframes fadeIn {{ from {{ opacity:0;transform:translateY(8px) }} to {{ opacity:1;transform:translateY(0) }} }}
    @keyframes blink {{ 0%,100% {{ opacity:1 }} 50% {{ opacity:0 }} }}
    @keyframes pulseGlow {{ 0%,100% {{ opacity:0.6 }} 50% {{ opacity:1 }} }}
    .geist-grid {{
      background-image: linear-gradient(to right, {grid_line} 1px, transparent 1px),
                        linear-gradient(to bottom, {grid_line} 1px, transparent 1px);
      background-size: 40px 40px;
    }}
    .prism-glow {{
      background: radial-gradient(circle at 50% 50%, rgba(0, 111, 240, 0.1) 0%, transparent 70%);
    }}
    .terminal-header {{
      background: linear-gradient(to bottom, #2f3131, #1b1c1c);
    }}
    .material-symbols-outlined {{
      font-variation-settings: 'FILL' 0, 'wght' 400, 'GRAD' 0, 'opsz' 24;
      display: inline-block; line-height: 1; vertical-align: middle;
    }}
    /* Fixed navs/asides must have explicit left:0 to avoid inheriting parent offset */
    nav.fixed, nav[class*="fixed"][class*="top-0"] {{ left: 0 !important; width: 100vw !important; }}
    aside.fixed, aside[class*="fixed"][class*="left-0"] {{ left: 0 !important; }}
    /* Footer alignment with sidebar */
    footer.ml-64, footer[class*="ml-64"] {{ margin-left: 256px; }}
    .anim {{ animation: fadeIn 0.6s ease-out both; }}
    .anim-d1 {{ animation-delay: 0.1s; }}
    .anim-d2 {{ animation-delay: 0.2s; }}
    .anim-d3 {{ animation-delay: 0.3s; }}
    .anim-d4 {{ animation-delay: 0.4s; }}
    .cursor-blink {{ animation: blink 1s step-end infinite; }}
    .pulse-glow {{ animation: pulseGlow 2s ease-in-out infinite; }}

    /* Responsive — Mobile first */
    @media (max-width: 768px) {{
      /* Sidebar → hidden on mobile (bottom nav replaces it) */
      aside {{ display: none !important; }}
      .cronus-bottom-nav {{ display: flex !important; }}

      /* Main content: full width, no sidebar offset */
      #cronus-main, #cronus-content, main {{
        margin-left: 0 !important;
        max-width: 100% !important;
        padding: 1rem 1rem 5rem !important;
        padding-top: 4rem !important;
      }}

      /* Fixed topbar: compact on mobile */
      nav[class*="fixed"][class*="top-0"] {{
        padding: 0 0.75rem !important;
        height: 48px !important;
      }}
      nav[class*="fixed"] input {{ display: none !important; }}
      nav[class*="fixed"] .hidden {{ display: none !important; }}
      nav[class*="fixed"] [class*="w-8"] {{ width: 28px !important; height: 28px !important; }}

      /* Tailwind grid responsive overrides */
      .grid-cols-4, [class*="grid-cols-4"] {{ grid-template-columns: repeat(2, 1fr) !important; }}
      .grid-cols-3, [class*="lg:grid-cols-3"] {{ grid-template-columns: 1fr !important; }}
      .lg\:grid-cols-3 {{ grid-template-columns: 1fr !important; }}
      .lg\:grid-cols-4 {{ grid-template-columns: repeat(2, 1fr) !important; }}
      .md\:grid-cols-2 {{ grid-template-columns: 1fr !important; }}
      .md\:grid-cols-4 {{ grid-template-columns: repeat(2, 1fr) !important; }}

      /* Text sizes */
      .text-6xl {{ font-size: 2rem !important; }}
      .text-4xl {{ font-size: 1.5rem !important; }}

      /* Topbar: compact */
      header {{
        padding: 0 1rem !important;
      }}
      header nav {{ display: none !important; }}

      /* Grids: collapse to 1-2 cols */
      [style*="grid-template-columns:repeat(12"] {{ grid-template-columns: 1fr !important; }}
      [style*="grid-template-columns:repeat(3"] {{ grid-template-columns: 1fr !important; }}
      [style*="grid-template-columns:repeat(4"] {{ grid-template-columns: repeat(2, 1fr) !important; }}
      [style*="grid-template-columns: 1fr 1fr"] {{ grid-template-columns: 1fr !important; }}
      [style*="grid-column:span 2"] {{ grid-column: span 2 !important; }}
      [style*="grid-column:span 4"] {{ grid-column: span 1 !important; }}
      [style*="grid-column:span 6"] {{ grid-column: span 1 !important; }}
      [style*="grid-column:span 8"] {{ grid-column: span 1 !important; }}

      /* Hero + template layouts — override Tailwind with high specificity */
      [style*="min-height:80vh"] {{ min-height: auto !important; padding-top: 80px !important; padding-bottom: 40px !important; }}

      /* Force 12-col grids to stack vertically */
      section.grid, section[class*="grid"] {{ display: flex !important; flex-direction: column !important; }}
      .grid.grid-cols-12, [class*="grid"][class*="cols-12"] {{ display: flex !important; flex-direction: column !important; }}

      /* All col-span become full width */
      [class*="col-span"] {{ width: 100% !important; max-width: 100% !important; grid-column: span 1 !important; }}

      /* Remove sidebar margin */
      .ml-64, [class*="ml-64"] {{ margin-left: 0 !important; }}

      /* Fixed topbar must span full width from left:0 */
      nav[class*="fixed"][class*="top-0"] {{
        left: 0 !important;
        width: 100% !important;
      }}

      /* Side panel border fix */
      [class*="border-l"] {{ border-left: none !important; }}

      /* Chart height */
      .h-80, [class*="h-80"] {{ height: 200px !important; }}

      /* Typography */
      h1, .text-5xl {{ font-size: 28px !important; }}
      h2, .text-3xl {{ font-size: 22px !important; }}
      h3 {{ font-size: 18px !important; }}

      /* Tables: horizontal scroll */
      table {{ display: block; overflow-x: auto; white-space: nowrap; -webkit-overflow-scrolling: touch; }}
      thead, tbody {{ display: table; width: 100%; }}
      td, th {{ min-width: 100px; padding: 12px 16px !important; }}

      /* Cards & padding */
      .p-8, .p-12 {{ padding: 1rem !important; }}
      .p-6 {{ padding: 0.75rem !important; }}
      .mb-12 {{ margin-bottom: 1.5rem !important; }}
      .gap-12 {{ gap: 1rem !important; }}
      [style*="padding:20px"], [style*="padding:24px"] {{ padding: 16px !important; }}

      /* Chart bars container */
      [style*="height:200px"] {{ height: 150px !important; }}

      /* Footer: above bottom nav */
      footer {{ bottom: 3.5rem !important; }}

      /* Stats grid */
      .grid-cols-2 {{ grid-template-columns: repeat(2, 1fr) !important; }}
    }}

    /* Bottom nav — hidden on desktop, shown on mobile */
    .cronus-bottom-nav {{
      display: none;
      position: fixed;
      bottom: 0;
      left: 0;
      width: 100%;
      height: 3.5rem;
      background: #0e0e0e;
      border-top: 1px solid rgba(72,72,72,0.15);
      z-index: 60;
      align-items: center;
      justify-content: space-around;
      padding: 0;
    }}
    .cronus-bottom-nav a {{
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 2px;
      color: rgba(255,255,255,0.4);
      text-decoration: none;
      font-size: 9px;
      font-weight: 600;
      letter-spacing: 0.05em;
      text-transform: uppercase;
      transition: color 0.15s;
    }}
    .cronus-bottom-nav a.active {{
      color: #87adff;
    }}
    .cronus-bottom-nav a .material-symbols-outlined {{
      font-size: 22px;
    }}

    /* Page entrance */
    @keyframes slideUp {{ from {{ opacity:0; transform:translateY(24px) }} to {{ opacity:1; transform:translateY(0) }} }}
    @keyframes slideDown {{ from {{ opacity:0; transform:translateY(-12px) }} to {{ opacity:1; transform:translateY(0) }} }}
    @keyframes scaleIn {{ from {{ opacity:0; transform:scale(0.96) }} to {{ opacity:1; transform:scale(1) }} }}
    @keyframes slideRight {{ from {{ opacity:0; transform:translateX(-16px) }} to {{ opacity:1; transform:translateX(0) }} }}
    @keyframes fillWidth {{ from {{ width:0 }} to {{ width:var(--target-width) }} }}
    @keyframes countUp {{ from {{ opacity:0; transform:translateY(8px) }} to {{ opacity:1; transform:translateY(0) }} }}
    @keyframes shimmer {{ 0% {{ background-position:-200% 0 }} 100% {{ background-position:200% 0 }} }}
    @keyframes pulse {{ 0%,100% {{ opacity:1 }} 50% {{ opacity:0.5 }} }}
    @keyframes float {{ 0%,100% {{ transform:translateY(0) }} 50% {{ transform:translateY(-6px) }} }}

    /* Utility classes */
    .anim-fade {{ animation:fadeIn 0.6s ease-out both }}
    .anim-slide-up {{ animation:slideUp 0.6s cubic-bezier(0.16,1,0.3,1) both }}
    .anim-slide-down {{ animation:slideDown 0.4s ease-out both }}
    .anim-scale {{ animation:scaleIn 0.5s cubic-bezier(0.16,1,0.3,1) both }}
    .anim-slide-right {{ animation:slideRight 0.5s cubic-bezier(0.16,1,0.3,1) both }}

    /* Stagger delays */
    .d1 {{ animation-delay:0.05s }} .d2 {{ animation-delay:0.1s }} .d3 {{ animation-delay:0.15s }}
    .d4 {{ animation-delay:0.2s }} .d5 {{ animation-delay:0.25s }} .d6 {{ animation-delay:0.3s }}
    .d7 {{ animation-delay:0.35s }} .d8 {{ animation-delay:0.4s }} .d9 {{ animation-delay:0.45s }}
    .d10 {{ animation-delay:0.5s }}

    /* Card hover */
    .card-hover {{ transition:all 0.3s cubic-bezier(0.16,1,0.3,1) }}
    .card-hover:hover {{ transform:translateY(-2px); box-shadow:0 12px 40px rgba(0,0,0,0.08) }}

    /* Button hover */
    .btn-hover {{ transition:all 0.2s cubic-bezier(0.16,1,0.3,1) }}
    .btn-hover:hover {{ transform:translateY(-1px); box-shadow:0 4px 12px rgba(0,0,0,0.15) }}
    .btn-hover:active {{ transform:translateY(0); box-shadow:none }}

    /* Link hover */
    .link-hover {{ transition:color 0.2s ease, opacity 0.2s ease }}
    .link-hover:hover {{ opacity:0.7 }}

    /* Nav item */
    .nav-hover {{ transition:all 0.15s ease }}
    .nav-hover:hover {{ background:rgba(0,0,0,0.04) }}

    /* Progress bar fill */
    .progress-fill {{ animation:fillWidth 1.2s cubic-bezier(0.16,1,0.3,1) 0.3s both }}

    /* Scroll reveal */
    .reveal {{ opacity:0; transform:translateY(20px); transition:all 0.7s cubic-bezier(0.16,1,0.3,1) }}
    .reveal.visible {{ opacity:1; transform:translateY(0) }}
  </style>
  <style>{anim_css}</style>
  <style>{tailwind_css}</style>
</head>
<body style="margin:0;padding:0;width:100%;max-width:100vw;overflow-x:hidden">
  <div class="aura-background-component top-0 w-full -z-10 absolute h-[900px]" data-alpha-mask="80" style="{unicorn_display}mask-image: linear-gradient(to bottom, transparent, black 0%, black 80%, transparent); -webkit-mask-image: linear-gradient(to bottom, transparent, black 0%, black 80%, transparent)"><div class="aura-background-component top-0 w-full -z-10 absolute h-full"><div data-us-project="bKN5upvoulAmWvInmHza" class="absolute w-full h-full left-0 top-0 -z-10"></div><script type="text/javascript">!function(){{if(!window.UnicornStudio){{window.UnicornStudio={{isInitialized:!1}};var i=document.createElement("script");i.src="https://cdn.jsdelivr.net/gh/hiunicornstudio/unicornstudio.js@v1.4.29/dist/unicornStudio.umd.js",i.onload=function(){{window.UnicornStudio.isInitialized||(UnicornStudio.init(),window.UnicornStudio.isInitialized=!0)}},(document.head||document.body).appendChild(i)}}}}();</script></div></div>
  {nav_html}
  <main id="cronus-main" style="min-height:100vh;width:100%;box-sizing:border-box">
  {clean_body}
  </main>
  <script>
  // Auto-detect fixed sidebar and adjust main content offset (desktop only)
  !function(){{
    if(window.innerWidth<=768)return; // skip on mobile
    var aside=document.querySelector('aside');
    // If page already has #cronus-content with its own margin, skip auto-offset
    // (template pages handle their own layout via render_custom)
    var cronusContent=document.getElementById('cronus-content');
    if(cronusContent)return;
    var main=document.getElementById('cronus-main');
    if(aside&&main){{
      var s=getComputedStyle(aside);
      if(s.position==='fixed'){{
        var w=aside.offsetWidth||256;
        main.style.marginLeft=w+'px';
        main.style.maxWidth='calc(100% - '+w+'px)';
      }}
    }}
    // Handle resize (skip if page uses #cronus-content)
    if(!document.getElementById('cronus-content')){{
      window.addEventListener('resize',function(){{
        var main=document.getElementById('cronus-main');
        if(!main)return;
        if(window.innerWidth<=768){{
          main.style.marginLeft='';
          main.style.maxWidth='';
        }}else{{
          var aside=document.querySelector('aside');
          if(aside){{
            var s=getComputedStyle(aside);
            if(s.position==='fixed'){{
              var w=aside.offsetWidth||256;
              main.style.marginLeft=w+'px';
              main.style.maxWidth='calc(100% - '+w+'px)';
            }}
          }}
        }}
      }});
    }}
  }}();

  // Fix template layouts inside #cronus-main
  !function(){{
    var main=document.getElementById('cronus-main');
    if(!main)return;
    // Remove ml-64 from template children (main already has margin-left)
    main.querySelectorAll('.ml-64,[class*="ml-64"]').forEach(function(el){{
      el.style.marginLeft='0';
    }});
    // Fix 12-col grid layouts — only when Tailwind CDN is NOT loaded
    // (templates already use Tailwind classes that resolve natively)
    var hasTailwind=!!document.querySelector('script[src*="tailwindcss"]');
    if(!hasTailwind){{
      function fixGridLayout(){{
        var w=main.offsetWidth;
        var narrow=w<900;
        main.querySelectorAll('.grid-cols-12,.grid.grid-cols-12').forEach(function(g){{
          g.style.display='flex';
          g.style.flexDirection=narrow?'column':'row';
          g.style.overflow='hidden';
          g.querySelectorAll('[class*="col-span"]').forEach(function(c){{
            c.style.overflow='hidden';
            c.style.minWidth='0';
            if(narrow){{
              c.style.flex='none';
              c.style.width='100%';
            }}else{{
              var cls=c.className;
              if(cls.indexOf('col-span-8')>=0)c.style.flex='2 1 0';
              else if(cls.indexOf('col-span-4')>=0)c.style.flex='1 1 0';
              else c.style.flex='1 1 0';
              c.style.width='';
            }}
          }});
        }});
      }}
      fixGridLayout();
      window.addEventListener('resize',fixGridLayout);
    }}

    // Kernel Logs animation — reveal each log entry one by one, then stream new ones
    var logContainer=main.querySelector('.font-mono.space-y-3,[class*="font-mono"][class*="space-y"]');
    if(logContainer){{
      var logEntries=Array.from(logContainer.children);
      // Hide all logs initially
      logEntries.forEach(function(el){{
        el.style.opacity='0';
        el.style.transform='translateY(6px)';
        el.style.transition='opacity 0.4s ease, transform 0.4s ease';
      }});
      // Reveal one by one
      logEntries.forEach(function(el,i){{
        setTimeout(function(){{
          el.style.opacity='1';
          el.style.transform='translateY(0)';
        }},800+i*300);
      }});
      // After initial reveal, start streaming new fake logs
      var logTypes=[
        ['INFO','text-tertiary','Cache invalidation cycle completed: 2.1ms'],
        ['INFO','text-tertiary','TLS certificate auto-renewed for *.nova-core.io'],
        ['DEBG','text-primary-dim','Heartbeat ping to cluster Alpha-9: 4ms'],
        ['INFO','text-tertiary','Worker pool scaled to 8 instances (auto)'],
        ['WARN','text-error','Connection timeout on DB replica-03 (retrying)'],
        ['INFO','text-tertiary','Rate limiter reset for API tier-2 clients'],
        ['DEBG','text-primary-dim','GC pause: 1.2ms (within budget)'],
        ['INFO','text-tertiary','Webhook delivery confirmed: 847/847 sent'],
        ['WARN','text-error','Disk I/O spike on node-v7 (87% threshold)'],
        ['INFO','text-tertiary','Session cleanup: 42 expired tokens purged'],
        ['INFO','text-tertiary','Config hot-reload: 3 services updated'],
        ['DEBG','text-primary-dim','Metrics export to Prometheus: OK'],
        ['CRIT','text-error','Memory allocation failed on worker-12 (recovered)'],
        ['INFO','text-tertiary','Load balancer health check: all nodes green'],
      ];
      var logIdx=0;
      setTimeout(function streamLog(){{
        var now=new Date();
        var ts=String(now.getHours()).padStart(2,'0')+':'+String(now.getMinutes()).padStart(2,'0')+':'+String(now.getSeconds()).padStart(2,'0');
        var l=logTypes[logIdx%logTypes.length];
        var entry=document.createElement('div');
        entry.className='flex gap-3';
        entry.style.cssText='opacity:0;transform:translateY(6px);transition:opacity 0.4s ease,transform 0.4s ease';
        entry.innerHTML='<span class="shrink-0 text-on-surface-variant">'+ts+'</span><span class="'+l[1]+'">['+l[0]+']</span><span class="text-white/70">'+l[2]+'</span>';
        logContainer.insertBefore(entry,logContainer.firstChild);
        requestAnimationFrame(function(){{requestAnimationFrame(function(){{
          entry.style.opacity='1';
          entry.style.transform='translateY(0)';
        }})}});
        // Remove old entries to keep list short
        while(logContainer.children.length>10)logContainer.removeChild(logContainer.lastChild);
        logIdx++;
        setTimeout(streamLog,3000+Math.random()*4000);
      }},800+logEntries.length*300+1500);
    }}
    // Add padding to main ONLY if page doesn't have a full-width template grid
    var hasFullGrid=main.querySelector('.grid-cols-12,.grid.grid-cols-12');
    if(!hasFullGrid){{
      main.style.padding='2rem 3rem';
    }}
  }}();

  // Force responsive layout for Tailwind template grids
  function cronusResponsive(){{
    var mobile=window.innerWidth<=768;
    // 12-col grids (hero layouts)
    document.querySelectorAll('.grid-cols-12,.grid.grid-cols-12').forEach(function(g){{
      if(mobile){{
        g.style.display='flex';
        g.style.flexDirection='column';
      }}else{{
        g.style.display='';
        g.style.flexDirection='';
      }}
    }});
    // col-span children
    document.querySelectorAll('[class*="col-span-"]').forEach(function(c){{
      if(mobile){{
        c.style.width='100%';
        c.style.maxWidth='100%';
      }}else{{
        c.style.width='';
        c.style.maxWidth='';
      }}
    }});
  }}
  cronusResponsive();
  window.addEventListener('resize',cronusResponsive);
  </script>
  <script>{runtime}</script>
  <script>{hmr}</script>
  <script>
    document.addEventListener('DOMContentLoaded',function(){{
      var io=new IntersectionObserver(function(entries){{
        entries.forEach(function(e){{if(e.isIntersecting){{e.target.classList.add('visible');io.unobserve(e.target)}}}})
      }},{{threshold:0.1,rootMargin:'0px 0px -40px 0px'}});
      document.querySelectorAll('.reveal').forEach(function(el){{io.observe(el)}});
      document.querySelectorAll('.stagger').forEach(function(container){{
        Array.from(container.children).forEach(function(child,i){{
          child.style.animationDelay=(0.05+i*0.06)+'s';
        }});
      }});
    }});
  </script>
  {anim_js}
  {action_js}
  {bottom_nav}
  <script>
  // SPA Router — intercept internal links, swap content without full reload
  !function(){{
    var navInFlight=false;
    function navigateTo(url){{
      var main=document.getElementById('cronus-content')||document.getElementById('cronus-main');
      if(!main||navInFlight)return false;
      navInFlight=true;
      window.__hmrPaused=true;

      // Prefetch immediately while animating out
      var fetchPromise=fetch(url).then(function(r){{return r.text()}});

      // Smooth fade-out
      main.style.transition='opacity 0.2s cubic-bezier(0.4,0,0.2,1), transform 0.2s cubic-bezier(0.4,0,0.2,1)';
      main.style.opacity='0';
      main.style.transform='translateY(8px)';

      // Wait for BOTH fade-out AND fetch to complete
      var fadeOutDone=new Promise(function(r){{setTimeout(r,220)}});

      Promise.all([fetchPromise,fadeOutDone]).then(function(results){{
        var html=results[0];
        var doc=new DOMParser().parseFromString(html,'text/html');
        var newContent=doc.getElementById('cronus-content')||doc.getElementById('cronus-main');
        var source=newContent||doc.querySelector('body');
        var pageScripts=[];
        if(source){{
          // Collect inline scripts, then remove from DOM before swap
          source.querySelectorAll('script').forEach(function(s){{
            if(!s.src)pageScripts.push(s.textContent);
            s.remove();
          }});
          main.innerHTML=source.innerHTML;
        }}

        history.pushState(null,'',url);
        var newTitle=doc.querySelector('title');
        if(newTitle)document.title=newTitle.textContent;

        // Update sidebar active state — toggle .active class for smooth CSS transition
        var navContainer=document.getElementById('admin-nav')||document.getElementById('user-nav');
        if(navContainer){{
          navContainer.querySelectorAll('[data-nav]').forEach(function(a){{
            if(a.getAttribute('href')===url)a.classList.add('active');
            else a.classList.remove('active');
          }});
        }}
        // Settings link for user sidebar
        var settingsLink=document.getElementById('user-settings-link');
        if(settingsLink){{
          if(url==='/settings')settingsLink.classList.add('active');
          else settingsLink.classList.remove('active');
        }}
        document.querySelectorAll('.cronus-bottom-nav a').forEach(function(a){{
          if(a.getAttribute('href')===url)a.classList.add('active');
          else a.classList.remove('active');
        }});

        window.scrollTo({{top:0,behavior:'instant'}});

        // Smooth fade-in
        main.style.transform='translateY(12px)';
        main.style.opacity='0';
        requestAnimationFrame(function(){{requestAnimationFrame(function(){{
          main.style.transition='opacity 0.35s cubic-bezier(0,0,0.2,1), transform 0.35s cubic-bezier(0,0,0.2,1)';
          main.style.opacity='1';
          main.style.transform='translateY(0)';
        }})}});

        // Run animations + page scripts after fade-in completes
        setTimeout(function(){{
          if(window.__cronusAnimateContent)window.__cronusAnimateContent(main);
          // Execute inline scripts from the new page
          pageScripts.forEach(function(code){{
            try{{(new Function(code))()}}catch(e){{console.warn('[SPA] script error:',e)}}
          }});
          navInFlight=false;
          setTimeout(function(){{window.__hmrPaused=false}},500);
        }},400);
      }}).catch(function(){{
        window.__hmrPaused=false;
        navInFlight=false;
        window.location.href=url;
      }});
      return true;
    }}
    // Intercept clicks on internal links (only when sidebar layout is active)
    document.addEventListener('click',function(e){{
      var a=e.target.closest('a[href]');
      if(!a)return;
      var href=a.getAttribute('href');
      if(!href||href==='#'||href.startsWith('http')||href.startsWith('mailto'))return;
      if(href===location.pathname)return;
      if(a.hasAttribute('download')||a.getAttribute('target')==='_blank')return;
      if(!document.getElementById('admin-nav')&&!document.getElementById('user-nav'))return;
      e.preventDefault();
      navigateTo(href);
    }});
    // Handle browser back/forward
    window.addEventListener('popstate',function(){{
      navigateTo(location.pathname);
    }});
  }}();
  </script>
  <script>
  // Premium Animations Runtime — contextual, purposeful animations
  window.__cronusAnimateContent=function(scope){{
    scope=scope||document;
    var io=new IntersectionObserver(function(entries){{
      entries.forEach(function(e){{
        if(!e.isIntersecting)return;
        var el=e.target;
        el.dataset.visible='1';
        io.unobserve(el);

        // 1. Number count-up: elements with large numeric text (KPIs, stats)
        var bigNums=el.querySelectorAll('.text-4xl,.text-3xl,.text-5xl,.text-2xl');
        bigNums.forEach(function(n){{
          var raw=n.childNodes[0];
          if(!raw||raw.nodeType!==3)return;
          var txt=raw.textContent.trim();
          var num=parseFloat(txt.replace(/,/g,''));
          if(isNaN(num)||num===0)return;
          var start=0;
          var dur=1200;
          var t0=performance.now();
          var hasDecimal=txt.indexOf('.')>=0;
          var decimals=hasDecimal?(txt.split('.')[1]||'').replace(/[^0-9]/g,'').length:0;
          function tick(now){{
            var p=Math.min((now-t0)/dur,1);
            // ease-out-expo
            var ep=p===1?1:1-Math.pow(2,-10*p);
            var v=start+(num-start)*ep;
            raw.textContent=hasDecimal?v.toFixed(decimals):Math.round(v).toLocaleString();
            if(p<1)requestAnimationFrame(tick);
          }}
          requestAnimationFrame(tick);
        }});

        // 2. Progress bars: elements with width set as percentage
        el.querySelectorAll('[class*="h-full"],[class*="h-1"],[class*="h-2"]').forEach(function(bar){{
          var w=bar.style.width||bar.className.match(/w-\[(\d+)%\]/);
          if(!w)return;
          var target=typeof w==='string'?w:(w[1]+'%');
          if(target.indexOf('%')<0)return;
          bar.style.width='0%';
          bar.style.transition='width 1.4s cubic-bezier(0.16,1,0.3,1)';
          requestAnimationFrame(function(){{requestAnimationFrame(function(){{
            bar.style.width=target;
          }})}});
        }});

        // 3. Chart bars: grow from bottom
        el.querySelectorAll('[class*="rounded-t"],[class*="hover:bg-white"]').forEach(function(bar,i){{
          if(bar.tagName==='A'||bar.tagName==='BUTTON')return;
          var h=bar.style.height||getComputedStyle(bar).height;
          bar.style.height='0';
          bar.style.transition='height 0.8s cubic-bezier(0.16,1,0.3,1) '+(i*0.05)+'s';
          requestAnimationFrame(function(){{requestAnimationFrame(function(){{
            bar.style.height=h;
          }})}});
        }});

        // 4. Stagger children: glass-panel cards, table rows
        var staggerTargets=el.querySelectorAll('.glass-panel,tr,a[href]');
        staggerTargets.forEach(function(child,i){{
          if(child.closest('[data-visible]')!==el)return;
          child.style.opacity='0';
          child.style.transform='translateY(12px)';
          child.style.transition='opacity 0.5s ease,transform 0.5s cubic-bezier(0.16,1,0.3,1)';
          child.style.transitionDelay=(i*0.06)+'s';
          requestAnimationFrame(function(){{requestAnimationFrame(function(){{
            child.style.opacity='1';
            child.style.transform='translateY(0)';
          }})}});
        }});
      }});
    }},{{threshold:0.05,rootMargin:'0px 0px -20px 0px'}});

    // Observe all major content blocks
    document.querySelectorAll('.glass-panel,.grid,[class*="grid-cols"],table,.rounded-xl,header,section,main>div,#cronus-content>div,#cronus-content>header,#cronus-content>section').forEach(function(el){{
      if(el.offsetHeight>10)io.observe(el);
    }});

    // 5. SVG path draw-in animation
    document.querySelectorAll('svg path[d]').forEach(function(path){{
      try{{
        var len=path.getTotalLength();
        if(len<50||len>5000)return;
        var stroke=path.getAttribute('stroke');
        var fill=path.getAttribute('fill');
        // Animate stroke paths (chart lines)
        if(stroke&&stroke!=='none'){{
          path.style.strokeDasharray=len;
          path.style.strokeDashoffset=len;
          path.style.transition='stroke-dashoffset 2s cubic-bezier(0.16,1,0.3,1) 0.3s';
          requestAnimationFrame(function(){{path.style.strokeDashoffset='0'}});
        }}
        // Animate fill paths (area charts) — fade in
        if(fill&&fill!=='none'){{
          path.style.opacity='0';
          path.style.transition='opacity 1.5s ease 0.5s';
          requestAnimationFrame(function(){{requestAnimationFrame(function(){{
            path.style.opacity='';
          }})}});
        }}
      }}catch(e){{}}
    }});

    // 6. Hover micro-interactions for glass-panels
    document.querySelectorAll('.glass-panel').forEach(function(el){{
      el.style.transition=(el.style.transition||'')+',transform 0.3s ease,box-shadow 0.3s ease';
      el.addEventListener('mouseenter',function(){{
        el.style.transform='translateY(-2px)';
        el.style.boxShadow='0 8px 32px rgba(255,255,255,0.06)';
      }});
      el.addEventListener('mouseleave',function(){{
        el.style.transform='translateY(0)';
        el.style.boxShadow='';
      }});
    }});

    // 7. Pulsing status dots
    document.querySelectorAll('[class*="bg-green-500"][class*="rounded-full"]').forEach(function(dot){{
      if(dot.offsetWidth<=8){{
        dot.style.animation='pulse 2s ease-in-out infinite';
      }}
    }});

    // 8. Table row hover glow
    document.querySelectorAll('tbody tr').forEach(function(row){{
      row.style.transition='background 0.2s ease';
    }});
  }};
  // Run on initial load
  window.__cronusAnimateContent();
  </script>
  <script>
  // Token auto-refresh — re-authenticate silently before expiry
  !function(){{
    var token=localStorage.getItem('token');
    if(!token)return;
    // Decode JWT payload to check expiry
    try{{
      var payload=JSON.parse(atob(token.split('.')[1]));
      var exp=payload.exp*1000; // ms
      var now=Date.now();
      var remaining=exp-now;
      // If token expires in less than 1 hour, refresh now
      if(remaining<3600000&&remaining>0){{
        fetch('/api/auth/me',{{headers:{{'Authorization':'Bearer '+token}}}})
          .then(function(r){{return r.json()}})
          .then(function(d){{
            if(d.token){{
              localStorage.setItem('token',d.token);
              document.cookie='cronus_token='+d.token+';path=/;max-age=604800;SameSite=Strict';
            }}
          }}).catch(function(){{}});
      }}
      // Schedule refresh 1 hour before expiry
      if(remaining>3600000){{
        setTimeout(function(){{
          var t=localStorage.getItem('token');
          if(!t)return;
          fetch('/api/auth/me',{{headers:{{'Authorization':'Bearer '+t}}}})
            .then(function(r){{return r.json()}})
            .then(function(d){{
              if(d.token){{
                localStorage.setItem('token',d.token);
                document.cookie='cronus_token='+d.token+';path=/;max-age=604800;SameSite=Strict';
              }}
            }}).catch(function(){{}});
        }},remaining-3600000);
      }}
    }}catch(e){{}}
    // Also inject token into every fetch for SPA navigation
    var origFetch=window.fetch;
    window.fetch=function(url,opts){{
      opts=opts||{{}};
      if(typeof url==='string'&&url.startsWith('/')&&!url.startsWith('/api/auth/login')){{
        var t=localStorage.getItem('token');
        if(t){{
          opts.headers=opts.headers||{{}};
          if(!opts.headers['Authorization']&&!opts.headers['authorization']){{
            opts.headers['Authorization']='Bearer '+t;
          }}
          // Also set cookie for page requests
          opts.credentials='same-origin';
        }}
      }}
      return origFetch.call(this,url,opts);
    }};
  }}();
  </script>
  <script>
  // ── CRONUS SPA Router ──────────────────────────────
  // Intercepts internal link clicks, fetches new page, swaps body.
  // No full reload. Sidebar persists visually. ~800 bytes.
  !function(){{
    if(typeof history.pushState!=='function')return;
    var transitioning=false;

    function isLocal(a){{
      if(!a.href)return false;
      if(a.target==='_blank'||a.hasAttribute('download'))return false;
      if(a.href.indexOf(location.origin)!==0)return false;
      if(a.href.indexOf('#')>-1&&a.href.split('#')[0]===location.href.split('#')[0])return false;
      if(a.href.match(/\.(pdf|zip|png|jpg|csv)$/i))return false;
      return true;
    }}

    function navigate(url,push){{
      if(transitioning)return;
      transitioning=true;
      // Fade out
      document.body.style.opacity='0.6';
      document.body.style.transition='opacity 0.1s ease';

      fetch(url,{{headers:{{'X-CRONUS-SPA':'1'}},credentials:'same-origin'}})
        .then(function(r){{
          if(!r.ok){{ window.location.href=url; return null; }}
          return r.text();
        }})
        .then(function(html){{
          if(!html)return;
          var doc=new DOMParser().parseFromString(html,'text/html');
          // Swap body
          document.body.innerHTML=doc.body.innerHTML;
          // Copy body attributes
          Array.from(doc.body.attributes).forEach(function(a){{
            document.body.setAttribute(a.name,a.value);
          }});
          // Update title
          var t=doc.querySelector('title');
          if(t)document.title=t.textContent;
          // Update head styles (for style_block changes between pages)
          var oldStyles=document.querySelectorAll('style[data-cronus-page]');
          oldStyles.forEach(function(s){{s.remove()}});
          doc.querySelectorAll('style').forEach(function(s){{
            if(s.textContent.indexOf('body')>-1||s.textContent.indexOf('#cronus')>-1){{
              var ns=s.cloneNode(true);
              ns.setAttribute('data-cronus-page','1');
              document.head.appendChild(ns);
            }}
          }});
          // Execute scripts
          document.querySelectorAll('script:not([src])').forEach(function(s){{
            if(s.textContent.indexOf('CRONUS SPA Router')>-1)return; // skip self
            if(s.textContent.indexOf('tailwind')>-1)return; // skip tailwind config
            try{{ new Function(s.textContent)(); }}catch(e){{}}
          }});
          // Re-run Tailwind if present
          if(window.tailwind&&window.tailwind.refresh){{
            setTimeout(function(){{window.tailwind.refresh()}},50);
          }}
          // Push history
          if(push)history.pushState(null,document.title,url);
          // Scroll top
          window.scrollTo(0,0);
          // Fade in
          document.body.style.opacity='1';
          transitioning=false;
          // Re-attach click listener
          listen();
        }})
        .catch(function(){{
          window.location.href=url;
          transitioning=false;
        }});
    }}

    function listen(){{
      document.addEventListener('click',function(e){{
        var a=e.target.closest('a');
        if(!a||!isLocal(a))return;
        if(!document.getElementById('admin-nav')&&!document.getElementById('user-nav'))return;
        e.preventDefault();
        if(a.href===location.href)return;
        navigate(a.href,true);
      }});
    }}

    listen();
    window.addEventListener('popstate',function(){{
      navigate(location.href,false);
    }});
  }}();
  </script>
</body>
</html>"##,
        app_name = app_name,
        html_class = html_class,
        tw_config_script = tw_config_script,
        head_styles = head_styles,
        css_vars = css_vars, sel_bg = sel_bg, scroll_thumb = scroll_thumb, grid_line = grid_line,
        unicorn_display = if head_styles.contains("fadeInUp") || clean_body.contains("animate-on-scroll") { "" } else { "display:none;" },
        nav_html = nav_html,
        clean_body = clean_body,
        bottom_nav = bottom_nav_html,
        anim_css = CRONUS_ANIMATIONS_CSS,
        anim_js = CRONUS_ANIMATIONS_JS,
        tailwind_css = crate::tailwind::CRONUS_TAILWIND,
        runtime = crate::render::CRONUS_RUNTIME_JS,
        hmr = crate::hmr::HMR_CLIENT_JS,
        action_js = crate::runtime_js::CRONUS_ACTION_JS,
    )
}

// ══════════════════════════════════════════════════
// DASHBOARD LAYOUT — Light, Geist design system
// ══════════════════════════════════════════════════

pub fn render_layout_dashboard(app_name: &str, body: &str, theme: &str) -> String {
    let t = crate::theme::get();
    let is_dark = theme == "dark" || theme == "obsidian";
    let (html_class, bg_color, text_color, selection_bg, scrollbar_color) = if is_dark {
        ("dark", t.background.as_str(), t.on_surface.as_str(), "rgba(173,198,255,0.2)", "rgba(255,255,255,0.1)")
    } else {
        ("light", "#f9f9f9", "#1a1c1c", "rgba(0,111,240,0.15)", "rgba(0,0,0,0.1)")
    };
    let theme_css = crate::theme::css_vars();
    let font_links = crate::theme::font_links();
    let tailwind_cdn = crate::theme::tailwind_cdn_script();
    format!(
        r##"<!DOCTYPE html>
<html class="{html_class}" lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{app_name}</title>
  {font_links}
  {tailwind_cdn}
  <style>
    {theme_css}
    body {{ font-family:var(--font-body); background:{bg_color}; color:{text_color}; margin:0; -webkit-font-smoothing:antialiased; }}
    * {{ box-sizing:border-box; }}
    .material-symbols-outlined {{ font-variation-settings:'FILL' 0,'wght' 400,'GRAD' 0,'opsz' 24; font-size:20px; display:inline-block; line-height:1; vertical-align:middle; }}
    .prism-bg {{ background: radial-gradient(circle at top right,rgba(0,111,240,0.08),transparent 40%),radial-gradient(circle at bottom left,rgba(0,56,129,0.05),transparent 40%); }}
    .engineering-grid {{ background-image:linear-gradient(to right,rgba(198,198,198,0.1) 1px,transparent 1px),linear-gradient(to bottom,rgba(198,198,198,0.1) 1px,transparent 1px); background-size:40px 40px; }}
    .ghost-border {{ border:1px solid rgba(198,198,198,0.2); }}
    .technical-border {{ border:0.5px solid rgba(76,69,70,0.15); }}
    .liquid-glass {{ background:linear-gradient(135deg,rgba(173,198,255,0.05) 0%,rgba(194,193,255,0.05) 100%); backdrop-filter:blur(32px); }}
    ::selection {{ background:{selection_bg}; }}
    ::-webkit-scrollbar {{ width:4px; }}
    ::-webkit-scrollbar-thumb {{ background:{scrollbar_color}; border-radius:2px; }}
    @keyframes fadeIn {{ from {{ opacity:0;transform:translateY(4px) }} to {{ opacity:1;transform:translateY(0) }} }}
    .anim {{ animation:fadeIn 0.4s ease-out both; }}

    /* Page entrance */
    @keyframes slideUp {{ from {{ opacity:0; transform:translateY(24px) }} to {{ opacity:1; transform:translateY(0) }} }}
    @keyframes slideDown {{ from {{ opacity:0; transform:translateY(-12px) }} to {{ opacity:1; transform:translateY(0) }} }}
    @keyframes scaleIn {{ from {{ opacity:0; transform:scale(0.96) }} to {{ opacity:1; transform:scale(1) }} }}
    @keyframes slideRight {{ from {{ opacity:0; transform:translateX(-16px) }} to {{ opacity:1; transform:translateX(0) }} }}
    @keyframes fillWidth {{ from {{ width:0 }} to {{ width:var(--target-width) }} }}
    @keyframes countUp {{ from {{ opacity:0; transform:translateY(8px) }} to {{ opacity:1; transform:translateY(0) }} }}
    @keyframes shimmer {{ 0% {{ background-position:-200% 0 }} 100% {{ background-position:200% 0 }} }}
    @keyframes pulse {{ 0%,100% {{ opacity:1 }} 50% {{ opacity:0.5 }} }}
    @keyframes float {{ 0%,100% {{ transform:translateY(0) }} 50% {{ transform:translateY(-6px) }} }}

    /* Utility classes */
    .anim-fade {{ animation:fadeIn 0.6s ease-out both }}
    .anim-slide-up {{ animation:slideUp 0.6s cubic-bezier(0.16,1,0.3,1) both }}
    .anim-slide-down {{ animation:slideDown 0.4s ease-out both }}
    .anim-scale {{ animation:scaleIn 0.5s cubic-bezier(0.16,1,0.3,1) both }}
    .anim-slide-right {{ animation:slideRight 0.5s cubic-bezier(0.16,1,0.3,1) both }}

    /* Stagger delays */
    .d1 {{ animation-delay:0.05s }} .d2 {{ animation-delay:0.1s }} .d3 {{ animation-delay:0.15s }}
    .d4 {{ animation-delay:0.2s }} .d5 {{ animation-delay:0.25s }} .d6 {{ animation-delay:0.3s }}
    .d7 {{ animation-delay:0.35s }} .d8 {{ animation-delay:0.4s }} .d9 {{ animation-delay:0.45s }}
    .d10 {{ animation-delay:0.5s }}

    /* Card hover */
    .card-hover {{ transition:all 0.3s cubic-bezier(0.16,1,0.3,1) }}
    .card-hover:hover {{ transform:translateY(-2px); box-shadow:0 12px 40px rgba(0,0,0,0.08) }}

    /* Button hover */
    .btn-hover {{ transition:all 0.2s cubic-bezier(0.16,1,0.3,1) }}
    .btn-hover:hover {{ transform:translateY(-1px); box-shadow:0 4px 12px rgba(0,0,0,0.15) }}
    .btn-hover:active {{ transform:translateY(0); box-shadow:none }}

    /* Link hover */
    .link-hover {{ transition:color 0.2s ease, opacity 0.2s ease }}
    .link-hover:hover {{ opacity:0.7 }}

    /* Nav item */
    .nav-hover {{ transition:all 0.15s ease }}
    .nav-hover:hover {{ background:rgba(0,0,0,0.04) }}

    /* Progress bar fill */
    .progress-fill {{ animation:fillWidth 1.2s cubic-bezier(0.16,1,0.3,1) 0.3s both }}

    /* Scroll reveal */
    .reveal {{ opacity:0; transform:translateY(20px); transition:all 0.7s cubic-bezier(0.16,1,0.3,1) }}
    .reveal.visible {{ opacity:1; transform:translateY(0) }}
  </style>
  <style>{anim_css}</style>
  <style>
    .cronus-dashboard-main {{ margin-left:256px; min-height:100vh; padding:0 32px 48px; }}
    @media(max-width:768px) {{ .cronus-dashboard-main {{ margin-left:0; }} }}
  </style>
</head>
<body>
  {body}
  <script>
    // Auto-wrap: if page has a fixed sidebar, wrap non-sidebar content in dashboard-main
    (function(){{
      var aside = document.querySelector('aside[style*="position:fixed"]');
      if (!aside) return;
      var main = document.createElement('div');
      main.className = 'cronus-dashboard-main';
      var children = Array.from(document.body.children);
      children.forEach(function(child) {{
        if (child.tagName !== 'ASIDE' && child.tagName !== 'SCRIPT' && child.id !== 'cronus-audit-widget') {{
          main.appendChild(child);
        }}
      }});
      document.body.appendChild(main);
      // Move aside before main
      document.body.insertBefore(aside, main);
    }})();
  </script>
  <script>{runtime}</script>
  <script>{hmr}</script>
  <script>
    document.addEventListener('DOMContentLoaded',function(){{
      var io=new IntersectionObserver(function(entries){{
        entries.forEach(function(e){{if(e.isIntersecting){{e.target.classList.add('visible');io.unobserve(e.target)}}}})
      }},{{threshold:0.1,rootMargin:'0px 0px -40px 0px'}});
      document.querySelectorAll('.reveal').forEach(function(el){{io.observe(el)}});
      document.querySelectorAll('.stagger').forEach(function(container){{
        Array.from(container.children).forEach(function(child,i){{
          child.style.animationDelay=(0.05+i*0.06)+'s';
        }});
      }});
    }});
  </script>
  {anim_js}
</body>
</html>"##,
        app_name = app_name,
        body = body,
        anim_css = CRONUS_ANIMATIONS_CSS,
        anim_js = CRONUS_ANIMATIONS_JS,
        runtime = crate::render::CRONUS_RUNTIME_JS,
        hmr = crate::hmr::HMR_CLIENT_JS,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{LayoutNavItem, LayoutNode};
    use std::collections::HashMap;

    #[test]
    fn render_layout_declarative_has_no_hardcoded_brand_colors() {
        // Minimal layout with a brand and 2 sidebar items
        let mut sidebar_config = HashMap::new();
        sidebar_config.insert("brand".to_string(), "TestApp".to_string());

        let layout = LayoutNode {
            name: "Main".to_string(),
            sidebar_items: vec![
                LayoutNavItem {
                    label: "Home".to_string(),
                    route: "/".to_string(),
                    icon: Some("home".to_string()),
                    requires: None,
                    is_divider: false,
                },
                LayoutNavItem {
                    label: "Dashboard".to_string(),
                    route: "/dashboard".to_string(),
                    icon: Some("dashboard".to_string()),
                    requires: None,
                    is_divider: false,
                },
            ],
            sidebar_config,
            topbar_config: HashMap::new(),
        };

        let html = render_layout_declarative("TestApp", &layout, "/dashboard", "<p>body</p>");

        // The runtime must NOT inject any hardcoded brand colors.
        // #CC0000 was the old Cooud red that leaked into every user app before the 2026-04-10 fix.
        assert!(
            !html.contains("#CC0000"),
            "render_layout_declarative output contains hardcoded #CC0000; brand color leak regression"
        );
        assert!(
            !html.contains("#cc0000"),
            "render_layout_declarative output contains hardcoded #cc0000 (lowercase); brand color leak regression"
        );

        // Sanity: the output should still contain the user's brand text and routes
        assert!(html.contains("TestApp"), "brand text missing from output");
        assert!(html.contains("/dashboard"), "route missing from output");
    }
}
