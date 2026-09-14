//! Auto-generated documentation pages extracted from main.rs.

use crate::cli::objective_kernel::reconcile_field_type_str;
use crate::graph;
use crate::parser::HttpMethod;

use super::state::AppState;

/// Render the auto-generated API documentation page.
pub(crate) fn render_auto_docs(state: &AppState) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    let app_name = &state.app.name;
    let port = state.app.port;
    let t = crate::theme::get();

    // --- Sidebar nav items ---
    let mut nav_html = String::new();
    nav_html.push_str(r##"<a class="flex items-center gap-3 py-2 px-8 font-['Space_Grotesk'] text-sm uppercase tracking-widest text-white font-bold border-l-2 border-[var(--secondary)] doc-nav" data-scroll="overview"><span class="material-symbols-outlined text-lg">menu_book</span>Overview</a>"##);
    nav_html.push_str(r##"<a class="flex items-center gap-3 py-2 px-8 font-['Space_Grotesk'] text-sm uppercase tracking-widest text-[#ababab] hover:text-white transition-all doc-nav" data-scroll="entities"><span class="material-symbols-outlined text-lg">database</span>Entities</a>"##);
    nav_html.push_str(r##"<a class="flex items-center gap-3 py-2 px-8 font-['Space_Grotesk'] text-sm uppercase tracking-widest text-[#ababab] hover:text-white transition-all doc-nav" data-scroll="api"><span class="material-symbols-outlined text-lg">api</span>API Reference</a>"##);
    nav_html.push_str(r##"<a class="flex items-center gap-3 py-2 px-8 font-['Space_Grotesk'] text-sm uppercase tracking-widest text-[#ababab] hover:text-white transition-all doc-nav" data-scroll="pages"><span class="material-symbols-outlined text-lg">web</span>Pages</a>"##);
    nav_html.push_str(r##"<a class="flex items-center gap-3 py-2 px-8 font-['Space_Grotesk'] text-sm uppercase tracking-widest text-[#ababab] hover:text-white transition-all doc-nav" data-scroll="webhooks"><span class="material-symbols-outlined text-lg">webhook</span>Webhooks</a>"##);
    nav_html.push_str(r##"<a class="flex items-center gap-3 py-2 px-8 font-['Space_Grotesk'] text-sm uppercase tracking-widest text-[#ababab] hover:text-white transition-all doc-nav" data-scroll="audit"><span class="material-symbols-outlined text-lg">policy</span>Audit Trail</a>"##);

    // --- TOC (right sidebar) ---
    let mut toc_html = String::new();
    toc_html.push_str(r##"<li><a class="text-sm text-[var(--primary)] font-medium flex items-center gap-2 doc-nav" data-scroll="overview" style="cursor:pointer"><div class="w-1.5 h-1.5 rounded-full bg-[var(--primary)]" style="box-shadow:0 0 8px rgba(135,173,255,0.8)"></div>Overview</a></li>"##);
    toc_html.push_str(r##"<li><a class="text-sm text-[#ababab] hover:text-white transition-colors flex items-center gap-2 doc-nav" data-scroll="entities" style="cursor:pointer"><div class="w-1 h-1 rounded-full bg-[#484848]"></div>Entities</a></li>"##);
    toc_html.push_str(r##"<li><a class="text-sm text-[#ababab] hover:text-white transition-colors flex items-center gap-2 doc-nav" data-scroll="api" style="cursor:pointer"><div class="w-1 h-1 rounded-full bg-[#484848]"></div>API Reference</a></li>"##);
    toc_html.push_str(r##"<li><a class="text-sm text-[#ababab] hover:text-white transition-colors flex items-center gap-2 doc-nav" data-scroll="pages" style="cursor:pointer"><div class="w-1 h-1 rounded-full bg-[#484848]"></div>Pages</a></li>"##);
    toc_html.push_str(r##"<li><a class="text-sm text-[#ababab] hover:text-white transition-colors flex items-center gap-2 doc-nav" data-scroll="webhooks" style="cursor:pointer"><div class="w-1 h-1 rounded-full bg-[#484848]"></div>Webhooks</a></li>"##);
    toc_html.push_str(r##"<li><a class="text-sm text-[#ababab] hover:text-white transition-colors flex items-center gap-2 doc-nav" data-scroll="audit" style="cursor:pointer"><div class="w-1 h-1 rounded-full bg-[#484848]"></div>Audit Trail</a></li>"##);

    // --- Entities section ---
    let mut entities_html = String::new();
    for (_i, entity) in state.entities.iter().enumerate() {
        if entity.name.starts_with('_') {
            continue;
        }
        let shared_badge = if entity.shared {
            format!(
                r##" <span style="font-size:10px;padding:2px 8px;border-radius:4px;background:rgba(129,236,255,0.1);color:{tertiary};margin-left:8px">shared</span>"##,
                tertiary = t.tertiary
            )
        } else {
            String::new()
        };

        let mut fields_html = String::new();
        for field in &entity.fields {
            let type_name = reconcile_field_type_str(&field.field_type);
            let mut badges = String::new();
            if field.required {
                badges.push_str(&format!(
                    r##"<span style="color:{};font-size:10px;margin-left:8px">required</span>"##,
                    t.primary
                ));
            }
            if field.unique {
                badges.push_str(&format!(
                    r##"<span style="color:{};font-size:10px;margin-left:8px">unique</span>"##,
                    t.secondary
                ));
            }
            if field.sensitive {
                badges.push_str(&format!(
                    r##"<span style="color:{};font-size:10px;margin-left:8px">sensitive</span>"##,
                    t.error
                ));
            }
            if let Some(ref vals) = field.enum_values {
                let joined = vals.join(" | ");
                badges.push_str(&format!(
                    r##"<span style="color:{};font-size:10px;margin-left:8px">[{}]</span>"##,
                    t.on_surface_variant, joined
                ));
            }
            if let Some(min_val) = field.min {
                badges.push_str(&format!(
                    r##"<span style="color:#10b981;font-size:10px;margin-left:8px">min:{}</span>"##,
                    min_val
                ));
            }
            if let Some(max_val) = field.max {
                badges.push_str(&format!(
                    r##"<span style="color:#10b981;font-size:10px;margin-left:8px">max:{}</span>"##,
                    max_val
                ));
            }
            if let Some(min_len) = field.min_length {
                badges.push_str(&format!(
                    r##"<span style="color:#10b981;font-size:10px;margin-left:8px">min:{}</span>"##,
                    min_len
                ));
            }
            if let Some(max_len) = field.max_length {
                badges.push_str(&format!(
                    r##"<span style="color:#10b981;font-size:10px;margin-left:8px">max:{}</span>"##,
                    max_len
                ));
            }
            if let Some(ref pat) = field.pattern {
                badges.push_str(&format!(r##"<span style="color:#10b981;font-size:10px;margin-left:8px">match:{}</span>"##, pat));
            }
            let field_doc_html = if let Some(ref doc) = field.doc {
                let mut parts = Vec::new();
                if !doc.summary.is_empty() {
                    parts.push(format!(
                        r##"<span style="color:#757575;font-size:11px;margin-left:8px">{}</span>"##,
                        doc.summary
                    ));
                }
                for tag in &doc.tags {
                    if tag.name == "ai" {
                        continue;
                    }
                    let tag_color = match tag.name.as_str() {
                        "example" => "#10b981",
                        "business" => "#f59e0b",
                        "deprecated" => "#ef4444",
                        _ => "#484848",
                    };
                    parts.push(format!(
                        r##"<span style="color:{};font-size:10px;margin-left:8px">@{} {}</span>"##,
                        tag_color, tag.name, tag.value
                    ));
                }
                parts.join("")
            } else {
                String::new()
            };

            fields_html.push_str(&format!(
                r##"<div style="padding:8px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;justify-content:space-between;align-items:center"><div style="display:flex;align-items:center;gap:8px"><span style="color:{on_surface};font-family:monospace;font-size:13px">{}</span><span style="color:{primary};font-size:11px;font-family:monospace">{}</span></div><div>{}</div></div>{}</div>"##,
                field.name, type_name, badges, field_doc_html, on_surface = t.on_surface, primary = t.primary
            ));
        }

        let entity_doc_html = if let Some(ref doc) = entity.doc {
            let mut html = String::new();
            if !doc.summary.is_empty() {
                html.push_str(&format!(
                    r##"<p style="color:{};font-size:13px;margin:4px 0 0">{}</p>"##,
                    t.on_surface_variant, doc.summary
                ));
            }
            if !doc.description.is_empty() {
                html.push_str(&format!(
                    r##"<p style="color:#757575;font-size:12px;margin:4px 0 0">{}</p>"##,
                    doc.description
                ));
            }
            let tags_html: String = doc.tags.iter().filter(|tg| tg.name != "ai").map(|tg| {
                let color = match tg.name.as_str() {
                    "owner" => t.primary.as_str(),
                    "lifecycle" => t.tertiary.as_str(),
                    "since" => "#757575",
                    _ => "#484848",
                };
                format!(r##"<span style="font-size:10px;padding:2px 8px;border-radius:4px;background:rgba(255,255,255,0.03);color:{};margin-right:6px">@{} {}</span>"##, color, tg.name, tg.value)
            }).collect();
            if !tags_html.is_empty() {
                html.push_str(&format!(
                    r##"<div style="margin-top:8px;display:flex;flex-wrap:wrap;gap:4px">{}</div>"##,
                    tags_html
                ));
            }
            html
        } else {
            String::new()
        };

        // Build transition diagram HTML if entity has transitions
        let transitions_html = if !entity.transitions.is_empty() {
            let mut html = String::new();
            html.push_str(&format!(r##"<div style="margin-top:16px;padding-top:16px;border-top:1px solid rgba(255,255,255,0.05)"><div style="display:flex;align-items:center;gap:6px;margin-bottom:12px"><span class="material-symbols-outlined" style="font-size:16px;color:{secondary}">swap_horiz</span><span style="font-family:Space Grotesk,sans-serif;font-size:14px;font-weight:600;color:{secondary}">State Transitions</span></div>"##, secondary = t.secondary));
            for trans in &entity.transitions {
                html.push_str(&format!(
                    r##"<div style="margin-bottom:8px"><span style="color:{primary};font-size:12px;font-family:monospace">{field}</span></div>"##,
                    primary = t.primary, field = trans.field
                ));
                html.push_str(
                    r##"<div style="display:flex;flex-wrap:wrap;gap:8px;margin-bottom:12px">"##,
                );
                for rule in &trans.rules {
                    let targets = rule.to.join(", ");
                    html.push_str(&format!(
                        r##"<div style="background:rgba(210,119,255,0.06);border:1px solid rgba(210,119,255,0.15);border-radius:8px;padding:8px 12px;font-size:12px"><span style="color:#e2e2e2;font-family:monospace">{from}</span> <span style="color:#757575">-></span> <span style="color:#81ecff;font-family:monospace">{to}</span></div>"##,
                        from = rule.from,
                        to = targets,
                    ));
                }
                html.push_str("</div>");
            }
            html.push_str("</div>");
            html
        } else {
            String::new()
        };

        entities_html.push_str(&format!(
            r##"<div style="background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(135,173,255,0.2);padding:24px;margin-bottom:16px"><div style="margin-bottom:16px"><div style="display:flex;align-items:center"><span style="font-family:Space Grotesk,sans-serif;font-size:18px;font-weight:700;color:#fff">{name}</span>{shared}</div>{doc}</div><div>{fields}</div>{transitions}</div>"##,
            name = entity.name,
            shared = shared_badge,
            doc = entity_doc_html,
            fields = fields_html,
            transitions = transitions_html,
        ));
    }

    // --- API section ---
    let mut api_html = String::new();
    for api in &state.apis {
        let mut routes_html = String::new();
        for route in &api.routes {
            let method_str = match &route.method {
                HttpMethod::GET => "GET",
                HttpMethod::POST => "POST",
                HttpMethod::PATCH => "PATCH",
                HttpMethod::PUT => "PUT",
                HttpMethod::DELETE => "DELETE",
            };
            let method_color = match method_str {
                "GET" => "#10b981",
                "POST" => "#87adff",
                "PATCH" | "PUT" => "#f59e0b",
                "DELETE" => "#ef4444",
                _ => "#ababab",
            };
            let route_doc_html = if let Some(ref doc) = route.doc {
                let mut parts = Vec::new();
                if !doc.summary.is_empty() {
                    parts.push(format!(r##"<div style="color:#ababab;font-size:12px;margin:4px 0 0 72px">{}</div>"##, doc.summary));
                }
                for tag in &doc.tags {
                    if tag.name == "ai" {
                        continue;
                    }
                    let tag_color = match tag.name.as_str() {
                        "param" => "#87adff",
                        "returns" => "#10b981",
                        "deprecated" => "#ef4444",
                        "example" => "#f59e0b",
                        _ => "#757575",
                    };
                    parts.push(format!(r##"<div style="color:{};font-size:11px;margin:2px 0 0 72px">@{} {}</div>"##, tag_color, tag.name, tag.value));
                }
                parts.join("")
            } else {
                String::new()
            };
            routes_html.push_str(&format!(
                r##"<div style="padding:10px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;align-items:center;gap:12px"><span style="font-family:monospace;font-size:11px;font-weight:700;color:{color};min-width:60px">{method}</span><span style="font-family:monospace;font-size:13px;color:#e2e2e2">{path}</span><span style="font-size:11px;color:#ababab;margin-left:auto">{name}</span></div>{route_doc}</div>"##,
                color = method_color,
                method = method_str,
                path = route.path,
                name = route.name,
                route_doc = route_doc_html,
            ));
        }
        api_html.push_str(&format!(
            r##"<div style="background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(135,173,255,0.2);padding:24px;margin-bottom:16px"><h3 style="font-family:Space Grotesk,sans-serif;font-size:16px;font-weight:700;color:#fff;margin:0 0 16px"><span style="font-family:monospace;color:var(--primary)">{base}</span></h3>{routes}</div>"##,
            base = api.prefix,
            routes = routes_html,
        ));
    }

    // --- Pages section ---
    let mut pages_html = String::new();
    for page in &state.pages {
        let route = &page.route;
        let title = page.title.as_deref().unwrap_or("-");
        let _ptype = &page.page_type;
        let section_count = page.sections.len();
        let auth = if page.requires.is_some() {
            "auth required"
        } else {
            "public"
        };
        let auth_color = if page.requires.is_some() {
            "#f59e0b"
        } else {
            "#10b981"
        };
        let page_doc_html = if let Some(ref doc) = page.doc {
            let mut parts = Vec::new();
            if !doc.summary.is_empty() {
                parts.push(format!(
                    r##"<div style="color:#ababab;font-size:12px;margin:4px 0 0 0">{}</div>"##,
                    doc.summary
                ));
            }
            if !doc.description.is_empty() {
                parts.push(format!(
                    r##"<div style="color:#757575;font-size:11px;margin:2px 0 0 0">{}</div>"##,
                    doc.description
                ));
            }
            for tag in &doc.tags {
                if tag.name == "ai" {
                    continue;
                }
                let tag_color = match tag.name.as_str() {
                    "requires" => "#f59e0b",
                    "layout" => "#87adff",
                    "since" => "#757575",
                    _ => "#484848",
                };
                parts.push(format!(
                    r##"<div style="color:{};font-size:10px;margin:2px 0 0 0">@{} {}</div>"##,
                    tag_color, tag.name, tag.value
                ));
            }
            parts.join("")
        } else {
            String::new()
        };
        pages_html.push_str(&format!(
            r##"<div style="padding:12px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;justify-content:space-between;align-items:center"><div style="display:flex;align-items:center;gap:12px"><span style="font-family:monospace;font-size:14px;color:var(--primary)">{route}</span><span style="font-size:12px;color:#ababab">{title}</span></div><div style="display:flex;align-items:center;gap:12px"><span style="font-size:10px;color:#ababab">{sections} sections</span><span style="font-size:10px;padding:2px 8px;border-radius:4px;background:rgba(255,255,255,0.03);color:{auth_color}">{auth}</span></div></div>{page_doc}</div>"##,
            route = route, title = title, sections = section_count, auth = auth, auth_color = auth_color, page_doc = page_doc_html,
        ));
    }

    // --- Webhooks section ---
    let mut webhooks_html = String::new();
    for wh in &state.webhooks {
        let mut hooks_html = String::new();
        for hook in &wh.hooks {
            let event_color = match hook.event.as_str() {
                "create" => "#10b981",
                "update" => "#f59e0b",
                "delete" => "#ef4444",
                _ => "#ababab",
            };
            hooks_html.push_str(&format!(
                r##"<div style="display:flex;align-items:center;gap:12px;padding:10px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><span style="font-family:monospace;font-size:11px;font-weight:700;color:{color}">on {event}</span><span style="font-size:11px;color:#ababab">→</span><span style="font-family:monospace;font-size:11px;color:var(--primary)">{method}</span><span style="font-family:monospace;font-size:12px;color:#e2e2e2;word-break:break-all">{url}</span></div>"##,
                color = event_color,
                event = hook.event,
                method = hook.method,
                url = hook.url,
            ));
        }
        webhooks_html.push_str(&format!(
            r##"<div style="background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(135,173,255,0.2);padding:24px;margin-bottom:16px"><h3 style="font-family:Space Grotesk,sans-serif;font-size:16px;font-weight:700;color:#fff;margin:0 0 16px">{entity}</h3>{hooks}</div>"##,
            entity = wh.entity,
            hooks = hooks_html,
        ));
    }

    let has_webhooks = !state.webhooks.is_empty();

    // --- .cronus source preview ---
    let mut cronus_preview = String::new();
    cronus_preview.push_str(&format!(
        r##"<span style="color:var(--secondary)">app</span> <span style="color:#10b981">"{name}"</span> {{\n  <span style="color:#ababab">stack</span> fullstack\n  <span style="color:#ababab">port</span> <span style="color:#f59e0b">{port}</span>\n  <span style="color:#ababab">database</span> sqlite <span style="color:#10b981">"./data.db"</span>\n}}"##,
        name = app_name, port = port,
    ));

    // --- App doc-comment for Overview ---
    let app_doc_html = if let Some(ref doc) = state.app.doc {
        let mut html = String::new();
        if !doc.summary.is_empty() {
            html.push_str(&format!(r##"<p style="font-size:16px;color:#ababab;line-height:1.6;margin:16px 0 0">{}</p>"##, doc.summary));
        }
        if !doc.description.is_empty() {
            html.push_str(&format!(r##"<p style="font-size:14px;color:#757575;line-height:1.6;margin:8px 0 0">{}</p>"##, doc.description));
        }
        let tags: Vec<String> = doc.tags.iter().filter(|t| t.name != "ai").map(|t| {
            let color = match t.name.as_str() {
                "version" => "#87adff",
                "author" => "#81ecff",
                "since" => "#757575",
                _ => "#484848",
            };
            format!(r##"<span style="font-size:10px;padding:2px 8px;border-radius:4px;background:rgba(255,255,255,0.03);color:{};margin-right:6px">@{} {}</span>"##, color, t.name, t.value)
        }).collect();
        if !tags.is_empty() {
            html.push_str(&format!(
                r##"<div style="margin-top:12px;display:flex;flex-wrap:wrap;gap:4px">{}</div>"##,
                tags.join("")
            ));
        }
        html
    } else {
        String::new()
    };

    // --- Full page ---
    format!(
        r##"<!DOCTYPE html>
<html lang="en"><head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width,initial-scale=1.0"/>
<title>{app_name} | Documentation</title>
<script{script_nonce} src="https://cdn.tailwindcss.com"></script>
<link href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700;900&family=Inter:wght@300;400;500;600&display=swap" rel="stylesheet"/>
<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet"/>
<style>
body {{ background:#0e0e0e; color:#fff; font-family:'Inter',sans-serif; margin:0 }}
.material-symbols-outlined {{ font-variation-settings:'FILL' 0,'wght' 300,'GRAD' 0,'opsz' 24 }}
::-webkit-scrollbar {{ width:4px }} ::-webkit-scrollbar-track {{ background:#0e0e0e }} ::-webkit-scrollbar-thumb {{ background:#262626;border-radius:10px }}
</style>
</head>
<body>
<header style="position:fixed;top:0;width:100%;z-index:50;height:64px;background:rgba(0,0,0,0.8);backdrop-filter:blur(40px);border-bottom:1px solid rgba(255,255,255,0.05);display:flex;align-items:center;justify-content:space-between;padding:0 24px;box-sizing:border-box">
  <div style="display:flex;align-items:center;gap:32px">
    <span style="font-family:Space Grotesk,sans-serif;font-size:18px;font-weight:900;color:#fff;letter-spacing:-0.03em">{app_name}_DOCS</span>
    <nav style="display:flex;gap:24px;font-family:Space Grotesk,sans-serif;font-weight:700;font-size:14px">
      <a href="/docs" style="color:#fff;border-bottom:2px solid #87adff;padding-bottom:2px;text-decoration:none">API Docs</a>
      <a href="/docs/design" style="color:#757575;text-decoration:none">Design System</a>
      <a href="/" style="color:#757575;text-decoration:none">Dashboard</a>
    </nav>
  </div>
</header>

<div style="display:flex;padding-top:64px;min-height:100vh">
  <aside style="width:280px;position:fixed;left:0;top:64px;height:calc(100vh - 64px);background:#0e0e0e;border-right:1px solid rgba(255,255,255,0.03);display:flex;flex-direction:column;padding:32px 0;overflow-y:auto">
    <div style="padding:0 32px;margin-bottom:32px">
      <div style="display:flex;align-items:center;gap:12px">
        <div style="width:32px;height:32px;border-radius:8px;background:linear-gradient(135deg,var(--primary),var(--secondary));display:flex;align-items:center;justify-content:center;color:#fff;font-weight:700;font-size:14px">N</div>
        <div><div style="font-family:Space Grotesk,sans-serif;font-weight:700;color:#fff;font-size:14px">Core Engine</div><div style="font-size:10px;color:#757575;text-transform:uppercase;letter-spacing:0.15em">v{port}</div></div>
      </div>
    </div>
    <nav style="display:flex;flex-direction:column;gap:4px">{nav}</nav>
  </aside>

  <main style="flex:1;margin-left:280px;margin-right:240px;padding:48px 64px;max-width:800px">
    <header style="margin-bottom:48px" id="overview">
      <div style="display:flex;align-items:center;gap:8px;margin-bottom:16px">
        <span style="font-family:monospace;font-size:12px;color:var(--primary);text-transform:uppercase;letter-spacing:-0.03em">Auto-Generated</span>
        <div style="width:4px;height:4px;border-radius:50%;background:#484848"></div>
        <span style="font-family:monospace;font-size:12px;color:#757575;text-transform:uppercase">{entity_count} entities · {api_count} API groups · {page_count} pages</span>
      </div>
      <h1 style="font-family:Space Grotesk,sans-serif;font-size:48px;font-weight:900;letter-spacing:-0.03em;margin:0 0 24px;background:linear-gradient(to right,#fff,#fff,#757575);-webkit-background-clip:text;-webkit-text-fill-color:transparent">Documentation</h1>
      <p style="font-size:18px;color:#757575;line-height:1.6">Complete reference for <strong style="color:#ababab">{app_name}</strong>, auto-generated from the .cronus source. Every entity, API endpoint, and page is documented here — always in sync with the code.</p>
      {app_doc}
    </header>

    <section style="margin-bottom:80px">
      <h2 style="font-family:Space Grotesk,sans-serif;font-size:24px;font-weight:700;margin-bottom:24px;display:flex;align-items:center;gap:12px"><span style="color:var(--secondary)">01.</span> App Configuration</h2>
      <div style="position:relative">
        <div style="position:absolute;inset:-4px;background:linear-gradient(to right,rgba(135,173,255,0.2),rgba(210,119,255,0.2));border-radius:16px;filter:blur(20px);opacity:0.25"></div>
        <div style="position:relative;background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(135,173,255,0.2);overflow:hidden">
          <div style="display:flex;align-items:center;justify-content:space-between;padding:8px 16px;background:rgba(38,38,38,0.5);border-bottom:1px solid rgba(255,255,255,0.03)">
            <div style="display:flex;gap:6px"><div style="width:10px;height:10px;border-radius:50%;background:rgba(239,68,68,0.2);border:1px solid rgba(239,68,68,0.4)"></div><div style="width:10px;height:10px;border-radius:50%;background:rgba(245,158,11,0.2);border:1px solid rgba(245,158,11,0.4)"></div><div style="width:10px;height:10px;border-radius:50%;background:rgba(16,185,129,0.2);border:1px solid rgba(16,185,129,0.4)"></div></div>
            <span style="font-family:monospace;font-size:10px;color:#757575">app.cronus</span>
          </div>
          <div style="padding:24px;font-family:monospace;font-size:14px;line-height:1.8;white-space:pre">{cronus_preview}</div>
        </div>
      </div>
    </section>

    <section style="margin-bottom:80px" id="entities">
      <h2 style="font-family:Space Grotesk,sans-serif;font-size:24px;font-weight:700;margin-bottom:24px;display:flex;align-items:center;gap:12px"><span style="color:var(--secondary)">02.</span> Entities</h2>
      <p style="color:#757575;margin-bottom:24px">Each entity maps to a SQLite table with auto-migration, CRUD API, and type validation.</p>
      {entities}
    </section>

    <section style="margin-bottom:80px" id="api">
      <h2 style="font-family:Space Grotesk,sans-serif;font-size:24px;font-weight:700;margin-bottom:24px;display:flex;align-items:center;gap:12px"><span style="color:var(--secondary)">03.</span> API Reference</h2>
      <p style="color:#757575;margin-bottom:24px">All endpoints are auto-generated from the <code style="background:#191919;color:var(--primary);padding:2px 6px;border-radius:4px;font-size:13px;font-family:monospace">api</code> blocks. Auth via <code style="background:#191919;color:var(--primary);padding:2px 6px;border-radius:4px;font-size:13px;font-family:monospace">Bearer</code> JWT token.</p>
      {api}
    </section>

    <section style="margin-bottom:80px" id="pages">
      <h2 style="font-family:Space Grotesk,sans-serif;font-size:24px;font-weight:700;margin-bottom:24px;display:flex;align-items:center;gap:12px"><span style="color:var(--secondary)">04.</span> Pages</h2>
      <div style="background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(135,173,255,0.2);padding:24px">
        {pages}
      </div>
    </section>

    <section style="margin-bottom:80px" id="webhooks">
      <h2 style="font-family:Space Grotesk,sans-serif;font-size:24px;font-weight:700;margin-bottom:24px;display:flex;align-items:center;gap:12px"><span style="color:var(--secondary)">05.</span> Webhooks</h2>
      {webhooks_section}
    </section>

    <section style="margin-bottom:80px" id="audit">
      <h2 style="font-family:Space Grotesk,sans-serif;font-size:24px;font-weight:700;margin-bottom:24px;display:flex;align-items:center;gap:12px"><span style="color:var(--secondary)">06.</span> Audit Trail</h2>
      <p style="color:#757575;margin-bottom:24px">Every data mutation (INSERT, UPDATE, DELETE) is recorded in an append-only <code style="background:#191919;color:var(--primary);padding:2px 6px;border-radius:4px;font-size:13px;font-family:monospace">_audit_log</code> table with cryptographic hash chaining for tamper detection.</p>

      <div style="background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(135,173,255,0.2);padding:24px;margin-bottom:16px">
        <h3 style="font-family:Space Grotesk,sans-serif;font-size:16px;font-weight:700;color:#fff;margin:0 0 16px">_audit_log Table Structure</h3>
        <div style="padding:8px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;align-items:center;gap:8px"><span style="color:#e2e2e2;font-family:monospace;font-size:13px">id</span><span style="color:var(--primary);font-size:11px;font-family:monospace">INTEGER</span><span style="color:var(--primary);font-size:10px;margin-left:8px">primary key</span></div></div>
        <div style="padding:8px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;align-items:center;gap:8px"><span style="color:#e2e2e2;font-family:monospace;font-size:13px">entity_type</span><span style="color:var(--primary);font-size:11px;font-family:monospace">TEXT</span><span style="color:#757575;font-size:11px;margin-left:8px">e.g. "Deployment", "User"</span></div></div>
        <div style="padding:8px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;align-items:center;gap:8px"><span style="color:#e2e2e2;font-family:monospace;font-size:13px">record_id</span><span style="color:var(--primary);font-size:11px;font-family:monospace">TEXT</span><span style="color:#757575;font-size:11px;margin-left:8px">UUID of the affected record</span></div></div>
        <div style="padding:8px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;align-items:center;gap:8px"><span style="color:#e2e2e2;font-family:monospace;font-size:13px">action</span><span style="color:var(--primary);font-size:11px;font-family:monospace">TEXT</span><span style="color:#757575;font-size:11px;margin-left:8px">INSERT | UPDATE | DELETE</span></div></div>
        <div style="padding:8px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;align-items:center;gap:8px"><span style="color:#e2e2e2;font-family:monospace;font-size:13px">user_id</span><span style="color:var(--primary);font-size:11px;font-family:monospace">TEXT</span><span style="color:#757575;font-size:11px;margin-left:8px">User who performed the action (or "system")</span></div></div>
        <div style="padding:8px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;align-items:center;gap:8px"><span style="color:#e2e2e2;font-family:monospace;font-size:13px">diff</span><span style="color:var(--primary);font-size:11px;font-family:monospace">TEXT</span><span style="color:#757575;font-size:11px;margin-left:8px">JSON diff of changed fields (before/after)</span></div></div>
        <div style="padding:8px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;align-items:center;gap:8px"><span style="color:#e2e2e2;font-family:monospace;font-size:13px">hash</span><span style="color:var(--primary);font-size:11px;font-family:monospace">TEXT</span><span style="color:var(--secondary);font-size:10px;margin-left:8px">SHA-256 chain hash</span></div></div>
        <div style="padding:8px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;align-items:center;gap:8px"><span style="color:#e2e2e2;font-family:monospace;font-size:13px">prev_hash</span><span style="color:var(--primary);font-size:11px;font-family:monospace">TEXT</span><span style="color:var(--secondary);font-size:10px;margin-left:8px">Hash of previous entry (blockchain-style)</span></div></div>
        <div style="padding:8px 0"><div style="display:flex;align-items:center;gap:8px"><span style="color:#e2e2e2;font-family:monospace;font-size:13px">created_at</span><span style="color:var(--primary);font-size:11px;font-family:monospace">DATETIME</span><span style="color:#757575;font-size:11px;margin-left:8px">UTC timestamp</span></div></div>
      </div>

      <div style="background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(210,119,255,0.2);padding:24px;margin-bottom:16px">
        <h3 style="font-family:Space Grotesk,sans-serif;font-size:16px;font-weight:700;color:#fff;margin:0 0 16px">Hash Chaining</h3>
        <p style="color:#ababab;font-size:13px;line-height:1.6;margin:0 0 12px">Each audit entry's <code style="background:#191919;color:var(--secondary);padding:2px 6px;border-radius:4px;font-size:12px;font-family:monospace">hash</code> is computed as <code style="background:#191919;color:var(--secondary);padding:2px 6px;border-radius:4px;font-size:12px;font-family:monospace">SHA-256(prev_hash + entity_type + record_id + action + diff + timestamp)</code>. The first entry uses a genesis hash of all zeros.</p>
        <p style="color:#ababab;font-size:13px;line-height:1.6;margin:0 0 12px">This creates a tamper-evident chain: modifying any past entry breaks the hash sequence for all subsequent entries. The <code style="background:#191919;color:var(--primary);padding:2px 6px;border-radius:4px;font-size:12px;font-family:monospace">/api/audit/trail/verify</code> endpoint walks the full chain and validates every link.</p>
        <div style="background:#0e0e0e;border-radius:8px;padding:16px;font-family:monospace;font-size:12px;line-height:1.8;color:#e2e2e2;margin-top:12px"><span style="color:#757575">// Chain structure</span><br/><span style="color:var(--secondary)">entry[0].hash</span> = SHA256(<span style="color:#484848">"0000...0000"</span> + data)<br/><span style="color:var(--secondary)">entry[1].hash</span> = SHA256(<span style="color:#10b981">entry[0].hash</span> + data)<br/><span style="color:var(--secondary)">entry[N].hash</span> = SHA256(<span style="color:#10b981">entry[N-1].hash</span> + data)</div>
      </div>

      <div style="background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(135,173,255,0.2);padding:24px;margin-bottom:16px">
        <h3 style="font-family:Space Grotesk,sans-serif;font-size:16px;font-weight:700;color:#fff;margin:0 0 16px">API Endpoints</h3>
        <div style="display:flex;align-items:center;gap:12px;padding:10px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><span style="font-family:monospace;font-size:11px;font-weight:700;color:#10b981;min-width:60px">GET</span><span style="font-family:monospace;font-size:13px;color:#e2e2e2">/api/audit/trail</span><span style="font-size:11px;color:#ababab;margin-left:auto">Returns recent audit entries with hash validation status. Query param: <code style="background:#191919;color:var(--primary);padding:1px 4px;border-radius:3px;font-size:11px">?limit=N</code></span></div>
        <div style="display:flex;align-items:center;gap:12px;padding:10px 0"><span style="font-family:monospace;font-size:11px;font-weight:700;color:#10b981;min-width:60px">GET</span><span style="font-family:monospace;font-size:13px;color:#e2e2e2">/api/audit/trail/verify</span><span style="font-size:11px;color:#ababab;margin-left:auto">Walks the full hash chain and returns <code style="background:#191919;color:var(--primary);padding:1px 4px;border-radius:3px;font-size:11px">{{"valid": true}}</code> or <code style="background:#191919;color:var(--primary);padding:1px 4px;border-radius:3px;font-size:11px">{{"valid": false, "broken_at": N}}</code></span></div>
      </div>
    </section>

    <div style="background:rgba(135,173,255,0.1);border-left:2px solid #87adff;padding:24px;border-radius:0 12px 12px 0;display:flex;gap:16px;margin-bottom:48px">
      <span class="material-symbols-outlined" style="color:var(--primary)">auto_awesome</span>
      <div><h4 style="font-weight:700;color:var(--primary);margin:0 0 4px;font-size:14px">Auto-Generated</h4><p style="font-size:13px;color:#ababab;margin:0">This documentation is generated at runtime from the parsed .cronus file. It is always in sync — modify the source and the docs update automatically.</p></div>
    </div>
  </main>

  <aside style="width:240px;position:fixed;right:0;top:64px;height:calc(100vh - 64px);padding:48px 32px;overflow-y:auto">
    <h5 style="font-family:Space Grotesk,sans-serif;font-size:10px;font-weight:900;color:#757575;text-transform:uppercase;letter-spacing:0.15em;margin:0 0 24px">On This Page</h5>
    <ul style="list-style:none;padding:0;margin:0;display:flex;flex-direction:column;gap:16px">{toc}</ul>
    <div style="margin-top:48px;background:rgba(31,31,31,0.5);padding:24px;border-radius:12px;border:1px solid rgba(255,255,255,0.03)">
      <div style="display:flex;align-items:center;gap:8px;margin-bottom:12px"><div style="width:8px;height:8px;border-radius:50%;background:#81ecff;animation:pulse 2s infinite;box-shadow:0 0 10px #81ecff"></div><span style="font-size:10px;font-weight:700;color:#81ecff;text-transform:uppercase">Live Sync</span></div>
      <p style="font-size:11px;color:#757575;margin:0;line-height:1.5">Docs auto-update when .cronus source changes.</p>
    </div>
  </aside>
</div>
<style>
@keyframes pulse{{0%,100%{{opacity:1}}50%{{opacity:0.5}}}}
@keyframes docFadeIn{{from{{opacity:0;transform:translateY(12px)}}to{{opacity:1;transform:translateY(0)}}}}
html{{scroll-behavior:smooth}}
main>section,main>header,main>div{{animation:docFadeIn 0.4s cubic-bezier(0,0,0.2,1) both}}
main>section:nth-child(2){{animation-delay:0.05s}}
main>section:nth-child(3){{animation-delay:0.1s}}
main>section:nth-child(4){{animation-delay:0.15s}}
main>section:nth-child(5){{animation-delay:0.2s}}
main>section:nth-child(6){{animation-delay:0.25s}}
.doc-nav{{cursor:pointer}}
</style>
<script{script_nonce}>
document.querySelectorAll('[data-scroll]').forEach(function(a){{
  a.addEventListener('click',function(e){{
    e.preventDefault();
    var id=a.dataset.scroll;
    var el=document.getElementById(id);
    if(el){{
      el.scrollIntoView({{behavior:'smooth',block:'start'}});
      // Update active state
      document.querySelectorAll('.doc-nav').forEach(function(n){{
        n.style.color='#ababab';n.style.fontWeight='400';n.style.borderLeft='';
      }});
      a.style.color='#fff';a.style.fontWeight='700';
    }}
  }});
}});
// Scroll spy: highlight active nav on scroll
var sections=document.querySelectorAll('main>section[id]');
var navItems=document.querySelectorAll('.doc-nav[data-scroll]');
window.addEventListener('scroll',function(){{
  var scrollPos=window.scrollY+100;
  sections.forEach(function(sec){{
    if(sec.offsetTop<=scrollPos&&sec.offsetTop+sec.offsetHeight>scrollPos){{
      navItems.forEach(function(n){{
        if(n.dataset.scroll===sec.id){{n.style.color='#fff';n.style.fontWeight='700'}}
        else{{n.style.color='#ababab';n.style.fontWeight='400'}}
      }});
    }}
  }});
}});
</script>
</body></html>"##,
        app_name = app_name,
        port = port,
        nav = nav_html,
        toc = toc_html,
        entities = entities_html,
        api = api_html,
        pages = pages_html,
        webhooks_section = if has_webhooks {
            webhooks_html
        } else {
            r#"<p style="color:#757575">No webhooks configured. Add a <code style="background:#191919;color:var(--primary);padding:2px 6px;border-radius:4px;font-size:13px;font-family:monospace">webhook</code> block to your .cronus file.</p>"#.to_string()
        },
        cronus_preview = cronus_preview,
        app_doc = app_doc_html,
        entity_count = state
            .entities
            .iter()
            .filter(|e| !e.name.starts_with('_'))
            .count(),
        api_count = state.apis.len(),
        page_count = state.pages.len(),
    )
}

/// Design System page -- live rendered components with the project's theme tokens.
pub(crate) fn render_design_system(state: &AppState) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    let app_name = &state.app.name;
    let t = crate::theme::get();

    // Extract style info
    let accent = state
        .style
        .as_ref()
        .and_then(|s| s.accent.as_deref())
        .unwrap_or("#87adff");
    let font = state
        .style
        .as_ref()
        .and_then(|s| s.font.as_deref())
        .unwrap_or("Inter");
    let theme_mode = state
        .style
        .as_ref()
        .and_then(|s| s.theme.as_deref())
        .unwrap_or("dark");

    // Color palette from theme tokens
    let colors = vec![
        ("Background", &t.background),
        ("Surface", &t.surface),
        ("Surface Container", &t.surface_container),
        ("Surface Bright", &t.surface_bright),
        ("On Surface", &t.on_surface),
        ("On Surface Variant", &t.on_surface_variant),
        ("Primary", &t.primary),
        ("Secondary", &t.secondary),
        ("Tertiary", &t.tertiary),
        ("Error", &t.error),
        ("Outline", &t.outline),
        ("Outline Variant", &t.outline_variant),
    ];

    let mut palette_html = String::new();
    for (name, color) in &colors {
        palette_html.push_str(&format!(
            r##"<div style="display:flex;flex-direction:column;align-items:center;gap:8px"><div style="width:100%;aspect-ratio:1;border-radius:8px;background:{color};border:1px solid rgba(255,255,255,0.1)"></div><span style="font-size:11px;color:#e2e2e2;font-weight:500;text-align:center">{name}</span><span style="font-family:monospace;font-size:9px;color:#757575">{color}</span></div>"##,
            name = name, color = color,
        ));
    }

    // Typography scale
    let type_scale = vec![
        ("Display", "48px", "900", font, "The quick brown fox"),
        ("Headline", "32px", "700", font, "The quick brown fox jumps"),
        (
            "Title",
            "20px",
            "700",
            "Inter",
            "The quick brown fox jumps over the lazy dog",
        ),
        (
            "Body",
            "14px",
            "400",
            "Inter",
            "The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs.",
        ),
        ("Label", "11px", "700", font, "UPPERCASE TRACKING WIDE"),
        (
            "Mono",
            "13px",
            "400",
            "monospace",
            "const x = await fetch('/api/data');",
        ),
    ];

    let mut type_html = String::new();
    for (name, size, weight, family, sample) in &type_scale {
        let ls = if *name == "Label" {
            "letter-spacing:0.15em;text-transform:uppercase;"
        } else {
            ""
        };
        type_html.push_str(&format!(
            r##"<div style="padding:20px 0;border-bottom:1px solid rgba(255,255,255,0.03)"><div style="display:flex;justify-content:space-between;align-items:baseline;margin-bottom:8px"><span style="font-size:10px;color:#757575;text-transform:uppercase;letter-spacing:0.15em;font-family:Space Grotesk,sans-serif;font-weight:700">{name}</span><span style="font-family:monospace;font-size:10px;color:#484848">{size} / {weight}</span></div><p style="font-family:{family},sans-serif;font-size:{size};font-weight:{weight};color:#e2e2e2;margin:0;{ls}">{sample}</p></div>"##,
            name = name, size = size, weight = weight, family = family, sample = sample, ls = ls,
        ));
    }

    // Section helper: wraps content in a glass panel
    let section = |id: &str, num: &str, title: &str, desc: &str, content: &str| -> String {
        format!(
            r##"<section style="margin-bottom:80px" id="{id}">
      <h2 style="font-family:Space Grotesk,sans-serif;font-size:24px;font-weight:700;margin-bottom:8px;display:flex;align-items:center;gap:12px"><span style="color:var(--secondary)">{num}.</span> {title}</h2>
      <p style="color:#757575;margin-bottom:24px;font-size:14px">{desc}</p>
      {content}
    </section>"##,
            id = id,
            num = num,
            title = title,
            desc = desc,
            content = content,
        )
    };

    // Component: wrap in glass card with label + code
    let comp = |name: &str, cronus_syntax: &str, rendered: &str| -> String {
        format!(
            r##"<div style="background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(135,173,255,0.2);margin-bottom:16px;overflow:hidden"><div style="padding:16px 24px;border-bottom:1px solid rgba(255,255,255,0.03);display:flex;justify-content:space-between;align-items:center"><span style="font-family:Space Grotesk,sans-serif;font-size:13px;font-weight:700;color:#e2e2e2">{name}</span><code style="font-size:10px;color:var(--primary);background:#191919;padding:2px 8px;border-radius:4px">{syntax}</code></div><div style="padding:24px;display:flex;flex-wrap:wrap;align-items:center;gap:12px">{rendered}</div></div>"##,
            name = name,
            syntax = cronus_syntax,
            rendered = rendered,
        )
    };

    // Render live components using the dark theme inline styles
    let btn_style = |bg: &str, color: &str, border: &str| -> String {
        format!("padding:10px 20px;border-radius:8px;font-family:Space Grotesk,sans-serif;font-size:12px;font-weight:700;letter-spacing:0.08em;text-transform:uppercase;cursor:pointer;transition:all 0.15s;border:{};background:{};color:{}", border, bg, color)
    };

    let buttons = format!(
        r##"<button style="{}">Primary</button><button style="{}">Secondary</button><button style="{}">Ghost</button><button style="{}">Danger</button><button style="{};font-size:10px;padding:6px 12px">Small</button><button style="{};font-size:14px;padding:14px 28px">Large</button>"##,
        btn_style(
            "linear-gradient(135deg,var(--primary),var(--secondary))",
            "#000",
            "none"
        ),
        btn_style("#191919", "#e2e2e2", "0.5px solid rgba(255,255,255,0.1)"),
        btn_style("transparent", "#ababab", "1px solid transparent"),
        btn_style(
            "rgba(239,68,68,0.1)",
            "#ef4444",
            "1px solid rgba(239,68,68,0.2)"
        ),
        btn_style(
            "linear-gradient(135deg,var(--primary),var(--secondary))",
            "#000",
            "none"
        ),
        btn_style(
            "linear-gradient(135deg,var(--primary),var(--secondary))",
            "#000",
            "none"
        ),
    );

    let input_style = "width:240px;background:#000;border:1px solid rgba(255,255,255,0.05);border-radius:8px;padding:12px 16px;color:#fff;font-size:14px;outline:none;font-family:Inter,sans-serif";
    let inputs = format!(
        r##"<input type="text" placeholder="Text input" style="{s}"><input type="email" placeholder="email@example.com" style="{s}"><input type="password" placeholder="••••••••" style="{s}"><textarea placeholder="Textarea" style="{s};height:60px;resize:none"></textarea>"##,
        s = input_style,
    );

    let selects = format!(
        r##"<select style="{s};appearance:none;cursor:pointer"><option>Select option</option><option>us-east-1</option><option>eu-west-2</option><option>ap-south-1</option></select>"##,
        s = input_style,
    );

    let badges = r##"<span style="display:inline-flex;align-items:center;gap:6px;padding:4px 12px;font-size:11px;font-weight:600;border-radius:999px;background:rgba(16,185,129,0.12);color:#10b981"><span style="width:6px;height:6px;border-radius:50%;background:#10b981"></span>Live</span><span style="display:inline-flex;align-items:center;gap:6px;padding:4px 12px;font-size:11px;font-weight:600;border-radius:999px;background:rgba(59,130,246,0.12);color:#3b82f6"><span style="width:6px;height:6px;border-radius:50%;background:#3b82f6;animation:pulse 2s infinite"></span>Rolling</span><span style="display:inline-flex;align-items:center;gap:6px;padding:4px 12px;font-size:11px;font-weight:600;border-radius:999px;background:rgba(239,68,68,0.12);color:#ef4444"><span style="width:6px;height:6px;border-radius:50%;background:#ef4444"></span>Failed</span><span style="display:inline-flex;align-items:center;gap:6px;padding:4px 12px;font-size:11px;font-weight:600;border-radius:999px;background:rgba(245,158,11,0.12);color:#f59e0b"><span style="width:6px;height:6px;border-radius:50%;background:#f59e0b"></span>Warning</span><span style="display:inline-flex;align-items:center;gap:6px;padding:4px 12px;font-size:11px;font-weight:600;border-radius:999px;background:rgba(113,113,122,0.12);color:#71717a"><span style="width:6px;height:6px;border-radius:50%;background:#71717a"></span>Pending</span>"##;

    let cards = r##"<div style="background:#1b1b1b;border:0.5px solid rgba(76,69,70,0.15);border-radius:12px;padding:20px 24px;width:200px"><div style="display:flex;align-items:center;gap:6px;margin-bottom:12px"><span class="material-symbols-outlined" style="font-size:16px;color:var(--primary)">trending_up</span><span style="font-size:13px;font-weight:500;color:rgba(226,226,226,0.5)">Requests</span><span style="font-size:10px;padding:2px 6px;border-radius:4px;background:rgba(16,185,129,0.1);color:#10b981">+12%</span></div><span style="font-size:36px;font-weight:700;letter-spacing:-0.03em;color:#e2e2e2">1.2M</span><p style="font-size:11px;color:rgba(226,226,226,0.3);margin:8px 0 0">Last 24h</p></div><div style="background:#1b1b1b;border:0.5px solid rgba(76,69,70,0.15);border-radius:12px;padding:20px 24px;width:200px"><div style="display:flex;align-items:center;gap:6px;margin-bottom:12px"><span class="material-symbols-outlined" style="font-size:16px;color:#ef4444">error_outline</span><span style="font-size:13px;font-weight:500;color:rgba(226,226,226,0.5)">Error Rate</span><span style="font-size:10px;padding:2px 6px;border-radius:4px;background:rgba(16,185,129,0.1);color:#10b981">-0.01%</span></div><span style="font-size:36px;font-weight:700;letter-spacing:-0.03em;color:#e2e2e2">0.02%</span><p style="font-size:11px;color:rgba(226,226,226,0.3);margin:8px 0 0">5xx responses</p></div>"##;

    let alerts = r##"<div style="width:100%;display:flex;flex-direction:column;gap:8px"><div style="background:rgba(135,173,255,0.1);border-left:2px solid #87adff;padding:16px 20px;border-radius:0 8px 8px 0;display:flex;gap:12px"><span class="material-symbols-outlined" style="color:var(--primary);font-size:18px">info</span><div><p style="font-size:13px;font-weight:600;color:var(--primary);margin:0">Info</p><p style="font-size:12px;color:#ababab;margin:4px 0 0">This is an informational alert.</p></div></div><div style="background:rgba(16,185,129,0.1);border-left:2px solid #10b981;padding:16px 20px;border-radius:0 8px 8px 0;display:flex;gap:12px"><span class="material-symbols-outlined" style="color:#10b981;font-size:18px">check_circle</span><div><p style="font-size:13px;font-weight:600;color:#10b981;margin:0">Success</p><p style="font-size:12px;color:#ababab;margin:4px 0 0">Operation completed successfully.</p></div></div><div style="background:rgba(245,158,11,0.1);border-left:2px solid #f59e0b;padding:16px 20px;border-radius:0 8px 8px 0;display:flex;gap:12px"><span class="material-symbols-outlined" style="color:#f59e0b;font-size:18px">warning</span><div><p style="font-size:13px;font-weight:600;color:#f59e0b;margin:0">Warning</p><p style="font-size:12px;color:#ababab;margin:4px 0 0">Memory pressure is above 85%.</p></div></div><div style="background:rgba(239,68,68,0.1);border-left:2px solid #ef4444;padding:16px 20px;border-radius:0 8px 8px 0;display:flex;gap:12px"><span class="material-symbols-outlined" style="color:#ef4444;font-size:18px">error</span><div><p style="font-size:13px;font-weight:600;color:#ef4444;margin:0">Error</p><p style="font-size:12px;color:#ababab;margin:4px 0 0">Deployment failed on us-west-2.</p></div></div></div>"##;

    let modal_preview = r##"<div style="background:#191919;border-radius:12px;padding:32px;border-top:0.5px solid rgba(135,173,255,0.2);box-shadow:0 0 60px rgba(135,173,255,0.04);width:100%;max-width:420px"><div style="text-align:center;margin-bottom:24px"><div style="width:40px;height:40px;border-radius:12px;background:linear-gradient(135deg,var(--primary),var(--secondary));display:inline-flex;align-items:center;justify-content:center;margin-bottom:12px"><span class="material-symbols-outlined" style="color:#fff;font-size:20px">rocket_launch</span></div><h3 style="font-family:Space Grotesk,sans-serif;font-size:20px;font-weight:700;margin:0 0 4px;color:#fff">New Deployment</h3><p style="font-size:12px;color:#ababab;margin:0">Configure and launch a new deployment.</p></div><div style="display:flex;flex-direction:column;gap:12px;margin-bottom:20px"><input placeholder="Service name" style="background:#000;border:1px solid rgba(255,255,255,0.05);border-radius:8px;padding:10px 14px;color:#fff;font-size:13px;outline:none"><select style="background:#000;border:1px solid rgba(255,255,255,0.05);border-radius:8px;padding:10px 14px;color:#fff;font-size:13px;outline:none;appearance:none"><option>us-east-1</option><option>eu-west-2</option></select></div><div style="display:flex;gap:12px"><button style="flex:1;padding:10px;border:0.5px solid rgba(255,255,255,0.1);border-radius:8px;background:#191919;color:#ababab;font-size:12px;cursor:pointer">Cancel</button><button style="flex:1;padding:10px;border:none;border-radius:8px;background:linear-gradient(135deg,var(--primary),var(--secondary));color:#000;font-weight:700;font-size:12px;cursor:pointer">Deploy Now</button></div></div>"##;

    // Build sections
    let content = vec![
        section("colors", "01", "Color Palette", "Material Design 3 tokens derived from the accent color. Every surface, text, and interactive element uses these tokens.", &format!(r##"<div style="background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(135,173,255,0.2);padding:24px;display:grid;grid-template-columns:repeat(4,1fr);gap:16px">{}</div>"##, palette_html)),
        section("typography", "02", "Typography", &format!("Headline: {} · Body: Inter · Label: {} · Mono: system", font, font), &format!(r##"<div style="background:rgba(25,25,25,0.8);backdrop-filter:blur(40px);border-radius:12px;border:1px solid rgba(255,255,255,0.03);border-top:0.5px solid rgba(135,173,255,0.2);padding:24px">{}</div>"##, type_html)),
        section("buttons", "03", "Buttons", "Kinetic triggers with gradient primary, ghost, outline, and danger variants.", &comp("Button", "action \"Label\" style:primary", &buttons)),
        section("inputs", "04", "Inputs", "Terminal-style inputs with etched black background and focus glow.", &comp("Input", "field \"Name\" type:text", &inputs)),
        section("selects", "05", "Select", "Dropdown selects with consistent styling.", &comp("Select", "field \"Region\" type:select options:\"...\"", &selects)),
        section("badges", "06", "Status Badges", "Semantic status indicators with pulse animation for active states.", &comp("Badge", "status enum [\"Live\", \"Rolling\", \"Failed\"]", badges)),
        section("kpi", "07", "KPI Cards", "Data-driven metric cards with icon, value, trend badge, and subtitle.", &comp("KPI Card", "section kpi cols:4 { bind Entity }", cards)),
        section("alerts", "08", "Alerts", "Contextual messages with severity levels and edge lighting.", &comp("Alert", "section alert { ... }", alerts)),
        section("modal", "09", "Modal", "Glassmorphic dialog with backdrop blur, edge lighting, entity binding, and form fields.", &comp("Modal", "section modal entity:\"Entity\" { ... }", modal_preview)),
    ].join("\n");

    // TOC
    let toc_items = vec![
        ("colors", "Color Palette"),
        ("typography", "Typography"),
        ("buttons", "Buttons"),
        ("inputs", "Inputs"),
        ("selects", "Select"),
        ("badges", "Status Badges"),
        ("kpi", "KPI Cards"),
        ("alerts", "Alerts"),
        ("modal", "Modal"),
    ];
    let toc: String = toc_items.iter().enumerate().map(|(i, (id, name))| {
        let dot = if i == 0 {
            r##"<div style="width:6px;height:6px;border-radius:50%;background:var(--primary);box-shadow:0 0 8px rgba(135,173,255,0.8)"></div>"##
        } else {
            r##"<div style="width:4px;height:4px;border-radius:50%;background:#484848"></div>"##
        };
        let color = if i == 0 { "#87adff" } else { "#ababab" };
        format!(r##"<li><a class="doc-nav" data-scroll="{id}" style="font-size:12px;color:{color};display:flex;align-items:center;gap:8px;text-decoration:none;cursor:pointer">{dot}{name}</a></li>"##, id = id, color = color, dot = dot, name = name)
    }).collect::<Vec<_>>().join("");

    format!(
        r##"<!DOCTYPE html>
<html lang="en"><head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width,initial-scale=1.0"/>
<title>{app_name} | Design System</title>
<link href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700;900&family=Inter:wght@300;400;500;600&display=swap" rel="stylesheet"/>
<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet"/>
<style>
body {{ background:#0e0e0e; color:#fff; font-family:'Inter',sans-serif; margin:0 }}
.material-symbols-outlined {{ font-variation-settings:'FILL' 0,'wght' 300,'GRAD' 0,'opsz' 24 }}
::-webkit-scrollbar {{ width:4px }} ::-webkit-scrollbar-track {{ background:#0e0e0e }} ::-webkit-scrollbar-thumb {{ background:#262626;border-radius:10px }}
@keyframes pulse {{ 0%,100%{{opacity:1}} 50%{{opacity:0.5}} }}
</style>
</head>
<body>
<header style="position:fixed;top:0;width:100%;z-index:50;height:64px;background:rgba(0,0,0,0.8);backdrop-filter:blur(40px);border-bottom:1px solid rgba(255,255,255,0.05);display:flex;align-items:center;justify-content:space-between;padding:0 24px;box-sizing:border-box">
  <div style="display:flex;align-items:center;gap:32px">
    <span style="font-family:Space Grotesk,sans-serif;font-size:18px;font-weight:900;color:#fff;letter-spacing:-0.03em">{app_name}_DESIGN</span>
    <nav style="display:flex;gap:24px;font-family:Space Grotesk,sans-serif;font-weight:700;font-size:14px">
      <a href="/docs" style="color:#757575;text-decoration:none">API Docs</a>
      <a href="/docs/design" style="color:#fff;border-bottom:2px solid #d277ff;padding-bottom:2px;text-decoration:none">Design System</a>
      <a href="/" style="color:#757575;text-decoration:none">Dashboard</a>
    </nav>
  </div>
</header>

<div style="display:flex;padding-top:64px;min-height:100vh">
  <main style="flex:1;padding:48px 64px;max-width:780px;margin-left:auto;margin-right:260px">
    <header style="margin-bottom:60px">
      <div style="display:flex;align-items:center;gap:8px;margin-bottom:16px">
        <span style="font-family:monospace;font-size:12px;color:var(--secondary);text-transform:uppercase;letter-spacing:-0.03em">Auto-Generated</span>
        <div style="width:4px;height:4px;border-radius:50%;background:#484848"></div>
        <span style="font-family:monospace;font-size:12px;color:#757575">Theme: {theme} · Accent: {accent} · Font: {font}</span>
      </div>
      <h1 style="font-family:Space Grotesk,sans-serif;font-size:48px;font-weight:900;letter-spacing:-0.03em;margin:0 0 24px;background:linear-gradient(to right,#fff,#fff,#757575);-webkit-background-clip:text;-webkit-text-fill-color:transparent">Design System</h1>
      <p style="font-size:18px;color:#757575;line-height:1.6">Component library and design tokens for <strong style="color:#ababab">{app_name}</strong>. Every component shown here is rendered live using the project's theme — what you see is what CRONUS generates.</p>
    </header>

    {content}

    <div style="background:rgba(210,119,255,0.1);border-left:2px solid #d277ff;padding:24px;border-radius:0 12px 12px 0;display:flex;gap:16px;margin-bottom:48px">
      <span class="material-symbols-outlined" style="color:var(--secondary)">palette</span>
      <div><h4 style="font-weight:700;color:var(--secondary);margin:0 0 4px;font-size:14px">Live Components</h4><p style="font-size:13px;color:#ababab;margin:0">Every component above is rendered with the same engine that powers your dashboard. Change the <code style="background:#191919;color:var(--primary);padding:2px 6px;border-radius:4px;font-size:12px">style</code> block in your .cronus and the design system updates automatically.</p></div>
    </div>
  </main>

  <aside style="width:200px;position:fixed;right:0;top:64px;height:calc(100vh - 64px);padding:48px 24px;overflow-y:auto">
    <h5 style="font-family:Space Grotesk,sans-serif;font-size:10px;font-weight:900;color:#757575;text-transform:uppercase;letter-spacing:0.15em;margin:0 0 24px">Components</h5>
    <ul style="list-style:none;padding:0;margin:0;display:flex;flex-direction:column;gap:12px">{toc}</ul>
  </aside>
</div>
<style>
@keyframes pulse{{0%,100%{{opacity:1}}50%{{opacity:0.5}}}}
@keyframes docFadeIn{{from{{opacity:0;transform:translateY(12px)}}to{{opacity:1;transform:translateY(0)}}}}
html{{scroll-behavior:smooth}}
main>section{{animation:docFadeIn 0.4s cubic-bezier(0,0,0.2,1) both}}
main>section:nth-child(2){{animation-delay:0.05s}}
main>section:nth-child(3){{animation-delay:0.1s}}
main>section:nth-child(4){{animation-delay:0.15s}}
main>section:nth-child(5){{animation-delay:0.2s}}
main>section:nth-child(6){{animation-delay:0.25s}}
main>section:nth-child(7){{animation-delay:0.3s}}
main>section:nth-child(8){{animation-delay:0.35s}}
main>section:nth-child(9){{animation-delay:0.4s}}
main>section:nth-child(10){{animation-delay:0.45s}}
</style>
<script{script_nonce}>
document.querySelectorAll('[data-scroll]').forEach(function(a){{
  a.addEventListener('click',function(e){{
    e.preventDefault();
    var el=document.getElementById(a.dataset.scroll);
    if(el)el.scrollIntoView({{behavior:'smooth',block:'start'}});
    document.querySelectorAll('.doc-nav').forEach(function(n){{n.style.color='#ababab'}});
    a.style.color='#fff';
  }});
}});
var sections=document.querySelectorAll('main>section[id]');
var navItems=document.querySelectorAll('.doc-nav[data-scroll]');
window.addEventListener('scroll',function(){{
  var sp=window.scrollY+100;
  sections.forEach(function(sec){{
    if(sec.offsetTop<=sp&&sec.offsetTop+sec.offsetHeight>sp){{
      navItems.forEach(function(n){{n.style.color=n.dataset.scroll===sec.id?'#fff':'#ababab'}});
    }}
  }});
}});
</script>
</body></html>"##,
        app_name = app_name,
        accent = accent,
        font = font,
        theme = theme_mode,
        content = content,
        toc = toc,
    )
}

/// Relationship graph page -- interactive Mermaid diagram of entity relations,
/// page bindings, and webhook flows.
pub(crate) fn render_graph_page(state: &AppState) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    let app_name = &state.app.name;
    let relationship_graph =
        graph::build_graph_from_state(&state.entities, &state.pages, &state.webhooks);
    let mermaid_code = graph::to_mermaid(&relationship_graph);
    // Escape backticks and backslashes for safe JS embedding
    let _mermaid_escaped = mermaid_code
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${");

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>{app_name} | Relationship Graph</title>
<link href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700;900&family=Inter:wght@300;400;500;600&display=swap" rel="stylesheet"/>
<script{script_nonce} src="https://cdn.jsdelivr.net/npm/mermaid/dist/mermaid.min.js"></script>
<style>
body {{ background:#0e0e0e; color:#fff; font-family:'Inter',sans-serif; margin:0 }}
::-webkit-scrollbar {{ width:4px }} ::-webkit-scrollbar-track {{ background:#0e0e0e }} ::-webkit-scrollbar-thumb {{ background:#262626;border-radius:10px }}
.graph-container {{ padding:32px; display:flex; justify-content:center; align-items:flex-start; min-height:calc(100vh - 64px - 64px) }}
.mermaid {{ background:#141414; border:1px solid rgba(255,255,255,0.06); border-radius:12px; padding:40px; min-width:600px; max-width:100%; overflow-x:auto }}
.mermaid svg {{ max-width:100% }}
.stats {{ display:flex; gap:24px; justify-content:center; padding:0 32px 24px; font-family:'Space Grotesk',sans-serif; font-size:13px; color:#757575 }}
.stats span {{ background:#141414; border:1px solid rgba(255,255,255,0.06); border-radius:8px; padding:8px 16px }}
.stats .count {{ color:var(--primary); font-weight:700 }}
</style>
</head>
<body>
<header style="position:fixed;top:0;width:100%;z-index:50;height:64px;background:rgba(0,0,0,0.8);backdrop-filter:blur(40px);border-bottom:1px solid rgba(255,255,255,0.05);display:flex;align-items:center;justify-content:space-between;padding:0 24px;box-sizing:border-box">
  <div style="display:flex;align-items:center;gap:32px">
    <span style="font-family:Space Grotesk,sans-serif;font-size:18px;font-weight:900;color:#fff;letter-spacing:-0.03em">{app_name}_DOCS</span>
    <nav style="display:flex;gap:24px;font-family:Space Grotesk,sans-serif;font-weight:700;font-size:14px">
      <a href="/docs" style="color:#757575;text-decoration:none">API Docs</a>
      <a href="/docs/design" style="color:#757575;text-decoration:none">Design System</a>
      <a href="/docs/graph" style="color:#fff;border-bottom:2px solid #87adff;padding-bottom:2px;text-decoration:none">Graph</a>
      <a href="/" style="color:#757575;text-decoration:none">Dashboard</a>
    </nav>
  </div>
</header>

<div style="padding-top:80px">
  <div class="stats">
    <span>Entities <span class="count">{entity_count}</span></span>
    <span>Relations <span class="count">{relation_count}</span></span>
    <span>Page Bindings <span class="count">{binding_count}</span></span>
    <span>Webhook Flows <span class="count">{webhook_count}</span></span>
  </div>
  <div class="graph-container">
    <pre class="mermaid" id="graph">{mermaid_code}</pre>
  </div>
</div>

<script{script_nonce}>
mermaid.initialize({{
  startOnLoad: true,
  theme: 'dark',
  themeVariables: {{
    primaryColor: '#87adff',
    primaryTextColor: '#fff',
    primaryBorderColor: '#87adff',
    lineColor: '#555',
    secondaryColor: '#d277ff',
    tertiaryColor: '#141414',
    background: '#0e0e0e',
    mainBkg: '#1a1a1a',
    nodeBorder: '#87adff',
    clusterBkg: '#141414',
    clusterBorder: '#262626',
    titleColor: '#fff',
    edgeLabelBackground: '#1a1a1a',
    fontFamily: 'Space Grotesk, sans-serif',
  }},
  flowchart: {{
    htmlLabels: true,
    curve: 'basis',
    padding: 20,
  }},
}});
</script>
</body>
</html>"##,
        app_name = app_name,
        mermaid_code = mermaid_code,
        entity_count = state
            .entities
            .iter()
            .filter(|e| !e.name.starts_with('_'))
            .count(),
        relation_count = relationship_graph.entity_relations.len(),
        binding_count = relationship_graph.page_bindings.len(),
        webhook_count = relationship_graph.webhook_flows.len(),
    )
}
