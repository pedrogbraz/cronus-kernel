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

[data-slot="catalog"] {
  width: 100%;
  padding: 0.25rem 0 3.5rem;
}
[data-slot="catalog"] .sr-only {
  position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;
  overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0;
}
[data-slot="catalog-header"] {
  padding-bottom: 2rem;
  border-bottom: 1px solid var(--cronus-border);
}
[data-slot="catalog-eyebrow"] {
  margin: 0 0 0.75rem;
  font-size: 0.75rem; font-weight: 500;
  letter-spacing: 0.16em; text-transform: uppercase;
  color: var(--cronus-fg-tertiary, var(--cronus-fg-secondary));
}
[data-slot="catalog-title-row"] {
  display: flex; flex-wrap: wrap; align-items: flex-end;
  justify-content: space-between; gap: 1rem;
}
[data-slot="catalog-header"] h1 {
  margin: 0;
  font-family: var(--cronus-font-display, inherit);
  font-size: clamp(2rem, 4vw, 3rem); font-weight: 400;
  letter-spacing: -0.03em; line-height: 1.1;
  color: var(--cronus-fg);
}
[data-slot="catalog-lead"] {
  max-width: 40rem; margin: 1rem 0 0;
  font-size: 1.0625rem; line-height: 1.7;
  color: var(--cronus-fg-secondary);
}
[data-slot="catalog-home"] {
  display: inline-flex; align-items: center; margin-top: 1.25rem;
  font-size: 0.875rem; color: var(--cronus-fg-secondary); text-decoration: none;
  transition: color 200ms var(--cronus-ease);
}
[data-slot="catalog-home"]:hover { color: var(--cronus-fg); }
[data-slot="catalog-stats"] {
  display: flex; flex-wrap: wrap; gap: 0.5rem;
}
[data-slot="catalog-stats"] span {
  display: inline-flex; align-items: center; gap: 0.4rem;
  height: 2rem; padding: 0 0.75rem;
  border-radius: 9999px; border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-raised, var(--cronus-surface-overlay));
  font-size: 0.8125rem; color: var(--cronus-fg-tertiary, var(--cronus-fg-secondary));
}
[data-slot="catalog-stats"] strong {
  font-weight: 600; color: var(--cronus-fg);
  font-variant-numeric: tabular-nums;
}
[data-slot="catalog-toolbar"] {
  position: sticky; top: 0; z-index: 20;
  display: flex; flex-direction: column; gap: 0.75rem;
  padding: 1rem 0;
  margin: 0 0 0.25rem;
  background: color-mix(in oklch, var(--cronus-surface-base) 82%, transparent);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid var(--cronus-border);
}
@media (min-width: 640px) {
  [data-slot="catalog-toolbar"] {
    flex-direction: row; align-items: center;
  }
}
[data-slot="catalog-search-wrap"] { display: block; width: 100%; max-width: 20rem; }
[data-slot="catalog-search"] {
  width: 100%; height: 2.5rem; padding: 0 0.875rem; box-sizing: border-box;
  border-radius: var(--cronus-radius-lg);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
  font: inherit; font-size: 0.875rem; outline: none;
  transition: border-color 200ms var(--cronus-ease), box-shadow 200ms var(--cronus-ease);
}
[data-slot="catalog-search"]::placeholder {
  color: var(--cronus-fg-tertiary, var(--cronus-fg-secondary));
}
[data-slot="catalog-search"]:focus {
  border-color: var(--cronus-ring, var(--cronus-primary));
  box-shadow: 0 0 0 3px color-mix(in oklch, var(--cronus-ring, var(--cronus-primary)) 25%, transparent);
}
[data-slot="catalog-nav"] {
  display: flex; flex-wrap: wrap; gap: 0.5rem; flex: 1;
}
[data-slot="catalog-nav"] a {
  display: inline-flex; align-items: center;
  height: 2rem; padding: 0 0.75rem;
  border-radius: 9999px; border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-raised, var(--cronus-surface-overlay));
  color: var(--cronus-fg-secondary); font-size: 0.8125rem;
  text-decoration: none;
  transition: border-color 200ms var(--cronus-ease), color 200ms var(--cronus-ease);
}
[data-slot="catalog-nav"] a:hover {
  color: var(--cronus-fg);
  border-color: color-mix(in oklch, var(--cronus-fg) 22%, var(--cronus-border));
}
[data-slot="catalog-nav"] a[hidden] { display: none; }
[data-slot="catalog-section"] {
  scroll-margin-top: 5.5rem;
  padding-top: 2.25rem;
}
[data-slot="catalog-section"] + [data-slot="catalog-section"] {
  margin-top: 1.25rem;
  border-top: 1px solid var(--cronus-border);
}
[data-slot="catalog-section"] > h2 {
  margin: 0 0 1.25rem;
  font-family: var(--cronus-font-display, inherit);
  font-size: 1.5rem; font-weight: 400; letter-spacing: -0.02em;
  color: var(--cronus-fg);
}
[data-slot="catalog-grid"] {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(16.5rem, 1fr));
  gap: 1rem;
}
[data-slot="catalog-specimen"] {
  display: flex; flex-direction: column;
  border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-xl);
  background: var(--cronus-surface-raised, var(--cronus-surface-overlay));
  overflow: visible;
  animation: catalog-in 400ms var(--cronus-ease) both;
  transition: border-color 200ms var(--cronus-ease);
}
[data-slot="catalog-specimen"][hidden],
[data-slot="catalog-section"][hidden] {
  display: none;
}
[data-slot="catalog-specimen"]:hover {
  border-color: color-mix(in oklch, var(--cronus-fg) 18%, var(--cronus-border));
}
[data-slot="catalog-specimen"][data-wide="true"] {
  grid-column: 1 / -1;
}
[data-slot="catalog-specimen"] > header {
  display: flex; align-items: baseline; justify-content: space-between;
  gap: 0.75rem;
  padding: 0.75rem 1.15rem;
  border-bottom: 1px solid var(--cronus-border);
}
[data-slot="catalog-family"] {
  font-size: 0.875rem; font-weight: 500; color: var(--cronus-fg);
}
[data-slot="catalog-meta"] {
  font-size: 0.75rem;
  font-family: var(--cronus-font-mono, ui-monospace, monospace);
  color: var(--cronus-fg-tertiary, var(--cronus-fg-secondary));
}
[data-slot="catalog-canvas"] {
  display: flex; flex-wrap: wrap; align-items: center;
  gap: 0.75rem;
  min-height: 4.75rem;
  padding: 1.15rem 1.15rem 1.35rem;
}
[data-slot="catalog-specimen"][data-wide="true"] [data-slot="catalog-canvas"] {
  display: block;
}
@keyframes catalog-in {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: none; }
}
@media (prefers-reduced-motion: reduce) {
  [data-slot="catalog-specimen"] { animation: none; }
}
@media (max-width: 639px) {
  [data-slot="catalog-grid"] {
    grid-template-columns: 1fr;
  }
  [data-slot="catalog-specimen"][data-wide="true"] {
    grid-column: auto;
  }
}
[data-slot="catalog-specimen"] {
  content-visibility: auto;
  contain-intrinsic-size: auto 12rem;
}

/* Native overlays — closed until invoked. Popover API + dialog. */
[popover] {
  inset: unset; margin: 0;
  border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-xl);
  background: var(--cronus-surface-floating, var(--cronus-surface-overlay));
  color: var(--cronus-fg);
  padding: 0.35rem;
  box-shadow: var(--cronus-shadow-lg, 0 16px 40px rgba(0,0,0,.35));
}
@supports (top: anchor(bottom)) {
}
  [popover]:not([data-slot="sheet-content"]) {
    position: absolute;
    top: calc(anchor(bottom) + 0.35rem);
    left: anchor(left);
  }
}
@supports not (top: anchor(bottom)) {
}
  [popover]:not([data-slot="sheet-content"]) {
    position: fixed;
    top: 18%;
    left: 50%;
    translate: -50% 0;
  }
}
[popover]:popover-open {
  animation: cronus-pop-in 180ms var(--cronus-ease) both;
}
@starting-style {
  [popover]:popover-open {
    opacity: 0;
    transform: translateY(6px) scale(0.98);
  }
}
[data-slot="dropdown-menu"],
[data-slot="combobox"],
[data-slot="date-picker"],
[data-slot="time-picker"],
[data-slot="hover-card"] {
  position: relative;
  display: inline-flex;
  flex-direction: column;
  align-items: flex-start;
}
[data-slot="dropdown-menu-trigger"],
[data-slot="hover-card"] > button,
button:has(+ [popover][data-slot="popover-content"]),
button:has(+ [popover][data-slot="sheet-content"]) {
  display: inline-flex; align-items: center; justify-content: center;
  height: 2.5rem; padding: 0 1rem;
  border-radius: var(--cronus-radius-lg);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-overlay);
  color: var(--cronus-fg);
  font: inherit; font-size: 0.875rem; cursor: pointer;
  transition: background 150ms var(--cronus-ease), border-color 150ms var(--cronus-ease);
}
[data-slot="dropdown-menu-item"],
[data-slot="combobox-item"] {
  transition: background 120ms var(--cronus-ease);
}
[data-slot="dropdown-menu-item"]:hover,
[data-slot="combobox-item"]:hover {
  background: var(--cronus-surface-overlay);
}
[data-slot="sheet-content"][popover] {
  height: 100dvh; width: min(24rem, 92vw);
  margin: 0; padding: 1.5rem;
  border-radius: var(--cronus-radius-xl) 0 0 var(--cronus-radius-xl);
  position: fixed; inset: 0 0 0 auto;
  animation: cronus-slide-in-right 280ms var(--cronus-ease) both;
}
[data-slot="hover-card-content"] {
  position: absolute; z-index: 50; top: calc(100% + 0.35rem); left: 0;
  width: 16rem; padding: 0.75rem; box-sizing: border-box;
  border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-xl);
  background: var(--cronus-surface-floating, var(--cronus-surface-overlay));
  color: var(--cronus-fg);
  box-shadow: var(--cronus-shadow-lg, none);
  opacity: 0; visibility: hidden; pointer-events: none;
  transform: translateY(4px) scale(0.98);
  transition: opacity 160ms var(--cronus-ease), transform 160ms var(--cronus-ease), visibility 160ms;
}
[data-slot="hover-card"]:hover [data-slot="hover-card-content"],
[data-slot="hover-card"]:focus-within [data-slot="hover-card-content"] {
  opacity: 1; visibility: visible; pointer-events: auto; transform: none;
}
dialog[data-slot="dialog-content"] {
  position: fixed; inset: 0; margin: auto;
  width: min(24rem, calc(100vw - 2rem));
  height: fit-content;
  border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-xl);
  background: var(--cronus-surface-floating, var(--cronus-surface-overlay));
  color: var(--cronus-fg);
  padding: 0;
  box-shadow: var(--cronus-shadow-lg, 0 16px 40px rgba(0,0,0,.4));
  animation: cronus-pop-in 200ms var(--cronus-ease) both;
}
dialog[data-slot="dialog-content"]::backdrop {
  background: color-mix(in oklch, black 55%, transparent);
  animation: cronus-overlay-in 200ms var(--cronus-ease) both;
}
@media (prefers-reduced-motion: reduce) {
  [popover]:popover-open,
  [data-slot="sheet-content"][popover],
  dialog[data-slot="dialog-content"],
  dialog[data-slot="dialog-content"]::backdrop,
  [data-slot="hover-card-content"] {
    animation: none; transition: none;
  }
}
[data-slot="checkbox-text"], [data-slot="switch-text"] {
  font-size: 0.875rem; color: var(--cronus-fg); line-height: 1;
}

