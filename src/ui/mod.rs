//! CRONUS UI — Server-Side HTML Renderer
//!
//! Generates complete HTML pages from the AST.
//! No React, no frameworks — pure HTML + Tailwind CDN + vanilla JS.

pub mod audit_layout;
pub mod component;
pub mod dashboard;
pub mod layout;
pub mod page;
mod section_chart;
pub(crate) mod section_extra;
mod section_features;
mod section_form;
mod section_hero;
mod section_kpi;
mod section_misc;
pub(crate) mod section_mode;
mod util;

pub use component::{render_components_inline, render_components_page, render_light_app_page};
pub use dashboard::{
    render_billing_dashboard, render_checkout_dashboard, render_generic_dashboard,
    render_order_detail_dashboard, render_payment_links_dashboard, render_payouts_dashboard,
    render_security_dashboard, render_settings_dashboard, render_unified_dashboard,
};

pub use layout::*;
pub use page::{render_auth_page, render_page};

use crate::parser::SectionNode;

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
::view-transition-old(root){animation:fadeOut .15s ease-out}
::view-transition-new(root){animation:fadeIn .2s ease-in}
::view-transition-old(cronus-content){animation:slideDown .2s ease-out reverse}
::view-transition-new(cronus-content){animation:slideUp .25s cubic-bezier(.16,1,.3,1)}
@keyframes fadeOut{from{opacity:1}to{opacity:0}}
#cronus-main{view-transition-name:cronus-content}
[data-section]{view-transition-name:auto}
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

