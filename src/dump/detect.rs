#![allow(dead_code, unused_imports, unused_variables)]
//! Section detection from DOM nodes.
//!
//! Orchestrates pattern matching on parsed DOM and produces
//! `SectionBlueprint` instances ready for `.cronus` emission.

use std::collections::HashMap;
use super::dom::{self, DomNode};
use super::patterns;

// ---------------------------------------------------------------------------
// Data models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SectionBlueprint {
    pub section_type: String,
    pub confidence: f32,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub config: HashMap<String, String>,
    pub items: Vec<ItemBlueprint>,
}

#[derive(Debug, Clone)]
pub struct ItemBlueprint {
    pub item_type: String,
    pub title: String,
    pub description: Option<String>,
    pub config: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Main detection entry point
// ---------------------------------------------------------------------------

/// Walk every top-level DOM block, classify it, and extract content.
pub fn detect_sections(nodes: &[DomNode]) -> Vec<SectionBlueprint> {
    let mut sections: Vec<SectionBlueprint> = Vec::new();

    for node in nodes {
        let (section_type, confidence) = patterns::classify_node(node);

        if confidence <= 0.3 {
            // Below threshold -- try splitting large blocks into sub-blocks
            if should_split_block(node) {
                let sub_sections = split_and_detect(node);
                if !sub_sections.is_empty() {
                    sections.extend(sub_sections);
                    continue;
                }
            }
            // Still emit a generic section so no text is lost
            let generic = extract_generic(node);
            if generic.title.is_some() || !generic.items.is_empty() {
                sections.push(generic);
            }
            continue;
        }

        let blueprint = match section_type {
            "topbar" => extract_topbar(node),
            "hero" => extract_hero(node),
            "features" => extract_features(node),
            "stats" => extract_stats(node),
            "cta" => extract_cta(node),
            "footer" => extract_footer(node),
            "terminal" => extract_terminal(node),
            "sidebar" => extract_sidebar(node),
            "page-header" => extract_page_header(node),
            "stat-cards" => extract_stat_cards(node),
            "product-grid" => extract_product_grid(node),
            "team-list" => extract_team_list(node),
            "card" => extract_content_card(node),
            "info-panel" => extract_info_panel(node),
            _ => extract_generic(node),
        };

        // Override confidence from the pattern engine
        let mut blueprint = blueprint;
        blueprint.confidence = confidence;
        blueprint.section_type = section_type.to_string();

        sections.push(blueprint);
    }

    // Sort sections for dashboard pages: sidebar/topbar first, footer last,
    // page-header before content sections.
    sort_dashboard_sections(&mut sections);

    sections
}

/// Check if a block is large enough to warrant splitting into sub-blocks.
/// A block with many children and deep nesting is a candidate.
fn should_split_block(node: &DomNode) -> bool {
    // At least 3 direct children (so there's something to split)
    if node.children.len() < 3 {
        return false;
    }
    // Count total descendants to gauge block size
    let desc_count = count_descendants(node);
    desc_count > 15
}

/// Count all descendants of a node.
fn count_descendants(node: &DomNode) -> usize {
    let mut count = node.children.len();
    for child in &node.children {
        count += count_descendants(child);
    }
    count
}

/// Split a large generic block into sub-blocks and detect each one.
fn split_and_detect(node: &DomNode) -> Vec<SectionBlueprint> {
    let mut sections = Vec::new();

    for child in &node.children {
        // Skip empty/trivial children
        if child.full_text.trim().is_empty() && child.children.is_empty() {
            continue;
        }

        let (child_type, child_confidence) = patterns::classify_node(child);

        if child_confidence > 0.3 {
            // This child classifies as something specific
            let blueprint = match child_type {
                "topbar" => extract_topbar(child),
                "hero" => extract_hero(child),
                "features" => extract_features(child),
                "stats" => extract_stats(child),
                "cta" => extract_cta(child),
                "footer" => extract_footer(child),
                "terminal" => extract_terminal(child),
                "sidebar" => extract_sidebar(child),
                "page-header" => extract_page_header(child),
                "stat-cards" => extract_stat_cards(child),
                "product-grid" => extract_product_grid(child),
                "team-list" => extract_team_list(child),
                "card" => extract_content_card(child),
                "info-panel" => extract_info_panel(child),
                _ => extract_generic(child),
            };
            let mut blueprint = blueprint;
            blueprint.confidence = child_confidence;
            blueprint.section_type = child_type.to_string();
            sections.push(blueprint);
        } else {
            // Generic child — still extract it if it has content
            let generic = extract_generic(child);
            if generic.title.is_some() || !generic.items.is_empty() {
                sections.push(generic);
            }
        }
    }

    sections
}

/// Sort sections for dashboard layout ordering:
/// sidebar → topbar → page-header → content sections → footer.
fn sort_dashboard_sections(sections: &mut Vec<SectionBlueprint>) {
    fn section_order(s: &SectionBlueprint) -> u32 {
        match s.section_type.as_str() {
            "sidebar" => 0,
            "topbar" => 1,
            "page-header" => 2,
            "footer" => 100,
            _ => 50,
        }
    }
    sections.sort_by_key(|s| section_order(s));
}

// ---------------------------------------------------------------------------
// Extraction: topbar
// ---------------------------------------------------------------------------

fn extract_topbar(node: &DomNode) -> SectionBlueprint {
    let mut config: HashMap<String, String> = HashMap::new();

    // Brand: first bold/large text child, or first text in the nav
    let brand = find_brand_text(node).unwrap_or_default();
    if !brand.is_empty() {
        config.insert("brand".into(), brand);
    }

    // Nav links
    let links = dom::extract_links(node);
    let nav_texts: Vec<String> = links.iter().map(|(text, _)| text.clone()).collect();
    if !nav_texts.is_empty() {
        config.insert("nav".into(), nav_texts.join(", "));
    }

    // CTA button (last button found is usually the primary CTA)
    let buttons = extract_clean_buttons(node);
    if let Some(cta) = buttons.last() {
        config.insert("cta_text".into(), cta.clone());
    }

    SectionBlueprint {
        section_type: "topbar".into(),
        confidence: 0.0,
        title: None,
        subtitle: None,
        config,
        items: Vec::new(),
    }
}

/// Find the brand/logo text in a nav-like node.
/// Heuristic: first child with large font class, or first bold text,
/// or simply the first non-link text.
fn find_brand_text(node: &DomNode) -> Option<String> {
    // Try finding a node with large text classes (brand logos)
    let large_classes = ["text-xl", "text-2xl", "text-3xl", "font-bold", "font-semibold", "logo"];
    for cls in &large_classes {
        let matches = dom::find_by_class(node, cls);
        for m in matches {
            // Skip material-symbols icon spans
            if is_material_icon_span(m) {
                continue;
            }
            let txt = dom::clean_node_text(m);
            if !txt.is_empty() && txt.len() < 50 {
                return Some(txt);
            }
        }
    }

    // Fallback: first direct text that isn't empty
    if !node.text.is_empty() && node.text.len() < 50 {
        return Some(node.text.trim().to_string());
    }
    for child in &node.children {
        if child.tag != "a" && child.tag != "button" && !is_material_icon_span(child) {
            let txt = dom::clean_node_text(child);
            if !txt.is_empty() && txt.len() < 50 {
                return Some(txt);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Extraction: hero
// ---------------------------------------------------------------------------

fn extract_hero(node: &DomNode) -> SectionBlueprint {
    let mut config: HashMap<String, String> = HashMap::new();
    let mut items: Vec<ItemBlueprint> = Vec::new();

    // h1 -> title
    let title = find_heading_by_tag(node, "h1")
        .or_else(|| dom::find_heading(node));

    // First <p> after heading -> subtitle
    let subtitle = dom::find_paragraph(node);

    // ---------------------------------------------------------------
    // A) Badge: look for rounded-full elements with small text that
    //    also have animate-pulse, badge, or inline-flex indicators.
    // ---------------------------------------------------------------
    let badge_text = find_hero_badge(node);
    if let Some(ref b) = badge_text {
        config.insert("badge".into(), b.clone());
    }

    // CTA buttons (cleaned of material icon text)
    let buttons = extract_clean_buttons(node);
    if let Some(first) = buttons.first() {
        config.insert("cta_text".into(), first.clone());
    }
    // Links associated with buttons for hrefs
    let links = dom::extract_links(node);
    if let Some(first_link) = links.first() {
        config.insert("cta_link".into(), first_link.1.clone());
    }
    if buttons.len() > 1 {
        config.insert("cta2_text".into(), buttons[1].clone());
    }
    if links.len() > 1 {
        config.insert("cta2_link".into(), links[1].1.clone());
    }

    // ---------------------------------------------------------------
    // B) Terminal: find div-based terminal (dark bg + rounded) inside
    //    the hero, extract title, lines, and floating chips.
    //    Items go AFTER the CTAs (already in config above).
    // ---------------------------------------------------------------

    // Strategy 1: <pre>/<code> blocks (classic terminals)
    let pre_nodes = dom::find_by_tag(node, "pre");
    let code_nodes = dom::find_by_tag(node, "code");
    let classic_terminal: Vec<&DomNode> = pre_nodes.into_iter().chain(code_nodes.into_iter()).collect();
    for tn in &classic_terminal {
        let text = tn.full_text.trim().to_string();
        if text.is_empty() {
            continue;
        }
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let (item_type, line_config) = classify_terminal_line(trimmed, tn);
            items.push(ItemBlueprint {
                item_type,
                title: trimmed.to_string(),
                description: None,
                config: line_config,
            });
        }
    }

    // Strategy 2: div-based terminal (bg-black / bg-[# dark + rounded)
    if items.is_empty() {
        if let Some((terminal_container, terminal_parent)) = find_hero_terminal(node) {
            // Terminal header item: find font-mono text in the header area
            let terminal_title = find_terminal_title(terminal_container);
            if let Some(ref t) = terminal_title {
                let mut term_config = HashMap::new();
                term_config.insert("style".into(), "terminal".into());
                items.push(ItemBlueprint {
                    item_type: "item".into(),
                    title: t.clone(),
                    description: terminal_title.clone(),
                    config: term_config,
                });
            }

            // Find the terminal body: the div with command lines, not the header bar.
            let body = find_terminal_body(terminal_container);
            if let Some(body_node) = body {
                extract_hero_terminal_lines(body_node, &mut items);
            } else {
                // Fallback: extract lines from the whole container
                extract_hero_terminal_lines(terminal_container, &mut items);
            }

            // Floating chips: siblings after the terminal container
            if let Some(parent) = terminal_parent {
                extract_hero_chips(parent, &mut items);
            }
            extract_hero_chips(terminal_container, &mut items);
        }
    }

    // ---------------------------------------------------------------
    // C) Embedded stats: some heroes have inline metrics (e.g.
    //    grid of cards with large numbers + small labels).
    //    Detect and extract them as stat items inside the hero.
    // ---------------------------------------------------------------
    extract_hero_embedded_stats(node, &mut items);

    SectionBlueprint {
        section_type: "hero".into(),
        confidence: 0.0,
        title,
        subtitle,
        config,
        items,
    }
}

/// Extract stat-like metric cards embedded within a hero section.
/// Looks for a grid container (with grid-cols-) whose children each have
/// a large number (text-3xl+) and a small uppercase label.
fn extract_hero_embedded_stats(node: &DomNode, items: &mut Vec<ItemBlueprint>) {
    // Find actual CSS grid containers (grid-cols-*), not grid-pattern etc.
    let grid_nodes = dom::find_by_class(node, "grid-cols-");
    for grid in &grid_nodes {
        // Skip the grid if it IS the hero root itself (avoid picking up the
        // overall hero wrapper)
        if std::ptr::eq(*grid, node) {
            continue;
        }

        let mut stat_candidates: Vec<(String, String)> = Vec::new();

        for child in &grid.children {
            let large = find_large_text(child);
            let label = find_hero_stat_label(child);
            if let (Some(val), Some(lbl)) = (large, label) {
                stat_candidates.push((val, lbl));
            }
        }

        // Need at least 2 stat cards to qualify
        if stat_candidates.len() >= 2 {
            for (value, label) in &stat_candidates {
                items.push(ItemBlueprint {
                    item_type: "item".into(),
                    title: label.clone(),
                    description: Some(value.clone()),
                    config: {
                        let mut c = HashMap::new();
                        c.insert("role".into(), "stat".into());
                        c
                    },
                });
            }
            return;
        }
    }
}

/// Find a stat label inside a hero-embedded stat card.
/// Looks for small uppercase text, filtering out material icon leaks.
fn find_hero_stat_label(node: &DomNode) -> Option<String> {
    let candidates = dom::find_by_class(node, "uppercase");
    for m in &candidates {
        let is_small = dom::has_class(m, "text-xs") || dom::has_class(m, "text-sm");
        if !is_small {
            continue;
        }
        if is_material_icon_span(m) {
            continue;
        }
        let txt = m.full_text.trim().to_string();
        if txt.is_empty() || txt.len() > 100 {
            continue;
        }
        // Skip lines that start with material icon names
        let icon_names = ["trending_down", "trending_up", "bolt", "check_circle",
                          "arrow_forward", "arrow_back", "public", "speed"];
        if icon_names.iter().any(|i| txt.starts_with(i)) {
            continue;
        }
        return Some(txt);
    }
    None
}

/// Find the hero badge: an element with rounded-full AND (animate-pulse OR
/// badge OR inline-flex) that contains small text (text-xs/text-sm).
/// Returns the badge text excluding dot-span content.
fn find_hero_badge(node: &DomNode) -> Option<String> {
    // First try the classic "badge"/"pill" class
    let badge_nodes = dom::find_by_class(node, "badge");
    let pill_nodes = dom::find_by_class(node, "pill");
    for n in badge_nodes.iter().chain(pill_nodes.iter()) {
        let txt = n.full_text.trim().to_string();
        if !txt.is_empty() {
            return Some(txt);
        }
    }

    // Modern pattern: rounded-full container with animate-pulse or inline-flex
    let rounded_nodes = dom::find_by_class(node, "rounded-full");
    for n in &rounded_nodes {
        let has_indicator = dom::has_class(n, "animate-pulse")
            || dom::has_class(n, "inline-flex")
            || n.children.iter().any(|c| dom::has_class(c, "animate-pulse"));

        if !has_indicator {
            continue;
        }

        // Must have small text (text-xs or text-sm) somewhere
        let has_small_text = dom::has_class(n, "text-xs")
            || dom::has_class(n, "text-sm")
            || n.children.iter().any(|c| {
                dom::has_class(c, "text-xs") || dom::has_class(c, "text-sm")
            });

        if !has_small_text {
            continue;
        }

        // Extract text, excluding dot spans (w-2 h-2 rounded-full with no real text)
        let text = extract_badge_text(n);
        if !text.is_empty() {
            return Some(text);
        }
    }

    None
}

/// Extract badge text from a badge container, skipping dot/pulse indicator spans.
fn extract_badge_text(node: &DomNode) -> String {
    let mut parts: Vec<String> = Vec::new();

    // If this node is a tiny dot indicator, skip it entirely
    if is_badge_dot(node) {
        return String::new();
    }

    let direct = node.text.trim().to_string();
    if !direct.is_empty() {
        parts.push(direct);
    }

    for child in &node.children {
        if is_badge_dot(child) {
            continue;
        }
        let child_text = extract_badge_text(child);
        if !child_text.is_empty() {
            parts.push(child_text);
        }
    }

    parts.join(" ").trim().to_string()
}

/// Check if a node is a tiny dot indicator (w-2 h-2 rounded-full with animate-pulse).
fn is_badge_dot(node: &DomNode) -> bool {
    dom::has_class(node, "rounded-full")
        && (dom::has_class(node, "w-2") || dom::has_class(node, "w-1"))
        && (dom::has_class(node, "animate-pulse") || dom::has_class(node, "bg-"))
        && node.full_text.trim().is_empty()
}

/// Find the terminal container inside a hero section.
/// Returns (terminal_container, optional parent for chip extraction).
fn find_hero_terminal<'a>(node: &'a DomNode) -> Option<(&'a DomNode, Option<&'a DomNode>)> {
    let dark_classes = ["bg-black", "bg-[#"];
    for cls in &dark_classes {
        let dark_nodes = dom::find_by_class(node, cls);
        for dark in &dark_nodes {
            if dom::has_class(dark, "rounded") {
                // Find the parent wrapper (for chip extraction)
                let parent = find_parent_of(node, dark);
                return Some((dark, parent));
            }
        }
    }
    None
}

/// Walk the tree to find the parent of a target node (by pointer equality).
fn find_parent_of<'a>(root: &'a DomNode, target: &DomNode) -> Option<&'a DomNode> {
    for child in &root.children {
        if std::ptr::eq(child, target) {
            return Some(root);
        }
        if let Some(p) = find_parent_of(child, target) {
            return Some(p);
        }
    }
    None
}

/// Find the font-mono title text in the terminal header bar.
fn find_terminal_title(terminal: &DomNode) -> Option<String> {
    let mono_nodes = dom::find_by_class(terminal, "font-mono");
    for m in &mono_nodes {
        let txt = m.full_text.trim().to_string();
        // Terminal title is typically short (e.g. "zsh -- vercel")
        if !txt.is_empty() && txt.len() < 60 {
            return Some(txt);
        }
    }
    None
}

/// Find the terminal body section (the div with command lines, not the header).
fn find_terminal_body<'a>(terminal: &'a DomNode) -> Option<&'a DomNode> {
    for child in &terminal.children {
        // Skip the header bar (terminal-header class or contains traffic light dots)
        let is_header = dom::has_class(child, "terminal-header")
            || child.children.iter().any(|c| is_traffic_light_dot(c)
                || c.children.iter().any(|gc| is_traffic_light_dot(gc)));

        if is_header {
            continue;
        }

        // The body should have font-mono or padding and contain lines
        if dom::has_class(child, "font-mono") || dom::has_class(child, "p-") {
            return Some(child);
        }
    }
    None
}