[data-slot="date-picker-trigger"],
[data-slot="time-picker-trigger"] {
  display: inline-flex; align-items: center; justify-content: flex-start; gap: 0.5rem;
  width: 15rem; height: 2.5rem; padding: 0 0.75rem; box-sizing: border-box;
  border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-lg);
  background: var(--cronus-surface-inset);
  color: var(--cronus-fg);
  font: inherit; font-size: 0.875rem; font-weight: 400;
  cursor: pointer; outline: none;
  transition: border-color 150ms var(--cronus-ease), box-shadow 150ms var(--cronus-ease);
}
[data-slot="date-picker-trigger"]:hover,
[data-slot="time-picker-trigger"]:hover {
  border-color: color-mix(in oklch, var(--cronus-fg) 22%, var(--cronus-border));
}
[data-slot="date-picker-trigger"]:focus-visible,
[data-slot="time-picker-trigger"]:focus-visible {
  border-color: var(--cronus-ring, var(--cronus-primary));
  box-shadow: 0 0 0 3px color-mix(in oklch, var(--cronus-ring, var(--cronus-primary)) 25%, transparent);
}
[data-slot="date-picker-trigger"] svg,
[data-slot="time-picker-trigger"] svg {
  width: 1rem; height: 1rem; flex-shrink: 0; opacity: 0.7;
}
[data-slot="date-picker-content"][popover],
[data-slot="time-picker-content"][popover] {
  z-index: 50;
  padding: 0.75rem 0.85rem 0.85rem;
  min-width: 17.75rem;
  box-sizing: border-box;
}
[data-slot="date-picker-calendar"] {
  display: flex; flex-direction: column; gap: 0.65rem;
}
[data-slot="date-picker-caption"] {
  font-size: 0.875rem; font-weight: 500; color: var(--cronus-fg);
  text-align: center; letter-spacing: -0.02em;
  padding: 0.15rem 0 0.1rem;
}
[data-slot="date-picker-calendar"] [role="row"] {
  display: grid; grid-template-columns: repeat(7, 2.25rem);
  justify-content: center; gap: 0.1rem;
}
[data-slot="date-picker-weekday"] {
  display: flex; align-items: center; justify-content: center;
  height: 1.75rem;
  font-size: 0.6875rem; font-weight: 500; letter-spacing: 0.04em;
  text-transform: uppercase; color: var(--cronus-fg-tertiary);
}
[data-slot="date-picker-day"] {
  display: inline-flex; align-items: center; justify-content: center;
  width: 2.25rem; height: 2.25rem; padding: 0; margin: 0;
  border: 0; border-radius: var(--cronus-radius-md);
  background: transparent; color: var(--cronus-fg);
  font: inherit; font-size: 0.8125rem; font-variant-numeric: tabular-nums;
  cursor: pointer;
  transition: background 120ms var(--cronus-ease), color 120ms var(--cronus-ease);
}
[data-slot="date-picker-day"]:hover {
  background: var(--cronus-surface-overlay);
}
[data-slot="date-picker-day"][aria-selected="true"] {
  background: var(--cronus-primary); color: var(--cronus-primary-foreground);
  font-weight: 500;
}
[data-slot="time-picker-columns"] {
  display: flex; align-items: flex-start; justify-content: center; gap: 0;
}
[data-slot="time-picker-column"] {
  display: flex; flex-direction: column; align-items: stretch;
  min-width: 3.75rem; max-height: 14rem; overflow-y: auto;
  padding: 0 0.25rem 0.25rem;
  scrollbar-width: thin;
}
[data-slot="time-picker-column"] + [data-slot="time-picker-column"] {
  border-inline-start: 1px solid var(--cronus-border);
}
[data-slot="time-picker-column"] > span {
  position: sticky; top: 0; z-index: 1;
  display: flex; align-items: center; justify-content: center;
  height: 1.75rem; margin-bottom: 0.2rem;
  font-size: 0.6875rem; font-weight: 500; letter-spacing: 0.08em;
  text-transform: uppercase; color: var(--cronus-fg-tertiary);
  background: var(--cronus-surface-floating, var(--cronus-surface-overlay));
}
[data-slot="time-picker-option"] {
  display: flex; align-items: center; justify-content: center;
  height: 2rem; border-radius: var(--cronus-radius-md);
  font-size: 0.875rem; font-variant-numeric: tabular-nums;
  color: var(--cronus-fg-secondary); cursor: pointer;
  transition: background 120ms var(--cronus-ease), color 120ms var(--cronus-ease);
}
[data-slot="time-picker-option"]:hover {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="time-picker-option"][aria-selected="true"] {
  background: var(--cronus-primary); color: var(--cronus-primary-foreground);
  font-weight: 500;
}
[data-slot="time-picker-footer"] {
  display: flex; justify-content: flex-end; gap: 0.35rem;
  margin-top: 0.6rem; padding-top: 0.6rem;
  border-top: 1px solid var(--cronus-border);
}
[data-slot="time-picker-now"], [data-slot="time-picker-done"] {
  display: inline-flex; align-items: center; justify-content: center;
  height: 2rem; padding: 0 0.75rem;
  border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-md);
  background: transparent; color: var(--cronus-fg);
  font: inherit; font-size: 0.75rem; font-weight: 500; cursor: pointer;
}
[data-slot="time-picker-done"] {
  background: var(--cronus-primary); color: var(--cronus-primary-foreground);
  border-color: transparent;
}

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

