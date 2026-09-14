#![allow(dead_code, unused_imports)]
use crate::parser::SectionNode;
use std::collections::HashMap;

/// Describes a section that has live:true
pub struct LiveSection {
    pub section_id: String,
    pub entity: String,
    pub interval: Option<u32>,
    pub aggregate: Option<String>,
}

/// Describes a KPI item for live aggregate updates
pub struct KpiItem {
    pub name: String,
    pub field: String,
    pub aggregate: String,
}

/// Generates the SSE client JavaScript for all live sections.
///
/// Connects to /api/sse, listens for data_change events, and refreshes
/// matching sections. Falls back to polling on SSE disconnect.
pub fn render_live_script(sections: &[LiveSection]) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    if sections.is_empty() {
        return String::new();
    }

    // Build JSON array of live sections
    let mut sections_json = String::from("[");
    for (i, sec) in sections.iter().enumerate() {
        if i > 0 {
            sections_json.push(',');
        }
        sections_json.push_str(&format!(
            r#"{{"sectionId":"{}","entity":"{}","interval":{}}}"#,
            escape_js(&sec.section_id),
            escape_js(&sec.entity),
            sec.interval
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".to_string()),
        ));
    }
    sections_json.push(']');

    format!(
        r##"<script{script_nonce}>
(function(){{
  var liveSections={sections_json};

  function cronusRefreshSection(sectionId,entity){{
    fetch('/api/'+entity.toLowerCase()+'s')
      .then(function(r){{return r.json()}})
      .then(function(data){{
        var el=document.getElementById(sectionId);
        if(el){{
          el.dispatchEvent(new CustomEvent('cronus:update',{{detail:data}}));
          el.style.transition='opacity 0.2s';
          el.style.opacity='0.7';
          setTimeout(function(){{el.style.opacity='1'}},200);
        }}
      }})
      .catch(function(err){{console.warn('[CRONUS] Refresh failed:',err)}});
  }}
  window.cronusRefreshSection=cronusRefreshSection;

  var evtSource=new EventSource('/api/sse');
  var pollingStarted=false;

  evtSource.addEventListener('data_change',function(e){{
    try{{
      var data=JSON.parse(e.data);
      liveSections.forEach(function(sec){{
        if(data.entity.toLowerCase()===sec.entity.toLowerCase()){{
          cronusRefreshSection(sec.sectionId,sec.entity);
        }}
      }});
    }}catch(err){{}}
  }});

  evtSource.onerror=function(){{
    if(pollingStarted)return;
    pollingStarted=true;
    console.warn('[CRONUS] SSE disconnected, falling back to polling');
    var ind=document.getElementById('cronus-live-indicator');
    if(ind){{var dot=ind.querySelector('span');if(dot)dot.style.background='#ef4444'}}
    liveSections.forEach(function(sec){{
      var ms=sec.interval||10000;
      setInterval(function(){{cronusRefreshSection(sec.sectionId,sec.entity)}},ms);
    }});
  }};

  var indicator=document.createElement('div');
  indicator.id='cronus-live-indicator';
  indicator.innerHTML='<span style="width:8px;height:8px;border-radius:50%;background:#22c55e;display:inline-block;animation:pulse 2s infinite"></span> Live';
  indicator.style.cssText='position:fixed;bottom:16px;left:16px;z-index:100;font-size:12px;color:#71717a;display:flex;align-items:center;gap:6px;font-family:Inter,sans-serif';
  document.body.appendChild(indicator);

  evtSource.addEventListener('open',function(){{
    var dot=indicator.querySelector('span');
    if(dot)dot.style.background='#22c55e';
  }});
}})();
</script>"##,
        sections_json = sections_json,
    )
}

