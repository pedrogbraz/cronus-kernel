#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS Dump — HTML→.cronus converter
//!
//! Analyzes HTML pages and generates .cronus files that describe
//! the same content using CRONUS language primitives.

pub mod dom;
pub mod detect;
pub mod patterns;
pub mod emit;
pub mod openapi;
pub mod prisma;
pub mod project;
pub mod routes;
pub mod style_extract;
pub mod typescript;

use std::collections::HashMap;

/// Main entry point: takes raw HTML, returns .cronus source
pub fn dump_html(html: &str) -> String {
    // 1. Parse DOM
    let nodes = dom::parse_html(html);

    // 2. Detect theme
    let theme = dom::detect_theme_with_html(html, &nodes);

    // 3. Extract app info
    let app_name = dom::extract_title(html);

    // 4. Detect accent color from HTML (inline SVG fills, style tags, etc.)
    let accent = detect_accent_from_html(html)
        .unwrap_or_else(|| if theme == "dark" { "blue".into() } else { "black".into() });

    // 5. Detect font from HTML
    let font = detect_font_from_html(html).unwrap_or_else(|| "Inter".into());

    // 6. Detect sections
    let sections = detect::detect_sections(&nodes);

    // 6b. Extract CSS custom properties (design tokens) from HTML
    let css_vars = extract_css_variables(html);

    // 7. Build CronusFile
    let file = emit::CronusFile {
        app_name,
        port: 5175,
        theme: theme.to_string(),
        style_config: {
            let mut m = HashMap::new();
            m.insert("font".into(), font);
            m.insert("accent".into(), accent);
            // Merge extracted CSS variables
            for (k, v) in css_vars {
                m.entry(k).or_insert(v);
            }
            m
        },
        sections,
    };

    // 8. Emit .cronus
    emit::emit_cronus(&file)
}

/// Known hex -> color name mappings (Tailwind palette)
fn hex_to_color_name(hex: &str) -> Option<&'static str> {
    let hex = hex.to_lowercase();
    match hex.as_str() {
        "#2563eb" | "#3b82f6" | "#1d4ed8" => Some("blue"),
        "#6366f1" | "#4f46e5" | "#818cf8" => Some("indigo"),
        "#8b5cf6" | "#7c3aed" | "#a78bfa" => Some("violet"),
        "#a855f7" | "#9333ea" | "#c084fc" => Some("purple"),
        "#ec4899" | "#db2777" | "#f472b6" => Some("pink"),
        "#ef4444" | "#dc2626" | "#f87171" => Some("red"),
        "#f97316" | "#ea580c" | "#fb923c" => Some("orange"),
        "#f59e0b" | "#d97706" | "#fbbf24" => Some("amber"),
        "#eab308" | "#ca8a04" | "#facc15" => Some("yellow"),
        "#84cc16" | "#65a30d" | "#a3e635" => Some("lime"),
        "#22c55e" | "#16a34a" | "#4ade80" => Some("green"),
        "#10b981" | "#059669" | "#34d399" => Some("emerald"),
        "#14b8a6" | "#0d9488" | "#2dd4bf" => Some("teal"),
        "#06b6d4" | "#0891b2" | "#22d3ee" => Some("cyan"),
        "#0ea5e9" | "#0284c7" | "#38bdf8" => Some("sky"),
        "#f43f5e" | "#e11d48" | "#fb7185" => Some("rose"),
        "#d946ef" | "#c026d3" | "#e879f9" => Some("fuchsia"),
        "#000" | "#000000" | "#0a0a0a" | "#09090b" => Some("black"),
        "#fff" | "#ffffff" | "#fafafa" => Some("white"),
        _ => None,
    }
}

