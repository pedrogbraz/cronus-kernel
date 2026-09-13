//! Cronus UI token + Button slice.
//!
//! `.cronus` files already declare `style { preset aurora; accent-hex "..." }`
//! and `component Button layout:inline style:button+primary+md`. Without this
//! module the kernel paints Obsidian/amber hardcodes, so a complete `.cronus`
//! catalog can never look like cronus-ui.
//!
//! Values copied from `@cronus-ui/tokens` (aurora / neutral Ã— light / dark).
//! Components use ONLY `var(--cronus-*)` â€” no palette scales (`zinc-900`).

/// Vendored `@cronus-ui/tokens/styles/tokens.css` (runtime layer + looks).
/// Snapshot of packages/tokens â€” do not hand-edit palettes here.
const TOKENS_CSS: &str = include_str!("cronus_ui_tokens.css");

pub fn is_named_preset(preset: &str) -> bool {
    matches!(
        preset.trim(),
        "aurora" | "neutral" | "midnight" | "sunset" | "emerald"
    )
}

/// Token CSS.
///
/// Always emits **fallback aliases** on `:root` so `--cronus-*` exists without
/// stealing existing `--background` / `--foreground` (legacy theme keeps working).
/// Named presets inject the real tokens.css + CVA chrome.
pub fn token_css(preset: &str, mode: &str) -> String {
    let preset = preset.trim();
    if !is_named_preset(preset) {
        return format!("{FALLBACK_ROOT}\n{COMPONENT_CHROME}");
    }
    let mut css = format!("{FALLBACK_ROOT}\n{TOKENS_CSS}\n{COMPONENT_CHROME}");
    // Author opted in â€” apply that preset on :root as well as the data attr.
    let needle = format!("[data-cronus-theme=\"{preset}\"] {{");
    let repl = format!(":root, [data-cronus-theme=\"{preset}\"] {{");
    if css.contains(&needle) {
        css = css.replacen(&needle, &repl, 1);
    }
    let _ = mode;
    css
}

/// Full vendored tokens + chrome for the audit canvas. Theme/mode are selected
/// by `data-cronus-theme` / `data-cronus-mode` on `<html>` — do not steal
/// `:root` for a single preset (`token_css` does, and ignores `mode`).
pub fn audit_stylesheet() -> String {
    format!("{FALLBACK_ROOT}\n{TOKENS_CSS}\n{COMPONENT_CHROME}")
}

pub fn vendored_tokens_css() -> &'static str {
    TOKENS_CSS
}

pub fn component_chrome_css() -> &'static str {
    COMPONENT_CHROME
}

