//! Hero section renderers
use crate::parser::SectionNode;

/// `open` + `content` + `close`, or nothing when `content` is empty.
/// Heroes never invent a heading, subtitle or CTA the `.cronus` file did not declare.
fn element(open: &str, content: &str, close: &str) -> String {
    if content.is_empty() {
        String::new()
    } else {
        format!("{open}{content}{close}")
    }
}

pub(super) fn render_hero(section: &SectionNode, accent: &str, theme: &str) -> String {
    let title = section.title.as_deref().unwrap_or("");
    let subtitle = section.subtitle.as_deref().unwrap_or("");
    let badge = section.config.get("badge").map(|s| s.as_str());
    let cta_primary = section
        .config
        .get("cta_text")
        .or(section.config.get("cta"))
        .map(|s| s.as_str())
        .unwrap_or("");
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

    // Convert accent name to hex color (pass through if already hex)
    let accent_hex = if accent.starts_with('#') {
        accent
    } else {
        match accent {
            "blue" => "#2563eb",
            "indigo" => "#6366f1",
            "amber" => "#f59e0b",
            "emerald" => "#10b981",
            "rose" => "#f43f5e",
            "violet" => "#8b5cf6",
            "sky" => "#0ea5e9",
            "orange" => "#f97316",
            "red" => "#ef4444",
            "green" => "#22c55e",
            "purple" => "#a855f7",
            "pink" => "#ec4899",
            "cyan" => "#06b6d4",
            "teal" => "#14b8a6",
            _ => "#2563eb",
        }
    };

    // Extract badge from items if not in config
    let badge_text = badge.or_else(|| {
        section
            .items
            .iter()
            .find(|i| {
                i.get("badge").is_some() || i.get("title").map(|t| t.len() < 60).unwrap_or(false)
            })
            .and_then(|i| i.get("badge").or(i.get("title")))
            .map(|s| s.as_str())
    });

    // Detect light theme: explicit style:light, or cta2 presence ONLY when global theme is not dark
    let style_hint = section
        .config
        .get("style")
        .map(|s| s.as_str())
        .unwrap_or("");
    let is_dark = style_hint.contains("dark") || theme == "dark";
    let is_light = style_hint.contains("light") || (!is_dark && cta2_text.is_some());

    if is_light {
        return render_developer_landing_hero(
            section,
            title,
            subtitle,
            badge_text,
            cta_primary,
            cta_link,
            cta2_text,
            cta2_link,
            accent_hex,
        );
    }

    // === Dark theme hero ===

    // Extract background image from items with role:background
    let bg_image_url = section
        .items
        .iter()
        .find(|i| i.get("role").map(|s| s.as_str()) == Some("background"))
        .and_then(|i| {
            i.get("title")
                .or(i.get("image"))
                .or(i.get("src"))
                .or(i.get("url"))
        })
        .map(|s| s.as_str())
        .unwrap_or("");

    // Detect terminal items early for layout branching
    let has_terminal_items = section.items.iter().any(|i| {
        let t = i
            .get("_type")
            .or(i.get("type"))
            .map(|s| s.as_str())
            .unwrap_or("");
        matches!(t, "line" | "output" | "success" | "prompt" | "terminal")
    });

    // Two-column layout: no background image AND no terminal items (Ultima/Fintech style)
    let is_two_col = bg_image_url.is_empty() && !has_terminal_items;

    if is_two_col {
        return render_two_col_hero(
            section,
            title,
            subtitle,
            badge_text,
            cta_primary,
            cta_link,
            cta2_text,
            cta2_link,
            accent_hex,
        );
    }

    // === Centered layout — Premium design (bg image or terminal) ===

    // Resolve accent to Tailwind color family
    let (tw_accent, tw_accent_dark, tw_accent_glow) = match accent_hex {
        "#CC0000" | "#ef4444" => ("red", "red-950", "204,0,0"),
        "#f97316" => ("orange", "orange-950", "249,115,22"),
        "#10b981" => ("emerald", "emerald-950", "16,185,129"),
        "#6366f1" => ("indigo", "indigo-950", "99,102,241"),
        "#8b5cf6" => ("violet", "violet-950", "139,92,246"),
        "#0ea5e9" => ("sky", "sky-950", "14,165,233"),
        "#2563eb" => ("blue", "blue-950", "37,99,235"),
        _ => ("red", "red-950", "204,0,0"),
    };

    let bg_image_html = if !bg_image_url.is_empty() {
        format!(
            r#"<div class="absolute inset-0" style="z-index:-1"><img src="{}" alt="" class="w-full h-full object-cover opacity-60" /><div class="absolute inset-0 bg-gradient-to-b from-transparent via-black/40 to-[#020202]"></div></div>"#,
            bg_image_url
        )
    } else {
        String::new()
    };

    // Version badge — pill with accent dot + pulsing indicator
    let badge_html = badge_text.map(|b| format!(
        r#"<div class="[animation:fadeInUp_0.8s_ease-out_0.1s_both] inline-flex items-center gap-2 px-3 py-1 rounded-full bg-{accent_dark}/10 border border-{accent}-500/20 shadow-[0_0_15px_rgba({glow},0.15)] mb-8">
      <span class="flex h-1.5 w-1.5 relative"><span class="absolute inline-flex h-full w-full rounded-full bg-{accent}-400 opacity-75 animate-ping"></span><span class="relative inline-flex h-1.5 w-1.5 rounded-full bg-{accent}-500"></span></span>
      <span class="text-[10px] font-medium tracking-wider uppercase text-{accent}-200">{text}</span>
    </div>"#, text = b, accent = tw_accent, accent_dark = tw_accent_dark, glow = tw_accent_glow
    )).unwrap_or_default();

    // CTA2 — ghost button
    let cta2_html = cta2_text.map(|t| format!(
        r#"<a href="{link}" class="[animation:fadeInUp_0.8s_ease-out_0.5s_both] inline-flex items-center gap-2 px-8 py-3 rounded-full border border-white/10 text-white text-sm font-medium hover:bg-white/[0.05] transition-colors">{text}</a>"#,
        link = cta2_link, text = t
    )).unwrap_or_default();

    // Terminal mockup — glass panel with blur, 3 dots header, syntax-colored lines
    let terminal_html = if has_terminal_items {
        let terminal_title = section
            .items
            .iter()
            .find(|i| {
                let t = i
                    .get("_type")
                    .or(i.get("type"))
                    .map(|s| s.as_str())
                    .unwrap_or("");
                t == "terminal" || i.get("style").map(|s| s.as_str()) == Some("terminal")
            })
            .and_then(|i| i.get("description").or(i.get("title")))
            .map(|s| s.as_str())
            .unwrap_or("terminal");

        let mut terminal_lines = String::new();
        let mut line_num = 0u32;
        for item in &section.items {
            let item_type = item
                .get("_type")
                .or(item.get("type"))
                .map(|s| s.as_str())
                .unwrap_or("");
            let text = item.get("title").map(|s| s.as_str()).unwrap_or("");
            match item_type {
                "terminal" | "line" => {
                    line_num += 1;
                    // Parse command: "monolith deploy --project hyper-cluster-01"
                    let parts: Vec<&str> = text.trim_start_matches("$ ").splitn(2, ' ').collect();
                    let cmd_name = parts.first().unwrap_or(&"");
                    let cmd_rest = parts.get(1).unwrap_or(&"");
                    // Split rest into action and flags
                    let rest_parts: Vec<&str> = cmd_rest.splitn(2, ' ').collect();
                    let action = rest_parts.first().unwrap_or(&"");
                    let flags_raw = rest_parts.get(1).unwrap_or(&"");
                    // Render flags with quoted strings in tertiary color (#e5e2e1)
                    let mut flags_html = String::new();
                    if !flags_raw.is_empty() {
                        let mut in_quote = false;
                        let mut buf = String::new();
                        for ch in flags_raw.chars() {
                            if ch == '"' {
                                if in_quote {
                                    flags_html.push_str(&format!(
                                        r#"<span style="color:#e5e2e1">&quot;{}&quot;</span>"#,
                                        buf
                                    ));
                                    buf.clear();
                                    in_quote = false;
                                } else {
                                    if !buf.is_empty() {
                                        flags_html.push_str(&format!(
                                            r#"<span style="color:rgba(255,255,255,0.4)">{}</span>"#, buf
                                        ));
                                        buf.clear();
                                    }
                                    in_quote = true;
                                }
                            } else {
                                buf.push(ch);
                            }
                        }
                        if !buf.is_empty() {
                            flags_html.push_str(&format!(
                                r#"<span style="color:rgba(255,255,255,0.4)">{}</span>"#,
                                buf
                            ));
                        }
                    }
                    terminal_lines.push_str(&format!(
                        r#"<div style="display:flex;gap:16px;margin-bottom:4px"><span style="color:rgba(255,255,255,0.3)">{num:02}</span><span style="color:rgba(255,255,255,0.5)">{name}</span> <span style="color:#fff;font-weight:700">{action}</span> {flags}</div>"#,
                        num=line_num, name=cmd_name, action=action, flags=flags_html
                    ));
                }
                "output" => {
                    line_num += 1;
                    // Detect "URL: https://..." lines — render label muted, URL underlined tertiary
                    if text.starts_with("URL:") {
                        let url_part = text["URL:".len()..].trim();
                        terminal_lines.push_str(&format!(
                            r#"<div style="display:flex;gap:16px;margin-bottom:4px"><span style="color:rgba(255,255,255,0.3)">{num:02}</span><span style="color:rgba(255,255,255,0.5)">URL:</span> <span style="color:#e5e2e1;text-decoration:underline">{url}</span></div>"#,
                            num=line_num, url=url_part
                        ));
                    } else {
                        terminal_lines.push_str(&format!(
                            r#"<div style="display:flex;gap:16px;margin-bottom:4px"><span style="color:rgba(255,255,255,0.3)">{num:02}</span><span style="color:rgba(255,255,255,0.4)">{text}</span></div>"#,
                            num=line_num, text=text
                        ));
                    }
                }
                "prompt" => {
                    line_num += 1;
                    let answer = item.get("answer").map(|s| s.as_str()).unwrap_or("");
                    terminal_lines.push_str(&format!(
                        r#"<div style="display:flex;gap:16px;margin-bottom:4px"><span style="color:rgba(255,255,255,0.3)">{num:02}</span><span style="color:#e5e5e5">{text}</span> <span style="color:#60a5fa">{answer}</span></div>"#,
                        num=line_num, text=text, answer=answer
                    ));
                }
                "success" => {
                    line_num += 1;
                    // Split trailing time pattern (e.g. "1.4s", "200ms") into muted span
                    let trimmed = text.trim_end();
                    let time_sep = trimmed.rfind(' ');
                    let (main_text, time_html) = if let Some(pos) = time_sep {
                        let candidate = &trimmed[pos + 1..];
                        // Match patterns like "1.4s", "200ms", "0.3s"
                        let is_time = candidate.ends_with('s')
                            && candidate[..candidate.len() - 1]
                                .replace("ms", "")
                                .replace('.', "")
                                .chars()
                                .all(|c| c.is_ascii_digit());
                        if is_time && !candidate.is_empty() {
                            (
                                &trimmed[..pos],
                                format!(
                                    r#" <span style="color:rgba(255,255,255,0.3)">{}</span>"#,
                                    candidate
                                ),
                            )
                        } else {
                            (trimmed, String::new())
                        }
                    } else {
                        (trimmed, String::new())
                    };
                    terminal_lines.push_str(&format!(
                        r#"<div style="display:flex;gap:16px;margin-bottom:4px"><span style="color:rgba(255,255,255,0.3)">{num:02}</span><span style="color:#fff;font-weight:700">✓ {main}</span>{time}</div>"#,
                        num=line_num, main=main_text, time=time_html
                    ));
                }
                _ => {}
            }
        }
        // No blinking cursor for clean look

        format!(
            r##"<div class="anim-slide-up d6" style="max-width:896px;width:100%;margin:96px auto 0;background:rgba(31,31,31,0.7);backdrop-filter:blur(20px);-webkit-backdrop-filter:blur(20px);border-radius:16px;overflow:hidden;border:0.5px solid rgba(255,255,255,0.1);box-shadow:0 32px 64px rgba(0,0,0,0.5)">
      <div style="display:flex;align-items:center;justify-content:space-between;padding:12px 16px;background:rgba(53,53,53,0.5);border-bottom:1px solid rgba(255,255,255,0.05)">
        <div style="display:flex;gap:6px">
          <span style="width:10px;height:10px;border-radius:50%;background:rgba(255,255,255,0.1)"></span>
          <span style="width:10px;height:10px;border-radius:50%;background:rgba(255,255,255,0.1)"></span>
          <span style="width:10px;height:10px;border-radius:50%;background:rgba(255,255,255,0.1)"></span>
        </div>
        <span style="font-family:'Space Grotesk',monospace;font-size:10px;color:rgba(255,255,255,0.3);text-transform:uppercase;letter-spacing:0.15em">{title}</span>
        <div style="width:48px"></div>
      </div>
      <div style="padding:24px;font-family:'SF Mono','JetBrains Mono',monospace;font-size:13px;line-height:1.7;color:#e5e5e5">
        {lines}
      </div>
    </div>"##,
            title = terminal_title,
            lines = terminal_lines,
        )
    } else {
        String::new()
    };

    // Split title: last word(s) get muted color for visual contrast
    let title_parts: Vec<&str> = title.rsplitn(2, ' ').collect();
    let title_html = if title_parts.len() == 2 {
        format!(
            r#"{}<br><span class="text-neutral-500">{}</span>"#,
            title_parts[1], title_parts[0]
        )
    } else {
        title.to_string()
    };

    format!(
        r##"<section class="relative overflow-hidden min-h-[90vh] flex flex-col items-center justify-center" style="z-index:2;background:transparent">
  {bg_image_html}
  <div class="absolute inset-0 mx-auto max-w-7xl border-r border-l border-white/[0.03]" style="background-image:linear-gradient(to right, rgba(255,255,255,0.03) 1px, transparent 1px);background-size:16.66% 100%"></div>
  <div class="relative z-10 max-w-4xl w-full mx-auto px-6 py-32 text-center flex flex-col items-center">
    {badge_html}
    {title_html}
    {subtitle_html}
    <div class="[animation:fadeInUp_0.8s_ease-out_0.4s_both] flex flex-col items-center justify-center gap-4">
      {cta_html}
      {cta2_html}
    </div>
    {terminal_html}
  </div>
</section>"##,
        bg_image_html = bg_image_html,
        badge_html = badge_html,
        title_html = element(
            r#"<h1 class="[animation:fadeInUp_0.8s_ease-out_0.2s_both] text-5xl md:text-7xl font-medium leading-[0.95] tracking-tight text-white mb-6">
      "#,
            &title_html,
            "\n    </h1>",
        ),
        subtitle_html = element(
            r#"<p class="[animation:fadeInUp_0.8s_ease-out_0.3s_both] font-light leading-relaxed max-w-xl mx-auto text-lg text-neutral-400 tracking-tight mb-10">"#,
            subtitle,
            "</p>",
        ),
        cta_html = element(
            &format!(
                r#"<a href="{cta_link}" class="bg-white flex font-medium gap-2 group hover:bg-gray-200 items-center px-8 py-3 rounded-full text-black text-sm transition-all">
        <span>"#
            ),
            cta_primary,
            r#"</span>
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" class="group-hover:translate-x-0.5 transition-transform"><path d="M5 12h14m-7-7l7 7l-7 7" stroke-linejoin="round" stroke-width="2" stroke="currentColor" fill="none" stroke-linecap="round"></path></svg>
      </a>"#,
        ),
        cta2_html = cta2_html,
        terminal_html = terminal_html,
    )
}

