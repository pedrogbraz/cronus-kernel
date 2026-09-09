//! Cronus UI token + Button slice.
//!
//! `.cronus` files already declare `style { preset aurora; accent-hex "..." }`
//! and `component Button layout:inline style:button+primary+md`. Without this
//! module the kernel paints Obsidian/amber hardcodes, so a complete `.cronus`
//! catalog can never look like cronus-ui.
//!
//! Values copied from `@cronus-ui/tokens` (aurora / neutral × light / dark).
//! Components use ONLY `var(--cronus-*)` — no palette scales (`zinc-900`).

/// Token CSS.
///
/// Always emits **fallback aliases** on `:root` so `--cronus-*` exists without
/// stealing existing `--background` / `--foreground` (legacy theme keeps working).
/// Cronus-ui palettes apply only under `[data-cronus-theme="aurora"|...]`.
pub fn token_css(preset: &str, mode: &str) -> String {
    let preset = preset.trim();
    if preset.is_empty()
        || preset == "legacy"
        || preset == "obsidian"
        || preset == "default"
    {
        return FALLBACK_ROOT.to_string();
    }
    let dark = mode != "light";
    let (p, mode) = match (preset, dark) {
        ("neutral", true) => ("neutral", "dark"),
        ("neutral", false) => ("neutral", "light"),
        ("midnight", true) => ("midnight", "dark"),
        ("midnight", false) => ("midnight", "light"),
        ("sunset", true) => ("sunset", "dark"),
        ("sunset", false) => ("sunset", "light"),
        ("emerald", true) => ("emerald", "dark"),
        ("emerald", false) => ("emerald", "light"),
        (_, false) => ("aurora", "light"),
        _ => ("aurora", "dark"),
    };
    let mut css = format!(
        "{}\n{}\n{}",
        FALLBACK_ROOT,
        AURORA_BASE,
        match (p, mode) {
            ("aurora", "light") => AURORA_LIGHT,
            ("neutral", "light") => NEUTRAL_LIGHT,
            ("neutral", "dark") => NEUTRAL_DARK,
            ("midnight", "dark") => MIDNIGHT_DARK,
            ("midnight", "light") => MIDNIGHT_LIGHT,
            ("sunset", "dark") => SUNSET_DARK,
            ("sunset", "light") => SUNSET_LIGHT,
            ("emerald", "dark") => EMERALD_DARK,
            ("emerald", "light") => EMERALD_LIGHT,
            _ => "",
        }
    );
    // Author opted in via `style { preset aurora }` — apply on :root too.
    let needle = format!("[data-cronus-theme=\"{p}\"] {{");
    let repl = format!(":root, [data-cronus-theme=\"{p}\"] {{");
    css = css.replacen(&needle, &repl, 1);
    css
}

/// Button matching cronus-ui CONTRACT: semantic tokens, variants, sizes,
/// `data-slot`, `data-variant`, focus-visible, href → `<a>`.
/// `danger` is accepted as an alias of `destructive` (legacy kernel name).
pub fn button(label: &str, variant: &str, size: &str, href: Option<&str>) -> String {
    button_ex(label, variant, size, href, false)
}