/// Button matching cronus-ui CONTRACT: semantic tokens, variants, sizes,
/// `data-slot`, `data-variant`, focus-visible, href â†’ `<a>`.
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
    format!(
        "<{tag}{href_attr}{type_attr}{disabled_attr} data-slot=\"button\" data-variant=\"{variant}\" data-size=\"{size}\" class=\"cui-btn\"{disabled_style}>{label}</{tag}>",
        size = size,
        disabled_style = if disabled {
            " style=\"opacity:0.5;pointer-events:none;\""
        } else {
            ""
        },
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
[data-slot]:focus-visible, [data-slot] :focus-visible {
  outline: 2px solid var(--cronus-ring, var(--cronus-primary)) !important;
  outline-offset: 2px;
}
@media (prefers-reduced-motion: reduce) {
  [data-slot] { animation: none !important; transition: none !important; }
}
"#;

/// CVA from packages/ui as CSS. Sizes/variants match button.tsx / input.tsx / badge.tsx.
const COMPONENT_CHROME: &str = r#"
:root, [data-cronus-theme] {
  --cronus-radius-sm: max(0px, calc(var(--cronus-radius, 14px) - 8px));
  --cronus-radius-md: max(0px, calc(var(--cronus-radius, 14px) - 4px));
  --cronus-radius-lg: var(--cronus-radius, 14px);
  --cronus-radius-xl: calc(var(--cronus-radius, 14px) + 4px);
  --ease-out-quart: cubic-bezier(0.16, 1, 0.3, 1);
  --cronus-ease: cubic-bezier(.22, 1, .36, 1);
}
html[data-cronus-theme] {
  background: var(--cronus-surface-base);
  color: var(--cronus-fg);
}
html[data-cronus-theme] body {
  background: var(--cronus-surface-base);
  color: var(--cronus-fg);
  font-family: var(--cronus-font-sans);
  font-weight: 400;
  -webkit-font-smoothing: antialiased;
  letter-spacing: 0;
}
html[data-cronus-theme] h1, html[data-cronus-theme] h2, html[data-cronus-theme] h3 {
  font-weight: 400;
}
html[data-cronus-theme] h1 { letter-spacing: -0.03em; }
html[data-cronus-theme] h2, html[data-cronus-theme] .text-4xl { letter-spacing: -0.025em; }
html[data-cronus-theme] h3, html[data-cronus-theme] .text-xl { letter-spacing: -0.02em; }

[data-slot="button"].cui-btn, [data-slot="button"] {
  display: inline-flex; align-items: center; justify-content: center; gap: 0.5rem;
  white-space: nowrap; border-radius: var(--cronus-radius-lg); font-weight: 500;
  line-height: 1; cursor: pointer; text-decoration: none;
  outline: none; border: 1px solid transparent;
  font-family: inherit;
  transition: background 150ms var(--ease-out-quart), box-shadow 150ms var(--ease-out-quart),
    transform 150ms var(--ease-out-quart), opacity 150ms var(--ease-out-quart), border-color 150ms;
}
[data-slot="button"]:active { transform: scale(0.98); }
[data-slot="button"][data-size="sm"] { height: 2rem; padding: 0 0.75rem; font-size: 0.75rem; }
[data-slot="button"][data-size="md"], [data-slot="button"]:not([data-size]) { height: 2.5rem; padding: 0 1rem; font-size: 0.875rem; }
[data-slot="button"][data-size="lg"] { height: 2.75rem; padding: 0 1.5rem; font-size: 1rem; }
[data-slot="button"][data-size="icon"] { height: 2.25rem; width: 2.25rem; padding: 0; }
[data-slot="button"][data-size="icon-sm"] { height: 2rem; width: 2rem; padding: 0; }
[data-slot="button"][data-variant="primary"], [data-slot="button"]:not([data-variant]) {
  background: var(--cronus-primary); color: var(--cronus-primary-foreground);
  box-shadow: var(--cronus-shadow-xs, 0 1px 2px rgba(0,0,0,.2));
}
[data-slot="button"][data-variant="primary"]:hover { opacity: 0.9; }
[data-slot="button"][data-variant="secondary"] {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
  border-color: var(--cronus-border);
}
[data-slot="button"][data-variant="outline"] {
  background: transparent; color: var(--cronus-fg); border-color: var(--cronus-border);
  box-shadow: var(--cronus-shadow-xs, none);
}
[data-slot="button"][data-variant="outline"]:hover { background: var(--cronus-surface-overlay); }
[data-slot="button"][data-variant="ghost"] {
  background: transparent; color: var(--cronus-fg-secondary); border-color: transparent;
}
[data-slot="button"][data-variant="ghost"]:hover { background: var(--cronus-surface-overlay); color: var(--cronus-fg); }
[data-slot="button"][data-variant="destructive"] {
  background: color-mix(in oklch, var(--cronus-error), black 30%); color: #fff;
}
[data-slot="button"][data-variant="link"] {
  background: transparent; color: var(--cronus-primary-text); border: 0;
  text-underline-offset: 4px;
}
[data-slot="button"][data-variant="link"]:hover { text-decoration: underline; }

[data-slot="input"], [data-slot$="-control"]:is(input, textarea, select) {
  display: flex; height: 2.5rem; width: 100%; box-sizing: border-box;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
  padding: 0 0.75rem; font-size: 0.875rem; font-family: inherit;
  outline: none;
  transition: border-color 150ms var(--ease-out-quart), box-shadow 150ms var(--ease-out-quart);
}
[data-slot="input"][aria-invalid="true"] {
  border-color: var(--cronus-error);
}
[data-slot="input"]:disabled {
  opacity: 0.5; pointer-events: none;
}
[data-slot="input"]::placeholder { color: var(--cronus-fg-tertiary); }

[data-slot="textarea"] {
  display: block; min-height: 5rem; width: 100%; box-sizing: border-box;
  field-sizing: content; resize: vertical;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
  padding: 0.5rem 0.75rem; font-size: 0.875rem; font-family: inherit;
  outline: none;
  transition: border-color 150ms var(--ease-out-quart), box-shadow 150ms var(--ease-out-quart);
}
[data-slot="textarea"][aria-invalid="true"] {
  border-color: var(--cronus-error);
}
[data-slot="textarea"]:disabled {
  opacity: 0.5; pointer-events: none;
}
[data-slot="textarea"]::placeholder { color: var(--cronus-fg-tertiary); }
[data-slot="rich-text-editor"] textarea {
  height: auto; padding: 0.6rem 0.75rem;
}

[data-slot="badge"] {
  display: inline-flex; align-items: center; gap: 0.25rem;
  border-radius: var(--cronus-radius-md); border: 1px solid var(--cronus-border);
  padding: 0.125rem 0.5rem; font-size: 0.75rem; font-weight: 500;
  white-space: nowrap;
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="badge"][data-variant="primary"] {
  background: var(--cronus-primary); color: var(--cronus-primary-foreground); border-color: transparent;
}
[data-slot="badge"][data-variant="secondary"] {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg-secondary); border-color: var(--cronus-border);
}
[data-slot="badge"][data-variant="outline"] {
  background: transparent; color: var(--cronus-fg); border-color: var(--cronus-border);
}
[data-slot="badge"][data-variant="success"] {
  background: color-mix(in oklch, var(--cronus-success) 15%, transparent);
  color: var(--cronus-success-text); border-color: transparent;
}
[data-slot="badge"][data-variant="warning"] {
  background: color-mix(in oklch, var(--cronus-warning) 15%, transparent);
  color: var(--cronus-warning-text); border-color: transparent;
}
[data-slot="badge"][data-variant="destructive"], [data-slot="badge"][data-variant="error"] {
  background: color-mix(in oklch, var(--cronus-error) 15%, transparent);
  color: var(--cronus-error-text); border-color: transparent;
}
[data-slot="badge"][data-variant="info"] {
  background: color-mix(in oklch, var(--cronus-info) 15%, transparent);
  color: var(--cronus-info-text); border-color: transparent;
}

