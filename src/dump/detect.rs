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
    pub template: Option<String>,
    pub style_block: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ItemBlueprint {
    pub item_type: String,
    pub title: String,
    pub description: Option<String>,
    pub config: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// HTML template serialization
// ---------------------------------------------------------------------------

/// Serialize a DomNode back to an HTML string.
fn dom_to_html(node: &DomNode) -> String {
    let mut html = String::new();

    // Opening tag
    html.push('<');
    html.push_str(&node.tag);

    // Attributes (skip "class" — we emit classes separately)
    for (key, value) in &node.attrs {
        if key == "class" { continue; }
        html.push_str(&format!(" {}=\"{}\"", key, value.replace('"', "&quot;")));
    }

    // Classes
    if !node.classes.is_empty() {
        html.push_str(&format!(" class=\"{}\"", node.classes.join(" ")));
    }

    html.push('>');

    // Children
    for child in &node.children {
        if child.tag == "_text" {
            html.push_str(&child.text);
        } else {
            html.push_str(&dom_to_html(child));
        }
    }

    // Self-closing tags don't get a closing tag
    let void_elements = ["img", "br", "hr", "input", "meta", "link"];
    if !void_elements.contains(&node.tag.as_str()) {
        html.push_str(&format!("</{}>", node.tag));
    }

    html
}

/// Replace known content values in the HTML template with {{placeholder}} tokens
/// so the semantic layer can override them at render time.
fn templatize(html: &str, section: &SectionBlueprint) -> String {
    let mut tmpl = html.to_string();

    // Replace title text with {{title}}
    if let Some(ref title) = section.title {
        if !title.is_empty() {
            tmpl = tmpl.replace(title, "{{title}}");
        }
    }

    // Replace subtitle with {{subtitle}}
    if let Some(ref subtitle) = section.subtitle {
        if !subtitle.is_empty() {
            tmpl = tmpl.replace(subtitle, "{{subtitle}}");
        }
    }

    // Replace CTA text with {{cta.text}} and href with {{cta.href}}
    if let Some(cta_text) = section.config.get("cta_text") {
        if !cta_text.is_empty() {
            tmpl = tmpl.replace(cta_text, "{{cta.text}}");
        }
    }
    if let Some(cta_link) = section.config.get("cta_link") {
        if !cta_link.is_empty() && cta_link != "#" {
            tmpl = tmpl.replace(cta_link, "{{cta.href}}");
        }
    }

    tmpl
}

/// Extract all `<style>` tag contents from raw HTML.
pub fn extract_page_styles(html: &str) -> String {
    let mut styles = String::new();
    let mut pos = 0;
    while let Some(start) = html[pos..].find("<style") {
        let abs_start = pos + start;
        if let Some(tag_end) = html[abs_start..].find('>') {
            let content_start = abs_start + tag_end + 1;
            if let Some(end) = html[content_start..].find("</style>") {
                styles.push_str(&html[content_start..content_start + end]);
                styles.push('\n');
                pos = content_start + end;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    styles
}

// ---------------------------------------------------------------------------
// Main detection entry point
// ---------------------------------------------------------------------------

/// Walk every top-level DOM block, classify it, and extract content.
///
/// When `page_css` is provided (extracted from `<style>` tags), it is attached
/// as `style_block` on the first emitted section.
pub fn detect_sections_with_templates(
    nodes: &[DomNode],
    page_css: Option<&str>,
) -> Vec<SectionBlueprint> {
    let mut sections: Vec<SectionBlueprint> = Vec::new();

    for node in nodes {
        let (section_type, confidence) = patterns::classify_node(node);

        if confidence <= 0.3 {
            // Below threshold -- try splitting large blocks into sub-blocks
            if should_split_block(node) {
                let sub_sections = split_and_detect(node);
                if !sub_sections.is_empty() {
                    // Attach templates to sub-sections
                    for mut s in sub_sections {
                        attach_template(&mut s, node);
                        sections.push(s);
                    }
                    continue;
                }
            }
            // Still emit a generic section so no text is lost
            let mut generic = extract_generic(node);
            if generic.title.is_some() || !generic.items.is_empty() {
                attach_template(&mut generic, node);
                sections.push(generic);
            }
            continue;
        }

        // Deep-split: container elements (section, main, div) whose direct
        // children individually classify as 2+ distinct semantic types should
        // be exploded into separate sections instead of treated as one blob.
        if should_deep_split(node, section_type, confidence) {
            let sub_sections = deep_split_and_detect(node);
            if sub_sections.len() >= 2 {
                for s in sub_sections {
                    // Templates are already attached by deep_split_and_detect
                    sections.push(s);
                }
                continue;
            }
        }

        let blueprint = match section_type {
            "order-header" => extract_order_header(node),
            "line-items" => extract_line_items_section(node),
            "price-breakdown" => extract_price_breakdown(node),
            "shipping-timeline" => extract_shipping_timeline(node),
            "customer-profile" => extract_customer_profile(node),
            "payment-info" => extract_payment_info(node),
            "staff-notes" => extract_staff_notes(node),
            "settings-profile" => extract_settings_profile(node),
            "api-keys" => extract_api_keys(node),
            "subscription-card" => extract_subscription_card(node),
            "danger-zone" => extract_danger_zone(node),
            "topbar" => extract_topbar(node),
            "hero" => extract_hero(node),
            "features" | "faq" => extract_features(node),
            "testimonial" => extract_testimonials(node),
            "stats" => extract_stats(node),
            "pricing" => extract_pricing(node),
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
            "form" => extract_form(node),
            "tabs" => extract_tabs(node),
            "kpi-grid" => extract_kpi_grid(node),
            "chart" => extract_chart(node),
            "data-table" => extract_data_table(node),
            _ => extract_generic(node),
        };

        // Override confidence from the pattern engine
        let mut blueprint = blueprint;
        blueprint.confidence = confidence;
        blueprint.section_type = section_type.to_string();

        // Attach HTML template from the source DomNode
        attach_template(&mut blueprint, node);

        sections.push(blueprint);
    }

    // Attach page-level CSS to the first section
    if let Some(css) = page_css {
        if !css.trim().is_empty() {
            if let Some(first) = sections.first_mut() {
                first.style_block = Some(css.to_string());
            }
        }
    }

    // Merge consecutive page-header fragments into a single section
    merge_page_headers(&mut sections);

    // Only sort for dashboard pages (no templates).
    // When templates are present, preserve original DOM order.
    let has_any_template = sections.iter().any(|s| s.template.is_some());
    if !has_any_template {
        sort_dashboard_sections(&mut sections);
    }

    sections
}

/// Backward-compatible entry point (without template extraction).
pub fn detect_sections(nodes: &[DomNode]) -> Vec<SectionBlueprint> {
    detect_sections_with_templates(nodes, None)
}

/// Serialize a DomNode to HTML, templatize it, and store on the blueprint.
fn attach_template(blueprint: &mut SectionBlueprint, node: &DomNode) {
    let raw_html = dom_to_html(node);
    let templated = templatize(&raw_html, blueprint);
    blueprint.template = Some(templated);
}

/// Post-process all templates to auto-trigger scroll animations.
/// Replaces `animate-on-scroll` in class attributes (not in CSS) with
/// `animate-on-scroll animate` so elements are visible without JS observer.
pub fn fix_animation_visibility(sections: &mut [SectionBlueprint]) {
    for section in sections.iter_mut() {
        if let Some(ref mut tmpl) = section.template {
            // Only replace inside class="..." attributes, not in CSS selectors
            // Strategy: find each `class="...animate-on-scroll..."` and inject `animate`
            let marker = "class=\"";
            let mut result = String::with_capacity(tmpl.len() + 256);
            let mut search_from = 0;
            while let Some(found) = tmpl[search_from..].find(marker) {
                let abs = search_from + found;
                // Copy everything up to and including class="
                result.push_str(&tmpl[search_from..abs + marker.len()]);
                let class_start = abs + marker.len();
                // Find closing quote
                if let Some(end) = tmpl[class_start..].find('"') {
                    let class_val = &tmpl[class_start..class_start + end];
                    if class_val.contains("animate-on-scroll") && !class_val.contains("animate-on-scroll animate") {
                        result.push_str(&class_val.replace("animate-on-scroll", "animate-on-scroll animate"));
                    } else {
                        result.push_str(class_val);
                    }
                    result.push('"');
                    search_from = class_start + end + 1;
                } else {
                    search_from = class_start;
                }
            }
            // Append remainder
            result.push_str(&tmpl[search_from..]);
            *tmpl = result;
        }
    }
}

/// Check if a block is large enough to warrant splitting into sub-blocks.
/// A block with many children and deep nesting is a candidate.
fn should_split_block(node: &DomNode) -> bool {
    // Don't split grid containers — they are cohesive layout blocks
    let classes = node.attrs.get("class").map(|s| s.as_str()).unwrap_or("");
    if classes.contains("grid-cols-") || classes.contains("grid ") {
        return false;
    }
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
                "features" | "faq" => extract_features(child),
                "testimonial" => extract_testimonials(child),
                "stats" => extract_stats(child),
                "pricing" => extract_pricing(child),
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
                "kpi-grid" => extract_kpi_grid(child),
                "chart" => extract_chart(child),
                "data-table" => extract_data_table(child),
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

/// Check if a node that already classified as something (confidence > 0.3)
/// should actually be split into its children because they individually
/// classify as 2+ distinct semantic types.
///
/// This catches the case where a `<section>` wraps a page-header, KPI grid,
/// chart, and table — the mixed signals make the parent classify as "features"
/// but each child is a distinct dashboard section.
fn should_deep_split(node: &DomNode, section_type: &str, confidence: f32) -> bool {
    // Only consider container-like elements
    let is_container = matches!(node.tag.as_str(), "section" | "main" | "div" | "article");
    if !is_container {
        return false;
    }

    // Don't split elements that are already specific dashboard types
    // (sidebar, topbar, footer are self-contained)
    if matches!(section_type, "sidebar" | "topbar" | "footer" | "hero") {
        return false;
    }

    // Don't split grid layout containers — their children are meant to be
    // rendered together in the grid.  The template will preserve the layout.
    let classes = node.attrs.get("class").map(|s| s.as_str()).unwrap_or("");
    if classes.contains("grid-cols-") || classes.contains("grid ") {
        return false;
    }

    // Don't split nodes that already classify with high confidence as a
    // known composite type (kpi-grid, stat-cards, data-table, chart, pricing).
    // These are self-contained visual blocks.
    if confidence >= 0.6 && matches!(section_type,
        "kpi-grid" | "stat-cards" | "stats" | "data-table" | "chart" | "pricing" | "form" | "tabs"
    ) {
        return false;
    }

    // Need at least 2 non-trivial direct children
    let meaningful_children: Vec<&DomNode> = node.children.iter()
        .filter(|c| c.tag != "_text" && !c.full_text.trim().is_empty())
        .collect();
    if meaningful_children.len() < 2 {
        return false;
    }

    // Check how many children individually classify as distinct types
    let mut child_types = std::collections::HashSet::new();
    let mut classified_count = 0;
    for child in &meaningful_children {
        let (child_type, child_conf) = patterns::classify_node(child);
        if child_conf > 0.3 && child_type != "generic" {
            child_types.insert(child_type);
            classified_count += 1;
        }
    }

    // Split if we found 2+ children that classify as distinct types
    // (e.g. page-header + kpi-grid + chart + data-table)
    child_types.len() >= 2 || classified_count >= 2
}

/// Deep-split a container node: classify each direct child individually
/// and produce separate SectionBlueprints with per-child templates.
fn deep_split_and_detect(node: &DomNode) -> Vec<SectionBlueprint> {
    let mut sections = Vec::new();

    for child in &node.children {
        // Skip empty/trivial children (text nodes, whitespace)
        if child.tag == "_text" || (child.full_text.trim().is_empty() && child.children.is_empty()) {
            continue;
        }

        let (child_type, child_confidence) = patterns::classify_node(child);

        if child_confidence > 0.3 {
            let blueprint = match child_type {
                "topbar" => extract_topbar(child),
                "hero" => extract_hero(child),
                "features" | "faq" => extract_features(child),
                "testimonial" => extract_testimonials(child),
                "stats" => extract_stats(child),
                "pricing" => extract_pricing(child),
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
                "form" => extract_form(child),
                "tabs" => extract_tabs(child),
                "kpi-grid" => extract_kpi_grid(child),
                "chart" => extract_chart(child),
                "data-table" => extract_data_table(child),
                _ => extract_generic(child),
            };
            let mut blueprint = blueprint;
            blueprint.confidence = child_confidence;
            blueprint.section_type = child_type.to_string();
            attach_template(&mut blueprint, child);
            sections.push(blueprint);
        } else {
            // Child doesn't classify well — still emit if it has content
            let mut generic = extract_generic(child);
            if generic.title.is_some() || !generic.items.is_empty() {
                attach_template(&mut generic, child);
                sections.push(generic);
            }
        }
    }

    sections
}

/// Merge consecutive page-header sections into one.
///
/// When `extract_deep_blocks` splits a page-header div into its children
/// (badge div, h2, p), each piece may be independently classified as
/// "page-header". This pass collapses consecutive runs into a single section,
/// combining title, subtitle, badge, and items.
fn merge_page_headers(sections: &mut Vec<SectionBlueprint>) {
    if sections.len() < 2 {
        return;
    }

    let mut merged: Vec<SectionBlueprint> = Vec::new();
    let mut i = 0;

    while i < sections.len() {
        if sections[i].section_type != "page-header" {
            merged.push(sections[i].clone());
            i += 1;
            continue;
        }

        // Start a run of consecutive page-header sections
        let mut combined = sections[i].clone();
        let mut j = i + 1;

        while j < sections.len() && sections[j].section_type == "page-header" {
            let next = &sections[j];

            // Merge title: keep the first non-empty title
            if combined.title.is_none() {
                if let Some(ref t) = next.title {
                    combined.title = Some(t.clone());
                }
            }
            // Merge subtitle: keep the first non-empty subtitle
            if combined.subtitle.is_none() {
                if let Some(ref s) = next.subtitle {
                    combined.subtitle = Some(s.clone());
                }
            }
            // Merge config (badge, badge_dot, etc.) — don't overwrite existing keys
            for (k, v) in &next.config {
                combined.config.entry(k.clone()).or_insert_with(|| v.clone());
            }
            // Merge items (action buttons, etc.)
            combined.items.extend(next.items.clone());
            // Keep the highest confidence
            if next.confidence > combined.confidence {
                combined.confidence = next.confidence;
            }

            j += 1;
        }

        merged.push(combined);
        i = j;
    }

    *sections = merged;
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
    let mut items: Vec<ItemBlueprint> = Vec::new();

    // Brand: first bold/large text child, or first text in the nav
    let brand = find_brand_text(node).unwrap_or_default();
    if !brand.is_empty() {
        config.insert("brand".into(), brand);
    }

    // ---------------------------------------------------------------
    // Search input: extract placeholder text from <input> elements
    // ---------------------------------------------------------------
    let inputs = dom::find_by_tag(node, "input");
    for input in &inputs {
        if let Some(placeholder) = input.attrs.get("placeholder") {
            let ph = placeholder.trim().to_string();
            if !ph.is_empty() {
                let mut item_config: HashMap<String, String> = HashMap::new();
                item_config.insert("icon".into(), "search".into());
                items.push(ItemBlueprint {
                    item_type: "search".into(),
                    title: ph,
                    description: None,
                    config: item_config,
                });
            }
        }
    }

    // ---------------------------------------------------------------
    // Button icons: extract material icons from <button> elements
    // ---------------------------------------------------------------
    let btn_nodes = dom::find_by_tag(node, "button");
    for btn in &btn_nodes {
        if let Some(icon) = extract_material_icon(btn) {
            let label = capitalize_icon_name(&icon);
            let mut item_config: HashMap<String, String> = HashMap::new();
            item_config.insert("icon".into(), icon);
            items.push(ItemBlueprint {
                item_type: "action".into(),
                title: label,
                description: None,
                config: item_config,
            });
        } else {
            // Button with text but no icon (CTA)
            let text = dom::clean_node_text(btn);
            if !text.is_empty() {
                config.insert("cta_text".into(), text);
            }
        }
    }

    // ---------------------------------------------------------------
    // Avatar image: <img> with alt containing "profile" or "avatar"
    // ---------------------------------------------------------------
    let imgs = dom::find_by_tag(node, "img");
    for img in &imgs {
        let alt = img.attrs.get("alt").cloned().unwrap_or_default();
        let src = img.attrs.get("src").cloned().unwrap_or_default();
        if !alt.is_empty() || !src.is_empty() {
            let mut item_config: HashMap<String, String> = HashMap::new();
            if !src.is_empty() {
                item_config.insert("src".into(), src);
            }
            items.push(ItemBlueprint {
                item_type: "image".into(),
                title: if !alt.is_empty() { alt } else { "Avatar".into() },
                description: None,
                config: item_config,
            });
        }
    }

    // ---------------------------------------------------------------
    // Nav links (fallback for topbars with traditional link navigation)
    // ---------------------------------------------------------------
    if items.is_empty() {
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
    }

    SectionBlueprint {
        section_type: "topbar".into(),
        confidence: 0.0,
        title: None,
        subtitle: None,
        config,
        items,
        template: None,
        style_block: None,
    }
}

/// Capitalize an icon name for display: "notifications" -> "Notifications".
fn capitalize_icon_name(icon: &str) -> String {
    let cleaned = icon.replace('_', " ");
    let mut chars = cleaned.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let upper: String = first.to_uppercase().collect();
            upper + chars.as_str()
        }
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
        template: None,
        style_block: None,
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
    // First try the classic "badge"/"pill"/"tag" class
    let badge_nodes = dom::find_by_class(node, "badge");
    let pill_nodes = dom::find_by_class(node, "pill");
    let tag_nodes = dom::find_by_class(node, "tag");
    for n in badge_nodes.iter().chain(pill_nodes.iter()).chain(tag_nodes.iter()) {
        // Skip nodes that are too large (likely not a badge)
        let txt = n.full_text.trim().to_string();
        if !txt.is_empty() && txt.len() < 60 {
            return Some(txt);
        }
    }

    // Check for short <span> elements that appear before h1 (common badge pattern)
    let mut found_h1 = false;
    for child in &node.children {
        if child.tag == "h1" { found_h1 = true; }
        // Look one level deeper for span-before-h1 pattern
        for grandchild in &child.children {
            if grandchild.tag == "h1" { found_h1 = true; }
            if !found_h1 && grandchild.tag == "span" {
                let txt = grandchild.full_text.trim().to_string();
                if !txt.is_empty() && txt.len() < 60 && txt.len() > 3 {
                    // Check if it looks like a badge (has badge/tag/pill class or is short inline text)
                    let has_badge_class = grandchild.classes.iter().any(|c| {
                        c.contains("badge") || c.contains("tag") || c.contains("pill")
                            || c.contains("label") || c.contains("chip")
                    });
                    if has_badge_class {
                        return Some(txt);
                    }
                }
            }
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
        let mut chip_title = text.clone();
        if let Some(icon) = extract_chip_icon(card) {
            // Remove icon text from the chip title to avoid duplication
            // e.g. "NEXT Ready for Next.js" -> "Ready for Next.js" with icon "NEXT"
            chip_title = chip_title
                .replace(&icon, "")
                .trim()
                .to_string();
            if chip_title.is_empty() {
                chip_title = text;
            }
            chip_config.insert("icon".into(), icon);
        }

        // Deduplicate text if clean_button_text doubled it from nested elements
        let words: Vec<&str> = chip_title.split_whitespace().collect();
        if words.len() >= 4 && words.len() % 2 == 0 {
            let half = words.len() / 2;
            if words[..half] == words[half..] {
                chip_title = words[..half].join(" ");
            }
        }

        items.push(ItemBlueprint {
            item_type: "chip".into(),
            title: chip_title,
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
                // Skip if this chip title already exists (avoid duplicates from hero-level extraction)
                let already_exists = items.iter().any(|i| i.item_type == "chip" && i.title == chip_text);
                if !chip_text.is_empty() && chip_text.len() < 60 && !already_exists {
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
        template: None,
        style_block: None,
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
    let subtitle = find_subtitle_paragraph(node, &title);

    // Try BEM-style stat cards first (stat-card__value + stat-card__label)
    // find_by_class matches substrings, so "stat-card" also matches "stat-card__value".
    // Filter to only the BEM block element (exact class "stat-card", not "stat-card__*").
    let all_stat_nodes = dom::find_by_class(node, "stat-card");
    let stat_cards: Vec<&DomNode> = all_stat_nodes.into_iter()
        .filter(|n| n.classes.iter().any(|c| c == "stat-card"))
        .collect();
    if !stat_cards.is_empty() {
        for card in &stat_cards {
            // Find __value child (not the card itself)
            let value = dom::find_by_class(card, "__value")
                .into_iter()
                .filter(|n| !std::ptr::eq(*n, *card))
                .next()
                .map(|v| clean_stat_value(&dom::clean_node_text(v)))
                .unwrap_or_default();
            let label = dom::find_by_class(card, "__label")
                .into_iter()
                .filter(|n| !std::ptr::eq(*n, *card))
                .next()
                .map(|l| dom::clean_node_text(l))
                .unwrap_or_default();

            if !value.is_empty() || !label.is_empty() {
                let mut config = HashMap::new();
                if !value.is_empty() {
                    config.insert("value".into(), value);
                }
                items.push(ItemBlueprint {
                    item_type: "stat".into(),
                    title: label,
                    description: None,
                    config,
                });
            }
        }
    }

    // Fallback: Tailwind utility class approach
    if items.is_empty() {
        let cards = find_card_children(node);
        for card in &cards {
            let large_text = find_large_text(card);
            let label = find_small_label(card);

            if large_text.is_some() || label.is_some() {
                let mut config = HashMap::new();
                if let Some(ref val) = large_text {
                    config.insert("value".into(), val.clone());
                }
                items.push(ItemBlueprint {
                    item_type: "stat".into(),
                    title: label.unwrap_or_default(),
                    description: None,
                    config,
                });
            }
        }
    }

    // If no cards found, try scanning all children for number-like text
    if items.is_empty() {
        for child in &node.children {
            let text = child.full_text.trim().to_string();
            if looks_like_stat(&text) {
                items.push(ItemBlueprint {
                    item_type: "stat".into(),
                    title: text,
                    description: None,
                    config: HashMap::new(),
                });
            }
        }
    }

    // Determine cols from item count
    let mut section_config = HashMap::new();
    if items.len() >= 3 {
        section_config.insert("cols".into(), items.len().to_string());
    }

    SectionBlueprint {
        section_type: "stats".into(),
        confidence: 0.0,
        title,
        subtitle,
        config: section_config,
        items,
        template: None,
        style_block: None,
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

/// Find subtitle paragraph that is NOT the section title.
fn find_subtitle_paragraph(node: &DomNode, title: &Option<String>) -> Option<String> {
    let paragraphs = dom::find_by_tag(node, "p");
    for p in &paragraphs {
        let text = dom::clean_node_text(p);
        if !text.is_empty() && text.len() < 200 {
            // Skip if same as title
            if let Some(ref t) = title {
                if text == *t {
                    continue;
                }
            }
            return Some(text);
        }
    }
    None
}

/// Clean a stat value by collapsing spaces around suffixes/prefixes.
/// E.g. "+98 %" -> "+98%", "500 +" -> "500+", "+2.5 x" -> "+2.5x"
fn clean_stat_value(text: &str) -> String {
    let mut result = text.trim().to_string();
    // Remove spaces before common stat suffixes
    for suffix in &[" %", " +", " x", " X", " K", " M", " k", " m"] {
        if result.ends_with(suffix) {
            let new_len = result.len() - suffix.len();
            let suffix_char = &suffix[1..];
            result = format!("{}{}", &result[..new_len], suffix_char);
        }
    }
    // Remove spaces after common stat prefixes
    for prefix in &["+ ", "- "] {
        if result.starts_with(prefix) {
            let prefix_char = &prefix[..1];
            result = format!("{}{}", prefix_char, &result[prefix.len()..]);
        }
    }
    result
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
// Extraction: testimonials
// ---------------------------------------------------------------------------

fn extract_testimonials(node: &DomNode) -> SectionBlueprint {
    let config: HashMap<String, String> = HashMap::new();

    let title = find_heading_by_tag(node, "h2")
        .or_else(|| dom::find_heading(node));
    let subtitle = find_subtitle_paragraph(node, &title);

    // Collect testimonial cards: look for children (or grandchildren) that
    // contain a quote paragraph and an author name.
    let mut items: Vec<ItemBlueprint> = Vec::new();

    fn is_heading_container(node: &DomNode) -> bool {
        node.classes.iter().any(|c| c.contains("header") && !c.contains("card"))
    }

    fn collect_testimonial_cards(parent: &DomNode, out: &mut Vec<ItemBlueprint>, skip_headers: bool) {
        for child in &parent.children {
            // Skip section header containers (contain title/subtitle, not testimonials)
            if skip_headers && is_heading_container(child) {
                continue;
            }

            // Check if this child IS a testimonial card (exact BEM block match, not a grid wrapper)
            let is_card = child.classes.iter().any(|c|
                c == "testimonial-card" || c == "review-card" || c == "quote-card"
                || c == "testimonial" || c == "review" || c == "quote"
            );

            if is_card {
                // Extract quote from direct or nested <p> tags
                let paragraphs = dom::find_by_tag(child, "p");
                let quote = paragraphs.iter()
                    .map(|p| dom::clean_node_text(p))
                    .filter(|t| t.len() > 30)
                    .next();

                if let Some(quote_text) = quote {
                    // Try to find an author name (short text in a nested div/span)
                    let mut author = String::new();
                    let mut role = String::new();
                    let name_nodes = dom::find_by_class(child, "name")
                        .into_iter()
                        .chain(dom::find_by_class(child, "author").into_iter());
                    for n in name_nodes {
                        let txt = dom::clean_node_text(n);
                        if !txt.is_empty() && txt.len() < 40 {
                            author = txt;
                            break;
                        }
                    }
                    // Try to find role
                    let role_nodes = dom::find_by_class(child, "role")
                        .into_iter()
                        .chain(dom::find_by_class(child, "position").into_iter());
                    for n in role_nodes {
                        let txt = dom::clean_node_text(n);
                        if !txt.is_empty() && txt.len() < 60 && txt != author {
                            role = txt;
                            break;
                        }
                    }

                    let mut item_config: HashMap<String, String> = HashMap::new();
                    if !role.is_empty() {
                        item_config.insert("role".into(), role);
                    }

                    let item_title = if !author.is_empty() { author } else { "Anonymous".to_string() };

                    out.push(ItemBlueprint {
                        item_type: "testimonial".into(),
                        title: item_title,
                        description: Some(quote_text),
                        config: item_config,
                    });
                }
            } else if !child.children.is_empty() {
                // Not a card — recurse deeper (e.g. grid wrapper, section container)
                collect_testimonial_cards(child, out, false);
            }
        }
    }

    collect_testimonial_cards(node, &mut items, true);

    SectionBlueprint {
        section_type: "testimonial".into(),
        confidence: 0.0,
        title,
        subtitle,
        config,
        items,
        template: None,
        style_block: None,
    }
}

// ---------------------------------------------------------------------------
// Extraction: pricing
// ---------------------------------------------------------------------------

fn extract_pricing(node: &DomNode) -> SectionBlueprint {
    let title = find_heading_by_tag(node, "h2")
        .or_else(|| dom::find_heading(node));
    let subtitle = find_subtitle_paragraph(node, &title);

    let mut items: Vec<ItemBlueprint> = Vec::new();

    // Find pricing cards: look for BEM classes first, then generic card patterns
    let pricing_cards = dom::find_by_class(node, "pricing-card");
    let cards: Vec<&DomNode> = if !pricing_cards.is_empty() {
        // Filter out nested elements — only keep top-level pricing-card divs
        pricing_cards.into_iter()
            .filter(|c| c.classes.iter().any(|cls| cls == "pricing-card" || cls.starts_with("pricing-card ")))
            .collect()
    } else {
        find_card_children(node)
    };

    for card in &cards {
        // Extract plan name from h3/h4 or BEM __title
        let plan_name = find_heading_by_tag(card, "h3")
            .or_else(|| find_heading_by_tag(card, "h4"))
            .or_else(|| {
                dom::find_by_class(card, "__title")
                    .first()
                    .map(|n| dom::clean_node_text(n))
            })
            .unwrap_or_default();

        // Extract price from BEM __price or text matching price pattern
        let price = dom::find_by_class(card, "__price")
            .first()
            .map(|n| dom::clean_node_text(n))
            .or_else(|| {
                // Fallback: find any text with R$ or $ pattern
                find_price_text_in(card)
            })
            .unwrap_or_default();

        // Extract feature list items from <ul>/<li>
        let mut features: Vec<String> = Vec::new();
        let uls = dom::find_by_tag(card, "ul");
        for ul in &uls {
            let lis = dom::find_by_tag(ul, "li");
            for li in &lis {
                let text = dom::clean_node_text(li);
                if !text.is_empty() {
                    features.push(text);
                }
            }
        }

        // Check if this plan is "featured" (popular/recommended)
        let is_featured = card.classes.iter().any(|c| c.contains("featured") || c.contains("popular") || c.contains("recommended"))
            || dom::find_by_class(card, "badge").len() > 0;

        let mut config = HashMap::new();
        let has_price = !price.is_empty();
        let has_name = !plan_name.is_empty();
        if has_price {
            config.insert("price".into(), price);
        }
        if is_featured {
            config.insert("featured".into(), "true".into());
        }
        if !features.is_empty() {
            config.insert("_features".into(), features.join("||"));
        }

        // Extract CTA button text
        let buttons = dom::extract_buttons(card);
        if let Some(text) = buttons.first() {
            if !text.is_empty() {
                config.insert("cta".into(), text.clone());
            }
        }

        if has_name || has_price {
            items.push(ItemBlueprint {
                item_type: "plan".into(),
                title: plan_name,
                description: None,
                config,
            });
        }
    }

    SectionBlueprint {
        section_type: "pricing".into(),
        confidence: 0.0,
        title,
        subtitle,
        config: HashMap::new(),
        items,
        template: None,
        style_block: None,
    }
}

/// Find price text in a node subtree.
fn find_price_text_in(node: &DomNode) -> Option<String> {
    let text = node.full_text.trim().to_string();
    if text.contains("R$") || text.contains('$') {
        // Try to find the specific element with price
        for child in &node.children {
            let child_text = dom::clean_node_text(child);
            if child_text.contains("R$") || child_text.contains('$') {
                if child_text.len() < 30 {
                    return Some(child_text);
                }
            }
            if let Some(found) = find_price_text_in(child) {
                return Some(found);
            }
        }
    }
    None
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
        template: None,
        style_block: None,
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

    // Descriptive text: bold + paragraph pairs (e.g. "Enterprise-grade security" + subtitle)
    let bold_nodes = dom::find_by_class(node, "font-bold");
    for bn in &bold_nodes {
        if is_material_icon_span(bn) {
            continue;
        }
        let txt = dom::clean_node_text(bn);
        if !txt.is_empty() && txt.len() < 80 && txt.len() > 3 {
            // Check for a nearby descriptive paragraph
            let parent = find_parent_of(node, bn);
            let desc = parent.and_then(|p| {
                let ps = dom::find_by_tag(p, "p");
                for p_node in &ps {
                    if dom::has_class(p_node, "text-xs") || dom::has_class(p_node, "text-sm") {
                        let p_text = dom::clean_node_text(p_node);
                        if !p_text.is_empty() && p_text != txt {
                            return Some(p_text);
                        }
                    }
                }
                None
            });
            // Extract icon if present
            let mut item_config: HashMap<String, String> = HashMap::new();
            if let Some(p) = parent {
                if let Some(icon) = extract_material_icon(p) {
                    item_config.insert("icon".into(), icon);
                }
            }
            items.push(ItemBlueprint {
                item_type: "item".into(),
                title: txt,
                description: desc,
                config: item_config,
            });
        }
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
        template: None,
        style_block: None,
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
        template: None,
        style_block: None,
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

    // ---------------------------------------------------------------
    // 1) Brand: first bold/large heading (h1/h2 with font-bold)
    // ---------------------------------------------------------------
    let brand = find_sidebar_brand(node);
    if let Some(ref b) = brand {
        config.insert("brand".into(), b.clone());
    }

    // ---------------------------------------------------------------
    // 2) Subtitle: small uppercase text near the brand (text-xs uppercase)
    // ---------------------------------------------------------------
    let subtitle_text = find_sidebar_subtitle(node);
    if let Some(ref s) = subtitle_text {
        config.insert("subtitle".into(), s.clone());
    }

    // ---------------------------------------------------------------
    // 3) Detect the border-t divider to separate main vs bottom items
    // ---------------------------------------------------------------
    // Walk direct children. Items in a container AFTER a border-t div
    // are "bottom" items (Support, Docs, etc).
    let border_t_nodes = dom::find_by_class(node, "border-t");
    let has_bottom_section = !border_t_nodes.is_empty();

    // Collect all <a> tags in the sidebar
    let a_nodes = dom::find_by_tag(node, "a");

    for a_node in &a_nodes {
        let text = dom::clean_node_text(a_node);
        if text.is_empty() {
            continue;
        }

        let href = a_node.attrs.get("href").cloned().unwrap_or_else(|| "#".into());

        let mut item_config: HashMap<String, String> = HashMap::new();
        item_config.insert("href".into(), href);

        // Extract material icon from data-icon attr or text content
        if let Some(icon) = extract_material_icon(a_node) {
            item_config.insert("icon".into(), icon);
        }

        // Detect active state: bg-zinc-100, bg-gray-100, bg-muted,
        // text-black, font-semibold on the <a> itself
        let active = a_node.classes.iter().any(|c| {
            c.contains("bg-zinc-100")
                || c.contains("bg-gray-100")
                || c.contains("bg-muted")
                || c == "text-black"
                || c == "font-semibold"
        });
        if active {
            item_config.insert("active".into(), "true".into());
        }

        // Detect position:bottom — the <a> is inside a border-t container
        if has_bottom_section {
            let is_bottom = is_descendant_of_any(a_node, &border_t_nodes);
            if is_bottom {
                item_config.insert("position".into(), "bottom".into());
            }
        }

        items.push(ItemBlueprint {
            item_type: "nav-link".into(),
            title: text,
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
        template: None,
        style_block: None,
    }
}

/// Find brand text in sidebar: first h1/h2 with font-bold or tracking-tighter.
fn find_sidebar_brand(node: &DomNode) -> Option<String> {
    for tag in &["h1", "h2"] {
        let headings = dom::find_by_tag(node, tag);
        for h in headings {
            if dom::has_class(h, "font-bold") || dom::has_class(h, "tracking-tighter") || dom::has_class(h, "font-semibold") {
                let text = dom::clean_node_text(h);
                if !text.is_empty() && text.len() < 50 {
                    return Some(text);
                }
            }
        }
    }
    // Fallback to generic find_brand_text
    find_brand_text(node)
}

/// Find subtitle in sidebar: text-xs uppercase text (e.g. "Enterprise").
fn find_sidebar_subtitle(node: &DomNode) -> Option<String> {
    // Look for elements with text-xs + uppercase (common pattern for subtitles)
    let candidates = dom::find_by_class(node, "uppercase");
    for c in &candidates {
        if dom::has_class(c, "text-xs") || dom::has_class(c, "tracking-widest") {
            let text = dom::clean_node_text(c);
            if !text.is_empty() && text.len() < 50 {
                return Some(text);
            }
        }
    }
    // Also try <p> with text-xs
    let ps = dom::find_by_tag(node, "p");
    for p in ps {
        if dom::has_class(p, "text-xs") && dom::has_class(p, "uppercase") {
            let text = dom::clean_node_text(p);
            if !text.is_empty() {
                return Some(text);
            }
        }
    }
    None
}

/// Check if a node is a descendant of any node in the given list.
/// Uses full_text identity comparison as a heuristic since DomNode
/// doesn't carry pointer identity.
fn is_descendant_of_any(needle: &DomNode, ancestors: &[&DomNode]) -> bool {
    for ancestor in ancestors {
        if node_contains(ancestor, needle) {
            return true;
        }
    }
    false
}

/// Check if `haystack` contains `needle` anywhere in its subtree (by text + tag match).
fn node_contains(haystack: &DomNode, needle: &DomNode) -> bool {
    for child in &haystack.children {
        // Match by tag + full_text + classes as a proxy for identity
        if child.tag == needle.tag
            && child.full_text == needle.full_text
            && child.classes == needle.classes
        {
            return true;
        }
        if node_contains(child, needle) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Extraction: page-header
// ---------------------------------------------------------------------------

fn extract_page_header(node: &DomNode) -> SectionBlueprint {
    let mut config: HashMap<String, String> = HashMap::new();
    let mut items: Vec<ItemBlueprint> = Vec::new();

    // Title from h1 or h2 (dashboard pages often use h2 with large text)
    let title = find_heading_by_tag(node, "h1")
        .or_else(|| find_heading_by_tag(node, "h2"));

    // Subtitle from p
    let subtitle = dom::find_paragraph(node);

    // Badge detection: uppercase small text like "DEVELOPER" with optional pulse dot
    let badge_text = find_page_header_badge(node);
    if let Some(ref badge) = badge_text {
        config.insert("badge".into(), badge.clone());
    }

    // Detect pulse/status dot in badge area
    if has_pulse_dot(node) {
        config.insert("badge_dot".into(), "pulse".into());
    }

    // Action buttons with icon detection
    let btn_nodes = dom::find_by_tag(node, "button");
    for bn in &btn_nodes {
        let btn_text = clean_button_text(bn);
        if btn_text.is_empty() {
            continue;
        }
        let mut item_config: HashMap<String, String> = HashMap::new();
        if let Some(icon) = extract_material_icon(bn) {
            item_config.insert("icon".into(), icon);
        }
        if dom::has_class(bn, "bg-primary") || dom::has_class(bn, "btn-primary") {
            item_config.insert("style".into(), "primary".into());
        }
        items.push(ItemBlueprint {
            item_type: "action".into(),
            title: btn_text,
            description: None,
            config: item_config,
        });
    }

    SectionBlueprint {
        section_type: "page-header".into(),
        confidence: 0.0,
        title,
        subtitle,
        config,
        items,
        template: None,
        style_block: None,
    }
}

/// Find a badge text in a page header (e.g. "DEVELOPER" with uppercase + tracking).
fn find_page_header_badge(node: &DomNode) -> Option<String> {
    // Look for uppercase + tracking-wider/tracking-widest small text
    let upper_nodes = dom::find_by_class(node, "uppercase");
    for u in &upper_nodes {
        if dom::has_class(u, "tracking-wider") || dom::has_class(u, "tracking-widest") {
            let txt = dom::clean_node_text(u);
            if !txt.is_empty() && txt.len() < 30 {
                return Some(txt);
            }
        }
    }
    // Also check for small rounded-full spans with short text (badge pills)
    let badge_nodes = dom::find_by_class(node, "rounded-full");
    for b in &badge_nodes {
        if b.tag == "button" || b.tag == "a" || is_material_icon_span(b) {
            continue;
        }
        let txt = dom::clean_node_text(b);
        if !txt.is_empty() && txt.len() < 30 {
            return Some(txt);
        }
    }
    None
}

/// Check if a node contains a pulse/animated dot (animate-pulse on a small rounded element).
fn has_pulse_dot(node: &DomNode) -> bool {
    if dom::has_class(node, "animate-pulse") {
        return true;
    }
    for child in &node.children {
        if has_pulse_dot(child) {
            return true;
        }
    }
    false
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
        let mut card_config: HashMap<String, String> = HashMap::new();

        // Label: small uppercase text (tracking-widest, uppercase, text-xs/text-sm)
        let label = find_stat_label(card);
        // Value: large bold text (text-2xl, text-3xl, font-bold)
        let value = find_stat_value(card);

        // Icon: material icon in the stat card (e.g. billing stats: hub, speed, shield)
        if let Some(icon) = extract_material_icon(card) {
            card_config.insert("icon".into(), icon);
        }

        if label.is_some() || value.is_some() {
            items.push(ItemBlueprint {
                item_type: "stat".into(),
                title: label.unwrap_or_default(),
                description: value,
                config: card_config,
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
        template: None,
        style_block: None,
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
    let label_classes = ["uppercase", "tracking-widest", "tracking-wider", "tracking-tighter"];
    for cls in &label_classes {
        let matches = dom::find_by_class(node, cls);
        for m in matches {
            // Must be small text (label, not a heading)
            let is_small = dom::has_class(m, "text-xs") || dom::has_class(m, "text-sm")
                || m.classes.iter().any(|c| c.contains("text-["));
            // Skip if not small and not uppercase
            if !is_small && !dom::has_class(m, "uppercase") {
                continue;
            }
            // Skip material icons
            if is_material_icon_span(m) {
                continue;
            }
            let txt = m.full_text.trim().to_string();
            if !txt.is_empty() && txt.len() < 60 {
                // Skip if this is just a number (that's the value, not the label)
                if txt.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '%' || c == '$' || c == ',' || c == 'M' || c == 'K') {
                    continue;
                }
                return Some(txt);
            }
        }
    }
    // Fallback: find text-xs or text-sm elements
    for cls in &["text-xs", "text-sm"] {
        let matches = dom::find_by_class(node, cls);
        for m in matches {
            if is_material_icon_span(m) {
                continue;
            }
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

            // Extract action buttons inside this product card (Copy link, Finish setup, etc.)
            let card_buttons = extract_clean_buttons(card);
            for btn in &card_buttons {
                if !btn.is_empty() {
                    let mut btn_config: HashMap<String, String> = HashMap::new();
                    btn_config.insert("role".into(), "child".into());
                    // Try to find icon on the button
                    let btn_nodes = dom::find_by_tag(card, "button");
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
        }
    }

    SectionBlueprint {
        section_type: "product-grid".into(),
        confidence: 0.0,
        title: None,
        subtitle: None,
        config: section_config,
        items,
        template: None,
        style_block: None,
    }
}

/// Find price text ($XX.XX) in a node, including sibling suffix like "/ month" or "ETH".
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

    // Strategy 2: Find a container with items-baseline (price + suffix pattern)
    // e.g. <div class="flex items-baseline gap-1"><span>$99.00</span><span>/ month</span></div>
    let baseline_nodes = dom::find_by_class(node, "items-baseline");
    for bn in &baseline_nodes {
        let full = bn.full_text.trim().to_string();
        if full.contains('$') && full.len() < 60 {
            return Some(full);
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
    let badge_classes = ["rounded-full", "badge", "chip", "rounded"];
    for cls in &badge_classes {
        let matches = dom::find_by_class(node, cls);
        for m in matches {
            // Skip buttons, links, and material icon spans
            if m.tag == "button" || m.tag == "a" || is_material_icon_span(m) {
                continue;
            }
            let txt = m.full_text.trim().to_string();
            let lower = txt.to_lowercase();
            if (lower.contains("active") || lower.contains("draft")
                || lower.contains("paused") || lower.contains("archived")
                || lower.contains("testing") || lower.contains("inactive")
                || lower.contains("pending") || lower.contains("enabled")
                || lower.contains("disabled") || lower.contains("new feature")
                || lower.contains("beta") || lower.contains("premium")
                || lower.contains("pro") || lower.contains("new"))
                && txt.len() < 30
            {
                return Some(txt);
            }
        }
    }
    // Also check for colored span badges (bg-green-*, bg-zinc-*, bg-red-*)
    let color_badge_prefixes = ["bg-green-", "bg-red-", "bg-yellow-", "bg-zinc-", "bg-blue-", "bg-orange-", "bg-white"];
    for prefix in &color_badge_prefixes {
        let matches = dom::find_by_class(node, prefix);
        for m in matches {
            if m.tag == "button" || m.tag == "a" || is_material_icon_span(m) {
                continue;
            }
            let txt = m.full_text.trim().to_string();
            if !txt.is_empty() && txt.len() < 30 {
                return Some(txt);
            }
        }
    }
    // Also check for uppercase inline-block badges (small bold text with bg)
    let uppercase_nodes = dom::find_by_class(node, "uppercase");
    for m in &uppercase_nodes {
        if is_material_icon_span(m) || m.tag == "button" || m.tag == "a" {
            continue;
        }
        let is_small = dom::has_class(m, "text-xs") || dom::has_class(m, "text-sm")
            || m.classes.iter().any(|c| c.contains("text-[10px]") || c.contains("text-[11px]"));
        if !is_small {
            continue;
        }
        let has_bg = m.classes.iter().any(|c| c.starts_with("bg-"));
        let is_inline = dom::has_class(m, "inline-block") || dom::has_class(m, "inline-flex");
        if !has_bg && !is_inline {
            continue;
        }
        let txt = m.full_text.trim().to_string();
        if !txt.is_empty() && txt.len() < 30 {
            return Some(txt);
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
        template: None,
        style_block: None,
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
    let mut section_config: HashMap<String, String> = HashMap::new();

    // Title from h2/h3/h4
    let title = find_heading_by_tag(node, "h3")
        .or_else(|| find_heading_by_tag(node, "h4"))
        .or_else(|| find_heading_by_tag(node, "h2"));

    // Subtitle from first p (skip paragraphs inside divide-y / row containers)
    let subtitle = find_card_subtitle(node);

    // Extract header-area material icon (icon in flex header near title)
    if let Some(icon) = extract_header_icon(node) {
        section_config.insert("icon".into(), icon);
    }

    // -----------------------------------------------------------------------
    // DOM-ordered walk: collect labels, code, and actions in document order
    // so that interleaved structures (label->code->action, label->code->action)
    // are preserved instead of grouping all labels, then all codes, etc.
    // -----------------------------------------------------------------------
    let mut seen_labels: Vec<String> = Vec::new();
    let mut seen_btns: Vec<String> = Vec::new();
    let mut seen_codes: Vec<String> = Vec::new();
    card_walk_ordered(node, &mut items, &mut seen_labels, &mut seen_btns, &mut seen_codes, node);

    // Extract row entries from divide-y containers or font-mono URLs with status
    let row_entries = find_webhook_entries(node);
    for (url, status, events) in row_entries {
        let mut row_config: HashMap<String, String> = HashMap::new();
        if let Some(ref s) = status {
            row_config.insert("status".into(), s.to_lowercase());
        }
        items.push(ItemBlueprint {
            item_type: "row".into(),
            title: url,
            description: events,
            config: row_config,
        });
    }

    // Key-value rows (billing info like "Billing cycle" -> "Annual...")
    let kv_items = extract_key_value_rows(node);
    items.extend(kv_items);

    // Progress bars (usage meters)
    let progress_items = extract_labeled_progress_bars(node);
    items.extend(progress_items);

    // Invoice rows
    let invoice_items = extract_invoice_rows(node);
    items.extend(invoice_items);

    // Payment method rows
    let payment_items = extract_payment_method_rows(node);
    items.extend(payment_items);

    // Inline badges (e.g. "Active Plan")
    let badge_items = extract_inline_badges(node);
    items.extend(badge_items);

    // Links (e.g. "View detailed analytics")
    let links = dom::extract_links(node);
    for (text, href) in &links {
        if !text.is_empty() {
            let mut link_config: HashMap<String, String> = HashMap::new();
            link_config.insert("href".into(), href.clone());
            items.push(ItemBlueprint {
                item_type: "link".into(),
                title: text.clone(),
                description: None,
                config: link_config,
            });
        }
    }

    SectionBlueprint {
        section_type: "card".into(),
        confidence: 0.0,
        title,
        subtitle,
        config: section_config,
        items,
        template: None,
        style_block: None,
    }
}

/// Recursively walk a card's DOM subtree in document order, emitting label,
/// code, and action items as they are encountered.  This preserves the
/// interleaved ordering (label -> code -> action per key group) instead of
/// batching all items of the same type together.
///
/// `root` is the card-level node used for ancestor lookups.
fn card_walk_ordered(
    node: &DomNode,
    items: &mut Vec<ItemBlueprint>,
    seen_labels: &mut Vec<String>,
    seen_btns: &mut Vec<String>,
    seen_codes: &mut Vec<String>,
    root: &DomNode,
) {
    // -- Skip nodes inside divide-y containers (handled by find_webhook_entries) --
    if dom::has_class(node, "divide-y") {
        return;
    }

    // -- Label: <label> tag OR element with uppercase + tracking-widest/tracking-wider --
    let is_label_tag = node.tag == "label";
    let is_label_class = dom::has_class(node, "uppercase")
        && (dom::has_class(node, "tracking-widest") || dom::has_class(node, "tracking-wider"));

    if is_label_tag || is_label_class {
        let txt = if is_label_tag {
            dom::clean_node_text(node)
        } else {
            node.full_text.trim().to_string()
        };
        if !txt.is_empty() && txt.len() < 80 && !seen_labels.contains(&txt) {
            seen_labels.push(txt.clone());
            let mut label_config: HashMap<String, String> = HashMap::new();
            if dom::has_class(node, "uppercase") || dom::has_class(node, "tracking-widest") {
                label_config.insert("style".into(), "uppercase".into());
            }
            items.push(ItemBlueprint {
                item_type: "label".into(),
                title: txt,
                description: None,
                config: label_config,
            });
            // Don't recurse into label children -- already captured
            return;
        }
    }

    // -- Code: element with font-mono class containing non-URL text --
    if dom::has_class(node, "font-mono") && !is_material_icon_span(node) {
        // Extract text excluding nested button text (e.g. "Reveal" inside mono div)
        let txt = extract_mono_text_clean(node);
        if !txt.is_empty() && txt.len() < 200 && !seen_codes.contains(&txt) {
            // Skip URL-like text (handled by row extraction)
            let is_url = txt.starts_with("http") || txt.contains("://");
            // Skip font-mono + font-bold inside divide-y (webhook URL row titles)
            let is_divide_row = dom::has_class(node, "font-bold") && {
                let parent = find_parent_of(root, node);
                parent.map_or(false, |p| {
                    let gp = find_parent_of(root, p);
                    gp.map_or(false, |g| dom::has_class(g, "divide-y"))
                })
            };
            if !is_url && !is_divide_row {
                seen_codes.push(txt.clone());
                let mut code_config: HashMap<String, String> = HashMap::new();
                code_config.insert("_type".into(), "code".into());
                items.push(ItemBlueprint {
                    item_type: "code".into(),
                    title: txt,
                    description: None,
                    config: code_config,
                });
                // Don't recurse -- we already captured the mono content
                return;
            }
        }
    }

    // -- Action: <button> or <a> styled as button --
    let is_button = node.tag == "button";
    let is_a_button = node.tag == "a"
        && (dom::has_class(node, "rounded-full")
            || dom::has_class(node, "btn")
            || dom::has_class(node, "bg-primary"));

    if is_button || is_a_button {
        let mut btn_text = clean_button_text(node);
        let icon = extract_material_icon(node);

        // When button has only a material icon and no visible text, derive
        // a human-readable title from the icon name (e.g. content_copy -> "Copy")
        if btn_text.is_empty() {
            if let Some(ref icon_name) = icon {
                btn_text = icon_name_to_action_title(icon_name);
            }
        }

        if !btn_text.is_empty() && !seen_btns.contains(&btn_text) {
            seen_btns.push(btn_text.clone());
            let mut btn_config: HashMap<String, String> = HashMap::new();
            btn_config.insert("_type".into(), "action".into());
            if let Some(ref i) = icon {
                btn_config.insert("icon".into(), i.clone());
            }
            if dom::has_class(node, "bg-primary") || dom::has_class(node, "btn-primary") {
                btn_config.insert("style".into(), "primary".into());
            }
            items.push(ItemBlueprint {
                item_type: "action".into(),
                title: btn_text,
                description: None,
                config: btn_config,
            });
        }
        // Don't recurse into button children
        return;
    }

    // -- Recurse into children --
    for child in &node.children {
        card_walk_ordered(child, items, seen_labels, seen_btns, seen_codes, root);
    }
}

/// Extract text from a font-mono element, excluding nested <button> text.
/// This handles cases like a code div containing both the value and a
/// "Reveal" button -- we want only the code value, not the button label.
fn extract_mono_text_clean(node: &DomNode) -> String {
    let mut parts: Vec<String> = Vec::new();

    // Add direct text
    let direct = node.text.trim().to_string();
    if !direct.is_empty() && !is_material_icon_text(&direct) {
        parts.push(direct);
    }

    // Recurse into children, skipping buttons and material icon spans
    for child in &node.children {
        if child.tag == "button" || is_material_icon_span(child) {
            continue;
        }
        let child_text = extract_mono_text_clean(child);
        if !child_text.is_empty() {
            parts.push(child_text);
        }
    }

    let result = parts.join(" ");
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Convert a material icon name to a human-readable action title.
/// e.g. "content_copy" -> "Copy", "content_paste" -> "Paste"
fn icon_name_to_action_title(icon: &str) -> String {
    match icon {
        "content_copy" => "Copy".into(),
        "content_paste" => "Paste".into(),
        "content_cut" => "Cut".into(),
        "delete" => "Delete".into(),
        "edit" => "Edit".into(),
        "add" => "Add".into(),
        "remove" => "Remove".into(),
        "close" => "Close".into(),
        "search" => "Search".into(),
        "share" => "Share".into(),
        "download" => "Download".into(),
        "upload" => "Upload".into(),
        "visibility" => "Show".into(),
        "visibility_off" => "Hide".into(),
        "lock" => "Lock".into(),
        "lock_open" => "Unlock".into(),
        "settings" => "Settings".into(),
        "refresh" | "sync" => "Refresh".into(),
        "open_in_new" => "Open".into(),
        _ => {
            // Generic fallback: replace underscores, capitalize first letter
            let words: Vec<String> = icon.split('_')
                .map(|w| {
                    let mut c = w.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().to_string() + c.as_str(),
                    }
                })
                .collect();
            words.join(" ")
        }
    }
}

/// Find the subtitle paragraph for a card, skipping paragraphs inside
/// divide-y containers (which are row details, not the card subtitle).
fn find_card_subtitle(node: &DomNode) -> Option<String> {
    let paragraphs = dom::find_by_tag(node, "p");
    for p in &paragraphs {
        let txt = p.full_text.trim().to_string();
        if txt.is_empty() {
            continue;
        }
        // Skip if inside a divide-y container (webhook/row detail)
        let parent = find_parent_of(node, p);
        if let Some(par) = parent {
            if dom::has_class(par, "divide-y") {
                continue;
            }
            let grandparent = find_parent_of(node, par);
            if let Some(gp) = grandparent {
                if dom::has_class(gp, "divide-y") {
                    continue;
                }
            }
        }
        // Skip event detail lines
        let lower = txt.to_lowercase();
        if lower.starts_with("events:") {
            continue;
        }
        return Some(txt);
    }
    None
}

/// Extract the header-area material icon from a card.
/// Looks for an icon in the flex header row (near the title).
fn extract_header_icon(node: &DomNode) -> Option<String> {
    // Check direct children first (flex header row)
    for child in &node.children {
        if dom::has_class(child, "flex") || dom::has_class(child, "justify-between") {
            if let Some(icon) = extract_material_icon(child) {
                return Some(icon);
            }
        }
    }
    None
}

/// Find webhook/row entries: rows in divide-y containers or font-mono URLs with status badges.
fn find_webhook_entries(node: &DomNode) -> Vec<(String, Option<String>, Option<String>)> {
    let mut entries = Vec::new();

    // Strategy 1: Find divide-y containers (structured row lists like webhook endpoints)
    let divide_containers = dom::find_by_class(node, "divide-y");
    for container in &divide_containers {
        for row_child in &container.children {
            // Each row should have font-mono URL + optional status badge + event text
            let mono_in_row = dom::find_by_class(row_child, "font-mono");
            let mut url_text: Option<String> = None;
            for mono in &mono_in_row {
                if is_material_icon_span(mono) {
                    continue;
                }
                let txt = mono.full_text.trim().to_string();
                if !txt.is_empty() && (txt.starts_with("http") || txt.contains("://")
                    || txt.contains(".") || dom::has_class(mono, "font-bold"))
                {
                    url_text = Some(txt);
                    break;
                }
            }
            if let Some(url) = url_text {
                let status = find_status_badge(row_child);
                let events = find_row_detail_text(row_child);
                entries.push((url, status, events));
            }
        }
    }

    // Strategy 2: Fallback — font-mono URLs not in divide-y
    if entries.is_empty() {
        let mono_nodes = dom::find_by_class(node, "font-mono");
        for mono in &mono_nodes {
            let txt = mono.full_text.trim().to_string();
            if txt.starts_with("http") || txt.contains("://") {
                let parent = find_parent_of(node, mono);
                let status = parent.and_then(|p| find_status_badge(p));
                let events = parent.and_then(|p| dom::find_paragraph(p));
                entries.push((txt, status, events));
            }
        }
    }

    entries
}

/// Find detail text in a row (e.g. "Events: payment.succeeded, ...").
fn find_row_detail_text(node: &DomNode) -> Option<String> {
    let paragraphs = dom::find_by_tag(node, "p");
    for p in &paragraphs {
        let txt = p.full_text.trim().to_string();
        if !txt.is_empty() {
            return Some(txt);
        }
    }
    // Fallback: text-xs/text-sm spans with substantive content
    for cls in &["text-xs", "text-sm"] {
        let small_nodes = dom::find_by_class(node, cls);
        for s in &small_nodes {
            if is_material_icon_span(s) {
                continue;
            }
            let txt = s.full_text.trim().to_string();
            if !txt.is_empty() && txt.len() > 10 {
                return Some(txt);
            }
        }
    }
    None
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
        template: None,
        style_block: None,
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
// Extraction: form
// ---------------------------------------------------------------------------

fn extract_form(node: &DomNode) -> SectionBlueprint {
    let mut config: HashMap<String, String> = HashMap::new();

    let title = find_heading_by_tag(node, "h2")
        .or_else(|| find_heading_by_tag(node, "h3"))
        .or_else(|| dom::find_heading(node));
    let subtitle = dom::find_paragraph(node);

    // Extract form action if present
    if node.tag == "form" {
        if let Some(action) = node.attrs.get("action") {
            if !action.is_empty() {
                config.insert("action".into(), action.clone());
            }
        }
        if let Some(method) = node.attrs.get("method") {
            config.insert("method".into(), method.to_uppercase());
        }
    } else {
        // Check for descendant <form>
        let forms = dom::find_by_tag(node, "form");
        if let Some(form) = forms.first() {
            if let Some(action) = form.attrs.get("action") {
                if !action.is_empty() {
                    config.insert("action".into(), action.clone());
                }
            }
        }
    }

    // Extract form fields
    let items = extract_form_items(node);

    // Extract submit/action buttons
    let mut all_items = items;
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
            all_items.push(ItemBlueprint {
                item_type: "action".into(),
                title: btn.clone(),
                description: None,
                config: item_config,
            });
        }
    }

    SectionBlueprint {
        section_type: "form".into(),
        confidence: 0.0,
        title,
        subtitle,
        config,
        items: all_items,
        template: None,
        style_block: None,
    }
}

// ---------------------------------------------------------------------------
// Helper: find nodes with specific attribute
// ---------------------------------------------------------------------------

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
// Extraction: tabs
// ---------------------------------------------------------------------------

fn extract_tabs(node: &DomNode) -> SectionBlueprint {
    let mut items: Vec<ItemBlueprint> = Vec::new();

    let title = find_heading_by_tag(node, "h2")
        .or_else(|| dom::find_heading(node));

    // Strategy 1: Find [role="tablist"] container
    let tablists = find_nodes_with_attr(node, "role", "tablist");
    for tablist in &tablists {
        for child in &tablist.children {
            if child.tag == "button" || child.tag == "a" {
                let label = dom::clean_node_text(child);
                if label.is_empty() {
                    continue;
                }
                let mut item_config: HashMap<String, String> = HashMap::new();
                let is_active = child.attrs.get("aria-selected")
                    .map(|v| v == "true")
                    .unwrap_or(false)
                    || dom::has_class(child, "active")
                    || dom::has_class(child, "bg-white")
                    || dom::has_class(child, "border-b-2");
                if is_active {
                    item_config.insert("active".into(), "true".into());
                }
                items.push(ItemBlueprint {
                    item_type: "item".into(),
                    title: label,
                    description: None,
                    config: item_config,
                });
            }
        }
    }

    // Strategy 2: Border-b container with button/link children
    if items.is_empty() {
        let border_nodes = dom::find_by_class(node, "border-b");
        for bn in &border_nodes {
            for child in &bn.children {
                if child.tag == "button" || child.tag == "a" {
                    let label = dom::clean_node_text(child);
                    if label.is_empty() || label.len() > 30 {
                        continue;
                    }
                    let mut item_config: HashMap<String, String> = HashMap::new();
                    let is_active = dom::has_class(child, "active")
                        || dom::has_class(child, "border-b-2")
                        || dom::has_class(child, "text-black")
                        || dom::has_class(child, "font-semibold");
                    if is_active {
                        item_config.insert("active".into(), "true".into());
                    }
                    items.push(ItemBlueprint {
                        item_type: "item".into(),
                        title: label,
                        description: None,
                        config: item_config,
                    });
                }
            }
            if items.len() >= 2 {
                break;
            }
        }
    }

    // Strategy 3: Direct button/link children of this node
    if items.is_empty() {
        for child in &node.children {
            if child.tag == "button" || child.tag == "a" {
                let label = dom::clean_node_text(child);
                if label.is_empty() || label.len() > 30 {
                    continue;
                }
                let mut item_config: HashMap<String, String> = HashMap::new();
                if child.attrs.get("aria-selected").map(|v| v == "true").unwrap_or(false) {
                    item_config.insert("active".into(), "true".into());
                }
                items.push(ItemBlueprint {
                    item_type: "item".into(),
                    title: label,
                    description: None,
                    config: item_config,
                });
            }
        }
    }

    SectionBlueprint {
        section_type: "tabs".into(),
        confidence: 0.0,
        title,
        subtitle: None,
        config: HashMap::new(),
        items,
        template: None,
        style_block: None,
    }
}

// ---------------------------------------------------------------------------
// Extraction: generic (fallback) — smart dashboard element detection
// ---------------------------------------------------------------------------

// ═══════════════════════════════════════════════════════════════
// Dashboard-specific extractors
// ═══════════════════════════════════════════════════════════════

fn extract_order_header(node: &DomNode) -> SectionBlueprint {
    let mut config: HashMap<String, String> = HashMap::new();
    let title = find_heading_by_tag(node, "h2").or_else(|| dom::find_heading(node));
    let subtitle = dom::find_paragraph(node);

    // Back link: text containing "Back to"
    let all_text = collect_all_text_nodes(node);
    for t in &all_text {
        if t.to_lowercase().contains("back to") {
            config.insert("back_link".into(), t.trim().to_string());
            break;
        }
    }

    // Badge: short uppercase text in a small colored container
    let badge_nodes = dom::find_by_class(node, "uppercase");
    for bn in &badge_nodes {
        let txt = dom::clean_node_text(bn).trim().to_string();
        if !txt.is_empty() && txt.len() < 20 && txt.to_lowercase() != "back to orders" {
            let lower = txt.to_lowercase();
            let statuses = ["shipped", "processing", "fulfilled", "pending", "completed", "cancelled"];
            if statuses.iter().any(|s| lower.contains(s)) {
                config.insert("badge".into(), txt);
                break;
            }
        }
    }

    // Action buttons
    let buttons = extract_clean_buttons(node);
    for (i, btn) in buttons.iter().enumerate() {
        let lower = btn.to_lowercase();
        if lower.contains("refund") || lower.contains("process") {
            config.insert("action_primary".into(), btn.clone());
        } else if lower.contains("invoice") || lower.contains("download") {
            config.insert("action_secondary".into(), btn.clone());
        } else if i == 0 && !config.contains_key("action_secondary") {
            config.insert("action_secondary".into(), btn.clone());
        } else if !config.contains_key("action_primary") {
            config.insert("action_primary".into(), btn.clone());
        }
    }

    SectionBlueprint { section_type: "order-header".into(), confidence: 0.0, title, subtitle, config, items: Vec::new(), template: None, style_block: None }
}

fn extract_line_items_section(node: &DomNode) -> SectionBlueprint {
    let mut items: Vec<ItemBlueprint> = Vec::new();
    let title = dom::find_heading(node);

    // Find product rows: divs with img + text + price pattern
    fn find_product_rows(node: &DomNode, items: &mut Vec<ItemBlueprint>) {
        // A product row typically has: img, product name (h4/bold), price ($X), maybe SKU
        let imgs = dom::find_by_tag(node, "img");
        if !imgs.is_empty() {
            // This node likely contains products
            for child in &node.children {
                let child_text = &child.full_text;
                let has_img = !dom::find_by_tag(child, "img").is_empty();
                let has_price = child_text.contains('$');
                if has_img && has_price {
                    let mut config = HashMap::new();
                    // Extract image
                    if let Some(img) = dom::find_by_tag(child, "img").first() {
                        if let Some(src) = img.attrs.get("src") {
                            config.insert("image".into(), src.clone());
                        }
                    }
                    // Extract product name (h4 or first bold text)
                    let h4s = dom::find_by_tag(child, "h4");
                    let name = h4s.first().map(|h| dom::clean_node_text(h))
                        .unwrap_or_default();
                    // Extract SKU (mono text with SKU: prefix)
                    for t in collect_all_text_nodes(child) {
                        let trimmed = t.trim();
                        if trimmed.to_lowercase().starts_with("sku:") || trimmed.to_lowercase().starts_with("sku ") {
                            config.insert("sku".into(), trimmed.trim_start_matches("SKU:").trim_start_matches("SKU ").trim().to_string());
                        }
                    }
                    // Extract price ($X,XXX.XX)
                    for t in collect_all_text_nodes(child) {
                        let trimmed = t.trim();
                        if trimmed.starts_with('$') && trimmed.len() > 2 {
                            config.insert("price".into(), trimmed.to_string());
                            break;
                        }
                    }
                    // Extract qty
                    for t in collect_all_text_nodes(child) {
                        let lower = t.to_lowercase();
                        if lower.starts_with("qty:") || lower.starts_with("qty ") {
                            config.insert("qty".into(), lower.trim_start_matches("qty:").trim_start_matches("qty ").trim().to_string());
                        }
                    }
                    // Extract variant (subtitle-like text, not SKU, not price, not qty)
                    let ps = dom::find_by_tag(child, "p");
                    for p in &ps {
                        let pt = dom::clean_node_text(p);
                        let lower = pt.to_lowercase();
                        if !lower.starts_with("sku") && !lower.starts_with("qty") && !pt.starts_with('$')
                            && pt.len() > 5 && pt.len() < 100
                            && (dom::has_class(p, "text-sm") || dom::has_class(p, "text-on-surface-variant")) {
                            config.insert("variant".into(), pt);
                            break;
                        }
                    }
                    if !name.is_empty() {
                        items.push(ItemBlueprint { item_type: "item".into(), title: name, description: None, config });
                    }
                }
            }
        }
        // Recurse into children if no products found at this level
        if items.is_empty() {
            for child in &node.children {
                find_product_rows(child, items);
                if !items.is_empty() { break; }
            }
        }
    }

    find_product_rows(node, &mut items);

    SectionBlueprint { section_type: "line-items".into(), confidence: 0.0, title, subtitle: None, config: HashMap::new(), items, template: None, style_block: None }
}

fn extract_price_breakdown(node: &DomNode) -> SectionBlueprint {
    let mut items: Vec<ItemBlueprint> = Vec::new();
    // Find label + price pairs
    let all_text = collect_all_text_nodes(node);
    let mut i = 0;
    while i < all_text.len() {
        let t = all_text[i].trim();
        // Check if next text is a price
        if i + 1 < all_text.len() {
            let next = all_text[i + 1].trim();
            if next.starts_with('$') && !t.starts_with('$') && t.len() > 2 {
                let mut config = HashMap::new();
                config.insert("value".into(), next.to_string());
                let lower = t.to_lowercase();
                if lower.contains("total") && !lower.contains("subtotal") {
                    config.insert("style".into(), "total".into());
                }
                items.push(ItemBlueprint { item_type: "item".into(), title: t.to_string(), description: None, config });
                i += 2;
                continue;
            }
        }
        i += 1;
    }
    SectionBlueprint { section_type: "price-breakdown".into(), confidence: 0.0, title: None, subtitle: None, config: HashMap::new(), items, template: None, style_block: None }
}

fn extract_shipping_timeline(node: &DomNode) -> SectionBlueprint {
    let mut items: Vec<ItemBlueprint> = Vec::new();
    let title = dom::find_heading(node);

    // Timeline steps: look for repeated div patterns with status text + date
    fn find_timeline_steps(node: &DomNode, items: &mut Vec<ItemBlueprint>) {
        // Check children for step-like patterns
        for child in &node.children {
            let text = dom::clean_node_text(child);
            if text.is_empty() { continue; }

            // Look for bold text (step title) + small date text
            let bold_nodes = dom::find_by_class(child, "font-bold");
            for bn in &bold_nodes {
                if is_material_icon_span(bn) { continue; }
                let step_title = dom::clean_node_text(bn).trim().to_string();
                if step_title.is_empty() || step_title.len() > 60 { continue; }

                let mut config = HashMap::new();
                // Find date subtitle nearby
                let parent_text = &child.full_text;
                // Look for date patterns in sibling text
                for t in collect_all_text_nodes(child) {
                    let trimmed = t.trim();
                    if trimmed != step_title && trimmed.len() > 5 && trimmed.len() < 60 {
                        if trimmed.contains("202") || trimmed.contains("AM") || trimmed.contains("PM") || trimmed.to_lowercase().starts_with("est:") {
                            config.insert("subtitle".into(), trimmed.to_string());
                        } else if trimmed.len() > 10 && (trimmed.contains("carrier") || trimmed.contains("package") || trimmed.contains("tracking")) {
                            config.insert("detail".into(), trimmed.to_string());
                        }
                    }
                }

                // Detect step style
                let has_pulse = dom::find_by_class(child, "animate-pulse").len() > 0;
                let has_check = child.full_text.to_lowercase().contains("check");
                let is_dimmed = dom::has_class(bn, "text-on-surface/40") || child.full_text.contains("/40");
                if has_pulse {
                    config.insert("style".into(), "active".into());
                } else if is_dimmed || step_title.to_lowercase().contains("expected") {
                    config.insert("style".into(), "future".into());
                } else {
                    config.insert("style".into(), "completed".into());
                }

                items.push(ItemBlueprint { item_type: "item".into(), title: step_title, description: None, config });
            }
            // Recurse
            if items.is_empty() {
                find_timeline_steps(child, items);
            }
        }
    }

    find_timeline_steps(node, &mut items);

    SectionBlueprint { section_type: "shipping-timeline".into(), confidence: 0.0, title, subtitle: None, config: HashMap::new(), items, template: None, style_block: None }
}

fn extract_customer_profile(node: &DomNode) -> SectionBlueprint {
    let mut config: HashMap<String, String> = HashMap::new();
    let title = dom::find_heading(node);

    // Customer name (large bold text, usually h4 or strong)
    let h4s = dom::find_by_tag(node, "h4");
    if let Some(h4) = h4s.first() {
        let name = dom::clean_node_text(h4).trim().to_string();
        if !name.is_empty() { config.insert("customer_name".into(), name); }
    }

    // Avatar
    let imgs = dom::find_by_tag(node, "img");
    for img in &imgs {
        if let Some(src) = img.attrs.get("src") {
            // Avatar is typically in a rounded-full container
            if img.classes.iter().any(|c| c.contains("rounded-full") || c.contains("object-cover")) {
                config.insert("customer_avatar".into(), src.clone());
                break;
            }
        }
    }

    // Extract labeled fields
    let all_text = collect_all_text_nodes(node);
    for (i, t) in all_text.iter().enumerate() {
        let lower = t.to_lowercase().trim().to_string();
        if lower.contains("email") && lower.contains("address") {
            config.insert("email_label".into(), t.trim().to_string());
            if i + 1 < all_text.len() {
                let val = all_text[i + 1].trim();
                if val.contains('@') { config.insert("customer_email".into(), val.to_string()); }
            }
        }
        if lower.contains("tier") || lower.contains("member") {
            if !lower.contains("heading") && t.trim().len() < 30 {
                config.insert("customer_tier".into(), t.trim().to_string());
            }
        }
        if lower.contains("shipping") && lower.contains("destination") || lower.contains("address") && !lower.contains("email") {
            config.insert("address_label".into(), t.trim().to_string());
            // Next block of text is likely the address
            if i + 1 < all_text.len() {
                let addr_parts: Vec<&str> = all_text[i+1..].iter()
                    .take(5)
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty() && s.len() < 50)
                    .collect();
                if !addr_parts.is_empty() {
                    config.insert("customer_address".into(), addr_parts.join("\\n"));
                }
            }
        }
    }

    SectionBlueprint { section_type: "customer-profile".into(), confidence: 0.0, title, subtitle: None, config, items: Vec::new(), template: None, style_block: None }
}

fn extract_payment_info(node: &DomNode) -> SectionBlueprint {
    let mut items: Vec<ItemBlueprint> = Vec::new();

    // Find payment cards (grid children)
    for child in &node.children {
        let text = &child.full_text;
        let heading = dom::find_heading(child);
        if heading.is_none() && text.len() < 10 { continue; }

        let mut config = HashMap::new();
        let h = heading.unwrap_or_default();

        // Extract details
        let bold_nodes = dom::find_by_class(child, "font-bold");
        for bn in &bold_nodes {
            if is_material_icon_span(bn) { continue; }
            let t = dom::clean_node_text(bn).trim().to_string();
            if !t.is_empty() && t != h && t.len() < 60 {
                config.insert("title".into(), t);
                break;
            }
        }
        // Subtitle
        let ps = dom::find_by_tag(child, "p");
        for p in &ps {
            if dom::has_class(p, "text-xs") || dom::has_class(p, "text-sm") {
                let pt = dom::clean_node_text(p).trim().to_string();
                let lower = pt.to_lowercase();
                if !pt.is_empty() && !lower.contains("payment") && !lower.contains("security") && pt.len() < 80 {
                    config.insert("subtitle".into(), pt);
                    break;
                }
            }
        }
        // Icon (Material Symbols)
        let icons = dom::find_by_class(child, "material-symbols");
        for icon in &icons {
            let icon_text = dom::clean_node_text(icon).trim().to_string();
            if !icon_text.is_empty() {
                config.insert("icon".into(), icon_text);
                break;
            }
        }
        // Label (VISA, etc.)
        if text.contains("VISA") { config.insert("label".into(), "VISA".into()); }
        else if text.contains("MASTERCARD") { config.insert("label".into(), "MASTERCARD".into()); }

        items.push(ItemBlueprint { item_type: "item".into(), title: h, description: None, config });
    }

    SectionBlueprint { section_type: "payment-info".into(), confidence: 0.0, title: None, subtitle: None, config: HashMap::new(), items, template: None, style_block: None }
}

fn extract_staff_notes(node: &DomNode) -> SectionBlueprint {
    let mut items: Vec<ItemBlueprint> = Vec::new();
    let mut config: HashMap<String, String> = HashMap::new();
    let title = dom::find_heading(node);

    // Find italic/quoted text
    let italics = dom::find_by_class(node, "italic");
    for it in &italics {
        let text = dom::clean_node_text(it).trim().trim_matches('"').to_string();
        if !text.is_empty() && text.len() > 10 {
            let mut item_config = HashMap::new();
            // Find author (bold text after the quote)
            let bolds = dom::find_by_class(node, "font-bold");
            for b in &bolds {
                let bt = dom::clean_node_text(b).trim().to_string();
                if bt.contains("—") || bt.contains("-") {
                    item_config.insert("meta".into(), bt);
                    break;
                }
            }
            items.push(ItemBlueprint { item_type: "item".into(), title: text, description: None, config: item_config });
        }
    }

    // Action button
    let buttons = extract_clean_buttons(node);
    if let Some(btn) = buttons.first() {
        config.insert("action_label".into(), btn.clone());
    }

    SectionBlueprint { section_type: "staff-notes".into(), confidence: 0.0, title, subtitle: None, config, items, template: None, style_block: None }
}

fn extract_settings_profile(node: &DomNode) -> SectionBlueprint {
    let mut items: Vec<ItemBlueprint> = Vec::new();
    let mut config: HashMap<String, String> = HashMap::new();
    let title = dom::find_heading(node);

    // Extract form fields
    let inputs = dom::find_by_tag(node, "input");
    for input in &inputs {
        let input_type = input.attrs.get("type").map(|s| s.as_str()).unwrap_or("text");
        let value = input.attrs.get("value").map(|s| s.as_str()).unwrap_or("");
        // Find label: look for nearby label/span with uppercase text
        let label = input.attrs.get("placeholder").cloned().unwrap_or_default();
        let mut item_config = HashMap::new();
        item_config.insert("type".into(), input_type.to_string());
        if !value.is_empty() { item_config.insert("value".into(), value.to_string()); }
        items.push(ItemBlueprint { item_type: "item".into(), title: if label.is_empty() { input_type.to_string() } else { label }, description: None, config: item_config });
    }

    // Note text (italic/small)
    let italics = dom::find_by_class(node, "italic");
    for it in &italics {
        let text = dom::clean_node_text(it).trim().to_string();
        if !text.is_empty() && text.len() > 10 {
            config.insert("note".into(), text);
            break;
        }
    }

    // Save button
    let buttons = extract_clean_buttons(node);
    if let Some(btn) = buttons.first() {
        config.insert("action_label".into(), btn.clone());
    }

    SectionBlueprint { section_type: "settings-profile".into(), confidence: 0.0, title, subtitle: None, config, items, template: None, style_block: None }
}

fn extract_api_keys(node: &DomNode) -> SectionBlueprint {
    let mut items: Vec<ItemBlueprint> = Vec::new();
    let mut config: HashMap<String, String> = HashMap::new();
    let title = dom::find_heading(node);
    let subtitle = dom::find_paragraph(node);

    // Find monospace key strings
    let mono_nodes = dom::find_by_class(node, "font-mono");
    for mn in &mono_nodes {
        let text = dom::clean_node_text(mn).trim().to_string();
        if text.len() > 5 && (text.contains("_") || text.contains("...") || text.contains("sk_") || text.contains("pk_")) {
            let mut item_config = HashMap::new();
            // Find icon nearby
            let parent = find_parent_of(node, mn);
            if let Some(p) = parent {
                let icons = dom::find_by_class(p, "material-symbols");
                for icon in &icons {
                    let it = dom::clean_node_text(icon).trim().to_string();
                    if !it.is_empty() { item_config.insert("icon".into(), it); break; }
                }
                // Find subtitle (small text)
                for t in collect_all_text_nodes(p) {
                    let trimmed = t.trim();
                    if trimmed != text && trimmed.len() > 5 && trimmed.len() < 60 && !trimmed.contains("copy") && !trimmed.contains("delete") {
                        item_config.insert("subtitle".into(), trimmed.to_string());
                        break;
                    }
                }
            }
            items.push(ItemBlueprint { item_type: "item".into(), title: text, description: None, config: item_config });
        }
    }

    // Generate button
    let buttons = extract_clean_buttons(node);
    if let Some(btn) = buttons.first() {
        config.insert("action_label".into(), btn.clone());
    }

    SectionBlueprint { section_type: "api-keys".into(), confidence: 0.0, title, subtitle, config, items, template: None, style_block: None }
}

fn extract_subscription_card(node: &DomNode) -> SectionBlueprint {
    let mut items: Vec<ItemBlueprint> = Vec::new();
    let mut config: HashMap<String, String> = HashMap::new();

    // Plan name from badge-like element
    let badge_nodes = dom::find_by_class(node, "uppercase");
    for bn in &badge_nodes {
        let txt = dom::clean_node_text(bn).trim().to_string();
        if !txt.is_empty() && txt.len() < 20 {
            let lower = txt.to_lowercase();
            if lower == "enterprise" || lower == "pro" || lower == "premium" || lower == "starter" || lower == "free" {
                config.insert("title_override".into(), txt);
                break;
            }
        }
    }

    let title = dom::find_heading(node);
    let subtitle = dom::find_paragraph(node);

    // Large price
    let all_text = collect_all_text_nodes(node);
    for t in &all_text {
        let trimmed = t.trim();
        if trimmed.starts_with('$') && trimmed.len() > 3 {
            let mut item_config = HashMap::new();
            item_config.insert("badge".into(), config.get("title_override").cloned().unwrap_or_default());
            items.push(ItemBlueprint { item_type: "item".into(), title: trimmed.to_string(), description: None, config: item_config });
            break;
        }
    }

    // Check-circle features
    for t in &all_text {
        let trimmed = t.trim();
        if trimmed.len() > 3 && trimmed.len() < 50 && !trimmed.starts_with('$') && !trimmed.to_lowercase().contains("billing") && !trimmed.to_lowercase().contains("monthly") {
            // Check if nearby a check_circle icon
            let lower = trimmed.to_lowercase();
            if lower != "enterprise" && lower != "pro" && lower != "premium" {
                let mut item_config = HashMap::new();
                item_config.insert("icon".into(), "check_circle".into());
                // Only add if it looks like a feature (not a date or ID)
                if !trimmed.contains("ID:") && !trimmed.contains("202") {
                    items.push(ItemBlueprint { item_type: "item".into(), title: trimmed.to_string(), description: None, config: item_config });
                }
            }
        }
    }

    // Manage button
    let buttons = extract_clean_buttons(node);
    if let Some(btn) = buttons.first() {
        config.insert("action_label".into(), btn.clone());
    }

    SectionBlueprint { section_type: "subscription-card".into(), confidence: 0.0, title, subtitle, config, items, template: None, style_block: None }
}

fn extract_danger_zone(node: &DomNode) -> SectionBlueprint {
    let mut config: HashMap<String, String> = HashMap::new();
    let mut items: Vec<ItemBlueprint> = Vec::new();
    let title = dom::find_heading(node);

    // Second heading (action title)
    let h4s = dom::find_by_tag(node, "h4");
    if let Some(h4) = h4s.first() {
        let t = dom::clean_node_text(h4).trim().to_string();
        if !t.is_empty() { config.insert("subtitle_text".into(), t); }
    }

    // Description
    let ps = dom::find_by_tag(node, "p");
    for p in &ps {
        let pt = dom::clean_node_text(p).trim().to_string();
        if pt.len() > 20 {
            items.push(ItemBlueprint { item_type: "item".into(), title: pt, description: None, config: HashMap::new() });
            break;
        }
    }

    // Destructive action button
    let buttons = extract_clean_buttons(node);
    if let Some(btn) = buttons.first() {
        config.insert("action_label".into(), btn.clone());
    }

    SectionBlueprint { section_type: "danger-zone".into(), confidence: 0.0, title, subtitle: config.remove("subtitle_text"), config, items, template: None, style_block: None }
}

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

    // 5b. Labeled progress bars (label + value + bar)
    if items.iter().all(|i| i.item_type != "meter") {
        let labeled_progress = extract_labeled_progress_bars(node);
        if !labeled_progress.is_empty() {
            items.extend(labeled_progress);
        }
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

    // 7b. Inline badges: small bold uppercase text that acts as a tag/label
    // (e.g. "Active Plan" badge in billing page)
    let inline_badge_items = extract_inline_badges(node);
    if !inline_badge_items.is_empty() {
        items.extend(inline_badge_items);
    }

    // 8. Code/mono text blocks
    let code_items = extract_code_items(node);
    if !code_items.is_empty() {
        items.extend(code_items);
    }

    // 8b. Key-value rows (billing info rows)
    let kv_items = extract_key_value_rows(node);
    if !kv_items.is_empty() {
        items.extend(kv_items);
    }

    // 8c. Invoice rows (repeated date + amount entries)
    let invoice_items = extract_invoice_rows(node);
    if !invoice_items.is_empty() {
        items.extend(invoice_items);
    }

    // 8d. Payment method rows
    let payment_items = extract_payment_method_rows(node);
    if !payment_items.is_empty() {
        items.extend(payment_items);
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
        template: None,
        style_block: None,
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

        // Key-value rows: flex justify-between with two text elements (billing info)
        // e.g. "Billing cycle" → "Annual (Renews Oct 24, 2024)"
        let kv_items = extract_key_value_rows(card);
        for kv in kv_items {
            let mut kv_with_role = kv;
            kv_with_role.config.insert("role".into(), "child".into());
            items.push(kv_with_role);
        }

        // Progress bars inside card (usage status)
        let progress_items = extract_labeled_progress_bars(card);
        for pi in progress_items {
            let mut pi_with_role = pi;
            pi_with_role.config.insert("role".into(), "child".into());
            items.push(pi_with_role);
        }

        // Invoice/list rows: repeated flex justify-between entries with date + amount
        let invoice_items = extract_invoice_rows(card);
        for inv in invoice_items {
            let mut inv_with_role = inv;
            inv_with_role.config.insert("role".into(), "child".into());
            items.push(inv_with_role);
        }

        // Payment method rows: flex items-center justify-between with card info
        let payment_items = extract_payment_method_rows(card);
        for pm in payment_items {
            let mut pm_with_role = pm;
            pm_with_role.config.insert("role".into(), "child".into());
            items.push(pm_with_role);
        }

        // Links inside card (e.g. "View detailed analytics")
        let card_links = dom::extract_links(card);
        for (text, href) in &card_links {
            if !text.is_empty() {
                let mut link_config: HashMap<String, String> = HashMap::new();
                link_config.insert("role".into(), "child".into());
                link_config.insert("href".into(), href.clone());
                items.push(ItemBlueprint {
                    item_type: "link".into(),
                    title: text.clone(),
                    description: None,
                    config: link_config,
                });
            }
        }
    }

    items
}

/// Extract key-value rows from a card: flex justify-between containers with
/// a label on the left and a value on the right, separated by a border-b.
/// Pattern: "Billing cycle" → "Annual (Renews Oct 24, 2024)"
fn extract_key_value_rows(node: &DomNode) -> Vec<ItemBlueprint> {
    let mut items = Vec::new();

    let jb_nodes = dom::find_by_class(node, "justify-between");
    for jb in &jb_nodes {
        // Skip if it's a button row or very short content
        if jb.tag == "button" || jb.tag == "a" {
            continue;
        }
        // Key-value rows typically have border-b or are in a space-y container
        let has_border = dom::has_class(jb, "border-b")
            || jb.classes.iter().any(|c| c.contains("border-"));

        if !has_border && !dom::has_class(jb, "items-end") && !dom::has_class(jb, "items-center") {
            continue;
        }

        // Need exactly 2 meaningful text children (key and value)
        let children_with_text: Vec<&DomNode> = jb.children.iter()
            .filter(|c| !c.full_text.trim().is_empty())
            .collect();

        if children_with_text.len() != 2 {
            continue;
        }

        let key_text = dom::clean_node_text(children_with_text[0]);
        let val_text = dom::clean_node_text(children_with_text[1]);

        if key_text.is_empty() || val_text.is_empty() {
            continue;
        }
        // Skip if key is too long (not a label)
        if key_text.len() > 60 {
            continue;
        }

        let mut config: HashMap<String, String> = HashMap::new();
        config.insert("_type".into(), "kv-row".into());
        config.insert("value".into(), val_text.clone());

        items.push(ItemBlueprint {
            item_type: "kv-row".into(),
            title: key_text,
            description: Some(val_text),
            config,
        });
    }

    items
}

/// Extract labeled progress bars: a label + value header followed by a progress bar track.
/// Pattern: "API Requests" 8.4M/10M with w-[84%] bar
fn extract_labeled_progress_bars(node: &DomNode) -> Vec<ItemBlueprint> {
    let mut items = Vec::new();

    // Find progress bar tracks: rounded div with bg-surface/bg-gray containing a w-[%] child
    find_labeled_progress_recursive(node, &mut items);

    items
}

fn find_labeled_progress_recursive(node: &DomNode, items: &mut Vec<ItemBlueprint>) {
    for (i, child) in node.children.iter().enumerate() {
        // Check if this child is a progress bar track (h-1.5 or h-2 with rounded + overflow)
        let is_track = (dom::has_class(child, "rounded-full") || dom::has_class(child, "rounded"))
            && (dom::has_class(child, "overflow-hidden") || dom::has_class(child, "h-1")
                || child.classes.iter().any(|c| c.starts_with("h-1") || c.starts_with("h-2")))
            && (dom::has_class(child, "bg-surface") || dom::has_class(child, "bg-gray")
                || dom::has_class(child, "bg-zinc") || dom::has_class(child, "bg-neutral")
                || dom::has_class(child, "bg-muted"));

        if !is_track {
            find_labeled_progress_recursive(child, items);
            continue;
        }

        // Extract percentage from inner bar's w-[XX%] class
        let mut pct = String::new();
        for bar_child in &child.children {
            for cls in &bar_child.classes {
                if cls.starts_with("w-[") && cls.contains('%') {
                    pct = cls.trim_start_matches("w-[").trim_end_matches(']').to_string();
                    break;
                }
            }
        }

        // Look for the label/value pair in the preceding sibling
        let mut label = String::new();
        let mut value = String::new();
        if i > 0 {
            let prev = &node.children[i - 1];
            // The preceding div typically has justify-between with label + value
            if dom::has_class(prev, "justify-between") {
                let texts: Vec<String> = prev.children.iter()
                    .map(|c| c.full_text.trim().to_string())
                    .filter(|t| !t.is_empty())
                    .collect();
                if texts.len() >= 2 {
                    label = texts[0].clone();
                    value = texts[1].clone();
                } else if texts.len() == 1 {
                    label = texts[0].clone();
                }
            } else {
                label = dom::clean_node_text(prev);
            }
        }

        let mut config: HashMap<String, String> = HashMap::new();
        config.insert("_type".into(), "meter".into());
        if !pct.is_empty() {
            config.insert("progress".into(), pct.clone());
        }
        if !value.is_empty() {
            config.insert("value".into(), value.clone());
        }

        let desc = if !pct.is_empty() && !value.is_empty() {
            Some(format!("{} ({})", value, pct))
        } else if !pct.is_empty() {
            Some(pct)
        } else if !value.is_empty() {
            Some(value)
        } else {
            None
        };

        items.push(ItemBlueprint {
            item_type: "meter".into(),
            title: label,
            description: desc,
            config,
        });
    }
}

/// Extract invoice-like rows: repeated flex justify-between entries inside a space-y container.
/// Each row has a title (SEP-2023), date subtitle, and amount ($2,400.00).
fn extract_invoice_rows(node: &DomNode) -> Vec<ItemBlueprint> {
    let mut items = Vec::new();

    // Look for space-y containers with multiple justify-between children
    let spacey_nodes = dom::find_by_class(node, "space-y-");
    let containers: Vec<&DomNode> = if spacey_nodes.is_empty() {
        // Fallback: check direct children
        vec![node]
    } else {
        spacey_nodes
    };

    for container in containers {
        let jb_children: Vec<&DomNode> = container.children.iter()
            .filter(|c| dom::has_class(c, "justify-between") && c.tag != "button")
            .collect();

        // Need at least 2 repeated rows to qualify as a list
        if jb_children.len() < 2 {
            continue;
        }

        // Check if rows look like invoices (have $ amounts or dates)
        let mut invoice_count = 0;
        for row in &jb_children {
            let text = row.full_text.to_string();
            if text.contains('$') || text.contains("20") {
                invoice_count += 1;
            }
        }
        if invoice_count < 2 {
            continue;
        }

        for row in &jb_children {
            // Left side: title + subtitle
            // Right side: amount + icon
            let children_with_text: Vec<&DomNode> = row.children.iter()
                .filter(|c| !c.full_text.trim().is_empty())
                .collect();

            if children_with_text.is_empty() {
                continue;
            }

            let mut title = String::new();
            let mut subtitle = None;
            let mut amount = None;

            // Left side: usually first child with bold title + small date
            if let Some(left) = children_with_text.first() {
                let bold = dom::find_by_class(left, "font-bold");
                if let Some(b) = bold.first() {
                    title = b.full_text.trim().to_string();
                }
                // Date/subtitle: text-xs or text-secondary child
                let small = dom::find_by_class(left, "text-xs");
                if let Some(s) = small.first() {
                    let txt = s.full_text.trim().to_string();
                    if txt != title {
                        subtitle = Some(txt);
                    }
                }
                if title.is_empty() {
                    title = dom::clean_node_text(left);
                }
            }

            // Right side: amount (contains $)
            if children_with_text.len() >= 2 {
                let right = children_with_text.last().unwrap();
                let bold = dom::find_by_class(right, "font-bold");
                for b in &bold {
                    let txt = b.full_text.trim().to_string();
                    if txt.contains('$') {
                        amount = Some(txt);
                        break;
                    }
                }
                if amount.is_none() {
                    let txt = right.full_text.trim().to_string();
                    if txt.contains('$') {
                        amount = Some(txt);
                    }
                }
            }

            if title.is_empty() {
                continue;
            }

            let mut config: HashMap<String, String> = HashMap::new();
            config.insert("_type".into(), "invoice-row".into());
            if let Some(ref a) = amount {
                config.insert("amount".into(), a.clone());
            }

            items.push(ItemBlueprint {
                item_type: "invoice-row".into(),
                title,
                description: subtitle.or(amount),
                config,
            });
        }
    }

    items
}

/// Extract payment method rows: flex items-center justify-between with card icon + info.
/// Pattern: VISA icon + "Visa ending in 4242" + "Default" badge
fn extract_payment_method_rows(node: &DomNode) -> Vec<ItemBlueprint> {
    let mut items = Vec::new();

    // Find rows with justify-between that contain card-like info
    let jb_nodes = dom::find_by_class(node, "justify-between");
    for row in &jb_nodes {
        if row.tag == "button" || row.tag == "a" {
            continue;
        }

        let text = row.full_text.to_string();
        let lower = text.to_lowercase();

        // Must look like a payment method (card brand names, "ending in", "pay")
        let is_payment = lower.contains("visa") || lower.contains("mastercard")
            || lower.contains("amex") || lower.contains("apple pay")
            || lower.contains("google pay") || lower.contains("ending in")
            || lower.contains("paypal") || lower.contains("expires");

        if !is_payment {
            continue;
        }

        // Extract the card name (bold text)
        let bold = dom::find_by_class(row, "font-bold");
        let card_name = bold.iter()
            .map(|b| b.full_text.trim().to_string())
            .find(|t| !t.is_empty() && t.len() < 60 && !is_material_icon_text(t))
            .unwrap_or_default();

        if card_name.is_empty() {
            continue;
        }

        let mut config: HashMap<String, String> = HashMap::new();
        config.insert("_type".into(), "payment-method".into());

        // Check for "Default" badge
        if lower.contains("default") {
            config.insert("default".into(), "true".into());
        }

        // Extract subtitle (expiry, added date)
        let small = dom::find_by_class(row, "text-xs");
        let subtitle = small.iter()
            .map(|s| s.full_text.trim().to_string())
            .find(|t| !t.is_empty() && t.len() < 80);

        items.push(ItemBlueprint {
            item_type: "payment-method".into(),
            title: card_name,
            description: subtitle,
            config,
        });
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

/// Extract inline badge labels: small bold uppercase text blocks (bg-black text-white)
/// that act as status tags. E.g. "Active Plan" badge in billing pages.
fn extract_inline_badges(node: &DomNode) -> Vec<ItemBlueprint> {
    let mut items = Vec::new();

    // Look for small span/div with uppercase + font-bold + bg-* (colored inline badges)
    let uppercase_nodes = dom::find_by_class(node, "uppercase");
    for badge in &uppercase_nodes {
        // Must be small text
        let is_small = dom::has_class(badge, "text-xs") || dom::has_class(badge, "text-sm")
            || badge.classes.iter().any(|c| c.contains("text-[10px]") || c.contains("text-[11px]"));
        if !is_small {
            continue;
        }
        // Must be bold
        if !dom::has_class(badge, "font-bold") && !dom::has_class(badge, "font-semibold") {
            continue;
        }
        // Must have a bg color (not just text)
        let has_bg = badge.classes.iter().any(|c| c.starts_with("bg-"));
        if !has_bg {
            continue;
        }
        // Skip if it's a rounded-full (already captured by extract_badge_items)
        if dom::has_class(badge, "rounded-full") {
            continue;
        }

        let txt = badge.full_text.trim().to_string();
        if txt.is_empty() || txt.len() > 40 {
            continue;
        }
        // Skip material icons
        if is_material_icon_span(badge) || is_material_icon_text(&txt) {
            continue;
        }

        let mut config: HashMap<String, String> = HashMap::new();
        config.insert("_type".into(), "badge".into());
        config.insert("style".into(), "inline".into());

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
    "code", "terminal", "data_object", "deployed_code", "content_copy", "content_paste",
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
/// Collect all visible text nodes from a DOM subtree (skipping icons)
fn collect_all_text_nodes(node: &DomNode) -> Vec<String> {
    let mut texts = Vec::new();
    if is_material_icon_span(node) { return texts; }
    let t = node.text.trim();
    if !t.is_empty() { texts.push(t.to_string()); }
    for child in &node.children {
        texts.extend(collect_all_text_nodes(child));
    }
    texts
}

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

// ---------------------------------------------------------------------------
// Extraction: KPI grid (dashboard metric cards)
// ---------------------------------------------------------------------------

fn extract_kpi_grid(node: &DomNode) -> SectionBlueprint {
    let mut items: Vec<ItemBlueprint> = Vec::new();
    let mut config: HashMap<String, String> = HashMap::new();

    // Detect grid columns from classes
    for cls in &node.classes {
        if cls.starts_with("grid-cols-") {
            let cols = cls.strip_prefix("grid-cols-").unwrap_or("4");
            config.insert("cols".into(), cols.to_string());
            break;
        }
    }
    // Also check one level deeper
    if !config.contains_key("cols") {
        for child in &node.children {
            for cls in &child.classes {
                if cls.starts_with("grid-cols-") {
                    let cols = cls.strip_prefix("grid-cols-").unwrap_or("4");
                    config.insert("cols".into(), cols.to_string());
                    break;
                }
            }
        }
    }

    // Find the grid container (may be node itself or a child)
    let grid_children: Vec<&DomNode> = if node.classes.iter().any(|c| c.starts_with("grid-cols") || c == "grid") {
        node.children.iter().collect()
    } else {
        let mut found: Vec<&DomNode> = Vec::new();
        for child in &node.children {
            if child.classes.iter().any(|c| c.starts_with("grid-cols") || c == "grid") {
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

    for card in &grid_children {
        let mut card_config: HashMap<String, String> = HashMap::new();

        // Detect col-span
        for cls in &card.classes {
            if cls.starts_with("col-span-") {
                let span = cls.strip_prefix("col-span-").unwrap_or("1");
                card_config.insert("span".into(), span.to_string());
                break;
            }
        }

        // Extract value: large text (text-2xl/3xl/4xl) or text with $ sign
        let value = find_kpi_value(card);
        // Extract label: small text (text-xs/text-sm/uppercase)
        let label = find_stat_label(card);
        // Extract badge: percentage text like +12.4%
        let badge = find_percentage_badge(card);
        // Extract subtitle: secondary small text
        let subtitle = find_kpi_subtitle(card);
        // Extract icon
        if let Some(icon) = extract_material_icon(card) {
            card_config.insert("icon".into(), icon);
        }

        if let Some(ref v) = value {
            card_config.insert("value".into(), v.clone());
        }
        if let Some(ref b) = badge {
            card_config.insert("badge".into(), b.clone());
        }
        if let Some(ref s) = subtitle {
            card_config.insert("subtitle".into(), s.clone());
        }

        // Detect mini chart (SVG or bar elements inside the card)
        let has_svg = dom::find_by_tag(card, "svg").len() > 0;
        let has_bars = card.children.iter().any(|c|
            c.classes.iter().any(|cls| cls.contains("bar") || cls.contains("chart"))
        );
        if has_svg || has_bars {
            card_config.insert("mini_chart".into(), "true".into());
        }

        let item_label = label.unwrap_or_default();
        if !item_label.is_empty() || value.is_some() {
            items.push(ItemBlueprint {
                item_type: "kpi".into(),
                title: item_label,
                description: value,
                config: card_config,
            });
        }
    }

    SectionBlueprint {
        section_type: "kpi-grid".into(),
        confidence: 0.0,
        title: None,
        subtitle: None,
        config,
        items,
        template: None,
        style_block: None,
    }
}

/// Find the main KPI value in a card (large number, often with $ or formatted).
fn find_kpi_value(node: &DomNode) -> Option<String> {
    // First try: text-3xl/4xl/5xl (the big number)
    for cls in &["text-4xl", "text-5xl", "text-3xl", "text-2xl"] {
        let matches = dom::find_by_class(node, cls);
        for m in matches {
            let txt = m.full_text.trim().to_string();
            if !txt.is_empty() && txt.len() < 40 {
                // Prefer values with digits
                if txt.chars().any(|c| c.is_ascii_digit()) {
                    return Some(txt);
                }
            }
        }
    }
    // Fallback: font-bold with digits
    let bold_matches = dom::find_by_class(node, "font-bold");
    for m in bold_matches {
        let txt = m.full_text.trim().to_string();
        if !txt.is_empty() && txt.len() < 40 && txt.chars().any(|c| c.is_ascii_digit()) {
            return Some(txt);
        }
    }
    // Fallback: font-semibold with digits
    let semi_matches = dom::find_by_class(node, "font-semibold");
    for m in semi_matches {
        let txt = m.full_text.trim().to_string();
        if !txt.is_empty() && txt.len() < 40 && txt.chars().any(|c| c.is_ascii_digit()) {
            return Some(txt);
        }
    }
    None
}

/// Find percentage badge text like "+12.4%" or "-3.2%" in a node.
fn find_percentage_badge(node: &DomNode) -> Option<String> {
    let text = &node.full_text;
    // Scan for +X.X% or -X.X% patterns
    let bytes = text.as_bytes();
    let len = bytes.len();
    for i in 0..len {
        if bytes[i] == b'+' || bytes[i] == b'-' {
            let mut j = i + 1;
            while j < len && (bytes[j].is_ascii_digit() || bytes[j] == b'.' || bytes[j] == b',') {
                j += 1;
            }
            if j > i + 1 && j < len && bytes[j] == b'%' {
                let badge = &text[i..=j];
                return Some(badge.to_string());
            }
        }
    }
    // Also check children with badge-like classes
    for child in &node.children {
        if child.classes.iter().any(|c| c.contains("badge") || c.contains("pill") || c.contains("tag")) {
            let txt = child.full_text.trim().to_string();
            if txt.contains('%') {
                return Some(txt);
            }
        }
        if let Some(b) = find_percentage_badge(child) {
            return Some(b);
        }
    }
    None
}

/// Find a KPI subtitle (secondary text below the value, often text-xs/text-sm).
fn find_kpi_subtitle(node: &DomNode) -> Option<String> {
    // Look for text-xs or text-sm that isn't the label
    let mut candidates: Vec<String> = Vec::new();
    for cls in &["text-xs", "text-sm"] {
        let matches = dom::find_by_class(node, cls);
        for m in matches {
            if is_material_icon_span(m) { continue; }
            let txt = m.full_text.trim().to_string();
            if !txt.is_empty() && txt.len() < 80 && txt.len() > 3 {
                // Skip if it's purely numeric (that's a value, not subtitle)
                if txt.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '%' || c == '$' || c == ',') {
                    continue;
                }
                // Skip if it's uppercase and short (that's a label)
                if dom::has_class(m, "uppercase") && txt.len() < 25 {
                    continue;
                }
                candidates.push(txt);
            }
        }
    }
    // Return the longest candidate (likely the subtitle, not a label)
    candidates.sort_by(|a, b| b.len().cmp(&a.len()));
    candidates.into_iter().next()
}

// ---------------------------------------------------------------------------
// Extraction: chart section
// ---------------------------------------------------------------------------

fn extract_chart(node: &DomNode) -> SectionBlueprint {
    let mut config: HashMap<String, String> = HashMap::new();
    let mut items: Vec<ItemBlueprint> = Vec::new();

    let title = dom::find_heading(node);
    let subtitle = dom::find_paragraph(node);

    // Detect chart type from classes
    let all_classes: Vec<String> = collect_all_classes(node);
    let chart_type = if all_classes.iter().any(|c| c.contains("area")) {
        "area"
    } else if all_classes.iter().any(|c| c.contains("bar-chart") || c.contains("bar_chart")) {
        "bar"
    } else if all_classes.iter().any(|c| c.contains("line")) {
        "line"
    } else if all_classes.iter().any(|c| c.contains("pie") || c.contains("donut")) {
        "pie"
    } else {
        "area" // default
    };
    config.insert("chart_type".into(), chart_type.into());

    // Extract period selector buttons
    let buttons = dom::extract_buttons(node);
    let links = dom::extract_links(node);
    let mut periods: Vec<String> = Vec::new();
    let period_keywords = ["7d", "30d", "90d", "1y", "12m", "6m", "3m", "1m",
        "week", "month", "year", "quarter", "daily", "weekly", "monthly"];
    for btn_text in &buttons {
        let lower = btn_text.to_lowercase();
        if period_keywords.iter().any(|kw| lower.contains(kw)) {
            periods.push(btn_text.clone());
        }
    }
    for (link_text, _) in &links {
        let lower = link_text.to_lowercase();
        if period_keywords.iter().any(|kw| lower.contains(kw)) {
            periods.push(link_text.clone());
        }
    }
    if !periods.is_empty() {
        config.insert("periods".into(), periods.join(" | "));
    }

    // Extract axis labels (look for a row of short text elements — typically flex children)
    let axis_labels = extract_axis_labels(node);
    if !axis_labels.is_empty() {
        config.insert("x_axis".into(), axis_labels.join(" | "));
    }

    // Extract legend items (colored indicators + text)
    let legend_items = extract_legend_items(node);
    for legend in legend_items {
        items.push(legend);
    }

    SectionBlueprint {
        section_type: "chart".into(),
        confidence: 0.0,
        title,
        subtitle,
        config,
        items,
        template: None,
        style_block: None,
    }
}

/// Collect all classes from a node and its descendants.
fn collect_all_classes(node: &DomNode) -> Vec<String> {
    let mut classes = node.classes.clone();
    for child in &node.children {
        classes.extend(collect_all_classes(child));
    }
    classes
}

/// Extract x-axis labels from a chart: typically a flex/grid row of short text elements.
fn extract_axis_labels(node: &DomNode) -> Vec<String> {
    // Look for a child div with flex that contains multiple short text elements
    for child in &node.children {
        if child.classes.iter().any(|c| c == "flex" || c.contains("justify-between") || c.contains("gap-")) {
            let labels: Vec<String> = child.children.iter()
                .filter_map(|c| {
                    let txt = c.full_text.trim().to_string();
                    if !txt.is_empty() && txt.len() < 15 {
                        Some(txt)
                    } else {
                        None
                    }
                })
                .collect();
            if labels.len() >= 3 {
                return labels;
            }
        }
        // Recurse
        let sub = extract_axis_labels(child);
        if !sub.is_empty() {
            return sub;
        }
    }
    Vec::new()
}

/// Extract legend items (colored dot/line + label text).
fn extract_legend_items(node: &DomNode) -> Vec<ItemBlueprint> {
    let mut legends = Vec::new();
    for child in &node.children {
        // Legend items typically have a small colored indicator + text
        if child.classes.iter().any(|c| c.contains("legend") || c.contains("indicator")) {
            let txt = child.full_text.trim().to_string();
            if !txt.is_empty() {
                let mut cfg = HashMap::new();
                if let Some(color) = detect_text_color(child) {
                    cfg.insert("color".into(), color);
                }
                legends.push(ItemBlueprint {
                    item_type: "legend".into(),
                    title: txt,
                    description: None,
                    config: cfg,
                });
            }
        }
        legends.extend(extract_legend_items(child));
    }
    legends
}

// ---------------------------------------------------------------------------
// Extraction: data table (with headers, rows, search, badges)
// ---------------------------------------------------------------------------

fn extract_data_table(node: &DomNode) -> SectionBlueprint {
    let mut items: Vec<ItemBlueprint> = Vec::new();
    let mut config: HashMap<String, String> = HashMap::new();

    // Title from heading
    let title = dom::find_heading(node);
    let subtitle = dom::find_paragraph(node);

    // Search input detection
    let inputs = dom::find_by_tag(node, "input");
    for input in &inputs {
        let input_type = input.attrs.get("type").map(|s| s.as_str()).unwrap_or("text");
        let placeholder = input.attrs.get("placeholder").map(|s| s.as_str()).unwrap_or("");
        if input_type == "search" || placeholder.to_lowercase().contains("search") {
            config.insert("search".into(), "true".into());
            if !placeholder.is_empty() {
                config.insert("search_placeholder".into(), placeholder.to_string());
            }
            break;
        }
    }

    // Extract table data using existing helper
    let (table_caption, table_items) = extract_table_items(node);
    if let Some(cap) = table_caption {
        config.insert("caption".into(), cap);
    }
    items.extend(table_items);

    // Detect "View All" or similar footer link
    let links = dom::extract_links(node);
    for (text, href) in &links {
        let lower = text.to_lowercase();
        if lower.contains("view all") || lower.contains("ver todos") || lower.contains("see all")
            || lower.contains("show all") || lower.contains("more")
        {
            config.insert("footer_link".into(), text.clone());
            config.insert("footer_href".into(), href.clone());
            break;
        }
    }

    // Extract column names from config (already set by extract_table_items via header item)
    if let Some(header_item) = items.iter().find(|i| i.item_type == "table-header") {
        let cols = header_item.title.clone();
        config.insert("columns".into(), cols);
    }

    // Detect status badges in table rows and enrich row items
    for item in &mut items {
        if item.item_type == "row" {
            // Check row values for status-like text
            for (key, val) in item.config.clone() {
                let lower = val.to_lowercase();
                if lower == "active" || lower == "completed" || lower == "pending"
                    || lower == "failed" || lower == "inactive" || lower == "cancelled"
                    || lower == "approved" || lower == "rejected" || lower == "draft"
                {
                    item.config.insert(format!("{}_status", key), "badge".into());
                }
            }
        }
    }

    SectionBlueprint {
        section_type: "data-table".into(),
        confidence: 0.0,
        title,
        subtitle,
        config,
        items,
        template: None,
        style_block: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hero_extraction_developer_landing() {
        let html = std::fs::read_to_string(
            "/home/zedd/Downloads/stitch_performance_dashboard(1) (2)/stitch_performance_dashboard/developer_focused_landing_page/code.html"
        ).unwrap();

        let nodes = dom::parse_html(&html);
        let sections = detect_sections(&nodes);

        // Find the hero section
        let hero = sections.iter().find(|s| s.section_type == "hero");
        assert!(hero.is_some(), "Hero section not found. Sections: {:?}", sections.iter().map(|s| &s.section_type).collect::<Vec<_>>());
        let hero = hero.unwrap();

        // A) Badge
        let badge = hero.config.get("badge");
        assert!(badge.is_some(), "Badge not found in hero config. Config: {:?}", hero.config);
        let badge_text = badge.unwrap();
        assert!(badge_text.contains("Next.js 15"), "Badge should contain 'Next.js 15', got: {}", badge_text);

        // B) Terminal lines
        let lines: Vec<&ItemBlueprint> = hero.items.iter().filter(|i| i.item_type == "line").collect();
        assert!(!lines.is_empty(), "No terminal 'line' items found. Items: {:?}", hero.items.iter().map(|i| (&i.item_type, &i.title)).collect::<Vec<_>>());
        assert!(lines.iter().any(|l| l.title.contains("npm i -g vercel")), "Missing 'npm i -g vercel' line");
        assert!(lines.iter().any(|l| l.title.contains("vercel deploy")), "Missing 'vercel deploy' line");

        let outputs: Vec<&ItemBlueprint> = hero.items.iter().filter(|i| i.item_type == "output").collect();
        assert!(outputs.iter().any(|o| o.title.contains("Vercel CLI")), "Missing 'Vercel CLI' output");

        let prompts: Vec<&ItemBlueprint> = hero.items.iter().filter(|i| i.item_type == "prompt").collect();
        assert!(!prompts.is_empty(), "No prompt items found");
        // Check prompt has answer
        assert!(prompts.iter().any(|p| p.config.get("answer").is_some()), "Prompt should have answer config");

        let successes: Vec<&ItemBlueprint> = hero.items.iter().filter(|i| i.item_type == "success").collect();
        assert!(successes.len() >= 2, "Expected at least 2 success lines, got {}", successes.len());

        // C) Floating chips
        let chips: Vec<&ItemBlueprint> = hero.items.iter().filter(|i| i.item_type == "chip").collect();
        assert!(chips.len() >= 2, "Expected at least 2 chips, got {}. Items: {:?}",
            chips.len(), hero.items.iter().map(|i| (&i.item_type, &i.title)).collect::<Vec<_>>());

        // Check chip content — title should NOT contain icon text
        let next_chip = chips.iter().find(|c| c.title.contains("Next.js")).expect("Missing Next.js chip");
        assert!(!next_chip.title.starts_with("NEXT"), "Chip title should not start with icon text, got: {}", next_chip.title);
        assert_eq!(next_chip.title, "Ready for Next.js");
        assert_eq!(next_chip.config.get("icon").map(|s| s.as_str()), Some("NEXT"));

        let sv_chip = chips.iter().find(|c| c.title.contains("SvelteKit")).expect("Missing SvelteKit chip");
        assert!(!sv_chip.title.starts_with("SV"), "Chip title should not start with icon text, got: {}", sv_chip.title);
        assert_eq!(sv_chip.title, "SvelteKit Support");
        assert_eq!(sv_chip.config.get("icon").map(|s| s.as_str()), Some("SV"));

        println!("=== HERO EXTRACTION TEST PASSED ===");
        println!("Badge: {}", badge_text);
        for item in &hero.items {
            println!("  [{}] {} {:?}", item.item_type, item.title, item.config);
        }
    }
}
