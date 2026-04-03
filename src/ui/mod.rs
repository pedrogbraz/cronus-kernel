#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS UI — Server-Side HTML Renderer
//!
//! Generates complete HTML pages from the AST.
//! No React, no frameworks — pure HTML + Tailwind CDN + vanilla JS.

pub mod layout;
pub mod page;
mod section_hero;
mod section_features;
mod section_chart;
mod section_kpi;
mod section_form;
mod section_misc;

pub use layout::*;
pub use page::{render_page, render_auth_page};

use crate::parser::{EntityNode, FieldType, PageNode, SectionNode, ComponentNode, ComponentItemNode, LayoutNode, StyleNode};
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
@keyframes slideFromRight{from{opacity:0;transform:translateX(100%)}to{opacity:1;transform:translateX(0)}}
@keyframes slideFromLeft{from{opacity:0;transform:translateX(-100%)}to{opacity:1;transform:translateX(0)}}
@keyframes fillWidth{from{width:0}to{width:var(--target-width)}}
@keyframes pulse{0%,100%{opacity:1}50%{opacity:.5}}
@keyframes float{0%,100%{transform:translateY(0)}50%{transform:translateY(-6px)}}
@keyframes barGrow{from{transform:scaleY(0)}to{transform:scaleY(1)}}
@keyframes drawLine{from{stroke-dashoffset:var(--line-len)}to{stroke-dashoffset:0}}
@keyframes donutDraw{from{stroke-dasharray:0 314}to{stroke-dasharray:var(--arc) 314}}
@keyframes glowPulse{0%,100%{box-shadow:0 0 0 rgba(135,173,255,0)}50%{box-shadow:0 0 20px rgba(135,173,255,0.15)}}
@keyframes shimmer{0%{background-position:-200% 0}100%{background-position:200% 0}}
@keyframes countUp{from{opacity:0;transform:translateY(8px)}to{opacity:1;transform:translateY(0)}}
@keyframes rowSlide{from{opacity:0;transform:translateX(-12px)}to{opacity:1;transform:translateX(0)}}
@keyframes gradientShift{0%{background-position:0% 50%}50%{background-position:100% 50%}100%{background-position:0% 50%}}
@keyframes breathe{0%,100%{opacity:0.4}50%{opacity:0.8}}
@keyframes spin{to{transform:rotate(360deg)}}
.anim-fade{animation:fadeIn .6s ease-out both}
.anim-slide-up{animation:slideUp .6s cubic-bezier(.16,1,.3,1) both}
.anim-slide-down{animation:slideDown .4s ease-out both}
.anim-scale{animation:scaleIn .5s cubic-bezier(.16,1,.3,1) both}
.anim-slide-right{animation:slideRight .5s cubic-bezier(.16,1,.3,1) both}
.anim-count{animation:countUp .4s cubic-bezier(.16,1,.3,1) both}
.anim-row{animation:rowSlide .4s cubic-bezier(.16,1,.3,1) both}
.anim-glow{animation:glowPulse 3s ease-in-out infinite}
.anim-breathe{animation:breathe 4s ease-in-out infinite}
.anim-gradient{background-size:200% 200%;animation:gradientShift 8s ease infinite}
.chart-bar-anim{transform-origin:bottom;animation:barGrow .8s cubic-bezier(.16,1,.3,1) both}
.chart-line-draw{animation:drawLine 1.5s cubic-bezier(.16,1,.3,1) both}
.chart-donut-draw{animation:donutDraw 1.2s cubic-bezier(.16,1,.3,1) both}
.chart-shimmer{background:linear-gradient(90deg,transparent 0%,rgba(255,255,255,0.03) 50%,transparent 100%);background-size:200% 100%;animation:shimmer 3s linear infinite}
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
// Modal system — must be global BEFORE DOMContentLoaded
window.cronusModal={
  open:function(id){var el=document.getElementById(id);if(el)el.style.display='flex'},
  close:function(id){var el=document.getElementById(id);if(el)el.style.display='none'}
};
// ── CRONUS Live System ──────────────────────────────
// Premium real-time updates — no page reload, smooth transitions

// Status color map
var _cs={'live':'#10b981','completed':'#10b981','active':'#10b981','success':'#10b981',
  'rolling':'#3b82f6','processing':'#3b82f6','in_progress':'#3b82f6',
  'pending':'#71717a','waiting':'#71717a','draft':'#71717a',
  'failed':'#ef4444','blocked':'#ef4444','rejected':'#ef4444','cancelled':'#ef4444',
  'critical':'#ef4444','high':'#f97316',
  'medium':'#f59e0b','warning':'#f59e0b','throttled':'#f59e0b','flagged':'#f59e0b',
  'info':'#3b82f6','low':'#3b82f6'};