[data-slot="label"] {
  font-size: 0.875rem; font-weight: 500; color: var(--cronus-fg);
  line-height: 1; user-select: none;
}

[data-slot="checkbox"] {
  display: inline-flex; align-items: center; justify-content: center;
  width: 1rem; height: 1rem; padding: 0; margin: 0; flex-shrink: 0;
  box-sizing: border-box; font: inherit; cursor: pointer; outline: none;
  border-radius: var(--cronus-radius-sm); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset); color: var(--cronus-primary-foreground);
  box-shadow: var(--cronus-shadow-xs, none);
  transition: background 150ms var(--ease-out-quart), border-color 150ms var(--ease-out-quart);
}
[data-slot="checkbox"][data-state="checked"] {
  background: var(--cronus-primary); border-color: var(--cronus-primary);
  color: var(--cronus-primary-foreground);
}
[data-slot="checkbox"][aria-invalid="true"] {
  border-color: var(--cronus-error);
}
[data-slot="checkbox"]:disabled {
  opacity: 0.5; pointer-events: none;
}
[data-slot="checkbox-indicator"] {
  display: block; width: 0.35rem; height: 0.55rem;
  border: solid currentColor; border-width: 0 2px 2px 0;
  transform: rotate(45deg) translateY(-0.0625rem);
}