[data-slot="card"], [data-slot="spotlight-card"] {
  background: var(--cronus-surface-raised); border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-xl); box-shadow: var(--cronus-shadow-xs, none);
}
[data-slot="spotlight-card"] {
  position: relative; overflow: hidden;
  padding: 1.5rem; color: var(--cronus-fg);
}
[data-slot="spotlight-card"]::after {
  content: "";
  pointer-events: none;
  position: absolute; inset: -1px;
  opacity: 0;
  transition: opacity 300ms var(--cronus-ease);
  background: radial-gradient(320px circle at var(--spot-x, 50%) var(--spot-y, 50%), color-mix(in oklch, var(--cronus-primary) 14%, transparent), transparent 72%);
}
[data-slot="spotlight-card"]:hover::after { opacity: 1; }
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
}
[data-slot="field-description"] {
  margin: 0; font-size: 0.75rem; color: var(--cronus-fg-secondary);
}
[data-slot="input-group"] {
  display: flex; height: 2.5rem; width: 100%; align-items: stretch;
  overflow: hidden; box-sizing: border-box;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
  font-size: 0.875rem;
}
[data-slot="input-group-addon"] {
  display: flex; flex-shrink: 0; align-items: center; gap: 0.375rem;
  padding: 0 0.75rem; color: var(--cronus-fg-tertiary);
  border-right: 1px solid var(--cronus-border); user-select: none; white-space: nowrap;
}
[data-slot="input-group"] [data-slot="input"] {
  height: 100%; width: auto; flex: 1; min-width: 0;
  border: 0; border-radius: 0; background: transparent; box-shadow: none;
}
[data-slot="rating"] {
  display: inline-flex; align-items: center; gap: 0.25rem;
}
[data-slot="rating-item"] {
  display: inline-flex; color: var(--cronus-fg-tertiary, var(--cronus-fg-secondary));
  font-size: 1.25rem; line-height: 1;
}
[data-slot="rating-item"][data-state="on"] {
  color: var(--cronus-warning, var(--cronus-primary));
}
[data-slot="rating-item"]::before {
  content: "★";
}
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
}
[data-slot="copy-button"]:active { transform: scale(0.98); }
[data-slot="copy-button"]:hover { background: var(--cronus-surface-overlay); color: var(--cronus-fg); }
[data-slot="copy-button"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="fab"] {
  position: relative; display: inline-flex;
}
[data-slot="fab-button"] {
  display: inline-flex; align-items: center; justify-content: center;
  width: 3.5rem; height: 3.5rem; padding: 0; margin: 0;
  border: 0; border-radius: 9999px;
  background: var(--cronus-primary); color: var(--cronus-primary-foreground);
  box-shadow: var(--cronus-shadow-md, 0 4px 6px rgba(0,0,0,.2));
  cursor: pointer; outline: none; font: inherit;
  transition: transform 150ms var(--ease-out-quart), box-shadow 150ms var(--ease-out-quart),
    opacity 150ms var(--ease-out-quart);
}
[data-slot="fab-button"]:hover { opacity: 0.9; }
[data-slot="fab-button"]:active { transform: scale(0.95); }
[data-slot="fab-button"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="toggle-group"] {
  display: flex; align-items: center; gap: 0.25rem;
}
[data-slot="toggle-group-item"] {
  height: 2.5rem; padding: 0 0.75rem; border-radius: var(--cronus-radius-lg);
  border: 0; background: transparent; color: var(--cronus-fg-secondary);
  font-size: 0.875rem; font-weight: 500; font-family: inherit;
  cursor: pointer; outline: none;
  transition: background 150ms var(--ease-out-quart), color 150ms var(--ease-out-quart);
}
[data-slot="toggle-group-item"][data-state="on"] {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="toggle-group-item"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="metric"] {
  display: flex; flex-direction: column; gap: 0.25rem;
}
[data-slot="metric-label"] {
  font-size: 0.75rem; font-weight: 500; text-transform: uppercase;
  letter-spacing: 0.05em; color: var(--cronus-fg-tertiary);
}
[data-slot="metric-value"] {
  font-family: var(--cronus-font-display, inherit);
  font-size: 1.5rem; font-weight: 600; color: var(--cronus-fg);
  font-variant-numeric: tabular-nums;
}
[data-slot="avatar-group"] {
  display: flex; align-items: center;
}
[data-slot="avatar-group"] > * + * {
  margin-left: -0.5rem;
}
[data-slot="avatar-group"] [data-slot="avatar"] {
  width: 2.25rem; height: 2.25rem;
  box-shadow: 0 0 0 2px var(--cronus-surface-base);
}
[data-slot="avatar-group-overflow"] {
  position: relative; display: flex; flex-shrink: 0;
  align-items: center; justify-content: center;
  width: 2.25rem; height: 2.25rem; border-radius: 9999px;
  background: var(--cronus-surface-overlay); color: var(--cronus-fg-secondary);
  font-weight: 500; font-size: 0.875rem;
}
[data-slot="button-group"] {
  display: inline-flex;
}
[data-slot="button-group"][data-orientation="horizontal"], [data-slot="button-group"]:not([data-orientation]) {
  flex-direction: row;
}
[data-slot="button-group"][data-orientation="horizontal"] > *:not(:first-child),
[data-slot="button-group"]:not([data-orientation]) > *:not(:first-child) {
  border-top-left-radius: 0; border-bottom-left-radius: 0; margin-left: -1px;
}
[data-slot="button-group"][data-orientation="horizontal"] > *:not(:last-child),
[data-slot="button-group"]:not([data-orientation]) > *:not(:last-child) {
  border-top-right-radius: 0; border-bottom-right-radius: 0;
}
[data-slot="button-group"][data-orientation="vertical"] {
  flex-direction: column;
}
[data-slot="button-group"][data-orientation="vertical"] > *:not(:first-child) {
  border-top-left-radius: 0; border-top-right-radius: 0; margin-top: -1px;
}
[data-slot="button-group"][data-orientation="vertical"] > *:not(:last-child) {
  border-bottom-left-radius: 0; border-bottom-right-radius: 0;
}
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
}
[data-slot="combobox-trigger"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="combobox-content"] {
  display: flex; flex-direction: column; min-width: 8rem; padding: 0.25rem;
  border-radius: var(--cronus-radius-md); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-floating, var(--cronus-surface-overlay));
  color: var(--cronus-fg); box-shadow: var(--cronus-shadow-md, none);
}
[data-slot="combobox-item"] {
  display: flex; align-items: center; width: 100%;
  padding: 0.4rem 0.75rem; border: 0; border-radius: var(--cronus-radius-sm);
  font: inherit; text-align: left; cursor: pointer;
}
[data-slot="combobox-item"][aria-selected="true"] {
  background: var(--cronus-surface-overlay);
}
[data-slot="stepper"] {
  display: flex; width: 100%;
}
[data-slot="stepper"][data-orientation="horizontal"], [data-slot="stepper"]:not([data-orientation]) {
  flex-direction: row; align-items: center;
}
[data-slot="stepper"][data-orientation="vertical"] {
  flex-direction: column;
}
[data-slot="stepper-item"] {
  position: relative; display: flex; align-items: center;
}
[data-slot="stepper"][data-orientation="horizontal"] > [data-slot="stepper-item"]:not(:last-child) {
  flex: 1;
}
[data-slot="stepper-trigger"] {
  display: inline-flex; align-items: center; gap: 0.75rem;
  border: 0; background: transparent; color: inherit;
  font: inherit; text-align: left; cursor: pointer; outline: none;
}
[data-slot="stepper-title"] {
  font-size: 0.875rem; font-weight: 500; line-height: 1; color: var(--cronus-fg);
}
[data-slot="stepper-item"][data-state="upcoming"] [data-slot="stepper-title"] {
  color: var(--cronus-fg-tertiary);
}
[data-slot="input-otp"] {
  display: flex; align-items: center; gap: 0.5rem;
}
[data-slot="input-otp-group"] {
  display: flex; align-items: center;
}
[data-slot="input-otp-slot"] {
  position: relative; display: flex; align-items: center; justify-content: center;
  height: 2.5rem; width: 2.5rem; box-sizing: border-box;
  border-top: 1px solid var(--cronus-border);
  border-right: 1px solid var(--cronus-border);
  border-bottom: 1px solid var(--cronus-border);
  font-size: 0.875rem; color: var(--cronus-fg);
  font-variant-numeric: tabular-nums;
}
[data-slot="input-otp-slot"]:first-child {
  border-left: 1px solid var(--cronus-border);
  border-top-left-radius: var(--cronus-radius-lg);
  border-bottom-left-radius: var(--cronus-radius-lg);
}
[data-slot="input-otp-slot"]:last-child {
  border-top-right-radius: var(--cronus-radius-lg);
  border-bottom-right-radius: var(--cronus-radius-lg);
}
[data-slot="file-dropzone"] {
  position: relative; display: flex; flex-direction: column;
  align-items: center; justify-content: center; gap: 0.5rem;
  cursor: pointer; box-sizing: border-box;
  border-radius: var(--cronus-radius-xl);
  border: 2px dashed var(--cronus-border);
  background: var(--cronus-surface-inset);
  padding: 2.5rem 1.5rem; text-align: center;
  font-size: 0.875rem; color: var(--cronus-fg-secondary);
}
[data-slot="file-dropzone"][aria-disabled="true"] {
  cursor: not-allowed; opacity: 0.5;
}
[data-slot="file-dropzone"] .sr-only {
  position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;
  overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0;
}
[data-slot="file-dropzone"]:has(:focus-visible) {
  outline: 2px solid var(--cronus-ring, var(--cronus-primary));
  outline-offset: 2px;
button:has(+ [data-slot="popover-content"]) {
  font: inherit; cursor: pointer; color: var(--cronus-fg);
  background: transparent; border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-lg);
  height: 2.5rem; padding: 0 1rem; font-size: 0.875rem;
}
}
[data-slot="popover-content"] {
  z-index: 50; width: 18rem; box-sizing: border-box;
  padding: 0.75rem; outline: none;
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
  font-size: 0.875rem; box-shadow: var(--cronus-shadow-lg, none);
button:has(+ [data-slot="hover-card-content"]) {
}
}
[data-slot="hover-card-content"] {
  z-index: 50; width: 16rem; box-sizing: border-box;
}
[data-slot="dropdown-menu"] {
  position: relative; display: inline-flex; flex-direction: column; gap: 0.25rem;
}
[data-slot="dropdown-menu"] > button {
  display: inline-flex; align-items: center; justify-content: center;
  height: 2.5rem; padding: 0 1rem;
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
  font: inherit; font-size: 0.875rem; font-weight: 500; cursor: pointer;
}
[data-slot="dropdown-menu-content"] {
  min-width: 8rem; overflow: hidden;
  padding: 0.25rem; box-shadow: var(--cronus-shadow-lg, none);
}
[data-slot="dropdown-menu-item"] {
  border-radius: var(--cronus-radius-md); padding: 0.375rem 0.5rem;
  font-size: 0.875rem; cursor: default;
}
[data-slot="dropdown-menu-item"]:hover {
}
[data-slot="collapsible"] {
  display: flex; flex-direction: column; gap: 0.25rem;
}
[data-slot="collapsible"] > button {
  display: inline-flex; align-items: center;
}
[data-slot="collapsible-content"] {
  overflow: hidden; font-size: 0.875rem; color: var(--cronus-fg-secondary);
}
[data-slot="collapsible-content"][data-state="open"] {
  display: block;
}
[data-slot="mode-toggle"] {
  display: inline-flex; align-items: center; justify-content: center; flex-shrink: 0;
  width: 2.25rem; height: 2.25rem; padding: 0; margin: 0;
  border: 0; border-radius: var(--cronus-radius-lg);
  background: transparent; color: var(--cronus-fg-secondary);
  cursor: pointer; outline: none;
  transition: background 150ms var(--ease-out-quart), color 150ms var(--ease-out-quart);
}
[data-slot="mode-toggle"]:hover {
}
[data-slot="mode-toggle"]:disabled { opacity: 0.5; pointer-events: none; }
}
[data-slot="mode-toggle"] svg { width: 1.25rem; height: 1.25rem; }
}
[data-slot="mode-toggle-core"] {
  transform-origin: center; transform: scale(1);
  transition: transform 300ms var(--ease-out-quart);
}
[data-slot="mode-toggle"][data-mode="dark"] [data-slot="mode-toggle-core"] {
  transform: scale(1.75);
}
[data-slot="mode-toggle-rays"] {
  transform-origin: center; transform: scale(1); opacity: 1;
  transition: transform 300ms var(--ease-out-quart), opacity 300ms var(--ease-out-quart);
}
[data-slot="mode-toggle"][data-mode="dark"] [data-slot="mode-toggle-rays"] {
  transform: scale(0.5); opacity: 0;
}
[data-slot="mode-toggle-crescent"] {
  transform: translateX(0);
}
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
}
[data-slot="command-input"]::placeholder { color: var(--cronus-fg-tertiary); }
[data-slot="command-list"] {
  max-height: 20rem; overflow-y: auto; overflow-x: hidden; padding: 0.25rem;
}
[data-slot="command-item"] {
  display: flex; align-items: center; gap: 0.5rem;
  border-radius: var(--cronus-radius-md); padding: 0.375rem 0.5rem;
  font-size: 0.875rem; cursor: default;
}
[data-slot="command-item"]:hover {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="menubar"] {
  display: flex; align-items: center; gap: 0.25rem;
  border-radius: var(--cronus-radius-md); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-floating); padding: 0.25rem;
  box-shadow: var(--cronus-shadow-xs, none);
}
[data-slot="menubar-trigger"] {
  display: flex; align-items: center; border: 0;
  border-radius: var(--cronus-radius-md);
  padding: 0.25rem 0.75rem; font: inherit; font-size: 0.875rem; font-weight: 500;
  cursor: default; outline: none;
}
[data-slot="menubar-trigger"][data-state="open"],
[data-slot="menubar-trigger"]:hover {
}
[data-slot="menubar-content"] {
  min-width: 12rem; overflow: hidden;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  padding: 0.25rem; box-shadow: var(--cronus-shadow-lg, none);
}
[data-slot="menubar-item"] {
}
[data-slot="menubar-item"]:hover {
button:has(+ [data-slot="context-menu-content"]) {
  font: inherit; cursor: pointer; color: var(--cronus-fg);
  background: transparent; border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-lg);
  height: 2.5rem; padding: 0 1rem; font-size: 0.875rem;
}
}
[data-slot="context-menu-content"] {
  z-index: 50; min-width: 8rem; overflow: hidden; box-sizing: border-box;
}
[data-slot="context-menu-item"] {
}
[data-slot="context-menu-item"]:hover {
}
[data-slot="drawer"] {
  display: flex; flex-direction: column; gap: 0.5rem;
}
[data-slot="drawer"] > button {
  display: inline-flex; align-items: center; justify-content: center;
  height: 2.5rem; padding: 0 1rem;
  font: inherit; font-size: 0.875rem; font-weight: 500; cursor: pointer;
}
[data-slot="drawer-content"] {
  z-index: 50; display: flex; flex-direction: column;
  border-radius: var(--cronus-radius-xl) var(--cronus-radius-xl) 0 0;
  box-shadow: var(--cronus-shadow-lg, none);
}
[data-slot="drawer-title"] {
  font-size: 1.125rem; font-weight: 600; color: var(--cronus-fg);
  font-family: var(--cronus-font-display, inherit);
  padding: 1rem 1rem 0;
}
[data-slot="drawer-description"] {
  font-size: 0.875rem; color: var(--cronus-fg-secondary);
  padding: 0 1rem 1rem;
button:has(+ [data-slot="sheet-content"]) {
}
}
[data-slot="sheet-content"] {
  z-index: 50; display: flex; flex-direction: column; gap: 1rem;
  padding: 1.5rem; box-sizing: border-box;
  width: 75%; max-width: 24rem;
}
[data-slot="sheet-title"] {
}
[data-slot="sheet-description"] {
}
[data-slot="calendar"] {
  display: block; padding: 0.75rem; box-sizing: border-box;
  background: var(--cronus-surface-raised); color: var(--cronus-fg);
}
[data-slot="calendar"] table {
  width: 100%; border-collapse: collapse;
}
[data-slot="calendar"] caption {
  font-size: 0.875rem; font-weight: 500; padding-bottom: 0.5rem;
  color: var(--cronus-fg);
}
[data-slot="calendar"] th {
  width: 2.25rem; font-size: 0.75rem; font-weight: 400;
  color: var(--cronus-fg-tertiary); text-align: center;
}
[data-slot="calendar"] td {
  width: 2.25rem; height: 2.25rem; text-align: center;
  font-size: 0.875rem; color: var(--cronus-fg);
}
[data-slot="date-picker-trigger"] {
  display: inline-flex; align-items: center; justify-content: flex-start; gap: 0.5rem;
  width: 15rem; height: 2.5rem; padding: 0 0.75rem; box-sizing: border-box;
  font-size: 0.875rem; font-weight: 400; font-family: inherit; cursor: pointer;
}
[data-slot="date-picker-trigger"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="date-picker-trigger"][aria-invalid="true"] {
  border-color: var(--cronus-error);
}
[data-slot="date-picker-trigger"] svg { width: 1rem; height: 1rem; flex-shrink: 0; opacity: 0.7; }
[data-slot="date-picker-content"] {
  z-index: 50; width: auto; box-sizing: border-box;
  padding: 0.75rem; outline: none;
  font-size: 0.875rem; box-shadow: var(--cronus-shadow-lg, none);
}
[data-slot="date-picker-calendar"] {
  display: flex; flex-direction: column; gap: 0.25rem;
}
[data-slot="date-picker-calendar"] [role="row"] {
  display: grid; grid-template-columns: repeat(7, 2.25rem); justify-content: center;
}
[data-slot="date-picker-weekday"] {
  font-size: 0.75rem; font-weight: 400; color: var(--cronus-fg-tertiary);
  text-align: center;
}
[data-slot="date-picker-day"] {
  width: 2.25rem; height: 2.25rem; padding: 0; border: 0;
  font: inherit; font-size: 0.875rem; cursor: default;
}
[data-slot="date-picker-day"][aria-selected="true"] {
  background: var(--cronus-primary); color: var(--cronus-primary-foreground);
}
[data-slot="time-picker"] {
  display: inline-flex; flex-direction: column; gap: 0.25rem;
}
[data-slot="time-picker"] > button {
}
[data-slot="time-picker"] > button:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="time-picker"] > button svg { width: 1rem; height: 1rem; flex-shrink: 0; color: var(--cronus-fg-tertiary); }
[data-slot="time-picker-content"] {
}
[data-slot="time-picker-content"] > div {
  display: flex; align-items: flex-start; justify-content: center;
}
[data-slot="time-picker-column"] {
  display: flex; flex-direction: column; align-items: stretch;
}
[data-slot="time-picker-column"] > span {
  font-size: 0.6875rem; font-weight: 500; letter-spacing: 0.08em;
  text-transform: uppercase; color: var(--cronus-fg-tertiary);
}
[data-slot="time-picker-option"] {
  display: flex; align-items: center; justify-content: center;
  width: 3.5rem; height: 2.25rem; border-radius: var(--cronus-radius-md);
  font-size: 0.875rem; font-variant-numeric: tabular-nums; cursor: default;
  color: var(--cronus-fg-secondary);
}
[data-slot="time-picker-option"][aria-selected="true"] {
  background: color-mix(in oklch, var(--cronus-primary), black 30%);
  color: var(--cronus-primary-foreground); font-weight: 600;
}
[data-slot="time-picker-now"], [data-slot="time-picker-done"] {
  font: inherit; font-size: 0.75rem; cursor: pointer;
}
[data-slot="date-range-picker-trigger"] {
  width: 18.75rem; height: 2.5rem; padding: 0 0.75rem; box-sizing: border-box;
}
[data-slot="date-range-picker-trigger"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="date-range-picker-trigger"] svg { width: 1rem; height: 1rem; flex-shrink: 0; opacity: 0.7; }
[data-slot="date-range-picker-content"] {
  padding: 0; outline: none;
  display: flex; flex-direction: row;
}
[data-slot="date-range-picker-content"] legend {
  position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0, 0, 0, 0);
}
[data-slot="date-range-picker-presets"] {
  margin: 0; padding: 0.5rem; border: 0;
  border-right: 1px solid var(--cronus-border);
}
[data-slot="date-range-picker-preset"] {
  font: inherit; font-size: 0.75rem; text-align: left; cursor: default;
  border: 0; background: transparent; color: var(--cronus-fg);
  padding: 0.25rem 0.5rem; border-radius: var(--cronus-radius-md);
}
[data-slot="date-range-picker-calendar"] {
  margin: 0; padding: 0.75rem; border: 0;
  display: flex; flex-direction: row; gap: 1rem;
}
[data-slot="date-range-picker-month"] {
}
[data-slot="date-range-picker-month"] [role="row"] {
}
[data-slot="date-range-picker-weekday"] {
}
[data-slot="date-range-picker-day"] {
}
[data-slot="date-range-picker-day"][data-range="start"],
[data-slot="date-range-picker-day"][data-range="end"] {
}
[data-slot="date-range-picker-day"][data-range="middle"] {
  background: var(--cronus-surface-overlay); border-radius: 0;
}
[data-slot="area-chart"] {
  display: block; width: 100%; aspect-ratio: 2 / 1;
}
[data-slot="area-chart"] svg {
  display: block; width: 100%; height: 100%;
}
[data-slot="area-chart"] path {
  fill: var(--cronus-primary); fill-opacity: 0.28;
}
[data-slot="area-chart"] polyline {
  fill: none; stroke: var(--cronus-primary); stroke-width: 2;
  stroke-linejoin: round; stroke-linecap: round;
}
[data-slot="bar-chart"] {
}
[data-slot="bar-chart"] svg {
}
[data-slot="bar-chart"] rect {
  fill: var(--cronus-primary);
}
[data-slot="line-chart"] {
}
[data-slot="line-chart"] svg {
}
[data-slot="line-chart"] polyline {
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
[data-slot="radar-chart"] {
  display: flex; align-items: center; justify-content: center;
  width: 100%; height: 16rem;
}
[data-slot="radar-chart"] svg { width: 12rem; height: 12rem; }
[data-slot="radar-chart"] polygon {
  fill: var(--cronus-primary); fill-opacity: 0.28;
}
[data-slot="radar-chart"] polyline {
  fill: none; stroke: var(--cronus-primary); stroke-width: 2;
  stroke-linejoin: round; stroke-linecap: round;
}
[data-slot="scatter-chart"] {
  display: block; width: 100%; aspect-ratio: 2 / 1;
}
[data-slot="scatter-chart"] svg {
  display: block; width: 100%; height: 100%;
}
[data-slot="scatter-chart"] circle {
  fill: var(--cronus-primary);
}
[data-slot="ring-chart"] {
  display: flex; align-items: center; justify-content: center;
  width: 100%; height: 16rem;
}
[data-slot="ring-chart"] svg { width: 12rem; height: 12rem; }
[data-slot="ring-chart"] circle, [data-slot="ring-chart"] path {
  fill: none; stroke: var(--cronus-primary);
}
[data-slot="data-table"] {
  display: flex; flex-direction: column; gap: 0.75rem;
  overflow: auto; color: var(--cronus-fg);
}
[data-slot="sidebar"] {
  display: flex; flex-direction: column;
  width: 16rem; min-height: 100%; box-sizing: border-box;
  background: var(--cronus-surface-base); color: var(--cronus-fg);
  border-right: 1px solid var(--cronus-border);
}
[data-slot="sidebar-content"] {
  display: flex; flex-direction: column; flex: 1; min-height: 0; padding: 0.5rem;
}
[data-slot="sidebar-menu"] {
  list-style: none; margin: 0; padding: 0;
  display: flex; flex-direction: column; gap: 0.25rem;
}
[data-slot="sidebar-menu-item"] { position: relative; }
[data-slot="sidebar-menu-button"] {
  display: flex; align-items: center; gap: 0.5rem;
  width: 100%; box-sizing: border-box;
  height: 2rem; padding: 0 0.5rem;
  border-radius: var(--cronus-radius-md);
  color: var(--cronus-fg-secondary); text-decoration: none;
  font-size: 0.875rem;
}
[data-slot="sidebar-menu-button"]:hover {
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
}
[data-slot="sonner"] {
  position: fixed; z-index: 50; inset: auto 1rem 1rem auto;
  width: 22rem; max-width: calc(100% - 2rem);
}
[data-slot="toaster"] {
  display: flex; flex-direction: column; gap: 0.5rem;
}
[data-slot="toast"] {
  padding: 0.75rem 1rem; box-sizing: border-box;
  border-radius: var(--cronus-radius-lg);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
  font-size: 0.875rem; box-shadow: var(--cronus-shadow-lg, none);
}
[data-slot="navigation-menu"] {
  position: relative; z-index: 10;
  display: flex; max-width: max-content; flex: 1;
  align-items: center; justify-content: center;
}
[data-slot="navigation-menu-list"] {
  display: flex; flex: 1; align-items: center; justify-content: center; gap: 0.25rem;
}
[data-slot="navigation-menu-item"] { position: relative; }
[data-slot="navigation-menu-trigger"] {
  display: inline-flex; align-items: center; justify-content: center;
  height: 2.25rem; padding: 0 1rem;
  border: 0; border-radius: var(--cronus-radius-lg);
  background: transparent; color: var(--cronus-fg-secondary);
  font: inherit; font-size: 0.875rem; font-weight: 500; cursor: pointer;
}
[data-slot="navigation-menu-trigger"][data-state="open"],
[data-slot="navigation-menu-trigger"]:hover {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="navigation-menu-content"] {
  position: absolute; left: 0; top: 100%;
  min-width: 12rem; margin-top: 0.35rem; padding: 0.25rem;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
}
[data-slot="phone-input"] {
  display: flex; height: 2.5rem; width: 100%; align-items: stretch;
  overflow: hidden; box-sizing: border-box;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
  font-size: 0.875rem;
}
[data-slot="phone-input"][data-invalid="true"] {
  border-color: var(--cronus-error);
}
[data-slot="phone-input-country"] {
  display: flex; flex-shrink: 0; align-items: center; gap: 0.375rem;
  padding: 0 0.5rem 0 0.75rem; border: 0;
  background: transparent; color: var(--cronus-fg-secondary);
  font: inherit; font-size: 0.875rem; cursor: pointer;
  border-right: 1px solid var(--cronus-border);
}
[data-slot="phone-input-country"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="phone-input-field"] {
  height: 100%; width: auto; flex: 1; min-width: 0;
  border: 0; border-radius: 0; background: transparent; box-shadow: none;
  padding: 0 0.75rem; color: var(--cronus-fg); font: inherit; outline: none;
}
[data-slot="currency-input"] {
}
[data-slot="currency-input"][data-invalid=""] {
}
[data-slot="currency-input"][data-disabled=""] { opacity: 0.5; pointer-events: none; }
[data-slot="currency-input-prefix"] {
  padding: 0 0.75rem; color: var(--cronus-fg);
  border-right: 1px solid var(--cronus-border); user-select: none; white-space: nowrap;
  font-weight: 500;
}
[data-slot="currency-input-field"] {
  text-align: end; font-variant-numeric: tabular-nums;
}
[data-slot="color-picker"] {
  display: inline-flex; flex-direction: column; gap: 0.25rem;
}
[data-slot="color-picker-trigger"] {
  display: inline-flex; align-items: center; justify-content: flex-start; gap: 0.5rem;
  width: 100%; height: 2.5rem; padding: 0 0.75rem; box-sizing: border-box;
  background: transparent; color: var(--cronus-fg);
  font-size: 0.875rem; font-weight: 400; font-family: inherit; cursor: pointer;
}
[data-slot="color-picker-trigger"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="color-picker-swatch"] {
  width: 1.25rem; height: 1.25rem; flex-shrink: 0;
  border-radius: var(--cronus-radius-md); border: 1px solid var(--cronus-border);
  background: var(--cronus-primary);
}
[data-slot="color-picker-content"] {
  z-index: 50; width: 16rem; box-sizing: border-box;
  padding: 0.75rem; outline: none;
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
  font-size: 0.875rem; box-shadow: var(--cronus-shadow-lg, none);
  display: flex; flex-direction: column; gap: 0.75rem;
}
[data-slot="color-picker-swatches"] {
  display: flex; flex-wrap: wrap; gap: 0.375rem;
}
[data-slot="color-picker-swatch-button"] {
  width: 1.5rem; height: 1.5rem; padding: 0;
  cursor: default;
}
[data-slot="color-picker-swatch-button"]:nth-child(1) { background: var(--cronus-primary); }
[data-slot="color-picker-swatch-button"]:nth-child(2) { background: var(--cronus-error); }
[data-slot="color-picker-swatch-button"]:nth-child(3) { background: var(--cronus-success); }
[data-slot="color-picker-swatch-button"]:nth-child(4) { background: var(--cronus-warning); }
[data-slot="color-picker-swatch-button"]:nth-child(5) { background: var(--cronus-info); }
[data-slot="scroll-area"] {
  position: relative; overflow: hidden;
  max-height: 12rem;
}
[data-slot="scroll-area-viewport"] {
  width: 100%; height: 100%; max-height: inherit;
  overflow: auto; outline: none;
  font-size: 0.875rem; color: var(--cronus-fg);
}
[data-slot="scroll-bar"] {
  position: absolute; top: 0; right: 0; bottom: 0;
  display: flex; width: 0.625rem;
  border-left: 1px solid transparent; padding: 1px;
  touch-action: none; user-select: none;
}
[data-slot="scroll-bar"]::before {
  content: ""; flex: 1; border-radius: 9999px;
  background: var(--cronus-border);
}
[data-slot="toolbar"] {
  display: inline-flex; align-items: center; gap: 0.25rem;
  border-radius: var(--cronus-radius-lg);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-raised); color: var(--cronus-fg);
  padding: 0.25rem;
}
[data-slot="toolbar-button"] {
  display: inline-flex; align-items: center; justify-content: center; gap: 0.375rem;
  height: 2rem; min-width: 2rem; padding: 0 0.5rem;
  border: 0; border-radius: var(--cronus-radius-md);
  font: inherit; font-size: 0.875rem; font-weight: 500; cursor: pointer;
}
[data-slot="toolbar-button"]:hover,
[data-slot="toolbar-button"][data-state="on"] {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="status-dot"] {
  display: inline-flex; align-items: center; gap: 0.375rem;
}
[data-slot="status-dot-indicator"] {
  position: relative; display: inline-block; flex-shrink: 0;
  width: 0.625rem; height: 0.625rem; border-radius: 9999px;
  background: var(--cronus-success);
}
[data-slot="status-dot"][data-status="online"] [data-slot="status-dot-indicator"],
[data-slot="status-dot"][data-status="success"] [data-slot="status-dot-indicator"] {
}
[data-slot="status-dot"][data-status="offline"] [data-slot="status-dot-indicator"] {
  background: transparent; border: 2px solid var(--cronus-fg-muted);
}
[data-slot="status-dot"][data-status="busy"] [data-slot="status-dot-indicator"],
[data-slot="status-dot"][data-status="error"] [data-slot="status-dot-indicator"] {
  background: var(--cronus-error);
}
[data-slot="status-dot"][data-status="away"] [data-slot="status-dot-indicator"],
[data-slot="status-dot"][data-status="warning"] [data-slot="status-dot-indicator"] {
  background: var(--cronus-warning);
}
[data-slot="status-dot"][data-status="info"] [data-slot="status-dot-indicator"] {
  background: var(--cronus-info);
}
[data-slot="status-dot"][data-status="neutral"] [data-slot="status-dot-indicator"] {
  background: var(--cronus-fg-muted);
}
[data-slot="status-dot-label"] {
  font-size: 0.875rem; color: var(--cronus-fg-secondary);
}
[data-slot="tags-input"] {
  display: flex; min-height: 2.5rem; width: 100%; flex-wrap: wrap;
  align-items: center; gap: 0.375rem; box-sizing: border-box;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
  padding: 0.375rem 0.75rem; font-size: 0.875rem;
}
[data-slot="tags-input"][data-disabled=""] { opacity: 0.5; pointer-events: none; }
[data-slot="tags-input"][aria-invalid="true"] { border-color: var(--cronus-error); }
[data-slot="tags-input-item"] {
  display: inline-flex; align-items: center; gap: 0.25rem;
  border-radius: var(--cronus-radius-md); border: 1px solid var(--cronus-border);
  padding: 0.125rem 0.5rem; font-size: 0.75rem; font-weight: 500;
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="tags-input-remove"] {
  display: inline-flex; align-items: center; justify-content: center;
  width: 0.875rem; height: 0.875rem; padding: 0; border: 0;
  background: transparent; color: var(--cronus-fg-tertiary); cursor: pointer;
}
[data-slot="tags-input-field"] {
  min-width: 6rem; flex: 1; border: 0; background: transparent;
  color: var(--cronus-fg); font: inherit; outline: none;
}
[data-slot="tags-input-field"]::placeholder { color: var(--cronus-fg-tertiary); }
[data-slot="autocomplete"] {
  position: relative; display: flex; flex-direction: column; gap: 0.25rem;
  width: 100%;
}
[data-slot="autocomplete-input"] {
  display: flex; height: 2.5rem; width: 100%; box-sizing: border-box;
  padding: 0 0.75rem; font-size: 0.875rem; font-family: inherit; outline: none;
}
[data-slot="autocomplete-input"]::placeholder { color: var(--cronus-fg-tertiary); }
[data-slot="autocomplete-input"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="autocomplete-content"] {
  display: flex; flex-direction: column; min-width: 8rem; padding: 0.25rem;
  background: var(--cronus-surface-floating, var(--cronus-surface-overlay));
  color: var(--cronus-fg); box-shadow: var(--cronus-shadow-md, none);
}
[data-slot="autocomplete-item"] {
  display: flex; align-items: center; width: 100%;
  padding: 0.4rem 0.75rem; border: 0; border-radius: var(--cronus-radius-sm);
  background: transparent; color: inherit;
  font: inherit; text-align: left; cursor: pointer;
}
[data-slot="autocomplete-item"][aria-selected="true"] {
  background: var(--cronus-surface-overlay);
}
[data-slot="multi-select"] {
}
[data-slot="multi-select-trigger"] {
  display: inline-flex; align-items: center; justify-content: space-between;
  width: 100%; min-height: 2.5rem; padding: 0.375rem 0.75rem; box-sizing: border-box;
  background: transparent; color: var(--cronus-fg);
  font-size: 0.875rem; font-weight: 400; font-family: inherit; cursor: pointer;
}
[data-slot="multi-select-trigger"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="multi-select-trigger"][aria-invalid="true"] { border-color: var(--cronus-error); }
[data-slot="multi-select-content"] {
}
[data-slot="multi-select-item"] {
}
[data-slot="multi-select-item"][aria-selected="true"] {
}
[data-slot="credit-card-input"] {
  display: flex; flex-wrap: wrap; align-items: center; gap: 0.75rem;
  width: 100%; box-sizing: border-box;
  padding: 0.625rem 0.875rem;
  border-radius: var(--cronus-radius-xl); border: 1px solid var(--cronus-border);
  font-size: 0.875rem;
}
[data-slot="credit-card-input"][data-invalid=""] {
  border-color: var(--cronus-error);
}
[data-slot="credit-card-input"][data-disabled=""] { opacity: 0.6; pointer-events: none; }
[data-slot="credit-card-input"] input {
  min-width: 0; flex: 1; background: transparent; border: 0; outline: none;
  color: var(--cronus-fg); font: inherit; font-variant-numeric: tabular-nums;
}
[data-slot="credit-card-input"] input:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="floating-label-input"] {
  position: relative; display: block; width: 100%;
}
[data-slot="floating-label-input"] [data-slot="input"] {
  height: 3.5rem; padding-top: 1rem;
}
[data-slot="floating-label-input-label"] {
  position: absolute; left: 0.75rem; top: 1.15rem;
  transform-origin: left; transform: translateY(-0.7rem) scale(0.8);
  pointer-events: none; user-select: none;
  color: var(--cronus-fg-secondary); font-size: 0.875rem;
}
[data-slot="floating-label-input"]:has([data-slot="input"]:placeholder-shown) [data-slot="floating-label-input-label"] {
  top: 50%; transform: translateY(-50%) scale(1);
  color: var(--cronus-fg-tertiary);
}
[data-slot="floating-label-input"]:has([data-slot="input"]:focus) [data-slot="floating-label-input-label"],
[data-slot="floating-label-input"]:has([data-slot="input"]:not(:placeholder-shown)) [data-slot="floating-label-input-label"] {
  top: 1.15rem; transform: translateY(-0.7rem) scale(0.8);
  color: var(--cronus-fg-secondary);
}
[data-slot="floating-label-input"]:has([data-slot="input"]:focus) [data-slot="floating-label-input-label"] {
  color: var(--cronus-primary);
}
[data-slot="floating-label-input"][data-invalid=""] [data-slot="floating-label-input-label"],
[data-slot="floating-label-input"][data-invalid=""] [data-slot="floating-label-input-helper"] {
  color: var(--cronus-error);
}
[data-slot="floating-label-input"][data-disabled=""] { opacity: 0.5; pointer-events: none; }
[data-slot="floating-label-input-helper"] {
  margin-top: 0.375rem; padding: 0 0.25rem;
  font-size: 0.75rem; color: var(--cronus-fg-secondary);
}
[data-slot="split-button"] {
  display: inline-flex; align-items: stretch;
}
[data-slot="split-button"] > [data-slot="button"]:first-child {
  border-top-right-radius: 0; border-bottom-right-radius: 0; border-right-width: 0;
}
[data-slot="split-button"] > [data-slot="button"]:last-child {
  border-top-left-radius: 0; border-bottom-left-radius: 0; border-left-width: 0;
  aspect-ratio: 1; padding: 0; min-width: 2.5rem;
}
[data-slot="split-button"] > [data-slot="button"]:focus-visible {
  position: relative; z-index: 10;
}
[data-slot="split-button"][data-disabled=""] { opacity: 0.5; pointer-events: none; }
[data-slot="pill-nav"] {
  border-radius: 9999px;
  border: 1px solid var(--cronus-border);
  padding: 0.25rem;
}
[data-slot="pill-nav-item"] {
  position: relative; z-index: 0;
  border: 0; border-radius: 9999px;
  padding: 0.375rem 0.875rem;
  background: transparent; color: var(--cronus-fg-secondary);
  font: inherit; font-size: 0.875rem; font-weight: 500;
  text-decoration: none; cursor: pointer;
}
[data-slot="pill-nav-item"][aria-current="page"] {
  color: var(--cronus-fg);
  background: var(--cronus-surface-floating);
  box-shadow: var(--cronus-shadow-xs, none);
}
[data-slot="dock"] {
  display: inline-flex; align-items: flex-end; gap: 0.5rem;
  border-radius: var(--cronus-radius-xl);
  background: var(--cronus-surface-raised); color: var(--cronus-fg);
  padding: 0.5rem 0.75rem;
}
[data-slot="dock-item"] {
  width: 2.75rem; height: 2.75rem;
  border: 0; border-radius: var(--cronus-radius-lg);
  font: inherit; cursor: pointer; text-decoration: none;
}
[data-slot="workspace-switcher"] {
  display: flex; width: 100%; min-width: 0; align-items: center; gap: 0.5rem;
  padding: 0.375rem 0.5rem;
  font: inherit; font-size: 0.875rem; font-weight: 500; cursor: pointer;
}
[data-slot="workspace-switcher-content"] {
  min-width: 14rem; overflow: hidden;
  padding: 0.25rem; box-shadow: var(--cronus-shadow-lg, none);
  border-radius: var(--cronus-radius-lg);
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
}
[data-slot="workspace-switcher-item"] {
  border-radius: var(--cronus-radius-md); padding: 0.375rem 0.5rem;
  font-size: 0.875rem; cursor: default;
}
[data-slot="app-shell"] {
  display: flex; min-height: 100svh; width: 100%;
  background: var(--cronus-surface-base); color: var(--cronus-fg);
}
[data-slot="app-shell-content"] {
  position: relative; display: flex; min-height: 100svh; min-width: 0;
  flex: 1; flex-direction: column;
  background: var(--cronus-surface-base);
}
[data-slot="app-shell-header"] {
  position: sticky; top: 0; z-index: 30;
  display: flex; height: 3.5rem; flex-shrink: 0; align-items: center; gap: 0.5rem;
  border-bottom: 1px solid var(--cronus-border);
  padding: 0 1rem; font-size: 0.875rem; font-weight: 500;
}
[data-slot="app-shell-body"] {
  display: flex; min-height: 0; flex: 1; flex-direction: column;
}
[data-slot="table-of-contents"] {
  position: relative; font-size: 0.875rem; color: var(--cronus-fg);
}
[data-slot="table-of-contents-list"] {
  list-style: none; margin: 0; padding: 0;
  border-left: 1px solid var(--cronus-border);
}
[data-slot="table-of-contents-link"] {
  display: block; padding: 0.375rem 0.75rem;
  color: var(--cronus-fg-tertiary, var(--cronus-fg-secondary));
  text-decoration: none; line-height: 1.375;
}
[data-slot="table-of-contents-link"]:hover { color: var(--cronus-fg-secondary); }
[data-slot="table-of-contents-link"][aria-current="location"] {
  color: var(--cronus-fg); font-weight: 500;
}
[data-slot="form"] {
  display: flex; flex-direction: column; gap: 0.75rem;
}
[data-slot="form-item"] {
  display: flex; flex-direction: column; gap: 0.375rem;
}
[data-slot="form-label"] {
  font-size: 0.875rem; font-weight: 500; color: var(--cronus-fg);
  line-height: 1; user-select: none;
}
[data-slot="form-control"] { display: block; width: 100%; }
[data-slot="form-item"] input {
  display: flex; height: 2.5rem; width: 100%; box-sizing: border-box;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
  padding: 0 0.75rem; font-size: 0.875rem; font-family: inherit; outline: none;
}
[data-slot="form-description"] {
  margin: 0; font-size: 0.75rem; color: var(--cronus-fg-secondary);
}
[data-slot="signature-pad"] {
  position: relative; height: 10rem; width: 100%; overflow: hidden;
  box-sizing: border-box;
  border-radius: var(--cronus-radius-xl);
  border: 1px solid var(--cronus-border);
  box-shadow: var(--cronus-shadow-xs, none);
}
[data-slot="signature-pad-canvas"] {
  position: absolute; inset: 0; width: 100%; height: 100%; display: block;
}
[data-slot="signature-pad-hint"] {
  pointer-events: none; position: absolute;
  left: 1.25rem; right: 1.25rem; bottom: 1.75rem;
  font-size: 0.75rem; color: var(--cronus-fg-muted);
}
[data-slot="resizable"] {
  display: block; width: 100%; min-height: 12rem; box-sizing: border-box;
  border-radius: var(--cronus-radius-lg);
  background: var(--cronus-surface-raised); color: var(--cronus-fg);
  overflow: hidden;
}
[data-slot="resizable-panel-group"] {
  display: flex; height: 100%; width: 100%; min-height: 12rem;
}
[data-slot="resizable-panel"] {
  flex: 1; min-width: 0; min-height: 0; overflow: auto;
  padding: 0.75rem; font-size: 0.875rem; box-sizing: border-box;
}
[data-slot="resizable-handle"] {
  position: relative; display: flex; width: 1px;
  align-items: center; justify-content: center;
  background: var(--cronus-border); flex-shrink: 0;
}
[data-slot="scheduler"] {
  display: block; width: 100%; box-sizing: border-box;
}
[data-slot="scheduler-title"] {
  margin: 0; padding: 0.75rem;
  font-size: 0.875rem; font-weight: 600; color: var(--cronus-fg);
}
[data-slot="scheduler-grid"] {
  width: 100%; table-layout: fixed; border-collapse: collapse;
}
[data-slot="scheduler-weekdays"] th {
  padding: 0.375rem 0.5rem; font-size: 0.75rem; font-weight: 400;
  color: var(--cronus-fg-tertiary); text-align: center;
}
[data-slot="scheduler-grid"] td {
  height: 6rem; padding: 0.375rem; vertical-align: top;
  font-size: 0.75rem; color: var(--cronus-fg);
  border-right: 1px solid var(--cronus-border);
}
[data-slot="scheduler-event"] {
  display: block; margin-top: 0.25rem; padding: 0.125rem 0.375rem;
  border-radius: var(--cronus-radius-sm);
  background: color-mix(in oklch, var(--cronus-primary), transparent 85%);
  color: var(--cronus-primary); font-size: 0.75rem; font-weight: 500;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
[data-slot="alert-dialog"] {
  display: flex; flex-direction: column; gap: 0.5rem;
}
[data-slot="alert-dialog"] > button {
  display: inline-flex; align-items: center; justify-content: center;
  height: 2.5rem; padding: 0 1rem;
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
  font: inherit; font-size: 0.875rem; font-weight: 500; cursor: pointer;
}
[data-slot="alert-dialog-content"] {
  z-index: 50; display: grid; gap: 1rem;
  width: 100%; max-width: 32rem; box-sizing: border-box;
  padding: 1.5rem;
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
  box-shadow: var(--cronus-shadow-lg, none);
}
[data-slot="alert-dialog-title"] {
  font-size: 1.125rem; font-weight: 600; color: var(--cronus-fg);
  font-family: var(--cronus-font-display, inherit);
}
[data-slot="alert-dialog-description"] {
  font-size: 0.875rem; color: var(--cronus-fg-secondary);
}
[data-slot="alert-dialog-cancel"] {
  background: transparent; color: var(--cronus-fg);
}
[data-slot="alert-dialog-action"] {
  border: 1px solid transparent;
  background: var(--cronus-primary); color: var(--cronus-primary-foreground);
}
[data-slot="lightbox"] {
  z-index: 50; display: flex; flex-direction: column; gap: 0.75rem;
  box-sizing: border-box; padding: 1rem;
}
[data-slot="lightbox-counter"] {
}
[data-slot="lightbox-close"] {
  width: 2rem; height: 2rem; padding: 0; align-self: flex-end;
  border: 0; border-radius: var(--cronus-radius-md);
  background: transparent; color: var(--cronus-fg-secondary);
  font: inherit; cursor: pointer;
}
[data-slot="lightbox-caption"] {
  margin: 0; padding: 0.75rem 1rem 0;
  text-align: center; font-size: 0.875rem; color: var(--cronus-fg-secondary);
}
[data-slot="notification-trigger"] {
  height: 2.25rem; padding: 0 0.75rem;
  border: 0; border-radius: var(--cronus-radius-lg);
}
[data-slot="notification-center"] {
  z-index: 50; display: flex; flex-direction: column;
  width: 20rem; box-sizing: border-box; overflow: hidden;
}
[data-slot="notification-row"] {
  display: flex; width: 100%; box-sizing: border-box;
  align-items: flex-start; gap: 0.75rem;
  padding: 0.625rem 0.5rem; border: 0;
  font: inherit; font-size: 0.875rem; text-align: start; cursor: pointer;
}
[data-slot="notification-row"]:hover {
  background: var(--cronus-surface-overlay);
}
[data-slot="segmented-control"] {
  display: inline-flex; align-items: stretch; gap: 0.25rem;
  border-radius: var(--cronus-radius-lg);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-overlay); padding: 0.25rem;
}
[data-slot="segmented-control-item"] {
  position: relative;
  display: inline-flex; align-items: center; justify-content: center;
  padding: 0.375rem 0.75rem;
  border: 0; border-radius: var(--cronus-radius-md);
  background: transparent; color: var(--cronus-fg-secondary);
  font: inherit; font-size: 0.875rem; font-weight: 500; cursor: pointer;
}
[data-slot="segmented-control-item"][data-state="active"] {
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
  box-shadow: var(--cronus-shadow-sm, none);
}
[data-slot="segmented-control-item"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="usage-meter"] {
  display: flex; flex-direction: column; gap: 0.5rem;
}
[data-slot="usage-meter-label"] {
  font-size: 0.875rem; font-weight: 500; color: var(--cronus-fg);
}
[data-slot="usage-meter-fill"] {
  height: 0.5rem; border-radius: 9999px;
  background: var(--cronus-primary);
}
[data-slot="masonry"] {
  column-count: 3; column-gap: 1rem;
}
[data-slot="masonry-cell"] {
  break-inside: avoid; margin-bottom: 1rem;
  display: block; box-sizing: border-box;
  color: var(--cronus-fg); font-size: 0.875rem;
}
[data-slot="heatmap"] {
  display: inline-flex; flex-direction: column; gap: 0.5rem;
}
[data-slot="heatmap"] [role="img"] {
  display: grid; grid-auto-flow: column; grid-template-rows: repeat(7, auto); gap: 0.25rem;
}
[data-slot="heatmap-day"], [data-slot="heatmap-legend-swatch"] {
  width: 0.75rem; height: 0.75rem; border-radius: 3px;
  background: var(--cronus-surface-inset);
}
[data-slot="heatmap-day"][data-level="1"], [data-slot="heatmap-legend-swatch"][data-level="1"] {
  background: color-mix(in oklch, var(--cronus-primary) 25%, transparent);
}
[data-slot="heatmap-day"][data-level="2"], [data-slot="heatmap-legend-swatch"][data-level="2"] {
  background: color-mix(in oklch, var(--cronus-primary) 45%, transparent);
}
[data-slot="heatmap-day"][data-level="3"], [data-slot="heatmap-legend-swatch"][data-level="3"] {
  background: color-mix(in oklch, var(--cronus-primary) 70%, transparent);
}
[data-slot="heatmap-day"][data-level="4"], [data-slot="heatmap-legend-swatch"][data-level="4"] {
  background: var(--cronus-primary);
}
[data-slot="heatmap-legend"] {
  display: flex; align-items: center; gap: 0.25rem;
  font-size: 0.75rem; color: var(--cronus-fg-tertiary);
}
[data-slot="comparison-slider"] {
  position: relative; aspect-ratio: 16 / 9; width: 100%;
  overflow: hidden; user-select: none;
  border-radius: var(--cronus-radius-xl);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset);
}
[data-slot="comparison-after"], [data-slot="comparison-before"] {
  position: absolute; inset: 0; width: 100%; height: 100%;
  display: flex; align-items: center; justify-content: center;
  font-size: 0.875rem; color: var(--cronus-fg);
}
[data-slot="comparison-after"] {
  background: var(--cronus-surface-raised);
}
[data-slot="comparison-before"] {
  background: var(--cronus-surface-overlay);
  clip-path: inset(0 50% 0 0);
}
[data-slot="comparison-slider"]::after {
  content: "";
  position: absolute; top: 0; bottom: 0; left: 50%;
  width: 2px; transform: translateX(-50%);
  background: var(--cronus-surface-base); z-index: 10; pointer-events: none;
}
[data-slot="code-tabs"] {
  overflow: hidden; border-radius: var(--cronus-radius-xl);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-raised); color: var(--cronus-fg);
}
[data-slot="code-tabs-list"] {
  display: flex; align-items: center;
  border-bottom: 1px solid var(--cronus-border);
  background: var(--cronus-surface-overlay);
}
[data-slot="code-tabs-trigger"] {
  padding: 0.625rem 0.875rem; border: 0; background: transparent;
  color: var(--cronus-fg-tertiary); font: inherit; cursor: default;
}
[data-slot="code-tabs-trigger"][data-state="active"],
[data-slot="code-tabs-trigger"][aria-selected="true"] {
  color: var(--cronus-fg);
}
[data-slot="code-tabs-panel"] {
  overflow-x: auto;
}
[data-slot="code-tabs-pre"] {
  margin: 0; padding: 1rem 3.5rem 1rem 1rem;
  font-size: 0.875rem; line-height: 1.625;
}
[data-slot="code-tabs-code"] {
  display: block; white-space: pre;
  font-family: var(--cronus-font-mono, ui-monospace, monospace);
}
[data-slot="expandable-tabs"] {
  display: inline-flex; align-items: center; gap: 0.25rem;
  border-radius: 9999px;
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset);
  padding: 0.25rem;
}
[data-slot="expandable-tabs-item"] {
  display: inline-flex; align-items: center; gap: 0.5rem;
  border: 0; border-radius: 9999px;
  padding: 0.375rem 0.625rem;
  background: transparent; color: var(--cronus-fg-secondary);
  font: inherit; font-size: 0.875rem; font-weight: 500; cursor: pointer;
}
[data-slot="expandable-tabs-item"][aria-selected="true"] {
  color: var(--cronus-fg);
  background: var(--cronus-surface-floating);
  box-shadow: var(--cronus-shadow-xs, none);
}
[data-slot="live-line-chart"] {
  display: block; width: 100%; aspect-ratio: 2 / 1;
}
[data-slot="live-line-chart"] svg {
  display: block; width: 100%; height: 100%;
}
[data-slot="live-line-chart"] polyline {
  fill: none; stroke: var(--cronus-primary); stroke-width: 2;
  stroke-linejoin: round; stroke-linecap: round;
}
[data-slot="sunburst-chart"] {
  display: flex; align-items: center; justify-content: center;
  width: 100%; height: 16rem;
}
[data-slot="sunburst-chart"] svg { width: 12rem; height: 12rem; }
[data-slot="sunburst-chart"] path {
  stroke: var(--cronus-surface-base); stroke-width: 1;
}
[data-slot="choropleth-chart"] {
  display: block; width: 100%; height: 16rem;
}
[data-slot="choropleth-chart"] svg {
  display: block; width: 100%; height: 100%;
}
[data-slot="choropleth-chart"] path {
  fill: var(--cronus-primary); stroke: var(--cronus-border);
}
[data-slot="profit-loss-chart"] {
  display: block; width: 100%; aspect-ratio: 2 / 1;
}
[data-slot="profit-loss-chart"] svg {
}
[data-slot="profit-loss-chart"] polyline {
  fill: none; stroke-width: 2;
  stroke-linejoin: round; stroke-linecap: round;
}
[data-slot="profit-loss-chart"] polyline[stroke="var(--cronus-success)"] {
  stroke: var(--cronus-success);
}
[data-slot="profit-loss-chart"] polyline[stroke="var(--cronus-error)"] {
  stroke: var(--cronus-error);
}
[data-slot="scroll-progress"] {
  height: 0.25rem; width: 100%; overflow: hidden;
  background: var(--cronus-surface-inset);
}
[data-slot="scroll-progress-fill"] {
  height: 100%; background: var(--cronus-primary);
}
[data-slot="scroll-progress"][data-variant="circle"] {
  position: relative; display: inline-flex;
  align-items: center; justify-content: center;
  width: 2.5rem; height: 2.5rem; overflow: visible;
  background: transparent;
}
[data-slot="scroll-progress"][data-variant="circle"] svg {
  transform: rotate(-90deg);
}
[data-slot="scroll-progress-ring"] {
  fill: none; stroke: var(--cronus-primary);
}
[data-slot="scroll-progress-value"] {
  position: absolute; inset: 0;
  display: flex; align-items: center; justify-content: center;
  font-size: 0.75rem; font-weight: 500; color: var(--cronus-fg);
  font-variant-numeric: tabular-nums;
}
[data-slot="rich-text-editor"] {
  display: flex; flex-direction: column; overflow: hidden;
  width: 100%; min-width: 0; box-sizing: border-box;
  border-radius: var(--cronus-radius-lg);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-base); color: var(--cronus-fg);
}
[data-slot="rich-text-editor-toolbar"] {
  display: flex; flex-wrap: wrap; align-items: center; gap: 0.125rem;
  border-bottom: 1px solid var(--cronus-border);
  background: color-mix(in oklch, var(--cronus-surface-overlay) 40%, transparent);
  padding: 0.375rem;
}
[data-slot="rich-text-editor-toolbar"] button {
  display: inline-flex; align-items: center; justify-content: center;
  height: 2rem; padding: 0 0.375rem;
  border: 0; border-radius: var(--cronus-radius-md);
  background: transparent; color: var(--cronus-fg-secondary);
  font: inherit; font-size: 0.75rem; cursor: pointer;
}
[data-slot="rich-text-editor-toolbar"] [data-slot="separator"][data-orientation="vertical"] {
  height: 1.5rem; width: 1px; margin: 0 0.25rem;
  background: var(--cronus-border);
}
[data-slot="rich-text-editor-content-wrapper"] {
  position: relative; min-height: 10rem; box-sizing: border-box;
  padding: 0.75rem 1rem; font-size: 0.875rem; line-height: 1.6;
  color: var(--cronus-fg);
}
[data-slot="confirmation-dialog"] {
  z-index: 50; display: grid; gap: 1rem;
  width: 100%; max-width: 28rem; box-sizing: border-box;
  padding: 1.5rem;
  border-radius: var(--cronus-radius-xl);
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
  box-shadow: var(--cronus-shadow-lg, none);
}
[data-slot="confirmation-dialog-title"] {
  font-size: 1.125rem; font-weight: 600; color: var(--cronus-fg);
  font-family: var(--cronus-font-display, inherit);
}
[data-slot="confirmation-dialog-description"] {
  font-size: 0.875rem; color: var(--cronus-fg-secondary);
}
[data-slot="confirmation-dialog"] > button {
  height: 2.5rem; padding: 0 1rem;
  background: transparent; color: var(--cronus-fg);
  font: inherit; font-size: 0.875rem; font-weight: 500; cursor: pointer;
}
[data-slot="confirmation-dialog-confirm"] {
  border: 1px solid transparent;
  background: var(--cronus-primary); color: var(--cronus-primary-foreground);
}
[data-slot="invite-dialog"] {
}
[data-slot="invite-dialog-title"] {
}
[data-slot="invite-dialog-description"] {
}
[data-slot="invite-dialog"] > label {
  display: flex; flex-direction: column; gap: 0.35rem;
  font-size: 0.875rem; font-weight: 500; color: var(--cronus-fg);
}
[data-slot="invite-dialog"] input[type="email"] {
  display: flex; height: 2.5rem; width: 100%; box-sizing: border-box;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
  padding: 0 0.75rem; font-size: 0.875rem; font-family: inherit; outline: none;
}
[data-slot="invite-dialog"] > button {
}
[data-slot="invite-dialog-send"] {
}
[data-slot="shimmer"] {
  display: block; position: relative; overflow: hidden;
  height: 0.9rem; width: 8rem;
  border-radius: var(--cronus-radius-md);
  background: var(--cronus-surface-overlay);
}
[data-slot="shimmer"]::after {
  content: "";
  transform: translateX(-100%);
  background: linear-gradient(90deg, transparent, color-mix(in oklch, var(--cronus-fg) 10%, transparent), transparent);
  animation: cui-shimmer 2s linear infinite;
}
@keyframes cui-shimmer { 100% { transform: translateX(100%); } }
[data-slot="reveal"] {
  animation: cui-reveal 500ms var(--ease-out-quart) both;
}
@keyframes cui-reveal {
  from { opacity: 0; transform: translateY(16px); }
  to { opacity: 1; transform: translateY(0); }
}
[data-slot="text-shimmer"] {
  display: inline-block;
  color: transparent;
  background-image:
    linear-gradient(90deg, transparent 40%, var(--cronus-surface-base), transparent 60%),
    linear-gradient(var(--cronus-fg-tertiary, var(--cronus-fg-secondary)), var(--cronus-fg-tertiary, var(--cronus-fg-secondary)));
  background-size: 250% 100%, auto;
  background-repeat: no-repeat, padding-box;
  -webkit-background-clip: text;
  background-clip: text;
  animation: cui-text-shimmer 2s linear infinite;
}
@keyframes cui-text-shimmer {
  0% { background-position: 100% center, 0 0; }
  100% { background-position: 0% center, 0 0; }
}
[data-slot="particles"] {
  position: relative; overflow: hidden; display: inline-block;
  padding: 1rem 1.25rem;
  background-color: var(--cronus-surface-overlay);
  background-image:
    radial-gradient(circle, color-mix(in oklch, var(--cronus-fg) 35%, transparent) 1.2px, transparent 1.6px),
    radial-gradient(circle, color-mix(in oklch, var(--cronus-fg) 22%, transparent) 0.8px, transparent 1.2px);
  background-size: 24px 24px, 32px 28px;
  background-position: 0 0, 12px 8px;
}
[data-slot="particles"]::after {
  content: "";
  position: absolute; inset: 0; pointer-events: none;
  background-image: radial-gradient(circle, color-mix(in oklch, var(--cronus-fg) 40%, transparent) 1px, transparent 1.4px);
  background-size: 18px 22px;
  animation: cui-particles 14s linear infinite;
}
@keyframes cui-particles {
  from { background-position: 0 0; }
  to { background-position: 18px 22px; }
}
[data-slot="sparkles-text"] {
  position: relative; display: inline-block;
}
[data-slot="sparkles-text"]::before,
[data-slot="sparkles-text"]::after {
  position: absolute;
  width: 0.5rem; height: 0.5rem;
  background: var(--cronus-primary);
  clip-path: polygon(50% 0%, 61% 35%, 98% 35%, 68% 57%, 79% 91%, 50% 70%, 21% 91%, 32% 57%, 2% 35%, 39% 35%);
  animation: cui-sparkle 1.6s ease-in-out infinite;
  pointer-events: none;
}
[data-slot="sparkles-text"]::before {
  top: -0.35rem; left: 8%;
  animation-delay: 0s; animation-duration: 1.4s;
  top: 10%; inset-inline-end: -0.35rem;
  animation-delay: 0.7s; animation-duration: 1.65s;
}
@keyframes cui-sparkle {
  0%, 100% { transform: scale(0); opacity: 0; }
  40% { transform: scale(1); opacity: 1; }
  70% { transform: scale(0.6); opacity: 0.6; }
}
[data-slot="noise"] {
  isolation: isolate;
}
[data-slot="noise"]::after {
  opacity: 0.08;
  mix-blend-mode: overlay;
    repeating-radial-gradient(circle at 20% 30%, var(--cronus-fg) 0 0.4px, transparent 0.6px 3px),
    repeating-conic-gradient(from 20deg, transparent 0 12deg, color-mix(in oklch, var(--cronus-fg) 50%, transparent) 12.2deg 12.5deg, transparent 12.8deg 24deg);
  background-size: 5px 5px, 7px 7px;
}
[data-slot="morphing-popover"] {
  position: relative;
  display: inline-flex;
  flex-direction: column;
  align-items: flex-start;
}
[data-slot="morphing-popover-trigger"] {
  display: inline-flex; align-items: center; justify-content: center; gap: 0.5rem;
  height: 2.25rem; padding: 0 0.75rem;
  border-radius: var(--cronus-radius-lg);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-floating);
  color: var(--cronus-fg);
  font: inherit; font-size: 0.875rem; font-weight: 500; cursor: pointer;
}
[data-slot="morphing-popover-content"] {
  z-index: 50; width: 18rem; box-sizing: border-box;
  padding: 1rem; outline: none; overflow: hidden;
  border-radius: var(--cronus-radius-xl);
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
  font-size: 0.875rem; box-shadow: var(--cronus-shadow-lg, none);
}
[data-slot="bouncy-accordion"] {
  display: flex; flex-direction: column; align-items: center;
  width: 100%; max-width: 300px;
}
[data-slot="bouncy-accordion-trigger"] {
  display: flex; width: 100%; min-height: 45px; box-sizing: border-box;
  align-items: center; padding: 0 0.75rem;
  border: 0; border-radius: 20px;
  background: var(--cronus-surface-base); color: var(--cronus-fg);
  font: inherit; font-size: 0.875rem; cursor: pointer; text-align: start;
}
[data-slot="bouncy-accordion-content"] {
  width: 100%; padding: 0.5rem 0.75rem; box-sizing: border-box;
  font-size: 0.875rem; color: var(--cronus-fg-tertiary, var(--cronus-fg-secondary));
}
[data-slot="typing-text"] {
  display: inline; white-space: pre-wrap;
}
[data-slot="typing-text"]::after {
  display: inline-block;
  width: 1px; height: 1em;
  margin-inline-start: 1px;
  background: var(--cronus-fg);
  transform: translateY(0.1em);
  vertical-align: baseline;
  animation: cui-typing-caret 1s step-end infinite;
}
@keyframes cui-typing-caret {
  50% { opacity: 0; }
}
@media (prefers-reduced-motion: reduce) {
}
  [data-slot="typing-text"]::after { display: none; animation: none; }
