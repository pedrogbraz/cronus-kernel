// theme.rs — Design system token extraction and CSS variable generation
//
// Extracts color/font tokens from tailwind_config and style blocks,
// generates CSS custom properties so all renderers inherit the design system.

use std::collections::HashMap;
use std::sync::RwLock;

/// Global theme tokens — set once at startup from tailwind_config/style
static THEME: RwLock<Option<ThemeTokens>> = RwLock::new(None);
/// Named cronus-ui preset (`aurora` / `midnight` / …). Empty = legacy.
static PRESET: RwLock<String> = RwLock::new(String::new());

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
            while i < chars.len() && chars[i] != '"' {
                i += 1;
            }
            let key_end = i;
            i += 1; // skip closing "
                    // Skip whitespace and colon
            while i < chars.len() && (chars[i] == ' ' || chars[i] == ':') {
                i += 1;
            }
            // Check for "#hex"
            if i < chars.len() && chars[i] == '"' {
                let val_start = i + 1;
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    i += 1;
                }
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
    if let Some(v) = colors.get("background") {
        tokens.background = v.clone();
    }
    if let Some(v) = colors.get("surface") {
        tokens.surface = v.clone();
    }
    if let Some(v) = colors.get("surface-container") {
        tokens.surface_container = v.clone();
    }
    if let Some(v) = colors.get("surface-container-low") {
        tokens.surface_container_low = v.clone();
    }
    if let Some(v) = colors.get("surface-container-lowest") {
        tokens.surface_container_lowest = v.clone();
    }
    if let Some(v) = colors.get("surface-container-high") {
        tokens.surface_container_high = v.clone();
    }
    if let Some(v) = colors.get("surface-container-highest") {
        tokens.surface_container_highest = v.clone();
    }
    if let Some(v) = colors.get("surface-bright") {
        tokens.surface_bright = v.clone();
    }
    if let Some(v) = colors.get("on-surface") {
        tokens.on_surface = v.clone();
    }
    if let Some(v) = colors.get("on-surface-variant") {
        tokens.on_surface_variant = v.clone();
    }
    if let Some(v) = colors.get("on-background") {
        tokens.on_background = v.clone();
    }
    if let Some(v) = colors.get("primary") {
        tokens.primary = v.clone();
    }
    if let Some(v) = colors.get("secondary") {
        tokens.secondary = v.clone();
    }
    if let Some(v) = colors.get("tertiary") {
        tokens.tertiary = v.clone();
    }
    if let Some(v) = colors.get("error") {
        tokens.error = v.clone();
    }
    if let Some(v) = colors.get("outline") {
        tokens.outline = v.clone();
    }
    if let Some(v) = colors.get("outline-variant") {
        tokens.outline_variant = v.clone();
    }
    if let Some(v) = colors.get("secondary-container") {
        tokens.secondary_container = v.clone();
    }
    if let Some(v) = colors.get("error-container") {
        tokens.error_container = v.clone();
    }
    if let Some(v) = colors.get("on-error-container") {
        tokens.on_error_container = v.clone();
    }

    // Extract font families
    // Pattern: "headline": ["FontName"] or "body": ["FontName"]
    for (label, field) in [
        ("headline", &mut tokens.font_headline),
        ("body", &mut tokens.font_body),
        ("label", &mut tokens.font_label),
    ] {
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

// ── Color conversion helpers ──

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 {
        return (0, 0, 0);
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    (r, g, b)
}

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < 1e-6 {
        return (0.0, 0.0, l * 100.0);
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if (max - r).abs() < 1e-6 {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) * 60.0
    } else if (max - g).abs() < 1e-6 {
        ((b - r) / d + 2.0) * 60.0
    } else {
        ((r - g) / d + 4.0) * 60.0
    };
    (h, s * 100.0, l * 100.0)
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

fn hsl_to_hex(h: f64, s: f64, l: f64) -> String {
    let s = s / 100.0;
    let l = l / 100.0;
    let (r, g, b) = if s.abs() < 1e-6 {
        (l, l, l)
    } else {
        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;
        let hh = h / 360.0;
        (
            hue_to_rgb(p, q, hh + 1.0 / 3.0),
            hue_to_rgb(p, q, hh),
            hue_to_rgb(p, q, hh - 1.0 / 3.0),
        )
    };
    format!(
        "#{:02x}{:02x}{:02x}",
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8
    )
}

fn mix_hex(c1: &str, c2: &str, ratio: f64) -> String {
    let (r1, g1, b1) = hex_to_rgb(c1);
    let (r2, g2, b2) = hex_to_rgb(c2);
    let r = (r1 as f64 * (1.0 - ratio) + r2 as f64 * ratio).round() as u8;
    let g = (g1 as f64 * (1.0 - ratio) + g2 as f64 * ratio).round() as u8;
    let b = (b1 as f64 * (1.0 - ratio) + b2 as f64 * ratio).round() as u8;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// Derive a full color palette from a single accent hex color.
/// One declaration → 22 tokens. Zero hardcode outside this function.
pub fn derive_palette(accent: &str, theme_mode: &str, font: &str) -> ThemeTokens {
    let accent = if accent.starts_with('#') && accent.len() >= 7 {
        accent
    } else {
        "#87adff"
    };
    let (h, s, l) = {
        let (r, g, b) = hex_to_rgb(accent);
        rgb_to_hsl(r, g, b)
    };
    let is_dark = theme_mode != "light";

    let primary = accent.to_string();
    let secondary = hsl_to_hex((h + 40.0) % 360.0, (s * 0.8).min(100.0), l);
    let tertiary = hsl_to_hex((h + 80.0) % 360.0, (s * 0.7).min(100.0), l);

    let surface = if is_dark {
        mix_hex(accent, "#131313", 0.92)
    } else {
        mix_hex(accent, "#fafafa", 0.95)
    };
    let surface_container = if is_dark {
        mix_hex(accent, "#1f1f1f", 0.90)
    } else {
        mix_hex(accent, "#f0f0f0", 0.92)
    };
    let outline = if is_dark {
        mix_hex(accent, "#888888", 0.70)
    } else {
        mix_hex(accent, "#888888", 0.50)
    };
    let outline_variant = if is_dark {
        mix_hex(accent, "#444444", 0.75)
    } else {
        mix_hex(accent, "#cccccc", 0.70)
    };

    let font_name = if font.is_empty() { "Inter" } else { font };

    ThemeTokens {
        background: if is_dark {
            "#0a0a0a".into()
        } else {
            "#fafafa".into()
        },
        surface,
        surface_container,
        surface_container_low: if is_dark {
            mix_hex(accent, "#1b1b1b", 0.92)
        } else {
            mix_hex(accent, "#f5f5f5", 0.93)
        },
        surface_container_lowest: if is_dark {
            mix_hex(accent, "#0e0e0e", 0.95)
        } else {
            mix_hex(accent, "#fafafa", 0.97)
        },
        surface_container_high: if is_dark {
            mix_hex(accent, "#2a2a2a", 0.88)
        } else {
            mix_hex(accent, "#e8e8e8", 0.90)
        },
        surface_container_highest: if is_dark {
            mix_hex(accent, "#353535", 0.85)
        } else {
            mix_hex(accent, "#e0e0e0", 0.88)
        },
        surface_bright: if is_dark {
            mix_hex(accent, "#393939", 0.85)
        } else {
            mix_hex(accent, "#ffffff", 0.90)
        },
        on_surface: if is_dark {
            "#e2e2e2".into()
        } else {
            "#1a1a1a".into()
        },
        on_surface_variant: if is_dark {
            "#c0c0c0".into()
        } else {
            "#444444".into()
        },
        on_background: if is_dark {
            "#e2e2e2".into()
        } else {
            "#1a1a1a".into()
        },
        primary,
        secondary,
        tertiary,
        error: "#ef4444".into(),
        outline,
        outline_variant,
        secondary_container: hsl_to_hex(
            (h + 40.0) % 360.0,
            (s * 0.5).min(100.0),
            if is_dark { 25.0 } else { 85.0 },
        ),
        error_container: if is_dark {
            "#93000a".into()
        } else {
            "#ffdad6".into()
        },
        on_error_container: if is_dark {
            "#ffdad6".into()
        } else {
            "#93000a".into()
        },
        font_headline: font_name.into(),
        font_body: font_name.into(),
        font_label: font_name.into(),
        tailwind_config_js: None,
    }
}

/// Parse from style node accent/font (backward compat — calls derive_palette)
pub fn parse_from_style(accent: &str, font: &str) -> ThemeTokens {
    derive_palette(accent, "dark", font)
}

/// Set the global theme tokens
pub fn set_global(tokens: ThemeTokens) {
    *THEME.write().unwrap() = Some(tokens);
}

/// Named cronus-ui preset from `style { preset aurora }`. Empty = legacy path.
pub fn set_preset(preset: &str) {
    *PRESET.write().unwrap() = preset.trim().to_ascii_lowercase();
}

pub fn get_preset() -> String {
    PRESET.read().unwrap().clone()
}

/// Get current tokens (returns defaults if not set)
pub fn get() -> ThemeTokens {
    THEME.read().unwrap().clone().unwrap_or_default()
}

/// Generate CSS custom properties block from current theme
pub fn css_vars() -> String {
    let t = get();
    format!(
        r#":root {{
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
        bg = t.background,
        surface = t.surface,
        sc = t.surface_container,
        scl = t.surface_container_low,
        scll = t.surface_container_lowest,
        sch = t.surface_container_high,
        schh = t.surface_container_highest,
        sb = t.surface_bright,
        os = t.on_surface,
        osv = t.on_surface_variant,
        primary = t.primary,
        secondary = t.secondary,
        tertiary = t.tertiary,
        error = t.error,
        outline = t.outline,
        ov = t.outline_variant,
        sec_c = t.secondary_container,
        err_c = t.error_container,
        oec = t.on_error_container,
        fh = t.font_headline,
        fb = t.font_body,
        fl = t.font_label,
    )
}

/// Generate the Tailwind CDN script + config if available
pub fn tailwind_cdn_script() -> String {
    let script_nonce = crate::security::script_nonce_attr();
    let t = get();
    if let Some(ref config) = t.tailwind_config_js {
        format!(
            r#"<script{script_nonce} src="https://cdn.tailwindcss.com?plugins=forms,container-queries"></script>
<script{script_nonce}>{}</script>"#,
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
        if font.is_empty() {
            continue;
        }
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