[data-slot="switch"] {
  appearance: none; -webkit-appearance: none;
  display: inline-flex; align-items: center; flex-shrink: 0;
  height: 1.25rem; width: 2.25rem; padding: 0; margin: 0;
  border-radius: 9999px; border: 1px solid transparent;
  background: var(--cronus-fg-tertiary); cursor: pointer;
  transition: background 150ms var(--ease-out-quart);
}
[data-slot="switch"][data-state="checked"] {
  background: var(--cronus-primary);
}
[data-slot="switch"]:disabled {
  opacity: 0.5; pointer-events: none;
}
[data-slot="switch-thumb"] {
  pointer-events: none; display: block;
  width: 1rem; height: 1rem; border-radius: 9999px;
  background: var(--cronus-primary-foreground);
  box-shadow: var(--cronus-shadow-sm, 0 1px 2px rgba(0,0,0,.2));
  transform: translateX(0.125rem);
  transition: transform 150ms var(--ease-out-quart);
}
[data-slot="switch"][data-state="checked"] [data-slot="switch-thumb"] {
  transform: translateX(18px);
}

[data-slot="spinner"] {
  width: 1.25rem; height: 1.25rem; display: inline-block;
  color: currentColor; vertical-align: middle;
  animation: spin 1s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

[data-slot="separator"] {
  flex-shrink: 0; border: 0; background: var(--cronus-border);
}
[data-slot="separator"][data-orientation="horizontal"], [data-slot="separator"]:not([data-orientation]) {
  height: 1px; width: 100%;
}
[data-slot="separator"][data-orientation="vertical"] {
  height: 100%; width: 1px;
}

[data-slot="kbd"] {
  display: inline-flex; align-items: center; justify-content: center; gap: 0.25rem;
  height: 1.25rem; min-width: 1.25rem; padding: 0 0.375rem;
  border-radius: var(--cronus-radius-md); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-overlay); color: var(--cronus-fg-secondary);
  font-family: var(--cronus-font-mono, ui-monospace, monospace);
  font-size: 0.7rem; font-weight: 500; white-space: nowrap;
}

[data-slot="toggle"] {
  display: inline-flex; align-items: center; justify-content: center; gap: 0.5rem;
  height: 2.5rem; padding: 0 0.75rem; border-radius: var(--cronus-radius-lg);
  border: 0; background: transparent; color: var(--cronus-fg-secondary);
  font-size: 0.875rem; font-weight: 500; font-family: inherit;
  cursor: pointer; outline: none;
  transition: background 150ms var(--ease-out-quart), color 150ms var(--ease-out-quart);
}
[data-slot="toggle"][data-state="on"] {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}

[data-slot="progress"] {
  position: relative; height: 0.5rem; width: 100%; overflow: hidden;
  border-radius: 9999px; background: var(--cronus-surface-overlay);
}
[data-slot="progress-indicator"] {
  height: 100%; width: 100%; flex: 1; background: var(--cronus-primary);
  transition: transform 300ms var(--ease-out-quart);
}

[data-slot="skeleton"] {
  display: block; height: 0.9rem; width: 8rem;
  border-radius: var(--cronus-radius-md);
  background: var(--cronus-surface-overlay);
  animation: cui-pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
@keyframes cui-pulse { 50% { opacity: 0.5; } }

[data-slot="card"], [data-slot="glass-card"], [data-slot="spotlight-card"] {
  background: var(--cronus-surface-raised); border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-xl); box-shadow: var(--cronus-shadow-xs, none);
}

[data-slot="alert"] {
  display: grid; width: 100%; box-sizing: border-box;
  border: 1px solid var(--cronus-border); border-radius: var(--cronus-radius-lg);
  padding: 0.75rem 1rem; font-size: 0.875rem;
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="alert"][role="alert"] {
  background: color-mix(in oklch, var(--cronus-error) 10%, var(--cronus-surface-overlay));
  border-color: color-mix(in oklch, var(--cronus-error) 30%, transparent);
}
[data-slot="alert-title"] {
  font-weight: 500; color: var(--cronus-fg); line-height: 1.25;
}
[data-slot="alert-description"] {
  font-size: 0.875rem; color: var(--cronus-fg-secondary);
}

