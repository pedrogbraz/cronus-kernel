//! Page-level rendering functions — dispatches to page type renderers.

use crate::parser::{EntityNode, FieldType, PageNode, SectionNode};
use super::render_section;

// ══════════════════════════════════════════════════
// PAGE RENDERER (returns inner body HTML)
// ══════════════════════════════════════════════════

pub fn render_page(page: &PageNode, entities: &[EntityNode], accent: &str, theme: &str, db: Option<&crate::database::CronusDB>, route_params: &std::collections::HashMap<String, String>, owner_id: &str) -> String {
    match page.page_type.as_str() {
        // dashboard/list/form/detail are now layout hints — actual rendering
        // uses the same pipeline as `custom` so developer-defined sections
        // are always respected. This matches the "declare once, get it" principle.
        "dashboard" | "custom" => render_custom(page, accent, theme, db, route_params, owner_id, entities),
        "list" => render_list(page, entities, accent),
        "form" => render_form(page, entities, accent),
        "detail" => render_list(page, entities, accent),
        "checkout" => render_checkout(page),
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
    let _title = page.title.as_deref().unwrap_or("");

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
            r#"<th data-sort="{}" style="padding:10px 16px;text-align:left;font-size:11px;font-weight:500;color:var(--foreground-muted);text-transform:uppercase;letter-spacing:0.05em;cursor:pointer;user-select:none">{} <span class="sort-icon" style="font-size:10px"></span></th>"#, n, n
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
    var esc=window.cronusEscape;
    if (enumFields.indexOf(field)===-1) return '<span style="font-size:13px;color:var(--foreground-muted)">'+esc(val||'\u2014')+'</span>';
    var v=(val||'').toLowerCase();
    var isPaid=v==='active'||v==='paid'||v==='completed'||v==='succeeded'||v==='approved';
    var isFail=v==='failed'||v==='cancelled'||v==='rejected'||v==='error';
    var dotColor=isPaid?'oklch(0.696 0.17 162)':isFail?'oklch(0.704 0.191 22)':'oklch(0.769 0.188 70)';
    var textColor=isPaid?'oklch(0.696 0.17 162)':isFail?'oklch(0.704 0.191 22)':'oklch(0.769 0.188 70)';
    var bgColor=isPaid?'oklch(0.696 0.17 162/8%)':isFail?'oklch(0.704 0.191 22/8%)':'oklch(0.769 0.188 70/8%)';
    return '<span style="display:inline-flex;align-items:center;gap:6px;padding:2px 10px;border-radius:20px;font-size:11px;font-weight:500;background:'+bgColor+';color:'+textColor+';border:1px solid '+dotColor.replace(')','/20%)')+'"><span style="width:5px;height:5px;border-radius:50%;background:'+dotColor+'"></span>'+esc(val||'\u2014')+'</span>';
  }}

  var pageSize=10,currentPage=0;
  var sortCol=null,sortAsc=true;
  var filteredData=[];

  function renderRows(data) {{
    var tbody=document.getElementById('table-body');
    var empty=document.getElementById('empty-state');
    var countLabel=document.getElementById('count-label');
    if(!data.length){{tbody.innerHTML='';empty.style.display='block';countLabel.textContent='0 registros';renderPaginationLabel(0);return}}
    empty.style.display='none';
    countLabel.textContent=data.length+' registro'+(data.length!==1?'s':'');
    var esc=window.cronusEscape;
    tbody.innerHTML=data.map(function(row,i){{
      var cells=fields.map(function(f){{
        return '<td style="padding:10px 16px;font-size:13px;color:var(--foreground-muted)">'+badge(row[f],f)+'</td>';
      }}).join('');
      var safeId=esc(row.id||'');
      var bg=i%2===0?'':'background:var(--surface-hover)';
      return '<tr style="border-bottom:1px solid var(--surface-hover);'+bg+';cursor:pointer" onmouseover="this.style.background=\'var(--surface-hover)\'" onclick="cronusEdit(\''+lower+'\',\''+safeId+'\')" data-id="'+safeId+'">'+cells+
        '<td style="padding:10px 8px;text-align:center;display:flex;gap:4px;align-items:center;justify-content:center">'+
        '<button onclick="event.stopPropagation();cronusDelete(\''+lower+'\',\''+safeId+'\')" style="background:none;border:none;cursor:pointer;opacity:0.3;padding:2px" onmouseover="this.style.opacity=1" onmouseout="this.style.opacity=0.3"><span class="material-symbols-outlined" style="font-size:16px;color:#dc2626">delete</span></button>'+
        '<svg width="14" height="14" fill="none" stroke="var(--foreground-subtle)" stroke-width="1.5" viewBox="0 0 24 24"><path d="M9 18l6-6-6-6"/></svg>'+
        '</td></tr>';
    }}).join('');
  }}

  function renderPaginationLabel(total){{
    var pagLabel=document.getElementById('pagination-label');
    if(!pagLabel)return;
    if(total===0){{pagLabel.innerHTML='';return}}
    var start=currentPage*pageSize;
    var end=Math.min(start+pageSize,total);
    pagLabel.innerHTML='Mostrando '+(start+1)+'-'+end+' de '+total+
      ' <button onclick="cronusPrev()" style="margin-left:16px;padding:4px 12px;border:1px solid var(--border);border-radius:999px;font-size:12px;cursor:pointer;background:var(--card);color:var(--foreground)"'+(currentPage===0?' disabled style="margin-left:16px;padding:4px 12px;border:1px solid var(--border);border-radius:999px;font-size:12px;cursor:not-allowed;background:var(--card);color:var(--foreground-subtle);opacity:0.5"':'')+'>&#8592; Prev</button> '+
      '<button onclick="cronusNext()" style="padding:4px 12px;border:1px solid var(--border);border-radius:999px;font-size:12px;cursor:pointer;background:var(--card);color:var(--foreground)"'+(end>=total?' disabled style="padding:4px 12px;border:1px solid var(--border);border-radius:999px;font-size:12px;cursor:not-allowed;background:var(--card);color:var(--foreground-subtle);opacity:0.5"':'')+'>Next &#8594;</button>';
  }}

  function renderPaginated(data){{
    var start=currentPage*pageSize;
    var pageData=data.slice(start,start+pageSize);
    renderRows(pageData);
    renderPaginationLabel(data.length);
  }}

  function applySort(data){{
    if(!sortCol)return data;
    return data.slice().sort(function(a,b){{
      var va=(a[sortCol]||'').toString().toLowerCase();
      var vb=(b[sortCol]||'').toString().toLowerCase();
      return sortAsc?va.localeCompare(vb):vb.localeCompare(va);
    }});
  }}

  function refresh(){{
    var sorted=applySort(filteredData);
    renderPaginated(sorted);
  }}

  window.cronusPrev=function(){{if(currentPage>0){{currentPage--;refresh()}}}};
  window.cronusNext=function(){{if((currentPage+1)*pageSize<filteredData.length){{currentPage++;refresh()}}}};

  window.cronusDelete=function(entity,id){{
    if(!confirm('Deletar este registro?'))return;
    fetch('/api/'+entity+'s/'+id,{{method:'DELETE'}}).then(function(r){{
      if(r.ok){{allData=allData.filter(function(row){{return row.id!==id}});filteredData=filteredData.filter(function(row){{return row.id!==id}});refresh()}}
      else{{alert('Erro ao deletar')}}
    }}).catch(function(){{alert('Erro de conexao')}});
  }};

  window.cronusEdit=function(entity,id){{
    window.location.href='/'+entity+'s/'+id+'/edit';
  }};

  // Sort
  document.querySelectorAll('th[data-sort]').forEach(function(th){{
    th.addEventListener('click',function(){{
      var col=th.getAttribute('data-sort');
      if(sortCol===col)sortAsc=!sortAsc;else{{sortCol=col;sortAsc=true}}
      currentPage=0;
      refresh();
      document.querySelectorAll('th[data-sort]').forEach(function(t){{var si=t.querySelector('.sort-icon');if(si)si.textContent=''}});
      var icon=th.querySelector('.sort-icon');if(icon)icon.textContent=sortAsc?'\u2191':'\u2193';
    }});
  }});

  function load(){{
    fetch('/api/'+lower+'s').then(function(r){{return r.json()}}).then(function(d){{
      allData=Array.isArray(d)?d:[];filteredData=allData.slice();currentPage=0;refresh();
    }}).catch(function(){{allData=[];filteredData=[];refresh()}});
  }}

  document.getElementById('search-input').addEventListener('input',function(e){{
    var q=e.target.value.toLowerCase();
    filteredData=allData.filter(function(row){{
      return fields.some(function(f){{return(row[f]||'').toString().toLowerCase().indexOf(q)!==-1}});
    }});
    currentPage=0;
    refresh();
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
// AUTH PAGE (Login / Signup)
// ══════════════════════════════════════════════════

pub fn render_auth_page(page: &PageNode, is_login: bool) -> String {
    let title = if is_login { "Welcome back" } else { "Create your account" };
    let subtitle = if is_login { "Sign in to your account" } else { "Get started for free" };
    let btn_label = if is_login { "Sign In" } else { "Sign Up" };
    let action = if is_login { "/api/auth/login" } else { "/api/auth/signup" };
    let alt_text = if is_login { "Don't have an account?" } else { "Already have an account?" };
    let alt_link = if is_login { "/signup" } else { "/login" };
    let alt_label = if is_login { "Sign up" } else { "Sign in" };

    let app_name = page.title.as_deref().unwrap_or("G");
    let logo_letter = app_name.chars().next().unwrap_or('G').to_uppercase().to_string();

    let name_field = if is_login {
        String::new()
    } else {
        r#"<input type="text" name="name" placeholder="Full name" required style="width:100%;padding:10px 14px;font-size:14px;border:1px solid #e5e7eb;border-radius:10px;outline:none;box-sizing:border-box;transition:border-color 0.15s" onfocus="this.style.borderColor='#000'" onblur="this.style.borderColor='#e5e7eb'">"#.to_string()
    };

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{title}</title>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200" rel="stylesheet">
  <style>
    *{{margin:0;padding:0;box-sizing:border-box}}
    body{{font-family:'Inter',system-ui,-apple-system,sans-serif;-webkit-font-smoothing:antialiased}}
    .btn-hover:hover{{opacity:0.9;transform:translateY(-1px)}}
    input:focus{{border-color:#000!important;box-shadow:0 0 0 3px rgba(0,0,0,0.05)}}
  </style>
</head>
<body>
<div style="min-height:100vh;display:flex;align-items:center;justify-content:center;background:#f9f9f9">
  <div style="width:100%;max-width:400px;padding:32px">
    <div style="text-align:center;margin-bottom:32px">
      <div style="width:48px;height:48px;background:#000;border-radius:12px;display:flex;align-items:center;justify-content:center;margin:0 auto 16px">
        <span style="color:#fff;font-weight:700;font-size:20px">{logo_letter}</span>
      </div>
      <h1 style="font-size:24px;font-weight:700;margin:0 0 8px;color:#18181b">{title}</h1>
      <p style="font-size:14px;color:#71717a">{subtitle}</p>
    </div>
    <form id="auth-form" action="{action}" method="POST" style="display:flex;flex-direction:column;gap:16px">
      {name_field}
      <input type="email" name="email" placeholder="Email" required style="width:100%;padding:10px 14px;font-size:14px;border:1px solid #e5e7eb;border-radius:10px;outline:none;box-sizing:border-box;transition:border-color 0.15s" onfocus="this.style.borderColor='#000'" onblur="this.style.borderColor='#e5e7eb'">
      <input type="password" name="password" placeholder="Password" required style="width:100%;padding:10px 14px;font-size:14px;border:1px solid #e5e7eb;border-radius:10px;outline:none;box-sizing:border-box;transition:border-color 0.15s" onfocus="this.style.borderColor='#000'" onblur="this.style.borderColor='#e5e7eb'">
      <div id="auth-msg" style="display:none;padding:10px 14px;border-radius:10px;font-size:13px;text-align:center"></div>
      <button type="submit" class="btn-hover" style="width:100%;padding:10px 14px;font-size:14px;font-weight:600;border:none;border-radius:10px;background:#18181b;color:#fff;cursor:pointer;transition:all 0.15s">{btn_label}</button>
    </form>
    <p style="text-align:center;margin-top:16px;font-size:14px;color:#71717a">
      {alt_text} <a href="{alt_link}" style="color:#18181b;font-weight:600;text-decoration:none">{alt_label}</a>
    </p>
  </div>
</div>
<script>
document.getElementById('auth-form').addEventListener('submit',async function(e){{
  e.preventDefault();
  var data={{}};new FormData(this).forEach(function(v,k){{data[k]=v}});
  var action=this.getAttribute('action');
  var btn=this.querySelector('button[type=submit]');
  btn.disabled=true;btn.textContent='Loading...';
  try{{
    var r=await fetch(action,{{method:'POST',headers:{{'Content-Type':'application/json'}},body:JSON.stringify(data)}});
    var body=await r.json();
    if(r.ok&&body.token){{
      localStorage.setItem('token',body.token);
      localStorage.setItem('user',JSON.stringify(body.user||{{}}));
      window.location.href='/';
    }}else{{
      var msg=document.getElementById('auth-msg');
      msg.style.display='block';msg.style.background='#fef2f2';msg.style.color='#dc2626';msg.style.border='1px solid #fecaca';
      msg.textContent=body.error||'Invalid credentials';
      btn.disabled=false;btn.textContent='{btn_label}';
    }}
  }}catch(err){{
    var msg=document.getElementById('auth-msg');
    msg.style.display='block';msg.style.background='#fef2f2';msg.style.color='#dc2626';msg.style.border='1px solid #fecaca';
    msg.textContent='Connection error';
    btn.disabled=false;btn.textContent='{btn_label}';
  }}
}});
</script>
</body>
</html>"##,
        title = title,
        subtitle = subtitle,
        action = action,
        btn_label = btn_label,
        name_field = name_field,
        alt_text = alt_text,
        alt_link = alt_link,
        alt_label = alt_label,
        logo_letter = logo_letter,
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
      var esc=window.cronusEscape;
      var name=esc(row.name||row.title||'Sem nome');var s=(row.status||row.live||'active').toString().toLowerCase();
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
    var esc=window.cronusEscape;var h='';
    Object.keys(d).forEach(function(k){{
      if(k==='updated_at')return;
      var v=d[k];if(v===null||v===undefined)v='\u2014';
      h+='<div class="flex items-center justify-between px-6 py-4 border-b border-neutral-800/30 hover:bg-neutral-900/30 transition-colors">';
      h+='<span class="font-mono text-[10px] uppercase tracking-[0.15em] text-neutral-500">'+esc(k)+'</span>';
      h+='<span class="text-sm text-neutral-300 font-mono">'+esc(v)+'</span>';
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

fn render_custom(page: &PageNode, accent: &str, theme: &str, db: Option<&crate::database::CronusDB>, route_params: &std::collections::HashMap<String, String>, owner_id: &str, entities: &[EntityNode]) -> String {
    let shell_types = ["topbar", "sidebar"];
    let mut shell_parts: Vec<String> = Vec::new();
    let mut content_parts: Vec<String> = Vec::new();
    let mut footer_parts: Vec<String> = Vec::new();
    let mut in_grid = false;
    let mut has_topbar_section = false;
    let mut has_sidebar_section = false;

    for section in &page.sections {
        let st_lower = section.section_type.to_lowercase();
        let comp_lower = section.config.get("_component")
            .map(|s| s.to_lowercase())
            .unwrap_or_default();
        let is_shell = shell_types.contains(&section.section_type.as_str())
            || st_lower.contains("sidebar") || st_lower.contains("topbar")
            || st_lower.contains("styles")
            || comp_lower.contains("sidebar") || comp_lower.contains("topbar")
            || comp_lower.contains("styles");
        // Track which shell types are present (for layout offset)
        if st_lower.contains("topbar") || comp_lower.contains("topbar")
            || section.section_type == "topbar" {
            has_topbar_section = true;
        }
        if st_lower.contains("sidebar") || comp_lower.contains("sidebar")
            || section.section_type == "sidebar" {
            has_sidebar_section = true;
        }
        // Check if the bound entity is shared (visible to all authenticated users)
        let effective_owner = if let Some(ref binding) = section.binding {
            let is_shared = entities.iter().any(|e| e.name == binding.entity && e.shared);
            if is_shared { "" } else { owner_id }
        } else {
            owner_id
        };

        // Resolve binding against real DB (falls back to None if no DB)
        let bound_data = match db {
            Some(db) => crate::binding::resolve_binding(section, db, route_params, effective_owner),
            None => crate::binding::ResolvedData::None,
        };

        let is_column_layout = section.section_type == "layout"
            && matches!(
                section.config.get("style").map(|s| s.as_str()),
                Some("columns") | Some("grid")
            );

        let is_footer = section.section_type == "footer";
        let target = if is_footer { &mut footer_parts } else if is_shell { &mut shell_parts } else { &mut content_parts };

        if is_column_layout {
            if in_grid {
                target.push(crate::layout_system::render_column_layout_end());
            }
            target.push(render_section(section, accent, theme, &bound_data));
            in_grid = true;
        } else if section.section_type == "layout" && in_grid {
            target.push(crate::layout_system::render_column_layout_end());
            in_grid = false;
            target.push(render_section(section, accent, theme, &bound_data));
        } else {
            target.push(render_section(section, accent, theme, &bound_data));
        }
    }

    if in_grid {
        content_parts.push(crate::layout_system::render_column_layout_end());
    }

    // Use semantic section tracking + HTML fallback for layout detection
    let shell_html = shell_parts.join("\n");
    let has_fixed_sidebar = has_sidebar_section
        || (shell_html.contains("<aside") && shell_html.contains("fixed") && shell_html.contains("left-0"));
    let has_fixed_topbar = has_topbar_section
        || (shell_html.contains("fixed") && shell_html.contains("w-full") && shell_html.contains("top-0")
            && !shell_html.contains("left-0"));

    // When templates provide fixed sidebar/topbar, content needs margin/padding
    // to avoid being hidden underneath them.
    // Pages that use raw templates (landing, dashboards with custom sidebar)
    // handle their own layout via the template's style_block — skip the wrapper.
    let has_any_template = page.sections.iter().any(|s|
        s.template.is_some() || s.config.contains_key("template")
    );
    let has_marketing_section = page.sections.iter().any(|s|
        matches!(s.section_type.as_str(), "hero" | "features" | "topbar" | "footer" | "cta" | "testimonial" | "pricing" | "faq")
    );
    // Skip default wrapper whenever the dev supplies their own templates —
    // this matches the landing-page behavior and prevents the kernel from
    // injecting margin/padding that fights against the dev's CSS.
    let skip_wrapper = has_any_template || has_marketing_section;
    let content_style = if skip_wrapper {
        ""
    } else {
        match (has_fixed_sidebar, has_fixed_topbar) {
            (true, true) => " style=\"margin-left:256px;padding-top:80px;padding-left:32px;padding-right:32px;padding-bottom:48px;min-height:100vh\"",
            (true, false) => " style=\"margin-left:256px;padding:32px;min-height:100vh\"",
            (false, true) => " style=\"padding-top:64px;min-height:100vh\"",
            _ => "",
        }
    };

    let mut out = String::new();
    out.push_str(&shell_html);
    out.push_str(&format!("\n<main id=\"cronus-content\"{}>\n", content_style));
    out.push_str(&content_parts.join("\n"));
    out.push_str("\n</main>\n");
    if !footer_parts.is_empty() {
        out.push_str(&footer_parts.join("\n"));
    }
    out
}

fn render_checkout(page: &PageNode) -> String {
    let title = page.title.as_deref().unwrap_or("Checkout");
    // Extract product info from page config
    let product_name = page.config.get("product_name").map(|s| s.as_str()).unwrap_or("Product");
    let product_desc = page.config.get("product_desc").map(|s| s.as_str()).unwrap_or("");
    let product_price = page.config.get("price").map(|s| s.as_str()).unwrap_or("$0.00");
    let product_image = page.config.get("image").map(|s| s.as_str()).unwrap_or("");
    let subtotal = page.config.get("subtotal").map(|s| s.as_str()).unwrap_or(product_price);
    let shipping = page.config.get("shipping").map(|s| s.as_str()).unwrap_or("Free");
    let taxes = page.config.get("taxes").map(|s| s.as_str()).unwrap_or("$0.00");

    let image_html = if !product_image.is_empty() {
        format!(r##"<img src="{}" style="width:100%;height:100%;object-fit:cover" alt="{}">"##, product_image, product_name)
    } else {
        r##"<div style="width:100%;height:100%;background:#f3f3f3;display:flex;align-items:center;justify-content:center;color:#999;font-size:11px">No image</div>"##.to_string()
    };

    format!(
        r##"<!-- Header -->
<header style="width:100%;border-bottom:1px solid #e5e5e5;position:sticky;top:0;z-index:50;background:rgba(255,255,255,0.8);backdrop-filter:blur(20px);display:flex;justify-content:space-between;align-items:center;height:64px;padding:0 48px">
  <div style="font-size:18px;font-weight:700;letter-spacing:-0.03em;color:black;display:flex;align-items:center;gap:8px">
    <span style="width:24px;height:24px;background:black;border-radius:4px;display:flex;align-items:center;justify-content:center;color:white;font-size:10px;font-weight:700">GP</span>
    GeistPay
  </div>
  <button style="display:flex;align-items:center;gap:8px;color:#71717a;font-size:14px;font-weight:500;background:none;border:none;cursor:pointer" onmouseover="this.style.color='black'" onmouseout="this.style.color='#71717a'">
    <svg width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path d="M18 6L6 18M6 6l12 12"/></svg>
    Cancel
  </button>
</header>

<main style="max-width:1152px;margin:0 auto;padding:48px 24px 80px">
  <div style="display:grid;grid-template-columns:7fr 5fr;gap:96px">

    <!-- Payment Column -->
    <div>
      <h1 style="font-size:30px;font-weight:700;letter-spacing:-0.02em;margin-bottom:32px">{title}</h1>

      <!-- Express Checkout -->
      <div style="display:grid;grid-template-columns:1fr 1fr;gap:16px;margin-bottom:40px">
        <button style="background:black;color:white;height:48px;border-radius:999px;border:none;display:flex;align-items:center;justify-content:center;gap:8px;cursor:pointer;font-size:14px;font-weight:600;transition:opacity 0.2s" onmouseover="this.style.opacity='0.9'" onmouseout="this.style.opacity='1'">
          Pay with <span style="font-weight:700">Apple Pay</span>
        </button>
        <button style="background:white;color:black;height:48px;border-radius:999px;border:1px solid #e5e5e5;display:flex;align-items:center;justify-content:center;gap:8px;cursor:pointer;font-size:14px;font-weight:600;transition:background 0.2s" onmouseover="this.style.background='#fafafa'" onmouseout="this.style.background='white'">
          Pay with <span style="font-weight:700">Google Pay</span>
        </button>
      </div>

      <!-- Divider -->
      <div style="display:flex;align-items:center;gap:16px;margin-bottom:32px">
        <div style="flex:1;height:1px;background:#e5e5e5"></div>
        <span style="color:#a1a1aa;font-size:11px;font-weight:500;text-transform:uppercase;letter-spacing:0.1em">Or pay with card</span>
        <div style="flex:1;height:1px;background:#e5e5e5"></div>
      </div>

      <!-- Form -->
      <form data-entity="order" style="display:flex;flex-direction:column;gap:24px">
        <!-- Email -->
        <div style="display:flex;flex-direction:column;gap:8px">
          <label style="font-size:11px;font-weight:600;letter-spacing:0.1em;text-transform:uppercase;color:#71717a">Email address</label>
          <input name="email" type="email" placeholder="alex@example.com" style="width:100%;height:48px;padding:0 16px;border-radius:8px;border:1px solid rgba(198,198,198,0.4);background:white;font-size:14px;color:#1a1a1a;outline:none;transition:border-color 0.2s" onfocus="this.style.borderColor='black'" onblur="this.style.borderColor='rgba(198,198,198,0.4)'">
        </div>

        <!-- Card -->
        <div style="display:flex;flex-direction:column;gap:8px">
          <label style="font-size:11px;font-weight:600;letter-spacing:0.1em;text-transform:uppercase;color:#71717a">Card information</label>
          <div>
            <input name="card" type="text" placeholder="1234 5678 1234 5678" style="width:100%;height:48px;padding:0 16px;border-radius:8px 8px 0 0;border:1px solid rgba(198,198,198,0.4);background:white;font-size:14px;color:#1a1a1a;outline:none" onfocus="this.style.borderColor='black'" onblur="this.style.borderColor='rgba(198,198,198,0.4)'">
            <div style="display:grid;grid-template-columns:1fr 1fr">
              <input name="expiry" type="text" placeholder="MM / YY" style="width:100%;height:48px;padding:0 16px;border-radius:0 0 0 8px;border:1px solid rgba(198,198,198,0.4);border-top:none;background:white;font-size:14px;color:#1a1a1a;outline:none" onfocus="this.style.borderColor='black'" onblur="this.style.borderColor='rgba(198,198,198,0.4)'">
              <input name="cvc" type="text" placeholder="CVC" style="width:100%;height:48px;padding:0 16px;border-radius:0 0 8px 0;border:1px solid rgba(198,198,198,0.4);border-top:none;border-left:none;background:white;font-size:14px;color:#1a1a1a;outline:none" onfocus="this.style.borderColor='black'" onblur="this.style.borderColor='rgba(198,198,198,0.4)'">
            </div>
          </div>
        </div>

        <!-- Billing -->
        <div style="display:flex;flex-direction:column;gap:8px">
          <label style="font-size:11px;font-weight:600;letter-spacing:0.1em;text-transform:uppercase;color:#71717a">Billing address</label>
          <select name="country" style="width:100%;height:48px;padding:0 16px;border-radius:8px 8px 0 0;border:1px solid rgba(198,198,198,0.4);background:white;font-size:14px;color:#1a1a1a;outline:none;appearance:none">
            <option>United States</option><option>United Kingdom</option><option>Germany</option><option>Brazil</option>
          </select>
          <input name="zip" type="text" placeholder="ZIP code" style="width:100%;height:48px;padding:0 16px;border-radius:0 0 8px 8px;border:1px solid rgba(198,198,198,0.4);border-top:none;background:white;font-size:14px;color:#1a1a1a;outline:none" onfocus="this.style.borderColor='black'" onblur="this.style.borderColor='rgba(198,198,198,0.4)'">
        </div>

        <!-- Save info -->
        <label style="display:flex;align-items:center;gap:12px;cursor:pointer;padding-top:8px">
          <input type="checkbox" style="width:16px;height:16px;border-radius:4px;accent-color:black">
          <span style="font-size:14px;color:#52525b">Save my information for a faster checkout</span>
        </label>

        <!-- Pay button -->
        <button type="submit" style="width:100%;height:56px;border-radius:999px;background:black;color:white;font-size:18px;font-weight:700;letter-spacing:-0.01em;border:none;cursor:pointer;display:flex;align-items:center;justify-content:center;gap:8px;margin-top:16px;transition:opacity 0.2s" onmouseover="this.style.opacity='0.9'" onmouseout="this.style.opacity='1'">
          Pay {product_price}
          <svg width="16" height="16" fill="rgba(255,255,255,0.5)" viewBox="0 0 24 24"><path d="M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zm-6 9c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2zm3.1-9H8.9V6c0-1.71 1.39-3.1 3.1-3.1s3.1 1.39 3.1 3.1v2z"/></svg>
        </button>

        <p style="text-align:center;font-size:12px;color:#a1a1aa;margin-top:8px;padding:0 32px;line-height:1.6">
          By confirming your payment, you agree to our Terms of Service and Privacy Policy. Secure processing by GeistPay.
        </p>
      </form>
    </div>

    <!-- Summary Column -->
    <div>
      <div style="position:sticky;top:96px;display:flex;flex-direction:column;gap:32px">

        <!-- Product Card -->
        <div style="background:white;border:1px solid rgba(198,198,198,0.2);border-radius:12px;padding:32px;display:flex;flex-direction:column;gap:32px">
          <div style="display:flex;gap:24px">
            <div style="width:96px;height:96px;border-radius:8px;overflow:hidden;flex-shrink:0;border:1px solid rgba(198,198,198,0.2)">
              {image_html}
            </div>
            <div style="display:flex;flex-direction:column;justify-content:center">
              <h3 style="font-size:18px;font-weight:700;letter-spacing:-0.02em">{product_name}</h3>
              <p style="font-size:14px;color:#71717a">{product_desc}</p>
              <p style="font-size:14px;font-weight:500;margin-top:8px">Qty: 1</p>
            </div>
          </div>

          <!-- Price breakdown -->
          <div style="display:flex;flex-direction:column;gap:16px;padding-top:16px;border-top:1px solid #f4f4f5">
            <div style="display:flex;justify-content:space-between;font-size:14px"><span style="color:#71717a">Subtotal</span><span style="font-weight:500">{subtotal}</span></div>
            <div style="display:flex;justify-content:space-between;font-size:14px"><span style="color:#71717a">Shipping</span><span style="font-weight:500">{shipping}</span></div>
            <div style="display:flex;justify-content:space-between;font-size:14px"><span style="color:#71717a">Taxes</span><span style="font-weight:500">{taxes}</span></div>
            <div style="display:flex;justify-content:space-between;font-size:20px;font-weight:700;letter-spacing:-0.02em;padding-top:16px;border-top:1px solid #f4f4f5">
              <span>Total</span><span>{product_price}</span>
            </div>
          </div>
        </div>

        <!-- Trust indicators -->
        <div style="display:grid;grid-template-columns:1fr 1fr;gap:16px">
          <div style="padding:16px;border-radius:8px;background:#f3f3f3;display:flex;flex-direction:column;align-items:center;text-align:center;gap:8px">
            <svg width="20" height="20" fill="none" stroke="#71717a" stroke-width="1.5" viewBox="0 0 24 24"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
            <span style="font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:0.1em;color:#71717a">Buyer Protection</span>
          </div>
          <div style="padding:16px;border-radius:8px;background:#f3f3f3;display:flex;flex-direction:column;align-items:center;text-align:center;gap:8px">
            <svg width="20" height="20" fill="none" stroke="#71717a" stroke-width="1.5" viewBox="0 0 24 24"><rect x="1" y="3" width="15" height="13" rx="1"/><path d="M16 8h4l3 3v5h-7V8z"/><circle cx="5.5" cy="18.5" r="2.5"/><circle cx="18.5" cy="18.5" r="2.5"/></svg>
            <span style="font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:0.1em;color:#71717a">Free 2-Day Air</span>
          </div>
        </div>

        <!-- Quote -->
        <div style="position:relative;padding:24px;background:black;color:white;border-radius:12px;overflow:hidden">
          <div style="position:relative;z-index:10">
            <p style="font-size:14px;font-weight:500;font-style:italic;line-height:1.6;opacity:0.9">"The standard for digital payments in the engineering space. Fast, secure, and beautiful."</p>
            <p style="font-size:12px;font-weight:700;margin-top:16px;letter-spacing:0.08em;text-transform:uppercase">Vogue Tech Review</p>
          </div>
          <div style="position:absolute;inset:0;background:linear-gradient(135deg,rgba(0,111,240,0.2),transparent);opacity:0.5"></div>
        </div>

      </div>
    </div>
  </div>
</main>

<!-- Footer -->
<footer style="margin-top:80px;padding:48px 24px;border-top:1px solid #e5e5e5">
  <div style="max-width:1152px;margin:0 auto;display:flex;justify-content:space-between;align-items:center">
    <div style="display:flex;align-items:center;gap:24px">
      <span style="font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:0.1em;color:#a1a1aa">Powered by GeistPay</span>
    </div>
    <div style="display:flex;gap:32px">
      <a href="#" style="font-size:12px;font-weight:500;color:#71717a;text-decoration:none" onmouseover="this.style.color='black'" onmouseout="this.style.color='#71717a'">Help</a>
      <a href="#" style="font-size:12px;font-weight:500;color:#71717a;text-decoration:none" onmouseover="this.style.color='black'" onmouseout="this.style.color='#71717a'">Terms</a>
      <a href="#" style="font-size:12px;font-weight:500;color:#71717a;text-decoration:none" onmouseover="this.style.color='black'" onmouseout="this.style.color='#71717a'">Privacy</a>
    </div>
  </div>
</footer>"##,
        title = title,
        product_name = product_name,
        product_desc = product_desc,
        product_price = product_price,
        subtotal = subtotal,
        shipping = shipping,
        taxes = taxes,
        image_html = image_html,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{PageNode, SectionNode};
    use std::collections::HashMap;

    /// Regression for the 2026-04-10 fix: `type:dashboard` used to render a
    /// hardcoded Cooud banking template (Saldo/BOOST/METAS DE VENDAS) and
    /// completely ignored user-defined sections. The fix made `"dashboard"`
    /// delegate to `render_custom` so user sections are respected.
    #[test]
    fn dashboard_type_renders_user_sections_not_hardcoded_template() {
        // A minimal user-defined section that carries an inline HTML template.
        // Using `template = Some(...)` bypasses section-type dispatch entirely:
        // `render_section` routes straight to `render_template` (see ui/mod.rs).
        let user_section = SectionNode {
            section_type: "custom".to_string(),
            title: None,
            subtitle: None,
            config: HashMap::new(),
            items: Vec::new(),
            plans: Vec::new(),
            binding: None,
            actions: Vec::new(),
            visibility: None,
            template: Some(
                "<div id=\"user-marker\">MY_UNIQUE_KPI_MARKER</div>".to_string(),
            ),
            style_block: None,
            doc: None,
        };

        let page = PageNode {
            route: "/dashboard".to_string(),
            page_type: "dashboard".to_string(),
            entity: None,
            title: None,
            sections: vec![user_section],
            config: HashMap::new(),
            components: Vec::new(),
            requires: None,
            doc: None,
        };

        let route_params: HashMap<String, String> = HashMap::new();
        let html = render_page(
            &page,
            &[],
            "blue",
            "dark",
            None,
            &route_params,
            "test-user",
        );

        // The user-defined marker MUST appear in the output.
        assert!(
            html.contains("MY_UNIQUE_KPI_MARKER"),
            "user section marker not found in output; dashboard type ignored user sections. HTML:\n{}",
            html
        );

        // The historical hardcoded Cooud banking template strings MUST NOT
        // appear — their presence would mean `type:dashboard` regressed back
        // to the pre-fix hardcoded branch.
        for forbidden in &["Saldo", "BOOST", "METAS DE VENDAS", "Iniciante", "Bronze"] {
            assert!(
                !html.contains(forbidden),
                "dashboard page rendered hardcoded template (found '{}'); regression of the 2026-04-10 fix",
                forbidden
            );
        }
    }
}
