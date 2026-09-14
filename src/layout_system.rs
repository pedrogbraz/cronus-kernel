use crate::parser::SectionNode;
use std::sync::atomic::{AtomicU32, Ordering};

static LAYOUT_COUNTER: AtomicU32 = AtomicU32::new(0);

fn next_id(prefix: &str) -> String {
    let n = LAYOUT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}", prefix, n)
}

/// Renders an app shell layout (sidebar + topbar + content) from a "layout" section.
///
/// Items: "sidebar" (width, collapsible, component), "topbar" (height, component), "content" (role).
pub fn render_layout_section(section: &SectionNode) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    let layout_id = next_id("layout");

    // Extract sidebar, topbar, content from items
    let mut sidebar_width = "260px";
    let mut sidebar_collapsible = false;
    let mut sidebar_bg = "#fff";
    let mut topbar_height = "56px";
    let mut topbar_bg = "#fff";
    let mut has_sidebar = false;
    let mut has_topbar = false;

    for item in &section.items {
        let title = item.get("title").map(|s| s.as_str()).unwrap_or("");
        match title {
            "sidebar" => {
                has_sidebar = true;
                if let Some(w) = item.get("width") {
                    sidebar_width = w;
                }
                if item
                    .get("collapsible")
                    .map(|s| s == "true")
                    .unwrap_or(false)
                {
                    sidebar_collapsible = true;
                }
                if let Some(bg) = item.get("background") {
                    sidebar_bg = bg;
                }
            }
            "topbar" => {
                has_topbar = true;
                if let Some(h) = item.get("height") {
                    topbar_height = h;
                }
                if let Some(bg) = item.get("background") {
                    topbar_bg = bg;
                }
            }
            _ => {} // "content" or others — handled by main area
        }
    }

    // Sidebar HTML
    let sidebar_html = if has_sidebar {
        format!(
            r#"<aside id="cronus-sidebar" style="width:{width};min-height:100vh;background:{bg};border-right:1px solid #f0f0f0;transition:transform 0.3s cubic-bezier(0.16,1,0.3,1);overflow-y:auto;flex-shrink:0;z-index:40">
  <div id="cronus-sidebar-content" style="padding:16px"></div>
</aside>"#,
            width = sidebar_width,
            bg = sidebar_bg,
        )
    } else {
        String::new()
    };

    // Hamburger button (only if sidebar exists)
    let hamburger = if has_sidebar {
        r#"<button class="cronus-hamburger" onclick="cronusToggleSidebar()" style="display:none;align-items:center;justify-content:center;background:none;border:none;cursor:pointer;padding:8px;border-radius:8px;margin-right:8px" onmouseover="this.style.background='#f5f5f5'" onmouseout="this.style.background='none'">
        <span class="material-symbols-outlined" style="font-size:22px">menu</span>
      </button>"#
    } else {
        ""
    };

    // Topbar HTML
    let topbar_html = if has_topbar {
        format!(
            r#"<header id="cronus-topbar" style="height:{height};background:{bg};border-bottom:1px solid #f0f0f0;display:flex;align-items:center;padding:0 24px;flex-shrink:0">
      {hamburger}
      <div id="cronus-topbar-content" style="flex:1;display:flex;align-items:center;justify-content:space-between"></div>
    </header>"#,
            height = topbar_height,
            bg = topbar_bg,
            hamburger = hamburger,
        )
    } else {
        String::new()
    };

    // Collapse button inside sidebar
    let collapse_btn = if sidebar_collapsible {
        r#"<button id="cronus-collapse-btn" onclick="cronusCollapseSidebar()" style="position:absolute;top:12px;right:-12px;width:24px;height:24px;border-radius:50%;background:#fff;border:1px solid #e5e5e5;display:flex;align-items:center;justify-content:center;cursor:pointer;z-index:41;box-shadow:0 2px 4px rgba(0,0,0,0.08)">
        <span class="material-symbols-outlined" style="font-size:14px">chevron_left</span>
      </button>"#
    } else {
        ""
    };

    // Sidebar wrapper with relative for collapse button
    let sidebar_outer = if has_sidebar {
        format!(
            r#"<div style="position:relative;flex-shrink:0">{sidebar}{collapse}</div>"#,
            sidebar = sidebar_html,
            collapse = collapse_btn,
        )
    } else {
        String::new()
    };

    // JS for sidebar toggle and collapse
    let js = format!(
        r##"<script{script_nonce}>
(function(){{
  window.cronusToggleSidebar=function(){{
    var sb=document.getElementById('cronus-sidebar');
    if(!sb)return;
    sb.classList.toggle('open');
    var overlay=document.querySelector('.cronus-overlay');
    if(!overlay){{
      overlay=document.createElement('div');
      overlay.className='cronus-overlay';
      overlay.style.cssText='display:none;position:fixed;inset:0;background:rgba(0,0,0,0.3);z-index:39;transition:opacity 0.3s';
      overlay.onclick=function(){{cronusToggleSidebar()}};
      document.body.appendChild(overlay);
    }}
    overlay.style.display=sb.classList.contains('open')?'block':'none';
  }};
  {collapse_js}
}})();
</script>"##,
        collapse_js = if sidebar_collapsible {
            r#"var collapsed=false;
  window.cronusCollapseSidebar=function(){
    var sb=document.getElementById('cronus-sidebar');
    var btn=document.getElementById('cronus-collapse-btn');
    if(!sb)return;
    collapsed=!collapsed;
    sb.style.width=collapsed?'64px':'260px';
    if(btn){
      var icon=btn.querySelector('.material-symbols-outlined');
      if(icon)icon.textContent=collapsed?'chevron_right':'chevron_left';
    }
    var items=sb.querySelectorAll('.sidebar-label');
    items.forEach(function(el){el.style.display=collapsed?'none':''});
  };"#
        } else {
            ""
        },
    );

    format!(
        r##"<div class="cronus-layout" id="{id}" style="display:flex;min-height:100vh;background:#fafafa">
  {sidebar}
  <div style="flex:1;display:flex;flex-direction:column;min-width:0">
    {topbar}
    <main id="cronus-content" style="flex:1;padding:24px;overflow-y:auto">
    </main>
  </div>
</div>
{js}"##,
        id = layout_id,
        sidebar = sidebar_outer,
        topbar = topbar_html,
        js = js,
    )
}

/// Renders a CSS grid wrapper for column-based layouts.
///
/// Items define named slots with optional `span` (fr units).
/// Subsequent sections in the page are auto-placed into the grid by the browser.
pub fn render_column_layout(section: &SectionNode) -> String {
    let cols = section
        .config
        .get("cols")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(2);
    let gap = section
        .config
        .get("gap")
        .map(|s| s.as_str())
        .unwrap_or("24px");
    let id = next_id("grid");

    // Build grid-template-columns from items
    let mut template_parts = Vec::new();
    for item in &section.items {
        let span = item
            .get("span")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(1);
        template_parts.push(format!("{}fr", span));
    }
    let template = if template_parts.is_empty() {
        format!("repeat({}, 1fr)", cols)
    } else {
        template_parts.join(" ")
    };

    // Responsive: stack on mobile
    format!(
        r##"<div id="{id}" class="cronus-grid-layout" style="display:grid;grid-template-columns:{template};gap:{gap}">
<style>
@media(max-width:768px){{
  #{id}{{grid-template-columns:1fr!important}}
}}
</style>"##,
        id = id,
        template = template,
        gap = gap,
    )
}

/// Closes a grid layout div opened by `render_column_layout`.
pub fn render_column_layout_end() -> String {
    "</div>".to_string()
}