function _esc(s){
  return String(s==null?'':s).replace(/[&<>"']/g,function(c){return {'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]});
}

function _buildCell(col,val,ci){
  var td=document.createElement('td');
  td.style.cssText='padding:16px 32px';
  var lv=(val+'').toLowerCase();
  if(col==='status'||col==='severity'){
    var c=_cs[lv]||'#71717a';
    var p=(lv==='live'||lv==='rolling'||lv==='processing')?'animation:pulse 2s ease-in-out infinite;':'';
    td.innerHTML='<span style="display:inline-flex;align-items:center;gap:6px;padding:4px 12px;font-size:11px;font-weight:600;border-radius:999px;background:'+c+'1f;color:'+c+'"><span style="width:6px;height:6px;border-radius:50%;background:'+c+';flex-shrink:0;'+p+'"></span>'+_esc(val)+'</span>';
  } else if(ci===0){
    td.innerHTML='<span style="font-size:13px;color:#e2e2e2;font-weight:600">'+_esc(val)+'</span>';
  } else {
    td.innerHTML='<span style="font-size:13px;color:rgba(226,226,226,0.8)">'+_esc(val)+'</span>';
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
      // Kernel forms are submitted once, by the action runtime, via /_form.
      if(window._cronusActions&&form.hasAttribute('data-cronus-form')) return;
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

pub(crate) fn render_section(
    section: &SectionNode,
    accent: &str,
    theme: &str,
    bound_data: &crate::binding::ResolvedData,
) -> String {
    let entity = section
        .binding
        .as_ref()
        .map(|b| b.entity.as_str())
        .unwrap_or("");
    // `system` renders the dark branch; `section_mode::adapt` then maps the
    // dark palette to the page mode (no-op in dark).
    let render_theme = section_mode::render_theme(theme);
    let html = crate::cronus_ui_data::with_binding(entity, bound_data, || {
        render_section_inner(section, accent, render_theme, bound_data)
    });
    section_mode::adapt(html, theme)
}

fn attach_voodoo_form(html: String, entity: &str) -> String {
    if !crate::voodoo::enabled() || entity.is_empty() {
        return html;
    }
    if html.contains("v-submit=") {
        return html;
    }
    let needle = "<form";
    if let Some(pos) = html.find(needle) {
        let path = format!("/api/{}", entity.to_lowercase());
        let inject = format!(
            "<form v-submit=\"{path}\" v-method=\"POST\" data-cronus-entity=\"{entity}\" v-toast-success=\"Saved\""
        );
        let mut out = String::with_capacity(html.len() + inject.len());
        out.push_str(&html[..pos]);
        out.push_str(&inject);
        out.push_str(&html[pos + needle.len()..]);
        return out;
    }
    html
}

fn render_section_inner(
    section: &SectionNode,
    accent: &str,
    theme: &str,
    bound_data: &crate::binding::ResolvedData,
) -> String {
    let script_nonce = crate::security::script_nonce_attr();
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
            crate::contracts::ParseWarning::UnknownKey {
                section, key, item, ..
            } => {
                eprintln!(
                    "  \x1b[33m⚠\x1b[0m Section \"{}\": unexpected key \"{}\" on item \"{}\"",
                    section, key, item
                );
            }
            crate::contracts::ParseWarning::MissingRequired {
                section, key, item, ..
            } => {
                eprintln!(
                    "  \x1b[31m✗\x1b[0m Section \"{}\": missing required key \"{}\" on item \"{}\"",
                    section, key, item
                );
            }
            crate::contracts::ParseWarning::AliasUsed {
                alias, canonical, ..
            } => {
                eprintln!(
                    "  \x1b[36mℹ\x1b[0m Section \"{}\" is an alias for \"{}\"",
                    alias, canonical
                );
            }
            crate::contracts::ParseWarning::MinItemsViolation {
                section,
                expected,
                actual,
                ..
            } => {
                eprintln!(
                    "  \x1b[33m⚠\x1b[0m Section \"{}\": expected at least {} item(s), found {}",
                    section, expected, actual
                );
            }
            crate::contracts::ParseWarning::UnknownConfig { section, key, .. } => {
                eprintln!(
                    "  \x1b[33m⚠\x1b[0m Section \"{}\": unknown config key \"{}\"",
                    section, key
                );
            }
        }
    }
    if strict
        && warnings.iter().any(|w| {
            matches!(
                w,
                crate::contracts::ParseWarning::UnknownSection { .. }
                    | crate::contracts::ParseWarning::UnknownKey { .. }
                    | crate::contracts::ParseWarning::MissingRequired { .. }
                    | crate::contracts::ParseWarning::MinItemsViolation { .. }
                    | crate::contracts::ParseWarning::UnknownConfig { .. }
            )
        })
    {
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
    let effective_template: Option<&String> =
        section.template.as_ref().or(template_from_config.as_ref());

    if let Some(tmpl) = effective_template {
        let effective_style = section.style_block.clone().or(style_from_config);
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
        "product-grid" => section_extra::render_product_grid_section(section),
        "promo" => section_misc::render_promo(section),
        "info-bar" => section_misc::render_info_bar(section),
        "bento" => section_extra::render_bento(section, accent),
        "features-split" => section_features::render_features_split(section, accent),
        "team-list" => section_extra::render_team_list(section),
        "status-card" => section_extra::render_status_card(section),
        "policies" => section_extra::render_policies(section),
        "activity-table" => section_extra::render_activity_table(section),
        "edge" => section_extra::render_edge(section, accent),
        "sidebar" => section_extra::render_sidebar(section),
        "form" => section_form::render_form_section(section, bound_data),
        "card" | "live-keys" | "test-keys" | "webhooks" => {
            section_extra::render_card_section(section, bound_data)
        }
        "links" | "quick-links" => section_extra::render_links_section(section),
        "tabs" => crate::tabs::render_tabs(section),
        "accordion" => crate::feedback::render_accordion(section),
        "breadcrumb" => crate::navigation::render_breadcrumb(section),
        "alert" => crate::feedback::render_alert(section),
        "chart" => section_chart::render_chart_section(section, bound_data),
        "modal" => section_extra::render_modal_section(section),
        "sheet" => section_extra::render_sheet_section(section),
        "skeleton" | "loading" => section_extra::render_skeleton_section(section),
        "empty" => section_extra::render_empty_section(section),
        "error" => section_extra::render_error_section(section),
        "not-found" | "404" => section_extra::render_not_found_section(section),
        "kpi" => {
            if theme == "dark" || theme == "obsidian" {
                section_kpi::render_kpi_dashboard_dark(section, bound_data)
            } else {
                section_kpi::render_kpi_section(section, bound_data)
            }
        }
        "timeline" => section_extra::render_timeline_section(section, bound_data),
        "progress" => section_extra::render_progress_section(section, bound_data),
        "command" => crate::command_palette::render_command_palette(section),
        "table" => {
            let is_dark_table = section
                .config
                .get("style")
                .map(|s| s.contains("dark"))
                .unwrap_or(false)
                || theme == "dark"
                || theme == "obsidian";
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
            let style = section
                .config
                .get("style")
                .map(|s| s.as_str())
                .unwrap_or("");
            match style {
                "columns" | "grid" => crate::layout_system::render_column_layout(section),
                _ => crate::layout_system::render_layout_section(section),
            }
        }
        _ => {
            if crate::cronus_ui_widgets::FAMILIES.contains(&section.section_type.as_str()) {
                let comp =
                    crate::cronus_ui_data::component_from_section(&section.section_type, section);
                crate::cronus_ui_widgets::render(&comp)
                    .unwrap_or_else(|| section_extra::render_generic_section(section, accent))
            } else {
                section_extra::render_generic_section(section, accent)
            }
        }
    };

    let entity_name = section
        .binding
        .as_ref()
        .map(|b| b.entity.as_str())
        .unwrap_or("");
    let section_html = attach_voodoo_form(section_html, entity_name);

    // If we have bound data, wrap with data attributes for downstream JS/rendering
    let output = match bound_data {
        crate::binding::ResolvedData::Rows(rows) if !rows.is_empty() => {
            let count = rows.len();
            format!(
                "<div data-entity=\"{}\" data-bound-rows=\"{}\">{}</div>",
                section
                    .binding
                    .as_ref()
                    .map(|b| b.entity.as_str())
                    .unwrap_or(""),
                count,
                section_html
            )
        }
        crate::binding::ResolvedData::Record(Some(_)) => {
            format!(
                "<div data-entity=\"{}\" data-bound-rows=\"1\">{}</div>",
                section
                    .binding
                    .as_ref()
                    .map(|b| b.entity.as_str())
                    .unwrap_or(""),
                section_html
            )
        }
        crate::binding::ResolvedData::Count(n) => {
            format!(
                "<div data-entity=\"{}\" data-bound-count=\"{}\">{}</div>",
                section
                    .binding
                    .as_ref()
                    .map(|b| b.entity.as_str())
                    .unwrap_or(""),
                n,
                section_html
            )
        }
        _ => section_html,
    };

    // Live SSE: auto-refresh when entity data changes
    let is_live = section.binding.as_ref().map(|b| b.live).unwrap_or(false);
    let output = if is_live {
        let entity = section
            .binding
            .as_ref()
            .map(|b| b.entity.as_str())
            .unwrap_or("");
        let live_id = format!("live_{}", entity.to_lowercase());
        format!(
            r#"<div id="{live_id}" data-live-entity="{entity}">{output}</div>
<script{script_nonce}>
(function(){{
  var el=document.getElementById('{live_id}');
  if(!el)return;
  var es=new EventSource('/api/sse');
  es.addEventListener('data_change',function(e){{
    try{{
      var d=JSON.parse(e.data);
      if(d.entity==='{entity}'){{
        // Reload the page content via SPA router
        if(window.__cronusNavigate){{
          window.__cronusNavigate(location.href,false);
        }}else{{
          location.reload();
        }}
      }}
    }}catch(err){{}}
  }});
  es.onerror=function(){{
    // Stop hammering a dead stream; the next navigation re-subscribes.
    es.close();
  }};
}})();
</script>"#,
            live_id = live_id,
            entity = entity,
            output = output
        )
    } else {
        output
    };

    // Add data-cronus-debug attribute when DEBUG_MODE is active
    if crate::DEBUG_MODE.load(std::sync::atomic::Ordering::Relaxed) {
        let entity = section
            .binding
            .as_ref()
            .map(|b| b.entity.as_str())
            .unwrap_or("");
        let row_count = match bound_data {
            crate::binding::ResolvedData::Rows(rows) => rows.len(),
            crate::binding::ResolvedData::Count(n) => *n as usize,
            _ => 0,
        };
        let doc_summary = section
            .doc
            .as_ref()
            .map(|d| d.summary.replace('"', "\\\""))
            .unwrap_or_default();
        let debug_json = format!(
            r#"{{"type":"{}","entity":"{}","rows":{},"doc":"{}"}}"#,
            section.section_type, entity, row_count, doc_summary
        );
        format!("<div data-cronus-debug='{}'>{}</div>", debug_json, output)
    } else {
        output
    }
}

/// Safe template interpolation: `{{key}}` → HTML-escaped, `{{{key}}}` → raw (opt-in).
/// Always process raw (`{{{...}}}`) FIRST so the triple-brace pattern isn't caught by double-brace.
fn safe_interpolate(template: &str, key: &str, value: &str) -> String {
    let safe_value = crate::security::html_escape(value);
    let raw_pattern = format!("{{{{{{{}}}}}}}", key); // {{{key}}}
    let safe_pattern = format!("{{{{{}}}}}", key); // {{key}}
    template
        .replace(&raw_pattern, value) // raw FIRST
        .replace(&safe_pattern, &safe_value) // then safe
}

/// Render a section from its inline template block, replacing `{{placeholder}}`
/// tokens with values from the section's title, subtitle, config, and items.
fn render_template(template: &str, section: &SectionNode, style_block: &Option<String>) -> String {
    let mut html = String::new();

    // Unescape template (parser escapes quotes in StringLit)
    let template = template.replace("\\\"", "\"").replace("\\'", "'");
    // Scripts the app author wrote in the template are trusted: mark them for
    // the CSP nonce now, before any value is interpolated. A `<script{script_nonce}>` that
    // arrives through a placeholder stays unmarked and is blocked.
    let template = crate::security::mark_kernel_scripts(&template);
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