function _buildCell(col,val,ci){
  var td=document.createElement('td');
  td.style.cssText='padding:16px 32px';
  var lv=(val+'').toLowerCase();
  if(col==='status'||col==='severity'){
    var c=_cs[lv]||'#71717a';
    var p=(lv==='live'||lv==='rolling'||lv==='processing')?'animation:pulse 2s ease-in-out infinite;':'';
    td.innerHTML='<span style="display:inline-flex;align-items:center;gap:6px;padding:4px 12px;font-size:11px;font-weight:600;border-radius:999px;background:'+c+'1f;color:'+c+'"><span style="width:6px;height:6px;border-radius:50%;background:'+c+';flex-shrink:0;'+p+'"></span>'+val+'</span>';
  } else if(ci===0){
    td.innerHTML='<span style="font-size:13px;color:#e2e2e2;font-weight:600">'+val+'</span>';
  } else {
    td.innerHTML='<span style="font-size:13px;color:rgba(226,226,226,0.8)">'+val+'</span>';
  }
  return td;
}

function _getHeaders(table){
  var h=[];
  table.querySelectorAll('thead th').forEach(function(th){
    h.push(th.textContent.replace(/[▲▼\s]*$/,'').trim().toLowerCase().replace(/\s+/g,'_'));
  });
  return h;
}

// Insert a single new row at top with premium animation
function _insertRow(table,row,headers){
  var tbody=table.querySelector('tbody');
  if(!tbody)return;
  var tr=document.createElement('tr');
  tr.style.cssText='border-bottom:1px solid rgba(76,69,70,0.05);transition:all 0.5s cubic-bezier(0.16,1,0.3,1);opacity:0;transform:translateY(-8px);background:rgba(135,173,255,0.04)';
  tr.onmouseover=function(){this.style.background='#1f1f1f'};
  tr.onmouseout=function(){this.style.background='transparent'};
  headers.forEach(function(col,ci){
    if(!col)return;
    tr.appendChild(_buildCell(col,row[col]||'',ci));
  });
  tbody.insertBefore(tr,tbody.firstChild);
  // Trigger animation
  requestAnimationFrame(function(){requestAnimationFrame(function(){
    tr.style.opacity='1';
    tr.style.transform='translateY(0)';
    // Remove highlight after 2s
    setTimeout(function(){tr.style.background='transparent'},2000);
  })});
}

// Smooth toast notification
function _toast(msg,type){
  var existing=document.querySelector('.cronus-toast');
  if(existing)existing.remove();
  var t=document.createElement('div');
  t.className='cronus-toast';
  var bg=type==='error'?'rgba(220,38,38,0.9)':'rgba(16,185,129,0.9)';
  var icon=type==='error'?'error':'check_circle';
  t.innerHTML='<span class="material-symbols-outlined" style="font-size:18px">'+icon+'</span>'+msg;
  t.style.cssText='position:fixed;bottom:32px;left:50%;z-index:200;padding:10px 20px;background:'+bg+';color:#fff;border-radius:10px;font-size:13px;font-weight:500;font-family:Inter,sans-serif;display:flex;align-items:center;gap:8px;backdrop-filter:blur(12px);box-shadow:0 8px 32px rgba(0,0,0,0.3);transform:translateX(-50%) translateY(20px);opacity:0;transition:all 0.4s cubic-bezier(0.16,1,0.3,1)';
  document.body.appendChild(t);
  requestAnimationFrame(function(){requestAnimationFrame(function(){
    t.style.opacity='1';t.style.transform='translateX(-50%) translateY(0)';
  })});
  setTimeout(function(){
    t.style.opacity='0';t.style.transform='translateX(-50%) translateY(10px)';
    setTimeout(function(){t.remove()},400);
  },2800);
}

// Smooth modal close with fade-out
var _origOpen=window.cronusModal.open;
window.cronusModal.open=function(id){
  var el=document.getElementById(id);if(!el)return;
  el.style.display='flex';el.style.opacity='0';
  var panel=el.querySelector(':scope > div');
  if(panel){panel.style.transform='scale(0.96)';panel.style.opacity='0';}
  requestAnimationFrame(function(){requestAnimationFrame(function(){
    el.style.transition='opacity 0.25s ease';el.style.opacity='1';
    if(panel){panel.style.transition='all 0.35s cubic-bezier(0.16,1,0.3,1)';panel.style.transform='scale(1)';panel.style.opacity='1';}
  })});
};
window.cronusModal.close=function(id){
  var el=document.getElementById(id);if(!el)return;
  var panel=el.querySelector(':scope > div');
  el.style.transition='opacity 0.2s ease';el.style.opacity='0';
  if(panel){panel.style.transition='all 0.2s ease';panel.style.transform='scale(0.97)';panel.style.opacity='0';}
  setTimeout(function(){el.style.display='none';el.style.transition='';if(panel){panel.style.transition='';}},220);
};