[data-slot="banner"] {
  display: flex; width: 100%; align-items: center; gap: 0.75rem;
  box-sizing: border-box;
  border-bottom: 1px solid var(--cronus-border);
  padding: 0.625rem 1rem; font-size: 0.875rem;
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="banner-content"] {
  display: flex; min-width: 0; flex: 1; flex-wrap: wrap;
  align-items: center; gap: 0.5rem;
}
[data-slot="banner-title"] {
  font-weight: 500;
}
[data-slot="banner-description"] {
  color: var(--cronus-fg-secondary);
}

[data-slot="dialog-content"], dialog[data-slot] {
  background: var(--cronus-surface-floating); border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-xl); box-shadow: var(--cronus-shadow-lg, none);
  color: var(--cronus-fg);
}

[data-slot="tabs"] [role="tab"][aria-selected="true"] {
  color: var(--cronus-fg); border-bottom-color: var(--cronus-primary);
}
[data-slot="table"] table, [data-slot="data-table"] table {
  font-size: 0.875rem;
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
        assert!(html.contains("data-size=\"md\""));
        assert!(html.contains("class=\"cui-btn\""));
        assert!(html.contains("type=\"button\""));
        assert!(!html.contains("uppercase"));
        assert!(!html.contains("zinc-"));
        assert!(!html.contains("amber-"));
        assert!(html.starts_with("<button"));
        let css = token_css("aurora", "dark");
        assert!(css.contains("background: var(--cronus-primary)"));
        assert!(css.contains("height: 2.5rem"));
    }

    #[test]
    fn button_href_is_anchor() {
        let html = button("Docs", "link", "md", Some("/docs"));
        assert!(html.starts_with("<a "));
        assert!(html.contains("href=\"/docs\""));
        assert!(html.contains("data-variant=\"link\""));
        assert!(!html.contains("type="));
        let css = token_css("aurora", "dark");
        assert!(css.contains("color: var(--cronus-primary-text)"));
    }

    #[test]
    fn button_destructive_alias_danger() {
        let a = button("Delete", "destructive", "md", None);
        let b = button("Delete", "danger", "md", None);
        assert!(a.contains("data-variant=\"destructive\""));
        assert!(b.contains("data-variant=\"destructive\""));
        let css = token_css("aurora", "dark");
        assert!(css.contains("color-mix(in oklch, var(--cronus-error), black 30%)"));
        assert!(css.contains("color: #fff"));
    }

    #[test]
    fn button_sizes() {
        let sm = button("A", "primary", "sm", None);
        let icon = button("A", "primary", "icon", None);
        assert!(sm.contains("data-size=\"sm\""));
        assert!(icon.contains("data-size=\"icon\""));
        let css = token_css("legacy", "dark");
        assert!(css.contains("[data-size=\"sm\"] { height: 2rem"));
        assert!(css.contains("[data-size=\"icon\"] { height: 2.25rem"));
    }

    #[test]
    fn aurora_tokens_match_vendored_css() {
        let css = token_css("aurora", "dark");
        assert!(css.contains("oklch(0.685 0.169 237.3)"));
        assert!(css.contains("--cronus-font-display"));
        assert!(css.contains("--cronus-shadow-glow"));
        assert!(css.contains("--cronus-chart-1"));
        assert!(css.contains("letter-spacing: -0.03em"));
    }

    #[test]
    fn button_disabled() {
        let html = button_ex("Nope", "primary", "md", None, true);
        assert!(html.contains(" disabled"));
        assert!(html.contains("opacity:0.5"));
    }

    #[test]
    fn tokens_include_focus_visible_ring() {
        let css = token_css("legacy", "dark");
        assert!(css.contains(":focus-visible"));
        assert!(css.contains("--cronus-ring"));
        assert!(css.contains("prefers-reduced-motion"));
    }

    #[test]
    fn component_chrome_has_no_tailwind_palette() {
        let css = audit_stylesheet();
        assert!(!css.contains("--tw-"));
        assert!(!css.contains("@tailwind"));
        assert!(!css.contains("zinc-900"));
        assert!(!css.contains("bg-zinc-"));
    }
}