[data-slot="word-rotate"] {
  position: relative; display: inline-grid;
  height: 1.2em; overflow: hidden;
  vertical-align: baseline; white-space: nowrap;
}
[data-slot="timeline"] {
  display: flex; flex-direction: column; width: 100%; min-width: 0;
}
[data-slot="timeline-item"] {
  position: relative; display: grid;
  grid-template-columns: auto minmax(0, 1fr); column-gap: 0.75rem;
  padding-bottom: 1.5rem;
}
[data-slot="timeline-item"]:last-child { padding-bottom: 0; }
[data-slot="timeline-item"]::before {
  content: ""; width: 0.625rem; height: 0.625rem; margin-top: 0.25rem;
  border-radius: 999px; background: var(--cronus-fg-tertiary);
}
[data-slot="timeline-item"]:not(:last-child)::after {
  content: ""; position: absolute; left: 0.25rem; top: 1rem; bottom: 0;
  width: 1px; background: var(--cronus-border);
}
[data-slot="timeline-content"] {
  display: flex; flex-direction: column; gap: 0.25rem;
  min-width: 0; padding-top: 0.125rem;
  font-size: 0.875rem; color: var(--cronus-fg);
}
[data-slot="tree-view"] {
  font-size: 0.875rem; color: var(--cronus-fg); user-select: none;
}
[data-slot="tree-view-item"] { min-width: 0; }
[data-slot="tree-view-item-trigger"] {
  display: flex; align-items: center; gap: 0.375rem;
  height: 2rem; padding: 0 0.5rem;
  border-radius: var(--cronus-radius-md);
  color: var(--cronus-fg-secondary);
}
[data-slot="tree-view-item-trigger"]:hover {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="tilt-card"] {
  position: relative;
  border-radius: var(--cronus-radius-xl);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-raised);
  padding: 1.5rem;
  color: var(--cronus-fg);
  box-shadow: var(--cronus-shadow-sm, var(--cronus-shadow-xs, none));
  transform-style: preserve-3d;
  transform: perspective(1000px) rotateX(var(--tilt-rx, 0deg)) rotateY(var(--tilt-ry, 0deg)) scale(var(--tilt-scale, 1));
}
[data-slot="star-border"] {
}
[data-slot="star-border"]::before,
[data-slot="star-border"]::after {
  content: "";
  position: absolute;
  width: 0.375rem; height: 0.375rem;
  border-radius: 999px;
  background: var(--cronus-primary);
  pointer-events: none;
  offset-path: rect(0 auto auto 0 round 12px);
  animation: cui-star-border 6s linear infinite;
}
[data-slot="star-border"]::after { animation-delay: -3s; }
@keyframes cui-star-border {
  from { offset-distance: 0%; }
  to { offset-distance: 100%; }
}
[data-slot="glass-card"] {
  isolation: isolate;
  background: color-mix(in oklch, var(--cronus-surface-raised) 60%, transparent);
  box-shadow: var(--cronus-shadow-lg, none);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
}
[data-slot="glass-card"]::before {
  inset-inline: 0; top: 0; height: 1px;
  border-radius: inherit;
  background: linear-gradient(to right, transparent, var(--cronus-border), transparent);
}
[data-slot="terminal"] {
  overflow: hidden; border-radius: var(--cronus-radius-xl);
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
}
[data-slot="terminal-screen"] {
  padding: 1rem; display: flex; flex-direction: column; gap: 0.25rem;
  font-family: var(--cronus-font-mono, ui-monospace, monospace);
  font-size: 0.875rem; line-height: 1.625;
}
[data-slot="terminal-line"] {
  display: flex; gap: 0.5rem; min-width: 0;
  white-space: pre-wrap; word-break: break-word;
}
[data-slot="terminal-prompt"] {
  user-select: none; color: var(--cronus-fg-tertiary, var(--cronus-fg-secondary));
}
[data-slot="terminal-line"]:last-child::after {
  content: ""; display: inline-block;
  width: 0.55em; height: 1.1em; margin-left: 1px;
  background: var(--cronus-fg);
  animation: cui-terminal-caret 1s step-end infinite;
  vertical-align: text-bottom;
}
@keyframes cui-terminal-caret {
  50% { opacity: 0; }
}
@media (prefers-reduced-motion: reduce) {
}
  [data-slot="terminal-line"]:last-child::after { display: none; animation: none; }
