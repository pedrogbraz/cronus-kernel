//! Miscellaneous section renderers (topbar, footer, faq, testimonial, pricing, cta, etc.)
use crate::parser::SectionNode;

pub(super) fn render_topbar(section: &SectionNode, theme: &str) -> String {
    let script_nonce = crate::security::script_nonce_attr();
    let brand = section
        .config
        .get("brand")
        .map(|s| s.as_str())
        .or(section.title.as_deref())
        .unwrap_or("Brand");
    let is_dark = section
        .config
        .get("style")
        .map(|s| s.contains("dark"))
        .unwrap_or(false)
        || theme == "dark";

    // Detect dashboard mode: style:dashboard or style:sidebar offsets left for 256px sidebar
    let style_hint = section
        .config
        .get("style")
        .map(|s| s.as_str())
        .unwrap_or("");
    let has_sidebar = style_hint.contains("dashboard") || style_hint.contains("sidebar");
    let position_style = if has_sidebar {
        "position:fixed;top:0;right:0;left:256px;z-index:999"
    } else {
        "position:fixed;top:0;left:0;right:0;z-index:999"
    };

    // Accent color for active nav link
    let accent = section
        .config
        .get("accent")
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            if is_dark {
                "#adc6ff".to_string()
            } else {
                "#3b82f6".to_string()
            }
        });

    // Parse nav links from config (e.g. "Inventory, Analytics, Orders, Customers")
    let nav_links: Vec<String> = section.config.get("nav")
        .map(|nav| {
            nav.split(',').map(|link| {
                let l = link.trim();
                // Generate slug: "Getting Started" -> "/getting-started"
                let slug = format!("/{}", l.to_lowercase().replace(' ', "-"));
                let is_active = false; // Active state determined client-side
                let (color, border, weight) = if is_active {
                    (format!("{}", accent), format!("2px solid {}", accent), "500")
                } else {
                    ("rgba(226,226,226,0.6)".to_string(), "2px solid transparent".to_string(), "500")
                };
                format!(
                    r##"<a href="{slug}" data-nav style="color:{color};font-size:14px;font-weight:{weight};letter-spacing:-0.01em;text-decoration:none;padding:20px 0;border-bottom:{border};transition:all 0.2s" onmouseover="this.style.color='rgba(226,226,226,0.9)'" onmouseout="if(!this.classList.contains('active'))this.style.color='{color}'">{l}</a>"##,
                    slug=slug, l=l, color=color, border=border, weight=weight
                )
            }).collect()
        })
        .unwrap_or_default();

    let nav_html = nav_links.join("\n          ");

    // Collect action items from section items (icon buttons: notifications, settings, etc.)
    let action_buttons: Vec<String> = section.items.iter()
        .filter(|item| item.get("_type").map(|s| s.as_str()) == Some("action"))
        .map(|item| {
            let icon = item.get("icon").map(|s| s.as_str()).unwrap_or("circle");
            let title = item.get("title").map(|s| s.as_str()).unwrap_or("");
            format!(
                r##"<button title="{title}" style="width:36px;height:36px;display:flex;align-items:center;justify-content:center;background:none;border:1px solid rgba(76,69,70,0.2);border-radius:50%;cursor:pointer;transition:all 0.2s" onmouseover="this.style.background='rgba(255,255,255,0.06)';this.style.borderColor='rgba(76,69,70,0.4)'" onmouseout="this.style.background='none';this.style.borderColor='rgba(76,69,70,0.2)'"><span class="material-symbols-outlined" style="font-size:18px;color:rgba(226,226,226,0.6)">{icon}</span></button>"##,
                title=title, icon=icon
            )
        })
        .collect();

    let actions_html = action_buttons.join("\n      ");

    // Collect image items (avatar — 32px circle)
    let avatar_html: String = section.items.iter()
        .filter(|item| item.get("_type").map(|s| s.as_str()) == Some("image"))
        .map(|item| {
            let src = item.get("src").map(|s| s.as_str()).unwrap_or("");
            let alt = item.get("title").map(|s| s.as_str()).unwrap_or("Avatar");
            if src.is_empty() {
                format!(
                    r##"<div title="{alt}" style="width:32px;height:32px;border-radius:50%;background:linear-gradient(135deg,#2a2a2a,#3a3a3a);border:0.5px solid rgba(76,69,70,0.3);overflow:hidden;flex-shrink:0"></div>"##,
                    alt=alt
                )
            } else {
                format!(
                    r##"<div title="{alt}" style="width:32px;height:32px;border-radius:50%;border:0.5px solid rgba(76,69,70,0.3);overflow:hidden;flex-shrink:0"><img src="{src}" alt="{alt}" style="width:100%;height:100%;object-fit:cover"></div>"##,
                    alt=alt, src=src
                )
            }
        })
        .collect::<Vec<_>>()
        .join("");

    // Build right side: explicit actions+avatar, or fallback to default icons
    let right_side = if action_buttons.is_empty() && avatar_html.is_empty() {
        // Legacy/fallback: CTA button if configured
        let cta_text_opt = section
            .config
            .get("cta_text")
            .or(section.config.get("cta"))
            .map(|s| s.as_str());
        let cta_link = section
            .config
            .get("cta_link")
            .map(|s| s.as_str())
            .unwrap_or("/signup");
        let cta_btn = match cta_text_opt {
            Some(t) if !t.eq_ignore_ascii_case("search") => {
                format!(
                    r##"<a href="{cta_link}" style="display:inline-flex;align-items:center;padding:6px 16px;border-radius:8px;background:#fff;color:#000;font-weight:700;font-size:12px;text-decoration:none;transition:opacity 0.15s" onmouseover="this.style.opacity='0.9'" onmouseout="this.style.opacity='1'">{cta_text}</a>"##,
                    cta_link = cta_link,
                    cta_text = t
                )
            }
            _ => String::new(),
        };
        format!(
            r##"<button style="width:36px;height:36px;display:flex;align-items:center;justify-content:center;background:none;border:1px solid rgba(76,69,70,0.2);border-radius:50%;cursor:pointer;transition:all 0.2s" onmouseover="this.style.background='rgba(255,255,255,0.06)'" onmouseout="this.style.background='none'"><span class="material-symbols-outlined" style="font-size:18px;color:rgba(226,226,226,0.6)">search</span></button>
      <button style="width:36px;height:36px;display:flex;align-items:center;justify-content:center;background:none;border:1px solid rgba(76,69,70,0.2);border-radius:50%;cursor:pointer;transition:all 0.2s" onmouseover="this.style.background='rgba(255,255,255,0.06)'" onmouseout="this.style.background='none'"><span class="material-symbols-outlined" style="font-size:18px;color:rgba(226,226,226,0.6)">notifications</span></button>
      <div style="width:32px;height:32px;border-radius:50%;background:linear-gradient(135deg,#2a2a2a,#3a3a3a);border:0.5px solid rgba(76,69,70,0.3);overflow:hidden;flex-shrink:0"></div>
      {cta_btn}"##,
            cta_btn = cta_btn
        )
    } else {
        format!(
            "{actions}\n      {avatar}",
            actions = actions_html,
            avatar = avatar_html
        )
    };

    format!(
        r##"<header data-cronus-topbar class="anim-slide-down" style="{position};height:64px;background:rgba(19,19,19,0.8);backdrop-filter:blur(40px);-webkit-backdrop-filter:blur(40px);border-bottom:0.5px solid rgba(76,69,70,0.2);box-shadow:0 8px 32px rgba(0,0,0,0.36);font-family:Inter,system-ui,-apple-system,sans-serif;font-size:14px;letter-spacing:-0.02em">
  <div style="display:flex;justify-content:space-between;align-items:center;padding:0 24px;height:64px">
    <div style="display:flex;align-items:center;gap:32px">
      <a href="/" style="font-size:20px;font-weight:700;letter-spacing:-0.04em;color:#e2e2e2;text-decoration:none">{brand}</a>
      <nav style="display:flex;align-items:center;gap:24px;height:64px">
        {nav_html}
      </nav>
    </div>
    <div style="display:flex;align-items:center;gap:12px">
      {right_side}
    </div>
  </div>
</header>
<script{script_nonce}>!function(){{var p=location.pathname.replace(/\/$/,'')||'/';document.querySelectorAll('[data-cronus-topbar] [data-nav]').forEach(function(a){{var h=a.getAttribute('href');if(h===p||(p==='/'&&h==='/')){{a.style.color='{accent}';a.style.borderBottom='2px solid {accent}';a.classList.add('active')}}}})}}()</script>"##,
        position = position_style,
        brand = brand,
        nav_html = nav_html,
        right_side = right_side,
        accent = accent
    )
}