/// Two-column dark hero: Ultima/Fintech style — left text, right visual, gradient title, mesh bg
pub(super) fn render_two_col_hero(
    section: &SectionNode,
    title: &str,
    subtitle: &str,
    badge_text: Option<&str>,
    cta_primary: &str,
    cta_link: &str,
    cta2_text: Option<&str>,
    cta2_link: &str,
    accent_hex: &str,
) -> String {
    // Badge with animated pulse dot
    let badge_html = badge_text.map(|b| format!(
        r#"<div class="anim-fade d1" style="display:inline-flex;align-items:center;gap:8px;padding:6px 14px;border-radius:9999px;background:rgba(27,27,27,0.5);backdrop-filter:blur(8px);border:0.5px solid rgba(76,69,70,0.2);margin-bottom:32px">
      <span style="width:7px;height:7px;border-radius:50%;background:#22c55e;box-shadow:0 0 6px #22c55e80;animation:pulse-dot 2s ease-in-out infinite"></span>
      <span style="font-size:11px;font-weight:500;letter-spacing:0.08em;text-transform:uppercase;color:rgba(255,255,255,0.7)">{text}</span>
    </div>"#, text = b
    )).unwrap_or_default();

    // Title with gradient on text after last comma or period
    let title_html = if title.is_empty() {
        String::new()
    } else {
        let last_sep = title.rfind(',').or_else(|| title.rfind('.'));
        if let Some(pos) = last_sep {
            let before = &title[..=pos];
            let after = &title[pos + 1..];
            if after.trim().is_empty() {
                format!(r#"<span style="color:#ffffff">{}</span>"#, title)
            } else {
                format!(
                    r#"<span style="color:#ffffff">{before}</span><span style="background:linear-gradient(135deg,#adc6ff 0%,#c2c1ff 50%,#e9b3ff 100%);-webkit-background-clip:text;-webkit-text-fill-color:transparent;background-clip:text">{after}</span>"#,
                    before = before,
                    after = after
                )
            }
        } else {
            format!(r#"<span style="color:#ffffff">{}</span>"#, title)
        }
    };

    // CTA2 ghost button
    let cta2_html = cta2_text.map(|t| format!(
        r#"<a href="{link}" class="anim-scale d5" style="display:inline-flex;align-items:center;justify-content:center;padding:14px 36px;border-radius:999px;border:0.5px solid rgba(255,255,255,0.15);background:rgba(255,255,255,0.04);color:#ffffff;font-weight:600;font-size:16px;text-decoration:none;backdrop-filter:blur(12px);transition:all 0.2s" onmouseover="this.style.background='rgba(255,255,255,0.08)'" onmouseout="this.style.background='rgba(255,255,255,0.04)'">{text}</a>"#,
        link = cta2_link, text = t
    )).unwrap_or_default();

    // Credit-card visual is opt-in (`card_brand` / `card_number` / `card_holder`).
    // Never invent a fintech card when the .cronus file did not ask.
    let has_card = section.config.contains_key("card_brand")
        || section.config.contains_key("card_number")
        || section.config.contains_key("card_holder");
    let card_brand = section
        .config
        .get("card_brand")
        .cloned()
        .or_else(|| section.config.get("brand").cloned())
        .unwrap_or_default();
    let card_number = section
        .config
        .get("card_number")
        .cloned()
        .unwrap_or_default();
    let card_holder = section
        .config
        .get("card_holder")
        .cloned()
        .unwrap_or_default();

    let right_col = if has_card {
        format!(
            r#"<div style="display:flex;align-items:center;justify-content:center;min-height:320px">
      <div style="width:100%;max-width:420px;perspective:1000px;position:relative">
        <div style="position:absolute;top:50%;left:50%;transform:translate(-50%,-50%);width:80%;height:80%;background:radial-gradient(circle,{accent}20 0%,transparent 70%);filter:blur(60px);animation:pulse 4s ease-in-out infinite;z-index:0"></div>
        <div class="anim-scale d4" style="position:relative;z-index:1;width:100%;min-height:340px;border-radius:16px;backdrop-filter:blur(32px);border:1px solid var(--cronus-border,rgba(255,255,255,0.08));overflow:hidden;background:var(--cronus-surface,rgba(255,255,255,0.04))">
          <div style="position:relative;z-index:2;padding:28px 28px 24px;min-height:320px;display:flex;flex-direction:column;justify-content:space-between">
            <span style="font-size:18px;font-weight:800;letter-spacing:-0.03em;color:var(--cronus-fg,#fff)">{card_brand}</span>
            <div style="font-size:1.5rem;font-weight:500;letter-spacing:0.16em;color:var(--cronus-fg,#fff);font-variant-numeric:tabular-nums">{card_number}</div>
            <div>
              <div style="font-size:10px;letter-spacing:0.08em;text-transform:uppercase;color:var(--cronus-fg-secondary,rgba(255,255,255,0.45));margin-bottom:4px">Holder</div>
              <div style="font-size:12px;font-weight:600;letter-spacing:0.08em;text-transform:uppercase;color:var(--cronus-fg,#fff)">{card_holder}</div>
            </div>
          </div>
        </div>
      </div>
    </div>"#,
            accent = accent_hex,
            card_brand = card_brand,
            card_number = card_number,
            card_holder = card_holder,
        )
    } else {
        String::new()
    };
    let grid = if has_card {
        "minmax(0,1.2fr) minmax(0,0.8fr)"
    } else {
        "1fr"
    };
    let min_h = if has_card { "min-height:80vh;" } else { "" };

    format!(
        r##"<style>@keyframes pulse-dot{{0%,100%{{opacity:1;transform:scale(1)}}50%{{opacity:0.5;transform:scale(0.85)}}}}</style>
<section style="position:relative;overflow:hidden;{min_h}display:flex;align-items:center;background:var(--cronus-bg,#0a0a0a)">
  <div style="position:absolute;inset:0;z-index:0;pointer-events:none">
    <div style="position:absolute;top:-20%;left:-10%;width:60%;height:60%;background:radial-gradient(ellipse at center,color-mix(in srgb,var(--cronus-primary,{accent}) 12%,transparent) 0%,transparent 70%)"></div>
    <div style="position:absolute;bottom:0;left:0;right:0;height:1px;background:linear-gradient(90deg,transparent,var(--cronus-border,rgba(255,255,255,0.06)),transparent)"></div>
  </div>
  <div style="position:relative;z-index:10;max-width:1280px;width:100%;margin:0 auto;padding:120px 24px 80px;display:grid;grid-template-columns:{grid};align-items:center;gap:48px">
    <div style="display:flex;flex-direction:column;align-items:flex-start">
      {badge_html}
      {title_html}
      {subtitle_html}
      <div class="anim-slide-up d4" style="display:flex;flex-wrap:wrap;align-items:center;gap:16px">
        {cta_html}
        {cta2_html}
      </div>
    </div>
    {right_col}
  </div>
</section>"##,
        min_h = min_h,
        accent = accent_hex,
        grid = grid,
        badge_html = badge_html,
        title_html = element(
            r#"<h1 class="anim-slide-up d2" style="font-size:clamp(48px,8vw,96px);font-weight:800;letter-spacing:-0.04em;line-height:0.95;margin:0 0 28px 0">
        "#,
            &title_html,
            "\n      </h1>",
        ),
        subtitle_html = element(
            r#"<p class="anim-slide-up d3" style="max-width:560px;margin:0 0 40px;font-size:clamp(16px,1.6vw,19px);font-weight:400;color:var(--cronus-fg-secondary,rgba(255,255,255,0.55));line-height:1.7;text-align:left">"#,
            subtitle,
            "</p>",
        ),
        cta_html = element(
            &format!(
                r#"<a href="{cta_link}" style="display:inline-flex;align-items:center;justify-content:center;padding:14px 36px;border-radius:999px;background:var(--cronus-primary,{accent_hex});color:var(--cronus-primary-foreground,#fff);font-weight:700;font-size:16px;text-decoration:none;transition:transform 0.2s,box-shadow 0.2s" onmouseover="this.style.transform='translateY(-1px)'" onmouseout="this.style.transform='translateY(0)'">"#
            ),
            cta_primary,
            "</a>",
        ),
        cta2_html = cta2_html,
        right_col = right_col,
    )
}

/// Developer-landing hero: light theme, two-column grid with terminal window
pub(super) fn render_developer_landing_hero(
    section: &SectionNode,
    title: &str,
    subtitle: &str,
    badge_text: Option<&str>,
    cta_primary: &str,
    cta_link: &str,
    cta2_text: Option<&str>,
    cta2_link: &str,
    accent_hex: &str,
) -> String {
    // Badge
    let badge_html = badge_text.map(|b| format!(
        r#"<div class="anim anim-fade d1" style="display:inline-flex;align-items:center;gap:8px;padding:4px 12px;border-radius:999px;background:var(--cronus-border);border:1px solid var(--cronus-border);margin-bottom:24px">
      <span class="pulse-glow" style="width:8px;height:8px;border-radius:50%;background:var(--cronus-accent)"></span>
      <span style="font-size:12px;font-weight:500;letter-spacing:0.05em;text-transform:uppercase;color:var(--cronus-text)">{text}</span>
    </div>"#, text=b
    )).unwrap_or_default();

    // Split title by periods for line breaks (e.g. "Develop. Preview. Ship.")
    let title_lines: Vec<&str> = if title.contains('.') {
        title
            .split('.')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        title.split_whitespace().collect()
    };
    let title_html: String = title_lines
        .iter()
        .enumerate()
        .map(|(i, word)| {
            if i < title_lines.len() - 1 {
                format!("{}.<br>", word)
            } else {
                format!("{}.", word)
            }
        })
        .collect();

    // CTA2 (outline button)
    let cta2_html = cta2_text.map(|t| format!(
        r#"<a href="{link}" class="anim-scale d5 btn-hover" style="display:inline-flex;align-items:center;justify-content:center;padding:14px 32px;border-radius:999px;border:1px solid var(--cronus-border);color:var(--cronus-text);font-weight:700;font-size:16px;text-decoration:none;background:var(--cronus-surface);transition:all 0.2s">{text}</a>"#,
        link=cta2_link, text=t
    )).unwrap_or_default();

    // Terminal window HTML — built dynamically from section items
    let terminal_title = section
        .items
        .iter()
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
                let color = item
                    .get("color")
                    .map(|s| match s.as_str() {
                        "blue" => accent_hex,
                        "green" => "#28c840",
                        "yellow" => "#febc2e",
                        "red" => "#ff5f57",
                        _ => "#666",
                    })
                    .unwrap_or("#666");
                terminal_lines.push_str(&format!(
                    r#"<div style="color:{};margin-top:4px">{}</div>"#,
                    color, text
                ));
            }
            "prompt" => {
                let answer = item.get("answer").map(|s| s.as_str()).unwrap_or("");
                terminal_lines.push_str(&format!(
                    r#"<div><span style="color:#666">?</span> <span style="color:#e5e5e5">{text}</span> <span style="color:{accent}">{answer}</span></div>"#,
                    text=text, answer=answer, accent=accent_hex
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
    let chips_html = if chips.is_empty() {
        String::new()
    } else {
        format!(
            r#"<div style="position:absolute;bottom:-24px;right:-24px;display:flex;flex-direction:column;gap:8px;z-index:20">{}</div>"#,
            chips.join("\n")
        )
    };

    // Wrap terminal + chips in relative container
    let terminal_with_chips = format!(
        r#"<div style="position:relative">{}{}</div>"#,
        terminal_html, chips_html
    );

    // Stat cards embedded in hero (role:stat items)
    let stat_items: Vec<&std::collections::HashMap<String, String>> = section
        .items
        .iter()
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

    let h1_html = element(
        r#"<h1 class="anim anim-d1" style="font-size:clamp(48px,8vw,96px);font-weight:800;letter-spacing:-0.05em;color:var(--cronus-text);line-height:0.9;margin-bottom:32px">
      "#,
        &title_html,
        "\n    </h1>",
    );
    let cta_open = format!(
        r#"<a href="{cta_link}" class="anim-scale d4 btn-hover" style="display:inline-flex;align-items:center;justify-content:center;gap:8px;padding:14px 32px;border-radius:999px;background:var(--cronus-accent);color:#fff;font-weight:700;font-size:16px;text-decoration:none;transition:all 0.2s" onmouseover="this.style.transform='scale(1.05)'" onmouseout="this.style.transform='scale(1)'">"#
    );

    // Detect if hero has actual terminal content (line/output/prompt/success items)
    let has_terminal = section.items.iter().any(|i| {
        matches!(
            i.get("_type").map(|s| s.as_str()),
            Some("line") | Some("output") | Some("prompt") | Some("success")
        )
    });

    if !has_terminal {
        // Centered hero layout without terminal (no terminal items found)
        return format!(
            r##"<section style="position:relative;overflow:hidden;min-height:80vh;padding:96px 24px 80px;display:flex;flex-direction:column;align-items:center;justify-content:center;background:var(--cronus-bg);background-image:linear-gradient(to right,rgba(198,198,198,0.1) 1px,transparent 1px),linear-gradient(to bottom,rgba(198,198,198,0.1) 1px,transparent 1px);background-size:40px 40px">
  <div class="prism-glow" style="position:absolute;inset:0;pointer-events:none"></div>
  <div style="position:relative;z-index:10;max-width:var(--cronus-max-w);margin:0 auto;padding:0 24px;text-align:center">
    {badge_html}
    {title_html}
    {subtitle_html}
    <div class="anim anim-d2" style="display:flex;flex-wrap:wrap;align-items:center;justify-content:center;gap:16px">
      {cta_html}
      {cta2_html}
    </div>
  </div>
  {stats_html}
</section>"##,
            badge_html = badge_html,
            title_html = h1_html,
            subtitle_html = element(
                r#"<p class="anim anim-d2" style="max-width:640px;margin:0 auto 48px;font-size:18px;color:var(--cronus-text-muted);line-height:1.625">"#,
                subtitle,
                "</p>",
            ),
            cta_html = element(&cta_open, cta_primary, "</a>"),
            cta2_html = cta2_html,
            stats_html = stats_html,
        );
    }

    format!(
        r##"<section style="position:relative;overflow:hidden;padding:96px 24px 128px;background:var(--cronus-bg)">
  <div class="prism-glow" style="position:absolute;inset:0;pointer-events:none"></div>
  <div style="position:relative;z-index:10;max-width:1280px;margin:0 auto;padding:0 24px">
    <div style="display:grid;grid-template-columns:1fr 1fr;gap:64px;align-items:center">
      <!-- Left: Text content -->
      <div>
        {badge_html}
        {title_html}
        {subtitle_html}
        <div class="anim anim-d2" style="display:flex;flex-wrap:wrap;align-items:center;gap:16px">
          {cta_html}
          {cta2_html}
        </div>
      </div>
      <!-- Right: Terminal window -->
      {terminal_with_chips}
    </div>
  </div>
</section>"##,
        badge_html = badge_html,
        title_html = h1_html,
        subtitle_html = element(
            r#"<p class="anim anim-d2" style="max-width:512px;font-size:18px;color:var(--cronus-text-muted);line-height:1.625;margin-bottom:40px">"#,
            subtitle,
            "</p>",
        ),
        cta_html = element(
            &cta_open,
            cta_primary,
            r#"<span class="material-symbols-outlined" style="font-size:14px">arrow_forward</span></a>"#,
        ),
        cta2_html = cta2_html,
        terminal_with_chips = terminal_with_chips,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn hero(title: Option<&str>, subtitle: Option<&str>, config: &[(&str, &str)]) -> SectionNode {
        SectionNode {
            section_type: "hero".into(),
            title: title.map(str::to_string),
            subtitle: subtitle.map(str::to_string),
            config: config
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<HashMap<_, _>>(),
            items: vec![],
            plans: vec![],
            binding: None,
            actions: vec![],
            visibility: None,
            template: None,
            style_block: None,
            doc: None,
        }
    }

    /// (theme, config) pairs that reach each of the hero layouts:
    /// dark two-column, dark centered (terminal), light centered, light terminal.
    fn variants() -> Vec<(&'static str, Vec<(&'static str, &'static str)>, bool)> {
        vec![
            ("dark", vec![], false),
            ("dark", vec![], true),
            ("light", vec![("style", "light")], false),
            ("light", vec![("style", "light")], true),
        ]
    }

    fn render(section: &mut SectionNode, theme: &str, terminal: bool) -> String {
        if terminal {
            let mut line = HashMap::new();
            line.insert("_type".to_string(), "line".to_string());
            line.insert("title".to_string(), "cronus run".to_string());
            section.items = vec![line];
        }
        render_hero(section, "blue", theme)
    }

    #[test]
    fn hero_without_subtitle_or_title_renders_no_invented_copy() {
        for (theme, config, terminal) in variants() {
            let mut s = hero(Some("Ship it"), None, &config);
            let html = render(&mut s, theme, terminal);
            assert!(
                !html.contains("next generation platform"),
                "{theme}: {html}"
            );
            assert!(!html.contains("<p class=\"[animation") && !html.contains("{subtitle}"));
            assert!(!html.contains("Get Started"), "no invented CTA: {html}");
            assert!(!html.contains("/signup"), "no invented CTA link: {html}");

            let mut untitled = hero(None, None, &config);
            let html = render(&mut untitled, theme, terminal);
            assert!(!html.contains("Build Something Amazing"), "{theme}: {html}");
            assert!(!html.contains("<h1"), "no empty heading: {html}");
        }
    }

    #[test]
    fn hero_renders_declared_subtitle_and_cta() {
        for (theme, mut config, terminal) in variants() {
            config.push(("cta_text", "Start now"));
            config.push(("cta_link", "/start"));
            let mut s = hero(Some("Ship it"), Some("Declared subtitle"), &config);
            let html = render(&mut s, theme, terminal);
            assert!(html.contains(">Declared subtitle</p>"), "{theme}: {html}");
            assert!(html.contains("Start now") && html.contains("href=\"/start\""));
            assert!(html.contains("<h1"), "{theme}: {html}");
        }
    }

    #[test]
    fn hardcode_lint_is_clean_for_hero_without_subtitle() {
        for (theme, config) in [("dark", vec![]), ("light", vec![("style", "light")])] {
            let page = crate::parser::PageNode {
                route: "/".into(),
                page_type: "custom".into(),
                entity: None,
                title: None,
                sections: vec![hero(Some("Ship. Fast."), None, &config)],
                config: HashMap::new(),
                components: vec![],
                requires: None,
                doc: None,
                span: Default::default(),
            };
            let findings = crate::hardcode_lint::lint_all_pages(
                std::slice::from_ref(&page),
                &[],
                Some(&crate::parser::StyleNode {
                    theme: Some(theme.into()),
                    accent: None,
                    radius: None,
                    font: None,
                    config: HashMap::new(),
                    span: Default::default(),
                }),
            );
            let texts: Vec<_> = findings.iter().map(|f| f.text.as_str()).collect();
            assert!(texts.is_empty(), "{theme}: {texts:?}");
        }
    }
}