/// Extract terminal lines from a div-based terminal body in the hero section.
/// Walks the DOM tree to find individual "leaf" divs (each representing one
/// terminal line) and reads their child spans to classify the line properly.
fn extract_hero_terminal_lines(body: &DomNode, items: &mut Vec<ItemBlueprint>) {
    let leaf_divs = collect_terminal_leaf_divs(body);
    for leaf in leaf_divs {
        let bp = classify_terminal_leaf(leaf);
        if let Some(item) = bp {
            items.push(item);
        }
    }
}

/// Collect all "leaf" divs in the terminal body. A leaf div is one whose
/// children are spans/text (not more nested divs containing spans).
/// Wrapper divs like `<div class="mt-4 space-y-1">` are recursed into.
fn collect_terminal_leaf_divs<'a>(node: &'a DomNode) -> Vec<&'a DomNode> {
    let mut result = Vec::new();
    for child in &node.children {
        if child.tag == "div" || child.tag == "" {
            // Check if this div has child divs that themselves contain spans
            let has_nested_divs_with_content = child.children.iter().any(|gc| {
                gc.tag == "div" && !gc.full_text.trim().is_empty()
            });
            if has_nested_divs_with_content {
                // This is a wrapper div — recurse
                result.extend(collect_terminal_leaf_divs(child));
            } else {
                // This is a leaf div — it directly contains spans
                result.push(child);
            }
        }
    }
    result
}

/// Given a leaf div representing one terminal line, read its child spans
/// and classify the line type.
fn classify_terminal_leaf(node: &DomNode) -> Option<ItemBlueprint> {
    // Skip cursor blink
    if is_cursor_blink(node) {
        return None;
    }
    // Skip traffic-light dots
    if is_traffic_light_dot(node) {
        return None;
    }

    // Collect span texts and their classes.
    // Use full_text so nested elements (e.g. <a> inside <span>) are included.
    // Only fall back to direct text for very short tokens (symbols like "$", "✓")
    // where full_text would be identical anyway.
    let spans: Vec<(&str, &[String])> = node.children.iter()
        .filter(|c| c.tag == "span" || c.tag == "a" || c.tag == "")
        .map(|c| {
            let ft = c.full_text.trim();
            let txt = if !ft.is_empty() {
                ft
            } else {
                c.text.trim()
            };
            (txt, c.classes.as_slice())
        })
        .filter(|(txt, _)| !txt.is_empty())
        .collect();

    if spans.is_empty() {
        // No spans — check if there's direct text
        let direct = node.full_text.trim();
        if direct.is_empty() || direct == "_" {
            return None;
        }
        let (item_type, config) = classify_hero_terminal_line(direct, node);
        return Some(ItemBlueprint {
            item_type,
            title: direct.to_string(),
            description: None,
            config,
        });
    }

    let first_text = spans[0].0;
    let mut config = HashMap::new();

    // Classify based on first span
    if first_text == "$" {
        // Command line: "$ command"
        let cmd = spans.iter().skip(1).map(|(t, _)| *t).collect::<Vec<_>>().join(" ");
        let title = format!("$ {}", cmd);
        return Some(ItemBlueprint {
            item_type: "line".into(),
            title,
            description: None,
            config,
        });
    }

    if first_text == "?" {
        // Prompt line
        let question = if spans.len() > 1 { spans[1].0.to_string() } else { String::new() };
        // Answer is typically the third span with tertiary/blue color
        if spans.len() > 2 {
            let answer_text = spans[2].0.to_string();
            if !answer_text.is_empty() {
                config.insert("answer".into(), answer_text);
            }
        }
        return Some(ItemBlueprint {
            item_type: "prompt".into(),
            title: question,
            description: None,
            config,
        });
    }

    if first_text == "✓" || first_text == "\u{2713}" {
        // Success line
        let rest = spans.iter().skip(1).map(|(t, _)| *t).collect::<Vec<_>>().join(" ");
        // Detect color from spans
        for (_, classes) in &spans {
            if classes.iter().any(|c| c.contains("green")) {
                config.insert("color".into(), "green".into());
                break;
            }
        }
        return Some(ItemBlueprint {
            item_type: "success".into(),
            title: rest,
            description: None,
            config,
        });
    }

    if first_text == "_" && spans.len() == 1 {
        // Cursor blink
        return None;
    }

    // Default: output line — join all span texts
    let full = spans.iter().map(|(t, _)| *t).collect::<Vec<_>>().join(" ");

    // Detect color from span classes
    for (_, classes) in &spans {
        if classes.iter().any(|c| c.contains("tertiary-container") || c.contains("text-blue")) {
            config.insert("color".into(), "blue".into());
            break;
        }
        if classes.iter().any(|c| c.contains("green")) {
            config.insert("color".into(), "green".into());
            break;
        }
    }

    Some(ItemBlueprint {
        item_type: "output".into(),
        title: full,
        description: None,
        config,
    })
}

/// Check if a node is just a blinking cursor (animate-pulse with "_" text).
fn is_cursor_blink(node: &DomNode) -> bool {
    dom::has_class(node, "animate-pulse")
        && node.full_text.trim().len() <= 1
}

/// Classify a terminal line in the hero context with richer type detection.
fn classify_hero_terminal_line(text: &str, node: &DomNode) -> (String, HashMap<String, String>) {
    let mut config = HashMap::new();

    // Command line: starts with $
    if text.starts_with('$') || text.starts_with("$ ") {
        return ("line".into(), config);
    }

    // Success line: contains checkmark or has green text
    if text.contains('\u{2713}') || text.contains("\u{2713}")
        || dom::has_class(node, "text-green")
        || node.children.iter().any(|c| dom::has_class(c, "text-green"))
    {
        return ("success".into(), config);
    }

    // Prompt line: contains ? with a question and a colored answer
    if text.contains('?') && text.len() < 150 {
        let has_question_mark = node.children.iter().any(|c| {
            let t = c.full_text.trim().to_string();
            t == "?" || t.starts_with('?')
        });
        if has_question_mark || text.contains("[Y/n]") || text.contains("[y/N]") {
            // Try to extract the colored answer part
            let colored_children: Vec<&DomNode> = node.children.iter().filter(|c| {
                dom::has_class(c, "text-tertiary-container")
                    || dom::has_class(c, "text-blue")
                    || dom::has_class(c, "text-green")
            }).collect();
            if let Some(answer_node) = colored_children.first() {
                let answer = answer_node.full_text.trim().to_string();
                if !answer.is_empty() {
                    config.insert("answer".into(), answer);
                }
            }
            return ("prompt".into(), config);
        }
    }

    // Blue/tertiary output
    if dom::has_class(node, "text-tertiary-container")
        || dom::has_class(node, "text-blue")
        || node.children.iter().any(|c| dom::has_class(c, "text-tertiary-container"))
    {
        config.insert("color".into(), "blue".into());
        return ("output".into(), config);
    }

    ("output".into(), config)
}

/// Extract floating chips near the terminal (small white cards with shadow-lg).
fn extract_hero_chips(container: &DomNode, items: &mut Vec<ItemBlueprint>) {
    let shadow_nodes = dom::find_by_class(container, "shadow-lg");
    for card in &shadow_nodes {
        if !dom::has_class(card, "rounded") {
            continue;
        }
        let text = clean_button_text(card);
        if text.is_empty() || text.len() > 80 {
            continue;
        }

        let mut chip_config = HashMap::new();
        if let Some(icon) = extract_chip_icon(card) {
            chip_config.insert("icon".into(), icon);
        }

        items.push(ItemBlueprint {
            item_type: "chip".into(),
            title: text,
            description: None,
            config: chip_config,
        });
    }
}

/// Extract the icon/label from a chip's icon element (e.g. the "NEXT" or "SV" box).
fn extract_chip_icon(node: &DomNode) -> Option<String> {
    for child in &node.children {
        // Icon containers: small fixed-size elements with font-bold
        if (dom::has_class(child, "font-bold") || dom::has_class(child, "border"))
            && dom::has_class(child, "text-[")
        {
            let txt = child.full_text.trim().to_string();
            if !txt.is_empty() && txt.len() < 20 {
                return Some(txt);
            }
        }
        if (dom::has_class(child, "w-6") || dom::has_class(child, "w-5"))
            && dom::has_class(child, "h-")
        {
            let txt = child.full_text.trim().to_string();
            if !txt.is_empty() && txt.len() < 20 {
                return Some(txt);
            }
        }
        if let Some(icon) = extract_chip_icon(child) {
            return Some(icon);
        }
    }
    None
}


// ---------------------------------------------------------------------------
// Extraction: features
// ---------------------------------------------------------------------------

