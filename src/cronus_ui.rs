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

[data-slot="slider"] {
  position: relative; display: flex; width: 100%; align-items: center;
}
[data-slot="slider-track"] {
  position: relative; height: 0.375rem; width: 100%; flex-grow: 1;
  overflow: hidden; border-radius: 9999px; background: var(--cronus-surface-overlay);
}
[data-slot="slider-range"] {
  position: absolute; height: 100%; background: var(--cronus-primary);
}
[data-slot="slider-thumb"] {
  display: block; position: absolute; width: 1rem; height: 1rem;
  border-radius: 9999px; border: 1px solid var(--cronus-primary);
  background: var(--cronus-surface-base);
  box-shadow: var(--cronus-shadow-sm, 0 1px 2px rgba(0,0,0,.2));
  transform: translateX(-50%);
}

[data-slot="radio-group"] {
  display: grid; gap: 0.5rem;
}
[data-slot="radio-group-item"] {
  display: inline-flex; align-items: center; justify-content: center;
  width: 1rem; height: 1rem; padding: 0; margin: 0;
  border-radius: 9999px; border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset); color: var(--cronus-primary);
  cursor: pointer; outline: none; font: inherit;
}
[data-slot="radio-group-item"][data-state="checked"] {
  border-color: var(--cronus-primary);
}
[data-slot="radio-group-item"][data-state="checked"]::after {
  content: ""; width: 0.5rem; height: 0.5rem; border-radius: 9999px;
  background: var(--cronus-primary);
}

[data-slot="chip"] {
  display: inline-flex; align-items: center; height: 1.75rem;
  border-radius: 9999px; border: 1px solid var(--cronus-border);
  padding: 0 0.625rem; font-size: 0.875rem; font-weight: 500;
  white-space: nowrap; background: var(--cronus-surface-overlay);
  color: var(--cronus-fg);
}

[data-slot="avatar"] {
  position: relative; display: flex;
  width: 2.5rem; height: 2.5rem; flex-shrink: 0; overflow: hidden;
  border-radius: 9999px;
}
[data-slot="avatar-image"] {
  aspect-ratio: 1 / 1; width: 100%; height: 100%; object-fit: cover;
}
[data-slot="avatar-fallback"] {
  display: flex; width: 100%; height: 100%;
  align-items: center; justify-content: center; border-radius: 9999px;
  background: var(--cronus-surface-overlay); color: var(--cronus-fg-secondary);
  font-size: 0.875rem; font-weight: 500;
}

[data-slot="card"], [data-slot="glass-card"], [data-slot="spotlight-card"] {
  background: var(--cronus-surface-raised); border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-xl); box-shadow: var(--cronus-shadow-xs, none);
}
[data-slot="card"] {
  display: flex; flex-direction: column; gap: 1.5rem;
  padding-top: 1.5rem; padding-bottom: 1.5rem; color: var(--cronus-fg);
  box-shadow: var(--cronus-shadow-sm, var(--cronus-shadow-xs, none));
}
[data-slot="card-header"] {
  display: grid; min-width: 0; grid-auto-rows: min-content;
  grid-template-columns: minmax(0, 1fr) auto; align-items: start;
  gap: 0.375rem; padding-left: 1.5rem; padding-right: 1.5rem;
}
[data-slot="card-title"] {
  min-width: 0; font-weight: 500; line-height: 1; color: var(--cronus-fg);
}
[data-slot="card-description"] {
  grid-column: 1 / -1; font-size: 0.875rem; color: var(--cronus-fg-secondary);
}
[data-slot="card-content"] {
  min-width: 0; padding-left: 1.5rem; padding-right: 1.5rem;
  font-size: 0.875rem; color: var(--cronus-fg-secondary);
}

