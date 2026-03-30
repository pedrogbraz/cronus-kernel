#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS UI — Server-Side HTML Renderer
//!
//! Generates complete HTML pages from the AST.
//! No React, no frameworks — pure HTML + Tailwind CDN + vanilla JS.

use crate::parser::{EntityNode, FieldType, PageNode, SectionNode, ComponentNode, ComponentItemNode};
use crate::components;
use crate::render::CRONUS_RUNTIME_JS;
use crate::hmr::HMR_CLIENT_JS;
use crate::tailwind::CRONUS_TAILWIND;
use crate::animations::{CRONUS_ANIMATIONS, CRONUS_ANIMATE_JS};

// ══════════════════════════════════════════════════
// LAYOUT (wraps every page)
// ══════════════════════════════════════════════════

pub fn render_layout(app_name: &str, _pages: &[PageNode], _accent: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="pt-BR">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{app_name}</title>
  <style>{tailwind_css}</style>
  <style>{animations_css}</style>
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
      --accent: oklch(0.488 0.243 264);
      --accent-soft: oklch(0.488 0.243 264 / 12%);
      --success: oklch(0.696 0.17 162);
      --success-soft: oklch(0.696 0.17 162 / 12%);
      --warning: oklch(0.769 0.188 70);
      --danger: oklch(0.704 0.191 22);
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
</body>
</html>"#,
        app_name = app_name,
        body = body,
        tailwind_css = super::tailwind::CRONUS_TAILWIND,
        animations_css = super::animations::CRONUS_ANIMATIONS,
        runtime = super::render::CRONUS_RUNTIME_JS,
        animate_js = super::animations::CRONUS_ANIMATE_JS,
        hmr = super::hmr::HMR_CLIENT_JS,
    )
}

// ══════════════════════════════════════════════════
// PAGE RENDERER (returns inner body HTML)
// ══════════════════════════════════════════════════

pub fn render_page(page: &PageNode, entities: &[EntityNode], accent: &str) -> String {
    match page.page_type.as_str() {
        "dashboard" => render_dashboard(page, entities, accent),
        "list" => render_list(page, entities, accent),
        "form" => render_form(page, entities, accent),
        "detail" => render_list(page, entities, accent),
        "custom" => render_custom(page, accent),
        "components" => {
            // page type:components — placeholder, actual rendering happens in main.rs
            // where state.components is available
            let title = page.title.as_deref().unwrap_or("Components");
            format!(
                r#"<div style="padding:20px">
  <h1 style="font-size:16px;font-weight:400;color:var(--foreground);margin-bottom:16px">{}</h1>
  <p style="font-size:13px;color:var(--foreground-muted)">Component showcase — rendered from .cronus primitives</p>
</div>"#,
                title
            )
        }
        _ => format!(
            r#"<div style="padding:20px;color:var(--foreground-muted)">Unknown page type: {}</div>"#,
            page.page_type
        ),
    }
}

// ══════════════════════════════════════════════════
// DASHBOARD PAGE — Premium
// ══════════════════════════════════════════════════