/// Detect accent color from HTML content (SVG fills, strokes, CSS variables, inline styles)
fn detect_accent_from_html(html: &str) -> Option<String> {
    // 1. Check for CSS custom property --color-accent or similar
    // Look in <style> tags
    let style_start = html.find("<style");
    if let Some(start) = style_start {
        if let Some(end) = html[start..].find("</style>") {
            let style_block = &html[start..start+end];
            // Look for accent variable
            for line in style_block.lines() {
                let line = line.trim();
                if line.contains("accent") && line.contains(':') {
                    if let Some(after) = line.split(':').nth(1) {
                        let val = after.trim().trim_end_matches(';').trim();
                        if val.starts_with('#') {
                            if let Some(name) = hex_to_color_name(val) {
                                return Some(name.to_string());
                            }
                        }
                        // Return raw value if no match
                        if !val.is_empty() {
                            return Some(val.to_string());
                        }
                    }
                }
            }
        }
    }

    // 2. Count hex colors in SVG fill/stroke attributes and inline styles
    let mut color_counts: HashMap<String, u32> = HashMap::new();

    // Skip common neutral colors
    let neutral_colors = ["#000", "#000000", "#fff", "#ffffff", "#fafafa", "#f9f9f9",
        "#333", "#333333", "#666", "#666666", "#999", "#999999", "#ccc", "#cccccc",
        "#111", "#111111", "#222", "#222222", "#444", "#555", "#777", "#888", "#aaa", "#bbb", "#ddd", "#eee",
        "#0a0a0a", "#09090b", "#171717", "#262626", "#404040", "#525252", "#737373", "#a3a3a3", "#d4d4d4", "#e5e5e5", "#f5f5f5"];

    // Manual hex color extraction (no regex crate needed)
    let bytes = html.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'#' && i + 1 < bytes.len() && is_hex_char(bytes[i + 1]) {
            // Try to read 6-char hex
            let start = i;
            i += 1;
            let mut hex_len = 0;
            while i < bytes.len() && is_hex_char(bytes[i]) && hex_len < 6 {
                hex_len += 1;
                i += 1;
            }
            if hex_len == 6 || hex_len == 3 {
                let hex = html[start..start + 1 + hex_len].to_lowercase();
                if !neutral_colors.contains(&hex.as_str()) {
                    *color_counts.entry(hex).or_insert(0) += 1;
                }
            }
        } else {
            i += 1;
        }
    }

    // Find the most common non-neutral hex color
    if let Some((hex, _count)) = color_counts.into_iter().max_by_key(|(_, c)| *c) {
        if let Some(name) = hex_to_color_name(&hex) {
            return Some(name.to_string());
        }
        // If no named match, return the hex itself
        return Some(hex);
    }

    None
}

fn is_hex_char(b: u8) -> bool {
    b.is_ascii_hexdigit()
}

/// Detect font from HTML (Google Fonts links, font-family in style tags)
fn detect_font_from_html(html: &str) -> Option<String> {
    // Check for Google Fonts links
    if html.contains("fonts.googleapis.com") || html.contains("fonts.google") {
        if let Some(pos) = html.find("family=") {
            let after = &html[pos + 7..];
            let end = after.find(|c: char| c == '&' || c == '\'' || c == '"' || c == ')' || c == '>').unwrap_or(after.len());
            let family = after[..end].split(':').next().unwrap_or("").replace('+', " ");
            if !family.is_empty() {
                return Some(family);
            }
        }
    }
    None
}