pub(super) fn render_checkout_section(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("Checkout");
    let mut exp = String::new();
    let mut fld = String::new();
    let mut sub = String::from("Pay");
    let mut chk = String::new();
    for item in &section.items {
        let n = item
            .get("title")
            .or_else(|| item.get("name"))
            .map(|s| s.as_str())
            .unwrap_or("");
        let d = item
            .get("description")
            .or_else(|| item.get("desc"))
            .map(|s| s.as_str())
            .unwrap_or("");
        if d.starts_with("express") {
            let dk = d.contains("dark");
            let (b, c, br) = if dk {
                ("black", "white", "none")
            } else {
                ("white", "black", "1px solid #e5e5e5")
            };
            exp.push_str(&format!(r##"<button style="background:{b};color:{c};height:48px;border-radius:999px;border:{br};display:flex;align-items:center;justify-content:center;gap:8px;cursor:pointer;font-size:14px;font-weight:600;transition:opacity 0.2s" onmouseover="this.style.opacity='0.9'" onmouseout="this.style.opacity='1'">Pay with <b>{n}</b></button>"##, b=b,c=c,br=br,n=n));
        } else if d.starts_with("field:") {
            let p: Vec<&str> = d.splitn(3, ':').collect();
            let ft = *p.get(1).unwrap_or(&"text");
            let ph = *p.get(2).unwrap_or(&"");
            if ft == "card" {
                fld.push_str(&format!(r##"<div style="display:flex;flex-direction:column;gap:8px"><label style="font-size:11px;font-weight:600;letter-spacing:0.1em;text-transform:uppercase;color:#71717a">{n}</label><input type="text" placeholder="{ph}" style="width:100%;height:48px;padding:0 16px;border-radius:8px 8px 0 0;border:1px solid rgba(198,198,198,0.4);background:white;font-size:14px;outline:none" onfocus="this.style.borderColor='black'" onblur="this.style.borderColor='rgba(198,198,198,0.4)'"><div style="display:grid;grid-template-columns:1fr 1fr"><input type="text" placeholder="MM / YY" style="width:100%;height:48px;padding:0 16px;border-radius:0 0 0 8px;border:1px solid rgba(198,198,198,0.4);border-top:none;background:white;font-size:14px;outline:none"><input type="text" placeholder="CVC" style="width:100%;height:48px;padding:0 16px;border-radius:0 0 8px 0;border:1px solid rgba(198,198,198,0.4);border-top:none;border-left:none;background:white;font-size:14px;outline:none"></div></div>"##, n=n, ph=ph));
            } else if ft == "select" {
                let opts: String = ph
                    .split(',')
                    .map(|o| format!("<option>{}</option>", o.trim()))
                    .collect::<Vec<_>>()
                    .join("");
                fld.push_str(&format!(r##"<div style="display:flex;flex-direction:column;gap:8px"><label style="font-size:11px;font-weight:600;letter-spacing:0.1em;text-transform:uppercase;color:#71717a">{n}</label><select style="width:100%;height:48px;padding:0 16px;border-radius:8px;border:1px solid rgba(198,198,198,0.4);background:white;font-size:14px;outline:none;appearance:none">{opts}</select></div>"##, n=n, opts=opts));
            } else {
                let it = if ft == "email" { "email" } else { "text" };
                fld.push_str(&format!(r##"<div style="display:flex;flex-direction:column;gap:8px"><label style="font-size:11px;font-weight:600;letter-spacing:0.1em;text-transform:uppercase;color:#71717a">{n}</label><input type="{it}" placeholder="{ph}" style="width:100%;height:48px;padding:0 16px;border-radius:8px;border:1px solid rgba(198,198,198,0.4);background:white;font-size:14px;outline:none" onfocus="this.style.borderColor='black'" onblur="this.style.borderColor='rgba(198,198,198,0.4)'"></div>"##, n=n, it=it, ph=ph));
            }
        } else if d == "checkbox" {
            chk = format!(
                r##"<label style="display:flex;align-items:center;gap:12px;cursor:pointer;padding-top:8px"><input type="checkbox" style="width:16px;height:16px;accent-color:black"><span style="font-size:14px;color:#52525b">{n}</span></label>"##,
                n = n
            );
        } else if d == "submit" {
            sub = n.to_string();
        }
    }
    format!(
        r##"<main style="max-width:640px;margin:0 auto;padding:48px 24px 80px"><h1 style="font-size:30px;font-weight:700;letter-spacing:-0.02em;margin-bottom:32px">{title}</h1><div style="display:grid;grid-template-columns:1fr 1fr;gap:16px;margin-bottom:40px">{exp}</div><div style="display:flex;align-items:center;gap:16px;margin-bottom:32px"><div style="flex:1;height:1px;background:#e5e5e5"></div><span style="color:#a1a1aa;font-size:11px;font-weight:500;text-transform:uppercase;letter-spacing:0.1em">Or pay with card</span><div style="flex:1;height:1px;background:#e5e5e5"></div></div><form data-entity="order" style="display:flex;flex-direction:column;gap:24px">{fld}{chk}<button type="submit" style="width:100%;height:56px;border-radius:999px;background:black;color:white;font-size:18px;font-weight:700;border:none;cursor:pointer;display:flex;align-items:center;justify-content:center;gap:8px;margin-top:16px;transition:opacity 0.2s" onmouseover="this.style.opacity='0.9'" onmouseout="this.style.opacity='1'">{sub} <svg width="16" height="16" fill="rgba(255,255,255,0.5)" viewBox="0 0 24 24"><path d="M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zm-6 9c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2zm3.1-9H8.9V6c0-1.71 1.39-3.1 3.1-3.1s3.1 1.39 3.1 3.1v2z"/></svg></button><p style="text-align:center;font-size:12px;color:#a1a1aa;margin-top:8px;line-height:1.6">By confirming your payment, you agree to our Terms of Service and Privacy Policy.</p></form></main>"##,
        title = title,
        exp = exp,
        fld = fld,
        chk = chk,
        sub = sub
    )
}

pub(super) fn render_testimonial(section: &SectionNode, theme: &str) -> String {
    let style_hint = section
        .config
        .get("style")
        .map(|s| s.as_str())
        .unwrap_or("");
    let is_dark = style_hint.contains("dark") || theme == "dark";
    let is_light = !is_dark;

    let section_title = section.title.as_deref().unwrap_or("");
    let section_subtitle = section.subtitle.as_deref().unwrap_or("");

    // If no items, fall back to single testimonial from title/subtitle
    if section.items.is_empty() {
        return format!(
            r##"<section style="padding:80px 24px;background:var(--cronus-bg)">
  <div style="max-width:640px;margin:0 auto">
    <div style="background:var(--cronus-surface);border:1px solid var(--cronus-border);border-radius:var(--cronus-radius);padding:32px">
      <p style="font-size:16px;font-style:italic;line-height:1.7;color:var(--cronus-text);margin-bottom:20px">"{quote}"</p>
      <p style="font-size:13px;font-weight:700;letter-spacing:0.05em;text-transform:uppercase;color:var(--cronus-text-muted)">{author}</p>
    </div>
  </div>
</section>"##,
            quote = section_title,
            author = section_subtitle
        );
    }

    // Multi-item testimonial grid
    let cards: Vec<String> = section.items.iter().map(|item| {
        let quote = item.get("description").or_else(|| item.get("title")).map(|s| s.as_str()).unwrap_or("");
        let name = item.get("name").or_else(|| item.get("title")).map(|s| s.as_str()).unwrap_or("Anonymous");
        let role = item.get("role").or_else(|| item.get("meta")).or_else(|| item.get("subtitle")).map(|s| s.as_str()).unwrap_or("");

        // Generate initials from name
        let initials: String = name.split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .map(|c| c.to_uppercase().to_string())
            .collect();

        format!(
            r##"<div style="background:var(--cronus-surface);border:1px solid var(--cronus-border);border-radius:var(--cronus-radius);padding:28px;display:flex;flex-direction:column;justify-content:space-between;gap:20px">
  <p style="font-size:15px;font-style:italic;line-height:1.7;color:var(--cronus-text)">"{quote}"</p>
  <div style="display:flex;align-items:center;gap:12px">
    <div style="width:36px;height:36px;border-radius:50%;background:var(--cronus-border);display:flex;align-items:center;justify-content:center;font-size:13px;font-weight:700;color:var(--cronus-text-muted)">{initials}</div>
    <div>
      <div style="font-size:14px;font-weight:600;color:var(--cronus-text)">{name}</div>
      <div style="font-size:12px;color:var(--cronus-text-muted)">{role}</div>
    </div>
  </div>
</div>"##,
            quote=quote, initials=initials, name=name, role=role
        )
    }).collect();

    // Section header
    let header = if !section_title.is_empty() {
        format!(
            r##"<div style="text-align:center;margin-bottom:48px">
    <h2 style="font-size:36px;font-weight:800;letter-spacing:-0.04em;color:var(--cronus-text);margin-bottom:12px">{title}</h2>
    <p style="font-size:16px;color:var(--cronus-text-muted);max-width:600px;margin:0 auto">{subtitle}</p>
  </div>"##,
            title = section_title,
            subtitle = section_subtitle
        )
    } else {
        String::new()
    };

    let cols = if cards.len() <= 2 { cards.len() } else { 3 };
    format!(
        r##"<section style="padding:80px 24px;background:var(--cronus-bg)">
  <div style="max-width:1280px;margin:0 auto">
    {header}
    <div style="display:grid;grid-template-columns:repeat({cols},1fr);gap:20px">
      {cards}
    </div>
  </div>
</section>"##,
        header = header,
        cols = cols,
        cards = cards.join("\n      ")
    )
}

pub(super) fn render_pricing(section: &SectionNode, _accent: &str) -> String {
    let title = section.title.as_deref().unwrap_or("Pricing");

    let plans: Vec<String> = section.plans.iter().map(|plan| {
        let features: Vec<String> = plan.features.iter()
            .map(|f| format!(
                r#"<li style="display:flex;align-items:center;gap:8px;font-size:14px;color:var(--cronus-fg-secondary);"><span style="color:var(--cronus-primary)">✓</span> {}</li>"#,
                f
            ))
            .collect();

        let border = if plan.featured {
            "2px solid var(--cronus-primary)"
        } else {
            "1px solid var(--cronus-border)"
        };
        let badge = if plan.featured {
            r#"<span style="font-size:11px;background:var(--cronus-primary);color:var(--cronus-primary-foreground);padding:2px 8px;border-radius:999px;">Popular</span>"#
        } else {
            ""
        };

        format!(
            r#"<div style="background:var(--cronus-surface);border:{border};border-radius:var(--cronus-radius,12px);padding:24px;display:flex;flex-direction:column">
  <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:16px">
    <h3 style="font-weight:600;color:var(--cronus-fg);margin:0">{name}</h3>
    {badge}
  </div>
  <p style="font-size:30px;font-weight:700;color:var(--cronus-fg);margin:0 0 24px">{price}</p>
  <ul style="list-style:none;padding:0;margin:0 0 24px;display:flex;flex-direction:column;gap:8px;flex:1">{features}</ul>
  <button type="button" style="width:100%;background:var(--cronus-primary);color:var(--cronus-primary-foreground);padding:10px 0;border:0;border-radius:10px;font-size:14px;font-weight:600;cursor:pointer">Choose Plan</button>
</div>"#,
            border = border, name = plan.name, badge = badge,
            price = plan.price, features = features.join("\n    "),
        )
    }).collect();

    format!(
        r#"<section style="padding:64px 24px">
  <h2 style="font-size:30px;font-weight:700;text-align:center;margin:0 0 40px;color:var(--cronus-fg)">{title}</h2>
  <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:24px;max-width:56rem;margin:0 auto">
    {plans}
  </div>
</section>"#,
        title = title,
        plans = plans.join("\n    "),
    )
}

pub(super) fn render_cta(section: &SectionNode, _accent: &str, theme: &str) -> String {
    let title = section.title.as_deref().unwrap_or("Get Started");
    let subtitle = section.subtitle.as_deref().unwrap_or("");
    let cta_text = section
        .config
        .get("cta_text")
        .map(|s| s.as_str())
        .unwrap_or("Get Started");
    let cta_link = section
        .config
        .get("cta_link")
        .map(|s| s.as_str())
        .unwrap_or("/signup");
    let cta2_text = section.config.get("cta2_text").map(|s| s.as_str());
    let cta2_link = section
        .config
        .get("cta2_link")
        .map(|s| s.as_str())
        .unwrap_or("");
    let footnote = section.config.get("footnote").map(|s| s.as_str());
    let is_dark = section
        .config
        .get("style")
        .map(|s| s.contains("dark"))
        .unwrap_or(false)
        || theme == "dark";
    let is_light = !is_dark
        && (cta2_text.is_some()
            || section
                .config
                .get("style")
                .map(|s| s.contains("light"))
                .unwrap_or(false));

    if is_light {
        // Light theme CTA: italic title, centered
        let cta2_html = cta2_text.map(|t| format!(
            r#"<a href="{link}" style="display:inline-flex;align-items:center;justify-content:center;padding:16px 48px;border-radius:999px;border:1px solid var(--cronus-border);color:var(--cronus-text);font-weight:700;font-size:18px;text-decoration:none;background:var(--cronus-surface);transition:all 0.2s">{text}</a>"#,
            link=cta2_link, text=t
        )).unwrap_or_default();

        let footnote_html = footnote.or(Some(subtitle)).filter(|s| !s.is_empty()).map(|f| format!(
            r#"<p style="font-size:14px;color:var(--cronus-text-muted);margin-top:32px">{}</p>"#, f
        )).unwrap_or_default();

        return format!(
            r##"<section style="padding:128px 24px;position:relative;overflow:hidden;background:var(--cronus-bg)">
  <div style="position:relative;max-width:960px;margin:0 auto;text-align:center">
    <h2 class="anim reveal" style="font-size:clamp(36px,5vw,72px);font-weight:800;letter-spacing:-0.04em;color:var(--cronus-text);margin-bottom:32px;line-height:1;font-style:italic">{title}</h2>
    <div class="anim anim-d1" style="display:flex;flex-wrap:wrap;align-items:center;justify-content:center;gap:16px">
      <a href="{cta_link}" class="reveal btn-hover" style="display:inline-flex;align-items:center;justify-content:center;padding:16px 48px;border-radius:999px;background:var(--cronus-accent);color:#fff;font-weight:700;font-size:18px;text-decoration:none;transition:all 0.2s" onmouseover="this.style.transform='scale(1.05)'" onmouseout="this.style.transform='scale(1)'">{cta_text}</a>
      {cta2_html}
    </div>
    {footnote_html}
  </div>
</section>"##,
            title = title,
            cta_link = cta_link,
            cta_text = cta_text,
            cta2_html = cta2_html,
            footnote_html = footnote_html,
        );
    }

    // Dark theme CTA — glass card, sentence-case title, logo bar
    let subtitle_html = if subtitle.is_empty() {
        String::new()
    } else {
        format!(
            r#"<p style="font-size:20px;color:#9ca3af;margin-bottom:48px;max-width:600px;margin-left:auto;margin-right:auto;line-height:1.6">{}</p>"#,
            subtitle
        )
    };

    let cta2_html = cta2_text.map(|t| format!(
        r##"<a href="{link}" style="display:inline-flex;align-items:center;justify-content:center;padding:20px 48px;border-radius:16px;background:#2a2a2a;color:white;font-weight:900;font-size:13px;letter-spacing:0.1em;text-transform:uppercase;text-decoration:none;border:0.5px solid rgba(255,255,255,0.15);transition:all 0.2s" onmouseover="this.style.background='#333'" onmouseout="this.style.background='#2a2a2a'">{text}</a>"##,
        link=cta2_link, text=t
    )).unwrap_or_default();

    let footnote_html = footnote
        .map(|f| {
            format!(
                r#"<p style="font-size:14px;color:#6b7280;margin-top:32px">{}</p>"#,
                f
            )
        })
        .unwrap_or_default();

    // Logo bar from items (non-CTA items like "GOLDMAN", "MORGAN", etc.)
    let logo_items: Vec<String> = section.items.iter().filter_map(|item| {
        item.get("title").or_else(|| item.get("name")).map(|name| {
            format!(r#"<span style="font-weight:900;font-size:20px;font-style:italic;letter-spacing:-0.05em;color:white">{}</span>"#, name)
        })
    }).collect();

    let logo_bar_html = if !logo_items.is_empty() {
        format!(
            r#"<div style="padding-top:48px;display:flex;justify-content:center;gap:48px;opacity:0.3;filter:grayscale(100%)">{}</div>"#,
            logo_items.join("\n")
        )
    } else {
        String::new()
    };

    format!(
        r##"<section style="padding:160px 24px;position:relative;overflow:hidden">
  <div class="cronus-cta-card" style="position:relative;max-width:1024px;margin:0 auto;background:rgba(27,27,27,0.5);backdrop-filter:blur(32px);-webkit-backdrop-filter:blur(32px);border:0.5px solid rgba(76,69,70,0.15);border-radius:32px;padding:48px 24px;overflow:hidden">
    <div style="text-align:center">
      <h2 class="anim reveal" style="font-size:clamp(36px,5vw,72px);font-weight:900;letter-spacing:-0.04em;color:white;margin-bottom:24px;line-height:1.1">{title}</h2>
      {subtitle_html}
      <div class="anim anim-d1" style="display:flex;flex-direction:column;align-items:center;justify-content:center;gap:24px">
        <div style="display:flex;flex-direction:column;align-items:center;justify-content:center;gap:24px">
          <a href="{cta_link}" class="reveal" style="display:inline-flex;align-items:center;justify-content:center;padding:20px 48px;border-radius:16px;background:linear-gradient(180deg,#fff,#d4d4d4);color:black;font-weight:900;font-size:13px;letter-spacing:0.1em;text-transform:uppercase;text-decoration:none;transition:transform 0.2s" onmouseover="this.style.transform='scale(1.05)'" onmouseout="this.style.transform='scale(1)'">{cta_text}</a>
          {cta2_html}
        </div>
      </div>
      {footnote_html}
      {logo_bar_html}
    </div>
  </div>
</section>
<style>@media(min-width:768px){{.cronus-cta-card{{padding:96px 64px!important}}}}@media(min-width:768px){{.anim-d1 div{{flex-direction:row!important}}}}</style>"##,
        title = title,
        subtitle_html = subtitle_html,
        cta_link = cta_link,
        cta_text = cta_text,
        cta2_html = cta2_html,
        footnote_html = footnote_html,
        logo_bar_html = logo_bar_html,
    )
}

pub(super) fn render_trusted(section: &SectionNode) -> String {
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
        title = title,
        subtitle = subtitle,
        logos = logos.join("\n        "),
    )
}

pub(super) fn render_footer(section: &SectionNode, theme: &str) -> String {
    let copyright = section
        .config
        .get("copyright")
        .map(|s| s.as_str())
        .unwrap_or("&copy; 2024 Vercel Inc.");
    let has_copyright = section.config.contains_key("copyright");
    let brand = section
        .config
        .get("brand")
        .or_else(|| section.title.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("MONOLITH_OS");
    let is_dark = section
        .config
        .get("style")
        .map(|s| s.contains("dark"))
        .unwrap_or(false)
        || theme == "dark";

    // Light minimal footer: only when copyright is set AND we are NOT in dark theme
    if has_copyright && !is_dark {
        // Light minimal footer
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
                let title = item.get("title").map(|s| s.as_str()).unwrap_or("");
                if !title.is_empty() {
                    all_links.push(title.to_string());
                }
                let desc = item
                    .get("description")
                    .or_else(|| item.get("desc"))
                    .map(|s| s.as_str())
                    .unwrap_or("");
                for link in desc.split(',') {
                    let l = link.trim();
                    if !l.is_empty() {
                        all_links.push(l.to_string());
                    }
                }
            }
        }

        let split_at = if all_links.len() > 2 {
            all_links.len() - 2
        } else {
            all_links.len()
        };
        let left_links: Vec<String> = all_links[..split_at].iter().map(|l| format!(
            r##"<a href="#" style="color:var(--cronus-text-muted);font-size:12px;text-decoration:none;transition:color 0.15s">{}</a>"##, l
        )).collect();
        let right_links_html: Vec<String> = all_links[split_at..].iter().map(|l| format!(
            r##"<a href="#" style="color:var(--cronus-text-muted);font-size:12px;text-decoration:none;transition:color 0.15s">{}</a>"##, l
        )).collect();

        return format!(
            r##"<footer class="anim-fade" style="border-top:1px solid var(--cronus-border);background:var(--cronus-bg);padding:48px 24px">
  <div style="max-width:1280px;margin:0 auto;display:flex;flex-wrap:wrap;justify-content:space-between;align-items:center;gap:16px">
    <div style="display:flex;align-items:center;gap:16px">
      <span style="font-size:12px;color:var(--cronus-text-muted)">{copyright}</span>
      {left_links}
    </div>
    <div style="display:flex;align-items:center;gap:24px">
      {right_links}
    </div>
  </div>
</footer>"##,
            copyright = copyright,
            left_links = left_links.join("\n      "),
            right_links = right_links_html.join("\n      "),
        );
    }

    // Dark theme footer — #0e0e0e bg, Space Grotesk, 6-col grid, brand col span-2
    let brand_desc = section.subtitle.as_deref().unwrap_or("");

    // Build column items from section.items (title = column header, description = comma-separated links)
    let columns_html: String = section.items.iter().map(|item| {
        let col_title = item.get("title").or_else(|| item.get("name")).map(|s| s.as_str()).unwrap_or("Links");
        let desc = item.get("description").or_else(|| item.get("desc")).map(|s| s.as_str()).unwrap_or("");
        let links: Vec<String> = desc.split(',').filter(|l| !l.trim().is_empty()).map(|link| {
            let l = link.trim();
            format!(r##"<a href="#" style="color:rgba(255,255,255,0.4);font-size:10px;text-transform:uppercase;letter-spacing:0.1em;text-decoration:none;transition:color 0.2s" onmouseover="this.style.color='white'" onmouseout="this.style.color='rgba(255,255,255,0.4)'">{}</a>"##, l)
        }).collect();
        format!(
            r#"<div style="display:flex;flex-direction:column;gap:16px">
      <h4 style="color:white;font-size:10px;text-transform:uppercase;letter-spacing:0.1em;font-weight:600;margin-bottom:8px">{}</h4>
      {}
    </div>"#,
            col_title, links.join("\n      ")
        )
    }).collect::<Vec<_>>().join("\n    ");

    // Bottom bar nav links from config
    let nav_links_html: String = if let Some(nav) = section.config.get("nav") {
        nav.split(',').filter(|l| !l.trim().is_empty()).map(|link| {
            let l = link.trim();
            format!(r##"<a href="#" style="color:rgba(255,255,255,0.4);font-size:10px;text-transform:uppercase;letter-spacing:0.1em;text-decoration:none;transition:color 0.2s" onmouseover="this.style.color='white'" onmouseout="this.style.color='rgba(255,255,255,0.4)'">{}</a>"##, l)
        }).collect::<Vec<_>>().join("\n        ")
    } else {
        String::new()
    };

    let brand_desc_html = if brand_desc.is_empty() {
        String::new()
    } else {
        format!(
            r#"<p style="font-size:14px;color:rgba(255,255,255,0.4);line-height:1.7;text-transform:none;letter-spacing:normal;margin-top:16px;max-width:320px">{}</p>"#,
            brand_desc
        )
    };

    format!(
        r##"<style>.cronus-footer-grid{{display:grid;grid-template-columns:repeat(2,1fr);gap:48px}}@media(min-width:768px){{.cronus-footer-grid{{grid-template-columns:2fr repeat(4,1fr)}}.cronus-footer-brand{{grid-column:span 1!important}}}}</style>
<footer style="border-top:0.5px solid rgba(255,255,255,0.1);background:#0e0e0e;font-family:'Space Grotesk',sans-serif;font-size:10px;text-transform:uppercase;letter-spacing:0.1em;color:rgba(255,255,255,0.4)">
  <div style="max-width:1280px;margin:0 auto;padding:80px 32px">
    <div class="cronus-footer-grid">
      <div class="cronus-footer-brand" style="grid-column:span 2">
        <div style="font-size:20px;font-weight:900;letter-spacing:-0.04em;text-transform:uppercase;color:white;margin-bottom:24px">{brand}</div>
        {brand_desc_html}
      </div>
      {columns_html}
    </div>
  </div>
  <div style="border-top:1px solid rgba(255,255,255,0.05);padding:24px 32px">
    <div style="max-width:1280px;margin:0 auto;display:flex;flex-wrap:wrap;justify-content:space-between;align-items:center;gap:16px">
      <span style="color:rgba(255,255,255,0.4)">{copyright}</span>
      <div style="display:flex;gap:24px">
        {nav_links}
      </div>
    </div>
  </div>
</footer>"##,
        brand = brand,
        brand_desc_html = brand_desc_html,
        columns_html = columns_html,
        copyright = copyright,
        nav_links = nav_links_html,
    )
}

pub(super) fn render_faq(section: &SectionNode, _accent: &str) -> String {
    let title = section.title.as_deref().unwrap_or("FAQ");
    let subtitle = section.subtitle.as_deref().unwrap_or("");

    let subtitle_html = if subtitle.is_empty() {
        String::new()
    } else {
        format!(
            r#"<p style="text-align:center;font-size:16px;color:var(--cronus-fg-secondary);margin:0 auto 48px;max-width:600px">{}</p>"#,
            subtitle
        )
    };

    let items: Vec<String> = section.items.iter().map(|item| {
        let q = item.get("title").map(|s| s.as_str()).unwrap_or("Question");
        let a = item.get("description").map(|s| s.as_str()).unwrap_or("");
        format!(
            r#"<details class="cronus-accordion-trigger" style="border-bottom:1px solid var(--cronus-border)">
  <summary style="display:flex;align-items:center;justify-content:space-between;padding:20px 0;cursor:pointer;font-size:16px;font-weight:600;color:var(--cronus-fg);list-style:none;-webkit-appearance:none">
    {q}
    <span style="font-size:20px;color:var(--cronus-fg-secondary);flex-shrink:0;margin-left:16px">+</span>
  </summary>
  <div style="padding:0 0 20px;font-size:15px;color:var(--cronus-fg-secondary);line-height:1.7">{a}</div>
</details>"#,
            q = q, a = a,
        )
    }).collect();

    format!(
        r#"<section style="padding:80px 24px">
  <div style="max-width:var(--cronus-max-w, 1120px);margin:0 auto">
    <h2 style="font-size:32px;font-weight:700;text-align:center;letter-spacing:-0.02em;margin-bottom:16px;color:var(--cronus-fg)">{title}</h2>
    {subtitle_html}
    <div>
      {items}
    </div>
  </div>
</section>"#,
        title = title,
        subtitle_html = subtitle_html,
        items = items.join("\n    "),
    )
}

pub(super) fn render_stats(section: &SectionNode, accent: &str) -> String {
    let title = section.title.as_deref().unwrap_or("Stats");

    let items: Vec<String> = section
        .items
        .iter()
        .map(|item| {
            let label = item.get("title").map(|s| s.as_str()).unwrap_or("Stat");
            let value = item.get("description").map(|s| s.as_str()).unwrap_or("");
            format!(
                r#"<div class="text-center reveal">
  <p class="text-4xl font-bold text-{accent}-400 tabular-nums">{value}</p>
  <p class="mt-2 text-sm text-neutral-500 font-mono uppercase tracking-wider">{label}</p>
</div>"#,
                label = label,
                value = value,
                accent = accent,
            )
        })
        .collect();

    format!(
        r#"<section class="py-16">
  <h2 class="text-3xl font-bold text-center mb-10">{title}</h2>
  <div class="grid grid-cols-2 md:grid-cols-4 gap-8 max-w-4xl mx-auto">
    {items}
  </div>
</section>"#,
        title = title,
        items = items.join("\n    "),
    )
}

pub(super) fn render_page_header_section(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("Page Title");
    let subtitle = section.subtitle.as_deref().unwrap_or("");
    let eyebrow = section
        .config
        .get("eyebrow")
        .map(|s| s.as_str())
        .unwrap_or("");
    let style_hint = section
        .config
        .get("style")
        .map(|s| s.as_str())
        .unwrap_or("");
    let is_dark_explicit = style_hint.contains("dark")
        || section
            .config
            .get("theme")
            .map(|s| s == "dark")
            .unwrap_or(false);

    // Static colors for explicitly-dark headers — use theme tokens
    let t = crate::theme::get();
    let sub_color_dark = format!("{}80", t.on_surface);
    let (title_color, sub_color, eyebrow_color, btn_bg, btn_color) = if is_dark_explicit {
        (
            t.on_surface.as_str(),
            sub_color_dark.as_str(),
            t.primary.as_str(),
            t.primary.as_str(),
            "#002e69",
        )
    } else {
        ("#000", "#71717a", "#3b82f6", "#000", "#fff")
    };

    let eyebrow_html = if eyebrow.is_empty() {
        String::new()
    } else {
        format!(
            r#"<span data-ph-eyebrow style="font-size:10px;font-weight:500;text-transform:uppercase;letter-spacing:0.3em;color:{};display:block;margin-bottom:8px">{}</span>"#,
            eyebrow_color, eyebrow
        )
    };

    let subtitle_html = if subtitle.is_empty() {
        String::new()
    } else {
        format!(
            r#"<p data-ph-sub style="font-size:14px;color:{};margin:8px 0 0;max-width:32em;line-height:1.6">{}</p>"#,
            sub_color, subtitle
        )
    };

    // When not explicitly dark, inject a small script that checks the body background
    // at runtime and flips colors if the page is dark-themed.
    let auto_dark_js = if is_dark_explicit {
        String::new()
    } else {
        crate::security::mark_kernel_scripts(
            r#"<script>(function(){var b=getComputedStyle(document.body).backgroundColor;if(!b||b==='rgba(0, 0, 0, 0)')b='';if(b){var m=b.match(/\d+/g);if(m&&m.length>=3){var lum=(parseInt(m[0])*299+parseInt(m[1])*587+parseInt(m[2])*114)/1000;if(lum<50){var w=document.querySelector('[data-ph-wrapper]');if(w){var h=w.querySelector('h2');if(h)h.style.color='#e2e2e2';var s=w.querySelector('[data-ph-sub]');if(s)s.style.color='rgba(226,226,226,0.5)';var e=w.querySelector('[data-ph-eyebrow]');if(e)e.style.color='#adc6ff'}}}}})();</script>"#,
        )
    };

    format!(
        r##"<div data-ph-wrapper style="padding:80px 0 48px">
  <div>
    {eyebrow}
    <h2 style="font-size:clamp(32px,5vw,48px);font-weight:700;letter-spacing:-0.04em;color:{title_color};margin:0;line-height:1.1">{title}</h2>
    {subtitle_html}
  </div>
</div>{auto_dark_js}"##,
        eyebrow = eyebrow_html,
        title_color = title_color,
        title = title,
        subtitle_html = subtitle_html,
        auto_dark_js = auto_dark_js,
    )
}

pub(super) fn render_promo(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("");
    let subtitle = section.subtitle.as_deref().unwrap_or("");

    // Badge: from config or first item with "badge" key
    let badge_text = section
        .config
        .get("badge")
        .or_else(|| section.items.iter().find_map(|i| i.get("badge")))
        .map(|s| s.as_str());

    let cta_text = section.config.get("cta_text").map(|s| s.as_str());
    let cta_link = section
        .config
        .get("cta_link")
        .map(|s| s.as_str())
        .unwrap_or("");

    // If config has span:2, this section is intended to span 2 grid columns in parent layout

    let badge_html = badge_text.map(|b| {
        format!(r#"<span style="display:inline-block;font-size:10px;font-weight:600;text-transform:uppercase;letter-spacing:0.1em;padding:4px 12px;border-radius:999px;background:rgba(255,255,255,0.2);color:white">{b}</span>"#, b=b)
    }).unwrap_or_default();

    let subtitle_html = if subtitle.is_empty() {
        String::new()
    } else {
        format!(
            r#"<p style="font-size:14px;color:#a1a1aa;max-width:480px;line-height:1.6">{subtitle}</p>"#,
            subtitle = subtitle
        )
    };

    let cta_html = cta_text.map(|ct| {
        format!(r##"<a href="{link}" style="display:inline-flex;align-items:center;justify-content:center;padding:10px 24px;border-radius:999px;background:white;color:black;font-weight:700;font-size:14px;text-decoration:none;transition:opacity 0.2s" onmouseover="this.style.opacity='0.9'" onmouseout="this.style.opacity='1'">{ct}</a>"##, link=cta_link, ct=ct)
    }).unwrap_or_default();

    format!(
        r##"<div style="background:black;color:white;border-radius:12px;padding:32px;position:relative;overflow:hidden;display:flex;flex-direction:column;gap:16px"><div style="position:absolute;right:0;top:0;bottom:0;width:50%;background:radial-gradient(ellipse at 80% 50%,rgba(0,111,240,0.15),transparent 70%);pointer-events:none"></div><div style="position:relative;display:flex;flex-direction:column;gap:16px">{badge_html}<h2 style="font-size:30px;font-weight:700;letter-spacing:-0.02em;color:white;margin:0">{title}</h2>{subtitle_html}{cta_html}</div></div>"##,
        badge_html = badge_html,
        title = title,
        subtitle_html = subtitle_html,
        cta_html = cta_html,
    )
}

// ══════════════════════════════════════════════════
// LIGHT-THEME SECTION RENDERERS (Geist-inspired)
// ══════════════════════════════════════════════════

pub(super) fn render_info_bar(section: &SectionNode) -> String {
    let title = section.title.as_deref().unwrap_or("");
    let subtitle = section.subtitle.as_deref().unwrap_or("");

    let config_icon = section.config.get("icon").map(|s| s.as_str());

    let shield_svg = r#"<svg width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>"#;

    let icon_html = if let Some(icon) = config_icon {
        format!(
            r#"<span style="display:flex;align-items:center;justify-content:center;width:24px;height:24px">{icon}</span>"#,
            icon = icon
        )
    } else if section
        .items
        .iter()
        .any(|i| i.get("type").map(|t| t == "icon").unwrap_or(false))
    {
        let icon_item = section
            .items
            .iter()
            .find(|i| i.get("type").map(|t| t == "icon").unwrap_or(false))
            .unwrap();
        let icon_val = icon_item
            .get("title")
            .or_else(|| icon_item.get("name"))
            .map(|s| s.as_str())
            .unwrap_or("");
        format!(
            r#"<span style="display:flex;align-items:center;justify-content:center;width:24px;height:24px">{icon_val}</span>"#,
            icon_val = icon_val
        )
    } else {
        format!(
            r#"<span style="display:flex;align-items:center;justify-content:center;width:24px;height:24px">{shield}</span>"#,
            shield = shield_svg
        )
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
        format!(
            r#"<span style="font-size:14px;font-weight:700;color:#000">{title}</span>"#,
            title = title
        )
    };

    let subtitle_html = if subtitle.is_empty() {
        String::new()
    } else {
        format!(
            r#"<span style="font-size:12px;color:#5e5e5e">{subtitle}</span>"#,
            subtitle = subtitle
        )
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
        icon_html = icon_html,
        title_html = title_html,
        subtitle_html = subtitle_html,
        links = links.join("\n    "),
    )
}