fn render_dashboard(page: &PageNode, entities: &[EntityNode], _accent: &str) -> String {
    let _title = page.title.as_deref().unwrap_or("Dashboard");

    r##"<div>
  <!-- Header -->
  <div style="display:flex;align-items:flex-start;justify-content:space-between;padding:4px 4px 20px">
    <div>
      <h1 style="font-size:16px;font-weight:400;color:var(--foreground);letter-spacing:-0.01em">Dashboard</h1>
      <p style="font-size:11px;color:var(--foreground-muted);letter-spacing:-0.01em">Bem-vindo ao seu Dashboard, Zedd</p>
    </div>
    <div style="display:flex;gap:8px;align-items:center">
      <button style="display:flex;align-items:center;gap:6px;padding:7px 14px;font-size:12px;font-weight:500;border-radius:8px;background:var(--card);border:1px solid var(--border-strong);color:var(--foreground);cursor:pointer">
        <svg width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24"><rect x="3" y="4" width="18" height="18" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/></svg>
        Diario <svg width="10" height="10" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path d="M6 9l6 6 6-6"/></svg>
      </button>
      <button style="display:flex;align-items:center;gap:6px;padding:7px 14px;font-size:12px;font-weight:500;border-radius:8px;border:1px solid var(--border-strong);color:var(--foreground-muted);background:transparent;cursor:pointer">
        <svg width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24"><rect x="3" y="4" width="18" height="18" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/></svg>
        Este mes <svg width="10" height="10" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path d="M6 9l6 6 6-6"/></svg>
      </button>
      <button style="display:flex;align-items:center;gap:6px;padding:7px 14px;font-size:12px;font-weight:500;border-radius:8px;border:1px solid var(--border-strong);color:var(--foreground-muted);background:transparent;cursor:pointer">
        <svg width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24"><path d="M3 9l9-7 9 7v11a2 2 0 01-2 2H5a2 2 0 01-2-2z"/></svg>
        dQADW <svg width="10" height="10" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path d="M6 9l6 6 6-6"/></svg>
      </button>
    </div>
  </div>

  <!-- Two-column layout -->
  <div style="display:flex;gap:16px;align-items:flex-start">

    <!-- LEFT PANEL (280px) -->
    <div style="width:280px;flex-shrink:0;display:flex;flex-direction:column">

      <!-- Saldo card -->
      <div style="border-radius:var(--radius-card);border:1px solid var(--border);padding:20px;background:linear-gradient(135deg,var(--card-soft),var(--card));margin-bottom:12px;box-shadow:0 1px 3px rgba(0,0,0,0.1)">
        <div style="display:flex;align-items:center;gap:8px;margin-bottom:8px">
          <svg width="16" height="16" fill="none" stroke="var(--foreground-muted)" stroke-width="1.5" viewBox="0 0 24 24"><rect x="2" y="5" width="20" height="14" rx="2"/><path d="M2 10h20"/></svg>
          <span style="font-size:13px;color:var(--foreground-muted)">Saldo</span>
        </div>
        <div style="display:flex;align-items:center;gap:12px;margin-bottom:16px">
          <span style="font-size:28px;font-weight:700;color:var(--foreground);letter-spacing:-0.02em" id="balance">R$ 0,00</span>
          <span style="padding:4px 12px;font-size:11px;font-weight:600;border-radius:8px;background:var(--foreground);color:var(--background);letter-spacing:0.08em;display:flex;align-items:center;gap:4px">&#9889; BOOST</span>
        </div>
        <div style="display:flex;gap:0;margin-bottom:16px">
          <span style="font-size:12px;padding:5px 14px;border-radius:8px;background:var(--secondary);color:var(--foreground);font-weight:500">Disponivel</span>
          <span style="font-size:12px;padding:5px 14px;color:var(--foreground-subtle)">Reservado</span>
        </div>
        <div style="display:flex;gap:0;border-radius:14px;overflow:hidden;border:1px solid var(--border)">
          <button style="flex:1;padding:12px;font-size:12px;font-weight:500;color:var(--foreground);background:var(--surface-hover);border:none;display:flex;flex-direction:column;align-items:center;gap:6px;cursor:pointer">
            <svg width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24"><path d="M7 16V4m0 0L3 8m4-4l4 4M17 8v12m0 0l4-4m-4 4l-4-4"/></svg>
            Transacoes
          </button>
          <button style="flex:1;padding:12px;font-size:12px;color:var(--foreground-subtle);background:transparent;border:none;border-left:1px solid var(--border);display:flex;flex-direction:column;align-items:center;gap:6px;cursor:pointer">
            <svg width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24"><rect x="2" y="3" width="20" height="18" rx="2"/><path d="M2 9h20M9 21V9"/></svg>
            Repasses
          </button>
        </div>
      </div>

      <!-- Metricas card -->
      <div style="border-radius:var(--radius-card);border:1px solid var(--border);padding:4px 0;background:linear-gradient(135deg,var(--card-soft),var(--card));margin-bottom:12px;box-shadow:0 1px 3px rgba(0,0,0,0.1)">
        <!-- Receita bruta -->
        <div style="display:flex;align-items:center;justify-content:space-between;padding:14px 20px">
          <div style="display:flex;align-items:center;gap:12px">
            <span style="width:32px;height:32px;border-radius:10px;background:var(--success-soft);display:flex;align-items:center;justify-content:center">
              <svg width="16" height="16" fill="none" stroke="var(--success)" stroke-width="1.5" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>
            </span>
            <span style="font-size:13px;color:var(--foreground-muted)">Receita bruta</span>
          </div>
          <span style="font-size:13px;font-weight:600;color:var(--foreground)" id="dash-gross">$0.00</span>
        </div>
        <!-- Receita liquida -->
        <div style="display:flex;align-items:center;justify-content:space-between;padding:14px 20px">
          <div style="display:flex;align-items:center;gap:12px">
            <span style="width:32px;height:32px;border-radius:10px;background:var(--accent-soft);display:flex;align-items:center;justify-content:center">
              <svg width="16" height="16" fill="none" stroke="var(--accent)" stroke-width="1.5" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>
            </span>
            <span style="font-size:13px;color:var(--foreground-muted)">Receita liquida</span>
          </div>
          <span style="font-size:13px;font-weight:600;color:var(--foreground)" id="dash-net">$0.00</span>
        </div>
        <!-- Pedidos -->
        <div style="display:flex;align-items:center;justify-content:space-between;padding:14px 20px">
          <div style="display:flex;align-items:center;gap:12px">
            <span style="width:32px;height:32px;border-radius:10px;background:oklch(0.627 0.265 303/12%);display:flex;align-items:center;justify-content:center">
              <svg width="16" height="16" fill="none" stroke="oklch(0.627 0.265 303)" stroke-width="1.5" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>
            </span>
            <span style="font-size:13px;color:var(--foreground-muted)">Pedidos</span>
          </div>
          <span style="font-size:13px;font-weight:600;color:var(--foreground)" id="dash-orders">0</span>
        </div>
        <!-- Ticket medio -->
        <div style="display:flex;align-items:center;justify-content:space-between;padding:14px 20px">
          <div style="display:flex;align-items:center;gap:12px">
            <span style="width:32px;height:32px;border-radius:10px;background:oklch(0.769 0.188 70 / 12%);display:flex;align-items:center;justify-content:center">
              <svg width="16" height="16" fill="none" stroke="var(--warning)" stroke-width="1.5" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>
            </span>
            <span style="font-size:13px;color:var(--foreground-muted)">Ticket medio</span>
          </div>
          <span style="font-size:13px;font-weight:600;color:var(--foreground)" id="dash-ticket">$0.00</span>
        </div>
      </div>

      <!-- Metas de vendas card -->
      <div style="border-radius:var(--radius-card);border:1px solid var(--border);padding:20px;background:linear-gradient(135deg,var(--card-soft),var(--card));box-shadow:0 1px 3px rgba(0,0,0,0.1)">
        <p style="font-size:10px;font-weight:600;color:var(--foreground-subtle);text-transform:uppercase;letter-spacing:0.1em;margin-bottom:16px">METAS DE VENDAS</p>
        <div style="display:flex;align-items:center;gap:12px;margin-bottom:4px">
          <span style="width:32px;height:32px;border-radius:10px;background:var(--accent-soft);display:flex;align-items:center;justify-content:center;font-size:14px">&#127942;</span>
          <div>
            <p style="font-size:14px;font-weight:500;color:var(--foreground)">Iniciante</p>
            <p style="font-size:11px;color:var(--foreground-subtle)">Proxima: Bronze</p>
          </div>
          <svg width="14" height="14" fill="none" stroke="var(--foreground-subtle)" stroke-width="1.5" viewBox="0 0 24 24" style="margin-left:auto"><path d="M9 18l6-6-6-6"/></svg>
        </div>
        <div style="display:flex;justify-content:space-between;font-size:11px;color:var(--foreground-subtle);margin:12px 0 6px">
          <span>R$ 0,00</span><span>R$ 100.000,00</span>
        </div>
        <div style="height:4px;border-radius:2px;background:var(--secondary);overflow:hidden">
          <div style="height:100%;width:0%;background:var(--accent);border-radius:2px;transition:width 1s" id="goal-bar"></div>
        </div>
        <p style="font-size:11px;color:var(--foreground-subtle);margin-top:8px" id="dash-goal-pct">0%</p>
        <div style="margin-top:20px;border-top:1px solid var(--border);padding-top:16px">
          <p style="font-size:11px;color:var(--foreground-subtle);margin-bottom:12px">Premiacoes</p>
          <div style="display:flex;gap:0;justify-content:space-between">
            <div style="text-align:center"><span style="display:block;width:28px;height:28px;border-radius:8px;background:var(--surface-hover);margin:0 auto 4px;display:flex;align-items:center;justify-content:center;font-size:11px;color:var(--foreground-subtle)">&#127941;</span><span style="font-size:10px;color:var(--foreground-subtle)">R$100k</span></div>
            <div style="text-align:center"><span style="display:block;width:28px;height:28px;border-radius:8px;background:var(--surface-hover);margin:0 auto 4px;display:flex;align-items:center;justify-content:center;font-size:11px;color:var(--foreground-subtle)">&#129352;</span><span style="font-size:10px;color:var(--foreground-subtle)">R$1M</span></div>
            <div style="text-align:center"><span style="display:block;width:28px;height:28px;border-radius:8px;background:var(--surface-hover);margin:0 auto 4px;display:flex;align-items:center;justify-content:center;font-size:11px;color:var(--foreground-subtle)">&#129351;</span><span style="font-size:10px;color:var(--foreground-subtle)">R$10M</span></div>
            <div style="text-align:center"><span style="display:block;width:28px;height:28px;border-radius:8px;background:var(--surface-hover);margin:0 auto 4px;display:flex;align-items:center;justify-content:center;font-size:11px;color:var(--foreground-subtle)">&#128142;</span><span style="font-size:10px;color:var(--foreground-subtle)">R$50M</span></div>
          </div>
        </div>
      </div>

    </div>

    <!-- RIGHT PANEL — Transaction Chart -->
    <div style="flex:1;border-radius:var(--radius-card);border:1px solid var(--border);padding:20px;background:linear-gradient(135deg,var(--card),var(--background));display:flex;flex-direction:column;box-shadow:0 1px 3px rgba(0,0,0,0.1)">
      <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:16px">
        <div style="display:flex;align-items:center;gap:8px">
          <span style="border-left:3px solid var(--accent);padding-left:10px;font-size:11px;font-weight:600;color:var(--foreground-muted);text-transform:uppercase;letter-spacing:0.08em">TRANSACOES</span>
        </div>
        <div style="display:flex;align-items:center;gap:12px;font-size:11px;color:var(--foreground-subtle)">
          <span>&#8592; ONTEM</span>
          <span style="color:var(--foreground-muted)" id="dash-yesterday-total">$0.00</span>
          <span>&#8212;0%</span>
        </div>
      </div>
      <div style="display:flex;align-items:center;gap:8px;margin-bottom:24px">
        <span style="width:8px;height:8px;border-radius:50%;background:var(--accent)"></span>
        <span style="font-size:11px;color:var(--foreground-muted);text-transform:uppercase;letter-spacing:0.05em">HOJE</span>
        <span style="font-size:14px;font-weight:600;color:var(--foreground)" id="today-total">$0.00</span>
      </div>
      <!-- Chart area with grid lines -->
      <div style="flex:1;position:relative;min-height:300px">
        <div style="position:absolute;top:0;left:0;right:0;border-top:1px solid var(--surface-hover)"></div>
        <div style="position:absolute;top:25%;left:0;right:0;border-top:1px solid var(--surface-hover)"></div>
        <div style="position:absolute;top:50%;left:0;right:0;border-top:1px solid var(--surface-hover)"></div>
        <div style="position:absolute;top:75%;left:0;right:0;border-top:1px solid var(--surface-hover)"></div>
        <div style="position:absolute;bottom:0;left:0;right:0;border-top:1px solid var(--surface-hover)"></div>
        <!-- Blue dot at bottom center -->
        <div style="position:absolute;bottom:0;left:50%;transform:translateX(-50%);width:10px;height:10px;border-radius:50%;background:var(--accent);box-shadow:0 0 10px var(--accent-soft)"></div>
      </div>
      <!-- X axis -->
      <div style="display:flex;justify-content:space-between;padding-top:8px;font-size:10px;color:var(--foreground-subtle)">
        <span>00:00</span><span>04:00</span><span>08:00</span><span>12:00</span><span>16:00</span><span>20:00</span><span>23:59</span>
      </div>
    </div>

  </div>
</div>

<script>
(function() {
  function formatMoney(cents) {
    var val = (cents || 0) / 100;
    return '$' + val.toFixed(2);
  }

  function formatBRL(cents) {
    var val = (cents || 0) / 100;
    return 'R$ ' + val.toFixed(2).replace('.', ',');
  }

  function loadData() {
    fetch('/api/orders').then(function(r) { return r.json(); }).then(function(data) {
      var orders = Array.isArray(data) ? data : [];

      var now = new Date();
      var todayOrders = orders.filter(function(o) {
        var d = new Date(o.createdAt || o.created_at || 0);
        return d.toDateString() === now.toDateString();
      });

      var gross = 0;
      todayOrders.forEach(function(o) { gross += (o.price || o.amount || 0); });
      var net = Math.round(gross * 0.9);
      var count = todayOrders.length;
      var ticket = count > 0 ? Math.round(gross / count) : 0;

      var el;
      el = document.getElementById('dash-gross'); if (el) el.textContent = formatMoney(gross);
      el = document.getElementById('dash-net'); if (el) el.textContent = formatMoney(net);
      el = document.getElementById('dash-orders'); if (el) el.textContent = count;
      el = document.getElementById('dash-ticket'); if (el) el.textContent = formatMoney(ticket);
      el = document.getElementById('balance'); if (el) el.textContent = formatBRL(net);
      el = document.getElementById('today-total'); if (el) el.textContent = formatMoney(gross);

      // Goal bar (R$ 100.000 = 10000000 cents)
      var goalTarget = 10000000;
      var pct = Math.min(100, Math.round((gross / goalTarget) * 100));
      el = document.getElementById('goal-bar'); if (el) el.style.width = pct + '%';
      el = document.getElementById('dash-goal-pct'); if (el) el.textContent = pct + '%';

    }).catch(function() {});
  }

  loadData();
})();
</script>"##.to_string()
}

