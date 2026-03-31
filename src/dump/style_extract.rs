#![allow(dead_code, unused_imports, unused_variables)]
//! Style extraction for CRONUS Black Hole dump system
//!
//! Analyzes Tailwind config, CSS files, and source files to detect theme,
//! accent color, and font choices for .cronus style blocks.
//! v2: extracts from @theme CSS variables (Tailwind v4 style).

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Extract style block from Tailwind config and source files
pub fn extract_style(tailwind_config: &str, ts_files: &[PathBuf]) -> String {
    let theme = detect_theme(tailwind_config, ts_files);
    let accent = detect_accent_color(ts_files);
    let font = detect_font(tailwind_config, ts_files);

    build_style_block(&theme, &accent, &font)
}

/// Extract style block from CSS @theme variables (no tailwind config needed)
pub fn extract_style_from_css(css: &str, ts_files: &[PathBuf]) -> String {
    let theme = if css.contains("#000") || css.contains("bg: #0") || css.contains("background: #0") {
        "dark".to_string()
    } else {
        detect_theme("", ts_files)
    };

    // Extract accent from --color-*-accent or prominent color variable
    let accent = extract_accent_from_css(css)
        .unwrap_or_else(|| detect_accent_color(ts_files));

    // Extract font from CSS --font-sans or @import
    let font = extract_font_from_css(css)
        .unwrap_or_else(|| detect_font("", ts_files));

    build_style_block(&theme, &accent, &font)
}

fn build_style_block(theme: &str, accent: &str, font: &str) -> String {
    let mut style = String::from("style {\n");
    style.push_str(&format!("  theme {}\n", theme));
    style.push_str(&format!("  accent \"{}\"\n", accent));
    if !font.is_empty() {
        style.push_str(&format!("  font \"{}\"\n", font));
    }
    style.push_str("  radius lg\n");
    style.push_str("}\n");
    style
}

/// Extract accent color from CSS variables
fn extract_accent_from_css(css: &str) -> Option<String> {
    // Look for --color-*-accent: #HEXVAL or oklch(...)
    for line in css.lines() {
        let line = line.trim();
        if line.contains("accent") && line.contains(':') {
            let after_colon = line.split(':').nth(1)?.trim().trim_end_matches(';');
            if !after_colon.is_empty() {
                return Some(after_colon.to_string());
            }
        }
    }
    None
}

/// Extract font from CSS variables or @import
fn extract_font_from_css(css: &str) -> Option<String> {
    // Check --font-sans or --font-display
    for line in css.lines() {
        let line = line.trim();
        if (line.contains("--font-sans") || line.contains("--font-display")) && line.contains(':') {
            let after_colon = line.split(':').nth(1)?.trim().trim_end_matches(';');
            // Extract first font name from the font stack
            let first_font = after_colon
                .split(',')
                .next()?
                .trim()
                .trim_matches('\'')
                .trim_matches('"')
                .to_string();
            if !first_font.is_empty() && first_font != "system-ui" && first_font != "ui-monospace" {
                return Some(first_font);
            }
        }
    }

    // Check @import for Google Fonts
    for line in css.lines() {
        if line.contains("fonts.googleapis.com") || line.contains("fonts.google") {
            // Try to extract the first family name
            if let Some(family_pos) = line.find("family=") {
                let after = &line[family_pos + 7..];
                let end = after.find(|c: char| c == '&' || c == '\'' || c == '"' || c == ')').unwrap_or(after.len());
                let family = after[..end].split(':').next().unwrap_or("").replace('+', " ");
                if !family.is_empty() {
                    return Some(family);
                }
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Theme detection
// ---------------------------------------------------------------------------

fn detect_theme(config: &str, ts_files: &[PathBuf]) -> String {
    // Check tailwind config for darkMode
    if config.contains("darkMode") {
        return "dark".to_string();
    }

    // Count dark: vs light: classes in source files
    let mut dark_count = 0u32;
    let mut light_count = 0u32;

    for file in ts_files.iter().take(50) {
        if let Ok(source) = fs::read_to_string(file) {
            dark_count += source.matches("dark:").count() as u32;
            dark_count += source.matches("bg-neutral-9").count() as u32;
            dark_count += source.matches("bg-gray-9").count() as u32;
            dark_count += source.matches("bg-zinc-9").count() as u32;
            dark_count += source.matches("#0a0a0a").count() as u32;
            dark_count += source.matches("#050505").count() as u32;
            light_count += source.matches("bg-white").count() as u32;
            light_count += source.matches("bg-gray-50").count() as u32;
        }
    }

    if dark_count > light_count {
        "dark"
    } else {
        "light"
    }
    .to_string()
}

// ---------------------------------------------------------------------------
// Accent color detection
// ---------------------------------------------------------------------------

fn detect_accent_color(ts_files: &[PathBuf]) -> String {
    let mut color_counts: HashMap<String, u32> = HashMap::new();

    let colors = [
        "amber", "blue", "indigo", "emerald", "rose", "violet", "sky", "orange", "red", "green",
        "purple", "pink", "cyan", "teal", "lime", "yellow", "fuchsia",
    ];

    for file in ts_files.iter().take(100) {
        if let Ok(source) = fs::read_to_string(file) {
            for color in &colors {
                let count = source.matches(color).count() as u32;
                if count > 0 {
                    *color_counts.entry(color.to_string()).or_insert(0) += count;
                }
            }
        }
    }

    color_counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(color, _)| color)
        .unwrap_or_else(|| "indigo".to_string())
}

// ---------------------------------------------------------------------------
// Font detection
// ---------------------------------------------------------------------------

fn detect_font(config: &str, ts_files: &[PathBuf]) -> String {
    // Check tailwind config for font family
    if config.contains("Inter") {
        return "Inter".to_string();
    }
    if config.contains("Geist") {
        return "Geist".to_string();
    }
    if config.contains("Poppins") {
        return "Poppins".to_string();
    }
    if config.contains("Roboto") {
        return "Roboto".to_string();
    }

    // Check source for Google Fonts or next/font imports
    for file in ts_files.iter().take(20) {
        if let Ok(source) = fs::read_to_string(file) {
            if source.contains("Inter") {
                return "Inter".to_string();
            }
            if source.contains("Geist") {
                return "Geist".to_string();
            }
            if source.contains("fonts.google") {
                if source.contains("Poppins") {
                    return "Poppins".to_string();
                }
                if source.contains("Roboto") {
                    return "Roboto".to_string();
                }
                if source.contains("Open_Sans") {
                    return "Open Sans".to_string();
                }
            }
        }
    }

    "Inter".to_string() // default
}