/// Generates optimistic mutation JavaScript for an entity.
///
/// Dispatches a `cronus:optimistic` event locally before sending to server,
/// then rolls back on failure.
pub fn render_optimistic_mutation(entity: &str, _action: &str) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    format!(
        r##"<script{script_nonce}>
window.cronusMutate=function(entity,action,data){{
  var el=document.querySelector('[data-entity="'+entity+'"]');
  if(el){{
    el.dispatchEvent(new CustomEvent('cronus:optimistic',{{detail:{{action:action,data:data}}}}));
  }}

  var method=action==='delete'?'DELETE':action==='create'?'POST':'PUT';
  var url='/api/'+entity.toLowerCase()+'s'+(data.id?'/'+data.id:'');

  return fetch(url,{{
    method:method,
    headers:{{'Content-Type':'application/json'}},
    body:JSON.stringify(data)
  }}).then(function(r){{
    if(!r.ok){{
      if(el&&window.cronusRefreshSection)cronusRefreshSection(el.id,entity);
      if(window.cronusToast)cronusToast('Operation failed','error');
      else if(window.cronusShowToast)cronusShowToast('Operation failed','error');
    }}
    return r.json();
  }});
}};
</script>"##
    )
}

/// Generates live KPI handler JavaScript for aggregate sections.
///
/// Listens for `cronus:update` on the section and re-fetches aggregate stats.
pub fn render_live_kpi_handler(section_id: &str, entity: &str, items: &[KpiItem]) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    if items.is_empty() {
        return String::new();
    }

    // Build the update assignments
    let mut update_lines = String::new();
    for item in items {
        let kpi_key = format!("{}_{}", escape_js(&item.field), escape_js(&item.aggregate));
        let name_escaped = escape_js(&item.name);
        update_lines.push_str(&format!(
            r#"      var kEl=document.querySelector('[data-kpi="{}"]');if(kEl&&stats.{})kEl.textContent=stats.{};"#,
            name_escaped, kpi_key, kpi_key,
        ));
        update_lines.push('\n');
    }

    format!(
        r##"<script{script_nonce}>
(function(){{
  var el=document.getElementById('{section_id}');
  if(!el)return;
  el.addEventListener('cronus:update',function(){{
    fetch('/api/{entity}s/stats')
      .then(function(r){{return r.json()}})
      .then(function(stats){{
{update_lines}      }})
      .catch(function(err){{console.warn('[CRONUS] KPI refresh failed:',err)}});
  }});
}})();
</script>"##,
        section_id = escape_js(section_id),
        entity = escape_js(entity),
        update_lines = update_lines,
    )
}

/// Generates live table handler JavaScript for table sections.
///
/// Listens for `cronus:update` on the section and re-renders table rows
/// from fresh API data, preserving the header.
pub fn render_live_table_handler(section_id: &str, entity: &str) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    format!(
        r##"<script{script_nonce}>
(function(){{
  var el=document.getElementById('{section_id}');
  if(!el)return;
  el.addEventListener('cronus:update',function(e){{
    var rows=Array.isArray(e.detail)?e.detail:e.detail.data||[];
    var tbody=el.querySelector('tbody');
    if(!tbody)return;

    var headers=[];
    el.querySelectorAll('thead th').forEach(function(th){{headers.push(th.textContent.trim().toLowerCase())}});
    if(!headers.length)return;

    var html='';
    rows.forEach(function(row,idx){{
      html+='<tr style="border-bottom:1px solid #f4f4f5;animation:cronusFadeIn 0.3s ease '+(idx*0.03)+'s both">';
      headers.forEach(function(h){{
        var val=row[h]||row[h.replace(/ /g,'_')]||'';
        if(h==='status'){{
          var color=val==='active'||val==='success'||val==='paid'?'#059669':val==='failed'||val==='error'?'#dc2626':'#71717a';
          html+='<td style="padding:12px 16px;font-size:14px"><span style="display:inline-flex;align-items:center;gap:6px;font-size:12px;font-weight:600;color:'+color+'"><span style="width:6px;height:6px;border-radius:50%;background:'+color+'"></span>'+val+'</span></td>';
        }}else{{
          html+='<td style="padding:12px 16px;font-size:14px;color:#1a1c1c;font-family:Inter,sans-serif">'+val+'</td>';
        }}
      }});
      html+='</tr>';
    }});

    tbody.innerHTML=html;
  }});
}})();
</script>"##,
        section_id = escape_js(section_id),
    )
}

/// Escape string for safe JS embedding
fn escape_js(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