// ══════════════════════════════════════════════════
// LIST PAGE — Premium
// ══════════════════════════════════════════════════

fn render_list(page: &PageNode, entities: &[EntityNode], accent: &str) -> String {
    let entity_name = page.entity.as_deref().unwrap_or("");
    let entity = entities.iter().find(|e| e.name.eq_ignore_ascii_case(entity_name));
    let title = page.title.as_deref().unwrap_or(entity_name);
    let lower = entity_name.to_lowercase();

    // Product pages — will use card grid in future
    // let is_product = title.to_lowercase().contains("produto");
    // if is_product { return render_product_grid(title, &lower, entity); }

    let field_names: Vec<String> = match entity {
        Some(e) => e.fields.iter()
            .filter(|f| f.name != "id" && f.name != "createdAt" && f.name != "updatedAt")
            .take(5)
            .map(|f| f.name.clone())
            .collect(),
        None => vec!["id".into()],
    };

    // Find enum fields for status badge coloring
    let enum_fields: Vec<String> = match entity {
        Some(e) => e.fields.iter()
            .filter(|f| f.field_type == FieldType::Enum)
            .map(|f| f.name.clone())
            .collect(),
        None => vec![],
    };

    let headers: String = field_names.iter()
        .map(|n| format!(
            r#"<th class="text-left px-5 py-3 font-mono text-[10px] uppercase tracking-[0.15em] text-neutral-500">{}</th>"#, n
        ))
        .collect::<Vec<_>>()
        .join("\n            ");

    let fields_js: String = field_names.iter()
        .map(|n| format!(r#"'{}'"#, n))
        .collect::<Vec<_>>()
        .join(",");

    let enum_fields_js: String = enum_fields.iter()
        .map(|n| format!(r#"'{}'"#, n))
        .collect::<Vec<_>>()
        .join(",");

    let headers_oklch: String = field_names.iter()
        .map(|n| format!(
            r#"<th style="padding:10px 16px;text-align:left;font-size:11px;font-weight:500;color:var(--foreground-muted);text-transform:uppercase;letter-spacing:0.05em">{}</th>"#, n
        ))
        .collect::<Vec<_>>()
        .join("\n            ");

    format!(
        r##"<div style="flex:1;min-height:0;display:flex;flex-direction:column">
  <div style="display:flex;align-items:center;justify-content:space-between;padding:0 4px 16px">
    <div>
      <h1 style="font-size:16px;font-weight:400;color:var(--foreground)">{title}</h1>
      <span style="font-size:11px;color:var(--foreground-muted)" id="count-label">Carregando...</span>
    </div>
    <div style="display:flex;gap:8px;align-items:center">
      <input id="search-input" type="search" placeholder="Buscar..."
        style="padding:6px 12px;font-size:13px;border-radius:10px;background:var(--card);border:1px solid var(--border-strong);color:var(--foreground);outline:none;width:200px">
      <button onclick="document.getElementById('new-modal').style.display='flex'"
        style="padding:6px 16px;font-size:13px;font-weight:500;border-radius:10px;background:var(--foreground);color:var(--background);border:none;cursor:pointer">Novo</button>
    </div>
  </div>

  <div style="border-radius:var(--radius-card);border:1px solid var(--border);overflow:hidden;flex:1;display:flex;flex-direction:column">
    <table style="width:100%;border-collapse:collapse">
      <thead>
        <tr style="border-bottom:1px solid var(--border)">
          {headers_oklch}
          <th style="width:40px"></th>
        </tr>
      </thead>
      <tbody id="table-body"></tbody>
    </table>
    <div id="empty-state" style="display:none;padding:48px;text-align:center">
      <p style="font-size:13px;color:var(--foreground-muted)">Nenhum registro encontrado</p>
    </div>
    <div style="margin-top:auto;padding:10px 16px;border-top:1px solid var(--border);font-size:11px;color:var(--foreground-subtle);text-align:right" id="pagination-label"></div>
  </div>
</div>

<script>
(function() {{
  var fields = [{fields_js}];
  var enumFields = [{enum_fields_js}];
  var lower = '{lower}';
  var allData = [];

  function badge(val, field) {{
    if (enumFields.indexOf(field)===-1) return '<span style="font-size:13px;color:var(--foreground-muted)">'+(val||'\u2014')+'</span>';
    var v=(val||'').toLowerCase();
    var isPaid=v==='active'||v==='paid'||v==='completed'||v==='succeeded'||v==='approved';
    var isFail=v==='failed'||v==='cancelled'||v==='rejected'||v==='error';
    var dotColor=isPaid?'oklch(0.696 0.17 162)':isFail?'oklch(0.704 0.191 22)':'oklch(0.769 0.188 70)';
    var textColor=isPaid?'oklch(0.696 0.17 162)':isFail?'oklch(0.704 0.191 22)':'oklch(0.769 0.188 70)';
    var bgColor=isPaid?'oklch(0.696 0.17 162/8%)':isFail?'oklch(0.704 0.191 22/8%)':'oklch(0.769 0.188 70/8%)';
    return '<span style="display:inline-flex;align-items:center;gap:6px;padding:2px 10px;border-radius:20px;font-size:11px;font-weight:500;background:'+bgColor+';color:'+textColor+';border:1px solid '+dotColor.replace(')','/20%)')+'"><span style="width:5px;height:5px;border-radius:50%;background:'+dotColor+'"></span>'+(val||'\u2014')+'</span>';
  }}

  function renderRows(data) {{
    var tbody=document.getElementById('table-body');
    var empty=document.getElementById('empty-state');
    var countLabel=document.getElementById('count-label');
    var pagLabel=document.getElementById('pagination-label');
    if(!data.length){{tbody.innerHTML='';empty.style.display='block';countLabel.textContent='0 registros';pagLabel.textContent='';return}}
    empty.style.display='none';
    countLabel.textContent=data.length+' registro'+(data.length!==1?'s':'');
    pagLabel.textContent='Mostrando 1-'+data.length+' de '+data.length;
    tbody.innerHTML=data.map(function(row,i){{
      var cells=fields.map(function(f){{
        return '<td style="padding:10px 16px;font-size:13px;color:var(--foreground-muted)">'+badge(row[f],f)+'</td>';
      }}).join('');
      var bg=i%2===0?'':'background:var(--surface-hover)';
      return '<tr style="border-bottom:1px solid var(--surface-hover);'+bg+';cursor:pointer" onmouseover="this.style.background=\'var(--surface-hover)\'">'+cells+
        '<td style="padding:10px 8px;text-align:center"><svg width="14" height="14" fill="none" stroke="var(--foreground-subtle)" stroke-width="1.5" viewBox="0 0 24 24"><path d="M9 18l6-6-6-6"/></svg></td></tr>';
    }}).join('');
  }}

  function load(){{
    fetch('/api/'+lower+'s').then(function(r){{return r.json()}}).then(function(d){{
      allData=Array.isArray(d)?d:[];renderRows(allData);
    }}).catch(function(){{renderRows([])}});
  }}

  document.getElementById('search-input').addEventListener('input',function(e){{
    var q=e.target.value.toLowerCase();
    renderRows(allData.filter(function(row){{
      return fields.some(function(f){{return(row[f]||'').toString().toLowerCase().indexOf(q)!==-1}});
    }}));
  }});

  load();
}})();
</script>"##,
        title = title,
        headers_oklch = headers_oklch,
        fields_js = fields_js,
        enum_fields_js = enum_fields_js,
        lower = lower,
    )
}

// ══════════════════════════════════════════════════
// FORM PAGE
// ══════════════════════════════════════════════════

fn render_product_grid(title: &str, lower: &str, _entity: Option<&EntityNode>) -> String {
    format!(r##"<div>
  <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:16px">
    <h1 style="font-size:20px;font-weight:600;color:var(--foreground)">{title}</h1>
    <div style="display:flex;gap:8px;align-items:center">
      <input id="product-search" type="text" placeholder="Buscar..." style="padding:6px 14px;font-size:12px;border-radius:10px;border:1px solid var(--border-strong);background:var(--card);color:var(--foreground);outline:none;width:200px">
      <a href="/{lower}/new" style="padding:6px 18px;font-size:12px;font-weight:500;border-radius:10px;background:var(--foreground);color:var(--background);text-decoration:none">+ Novo</a>
    </div>
  </div>
  <div id="product-grid" style="display:grid;grid-template-columns:repeat(auto-fill,minmax(220px,1fr));gap:16px"></div>
  <div id="empty-products" style="display:none;text-align:center;padding:60px 0;color:var(--foreground-subtle);font-size:13px">Nenhum produto</div>
</div>
<script>
(function(){{
  var lower='{lower}';var allData=[];
  function renderCards(data){{
    var g=document.getElementById('product-grid');var e=document.getElementById('empty-products');
    if(!data.length){{g.innerHTML='';e.style.display='block';return}}
    e.style.display='none';
    g.innerHTML=data.map(function(row){{
      var name=row.name||row.title||'Sem nome';var s=(row.status||row.live||'active').toString().toLowerCase();
      var ok=s==='active'||s==='true'||s==='ativo';var dc=ok?'oklch(0.696 0.17 162)':'oklch(0.5 0 0)';var lb=ok?'Ativo':'Rascunho';
      return '<div style="border-radius:18px;border:1px solid var(--border);overflow:hidden;background:var(--card);cursor:pointer">'
        +'<div style="height:140px;background:var(--surface-hover);display:flex;align-items:center;justify-content:center"><svg width="32" height="32" fill="none" stroke="var(--foreground-subtle)" stroke-width="1" viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="M21 15l-5-5L5 21"/></svg></div>'
        +'<div style="padding:14px"><p style="font-size:14px;font-weight:500;color:var(--foreground);margin:0 0 4px">'+name+'</p>'
        +'<span style="display:inline-flex;align-items:center;gap:4px;font-size:11px;padding:2px 8px;border-radius:12px;background:'+dc.replace(')','/10%)')+';color:'+dc+'"><span style="width:4px;height:4px;border-radius:50%;background:'+dc+'"></span>'+lb+'</span>'
        +'</div></div>';
    }}).join('');
  }}
  fetch('/api/'+lower+'s').then(function(r){{return r.json()}}).then(function(d){{allData=Array.isArray(d)?d:[];renderCards(allData)}}).catch(function(){{renderCards([])}});
  document.getElementById('product-search').addEventListener('input',function(e){{
    var q=e.target.value.toLowerCase();renderCards(allData.filter(function(r){{return JSON.stringify(r).toLowerCase().indexOf(q)!==-1}}));
  }});
}})();
</script>"##, title=title, lower=lower)
}

fn render_form(page: &PageNode, entities: &[EntityNode], accent: &str) -> String {
    let _ = accent; // oklch monocromatic — no accent colors in forms
    let entity_name = page.entity.as_deref().unwrap_or("");
    let entity = entities.iter().find(|e| e.name.eq_ignore_ascii_case(entity_name));
    let title = page.title.as_deref().unwrap_or(entity_name);
    let lower = entity_name.to_lowercase();

    // Determine form context label
    let context_label = if title.to_lowercase().contains("sign") || title.to_lowercase().contains("log") {
        "Authentication"
    } else {
        entity_name
    };

    let inputs: Vec<String> = match entity {
        Some(e) => e.fields.iter()
            .filter(|f| f.name != "id" && f.name != "createdAt" && f.name != "updatedAt")
            .map(|f| {
                let input_type = match f.field_type {
                    FieldType::Email => "email",
                    FieldType::Number | FieldType::Money | FieldType::Percentage => "number",
                    FieldType::Boolean => "checkbox",
                    FieldType::Date => "date",
                    FieldType::Url => "url",
                    FieldType::Phone => "tel",
                    _ => if f.sensitive { "password" } else { "text" },
                };
                let required = if f.required { " required" } else { "" };
                let placeholder = match f.field_type {
                    FieldType::Email => "you@example.com",
                    FieldType::Phone => "+1 (555) 000-0000",
                    FieldType::Url => "https://",
                    _ => "",
                };
                let ph = if !placeholder.is_empty() { format!(r#" placeholder="{}""#, placeholder) } else { String::new() };

                if f.field_type == FieldType::Text {
                    format!(
                        r#"<div>
      <label class="block text-xs font-medium mb-1.5" style="color:var(--foreground-muted)">{name}</label>
      <textarea name="{name}" rows="3" class="w-full px-3 py-2 text-sm outline-none" style="background:var(--card);border:1px solid var(--border-strong);border-radius:0.875rem;color:var(--foreground) resize-none"{required}></textarea>
    </div>"#,
                        name = f.name, required = required,
                    )
                } else if f.field_type == FieldType::Boolean {
                    format!(
                        r#"<div class="flex items-center gap-3 py-2">
      <input type="checkbox" name="{name}" id="{name}" class="w-4 h-4 rounded" style="accent-color:var(--foreground)">
      <label for="{name}" class="font-mono text-[10px] uppercase tracking-[0.15em] text-neutral-500">{name}</label>
    </div>"#,
                        name = f.name,
                    )
                } else if f.field_type == FieldType::Enum {
                    let options: Vec<String> = f.enum_values.as_ref()
                        .map(|vals| vals.iter()
                            .map(|v| format!(r#"<option value="{v}" class="bg-neutral-900">{v}</option>"#, v = v))
                            .collect())
                        .unwrap_or_default();
                    format!(
                        r#"<div>
      <label class="block text-xs font-medium mb-1.5" style="color:var(--foreground-muted)">{name}</label>
      <select name="{name}" class="w-full px-3 py-2 text-sm outline-none appearance-none" style="background:var(--card);border:1px solid var(--border-strong);border-radius:0.875rem;color:var(--foreground)"{required}>
        <option value="" class="bg-neutral-900">Select...</option>
        {options}
      </select>
    </div>"#,
                        name = f.name, required = required,
                        options = options.join("\n        "),
                    )
                } else {
                    format!(
                        r#"<div>
      <label class="block text-xs font-medium mb-1.5" style="color:var(--foreground-muted)">{name}</label>
      <input type="{input_type}" name="{name}" class="w-full px-3 py-2 text-sm outline-none" style="background:var(--card);border:1px solid var(--border-strong);border-radius:0.875rem;color:var(--foreground)"{required}{ph}>
    </div>"#,
                        name = f.name, input_type = input_type,
                        required = required, ph = ph,
                    )
                }
            })
            .collect(),
        None => vec![],
    };

    format!(
        r##"<div class="max-w-lg mx-auto mt-12">
  <div class="flex items-center gap-3 mb-8">
    <p class="font-mono text-[10px] uppercase tracking-[0.2em]" style="color:var(--foreground-muted)">// {context_label}</p>
    <div class="flex-1 h-px bg-neutral-800/50"></div>
  </div>

  <h1 class="text-3xl font-bold tracking-tight mb-8">{title}</h1>

  <form id="entity-form" class="space-y-5">
    {inputs}

    <div id="form-msg" class="hidden rounded-lg px-4 py-3 text-sm font-mono"></div>

    <button type="submit" id="submit-btn"
      class="w-full py-2.5 text-sm font-medium" style="background:var(--foreground);color:var(--background);border-radius:0.875rem">
      {submit_label}
    </button>
  </form>
</div>

<script>
document.getElementById('entity-form').addEventListener('submit', function(e) {{
  e.preventDefault();
  var form = e.target;
  var btn = document.getElementById('submit-btn');
  var msg = document.getElementById('form-msg');
  btn.disabled = true;
  btn.textContent = 'Saving...';

  var data = {{}};
  new FormData(form).forEach(function(v, k) {{ data[k] = v; }});
  data.id = crypto.randomUUID ? crypto.randomUUID() : Date.now().toString(36);

  fetch('/api/{lower}s', {{
    method: 'POST',
    headers: {{ 'Content-Type': 'application/json' }},
    body: JSON.stringify(data)
  }}).then(function(r) {{
    msg.classList.remove('hidden');
    btn.disabled = false;
    btn.textContent = '{submit_label}';
    if (r.ok) {{
      msg.className = 'rounded-lg px-4 py-3 text-sm font-mono bg-emerald-500/10 text-emerald-400 border border-emerald-500/20';
      msg.textContent = '\u2713 Saved successfully';
      form.reset();
      setTimeout(function() {{ msg.classList.add('hidden'); }}, 3000);
    }} else {{
      msg.className = 'rounded-lg px-4 py-3 text-sm font-mono bg-red-500/10 text-red-400 border border-red-500/20';
      msg.textContent = '\u2717 Error saving — please try again';
    }}
  }}).catch(function() {{
    btn.disabled = false;
    btn.textContent = '{submit_label}';
    msg.classList.remove('hidden');
    msg.className = 'rounded-lg px-4 py-3 text-sm font-mono bg-red-500/10 text-red-400 border border-red-500/20';
    msg.textContent = '\u2717 Connection error';
  }});
}});
</script>"##,
        title = title,
        context_label = context_label,
        inputs = inputs.join("\n    "),
        lower = lower,
        submit_label = if context_label == "Authentication" { "Sign In" } else { "Save" },
    )
}

// ══════════════════════════════════════════════════
// DETAIL PAGE (single entity record view)
// ══════════════════════════════════════════════════

fn render_detail(page: &PageNode, _entities: &[EntityNode], accent: &str) -> String {
    let entity_name = page.entity.as_deref().unwrap_or("");
    let title = page.title.as_deref().unwrap_or(entity_name);
    let lower = entity_name.to_lowercase();

    format!(
        r##"<div class="max-w-2xl mx-auto">
  <div class="flex items-center gap-3 mb-8">
    <a href="/{lower}s" class="text-neutral-500 hover:text-white text-sm transition-colors">&larr; {title}</a>
    <div class="flex-1 h-px bg-neutral-800/50"></div>
    <p class="font-mono text-[10px] uppercase tracking-[0.2em] text-{accent}-500">// detail</p>
  </div>
  <h1 class="text-3xl font-bold tracking-tight mb-8" id="detail-title">{title}</h1>
  <div class="bg-neutral-950 border border-neutral-800/60 rounded-lg overflow-hidden" id="detail-body">
    <div class="p-8 text-center text-neutral-600 font-mono text-sm">Loading...</div>
  </div>
  <div class="mt-6 flex items-center gap-3">
    <a href="/{lower}s" class="px-4 py-2 bg-neutral-800 text-neutral-300 hover:bg-neutral-700 rounded text-sm transition-colors">&larr; Back</a>
    <button onclick="cronusDelete()" class="px-4 py-2 bg-red-500/10 text-red-400 border border-red-500/20 hover:bg-red-500/20 rounded text-sm transition-colors">Delete</button>
  </div>
</div>
<script>
(function(){{
  var id=window.location.pathname.split('/').pop();
  fetch('/api/{lower}s/'+id).then(function(r){{return r.json()}}).then(function(d){{
    if(d.error){{document.getElementById('detail-body').innerHTML='<div class="p-8 text-center text-red-400 font-mono">Not found</div>';return;}}
    var h='';
    Object.keys(d).forEach(function(k){{
      if(k==='updated_at')return;
      var v=d[k];if(v===null||v===undefined)v='\u2014';
      h+='<div class="flex items-center justify-between px-6 py-4 border-b border-neutral-800/30 hover:bg-neutral-900/30 transition-colors">';
      h+='<span class="font-mono text-[10px] uppercase tracking-[0.15em] text-neutral-500">'+k+'</span>';
      h+='<span class="text-sm text-neutral-300 font-mono">'+v+'</span>';
      h+='</div>';
    }});
    document.getElementById('detail-body').innerHTML=h;
    if(d.name||d.title)document.getElementById('detail-title').textContent=d.name||d.title;
  }}).catch(function(){{document.getElementById('detail-body').innerHTML='<div class="p-8 text-center text-red-400">Error</div>';}});
  window.cronusDelete=function(){{if(confirm('Delete?'))fetch('/api/{lower}s/'+id,{{method:'DELETE'}}).then(function(){{window.location.href='/{lower}s';}});}};
}})();
</script>"##,
        title = title, lower = lower, accent = accent,
    )
}

// ══════════════════════════════════════════════════
// CUSTOM PAGE (sections: hero, features, pricing)
// ══════════════════════════════════════════════════

fn render_custom(page: &PageNode, accent: &str) -> String {
    let sections: Vec<String> = page.sections.iter()
        .map(|s| render_section(s, accent))
        .collect();
    sections.join("\n")
}

fn render_section(section: &SectionNode, accent: &str) -> String {
    match section.section_type.as_str() {
        "hero" => render_hero(section, accent),
        "features" => render_features(section, accent),
        "pricing" => render_pricing(section, accent),
        "cta" => render_cta(section, accent),
        "faq" => render_faq(section, accent),
        "stats" => render_stats(section, accent),
        _ => render_generic_section(section, accent),
    }
}

fn render_hero(section: &SectionNode, accent: &str) -> String {
    let title = section.title.as_deref().unwrap_or("Build Something Amazing");
    let subtitle = section.subtitle.as_deref().unwrap_or("The next generation platform for modern teams.");
    let cta = section.config.get("cta").map(|s| s.as_str()).unwrap_or("Get Started");

    format!(
        r#"<section class="relative overflow-hidden">
  <div class="absolute inset-0 pointer-events-none">
    <div class="absolute top-0 left-1/2 -translate-x-1/2 w-[800px] h-[600px] opacity-[0.07]" style="background:radial-gradient(ellipse,rgba(245,158,11,0.5),transparent 70%)"></div>
  </div>
  <div class="relative max-w-7xl mx-auto px-6 pt-32 pb-24">
    <div class="max-w-3xl">
      <span class="anim inline-flex items-center gap-2 font-mono text-[10px] tracking-[0.2em] text-emerald-500 uppercase mb-8">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
        CLUSTER: READY // SYSTEM ONLINE
      </span>
      <h1 class="anim anim-d1 text-5xl md:text-7xl font-extrabold uppercase tracking-tight leading-[0.9]">
        <span class="text-white">{title_line1}</span><br>
        <span class="text-neutral-600">{title_line2}</span>
      </h1>
      <p class="anim anim-d2 mt-8 text-lg text-neutral-400 max-w-xl leading-relaxed">{subtitle}</p>
      <div class="anim anim-d3 mt-10 flex items-center gap-4">
        <a href="/signup" class="px-8 py-3.5 bg-{accent}-500 text-black font-semibold text-sm uppercase tracking-wider hover:bg-{accent}-400 transition-colors">{cta}</a>
        <a href="/docs" class="px-8 py-3.5 border border-neutral-700 text-neutral-300 text-sm uppercase tracking-wider hover:border-neutral-500 hover:text-white transition-all">Documentation</a>
      </div>
    </div>
  </div>
</section>"#,
        title_line1 = title.split_whitespace().take(2).collect::<Vec<_>>().join(" "),
        title_line2 = title.split_whitespace().skip(2).collect::<Vec<_>>().join(" "),
        subtitle = subtitle, accent = accent, cta = cta,
    )
}

fn render_features(section: &SectionNode, accent: &str) -> String {
    let title = section.title.as_deref().unwrap_or("Features");
    let _ = accent; // available for future use

    let items: Vec<String> = section.items.iter().map(|item| {
        let name = item.get("title").or_else(|| item.get("name")).map(|s| s.as_str()).unwrap_or("Feature");
        let desc = item.get("description").or_else(|| item.get("desc")).map(|s| s.as_str()).unwrap_or("");
        let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("*");
        format!(
            r#"<div class="bg-neutral-900 border border-neutral-800 rounded-lg p-6">
  <div class="text-2xl mb-3">{icon}</div>
  <h3 class="font-semibold text-white">{name}</h3>
  <p class="mt-1 text-sm text-neutral-400">{desc}</p>
</div>"#,
            icon = icon, name = name, desc = desc,
        )
    }).collect();

    format!(
        r#"<section class="py-16">
  <h2 class="text-3xl font-bold text-center mb-10">{title}</h2>
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
    {items}
  </div>
</section>"#,
        title = title,
        items = items.join("\n    "),
    )
}

fn render_pricing(section: &SectionNode, accent: &str) -> String {
    let title = section.title.as_deref().unwrap_or("Pricing");

    let plans: Vec<String> = section.plans.iter().map(|plan| {
        let features: Vec<String> = plan.features.iter()
            .map(|f| format!(r#"<li class="flex items-center gap-2 text-sm text-neutral-300"><span class="text-{}-400">✓</span> {}</li>"#, accent, f))
            .collect();

        let border = if plan.featured {
            format!("border-{}-500", accent)
        } else {
            "border-neutral-800".to_string()
        };
        let badge = if plan.featured {
            format!(r#"<span class="text-xs bg-{}-600 text-white px-2 py-0.5 rounded-full">Popular</span>"#, accent)
        } else {
            String::new()
        };

        format!(
            r#"<div class="bg-neutral-900 border {border} rounded-lg p-6 flex flex-col">
  <div class="flex items-center justify-between mb-4">
    <h3 class="font-semibold text-white">{name}</h3>
    {badge}
  </div>
  <p class="text-3xl font-bold text-white mb-6">{price}</p>
  <ul class="space-y-2 mb-6 flex-1">{features}</ul>
  <button class="w-full bg-{accent}-600 hover:bg-{accent}-500 text-white py-2 rounded text-sm font-medium transition">Choose Plan</button>
</div>"#,
            border = border, name = plan.name, badge = badge,
            price = plan.price, features = features.join("\n    "),
            accent = accent,
        )
    }).collect();

    format!(
        r#"<section class="py-16">
  <h2 class="text-3xl font-bold text-center mb-10">{title}</h2>
  <div class="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-4xl mx-auto">
    {plans}
  </div>
</section>"#,
        title = title,
        plans = plans.join("\n    "),
    )
}

fn render_cta(section: &SectionNode, accent: &str) -> String {
    let title = section.title.as_deref().unwrap_or("Get Started");
    let subtitle = section.subtitle.as_deref().unwrap_or("");
    let cta_text = section.config.get("cta_text").map(|s| s.as_str()).unwrap_or("Get Started");
    let cta_link = section.config.get("cta_link").map(|s| s.as_str()).unwrap_or("/signup");

    format!(
        r#"<section class="py-24 relative overflow-hidden">
  <div class="absolute inset-0 pointer-events-none">
    <div class="absolute bottom-0 left-1/2 -translate-x-1/2 w-[600px] h-[400px] opacity-[0.05]" style="background:radial-gradient(ellipse,rgba(59,130,246,0.5),transparent 70%)"></div>
  </div>
  <div class="relative max-w-3xl mx-auto px-6 text-center">
    <h2 class="text-4xl font-bold tracking-tight mb-4">{title}</h2>
    <p class="text-lg text-neutral-400 mb-8 max-w-xl mx-auto">{subtitle}</p>
    <a href="{cta_link}" class="inline-block px-8 py-3.5 bg-{accent}-500 text-black font-semibold text-sm uppercase tracking-wider hover:bg-{accent}-400 transition-colors">{cta_text}</a>
  </div>
</section>"#,
        title = title, subtitle = subtitle, cta_text = cta_text, cta_link = cta_link, accent = accent,
    )
}

fn render_faq(section: &SectionNode, accent: &str) -> String {
    let title = section.title.as_deref().unwrap_or("FAQ");

    let items: Vec<String> = section.items.iter().map(|item| {
        let q = item.get("title").map(|s| s.as_str()).unwrap_or("Question");
        let a = item.get("description").map(|s| s.as_str()).unwrap_or("");
        format!(
            r#"<details class="group border-b border-neutral-800">
  <summary class="flex items-center justify-between py-4 cursor-pointer text-sm font-medium text-white hover:text-{accent}-400 transition-colors">
    {q}
    <span class="text-neutral-600 group-open:rotate-45 transition-transform text-lg">+</span>
  </summary>
  <p class="pb-4 text-sm text-neutral-400 leading-relaxed">{a}</p>
</details>"#,
            q = q, a = a, accent = accent,
        )
    }).collect();

    format!(
        r#"<section class="py-16 max-w-2xl mx-auto px-6">
  <h2 class="text-3xl font-bold text-center mb-10">{title}</h2>
  <div class="divide-y divide-neutral-800">
    {items}
  </div>
</section>"#,
        title = title, items = items.join("\n    "),
    )
}

fn render_stats(section: &SectionNode, accent: &str) -> String {
    let title = section.title.as_deref().unwrap_or("Stats");

    let items: Vec<String> = section.items.iter().map(|item| {
        let label = item.get("title").map(|s| s.as_str()).unwrap_or("Stat");
        let value = item.get("description").map(|s| s.as_str()).unwrap_or("0");
        format!(
            r#"<div class="text-center">
  <p class="text-4xl font-bold text-{accent}-400 tabular-nums">{value}</p>
  <p class="mt-2 text-sm text-neutral-500 font-mono uppercase tracking-wider">{label}</p>
</div>"#,
            label = label, value = value, accent = accent,
        )
    }).collect();

    format!(
        r#"<section class="py-16">
  <h2 class="text-3xl font-bold text-center mb-10">{title}</h2>
  <div class="grid grid-cols-2 md:grid-cols-4 gap-8 max-w-4xl mx-auto">
    {items}
  </div>
</section>"#,
        title = title, items = items.join("\n    "),
    )
}

fn render_generic_section(section: &SectionNode, accent: &str) -> String {
    let title = section.title.as_deref().unwrap_or(&section.section_type);
    let subtitle = section.subtitle.as_deref().unwrap_or("");
    let entity = section.config.get("entity").map(|s| s.as_str());

    let entity_html = entity.map(|e| {
        let lower = e.to_lowercase();
        format!(
            r#"<div data-list="{lower}" data-cols="id,name,status" class="mt-6 border border-neutral-800 rounded-lg p-4">
  <p class="text-neutral-600 font-mono text-sm">Loading {e} data...</p>
</div>"#,
            lower = lower, e = e,
        )
    }).unwrap_or_default();

    format!(
        r#"<section class="py-12 max-w-6xl mx-auto px-6">
  <div class="flex items-center gap-3 mb-6">
    <p class="font-mono text-[10px] uppercase tracking-[0.2em] text-{accent}-500">// {title}</p>
    <div class="flex-1 h-px bg-neutral-800/50"></div>
  </div>
  <h2 class="text-2xl font-bold mb-2">{title}</h2>
  <p class="text-neutral-400 text-sm">{subtitle}</p>
  {entity_html}
</section>"#,
        title = title, subtitle = subtitle, accent = accent, entity_html = entity_html,
    )
}

// ══════════════════════════════════════════════════
// COMPONENT RENDERER — Bridges .cronus components to HTML
// ══════════════════════════════════════════════════

/// Render a single ComponentNode to HTML by dispatching on layout
pub fn render_component(comp: &ComponentNode) -> String {
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

/// Render multiple components into a single HTML block
pub fn render_components_page(comps: &[ComponentNode]) -> String {
    comps.iter()
        .map(|c| render_component(c))
        .collect::<Vec<_>>()
        .join("\n")
}

// Helper: get item text by kind
fn item_by_kind<'a>(items: &'a [ComponentItemNode], kind: &str) -> Option<&'a str> {
    items.iter()
        .find(|i| i.item_type == kind)
        .map(|i| i.text.as_str())
}

// Helper: get all items of a kind
fn items_by_kind<'a>(items: &'a [ComponentItemNode], kind: &str) -> Vec<&'a ComponentItemNode> {
    items.iter().filter(|i| i.item_type == kind).collect()
}

// ── inline: Button, Badge ──

fn render_inline_component(comp: &ComponentNode, style: &str) -> String {
    let label = item_by_kind(&comp.items, "label").unwrap_or(&comp.name);
    let icon = item_by_kind(&comp.items, "icon");

    if style.contains("badge") {
        // Badge
        let tone = comp.items.iter()
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
        // Button
        let variant = if style.contains("primary") { "primary" }
            else if style.contains("secondary") { "secondary" }
            else if style.contains("ghost") { "ghost" }
            else if style.contains("danger") { "danger" }
            else if style.contains("outline") { "outline" }
            else { "primary" };
        let size = if style.contains("lg") { "lg" }
            else if style.contains("sm") { "sm" }
            else { "md" };
        let href = comp.items.iter()
            .find(|i| i.link.is_some())
            .and_then(|i| i.link.as_deref());
        components::button(label, variant, size, href)
    }
}

// ── stack: StatCard, EmptyState, Card, Alert ──

fn render_stack_component(comp: &ComponentNode, style: &str) -> String {
    if style.contains("metric") || style.contains("stat") {
        // StatCard
        let label = item_by_kind(&comp.items, "label").unwrap_or("Metric");
        let value = item_by_kind(&comp.items, "value").unwrap_or("0");
        let trend = comp.items.iter().find(|i| i.item_type == "trend");
        let change_pct = trend.map(|t| {
            t.text.trim_end_matches('%').parse::<f32>().unwrap_or(0.0)
        });
        let icon = item_by_kind(&comp.items, "icon").unwrap_or("");
        components::stat_card(label, value, change_pct, icon)
    } else if style.contains("empty") {
        // EmptyState
        let icon = item_by_kind(&comp.items, "icon").unwrap_or("📭");
        let title = item_by_kind(&comp.items, "title").unwrap_or(item_by_kind(&comp.items, "label").unwrap_or("No data"));
        let text = item_by_kind(&comp.items, "text").unwrap_or("");
        let action = item_by_kind(&comp.items, "action");
        components::empty_state(icon, title, text, action)
    } else if style.contains("alert") {
        // Alert
        let tone = if style.contains("success") { "success" }
            else if style.contains("warning") { "warning" }
            else if style.contains("danger") || style.contains("error") { "error" }
            else { "info" };
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
            label, actions.join("\n  ")
        )
    } else {
        // Generic card
        let title = item_by_kind(&comp.items, "title").unwrap_or(item_by_kind(&comp.items, "label").unwrap_or(&comp.name));
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

fn render_grid_component(comp: &ComponentNode, style: &str) -> String {
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
        let cols = if style.contains("3col") { "3" } else if style.contains("2col") { "2" } else { "3" };
        format!(
            r#"<div style="display:grid;grid-template-columns:repeat({},1fr);gap:16px">{}</div>"#,
            cols, plans.join("\n")
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
        let cols = if style.contains("3col") { "3" } else if style.contains("2col") { "2" } else { "3" };
        format!(
            r#"<div style="display:grid;grid-template-columns:repeat({},1fr);gap:16px">{}</div>"#,
            cols, items.join("\n")
        )
    }
}

// ── table: DataTable ──

fn render_table_component(comp: &ComponentNode, style: &str) -> String {
    let source = item_by_kind(&comp.items, "source").unwrap_or("item");
    let cols_str = item_by_kind(&comp.items, "columns").unwrap_or("id,name");
    let cols: Vec<&str> = cols_str.split(',').map(|s| s.trim()).collect();

    let headers: Vec<&str> = cols.clone();
    let rows: Vec<Vec<String>> = vec![]; // SSR empty — runtime fills via data-list

    let lower = source.to_lowercase();
    let cols_attr = cols.join(",");
    format!(
        r#"<div style="border-radius:var(--radius-card);border:1px solid var(--border);overflow:hidden">
  <div data-list="{lower}" data-cols="{cols_attr}">
    <p style="padding:16px;font-size:13px;color:var(--foreground-muted)">Loading {source}...</p>
  </div>
</div>"#,
        lower = lower, cols_attr = cols_attr, source = source,
    )
}

// ── hero ──

fn render_hero_component(comp: &ComponentNode, style: &str) -> String {
    let badge_text = item_by_kind(&comp.items, "badge").unwrap_or("");
    let title = item_by_kind(&comp.items, "title").unwrap_or("Welcome");
    let subtitle = item_by_kind(&comp.items, "subtitle").unwrap_or("");
    let ctas: Vec<String> = items_by_kind(&comp.items, "cta").iter().map(|c| {
        let href = c.link.as_deref().unwrap_or("#");
        let tone = c.config.get("tone").map(|s| s.as_str()).unwrap_or("primary");
        let variant = if tone == "primary" || tone == "default" { "primary" } else { "outline" };
        components::button(&c.text, variant, "lg", Some(href))
    }).collect();

    let badge_html = if !badge_text.is_empty() {
        format!(
            r#"<span style="display:inline-flex;align-items:center;gap:6px;font-size:10px;font-weight:600;letter-spacing:0.15em;text-transform:uppercase;color:var(--accent);margin-bottom:20px">
    <span style="width:6px;height:6px;border-radius:50%;background:var(--accent)"></span>
    {}
  </span>"#, badge_text
        )
    } else { String::new() };

    format!(
        r#"<section style="text-align:center;padding:80px 24px 48px;max-width:720px;margin:0 auto">
  {badge_html}
  <h1 style="font-size:48px;font-weight:800;color:var(--foreground);letter-spacing:-0.03em;line-height:1.05;margin-bottom:16px">{title}</h1>
  <p style="font-size:16px;color:var(--foreground-muted);line-height:1.6;max-width:540px;margin:0 auto 32px">{subtitle}</p>
  <div style="display:flex;gap:12px;justify-content:center">{ctas}</div>
</section>"#,
        badge_html = badge_html, title = title, subtitle = subtitle,
        ctas = ctas.join("\n    "),
    )
}

// ── modal ──

fn render_modal_component(comp: &ComponentNode, style: &str) -> String {
    let id = comp.name.to_lowercase().replace(' ', "-");
    let title = item_by_kind(&comp.items, "title").unwrap_or("Dialog");
    let text = item_by_kind(&comp.items, "text").unwrap_or("");
    let actions: Vec<String> = items_by_kind(&comp.items, "action").iter().map(|a| {
        let tone = a.config.get("tone").map(|s| s.as_str()).unwrap_or("secondary");
        let variant = if tone == "danger" { "danger" } else if tone == "primary" { "primary" } else { "secondary" };
        components::button(&a.text, variant, "md", None)
    }).collect();

    let content = format!(
        r#"<p style="font-size:13px;color:var(--foreground-muted);margin-bottom:16px">{}</p>
<div style="display:flex;gap:8px;justify-content:flex-end">{}</div>"#,
        text, actions.join("\n    ")
    );
    components::modal(title, &content, &id)
}

// ── sidebar ──

fn render_sidebar_component(comp: &ComponentNode, _style: &str) -> String {
    let nav_items: Vec<String> = items_by_kind(&comp.items, "item").iter().map(|item| {
        let href = item.link.as_deref().unwrap_or("#");
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

fn render_tabs_component(comp: &ComponentNode, style: &str) -> String {
    let tab_items: Vec<(&str, &str)> = items_by_kind(&comp.items, "tab").iter().map(|t| {
        (t.text.as_str(), "")
    }).collect();
    let active = comp.items.iter()
        .position(|i| i.item_type == "tab" && i.config.get("active").map(|v| v == "true").unwrap_or(false))
        .unwrap_or(0);
    components::tabs(&tab_items, active)
}

// ── menu (dropdown) ──

fn render_menu_component(comp: &ComponentNode, _style: &str) -> String {
    let trigger = item_by_kind(&comp.items, "trigger").unwrap_or("Menu");
    let menu_items: Vec<(&str, &str)> = items_by_kind(&comp.items, "action").iter().map(|a| {
        let href = a.link.as_deref().unwrap_or("#");
        (a.text.as_str(), href)
    }).collect();
    components::dropdown(trigger, &menu_items)
}