/// Extract CSS custom properties (design tokens) from `<style>` tags in HTML.
/// Maps common CSS variable naming patterns to .cronus style keys.
/// Also detects inline style patterns on body/root elements.
fn extract_css_variables(html: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();

    // Phase 1: Find CSS custom properties (--name: value) in <style> blocks
    let mut search_from = 0;
    while let Some(start) = html[search_from..].find("<style") {
        let abs_start = search_from + start;
        if let Some(end) = html[abs_start..].find("</style>") {
            let style_block = &html[abs_start..abs_start + end];
            for line in style_block.lines() {
                let line = line.trim();
                if line.starts_with("--") && line.contains(':') {
                    if let Some(colon) = line.find(':') {
                        let name = line[2..colon].trim().to_string();
                        let value = line[colon + 1..].trim().trim_end_matches(';').trim().to_string();
                        if value.is_empty() { continue; }
                        // Map CSS var names to .cronus style keys
                        let key = match name.as_str() {
                            "color-bg" => "background",
                            "color-surface" => "surface",
                            "color-text" => "text",
                            "color-text-muted" => "text-muted",
                            "color-accent" => "accent-hex",
                            "color-accent-hover" => "accent-hover",
                            "color-border" => "border",
                            "max-w" => "max-width",
                            "font" => "font",
                            "radius" => "radius",
                            _ => &name,
                        };
                        vars.insert(key.to_string(), value);
                    }
                }
            }
            search_from = abs_start + end + 8;
        } else {
            break;
        }
    }

    // Phase 2: Detect design tokens from inline styles on <body> and top-level elements
    // Look for body style="background:X; color:Y" patterns
    if vars.is_empty() {
        // Extract body background and color
        if let Some(body_pos) = html.find("<body") {
            if let Some(style_pos) = html[body_pos..body_pos + 200.min(html.len() - body_pos)].find("style=\"") {
                let style_start = body_pos + style_pos + 7;
                if let Some(style_end) = html[style_start..].find('"') {
                    let style_val = &html[style_start..style_start + style_end];
                    // Extract background
                    if let Some(bg) = extract_css_prop(style_val, "background") {
                        if bg.starts_with('#') && !is_neutral_dark(&bg) {
                            vars.insert("background".into(), bg);
                        }
                    }
                    // Extract color
                    if let Some(color) = extract_css_prop(style_val, "color") {
                        if color.starts_with('#') {
                            vars.insert("text".into(), color);
                        }
                    }
                }
            }
        }

        // Detect border-radius from the most common radius value
        if let Some(radius) = detect_common_radius(html) {
            vars.insert("radius".into(), radius);
        }

        // Detect max-width from the main container
        if let Some(max_w) = detect_max_width(html) {
            vars.insert("max-width".into(), max_w);
        }
    }

    vars
}

/// Extract a CSS property value from an inline style string
fn extract_css_prop(style: &str, prop: &str) -> Option<String> {
    let patterns = [
        format!("{}:", prop),
        format!("{}-color:", prop),
    ];
    for pat in &patterns {
        if let Some(pos) = style.find(pat.as_str()) {
            let after = &style[pos + pat.len()..];
            let end = after.find(|c: char| c == ';' || c == '"').unwrap_or(after.len());
            let val = after[..end].trim().to_string();
            if !val.is_empty() {
                return Some(val);
            }
        }
    }
    None
}

fn is_neutral_dark(hex: &str) -> bool {
    matches!(hex.to_lowercase().as_str(),
        "#000" | "#000000" | "#0a0a0a" | "#111" | "#111111" | "#1a1a1a" | "#222" | "#333")
}

/// Detect the most commonly used border-radius in the HTML
fn detect_common_radius(html: &str) -> Option<String> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    let needle = "border-radius:";
    let mut pos = 0;
    while let Some(found) = html[pos..].find(needle) {
        let abs = pos + found + needle.len();
        let after = &html[abs..];
        let end = after.find(|c: char| c == ';' || c == '"' || c == '}').unwrap_or(after.len());
        let val = after[..end].trim().to_string();
        if !val.is_empty() && val != "50%" && val != "999px" && val != "9999px" {
            *counts.entry(val).or_insert(0) += 1;
        }
        pos = abs + end;
    }
    counts.into_iter().max_by_key(|(_, c)| *c).map(|(v, _)| v)
}

/// Detect max-width from the main content container
fn detect_max_width(html: &str) -> Option<String> {
    let needle = "max-width:";
    let mut counts: HashMap<String, u32> = HashMap::new();
    let mut pos = 0;
    while let Some(found) = html[pos..].find(needle) {
        let abs = pos + found + needle.len();
        let after = &html[abs..];
        let end = after.find(|c: char| c == ';' || c == '"' || c == '}').unwrap_or(after.len());
        let val = after[..end].trim().to_string();
        if val.contains("px") && val != "480px" && val != "640px" {
            *counts.entry(val).or_insert(0) += 1;
        }
        pos = abs + end;
    }
    counts.into_iter().max_by_key(|(_, c)| *c).map(|(v, _)| v)
}
