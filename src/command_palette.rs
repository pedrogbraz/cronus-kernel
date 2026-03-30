use std::collections::HashMap;
use crate::parser::SectionNode;

/// Renders a command palette (Cmd+K overlay) from a "command" section.
/// Returns a complete HTML string with inline <style> and <script>.
pub fn render_command_palette(section: &SectionNode) -> String {
    let palette_id = section.config.get("id").map(|s| s.as_str()).unwrap_or("command-palette");

    // Build items HTML
    let mut items_html = String::new();
    let mut shortcut_bindings = String::new();

    for (i, item) in section.items.iter().enumerate() {
        let name = item.get("title").or(item.get("name")).map(|s| s.as_str()).unwrap_or("");
        let shortcut = item.get("shortcut").map(|s| s.as_str()).unwrap_or("");
        let link = item.get("link").map(|s| s.as_str()).unwrap_or("#");
        let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("arrow_forward");

        if name.is_empty() {
            continue;
        }

        // Shortcut badge
        let shortcut_html = if shortcut.is_empty() {
            String::new()
        } else {
            format!(
                r#"<span style="background:#e5e5e5;color:#525252;font-size:11px;font-family:'JetBrains Mono',monospace;padding:2px 8px;border-radius:4px">{}</span>"#,
                shortcut
            )
        };

        items_html.push_str(&format!(
            r#"<a href="{link}" data-cmd-item="{name}" data-cmd-idx="{idx}" onclick="document.getElementById('{pid}').classList.add('hidden')" style="display:flex;align-items:center;justify-content:space-between;padding:10px 16px;text-decoration:none;color:inherit;border-radius:8px;transition:background 0.1s" onmouseover="cronusCmdSelect({idx})" class="cmd-item">
  <span style="display:flex;align-items:center;gap:12px">
    <span class="material-symbols-outlined" style="font-size:18px;color:#a3a3a3">{icon}</span>
    <span style="font-size:14px;font-weight:500">{name}</span>
  </span>
  {shortcut_html}
</a>"#,
            link = link, name = name, idx = i, pid = palette_id,
            icon = icon, shortcut_html = shortcut_html,
        ));

        // Register keyboard shortcut bindings
        if !shortcut.is_empty() {
            // Parse shortcut: "⌘B" -> metaKey + 'b', "⌘N" -> metaKey + 'n'
            let key_char = shortcut.chars().last().unwrap_or(' ').to_lowercase().next().unwrap_or(' ');
            if key_char.is_alphanumeric() || key_char == ',' {
                let key_str = if key_char == ',' { "," } else { &key_char.to_string() };
                shortcut_bindings.push_str(&format!(
                    r#"if((e.metaKey||e.ctrlKey)&&e.key==='{key}'){{e.preventDefault();window.location.href='{link}';}}"#,
                    key = key_str, link = link,
                ));
            }
        }
    }

    format!(
        r##"<div id="{pid}" class="hidden" style="position:fixed;inset:0;z-index:200;background:rgba(0,0,0,0.5);backdrop-filter:blur(8px);display:flex;justify-content:center;padding-top:20vh" onclick="if(event.target===this)this.classList.add('hidden')">
  <div style="background:#fff;border-radius:16px;box-shadow:0 24px 48px rgba(0,0,0,0.2);width:100%;max-width:520px;height:fit-content;overflow:hidden;animation:scaleIn 0.2s cubic-bezier(0.16,1,0.3,1)">
    <div style="padding:16px;border-bottom:1px solid #f5f5f5">
      <input id="{pid}-input" type="text" placeholder="Type a command..." oninput="cronusFilterCommands(this)" onkeydown="cronusCmdKeyNav(event)" style="width:100%;border:none;outline:none;font-size:16px;font-family:Inter,sans-serif;background:transparent;box-sizing:border-box" autocomplete="off">
    </div>
    <div id="{pid}-list" style="padding:8px;max-height:320px;overflow-y:auto">
      {items}
    </div>
    <div style="padding:8px 16px;border-top:1px solid #f5f5f5;display:flex;gap:16px;justify-content:center">
      <span style="font-size:11px;color:#a3a3a3;display:flex;align-items:center;gap:4px"><span style="background:#e5e5e5;padding:1px 6px;border-radius:3px;font-family:monospace;font-size:10px">↑↓</span> navigate</span>
      <span style="font-size:11px;color:#a3a3a3;display:flex;align-items:center;gap:4px"><span style="background:#e5e5e5;padding:1px 6px;border-radius:3px;font-family:monospace;font-size:10px">↵</span> select</span>
      <span style="font-size:11px;color:#a3a3a3;display:flex;align-items:center;gap:4px"><span style="background:#e5e5e5;padding:1px 6px;border-radius:3px;font-family:monospace;font-size:10px">esc</span> close</span>
    </div>
  </div>
</div>
<style>
#{pid} .cmd-item{{cursor:pointer}}
#{pid} .cmd-item.active,#{pid} .cmd-item:hover{{background:#f5f5f5}}
#{pid}.hidden{{display:none!important}}
</style>
<script>
(function(){{
  var pid='{pid}';
  var selectedIdx=-1;

  // Cmd+K toggle
  document.addEventListener('keydown',function(e){{
    if((e.metaKey||e.ctrlKey)&&e.key==='k'){{
      e.preventDefault();
      var el=document.getElementById(pid);
      if(el.classList.contains('hidden')){{
        el.classList.remove('hidden');
        el.style.display='flex';
        var inp=document.getElementById(pid+'-input');
        inp.value='';
        inp.focus();
        cronusFilterCommands(inp);
        selectedIdx=-1;
        cronusCmdHighlight();
      }}else{{
        el.classList.add('hidden');
      }}
    }}
    if(e.key==='Escape'){{
      var el=document.getElementById(pid);
      if(el)el.classList.add('hidden');
    }}
    // Per-item shortcuts
    {shortcut_bindings}
  }});

  // Fuzzy filter
  window.cronusFilterCommands=function(input){{
    var q=input.value.toLowerCase();
    var items=document.querySelectorAll('[data-cmd-item]');
    var first=-1;
    items.forEach(function(el,i){{
      var match=el.getAttribute('data-cmd-item').toLowerCase().indexOf(q)!==-1;
      el.style.display=match?'':'none';
      if(match&&first===-1)first=i;
    }});
    selectedIdx=first;
    cronusCmdHighlight();
  }};

  // Keyboard nav
  window.cronusCmdKeyNav=function(e){{
    var items=Array.from(document.querySelectorAll('[data-cmd-item]')).filter(function(el){{return el.style.display!=='none'}});
    if(!items.length)return;
    if(e.key==='ArrowDown'){{
      e.preventDefault();
      selectedIdx=Math.min(selectedIdx+1,items.length-1);
      cronusCmdHighlight();
    }}else if(e.key==='ArrowUp'){{
      e.preventDefault();
      selectedIdx=Math.max(selectedIdx-1,0);
      cronusCmdHighlight();
    }}else if(e.key==='Enter'){{
      e.preventDefault();
      if(selectedIdx>=0&&selectedIdx<items.length){{
        items[selectedIdx].click();
      }}
    }}
  }};

  // Highlight selected
  window.cronusCmdSelect=function(idx){{selectedIdx=idx;cronusCmdHighlight()}};
  function cronusCmdHighlight(){{
    var items=Array.from(document.querySelectorAll('[data-cmd-item]')).filter(function(el){{return el.style.display!=='none'}});
    items.forEach(function(el,i){{
      el.classList.toggle('active',i===selectedIdx);
    }});
  }}
}})();
</script>"##,
        pid = palette_id, items = items_html, shortcut_bindings = shortcut_bindings,
    )
}

/// Renders just the trigger button (pill with ⌘K) for injection into the topbar.
pub fn render_command_trigger() -> String {
    r#"<button onclick="var el=document.getElementById('command-palette');el.classList.remove('hidden');el.style.display='flex';document.getElementById('command-palette-input').value='';document.getElementById('command-palette-input').focus()" style="display:inline-flex;align-items:center;gap:6px;padding:4px 12px;background:#f5f5f5;border:1px solid #e5e5e5;border-radius:999px;font-size:12px;color:#737373;cursor:pointer;font-family:Inter,sans-serif;transition:all 0.15s" onmouseover="this.style.background='#e5e5e5'" onmouseout="this.style.background='#f5f5f5'"><span class="material-symbols-outlined" style="font-size:14px">search</span><span style="font-family:'JetBrains Mono',monospace;font-size:11px">⌘K</span></button>"#.to_string()
}