[data-slot="video-player"] {
  position: relative; isolation: isolate; width: 100%;
  background: var(--cronus-surface-inset);
  aspect-ratio: 16 / 9;
}
[data-slot="video-player-video"] {
  position: absolute; inset: 0; width: 100%; height: 100%;
  object-fit: contain;
}
[data-slot="video-player-controls"] {
  position: absolute; inset-inline: 0; bottom: 0; z-index: 20;
  display: flex; align-items: center; gap: 0.375rem;
  border-top: 1px solid var(--cronus-border);
  background: color-mix(in oklch, var(--cronus-surface-base) 90%, transparent);
  padding: 0.5rem 0.625rem;
}
[data-slot="video-player-play"] {
  display: inline-flex; width: auto; min-width: 2rem; height: 2rem;
  align-items: center; justify-content: center;
  padding: 0 0.5rem; border: 0; border-radius: var(--cronus-radius-md);
  background: transparent; color: var(--cronus-fg);
  font: inherit; font-size: 0.75rem; font-weight: 500; cursor: pointer;
}
[data-slot="text-effect"] {
  display: inline-block;
  animation: cui-text-effect 400ms var(--ease-out-quart) both;
}
@keyframes cui-text-effect {
  from { opacity: 0; filter: blur(8px); transform: translateY(8px); }
  to { opacity: 1; filter: blur(0); transform: translateY(0); }
}
  [data-slot="text-effect"] { animation: none; }
