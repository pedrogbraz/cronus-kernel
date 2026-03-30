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
// SHARED ANIMATION CSS + JS (injected into every page)
// ══════════════════════════════════════════════════

const CRONUS_ANIMATIONS_CSS: &str = r##"
@keyframes fadeIn{from{opacity:0}to{opacity:1}}
@keyframes slideUp{from{opacity:0;transform:translateY(24px)}to{opacity:1;transform:translateY(0)}}
@keyframes slideDown{from{opacity:0;transform:translateY(-12px)}to{opacity:1;transform:translateY(0)}}
@keyframes scaleIn{from{opacity:0;transform:scale(0.96)}to{opacity:1;transform:scale(1)}}
@keyframes slideRight{from{opacity:0;transform:translateX(-16px)}to{opacity:1;transform:translateX(0)}}
@keyframes fillWidth{from{width:0}to{width:var(--target-width)}}
@keyframes pulse{0%,100%{opacity:1}50%{opacity:.5}}
@keyframes float{0%,100%{transform:translateY(0)}50%{transform:translateY(-6px)}}
.anim-fade{animation:fadeIn .6s ease-out both}
.anim-slide-up{animation:slideUp .6s cubic-bezier(.16,1,.3,1) both}
.anim-slide-down{animation:slideDown .4s ease-out both}
.anim-scale{animation:scaleIn .5s cubic-bezier(.16,1,.3,1) both}
.anim-slide-right{animation:slideRight .5s cubic-bezier(.16,1,.3,1) both}
.d1{animation-delay:.05s}.d2{animation-delay:.1s}.d3{animation-delay:.15s}
.d4{animation-delay:.2s}.d5{animation-delay:.25s}.d6{animation-delay:.3s}
.d7{animation-delay:.35s}.d8{animation-delay:.4s}.d9{animation-delay:.45s}.d10{animation-delay:.5s}
.card-hover{transition:all .3s cubic-bezier(.16,1,.3,1)}
.card-hover:hover{transform:translateY(-2px);box-shadow:0 12px 40px rgba(0,0,0,.08)}
.btn-hover{transition:all .2s cubic-bezier(.16,1,.3,1)}
.btn-hover:hover{transform:translateY(-1px);box-shadow:0 4px 12px rgba(0,0,0,.15)}
.btn-hover:active{transform:translateY(0);box-shadow:none}
.link-hover{transition:color .2s,opacity .2s}.link-hover:hover{opacity:.7}
.nav-hover{transition:all .15s}.nav-hover:hover{background:rgba(0,0,0,.04)}
.progress-fill{animation:fillWidth 1.2s cubic-bezier(.16,1,.3,1) .3s both}
.reveal{opacity:0;transform:translateY(20px);transition:all .7s cubic-bezier(.16,1,.3,1)}
.reveal.visible{opacity:1;transform:translateY(0)}
"##;

const CRONUS_ANIMATIONS_JS: &str = r##"
<script>
document.addEventListener('DOMContentLoaded',()=>{
  const io=new IntersectionObserver(e=>{e.forEach(e=>{if(e.isIntersecting){e.target.classList.add('visible');io.unobserve(e.target)}})},{threshold:.1,rootMargin:'0px 0px -40px 0px'});
  document.querySelectorAll('.reveal').forEach(el=>io.observe(el));
  document.querySelectorAll('.stagger').forEach(c=>{Array.from(c.children).forEach((ch,i)=>{ch.style.animationDelay=(.05+i*.06)+'s'})});

  // Modal system
  window.cronusModal={
    open:function(id){document.getElementById(id).style.display='flex'},
    close:function(id){document.getElementById(id).style.display='none'}
  };

  // Generic CRUD modal
  window.cronusCreateModal=function(entity,fields){
    var old=document.getElementById('create-modal');if(old)old.remove();
    var html='<div id="create-modal" style="display:none;position:fixed;inset:0;z-index:100;background:rgba(0,0,0,0.5);align-items:center;justify-content:center;backdrop-filter:blur(4px)" onclick="if(event.target===this)cronusModal.close(\'create-modal\')">';
    html+='<div style="background:#fff;border-radius:16px;padding:32px;width:100%;max-width:480px;box-shadow:0 24px 48px rgba(0,0,0,0.15);animation:scaleIn 0.3s cubic-bezier(0.16,1,0.3,1)">';
    html+='<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:24px"><h3 style="font-size:20px;font-weight:700;margin:0">New '+entity+'</h3><button onclick="cronusModal.close(\'create-modal\')" style="background:none;border:none;cursor:pointer;padding:4px"><span class="material-symbols-outlined">close</span></button></div>';
    html+='<form id="create-form" onsubmit="return cronusSubmitCreate(event,\''+entity+'\')" style="display:flex;flex-direction:column;gap:16px">';
    fields.forEach(function(f){
      if(f.type==='enum'){
        html+='<div><label style="display:block;font-size:12px;font-weight:600;text-transform:uppercase;letter-spacing:0.1em;color:#71717a;margin-bottom:6px">'+f.name+'</label>';
        html+='<select name="'+f.name+'" style="width:100%;padding:10px 14px;border:1px solid #e5e7eb;border-radius:8px;font-size:14px;outline:none;font-family:Inter,sans-serif">';
        f.values.forEach(function(v){html+='<option value="'+v+'">'+v+'</option>'});
        html+='</select></div>';
      } else if(f.type==='boolean'){
        html+='<label style="display:flex;align-items:center;gap:10px;cursor:pointer"><input type="checkbox" name="'+f.name+'" style="width:18px;height:18px;accent-color:#000"><span style="font-size:14px">'+f.name+'</span></label>';
      } else {
        var inputType=f.type==='email'?'email':f.sensitive?'password':'text';
        html+='<div><label style="display:block;font-size:12px;font-weight:600;text-transform:uppercase;letter-spacing:0.1em;color:#71717a;margin-bottom:6px">'+f.name+'</label>';
        html+='<input type="'+inputType+'" name="'+f.name+'" '+(f.required?'required ':'')+' placeholder="Enter '+f.name.toLowerCase()+'..." style="width:100%;padding:10px 14px;border:1px solid #e5e7eb;border-radius:8px;font-size:14px;outline:none;font-family:Inter,sans-serif;transition:border-color 0.2s" onfocus="this.style.borderColor=\'#000\'" onblur="this.style.borderColor=\'#e5e7eb\'"></div>';
      }
    });
    html+='<div style="display:flex;gap:12px;margin-top:8px"><button type="button" onclick="cronusModal.close(\'create-modal\')" style="flex:1;padding:12px;border:1px solid #e5e7eb;border-radius:999px;background:#fff;font-size:14px;font-weight:600;cursor:pointer;font-family:Inter,sans-serif">Cancel</button>';
    html+='<button type="submit" style="flex:1;padding:12px;border:none;border-radius:999px;background:#000;color:#fff;font-size:14px;font-weight:700;cursor:pointer;font-family:Inter,sans-serif">Create</button></div>';
    html+='</form></div></div>';
    document.body.insertAdjacentHTML('beforeend',html);
  };

  window.cronusSubmitCreate=async function(e,entity){
    e.preventDefault();
    var form=document.getElementById('create-form');
    var data={};
    new FormData(form).forEach(function(v,k){data[k]=v});
    data.id=crypto.randomUUID?crypto.randomUUID():Date.now().toString(36);
    try{
      var r=await fetch('/api/'+entity.toLowerCase()+'s',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(data)});
      if(r.ok){cronusModal.close('create-modal');location.reload()}
      else{var err=await r.json();cronusToast(err.error||'Error','error')}
    }catch(ex){cronusToast('Connection error','error')}
    return false;
  };

  // Delete with confirmation
  window.cronusDelete=async function(entity,id){
    if(!confirm('Delete this '+entity+'?'))return;
    try{
      await fetch('/api/'+entity.toLowerCase()+'s/'+id,{method:'DELETE'});
      location.reload();
    }catch(ex){cronusToast('Error deleting','error')}
  };

  // Toast notification
  window.cronusToast=function(msg,type){
    var t=document.createElement('div');
    t.style.cssText='position:fixed;bottom:24px;right:24px;z-index:200;padding:12px 24px;border-radius:999px;font-size:14px;font-weight:600;color:#fff;animation:fadeIn 0.3s ease-out;font-family:Inter,sans-serif;box-shadow:0 4px 12px rgba(0,0,0,0.15)';
    t.style.background=type==='error'?'#dc2626':'#000';
    t.textContent=msg;
    document.body.appendChild(t);
    setTimeout(function(){t.style.opacity='0';t.style.transition='opacity 0.3s';setTimeout(function(){t.remove()},300)},3000);
  };

  // Schema-driven create modal
  window.cronusOpenCreate=async function(entity){
    try{
      var r=await fetch('/api/schema');
      var schema=await r.json();
      var entitySchema=schema.entities.find(function(e){return e.entity.toLowerCase()===entity.toLowerCase()});
      if(!entitySchema){cronusToast('Entity not found','error');return}
      var fields=entitySchema.fields.map(function(f){
        return {name:f.name,type:f.type,required:f.required,values:f.enum_values||[],sensitive:f.name==='password'};
      });
      cronusCreateModal(entity,fields);
      cronusModal.open('create-modal');
    }catch(ex){cronusToast('Error loading schema','error')}
  };

  // Fetch live counts for stat cards
  (function(){
    var stats=document.querySelectorAll('[data-entity-count]');
    stats.forEach(function(el){
      var entity=el.getAttribute('data-entity-count');
      fetch('/api/'+entity).then(function(r){return r.json()}).then(function(d){
        if(Array.isArray(d))el.textContent=d.length;
      }).catch(function(){});
    });
  })();

  // Live data containers
  (function(){
    var container=document.querySelector('[data-live]');
    if(!container)return;
    var entity=container.getAttribute('data-live');
    fetch('/api/'+entity).then(function(r){return r.json()}).then(function(members){
      if(!Array.isArray(members)||!members.length)return;
      var html='';
      members.forEach(function(m){
        var initial=(m.name||'?')[0].toUpperCase();
        var roleBg=m.role==='Admin'?'#000':m.role==='Developer'?'#006ff0':'#71717a';
        html+='<div style="display:flex;align-items:center;justify-content:space-between;padding:12px 0;border-bottom:1px solid #f3f3f3">';
        html+='<div style="display:flex;align-items:center;gap:16px">';
        html+='<div style="width:40px;height:40px;border-radius:50%;background:#e8e8e8;display:flex;align-items:center;justify-content:center;font-weight:600;font-size:14px">'+initial+'</div>';
        html+='<div><div style="font-weight:600;font-size:14px">'+m.name+'</div><div style="font-size:12px;color:#71717a">'+m.email+'</div></div>';
        html+='</div>';
        html+='<div style="display:flex;align-items:center;gap:12px">';
        html+='<span style="padding:4px 12px;border-radius:999px;font-size:11px;font-weight:600;color:#fff;background:'+roleBg+'">'+m.role+'</span>';
        html+='<button onclick="cronusDelete(\'TeamMember\',\''+m.id+'\')" style="background:none;border:none;cursor:pointer;opacity:0.3" onmouseover="this.style.opacity=1" onmouseout="this.style.opacity=0.3"><span class="material-symbols-outlined" style="font-size:18px;color:#dc2626">delete</span></button>';
        html+='</div></div>';
      });
      container.innerHTML=html;
    }).catch(function(){});
  })();

  // Form submission with feedback
  document.querySelectorAll('#cronus-form').forEach(function(form){
    form.addEventListener('submit',async function(e){
      e.preventDefault();
      var btn=form.querySelector('button[type=submit]');
      var msg=document.getElementById('form-msg');
      var action=form.getAttribute('action')||'/api/'+form.getAttribute('data-entity')+'s';
      var method=form.getAttribute('method')||'POST';
      var origLabel=btn.textContent;
      btn.disabled=true;btn.textContent='Saving...';
      var data={};
      new FormData(form).forEach(function(v,k){data[k]=v});
      try{
        var r=await fetch(action,{method:method,headers:{'Content-Type':'application/json'},body:JSON.stringify(data)});
        var body=await r.json();
        if(r.ok){
          msg.style.display='block';msg.style.background='#f0fdf4';msg.style.color='#16a34a';msg.style.border='1px solid #bbf7d0';
          msg.textContent='Saved successfully';form.reset();
          cronusToast('Created successfully','success');
          setTimeout(function(){msg.style.display='none'},3000);
        } else {
          msg.style.display='block';msg.style.background='#fef2f2';msg.style.color='#dc2626';msg.style.border='1px solid #fecaca';
          msg.textContent=(body.error||'Error saving');
        }
      }catch(err){
        msg.style.display='block';msg.style.background='#fef2f2';msg.style.color='#dc2626';msg.style.border='1px solid #fecaca';
        msg.textContent='Connection error';
      }
      btn.disabled=false;btn.textContent=btn.getAttribute('data-label')||origLabel;
    });
  });

  // Tabs
  document.querySelectorAll('.cronus-tab').forEach(btn=>{
    btn.addEventListener('click',function(){
      var panel=this.getAttribute('data-tab');
      var parent=this.closest('div').parentElement;
      parent.querySelectorAll('.cronus-tab').forEach(t=>{t.style.color='#71717a';t.style.borderBottomColor='transparent';t.style.fontWeight='500'});
      this.style.color='#000';this.style.borderBottomColor='#000';this.style.fontWeight='600';
      parent.querySelectorAll('.cronus-tab-panel').forEach(p=>{p.style.display='none'});
      parent.querySelector('[data-panel="'+panel+'"]').style.display='block';
      parent.querySelector('[data-panel="'+panel+'"]').style.animation='fadeIn 0.3s ease-out';
    });
  });

  // Accordion
  document.querySelectorAll('.cronus-accordion-trigger').forEach(btn=>{
    btn.addEventListener('click',function(){
      var content=this.nextElementSibling;
      var icon=this.querySelector('.material-symbols-outlined');
      if(content.style.display==='none'){content.style.display='block';icon.style.transform='rotate(180deg)'}
      else{content.style.display='none';icon.style.transform='rotate(0)'}
    });
  });

  // Chart bars animate on scroll
  document.querySelectorAll('.chart-bar').forEach(bar=>{
    var h=bar.style.height;bar.style.height='0';
    new IntersectionObserver(e=>{if(e[0].isIntersecting){bar.style.height=h}},{threshold:0.1}).observe(bar);
  });
});
</script>
"##;

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
  <style>{anim_css}</style>
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
  {anim_js}
</body>
</html>"#,
        app_name = app_name,
        body = body,
        tailwind_css = super::tailwind::CRONUS_TAILWIND,
        animations_css = super::animations::CRONUS_ANIMATIONS,
        anim_css = CRONUS_ANIMATIONS_CSS,
        anim_js = CRONUS_ANIMATIONS_JS,
        runtime = super::render::CRONUS_RUNTIME_JS,
        animate_js = super::animations::CRONUS_ANIMATE_JS,
        hmr = super::hmr::HMR_CLIENT_JS,
    )
}

/// Full-width layout for landing pages (no sidebar)
pub fn render_layout_landing(app_name: &str, body: &str, theme: &str) -> String {
    let is_light = theme == "light";
    let bg = if is_light { "#f9f9f9" } else { "#000" };
    let fg = if is_light { "#1a1a1a" } else { "#fff" };
    let sel_bg = if is_light { "rgba(0,0,0,0.08)" } else { "rgba(0,111,240,0.3)" };
    let scroll_thumb = if is_light { "rgba(0,0,0,0.1)" } else { "rgba(255,255,255,0.1)" };
    let grid_line = if is_light { "rgba(0,0,0,0.05)" } else { "rgba(255,255,255,0.03)" };
    // If body already contains a topbar section (rendered <header or <nav with data-topbar),
    // skip the built-in navbar to avoid duplication
    let has_topbar = body.contains("data-cronus-topbar");
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
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{app_name}</title>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet">
  <style>
    * {{ margin: 0; padding: 0; box-sizing: border-box; }}
    body {{ background: {bg}; color: {fg}; font-family: 'Inter', -apple-system, system-ui, sans-serif; -webkit-font-smoothing: antialiased; }}
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
    .anim {{ animation: fadeIn 0.6s ease-out both; }}
    .anim-d1 {{ animation-delay: 0.1s; }}
    .anim-d2 {{ animation-delay: 0.2s; }}
    .anim-d3 {{ animation-delay: 0.3s; }}
    .anim-d4 {{ animation-delay: 0.4s; }}
    .cursor-blink {{ animation: blink 1s step-end infinite; }}
    .pulse-glow {{ animation: pulseGlow 2s ease-in-out infinite; }}

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
<body>
  {nav_html}
  <main class="geist-grid" style="padding-top:64px;min-height:100vh">
  {body}
  </main>
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
        bg = bg, fg = fg, sel_bg = sel_bg, scroll_thumb = scroll_thumb, grid_line = grid_line,
        nav_html = nav_html,
        body = body,
        anim_css = CRONUS_ANIMATIONS_CSS,
        anim_js = CRONUS_ANIMATIONS_JS,
        tailwind_css = super::tailwind::CRONUS_TAILWIND,
        runtime = super::render::CRONUS_RUNTIME_JS,
        hmr = super::hmr::HMR_CLIENT_JS,
    )
}

