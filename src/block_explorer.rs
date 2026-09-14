//! Block Explorer — visual block viewer with CRONUS design system
//! GET /blocks

use serde_json::Value;

pub fn render_explorer(
    _hydra_path: &str,
    trust_metrics: &[crate::trust::BlockMetrics],
    script_registry: &crate::scripting::ScriptRegistry,
) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    let hydra_blocks = load_hydra_blocks();

    let mut blocks_js = String::from("[");

    // Live scripts
    for script in &script_registry.scripts {
        let mut items_js = String::from("[");
        let mut total_execs: u64 = 0;
        let mut total_errors: u64 = 0;
        let mut max_score: f64 = 0.0;
        let mut stmts_total: usize = 0;

        for block in &script.blocks {
            let (name, btype, stmts) = match block {
                crate::scripting::ast::ScriptBlock::OnEvent(on) => (
                    format!("{}.{}", on.entity, on.event),
                    "event",
                    on.body.len(),
                ),
                crate::scripting::ast::ScriptBlock::Endpoint(ep) => (
                    format!("{} {}", ep.method, ep.path),
                    "endpoint",
                    ep.body.len(),
                ),
                crate::scripting::ast::ScriptBlock::Schedule(s) => {
                    (s.name.clone(), "schedule", s.body.len())
                }
                crate::scripting::ast::ScriptBlock::OnWebhook(w) => {
                    (w.path.clone(), "webhook", w.body.len())
                }
            };
            let m = trust_metrics.iter().find(|m| m.block_id.contains(&name));
            let (execs, errs, lat, score) = match m {
                Some(m) => {
                    let ev = m.to_evidence();
                    let g = crate::trust::TrustGates::new_clean();
                    let t = crate::trust::TrustProfile::from_evidence(&ev, g);
                    (m.executions, m.errors, m.avg_latency_ms(), t.score())
                }
                None => (0, 0, 0.0, 0.0),
            };
            total_execs += execs;
            total_errors += errs;
            stmts_total += stmts;
            if score > max_score {
                max_score = score;
            }
            items_js.push_str(&format!(
                "{{n:\"{}\",t:\"{}\",ex:{},er:{},lt:{:.1},sc:{:.3},st:{}}},",
                name.replace('"', "\\\""),
                btype,
                execs,
                errs,
                lat,
                score,
                stmts
            ));
        }
        items_js.push(']');
        let hash = simple_hash(&script.name);
        blocks_js.push_str(&format!(
            "{{name:\"{}\",hash:\"{}\",ver:\"{}\",items:{},execs:{},errs:{},sc:{:.3},live:true,stmts:{}}},",
            script.name.replace('"', ""), hash, script.version, items_js,
            total_execs, total_errors, max_score, stmts_total
        ));
    }

    // Hydra grouped by source
    let mut groups: std::collections::HashMap<String, Vec<&Value>> =
        std::collections::HashMap::new();
    for hb in &hydra_blocks {
        let src = hb
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        groups.entry(src).or_default().push(hb);
    }
    for (source, blocks) in groups.iter().take(20) {
        let mut items_js = String::from("[");
        let mut avg_score = 0.0;
        for hb in blocks.iter().take(40) {
            let name = hb.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let tags: Vec<&str> = hb
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|t| t.as_str()).collect())
                .unwrap_or_default();
            let btype = if tags.iter().any(|t| *t == "get") {
                "query"
            } else if tags.iter().any(|t| *t == "post") {
                "mutation"
            } else if tags.iter().any(|t| *t == "delete") {
                "delete"
            } else if tags.iter().any(|t| *t == "patch") {
                "update"
            } else {
                "block"
            };
            let sc = hb
                .get("score")
                .and_then(|v| v.get("final"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let provides: Vec<&str> = hb
                .get("provides")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|t| t.as_str()).collect())
                .unwrap_or_default();
            let requires: Vec<&str> = hb
                .get("requires")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|t| t.as_str()).collect())
                .unwrap_or_default();
            avg_score += sc;
            items_js.push_str(&format!(
                "{{n:\"{}\",t:\"{}\",ex:0,er:0,lt:0,sc:{:.2},st:0,prov:\"{}\",req:\"{}\",tags:\"{}\"}},",
                name.replace('"', ""), btype, sc,
                provides.join(", "), requires.join(", "),
                tags.iter().take(4).cloned().collect::<Vec<&str>>().join(", ")
            ));
        }
        items_js.push(']');
        avg_score /= blocks.len().max(1) as f64;
        blocks_js.push_str(&format!(
            "{{name:\"{}\",hash:\"{}\",ver:\"prod\",items:{},execs:0,errs:0,sc:{:.2},live:false,stmts:0,count:{}}},",
            source.replace('"', ""), simple_hash(source), items_js, avg_score, blocks.len()
        ));
    }
    blocks_js.push(']');

    let total_blocks = script_registry.scripts.len() + groups.len();
    let total_items: usize = script_registry
        .scripts
        .iter()
        .map(|s| s.blocks.len())
        .sum::<usize>()
        + hydra_blocks.len();
    let total_execs: u64 = trust_metrics.iter().map(|m| m.executions).sum();

    format!(
        r##"<!DOCTYPE html>
<html lang="en"><head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Blocks — CRONUS</title>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;900&family=Space+Grotesk:wght@500;600;700&family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet">
<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200" rel="stylesheet">
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
:root{{
  --bg:#0a0a0a;--surface:rgba(25,25,25,0.8);--card:rgba(20,20,20,0.9);
  --border:rgba(255,255,255,0.04);--border-accent:rgba(135,173,255,0.15);
  --text:#e2e2e2;--text-muted:#757575;--text-dim:#484848;
  --primary:#adc6ff;--secondary:#c2c1ff;--tertiary:#e9b3ff;
  --success:#10b981;--danger:#ef4444;--warning:#eab308;
  --radius:12px;
}}
body{{background:var(--bg);color:var(--text);font-family:'Inter',system-ui,-apple-system,sans-serif;-webkit-font-smoothing:antialiased;min-height:100vh}}
::selection{{background:rgba(135,173,255,0.2)}}

/* ── Layout ── */
.shell{{max-width:1440px;margin:0 auto;padding:40px 32px}}
.topbar{{display:flex;justify-content:space-between;align-items:flex-end;margin-bottom:40px;padding-bottom:20px;border-bottom:1px solid var(--border)}}
.topbar h1{{font-family:'Space Grotesk',sans-serif;font-size:22px;font-weight:700;letter-spacing:-0.03em;color:#fff}}
.topbar h1 span{{color:var(--primary)}}
.topbar p{{color:var(--text-muted);font-size:13px;margin-top:4px}}

/* ── KPIs ── */
.kpis{{display:flex;gap:12px;margin-bottom:32px}}
.kpi{{background:var(--surface);backdrop-filter:blur(40px);border:1px solid var(--border);border-top:0.5px solid var(--border-accent);border-radius:var(--radius);padding:20px 24px;flex:1}}
.kpi-val{{font-family:'Space Grotesk',sans-serif;font-size:28px;font-weight:700;letter-spacing:-0.03em}}
.kpi-label{{font-size:11px;font-weight:500;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.06em;margin-top:4px}}

/* ── Section headers ── */
.sec-header{{display:flex;align-items:center;gap:10px;margin-bottom:16px}}
.sec-header h2{{font-family:'Space Grotesk',sans-serif;font-size:15px;font-weight:600;color:#fff;letter-spacing:-0.01em}}
.sec-header .material-symbols-outlined{{font-size:18px;color:var(--primary)}}
.sec-divider{{flex:1;height:1px;background:var(--border)}}

/* ── Grid ── */
.grid{{display:grid;grid-template-columns:repeat(auto-fill,minmax(320px,1fr));gap:12px;margin-bottom:40px}}

/* ── Block Cards ── */
.block{{background:var(--surface);backdrop-filter:blur(40px);border:1px solid var(--border);border-top:0.5px solid var(--border-accent);border-radius:var(--radius);padding:20px;cursor:pointer;transition:all 0.2s ease}}
.block:hover{{border-color:rgba(135,173,255,0.25);transform:translateY(-2px)}}
.block.active{{border-color:var(--primary);box-shadow:0 8px 32px rgba(135,173,255,0.08)}}
.block.live{{border-top-color:rgba(16,185,129,0.3)}}

.block-head{{display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:12px}}
.block-name{{font-family:'Space Grotesk',sans-serif;font-size:15px;font-weight:700;color:#fff;letter-spacing:-0.02em}}
.block-badge{{font-size:10px;font-weight:600;letter-spacing:0.04em;padding:3px 10px;border-radius:999px}}
.block-meta{{font-size:11px;color:var(--text-dim);font-family:'JetBrains Mono',monospace;margin-bottom:14px}}

.block-stats{{display:flex;gap:20px;margin-bottom:14px;padding-bottom:12px;border-bottom:1px solid var(--border)}}
.block-stat-val{{font-size:16px;font-weight:700;font-family:'Space Grotesk',sans-serif}}
.block-stat-label{{font-size:10px;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.06em;margin-top:2px}}

/* ── Items inside blocks ── */
.block-items{{display:flex;flex-direction:column;gap:4px}}
.block-item{{display:flex;align-items:center;gap:8px;padding:6px 8px;background:rgba(255,255,255,0.02);border-radius:6px;font-size:12px}}
.block-item:hover{{background:rgba(255,255,255,0.04)}}
.item-dot{{width:6px;height:6px;border-radius:50%;flex-shrink:0}}
.item-name{{flex:1;color:var(--text-muted);overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-family:'JetBrains Mono',monospace;font-size:11px}}
.item-score{{font-weight:600;font-size:11px}}
.block-more{{font-size:11px;color:var(--text-dim);padding:4px 8px}}

/* ── Trust bar ── */
.trust-bar{{width:100%;height:3px;background:rgba(255,255,255,0.04);border-radius:2px;margin-top:12px;overflow:hidden}}
.trust-bar i{{display:block;height:100%;border-radius:2px;transition:width 1s cubic-bezier(.16,1,.3,1)}}

/* ── Drawer ── */
.overlay{{position:fixed;inset:0;background:rgba(0,0,0,0.5);backdrop-filter:blur(4px);z-index:99;opacity:0;pointer-events:none;transition:opacity 0.3s}}
.overlay.open{{opacity:1;pointer-events:auto}}
.drawer{{position:fixed;top:0;right:-560px;width:540px;height:100vh;background:rgba(14,14,14,0.98);backdrop-filter:blur(60px);border-left:1px solid var(--border);z-index:100;transition:right 0.35s cubic-bezier(.16,1,.3,1);overflow-y:auto}}
.drawer.open{{right:0}}
.drawer::-webkit-scrollbar{{width:4px}}
.drawer::-webkit-scrollbar-thumb{{background:rgba(255,255,255,0.06);border-radius:2px}}

.d-close{{position:absolute;top:16px;right:16px;background:rgba(255,255,255,0.04);border:1px solid var(--border);color:var(--text-muted);width:32px;height:32px;display:flex;align-items:center;justify-content:center;cursor:pointer;border-radius:8px;font-family:inherit;font-size:14px;transition:all 0.15s}}
.d-close:hover{{color:#fff;background:rgba(255,255,255,0.08)}}
.d-head{{padding:32px 28px 20px;border-bottom:1px solid var(--border)}}
.d-title{{font-family:'Space Grotesk',sans-serif;font-size:20px;font-weight:700;color:#fff;letter-spacing:-0.03em;margin-bottom:4px}}
.d-meta{{font-size:11px;color:var(--text-dim);font-family:'JetBrains Mono',monospace}}
.d-stats{{display:flex;gap:24px;padding:20px 28px;border-bottom:1px solid var(--border)}}
.d-stat-v{{font-family:'Space Grotesk',sans-serif;font-size:20px;font-weight:700;color:#fff}}
.d-stat-l{{font-size:10px;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.06em;margin-top:2px}}
.d-sec{{font-family:'Space Grotesk',sans-serif;font-size:12px;font-weight:600;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.08em;padding:20px 28px 8px}}
.d-items{{padding:0 28px 28px}}
.d-item{{background:rgba(255,255,255,0.02);border:1px solid var(--border);border-radius:10px;padding:16px;margin-bottom:8px;transition:border-color 0.2s}}
.d-item:hover{{border-color:rgba(255,255,255,0.08)}}
.d-item-head{{display:flex;justify-content:space-between;align-items:center;margin-bottom:8px}}
.d-item-name{{font-family:'JetBrains Mono',monospace;font-size:12px;font-weight:600;color:#fff}}
.d-item-type{{font-size:10px;font-weight:600;letter-spacing:0.04em;padding:3px 10px;border-radius:999px}}
.d-item-row{{display:flex;gap:16px;margin-bottom:4px}}
.d-item-lbl{{font-size:10px;color:var(--text-muted);min-width:64px;text-transform:uppercase;letter-spacing:0.04em}}
.d-item-val{{font-size:12px;font-weight:600;font-family:'JetBrains Mono',monospace}}
.d-item-bar{{width:100%;height:3px;background:rgba(255,255,255,0.04);border-radius:2px;margin-top:8px;overflow:hidden}}
.d-item-bar i{{display:block;height:100%;border-radius:2px}}
.d-tags{{display:flex;flex-wrap:wrap;gap:4px;margin-top:8px}}
.d-tag{{font-size:10px;color:var(--text-muted);background:rgba(255,255,255,0.03);padding:3px 8px;border-radius:6px;font-family:'JetBrains Mono',monospace}}

/* ── Nav links ── */
.nav-links{{display:flex;gap:16px}}
.nav-links a{{color:var(--text-muted);font-size:12px;text-decoration:none;padding:6px 12px;border-radius:8px;background:rgba(255,255,255,0.03);border:1px solid var(--border);transition:all 0.15s}}
.nav-links a:hover{{color:#fff;border-color:rgba(255,255,255,0.1)}}
</style>
</head><body>
<div class="shell">
  <div class="topbar">
    <div>
      <h1><span>Blocks</span> Explorer</h1>
      <p>Compiled blocks, trust scores, and promotion pipeline</p>
    </div>
    <div style="display:flex;align-items:center;gap:24px">
      <div class="nav-links">
        <a href="/zeus">Zeus</a>
        <a href="/trust">Trust</a>
        <a href="/docs">Docs</a>
      </div>
    </div>
  </div>

  <div class="kpis">
    <div class="kpi"><div class="kpi-val" style="color:var(--primary)">{total_blocks}</div><div class="kpi-label">Blocks</div></div>
    <div class="kpi"><div class="kpi-val" style="color:var(--secondary)">{total_items}</div><div class="kpi-label">Items</div></div>
    <div class="kpi"><div class="kpi-val" style="color:var(--success)">{total_execs}</div><div class="kpi-label">Executions</div></div>
  </div>

  <div class="sec-header">
    <span class="material-symbols-outlined">deployed_code</span>
    <h2>Block Registry</h2>
    <div class="sec-divider"></div>
  </div>
  <div class="grid" id="ch"></div>
</div>

<div class="overlay" id="ov" onclick="closeDrawer()"></div>
<div class="drawer" id="dr">
  <button class="d-close" onclick="closeDrawer()">
    <span class="material-symbols-outlined" style="font-size:16px">close</span>
  </button>
  <div id="dr-content"></div>
</div>

<script{script_nonce}>
var B={blocks_js};
function tc(v){{return v>=.8?'#adc6ff':v>=.6?'#10b981':v>=.3?'#eab308':'#ef4444'}}
function sc(v){{if(v>=.8)return['OFFICIAL','#adc6ff'];if(v>=.6)return['PRODUCTION','#10b981'];if(v>=.3)return['APPROVED','#eab308'];return['PENDING','#484848']}}
function dc(t){{return t=='event'?'#c2c1ff':t=='endpoint'?'#adc6ff':t=='webhook'?'#e9b3ff':t=='schedule'?'#eab308':t=='mutation'?'#e9b3ff':t=='query'?'#10b981':t=='delete'?'#ef4444':t=='update'?'#adc6ff':'#757575'}}

var ch=document.getElementById('ch');
var html='';
B.forEach(function(b,bi){{
  var s=sc(b.sc);var c=tc(b.sc);var p=Math.round(b.sc*100);
  var preview=b.items.slice(0,4);

  html+='<div class="block'+(b.live?' live':'')+'" data-idx="'+bi+'" onclick="openBlock('+bi+')">';
  html+='<div class="block-head"><span class="block-name">'+b.name.substring(0,28)+'</span>';
  html+='<span class="block-badge" style="background:'+s[1]+'12;color:'+s[1]+'">'+s[0]+'</span></div>';
  html+='<div class="block-meta">#'+b.hash+' &middot; v'+b.ver+(b.count?' &middot; '+b.count+' items':'')+'</div>';
  html+='<div class="block-stats">';
  html+='<div><div class="block-stat-val" style="color:var(--secondary)">'+b.execs+'</div><div class="block-stat-label">exec</div></div>';
  html+='<div><div class="block-stat-val" style="color:'+(b.errs>0?'var(--danger)':'var(--success)')+'">'+b.errs+'</div><div class="block-stat-label">err</div></div>';
  html+='<div><div class="block-stat-val" style="color:'+c+'">'+p+'%</div><div class="block-stat-label">trust</div></div>';
  html+='</div>';
  html+='<div class="block-items">';
  preview.forEach(function(it){{
    html+='<div class="block-item"><div class="item-dot" style="background:'+dc(it.t)+'"></div>';
    html+='<span class="item-name">'+it.n+'</span>';
    html+='<span class="item-score" style="color:'+tc(it.sc)+'">'+Math.round(it.sc*100)+'%</span></div>';
  }});
  if(b.items.length>4)html+='<div class="block-more">+' +(b.items.length-4)+' more</div>';
  html+='</div>';
  html+='<div class="trust-bar"><i style="width:'+p+'%;background:'+c+'"></i></div>';
  html+='</div>';
}});
ch.innerHTML=html;

var activeIdx=-1;
function openBlock(idx){{
  var b=B[idx];if(!b)return;
  activeIdx=idx;
  document.querySelectorAll('.block').forEach(function(el){{el.classList.remove('active')}});
  document.querySelector('.block[data-idx="'+idx+'"]').classList.add('active');

  var s=sc(b.sc);var c=tc(b.sc);var p=Math.round(b.sc*100);
  var h='<div class="d-head">';
  h+='<div class="d-title">'+b.name+'</div>';
  h+='<div class="d-meta">#'+b.hash+' &middot; v'+b.ver+' &middot; '+(b.live?'LIVE SCRIPT':'HYDRA REGISTRY')+(b.count?' &middot; '+b.count+' items in source':'')+'</div>';
  h+='</div>';

  h+='<div class="d-stats">';
  h+='<div><div class="d-stat-v" style="color:var(--secondary)">'+b.execs+'</div><div class="d-stat-l">Executions</div></div>';
  h+='<div><div class="d-stat-v" style="color:'+(b.errs>0?'var(--danger)':'var(--success)')+'">'+b.errs+'</div><div class="d-stat-l">Errors</div></div>';
  h+='<div><div class="d-stat-v" style="color:'+c+'">'+p+'%</div><div class="d-stat-l">Trust</div></div>';
  h+='<div><div class="d-stat-v">'+b.items.length+'</div><div class="d-stat-l">Items</div></div>';
  if(b.stmts)h+='<div><div class="d-stat-v">'+b.stmts+'</div><div class="d-stat-l">Stmts</div></div>';
  h+='</div>';

  h+='<div class="d-sec">Compiled Items ('+b.items.length+')</div>';
  h+='<div class="d-items">';
  b.items.forEach(function(it){{
    var itc=tc(it.sc);var itp=Math.round(it.sc*100);var itdc=dc(it.t);
    h+='<div class="d-item">';
    h+='<div class="d-item-head"><span class="d-item-name">'+it.n+'</span>';
    h+='<span class="d-item-type" style="background:'+itdc+'14;color:'+itdc+'">'+it.t.toUpperCase()+'</span></div>';

    h+='<div class="d-item-row"><span class="d-item-lbl">Trust</span><span class="d-item-val" style="color:'+itc+'">'+itp+'%</span></div>';

    if(it.ex>0){{
      h+='<div class="d-item-row"><span class="d-item-lbl">Execs</span><span class="d-item-val" style="color:var(--secondary)">'+it.ex+'</span></div>';
      h+='<div class="d-item-row"><span class="d-item-lbl">Errors</span><span class="d-item-val" style="color:'+(it.er>0?'var(--danger)':'var(--success)')+'">'+it.er+'</span></div>';
      h+='<div class="d-item-row"><span class="d-item-lbl">Latency</span><span class="d-item-val">'+it.lt.toFixed(1)+'ms</span></div>';
    }}
    if(it.st)h+='<div class="d-item-row"><span class="d-item-lbl">Stmts</span><span class="d-item-val">'+it.st+'</span></div>';
    if(it.prov)h+='<div class="d-item-row"><span class="d-item-lbl">Provides</span><span class="d-item-val" style="color:var(--success)">'+it.prov+'</span></div>';
    if(it.req)h+='<div class="d-item-row"><span class="d-item-lbl">Requires</span><span class="d-item-val" style="color:var(--tertiary)">'+it.req+'</span></div>';

    h+='<div class="d-item-bar"><i style="width:'+itp+'%;background:'+itc+'"></i></div>';

    if(it.tags){{
      h+='<div class="d-tags">';
      it.tags.split(', ').forEach(function(tag){{if(tag)h+='<span class="d-tag">'+tag+'</span>'}});
      h+='</div>';
    }}

    h+='</div>';
  }});
  h+='</div>';

  document.getElementById('dr-content').innerHTML=h;
  document.getElementById('dr').classList.add('open');
  document.getElementById('ov').classList.add('open');
}}

function closeDrawer(){{
  document.getElementById('dr').classList.remove('open');
  document.getElementById('ov').classList.remove('open');
  document.querySelectorAll('.block').forEach(function(el){{el.classList.remove('active')}});
  activeIdx=-1;
}}
document.addEventListener('keydown',function(e){{if(e.key==='Escape')closeDrawer()}});
</script>
</body></html>"##,
        total_blocks = total_blocks,
        total_items = total_items,
        total_execs = total_execs,
        blocks_js = blocks_js,
    )
}

fn simple_hash(s: &str) -> String {
    let h = s
        .as_bytes()
        .iter()
        .fold(0u32, |h, &b| h.wrapping_mul(31).wrapping_add(b as u32));
    format!("{:08x}", h)
}

fn load_hydra_blocks() -> Vec<Value> {
    for p in &[
        "/home/zedd/Documentos/CRONUS/hydra/blocks/index.json",
        "./hydra/blocks/index.json",
    ] {
        if let Ok(data) = std::fs::read_to_string(p) {
            if let Ok(parsed) = serde_json::from_str::<Value>(&data) {
                if let Some(blocks) = parsed.get("blocks").and_then(|b| b.as_array()) {
                    return blocks.clone();
                }
            }
        }
    }
    Vec::new()
}