pub fn button_ex(
    label: &str,
    variant: &str,
    size: &str,
    href: Option<&str>,
    disabled: bool,
) -> String {
    let variant = match variant {
        "danger" | "destructive" => "destructive",
        "secondary" => "secondary",
        "outline" => "outline",
        "ghost" => "ghost",
        "link" => "link",
        _ => "primary",
    };
    let (height, pad, font, _icon) = match size {
        "sm" => ("2rem", "0 0.75rem", "0.75rem", "0.875rem"),
        "lg" => ("2.75rem", "0 1.5rem", "1rem", "1rem"),
        "icon" => ("2.25rem", "0", "0.875rem", "1rem"),
        "icon-sm" => ("2rem", "0", "0.75rem", "0.875rem"),
        _ => ("2.5rem", "0 1rem", "0.875rem", "1rem"),
    };
    let square = size == "icon" || size == "icon-sm";
    let width = if square { height } else { "auto" };
    let look = match variant {
        "secondary" => {
            "background:var(--cronus-surface-overlay);color:var(--cronus-fg);border:1px solid var(--cronus-border);"
        }
        "outline" => {
            "background:transparent;color:var(--cronus-fg);border:1px solid var(--cronus-border);"
        }
        "ghost" => {
            "background:transparent;color:var(--cronus-fg-secondary);border:1px solid transparent;"
        }
        "destructive" => {
            // text-white on a darkened error fill — same CONTRACT exception as button.tsx
            "background:color-mix(in oklch,var(--cronus-error),black 30%);color:#fff;border:1px solid transparent;"
        }
        "link" => {
            "background:transparent;color:var(--cronus-primary-text);border:1px solid transparent;text-decoration:none;"
        }
        _ => {
            "background:var(--cronus-primary);color:var(--cronus-primary-foreground);border:1px solid transparent;"
        }
    };
    let tag = if href.is_some() { "a" } else { "button" };
    let href_attr = href.map(|h| format!(" href=\"{}\"", h)).unwrap_or_default();
    let type_attr = if href.is_none() {
        " type=\"button\""
    } else {
        ""
    };
    let disabled_attr = if disabled && href.is_none() {
        " disabled"
    } else {
        ""
    };
    let disabled_style = if disabled {
        "opacity:0.5;pointer-events:none;"
    } else {
        ""
    };
    format!(
        "<{tag}{href_attr}{type_attr}{disabled_attr} data-slot=\"button\" data-variant=\"{variant}\" style=\"display:inline-flex;align-items:center;justify-content:center;gap:0.5rem;height:{height};width:{width};padding:{pad};font-size:{font};font-weight:500;line-height:1;white-space:nowrap;border-radius:0.5rem;cursor:pointer;text-decoration:none;outline:none;transition:opacity 150ms var(--cronus-ease,cubic-bezier(.22,1,.36,1)),background 150ms,border-color 150ms,box-shadow 150ms,transform 150ms;{look}{disabled_style}\">{label}</{tag}>"
    )
}