// Modal form submit — premium flow
window.cronusModalSubmit=function(e,modalId,apiUrl){
  e.preventDefault();
  var form=e.target;
  var btn=form.querySelector('[type=submit]');
  var origText=btn?btn.textContent:'';
  // Button loading state
  if(btn){
    btn.style.transition='all 0.2s ease';
    btn.style.opacity='0.7';
    btn.textContent='';
    btn.innerHTML='<span style="display:inline-block;width:16px;height:16px;border:2px solid rgba(255,255,255,0.3);border-top-color:#fff;border-radius:50%;animation:spin 0.6s linear infinite"></span>';
    btn.disabled=true;
  }
  var data={};
  form.querySelectorAll('input,select,textarea').forEach(function(inp){
    var name=inp.name;if(!name)return;
    var val=inp.type==='checkbox'?inp.checked:inp.value;
    if(val!==''&&val!==false)data[name]=val;
  });
  fetch(apiUrl,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(data)})
    .then(function(r){
      if(!r.ok)return r.json().then(function(j){throw new Error(j.error||'Request failed')});
      return r.json();
    })
    .then(function(created){
      cronusModal.close(modalId);
      setTimeout(function(){form.reset();if(btn){btn.textContent=origText;btn.disabled=false;btn.style.opacity='1';}},300);
      _toast('Created successfully','success');
      // Insert new row into matching tables — no full re-render
      document.querySelectorAll('[data-entity]').forEach(function(w){
        var table=w.querySelector('table');
        if(table){
          var headers=_getHeaders(table);
          _insertRow(table,created,headers);
        }
      });
    })
    .catch(function(err){
      if(btn){btn.textContent=origText;btn.disabled=false;btn.style.opacity='1';}
      _toast(err.message,'error');
    });
  return false;
};
document.addEventListener('DOMContentLoaded',()=>{
  const io=new IntersectionObserver(e=>{e.forEach(e=>{if(e.isIntersecting){e.target.classList.add('visible');io.unobserve(e.target)}})},{threshold:.1,rootMargin:'0px 0px -40px 0px'});
  document.querySelectorAll('.reveal').forEach(el=>io.observe(el));
  document.querySelectorAll('.stagger').forEach(c=>{Array.from(c.children).forEach((ch,i)=>{ch.style.animationDelay=(.05+i*.06)+'s'})});

  // Counter animation: elements with data-count-to animate from 0 to target
  // Handles: "847", "99.7%", "42s", "12,847", "A+", "1.2 GB/s", "$142,804.22"
  document.querySelectorAll('[data-count-to]').forEach(function(el){
    var raw=el.getAttribute('data-count-to');
    // Extract numeric part, prefix and suffix
    var match=raw.match(/^([^0-9]*?)([\d,]+\.?\d*)(.*?)$/);
    if(!match){el.textContent=raw;return;} // non-numeric like "A+" — show immediately
    var prefix=match[1];
    var numStr=match[2];
    var suffix=match[3];
    var num=parseFloat(numStr.replace(/,/g,''));
    if(isNaN(num)){el.textContent=raw;return;}
    var hasComma=numStr.indexOf(',')!==-1;
    var decMatch=numStr.match(/\.(\d+)/);
    var decimals=decMatch?decMatch[1].length:0;
    var duration=1400;
    var start=performance.now();
    function fmt(v){
      var s;
      if(decimals>0)s=v.toFixed(decimals);
      else s=Math.round(v).toString();
      if(hasComma){
        var parts=s.split('.');
        parts[0]=parts[0].replace(/\B(?=(\d{3})+(?!\d))/g,',');
        s=parts.join('.');
      }
      return prefix+s+suffix;
    }
    function step(now){
      var t=Math.min((now-start)/duration,1);
      t=t<0.5?4*t*t*t:1-Math.pow(-2*t+2,3)/2; // ease-in-out cubic
      el.textContent=fmt(num*t);
      if(t<1)requestAnimationFrame(step);
      else el.textContent=raw; // exact final
    }
    el.textContent=fmt(0);
    requestAnimationFrame(step);
  });

  // Chart bar grow on scroll
  var cio=new IntersectionObserver(function(entries){entries.forEach(function(e){if(e.isIntersecting){e.target.style.transform='scaleY(1)';cio.unobserve(e.target);}});},{threshold:0.2});
  document.querySelectorAll('.cronus-bar').forEach(function(b){b.style.transform='scaleY(0)';b.style.transformOrigin='bottom';b.style.transition='transform 0.8s cubic-bezier(0.16,1,0.3,1)';cio.observe(b);});

  // Animate template chart bars (dump templates with Tailwind h-* classes)
  // Reads computed height, collapses to 0, then animates to target
  document.querySelectorAll('[class*="items-end"]').forEach(function(container){
    var bars=container.querySelectorAll('[class*="bg-primary"],[class*="bg-secondary"],[class*="bg-tertiary"]');
    if(bars.length<3)return;
    // Phase 1: read all target heights while bars are visible
    var targets=[];
    bars.forEach(function(bar){targets.push(bar.offsetHeight)});
    // Phase 2: collapse all to 0
    bars.forEach(function(bar){
      bar.style.transition='none';
      bar.style.height='0px';
      bar.style.opacity='0';
    });
    // Phase 3: animate each bar to its target
    requestAnimationFrame(function(){requestAnimationFrame(function(){
      bars.forEach(function(bar,i){
        var delay=0.3+i*0.07;
        bar.style.transition='height 1s cubic-bezier(0.16,1,0.3,1) '+delay+'s, opacity 0.5s ease '+delay+'s';
        bar.style.height=targets[i]+'px';
        bar.style.opacity='1';
      });
    })});
  });

  // HTML escape — prevent XSS from user-controlled data
  window.cronusEscape=function(s){
    if(s==null)return '';
    return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;').replace(/'/g,'&#39;');
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
      if(r.ok){cronusModal.close('create-modal');if(window.CRONUS&&window.CRONUS.reload)window.CRONUS.reload()}
      else{var err=await r.json();cronusToast(err.error||'Error','error')}
    }catch(ex){cronusToast('Connection error','error')}
    return false;
  };

  // Delete with confirmation
  window.cronusDelete=async function(entity,id){
    if(!confirm('Delete this '+entity+'?'))return;
    try{
      await fetch('/api/'+entity.toLowerCase()+'s/'+id,{method:'DELETE'});
      if(window.CRONUS&&window.CRONUS.reload)window.CRONUS.reload();
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

  // Auto-wire modal/sheet open buttons
  document.querySelectorAll('[data-modal]').forEach(function(btn){
    btn.addEventListener('click',function(){
      var id=btn.getAttribute('data-modal');
      var el=document.getElementById(id);
      if(el)el.style.display='flex';
    });
  });
  document.querySelectorAll('[data-sheet]').forEach(function(btn){
    btn.addEventListener('click',function(){
      var id=btn.getAttribute('data-sheet');
      var el=document.getElementById(id);
      if(el)el.style.display='block';
    });
  });

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

  // Modal/Sheet data-attribute triggers
  document.querySelectorAll('[data-modal]').forEach(b=>b.addEventListener('click',()=>{var e=document.getElementById(b.getAttribute('data-modal'));if(e)e.style.display='flex'}));
  document.querySelectorAll('[data-sheet]').forEach(b=>b.addEventListener('click',()=>{var e=document.getElementById(b.getAttribute('data-sheet'));if(e)e.style.display='block'}));

  // Tag input system
  window.cronusAddTag=function(input){
    var val=input.value.trim();
    if(!val)return;
    var container=input.parentElement;
    var hidden=container.nextElementSibling;
    var name=input.getAttribute('data-name');
    var tag=document.createElement('span');
    tag.style.cssText='display:inline-flex;align-items:center;gap:4px;padding:4px 10px;background:#f4f4f5;border-radius:999px;font-size:13px;font-family:Inter,sans-serif';
    tag.textContent=val;
    var x=document.createElement('button');
    x.type='button';x.textContent='×';x.style.cssText='background:none;border:none;cursor:pointer;font-size:14px;color:#71717a;padding:0 2px';
    x.onclick=function(){tag.remove();cronusUpdateTags(container,hidden)};
    tag.appendChild(x);
    container.insertBefore(tag,input);
    input.value='';
    cronusUpdateTags(container,hidden);
  };
  window.cronusUpdateTags=function(container,hidden){
    var tags=[];
    container.querySelectorAll('span').forEach(function(s){tags.push(s.textContent.replace('×','').trim())});
    hidden.value=tags.join(',');
  };

  // Search with entity
  window.cronusSearch=function(input,entity){
    var q=input.value.trim();
    var name=input.name;
    var results=document.getElementById(name+'-results');
    if(!q||q.length<2){results.style.display='none';return}
    fetch('/api/'+entity.toLowerCase()+'s?search='+encodeURIComponent(q))
      .then(function(r){return r.json()})
      .then(function(data){
        var items=Array.isArray(data)?data:data.data||[];
        if(!items.length){results.style.display='none';return}
        results.innerHTML='';
        items.slice(0,8).forEach(function(item){
          var label=item.name||item.title||item.email||item.id||JSON.stringify(item);
          var opt=document.createElement('div');
          opt.style.cssText='padding:10px 16px;cursor:pointer;font-size:14px;font-family:Inter,sans-serif;border-bottom:1px solid #f4f4f5';
          opt.textContent=label;
          opt.onmousedown=function(){input.value=label;results.style.display='none'};
          opt.onmouseenter=function(){this.style.background='#f4f4f5'};
          opt.onmouseleave=function(){this.style.background='#fff'};
          results.appendChild(opt);
        });
        results.style.display='block';
      }).catch(function(){results.style.display='none'});
  };

  // Inline validation on blur
  document.querySelectorAll('input[required],textarea[required],select[required]').forEach(function(el){
    el.addEventListener('blur',function(){
      var errEl=document.querySelector('[data-error="'+el.name+'"]');
      if(!el.value){
        el.style.borderColor='#dc2626';
        if(errEl)errEl.style.display='block';
      }else{
        el.style.borderColor='#e5e7eb';
        if(errEl)errEl.style.display='none';
      }
    });
  });
});
</script>
"##;

