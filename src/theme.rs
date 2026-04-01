// theme.rs — Design system token extraction and CSS variable generation
//
// Extracts color/font tokens from tailwind_config and style blocks,
// generates CSS custom properties so all renderers inherit the design system.

use std::collections::HashMap;
use std::sync::RwLock;

/// Global theme tokens — set once at startup from tailwind_config/style
static THEME: RwLock<Option<ThemeTokens>> = RwLock::new(None);

#[derive(Debug, Clone)]
pub struct ThemeTokens {
    // Core surface colors
    pub background: String,
    pub surface: String,
    pub surface_container: String,
    pub surface_container_low: String,
    pub surface_container_lowest: String,
    pub surface_container_high: String,
    pub surface_container_highest: String,
    pub surface_bright: String,

    // Text colors
    pub on_surface: String,
    pub on_surface_variant: String,
    pub on_background: String,

    // Accent colors
    pub primary: String,
    pub secondary: String,
    pub tertiary: String,
    pub error: String,

    // Borders
    pub outline: String,
    pub outline_variant: String,

    // Additional
    pub secondary_container: String,
    pub error_container: String,
    pub on_error_container: String,

    // Fonts
    pub font_headline: String,
    pub font_body: String,
    pub font_label: String,

    // Raw tailwind config (for CDN injection)
    pub tailwind_config_js: Option<String>,
}

impl Default for ThemeTokens {
    fn default() -> Self {
        // Obsidian defaults (backward compatible)
        ThemeTokens {
            background: "#131313".into(),
            surface: "#131313".into(),
            surface_container: "#1f1f1f".into(),
            surface_container_low: "#1b1b1b".into(),
            surface_container_lowest: "#0e0e0e".into(),
            surface_container_high: "#2a2a2a".into(),
            surface_container_highest: "#353535".into(),
            surface_bright: "#393939".into(),
            on_surface: "#e2e2e2".into(),
            on_surface_variant: "#cfc4c5".into(),
            on_background: "#e2e2e2".into(),
            primary: "#adc6ff".into(),
            secondary: "#c2c1ff".into(),
            tertiary: "#e9b3ff".into(),
            error: "#ffb4ab".into(),
            outline: "#988e90".into(),
            outline_variant: "#4c4546".into(),
            secondary_container: "#3630bf".into(),
            error_container: "#93000a".into(),
            on_error_container: "#ffdad6".into(),
            font_headline: "Inter Display".into(),
            font_body: "Inter".into(),
            font_label: "Inter".into(),
            tailwind_config_js: None,
        }
    }
}

/// Parse tokens from a tailwind_config string (the raw JS object)
pub fn parse_from_tailwind_config(config_str: &str) -> ThemeTokens {
    let mut tokens = ThemeTokens::default();
    tokens.tailwind_config_js = Some(config_str.to_string());

    // Extract "key": "#hexval" pairs
    let mut colors: HashMap<String, String> = HashMap::new();
    let cleaned = config_str.replace("\\\"", "\"");
    let mut i = 0;
    let chars: Vec<char> = cleaned.chars().collect();
    while i < chars.len() {
        // Find "key": "#value"
        if chars[i] == '"' {
            let key_start = i + 1;
            i += 1;
            while i < chars.len() && chars[i] != '"' { i += 1; }
            let key_end = i;
            i += 1; // skip closing "
            // Skip whitespace and colon
            while i < chars.len() && (chars[i] == ' ' || chars[i] == ':') { i += 1; }
            // Check for "#hex"
            if i < chars.len() && chars[i] == '"' {
                let val_start = i + 1;
                i += 1;
                while i < chars.len() && chars[i] != '"' { i += 1; }
                let val_end = i;
                i += 1;
                let key: String = chars[key_start..key_end].iter().collect();
                let val: String = chars[val_start..val_end].iter().collect();
                if val.starts_with('#') {
                    colors.insert(key, val);
                }
            }
        } else {
            i += 1;
        }
    }

    // Map colors to token fields
    if let Some(v) = colors.get("background") { tokens.background = v.clone(); }
    if let Some(v) = colors.get("surface") { tokens.surface = v.clone(); }
    if let Some(v) = colors.get("surface-container") { tokens.surface_container = v.clone(); }
    if let Some(v) = colors.get("surface-container-low") { tokens.surface_container_low = v.clone(); }
    if let Some(v) = colors.get("surface-container-lowest") { tokens.surface_container_lowest = v.clone(); }
    if let Some(v) = colors.get("surface-container-high") { tokens.surface_container_high = v.clone(); }
    if let Some(v) = colors.get("surface-container-highest") { tokens.surface_container_highest = v.clone(); }
    if let Some(v) = colors.get("surface-bright") { tokens.surface_bright = v.clone(); }
    if let Some(v) = colors.get("on-surface") { tokens.on_surface = v.clone(); }
    if let Some(v) = colors.get("on-surface-variant") { tokens.on_surface_variant = v.clone(); }
    if let Some(v) = colors.get("on-background") { tokens.on_background = v.clone(); }
    if let Some(v) = colors.get("primary") { tokens.primary = v.clone(); }
    if let Some(v) = colors.get("secondary") { tokens.secondary = v.clone(); }
    if let Some(v) = colors.get("tertiary") { tokens.tertiary = v.clone(); }
    if let Some(v) = colors.get("error") { tokens.error = v.clone(); }
    if let Some(v) = colors.get("outline") { tokens.outline = v.clone(); }
    if let Some(v) = colors.get("outline-variant") { tokens.outline_variant = v.clone(); }
    if let Some(v) = colors.get("secondary-container") { tokens.secondary_container = v.clone(); }
    if let Some(v) = colors.get("error-container") { tokens.error_container = v.clone(); }
    if let Some(v) = colors.get("on-error-container") { tokens.on_error_container = v.clone(); }

    // Extract font families
    // Pattern: "headline": ["FontName"] or "body": ["FontName"]
    for (label, field) in [("headline", &mut tokens.font_headline), ("body", &mut tokens.font_body), ("label", &mut tokens.font_label)] {
        let pattern = format!("\"{}\"", label);
        if let Some(pos) = cleaned.find(&pattern) {
            let rest = &cleaned[pos..];
            if let Some(bracket_start) = rest.find('[') {
                if let Some(quote_start) = rest[bracket_start..].find('"') {
                    let font_start = bracket_start + quote_start + 1;
                    if let Some(quote_end) = rest[font_start..].find('"') {
                        *field = rest[font_start..font_start + quote_end].to_string();
                    }
                }
            }
        }
    }

    tokens
}