[data-slot="animated-list"] {
  display: flex; flex-direction: column; gap: 0.5rem;
  list-style: none; margin: 0; padding: 0;
}
[data-slot="animated-list-item"] {
  animation: cui-animated-list 0.35s var(--cronus-ease) both;
}
[data-slot="animated-list-item"]:nth-child(1) { animation-delay: 0s; }
[data-slot="animated-list-item"]:nth-child(2) { animation-delay: 0.08s; }
[data-slot="animated-list-item"]:nth-child(3) { animation-delay: 0.16s; }
[data-slot="animated-list-item"]:nth-child(4) { animation-delay: 0.24s; }
[data-slot="animated-list-item"]:nth-child(5) { animation-delay: 0.32s; }
@keyframes cui-animated-list {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}
[data-slot="carousel"] {
  position: relative; outline: none;
}
[data-slot="carousel-content"] {
  display: flex; overflow-x: auto;
  scroll-snap-type: x mandatory;
  scrollbar-width: none;
}
[data-slot="carousel-item"] {
  min-width: 0; flex: 0 0 100%;
  scroll-snap-align: start; box-sizing: border-box;
  color: var(--cronus-fg);
}
[data-slot="carousel-previous"], [data-slot="carousel-next"] {
  display: inline-flex; align-items: center; justify-content: center;
  width: 2rem; height: 2rem; margin-top: 0.5rem;
  border-radius: 9999px; border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-raised); color: var(--cronus-fg);
  font: inherit; font-size: 0.75rem; cursor: pointer;
}
[data-slot="carousel-previous"]:disabled, [data-slot="carousel-next"]:disabled {
  opacity: 0.5; pointer-events: none;
}
[data-slot="code-block"] {
  margin: 0; overflow: auto;
  border-radius: var(--cronus-radius-xl);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-raised); color: var(--cronus-fg);
  padding: 1rem; font-size: 0.875rem; line-height: 1.625;
  font-family: var(--cronus-font-mono, ui-monospace, monospace);
}
[data-slot="code-block"] code {
  display: block; white-space: pre; font-family: inherit;
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

    #[test]
    fn component_chrome_braces_are_balanced() {
        let css = component_chrome_css();
        let open = css.chars().filter(|&c| c == '{').count();
        let close = css.chars().filter(|&c| c == '}').count();
        assert!(
            close >= open,
            "unclosed CSS rules: {open} open vs {close} close"
        );
        assert!(
            close - open <= 8,
            "too many extra CSS closes: {open} vs {close}"
        );
        assert!(css.contains("[popover]"));
        assert!(css.contains("cronus-pop-in"));
    }
}
