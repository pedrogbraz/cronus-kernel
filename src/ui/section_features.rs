//! Features section renderers (bento grids, feature splits)
use crate::parser::SectionNode;

pub(super) fn render_features(section: &SectionNode, _accent: &str, theme: &str) -> String {
    let style_hint = section
        .config
        .get("style")
        .map(|s| s.as_str())
        .unwrap_or("");
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

    // Detect if this is a product grid (all cards are span:4 with badge/action_text)
    let is_product_grid = cards.iter().all(|g| {
        let span: u32 = g.card.get("span").and_then(|s| s.parse().ok()).unwrap_or(4);
        span <= 4 && (g.card.get("badge").is_some() || g.card.get("action_text").is_some())
    }) && !cards.is_empty();

    // Helper: render a single card's HTML (without grid wrapper)
    let render_card = |group: &CardGroup, card_index: usize| -> String {
        let card = &group.card;
        let name = card
            .get("title")
            .or_else(|| card.get("name"))
            .map(|s| s.as_str())
            .unwrap_or("Feature");
        let desc = card
            .get("description")
            .or_else(|| card.get("desc"))
            .map(|s| s.as_str())
            .unwrap_or("");
        let icon_name = card.get("icon").map(|s| s.as_str()).unwrap_or("star");
        let span: u32 = card.get("span").and_then(|s| s.parse().ok()).unwrap_or(4);
        let anim_delay = format!("{}s", 0.5 + (card_index as f32) * 0.1);

        let has_image = group
            .children
            .iter()
            .any(|c| c.get("_type").map(|s| s.as_str()) == Some("image"));
        let has_chips = group
            .children
            .iter()
            .any(|c| c.get("_type").map(|s| s.as_str()) == Some("chip"));
        let has_badge = card.get("badge").is_some();
        let has_action = card.get("action_text").is_some();

        let img_src = group
            .children
            .iter()
            .filter(|c| c.get("_type").map(|s| s.as_str()) == Some("image"))
            .filter_map(|c| c.get("src").or(c.get("url")))
            .next()
            .map(|s| s.trim_matches('"').to_string())
            .unwrap_or_default();

        let img_alt = group
            .children
            .iter()
            .filter(|c| c.get("_type").map(|s| s.as_str()) == Some("image"))
            .filter_map(|c| c.get("alt"))
            .next()
            .map(|s| s.trim_matches('"').to_string())
            .unwrap_or_default();

        // Chip pills — span:12 cards with chip children
        if has_chips && span >= 12 {
            let chips: Vec<String> = group.children.iter()
                .filter(|c| c.get("_type").map(|s| s.as_str()) == Some("chip"))
                .map(|c| {
                    let t = c.get("title").map(|s| s.as_str()).unwrap_or("");
                    let icon = c.get("icon").map(|s| s.as_str()).unwrap_or("");
                    let icon_html = if !icon.is_empty() {
                        format!(r#"<span class="material-symbols-outlined group-hover:text-red-500 text-neutral-400 transition-colors" style="font-size:16px">{}</span>"#, icon)
                    } else { String::new() };
                    format!(
                        r#"<button class="bg-white/[0.02] border border-white/5 flex gap-3 group hover:bg-white/[0.05] items-center px-6 py-3 rounded-full transition-colors">{}<span class="text-neutral-300 text-sm">{}</span></button>"#,
                        icon_html, t
                    )
                }).collect();
            return format!(
                r#"<div class="[animation:fadeInUp_0.8s_ease-out_{delay}_both] lg:col-span-12 mb-24"><div class="flex flex-wrap gap-4 justify-center">{chips}</div></div>"#,
                delay = anim_delay,
                chips = chips.join("")
            );
        }

        // Product card — has badge and/or action_text
        if has_badge || has_action {
            let badge_html = card.get("badge").map(|b| format!(
                r#"<div class="absolute left-4 top-4"><span class="font-bold text-[10px] text-red-500 tracking-widest uppercase">{}</span></div>"#,
                b
            )).unwrap_or_default();

            let image_html = if !img_src.is_empty() {
                format!(
                    r#"<div class="flex items-center justify-center mb-8 mt-4 relative w-full z-10"><img alt="{alt}" src="{src}" class="drop-shadow-2xl h-full object-contain"></div>"#,
                    alt = name,
                    src = img_src
                )
            } else {
                String::new()
            };

            let price_html = if !desc.is_empty() {
                format!(
                    r#"<p class="font-mono mb-6 text-neutral-500 text-sm">{}</p>"#,
                    desc
                )
            } else {
                String::new()
            };

            let action_html = card.get("action_text").map(|a| format!(
                r#"<button class="border border-white/10 font-semibold hover:bg-white hover:text-black py-2.5 rounded-full text-white text-xs transition-colors w-full">{}</button>"#,
                a
            )).unwrap_or_default();

            return format!(
                r##"<div class="[animation:fadeInUp_0.8s_ease-out_{delay}_both] border border-white/5 group hover:bg-white/[0.03] overflow-hidden p-8 relative rounded-2xl transition-colors">
  {badge}
  {image}
  <h3 class="font-medium mb-2 text-lg text-white">{name}</h3>
  {price}
  {action}
</div>"##,
                delay = anim_delay,
                badge = badge_html,
                image = image_html,
                name = name,
                price = price_html,
                action = action_html,
            );
        }

        // Large card (span:8) — full image cover with gradient overlay
        if span >= 8 {
            let image_html = if !img_src.is_empty() {
                format!(
                    r#"<img src="{src}" alt="{alt}" class="absolute bottom-0 duration-700 group-hover:scale-105 h-full left-0 object-cover opacity-90 right-0 top-0 transition-transform w-full"><div class="absolute bg-gradient-to-t bottom-0 from-black left-0 right-0 to-transparent top-0 via-black/20"></div>"#,
                    src = img_src,
                    alt = img_alt
                )
            } else {
                String::new()
            };

            return format!(
                r##"<div class="[animation:fadeInUp_0.8s_ease-out_{delay}_both] bg-[#080808] border border-white/5 group h-[400px] hover:border-white/10 md:col-span-8 md:h-[500px] overflow-hidden relative rounded-3xl transition-all">
  {image}
  <div class="absolute bottom-0 left-0 p-8 w-full">
    <div class="flex gap-2 items-center mb-2 text-red-500">
      <span class="material-symbols-outlined" style="font-size:16px">{icon}</span>
      <span class="font-semibold text-xs tracking-widest uppercase">Craftsmanship</span>
    </div>
    <h3 class="font-normal mb-2 text-3xl text-white tracking-tight">{name}</h3>
    <p class="max-w-md text-neutral-400 text-sm">{desc}</p>
  </div>
</div>"##,
                delay = anim_delay,
                image = image_html,
                icon = icon_name,
                name = name,
                desc = desc,
            );
        }

        // Small card (span:4) — icon box + title + divider + desc
        let image_html = if !img_src.is_empty() {
            format!(
                r#"<img src="{src}" class="absolute bottom-0 duration-700 group-hover:opacity-80 group-hover:scale-105 h-full left-0 object-cover opacity-40 right-0 top-0 transition-all w-full z-10">"#,
                src = img_src
            )
        } else {
            String::new()
        };

        format!(
            r##"<div class="bg-[#080808] border border-white/5 flex-1 group hover:border-white/10 overflow-hidden p-6 relative rounded-3xl transition-all">
  {image}
  <div class="flex flex-col h-full justify-end relative z-10">
    <div class="backdrop-blur-md bg-white/5 border border-white/10 mb-auto p-2 rounded-lg w-fit">
      <span class="material-symbols-outlined text-white" style="font-size:20px">{icon}</span>
    </div>
    <h3 class="font-normal mt-4 text-white text-xl">{name}</h3>
    <div class="bg-white/10 h-px my-3 w-full"></div>
    <div class="flex items-center justify-between">
      <span class="text-neutral-500 text-xs">{desc}</span>
    </div>
  </div>
</div>"##,
            image = image_html,
            icon = icon_name,
            name = name,
            desc = desc,
        )
    };

    // Build the grid items, grouping consecutive span:4 cards after a span:8 into flex-col wrappers
    let mut grid_items: Vec<String> = Vec::new();
    let mut i = 0;
    let mut card_index: usize = 0;

    while i < cards.len() {
        let span: u32 = cards[i]
            .card
            .get("span")
            .and_then(|s| s.parse().ok())
            .unwrap_or(4);

        if is_product_grid {
            // Product grid: render each card individually, no col-span wrapping
            grid_items.push(render_card(&cards[i], card_index));
            card_index += 1;
            i += 1;
            continue;
        }

        // Chip row (span:12)
        if span >= 12 {
            grid_items.push(render_card(&cards[i], card_index));
            card_index += 1;
            i += 1;
            continue;
        }

        // Large card (span:8) — check if followed by span:4 pair
        if span >= 8 {
            grid_items.push(render_card(&cards[i], card_index));
            card_index += 1;
            i += 1;

            // Collect consecutive span:4 cards into a flex-col wrapper
            let mut small_cards: Vec<String> = Vec::new();
            while i < cards.len() {
                let next_span: u32 = cards[i]
                    .card
                    .get("span")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(4);
                if next_span >= 8 || next_span >= 12 {
                    break;
                }
                small_cards.push(render_card(&cards[i], card_index));
                card_index += 1;
                i += 1;
                if small_cards.len() == 2 {
                    break;
                }
            }

            if !small_cards.is_empty() {
                let wrapper_delay =
                    format!("{}s", 0.5 + ((card_index - small_cards.len()) as f32) * 0.1);
                grid_items.push(format!(
                    r#"<div class="[animation:fadeInUp_0.8s_ease-out_{delay}_both] flex flex-col gap-6 md:col-span-4 z-10">
  {cards}
</div>"#,
                    delay = wrapper_delay,
                    cards = small_cards.join("\n  ")
                ));
            }
            continue;
        }

        // Standalone span:4 card (not after a span:8)
        let anim_delay = format!("{}s", 0.5 + (card_index as f32) * 0.1);
        let card_html = render_card(&cards[i], card_index);
        grid_items.push(format!(
            r#"<div class="[animation:fadeInUp_0.8s_ease-out_{delay}_both] md:col-span-4">{card_html}</div>"#,
            delay = anim_delay,
            card_html = card_html
        ));
        card_index += 1;
        i += 1;
    }

    // Section header
    let section_header = if let Some(ref title) = section.title {
        let subtitle = section.subtitle.as_deref().unwrap_or("");
        let subtitle_html = if !subtitle.is_empty() {
            format!(
                r#"<p class="[animation:fadeInUp_0.8s_ease-out_0.3s_both] font-light mt-4 text-neutral-400">{}</p>"#,
                subtitle
            )
        } else {
            String::new()
        };
        format!(
            r#"<div class="mb-20 text-center">
      <h2 class="[animation:fadeInUp_0.8s_ease-out_0.2s_both] font-medium md:text-5xl text-3xl text-white tracking-tight">{}</h2>
      {}
    </div>"#,
            title, subtitle_html
        )
    } else {
        String::new()
    };

    // Grid class: product grid uses 3-col, bento uses 12-col
    let grid_class = if is_product_grid {
        "gap-6 grid grid-cols-1 md:grid-cols-3"
    } else {
        "gap-6 grid grid-cols-1 lg:grid-cols-12"
    };

    format!(
        r#"<section class="bg-[#030303] border-t border-white/5 overflow-hidden py-32 relative">
  <div class="max-w-7xl mx-auto px-6 relative z-10">
    {}
    <div class="{}">
      {}
    </div>
  </div>
</section>"#,
        section_header,
        grid_class,
        grid_items.join("\n      "),
    )
}

/// Material symbol icon helper
pub(super) fn get_material_icon(name: &str, color: &str, size: u32) -> String {
    match name {
        "commit" => format!(
            r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M16.89 12.5a5.001 5.001 0 00-9.78 0H2v-1h5.11a5.001 5.001 0 019.78 0H22v1h-5.11zM12 15a3 3 0 110-6 3 3 0 010 6z"/></svg>"#,
            s = size,
            c = color
        ),
        "extension" => format!(
            r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M20.5 11H19V7a2 2 0 00-2-2h-4V3.5a2.5 2.5 0 00-5 0V5H4a2 2 0 00-2 2v3.8h1.5a2.7 2.7 0 010 5.4H2V20a2 2 0 002 2h3.8v-1.5a2.7 2.7 0 015.4 0V22H17a2 2 0 002-2v-4h1.5a2.5 2.5 0 000-5z"/></svg>"#,
            s = size,
            c = color
        ),
        "public" | "globe" => format!(
            r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/></svg>"#,
            s = size,
            c = color
        ),
        "terminal" => format!(
            r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M20 4H4a2 2 0 00-2 2v12a2 2 0 002 2h16a2 2 0 002-2V6a2 2 0 00-2-2zm0 14H4V8h16v10zm-2-1h-6v-2h6v2zM7.5 17l-1.41-1.41L8.67 13l-2.59-2.59L7.5 9l4 4-4 4z"/></svg>"#,
            s = size,
            c = color
        ),
        "code" => format!(
            r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M9.4 16.6L4.8 12l4.6-4.6L8 6l-6 6 6 6 1.4-1.4zm5.2 0l4.6-4.6-4.6-4.6L16 6l6 6-6 6-1.4-1.4z"/></svg>"#,
            s = size,
            c = color
        ),
        "upload" | "cloud_upload" => format!(
            r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M19.35 10.04A7.49 7.49 0 0012 4C9.11 4 6.6 5.64 5.35 8.04A5.994 5.994 0 000 14c0 3.31 2.69 6 6 6h13c2.76 0 5-2.24 5-5 0-2.64-2.05-4.78-4.65-4.96zM14 13v4h-4v-4H7l5-5 5 5h-3z"/></svg>"#,
            s = size,
            c = color
        ),
        "rocket_launch" | "rocket" => format!(
            r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M12 2.5s4.5 2 4.5 9.5c0 2.08-.52 3.88-1.26 5.35l-1.62-1.62a2 2 0 00-3.24 0l-1.62 1.62A14.83 14.83 0 017.5 12c0-7.5 4.5-9.5 4.5-9.5zM5 18l1.38-1.37c.49-.49 1.11-.83 1.78-.97l1.85 1.85c-.06.85-.27 1.7-.64 2.49H5zm14 0h-4.37c-.37-.79-.58-1.64-.64-2.49l1.85-1.85c.67.14 1.29.48 1.78.97L19 18zM12 12.5a1.5 1.5 0 100-3 1.5 1.5 0 000 3z"/></svg>"#,
            s = size,
            c = color
        ),
        _ => format!(
            r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="{c}"><path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/></svg>"#,
            s = size,
            c = color
        ),
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
    let eyebrow = section
        .config
        .get("eyebrow")
        .or(section.config.get("badge"))
        .map(|s| s.as_str())
        .unwrap_or("");

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
            image_src = item
                .get("src")
                .or(item.get("url"))
                .map(|s| s.trim_matches('"').to_string())
                .unwrap_or_default();
            continue;
        }
        if item.get("image").is_some() || (item.get("src").is_some() && item.get("title").is_none())
        {
            image_src = item
                .get("src")
                .or(item.get("image"))
                .map(|s| s.trim_matches('"').to_string())
                .unwrap_or_default();
            continue;
        }

        let name = item
            .get("title")
            .or_else(|| item.get("name"))
            .map(|s| s.as_str())
            .unwrap_or("Feature");
        let desc = item
            .get("description")
            .or_else(|| item.get("desc"))
            .map(|s| s.as_str())
            .unwrap_or("");
        let icon_name = item.get("icon").map(|s| s.as_str()).unwrap_or("star");

        // Fix #2: Detect metric-like titles (digits, ms, $, B+, %) -> stat cards in 2-col grid
        let is_metric = name.chars().any(|c| c.is_ascii_digit())
            || name.contains("ms")
            || name.contains('$')
            || name.contains("B+")
            || name.contains('%');

        if is_metric {
            let meta = item.get("meta").map(|s| s.as_str()).unwrap_or("");
            let unit_html = if !meta.is_empty() {
                format!(
                    r#"<span style="font-size:1rem;color:rgba(255,255,255,0.4);margin-left:4px">{}</span>"#,
                    meta
                )
            } else {
                String::new()
            };
            stat_cards.push(format!(
                r#"<div>
  <div style="font-size:2.5rem;font-weight:700;color:#adc6ff;letter-spacing:-0.02em">{name}{unit}</div>
  <div style="font-size:12px;text-transform:uppercase;letter-spacing:0.15em;opacity:0.4;margin-top:8px;color:#fff">{desc}</div>
</div>"#,
                name = name, unit = unit_html, desc = desc
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

    // Right column — image with optional stat overlay
    let _has_metrics = !stat_cards.is_empty();
    let right_col = if !image_src.is_empty() {
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

    // CTA buttons
    let cta_text = section
        .config
        .get("cta_text")
        .map(|s| s.as_str())
        .unwrap_or("");
    let cta_link = section
        .config
        .get("cta_link")
        .map(|s| s.as_str())
        .unwrap_or("#");
    let cta2_text = section
        .config
        .get("cta2_text")
        .map(|s| s.as_str())
        .unwrap_or("");
    let cta2_link = section
        .config
        .get("cta2_link")
        .map(|s| s.as_str())
        .unwrap_or("#");
    let cta_html = if !cta_text.is_empty() {
        let cta2_btn = if !cta2_text.is_empty() {
            format!(
                r#"<a href="{}" class="bg-white/5 border border-white/10 font-medium gap-2 hover:bg-white/10 inline-flex items-center px-6 py-2.5 rounded-full text-sm text-white transition-colors">{}</a>"#,
                cta2_link, cta2_text
            )
        } else {
            String::new()
        };
        format!(
            r#"<div style="display:flex;gap:16px;margin-top:32px">
        <a href="{}" class="bg-white font-medium gap-2 hover:scale-105 inline-flex items-center px-6 py-2.5 rounded-full text-black text-sm transition-transform">{}</a>
        {}
      </div>"#,
            cta_link, cta_text, cta2_btn
        )
    } else {
        String::new()
    };

    // Eyebrow badge with pulse dot
    let eyebrow_html = if !eyebrow.is_empty() {
        format!(
            r#"<div class="bg-white/5 border border-white/10 font-medium gap-2 inline-flex items-center mb-6 px-3 py-1 rounded-full text-[10px] text-red-200 tracking-wider uppercase"><span class="flex h-1.5 relative w-1.5"><span class="absolute animate-ping bg-red-400 h-full inline-flex opacity-75 rounded-full w-full"></span><span class="bg-red-500 h-1.5 inline-flex relative rounded-full w-1.5"></span></span>{}</div>"#,
            eyebrow
        )
    } else {
        String::new()
    };

    format!(
        r#"<section class="md:mt-28 mt-20 relative">
  <div class="max-w-7xl md:px-8 mx-auto px-6">
    <div class="gap-16 grid items-center lg:grid-cols-2">
      <div>
        {eyebrow_html}
        {title}
        {subtitle}
        {features}
        {cta_html}
      </div>
      {right_col}
    </div>
  </div>
</section>"#,
        eyebrow_html = eyebrow_html,
        title = title_html,
        subtitle = subtitle_html,
        features = features_html,
        cta_html = cta_html,
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
        let name = item
            .get("title")
            .or_else(|| item.get("name"))
            .map(|s| s.as_str())
            .unwrap_or("Feature");
        let desc = item
            .get("description")
            .or_else(|| item.get("desc"))
            .map(|s| s.as_str())
            .unwrap_or("");

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
