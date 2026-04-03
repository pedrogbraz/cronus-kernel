//! Features section renderers (bento grids, feature splits)
use crate::parser::SectionNode;

pub(super) fn render_features(section: &SectionNode, _accent: &str, theme: &str) -> String {
    let style_hint = section.config.get("style").map(|s| s.as_str()).unwrap_or("");
    let is_dark = theme == "dark" || style_hint.contains("dark");

    // Split layout — two-column text+icons left, image right
    if style_hint.contains("split") {
        if is_dark {
            return render_features_split_dark(section);
        } else {
            return render_features_split(section, _accent);
        }
    }

    // Bento layout — trigger on style:bento, or auto-detect from mixed spans/children
    let has_spans = section.items.iter().any(|i| i.get("span").is_some());
    let has_typed_children = section.items.iter().any(|i| {
        let t = i.get("_type").map(|s| s.as_str()).unwrap_or("");
        matches!(t, "chip" | "label" | "code" | "image")
    });
    let is_bento = style_hint.contains("bento") || has_spans || has_typed_children;

    // Light bento → dedicated renderer
    if is_bento && !is_dark {
        return render_features_bento_light(section);
    }

    // Dark bento → 12-column bento grid
    if is_bento && is_dark {
        return render_features_bento_dark(section);
    }

    // Fallback: simple 3-column grid (no spans, no typed children)
    let icon_color = if is_dark { "#fff" } else { "#000" };

    let items: Vec<String> = section.items.iter().map(|item| {
        let name = item.get("title").or_else(|| item.get("name")).map(|s| s.as_str()).unwrap_or("Feature");
        let desc = item.get("description").or_else(|| item.get("desc")).map(|s| s.as_str()).unwrap_or("");
        let icon_name = item.get("icon").map(|s| s.as_str()).unwrap_or("star");

        let icon_svg = get_material_icon(icon_name, icon_color, 20);

        format!(
            r#"<div style="background:var(--cronus-surface);border-radius:var(--cronus-radius);border:1px solid var(--cronus-border);padding:32px;display:flex;flex-direction:column;justify-content:space-between;transition:border-color 0.3s">
  <div>
    <div style="width:40px;height:40px;border-radius:var(--cronus-radius);background:rgba(128,128,128,0.08);display:flex;align-items:center;justify-content:center;margin-bottom:24px;color:var(--cronus-text)">{icon_svg}</div>
    <h3 style="font-size:20px;font-weight:700;letter-spacing:-0.02em;color:var(--cronus-text);margin-bottom:8px">{name}</h3>
    <p style="color:var(--cronus-text-muted);font-size:14px;line-height:1.6">{desc}</p>
  </div>
</div>"#, icon_svg = icon_svg, name = name, desc = desc)
    }).collect();

    format!(
        r#"<section style="padding:64px 24px;width:100%">
  <div style="max-width:var(--cronus-max-w,1120px);margin:0 auto">
    <div style="display:grid;grid-template-columns:repeat(3,1fr);gap:24px;width:100%">
      {}
    </div>
  </div>
</section>"#,
        items.join("\n    "),
    )
}

/// Dark bento-style feature grid — 12-column layout with varying card sizes.
/// Groups flat items into cards + children (image, chip, etc.).
pub(super) fn render_features_bento_dark(section: &SectionNode) -> String {
    struct CardGroup {
        card: std::collections::HashMap<String, String>,
        children: Vec<std::collections::HashMap<String, String>>,
    }

    let mut cards: Vec<CardGroup> = Vec::new();

    for item in &section.items {
        let item_type = item.get("_type").map(|s| s.as_str()).unwrap_or("");
        if item_type.is_empty() {
            cards.push(CardGroup {
                card: item.clone(),
                children: Vec::new(),
            });
        } else if let Some(last) = cards.last_mut() {
            last.children.push(item.clone());
        }
    }

    // Surface colors cycle: card0=#1f1f1f, card1=#2a2a2a, card2=#0e0e0e, card3=#2a2a2a
    let surface_colors = ["#1f1f1f", "#2a2a2a", "#0e0e0e", "#2a2a2a"];

    let card_htmls: Vec<String> = cards.iter().enumerate().map(|(i, group)| {
        let card = &group.card;
        let name = card.get("title").or_else(|| card.get("name")).map(|s| s.as_str()).unwrap_or("Feature");
        let desc = card.get("description").or_else(|| card.get("desc")).map(|s| s.as_str()).unwrap_or("");
        let icon_name = card.get("icon").map(|s| s.as_str()).unwrap_or("star");
        let span: u32 = card.get("span").and_then(|s| s.parse().ok()).unwrap_or(4);
        let bg = surface_colors.get(i).unwrap_or(&"#1f1f1f");

        let has_image = group.children.iter().any(|c| c.get("_type").map(|s| s.as_str()) == Some("image"));
        let has_chips = group.children.iter().any(|c| c.get("_type").map(|s| s.as_str()) == Some("chip"));

        // Uniform subtle border on all cards
        let border_css = "border:0.5px solid rgba(76,69,70,0.15);";

        // Image HTML — absolute positioned, grayscale, reveals color on hover
        let image_html = if has_image {
            let img_src = group.children.iter()
                .filter(|c| c.get("_type").map(|s| s.as_str()) == Some("image"))
                .filter_map(|c| c.get("src").or(c.get("url")))
                .next()
                .map(|s| s.trim_matches('"'))
                .unwrap_or("");
            if !img_src.is_empty() {
                format!(
                    r#"<div style="position:absolute;bottom:0;right:0;width:66%;height:50%;overflow:visible;pointer-events:none"><img src="{src}" style="width:100%;height:100%;object-fit:cover;border-top-left-radius:12px;filter:grayscale(100%);opacity:0.3;transform:translateY(25%) translateX(25%);transition:transform 0.7s ease,filter 0.5s,opacity 0.5s;pointer-events:auto" onmouseover="this.style.transform='translateY(0) translateX(25%)';this.style.filter='none';this.style.opacity='0.6'" onmouseout="this.style.transform='translateY(25%) translateX(25%)';this.style.filter='grayscale(100%)';this.style.opacity='0.3'" alt=""></div>"#,
                    src = img_src
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Progress bar at bottom for small cards (span 4) with chips
        let progress_html = if i == 1 && span <= 4 {
            // SOC2 card (second card): compliance progress bar
            let chips: Vec<String> = group.children.iter()
                .filter(|c| c.get("_type").map(|s| s.as_str()) == Some("chip"))
                .map(|c| {
                    let t = c.get("title").map(|s| s.as_str()).unwrap_or("");
                    format!(
                        r#"<span style="padding:4px 12px;background:rgba(255,255,255,0.08);border-radius:999px;font-size:10px;font-weight:700;letter-spacing:0.1em;text-transform:uppercase;color:rgba(255,255,255,0.7)">{}</span>"#,
                        t.to_uppercase()
                    )
                }).collect();
            let chips_row = if !chips.is_empty() {
                format!(r#"<div style="display:flex;flex-wrap:wrap;gap:8px;margin-bottom:16px">{}</div>"#, chips.join(""))
            } else {
                String::new()
            };
            format!(
                r#"<div style="margin-top:auto;padding-top:24px;width:100%">
    {chips_row}
    <div style="height:4px;background:rgba(255,255,255,0.05);border-radius:9999px;overflow:hidden">
      <div style="height:100%;background:white;width:100%;opacity:0.5"></div>
    </div>
    <div style="display:flex;justify-content:space-between;margin-top:8px">
      <span style="font-family:'Space Grotesk';font-size:10px;color:rgba(255,255,255,0.3);text-transform:uppercase;letter-spacing:0.15em">Compliance status</span>
      <span style="font-family:'Space Grotesk';font-size:10px;color:rgba(255,255,255,0.3);text-transform:uppercase;letter-spacing:0.15em">100%</span>
    </div>
  </div>"#,
                chips_row = chips_row
            )
        } else if has_chips && span <= 4 {
            let chips: Vec<String> = group.children.iter()
                .filter(|c| c.get("_type").map(|s| s.as_str()) == Some("chip"))
                .map(|c| {
                    let t = c.get("title").map(|s| s.as_str()).unwrap_or("");
                    format!(
                        r#"<span style="padding:4px 12px;background:rgba(255,255,255,0.08);border-radius:999px;font-size:10px;font-weight:700;letter-spacing:0.1em;text-transform:uppercase;color:rgba(255,255,255,0.7)">{}</span>"#,
                        t.to_uppercase()
                    )
                }).collect();
            let chips_row = if !chips.is_empty() {
                format!(r#"<div style="display:flex;flex-wrap:wrap;gap:8px;margin-bottom:16px">{}</div>"#, chips.join(""))
            } else {
                String::new()
            };
            format!(
                r#"<div style="margin-top:auto;padding-top:24px">
    {chips_row}
    <div style="width:100%;height:4px;background:rgba(255,255,255,0.08);border-radius:2px;overflow:hidden">
      <div style="width:68%;height:100%;background:rgba(255,255,255,0.25);border-radius:2px"></div>
    </div>
  </div>"#,
                chips_row = chips_row
            )
        } else {
            String::new()
        };

        // Badge pill for first span:8 card (status indicator)
        let badge_text = card.get("badge").map(|s| s.as_str()).unwrap_or("");
        let badge_html = if i == 0 && span >= 8 && !badge_text.is_empty() {
            format!(
                r#"<div style="margin-top:auto;padding-top:24px"><div style="padding:8px 16px;border-radius:8px;background:#0e0e0e;border:0.5px solid rgba(76,69,70,0.2);display:inline-flex;align-items:center;gap:12px"><span style="width:8px;height:8px;border-radius:50%;background:#10b981;box-shadow:0 0 10px rgba(16,185,129,0.5);display:inline-block"></span><span style="font-size:12px;font-family:monospace;opacity:0.6;color:#fff">{}</span></div></div>"#,
                badge_text
            )
        } else {
            String::new()
        };

        // Code block for wide cards with style:code
        let card_style = card.get("style").map(|s| s.as_str()).unwrap_or("");
        let is_code_style = card_style == "code";

        // Bar chart or code block for wide cards (span 8) without images, not the first card
        let bar_chart_html = if span >= 8 && !has_image && i > 0 && !is_code_style {
            r#"<div style="width:50%;display:flex;align-items:flex-end;justify-content:flex-end;margin-left:auto">
      <div style="width:100%;height:128px;display:grid;grid-template-columns:repeat(6,1fr);gap:4px;align-items:end">
        <div style="background:rgba(255,255,255,0.10);border-radius:2px;height:25%"></div>
        <div style="background:rgba(255,255,255,0.20);border-radius:2px;height:50%"></div>
        <div style="background:rgba(255,255,255,0.40);border-radius:2px;height:75%"></div>
        <div style="background:rgba(255,255,255,0.60);border-radius:2px;height:100%"></div>
        <div style="background:rgba(255,255,255,0.30);border-radius:2px;height:50%"></div>
        <div style="background:rgba(255,255,255,0.80);border-radius:2px;height:100%"></div>
      </div>
    </div>"#.to_string()
        } else if span >= 8 && !has_image && is_code_style {
            // Code block visualization
            r#"<div style="width:50%;display:flex;align-items:center;justify-content:flex-end;margin-left:auto">
      <div style="background:#0e0e0e;border-radius:8px;padding:16px;border:0.5px solid rgba(76,69,70,0.1);font-family:monospace;font-size:12px;width:100%;line-height:1.8">
        <span style="color:#adc6ff">ultima</span><span style="color:rgba(255,255,255,0.5)"> .</span><span style="color:#fff">initiate_transfer</span><span style="color:rgba(255,255,255,0.5)">({</span><br>
        <span style="color:rgba(255,255,255,0.3)">&nbsp;&nbsp;</span><span style="color:#adc6ff">amount</span><span style="color:rgba(255,255,255,0.5)">: </span><span style="color:#7dd3a0">"1.2M"</span><span style="color:rgba(255,255,255,0.5)">,</span><br>
        <span style="color:rgba(255,255,255,0.3)">&nbsp;&nbsp;</span><span style="color:#adc6ff">currency</span><span style="color:rgba(255,255,255,0.5)">: </span><span style="color:#7dd3a0">"USD"</span><span style="color:rgba(255,255,255,0.5)">,</span><br>
        <span style="color:rgba(255,255,255,0.3)">&nbsp;&nbsp;</span><span style="color:#adc6ff">vault</span><span style="color:rgba(255,255,255,0.5)">: </span><span style="color:#7dd3a0">"Alpha_Prime"</span><br>
        <span style="color:rgba(255,255,255,0.5)">});</span>
      </div>
    </div>"#.to_string()
        } else {
            String::new()
        };

        let use_flex_row = !bar_chart_html.is_empty();
        let needs_relative = has_image;
        let min_height = if span >= 8 { "min-height:400px;" } else { "min-height:240px;" };
        let position = if needs_relative { "position:relative;" } else { "" };

        if use_flex_row {
            // Horizontal layout: left text + right bar chart
            format!(
                r##"<div style="grid-column:span {span};background:{bg};{border}border-radius:16px;padding:40px;display:flex;flex-direction:row;{min_h}overflow:hidden;transition:background 0.5s" onmouseover="this.style.background='#2a2a2a'" onmouseout="this.style.background='{bg}'">
  <div style="flex:1;display:flex;flex-direction:column">
    <span class="material-symbols-outlined" style="font-size:32px;color:#fff;margin-bottom:20px;display:block;transform:scale(1.5);transform-origin:top left">{icon}</span>
    <h3 style="font-size:{title_size};font-weight:700;letter-spacing:-0.02em;color:#fff;margin-bottom:12px">{name}</h3>
    <p style="color:#c6c6c6;font-size:14px;line-height:1.7;max-width:360px">{desc}</p>
  </div>
  {bar_chart}
</div>"##,
                span = span,
                bg = bg,
                border = border_css,
                min_h = min_height,
                icon = icon_name,
                name = name,
                desc = desc,
                bar_chart = bar_chart_html,
                title_size = if span >= 8 { "30px" } else { "20px" },
            )
        } else {
            // Vertical layout (default)
            // For span:4 cards without progress bar: icon at top, title+desc at bottom (space-between)
            let is_span4_no_progress = span <= 4 && progress_html.is_empty();
            let justify = if is_span4_no_progress { "justify-content:space-between;" } else { "" };

            if is_span4_no_progress {
                format!(
                    r##"<div style="grid-column:span {span};background:{bg};{border}border-radius:16px;padding:40px;display:flex;flex-direction:column;{justify}{min_h}{pos}overflow:hidden;transition:background 0.5s" onmouseover="this.style.background='#2a2a2a'" onmouseout="this.style.background='{bg}'">
  <div>
    <span class="material-symbols-outlined" style="font-size:32px;color:#fff;display:block;transform:scale(1.5);transform-origin:top left">{icon}</span>
  </div>
  <div>
    <h3 style="font-size:{title_size};font-weight:700;letter-spacing:-0.02em;color:#fff;margin-bottom:12px">{name}</h3>
    <p style="color:#c6c6c6;font-size:14px;line-height:1.7;max-width:420px">{desc}</p>
  </div>
  {badge}
  {image}
</div>"##,
                    span = span,
                    bg = bg,
                    border = border_css,
                    justify = justify,
                    min_h = min_height,
                    pos = position,
                    icon = icon_name,
                    name = name,
                    desc = desc,
                    badge = badge_html,
                    image = image_html,
                    title_size = "20px",
                )
            } else {
                format!(
                    r##"<div style="grid-column:span {span};background:{bg};{border}border-radius:16px;padding:40px;display:flex;flex-direction:column;{min_h}{pos}overflow:hidden;transition:background 0.5s" onmouseover="this.style.background='#2a2a2a'" onmouseout="this.style.background='{bg}'">
  <div>
    <span class="material-symbols-outlined" style="font-size:32px;color:#fff;margin-bottom:20px;display:block;transform:scale(1.5);transform-origin:top left">{icon}</span>
    <h3 style="font-size:{title_size};font-weight:700;letter-spacing:-0.02em;color:#fff;margin-bottom:12px">{name}</h3>
    <p style="color:#c6c6c6;font-size:14px;line-height:1.7;max-width:420px">{desc}</p>
  </div>
  {badge}
  {progress}
  {image}
</div>"##,
                    span = span,
                    bg = bg,
                    border = border_css,
                    min_h = min_height,
                    pos = position,
                    icon = icon_name,
                    name = name,
                    desc = desc,
                    badge = badge_html,
                    progress = progress_html,
                    image = image_html,
                    title_size = if span >= 8 { "30px" } else { "20px" },
                )
            }
        }
    }).collect();

    // Section header (if title exists)
    let section_header = if let Some(ref title) = section.title {
        let eyebrow = section.config.get("subtitle_label")
            .or(section.config.get("eyebrow"))
            .map(|s| s.as_str())
            .or(section.subtitle.as_deref())
            .unwrap_or("");
        let eyebrow_html = if !eyebrow.is_empty() {
            format!(
                r#"<h2 style="font-size:14px;font-weight:700;color:#adc6ff;letter-spacing:0.3em;text-transform:uppercase;margin-bottom:16px">{}</h2>"#,
                eyebrow
            )
        } else {
            String::new()
        };
        format!(
            r#"<div style="margin-bottom:96px;text-align:left">
      {}
      <h3 style="font-size:clamp(32px,4vw,48px);font-weight:700;letter-spacing:-0.02em;max-width:672px;color:white;margin-bottom:0">{}</h3>
    </div>"#,
            eyebrow_html, title
        )
    } else {
        String::new()
    };

    format!(
        r#"<section style="padding:128px 24px;width:100%;background:#0e0e0e">
  <div style="max-width:80rem;margin:0 auto">
    {}
    <div style="display:grid;grid-template-columns:repeat(12,1fr);gap:24px;width:100%">
      {}
    </div>
  </div>
</section>"#,
        section_header,
        card_htmls.join("\n      "),
    )
}

/// Material symbol icon helper
pub(super) fn get_material_icon(name: &str, color: &str, size: u32) -> String {
    match name {
        "commit" => format!(r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M16.89 12.5a5.001 5.001 0 00-9.78 0H2v-1h5.11a5.001 5.001 0 019.78 0H22v1h-5.11zM12 15a3 3 0 110-6 3 3 0 010 6z"/></svg>"#, s=size, c=color),
        "extension" => format!(r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M20.5 11H19V7a2 2 0 00-2-2h-4V3.5a2.5 2.5 0 00-5 0V5H4a2 2 0 00-2 2v3.8h1.5a2.7 2.7 0 010 5.4H2V20a2 2 0 002 2h3.8v-1.5a2.7 2.7 0 015.4 0V22H17a2 2 0 002-2v-4h1.5a2.5 2.5 0 000-5z"/></svg>"#, s=size, c=color),
        "public" | "globe" => format!(r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/></svg>"#, s=size, c=color),
        "terminal" => format!(r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M20 4H4a2 2 0 00-2 2v12a2 2 0 002 2h16a2 2 0 002-2V6a2 2 0 00-2-2zm0 14H4V8h16v10zm-2-1h-6v-2h6v2zM7.5 17l-1.41-1.41L8.67 13l-2.59-2.59L7.5 9l4 4-4 4z"/></svg>"#, s=size, c=color),
        "code" => format!(r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M9.4 16.6L4.8 12l4.6-4.6L8 6l-6 6 6 6 1.4-1.4zm5.2 0l4.6-4.6-4.6-4.6L16 6l6 6-6 6-1.4-1.4z"/></svg>"#, s=size, c=color),
        "upload" | "cloud_upload" => format!(r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M19.35 10.04A7.49 7.49 0 0012 4C9.11 4 6.6 5.64 5.35 8.04A5.994 5.994 0 000 14c0 3.31 2.69 6 6 6h13c2.76 0 5-2.24 5-5 0-2.64-2.05-4.78-4.65-4.96zM14 13v4h-4v-4H7l5-5 5 5h-3z"/></svg>"#, s=size, c=color),
        "rocket_launch" | "rocket" => format!(r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M12 2.5s4.5 2 4.5 9.5c0 2.08-.52 3.88-1.26 5.35l-1.62-1.62a2 2 0 00-3.24 0l-1.62 1.62A14.83 14.83 0 017.5 12c0-7.5 4.5-9.5 4.5-9.5zM5 18l1.38-1.37c.49-.49 1.11-.83 1.78-.97l1.85 1.85c-.06.85-.27 1.7-.64 2.49H5zm14 0h-4.37c-.37-.79-.58-1.64-.64-2.49l1.85-1.85c.67.14 1.29.48 1.78.97L19 18zM12 12.5a1.5 1.5 0 100-3 1.5 1.5 0 000 3z"/></svg>"#, s=size, c=color),
        _ => format!(r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/></svg>"#, s=size, c=color),
    }
}

/// Bento-style feature grid for light developer landing.
/// Reads ALL data from section.items dynamically:
/// - Items without `_type` (or empty) are feature cards
/// - Items with `_type` (label, chip, code, image) are children of the preceding card
pub(super) fn render_features_bento_light(section: &SectionNode) -> String {
    // Group flat items into cards + their children.
    // Items without _type (or empty) are feature CARDS.
    // Items with _type (label, chip, code, image) belong to the preceding card as children.
    struct CardGroup {
        card: std::collections::HashMap<String, String>,
        children: Vec<std::collections::HashMap<String, String>>,
    }

    let mut cards: Vec<CardGroup> = Vec::new();

    for item in &section.items {
        let item_type = item.get("_type").map(|s| s.as_str()).unwrap_or("");
        if item_type.is_empty() {
            cards.push(CardGroup {
                card: item.clone(),
                children: Vec::new(),
            });
        } else if let Some(last) = cards.last_mut() {
            last.children.push(item.clone());
        }
    }

    let card_htmls: Vec<String> = cards.iter().enumerate().map(|(i, group)| {
        let card = &group.card;
        let name = card.get("title").or_else(|| card.get("name")).map(|s| s.as_str()).unwrap_or("Feature");
        let desc = card.get("description").or_else(|| card.get("desc")).map(|s| s.as_str()).unwrap_or("");
        let icon_name = card.get("icon").map(|s| s.as_str()).unwrap_or("star");
        let span = card.get("span").and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);
        let card_style = card.get("style").map(|s| s.as_str()).unwrap_or("");
        let is_dark = card_style.contains("dark");
        let delay_class = format!("reveal card-hover anim anim-d{}", (i % 4) + 1);

        let col_span_css = if span > 1 {
            format!("grid-column:span {};", span)
        } else {
            String::new()
        };

        // Classify children
        let has_image = group.children.iter().any(|c| c.get("_type").map(|s| s.as_str()) == Some("image"));
        let has_label = group.children.iter().any(|c| c.get("_type").map(|s| s.as_str()) == Some("label"));
        let has_chips = group.children.iter().any(|c| c.get("_type").map(|s| s.as_str()) == Some("chip"));
        let has_code = group.children.iter().any(|c| c.get("_type").map(|s| s.as_str()) == Some("code"));
        let use_horizontal = span > 1 && has_image;

        if is_dark {
            // ── DARK CARD ──
            let chips_html = if has_chips {
                let chips: Vec<String> = group.children.iter()
                    .filter(|c| c.get("_type").map(|s| s.as_str()) == Some("chip"))
                    .map(|c| {
                        let t = c.get("title").map(|s| s.as_str()).unwrap_or("");
                        format!(
                            r#"<span style="padding:4px 12px;background:rgba(255,255,255,0.1);border-radius:999px;font-size:10px;font-weight:700;letter-spacing:0.1em;text-transform:uppercase;color:#fff">{}</span>"#,
                            t.to_uppercase()
                        )
                    }).collect();
                format!(
                    r#"<div style="display:flex;flex-wrap:wrap;gap:8px;margin-top:32px">{}</div>"#,
                    chips.join("")
                )
            } else {
                String::new()
            };

            format!(
                r##"<div class="{delay_class}" style="{col_span}background:#000;border-radius:12px;padding:32px;display:flex;flex-direction:column;justify-content:space-between">
  <div>
    <span class="material-symbols-outlined" style="font-size:32px;color:#fff;margin-bottom:16px;display:block">{icon}</span>
    <h3 style="font-size:24px;font-weight:700;letter-spacing:-0.02em;color:#fff;margin-bottom:12px">{name}</h3>
    <p style="color:#9ca3af;font-size:14px;line-height:1.6">{desc}</p>
  </div>
  {chips}
</div>"##,
                delay_class = delay_class,
                col_span = col_span_css,
                icon = icon_name,
                name = name,
                desc = desc,
                chips = chips_html,
            )
        } else if use_horizontal {
            // ── LIGHT CARD, SPAN 2, WITH IMAGE → HORIZONTAL LAYOUT ──
            let non_image_children = render_bento_children_light(&group.children, false, true);
            let img_src = group.children.iter()
                .filter(|c| c.get("_type").map(|s| s.as_str()) == Some("image"))
                .filter_map(|c| c.get("src").or(c.get("url")))
                .next()
                .map(|s| s.trim_matches('"'))
                .unwrap_or("");
            let img_alt = group.children.iter()
                .filter(|c| c.get("_type").map(|s| s.as_str()) == Some("image"))
                .filter_map(|c| c.get("title"))
                .next()
                .map(|s| s.as_str())
                .unwrap_or("");

            format!(
                r##"<div class="{delay_class}" style="{col_span}background:#fff;border:1px solid rgba(198,198,198,0.1);border-radius:12px;padding:32px;overflow:hidden">
  <div style="display:flex;gap:32px;align-items:center">
    <div style="flex:1">
      <span class="material-symbols-outlined" style="font-size:32px;color:#000;margin-bottom:16px;display:block">{icon}</span>
      <h3 style="font-size:24px;font-weight:700;letter-spacing:-0.02em;color:#1a1c1c;margin-bottom:12px">{name}</h3>
      <p style="color:#474747;font-size:14px;line-height:1.6">{desc}</p>
      {text_children}
    </div>
    <div style="flex:1">
      <img src="{img_src}" style="border-radius:8px;width:100%;box-shadow:0 20px 40px rgba(0,0,0,0.15);transition:transform 0.5s" onmouseover="this.style.transform='rotate(2deg)'" onmouseout="this.style.transform='none'" alt="{img_alt}">
    </div>
  </div>
</div>"##,
                delay_class = delay_class,
                col_span = col_span_css,
                icon = icon_name,
                name = name,
                desc = desc,
                text_children = non_image_children,
                img_src = img_src,
                img_alt = img_alt,
            )
        } else if has_label {
            // ── LIGHT CARD WITH LABEL → flow icons + label at bottom (Git Push pattern) ──
            let label_text = group.children.iter()
                .filter(|c| c.get("_type").map(|s| s.as_str()) == Some("label"))
                .filter_map(|c| c.get("title"))
                .next()
                .map(|s| s.as_str())
                .unwrap_or("");

            let flow_html = format!(
                r##"<div style="display:flex;align-items:center;gap:16px;margin-top:48px">
    <div style="display:flex">
      <div style="width:40px;height:40px;border-radius:50%;background:#f3f3f3;border:2px solid #fff;display:flex;align-items:center;justify-content:center"><span class="material-symbols-outlined" style="font-size:16px">code</span></div>
      <div style="width:40px;height:40px;border-radius:50%;background:#000;border:2px solid #fff;display:flex;align-items:center;justify-content:center;margin-left:-8px"><span class="material-symbols-outlined" style="font-size:16px;color:#fff">upload</span></div>
      <div style="width:40px;height:40px;border-radius:50%;background:#006ff0;border:2px solid #fff;display:flex;align-items:center;justify-content:center;margin-left:-8px"><span class="material-symbols-outlined" style="font-size:16px;color:#fff">rocket_launch</span></div>
    </div>
    <div style="flex:1;height:1px;background:rgba(198,198,198,0.3)"></div>
    <span style="font-size:11px;font-family:'JetBrains Mono',monospace;color:#9ca3af;letter-spacing:0.08em">{label}</span>
  </div>"##,
                label = label_text
            );

            format!(
                r##"<div class="{delay_class}" style="{col_span}background:#fff;border:1px solid rgba(198,198,198,0.1);border-radius:12px;padding:32px;display:flex;flex-direction:column;justify-content:space-between">
  <div>
    <span class="material-symbols-outlined" style="font-size:32px;color:#000;margin-bottom:16px;display:block">{icon}</span>
    <h3 style="font-size:24px;font-weight:700;letter-spacing:-0.02em;color:#1a1c1c;margin-bottom:12px">{name}</h3>
    <p style="color:#474747;font-size:14px;line-height:1.6;max-width:480px">{desc}</p>
  </div>
  {flow}
</div>"##,
                delay_class = delay_class,
                col_span = col_span_css,
                icon = icon_name,
                name = name,
                desc = desc,
                flow = flow_html,
            )
        } else if has_image && span <= 1 {
            // ── LIGHT CARD, SPAN 1, WITH IMAGE → vertical layout, image below ──
            let img_src = group.children.iter()
                .filter(|c| c.get("_type").map(|s| s.as_str()) == Some("image"))
                .filter_map(|c| c.get("src").or(c.get("url")))
                .next()
                .map(|s| s.trim_matches('"'))
                .unwrap_or("");
            let img_alt = group.children.iter()
                .filter(|c| c.get("_type").map(|s| s.as_str()) == Some("image"))
                .filter_map(|c| c.get("title"))
                .next()
                .map(|s| s.as_str())
                .unwrap_or("");

            let img_html = if !img_src.is_empty() {
                format!(
                    r#"<div style="margin-top:24px;aspect-ratio:1;border-radius:8px;overflow:hidden;background:#e8e8e8"><img src="{}" style="width:100%;height:100%;object-fit:cover;filter:grayscale(100%);transition:filter 0.5s" onmouseover="this.style.filter='none'" onmouseout="this.style.filter='grayscale(100%)'" alt="{}"></div>"#,
                    img_src, img_alt
                )
            } else {
                r#"<div style="margin-top:24px;aspect-ratio:1;background:#e8e8e8;border-radius:8px;overflow:hidden"></div>"#.to_string()
            };

            format!(
                r##"<div class="{delay_class}" style="{col_span}background:#fff;border:1px solid rgba(198,198,198,0.1);border-radius:12px;padding:32px;display:flex;flex-direction:column">
  <span class="material-symbols-outlined" style="font-size:32px;color:#000;margin-bottom:16px;display:block">{icon}</span>
  <h3 style="font-size:20px;font-weight:700;letter-spacing:-0.02em;color:#1a1c1c;margin-bottom:8px">{name}</h3>
  <p style="font-size:14px;color:#474747;flex:1">{desc}</p>
  {img}
</div>"##,
                delay_class = delay_class,
                col_span = col_span_css,
                icon = icon_name,
                name = name,
                desc = desc,
                img = img_html,
            )
        } else {
            // ── LIGHT CARD, generic fallback ──
            let children_html = render_bento_children_light(&group.children, false, false);
            format!(
                r##"<div class="{delay_class}" style="{col_span}background:#fff;border:1px solid rgba(198,198,198,0.1);border-radius:12px;padding:32px;display:flex;flex-direction:column;justify-content:space-between">
  <div>
    <span class="material-symbols-outlined" style="font-size:32px;color:#000;margin-bottom:16px;display:block">{icon}</span>
    <h3 style="font-size:20px;font-weight:700;letter-spacing:-0.02em;color:#1a1c1c;margin-bottom:8px">{name}</h3>
    <p style="color:#474747;font-size:14px;line-height:1.6;max-width:480px">{desc}</p>
  </div>
  {children}
</div>"##,
                delay_class = delay_class,
                col_span = col_span_css,
                icon = icon_name,
                name = name,
                desc = desc,
                children = children_html,
            )
        }
    }).collect();

    format!(
        r##"<section style="padding:96px 24px;background:rgba(243,243,243,0.5)">
  <div style="max-width:1280px;margin:0 auto">
    <div class="stagger" style="display:grid;grid-template-columns:repeat(3,1fr);gap:24px">
      {items}
    </div>
  </div>
</section>"##,
        items = card_htmls.join("\n      "),
    )
}

/// Render non-image child items (code, chip) for light bento cards.
/// `skip_images` filters out image children (used in horizontal layout where images are handled separately).
pub(super) fn render_bento_children_light(
    children: &[std::collections::HashMap<String, String>],
    _is_dark: bool,
    skip_images: bool,
) -> String {
    let mut code_pills: Vec<String> = Vec::new();
    let mut chip_pills: Vec<String> = Vec::new();

    for child in children {
        let child_type = child.get("_type").map(|s| s.as_str()).unwrap_or("");
        let title = child.get("title").map(|s| s.as_str()).unwrap_or("");

        match child_type {
            "code" => {
                code_pills.push(format!(
                    r#"<code style="padding:8px 16px;background:#eeeeee;border-radius:8px;font-family:'JetBrains Mono',monospace;font-size:13px;border:1px solid rgba(198,198,198,0.2)">{}</code>"#,
                    title
                ));
            }
            "chip" => {
                chip_pills.push(format!(
                    r#"<span style="padding:4px 12px;background:#e8e8e8;border-radius:999px;font-size:10px;font-weight:700;letter-spacing:0.1em;text-transform:uppercase;color:#1a1c1c">{}</span>"#,
                    title.to_uppercase()
                ));
            }
            "image" if skip_images => { /* handled externally */ }
            "label" => { /* handled by parent card logic */ }
            _ => {}
        }
    }

    let mut parts: Vec<String> = Vec::new();

    if !code_pills.is_empty() {
        parts.push(format!(
            r#"<div style="display:flex;flex-wrap:wrap;gap:12px;margin-top:24px">{}</div>"#,
            code_pills.join("")
        ));
    }
    if !chip_pills.is_empty() {
        parts.push(format!(
            r#"<div style="display:flex;flex-wrap:wrap;gap:8px;margin-top:32px">{}</div>"#,
            chip_pills.join("")
        ));
    }

    parts.join("\n")
}


pub(super) fn render_features_split_dark(section: &SectionNode) -> String {
    let eyebrow = section.config.get("eyebrow").map(|s| s.as_str()).unwrap_or("Security & Sovereignty");

    // Fix #1: No text-transform:uppercase — sentence case as written in .cronus
    let title_html = section.title.as_deref().map(|t| format!(
        r#"<h2 style="font-size:clamp(2.25rem,5vw,3.75rem);font-weight:900;letter-spacing:-0.04em;line-height:0.95;color:#fff;margin:0">{}</h2>"#, t
    )).unwrap_or_default();

    let subtitle_html = section.subtitle.as_deref().map(|s| format!(
        r#"<p style="color:rgba(255,255,255,0.5);font-size:15px;line-height:1.7;margin-top:20px;max-width:480px">{}</p>"#, s
    )).unwrap_or_default();

    // Collect feature items (non-image) and image
    let mut stat_cards: Vec<String> = Vec::new();
    let mut regular_items: Vec<String> = Vec::new();
    let mut image_src = String::new();

    for item in &section.items {
        let item_type = item.get("_type").map(|s| s.as_str()).unwrap_or("");
        if item_type == "image" {
            image_src = item.get("src").or(item.get("url")).map(|s| s.trim_matches('"').to_string()).unwrap_or_default();
            continue;
        }
        if item.get("image").is_some() || (item.get("src").is_some() && item.get("title").is_none()) {
            image_src = item.get("src").or(item.get("image")).map(|s| s.trim_matches('"').to_string()).unwrap_or_default();
            continue;
        }

        let name = item.get("title").or_else(|| item.get("name")).map(|s| s.as_str()).unwrap_or("Feature");
        let desc = item.get("description").or_else(|| item.get("desc")).map(|s| s.as_str()).unwrap_or("");
        let icon_name = item.get("icon").map(|s| s.as_str()).unwrap_or("star");

        // Fix #2: Detect metric-like titles (digits, ms, $, B+, %) -> stat cards in 2-col grid
        let is_metric = name.chars().any(|c| c.is_ascii_digit())
            || name.contains("ms") || name.contains('$') || name.contains("B+") || name.contains('%');

        if is_metric {
            stat_cards.push(format!(
                r#"<div>
  <div style="font-size:2.5rem;font-weight:700;color:#adc6ff;letter-spacing:-0.02em">{name}</div>
  <div style="font-size:12px;text-transform:uppercase;letter-spacing:0.15em;opacity:0.4;margin-top:8px;color:#fff">{desc}</div>
</div>"#,
                name = name, desc = desc
            ));
        } else {
            regular_items.push(format!(
                r#"<div style="display:flex;gap:16px;align-items:flex-start">
  <span class="material-symbols-outlined" style="font-size:28px;color:#fff;flex-shrink:0;margin-top:2px">{icon}</span>
  <div>
    <h4 style="font-size:16px;font-weight:700;color:#fff;margin:0 0 4px 0;letter-spacing:-0.01em">{name}</h4>
    <p style="font-size:14px;color:rgba(255,255,255,0.45);line-height:1.6;margin:0">{desc}</p>
  </div>
</div>"#,
                icon = icon_name, name = name, desc = desc
            ));
        }
    }

    // Build features HTML: stat cards in 2-col grid, regular items in vertical list
    let mut features_html = String::new();
    if !stat_cards.is_empty() {
        features_html.push_str(&format!(
            r#"<div style="display:grid;grid-template-columns:1fr 1fr;gap:32px;padding-top:32px">
          {}
        </div>"#,
            stat_cards.join("\n          ")
        ));
    }
    if !regular_items.is_empty() {
        features_html.push_str(&format!(
            r#"<div style="display:flex;flex-direction:column;gap:24px;margin-top:40px">
          {}
        </div>"#,
            regular_items.join("\n          ")
        ));
    }

    // Fix #3: Right column — full dashboard visualization when metrics exist, image fallback otherwise
    let has_metrics = !stat_cards.is_empty();
    let right_col = if has_metrics {
        let image_html = if !image_src.is_empty() {
            format!(r#"<img src="{src}" alt="" style="width:100%;border-radius:12px;display:block;margin-bottom:24px">"#, src = image_src)
        } else {
            String::new()
        };
        let dashboard_chart = r##"<div style="display:flex;justify-content:space-between;align-items:center">
        <div>
          <div style="font-size:12px;opacity:0.5;text-transform:uppercase;letter-spacing:-0.02em;font-family:monospace;color:#fff">Total Assets Under Management</div>
          <div style="font-size:2rem;font-weight:700;margin-top:8px;color:#fff">$1,204,550.00 <span style="font-size:14px;color:#10b981;font-weight:500">+12.4%</span></div>
        </div>
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="rgba(255,255,255,0.2)" stroke-width="1.5"><path d="M3 3h7v7H3zM14 3h7v7h-7zM3 14h7v7H3zM14 14h7v7h-7z"/></svg>
      </div>
      <div style="position:relative;height:256px;width:100%;margin-top:32px">
        <svg style="width:100%;height:100%" preserveAspectRatio="none" viewBox="0 0 800 256">
          <defs>
            <linearGradient id="cronusChartGrad" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#adc6ff" stop-opacity="0.3"/>
              <stop offset="100%" stop-color="#adc6ff" stop-opacity="0"/>
            </linearGradient>
          </defs>
          <path d="M0 200 Q 100 150, 200 180 T 400 100 T 600 50 T 800 80" fill="none" stroke="#adc6ff" stroke-width="3" stroke-linecap="round"/>
          <path d="M0 200 Q 100 150, 200 180 T 400 100 T 600 50 T 800 80 V 256 H 0 Z" fill="url(#cronusChartGrad)"/>
        </svg>
        <div style="position:absolute;top:40px;left:50%;transform:translateX(-50%);backdrop-filter:blur(32px);-webkit-backdrop-filter:blur(32px);background:rgba(31,31,31,0.5);border:0.5px solid rgba(173,198,255,0.2);padding:8px 16px;border-radius:8px;font-size:12px;font-family:monospace;color:#fff;box-shadow:0 25px 50px rgba(0,0,0,0.25);white-space:nowrap">Vol: $44.2k &middot; 14:02:11</div>
      </div>
      <div style="display:grid;grid-template-columns:repeat(4,1fr);gap:16px;margin-top:32px">
        <div style="height:4px;background:#adc6ff;border-radius:9999px"></div>
        <div style="height:4px;background:#2a2a2a;border-radius:9999px"></div>
        <div style="height:4px;background:#2a2a2a;border-radius:9999px"></div>
        <div style="height:4px;background:#2a2a2a;border-radius:9999px"></div>
      </div>"##;
        format!(
            r#"<div style="flex:1;min-width:0;position:relative;display:flex;align-items:center;justify-content:center">
  <div style="background:#0e0e0e;border-radius:24px;border:0.5px solid rgba(76,69,70,0.15);padding:4px;box-shadow:0 25px 50px rgba(0,0,0,0.25);overflow:hidden;width:100%">
    <div style="background:rgba(31,31,31,0.3);padding:32px;border-radius:20px">
      {image_html}
      {dashboard_chart}
    </div>
  </div>
</div>"#,
            image_html = image_html,
            dashboard_chart = dashboard_chart,
        )
    } else if !image_src.is_empty() {
        format!(
            r#"<div style="flex:1;min-width:0;position:relative;display:flex;align-items:center;justify-content:center">
  <div style="background:#0e0e0e;border-radius:24px;border:0.5px solid rgba(76,69,70,0.15);padding:4px;box-shadow:0 25px 50px rgba(0,0,0,0.25);overflow:hidden">
    <img src="{src}" alt="" style="width:100%;max-width:560px;border-radius:20px;display:block">
  </div>
</div>"#,
            src = image_src
        )
    } else {
        String::new()
    };

    format!(
        r#"<section style="background:#0e0e0e;padding:128px 24px;width:100%">
  <div style="max-width:80rem;margin:0 auto">
    <div style="display:flex;flex-direction:column;gap:80px;align-items:center" data-cronus-split>
      <style>@media(min-width:768px){{[data-cronus-split]{{flex-direction:row!important;gap:80px}}}}</style>
      <div style="flex:1;min-width:0">
        <span style="font-family:'Space Grotesk',sans-serif;font-size:10px;text-transform:uppercase;letter-spacing:0.3em;color:rgba(255,255,255,0.4);margin-bottom:16px;display:block">{eyebrow}</span>
        {title}
        {subtitle}
        {features}
      </div>
      {right_col}
    </div>
  </div>
</section>"#,
        eyebrow = eyebrow,
        title = title_html,
        subtitle = subtitle_html,
        features = features_html,
        right_col = right_col,
    )
}

pub(super) fn render_features_split(section: &SectionNode, accent: &str) -> String {
    let title_html = section.title.as_deref().map(|t| format!(
        r#"<h2 style="font-size:24px;font-weight:700;letter-spacing:-0.02em;color:#1a1c1c;margin-bottom:32px">{}</h2>"#, t
    )).unwrap_or_default();

    let mut feature_items: Vec<String> = Vec::new();
    let mut code_block = String::new();

    for item in &section.items {
        let name = item.get("title").or_else(|| item.get("name")).map(|s| s.as_str()).unwrap_or("Feature");
        let desc = item.get("description").or_else(|| item.get("desc")).map(|s| s.as_str()).unwrap_or("");

        if item.get("code").is_some() || desc.starts_with('{') {
            let code_content = item.get("code").map(|s| s.as_str()).unwrap_or(desc);
            code_block = format!(
                r#"<div style="background:#1a1a1a;border-radius:12px;padding:24px;overflow:hidden">
  <pre style="margin:0;color:white;font-size:13px;font-family:'SF Mono',SFMono-Regular,Consolas,monospace;line-height:1.6;white-space:pre-wrap;overflow-x:auto">{}</pre>
</div>"#,
                code_content
            );
        } else {
            let icon_letter = name.chars().next().unwrap_or('F');
            feature_items.push(format!(
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
            ));
        }
    }

    let right_col = if code_block.is_empty() {
        r#"<div style="background:#f5f5f5;border-radius:12px;padding:24px;min-height:200px;display:flex;align-items:center;justify-content:center">
  <span style="font-size:14px;color:#5e5e5e">Preview</span>
</div>"#.to_string()
    } else {
        code_block
    };

    format!(
        r#"<section style="max-width:1280px;margin:0 auto;padding:48px 24px">
  {title_html}
  <div style="display:flex;gap:48px;align-items:flex-start">
    <div style="flex:0 0 60%;display:flex;flex-direction:column;gap:24px">
      {features}
    </div>
    <div style="flex:0 0 40%">
      {right_col}
    </div>
  </div>
</section>"#,
        title_html = title_html,
        features = feature_items.join("\n      "),
        right_col = right_col,
    )
}