fn extract_features(node: &DomNode) -> SectionBlueprint {
    let mut config: HashMap<String, String> = HashMap::new();
    let mut items: Vec<ItemBlueprint> = Vec::new();

    // Detect grid columns from Tailwind class
    let cols = detect_grid_cols(node);
    if cols > 0 {
        config.insert("cols".into(), cols.to_string());
    }

    // Find the grid container first (div with grid-cols class)
    let grid_container = find_grid_container(node);

    // Section heading: only use h2/h3 that lives OUTSIDE the grid container.
    // If the heading is inside a grid card, it belongs to that card, not the section.
    let title = find_heading_outside_grid(node, grid_container);
    let subtitle = find_paragraph_outside_grid(node, grid_container);
    let cards = if let Some(gc) = grid_container {
        gc.children.iter().collect::<Vec<_>>()
    } else {
        find_card_children(node)
    };

    for card in &cards {
        // Use h3 specifically for card titles, fallback to any heading
        let card_title = find_heading_by_tag(card, "h3")
            .or_else(|| find_heading_by_tag(card, "h4"))
            .or_else(|| dom::find_heading(card))
            .unwrap_or_default();
        let card_desc = dom::find_paragraph(card);

        // Build item config
        let mut item_config: HashMap<String, String> = HashMap::new();

        // Detect col-span
        for cls in &card.classes {
            if cls.contains("col-span-") {
                if let Some(pos) = cls.find("col-span-") {
                    let after = &cls[pos + "col-span-".len()..];
                    let n: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
                    if !n.is_empty() {
                        item_config.insert("span".into(), n);
                    }
                }
            }
        }

        // Detect dark card style
        let is_dark = dom::has_class(card, "bg-black") || dom::has_class(card, "bg-primary")
            || dom::has_class(card, "bg-surface-container-highest");
        if is_dark {
            item_config.insert("style".into(), "dark".into());
        }

        // Extract material icon name
        if let Some(icon) = extract_material_icon(card) {
            item_config.insert("icon".into(), icon);
        }

        // Main card item first (so child items follow it)
        if !card_title.is_empty() || card_desc.is_some() {
            items.push(ItemBlueprint {
                item_type: "item".into(),
                title: card_title,
                description: card_desc,
                config: item_config.clone(),
            });
        }

        // Fix #2: Extract labels — font-mono small text NOT inside <code>/<pre>.
        // e.g. <span class="text-xs font-mono text-gray-400">AUTO-DEPLOYMENT ACTIVE</span>
        let mono_nodes = dom::find_by_class(card, "font-mono");
        for mono in &mono_nodes {
            if mono.tag == "code" || mono.tag == "pre" {
                continue;
            }
            let is_small = dom::has_class(mono, "text-xs") || dom::has_class(mono, "text-sm")
                || mono.classes.iter().any(|c| c.contains("text-["));
            if !is_small {
                continue;
            }
            let label_text = mono.full_text.trim().to_string();
            if !label_text.is_empty() && label_text.len() < 80 {
                let mut label_config: HashMap<String, String> = HashMap::new();
                label_config.insert("style".into(), "mono".into());
                label_config.insert("role".into(), "child".into());
                items.push(ItemBlueprint {
                    item_type: "label".into(),
                    title: label_text,
                    description: None,
                    config: label_config,
                });
            }
        }

        // Fix #5: Code blocks inside card are CHILDREN of the card's main item.
        let code_els = dom::find_by_tag(card, "code");
        for code_el in code_els {
            let code_text = code_el.full_text.trim().to_string();
            if !code_text.is_empty() {
                let mut code_config: HashMap<String, String> = HashMap::new();
                code_config.insert("role".into(), "child".into());
                items.push(ItemBlueprint {
                    item_type: "code".into(),
                    title: code_text,
                    description: None,
                    config: code_config,
                });
            }
        }

        // Fix #3: Chips inside card are CHILDREN of the card's main item.
        let chip_classes = ["badge", "chip", "rounded-full"];
        for chip_cls in &chip_classes {
            let chip_els = dom::find_by_class(card, chip_cls);
            for chip in &chip_els {
                if chip.tag == "button" || chip.tag == "a" || is_material_icon_span(chip) {
                    continue;
                }
                if is_traffic_light_dot(chip) {
                    continue;
                }
                let chip_text = clean_button_text(chip);
                if !chip_text.is_empty() && chip_text.len() < 60 {
                    let mut chip_config: HashMap<String, String> = HashMap::new();
                    chip_config.insert("role".into(), "child".into());
                    items.push(ItemBlueprint {
                        item_type: "chip".into(),
                        title: chip_text,
                        description: None,
                        config: chip_config,
                    });
                }
            }
        }

        // Fix #4: Images inside card are CHILDREN of the card's main item.
        let images = dom::extract_images(card);
        for (alt, src) in &images {
            let mut img_config: HashMap<String, String> = HashMap::new();
            img_config.insert("src".into(), src.clone());
            img_config.insert("role".into(), "child".into());
            items.push(ItemBlueprint {
                item_type: "image".into(),
                title: alt.clone(),
                description: None,
                config: img_config,
            });
        }
    }

    // Fix #1: If section title/subtitle duplicates the first card's content, clear it.
    let title = if let Some(ref t) = title {
        if items.first().map(|i| i.title.as_str()) == Some(t.as_str()) {
            None
        } else {
            Some(t.clone())
        }
    } else {
        None
    };
    let subtitle = if let Some(ref s) = subtitle {
        if items.first().and_then(|i| i.description.as_deref()) == Some(s.as_str()) {
            None
        } else {
            Some(s.clone())
        }
    } else {
        None
    };

    SectionBlueprint {
        section_type: "features".into(),
        confidence: 0.0,
        title,
        subtitle,
        config,
        items,
    }
}

/// Find the grid container (div with grid-cols class) inside a node.
fn find_grid_container<'a>(node: &'a DomNode) -> Option<&'a DomNode> {
    // Check if node itself is a grid
    if node.classes.iter().any(|c| c.contains("grid-cols")) {
        return Some(node);
    }
    // Check direct children
    for child in &node.children {
        if child.classes.iter().any(|c| c.contains("grid-cols")) {
            return Some(child);
        }
    }
    // Check one level deeper
    for child in &node.children {
        for grandchild in &child.children {
            if grandchild.classes.iter().any(|c| c.contains("grid-cols")) {
                return Some(grandchild);
            }
        }
    }
    None
}

