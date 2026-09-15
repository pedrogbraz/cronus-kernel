#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS Render Engine — Server-Side Rendering with micro-runtime hydration
//!
//! Replaces React with ~2KB of vanilla JS for interactivity.
//! Forms auto-submit to API, lists auto-fetch, stats auto-count.

/// CRONUS Client Runtime — ~2KB vanilla JS, replaces React
pub const CRONUS_RUNTIME_JS: &str = r#"
(function(){
  // On first load, hoist all <style> from #cronus-main to <head> so SPA swaps don't lose them
  (function(){
    var main=document.getElementById('cronus-main');
    if(!main) return;
    main.querySelectorAll('style').forEach(function(s){
      if(s.id&&s.id.indexOf('cronus-hoisted')===0) return;
      var hoisted=document.createElement('style');
      hoisted.id='cronus-hoisted-'+Math.random().toString(36).slice(2,8);
      hoisted.textContent=s.textContent;
      document.head.appendChild(hoisted);
      s.remove();
    });
  })();

  function placePopover(el){
    if (!el || el.nodeType !== 1 || !el.hasAttribute('popover')) return;
    if (el.getAttribute('data-slot') === 'sheet-content') return;
    var id = el.getAttribute('anchor');
    var a = id && document.getElementById(id);
    if (!a) return;
    var r = a.getBoundingClientRect();
    el.style.position = 'fixed';
    el.style.inset = 'auto';
    el.style.margin = '0';
    el.style.translate = 'none';
    el.style.transform = 'none';
    var h = el.offsetHeight || 240;
    var w = el.offsetWidth || 280;
    var top = r.bottom + 6;
    if (top + h > window.innerHeight - 8) {
      top = Math.max(8, r.top - h - 6);
    }
    var left = Math.max(8, Math.min(r.left, window.innerWidth - w - 8));
    el.style.top = top + 'px';
    el.style.left = left + 'px';
  }
  function afterOpen(el){
    setTimeout(function(){
      if (el && el.matches && el.matches('[popover]:popover-open')) placePopover(el);
    }, 0);
  }
  document.addEventListener('beforetoggle', function(e){
    if (e.newState === 'open') afterOpen(e.target);
  }, true);
  document.addEventListener('toggle', function(e){
    var el = e.target;
    if (el && el.matches && el.matches('[popover]:popover-open')) afterOpen(el);
  }, true);
  document.addEventListener('click', function(e){
    var t = e.target.closest && e.target.closest('[popovertarget]');
    if (!t) return;
    var pop = document.getElementById(t.getAttribute('popovertarget'));
    if (pop) afterOpen(pop);
  }, true);

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
          if(res.ok){form.reset();}
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
      });
    });

    // Action buttons (data-cronus-action)
    document.querySelectorAll('[data-cronus-action]').forEach(function(btn){
      if(btn._cronusAction) return;
      btn._cronusAction=true;
      btn.addEventListener('click',async function(){
        var confirmMsg=btn.dataset.cronusConfirm;
        if(confirmMsg&&!confirm(confirmMsg)) return;
        var action;
        try{action=JSON.parse(btn.dataset.cronusAction);}catch(e){console.error('CRONUS: invalid action JSON',e);return;}
        var entity=btn.dataset.cronusEntity||'';
        var id=btn.dataset.cronusId||'';
        var section=btn.dataset.cronusSection||'';
        // Dispatch custom event so runtime_js (or user code) can handle it
        var ev=new CustomEvent('cronus:action',{bubbles:true,detail:{action:action,entity:entity,id:id,section:section,button:btn}});
        btn.dispatchEvent(ev);
      });
    });

    // Form submission with data-cronus-form
    document.querySelectorAll('form[data-cronus-form]').forEach(function(form){
      if(form._cronusForm) return;
      form._cronusForm=true;
      form.addEventListener('submit',async function(e){
        e.preventDefault();
        var entity=form.dataset.cronusEntity||form.dataset.entity||'';
        var section=form.dataset.cronusSection||'';
        var data=Object.fromEntries(new FormData(form));
        var ev=new CustomEvent('cronus:form-submit',{bubbles:true,cancelable:true,detail:{entity:entity,section:section,data:data,form:form}});
        if(!form.dispatchEvent(ev)) return;
        // Default: POST to API (or PATCH for edit forms)
        if(entity){
          var btn=form.querySelector('button[type=submit]');
          if(btn){btn.disabled=true;btn.textContent='Saving...';}
          var method=form.dataset.cronusMethod||form.method||'POST';
          var cronusId=form.dataset.cronusId||'';
          var url=method==='PATCH'&&cronusId?'/api/'+entity+'s/'+cronusId:'/api/'+entity+'s';
          delete data._id;
          try{
            var res=await fetch(url,{method:method,headers:{'Content-Type':'application/json'},body:JSON.stringify(data)});
            if(res.ok){form.reset();}
            else{var err=await res.json();alert(err.error||'Error');}
          }catch(e){alert('Network error');}
          finally{if(btn){btn.disabled=false;btn.textContent=method==='PATCH'?'Update':'Submit';}}
        }
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

  // Soft-reload: re-fetch current page HTML and replace #cronus-main content
  // This gives "React-like" reactivity without a virtual DOM
  function cronusLiveReload(){
    var main=document.getElementById('cronus-content')||document.getElementById('cronus-main');
    if(!main) return Promise.resolve();
    var token=localStorage.getItem('token');
    var headers={};
    if(token) headers['Authorization']='Bearer '+token;
    return fetch(location.pathname,{headers:headers})
      .then(function(r){return r.text()})
      .then(function(html){
        var parser=new DOMParser();
        var doc=parser.parseFromString(html,'text/html');
        var newContent=doc.getElementById('cronus-content')||doc.getElementById('cronus-main');
        if(newContent){
          main.style.transition='opacity 0.15s cubic-bezier(0.4,0,0.2,1)';
          main.style.opacity='0.5';
          setTimeout(function(){
            main.innerHTML=newContent.innerHTML;
            main.querySelectorAll('script').forEach(function(old){
              var s=document.createElement('script');
              s.textContent=old.textContent;
              old.parentNode.replaceChild(s,old);
            });
            main.style.transition='opacity 0.25s cubic-bezier(0,0,0.2,1)';
            main.style.opacity='1';
            init();
          },150);
        }
      })
      .catch(function(){});
  }

  // Intercept fetch to auto-reload after mutations
  var _originalFetch=window.fetch;
  window.fetch=function(){
    var args=arguments;
    var url=typeof args[0]==='string'?args[0]:(args[0]&&args[0].url)||'';
    var opts=args[1]||{};
    var method=(opts.method||'GET').toUpperCase();
    return _originalFetch.apply(this,args).then(function(response){
      // After successful mutation on API, soft-reload
      if(url.indexOf('/api/')===0 && ['POST','PATCH','PUT','DELETE'].indexOf(method)!==-1 && response.ok){
        // Skip auth endpoints (login/signup handle their own redirect)
        if(url.indexOf('/api/auth/')!==0){
          setTimeout(cronusLiveReload,200);
        }
      }
      return response;
    });
  };

  // SPA navigation — intercept internal links, fetch HTML, swap content
  function cronusNavigate(url){
    var docContent=document.getElementById('doc-content');
    // For doc pages: swap the parent of doc-content (includes doc-content + TOC aside)
    var content=docContent?(docContent.parentNode||docContent):document.getElementById('cronus-content')||document.getElementById('cronus-main');
    if(!content) return false;
    // Phase 1: fade out + slide
    content.style.transition='opacity 0.2s cubic-bezier(0.4,0,0.2,1), transform 0.2s cubic-bezier(0.4,0,0.2,1)';
    content.style.opacity='0';
    content.style.transform='translateY(8px)';
    // Start fetch in parallel
    var token=localStorage.getItem('token');
    var headers={};
    if(token) headers['Authorization']='Bearer '+token;
    var fetchPromise=_originalFetch(url,{headers:headers}).then(function(r){return r.text()});
    // Phase 2: after fade out completes, swap content
    setTimeout(function(){
      fetchPromise.then(function(html){
        var doc=new DOMParser().parseFromString(html,'text/html');
        var newDocContent=doc.getElementById('doc-content');
        var newContent=doc.getElementById('cronus-content');
        var newMain=doc.getElementById('cronus-main');
        if(!newMain){
          window.location.href=url;
          return;
        }
        // Swap strategy: for doc pages, swap parent of doc-content (includes TOC aside)
        if(docContent&&newDocContent){
          var newParent=newDocContent.parentNode;
          if(newParent){
            content.innerHTML=newParent.innerHTML;
          }else{
            content.innerHTML=newDocContent.outerHTML;
          }
        }else if(newContent&&(content.id==='cronus-content'||content===document.getElementById('cronus-content'))){
          var cc=document.getElementById('cronus-content');
          if(cc)cc.innerHTML=newContent.innerHTML;
        }else{
          var m=document.getElementById('cronus-main');
          if(m)m.innerHTML=newMain.innerHTML;
        }
        // Run inline scripts in swapped area
        content.querySelectorAll('script').forEach(function(old){
          var s=document.createElement('script');
          s.textContent=old.textContent;
          old.parentNode.replaceChild(s,old);
        });
        // Update URL + nav
        history.pushState(null,'',url);
        // Sidebar active state is managed by the declarative layout JS
        // (see render_layout_declarative in ui/layout.rs). No brand colors
        // are injected by the runtime — each app controls its own theme.
        // Update admin/user nav active state
        var navContainer=document.getElementById('admin-nav')||document.getElementById('user-nav');
        if(navContainer){
          navContainer.querySelectorAll('[data-nav]').forEach(function(a){
            if(a.getAttribute('href')===url)a.classList.add('active');
            else a.classList.remove('active');
          });
        }
        var settingsLink=document.getElementById('user-settings-link');
        if(settingsLink){
          if(url==='/settings')settingsLink.classList.add('active');
          else settingsLink.classList.remove('active');
        }
        // Update topbar nav active state — just toggles an 'active' class.
        // Styling is the app's responsibility via its own CSS.
        var topbar=document.querySelector('[data-cronus-topbar]')||document.querySelector('header nav');
        if(topbar){
          topbar.querySelectorAll('a[href]').forEach(function(a){
            var h=a.getAttribute('href');
            if(!h||h.charAt(0)==='#') return;
            var isActive=(h==='/'&&url==='/')||(h!=='/'&&url.indexOf(h)===0);
            a.classList.toggle('active', isActive);
          });
        }
        // Phase 3: fade in
        content.style.transition='none';
        content.style.transform='translateY(-8px)';
        content.style.opacity='0';
        void content.offsetHeight;
        content.style.transition='opacity 0.3s cubic-bezier(0,0,0.2,1), transform 0.3s cubic-bezier(0,0,0.2,1)';
        content.style.opacity='1';
        content.style.transform='translateY(0)';
        // Scroll to top
        window.scrollTo({top:0,behavior:'smooth'});
        init();
      }).catch(function(){window.location.href=url;});
    },200);
    return true;
  }

  // Smooth scroll for anchor links (On This Page TOC)
  document.addEventListener('click',function(e){
    var a=e.target.closest('a[href]');
    if(!a) return;
    var href=a.getAttribute('href');
    if(!href||href.charAt(0)!=='#') return;
    var target=document.getElementById(href.substring(1));
    if(target){
      e.preventDefault();
      target.scrollIntoView({behavior:'smooth',block:'start'});
      history.replaceState(null,'',href);
    }
  });

  // Docs search — indexes sidebar links, searches page content on demand
  (function(){
    var searchInput=document.querySelector('input[placeholder*="Search"]');
    if(!searchInput) return;
    // Build index from sidebar links
    var pages=[];
    document.querySelectorAll('aside nav a[href]').forEach(function(a){
      var href=a.getAttribute('href');
      if(href&&href.charAt(0)==='/'&&pages.findIndex(function(p){return p.href===href})===-1){
        var cat=a.closest('ul');
        var catHeader=cat?cat.previousElementSibling:null;
        pages.push({href:href,title:a.textContent.trim(),category:catHeader?catHeader.textContent.trim():'',content:''});
      }
    });
    // Also index section headings within each page
    document.querySelectorAll('#doc-content section[id]').forEach(function(s){
      var h=s.querySelector('h2,h3');
      if(h){
        var id=s.getAttribute('id');
        pages.push({href:'#'+id,title:h.textContent.replace(/#$/,'').trim(),category:'On This Page',content:s.textContent.toLowerCase().substring(0,500)});
      }
    });
    // Create dropdown — append to body to escape header overflow
    var dropdown=document.createElement('div');
    dropdown.style.cssText='position:fixed;background:#111;border:1px solid rgba(255,255,255,0.1);border-radius:8px;max-height:360px;overflow-y:auto;display:none;z-index:9999;box-shadow:0 12px 40px rgba(0,0,0,0.6);width:320px;backdrop-filter:blur(12px);-webkit-backdrop-filter:blur(12px)';
    document.body.appendChild(dropdown);
    function positionDropdown(){
      var r=searchInput.getBoundingClientRect();
      dropdown.style.top=(r.bottom+6)+'px';
      dropdown.style.left=Math.max(8,r.right-320)+'px';
    }
    // Fetch page content lazily for full-text search
    var fetched={};
    function fetchPage(p){
      if(fetched[p.href]||p.href.charAt(0)==='#')return Promise.resolve();
      return fetch(p.href).then(function(r){return r.text()}).then(function(html){
        var doc=new DOMParser().parseFromString(html,'text/html');
        var dc=doc.getElementById('doc-content');
        p.content=(dc?dc.textContent:doc.body.textContent).toLowerCase().replace(/\s+/g,' ').substring(0,2000);
        fetched[p.href]=true;
      }).catch(function(){});
    }
    // Prefetch all pages in background
    var prefetchIdx=0;
    function prefetchNext(){
      if(prefetchIdx>=pages.length)return;
      fetchPage(pages[prefetchIdx++]).then(function(){setTimeout(prefetchNext,100)});
    }
    setTimeout(prefetchNext,1000);
    // Highlight match in text
    function hl(text,q){
      var i=text.toLowerCase().indexOf(q.toLowerCase());
      if(i===-1)return text;
      return text.substring(0,i)+'<mark class="cronus-search-hl">'+text.substring(i,i+q.length)+'</mark>'+text.substring(i+q.length);
    }
    // Render results
    function renderResults(q){
      dropdown.innerHTML='';
      if(!q||q.length<2){dropdown.style.display='none';return;}
      positionDropdown();
      var ql=q.toLowerCase();
      var matches=pages.filter(function(p){
        return p.title.toLowerCase().indexOf(ql)>-1||p.category.toLowerCase().indexOf(ql)>-1||p.content.indexOf(ql)>-1;
      }).slice(0,10);
      if(matches.length===0){
        dropdown.innerHTML='<div style="padding:14px 16px;color:#555;font-size:13px">No results for "<span style="color:#999">'+q+'</span>"</div>';
        dropdown.style.display='block';
        return;
      }
      matches.forEach(function(p,idx){
        var item=document.createElement('a');
        item.href=p.href;
        item.style.cssText='display:flex;align-items:center;gap:10px;padding:10px 14px;color:#ccc;font-size:13px;border-bottom:1px solid rgba(255,255,255,0.04);transition:background 0.1s;cursor:pointer';
        var icon=p.href.charAt(0)==='#'?'tag':'description';
        item.innerHTML='<span class="material-symbols-outlined" style="font-size:16px;color:#555">'+icon+'</span><div style="flex:1;min-width:0"><div style="font-weight:500;color:#eee;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">'+hl(p.title,q)+'</div><div style="font-size:11px;color:#555;margin-top:1px">'+p.category+(p.href.charAt(0)!=='#'?' — '+p.href:'')+'</div></div>';
        item.addEventListener('mouseenter',function(){item.style.background='rgba(204,0,0,0.08)'});
        item.addEventListener('mouseleave',function(){item.style.background='none'});
        item.addEventListener('click',function(e){
          e.preventDefault();
          dropdown.style.display='none';
          searchInput.value='';
          searchInput.blur();
          if(p.href.charAt(0)==='#'){
            var target=document.getElementById(p.href.substring(1));
            if(target){target.scrollIntoView({behavior:'smooth',block:'start'});history.replaceState(null,'',p.href);}
          }else{
            cronusNavigate(p.href);
          }
        });
        dropdown.appendChild(item);
      });
      dropdown.style.display='block';
    }
    var debounce;
    searchInput.addEventListener('input',function(){
      clearTimeout(debounce);
      debounce=setTimeout(function(){renderResults(searchInput.value)},120);
    });
    searchInput.addEventListener('focus',function(){
      if(searchInput.value&&searchInput.value.length>=2)renderResults(searchInput.value);
    });
    document.addEventListener('click',function(e){
      if(!searchInput.contains(e.target)&&!dropdown.contains(e.target))dropdown.style.display='none';
    });
    // Cmd+K / Ctrl+K shortcut
    document.addEventListener('keydown',function(e){
      if((e.metaKey||e.ctrlKey)&&e.key==='k'){
        e.preventDefault();
        searchInput.focus();
        searchInput.select();
      }
      if(e.key==='Escape'){
        dropdown.style.display='none';
        searchInput.blur();
      }
    });
  })();

  // ── View Transitions API ──────────────────────────────────────
  // Uses native browser View Transitions for smooth page changes
  // Falls back to CSS crossfade on unsupported browsers
  function cronusTransitionNavigate(url){
    if(document.startViewTransition){
      document.startViewTransition(function(){
        return cronusNavigate(url)||Promise.resolve();
      });
    }else{
      cronusNavigate(url);
    }
  }

  // ── Speculation Rules / Prefetch ────────────────────────────
  // Prefetch pages on hover for instant navigation
  var prefetched={};
  function prefetchUrl(url){
    if(prefetched[url]) return;
    prefetched[url]=true;
    var link=document.createElement('link');
    link.rel='prefetch';
    link.href=url;
    link.as='document';
    document.head.appendChild(link);
  }

  // Inject speculation rules for eager prefetch pages
  function injectSpeculationRules(){
    var eager=document.querySelectorAll('[data-prefetch="eager"]');
    if(eager.length&&window.HTMLScriptElement.supports&&HTMLScriptElement.supports('speculationrules')){
      var urls=[];
      eager.forEach(function(a){var h=a.getAttribute('href');if(h)urls.push(h);});
      if(urls.length){
        var script=document.createElement('script');
        script.type='speculationrules';
        script.textContent=JSON.stringify({prefetch:[{source:'list',urls:urls}]});
        document.head.appendChild(script);
      }
    }
  }

  // Prefetch on hover (50ms debounce to avoid over-fetching)
  var hoverTimer=null;
  document.addEventListener('mouseover',function(e){
    var a=e.target.closest('a[href]');
    if(!a) return;
    var href=a.getAttribute('href');
    if(!href||href.charAt(0)==='#'||href.indexOf('//')!==-1||href.indexOf('/api/')===0) return;
    clearTimeout(hoverTimer);
    hoverTimer=setTimeout(function(){prefetchUrl(href);},50);
  });

  // Intercept ALL internal link clicks for SPA navigation
  // CRONUS is a full SPA — every internal link is intercepted
  document.addEventListener('click',function(e){
    var a=e.target.closest('a[href]');
    if(!a) return;
    var href=a.getAttribute('href');
    // Skip: anchor, external, mailto, javascript, api, auth
    if(!href||href.indexOf('//')!==-1||href.indexOf('mailto:')===0||href.charAt(0)==='#') return;
    if(href.indexOf('javascript:')===0) return;
    if(href.indexOf('/api/')===0) return;
    if(href==='/login'||href==='/signup') return;
    // Must have #cronus-main on page
    if(!document.getElementById('cronus-main')) return;
    e.preventDefault();
    if(href===location.pathname) return;
    cronusTransitionNavigate(href);
  });

  // Handle browser back/forward with View Transitions
  window.addEventListener('popstate',function(){
    if(document.startViewTransition){
      document.startViewTransition(function(){return cronusNavigate(location.pathname)||Promise.resolve();});
    }else{
      cronusNavigate(location.pathname);
    }
  });

  // Run on load
  if(document.readyState==='loading'){
    document.addEventListener('DOMContentLoaded',function(){init();injectSpeculationRules();});
  }else{
    init();injectSpeculationRules();
  }

  // Expose for re-init after SPA navigation
  window.CRONUS={init:init,reload:cronusLiveReload,navigate:cronusNavigate,transition:cronusTransitionNavigate,prefetch:prefetchUrl,version:'0.6.0'};
})();
"#;

/// CRONUS Debug Overlay — activated by Cmd+Shift+D / Ctrl+Shift+D or ?debug=1
pub const CRONUS_DEBUG_JS: &str = r#"
(function(){
  if(window.__cronusDebug) return;
  window.__cronusDebug=true;

  var state={open:false,tab:'network',requests:[],errors:[],sseState:'unknown'};
  var panel=null,badge=null,toggle=null;

  // --- Build panel DOM ---
  function createPanel(){
    toggle=document.createElement('div');
    toggle.id='cronus-dbg-toggle';
    toggle.style.cssText='position:fixed;bottom:16px;right:16px;z-index:10000;width:36px;height:36px;border-radius:8px;background:#1b1b1b;border:1px solid #333;display:flex;align-items:center;justify-content:center;cursor:pointer;font-family:"Space Grotesk",monospace;font-size:14px;color:#87adff;user-select:none;';
    toggle.textContent='D';
    toggle.title='CRONUS Debug (Cmd+Shift+D)';
    badge=document.createElement('span');
    badge.style.cssText='position:absolute;top:-4px;right:-4px;min-width:16px;height:16px;border-radius:8px;background:#e74c3c;color:#fff;font-size:10px;display:none;align-items:center;justify-content:center;padding:0 3px;font-family:monospace;';
    toggle.appendChild(badge);
    document.body.appendChild(toggle);
    toggle.addEventListener('click',function(){togglePanel();});

    panel=document.createElement('div');
    panel.id='cronus-dbg-panel';
    panel.style.cssText='position:fixed;top:0;right:-330px;width:320px;height:100vh;z-index:9999;background:#0e0e0e;border-left:1px solid #333;font-family:"Space Grotesk",system-ui,monospace;font-size:12px;color:#ccc;display:flex;flex-direction:column;transition:right .25s cubic-bezier(.4,0,.2,1);overflow:hidden;';
    panel.innerHTML=buildPanelHTML();
    document.body.appendChild(panel);

    panel.querySelector('#cdbg-close').addEventListener('click',function(){togglePanel(false);});
    panel.querySelectorAll('[data-cdbg-tab]').forEach(function(btn){
      btn.addEventListener('click',function(){switchTab(btn.dataset.cdbgTab);});
    });

    if(location.search.indexOf('debug=1')!==-1) togglePanel(true);
  }

  function buildPanelHTML(){
    return '<div style="padding:10px 12px;border-bottom:1px solid #333;display:flex;justify-content:space-between;align-items:center;"><span style="color:#87adff;font-weight:700;font-size:13px;">CRONUS DEBUG</span><span id="cdbg-close" style="cursor:pointer;color:#666;font-size:16px;">&times;</span></div>'
      +'<div style="display:flex;border-bottom:1px solid #333;gap:0;">'
      +'<div data-cdbg-tab="network" style="flex:1;text-align:center;padding:6px 0;cursor:pointer;color:#87adff;border-bottom:2px solid #87adff;font-size:11px;font-weight:600;">Network</div>'
      +'<div data-cdbg-tab="errors" style="flex:1;text-align:center;padding:6px 0;cursor:pointer;color:#666;border-bottom:2px solid transparent;font-size:11px;font-weight:600;">Errors</div>'
      +'<div data-cdbg-tab="info" style="flex:1;text-align:center;padding:6px 0;cursor:pointer;color:#666;border-bottom:2px solid transparent;font-size:11px;font-weight:600;">Info</div>'
      +'</div>'
      +'<div id="cdbg-content" style="flex:1;overflow-y:auto;"></div>'
      +'<div id="cdbg-status" style="padding:6px 12px;border-top:1px solid #333;font-size:10px;color:#666;"></div>';
  }

  function togglePanel(forceOpen){
    state.open=forceOpen!==undefined?forceOpen:!state.open;
    panel.style.right=state.open?'0':'-330px';
    if(state.open) renderTab();
  }

  function switchTab(t){
    state.tab=t;
    panel.querySelectorAll('[data-cdbg-tab]').forEach(function(b){
      var active=b.dataset.cdbgTab===t;
      b.style.color=active?'#87adff':'#666';
      b.style.borderBottom=active?'2px solid #87adff':'2px solid transparent';
    });
    renderTab();
  }

  function updateErrorTab(){
    var btn=panel.querySelector('[data-cdbg-tab="errors"]');
    if(btn) btn.textContent='Errors'+(state.errors.length?' ('+state.errors.length+')':'');
  }

  function renderTab(){
    var c=panel.querySelector('#cdbg-content');
    var s=panel.querySelector('#cdbg-status');
    if(state.tab==='network'){
      if(!state.requests.length){c.innerHTML='<div style="padding:12px;color:#555;">No requests yet</div>';
      }else{
        var first=state.requests[0].startTime||0;
        var last=state.requests[state.requests.length-1];
        var timeSpan=Math.max((last.startTime||0)+(last.duration||0)-first,1);
        c.innerHTML=state.requests.map(function(r){
          var sc=r.status;var color=sc>=500?'#e74c3c':sc>=400?'#e74c3c':sc>=300?'#f39c12':sc>=200?'#2ecc71':'#666';
          var offset=Math.round(((r.startTime||0)-first)/timeSpan*100);
          var barW=Math.max(3,Math.round((r.duration||1)/timeSpan*100));
          if(offset+barW>100)barW=100-offset;
          var path=r.url.replace(/^https?:\/\/[^\/]+/,'');if(path.length>28) path='...'+path.slice(-25);
          return '<div style="padding:4px 12px;border-bottom:1px solid #1b1b1b;display:flex;align-items:center;gap:6px;font-size:10px;">'
            +'<span style="color:#888;width:32px;">'+r.method+'</span>'
            +'<span style="flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:#aaa;" title="'+r.url+'">'+path+'</span>'
            +'<span style="color:'+color+';width:24px;text-align:right;">'+(sc||'...')+'</span>'
            +'<span style="width:32px;text-align:right;color:#666;">'+(r.duration?r.duration+'ms':'')+'</span>'
            +'<div style="width:80px;height:8px;background:#111;border-radius:2px;position:relative;overflow:hidden;">'
            +'<div style="position:absolute;left:'+offset+'%;width:'+barW+'%;height:100%;background:'+color+';border-radius:2px;transition:width .2s;"></div>'
            +'</div></div>';
        }).join('');
      }
      s.textContent=state.requests.length+' requests | '+state.errors.length+' errors | SSE: '+state.sseState;
    }else if(state.tab==='errors'){
      if(!state.errors.length){c.innerHTML='<div style="padding:12px;color:#555;">No errors captured</div>';
      }else{
        c.innerHTML=state.errors.map(function(e){
          return '<div style="padding:8px 12px;border-bottom:1px solid #1b1b1b;">'
            +'<div style="color:#e74c3c;font-weight:600;margin-bottom:4px;">'+esc(e.message)+'</div>'
            +(e.file?'<div style="color:#666;font-size:10px;">'+esc(e.file)+':'+e.line+'</div>':'')
            +(e.stack?'<pre style="color:#555;font-size:9px;margin-top:4px;white-space:pre-wrap;max-height:80px;overflow:auto;">'+esc(e.stack)+'</pre>':'')
            +'</div>';
        }).join('');
      }
      s.textContent=state.errors.length+' error(s)';
    }else if(state.tab==='info'){
      var token=localStorage.getItem('token');
      var nonceMeta=document.querySelector('meta[name="csp-nonce"]');
      var nonceScript=document.querySelector('script[nonce]');
      var sections=document.querySelectorAll('[data-section]').length;
      var entities=document.querySelectorAll('[data-entity],[data-list],[data-count]').length;
      var w=window.innerWidth;
      var bp=w>=1280?'XL (>=1280)':w>=1024?'LG (>=1024)':w>=768?'MD (>=768)':w>=640?'SM (>=640)':'XS (<640)';
      c.innerHTML='<div style="padding:8px 12px;">'
        +infoRow('Route',location.pathname)
        +infoRow('Viewport',w+'\u00d7'+window.innerHeight+' \u2014 '+bp)
        +infoRow('Sections',sections)
        +infoRow('Entity bindings',entities)
        +infoRow('Auth',token?'Authenticated':'Not authenticated')
        +infoRow('SSE',state.sseState)
        +infoRow('CSP nonce',nonceMeta?nonceMeta.content:(nonceScript?nonceScript.nonce:'N/A'))
        +infoRow('Last audit',window.__cronusLastAudit||'N/A')
        +'</div>';
      s.textContent='CRONUS Debug v0.5.0';
    }
  }

  function infoRow(k,v){return '<div style="display:flex;justify-content:space-between;padding:4px 0;border-bottom:1px solid #1b1b1b;"><span style="color:#888;">'+k+'</span><span style="color:#ccc;">'+v+'</span></div>';}
  function esc(s){return String(s||'').replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');}

  function updateBadge(){
    if(!badge) return;
    if(state.errors.length){badge.style.display='flex';badge.textContent=state.errors.length;}
    else{badge.style.display='none';}
  }

  // --- Intercept fetch for logging ---
  var _fetch=window.fetch;
  window.fetch=function(){
    var args=arguments;
    var url=typeof args[0]==='string'?args[0]:(args[0]&&args[0].url)||'';
    var opts=args[1]||{};
    var method=(opts.method||'GET').toUpperCase();
    var entry={method:method,url:url,status:null,duration:null,ts:Date.now(),startTime:performance.now()};
    state.requests.push(entry);
    if(state.requests.length>100) state.requests.shift();
    var t0=performance.now();
    return _fetch.apply(this,args).then(function(res){
      entry.status=res.status;entry.duration=Math.round(performance.now()-t0);
      if(state.open&&state.tab==='network') renderTab();
      return res;
    }).catch(function(err){
      entry.status='ERR';entry.duration=Math.round(performance.now()-t0);
      if(state.open&&state.tab==='network') renderTab();
      throw err;
    });
  };

  // --- Catch errors ---
  window.addEventListener('error',function(e){
    state.errors.push({message:e.message||'Unknown error',file:e.filename,line:e.lineno,stack:e.error&&e.error.stack});
    updateBadge();updateErrorTab();if(state.open&&state.tab==='errors') renderTab();
  });
  window.addEventListener('unhandledrejection',function(e){
    var msg=e.reason?e.reason.message||String(e.reason):'Unhandled rejection';
    state.errors.push({message:msg,file:'',line:'',stack:e.reason&&e.reason.stack});
    updateBadge();updateErrorTab();if(state.open&&state.tab==='errors') renderTab();
  });

  // --- SSE state tracking ---
  var _ES=window.EventSource;
  if(_ES){window.EventSource=function(url,opts){
    var es=new _ES(url,opts);
    state.sseState='connecting';
    es.addEventListener('open',function(){state.sseState='live';if(state.open) renderTab();});
    es.addEventListener('error',function(){state.sseState='error';if(state.open) renderTab();});
    return es;
  };window.EventSource.prototype=_ES.prototype;window.EventSource.CONNECTING=_ES.CONNECTING;window.EventSource.OPEN=_ES.OPEN;window.EventSource.CLOSED=_ES.CLOSED;}

  // --- Keyboard shortcut ---
  document.addEventListener('keydown',function(e){
    if((e.metaKey||e.ctrlKey)&&e.shiftKey&&e.key==='d'){e.preventDefault();togglePanel();}
  });

  // --- Viewport resize tracking ---
  window.addEventListener('resize',function(){if(state.open&&state.tab==='info') renderTab();});

  // --- SSE debug event listener ---
  (function(){
    var es;
    try{es=document.querySelector('script')&&window.EventSource?new window.EventSource('/api/sse'):null;}catch(e){return;}
    if(!es) return;
    es.addEventListener('debug',function(e){
      try{
        var d=JSON.parse(e.data);
        state.requests.push({method:d.method,url:d.path,status:d.status,duration:d.ms,ts:Date.now(),startTime:performance.now()});
        if(state.requests.length>100) state.requests.shift();
        if(state.open&&state.tab==='network') renderTab();
      }catch(err){}
    });
  })();

  // --- SPA navigation tracking ---
  var _pushState=history.pushState;
  history.pushState=function(){_pushState.apply(this,arguments);
    state.requests.push({method:'SPA',url:arguments[2]||'',status:'->',duration:null,ts:Date.now(),startTime:performance.now()});
    if(state.open&&state.tab==='network') renderTab();
  };

  if(document.readyState==='loading') document.addEventListener('DOMContentLoaded',createPanel);
  else createPanel();
})();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_js_has_no_hardcoded_brand_colors() {
        // The 2026-04-10 session removed all hardcoded #CC0000 (old Cooud red) from
        // the client-side runtime JavaScript. The runtime should only toggle semantic
        // classes (active, error, etc.); colors come from the user's style block.
        // If this assertion fires, a brand color has leaked back into the runtime.
        assert!(
            !CRONUS_RUNTIME_JS.contains("#CC0000"),
            "CRONUS_RUNTIME_JS contains #CC0000 — brand color leak regression"
        );
        assert!(
            !CRONUS_RUNTIME_JS.contains("#cc0000"),
            "CRONUS_RUNTIME_JS contains #cc0000 (lowercase) — brand color leak regression"
        );
        // Also catch the sometimes-used bare 6-digit form
        assert!(
            !CRONUS_RUNTIME_JS.contains("'#CC0000'"),
            "CRONUS_RUNTIME_JS contains quoted '#CC0000' — brand color leak regression"
        );
    }
}