[data-slot="empty"] {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  gap: 0.75rem; border-radius: var(--cronus-radius-xl);
  border: 1px dashed var(--cronus-border);
  background: color-mix(in oklch, var(--cronus-surface-inset) 40%, transparent);
  padding: 3rem 1.5rem; text-align: center;
}
[data-slot="empty-title"] {
  font-size: 0.875rem; font-weight: 600; color: var(--cronus-fg);
}
[data-slot="empty-description"] {
  max-width: 24rem; font-size: 0.875rem; color: var(--cronus-fg-secondary);
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
  font-size: 0.875rem; width: 100%; border-collapse: collapse;
}
[data-slot="table"] th, [data-slot="data-table"] th {
  text-align: start; padding: 0.5rem 0.75rem; font-size: 0.75rem; font-weight: 500;
  color: var(--cronus-fg-secondary); border-bottom: 1px solid var(--cronus-border);
}
[data-slot="table"] td, [data-slot="data-table"] td {
  padding: 0.5rem 0.75rem; border-bottom: 1px solid var(--cronus-border);
  color: var(--cronus-fg);
}
[data-slot="select"] {
  display: flex; flex-direction: column; gap: 0.35rem; font-size: 0.875rem;
}
[data-slot="select"] select {
  height: 2.5rem; padding: 0 0.75rem; border-radius: var(--cronus-radius-md);
  border: 1px solid var(--cronus-border); background: var(--cronus-surface-inset);
  color: var(--cronus-fg); font: inherit;
}
[data-slot="dialog"] { display: inline-flex; flex-direction: column; gap: 0.5rem; }
[data-slot="dialog"] form { display: flex; flex-direction: column; gap: 0.75rem; padding: 1.25rem; min-width: 18rem; }
[data-slot="dialog-title"] { font-weight: 500; font-size: 1rem; }
[data-slot="dialog-description"] { margin: 0; color: var(--cronus-fg-secondary); font-size: 0.875rem; }
[data-slot="tabs"] [role="tablist"] {
  display: flex; gap: 0.15rem; border-bottom: 1px solid var(--cronus-border);
}
[data-slot="tabs"] [role="tab"] {
  padding: 0.4rem 0.75rem; border: 0; border-bottom: 2px solid transparent;
  background: transparent; color: var(--cronus-fg-secondary); cursor: pointer; font: inherit;
}
[data-slot="tabs"] [role="tab"][aria-selected="true"] {
  color: var(--cronus-fg); border-bottom-color: var(--cronus-primary);
}
[data-slot="tabs"] [role="tabpanel"] { padding: 0.75rem 0; font-size: 0.875rem; }
[data-slot="accordion"] { display: flex; flex-direction: column; }
[data-slot="accordion-item"] { border-bottom: 1px solid var(--cronus-border); padding: 0.5rem 0; }
[data-slot="accordion-trigger"] { cursor: pointer; font-size: 0.875rem; }
[data-slot="accordion-content"] { padding: 0.5rem 0; color: var(--cronus-fg-secondary); font-size: 0.875rem; }
[data-slot="pagination"] { display: flex; gap: 0.25rem; align-items: center; }
[data-slot="pagination-link"], [data-slot="pagination-previous"], [data-slot="pagination-next"] {
  display: inline-flex; align-items: center; justify-content: center;
  min-width: 2.25rem; height: 2.25rem; padding: 0 0.5rem;
  border-radius: var(--cronus-radius-md); border: 1px solid var(--cronus-border);
  color: var(--cronus-fg); text-decoration: none; font-size: 0.875rem;
}
[data-slot="pagination-link"][aria-current="page"] {
  background: var(--cronus-primary); color: var(--cronus-primary-foreground); border-color: transparent;
}
[data-slot="breadcrumb-list"] { list-style: none; padding: 0; margin: 0; display: flex; gap: 0.35rem; font-size: 0.8125rem; }
[data-slot="breadcrumb-link"] { color: var(--cronus-fg-secondary); text-decoration: none; }
[data-slot="breadcrumb-page"] { color: var(--cronus-fg); }
[data-slot="tooltip"] { position: relative; display: inline-block; }
[data-slot="tooltip-trigger"] { list-style: none; cursor: pointer; }
[data-slot="tooltip-content"] {
  position: absolute; z-index: 20; margin-top: 0.35rem; padding: 0.5rem 0.75rem;
  min-width: 8rem; background: var(--cronus-surface-floating);
  border: 1px solid var(--cronus-border); border-radius: var(--cronus-radius-md);
  color: var(--cronus-fg); font-size: 0.8125rem; box-shadow: var(--cronus-shadow-md, none);
}
[data-slot="password-input"], [data-slot="number-input"] {
  display: flex; flex-direction: column; gap: 0.35rem; font-size: 0.875rem;
}
[data-slot="password-input"] input, [data-slot="number-input"] input {
  height: 2.5rem; padding: 0 0.75rem; border-radius: var(--cronus-radius-md);
  border: 1px solid var(--cronus-border); background: var(--cronus-surface-inset);
  color: var(--cronus-fg); font: inherit;
}