// ══════════════════════════════════════════════════
// DASHBOARD LAYOUT — Light, Geist design system
// ══════════════════════════════════════════════════

pub fn render_layout_dashboard(app_name: &str, body: &str, theme: &str) -> String {
    let _ = theme; // reserved for future dark mode toggle
    format!(
        r##"<!DOCTYPE html>
<html class="light" lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{app_name}</title>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap" rel="stylesheet">
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet">
  <style>
    body {{ font-family:'Inter',sans-serif; background:#f9f9f9; color:#1a1c1c; margin:0; }}
    * {{ box-sizing:border-box; }}
    .material-symbols-outlined {{ font-variation-settings:'FILL' 0,'wght' 400; display:inline-block; line-height:1; vertical-align:middle; }}
    .prism-bg {{ background: radial-gradient(circle at top right,rgba(0,111,240,0.08),transparent 40%),radial-gradient(circle at bottom left,rgba(0,56,129,0.05),transparent 40%); }}
    .engineering-grid {{ background-image:linear-gradient(to right,rgba(198,198,198,0.1) 1px,transparent 1px),linear-gradient(to bottom,rgba(198,198,198,0.1) 1px,transparent 1px); background-size:40px 40px; }}
    .ghost-border {{ border:1px solid rgba(198,198,198,0.2); }}
    ::selection {{ background:rgba(0,111,240,0.15); }}
    ::-webkit-scrollbar {{ width:4px; }}
    ::-webkit-scrollbar-thumb {{ background:rgba(0,0,0,0.1); border-radius:2px; }}
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
</head>
<body>
  {body}
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
        runtime = super::render::CRONUS_RUNTIME_JS,
        hmr = super::hmr::HMR_CLIENT_JS,
    )
}

pub fn render_light_app_page(app_name: &str, comps: &[ComponentNode]) -> String {
    let topbar = comps.iter()
        .find(|c| c.style.as_deref().unwrap_or("").contains("topbar+light"))
        .map(render_light_topbar)
        .unwrap_or_default();
    let sidenav = comps.iter()
        .find(|c| c.style.as_deref().unwrap_or("").contains("sidenav+light"))
        .map(render_light_sidenav)
        .unwrap_or_default();
    let header = comps.iter()
        .find(|c| c.style.as_deref().unwrap_or("").contains("page-header+payouts"))
        .map(render_light_page_header)
        .unwrap_or_default();
    let balance = comps.iter()
        .find(|c| c.style.as_deref().unwrap_or("").contains("card+balance+light"))
        .map(render_light_balance_card)
        .unwrap_or_default();
    let upcoming = comps.iter()
        .find(|c| c.style.as_deref().unwrap_or("").contains("card+upcoming+light"))
        .map(render_light_upcoming_card)
        .unwrap_or_default();
    let actions = comps.iter()
        .find(|c| c.style.as_deref().unwrap_or("").contains("action-row+light"))
        .map(render_light_history_actions)
        .unwrap_or_default();
    let table = comps.iter()
        .find(|c| c.style.as_deref().unwrap_or("").contains("payouts-table+light"))
        .map(render_light_payouts_table)
        .unwrap_or_default();
    let support = comps.iter()
        .find(|c| c.style.as_deref().unwrap_or("").contains("support-banner+dark"))
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
<script>{hmr}</script>
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
        anim_js = CRONUS_ANIMATIONS_JS,
        hmr = super::hmr::HMR_CLIENT_JS,
    )
}

fn render_light_topbar(comp: &ComponentNode) -> String {
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

fn render_light_sidenav(comp: &ComponentNode) -> String {
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

fn render_light_page_header(comp: &ComponentNode) -> String {
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

fn render_light_balance_card(comp: &ComponentNode) -> String {
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

fn render_light_upcoming_card(comp: &ComponentNode) -> String {
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

fn render_light_history_actions(comp: &ComponentNode) -> String {
    let actions = items_by_kind(&comp.items, "action").iter().map(|a| {
        let icon = a.config.get("icon").map(|s| s.as_str()).unwrap_or("filter_list");
        format!(
            r#"<button style="display:inline-flex;align-items:center;gap:8px;background:#fff;color:#1a1c1c;padding:8px 16px;border-radius:999px;border:1px solid rgba(198,198,198,0.2);font-size:14px;font-weight:500;cursor:pointer"><span class="material-symbols-outlined" style="font-size:16px;color:#5e5e5e">{}</span>{}</button>"#,
            icon, a.text
        )
    }).collect::<Vec<_>>().join("");
    format!(r#"<div style="display:flex;gap:8px">{}</div>"#, actions)
}

fn render_light_payouts_table(comp: &ComponentNode) -> String {
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

fn render_light_support_banner(comp: &ComponentNode) -> String {
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

// ══════════════════════════════════════════════════
// PAGE RENDERER (returns inner body HTML)
// ══════════════════════════════════════════════════

pub fn render_page(page: &PageNode, entities: &[EntityNode], accent: &str, theme: &str) -> String {
    match page.page_type.as_str() {
        "dashboard" => render_dashboard(page, entities, accent),
        "list" => render_list(page, entities, accent),
        "form" => render_form(page, entities, accent),
        "detail" => render_list(page, entities, accent),
        "custom" => render_custom(page, accent, theme),
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
    if (enumFields.indexOf(field)===-1) return '<span style="font-size:13px;color:var(--foreground-muted)">'+(val||'\u2014')+'</span>';
    var v=(val||'').toLowerCase();
    var isPaid=v==='active'||v==='paid'||v==='completed'||v==='succeeded'||v==='approved';
    var isFail=v==='failed'||v==='cancelled'||v==='rejected'||v==='error';
    var dotColor=isPaid?'oklch(0.696 0.17 162)':isFail?'oklch(0.704 0.191 22)':'oklch(0.769 0.188 70)';
    var textColor=isPaid?'oklch(0.696 0.17 162)':isFail?'oklch(0.704 0.191 22)':'oklch(0.769 0.188 70)';
    var bgColor=isPaid?'oklch(0.696 0.17 162/8%)':isFail?'oklch(0.704 0.191 22/8%)':'oklch(0.769 0.188 70/8%)';
    return '<span style="display:inline-flex;align-items:center;gap:6px;padding:2px 10px;border-radius:20px;font-size:11px;font-weight:500;background:'+bgColor+';color:'+textColor+';border:1px solid '+dotColor.replace(')','/20%)')+'"><span style="width:5px;height:5px;border-radius:50%;background:'+dotColor+'"></span>'+(val||'\u2014')+'</span>';
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
    tbody.innerHTML=data.map(function(row,i){{
      var cells=fields.map(function(f){{
        return '<td style="padding:10px 16px;font-size:13px;color:var(--foreground-muted)">'+badge(row[f],f)+'</td>';
      }}).join('');
      var bg=i%2===0?'':'background:var(--surface-hover)';
      return '<tr style="border-bottom:1px solid var(--surface-hover);'+bg+';cursor:pointer" onmouseover="this.style.background=\'var(--surface-hover)\'" onclick="cronusEdit(\''+lower+'\',\''+((row.id||''))+'\')" data-id="'+(row.id||'')+'">'+cells+
        '<td style="padding:10px 8px;text-align:center;display:flex;gap:4px;align-items:center;justify-content:center">'+
        '<button onclick="event.stopPropagation();cronusDelete(\''+lower+'\',\''+(row.id||'')+'\')" style="background:none;border:none;cursor:pointer;opacity:0.3;padding:2px" onmouseover="this.style.opacity=1" onmouseout="this.style.opacity=0.3"><span class="material-symbols-outlined" style="font-size:16px;color:#dc2626">delete</span></button>'+
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

fn render_custom(page: &PageNode, accent: &str, theme: &str) -> String {
    let sections: Vec<String> = page.sections.iter()
        .map(|s| render_section(s, accent, theme))
        .collect();
    sections.join("\n")
}

fn render_section(section: &SectionNode, accent: &str, theme: &str) -> String {
    match section.section_type.as_str() {
        "hero" => render_hero(section, accent, theme),
        "features" => render_features(section, accent, theme),
        "pricing" => render_pricing(section, accent),
        "cta" => render_cta(section, accent, theme),
        "faq" => render_faq(section, accent),
        "stats" => render_stats(section, accent),
        "trusted" => render_trusted(section),
        "topbar" => render_topbar(section, theme),
        "checkout" => render_checkout_section(section),
        "testimonial" => render_testimonial(section),
        "footer" => render_footer(section, theme),
        "page-header" => render_page_header_section(section),
        "stat-cards" => render_stat_cards(section),
        "product-grid" => render_product_grid_section(section),
        "promo" => render_promo(section),
        "info-bar" => render_info_bar(section),
        "bento" => render_bento(section, accent),
        "features-split" => render_features_split(section, accent),
        "team-list" => render_team_list(section),
        "status-card" => render_status_card(section),
        "policies" => render_policies(section),
        "activity-table" => render_activity_table(section),
        "edge" => render_edge(section, accent),
        "sidebar" => render_sidebar(section),
        "form" => render_form_section(section),
        "card" | "live-keys" | "test-keys" | "webhooks" => render_card_section(section),
        "links" | "quick-links" => render_links_section(section),
        "tabs" => render_generic_section(section, accent),
        "accordion" => render_generic_section(section, accent),
        "breadcrumb" => render_generic_section(section, accent),
        "alert" => render_generic_section(section, accent),
        "chart" => render_generic_section(section, accent),
        _ => render_generic_section(section, accent),
    }
}

fn render_topbar(section: &SectionNode, theme: &str) -> String {
    let brand = section.config.get("brand").map(|s| s.as_str())
        .or(section.title.as_deref())
        .unwrap_or("Brand");
    let dark = section.config.get("style").map(|s| s.contains("dark")).unwrap_or(false) || theme == "dark";
    let (bg,bd,tx,mu) = if dark {
        ("rgba(0,0,0,0.8)","rgba(255,255,255,0.05)","#fff","#9ca3af")
    } else {
        ("rgba(255,255,255,0.8)","rgba(229,229,229,0.5)","#000","#71717a")
    };

    // Parse nav links from config (e.g. "Solutions, Resources, Docs, Pricing")
    let nav_links: Vec<String> = section.config.get("nav")
        .map(|nav| {
            nav.split(',').enumerate().map(|(i, link)| {
                let l = link.trim();
                // First link is "active" (bold)
                let (weight, color) = if i == 0 { ("600", tx) } else { ("500", mu) };
                format!(
                    r##"<a href="#" style="color:{color};font-size:14px;font-weight:{weight};letter-spacing:-0.025em;text-decoration:none;transition:color 0.15s" onmouseover="this.style.color='{tx}'" onmouseout="this.style.color='{color}'">{l}</a>"##,
                    color=color, weight=weight, tx=tx, l=l
                )
            }).collect()
        })
        .unwrap_or_default();

    let nav_html = nav_links.join("\n          ");

    // CTA button
    let cta_text = section.config.get("cta_text").or(section.config.get("cta")).map(|s| s.as_str()).unwrap_or("Deploy");
    let cta_link = section.config.get("cta_link").map(|s| s.as_str()).unwrap_or("/signup");
    let (btn_bg, btn_fg) = if dark { ("#fff","#000") } else { ("#000","#fff") };

    // Vercel triangle logo
    let logo_fill = tx;

    format!(r##"<header data-cronus-topbar class="anim-slide-down" style="position:fixed;top:0;width:100%;z-index:50;background:{bg};backdrop-filter:blur(12px);-webkit-backdrop-filter:blur(12px);border-bottom:1px solid {bd}">
  <div style="display:flex;justify-content:space-between;align-items:center;padding:0 24px;height:64px;max-width:1280px;margin:0 auto">
    <div style="display:flex;align-items:center;gap:32px">
      <a href="/" style="font-size:20px;font-weight:700;letter-spacing:-0.03em;color:{tx};display:flex;align-items:center;gap:8px;text-decoration:none">
        <svg width="22" height="20" viewBox="0 0 76 65" fill="{logo_fill}"><path d="M37.5274 0L75.0548 65L0 65L37.5274 0Z"/></svg>
        {brand}
      </a>
      <nav style="display:flex;align-items:center;gap:24px">
        {nav_html}
      </nav>
    </div>
    <div style="display:flex;align-items:center;gap:12px">
      <a href="#" style="color:{mu};font-size:14px;font-weight:500;text-decoration:none;padding:6px 16px;transition:color 0.15s" onmouseover="this.style.color='{tx}'" onmouseout="this.style.color='{mu}'">Contact</a>
      <a href="{cta_link}" style="display:inline-flex;align-items:center;padding:6px 20px;border-radius:999px;background:{btn_bg};color:{btn_fg};font-weight:700;font-size:14px;text-decoration:none;transition:transform 0.15s" onmouseover="this.style.transform='scale(1.05)'" onmouseout="this.style.transform='scale(1)'">{cta_text}</a>
    </div>
  </div>
</header>"##,
        bg=bg, bd=bd, tx=tx, mu=mu, brand=brand, logo_fill=logo_fill,
        nav_html=nav_html, cta_text=cta_text, cta_link=cta_link,
        btn_bg=btn_bg, btn_fg=btn_fg)
}

fn render_checkout_section(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("Checkout");
    let mut exp = String::new();
    let mut fld = String::new();
    let mut sub = String::from("Pay");
    let mut chk = String::new();
    for item in &section.items {
        let n = item.get("title").or_else(|| item.get("name")).map(|s| s.as_str()).unwrap_or("");
        let d = item.get("description").or_else(|| item.get("desc")).map(|s| s.as_str()).unwrap_or("");
        if d.starts_with("express") {
            let dk = d.contains("dark");
            let (b,c,br) = if dk {("black","white","none")} else {("white","black","1px solid #e5e5e5")};
            exp.push_str(&format!(r##"<button style="background:{b};color:{c};height:48px;border-radius:999px;border:{br};display:flex;align-items:center;justify-content:center;gap:8px;cursor:pointer;font-size:14px;font-weight:600;transition:opacity 0.2s" onmouseover="this.style.opacity='0.9'" onmouseout="this.style.opacity='1'">Pay with <b>{n}</b></button>"##, b=b,c=c,br=br,n=n));
        } else if d.starts_with("field:") {
            let p: Vec<&str> = d.splitn(3,':').collect();
            let ft = *p.get(1).unwrap_or(&"text"); let ph = *p.get(2).unwrap_or(&"");
            if ft == "card" {
                fld.push_str(&format!(r##"<div style="display:flex;flex-direction:column;gap:8px"><label style="font-size:11px;font-weight:600;letter-spacing:0.1em;text-transform:uppercase;color:#71717a">{n}</label><input type="text" placeholder="{ph}" style="width:100%;height:48px;padding:0 16px;border-radius:8px 8px 0 0;border:1px solid rgba(198,198,198,0.4);background:white;font-size:14px;outline:none" onfocus="this.style.borderColor='black'" onblur="this.style.borderColor='rgba(198,198,198,0.4)'"><div style="display:grid;grid-template-columns:1fr 1fr"><input type="text" placeholder="MM / YY" style="width:100%;height:48px;padding:0 16px;border-radius:0 0 0 8px;border:1px solid rgba(198,198,198,0.4);border-top:none;background:white;font-size:14px;outline:none"><input type="text" placeholder="CVC" style="width:100%;height:48px;padding:0 16px;border-radius:0 0 8px 0;border:1px solid rgba(198,198,198,0.4);border-top:none;border-left:none;background:white;font-size:14px;outline:none"></div></div>"##, n=n, ph=ph));
            } else if ft == "select" {
                let opts: String = ph.split(',').map(|o| format!("<option>{}</option>",o.trim())).collect::<Vec<_>>().join("");
                fld.push_str(&format!(r##"<div style="display:flex;flex-direction:column;gap:8px"><label style="font-size:11px;font-weight:600;letter-spacing:0.1em;text-transform:uppercase;color:#71717a">{n}</label><select style="width:100%;height:48px;padding:0 16px;border-radius:8px;border:1px solid rgba(198,198,198,0.4);background:white;font-size:14px;outline:none;appearance:none">{opts}</select></div>"##, n=n, opts=opts));
            } else {
                let it = if ft=="email"{"email"} else {"text"};
                fld.push_str(&format!(r##"<div style="display:flex;flex-direction:column;gap:8px"><label style="font-size:11px;font-weight:600;letter-spacing:0.1em;text-transform:uppercase;color:#71717a">{n}</label><input type="{it}" placeholder="{ph}" style="width:100%;height:48px;padding:0 16px;border-radius:8px;border:1px solid rgba(198,198,198,0.4);background:white;font-size:14px;outline:none" onfocus="this.style.borderColor='black'" onblur="this.style.borderColor='rgba(198,198,198,0.4)'"></div>"##, n=n, it=it, ph=ph));
            }
        } else if d == "checkbox" { chk = format!(r##"<label style="display:flex;align-items:center;gap:12px;cursor:pointer;padding-top:8px"><input type="checkbox" style="width:16px;height:16px;accent-color:black"><span style="font-size:14px;color:#52525b">{n}</span></label>"##, n=n);
        } else if d == "submit" { sub = n.to_string(); }
    }
    format!(r##"<main style="max-width:640px;margin:0 auto;padding:48px 24px 80px"><h1 style="font-size:30px;font-weight:700;letter-spacing:-0.02em;margin-bottom:32px">{title}</h1><div style="display:grid;grid-template-columns:1fr 1fr;gap:16px;margin-bottom:40px">{exp}</div><div style="display:flex;align-items:center;gap:16px;margin-bottom:32px"><div style="flex:1;height:1px;background:#e5e5e5"></div><span style="color:#a1a1aa;font-size:11px;font-weight:500;text-transform:uppercase;letter-spacing:0.1em">Or pay with card</span><div style="flex:1;height:1px;background:#e5e5e5"></div></div><form data-entity="order" style="display:flex;flex-direction:column;gap:24px">{fld}{chk}<button type="submit" style="width:100%;height:56px;border-radius:999px;background:black;color:white;font-size:18px;font-weight:700;border:none;cursor:pointer;display:flex;align-items:center;justify-content:center;gap:8px;margin-top:16px;transition:opacity 0.2s" onmouseover="this.style.opacity='0.9'" onmouseout="this.style.opacity='1'">{sub} <svg width="16" height="16" fill="rgba(255,255,255,0.5)" viewBox="0 0 24 24"><path d="M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zm-6 9c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2zm3.1-9H8.9V6c0-1.71 1.39-3.1 3.1-3.1s3.1 1.39 3.1 3.1v2z"/></svg></button><p style="text-align:center;font-size:12px;color:#a1a1aa;margin-top:8px;line-height:1.6">By confirming your payment, you agree to our Terms of Service and Privacy Policy.</p></form></main>"##, title=title, exp=exp, fld=fld, chk=chk, sub=sub)
}

fn render_testimonial(section: &SectionNode) -> String {
    let q = section.title.as_deref().unwrap_or("");
    let s = section.subtitle.as_deref().unwrap_or("");
    let dk = section.config.get("style").map(|s| s.contains("dark")).unwrap_or(false);
    let (bg,tx) = if dk {("black","white")} else {("#f3f3f3","#1a1a1a")};
    format!(r##"<div style="position:relative;padding:24px;background:{bg};color:{tx};border-radius:12px;overflow:hidden;max-width:640px;margin:24px auto"><p style="font-size:14px;font-weight:500;font-style:italic;line-height:1.6;opacity:0.9">"{q}"</p><p style="font-size:12px;font-weight:700;margin-top:16px;letter-spacing:0.08em;text-transform:uppercase">{s}</p><div style="position:absolute;inset:0;background:linear-gradient(135deg,rgba(0,111,240,0.2),transparent);opacity:0.5"></div></div>"##, bg=bg,tx=tx,q=q,s=s)
}

fn render_hero(section: &SectionNode, _accent: &str, theme: &str) -> String {
    let title = section.title.as_deref().unwrap_or("Build Something Amazing");
    let subtitle = section.subtitle.as_deref().unwrap_or("The next generation platform for modern teams.");
    let badge = section.config.get("badge").map(|s| s.as_str());
    let cta_primary = section.config.get("cta_text").or(section.config.get("cta")).map(|s| s.as_str()).unwrap_or("Get Started");
    let cta_link = section.config.get("cta_link").map(|s| s.as_str()).unwrap_or("/signup");
    let cta2_text = section.config.get("cta2_text").map(|s| s.as_str());
    let cta2_link = section.config.get("cta2_link").map(|s| s.as_str()).unwrap_or("");

    // Extract badge from items if not in config
    let badge_text = badge.or_else(|| {
        section.items.iter()
            .find(|i| i.get("badge").is_some() || i.get("title").map(|t| t.len() < 60).unwrap_or(false))
            .and_then(|i| i.get("badge").or(i.get("title")))
            .map(|s| s.as_str())
    });

    // Detect light theme: explicit style:light, or cta2 presence ONLY when global theme is not dark
    let style_hint = section.config.get("style").map(|s| s.as_str()).unwrap_or("");
    let is_dark = style_hint.contains("dark") || theme == "dark";
    let is_light = style_hint.contains("light") || (!is_dark && cta2_text.is_some());

    if is_light {
        return render_developer_landing_hero(section, title, subtitle, badge_text, cta_primary, cta_link, cta2_text, cta2_link);
    }

    // === Dark theme hero (original) ===
    let words: Vec<&str> = title.split_whitespace().collect();
    let mid = (words.len() + 1) / 2;
    let line1 = words[..mid].join(" ");
    let line2 = words[mid..].join(" ");

    let badge_html = badge_text.map(|b| format!(
        r#"<div class="anim-fade d1" style="display:inline-flex;align-items:center;gap:8px;padding:6px 16px;border-radius:999px;background:rgba(255,255,255,0.05);border:1px solid rgba(255,255,255,0.1);margin-bottom:32px;backdrop-filter:blur(8px)">
      <span style="width:8px;height:8px;border-radius:50%;background:#006ff0"></span>
      <span style="font-size:12px;font-weight:500;letter-spacing:0.05em;color:#a1a1aa">{}</span>
    </div>"#, b
    )).unwrap_or_default();

    let cta2_html = cta2_text.map(|t| format!(
        r#"<a href="{}" class="anim-scale d5 btn-hover" style="display:inline-flex;align-items:center;justify-content:center;padding:12px 32px;border-radius:999px;border:1px solid rgba(255,255,255,0.2);color:white;font-weight:600;font-size:16px;text-decoration:none;transition:all 0.2s" onmouseover="this.style.background='rgba(255,255,255,0.05)'" onmouseout="this.style.background='transparent'">{}</a>"#,
        cta2_link, t
    )).unwrap_or_default();

    format!(
        r##"<section style="position:relative;overflow:hidden;min-height:100vh;padding-top:80px;padding-bottom:80px;background:radial-gradient(circle at 50% -20%,rgba(0,111,240,0.15) 0%,rgba(0,0,0,0) 50%),conic-gradient(from 180deg at 50% 50%,rgba(255,255,255,0.03) 0deg,rgba(0,111,240,0.05) 120deg,rgba(255,0,128,0.05) 240deg,rgba(255,255,255,0.03) 360deg)">
  <!-- Grid background -->
  <div style="position:absolute;inset:0;background-image:linear-gradient(to right,rgba(255,255,255,0.03) 1px,transparent 1px),linear-gradient(to bottom,rgba(255,255,255,0.03) 1px,transparent 1px);background-size:40px 40px;opacity:0.4"></div>
  <div style="position:relative;z-index:10;max-width:1280px;margin:0 auto;padding:0 24px;text-align:center">
    {badge_html}
    <h1 class="anim-slide-up d2" style="font-size:clamp(48px,8vw,96px);font-weight:800;letter-spacing:-0.05em;color:white;margin-bottom:32px;line-height:1.1">
      {line1}<br>
      <span style="background:linear-gradient(to right,white,#6b7280);-webkit-background-clip:text;-webkit-text-fill-color:transparent;background-clip:text">{line2}</span>
    </h1>
    <p class="anim-slide-up d3" style="max-width:640px;margin:0 auto 48px;font-size:clamp(16px,2vw,20px);color:#9ca3af;line-height:1.6">{subtitle}</p>
    <div style="display:flex;flex-wrap:wrap;align-items:center;justify-content:center;gap:16px;margin-bottom:96px">
      <a href="{cta_link}" class="anim-scale d4 btn-hover" style="display:inline-flex;align-items:center;justify-content:center;padding:12px 32px;border-radius:999px;background:white;color:black;font-weight:600;font-size:16px;text-decoration:none;transition:transform 0.2s" onmouseover="this.style.transform='scale(1.02)'" onmouseout="this.style.transform='scale(1)'">{cta_primary}</a>
      {cta2_html}
    </div>
  </div>
</section>"##,
        badge_html = badge_html,
        line1 = line1, line2 = line2,
        subtitle = subtitle,
        cta_link = cta_link, cta_primary = cta_primary,
        cta2_html = cta2_html,
    )
}

/// Developer-landing hero: light theme, two-column grid with terminal window
fn render_developer_landing_hero(
    section: &SectionNode,
    title: &str,
    subtitle: &str,
    badge_text: Option<&str>,
    cta_primary: &str,
    cta_link: &str,
    cta2_text: Option<&str>,
    cta2_link: &str,
) -> String {
    // Badge
    let badge_html = badge_text.map(|b| format!(
        r#"<div class="anim anim-fade d1" style="display:inline-flex;align-items:center;gap:8px;padding:4px 12px;border-radius:999px;background:#e8e8e8;border:1px solid rgba(198,198,198,0.2);margin-bottom:24px">
      <span class="pulse-glow" style="width:8px;height:8px;border-radius:50%;background:#006ff0"></span>
      <span style="font-size:12px;font-weight:500;letter-spacing:0.05em;text-transform:uppercase;color:#1a1c1c">{}</span>
    </div>"#, b
    )).unwrap_or_default();

    // Split title by periods for line breaks (e.g. "Develop. Preview. Ship.")
    let title_lines: Vec<&str> = if title.contains('.') {
        title.split('.').map(|s| s.trim()).filter(|s| !s.is_empty()).collect()
    } else {
        title.split_whitespace().collect()
    };
    let title_html: String = title_lines.iter().enumerate().map(|(i, word)| {
        if i < title_lines.len() - 1 {
            format!("{}.<br>", word)
        } else {
            format!("{}.", word)
        }
    }).collect();

    // CTA2 (outline button)
    let cta2_html = cta2_text.map(|t| format!(
        r#"<a href="{link}" class="anim-scale d5 btn-hover" style="display:inline-flex;align-items:center;justify-content:center;padding:14px 32px;border-radius:999px;border:1px solid rgba(198,198,198,0.3);color:#1a1c1c;font-weight:700;font-size:16px;text-decoration:none;background:#fff;transition:all 0.2s" onmouseover="this.style.background='#f3f3f3'" onmouseout="this.style.background='#fff'">{text}</a>"#,
        link=cta2_link, text=t
    )).unwrap_or_default();

    // Terminal window HTML — built dynamically from section items
    let terminal_title = section.items.iter()
        .find(|i| i.get("style").map(|s| s.as_str()) == Some("terminal"))
        .and_then(|i| i.get("description"))
        .map(|s| s.as_str())
        .unwrap_or("terminal");

    let mut terminal_lines = String::new();
    for item in &section.items {
        let item_type = item.get("_type").map(|s| s.as_str()).unwrap_or("");
        let text = item.get("title").map(|s| s.as_str()).unwrap_or("");
        match item_type {
            "line" => {
                terminal_lines.push_str(&format!(
                    r#"<div><span style="color:#666">$</span> <span style="color:#e5e5e5">{}</span></div>"#,
                    text.trim_start_matches("$ ")
                ));
            }
            "output" => {
                let color = item.get("color").map(|s| match s.as_str() {
                    "blue" => "#006ff0",
                    "green" => "#28c840",
                    "yellow" => "#febc2e",
                    "red" => "#ff5f57",
                    _ => "#666",
                }).unwrap_or("#666");
                terminal_lines.push_str(&format!(
                    r#"<div style="color:{};margin-top:4px">{}</div>"#, color, text
                ));
            }
            "prompt" => {
                let answer = item.get("answer").map(|s| s.as_str()).unwrap_or("");
                terminal_lines.push_str(&format!(
                    r#"<div><span style="color:#666">?</span> <span style="color:#e5e5e5">{}</span> <span style="color:#006ff0">{}</span></div>"#,
                    text, answer
                ));
            }
            "success" => {
                terminal_lines.push_str(&format!(
                    r#"<div><span style="color:#28c840">✓</span> <span style="color:#e5e5e5">{}</span></div>"#,
                    text
                ));
            }
            _ => {}
        }
    }
    // Add blinking cursor
    terminal_lines.push_str(r#"<div style="margin-top:12px"><span class="cursor-blink" style="display:inline-block;width:8px;height:16px;background:#e5e5e5;vertical-align:middle"></span></div>"#);

    let terminal_html = format!(
        r##"<div class="anim anim-d3 anim-scale d4" style="background:#000;border-radius:12px;overflow:hidden;box-shadow:0 25px 50px rgba(0,0,0,0.25);border:1px solid #1f2937">
      <div class="terminal-header" style="display:flex;align-items:center;justify-content:space-between;padding:12px 16px;border-bottom:1px solid #1f2937">
        <div style="display:flex;gap:8px">
          <span style="width:12px;height:12px;border-radius:50%;background:#ff5f56"></span>
          <span style="width:12px;height:12px;border-radius:50%;background:#ffbd2e"></span>
          <span style="width:12px;height:12px;border-radius:50%;background:#27c93f"></span>
        </div>
        <span style="font-size:10px;font-family:'JetBrains Mono',monospace;color:#6b7280;text-transform:uppercase;letter-spacing:0.1em">{title}</span>
        <div style="width:48px"></div>
      </div>
      <div style="padding:24px;font-family:'JetBrains Mono',monospace;font-size:14px;line-height:1.625;color:#e5e5e5">
        {lines}
      </div>
    </div>"##,
        title = terminal_title,
        lines = terminal_lines,
    );

    // Floating chip badges from section items
    let chips: Vec<String> = section.items.iter()
        .filter(|i| i.get("_type").map(|s| s.as_str()) == Some("chip"))
        .map(|i| {
            let text = i.get("title").map(|s| s.as_str()).unwrap_or("");
            let icon = i.get("icon").map(|s| s.as_str()).unwrap_or("●");
            format!(
                r#"<div class="anim-fade d6" style="background:#fff;border:1px solid rgba(198,198,198,0.2);border-radius:8px;padding:12px;display:flex;align-items:center;gap:12px;box-shadow:0 4px 16px rgba(0,0,0,0.08);animation:float 3s ease-in-out infinite">
              <div style="width:24px;height:24px;display:flex;align-items:center;justify-content:center;font-weight:700;font-size:8px;border:1px solid #000;border-radius:3px">{}</div>
              <span style="font-size:12px;font-weight:600">{}</span>
            </div>"#, icon, text
            )
        })
        .collect();
    let chips_html = if chips.is_empty() { String::new() } else {
        format!(r#"<div style="position:absolute;bottom:-24px;right:-24px;display:flex;flex-direction:column;gap:8px;z-index:20">{}</div>"#, chips.join("\n"))
    };

    // Wrap terminal + chips in relative container
    let terminal_with_chips = format!(
        r#"<div style="position:relative">{}{}</div>"#,
        terminal_html, chips_html
    );

    // Stat cards embedded in hero (role:stat items)
    let stat_items: Vec<&std::collections::HashMap<String, String>> = section.items.iter()
        .filter(|i| i.get("role").map(|s| s.as_str()) == Some("stat"))
        .collect();

    let stats_html = if stat_items.is_empty() {
        String::new()
    } else {
        let cols = stat_items.len();
        let cards: Vec<String> = stat_items.iter().map(|item| {
            let label = item.get("title").map(|s| s.as_str()).unwrap_or("");
            let value = item.get("description").map(|s| s.as_str()).unwrap_or("");
            format!(
                r#"<div style="background:rgba(255,255,255,0.6);backdrop-filter:blur(8px);padding:32px;border:1px solid rgba(198,198,198,0.2);text-align:left">
              <div style="font-size:12px;font-weight:500;letter-spacing:0.1em;text-transform:uppercase;color:#777;margin-bottom:8px">{label}</div>
              <div style="font-size:clamp(32px,5vw,48px);font-weight:700;letter-spacing:-0.04em;color:#000">{value}</div>
            </div>"#,
                label = label, value = value
            )
        }).collect();
        format!(
            r#"<div style="max-width:1280px;margin:80px auto 0;padding:0 24px;position:relative;z-index:10">
          <div style="display:grid;grid-template-columns:repeat({cols},1fr);gap:1px;border-radius:12px;overflow:hidden;border:1px solid rgba(198,198,198,0.2)">
            {cards}
          </div>
        </div>"#,
            cols = cols,
            cards = cards.join("\n")
        )
    };

    // Detect if hero has terminal content or just stats
    let has_terminal = terminal_lines.contains("<div>");

    if !has_terminal && !stat_items.is_empty() {
        // Centered hero layout with stat cards below (no terminal)
        return format!(
            r##"<section style="position:relative;overflow:hidden;min-height:80vh;padding:96px 24px 80px;display:flex;flex-direction:column;align-items:center;justify-content:center;background-image:linear-gradient(to right,rgba(198,198,198,0.1) 1px,transparent 1px),linear-gradient(to bottom,rgba(198,198,198,0.1) 1px,transparent 1px);background-size:40px 40px">
  <div class="prism-glow" style="position:absolute;inset:0;pointer-events:none;background:radial-gradient(circle at 50% 50%,rgba(0,111,240,0.08) 0%,rgba(249,249,249,0) 70%)"></div>
  <div style="position:relative;z-index:10;max-width:1024px;margin:0 auto;padding:0 24px;text-align:center">
    {badge_html}
    <h1 class="anim anim-d1" style="font-size:clamp(48px,8vw,96px);font-weight:800;letter-spacing:-0.05em;color:#000;line-height:0.9;margin-bottom:32px">
      {title_html}
    </h1>
    <p class="anim anim-d2" style="max-width:640px;margin:0 auto 48px;font-size:18px;color:#474747;line-height:1.625">{subtitle}</p>
    <div class="anim anim-d2" style="display:flex;flex-wrap:wrap;align-items:center;justify-content:center;gap:16px">
      <a href="{cta_link}" class="anim-scale d4 btn-hover" style="display:inline-flex;align-items:center;justify-content:center;gap:8px;padding:14px 32px;border-radius:999px;background:#000;color:#fff;font-weight:700;font-size:16px;text-decoration:none;transition:all 0.2s" onmouseover="this.style.transform='scale(1.05)'" onmouseout="this.style.transform='scale(1)'">{cta_primary}</a>
      {cta2_html}
    </div>
  </div>
  {stats_html}
</section>"##,
            badge_html = badge_html,
            title_html = title_html,
            subtitle = subtitle,
            cta_link = cta_link,
            cta_primary = cta_primary,
            cta2_html = cta2_html,
            stats_html = stats_html,
        );
    }

    format!(
        r##"<section style="position:relative;overflow:hidden;padding:96px 24px 128px">
  <div class="prism-glow" style="position:absolute;inset:0;pointer-events:none"></div>
  <div style="position:relative;z-index:10;max-width:1280px;margin:0 auto;padding:0 24px">
    <div style="display:grid;grid-template-columns:1fr 1fr;gap:64px;align-items:center">
      <!-- Left: Text content -->
      <div>
        {badge_html}
        <h1 class="anim anim-d1" style="font-size:clamp(48px,8vw,96px);font-weight:800;letter-spacing:-0.05em;color:#000;line-height:0.9;margin-bottom:32px">
          {title_html}
        </h1>
        <p class="anim anim-d2" style="max-width:512px;font-size:18px;color:#474747;line-height:1.625;margin-bottom:40px">{subtitle}</p>
        <div class="anim anim-d2" style="display:flex;flex-wrap:wrap;align-items:center;gap:16px">
          <a href="{cta_link}" class="anim-scale d4 btn-hover" style="display:inline-flex;align-items:center;justify-content:center;gap:8px;padding:14px 32px;border-radius:999px;background:#000;color:#fff;font-weight:700;font-size:16px;text-decoration:none;transition:all 0.2s" onmouseover="this.style.transform='scale(1.05)'" onmouseout="this.style.transform='scale(1)'">{cta_primary}<span class="material-symbols-outlined" style="font-size:14px">arrow_forward</span></a>
          {cta2_html}
        </div>
      </div>
      <!-- Right: Terminal window -->
      {terminal_with_chips}
    </div>
  </div>
</section>"##,
        badge_html = badge_html,
        title_html = title_html,
        subtitle = subtitle,
        cta_link = cta_link,
        cta_primary = cta_primary,
        cta2_html = cta2_html,
        terminal_with_chips = terminal_with_chips,
    )
}

fn render_features(section: &SectionNode, _accent: &str, theme: &str) -> String {
    let style_hint = section.config.get("style").map(|s| s.as_str()).unwrap_or("");

    // Light bento layout — trigger on style:bento, or auto-detect from mixed spans/children
    let has_spans = section.items.iter().any(|i| i.get("span").is_some());
    let has_typed_children = section.items.iter().any(|i| {
        let t = i.get("_type").map(|s| s.as_str()).unwrap_or("");
        matches!(t, "chip" | "label" | "code" | "image")
    });
    if style_hint.contains("bento") || has_spans || has_typed_children {
        return render_features_bento_light(section);
    }

    let items: Vec<String> = section.items.iter().enumerate().map(|(i, item)| {
        let name = item.get("title").or_else(|| item.get("name")).map(|s| s.as_str()).unwrap_or("Feature");
        let desc = item.get("description").or_else(|| item.get("desc")).map(|s| s.as_str()).unwrap_or("");
        let icon_name = item.get("icon").map(|s| s.as_str()).unwrap_or("star");

        let icon_color = if theme == "dark" || style_hint.contains("dark") { "#fff" } else { "#000" };
        let icon_svg = get_material_icon(icon_name, icon_color, 20);

        format!(
            r#"<div style="grid-column:span 4;background:#0a0a0a;border-radius:12px;border:1px solid rgba(255,255,255,0.05);padding:32px;display:flex;flex-direction:column;justify-content:space-between;transition:border-color 0.3s" onmouseover="this.style.borderColor='rgba(255,255,255,0.2)'" onmouseout="this.style.borderColor='rgba(255,255,255,0.05)'">
  <div>
    <div style="width:40px;height:40px;border-radius:8px;background:rgba(255,255,255,0.05);display:flex;align-items:center;justify-content:center;margin-bottom:24px;color:white">{icon_svg}</div>
    <h3 style="font-size:20px;font-weight:700;letter-spacing:-0.02em;color:white;margin-bottom:8px">{name}</h3>
    <p style="color:#9ca3af;font-size:14px;line-height:1.6">{desc}</p>
  </div>
</div>"#, icon_svg = icon_svg, name = name, desc = desc)
    }).collect();

    format!(
        r#"<section style="max-width:1280px;margin:0 auto;padding:0 24px">
  <div style="display:grid;grid-template-columns:repeat(12,1fr);gap:16px">
    {}
  </div>
</section>"#,
        items.join("\n    "),
    )
}

/// Material symbol icon helper
fn get_material_icon(name: &str, color: &str, size: u32) -> String {
    match name {
        "commit" => format!(r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M16.89 12.5a5.001 5.001 0 00-9.78 0H2v-1h5.11a5.001 5.001 0 019.78 0H22v1h-5.11zM12 15a3 3 0 110-6 3 3 0 010 6z"/></svg>"#, s=size, c=color),
        "extension" => format!(r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M20.5 11H19V7a2 2 0 00-2-2h-4V3.5a2.5 2.5 0 00-5 0V5H4a2 2 0 00-2 2v3.8h1.5a2.7 2.7 0 010 5.4H2V20a2 2 0 002 2h3.8v-1.5a2.7 2.7 0 015.4 0V22H17a2 2 0 002-2v-4h1.5a2.5 2.5 0 000-5z"/></svg>"#, s=size, c=color),
        "public" | "globe" => format!(r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/></svg>"#, s=size, c=color),
        "terminal" => format!(r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M20 4H4a2 2 0 00-2 2v12a2 2 0 002 2h16a2 2 0 002-2V6a2 2 0 00-2-2zm0 14H4V8h16v10zm-2-1h-6v-2h6v2zM7.5 17l-1.41-1.41L8.67 13l-2.59-2.59L7.5 9l4 4-4 4z"/></svg>"#, s=size, c=color),
        "code" => format!(r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M9.4 16.6L4.8 12l4.6-4.6L8 6l-6 6 6 6 1.4-1.4zm5.2 0l4.6-4.6-4.6-4.6L16 6l6 6-6 6-1.4-1.4z"/></svg>"#, s=size, c=color),
        "upload" | "cloud_upload" => format!(r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M19.35 10.04A7.49 7.49 0 0012 4C9.11 4 6.6 5.64 5.35 8.04A5.994 5.994 0 000 14c0 3.31 2.69 6 6 6h13c2.76 0 5-2.24 5-5 0-2.64-2.05-4.78-4.65-4.96zM14 13v4h-4v-4H7l5-5 5 5h-3z"/></svg>"#, s=size, c=color),
        "rocket_launch" | "rocket" => format!(r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M12 2.5s4.5 2 4.5 9.5c0 2.08-.52 3.88-1.26 5.35l-1.62-1.62a2 2 0 00-3.24 0l-1.62 1.62A14.83 14.83 0 017.5 12c0-7.5 4.5-9.5 4.5-9.5zM5 18l1.38-1.37c.49-.49 1.11-.83 1.78-.97l1.85 1.85c-.06.85-.27 1.7-.64 2.49H5zm14 0h-4.37c-.37-.79-.58-1.64-.64-2.49l1.85-1.85c.67.14 1.29.48 1.78.97L19 18zM12 12.5a1.5 1.5 0 100-3 1.5 1.5 0 000 3z"/></svg>"#, s=size, c=color),
        _ => format!(r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/></svg>"#, s=size, c=color),
    }
}

/// Bento-style feature grid for light developer landing.
/// Reads ALL data from section.items dynamically:
/// - Items without `_type` (or empty) are feature cards
/// - Items with `_type` (label, chip, code, image) are children of the preceding card
fn render_features_bento_light(section: &SectionNode) -> String {
    // Group flat items into cards + their children.
    // Items without _type (or empty) are feature CARDS.
    // Items with _type (label, chip, code, image) belong to the preceding card as children.
    struct CardGroup {
        card: std::collections::HashMap<String, String>,
        children: Vec<std::collections::HashMap<String, String>>,
    }

    let mut cards: Vec<CardGroup> = Vec::new();

    for item in &section.items {
        let item_type = item.get("_type").map(|s| s.as_str()).unwrap_or("");
        if item_type.is_empty() {
            cards.push(CardGroup {
                card: item.clone(),
                children: Vec::new(),
            });
        } else if let Some(last) = cards.last_mut() {
            last.children.push(item.clone());
        }
    }

    let card_htmls: Vec<String> = cards.iter().enumerate().map(|(i, group)| {
        let card = &group.card;
        let name = card.get("title").or_else(|| card.get("name")).map(|s| s.as_str()).unwrap_or("Feature");
        let desc = card.get("description").or_else(|| card.get("desc")).map(|s| s.as_str()).unwrap_or("");
        let icon_name = card.get("icon").map(|s| s.as_str()).unwrap_or("star");
        let span = card.get("span").and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);
        let card_style = card.get("style").map(|s| s.as_str()).unwrap_or("");
        let is_dark = card_style.contains("dark");
        let delay_class = format!("reveal card-hover anim anim-d{}", (i % 4) + 1);

        let col_span_css = if span > 1 {
            format!("grid-column:span {};", span)
        } else {
            String::new()
        };

        // Classify children
        let has_image = group.children.iter().any(|c| c.get("_type").map(|s| s.as_str()) == Some("image"));
        let has_label = group.children.iter().any(|c| c.get("_type").map(|s| s.as_str()) == Some("label"));
        let has_chips = group.children.iter().any(|c| c.get("_type").map(|s| s.as_str()) == Some("chip"));
        let has_code = group.children.iter().any(|c| c.get("_type").map(|s| s.as_str()) == Some("code"));
        let use_horizontal = span > 1 && has_image;

        if is_dark {
            // ── DARK CARD ──
            let chips_html = if has_chips {
                let chips: Vec<String> = group.children.iter()
                    .filter(|c| c.get("_type").map(|s| s.as_str()) == Some("chip"))
                    .map(|c| {
                        let t = c.get("title").map(|s| s.as_str()).unwrap_or("");
                        format!(
                            r#"<span style="padding:4px 12px;background:rgba(255,255,255,0.1);border-radius:999px;font-size:10px;font-weight:700;letter-spacing:0.1em;text-transform:uppercase;color:#fff">{}</span>"#,
                            t.to_uppercase()
                        )
                    }).collect();
                format!(
                    r#"<div style="display:flex;flex-wrap:wrap;gap:8px;margin-top:32px">{}</div>"#,
                    chips.join("")
                )
            } else {
                String::new()
            };

            format!(
                r##"<div class="{delay_class}" style="{col_span}background:#000;border-radius:12px;padding:32px;display:flex;flex-direction:column;justify-content:space-between">
  <div>
    <span class="material-symbols-outlined" style="font-size:32px;color:#fff;margin-bottom:16px;display:block">{icon}</span>
    <h3 style="font-size:24px;font-weight:700;letter-spacing:-0.02em;color:#fff;margin-bottom:12px">{name}</h3>
    <p style="color:#9ca3af;font-size:14px;line-height:1.6">{desc}</p>
  </div>
  {chips}
</div>"##,
                delay_class = delay_class,
                col_span = col_span_css,
                icon = icon_name,
                name = name,
                desc = desc,
                chips = chips_html,
            )
        } else if use_horizontal {
            // ── LIGHT CARD, SPAN 2, WITH IMAGE → HORIZONTAL LAYOUT ──
            let non_image_children = render_bento_children_light(&group.children, false, true);
            let img_src = group.children.iter()
                .filter(|c| c.get("_type").map(|s| s.as_str()) == Some("image"))
                .filter_map(|c| c.get("src").or(c.get("url")))
                .next()
                .map(|s| s.trim_matches('"'))
                .unwrap_or("");
            let img_alt = group.children.iter()
                .filter(|c| c.get("_type").map(|s| s.as_str()) == Some("image"))
                .filter_map(|c| c.get("title"))
                .next()
                .map(|s| s.as_str())
                .unwrap_or("");

            format!(
                r##"<div class="{delay_class}" style="{col_span}background:#fff;border:1px solid rgba(198,198,198,0.1);border-radius:12px;padding:32px;overflow:hidden">
  <div style="display:flex;gap:32px;align-items:center">
    <div style="flex:1">
      <span class="material-symbols-outlined" style="font-size:32px;color:#000;margin-bottom:16px;display:block">{icon}</span>
      <h3 style="font-size:24px;font-weight:700;letter-spacing:-0.02em;color:#1a1c1c;margin-bottom:12px">{name}</h3>
      <p style="color:#474747;font-size:14px;line-height:1.6">{desc}</p>
      {text_children}
    </div>
    <div style="flex:1">
      <img src="{img_src}" style="border-radius:8px;width:100%;box-shadow:0 20px 40px rgba(0,0,0,0.15);transition:transform 0.5s" onmouseover="this.style.transform='rotate(2deg)'" onmouseout="this.style.transform='none'" alt="{img_alt}">
    </div>
  </div>
</div>"##,
                delay_class = delay_class,
                col_span = col_span_css,
                icon = icon_name,
                name = name,
                desc = desc,
                text_children = non_image_children,
                img_src = img_src,
                img_alt = img_alt,
            )
        } else if has_label {
            // ── LIGHT CARD WITH LABEL → flow icons + label at bottom (Git Push pattern) ──
            let label_text = group.children.iter()
                .filter(|c| c.get("_type").map(|s| s.as_str()) == Some("label"))
                .filter_map(|c| c.get("title"))
                .next()
                .map(|s| s.as_str())
                .unwrap_or("");

            let flow_html = format!(
                r##"<div style="display:flex;align-items:center;gap:16px;margin-top:48px">
    <div style="display:flex">
      <div style="width:40px;height:40px;border-radius:50%;background:#f3f3f3;border:2px solid #fff;display:flex;align-items:center;justify-content:center"><span class="material-symbols-outlined" style="font-size:16px">code</span></div>
      <div style="width:40px;height:40px;border-radius:50%;background:#000;border:2px solid #fff;display:flex;align-items:center;justify-content:center;margin-left:-8px"><span class="material-symbols-outlined" style="font-size:16px;color:#fff">upload</span></div>
      <div style="width:40px;height:40px;border-radius:50%;background:#006ff0;border:2px solid #fff;display:flex;align-items:center;justify-content:center;margin-left:-8px"><span class="material-symbols-outlined" style="font-size:16px;color:#fff">rocket_launch</span></div>
    </div>
    <div style="flex:1;height:1px;background:rgba(198,198,198,0.3)"></div>
    <span style="font-size:11px;font-family:'JetBrains Mono',monospace;color:#9ca3af;letter-spacing:0.08em">{label}</span>
  </div>"##,
                label = label_text
            );

            format!(
                r##"<div class="{delay_class}" style="{col_span}background:#fff;border:1px solid rgba(198,198,198,0.1);border-radius:12px;padding:32px;display:flex;flex-direction:column;justify-content:space-between">
  <div>
    <span class="material-symbols-outlined" style="font-size:32px;color:#000;margin-bottom:16px;display:block">{icon}</span>
    <h3 style="font-size:24px;font-weight:700;letter-spacing:-0.02em;color:#1a1c1c;margin-bottom:12px">{name}</h3>
    <p style="color:#474747;font-size:14px;line-height:1.6;max-width:480px">{desc}</p>
  </div>
  {flow}
</div>"##,
                delay_class = delay_class,
                col_span = col_span_css,
                icon = icon_name,
                name = name,
                desc = desc,
                flow = flow_html,
            )
        } else if has_image && span <= 1 {
            // ── LIGHT CARD, SPAN 1, WITH IMAGE → vertical layout, image below ──
            let img_src = group.children.iter()
                .filter(|c| c.get("_type").map(|s| s.as_str()) == Some("image"))
                .filter_map(|c| c.get("src").or(c.get("url")))
                .next()
                .map(|s| s.trim_matches('"'))
                .unwrap_or("");
            let img_alt = group.children.iter()
                .filter(|c| c.get("_type").map(|s| s.as_str()) == Some("image"))
                .filter_map(|c| c.get("title"))
                .next()
                .map(|s| s.as_str())
                .unwrap_or("");

            let img_html = if !img_src.is_empty() {
                format!(
                    r#"<div style="margin-top:24px;aspect-ratio:1;border-radius:8px;overflow:hidden;background:#e8e8e8"><img src="{}" style="width:100%;height:100%;object-fit:cover;filter:grayscale(100%);transition:filter 0.5s" onmouseover="this.style.filter='none'" onmouseout="this.style.filter='grayscale(100%)'" alt="{}"></div>"#,
                    img_src, img_alt
                )
            } else {
                r#"<div style="margin-top:24px;aspect-ratio:1;background:#e8e8e8;border-radius:8px;overflow:hidden"></div>"#.to_string()
            };

            format!(
                r##"<div class="{delay_class}" style="{col_span}background:#fff;border:1px solid rgba(198,198,198,0.1);border-radius:12px;padding:32px;display:flex;flex-direction:column">
  <span class="material-symbols-outlined" style="font-size:32px;color:#000;margin-bottom:16px;display:block">{icon}</span>
  <h3 style="font-size:20px;font-weight:700;letter-spacing:-0.02em;color:#1a1c1c;margin-bottom:8px">{name}</h3>
  <p style="font-size:14px;color:#474747;flex:1">{desc}</p>
  {img}
</div>"##,
                delay_class = delay_class,
                col_span = col_span_css,
                icon = icon_name,
                name = name,
                desc = desc,
                img = img_html,
            )
        } else {
            // ── LIGHT CARD, generic fallback ──
            let children_html = render_bento_children_light(&group.children, false, false);
            format!(
                r##"<div class="{delay_class}" style="{col_span}background:#fff;border:1px solid rgba(198,198,198,0.1);border-radius:12px;padding:32px;display:flex;flex-direction:column;justify-content:space-between">
  <div>
    <span class="material-symbols-outlined" style="font-size:32px;color:#000;margin-bottom:16px;display:block">{icon}</span>
    <h3 style="font-size:20px;font-weight:700;letter-spacing:-0.02em;color:#1a1c1c;margin-bottom:8px">{name}</h3>
    <p style="color:#474747;font-size:14px;line-height:1.6;max-width:480px">{desc}</p>
  </div>
  {children}
</div>"##,
                delay_class = delay_class,
                col_span = col_span_css,
                icon = icon_name,
                name = name,
                desc = desc,
                children = children_html,
            )
        }
    }).collect();

    format!(
        r##"<section style="padding:96px 24px;background:rgba(243,243,243,0.5)">
  <div style="max-width:1280px;margin:0 auto">
    <div class="stagger" style="display:grid;grid-template-columns:repeat(3,1fr);gap:24px">
      {items}
    </div>
  </div>
</section>"##,
        items = card_htmls.join("\n      "),
    )
}

/// Render non-image child items (code, chip) for light bento cards.
/// `skip_images` filters out image children (used in horizontal layout where images are handled separately).
fn render_bento_children_light(
    children: &[std::collections::HashMap<String, String>],
    _is_dark: bool,
    skip_images: bool,
) -> String {
    let mut code_pills: Vec<String> = Vec::new();
    let mut chip_pills: Vec<String> = Vec::new();

    for child in children {
        let child_type = child.get("_type").map(|s| s.as_str()).unwrap_or("");
        let title = child.get("title").map(|s| s.as_str()).unwrap_or("");

        match child_type {
            "code" => {
                code_pills.push(format!(
                    r#"<code style="padding:8px 16px;background:#eeeeee;border-radius:8px;font-family:'JetBrains Mono',monospace;font-size:13px;border:1px solid rgba(198,198,198,0.2)">{}</code>"#,
                    title
                ));
            }
            "chip" => {
                chip_pills.push(format!(
                    r#"<span style="padding:4px 12px;background:#e8e8e8;border-radius:999px;font-size:10px;font-weight:700;letter-spacing:0.1em;text-transform:uppercase;color:#1a1c1c">{}</span>"#,
                    title.to_uppercase()
                ));
            }
            "image" if skip_images => { /* handled externally */ }
            "label" => { /* handled by parent card logic */ }
            _ => {}
        }
    }

    let mut parts: Vec<String> = Vec::new();

    if !code_pills.is_empty() {
        parts.push(format!(
            r#"<div style="display:flex;flex-wrap:wrap;gap:12px;margin-top:24px">{}</div>"#,
            code_pills.join("")
        ));
    }
    if !chip_pills.is_empty() {
        parts.push(format!(
            r#"<div style="display:flex;flex-wrap:wrap;gap:8px;margin-top:32px">{}</div>"#,
            chip_pills.join("")
        ));
    }

    parts.join("\n")
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

fn render_cta(section: &SectionNode, _accent: &str, theme: &str) -> String {
    let title = section.title.as_deref().unwrap_or("Get Started");
    let subtitle = section.subtitle.as_deref().unwrap_or("");
    let cta_text = section.config.get("cta_text").map(|s| s.as_str()).unwrap_or("Get Started");
    let cta_link = section.config.get("cta_link").map(|s| s.as_str()).unwrap_or("/signup");
    let cta2_text = section.config.get("cta2_text").map(|s| s.as_str());
    let cta2_link = section.config.get("cta2_link").map(|s| s.as_str()).unwrap_or("");
    let footnote = section.config.get("footnote").map(|s| s.as_str());
    let is_dark = section.config.get("style").map(|s| s.contains("dark")).unwrap_or(false) || theme == "dark";
    let is_light = !is_dark && (cta2_text.is_some() || section.config.get("style").map(|s| s.contains("light")).unwrap_or(false));

    if is_light {
        // Light theme CTA: italic title, centered
        let cta2_html = cta2_text.map(|t| format!(
            r#"<a href="{link}" style="display:inline-flex;align-items:center;justify-content:center;padding:16px 48px;border-radius:999px;border:1px solid rgba(198,198,198,0.3);color:#1a1c1c;font-weight:700;font-size:18px;text-decoration:none;background:#fff;transition:all 0.2s" onmouseover="this.style.background='#f3f3f3'" onmouseout="this.style.background='#fff'">{text}</a>"#,
            link=cta2_link, text=t
        )).unwrap_or_default();

        let footnote_html = footnote.or(Some(subtitle)).filter(|s| !s.is_empty()).map(|f| format!(
            r#"<p style="font-size:14px;color:#474747;margin-top:32px">{}</p>"#, f
        )).unwrap_or_default();

        return format!(
            r##"<section style="padding:128px 24px;position:relative;overflow:hidden">
  <div style="position:relative;max-width:960px;margin:0 auto;text-align:center">
    <h2 class="anim reveal" style="font-size:clamp(36px,5vw,72px);font-weight:800;letter-spacing:-0.04em;color:#000;margin-bottom:32px;line-height:1;font-style:italic">{title}</h2>
    <div class="anim anim-d1" style="display:flex;flex-wrap:wrap;align-items:center;justify-content:center;gap:16px">
      <a href="{cta_link}" class="reveal btn-hover" style="display:inline-flex;align-items:center;justify-content:center;padding:16px 48px;border-radius:999px;background:#000;color:#fff;font-weight:700;font-size:18px;text-decoration:none;transition:all 0.2s" onmouseover="this.style.transform='scale(1.05)'" onmouseout="this.style.transform='scale(1)'">{cta_text}</a>
      {cta2_html}
    </div>
    {footnote_html}
  </div>
</section>"##,
            title=title, cta_link=cta_link, cta_text=cta_text,
            cta2_html=cta2_html, footnote_html=footnote_html,
        );
    }

    // Dark theme CTA (original)
    format!(
        r##"<section style="padding:96px 24px;position:relative;overflow:hidden">
  <div style="position:absolute;inset:0;pointer-events:none">
    <div style="position:absolute;bottom:0;left:50%;transform:translateX(-50%);width:600px;height:400px;opacity:0.08;background:radial-gradient(ellipse,rgba(0,111,240,0.5),transparent 70%)"></div>
  </div>
  <div style="position:relative;max-width:720px;margin:0 auto;text-align:center">
    <h2 style="font-size:clamp(32px,4vw,48px);font-weight:700;letter-spacing:-0.03em;color:white;margin-bottom:16px">{title}</h2>
    <p style="font-size:18px;color:#9ca3af;margin-bottom:32px;max-width:540px;margin-left:auto;margin-right:auto;line-height:1.6">{subtitle}</p>
    <a href="{cta_link}" style="display:inline-flex;align-items:center;justify-content:center;padding:12px 32px;border-radius:999px;background:white;color:black;font-weight:600;font-size:16px;text-decoration:none;transition:transform 0.2s" onmouseover="this.style.transform='scale(1.02)'" onmouseout="this.style.transform='scale(1)'">{cta_text}</a>
  </div>
</section>"##,
        title = title, subtitle = subtitle, cta_text = cta_text, cta_link = cta_link,
    )
}

fn render_trusted(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("Trusted by the best");
    let subtitle = section.subtitle.as_deref().unwrap_or("");

    let logos: Vec<String> = section.items.iter().map(|item| {
        let name = item.get("title").or_else(|| item.get("name")).map(|s| s.as_str()).unwrap_or("Company");
        format!(
            r#"<div style="font-size:24px;font-weight:700;letter-spacing:-0.03em;color:white">{}</div>"#,
            name
        )
    }).collect();

    format!(
        r##"<section style="padding:96px 0;border-top:1px solid rgba(255,255,255,0.05);border-bottom:1px solid rgba(255,255,255,0.05)">
  <div style="max-width:1280px;margin:0 auto;padding:0 24px">
    <div style="display:flex;flex-wrap:wrap;align-items:center;justify-content:space-between;gap:48px">
      <div style="max-width:420px">
        <h2 style="font-size:30px;font-weight:700;letter-spacing:-0.03em;color:white;margin-bottom:16px">{title}</h2>
        <p style="color:#9ca3af;font-size:15px;line-height:1.6">{subtitle}</p>
      </div>
      <div style="display:grid;grid-template-columns:repeat(3,1fr);gap:32px 48px;align-items:center;opacity:0.5;filter:grayscale(100%);transition:all 0.3s" onmouseover="this.style.filter='none';this.style.opacity='0.8'" onmouseout="this.style.filter='grayscale(100%)';this.style.opacity='0.5'">
        {logos}
      </div>
    </div>
  </div>
</section>"##,
        title = title, subtitle = subtitle, logos = logos.join("\n        "),
    )
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

fn render_footer(section: &SectionNode, theme: &str) -> String {
    let copyright = section.config.get("copyright").map(|s| s.as_str()).unwrap_or("&copy; 2024 Vercel Inc.");
    let has_copyright = section.config.contains_key("copyright");
    let is_dark = section.config.get("style").map(|s| s.contains("dark")).unwrap_or(false) || theme == "dark";

    // Light minimal footer: only when copyright is set AND we are NOT in dark theme
    if has_copyright && !is_dark {
        // Light minimal footer
        let mut all_links: Vec<String> = Vec::new();

        // First try nav config (comma-separated)
        if let Some(nav) = section.config.get("nav") {
            for link in nav.split(',') {
                let l = link.trim();
                if !l.is_empty() {
                    all_links.push(l.to_string());
                }
            }
        }

        // Then try items (each item is a link)
        if all_links.is_empty() {
            for item in &section.items {
                let title = item.get("title").map(|s| s.as_str()).unwrap_or("");
                if !title.is_empty() {
                    all_links.push(title.to_string());
                }
                // Also check description for comma-separated links
                let desc = item.get("description").or_else(|| item.get("desc")).map(|s| s.as_str()).unwrap_or("");
                for link in desc.split(',') {
                    let l = link.trim();
                    if !l.is_empty() {
                        all_links.push(l.to_string());
                    }
                }
            }
        }

        // Split: most links on left with copyright, last 2 on right
        let split_at = if all_links.len() > 2 { all_links.len() - 2 } else { all_links.len() };
        let left_links: Vec<String> = all_links[..split_at].iter().map(|l| format!(
            r##"<a href="#" style="color:#6b7280;font-size:12px;text-decoration:none;transition:color 0.15s" onmouseover="this.style.color='#000'" onmouseout="this.style.color='#6b7280'">{}</a>"##, l
        )).collect();
        let right_links_html: Vec<String> = all_links[split_at..].iter().map(|l| format!(
            r##"<a href="#" style="color:#6b7280;font-size:12px;text-decoration:none;transition:color 0.15s" onmouseover="this.style.color='#000'" onmouseout="this.style.color='#6b7280'">{}</a>"##, l
        )).collect();

        return format!(
            r##"<footer class="anim-fade" style="border-top:1px solid #e5e7eb;background:#fafafa;padding:48px 24px">
  <div style="max-width:1280px;margin:0 auto;display:flex;flex-wrap:wrap;justify-content:space-between;align-items:center;gap:16px">
    <div style="display:flex;align-items:center;gap:16px">
      <span style="font-size:12px;color:#6b7280">{copyright}</span>
      {left_links}
    </div>
    <div style="display:flex;align-items:center;gap:24px">
      {right_links}
    </div>
  </div>
</footer>"##,
            copyright=copyright,
            left_links=left_links.join("\n      "),
            right_links=right_links_html.join("\n      "),
        );
    }

    // Dark theme footer
    // Collect all link names from items or nav config
    let mut all_links: Vec<String> = Vec::new();
    if let Some(nav) = section.config.get("nav") {
        for link in nav.split(',') {
            let l = link.trim();
            if !l.is_empty() {
                all_links.push(l.to_string());
            }
        }
    }
    if all_links.is_empty() {
        for item in &section.items {
            if let Some(title) = item.get("title").or_else(|| item.get("name")) {
                if !title.is_empty() {
                    all_links.push(title.clone());
                }
            }
        }
    }

    // If items have descriptions (column-style data), render as columns
    let has_columns = section.items.iter().any(|i| {
        i.get("description").or_else(|| i.get("desc")).map(|s| s.contains(',')).unwrap_or(false)
    });

    let columns_html = if has_columns {
        let columns: Vec<String> = section.items.iter().map(|item| {
            let title = item.get("title").or_else(|| item.get("name")).map(|s| s.as_str()).unwrap_or("Links");
            let desc = item.get("description").or_else(|| item.get("desc")).map(|s| s.as_str()).unwrap_or("");
            let links: Vec<String> = desc.split(',').map(|link| {
                let l = link.trim();
                format!(r##"<a href="#" style="color:#6b7280;font-size:12px;text-decoration:none;transition:color 0.2s" onmouseover="this.style.color='white'" onmouseout="this.style.color='#6b7280'">{}</a>"##, l)
            }).collect();
            format!(
                r#"<div style="display:flex;flex-direction:column;gap:16px">
    <h4 style="color:white;font-size:14px;font-weight:600">{}</h4>
    {}
  </div>"#,
                title, links.join("\n    ")
            )
        }).collect();
        format!(r#"<div style="display:grid;grid-template-columns:repeat(3,1fr);gap:48px">{}</div>"#, columns.join("\n      "))
    } else if !all_links.is_empty() {
        // Flat link list — render as a simple grid
        let link_html: Vec<String> = all_links.iter().map(|l| {
            format!(r##"<a href="#" style="color:#6b7280;font-size:12px;text-decoration:none;transition:color 0.2s" onmouseover="this.style.color='white'" onmouseout="this.style.color='#6b7280'">{}</a>"##, l)
        }).collect();
        format!(r#"<div style="display:flex;flex-wrap:wrap;gap:16px 32px">{}</div>"#, link_html.join("\n        "))
    } else {
        String::new()
    };

    // Bottom row: last 3 links as right-side links (Status, Twitter, GitHub pattern)
    let bottom_right: Vec<String> = if all_links.len() > 3 {
        all_links[all_links.len()-3..].iter().map(|l| {
            format!(r##"<a href="#" style="color:#6b7280;font-size:12px;text-decoration:none;transition:color 0.2s" onmouseover="this.style.color='white'" onmouseout="this.style.color='#6b7280'">{}</a>"##, l)
        }).collect()
    } else {
        Vec::new()
    };

    format!(
        r##"<footer class="anim-fade" style="border-top:1px solid rgba(255,255,255,0.1);background:black;padding:48px 24px">
  <div style="max-width:1280px;margin:0 auto">
    <div style="display:flex;flex-wrap:wrap;justify-content:space-between;align-items:flex-start;gap:48px;margin-bottom:48px">
      {columns_html}
    </div>
    <div style="display:flex;flex-wrap:wrap;justify-content:space-between;align-items:center;gap:16px;padding-top:48px;border-top:1px solid rgba(255,255,255,0.05)">
      <span style="font-size:12px;color:#6b7280">{copyright}</span>
      <div style="display:flex;gap:24px">
        {bottom_right}
      </div>
    </div>
  </div>
</footer>"##,
        columns_html = columns_html,
        copyright = copyright,
        bottom_right = bottom_right.join("\n        "),
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
        let value = item.get("description").map(|s| s.as_str()).unwrap_or("");
        format!(
            r#"<div class="text-center reveal">
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

fn render_page_header_section(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("Page Title");
    let subtitle = section.subtitle.as_deref().unwrap_or("");

    let action_text = section.config.get("action_text")
        .map(|s| s.as_str())
        .or_else(|| {
            section.items.iter()
                .find(|i| i.get("action").is_some())
                .and_then(|i| i.get("action").or(i.get("title")))
                .map(|s| s.as_str())
        })
        .unwrap_or("Get Started");

    let action_icon = section.items.iter()
        .find(|i| i.get("action").is_some())
        .and_then(|i| i.get("icon"))
        .map(|s| s.as_str())
        .unwrap_or("arrow");

    let icon_svg = match action_icon {
        "plus" => r#"<svg width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path d="M12 5v14M5 12h14"/></svg>"#,
        "download" => r#"<svg width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3"/></svg>"#,
        _ => r#"<svg width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24"><path d="M5 12h14M12 5l7 7-7 7"/></svg>"#,
    };

    let subtitle_html = if subtitle.is_empty() {
        String::new()
    } else {
        format!(r#"<p style="font-size:16px;color:#71717a;margin:0">{subtitle}</p>"#, subtitle = subtitle)
    };

    format!(
        r##"<div style="display:flex;align-items:center;justify-content:space-between;padding:0 0 48px;font-family:'Inter',system-ui,-apple-system,sans-serif">
  <div style="display:flex;flex-direction:column;gap:8px">
    <h1 class="anim-slide-up d1" style="font-size:32px;font-weight:700;letter-spacing:-0.02em;color:#000;margin:0">{title}</h1>
    <div class="anim-slide-up d2">{subtitle_html}</div>
  </div>
  <button class="anim-scale d3 btn-hover" style="display:flex;align-items:center;gap:8px;background:#000;color:#fff;border:none;padding:10px 20px;border-radius:999px;font-size:14px;font-weight:600;cursor:pointer;transition:opacity 0.2s;font-family:inherit" onmouseover="this.style.opacity='0.85'" onmouseout="this.style.opacity='1'">{icon_svg} {action_text}</button>
</div>"##,
        title = title, subtitle_html = subtitle_html, icon_svg = icon_svg, action_text = action_text,
    )
}

fn render_stat_cards(section: &SectionNode) -> String {
    let cols = section.config.get("cols")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(3);

    let cards: Vec<String> = section.items.iter().enumerate().map(|(idx, item)| {
        let label = item.get("title").or_else(|| item.get("name")).map(|s| s.as_str()).unwrap_or("Metric");
        let value = item.get("description").or_else(|| item.get("desc")).map(|s| s.as_str()).unwrap_or("");
        let icon_html = item.get("icon").map(|icon| {
            format!(r#"<span style="font-size:18px;margin-bottom:8px;display:block">{icon}</span>"#, icon = icon)
        }).unwrap_or_default();
        let delay = format!("d{}", (idx % 10) + 1);

        format!(
            r##"<div class="anim-scale {delay} card-hover" style="background:#fff;border:1px solid rgba(198,198,198,0.2);border-radius:12px;padding:24px;box-shadow:0 40px 80px rgba(26,28,28,0.04)">
  {icon_html}
  <p style="font-size:12px;font-weight:600;text-transform:uppercase;letter-spacing:0.1em;color:#5e5e5e;margin:0 0 8px">{label}</p>
  <p style="font-size:28px;font-weight:700;color:#1a1c1c;margin:0">{value}</p>
</div>"##,
            delay = delay, icon_html = icon_html, label = label, value = value,
        )
    }).collect();

    format!(
        r##"<div style="display:grid;grid-template-columns:repeat({cols},1fr);gap:16px;font-family:'Inter',system-ui,-apple-system,sans-serif">
  {items}
</div>"##,
        cols = cols, items = cards.join("\n  "),
    )
}

fn render_info_bar(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("");
    let subtitle = section.subtitle.as_deref().unwrap_or("");

    let config_icon = section.config.get("icon").map(|s| s.as_str());

    let shield_svg = r#"<svg width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>"#;

    let icon_html = if let Some(icon) = config_icon {
        format!(r#"<span style="display:flex;align-items:center;justify-content:center;width:24px;height:24px">{icon}</span>"#, icon = icon)
    } else if section.items.iter().any(|i| i.get("type").map(|t| t == "icon").unwrap_or(false)) {
        let icon_item = section.items.iter().find(|i| i.get("type").map(|t| t == "icon").unwrap_or(false)).unwrap();
        let icon_val = icon_item.get("title").or_else(|| icon_item.get("name")).map(|s| s.as_str()).unwrap_or("");
        format!(r#"<span style="display:flex;align-items:center;justify-content:center;width:24px;height:24px">{icon_val}</span>"#, icon_val = icon_val)
    } else {
        format!(r#"<span style="display:flex;align-items:center;justify-content:center;width:24px;height:24px">{shield}</span>"#, shield = shield_svg)
    };

    let links: Vec<String> = section.items.iter()
        .filter(|i| i.get("type").map(|t| t != "icon").unwrap_or(true))
        .map(|item| {
            let name = item.get("title").or_else(|| item.get("name")).map(|s| s.as_str()).unwrap_or("Link");
            let href = item.get("description").or_else(|| item.get("desc")).or_else(|| item.get("href")).map(|s| s.as_str()).unwrap_or("");
            format!(
                r##"<a href="{href}" style="font-size:12px;color:#5e5e5e;text-transform:uppercase;letter-spacing:0.1em;text-decoration:none;font-weight:500;transition:color 0.15s" onmouseover="this.style.color='#000'" onmouseout="this.style.color='#5e5e5e'">{name}</a>"##,
                href = href, name = name,
            )
        })
        .collect();

    let title_html = if title.is_empty() {
        String::new()
    } else {
        format!(r#"<span style="font-size:14px;font-weight:700;color:#000">{title}</span>"#, title = title)
    };

    let subtitle_html = if subtitle.is_empty() {
        String::new()
    } else {
        format!(r#"<span style="font-size:12px;color:#5e5e5e">{subtitle}</span>"#, subtitle = subtitle)
    };

    format!(
        r##"<div style="display:flex;align-items:center;justify-content:space-between;padding:24px;background:#fafafa;border-top:1px solid rgba(198,198,198,0.2);font-family:'Inter',system-ui,-apple-system,sans-serif">
  <div style="display:flex;align-items:center;gap:12px">
    {icon_html}
    <div style="display:flex;flex-direction:column;gap:2px">
      {title_html}
      {subtitle_html}
    </div>
  </div>
  <div style="display:flex;align-items:center;gap:24px">
    {links}
  </div>
</div>"##,
        icon_html = icon_html, title_html = title_html, subtitle_html = subtitle_html, links = links.join("\n    "),
    )
}

fn render_product_grid_section(section: &SectionNode) -> String {
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
        r##"<section style="padding:32px 0">{title_html}<div style="display:grid;grid-template-columns:repeat({cols},1fr);gap:24px">{cards}</div></section>"##,
        title_html=title_html, cols=cols, cards=cards.join(""),
    )
}

fn render_promo(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("");
    let subtitle = section.subtitle.as_deref().unwrap_or("");

    // Badge: from config or first item with "badge" key
    let badge_text = section.config.get("badge")
        .or_else(|| section.items.iter().find_map(|i| i.get("badge")))
        .map(|s| s.as_str());

    let cta_text = section.config.get("cta_text").map(|s| s.as_str());
    let cta_link = section.config.get("cta_link").map(|s| s.as_str()).unwrap_or("");

    // If config has span:2, this section is intended to span 2 grid columns in parent layout

    let badge_html = badge_text.map(|b| {
        format!(r#"<span style="display:inline-block;font-size:10px;font-weight:600;text-transform:uppercase;letter-spacing:0.1em;padding:4px 12px;border-radius:999px;background:rgba(255,255,255,0.2);color:white">{b}</span>"#, b=b)
    }).unwrap_or_default();

    let subtitle_html = if subtitle.is_empty() {
        String::new()
    } else {
        format!(r#"<p style="font-size:14px;color:#a1a1aa;max-width:480px;line-height:1.6">{subtitle}</p>"#, subtitle=subtitle)
    };

    let cta_html = cta_text.map(|ct| {
        format!(r##"<a href="{link}" style="display:inline-flex;align-items:center;justify-content:center;padding:10px 24px;border-radius:999px;background:white;color:black;font-weight:700;font-size:14px;text-decoration:none;transition:opacity 0.2s" onmouseover="this.style.opacity='0.9'" onmouseout="this.style.opacity='1'">{ct}</a>"##, link=cta_link, ct=ct)
    }).unwrap_or_default();

    format!(
        r##"<div style="background:black;color:white;border-radius:12px;padding:32px;position:relative;overflow:hidden;display:flex;flex-direction:column;gap:16px"><div style="position:absolute;right:0;top:0;bottom:0;width:50%;background:radial-gradient(ellipse at 80% 50%,rgba(0,111,240,0.15),transparent 70%);pointer-events:none"></div><div style="position:relative;display:flex;flex-direction:column;gap:16px">{badge_html}<h2 style="font-size:30px;font-weight:700;letter-spacing:-0.02em;color:white;margin:0">{title}</h2>{subtitle_html}{cta_html}</div></div>"##,
        badge_html=badge_html, title=title, subtitle_html=subtitle_html, cta_html=cta_html,
    )
}

// ══════════════════════════════════════════════════
// LIGHT-THEME SECTION RENDERERS (Geist-inspired)
// ══════════════════════════════════════════════════

fn render_bento(section: &SectionNode, accent: &str) -> String {
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

fn render_features_split(section: &SectionNode, accent: &str) -> String {
    let title_html = section.title.as_deref().map(|t| format!(
        r#"<h2 style="font-size:24px;font-weight:700;letter-spacing:-0.02em;color:#1a1c1c;margin-bottom:32px">{}</h2>"#, t
    )).unwrap_or_default();

    let mut feature_items: Vec<String> = Vec::new();
    let mut code_block = String::new();

    for item in &section.items {
        let name = item.get("title").or_else(|| item.get("name")).map(|s| s.as_str()).unwrap_or("Feature");
        let desc = item.get("description").or_else(|| item.get("desc")).map(|s| s.as_str()).unwrap_or("");

        if item.get("code").is_some() || desc.starts_with('{') {
            let code_content = item.get("code").map(|s| s.as_str()).unwrap_or(desc);
            code_block = format!(
                r#"<div style="background:#1a1a1a;border-radius:12px;padding:24px;overflow:hidden">
  <pre style="margin:0;color:white;font-size:13px;font-family:'SF Mono',SFMono-Regular,Consolas,monospace;line-height:1.6;white-space:pre-wrap;overflow-x:auto">{}</pre>
</div>"#,
                code_content
            );
        } else {
            let icon_letter = name.chars().next().unwrap_or('F');
            feature_items.push(format!(
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
            ));
        }
    }

    let right_col = if code_block.is_empty() {
        r#"<div style="background:#f5f5f5;border-radius:12px;padding:24px;min-height:200px;display:flex;align-items:center;justify-content:center">
  <span style="font-size:14px;color:#5e5e5e">Preview</span>
</div>"#.to_string()
    } else {
        code_block
    };

    format!(
        r#"<section style="max-width:1280px;margin:0 auto;padding:48px 24px">
  {title_html}
  <div style="display:flex;gap:48px;align-items:flex-start">
    <div style="flex:0 0 60%;display:flex;flex-direction:column;gap:24px">
      {features}
    </div>
    <div style="flex:0 0 40%">
      {right_col}
    </div>
  </div>
</section>"#,
        title_html = title_html,
        features = feature_items.join("\n      "),
        right_col = right_col,
    )
}

fn render_team_list(section: &SectionNode) -> String {
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

fn render_policies(section: &SectionNode) -> String {
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

fn render_activity_table(section: &SectionNode) -> String {
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

fn render_status_card(section: &SectionNode) -> String {
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

fn render_edge(section: &SectionNode, accent: &str) -> String {
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

fn render_sidebar(section: &SectionNode) -> String {
    let brand = section.config.get("brand")
        .map(|s| s.as_str())
        .or(section.title.as_deref())
        .unwrap_or("");
    let subtitle = section.config.get("subtitle")
        .map(|s| s.as_str())
        .or(section.subtitle.as_deref())
        .unwrap_or("");

    // Active page from config (matches against item title)
    let active = section.config.get("active").map(|s| s.as_str()).unwrap_or("");

    // Build nav links from items
    let mut nav_items = String::new();
    let mut bottom_items = String::new();
    for item in &section.items {
        let title = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("");
        let href = item.get("href").map(|s| s.as_str()).unwrap_or("");
        let position = item.get("position").map(|s| s.as_str()).unwrap_or("top");
        let is_active = !active.is_empty() && title.eq_ignore_ascii_case(active);

        let link_html = if is_active {
            format!(
                r#"<a class="flex items-center gap-3 px-3 py-2 bg-zinc-100 text-black rounded-md" href="{href}" style="display:flex;align-items:center;gap:12px;padding:8px 12px;background:rgba(0,0,0,0.04);color:#000;border-radius:8px;font-size:14px;font-weight:500;text-decoration:none">
  <span class="material-symbols-outlined" style="font-size:20px">{icon}</span>
  <span>{title}</span>
</a>"#,
                href = href, icon = icon, title = title
            )
        } else {
            format!(
                r#"<a href="{href}" style="display:flex;align-items:center;gap:12px;padding:8px 12px;color:#71717a;border-radius:8px;font-size:14px;font-weight:500;text-decoration:none;transition:all 0.15s" onmouseover="this.style.color='#000';this.style.background='rgba(0,0,0,0.04)'" onmouseout="this.style.color='#71717a';this.style.background='transparent'">
  <span class="material-symbols-outlined" style="font-size:20px">{icon}</span>
  <span>{title}</span>
</a>"#,
                href = href, icon = icon, title = title
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

    format!(
        r##"<aside style="position:fixed;left:0;top:0;height:100%;width:256px;background:rgba(250,250,250,0.5);border-right:1px solid rgba(228,228,231,1);display:flex;flex-direction:column;z-index:50;padding:16px;gap:4px;font-family:'Inter',sans-serif;-webkit-font-smoothing:antialiased">
  <div style="margin-bottom:32px;padding:0 8px">
    <h1 style="font-weight:700;letter-spacing:-0.03em;color:#000;font-size:20px;margin:0">{brand}</h1>
    <p style="font-size:11px;color:#71717a;font-weight:500;letter-spacing:0.08em;text-transform:uppercase;margin:2px 0 0">{subtitle}</p>
  </div>
  <nav style="flex:1;display:flex;flex-direction:column;gap:2px">
    {nav_items}
  </nav>
  <div style="margin-top:auto;border-top:1px solid rgba(228,228,231,1);padding-top:16px;display:flex;flex-direction:column;gap:2px">
    {bottom_items}
  </div>
</aside>"##,
        brand = brand,
        subtitle = subtitle,
        nav_items = nav_items,
        bottom_items = bottom_items,
    )
}

fn render_card_section(section: &SectionNode) -> String {
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
                items_html.push_str(&format!(
                    r#"<div style="margin-top:8px">
  <a href="{href}" style="display:inline-flex;align-items:center;gap:6px;padding:8px 20px;font-size:14px;font-weight:600;color:#fff;background:#1a1c1c;border-radius:8px;text-decoration:none;transition:opacity 0.15s" onmouseover="this.style.opacity='0.85'" onmouseout="this.style.opacity='1'">{title}</a>
</div>"#,
                    href = href, title = item_title,
                ));
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

fn render_links_section(section: &SectionNode) -> String {
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

fn render_form_section(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("Form");
    let subtitle = section.subtitle.as_deref().unwrap_or("");
    let entity = section.config.get("entity").map(|s| s.as_str()).unwrap_or("");
    let action = section.config.get("action").map(|s| s.to_string())
        .unwrap_or_else(|| if !entity.is_empty() { format!("/api/{}s", entity.to_lowercase()) } else { "#".to_string() });
    let method = section.config.get("method").map(|s| s.as_str()).unwrap_or("POST");

    let mut fields_html = String::new();
    let mut actions_html = String::new();
    let mut links_html = String::new();

    for item in &section.items {
        let itype = item.get("_type").map(|s| s.as_str()).unwrap_or("");
        let item_title = item.get("title").map(|s| s.as_str()).unwrap_or("");

        if itype == "field" {
            let ftype = item.get("type").map(|s| s.as_str()).unwrap_or("text");
            let name_lower = item_title.to_lowercase().replace(' ', "_");
            let placeholder = item.get("placeholder").map(|s| s.as_str()).unwrap_or("");
            let required = if item.get("required").map(|s| s == "true").unwrap_or(false) { "required" } else { "" };
            let disabled = if item.get("disabled").map(|s| s == "true").unwrap_or(false) { "disabled" } else { "" };
            let readonly = if item.get("readonly").map(|s| s == "true").unwrap_or(false) { "readonly" } else { "" };
            let value = item.get("value").map(|s| s.as_str()).unwrap_or("");

            let label_style = "display:block;font-size:12px;font-weight:600;text-transform:uppercase;letter-spacing:0.1em;color:#71717a;margin-bottom:8px";
            let input_style = "width:100%;padding:12px 16px;border:1px solid #e5e7eb;border-radius:8px;font-size:14px;outline:none;font-family:Inter,sans-serif;transition:border-color 0.2s;box-sizing:border-box";

            match ftype {
                "text" | "email" | "password" | "url" | "tel" | "number" => {
                    fields_html.push_str(&format!(
                        r#"<div><label style="{label_style}">{label}</label><input type="{ftype}" name="{name}" placeholder="{placeholder}" value="{value}" {required} {disabled} {readonly} style="{input_style}" onfocus="this.style.borderColor='#000'" onblur="this.style.borderColor='#e5e7eb'"></div>"#,
                        label_style = label_style, label = item_title, ftype = ftype, name = name_lower,
                        placeholder = placeholder, value = value, required = required,
                        disabled = disabled, readonly = readonly, input_style = input_style,
                    ));
                }
                "select" => {
                    let options_raw = item.get("options").map(|s| s.as_str()).unwrap_or("");
                    let options: Vec<&str> = if options_raw.is_empty() { vec![] } else { options_raw.split("||").collect() };
                    let mut opts_html = format!(r#"<option value="">Select {}...</option>"#, item_title);
                    for opt in &options {
                        opts_html.push_str(&format!(r#"<option value="{v}">{v}</option>"#, v = opt));
                    }
                    fields_html.push_str(&format!(
                        r#"<div><label style="{label_style}">{label}</label><select name="{name}" {required} {disabled} style="{input_style};appearance:none;background:#fff url('data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 width=%2212%22 height=%2212%22 viewBox=%220 0 12 12%22><path d=%22M2 4l4 4 4-4%22 fill=%22none%22 stroke=%22%2371717a%22 stroke-width=%221.5%22/></svg>') no-repeat right 12px center">{options}</select></div>"#,
                        label_style = label_style, label = item_title, name = name_lower,
                        required = required, disabled = disabled, input_style = input_style, options = opts_html,
                    ));
                }
                "textarea" => {
                    let rows = item.get("rows").map(|s| s.as_str()).unwrap_or("4");
                    fields_html.push_str(&format!(
                        r#"<div><label style="{label_style}">{label}</label><textarea name="{name}" rows="{rows}" placeholder="{placeholder}" {required} {disabled} {readonly} style="{input_style};resize:vertical" onfocus="this.style.borderColor='#000'" onblur="this.style.borderColor='#e5e7eb'"></textarea></div>"#,
                        label_style = label_style, label = item_title, name = name_lower,
                        rows = rows, placeholder = placeholder, required = required,
                        disabled = disabled, readonly = readonly, input_style = input_style,
                    ));
                }
                "checkbox" => {
                    fields_html.push_str(&format!(
                        r#"<label style="display:flex;align-items:center;gap:12px;cursor:pointer"><input type="checkbox" name="{name}" {disabled} style="width:18px;height:18px;accent-color:#000"><span style="font-size:14px">{label}</span></label>"#,
                        name = name_lower, disabled = disabled, label = item_title,
                    ));
                }
                "radio" => {
                    let options_raw = item.get("options").map(|s| s.as_str()).unwrap_or("");
                    let options: Vec<&str> = if options_raw.is_empty() { vec![] } else { options_raw.split("||").collect() };
                    let mut radio_html = String::new();
                    for opt in &options {
                        radio_html.push_str(&format!(
                            r#"<label style="display:flex;align-items:center;gap:8px;cursor:pointer"><input type="radio" name="{name}" value="{val}" {disabled} style="accent-color:#000"><span style="font-size:14px">{val}</span></label>"#,
                            name = name_lower, val = opt, disabled = disabled,
                        ));
                    }
                    fields_html.push_str(&format!(
                        r#"<div><label style="{label_style}">{label}</label><div style="display:flex;flex-direction:column;gap:8px">{radios}</div></div>"#,
                        label_style = label_style, label = item_title, radios = radio_html,
                    ));
                }
                "file" => {
                    let accept = item.get("accept").map(|s| s.as_str()).unwrap_or("");
                    fields_html.push_str(&format!(
                        r#"<div><label style="{label_style}">{label}</label><input type="file" name="{name}" accept="{accept}" {required} {disabled} style="width:100%;padding:10px;border:1px dashed #e5e7eb;border-radius:8px;font-size:14px;cursor:pointer;box-sizing:border-box"></div>"#,
                        label_style = label_style, label = item_title, name = name_lower,
                        accept = accept, required = required, disabled = disabled,
                    ));
                }
                _ => {
                    // Fallback: treat as text
                    fields_html.push_str(&format!(
                        r#"<div><label style="{label_style}">{label}</label><input type="text" name="{name}" placeholder="{placeholder}" value="{value}" {required} {disabled} {readonly} style="{input_style}" onfocus="this.style.borderColor='#000'" onblur="this.style.borderColor='#e5e7eb'"></div>"#,
                        label_style = label_style, label = item_title, name = name_lower,
                        placeholder = placeholder, value = value, required = required,
                        disabled = disabled, readonly = readonly, input_style = input_style,
                    ));
                }
            }
        } else if itype == "action" {
            let variant = item.get("variant").map(|s| s.as_str())
                .or_else(|| item.get("style").map(|s| s.as_str()))
                .unwrap_or("primary");
            let (bg, color) = match variant {
                "secondary" | "outline" => ("#fff", "#000"),
                _ => ("#000", "#fff"),
            };
            let border = if variant == "outline" || variant == "secondary" { "1px solid #e5e7eb" } else { "none" };
            actions_html.push_str(&format!(
                r#"<button type="submit" data-label="{label}" style="width:100%;padding:14px;border:{border};border-radius:999px;background:{bg};color:{color};font-size:16px;font-weight:700;cursor:pointer;font-family:Inter,sans-serif" class="btn-hover">{label}</button>"#,
                label = item_title, bg = bg, color = color, border = border,
            ));
        } else if itype == "link" {
            let link = item.get("link").map(|s| s.as_str()).unwrap_or("#");
            links_html.push_str(&format!(
                r#"<a href="{link}" style="text-align:center;font-size:14px;color:#006ff0;text-decoration:none">{text}</a>"#,
                link = link, text = item_title,
            ));
        }
    }

    // If no explicit action item, add a default submit button
    if actions_html.is_empty() {
        actions_html = r#"<button type="submit" data-label="Save" style="width:100%;padding:14px;border:none;border-radius:999px;background:#000;color:#fff;font-size:16px;font-weight:700;cursor:pointer;font-family:Inter,sans-serif" class="btn-hover">Save</button>"#.to_string();
    }

    let subtitle_html = if subtitle.is_empty() {
        String::new()
    } else {
        format!(r#"<p style="font-size:14px;color:#5e5e5e;margin:0" class="anim-slide-up d2">{}</p>"#, subtitle)
    };

    let data_entity = if !entity.is_empty() { format!(r#" data-entity="{}""#, entity) } else { String::new() };

    format!(
        r##"<section style="max-width:480px;margin:0 auto;padding:48px 24px">
  <div style="margin-bottom:32px">
    <h2 style="font-size:24px;font-weight:700;letter-spacing:-0.02em;margin:0 0 8px" class="anim-slide-up d1">{title}</h2>
    {subtitle_html}
  </div>
  <form id="cronus-form" action="{action}" method="{method}"{data_entity} style="display:flex;flex-direction:column;gap:20px" class="anim-slide-up d3">
    {fields}
    <div id="form-msg" style="display:none;padding:12px 16px;border-radius:8px;font-size:14px;font-weight:500"></div>
    {actions}
    {links}
  </form>
</section>"##,
        title = title, subtitle_html = subtitle_html, action = action, method = method,
        data_entity = data_entity, fields = fields_html, actions = actions_html, links = links_html,
    )
}

fn render_tabs_section(section: &SectionNode) -> String {
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

fn render_accordion_section(section: &SectionNode) -> String {
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

fn render_breadcrumb_section(section: &SectionNode) -> String {
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

fn render_generic_section(section: &SectionNode, _accent: &str) -> String {
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

fn render_alert_section(section: &SectionNode) -> String {
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

fn render_chart_section(section: &SectionNode) -> String {
    let chart_type = section.config.get("type").map(|s| s.as_str()).unwrap_or("bar");
    let title = section.title.as_deref().unwrap_or("Chart");
    let subtitle = section.subtitle.as_deref().unwrap_or("");

    // Parse items: title = label, description = value (numeric)
    let data: Vec<(String, f64)> = section.items.iter().filter_map(|item| {
        let label = item.get("title")?.clone();
        let val_str = item.get("description")?;
        let val: f64 = val_str.trim().parse().ok()?;
        Some((label, val))
    }).collect();

    if data.is_empty() {
        return String::new();
    }

    match chart_type {
        "line" => render_chart_line(title, subtitle, &data),
        "donut" => render_chart_donut(title, subtitle, &data),
        _ => render_chart_bar(title, subtitle, &data),
    }
}

fn render_chart_bar(title: &str, subtitle: &str, data: &[(String, f64)]) -> String {
    let max_val = data.iter().map(|(_, v)| *v).fold(0.0_f64, f64::max);
    if max_val == 0.0 {
        return String::new();
    }

    let bars: Vec<String> = data.iter().map(|(label, value)| {
        let percent = (value / max_val) * 100.0;
        format!(
            r##"<div style="flex:1;display:flex;flex-direction:column;align-items:center;gap:8px">
        <span style="font-size:11px;font-weight:600;color:#1a1c1c">{value}</span>
        <div style="width:100%;background:#000;border-radius:4px 4px 0 0;transition:height 0.8s cubic-bezier(0.16,1,0.3,1);height:{percent}%" class="chart-bar"></div>
        <span style="font-size:11px;color:#71717a">{label}</span>
      </div>"##,
            value = *value as i64,
            percent = percent as i64,
            label = label,
        )
    }).collect();

    format!(
        r##"<div class="anim-slide-up card-hover" style="background:#fff;border:1px solid rgba(198,198,198,0.2);border-radius:12px;padding:24px">
  <div style="margin-bottom:20px">
    <h3 style="font-size:16px;font-weight:700;margin:0 0 4px">{title}</h3>
    <p style="font-size:13px;color:#71717a;margin:0">{subtitle}</p>
  </div>
  <div style="display:flex;align-items:flex-end;gap:8px;height:200px;padding-top:16px">
    {bars}
  </div>
</div>"##,
        title = title,
        subtitle = subtitle,
        bars = bars.join("\n    "),
    )
}

fn render_chart_line(title: &str, subtitle: &str, data: &[(String, f64)]) -> String {
    let max_val = data.iter().map(|(_, v)| *v).fold(0.0_f64, f64::max);
    if max_val == 0.0 || data.len() < 2 {
        return String::new();
    }

    let width = 600;
    let height = 200;
    let padding = 20;
    let usable_w = width - 2 * padding;
    let usable_h = height - 2 * padding;
    let n = data.len();

    let points: Vec<(i32, i32)> = data.iter().enumerate().map(|(i, (_, v))| {
        let x = padding + (i as i32 * usable_w / (n as i32 - 1));
        let y = padding + (usable_h as f64 * (1.0 - v / max_val)) as i32;
        (x, y)
    }).collect();

    let polyline_pts: String = points.iter().map(|(x, y)| format!("{},{}", x, y)).collect::<Vec<_>>().join(" ");
    let fill_pts = format!("{} {},{} {},{}",
        polyline_pts,
        points.last().unwrap().0, height,
        points.first().unwrap().0, height
    );

    let circles: String = points.iter().map(|(x, y)| {
        format!(r##"<circle cx="{}" cy="{}" r="4" fill="#000"/>"##, x, y)
    }).collect::<Vec<_>>().join("\n    ");

    let labels: String = data.iter().map(|(label, _)| {
        format!(r##"<span style="font-size:11px;color:#71717a">{}</span>"##, label)
    }).collect::<Vec<_>>().join("\n    ");

    format!(
        r##"<div class="anim-slide-up" style="background:#fff;border:1px solid rgba(198,198,198,0.2);border-radius:12px;padding:24px">
  <div style="margin-bottom:20px">
    <h3 style="font-size:16px;font-weight:700;margin:0 0 4px">{title}</h3>
    <p style="font-size:13px;color:#71717a;margin:0">{subtitle}</p>
  </div>
  <svg viewBox="0 0 {width} {height}" style="width:100%;height:200px">
    <polyline points="{polyline_pts}" fill="none" stroke="#000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
    <polyline points="{fill_pts}" fill="rgba(0,0,0,0.05)" stroke="none"/>
    {circles}
  </svg>
  <div style="display:flex;justify-content:space-between;padding-top:8px">
    {labels}
  </div>
</div>"##,
        title = title,
        subtitle = subtitle,
        width = width,
        height = height,
        polyline_pts = polyline_pts,
        fill_pts = fill_pts,
        circles = circles,
        labels = labels,
    )
}

fn render_chart_donut(title: &str, subtitle: &str, data: &[(String, f64)]) -> String {
    let total: f64 = data.iter().map(|(_, v)| *v).sum();
    if total == 0.0 {
        return String::new();
    }

    let circumference: f64 = 2.0 * std::f64::consts::PI * 50.0;
    let colors = ["#000", "#71717a", "#a1a1aa", "#d4d4d8", "#e5e7eb", "#f3f4f6"];

    let mut offset = 0.0_f64;
    let segments: Vec<String> = data.iter().enumerate().map(|(i, (_, v))| {
        let arc = (v / total) * circumference;
        let color = colors[i % colors.len()];
        let seg = format!(
            r##"<circle cx="60" cy="60" r="50" fill="none" stroke="{color}" stroke-width="10" stroke-dasharray="{arc} {circumference}" stroke-dashoffset="{offset}"/>"##,
            color = color,
            arc = arc,
            circumference = circumference,
            offset = -offset,
        );
        offset += arc;
        seg
    }).collect();

    let legend: String = data.iter().enumerate().map(|(i, (label, value))| {
        let color = colors[i % colors.len()];
        format!(
            r##"<div style="display:flex;align-items:center;gap:8px">
      <div style="width:12px;height:12px;border-radius:3px;background:{color};flex-shrink:0"></div>
      <span style="font-size:13px;color:#1a1c1c">{label}</span>
      <span style="font-size:13px;color:#71717a;margin-left:auto">{value}</span>
    </div>"##,
            color = color,
            label = label,
            value = *value as i64,
        )
    }).collect::<Vec<_>>().join("\n    ");

    format!(
        r##"<div class="anim-slide-up" style="background:#fff;border:1px solid rgba(198,198,198,0.2);border-radius:12px;padding:24px">
  <div style="margin-bottom:20px">
    <h3 style="font-size:16px;font-weight:700;margin:0 0 4px">{title}</h3>
    <p style="font-size:13px;color:#71717a;margin:0">{subtitle}</p>
  </div>
  <div style="display:flex;align-items:center;gap:32px">
    <svg viewBox="0 0 120 120" style="width:120px;height:120px;transform:rotate(-90deg)">
      <circle cx="60" cy="60" r="50" fill="none" stroke="#e5e7eb" stroke-width="10"/>
      {segments}
    </svg>
    <div style="display:flex;flex-direction:column;gap:12px">
      {legend}
    </div>
  </div>
</div>"##,
        title = title,
        subtitle = subtitle,
        segments = segments.join("\n      "),
        legend = legend,
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
        let value = item_by_kind(&comp.items, "value").unwrap_or("");
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
        let href = c.link.as_deref().unwrap_or("");
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
        let href = a.link.as_deref().unwrap_or("");
        (a.text.as_str(), href)
    }).collect();
    components::dropdown(trigger, &menu_items)
}

// ══════════════════════════════════════════════════
// DEDICATED DASHBOARD PAGE RENDERER — API Webhooks
// Produces a complete HTML page matching the Geist/Inter
// design system from the original stitch output.
// ══════════════════════════════════════════════════

pub fn render_dashboard_page(
    app_name: &str,
    sections: &[SectionNode],
    components: &[ComponentNode],
    theme: &str,
    current_route: &str,
) -> String {
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

<script>{runtime}</script>
<script>{hmr}</script>
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
        anim_js = CRONUS_ANIMATIONS_JS,
        runtime = super::render::CRONUS_RUNTIME_JS,
        hmr = super::hmr::HMR_CLIENT_JS,
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
    let _ = theme;

    let sidebar_comp = components.iter().find(|c| c.layout.as_deref() == Some("sidebar"));
    let topbar_comp = components.iter().find(|c| {
        c.layout.as_deref() == Some("inline") && c.style.as_deref().map(|s| s.contains("topbar")).unwrap_or(false)
    });

    let sidebar_html = build_dashboard_sidebar(sidebar_comp, None, current_route);
    let topbar_html = build_dashboard_topbar(topbar_comp);

    format!(
        r##"<!DOCTYPE html>
<html class="light" lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{app_name}</title>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap" rel="stylesheet">
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet">
  <style>
    body {{ font-family:'Inter',sans-serif; background:#f9f9f9; color:#1a1c1c; margin:0; }}
    * {{ box-sizing:border-box; }}
    .material-symbols-outlined {{ font-variation-settings:'FILL' 0,'wght' 400,'GRAD' 0,'opsz' 24; font-size:20px; display:inline-block; line-height:1; vertical-align:middle; }}
    .prism-bg {{ background: radial-gradient(circle at top right,rgba(0,111,240,0.08),transparent 40%),radial-gradient(circle at bottom left,rgba(0,56,129,0.05),transparent 40%); }}
    .engineering-grid {{ background-image:linear-gradient(to right,rgba(198,198,198,0.1) 1px,transparent 1px),linear-gradient(to bottom,rgba(198,198,198,0.1) 1px,transparent 1px); background-size:40px 40px; }}
    ::selection {{ background:rgba(0,111,240,0.15); }}
    ::-webkit-scrollbar {{ width:4px; }}
    ::-webkit-scrollbar-thumb {{ background:rgba(0,0,0,0.1); border-radius:2px; }}
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

<script>{runtime}</script>
<script>{hmr}</script>
{anim_js}
</body>
</html>"##,
        app_name = app_name,
        sidebar = sidebar_html,
        topbar = topbar_html,
        body = body,
        anim_css = CRONUS_ANIMATIONS_CSS,
        anim_js = CRONUS_ANIMATIONS_JS,
        runtime = super::render::CRONUS_RUNTIME_JS,
        hmr = super::hmr::HMR_CLIENT_JS,
    )
}

// ── Sidebar builder ─────────────────────────────

fn build_dashboard_sidebar(comp: Option<&ComponentNode>, section: Option<&SectionNode>, current_route: &str) -> String {
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

fn build_dashboard_topbar(comp: Option<&ComponentNode>) -> String {
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

fn build_dashboard_page_header(section: Option<&SectionNode>) -> String {
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

fn build_api_keys_card(section: Option<&SectionNode>, mode: &str) -> String {
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

fn build_webhooks_card(section: Option<&SectionNode>) -> String {
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

fn build_promo_panel(section: Option<&SectionNode>) -> String {
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

fn build_quick_links_panel(section: Option<&SectionNode>) -> String {
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

fn build_status_panel(section: Option<&SectionNode>) -> String {
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
    let _ = theme;

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

    format!(
        r##"<!DOCTYPE html>
<html class="light" lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
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
    <!-- 96px spacer (h-24) -->
    <div style="height:96px"></div>
  </div>
</main>

<script>{runtime}</script>
<script>{hmr}</script>
{anim_js}
</body>
</html>"##,
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
        anim_js = CRONUS_ANIMATIONS_JS,
        runtime = super::render::CRONUS_RUNTIME_JS,
        hmr = super::hmr::HMR_CLIENT_JS,
    )
}

// ── Billing page header ────────────────────────

fn build_billing_page_header(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };
    let title = sec.title.as_deref().unwrap_or("");
    let subtitle = sec.subtitle.as_deref().unwrap_or("");

    let subtitle_html = if !subtitle.is_empty() {
        format!(
            r#"<p style="color:#5e5e5e;font-size:18px;margin:8px 0 0;line-height:1.5">{}</p>"#,
            subtitle
        )
    } else {
        String::new()
    };

    format!(
        r#"<div style="margin-bottom:48px">
      <h2 class="anim-slide-up d1" style="font-size:56px;font-weight:800;letter-spacing:-0.05em;line-height:1.25;color:#1a1c1c;margin:0 0 8px">{title}</h2>
      <div class="anim-slide-up d2">{subtitle}</div>
    </div>"#,
        title = title,
        subtitle = subtitle_html,
    )
}

// ── Current Plan card ──────────────────────────

fn build_billing_current_plan(section: Option<&SectionNode>) -> String {
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
                r#"<span style="font-weight:600">{value}</span> <span style="font-size:14px;color:#5e5e5e">{suffix}</span>"#,
                value = value, suffix = suffix
            )
        } else {
            format!(r#"<span style="font-weight:600">{value}</span>"#, value = value)
        };

        rows_html.push_str(&format!(
            r#"<div style="display:flex;justify-content:space-between;align-items:flex-end;border-bottom:1px solid #f3f3f3;padding-bottom:16px;margin-bottom:24px">
              <span style="color:#5e5e5e">{label}</span>
              <span>{value}</span>
            </div>"#,
            label = label,
            value = value_html,
        ));
    }

    format!(
        r##"<section class="ghost-border anim-slide-up d1 card-hover" style="background:#fff;border-radius:12px;padding:32px;position:relative;overflow:hidden;box-shadow:0 1px 3px rgba(0,0,0,0.04)">
          <div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:32px">
            <div>
              <span class="anim-fade d1" style="background:#000;color:#fff;font-size:10px;font-weight:700;letter-spacing:0.1em;text-transform:uppercase;border-radius:4px;padding:2px 8px;display:inline-block;margin-bottom:16px">{badge}</span>
              <h3 style="font-size:30px;font-weight:700;letter-spacing:-0.02em;margin:0">{plan_name}</h3>
            </div>
            <button class="btn-hover" style="background:#000;color:#fff;padding:10px 24px;border-radius:999px;font-size:14px;font-weight:500;border:none;cursor:pointer">{action}</button>
          </div>
          {rows}
          <div style="position:absolute;right:-80px;bottom:-80px;width:240px;height:240px;border-radius:50%;background:#eeeeee;opacity:0.3;filter:blur(48px);pointer-events:none"></div>
        </section>"##,
        badge = badge_text,
        plan_name = plan_name,
        action = action_text,
        rows = rows_html,
    )
}

// ── Usage Status card ──────────────────────────

fn build_billing_usage_status(section: Option<&SectionNode>) -> String {
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

fn build_billing_stats(section: Option<&SectionNode>) -> String {
    let sec = match section {
        Some(s) => s,
        None => return String::new(),
    };

    let mut cards_html = String::new();
    let mut billing_stat_idx = 0u32;
    for item in &sec.items {
        let value = item.get("title").map(|s| s.as_str()).unwrap_or("");
        let label = item.get("meta")
            .or_else(|| item.get("description"))
            .map(|s| s.as_str())
            .unwrap_or("");
        let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("");
        billing_stat_idx += 1;
        let delay = format!("d{}", ((billing_stat_idx - 1) % 10) + 1);

        cards_html.push_str(&format!(
            r##"<div class="ghost-border anim-scale {delay} card-hover" style="background:#fff;border-radius:12px;padding:24px;box-shadow:0 1px 3px rgba(0,0,0,0.04)">
              <span class="material-symbols-outlined" style="color:#a1a1aa;margin-bottom:16px">{icon}</span>
              <div style="font-size:24px;font-weight:700;letter-spacing:-0.02em;margin-bottom:4px">{value}</div>
              <div style="font-size:12px;color:#5e5e5e;font-weight:500;text-transform:uppercase;letter-spacing:-0.05em">{label}</div>
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

fn build_billing_payment_methods(section: Option<&SectionNode>) -> String {
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

fn build_billing_recent_invoices(section: Option<&SectionNode>) -> String {
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

<script>{runtime}</script>
<script>{hmr}</script>
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
        anim_js = CRONUS_ANIMATIONS_JS,
        runtime = super::render::CRONUS_RUNTIME_JS,
        hmr = super::hmr::HMR_CLIENT_JS,
    )
}

// ── Payouts page header ────────────────────────

fn build_payouts_page_header(section: Option<&SectionNode>) -> String {
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

fn build_payouts_balance_card(section: Option<&SectionNode>) -> String {
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

fn build_payouts_upcoming_card(section: Option<&SectionNode>) -> String {
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

fn build_payouts_history(section: Option<&SectionNode>) -> String {
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

fn build_payouts_support_banner(section: Option<&SectionNode>) -> String {
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

fn build_unified_usage_plan(section: Option<&SectionNode>) -> String {
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

fn build_unified_billing_payments(section: Option<&SectionNode>) -> String {
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

<script>{runtime}</script>
<script>{hmr}</script>
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
        anim_js = CRONUS_ANIMATIONS_JS,
        runtime = super::render::CRONUS_RUNTIME_JS,
        hmr = super::hmr::HMR_CLIENT_JS,
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

<script>{runtime}</script>
<script>{hmr}</script>
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
        anim_js = CRONUS_ANIMATIONS_JS,
        runtime = super::render::CRONUS_RUNTIME_JS,
        hmr = super::hmr::HMR_CLIENT_JS,
    )
}

// ── Payment Links: page header ─────────────────

fn build_payment_links_page_header(section: Option<&SectionNode>) -> String {
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

fn build_payment_links_stat_cards(section: Option<&SectionNode>) -> String {
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

fn build_payment_links_product_grid(section: &SectionNode) -> String {
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

fn build_payment_links_promo(section: Option<&SectionNode>) -> String {
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

fn build_payment_links_info_bar(section: Option<&SectionNode>) -> String {
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

<script>{runtime}</script>
<script>{hmr}</script>
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
        anim_js = CRONUS_ANIMATIONS_JS,
        runtime = super::render::CRONUS_RUNTIME_JS,
        hmr = super::hmr::HMR_CLIENT_JS,
    )
}

fn build_checkout_topbar(section: Option<&SectionNode>) -> String {
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

fn build_checkout_form(section: Option<&SectionNode>) -> String {
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

fn build_checkout_product_summary(section: Option<&SectionNode>) -> String {
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

fn build_checkout_trust_indicators(section: Option<&SectionNode>) -> String {
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

fn build_checkout_testimonial(section: Option<&SectionNode>) -> String {
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

fn build_checkout_footer(section: Option<&SectionNode>) -> String {
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

<script>{runtime}</script>
<script>{hmr}</script>
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
        anim_js = CRONUS_ANIMATIONS_JS,
        runtime = super::render::CRONUS_RUNTIME_JS,
        hmr = super::hmr::HMR_CLIENT_JS,
    )
}

fn build_security_topbar(comp: Option<&ComponentNode>) -> String {
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

fn build_security_sidebar(comp: Option<&ComponentNode>, section: Option<&SectionNode>) -> String {
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

fn build_security_page_header(section: Option<&SectionNode>) -> String {
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

fn build_security_team_members(section: Option<&SectionNode>) -> String {
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

fn build_security_2fa_status(section: Option<&SectionNode>) -> String {
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

fn build_security_policies(section: Option<&SectionNode>) -> String {
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

fn build_security_login_activity(section: Option<&SectionNode>) -> String {
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