/// Aliases so `--cronus-*` exists on legacy pages without overriding
/// `--background` / `--foreground` / Obsidian.
const FALLBACK_ROOT: &str = r#":root {
  --cronus-primary: var(--primary, #0ea5e9);
  --cronus-primary-foreground: var(--background, #09090b);
  --cronus-primary-text: var(--cronus-primary);
  --cronus-accent: var(--accent, var(--cronus-primary));
  --cronus-surface-base: var(--cronus-bg, var(--background, #09090b));
  --cronus-surface-overlay: var(--surface, var(--cronus-surface, #18181b));
  --cronus-fg: var(--foreground, var(--cronus-text, #fafaf9));
  --cronus-fg-secondary: var(--foreground-muted, var(--cronus-text-muted, #a1a1aa));
  --cronus-border: var(--border, oklch(1 0 0 / 0.1));
  --cronus-ring: var(--cronus-primary);
  --cronus-error: var(--danger, var(--error, #f43f5e));
  --cronus-ease: cubic-bezier(.22, 1, .36, 1);
}
"#;

// Aurora dark — scoped so it does NOT replace the legacy theme unless
// the page opts in with data-cronus-theme="aurora" (or style.preset).
const AURORA_BASE: &str = r#"[data-cronus-theme="aurora"] {
  color-scheme: dark;
  --cronus-primary: oklch(0.685 0.169 237.3);
  --cronus-primary-foreground: oklch(0.145 0.005 285.8);
  --cronus-primary-text: oklch(0.685 0.169 237.3);
  --cronus-accent: oklch(0.715 0.143 215.2);
  --cronus-accent-foreground: oklch(0.145 0 0);
  --cronus-surface-base: oklch(0.145 0.005 285.8);
  --cronus-surface-inset: oklch(0.165 0.005 285.8);
  --cronus-surface-raised: oklch(0.195 0.005 285.8);
  --cronus-surface-overlay: oklch(0.235 0.006 285.9);
  --cronus-surface-elevated: oklch(0.27 0.006 286);
  --cronus-surface-floating: oklch(0.2 0.006 286);
  --cronus-fg: oklch(0.985 0.001 106.4);
  --cronus-fg-secondary: oklch(0.705 0.015 286);
  --cronus-fg-tertiary: oklch(0.62 0.014 286);
  --cronus-fg-muted: oklch(0.442 0.013 286);
  --cronus-fg-inverse: oklch(0.235 0.006 285.9);
  --cronus-border: oklch(1 0 0 / 0.1);
  --cronus-border-strong: oklch(1 0 0 / 0.14);
  --cronus-border-soft: oklch(1 0 0 / 0.06);
  --cronus-ring: oklch(0.685 0.169 237.3);
  --cronus-success: oklch(0.715 0.155 162.5);
  --cronus-success-text: oklch(0.715 0.155 162.5);
  --cronus-warning: oklch(0.769 0.166 70.08);
  --cronus-warning-text: oklch(0.769 0.166 70.08);
  --cronus-error: oklch(0.645 0.222 16.44);
  --cronus-error-text: oklch(0.69 0.222 16.44);
  --cronus-info: oklch(0.715 0.143 215.2);
  --cronus-info-text: oklch(0.715 0.143 215.2);
  --cronus-radius: 14px;
  --cronus-ease: cubic-bezier(.22, 1, .36, 1);
  --cronus-font-sans: "SF Pro Text", Geist, -apple-system, BlinkMacSystemFont, system-ui, sans-serif;
}
"#;

const AURORA_LIGHT: &str = r#"[data-cronus-theme="aurora"][data-cronus-mode="light"], :root[data-cronus-mode="light"] {
  color-scheme: light;
  --cronus-primary-text: oklch(0.515 0.12 237.3);
  --cronus-surface-base: oklch(1 0 0);
  --cronus-surface-inset: oklch(0.985 0 0);
  --cronus-surface-raised: oklch(1 0 0);
  --cronus-surface-overlay: oklch(0.967 0.001 286);
  --cronus-surface-elevated: oklch(1 0 0);
  --cronus-surface-floating: oklch(1 0 0);
  --cronus-fg: oklch(0.235 0.006 285.9);
  --cronus-fg-secondary: oklch(0.442 0.013 286);
  --cronus-fg-tertiary: oklch(0.54 0.014 286);
  --cronus-fg-muted: oklch(0.705 0.015 286);
  --cronus-fg-inverse: oklch(0.985 0.001 106.4);
  --cronus-border: oklch(0 0 0 / 0.1);
  --cronus-border-strong: oklch(0 0 0 / 0.14);
  --cronus-border-soft: oklch(0 0 0 / 0.06);
  --cronus-success: oklch(0.52 0.15 162);
  --cronus-success-text: oklch(0.48 0.15 162);
  --cronus-warning: oklch(0.52 0.12 70);
  --cronus-warning-text: oklch(0.518 0.12 70);
  --cronus-error: oklch(0.55 0.2 25);
  --cronus-error-text: oklch(0.531 0.2 25);
  --cronus-info: oklch(0.52 0.16 235);
  --cronus-info-text: oklch(0.49 0.16 235);
}
"#;

const NEUTRAL_LIGHT: &str = r#"[data-cronus-theme="neutral"] {
  color-scheme: light;
  --cronus-primary: oklch(0.205 0 0);
  --cronus-primary-foreground: oklch(0.985 0 0);
  --cronus-primary-text: oklch(0.205 0 0);
  --cronus-accent: oklch(0.96 0 0);
  --cronus-accent-foreground: oklch(0.205 0 0);
  --cronus-surface-base: oklch(0.985 0 0);
  --cronus-surface-inset: oklch(0.97 0 0);
  --cronus-surface-raised: oklch(1 0 0);
  --cronus-surface-overlay: oklch(0.96 0 0);
  --cronus-surface-elevated: oklch(1 0 0);
  --cronus-surface-floating: oklch(1 0 0);
  --cronus-fg: oklch(0.145 0 0);
  --cronus-fg-secondary: oklch(0.43 0 0);
  --cronus-fg-tertiary: oklch(0.53 0 0);
  --cronus-fg-muted: oklch(0.705 0 0);
  --cronus-fg-inverse: oklch(0.985 0 0);
  --cronus-border: oklch(0.915 0 0);
  --cronus-border-strong: oklch(0.86 0 0);
  --cronus-border-soft: oklch(0.94 0 0);
  --cronus-ring: oklch(0.705 0 0);
  --cronus-success: oklch(0.52 0.15 162);
  --cronus-success-text: oklch(0.48 0.15 162);
  --cronus-warning: oklch(0.52 0.12 70);
  --cronus-warning-text: oklch(0.518 0.12 70);
  --cronus-error: oklch(0.55 0.2 25);
  --cronus-error-text: oklch(0.531 0.2 25);
  --cronus-info: oklch(0.52 0.16 235);
  --cronus-info-text: oklch(0.49 0.16 235);
}
"#;

const NEUTRAL_DARK: &str = r#"[data-cronus-theme="neutral"][data-cronus-mode="dark"] {
  color-scheme: dark;
  --cronus-primary: oklch(0.93 0 0);
  --cronus-primary-foreground: oklch(0.11 0 0);
  --cronus-primary-text: oklch(0.93 0 0);
  --cronus-accent: oklch(0.27 0 0);
  --cronus-accent-foreground: oklch(0.93 0 0);
  --cronus-surface-base: oklch(0.11 0 0);
  --cronus-surface-inset: oklch(0.13 0 0);
  --cronus-surface-raised: oklch(0.16 0 0);
  --cronus-surface-overlay: oklch(0.2 0 0);
  --cronus-surface-elevated: oklch(0.23 0 0);
  --cronus-surface-floating: oklch(0.165 0 0);
  --cronus-fg: oklch(0.93 0 0);
  --cronus-fg-secondary: oklch(0.7 0 0);
  --cronus-fg-tertiary: oklch(0.62 0 0);
  --cronus-fg-muted: oklch(0.44 0 0);
  --cronus-fg-inverse: oklch(0.11 0 0);
  --cronus-border: oklch(1 0 0 / 0.1);
  --cronus-border-strong: oklch(1 0 0 / 0.16);
  --cronus-border-soft: oklch(1 0 0 / 0.06);
  --cronus-ring: oklch(0.55 0 0);
  --cronus-success: oklch(0.715 0.155 162.5);
  --cronus-error: oklch(0.704 0.191 22.216);
  --cronus-error-text: oklch(0.704 0.191 22.216);
}
"#;

const MIDNIGHT_DARK: &str = r#"[data-cronus-theme="midnight"] {
  --cronus-primary: oklch(0.55 0.205 280);
  --cronus-primary-foreground: oklch(0.985 0.005 285);
  --cronus-primary-text: oklch(0.665 0.205 280);
  --cronus-accent: oklch(0.53 0.2 292);
  --cronus-ring: oklch(0.55 0.205 280);
}
"#;

const MIDNIGHT_LIGHT: &str = r#"[data-cronus-theme="midnight"][data-cronus-mode="light"] {
  color-scheme: light;
  --cronus-primary: oklch(0.55 0.205 280);
  --cronus-primary-text: oklch(0.528 0.205 280);
  --cronus-ring: oklch(0.55 0.205 280);
}
"#;

const SUNSET_DARK: &str = r#"[data-cronus-theme="sunset"] {
  --cronus-primary: oklch(0.78 0.16 65);
  --cronus-primary-foreground: oklch(0.24 0.05 55);
  --cronus-primary-text: oklch(0.78 0.16 65);
  --cronus-accent: oklch(0.585 0.22 18);
  --cronus-ring: oklch(0.78 0.16 65);
}
"#;

const SUNSET_LIGHT: &str = r#"[data-cronus-theme="sunset"][data-cronus-mode="light"] {
  color-scheme: light;
  --cronus-primary: oklch(0.78 0.16 65);
  --cronus-primary-text: oklch(0.55 0.16 55);
  --cronus-ring: oklch(0.78 0.16 65);
}
"#;

const EMERALD_DARK: &str = r#"[data-cronus-theme="emerald"] {
  --cronus-primary: oklch(0.72 0.15 162);
  --cronus-primary-foreground: oklch(0.145 0.005 285.8);
  --cronus-primary-text: oklch(0.72 0.15 162);
  --cronus-accent: oklch(0.7 0.12 195);
  --cronus-ring: oklch(0.72 0.15 162);
}
"#;

const EMERALD_LIGHT: &str = r#"[data-cronus-theme="emerald"][data-cronus-mode="light"] {
  color-scheme: light;
  --cronus-primary: oklch(0.5 0.14 162);
  --cronus-primary-text: oklch(0.45 0.14 162);
  --cronus-ring: oklch(0.5 0.14 162);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aurora_dark_emits_semantic_vars() {
        let css = token_css("aurora", "dark");
        assert!(css.contains("--cronus-primary:"));
        assert!(css.contains("--cronus-surface-base:"));
        assert!(css.contains("--cronus-fg:"));
        assert!(css.contains("--cronus-error:"));
        assert!(!css.contains("zinc-900"));
        assert!(css.contains(":root, [data-cronus-theme=\"aurora\"]"));
    }

    #[test]
    fn legacy_preset_does_not_force_aurora_root() {
        let css = token_css("legacy", "dark");
        assert!(css.contains("--cronus-primary: var(--primary"));
        assert!(!css.contains("oklch(0.685 0.169 237.3)"));
        assert!(!css.contains("color-scheme: dark"));
    }

    #[test]
    fn button_contract() {
        let html = button("Save", "primary", "md", None);
        assert!(html.contains("data-slot=\"button\""));
        assert!(html.contains("data-variant=\"primary\""));
        assert!(html.contains("type=\"button\""));
        assert!(html.contains("--cronus-primary"));
        assert!(!html.contains("uppercase"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("amber-"));
        assert!(html.starts_with("<button"));
    }

    #[test]
    fn button_href_is_anchor() {
        let html = button("Docs", "link", "md", Some("/docs"));
        assert!(html.starts_with("<a "));
        assert!(html.contains("href=\"/docs\""));
        assert!(html.contains("data-variant=\"link\""));
        assert!(html.contains("--cronus-primary-text"));
        assert!(!html.contains("type="));
    }

    #[test]
    fn button_destructive_alias_danger() {
        let a = button("Delete", "destructive", "md", None);
        let b = button("Delete", "danger", "md", None);
        assert!(a.contains("data-variant=\"destructive\""));
        assert!(b.contains("data-variant=\"destructive\""));
        assert!(a.contains("--cronus-error"));
        assert!(a.contains("#fff")); // CONTRACT: text-white on darkened error fill
    }

    #[test]
    fn button_sizes() {
        let sm = button("A", "primary", "sm", None);
        let icon = button("A", "primary", "icon", None);
        assert!(sm.contains("2rem"));
        assert!(icon.contains("width:2.25rem"));
    }

    #[test]
    fn button_disabled() {
        let html = button_ex("Nope", "primary", "md", None, true);
        assert!(html.contains(" disabled"));
        assert!(html.contains("opacity:0.5"));
    }
}