/// Find an h2/h3 heading that lives OUTSIDE the grid container.
/// Walks direct children of the section node and skips the grid container subtree.
fn find_heading_outside_grid<'a>(node: &'a DomNode, grid: Option<&'a DomNode>) -> Option<String> {
    // Check direct headings at section level
    let heading_tags = ["h2", "h3"];
    for tag in &heading_tags {
        if node.tag == *tag {
            let text = dom::clean_node_text(node);
            if !text.is_empty() {
                return Some(text);
            }
        }
    }
    for child in &node.children {
        // Skip the grid container itself — headings inside it belong to cards
        if let Some(gc) = grid {
            if std::ptr::eq(child, gc) {
                continue;
            }
        }
        for tag in &heading_tags {
            if child.tag == *tag {
                let text = dom::clean_node_text(child);
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
        // Also check one level deeper (wrapper divs)
        for grandchild in &child.children {
            if let Some(gc) = grid {
                if std::ptr::eq(grandchild, gc) {
                    continue;
                }
            }
            for tag in &heading_tags {
                if grandchild.tag == *tag {
                    let text = dom::clean_node_text(grandchild);
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
            }
        }
    }
    None
}

/// Find a paragraph that lives OUTSIDE the grid container.
fn find_paragraph_outside_grid<'a>(node: &'a DomNode, grid: Option<&'a DomNode>) -> Option<String> {
    for child in &node.children {
        if let Some(gc) = grid {
            if std::ptr::eq(child, gc) {
                continue;
            }
        }
        if child.tag == "p" {
            let text = dom::clean_node_text(child);
            if !text.is_empty() {
                return Some(text);
            }
        }
        // One level deeper
        for grandchild in &child.children {
            if let Some(gc) = grid {
                if std::ptr::eq(grandchild, gc) {
                    continue;
                }
            }
            if grandchild.tag == "p" {
                let text = dom::clean_node_text(grandchild);
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
    }
    None
}

/// Detect grid-cols-N from Tailwind classes (searches node and immediate children).
fn detect_grid_cols(node: &DomNode) -> u32 {
    if let Some(n) = parse_grid_cols_from_classes(&node.classes) {
        return n;
    }
    for child in &node.children {
        if let Some(n) = parse_grid_cols_from_classes(&child.classes) {
            return n;
        }
    }
    0
}

fn parse_grid_cols_from_classes(classes: &[String]) -> Option<u32> {
    for cls in classes {
        // Matches grid-cols-1, grid-cols-2, ..., md:grid-cols-3, lg:grid-cols-4
        let needle = "grid-cols-";
        if let Some(pos) = cls.find(needle) {
            let after = &cls[pos + needle.len()..];
            if let Ok(n) = after.parse::<u32>() {
                return Some(n);
            }
        }
    }
    None
}

/// Find card-like children inside a node.
/// Cards are typically direct children of a grid div, or immediate <div> children.
fn find_card_children(node: &DomNode) -> Vec<&DomNode> {
    // First, try to find a grid container
    let grid_containers = dom::find_by_class(node, "grid");
    for gc in &grid_containers {
        if !gc.children.is_empty() {
            return gc.children.iter().collect();
        }
    }

    // Fallback: find divs with rounded/card/shadow classes
    let mut cards = Vec::new();
    for child in &node.children {
        if child.tag == "div" || child.tag == "article" {
            if dom::has_class(child, "rounded")
                || dom::has_class(child, "card")
                || dom::has_class(child, "shadow")
                || dom::has_class(child, "border")
                || dom::has_class(child, "p-")
            {
                cards.push(child);
            }
        }
    }

    // Last fallback: all immediate div children if section has multiple
    if cards.is_empty() && node.children.len() >= 2 {
        for child in &node.children {
            if child.tag == "div" {
                cards.push(child);
            }
        }
    }

    cards
}

// ---------------------------------------------------------------------------
// Extraction: stats
// ---------------------------------------------------------------------------

fn extract_stats(node: &DomNode) -> SectionBlueprint {
    let mut items: Vec<ItemBlueprint> = Vec::new();

    let title = find_heading_by_tag(node, "h2")
        .or_else(|| dom::find_heading(node));

    // Stats are typically cards with a large number and a label
    let cards = find_card_children(node);
    for card in &cards {
        // Large number: look for text with large font size classes
        let large_text = find_large_text(card);
        let label = find_small_label(card);

        if large_text.is_some() || label.is_some() {
            items.push(ItemBlueprint {
                item_type: "item".into(),
                title: label.unwrap_or_default(),
                description: large_text,
                config: HashMap::new(),
            });
        }
    }

    // If no cards found, try scanning all children for number-like text
    if items.is_empty() {
        for child in &node.children {
            let text = child.full_text.trim().to_string();
            if looks_like_stat(&text) {
                items.push(ItemBlueprint {
                    item_type: "item".into(),
                    title: text,
                    description: None,
                    config: HashMap::new(),
                });
            }
        }
    }

    SectionBlueprint {
        section_type: "stats".into(),
        confidence: 0.0,
        title,
        subtitle: None,
        config: HashMap::new(),
        items,
    }
}

/// Find large text (number/stat value) in a node -- looks for large font classes.
fn find_large_text(node: &DomNode) -> Option<String> {
    let large_classes = ["text-4xl", "text-5xl", "text-6xl", "text-7xl", "text-8xl", "text-3xl"];
    for cls in &large_classes {
        let matches = dom::find_by_class(node, cls);
        for m in matches {
            let txt = m.full_text.trim().to_string();
            if !txt.is_empty() {
                return Some(txt);
            }
        }
    }
    None
}

/// Find the small label text (description beneath a stat number).
fn find_small_label(node: &DomNode) -> Option<String> {
    let small_classes = ["text-sm", "text-xs", "text-muted", "text-gray", "uppercase", "tracking-wide"];
    for cls in &small_classes {
        let matches = dom::find_by_class(node, cls);
        for m in matches {
            let txt = m.full_text.trim().to_string();
            if !txt.is_empty() && txt.len() < 80 {
                return Some(txt);
            }
        }
    }
    // Fallback: find <p> or <span>
    dom::find_paragraph(node)
}

/// Rough check if text looks like a stat (contains digits, %, +, K, M, etc).
fn looks_like_stat(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.len() > 30 {
        return false;
    }
    trimmed.chars().any(|c| c.is_ascii_digit())
}

// ---------------------------------------------------------------------------
// Extraction: CTA
// ---------------------------------------------------------------------------

fn extract_cta(node: &DomNode) -> SectionBlueprint {
    let mut config: HashMap<String, String> = HashMap::new();

    let title = find_heading_by_tag(node, "h2")
        .or_else(|| dom::find_heading(node));
    let subtitle = dom::find_paragraph(node);

    let buttons = extract_clean_buttons(node);
    if let Some(first) = buttons.first() {
        config.insert("cta_text".into(), first.clone());
    }
    if buttons.len() > 1 {
        config.insert("cta2_text".into(), buttons[1].clone());
    }

    SectionBlueprint {
        section_type: "cta".into(),
        confidence: 0.0,
        title,
        subtitle,
        config,
        items: Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Extraction: footer
// ---------------------------------------------------------------------------

fn extract_footer(node: &DomNode) -> SectionBlueprint {
    let mut config: HashMap<String, String> = HashMap::new();
    let mut items: Vec<ItemBlueprint> = Vec::new();

    // Copyright text: look for text containing (c) or year patterns
    let copyright = find_copyright_text(node);
    if let Some(ref c) = copyright {
        config.insert("copyright".into(), c.clone());
    }

    // All links become items
    let links = dom::extract_links(node);
    let nav_texts: Vec<String> = links.iter().map(|(text, _)| text.clone()).collect();
    if !nav_texts.is_empty() {
        config.insert("nav".into(), nav_texts.join(", "));
    }
    for (text, href) in &links {
        let mut item_config = HashMap::new();
        item_config.insert("href".into(), href.clone());
        items.push(ItemBlueprint {
            item_type: "item".into(),
            title: text.clone(),
            description: None,
            config: item_config,
        });
    }

    SectionBlueprint {
        section_type: "footer".into(),
        confidence: 0.0,
        title: None,
        subtitle: None,
        config,
        items,
    }
}

/// Search for copyright text within a node tree.
fn find_copyright_text(node: &DomNode) -> Option<String> {
    let full = &node.full_text;
    if full.contains('\u{00A9}') || full.contains("(c)") || full.contains("©") || full.contains("Copyright") {
        // Try to find the specific child that holds it
        for child in &node.children {
            if let Some(c) = find_copyright_text(child) {
                return Some(c);
            }
        }
        // Fallback: use the node's direct text or first p
        if let Some(p) = dom::find_paragraph(node) {
            if p.contains('©') || p.contains("(c)") || p.contains("Copyright") {
                return Some(p);
            }
        }
        return Some(full.trim().to_string());
    }
    None
}

// ---------------------------------------------------------------------------
// Extraction: terminal
// ---------------------------------------------------------------------------

fn extract_terminal(node: &DomNode) -> SectionBlueprint {
    let mut items: Vec<ItemBlueprint> = Vec::new();

    let title = dom::find_heading(node);

    // Strategy 1: Collect text from <pre> and <code> blocks
    let pre_nodes = dom::find_by_tag(node, "pre");
    let code_nodes = dom::find_by_tag(node, "code");
    let sources: Vec<&DomNode> = pre_nodes.into_iter().chain(code_nodes.into_iter()).collect();

    for src in &sources {
        extract_terminal_lines_from_text(&src.full_text, src, &mut items);
    }

    // Strategy 2: Find div-based terminals (font-mono with dark bg)
    if items.is_empty() {
        let mono_nodes = dom::find_by_class(node, "font-mono");
        for mono in &mono_nodes {
            extract_terminal_lines_from_divs(mono, &mut items);
        }
    }

    // Strategy 3: Find dark background containers that look terminal-like
    if items.is_empty() {
        let dark_classes = ["bg-black", "bg-gray-900", "bg-neutral-900", "bg-zinc-900", "bg-slate-900"];
        for cls in &dark_classes {
            let dark_nodes = dom::find_by_class(node, cls);
            for dark in &dark_nodes {
                extract_terminal_lines_from_divs(dark, &mut items);
                if !items.is_empty() {
                    break;
                }
            }
            if !items.is_empty() {
                break;
            }
        }
    }

    // Strategy 4: Fallback to full text
    if items.is_empty() {
        extract_terminal_lines_from_text(&node.full_text, node, &mut items);
    }

    SectionBlueprint {
        section_type: "terminal".into(),
        confidence: 0.0,
        title,
        subtitle: None,
        config: HashMap::new(),
        items,
    }
}

/// Extract terminal lines from text content, classifying each line.
fn extract_terminal_lines_from_text(
    text: &str,
    context_node: &DomNode,
    items: &mut Vec<ItemBlueprint>,
) {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (item_type, config) = classify_terminal_line(trimmed, context_node);
        items.push(ItemBlueprint {
            item_type,
            title: trimmed.to_string(),
            description: None,
            config,
        });
    }
}

/// Extract terminal lines from div children (structured terminal UIs).
/// Each child div is treated as a line.
fn extract_terminal_lines_from_divs(node: &DomNode, items: &mut Vec<ItemBlueprint>) {
    // If node has children that are divs, each child is a line
    if node.children.is_empty() {
        // Leaf node — use its text
        let text = node.full_text.trim().to_string();
        if !text.is_empty() {
            let (item_type, config) = classify_terminal_line(&text, node);
            items.push(ItemBlueprint {
                item_type,
                title: text,
                description: None,
                config,
            });
        }
        return;
    }

    for child in &node.children {
        let text = child.full_text.trim().to_string();
        if text.is_empty() {
            // Recurse into empty wrappers
            extract_terminal_lines_from_divs(child, items);
            continue;
        }

        // Skip traffic-light dots (terminal chrome, not content)
        if is_traffic_light_dot(child) {
            continue;
        }

        // If this child has sub-divs that look like individual lines, recurse
        let child_divs: Vec<&DomNode> = child
            .children
            .iter()
            .filter(|c| c.tag == "div" || c.tag == "span" || c.tag == "p")
            .collect();

        if child_divs.len() > 1 && child_divs.iter().all(|d| !d.full_text.trim().is_empty()) {
            for sub in &child_divs {
                let sub_text = sub.full_text.trim().to_string();
                if sub_text.is_empty() || is_traffic_light_dot(sub) {
                    continue;
                }
                let (item_type, config) = classify_terminal_line(&sub_text, sub);
                items.push(ItemBlueprint {
                    item_type,
                    title: sub_text,
                    description: None,
                    config,
                });
            }
        } else {
            let (item_type, config) = classify_terminal_line(&text, child);
            items.push(ItemBlueprint {
                item_type,
                title: text,
                description: None,
                config,
            });
        }
    }
}

/// Classify a terminal line into type (line/success/prompt/output) and detect color.
fn classify_terminal_line(text: &str, node: &DomNode) -> (String, HashMap<String, String>) {
    let mut config = HashMap::new();

    // Detect color from node classes
    if let Some(color) = detect_text_color(node) {
        config.insert("color".into(), color);
    }

    let item_type = if text.starts_with('$') || text.starts_with("$ ") {
        "line"
    } else if text.starts_with('>') || text.starts_with("> ") {
        "line"
    } else if text.contains('\u{2713}') || text.contains("✓") || text.starts_with("✓") {
        "success"
    } else if text.contains('?') && text.len() < 100 && (text.contains("(Y/n)") || text.contains("(y/N)") || text.ends_with('?')) {
        "prompt"
    } else {
        "output"
    };

    (item_type.to_string(), config)
}

/// Check if a node is a traffic-light dot (terminal chrome decoration).
fn is_traffic_light_dot(node: &DomNode) -> bool {
    if !dom::has_class(node, "rounded-full") {
        return false;
    }
    // Tiny elements with colored bg (red/yellow/green dots)
    let dot_colors = ["bg-red-", "bg-yellow-", "bg-green-", "bg-orange-"];
    dot_colors.iter().any(|c| dom::has_class(node, c))
}

// ---------------------------------------------------------------------------
// Extraction: sidebar
// ---------------------------------------------------------------------------

fn extract_sidebar(node: &DomNode) -> SectionBlueprint {
    let mut config: HashMap<String, String> = HashMap::new();
    let mut items: Vec<ItemBlueprint> = Vec::new();

    // Brand text: look for bold/large text in the sidebar header area
    let brand = find_brand_text(node).unwrap_or_default();
    if !brand.is_empty() {
        config.insert("brand".into(), brand);
    }

    // Nav links with icon names
    let links = dom::extract_links(node);
    for (text, href) in &links {
        let clean = text.trim().to_string();
        if clean.is_empty() {
            continue;
        }

        let mut item_config: HashMap<String, String> = HashMap::new();
        item_config.insert("href".into(), href.clone());

        // Try to find the material icon associated with this link
        // Search all <a> tags and match by cleaned text content
        let a_nodes = dom::find_by_tag(node, "a");
        for a_node in &a_nodes {
            let a_clean = dom::clean_node_text(a_node);
            if a_clean == clean {
                if let Some(icon) = extract_material_icon(a_node) {
                    item_config.insert("icon".into(), icon);
                }
                break;
            }
        }

        items.push(ItemBlueprint {
            item_type: "nav-link".into(),
            title: clean,
            description: None,
            config: item_config,
        });
    }

    // Also extract buttons in sidebar (e.g., "Create Payment")
    let buttons = extract_clean_buttons(node);
    for btn in &buttons {
        if !btn.is_empty() {
            items.push(ItemBlueprint {
                item_type: "action".into(),
                title: btn.clone(),
                description: None,
                config: HashMap::new(),
            });
        }
    }

    SectionBlueprint {
        section_type: "sidebar".into(),
        confidence: 0.0,
        title: None,
        subtitle: None,
        config,
        items,
    }
}

// ---------------------------------------------------------------------------
// Extraction: page-header
// ---------------------------------------------------------------------------

fn extract_page_header(node: &DomNode) -> SectionBlueprint {
    let config: HashMap<String, String> = HashMap::new();
    let mut items: Vec<ItemBlueprint> = Vec::new();

    // Title from h1
    let title = find_heading_by_tag(node, "h1");

    // Subtitle from p
    let subtitle = dom::find_paragraph(node);

    // Action button as item
    let buttons = extract_clean_buttons(node);
    for btn in &buttons {
        if !btn.is_empty() {
            let mut item_config: HashMap<String, String> = HashMap::new();
            // Try to find icon on the button
            let btn_nodes = dom::find_by_tag(node, "button");
            for bn in &btn_nodes {
                let btn_text = clean_button_text(bn);
                if btn_text == *btn {
                    if let Some(icon) = extract_material_icon(bn) {
                        item_config.insert("icon".into(), icon);
                    }
                    break;
                }
            }
            items.push(ItemBlueprint {
                item_type: "action".into(),
                title: btn.clone(),
                description: None,
                config: item_config,
            });
        }
    }

    SectionBlueprint {
        section_type: "page-header".into(),
        confidence: 0.0,
        title,
        subtitle,
        config,
        items,
    }
}

// ---------------------------------------------------------------------------
// Extraction: stat-cards
// ---------------------------------------------------------------------------

fn extract_stat_cards(node: &DomNode) -> SectionBlueprint {
    let mut items: Vec<ItemBlueprint> = Vec::new();

    // Stat cards are grid children: each has a small uppercase label + large bold value
    let children_to_check: Vec<&DomNode> = if node.classes.iter().any(|c| c.contains("grid-cols")) {
        // Node itself is the grid
        node.children.iter().collect()
    } else {
        // Grid is one level deeper
        let mut found = Vec::new();
        for child in &node.children {
            if child.classes.iter().any(|c| c.contains("grid-cols")) {
                found = child.children.iter().collect();
                break;
            }
        }
        if found.is_empty() {
            node.children.iter().collect()
        } else {
            found
        }
    };

    for card in &children_to_check {
        // Label: small uppercase text (tracking-widest, uppercase, text-xs/text-sm)
        let label = find_stat_label(card);
        // Value: large bold text (text-2xl, text-3xl, font-bold)
        let value = find_stat_value(card);

        if label.is_some() || value.is_some() {
            items.push(ItemBlueprint {
                item_type: "stat".into(),
                title: label.unwrap_or_default(),
                description: value,
                config: HashMap::new(),
            });
        }
    }

    SectionBlueprint {
        section_type: "stat-cards".into(),
        confidence: 0.0,
        title: None,
        subtitle: None,
        config: HashMap::new(),
        items,
    }
}

/// Find the large bold stat value in a stat card.
fn find_stat_value(node: &DomNode) -> Option<String> {
    let value_classes = ["text-2xl", "text-3xl", "text-4xl", "text-5xl"];
    for cls in &value_classes {
        let matches = dom::find_by_class(node, cls);
        for m in matches {
            let txt = m.full_text.trim().to_string();
            if !txt.is_empty() {
                return Some(txt);
            }
        }
    }
    // Fallback: find font-bold that looks numeric
    let bold_matches = dom::find_by_class(node, "font-bold");
    for m in bold_matches {
        let txt = m.full_text.trim().to_string();
        if !txt.is_empty() && txt.len() < 30 && txt.chars().any(|c| c.is_ascii_digit()) {
            return Some(txt);
        }
    }
    None
}

/// Find the small uppercase label in a stat card (used by extract_stat_cards).
fn find_stat_label(node: &DomNode) -> Option<String> {
    let label_classes = ["uppercase", "tracking-widest", "tracking-wider"];
    for cls in &label_classes {
        let matches = dom::find_by_class(node, cls);
        for m in matches {
            let txt = m.full_text.trim().to_string();
            if !txt.is_empty() && txt.len() < 60 {
                return Some(txt);
            }
        }
    }
    // Fallback: find text-xs or text-sm elements
    for cls in &["text-xs", "text-sm"] {
        let matches = dom::find_by_class(node, cls);
        for m in matches {
            let txt = m.full_text.trim().to_string();
            if !txt.is_empty() && txt.len() < 60 {
                return Some(txt);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Extraction: product-grid
// ---------------------------------------------------------------------------

fn extract_product_grid(node: &DomNode) -> SectionBlueprint {
    let mut items: Vec<ItemBlueprint> = Vec::new();
    let mut section_config: HashMap<String, String> = HashMap::new();

    // Detect grid columns
    let cols = detect_grid_cols(node);
    if cols > 0 {
        section_config.insert("cols".into(), cols.to_string());
    }

    // Find the grid container
    let grid_container = find_grid_container(node);
    let cards: Vec<&DomNode> = if let Some(gc) = grid_container {
        gc.children.iter().collect()
    } else {
        find_card_children(node)
    };

    for card in &cards {
        let mut item_config: HashMap<String, String> = HashMap::new();

        // Name from h3
        let name = find_heading_by_tag(card, "h3")
            .or_else(|| find_heading_by_tag(card, "h2"))
            .unwrap_or_default();

        // Description from p
        let description = dom::find_paragraph(card);

        // Price: look for text with $ pattern
        let price = find_price_text(card);
        if let Some(ref p) = price {
            item_config.insert("price".into(), p.clone());
        }

        // Status badge (Active/Draft)
        let badge = find_status_badge(card);
        if let Some(ref b) = badge {
            item_config.insert("status".into(), b.clone());
        }

        // Icon (material symbol)
        if let Some(icon) = extract_material_icon(card) {
            item_config.insert("icon".into(), icon);
        }

        // Detect col-span
        for cls in &card.classes {
            if cls.contains("col-span-") {
                if let Some(pos) = cls.find("col-span-") {
                    let after = &cls[pos + "col-span-".len()..];
                    let n: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
                    if !n.is_empty() {
                        item_config.insert("span".into(), n);
                    }
                }
            }
        }

        // Dark card style
        if dom::has_class(card, "bg-black") || dom::has_class(card, "bg-primary") {
            item_config.insert("style".into(), "dark".into());
        }

        if !name.is_empty() || description.is_some() || price.is_some() {
            items.push(ItemBlueprint {
                item_type: "product".into(),
                title: name,
                description,
                config: item_config,
            });
        }
    }

    SectionBlueprint {
        section_type: "product-grid".into(),
        confidence: 0.0,
        title: None,
        subtitle: None,
        config: section_config,
        items,
    }
}

/// Find price text ($XX.XX) in a node.
fn find_price_text(node: &DomNode) -> Option<String> {
    // Look for text with $ sign in bold/large elements
    let bold_classes = ["font-bold", "text-2xl", "text-3xl", "tracking-tighter"];
    for cls in &bold_classes {
        let matches = dom::find_by_class(node, cls);
        for m in matches {
            let txt = m.full_text.trim().to_string();
            if txt.contains('$') && txt.len() < 40 {
                return Some(txt);
            }
        }
    }
    // Fallback: scan all text for $ pattern
    for child in &node.children {
        if let Some(p) = find_price_text(child) {
            return Some(p);
        }
    }
    None
}

/// Find status badge text (Active, Draft, etc.) in a node.
fn find_status_badge(node: &DomNode) -> Option<String> {
    let badge_classes = ["rounded-full", "badge", "chip"];
    for cls in &badge_classes {
        let matches = dom::find_by_class(node, cls);
        for m in matches {
            // Skip buttons and material icon spans
            if m.tag == "button" || m.tag == "a" || is_material_icon_span(m) {
                continue;
            }
            let txt = m.full_text.trim().to_string();
            let lower = txt.to_lowercase();
            if (lower.contains("active") || lower.contains("draft")
                || lower.contains("paused") || lower.contains("archived"))
                && txt.len() < 30
            {
                return Some(txt);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Extraction: team-list
// ---------------------------------------------------------------------------

fn extract_team_list(node: &DomNode) -> SectionBlueprint {
    let mut items: Vec<ItemBlueprint> = Vec::new();

    let title = find_heading_by_tag(node, "h2")
        .or_else(|| dom::find_heading(node));

    // Find member rows: look for repeated structures with name + email + role
    // Members are typically in a space-y container with flex rows
    let member_rows = find_member_rows(node);

    for row in &member_rows {
        let mut item_config: HashMap<String, String> = HashMap::new();

        // Name: bold text (font-bold in a <p> or <span>)
        let name = find_member_name(row);

        // Email: text containing @
        let email = find_email_text(row);
        if let Some(ref e) = email {
            item_config.insert("email".into(), e.clone());
        }

        // Role badge (Admin/Developer/Viewer)
        let role = find_role_badge(row);
        if let Some(ref r) = role {
            item_config.insert("role".into(), r.clone());
        }

        // Avatar image
        let images = dom::extract_images(row);
        if let Some((alt, src)) = images.first() {
            item_config.insert("avatar_src".into(), src.clone());
            if !alt.is_empty() {
                item_config.insert("avatar_alt".into(), alt.clone());
            }
        }

        if !name.is_empty() {
            items.push(ItemBlueprint {
                item_type: "member".into(),
                title: name,
                description: email,
                config: item_config,
            });
        }
    }

    SectionBlueprint {
        section_type: "team-list".into(),
        confidence: 0.0,
        title,
        subtitle: None,
        config: HashMap::new(),
        items,
    }
}

/// Find member rows in a team list node.
/// Looks for repeated flex rows with avatar + name + role structure.
fn find_member_rows<'a>(node: &'a DomNode) -> Vec<&'a DomNode> {
    let mut rows = Vec::new();

    // Look for children that are flex rows with justify-between (member row pattern)
    for child in &node.children {
        if dom::has_class(child, "justify-between") && has_avatar_or_image(child) {
            rows.push(child);
            continue;
        }
        // Check one level deeper (space-y wrapper)
        for grandchild in &child.children {
            if dom::has_class(grandchild, "justify-between") && has_avatar_or_image(grandchild) {
                rows.push(grandchild);
            }
        }
        // Check two levels deeper
        if rows.is_empty() {
            for grandchild in &child.children {
                for ggchild in &grandchild.children {
                    if dom::has_class(ggchild, "justify-between") && has_avatar_or_image(ggchild) {
                        rows.push(ggchild);
                    }
                }
            }
        }
    }
    rows
}

/// Check if a node contains an avatar (img or rounded-full placeholder).
fn has_avatar_or_image(node: &DomNode) -> bool {
    if dom::find_by_tag(node, "img").len() > 0 {
        return true;
    }
    if patterns::has_descendant_class(node, "rounded-full") {
        return true;
    }
    false
}

/// Find the member name (bold text that isn't an email or role), cleaned of icon text.
fn find_member_name(node: &DomNode) -> String {
    let bold_matches = dom::find_by_class(node, "font-bold");
    for m in &bold_matches {
        if is_material_icon_span(m) {
            continue;
        }
        let txt = dom::clean_node_text(m);
        if !txt.is_empty() && txt.len() < 50 && !txt.contains('@')
            && !is_role_text(&txt)
        {
            return txt;
        }
    }
    // Fallback: first <p> child with font-bold or font-semibold
    let p_nodes = dom::find_by_tag(node, "p");
    for p in &p_nodes {
        if dom::has_class(p, "font-bold") || dom::has_class(p, "font-semibold") {
            let txt = dom::clean_node_text(p);
            if !txt.is_empty() && !txt.contains('@') {
                return txt;
            }
        }
    }
    String::new()
}

/// Find email-like text (contains @) in a node.
fn find_email_text(node: &DomNode) -> Option<String> {
    // Look through all text nodes for @ pattern
    let p_nodes = dom::find_by_tag(node, "p");
    for p in &p_nodes {
        let txt = p.full_text.trim().to_string();
        if txt.contains('@') && txt.len() < 80 {
            return Some(txt);
        }
    }
    let span_nodes = dom::find_by_tag(node, "span");
    for s in &span_nodes {
        let txt = s.full_text.trim().to_string();
        if txt.contains('@') && txt.len() < 80 {
            return Some(txt);
        }
    }
    None
}

/// Find role badge text (Admin, Developer, Viewer, etc.).
fn find_role_badge(node: &DomNode) -> Option<String> {
    let badge_classes = ["rounded-full", "badge", "chip"];
    for cls in &badge_classes {
        let matches = dom::find_by_class(node, cls);
        for m in matches {
            if m.tag == "button" || m.tag == "a" || is_material_icon_span(m) {
                continue;
            }
            // Skip if it's an avatar (img container)
            if dom::find_by_tag(m, "img").len() > 0 {
                continue;
            }
            let txt = m.full_text.trim().to_string();
            if is_role_text(&txt) && txt.len() < 30 {
                return Some(txt);
            }
        }
    }
    None
}

/// Check if text looks like a role name.
fn is_role_text(text: &str) -> bool {
    let lower = text.to_lowercase();
    let roles = ["admin", "developer", "viewer", "editor", "owner", "member", "manager"];
    roles.iter().any(|r| lower.contains(r))
}

// ---------------------------------------------------------------------------
// Extraction: content card (API keys, forms, settings)
// ---------------------------------------------------------------------------

fn extract_content_card(node: &DomNode) -> SectionBlueprint {
    let mut items: Vec<ItemBlueprint> = Vec::new();

    // Title from h2/h3
    let title = find_heading_by_tag(node, "h3")
        .or_else(|| find_heading_by_tag(node, "h2"));

    // Subtitle from first p
    let subtitle = dom::find_paragraph(node);

    // Extract labels (uppercase tracking-widest text — form field labels)
    let label_nodes = dom::find_by_class(node, "uppercase");
    for label_node in &label_nodes {
        if !dom::has_class(label_node, "tracking-widest") && !dom::has_class(label_node, "tracking-wider") {
            continue;
        }
        let txt = label_node.full_text.trim().to_string();
        if !txt.is_empty() && txt.len() < 80 {
            items.push(ItemBlueprint {
                item_type: "label".into(),
                title: txt,
                description: None,
                config: HashMap::new(),
            });
        }
    }

    // Extract input/code values (font-mono text = API keys, codes)
    let mono_nodes = dom::find_by_class(node, "font-mono");
    for mono in &mono_nodes {
        if is_material_icon_span(mono) {
            continue;
        }
        let txt = mono.full_text.trim().to_string();
        if !txt.is_empty() && txt.len() < 200 {
            let mut code_config: HashMap<String, String> = HashMap::new();
            code_config.insert("_type".into(), "code".into());
            items.push(ItemBlueprint {
                item_type: "code".into(),
                title: txt,
                description: None,
                config: code_config,
            });
        }
    }

    // Extract action buttons (e.g. "Reveal", "Hide", "Add Endpoint")
    let buttons = extract_clean_buttons(node);
    for btn in &buttons {
        if !btn.is_empty() {
            let mut btn_config: HashMap<String, String> = HashMap::new();
            btn_config.insert("_type".into(), "action".into());
            items.push(ItemBlueprint {
                item_type: "action".into(),
                title: btn.clone(),
                description: None,
                config: btn_config,
            });
        }
    }

    // Extract webhook URLs and status badges (font-mono links with status spans)
    let webhook_divs = find_webhook_entries(node);
    for (url, status, events) in webhook_divs {
        let mut wh_config: HashMap<String, String> = HashMap::new();
        if let Some(ref s) = status {
            wh_config.insert("status".into(), s.clone());
        }
        items.push(ItemBlueprint {
            item_type: "webhook".into(),
            title: url,
            description: events,
            config: wh_config,
        });
    }

    SectionBlueprint {
        section_type: "card".into(),
        confidence: 0.0,
        title,
        subtitle,
        config: HashMap::new(),
        items,
    }
}

/// Find webhook-like entries: font-mono URL + status badge + event description.
fn find_webhook_entries(node: &DomNode) -> Vec<(String, Option<String>, Option<String>)> {
    let mut entries = Vec::new();
    let mono_nodes = dom::find_by_class(node, "font-mono");
    for mono in &mono_nodes {
        let txt = mono.full_text.trim().to_string();
        // URL-like: starts with http or contains a domain pattern
        if txt.starts_with("http") || txt.contains("://") {
            // Find sibling status badge
            let parent = find_parent_of(node, mono);
            let status = parent.and_then(|p| find_status_badge(p));
            let events = parent.and_then(|p| dom::find_paragraph(p));
            entries.push((txt, status, events));
        }
    }
    entries
}

// ---------------------------------------------------------------------------
// Extraction: info panel (promo, links, status)
// ---------------------------------------------------------------------------

fn extract_info_panel(node: &DomNode) -> SectionBlueprint {
    let mut config: HashMap<String, String> = HashMap::new();
    let mut items: Vec<ItemBlueprint> = Vec::new();

    // Determine sub-type based on content
    let is_dark = dom::has_class(node, "bg-primary") || dom::has_class(node, "bg-black")
        || dom::has_class(node, "bg-gray-900") || dom::has_class(node, "bg-zinc-900");
    let has_status_dot = patterns::has_descendant_class(node, "bg-green-500")
        || patterns::has_descendant_class(node, "bg-green-400");
    let has_link_list = dom::find_by_tag(node, "ul").len() > 0
        || dom::extract_links(node).len() >= 3;

    let sub_type = if has_status_dot {
        let text = node.full_text.to_lowercase();
        if text.contains("operational") || text.contains("status") || text.contains("health") {
            "status-card"
        } else {
            "info-panel"
        }
    } else if is_dark {
        "promo"
    } else if has_link_list {
        "links"
    } else {
        "info-panel"
    };
    config.insert("panel_type".into(), sub_type.to_string());

    // Title from h4/h3
    let title = find_heading_by_tag(node, "h4")
        .or_else(|| find_heading_by_tag(node, "h3"))
        .or_else(|| find_heading_by_tag(node, "h2"));

    // Subtitle/description from p
    let subtitle = dom::find_paragraph(node);

    // Extract links as items
    let links = dom::extract_links(node);
    for (text, href) in &links {
        let mut link_config: HashMap<String, String> = HashMap::new();
        link_config.insert("href".into(), href.clone());
        // Check for external icon
        if let Some(parent_a) = find_link_node_by_text(node, text) {
            if let Some(icon) = extract_material_icon(parent_a) {
                link_config.insert("icon".into(), icon);
            }
        }
        items.push(ItemBlueprint {
            item_type: "link".into(),
            title: text.clone(),
            description: None,
            config: link_config,
        });
    }

    // For status cards, extract the status text
    if sub_type == "status-card" {
        // Find text near the green dot
        let status_text = find_status_text(node);
        if let Some(st) = status_text {
            let mut st_config: HashMap<String, String> = HashMap::new();
            st_config.insert("status".into(), "operational".into());
            items.push(ItemBlueprint {
                item_type: "status".into(),
                title: st,
                description: None,
                config: st_config,
            });
        }
    }

    SectionBlueprint {
        section_type: sub_type.to_string(),
        confidence: 0.0,
        title,
        subtitle,
        config,
        items,
    }
}

/// Find the <a> node whose clean text matches the given text.
fn find_link_node_by_text<'a>(node: &'a DomNode, text: &str) -> Option<&'a DomNode> {
    let a_nodes = dom::find_by_tag(node, "a");
    for a_node in a_nodes {
        let a_text = dom::clean_node_text(a_node);
        if a_text == text {
            return Some(a_node);
        }
    }
    None
}

/// Find status text near a green status dot.
fn find_status_text(node: &DomNode) -> Option<String> {
    // Look for a span/p sibling of a green dot
    for child in &node.children {
        let has_dot = child.children.iter().any(|c| {
            dom::has_class(c, "rounded-full")
                && (dom::has_class(c, "bg-green-500") || dom::has_class(c, "bg-green-400"))
        });
        if has_dot {
            // Find the text sibling
            for sibling in &child.children {
                if sibling.tag == "span" || sibling.tag == "p" {
                    let txt = sibling.full_text.trim().to_string();
                    if !txt.is_empty() && txt.len() < 100 {
                        return Some(txt);
                    }
                }
            }
            // Fallback: child's own text minus the dot
            let txt = dom::clean_node_text(child);
            if !txt.is_empty() {
                return Some(txt);
            }
        }
        // Recurse
        if let Some(st) = find_status_text(child) {
            return Some(st);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Extraction: generic (fallback) — smart dashboard element detection
// ---------------------------------------------------------------------------

fn extract_generic(node: &DomNode) -> SectionBlueprint {
    let mut items: Vec<ItemBlueprint> = Vec::new();

    let title = dom::find_heading(node);
    let subtitle = dom::find_paragraph(node);

    // 1. Cards: rounded containers with headings (dashboard cards, settings panels)
    let card_items = extract_card_items(node);
    if !card_items.is_empty() {
        items.extend(card_items);
    }

    // 2. Tables: <table> or structured div grids with consistent rows
    let (table_title, table_items) = extract_table_items(node);
    if !table_items.is_empty() {
        items.extend(table_items);
    }

    // 3. Form fields: <label> + <input>/<select> pairs
    let form_items = extract_form_items(node);
    if !form_items.is_empty() {
        items.extend(form_items);
    }

    // 4. Stat cards: small cards with big number + small label
    let stat_items = extract_inline_stat_items(node);
    if !stat_items.is_empty() {
        items.extend(stat_items);
    }

    // 5. Progress bars: divs with percentage width children
    let progress_items = extract_progress_items(node);
    if !progress_items.is_empty() {
        items.extend(progress_items);
    }

    // 6. Status indicators: colored dots + text
    let status_items = extract_status_indicator_items(node);
    if !status_items.is_empty() {
        items.extend(status_items);
    }

    // 7. Badge/pills: rounded-full + small text + uppercase
    let badge_items = extract_badge_items(node);
    if !badge_items.is_empty() {
        items.extend(badge_items);
    }

    // 8. Code/mono text blocks
    let code_items = extract_code_items(node);
    if !code_items.is_empty() {
        items.extend(code_items);
    }

    // 9. Buttons/actions
    let buttons = extract_clean_buttons(node);
    for btn in &buttons {
        if !btn.is_empty() {
            let mut btn_config: HashMap<String, String> = HashMap::new();
            // Try to find icon on the button
            let btn_nodes = dom::find_by_tag(node, "button");
            for bn in &btn_nodes {
                let btn_text = clean_button_text(bn);
                if btn_text == *btn {
                    if let Some(icon) = extract_material_icon(bn) {
                        btn_config.insert("icon".into(), icon);
                    }
                    break;
                }
            }
            items.push(ItemBlueprint {
                item_type: "action".into(),
                title: btn.clone(),
                description: None,
                config: btn_config,
            });
        }
    }

    // 10. Fallback: headings + paragraphs + links + images not already captured
    if items.is_empty() {
        // Headings (cleaned of material-symbols text)
        let heading_tags = ["h1", "h2", "h3", "h4", "h5", "h6"];
        for tag in &heading_tags {
            let headings = dom::find_by_tag(node, tag);
            for h in headings {
                let text = dom::clean_node_text(h);
                if !text.is_empty() {
                    items.push(ItemBlueprint {
                        item_type: "item".into(),
                        title: text,
                        description: None,
                        config: HashMap::new(),
                    });
                }
            }
        }

        // Paragraphs
        let paragraphs = dom::find_by_tag(node, "p");
        for p in paragraphs {
            let text = dom::clean_node_text(p);
            if !text.is_empty() {
                items.push(ItemBlueprint {
                    item_type: "item".into(),
                    title: text,
                    description: None,
                    config: HashMap::new(),
                });
            }
        }

        // Links
        let links = dom::extract_links(node);
        for (text, href) in links {
            let mut link_config = HashMap::new();
            link_config.insert("href".into(), href);
            items.push(ItemBlueprint {
                item_type: "link".into(),
                title: text,
                description: None,
                config: link_config,
            });
        }

        // Images
        let images = dom::extract_images(node);
        for (alt, src) in images {
            let mut img_config = HashMap::new();
            img_config.insert("src".into(), src);
            items.push(ItemBlueprint {
                item_type: "image".into(),
                title: alt,
                description: None,
                config: img_config,
            });
        }
    }

    // Use table title as section title if we found one and had no heading
    let final_title = title.or(table_title);

    SectionBlueprint {
        section_type: "generic".into(),
        confidence: 0.0,
        title: final_title,
        subtitle,
        config: HashMap::new(),
        items,
    }
}

// ---------------------------------------------------------------------------
// Card extraction: rounded containers with headings
// ---------------------------------------------------------------------------

/// Extract card-like elements from a node. Cards are divs with border/rounded/shadow
/// that contain a heading (h3/h4) and inner content.
fn extract_card_items(node: &DomNode) -> Vec<ItemBlueprint> {
    let mut items = Vec::new();

    // Collect card-like direct children
    let card_nodes = find_card_like_children(node);
    if card_nodes.is_empty() {
        return items;
    }

    for card in &card_nodes {
        let card_title = find_heading_by_tag(card, "h3")
            .or_else(|| find_heading_by_tag(card, "h4"))
            .or_else(|| find_heading_by_tag(card, "h2"))
            .unwrap_or_default();

        let card_desc = dom::find_paragraph(card);

        // Skip cards with no discernible content
        if card_title.is_empty() && card_desc.is_none() {
            continue;
        }

        let mut card_config: HashMap<String, String> = HashMap::new();
        card_config.insert("_type".into(), "card".into());

        // Detect dark card style
        if dom::has_class(card, "bg-black") || dom::has_class(card, "bg-primary")
            || dom::has_class(card, "bg-surface-container-highest")
        {
            card_config.insert("style".into(), "dark".into());
        }

        // Extract material icon
        if let Some(icon) = extract_material_icon(card) {
            card_config.insert("icon".into(), icon);
        }

        // Extract status badge inside card
        if let Some(badge) = find_status_badge(card) {
            card_config.insert("status".into(), badge);
        }

        items.push(ItemBlueprint {
            item_type: "card".into(),
            title: card_title,
            description: card_desc,
            config: card_config,
        });

        // Extract child elements within the card: labels, code, buttons, badges
        // Labels (font-mono small text)
        let mono_nodes = dom::find_by_class(card, "font-mono");
        for mono in &mono_nodes {
            if mono.tag == "code" || mono.tag == "pre" || is_material_icon_span(mono) {
                continue;
            }
            let is_small = dom::has_class(mono, "text-xs") || dom::has_class(mono, "text-sm")
                || mono.classes.iter().any(|c| c.contains("text-["));
            if !is_small {
                continue;
            }
            let label_text = mono.full_text.trim().to_string();
            if !label_text.is_empty() && label_text.len() < 80 {
                let mut label_config: HashMap<String, String> = HashMap::new();
                label_config.insert("style".into(), "mono".into());
                label_config.insert("role".into(), "child".into());
                items.push(ItemBlueprint {
                    item_type: "label".into(),
                    title: label_text,
                    description: None,
                    config: label_config,
                });
            }
        }

        // Code blocks inside card
        let code_els = dom::find_by_tag(card, "code");
        for code_el in code_els {
            let code_text = code_el.full_text.trim().to_string();
            if !code_text.is_empty() {
                let mut code_config: HashMap<String, String> = HashMap::new();
                code_config.insert("role".into(), "child".into());
                items.push(ItemBlueprint {
                    item_type: "code".into(),
                    title: code_text,
                    description: None,
                    config: code_config,
                });
            }
        }

        // Action buttons inside card
        let card_buttons = extract_clean_buttons(card);
        for btn in &card_buttons {
            if !btn.is_empty() {
                let mut btn_config: HashMap<String, String> = HashMap::new();
                btn_config.insert("role".into(), "child".into());
                items.push(ItemBlueprint {
                    item_type: "action".into(),
                    title: btn.clone(),
                    description: None,
                    config: btn_config,
                });
            }
        }

        // Form fields inside card
        let card_fields = extract_form_items(card);
        for field in card_fields {
            let mut field_with_role = field;
            field_with_role.config.insert("role".into(), "child".into());
            items.push(field_with_role);
        }
    }

    items
}

/// Find direct children that look like cards (div with border/rounded/shadow + heading).
fn find_card_like_children(node: &DomNode) -> Vec<&DomNode> {
    let mut cards = Vec::new();

    // Check direct children
    for child in &node.children {
        if is_card_like(child) {
            cards.push(child);
            continue;
        }
        // Check one level deeper (wrapper div)
        for grandchild in &child.children {
            if is_card_like(grandchild) {
                cards.push(grandchild);
            }
        }
    }

    // Also check grid containers
    if cards.is_empty() {
        if let Some(gc) = find_grid_container(node) {
            for child in &gc.children {
                if is_card_like(child) || dom::find_heading(child).is_some() {
                    cards.push(child);
                }
            }
        }
    }

    cards
}

/// Check if a node looks like a card (has border/rounded/shadow + contains a heading).
fn is_card_like(node: &DomNode) -> bool {
    if node.tag != "div" && node.tag != "article" && node.tag != "section" {
        return false;
    }
    let has_card_class = dom::has_class(node, "rounded")
        || dom::has_class(node, "border")
        || dom::has_class(node, "shadow")
        || dom::has_class(node, "card")
        || dom::has_class(node, "ghost-border");

    if !has_card_class {
        return false;
    }

    // Must contain a heading or substantial text content
    dom::find_heading(node).is_some()
        || dom::find_paragraph(node).is_some()
        || !node.full_text.trim().is_empty()
}

// ---------------------------------------------------------------------------
// Table extraction: <table> or structured div grids
// ---------------------------------------------------------------------------

/// Extract table items from a node. Returns optional caption and row items.
fn extract_table_items(node: &DomNode) -> (Option<String>, Vec<ItemBlueprint>) {
    let mut items = Vec::new();
    let mut caption = None;

    // Strategy 1: HTML <table> elements
    let tables = dom::find_by_tag(node, "table");
    for table in &tables {
        // Caption from <caption> or preceding heading
        let cap_nodes = dom::find_by_tag(table, "caption");
        if let Some(cap) = cap_nodes.first() {
            let txt = dom::clean_node_text(cap);
            if !txt.is_empty() {
                caption = Some(txt);
            }
        }

        // Column headers from <thead> <th>
        let mut columns: Vec<String> = Vec::new();
        let thead_nodes = dom::find_by_tag(table, "thead");
        if let Some(thead) = thead_nodes.first() {
            let th_nodes = dom::find_by_tag(thead, "th");
            for th in &th_nodes {
                let txt = dom::clean_node_text(th);
                columns.push(txt);
            }
        }

        // If no thead, try first <tr> for headers
        if columns.is_empty() {
            let all_rows = dom::find_by_tag(table, "tr");
            if let Some(first_row) = all_rows.first() {
                let th_nodes = dom::find_by_tag(first_row, "th");
                if !th_nodes.is_empty() {
                    for th in &th_nodes {
                        let txt = dom::clean_node_text(th);
                        columns.push(txt);
                    }
                }
            }
        }

        // Emit column header item
        if !columns.is_empty() {
            let mut header_config: HashMap<String, String> = HashMap::new();
            header_config.insert("_type".into(), "table-header".into());
            header_config.insert("columns".into(), columns.join(" | "));
            items.push(ItemBlueprint {
                item_type: "table-header".into(),
                title: columns.join(" | "),
                description: None,
                config: header_config,
            });
        }

        // Data rows from <tbody> <tr>
        let tbody_nodes = dom::find_by_tag(table, "tbody");
        let data_rows: Vec<&DomNode> = if let Some(tbody) = tbody_nodes.first() {
            dom::find_by_tag(tbody, "tr")
        } else {
            // No tbody, use all tr except the first (header)
            let all_rows = dom::find_by_tag(table, "tr");
            if !columns.is_empty() && all_rows.len() > 1 {
                all_rows.into_iter().skip(1).collect()
            } else {
                all_rows
            }
        };

        for row in &data_rows {
            let td_nodes = dom::find_by_tag(row, "td");
            let cells: Vec<String> = td_nodes.iter()
                .map(|td| dom::clean_node_text(td))
                .collect();

            if cells.is_empty() || cells.iter().all(|c| c.is_empty()) {
                continue;
            }

            let mut row_config: HashMap<String, String> = HashMap::new();
            row_config.insert("_type".into(), "row".into());

            // Map cell values to column names if available
            for (i, cell) in cells.iter().enumerate() {
                if !cell.is_empty() {
                    let key = if i < columns.len() && !columns[i].is_empty() {
                        columns[i].clone()
                    } else {
                        format!("col_{}", i)
                    };
                    row_config.insert(key, cell.clone());
                }
            }

            let row_title = cells.iter()
                .find(|c| !c.is_empty())
                .cloned()
                .unwrap_or_default();

            items.push(ItemBlueprint {
                item_type: "row".into(),
                title: row_title,
                description: None,
                config: row_config,
            });
        }
    }

    // Strategy 2: Structured divs with role="table" or consistent row patterns
    if items.is_empty() {
        let role_tables: Vec<&DomNode> = find_nodes_with_attr(node, "role", "table");
        for rt in &role_tables {
            let rows = find_nodes_with_attr(rt, "role", "row");
            for row in &rows {
                let cells = find_nodes_with_attr(row, "role", "cell");
                let cell_texts: Vec<String> = cells.iter()
                    .map(|c| dom::clean_node_text(c))
                    .collect();

                if cell_texts.is_empty() || cell_texts.iter().all(|c| c.is_empty()) {
                    continue;
                }

                let mut row_config: HashMap<String, String> = HashMap::new();
                row_config.insert("_type".into(), "row".into());
                for (i, cell) in cell_texts.iter().enumerate() {
                    if !cell.is_empty() {
                        row_config.insert(format!("col_{}", i), cell.clone());
                    }
                }

                items.push(ItemBlueprint {
                    item_type: "row".into(),
                    title: cell_texts.first().cloned().unwrap_or_default(),
                    description: None,
                    config: row_config,
                });
            }
        }
    }

    (caption, items)
}

/// Find nodes with a specific attribute value, recursively.
fn find_nodes_with_attr<'a>(node: &'a DomNode, attr: &str, value: &str) -> Vec<&'a DomNode> {
    let mut results = Vec::new();
    if node.attrs.get(attr).map(|v| v.as_str()) == Some(value) {
        results.push(node);
    }
    for child in &node.children {
        results.extend(find_nodes_with_attr(child, attr, value));
    }
    results
}

// ---------------------------------------------------------------------------
// Form field extraction: <label> + <input>/<select> pairs
// ---------------------------------------------------------------------------

/// Extract form fields from a node. Finds <label> + <input>/<select> pairs.
fn extract_form_items(node: &DomNode) -> Vec<ItemBlueprint> {
    let mut items = Vec::new();

    // Strategy 1: <label> elements with associated inputs
    let labels = dom::find_by_tag(node, "label");
    for label in &labels {
        let label_text = dom::clean_node_text(label);
        if label_text.is_empty() {
            continue;
        }

        let mut field_config: HashMap<String, String> = HashMap::new();
        field_config.insert("_type".into(), "field".into());

        // Check for <input> or <select> inside the label
        let mut found_input = false;
        let inner_inputs = dom::find_by_tag(label, "input");
        let inner_selects = dom::find_by_tag(label, "select");
        let inner_textareas = dom::find_by_tag(label, "textarea");

        for input in inner_inputs.iter().chain(inner_selects.iter()).chain(inner_textareas.iter()) {
            found_input = true;
            extract_input_attrs(input, &mut field_config);
        }

        // Check for sibling input via "for" attribute → id matching
        if !found_input {
            if let Some(for_attr) = label.attrs.get("for") {
                if let Some(input) = find_node_by_id(node, for_attr) {
                    found_input = true;
                    extract_input_attrs(input, &mut field_config);
                }
            }
        }

        // Check for input as next sibling (common pattern: label followed by input)
        if !found_input {
            if let Some(input) = find_sibling_input(node, label) {
                // found_input would be true here but not read again
                extract_input_attrs(input, &mut field_config);
            }
        }

        items.push(ItemBlueprint {
            item_type: "field".into(),
            title: label_text,
            description: None,
            config: field_config,
        });
    }

    // Strategy 2: Input elements with placeholder text (no label)
    if items.is_empty() {
        let inputs = dom::find_by_tag(node, "input");
        for input in &inputs {
            let placeholder = input.attrs.get("placeholder").cloned().unwrap_or_default();
            let input_type = input.attrs.get("type").cloned().unwrap_or_else(|| "text".into());
            if placeholder.is_empty() && input_type == "hidden" {
                continue;
            }
            let title = if !placeholder.is_empty() {
                placeholder.clone()
            } else {
                format!("[{} input]", input_type)
            };

            let mut field_config: HashMap<String, String> = HashMap::new();
            field_config.insert("_type".into(), "field".into());
            extract_input_attrs(input, &mut field_config);

            items.push(ItemBlueprint {
                item_type: "field".into(),
                title,
                description: None,
                config: field_config,
            });
        }
    }

    items
}

/// Extract useful attributes from an input/select/textarea node.
fn extract_input_attrs(input: &DomNode, config: &mut HashMap<String, String>) {
    if let Some(t) = input.attrs.get("type") {
        config.insert("input_type".into(), t.clone());
    }
    if let Some(p) = input.attrs.get("placeholder") {
        if !p.is_empty() {
            config.insert("placeholder".into(), p.clone());
        }
    }
    if let Some(v) = input.attrs.get("value") {
        if !v.is_empty() {
            config.insert("value".into(), v.clone());
        }
    }
    if let Some(n) = input.attrs.get("name") {
        if !n.is_empty() {
            config.insert("name".into(), n.clone());
        }
    }
    if input.attrs.contains_key("disabled") {
        config.insert("disabled".into(), "true".into());
    }
    if input.attrs.contains_key("readonly") {
        config.insert("readonly".into(), "true".into());
    }
    if input.tag == "select" {
        // Extract <option> values
        let options = dom::find_by_tag(input, "option");
        let opt_texts: Vec<String> = options.iter()
            .filter_map(|o| {
                let txt = dom::clean_node_text(o);
                if txt.is_empty() { None } else { Some(txt) }
            })
            .collect();
        if !opt_texts.is_empty() {
            config.insert("options".into(), opt_texts.join(", "));
        }
    }
    if input.tag == "textarea" {
        config.insert("input_type".into(), "textarea".into());
    }
}

/// Find a node by its id attribute.
fn find_node_by_id<'a>(node: &'a DomNode, id: &str) -> Option<&'a DomNode> {
    if node.id.as_deref() == Some(id) {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_node_by_id(child, id) {
            return Some(found);
        }
    }
    None
}

/// Find a sibling input/select/textarea element near a label.
fn find_sibling_input<'a>(parent: &'a DomNode, label: &DomNode) -> Option<&'a DomNode> {
    let mut found_label = false;
    for child in &parent.children {
        if std::ptr::eq(child, label) {
            found_label = true;
            continue;
        }
        if found_label {
            if child.tag == "input" || child.tag == "select" || child.tag == "textarea" {
                return Some(child);
            }
            // Check one level deeper (wrapper div around input)
            for grandchild in &child.children {
                if grandchild.tag == "input" || grandchild.tag == "select" || grandchild.tag == "textarea" {
                    return Some(grandchild);
                }
            }
        }
    }
    // Also check children of the parent recursively (label and input may be in
    // separate wrapper divs at the same level)
    for child in &parent.children {
        if let Some(found) = find_sibling_input(child, label) {
            return Some(found);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Inline stat card extraction
// ---------------------------------------------------------------------------

/// Extract stat cards that appear inline: small cards with a big number + small label.
fn extract_inline_stat_items(node: &DomNode) -> Vec<ItemBlueprint> {
    let mut items = Vec::new();

    // Look for grid containers with stat-card-like children
    let grid_nodes = dom::find_by_class(node, "grid-cols-");
    for grid in &grid_nodes {
        if std::ptr::eq(*grid, node) {
            continue;
        }

        let mut candidates: Vec<ItemBlueprint> = Vec::new();
        for child in &grid.children {
            let large = find_large_text(child)
                .or_else(|| find_stat_value(child));
            let label = find_small_label(child)
                .or_else(|| find_stat_label(child));

            if let (Some(val), Some(lbl)) = (large, label) {
                let mut config: HashMap<String, String> = HashMap::new();
                config.insert("_type".into(), "stat".into());
                config.insert("value".into(), val.clone());
                candidates.push(ItemBlueprint {
                    item_type: "stat".into(),
                    title: lbl,
                    description: Some(val),
                    config,
                });
            }
        }

        if candidates.len() >= 2 {
            items.extend(candidates);
            return items; // Found stat group, done
        }
    }

    items
}

// ---------------------------------------------------------------------------
// Progress bar extraction
// ---------------------------------------------------------------------------

/// Extract progress bars: divs with an inner div that has a width percentage style.
fn extract_progress_items(node: &DomNode) -> Vec<ItemBlueprint> {
    let mut items = Vec::new();

    // Look for elements with role="progressbar"
    let progress_nodes = find_nodes_with_attr(node, "role", "progressbar");
    for pn in &progress_nodes {
        let value = pn.attrs.get("aria-valuenow")
            .or_else(|| pn.attrs.get("aria-value"))
            .cloned()
            .unwrap_or_default();
        let label = pn.attrs.get("aria-label")
            .cloned()
            .or_else(|| {
                let parent = find_parent_of(node, pn);
                parent.and_then(|p| dom::find_paragraph(p))
            })
            .unwrap_or_default();

        let mut config: HashMap<String, String> = HashMap::new();
        config.insert("_type".into(), "meter".into());
        if !value.is_empty() {
            config.insert("value".into(), value.clone());
        }
        items.push(ItemBlueprint {
            item_type: "meter".into(),
            title: label,
            description: if value.is_empty() { None } else { Some(format!("{}%", value)) },
            config,
        });
    }

    // Look for <progress> HTML elements
    let progress_els = dom::find_by_tag(node, "progress");
    for pe in &progress_els {
        let value = pe.attrs.get("value").cloned().unwrap_or_default();
        let max = pe.attrs.get("max").cloned().unwrap_or_else(|| "100".into());

        let mut config: HashMap<String, String> = HashMap::new();
        config.insert("_type".into(), "meter".into());
        if !value.is_empty() {
            config.insert("value".into(), value.clone());
            config.insert("max".into(), max);
        }
        items.push(ItemBlueprint {
            item_type: "meter".into(),
            title: String::new(),
            description: if value.is_empty() { None } else { Some(format!("{}%", value)) },
            config,
        });
    }

    // Look for div-based progress bars (outer track + inner bar with width style)
    if items.is_empty() {
        find_div_progress_bars(node, &mut items);
    }

    items
}

/// Find div-based progress bars by looking for a container with bg-* + rounded
/// that has an inner child with a style containing "width:" percentage.
fn find_div_progress_bars(node: &DomNode, items: &mut Vec<ItemBlueprint>) {
    // Check if this node is a progress bar track
    let is_track = (dom::has_class(node, "bg-gray") || dom::has_class(node, "bg-neutral")
        || dom::has_class(node, "bg-muted") || dom::has_class(node, "bg-surface"))
        && dom::has_class(node, "rounded");

    if is_track {
        for child in &node.children {
            if let Some(style) = child.attrs.get("style") {
                if let Some(pct) = extract_width_percent(style) {
                    let label = find_parent_of(node, node)
                        .and_then(|p| dom::find_paragraph(p))
                        .unwrap_or_default();

                    let mut config: HashMap<String, String> = HashMap::new();
                    config.insert("_type".into(), "meter".into());
                    config.insert("progress".into(), pct.clone());

                    items.push(ItemBlueprint {
                        item_type: "meter".into(),
                        title: label,
                        description: Some(pct),
                        config,
                    });
                    return;
                }
            }
            // Also check Tailwind w-[] classes for percentage
            for cls in &child.classes {
                if cls.starts_with("w-[") && cls.contains('%') {
                    let pct = cls.trim_start_matches("w-[").trim_end_matches(']').to_string();
                    let mut config: HashMap<String, String> = HashMap::new();
                    config.insert("_type".into(), "meter".into());
                    config.insert("progress".into(), pct.clone());
                    items.push(ItemBlueprint {
                        item_type: "meter".into(),
                        title: String::new(),
                        description: Some(pct),
                        config,
                    });
                    return;
                }
            }
        }
    }

    // Recurse into children
    for child in &node.children {
        find_div_progress_bars(child, items);
    }
}

/// Extract width percentage from an inline style string (e.g. "width: 75%").
fn extract_width_percent(style: &str) -> Option<String> {
    let lower = style.to_lowercase();
    if let Some(pos) = lower.find("width") {
        let after = &style[pos..];
        if let Some(colon) = after.find(':') {
            let val_part = after[colon + 1..].trim();
            if val_part.contains('%') {
                let pct: String = val_part.chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '%')
                    .collect();
                if !pct.is_empty() {
                    return Some(pct);
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Status indicator extraction: colored dots + text
// ---------------------------------------------------------------------------

/// Extract status indicators: colored dots (rounded-full with bg-green/red/yellow) + text.
fn extract_status_indicator_items(node: &DomNode) -> Vec<ItemBlueprint> {
    let mut items = Vec::new();
    find_status_indicators(node, &mut items);
    items
}

fn find_status_indicators(node: &DomNode, items: &mut Vec<ItemBlueprint>) {
    // Check if any child is a status dot
    let dot_colors = [
        ("bg-green-", "operational"),
        ("bg-red-", "error"),
        ("bg-yellow-", "warning"),
        ("bg-orange-", "warning"),
        ("bg-blue-", "info"),
    ];

    for child in &node.children {
        if !dom::has_class(child, "rounded-full") {
            continue;
        }
        // Tiny dot: w-2/w-3 h-2/h-3 with color bg
        let is_dot = (dom::has_class(child, "w-2") || dom::has_class(child, "w-3")
            || dom::has_class(child, "h-2") || dom::has_class(child, "h-3"))
            && child.full_text.trim().is_empty();

        if !is_dot {
            continue;
        }

        // Determine the color/status
        let mut status_type = "unknown";
        for (cls, st) in &dot_colors {
            if dom::has_class(child, cls) {
                status_type = st;
                break;
            }
        }

        // Find the text sibling next to the dot
        let mut found_dot = false;
        for sibling in &node.children {
            if std::ptr::eq(sibling, child) {
                found_dot = true;
                continue;
            }
            if found_dot {
                let txt = dom::clean_node_text(sibling);
                if !txt.is_empty() && txt.len() < 100 {
                    let mut config: HashMap<String, String> = HashMap::new();
                    config.insert("_type".into(), "status".into());
                    config.insert("status".into(), status_type.to_string());
                    items.push(ItemBlueprint {
                        item_type: "status".into(),
                        title: txt,
                        description: None,
                        config,
                    });
                    return;
                }
            }
        }

        // Fallback: get text from parent excluding the dot
        let parent_text = dom::clean_node_text(node);
        if !parent_text.is_empty() && parent_text.len() < 100 {
            let mut config: HashMap<String, String> = HashMap::new();
            config.insert("_type".into(), "status".into());
            config.insert("status".into(), status_type.to_string());
            items.push(ItemBlueprint {
                item_type: "status".into(),
                title: parent_text,
                description: None,
                config,
            });
            return;
        }
    }

    // Recurse, but only if we haven't found anything yet at this level
    if items.is_empty() {
        for child in &node.children {
            find_status_indicators(child, items);
            if !items.is_empty() {
                return;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Badge/pill extraction
// ---------------------------------------------------------------------------

/// Extract badge/pill elements: rounded-full + small text + often uppercase.
fn extract_badge_items(node: &DomNode) -> Vec<ItemBlueprint> {
    let mut items = Vec::new();

    let badge_nodes = dom::find_by_class(node, "rounded-full");
    for badge in &badge_nodes {
        // Skip buttons, links, avatars, traffic dots, material icons
        if badge.tag == "button" || badge.tag == "a" || badge.tag == "img" {
            continue;
        }
        if is_material_icon_span(badge) || is_traffic_light_dot(badge) || is_badge_dot(badge) {
            continue;
        }
        // Must have small text
        let is_small = dom::has_class(badge, "text-xs") || dom::has_class(badge, "text-sm")
            || badge.classes.iter().any(|c| c.contains("text-["));
        if !is_small {
            continue;
        }
        let txt = extract_badge_text(badge);
        if txt.is_empty() || txt.len() > 60 {
            continue;
        }
        // Skip if it looks like a nav element
        if badge.full_text.len() > 40 {
            continue;
        }

        let mut config: HashMap<String, String> = HashMap::new();
        config.insert("_type".into(), "badge".into());
        if dom::has_class(badge, "uppercase") {
            config.insert("style".into(), "uppercase".into());
        }

        items.push(ItemBlueprint {
            item_type: "badge".into(),
            title: txt,
            description: None,
            config,
        });
    }

    items
}

// ---------------------------------------------------------------------------
// Code/mono text extraction
// ---------------------------------------------------------------------------

/// Extract code/mono text blocks: elements with font-mono class or <code>/<pre> tags.
fn extract_code_items(node: &DomNode) -> Vec<ItemBlueprint> {
    let mut items = Vec::new();

    // <pre> and <code> blocks
    let pre_nodes = dom::find_by_tag(node, "pre");
    for pre in &pre_nodes {
        let txt = pre.full_text.trim().to_string();
        if !txt.is_empty() {
            let mut config: HashMap<String, String> = HashMap::new();
            config.insert("_type".into(), "code".into());
            items.push(ItemBlueprint {
                item_type: "code".into(),
                title: txt,
                description: None,
                config,
            });
        }
    }

    // Standalone <code> elements (not inside <pre>)
    let code_nodes = dom::find_by_tag(node, "code");
    for code in &code_nodes {
        // Skip if parent is <pre> (already captured)
        let is_in_pre = pre_nodes.iter().any(|pre| {
            dom::find_by_tag(pre, "code").iter().any(|c| std::ptr::eq(*c, *code))
        });
        if is_in_pre {
            continue;
        }
        let txt = code.full_text.trim().to_string();
        if !txt.is_empty() && txt.len() < 500 {
            let mut config: HashMap<String, String> = HashMap::new();
            config.insert("_type".into(), "code".into());
            config.insert("style".into(), "inline".into());
            items.push(ItemBlueprint {
                item_type: "code".into(),
                title: txt,
                description: None,
                config,
            });
        }
    }

    // font-mono blocks that are NOT inside code/pre and NOT buttons or icons
    let mono_nodes = dom::find_by_class(node, "font-mono");
    for mono in &mono_nodes {
        if mono.tag == "code" || mono.tag == "pre" || mono.tag == "button" {
            continue;
        }
        if is_material_icon_span(mono) {
            continue;
        }
        // Skip if already captured as code/pre content
        let txt = mono.full_text.trim().to_string();
        if txt.is_empty() || txt.len() > 500 {
            continue;
        }
        // Check if this text is already in items
        let already_exists = items.iter().any(|i| i.title == txt);
        if already_exists {
            continue;
        }

        let mut config: HashMap<String, String> = HashMap::new();
        config.insert("_type".into(), "code".into());
        config.insert("style".into(), "mono".into());
        items.push(ItemBlueprint {
            item_type: "code".into(),
            title: txt,
            description: None,
            config,
        });
    }

    items
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Common Material Symbols icon names that may appear as text content
/// inside `<span class="material-symbols-outlined">`.
const MATERIAL_ICON_NAMES: &[&str] = &[
    "arrow_forward", "arrow_back", "arrow_downward", "arrow_upward",
    "chevron_right", "chevron_left", "close", "menu", "search",
    "check", "check_circle", "add", "remove", "delete", "edit",
    "star", "favorite", "settings", "home", "person", "mail",
    "phone", "link", "open_in_new", "download", "upload",
    "play_arrow", "pause", "stop", "skip_next", "skip_previous",
    "expand_more", "expand_less", "more_vert", "more_horiz",
    "visibility", "visibility_off", "lock", "lock_open",
    "notifications", "info", "warning", "error", "help",
    "schedule", "calendar_today", "event", "place", "map",
    "shopping_cart", "payments", "credit_card", "receipt",
    "code", "terminal", "data_object", "deployed_code",
    "cloud", "cloud_upload", "cloud_download", "storage",
    "rocket_launch", "speed", "bolt", "auto_awesome",
    "dark_mode", "light_mode", "contrast", "palette",
    "folder", "description", "article", "draft",
    "group", "groups", "diversity_3", "share",
    "thumb_up", "thumb_down", "mood", "sentiment_satisfied",
    "trending_up", "trending_down", "analytics", "insights",
    "verified", "new_releases", "campaign", "flag",
    "security", "shield", "vpn_key", "key",
    "language", "translate", "public", "dns",
    "developer_board", "memory", "device_hub", "hub",
    "network_check", "wifi", "signal_cellular_alt", "cell_tower",
];

/// Clean button text by stripping material icon names from spans.
/// Extracts text from a DomNode but skips children that are
/// `<span class="material-symbols-outlined">`.
fn clean_button_text(node: &DomNode) -> String {
    let mut parts: Vec<String> = Vec::new();

    // Add this node's direct text (not from children)
    let direct = node.text.trim().to_string();
    if !direct.is_empty() && !is_material_icon_text(&direct) {
        parts.push(direct);
    }

    // Recurse into children, skipping material icon spans
    for child in &node.children {
        if is_material_icon_span(child) {
            continue;
        }
        let child_text = clean_button_text(child);
        if !child_text.is_empty() {
            parts.push(child_text);
        }
    }

    let result = parts.join(" ");
    // Final safety pass: strip any remaining icon names that slipped through
    let mut cleaned = result.clone();
    for icon in MATERIAL_ICON_NAMES {
        cleaned = cleaned.replace(icon, "");
    }
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Check if a node is a `<span class="material-symbols-outlined">`.
fn is_material_icon_span(node: &DomNode) -> bool {
    node.tag == "span"
        && node.classes.iter().any(|c| {
            c.contains("material-symbols-outlined")
                || c.contains("material-icons")
                || c.contains("material-symbols-rounded")
                || c.contains("material-symbols-sharp")
        })
}

/// Check if text is a known material icon name.
fn is_material_icon_text(text: &str) -> bool {
    let trimmed = text.trim();
    // Icon names are lowercase with underscores, no spaces
    if trimmed.is_empty() {
        return false;
    }
    MATERIAL_ICON_NAMES.contains(&trimmed)
        || (trimmed.chars().all(|c| c.is_ascii_lowercase() || c == '_')
            && trimmed.contains('_'))
}

/// Extract clean button texts from a node, stripping material icon text.
fn extract_clean_buttons(node: &DomNode) -> Vec<String> {
    let mut buttons = Vec::new();
    if node.tag == "button"
        || (node.tag == "a"
            && node
                .classes
                .iter()
                .any(|c| c.contains("btn") || c.contains("rounded-full")))
    {
        let text = clean_button_text(node);
        if !text.is_empty() {
            buttons.push(text);
        }
    }
    for child in &node.children {
        buttons.extend(extract_clean_buttons(child));
    }
    buttons
}

/// Find a heading by specific tag (e.g. "h1", "h2"), cleaned of icon text.
fn find_heading_by_tag(node: &DomNode, tag: &str) -> Option<String> {
    let matches = dom::find_by_tag(node, tag);
    for m in matches {
        let text = dom::clean_node_text(m);
        if !text.is_empty() {
            return Some(text);
        }
    }
    None
}

/// Extract the material icon name from a node's children.
/// Checks full_text first, then data-icon attribute, then direct text.
fn extract_material_icon(node: &DomNode) -> Option<String> {
    for child in &node.children {
        if is_material_icon_span(child) {
            // Try full_text first (icon name as text content)
            let icon = child.full_text.trim().to_string();
            if !icon.is_empty() {
                return Some(icon);
            }
            // Fallback: data-icon attribute
            if let Some(data_icon) = child.attrs.get("data-icon") {
                let di = data_icon.trim().to_string();
                if !di.is_empty() {
                    return Some(di);
                }
            }
            // Fallback: direct text
            let direct = child.text.trim().to_string();
            if !direct.is_empty() {
                return Some(direct);
            }
        }
        if let Some(icon) = extract_material_icon(child) {
            return Some(icon);
        }
    }
    None
}

/// Map Tailwind text color classes to color names for terminal output.
fn detect_text_color(node: &DomNode) -> Option<String> {
    let color_map: &[(&str, &str)] = &[
        ("text-tertiary-container", "blue"),
        ("text-green-400", "green"),
        ("text-green-500", "green"),
        ("text-red-400", "red"),
        ("text-red-500", "red"),
        ("text-yellow-400", "yellow"),
        ("text-yellow-500", "yellow"),
        ("text-blue-400", "blue"),
        ("text-blue-500", "blue"),
        ("text-purple-400", "purple"),
        ("text-purple-500", "purple"),
        ("text-orange-400", "orange"),
        ("text-gray-400", "dim"),
        ("text-gray-500", "dim"),
        ("text-muted", "dim"),
        ("text-primary", "primary"),
        ("text-on-primary", "primary"),
    ];

    for (cls, color) in color_map {
        if dom::has_class(node, cls) {
            return Some(color.to_string());
        }
    }
    // Check children too
    for child in &node.children {
        for (cls, color) in color_map {
            if dom::has_class(child, cls) {
                return Some(color.to_string());
            }
        }
    }
    None
}
