#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS Render Engine — Server-Side Rendering with micro-runtime hydration
//!
//! Replaces React with ~2KB of vanilla JS for interactivity.
//! Forms auto-submit to API, lists auto-fetch, stats auto-count.

/// CRONUS Client Runtime — ~2KB vanilla JS, replaces React
pub const CRONUS_RUNTIME_JS: &str = r#"
(function(){
  function init(){
    // Auto-bind forms to API
    document.querySelectorAll('form[data-entity]').forEach(function(form){
      if(form._cronus) return;
      form._cronus=true;
      var entity=form.dataset.entity;
      form.addEventListener('submit',async function(e){
        e.preventDefault();
        var data=Object.fromEntries(new FormData(form));
        var btn=form.querySelector('button[type=submit]');
        if(btn){btn.disabled=true;btn.textContent='Saving...';}
        try{
          var res=await fetch('/api/'+entity+'s',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(data)});
          if(res.ok){form.reset();window.location.reload();}
          else{var err=await res.json();alert(err.error||'Error');}
        }catch(e){alert('Network error');}
        finally{if(btn){btn.disabled=false;btn.textContent='Submit';}}
      });
    });

    // Auto-fetch lists
    document.querySelectorAll('[data-list]').forEach(async function(el){
      if(el._cronus) return;
      el._cronus=true;
      var entity=el.dataset.list;
      var cols=(el.dataset.cols||'').split(',').filter(Boolean);
      try{
        var res=await fetch('/api/'+entity+'s');
        var data=await res.json();
        if(!Array.isArray(data)) data=data.data||[];
        if(!data.length){el.innerHTML='<p class="text-neutral-500 p-4">No data yet</p>';return;}
        if(cols.length){
          var html='<table class="w-full"><thead class="border-b border-neutral-800"><tr>';
          cols.forEach(function(c){html+='<th class="px-4 py-3 text-left text-xs font-medium text-neutral-400 uppercase">'+c+'</th>';});
          html+='</tr></thead><tbody>';
          data.forEach(function(row){
            html+='<tr class="border-b border-neutral-800/50">';
            cols.forEach(function(c){html+='<td class="px-4 py-3 text-sm text-neutral-300">'+(row[c]||'—')+'</td>';});
            html+='</tr>';
          });
          html+='</tbody></table>';
          el.innerHTML=html;
        }else{
          el.innerHTML=data.map(function(item){
            return '<div class="p-3 border-b border-neutral-800 text-sm text-neutral-300">'+JSON.stringify(item)+'</div>';
          }).join('');
        }
      }catch(e){el.innerHTML='<p class="text-neutral-500 p-4">Error loading</p>';}
    });

    // Auto-fetch stats/counts
    document.querySelectorAll('[data-count]').forEach(async function(el){
      if(el._cronus) return;
      el._cronus=true;
      var entity=el.dataset.count;
      try{
        var res=await fetch('/api/'+entity+'s');
        var data=await res.json();
        if(!Array.isArray(data)) data=data.data||[];
        el.textContent=data.length;
      }catch(e){el.textContent='0';}
    });

    // Delete buttons
    document.querySelectorAll('[data-delete]').forEach(function(btn){
      if(btn._cronus) return;
      btn._cronus=true;
      btn.addEventListener('click',async function(){
        var entity=btn.dataset.delete;
        var id=btn.dataset.id;
        if(!confirm('Delete?')) return;
        await fetch('/api/'+entity+'s/'+id,{method:'DELETE'});
        window.location.reload();
      });
    });

    // Tab switches
    document.querySelectorAll('[data-tab]').forEach(function(btn){
      if(btn._cronus) return;
      btn._cronus=true;
      btn.addEventListener('click',function(){
        var group=btn.dataset.tabGroup||'default';
        var target=btn.dataset.tab;
        document.querySelectorAll('[data-tab-group="'+group+'"]').forEach(function(b){b.classList.remove('text-white','border-b-2');b.classList.add('text-neutral-400');});
        btn.classList.add('text-white','border-b-2');btn.classList.remove('text-neutral-400');
        document.querySelectorAll('[data-tab-content]').forEach(function(el){
          el.style.display=el.dataset.tabContent===target?'block':'none';
        });
      });
    });

    // Search filter
    document.querySelectorAll('[data-search]').forEach(function(input){
      if(input._cronus) return; input._cronus=true;
      var target = input.dataset.search;
      input.addEventListener('input', function(){
        var q = input.value.toLowerCase();
        var container = document.querySelector('[data-search-target="'+target+'"]');
        if(!container) return;
        container.querySelectorAll('[data-search-item]').forEach(function(row){
          row.style.display = row.textContent.toLowerCase().indexOf(q) !== -1 ? '' : 'none';
        });
      });
    });

    // Sort table columns
    document.querySelectorAll('[data-sort]').forEach(function(th){
      if(th._cronus) return; th._cronus=true;
      th.style.cursor='pointer';
      th.addEventListener('click', function(){
        var table = th.closest('table');
        var tbody = table.querySelector('tbody');
        if(!tbody) return;
        var idx = parseInt(th.dataset.sort);
        var rows = Array.from(tbody.querySelectorAll('tr'));
        var asc = th.dataset.sortDir !== 'asc';
        th.dataset.sortDir = asc ? 'asc' : 'desc';
        rows.sort(function(a,b){
          var av = (a.cells[idx]||{}).textContent||'';
          var bv = (b.cells[idx]||{}).textContent||'';
          var an = parseFloat(av.replace(/[^0-9.-]/g,''));
          var bn = parseFloat(bv.replace(/[^0-9.-]/g,''));
          if(!isNaN(an)&&!isNaN(bn)) return asc ? an-bn : bn-an;
          return asc ? av.localeCompare(bv) : bv.localeCompare(av);
        });
        rows.forEach(function(r){ tbody.appendChild(r); });
      });
    });

    // Modal open/close
    document.querySelectorAll('[data-modal-open]').forEach(function(btn){
      if(btn._cronus) return; btn._cronus=true;
      btn.addEventListener('click', function(){
        var el = document.getElementById(btn.dataset.modalOpen);
        if(el) el.style.display = 'flex';
      });
    });
    document.querySelectorAll('[data-modal-close]').forEach(function(btn){
      if(btn._cronus) return; btn._cronus=true;
      btn.addEventListener('click', function(){
        var m = btn.closest('[id]');
        if(m) m.style.display = 'none';
      });
    });

    // Two-way binding: input <-> text display
    (function(){
      var state = window.__cronusState || (window.__cronusState = {});
      document.querySelectorAll('[data-bind]').forEach(function(el){
        if(el._cronusBind) return; el._cronusBind=true;
        var key = el.dataset.bind;
        el.addEventListener('input', function(){
          state[key] = el.type==='checkbox' ? el.checked : el.value;
          document.querySelectorAll('[data-bind-text="'+key+'"]').forEach(function(t){
            t.textContent = state[key] || '';
          });
        });
      });
    })();

    // Declarative data loading
    document.querySelectorAll('[data-fetch]').forEach(async function(el){
      if(el._cronusFetch) return; el._cronusFetch=true;
      try {
        var res = await fetch(el.dataset.fetch);
        var data = await res.json();
        el._cronusData = data;
        var tmpl = el.dataset.render;
        if(tmpl === 'count') {
          var arr = Array.isArray(data) ? data : (data.data || []);
          el.textContent = arr.length;
        } else if(tmpl === 'sum') {
          var arr = Array.isArray(data) ? data : (data.data || []);
          var field = el.dataset.field || 'amount';
          var sum = arr.reduce(function(s,r){ return s + (parseFloat(r[field]) || 0); }, 0);
          el.textContent = (sum/100).toFixed(2);
        } else if(tmpl === 'json') {
          el.textContent = JSON.stringify(data, null, 2);
        }
        el.dispatchEvent(new CustomEvent('cronus:fetch:done', {bubbles:true, detail:data}));
      } catch(e) {
        el.dataset.error = 'true';
        el.textContent = 'Error loading';
      }
    });

    // Cursor/offset pagination
    document.querySelectorAll('[data-paginate]').forEach(function(el){
      if(el._cronusPaginate) return; el._cronusPaginate=true;
      var state = {offset:0, limit: parseInt(el.dataset.limit)||20, base: el.dataset.paginate};
      var target = el.dataset.target;
      el.querySelectorAll('[data-page-next]').forEach(function(btn){
        btn.addEventListener('click', function(){
          state.offset += state.limit;
          loadPage();
        });
      });
      el.querySelectorAll('[data-page-prev]').forEach(function(btn){
        btn.addEventListener('click', function(){
          state.offset = Math.max(0, state.offset - state.limit);
          loadPage();
        });
      });
      function loadPage(){
        var url = state.base + '?limit='+state.limit+'&offset='+state.offset;
        fetch(url).then(function(r){return r.json()}).then(function(d){
          var container = document.getElementById(target);
          if(container) container.dispatchEvent(new CustomEvent('cronus:data', {detail:d}));
        });
      }
    });
  }

  // Run on load
  if(document.readyState==='loading'){document.addEventListener('DOMContentLoaded',init);}
  else{init();}

  // Expose for re-init after SPA navigation
  window.CRONUS={init:init,version:'0.3.0'};
})();
"#;