/// Parse from style node accent/font
pub fn parse_from_style(accent: &str, font: &str) -> ThemeTokens {
    let mut tokens = ThemeTokens::default();
    if !accent.is_empty() && accent.starts_with('#') {
        tokens.primary = accent.to_string();
    }
    if !font.is_empty() {
        tokens.font_headline = font.to_string();
        tokens.font_body = font.to_string();
    }
    tokens
}

/// Set the global theme tokens
pub fn set_global(tokens: ThemeTokens) {
    *THEME.write().unwrap() = Some(tokens);
}

/// Get current tokens (returns defaults if not set)
pub fn get() -> ThemeTokens {
    THEME.read().unwrap().clone().unwrap_or_default()
}

/// Generate CSS custom properties block from current theme
pub fn css_vars() -> String {
    let t = get();
    format!(r#":root {{
  --bg: {bg};
  --surface: {surface};
  --surface-container: {sc};
  --surface-container-low: {scl};
  --surface-container-lowest: {scll};
  --surface-container-high: {sch};
  --surface-container-highest: {schh};
  --surface-bright: {sb};
  --on-surface: {os};
  --on-surface-variant: {osv};
  --primary: {primary};
  --secondary: {secondary};
  --tertiary: {tertiary};
  --error: {error};
  --outline: {outline};
  --outline-variant: {ov};
  --secondary-container: {sec_c};
  --error-container: {err_c};
  --on-error-container: {oec};
  --font-headline: '{fh}', sans-serif;
  --font-body: '{fb}', sans-serif;
  --font-label: '{fl}', sans-serif;
}}"#,
        bg = t.background, surface = t.surface,
        sc = t.surface_container, scl = t.surface_container_low,
        scll = t.surface_container_lowest, sch = t.surface_container_high,
        schh = t.surface_container_highest, sb = t.surface_bright,
        os = t.on_surface, osv = t.on_surface_variant,
        primary = t.primary, secondary = t.secondary, tertiary = t.tertiary,
        error = t.error, outline = t.outline, ov = t.outline_variant,
        sec_c = t.secondary_container, err_c = t.error_container, oec = t.on_error_container,
        fh = t.font_headline, fb = t.font_body, fl = t.font_label,
    )
}

/// Generate the Tailwind CDN script + config if available
pub fn tailwind_cdn_script() -> String {
    let t = get();
    if let Some(ref config) = t.tailwind_config_js {
        format!(
            r#"<script src="https://cdn.tailwindcss.com?plugins=forms,container-queries"></script>
<script>{}</script>"#,
            config
        )
    } else {
        String::new()
    }
}

/// Generate font link tags
pub fn font_links() -> String {
    let t = get();
    let mut fonts = std::collections::HashSet::new();
    fonts.insert(t.font_headline.clone());
    fonts.insert(t.font_body.clone());
    fonts.insert(t.font_label.clone());

    let mut links = String::new();
    for font in &fonts {
        if font.is_empty() { continue; }
        let encoded = font.replace(' ', "+");
        links.push_str(&format!(
            r#"<link href="https://fonts.googleapis.com/css2?family={}:wght@100..900&display=swap" rel="stylesheet">"#,
            encoded
        ));
        links.push('\n');
    }
    // Always include Material Symbols
    links.push_str(r#"<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" rel="stylesheet">"#);
    links
}