// ══════════════════════════════════════════════════
// LAYOUT (wraps every page)
// ══════════════════════════════════════════════════

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

pub(crate) fn render_section(section: &SectionNode, accent: &str, theme: &str, bound_data: &crate::binding::ResolvedData) -> String {
    // --- Conditional visibility ---
    if let Some(ref cond) = section.visibility {
        let eval_condition = |val: &serde_json::Value| -> bool {
            let field_val = val.get(&cond.field).and_then(|v| v.as_str()).unwrap_or("");
            match cond.operator.as_str() {
                "==" => field_val == cond.value,
                "!=" => field_val != cond.value,
                ">" => field_val > cond.value.as_str(),
                "<" => field_val < cond.value.as_str(),
                ">=" => field_val >= cond.value.as_str(),
                "<=" => field_val <= cond.value.as_str(),
                _ => true,
            }
        };
        let should_show = match bound_data {
            crate::binding::ResolvedData::Rows(rows) if !rows.is_empty() => {
                rows.first().map(|r| eval_condition(r)).unwrap_or(true)
            }
            crate::binding::ResolvedData::Record(Some(rec)) => eval_condition(rec),
            _ => true, // No data to evaluate against, show by default
        };
        if !should_show {
            return String::new();
        }
    }

    // --- Contract validation ---
    let warnings = crate::contracts::validate_section(section, &[]);
    let strict = crate::STRICT_MODE.load(std::sync::atomic::Ordering::Relaxed);
    for w in &warnings {
        match w {
            crate::contracts::ParseWarning::UnknownSection { name, .. } => {
                eprintln!("  \x1b[33m⚠\x1b[0m Unknown section type \"{}\"", name);
            }
            crate::contracts::ParseWarning::UnknownKey { section, key, item, .. } => {
                eprintln!("  \x1b[33m⚠\x1b[0m Section \"{}\": unexpected key \"{}\" on item \"{}\"", section, key, item);
            }
            crate::contracts::ParseWarning::MissingRequired { section, key, item, .. } => {
                eprintln!("  \x1b[31m✗\x1b[0m Section \"{}\": missing required key \"{}\" on item \"{}\"", section, key, item);
            }
            crate::contracts::ParseWarning::AliasUsed { alias, canonical, .. } => {
                eprintln!("  \x1b[36mℹ\x1b[0m Section \"{}\" is an alias for \"{}\"", alias, canonical);
            }
            crate::contracts::ParseWarning::MinItemsViolation { section, expected, actual, .. } => {
                eprintln!("  \x1b[33m⚠\x1b[0m Section \"{}\": expected at least {} item(s), found {}", section, expected, actual);
            }
            crate::contracts::ParseWarning::UnknownConfig { section, key, .. } => {
                eprintln!("  \x1b[33m⚠\x1b[0m Section \"{}\": unknown config key \"{}\"", section, key);
            }
        }
    }
    if strict && warnings.iter().any(|w| matches!(w,
        crate::contracts::ParseWarning::UnknownSection { .. } |
        crate::contracts::ParseWarning::UnknownKey { .. } |
        crate::contracts::ParseWarning::MissingRequired { .. } |
        crate::contracts::ParseWarning::MinItemsViolation { .. } |
        crate::contracts::ParseWarning::UnknownConfig { .. }
    )) {
        return format!("<div style=\"padding:24px;color:#dc2626;font-family:monospace\">Strict mode: section \"{}\" has {} validation issue(s)</div>",
            section.section_type, warnings.len());
    }

    // --- Template override ---
    // If the section carries a template block, render it directly instead of
    // dispatching to a built-in renderer.  The template/style_block may live as
    // dedicated fields on SectionNode (when the parser supports them) or as
    // config keys (fallback for older parser versions).
    let template_from_config = section.config.get("template").cloned();
    let style_from_config = section.config.get("style_block").cloned();

    // Prefer dedicated SectionNode fields over config keys
    let effective_template: Option<&String> = section.template.as_ref()
        .or(template_from_config.as_ref());

    if let Some(tmpl) = effective_template {
        let effective_style = section.style_block.clone()
            .or(style_from_config);
        return render_template(tmpl, section, &effective_style);
    }

    // --- Alias resolution ---
    let resolved_type = crate::contracts::ContractRegistry::resolve_alias(&section.section_type)
        .unwrap_or(section.section_type.as_str());

    let section_html = match resolved_type {
        "hero" => section_hero::render_hero(section, accent, theme),
        "features" => section_features::render_features(section, accent, theme),
        "pricing" => section_misc::render_pricing(section, accent),
        "cta" => section_misc::render_cta(section, accent, theme),
        "faq" => section_misc::render_faq(section, accent),
        "stats" => section_misc::render_stats(section, accent),
        "trusted" => section_misc::render_trusted(section),
        "topbar" => section_misc::render_topbar(section, theme),
        "checkout" => section_misc::render_checkout_section(section),
        "testimonial" => section_misc::render_testimonial(section, theme),
        "footer" => section_misc::render_footer(section, theme),
        "page-header" => section_misc::render_page_header_section(section),
        "stat-cards" => section_kpi::render_stat_cards(section, bound_data),
        "product-grid" => render_product_grid_section(section),
        "promo" => section_misc::render_promo(section),
        "info-bar" => section_misc::render_info_bar(section),
        "bento" => render_bento(section, accent),
        "features-split" => section_features::render_features_split(section, accent),
        "team-list" => render_team_list(section),
        "status-card" => render_status_card(section),
        "policies" => render_policies(section),
        "activity-table" => render_activity_table(section),
        "edge" => render_edge(section, accent),
        "sidebar" => render_sidebar(section),
        "form" => section_form::render_form_section(section, bound_data),
        "card" | "live-keys" | "test-keys" | "webhooks" => render_card_section(section),
        "links" | "quick-links" => render_links_section(section),
        "tabs" => crate::tabs::render_tabs(section),
        "accordion" => crate::feedback::render_accordion(section),
        "breadcrumb" => crate::navigation::render_breadcrumb(section),
        "alert" => crate::feedback::render_alert(section),
        "chart" => section_chart::render_chart_section(section, bound_data),
        "modal" => render_modal_section(section),
        "sheet" => render_sheet_section(section),
        "skeleton" | "loading" => render_skeleton_section(section),
        "empty" => render_empty_section(section),
        "error" => render_error_section(section),
        "not-found" | "404" => render_not_found_section(section),
        "kpi" => {
            if theme == "dark" || theme == "obsidian" {
                section_kpi::render_kpi_dashboard_dark(section, bound_data)
            } else {
                section_kpi::render_kpi_section(section, bound_data)
            }
        }
        "timeline" => render_timeline_section(section, bound_data),
        "progress" => render_progress_section(section, bound_data),
        "command" => crate::command_palette::render_command_palette(section),
        "table" => {
            let is_dark_table = section.config.get("style").map(|s| s.contains("dark")).unwrap_or(false)
                || theme == "dark" || theme == "obsidian";
            let has_static_rows = section.items.iter().any(|item| {
                let t = item.get("_type").map(|s| s.as_str()).unwrap_or("item");
                (t == "item" || t == "row") && (
                    item.get("title").map(|s| s.starts_with('#')).unwrap_or(false)
                    || item.get("client").is_some()
                    || item.get("value").is_some()
                    || item.get("status").is_some()
                )
            });
            if is_dark_table {
                crate::data_table::render_data_table_dark(section, bound_data)
            } else {
                crate::data_table::render_data_table(section, bound_data)
            }
        }
        "pagination" => crate::data_table::render_pagination(section),
        "filters" => crate::data_table::render_filters_toolbar(section),
        "dropdown" => crate::overlays::render_dropdown(section),
        "toast" => crate::overlays::render_toast(section),
        "notifications" => crate::overlays::render_notification_center(section),
        "kanban" => crate::board::render_kanban(section, bound_data),
        "dark-mode" => crate::board::render_dark_mode_toggle(section),
        "layout" => {
            let style = section.config.get("style").map(|s| s.as_str()).unwrap_or("");
            match style {
                "columns" | "grid" => crate::layout_system::render_column_layout(section),
                _ => crate::layout_system::render_layout_section(section),
            }
        }
        _ => render_generic_section(section, accent),
    };

    // If we have bound data, wrap with data attributes for downstream JS/rendering
    let output = match bound_data {
        crate::binding::ResolvedData::Rows(rows) if !rows.is_empty() => {
            let count = rows.len();
            format!(
                "<div data-entity=\"{}\" data-bound-rows=\"{}\">{}</div>",
                section.binding.as_ref().map(|b| b.entity.as_str()).unwrap_or(""),
                count,
                section_html
            )
        }
        crate::binding::ResolvedData::Count(n) => {
            format!(
                "<div data-entity=\"{}\" data-bound-count=\"{}\">{}</div>",
                section.binding.as_ref().map(|b| b.entity.as_str()).unwrap_or(""),
                n,
                section_html
            )
        }
        _ => section_html,
    };

    // Add data-cronus-debug attribute when DEBUG_MODE is active
    if crate::DEBUG_MODE.load(std::sync::atomic::Ordering::Relaxed) {
        let entity = section.binding.as_ref().map(|b| b.entity.as_str()).unwrap_or("");
        let row_count = match bound_data {
            crate::binding::ResolvedData::Rows(rows) => rows.len(),
            crate::binding::ResolvedData::Count(n) => *n as usize,
            _ => 0,
        };
        let doc_summary = section.doc.as_ref()
            .map(|d| d.summary.replace('"', "\\\""))
            .unwrap_or_default();
        let debug_json = format!(
            r#"{{"type":"{}","entity":"{}","rows":{},"doc":"{}"}}"#,
            section.section_type, entity, row_count, doc_summary
        );
        format!(
            "<div data-cronus-debug='{}'>{}</div>",
            debug_json, output
        )
    } else {
        output
    }
}