[data-slot="field"] {
  display: flex; flex-direction: column; gap: 0.375rem;
}
[data-slot="field-label"] {
  font-size: 0.875rem; font-weight: 500; color: var(--cronus-fg);
  line-height: 1; user-select: none;
[data-slot="field-description"] {
  margin: 0; font-size: 0.75rem; color: var(--cronus-fg-secondary);
[data-slot="input-group"] {
  display: flex; height: 2.5rem; width: 100%; align-items: stretch;
  overflow: hidden; box-sizing: border-box;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
  font-size: 0.875rem;
[data-slot="input-group-addon"] {
  display: flex; flex-shrink: 0; align-items: center; gap: 0.375rem;
  padding: 0 0.75rem; color: var(--cronus-fg-tertiary);
  border-right: 1px solid var(--cronus-border); user-select: none; white-space: nowrap;
[data-slot="input-group"] [data-slot="input"] {
  height: 100%; width: auto; flex: 1; min-width: 0;
  border: 0; border-radius: 0; background: transparent; box-shadow: none;
[data-slot="rating"] {
  display: inline-flex; align-items: center; gap: 0.25rem;
[data-slot="rating-item"] {
  display: inline-flex; color: var(--cronus-fg-tertiary, var(--cronus-fg-secondary));
  font-size: 1.25rem; line-height: 1;
[data-slot="rating-item"][data-state="on"] {
  color: var(--cronus-warning, var(--cronus-primary));
[data-slot="rating-item"]::before {
  content: "★";
[data-slot="copy-button"] {
  display: inline-flex; align-items: center; justify-content: center; gap: 0.5rem;
  white-space: nowrap; border-radius: var(--cronus-radius-lg); font-weight: 500;
  line-height: 1; cursor: pointer; text-decoration: none;
  outline: none; border: 1px solid transparent;
  font-family: inherit; font-size: 0.875rem;
  height: 2.25rem; padding: 0 0.75rem;
  background: transparent; color: var(--cronus-fg-secondary);
  transition: background 150ms var(--ease-out-quart), box-shadow 150ms var(--ease-out-quart),
    transform 150ms var(--ease-out-quart), opacity 150ms var(--ease-out-quart), border-color 150ms;
[data-slot="copy-button"]:active { transform: scale(0.98); }
[data-slot="copy-button"]:hover { background: var(--cronus-surface-overlay); color: var(--cronus-fg); }
[data-slot="copy-button"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="fab"] {
  position: relative; display: inline-flex;
[data-slot="fab-button"] {
  display: inline-flex; align-items: center; justify-content: center;
  width: 3.5rem; height: 3.5rem; padding: 0; margin: 0;
  border: 0; border-radius: 9999px;
  background: var(--cronus-primary); color: var(--cronus-primary-foreground);
  box-shadow: var(--cronus-shadow-md, 0 4px 6px rgba(0,0,0,.2));
  cursor: pointer; outline: none; font: inherit;
  transition: transform 150ms var(--ease-out-quart), box-shadow 150ms var(--ease-out-quart),
    opacity 150ms var(--ease-out-quart);
[data-slot="fab-button"]:hover { opacity: 0.9; }
[data-slot="fab-button"]:active { transform: scale(0.95); }
[data-slot="fab-button"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="toggle-group"] {
  display: flex; align-items: center; gap: 0.25rem;
[data-slot="toggle-group-item"] {
  height: 2.5rem; padding: 0 0.75rem; border-radius: var(--cronus-radius-lg);
  border: 0; background: transparent; color: var(--cronus-fg-secondary);
  font-size: 0.875rem; font-weight: 500; font-family: inherit;
  cursor: pointer; outline: none;
  transition: background 150ms var(--ease-out-quart), color 150ms var(--ease-out-quart);
[data-slot="toggle-group-item"][data-state="on"] {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
[data-slot="toggle-group-item"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="metric"] {
  display: flex; flex-direction: column; gap: 0.25rem;
[data-slot="metric-label"] {
  font-size: 0.75rem; font-weight: 500; text-transform: uppercase;
  letter-spacing: 0.05em; color: var(--cronus-fg-tertiary);
[data-slot="metric-value"] {
  font-family: var(--cronus-font-display, inherit);
  font-size: 1.5rem; font-weight: 600; color: var(--cronus-fg);
  font-variant-numeric: tabular-nums;
[data-slot="avatar-group"] {
  display: flex; align-items: center;
[data-slot="avatar-group"] > * + * {
  margin-left: -0.5rem;
[data-slot="avatar-group"] [data-slot="avatar"] {
  width: 2.25rem; height: 2.25rem;
  box-shadow: 0 0 0 2px var(--cronus-surface-base);
[data-slot="avatar-group-overflow"] {
  position: relative; display: flex; flex-shrink: 0;
  align-items: center; justify-content: center;
  width: 2.25rem; height: 2.25rem; border-radius: 9999px;
  background: var(--cronus-surface-overlay); color: var(--cronus-fg-secondary);
  font-weight: 500; font-size: 0.875rem;
[data-slot="button-group"] {
  display: inline-flex;
[data-slot="button-group"][data-orientation="horizontal"], [data-slot="button-group"]:not([data-orientation]) {
  flex-direction: row;
[data-slot="button-group"][data-orientation="horizontal"] > *:not(:first-child),
[data-slot="button-group"]:not([data-orientation]) > *:not(:first-child) {
  border-top-left-radius: 0; border-bottom-left-radius: 0; margin-left: -1px;
[data-slot="button-group"][data-orientation="horizontal"] > *:not(:last-child),
[data-slot="button-group"]:not([data-orientation]) > *:not(:last-child) {
  border-top-right-radius: 0; border-bottom-right-radius: 0;
[data-slot="button-group"][data-orientation="vertical"] {
  flex-direction: column;
[data-slot="button-group"][data-orientation="vertical"] > *:not(:first-child) {
  border-top-left-radius: 0; border-top-right-radius: 0; margin-top: -1px;
[data-slot="button-group"][data-orientation="vertical"] > *:not(:last-child) {
  border-bottom-left-radius: 0; border-bottom-right-radius: 0;
[data-slot="button-group"] > *:focus-visible {
  position: relative; z-index: 10;
}
[data-slot="combobox"] {
  position: relative; display: flex; flex-direction: column; gap: 0.25rem;
}
[data-slot="combobox-trigger"] {
  display: inline-flex; align-items: center; justify-content: space-between;
  width: 100%; height: 2.5rem; padding: 0 0.75rem; box-sizing: border-box;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  background: transparent; color: var(--cronus-fg);
  font-size: 0.875rem; font-weight: 400; font-family: inherit; cursor: pointer;
[data-slot="combobox-trigger"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="combobox-content"] {
  display: flex; flex-direction: column; min-width: 8rem; padding: 0.25rem;
  border-radius: var(--cronus-radius-md); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-floating, var(--cronus-surface-overlay));
  color: var(--cronus-fg); box-shadow: var(--cronus-shadow-md, none);
[data-slot="combobox-item"] {
  display: flex; align-items: center; width: 100%;
  padding: 0.4rem 0.75rem; border: 0; border-radius: var(--cronus-radius-sm);
  font: inherit; text-align: left; cursor: pointer;
[data-slot="combobox-item"][aria-selected="true"] {
  background: var(--cronus-surface-overlay);
[data-slot="stepper"] {
  display: flex; width: 100%;
[data-slot="stepper"][data-orientation="horizontal"], [data-slot="stepper"]:not([data-orientation]) {
  flex-direction: row; align-items: center;
[data-slot="stepper"][data-orientation="vertical"] {
  flex-direction: column;
[data-slot="stepper-item"] {
  position: relative; display: flex; align-items: center;
[data-slot="stepper"][data-orientation="horizontal"] > [data-slot="stepper-item"]:not(:last-child) {
  flex: 1;
[data-slot="stepper-trigger"] {
  display: inline-flex; align-items: center; gap: 0.75rem;
  border: 0; background: transparent; color: inherit;
  font: inherit; text-align: left; cursor: pointer; outline: none;
[data-slot="stepper-title"] {
  font-size: 0.875rem; font-weight: 500; line-height: 1; color: var(--cronus-fg);
[data-slot="stepper-item"][data-state="upcoming"] [data-slot="stepper-title"] {
  color: var(--cronus-fg-tertiary);
[data-slot="input-otp"] {
  display: flex; align-items: center; gap: 0.5rem;
[data-slot="input-otp-group"] {
  display: flex; align-items: center;
[data-slot="input-otp-slot"] {
  position: relative; display: flex; align-items: center; justify-content: center;
  height: 2.5rem; width: 2.5rem; box-sizing: border-box;
  border-top: 1px solid var(--cronus-border);
  border-right: 1px solid var(--cronus-border);
  border-bottom: 1px solid var(--cronus-border);
  font-size: 0.875rem; color: var(--cronus-fg);
  font-variant-numeric: tabular-nums;
[data-slot="input-otp-slot"]:first-child {
  border-left: 1px solid var(--cronus-border);
  border-top-left-radius: var(--cronus-radius-lg);
  border-bottom-left-radius: var(--cronus-radius-lg);
[data-slot="input-otp-slot"]:last-child {
  border-top-right-radius: var(--cronus-radius-lg);
  border-bottom-right-radius: var(--cronus-radius-lg);
[data-slot="file-dropzone"] {
  position: relative; display: flex; flex-direction: column;
  align-items: center; justify-content: center; gap: 0.5rem;
  cursor: pointer; box-sizing: border-box;
  border-radius: var(--cronus-radius-xl);
  border: 2px dashed var(--cronus-border);
  background: var(--cronus-surface-inset);
  padding: 2.5rem 1.5rem; text-align: center;
  font-size: 0.875rem; color: var(--cronus-fg-secondary);
[data-slot="file-dropzone"][aria-disabled="true"] {
  cursor: not-allowed; opacity: 0.5;
[data-slot="file-dropzone"] .sr-only {
  position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;
  overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0;
[data-slot="file-dropzone"]:has(:focus-visible) {
  outline: 2px solid var(--cronus-ring, var(--cronus-primary));
  outline-offset: 2px;
button:has(+ [data-slot="popover-content"]) {
  font: inherit; cursor: pointer; color: var(--cronus-fg);
  background: transparent; border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-lg);
  height: 2.5rem; padding: 0 1rem; font-size: 0.875rem;
[data-slot="popover-content"] {
  z-index: 50; width: 18rem; box-sizing: border-box;
  padding: 0.75rem; outline: none;
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
  font-size: 0.875rem; box-shadow: var(--cronus-shadow-lg, none);
button:has(+ [data-slot="hover-card-content"]) {
[data-slot="hover-card-content"] {
  z-index: 50; width: 16rem; box-sizing: border-box;
[data-slot="dropdown-menu"] {
  position: relative; display: inline-flex; flex-direction: column; gap: 0.25rem;
[data-slot="dropdown-menu"] > button {
  display: inline-flex; align-items: center; justify-content: center;
  height: 2.5rem; padding: 0 1rem;
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
  font: inherit; font-size: 0.875rem; font-weight: 500; cursor: pointer;
[data-slot="dropdown-menu-content"] {
  min-width: 8rem; overflow: hidden;
  padding: 0.25rem; box-shadow: var(--cronus-shadow-lg, none);
[data-slot="dropdown-menu-item"] {
  border-radius: var(--cronus-radius-md); padding: 0.375rem 0.5rem;
  font-size: 0.875rem; cursor: default;
[data-slot="dropdown-menu-item"]:hover {
[data-slot="collapsible"] {
  display: flex; flex-direction: column; gap: 0.25rem;
[data-slot="collapsible"] > button {
  display: inline-flex; align-items: center;
[data-slot="collapsible-content"] {
  overflow: hidden; font-size: 0.875rem; color: var(--cronus-fg-secondary);
[data-slot="collapsible-content"][data-state="open"] {
  display: block;
[data-slot="mode-toggle"] {
  display: inline-flex; align-items: center; justify-content: center; flex-shrink: 0;
  width: 2.25rem; height: 2.25rem; padding: 0; margin: 0;
  border: 0; border-radius: var(--cronus-radius-lg);
  background: transparent; color: var(--cronus-fg-secondary);
  cursor: pointer; outline: none;
  transition: background 150ms var(--ease-out-quart), color 150ms var(--ease-out-quart);
[data-slot="mode-toggle"]:hover {
[data-slot="mode-toggle"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="mode-toggle"] svg { width: 1.25rem; height: 1.25rem; }
[data-slot="mode-toggle-core"] {
  transform-origin: center; transform: scale(1);
  transition: transform 300ms var(--ease-out-quart);
[data-slot="mode-toggle"][data-mode="dark"] [data-slot="mode-toggle-core"] {
  transform: scale(1.75);
[data-slot="mode-toggle-rays"] {
  transform-origin: center; transform: scale(1); opacity: 1;
  transition: transform 300ms var(--ease-out-quart), opacity 300ms var(--ease-out-quart);
[data-slot="mode-toggle"][data-mode="dark"] [data-slot="mode-toggle-rays"] {
  transform: scale(0.5); opacity: 0;
[data-slot="mode-toggle-crescent"] {
  transform: translateX(0);
[data-slot="mode-toggle"][data-mode="dark"] [data-slot="mode-toggle-crescent"] {
  transform: translateX(-7px);
}
[data-slot="command"] {
  display: flex; flex-direction: column; overflow: hidden; width: 100%;
  box-sizing: border-box; border-radius: var(--cronus-radius-lg);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
}
[data-slot="command-input"] {
  display: flex; width: 100%; height: 2.5rem; box-sizing: border-box;
  border: 0; border-bottom: 1px solid var(--cronus-border);
  background: transparent; color: var(--cronus-fg);
  padding: 0 0.75rem; font-size: 0.875rem; font-family: inherit; outline: none;
[data-slot="command-input"]::placeholder { color: var(--cronus-fg-tertiary); }
[data-slot="command-list"] {
  max-height: 20rem; overflow-y: auto; overflow-x: hidden; padding: 0.25rem;
[data-slot="command-item"] {
  display: flex; align-items: center; gap: 0.5rem;
  border-radius: var(--cronus-radius-md); padding: 0.375rem 0.5rem;
  font-size: 0.875rem; cursor: default;
[data-slot="command-item"]:hover {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
[data-slot="menubar"] {
  display: flex; align-items: center; gap: 0.25rem;
  border-radius: var(--cronus-radius-md); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-floating); padding: 0.25rem;
  box-shadow: var(--cronus-shadow-xs, none);
[data-slot="menubar-trigger"] {
  display: flex; align-items: center; border: 0;
  border-radius: var(--cronus-radius-md);
  padding: 0.25rem 0.75rem; font: inherit; font-size: 0.875rem; font-weight: 500;
  cursor: default; outline: none;
[data-slot="menubar-trigger"][data-state="open"],
[data-slot="menubar-trigger"]:hover {
[data-slot="menubar-content"] {
  min-width: 12rem; overflow: hidden;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  padding: 0.25rem; box-shadow: var(--cronus-shadow-lg, none);
[data-slot="menubar-item"] {
[data-slot="menubar-item"]:hover {
button:has(+ [data-slot="context-menu-content"]) {
  font: inherit; cursor: pointer; color: var(--cronus-fg);
  background: transparent; border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-lg);
  height: 2.5rem; padding: 0 1rem; font-size: 0.875rem;
[data-slot="context-menu-content"] {
  z-index: 50; min-width: 8rem; overflow: hidden; box-sizing: border-box;
[data-slot="context-menu-item"] {
[data-slot="context-menu-item"]:hover {
[data-slot="drawer"] {
  display: flex; flex-direction: column; gap: 0.5rem;
[data-slot="drawer"] > button {
  display: inline-flex; align-items: center; justify-content: center;
  height: 2.5rem; padding: 0 1rem;
  font: inherit; font-size: 0.875rem; font-weight: 500; cursor: pointer;
[data-slot="drawer-content"] {
  z-index: 50; display: flex; flex-direction: column;
  border-radius: var(--cronus-radius-xl) var(--cronus-radius-xl) 0 0;
  box-shadow: var(--cronus-shadow-lg, none);
[data-slot="drawer-title"] {
  font-size: 1.125rem; font-weight: 600; color: var(--cronus-fg);
  font-family: var(--cronus-font-display, inherit);
  padding: 1rem 1rem 0;
[data-slot="drawer-description"] {
  font-size: 0.875rem; color: var(--cronus-fg-secondary);
  padding: 0 1rem 1rem;
button:has(+ [data-slot="sheet-content"]) {
[data-slot="sheet-content"] {
  z-index: 50; display: flex; flex-direction: column; gap: 1rem;
  padding: 1.5rem; box-sizing: border-box;
  width: 75%; max-width: 24rem;
[data-slot="sheet-title"] {
[data-slot="sheet-description"] {
[data-slot="calendar"] {
  display: block; padding: 0.75rem; box-sizing: border-box;
  background: var(--cronus-surface-raised); color: var(--cronus-fg);
[data-slot="calendar"] table {
  width: 100%; border-collapse: collapse;
[data-slot="calendar"] caption {
  font-size: 0.875rem; font-weight: 500; padding-bottom: 0.5rem;
  color: var(--cronus-fg);
[data-slot="calendar"] th {
  width: 2.25rem; font-size: 0.75rem; font-weight: 400;
  color: var(--cronus-fg-tertiary); text-align: center;
[data-slot="calendar"] td {
  width: 2.25rem; height: 2.25rem; text-align: center;
  font-size: 0.875rem; color: var(--cronus-fg);
[data-slot="date-picker-trigger"] {
  display: inline-flex; align-items: center; justify-content: flex-start; gap: 0.5rem;
  width: 15rem; height: 2.5rem; padding: 0 0.75rem; box-sizing: border-box;
  font-size: 0.875rem; font-weight: 400; font-family: inherit; cursor: pointer;
[data-slot="date-picker-trigger"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="date-picker-trigger"][aria-invalid="true"] {
  border-color: var(--cronus-error);
[data-slot="date-picker-trigger"] svg { width: 1rem; height: 1rem; flex-shrink: 0; opacity: 0.7; }
[data-slot="date-picker-content"] {
  z-index: 50; width: auto; box-sizing: border-box;
  padding: 0.75rem; outline: none;
  font-size: 0.875rem; box-shadow: var(--cronus-shadow-lg, none);
[data-slot="date-picker-calendar"] {
  display: flex; flex-direction: column; gap: 0.25rem;
[data-slot="date-picker-calendar"] [role="row"] {
  display: grid; grid-template-columns: repeat(7, 2.25rem); justify-content: center;
[data-slot="date-picker-weekday"] {
  font-size: 0.75rem; font-weight: 400; color: var(--cronus-fg-tertiary);
  text-align: center;
[data-slot="date-picker-day"] {
  width: 2.25rem; height: 2.25rem; padding: 0; border: 0;
  font: inherit; font-size: 0.875rem; cursor: default;
[data-slot="date-picker-day"][aria-selected="true"] {
  background: var(--cronus-primary); color: var(--cronus-primary-foreground);
[data-slot="time-picker"] {
  display: inline-flex; flex-direction: column; gap: 0.25rem;
[data-slot="time-picker"] > button {
[data-slot="time-picker"] > button:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="time-picker"] > button svg { width: 1rem; height: 1rem; flex-shrink: 0; color: var(--cronus-fg-tertiary); }
[data-slot="time-picker-content"] {
[data-slot="time-picker-content"] > div {
  display: flex; align-items: stretch; justify-content: center; gap: 0.25rem;
[data-slot="time-picker-column"] {
  display: flex; flex-direction: column; align-items: center; gap: 0.35rem;
[data-slot="time-picker-column"] > span {
  font-size: 0.6875rem; font-weight: 500; letter-spacing: 0.08em;
  text-transform: uppercase; color: var(--cronus-fg-tertiary);
[data-slot="time-picker-option"] {
  display: flex; align-items: center; justify-content: center;
  width: 3.5rem; height: 2.25rem; border-radius: var(--cronus-radius-md);
  font-size: 0.875rem; font-variant-numeric: tabular-nums; cursor: default;
  color: var(--cronus-fg-secondary);
[data-slot="time-picker-option"][aria-selected="true"] {
  background: color-mix(in oklch, var(--cronus-primary), black 30%);
  color: var(--cronus-primary-foreground); font-weight: 600;
[data-slot="time-picker-now"], [data-slot="time-picker-done"] {
  font: inherit; font-size: 0.75rem; cursor: default;
  background: transparent; color: var(--cronus-fg); padding: 0.25rem 0.5rem;
[data-slot="date-range-picker-trigger"] {
  width: 18.75rem; height: 2.5rem; padding: 0 0.75rem; box-sizing: border-box;
[data-slot="date-range-picker-trigger"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="date-range-picker-trigger"] svg { width: 1rem; height: 1rem; flex-shrink: 0; opacity: 0.7; }
[data-slot="date-range-picker-content"] {
  padding: 0; outline: none;
  display: flex; flex-direction: row;
[data-slot="date-range-picker-content"] legend {
  position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0, 0, 0, 0);
[data-slot="date-range-picker-presets"] {
  margin: 0; padding: 0.5rem; border: 0;
  border-right: 1px solid var(--cronus-border);
[data-slot="date-range-picker-preset"] {
  font: inherit; font-size: 0.75rem; text-align: left; cursor: default;
  border: 0; background: transparent; color: var(--cronus-fg);
  padding: 0.25rem 0.5rem; border-radius: var(--cronus-radius-md);
[data-slot="date-range-picker-calendar"] {
  margin: 0; padding: 0.75rem; border: 0;
  display: flex; flex-direction: row; gap: 1rem;
[data-slot="date-range-picker-month"] {
[data-slot="date-range-picker-month"] [role="row"] {
[data-slot="date-range-picker-weekday"] {
[data-slot="date-range-picker-day"] {
[data-slot="date-range-picker-day"][data-range="start"],
[data-slot="date-range-picker-day"][data-range="end"] {
[data-slot="date-range-picker-day"][data-range="middle"] {
  background: var(--cronus-surface-overlay); border-radius: 0;
}
[data-slot="sparkline"] {
  display: inline-block; overflow: visible; vertical-align: middle;
  color: var(--cronus-primary);
}
[data-slot="sparkline"][data-tone="success"] { color: var(--cronus-success); }
[data-slot="sparkline"][data-tone="warning"] { color: var(--cronus-warning); }
[data-slot="sparkline"][data-tone="error"] { color: var(--cronus-error); }
[data-slot="sparkline"][data-tone="info"] { color: var(--cronus-info); }
[data-slot="sparkline"][data-tone="fg"] { color: var(--cronus-fg); }
[data-slot="sparkline-line"] { fill: none; stroke: currentColor; }
[data-slot="sparkline-area"] { stroke: none; }
[data-slot="pie-chart"] {
  display: flex; align-items: center; justify-content: center;
  width: 100%; height: 16rem;
}
[data-slot="pie-chart"] svg { width: 12rem; height: 12rem; }
[data-slot="data-table"] {
  display: flex; flex-direction: column; gap: 0.75rem;
  overflow: auto; color: var(--cronus-fg);
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
