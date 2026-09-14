#![allow(dead_code, unused_imports)]
use crate::parser::SectionNode;
use std::collections::HashMap;
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
    let style = section
        .config
        .get("style")
        .map(|s| s.as_str())
        .unwrap_or("app-shell");
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

/// Wraps any section HTML with responsive CSS media queries based on config keys.
///
/// Reads: cols-sm, cols-md, cols-lg, cols-xl, responsive (scroll/stack/hide), hide-on (mobile/tablet/desktop).
pub fn render_responsive_wrapper(section_html: &str, config: &HashMap<String, String>) -> String {
    let section_id = next_id("rs");

    let cols_sm = config.get("cols-sm").or(config.get("cols_sm"));
    let cols_md = config.get("cols-md").or(config.get("cols_md"));
    let cols_lg = config.get("cols-lg").or(config.get("cols_lg"));
    let cols_xl = config.get("cols-xl").or(config.get("cols_xl"));
    let responsive = config.get("responsive").map(|s| s.as_str());
    let hide_on = config
        .get("hide-on")
        .or(config.get("hide_on"))
        .map(|s| s.as_str());

    // If no responsive config at all, return as-is
    let has_responsive = cols_sm.is_some()
        || cols_md.is_some()
        || cols_lg.is_some()
        || cols_xl.is_some()
        || responsive.is_some()
        || hide_on.is_some();

    if !has_responsive {
        return section_html.to_string();
    }

    let mut css = String::new();

    // Column breakpoints
    if let Some(c) = cols_sm {
        css.push_str(&format!(
            "@media(max-width:640px){{.cronus-section-{id}{{grid-template-columns:repeat({c},1fr)!important}}}}\n",
            id = section_id, c = c,
        ));
    }
    if let Some(c) = cols_md {
        css.push_str(&format!(
            "@media(max-width:768px){{.cronus-section-{id}{{grid-template-columns:repeat({c},1fr)!important}}}}\n",
            id = section_id, c = c,
        ));
    }
    if let Some(c) = cols_lg {
        css.push_str(&format!(
            "@media(max-width:1024px){{.cronus-section-{id}{{grid-template-columns:repeat({c},1fr)!important}}}}\n",
            id = section_id, c = c,
        ));
    }
    if let Some(c) = cols_xl {
        css.push_str(&format!(
            "@media(max-width:1280px){{.cronus-section-{id}{{grid-template-columns:repeat({c},1fr)!important}}}}\n",
            id = section_id, c = c,
        ));
    }

    // Responsive mode
    match responsive {
        Some("scroll") => {
            css.push_str(&format!(
                "@media(max-width:768px){{.cronus-section-{id}{{overflow-x:auto;white-space:nowrap}}.cronus-section-{id} table{{min-width:600px}}}}\n",
                id = section_id,
            ));
        }
        Some("stack") => {
            css.push_str(&format!(
                "@media(max-width:768px){{.cronus-section-{id}{{display:flex!important;flex-direction:column!important;gap:16px}}}}\n",
                id = section_id,
            ));
        }
        Some("hide") => {
            css.push_str(&format!(
                "@media(max-width:768px){{.cronus-section-{id}{{display:none!important}}}}\n",
                id = section_id,
            ));
        }
        _ => {}
    }

    // Hide on specific breakpoints
    match hide_on {
        Some("mobile") => {
            css.push_str(&format!(
                "@media(max-width:768px){{.cronus-section-{id}{{display:none!important}}}}\n",
                id = section_id,
            ));
        }
        Some("tablet") => {
            css.push_str(&format!(
                "@media(min-width:769px) and (max-width:1024px){{.cronus-section-{id}{{display:none!important}}}}\n",
                id = section_id,
            ));
        }
        Some("desktop") => {
            css.push_str(&format!(
                "@media(min-width:1025px){{.cronus-section-{id}{{display:none!important}}}}\n",
                id = section_id,
            ));
        }
        _ => {}
    }

    // Wrap with class and inject style
    let wrapped = section_html.replacen(
        "style=\"",
        &format!("class=\"cronus-section-{}\" style=\"", section_id),
        1,
    );

    // If we couldn't inject via style=, wrap with a div
    let final_html = if wrapped == section_html {
        format!(
            r#"<div class="cronus-section-{id}">{html}</div>"#,
            id = section_id,
            html = section_html,
        )
    } else {
        wrapped
    };

    format!(
        "<style>\n{css}</style>\n{html}",
        css = css,
        html = final_html
    )
}

/// Returns the viewport meta tag and base responsive CSS for mobile sidebar + hamburger.
pub fn render_responsive_meta() -> String {
    r##"<meta name="viewport" content="width=device-width, initial-scale=1">
<style>
/* CRONUS Responsive Base */
@media (max-width: 768px) {
  #cronus-sidebar {
    transform: translateX(-100%);
    position: fixed !important;
    left: 0;
    top: 0;
    height: 100vh;
    z-index: 40;
    box-shadow: 4px 0 24px rgba(0,0,0,0.1);
  }
  #cronus-sidebar.open {
    transform: translateX(0);
  }
  .cronus-hamburger {
    display: flex !important;
  }
  #cronus-collapse-btn {
    display: none !important;
  }
  .cronus-layout {
    flex-direction: column;
  }
}
@media (max-width: 640px) {
  #cronus-content {
    padding: 16px !important;
  }
  .cronus-layout h1, .cronus-layout h2 {
    font-size: 1.25rem !important;
  }
}
@media print {
  #cronus-sidebar, #cronus-topbar, .cronus-hamburger, .cronus-overlay {
    display: none !important;
  }
  #cronus-content {
    padding: 0 !important;
  }
}
</style>"##
        .to_string()
}