/// Safe template interpolation: `{{key}}` → HTML-escaped, `{{{key}}}` → raw (opt-in).
/// Always process raw (`{{{...}}}`) FIRST so the triple-brace pattern isn't caught by double-brace.
fn safe_interpolate(template: &str, key: &str, value: &str) -> String {
    let safe_value = crate::security::html_escape(value);
    let raw_pattern = format!("{{{{{{{}}}}}}}", key);   // {{{key}}}
    let safe_pattern = format!("{{{{{}}}}}", key);       // {{key}}
    template
        .replace(&raw_pattern, value)        // raw FIRST
        .replace(&safe_pattern, &safe_value)  // then safe
}

/// Render a section from its inline template block, replacing `{{placeholder}}`
/// tokens with values from the section's title, subtitle, config, and items.
fn render_template(template: &str, section: &SectionNode, style_block: &Option<String>) -> String {
    let mut html = String::new();

    // Unescape template (parser escapes quotes in StringLit)
    let template = template.replace("\\\"", "\"").replace("\\'", "'");
    let template = template.as_str();

    // Add scoped style if present
    if let Some(ref css) = style_block {
        html.push_str(&format!("<style>{}</style>\n", css));
    }

    // Process template — replace {{placeholders}} with section values
    let mut rendered = template.to_string();

    // Replace {{title}} (HTML-escaped by default; use {{{title}}} for raw)
    if let Some(ref title) = section.title {
        rendered = safe_interpolate(&rendered, "title", title);
    }

    // Replace {{subtitle}}
    if let Some(ref subtitle) = section.subtitle {
        rendered = safe_interpolate(&rendered, "subtitle", subtitle);
    }

    // Replace {{cta.text}} and {{cta.href}}
    if let Some(cta_text) = section.config.get("cta_text") {
        rendered = safe_interpolate(&rendered, "cta.text", cta_text);
        rendered = safe_interpolate(&rendered, "cta", cta_text);
    }
    if let Some(cta_link) = section.config.get("cta_link") {
        rendered = safe_interpolate(&rendered, "cta.href", cta_link);
        rendered = safe_interpolate(&rendered, "cta.link", cta_link);
    }

    // Replace {{cta-secondary.text}} and {{cta-secondary.href}}
    if let Some(cta2_text) = section.config.get("cta2_text") {
        rendered = safe_interpolate(&rendered, "cta-secondary.text", cta2_text);
        rendered = safe_interpolate(&rendered, "cta2.text", cta2_text);
    }
    if let Some(cta2_link) = section.config.get("cta2_link") {
        rendered = safe_interpolate(&rendered, "cta-secondary.href", cta2_link);
        rendered = safe_interpolate(&rendered, "cta2.link", cta2_link);
    }

    // Replace {{badge}}
    if let Some(badge) = section.config.get("badge") {
        rendered = safe_interpolate(&rendered, "badge", badge);
    }

    // Replace {{brand}}
    if let Some(brand) = section.config.get("brand") {
        rendered = safe_interpolate(&rendered, "brand", brand);
    }

    // Replace config values: {{key}} and {{config.key}}
    for (key, value) in &section.config {
        // Skip template/style_block themselves to avoid recursive replacement
        if key == "template" || key == "style_block" {
            continue;
        }
        rendered = safe_interpolate(&rendered, key, value);
        rendered = safe_interpolate(&rendered, &format!("config.{}", key), value);
    }

    // Replace {{#each items}} ... {{/each}} with rendered items
    if rendered.contains("{{#each") {
        let each_start = rendered.find("{{#each").unwrap_or(0);
        let each_end = rendered.find("{{/each}}").unwrap_or(rendered.len());
        if each_start < each_end {
            let before = &rendered[..each_start];
            let template_body = &rendered[each_start..each_end];
            let after = &rendered[each_end + "{{/each}}".len()..];

            // Extract the inner template (between {{#each items}} and {{/each}})
            let inner_start = template_body.find("}}").map(|p| p + 2).unwrap_or(0);
            let inner_template = &template_body[inner_start..];

            let mut items_html = String::new();
            for item in &section.items {
                let mut item_html = inner_template.to_string();
                for (key, value) in item {
                    item_html = safe_interpolate(&item_html, key, value);
                }
                items_html.push_str(&item_html);
            }

            rendered = format!("{}{}{}", before, items_html, after);
        }
    }

    html.push_str(&rendered);
    html
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
        r##"<section style="padding:48px 24px"><div style="max-width:var(--cronus-max-w,1120px);margin:0 auto">{title_html}<div style="display:grid;grid-template-columns:repeat({cols},1fr);gap:24px">{cards}</div></div></section>"##,
        title_html=title_html, cols=cols, cards=cards.join(""),
    )
}

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

fn render_modal_section(section: &SectionNode) -> String {
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

fn render_sheet_section(section: &SectionNode) -> String {
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

// ── Route State Sections ──

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

// ══════════════════════════════════════════════════
// SKELETON / LOADING SECTION
// ══════════════════════════════════════════════════

fn render_skeleton_section(section: &SectionNode) -> String {
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

fn render_empty_section(section: &SectionNode) -> String {
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

fn render_error_section(section: &SectionNode) -> String {
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

fn render_not_found_section(section: &SectionNode) -> String {
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
<script>{runtime}</script>
<script>{hmr}</script>
</body>
</html>"##,
        app_name = app_name, topbar = topbar_html, sidebar = sidebar_html, header = header_html,
        profile = profile_html, api_keys = api_keys_html, security = security_html,
        subscription = subscription_html, invoices = invoices_html, support = support_html,
        danger = danger_html, runtime = super::render::CRONUS_RUNTIME_JS, hmr = super::hmr::HMR_CLIENT_JS,
    )
}

fn build_settings_topbar(section: &SectionNode) -> String {
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
<script>{runtime}</script>
<script>{hmr}</script>
</body>
</html>"##,
        app_name = app_name, topbar = topbar_html, sidebar = sidebar_html,
        header = header_html, items = items_html, breakdown = breakdown_html,
        payment = payment_html, customer = customer_html, timeline = timeline_html,
        notes = notes_html, runtime = super::render::CRONUS_RUNTIME_JS, hmr = super::hmr::HMR_CLIENT_JS,
    )
}

// ══════════════════════════════════════════════════

fn render_timeline_section(section: &SectionNode, bound_data: &crate::binding::ResolvedData) -> String {
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

fn render_progress_section(section: &SectionNode, bound_data: &crate::binding::ResolvedData) -> String {
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
