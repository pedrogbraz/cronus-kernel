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
    format!("{AUDIT_PREFLIGHT}\n{FALLBACK_ROOT}\n{TOKENS_CSS}\n{COMPONENT_CHROME}")
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
        "<{tag}{href_attr}{type_attr}{disabled_attr} data-slot=\"button\" data-variant=\"{variant}\" data-size=\"{size}\" class=\"cui-btn\">{label}</{tag}>",
        size = size,
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

/// Tailwind v4.3 preflight, as the React audit reference receives it
/// (apps/www `@import "tailwindcss"`). Only the audit document uses it, placed
/// before COMPONENT_CHROME so chrome rules still win. `--theme()` font stacks
/// are omitted: the kernel already resolves the same sans/mono families.
const AUDIT_PREFLIGHT: &str = r#"*, ::after, ::before, ::backdrop, ::file-selector-button {
  box-sizing: border-box; margin: 0; padding: 0; border: 0 solid;
}
html, :host {
  line-height: 1.5; -webkit-text-size-adjust: 100%; tab-size: 4;
  -webkit-tap-highlight-color: transparent;
}
hr { height: 0; color: inherit; border-top-width: 1px; }
abbr:where([title]) { text-decoration: underline dotted; }
h1, h2, h3, h4, h5, h6 { font-size: inherit; font-weight: inherit; }
a { color: inherit; text-decoration: inherit; }
b, strong { font-weight: bolder; }
code, kbd, samp, pre { font-size: 1em; }
small { font-size: 80%; }
sub, sup { font-size: 75%; line-height: 0; position: relative; vertical-align: baseline; }
sub { bottom: -0.25em; }
sup { top: -0.5em; }
table { text-indent: 0; border-color: inherit; border-collapse: collapse; }
progress { vertical-align: baseline; }
summary { display: list-item; }
ol, ul, menu { list-style: none; }
img, svg, video, canvas, audio, iframe, embed, object { display: block; vertical-align: middle; }
img, video { max-width: 100%; height: auto; }
button, input, select, optgroup, textarea, ::file-selector-button {
  font: inherit; font-feature-settings: inherit; font-variation-settings: inherit;
  letter-spacing: inherit; color: inherit; border-radius: 0; background-color: transparent; opacity: 1;
}
::file-selector-button { margin-inline-end: 4px; }
::placeholder { opacity: 1; color: color-mix(in oklab, currentcolor 50%, transparent); }
textarea { resize: vertical; }
::-webkit-search-decoration { -webkit-appearance: none; }
button, input:where([type='button'], [type='reset'], [type='submit']), ::file-selector-button {
  appearance: button;
}
::-webkit-inner-spin-button, ::-webkit-outer-spin-button { height: auto; }
[hidden]:where(:not([hidden='until-found'])) { display: none !important; }
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
[data-slot="catalog-specimen"]:has(:popover-open),
[data-slot="catalog-specimen"]:has(span:hover > [data-slot="hover-card-content"]),
[data-slot="catalog-specimen"]:has(span:focus-within > [data-slot="hover-card-content"]) {
  z-index: 30;
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
[data-slot="combobox"] {
  position: relative;
  display: inline-flex;
  flex-direction: column;
  align-items: flex-start;
}
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
[data-slot="combobox-content"]:not(:popover-open),
[data-slot="sheet-content"]:not(:popover-open),
[data-slot="dropdown-menu-content"]:not(:popover-open),
[data-slot="popover-content"]:not(:popover-open) {
  display: none;
}
[data-slot="sheet-trigger"] {
  display: inline-flex; align-items: center; justify-content: center;
  height: 2.5rem; padding: 0 1rem;
  border-radius: var(--cronus-radius-lg);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-overlay);
  color: var(--cronus-fg);
  font: inherit; font-size: 0.875rem; cursor: pointer;
}
[data-slot="sheet-content"][popover]:popover-open {
  display: flex; flex-direction: column; gap: 0.5rem;
  box-sizing: border-box;
  height: 100dvh; width: min(24rem, 92vw); max-width: 24rem;
  margin: 0; padding: 1.5rem 1.5rem 1.75rem;
  border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-xl) 0 0 var(--cronus-radius-xl);
  background: var(--cronus-surface-floating, var(--cronus-surface-overlay));
  color: var(--cronus-fg);
  position: fixed; inset: 0 0 0 auto;
  box-shadow: var(--cronus-shadow-lg, 0 16px 40px rgba(0,0,0,.35));
  animation: cronus-slide-in-right 280ms var(--cronus-ease) both;
}
[data-slot="sheet-content"][popover]::backdrop {
  background: color-mix(in oklch, black 45%, transparent);
  animation: cronus-overlay-in 200ms var(--cronus-ease) both;
}
[data-slot="sheet-title"] {
  font-family: var(--cronus-font-display, inherit);
  font-size: 1.125rem; font-weight: 400; letter-spacing: -0.02em;
  color: var(--cronus-fg); padding-inline-end: 2.5rem;
}
[data-slot="sheet-description"] {
  font-size: 0.875rem; line-height: 1.5; color: var(--cronus-fg-secondary);
}
[data-slot="sheet-close"] {
  position: absolute; top: 1rem; right: 1rem;
  width: 2rem; height: 2rem; padding: 0; border: 0;
  border-radius: var(--cronus-radius-md);
  background: transparent; color: var(--cronus-fg-tertiary); cursor: pointer;
}
[data-slot="sheet-close"]:hover { color: var(--cronus-fg); }
[data-slot="sheet-close"]::before,
[data-slot="sheet-close"]::after {
  content: ""; position: absolute; top: 50%; left: 50%;
  width: 0.75rem; height: 1.5px; background: currentColor;
}
[data-slot="sheet-close"]::before { transform: translate(-50%, -50%) rotate(45deg); }
[data-slot="sheet-close"]::after { transform: translate(-50%, -50%) rotate(-45deg); }
[data-slot="combobox-content"]:popover-open {
  display: flex; flex-direction: column; min-width: 16rem; padding: 0.25rem;
  z-index: 50;
}
[data-slot="combobox-item"] {
  background: transparent; color: inherit;
}
[data-slot="toast"] {
  display: block; box-sizing: border-box;
  padding: 0.75rem 1rem;
  border-radius: var(--cronus-radius-lg);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
  font-size: 0.875rem; line-height: 1.25rem;
  box-shadow: var(--cronus-shadow-lg, 0 12px 32px rgba(0,0,0,.28));
}
span:has(> [data-slot="hover-card-content"]) { position: relative; display: inline-flex; }
[data-slot="button"]:has(+ [data-slot="hover-card-content"]),
[data-slot="button"]:has(+ [data-slot="popover-content"]),
[data-slot="button"]:has(+ [data-slot="dropdown-menu-content"]) {
  border: 0; line-height: 1.25rem;
}
[data-slot="hover-card-content"] {
  display: none;
  position: absolute; z-index: 50; top: calc(100% + 4px); left: 50%;
  width: 16rem; padding: 0.75rem; box-sizing: border-box;
  border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-lg);
  background: var(--cronus-surface-floating);
  color: var(--cronus-fg);
  box-shadow: var(--cronus-shadow-lg, 0 16px 40px rgba(0,0,0,.32));
  transform: translateX(-50%);
}
span:hover > [data-slot="hover-card-content"],
span:focus-within > [data-slot="hover-card-content"] {
  display: block;
}
[data-slot="popover-content"]:popover-open {
  display: block; z-index: 50; width: 18rem; box-sizing: border-box;
  padding: 0.75rem; outline: none;
  border: 1px solid var(--cronus-border); border-radius: var(--cronus-radius-lg);
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
  box-shadow: var(--cronus-shadow-lg, none);
}
[data-slot="dialog-trigger"] {
  display: inline-flex; align-items: center; justify-content: center;
  height: 2.5rem; padding: 0 1rem;
  border-radius: var(--cronus-radius-lg);
  border: 1px solid transparent;
  background: var(--cronus-primary); color: var(--cronus-primary-foreground);
  font: inherit; font-size: 0.875rem; font-weight: 500; cursor: pointer;
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
[data-slot="dialog"] form {
  display: flex; flex-direction: column; gap: 0.75rem;
  padding: 1.25rem 1.35rem 1.15rem;
}
[data-slot="dialog-title"] {
  font-family: var(--cronus-font-display, inherit);
  font-size: 1.125rem; font-weight: 400; letter-spacing: -0.02em;
  color: var(--cronus-fg);
}
[data-slot="dialog-description"] {
  margin: 0; font-size: 0.875rem; line-height: 1.5; color: var(--cronus-fg-secondary);
}
[data-slot="dialog-content"] [data-slot="button"][data-variant="outline"] {
  align-self: flex-end; width: auto; min-width: 5.5rem; height: 2.25rem;
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

[data-slot="date-picker-trigger"] {
  display: inline-flex; align-items: center; justify-content: flex-start; gap: 0.5rem;
  width: 15rem; height: 2.5rem; padding: 0 1rem; box-sizing: border-box;
  white-space: nowrap;
  border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-lg);
  background: transparent;
  box-shadow: var(--cronus-shadow-xs, none);
  color: var(--cronus-fg);
  font: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 400;
  cursor: pointer; outline: none;
  transition: background 150ms var(--cronus-ease), box-shadow 150ms var(--cronus-ease);
}
[data-slot="date-picker-trigger"][data-empty] { color: var(--cronus-fg-tertiary); }
[data-slot="date-picker-trigger"]:hover {
  background: var(--cronus-surface-overlay);
}
[data-slot="date-picker-trigger"]:focus-visible {
  box-shadow: 0 0 0 2px var(--cronus-surface-base), 0 0 0 4px var(--cronus-ring, var(--cronus-primary));
}
[data-slot="date-picker-trigger"] svg {
  width: 1rem; height: 1rem; flex-shrink: 0; opacity: 0.7;
}
[data-slot="date-picker-trigger"] > span {
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
[data-slot="time-picker"] {
  display: inline-flex; align-items: center; justify-content: flex-start; gap: 0.5rem;
  width: 15rem; height: 2.5rem; padding: 0 1rem; box-sizing: border-box;
  white-space: nowrap;
  border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-lg);
  background: transparent;
  box-shadow: var(--cronus-shadow-xs, none);
  color: var(--cronus-fg);
  font: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 400;
  cursor: pointer; outline: none;
  transition: background 150ms var(--cronus-ease), box-shadow 150ms var(--cronus-ease);
}
[data-slot="time-picker"]:hover {
  background: var(--cronus-surface-overlay);
}
[data-slot="time-picker"]:focus-visible {
  box-shadow: 0 0 0 2px var(--cronus-surface-base), 0 0 0 4px var(--cronus-ring, var(--cronus-primary));
}
[data-slot="time-picker"] svg {
  width: 1rem; height: 1rem; flex-shrink: 0; pointer-events: none;
  color: var(--cronus-fg-tertiary);
}
[data-slot="time-picker"] > span {
  font-variant-numeric: tabular-nums;
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
  line-height: 1.25rem; cursor: pointer; text-decoration: none;
  outline: none; border: 0 solid transparent;
  font-family: inherit;
  transition: background 150ms var(--ease-out-quart), box-shadow 150ms var(--ease-out-quart),
    transform 150ms var(--ease-out-quart), opacity 150ms var(--ease-out-quart), border-color 150ms;
}
[data-slot="button"]:active { transform: scale(0.98); }
[data-slot="button"][data-size="sm"] { height: 2rem; padding: 0 0.75rem; font-size: 0.75rem; line-height: 1rem; }
[data-slot="button"][data-size="md"], [data-slot="button"]:not([data-size]) { height: 2.5rem; padding: 0 1rem; font-size: 0.875rem; }
[data-slot="button"][data-size="lg"] { height: 2.75rem; padding: 0 1.5rem; font-size: 1rem; line-height: 1.5rem; }
[data-slot="button"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="button"][data-size="icon"] { height: 2.25rem; width: 2.25rem; padding: 0; }
[data-slot="button"][data-size="icon-sm"] { height: 2rem; width: 2rem; padding: 0; }
[data-slot="button"][data-variant="primary"], [data-slot="button"]:not([data-variant]) {
  background: var(--cronus-primary); color: var(--cronus-primary-foreground);
  box-shadow: var(--cronus-shadow-xs, 0 1px 2px rgba(0,0,0,.2));
}
[data-slot="button"][data-variant="primary"]:hover { opacity: 0.9; }
[data-slot="button"][data-variant="secondary"] {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
  border-width: 1px; border-color: var(--cronus-border);
}
[data-slot="button"][data-variant="outline"] {
  background: transparent; color: var(--cronus-fg); border-width: 1px; border-color: var(--cronus-border);
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
[data-slot="input"] { line-height: 1.25rem; }

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
[data-slot="textarea"] { line-height: 1.25rem; }

[data-slot="badge"] {
  display: inline-flex; align-items: center; gap: 0.25rem;
  border-radius: var(--cronus-radius-md); border: 1px solid var(--cronus-border);
  padding: 0.125rem 0.5rem; font-size: 0.75rem; line-height: 1rem; font-weight: 500;
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
  background: var(--cronus-surface-inset);
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
[data-slot="checkbox"] > span {
  display: flex; align-items: center; justify-content: center; color: currentColor;
}
[data-slot="checkbox"] > span > svg { width: 0.875rem; height: 0.875rem; }

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
[data-slot="switch"] > span {
  pointer-events: none; display: block;
  width: 1rem; height: 1rem; border-radius: 9999px;
  background: var(--cronus-primary-foreground);
  box-shadow: var(--cronus-shadow-sm, 0 1px 2px rgba(0,0,0,.2));
  transform: translateX(0.125rem);
  transition: transform 150ms var(--ease-out-quart);
}
[data-slot="switch"][data-state="checked"] > span {
  transform: translateX(18px);
}

[data-slot="spinner"] {
  width: 1.25rem; height: 1.25rem; display: block;
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
  line-height: 1.25rem;
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
[data-slot="progress"] > div {
  height: 100%; width: 100%; flex: 1; background: var(--cronus-primary);
  transition: transform 300ms var(--ease-out-quart);
}

[data-slot="skeleton"] {
  display: block; height: 1rem; width: 8rem;
  border-radius: var(--cronus-radius-md);
  background: var(--cronus-surface-overlay);
  animation: cui-pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
@keyframes cui-pulse { 50% { opacity: 0.5; } }

[data-slot="slider"] {
  position: relative; display: flex; width: 100%; align-items: center;
}
[data-slot="slider"] > span:first-child {
  position: relative; display: block; height: 0.375rem; width: 100%; flex-grow: 1;
  overflow: hidden; border-radius: 9999px; background: var(--cronus-surface-overlay);
}
[data-slot="slider"] > span:first-child > span {
  position: absolute; display: block; height: 100%; left: 0;
  right: calc(100% - var(--cui-slider-value, 0) * 1%); background: var(--cronus-primary);
}
[data-slot="slider"] > span:last-child {
  position: absolute; transform: translateX(-50%);
  left: calc(var(--cui-slider-value, 0) * 1% + 0.5rem - var(--cui-slider-value, 0) * 0.01rem);
}
[data-slot="slider"] > span:last-child > span {
  display: block; width: 1rem; height: 1rem; box-sizing: border-box;
  border-radius: 9999px; border: 1px solid var(--cronus-primary);
  background: var(--cronus-surface-base);
  box-shadow: var(--cronus-shadow-sm, 0 1px 2px rgba(0,0,0,.2));
}
[data-slot="slider"][data-value="0"] { --cui-slider-value: 0; }
[data-slot="slider"][data-value="1"] { --cui-slider-value: 1; }
[data-slot="slider"][data-value="2"] { --cui-slider-value: 2; }
[data-slot="slider"][data-value="3"] { --cui-slider-value: 3; }
[data-slot="slider"][data-value="4"] { --cui-slider-value: 4; }
[data-slot="slider"][data-value="5"] { --cui-slider-value: 5; }
[data-slot="slider"][data-value="6"] { --cui-slider-value: 6; }
[data-slot="slider"][data-value="7"] { --cui-slider-value: 7; }
[data-slot="slider"][data-value="8"] { --cui-slider-value: 8; }
[data-slot="slider"][data-value="9"] { --cui-slider-value: 9; }
[data-slot="slider"][data-value="10"] { --cui-slider-value: 10; }
[data-slot="slider"][data-value="11"] { --cui-slider-value: 11; }
[data-slot="slider"][data-value="12"] { --cui-slider-value: 12; }
[data-slot="slider"][data-value="13"] { --cui-slider-value: 13; }
[data-slot="slider"][data-value="14"] { --cui-slider-value: 14; }
[data-slot="slider"][data-value="15"] { --cui-slider-value: 15; }
[data-slot="slider"][data-value="16"] { --cui-slider-value: 16; }
[data-slot="slider"][data-value="17"] { --cui-slider-value: 17; }
[data-slot="slider"][data-value="18"] { --cui-slider-value: 18; }
[data-slot="slider"][data-value="19"] { --cui-slider-value: 19; }
[data-slot="slider"][data-value="20"] { --cui-slider-value: 20; }
[data-slot="slider"][data-value="21"] { --cui-slider-value: 21; }
[data-slot="slider"][data-value="22"] { --cui-slider-value: 22; }
[data-slot="slider"][data-value="23"] { --cui-slider-value: 23; }
[data-slot="slider"][data-value="24"] { --cui-slider-value: 24; }
[data-slot="slider"][data-value="25"] { --cui-slider-value: 25; }
[data-slot="slider"][data-value="26"] { --cui-slider-value: 26; }
[data-slot="slider"][data-value="27"] { --cui-slider-value: 27; }
[data-slot="slider"][data-value="28"] { --cui-slider-value: 28; }
[data-slot="slider"][data-value="29"] { --cui-slider-value: 29; }
[data-slot="slider"][data-value="30"] { --cui-slider-value: 30; }
[data-slot="slider"][data-value="31"] { --cui-slider-value: 31; }
[data-slot="slider"][data-value="32"] { --cui-slider-value: 32; }
[data-slot="slider"][data-value="33"] { --cui-slider-value: 33; }
[data-slot="slider"][data-value="34"] { --cui-slider-value: 34; }
[data-slot="slider"][data-value="35"] { --cui-slider-value: 35; }
[data-slot="slider"][data-value="36"] { --cui-slider-value: 36; }
[data-slot="slider"][data-value="37"] { --cui-slider-value: 37; }
[data-slot="slider"][data-value="38"] { --cui-slider-value: 38; }
[data-slot="slider"][data-value="39"] { --cui-slider-value: 39; }
[data-slot="slider"][data-value="40"] { --cui-slider-value: 40; }
[data-slot="slider"][data-value="41"] { --cui-slider-value: 41; }
[data-slot="slider"][data-value="42"] { --cui-slider-value: 42; }
[data-slot="slider"][data-value="43"] { --cui-slider-value: 43; }
[data-slot="slider"][data-value="44"] { --cui-slider-value: 44; }
[data-slot="slider"][data-value="45"] { --cui-slider-value: 45; }
[data-slot="slider"][data-value="46"] { --cui-slider-value: 46; }
[data-slot="slider"][data-value="47"] { --cui-slider-value: 47; }
[data-slot="slider"][data-value="48"] { --cui-slider-value: 48; }
[data-slot="slider"][data-value="49"] { --cui-slider-value: 49; }
[data-slot="slider"][data-value="50"] { --cui-slider-value: 50; }
[data-slot="slider"][data-value="51"] { --cui-slider-value: 51; }
[data-slot="slider"][data-value="52"] { --cui-slider-value: 52; }
[data-slot="slider"][data-value="53"] { --cui-slider-value: 53; }
[data-slot="slider"][data-value="54"] { --cui-slider-value: 54; }
[data-slot="slider"][data-value="55"] { --cui-slider-value: 55; }
[data-slot="slider"][data-value="56"] { --cui-slider-value: 56; }
[data-slot="slider"][data-value="57"] { --cui-slider-value: 57; }
[data-slot="slider"][data-value="58"] { --cui-slider-value: 58; }
[data-slot="slider"][data-value="59"] { --cui-slider-value: 59; }
[data-slot="slider"][data-value="60"] { --cui-slider-value: 60; }
[data-slot="slider"][data-value="61"] { --cui-slider-value: 61; }
[data-slot="slider"][data-value="62"] { --cui-slider-value: 62; }
[data-slot="slider"][data-value="63"] { --cui-slider-value: 63; }
[data-slot="slider"][data-value="64"] { --cui-slider-value: 64; }
[data-slot="slider"][data-value="65"] { --cui-slider-value: 65; }
[data-slot="slider"][data-value="66"] { --cui-slider-value: 66; }
[data-slot="slider"][data-value="67"] { --cui-slider-value: 67; }
[data-slot="slider"][data-value="68"] { --cui-slider-value: 68; }
[data-slot="slider"][data-value="69"] { --cui-slider-value: 69; }
[data-slot="slider"][data-value="70"] { --cui-slider-value: 70; }
[data-slot="slider"][data-value="71"] { --cui-slider-value: 71; }
[data-slot="slider"][data-value="72"] { --cui-slider-value: 72; }
[data-slot="slider"][data-value="73"] { --cui-slider-value: 73; }
[data-slot="slider"][data-value="74"] { --cui-slider-value: 74; }
[data-slot="slider"][data-value="75"] { --cui-slider-value: 75; }
[data-slot="slider"][data-value="76"] { --cui-slider-value: 76; }
[data-slot="slider"][data-value="77"] { --cui-slider-value: 77; }
[data-slot="slider"][data-value="78"] { --cui-slider-value: 78; }
[data-slot="slider"][data-value="79"] { --cui-slider-value: 79; }
[data-slot="slider"][data-value="80"] { --cui-slider-value: 80; }
[data-slot="slider"][data-value="81"] { --cui-slider-value: 81; }
[data-slot="slider"][data-value="82"] { --cui-slider-value: 82; }
[data-slot="slider"][data-value="83"] { --cui-slider-value: 83; }
[data-slot="slider"][data-value="84"] { --cui-slider-value: 84; }
[data-slot="slider"][data-value="85"] { --cui-slider-value: 85; }
[data-slot="slider"][data-value="86"] { --cui-slider-value: 86; }
[data-slot="slider"][data-value="87"] { --cui-slider-value: 87; }
[data-slot="slider"][data-value="88"] { --cui-slider-value: 88; }
[data-slot="slider"][data-value="89"] { --cui-slider-value: 89; }
[data-slot="slider"][data-value="90"] { --cui-slider-value: 90; }
[data-slot="slider"][data-value="91"] { --cui-slider-value: 91; }
[data-slot="slider"][data-value="92"] { --cui-slider-value: 92; }
[data-slot="slider"][data-value="93"] { --cui-slider-value: 93; }
[data-slot="slider"][data-value="94"] { --cui-slider-value: 94; }
[data-slot="slider"][data-value="95"] { --cui-slider-value: 95; }
[data-slot="slider"][data-value="96"] { --cui-slider-value: 96; }
[data-slot="slider"][data-value="97"] { --cui-slider-value: 97; }
[data-slot="slider"][data-value="98"] { --cui-slider-value: 98; }
[data-slot="slider"][data-value="99"] { --cui-slider-value: 99; }
[data-slot="slider"][data-value="100"] { --cui-slider-value: 100; }

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
[data-slot="radio-group-item"] > span { display: flex; align-items: center; justify-content: center; }
[data-slot="radio-group-item"] > span > svg {
  width: 0.5rem; height: 0.5rem; fill: var(--cronus-primary); color: var(--cronus-primary);
}

[data-slot="chip"] {
  display: inline-flex; align-items: center; height: 1.75rem;
  border-radius: 9999px; border: 1px solid var(--cronus-border);
  padding: 0 0.625rem; font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;
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
  font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;
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
  min-width: 0; overflow-wrap: break-word;
  font-family: var(--cronus-font-display, inherit); font-weight: 600; line-height: 1;
  color: var(--cronus-fg);
}
[data-slot="card-description"] {
  grid-column: 1 / -1; font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg-secondary);
}
[data-slot="card-content"] {
  min-width: 0; padding-left: 1.5rem; padding-right: 1.5rem;
  font-size: 0.875rem; color: var(--cronus-fg-secondary);
}
@media (max-width: 639.98px) {
  [data-slot="card-header"], [data-slot="card-content"] { padding-left: 1rem; padding-right: 1rem; }
}

[data-slot="empty"] {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  gap: 0.75rem; border-radius: var(--cronus-radius-xl);
  border: 1px dashed var(--cronus-border);
  background: color-mix(in oklch, var(--cronus-surface-inset) 40%, transparent);
  padding: 3rem 1.5rem; text-align: center;
}
[data-slot="empty-title"] {
  font-family: var(--cronus-font-display, inherit);
  font-size: 0.875rem; line-height: 1.25rem; font-weight: 600; color: var(--cronus-fg);
}
[data-slot="empty-description"] {
  max-width: 24rem; font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg-secondary);
}

[data-slot="alert"] {
  position: relative; display: grid; width: 100%; box-sizing: border-box;
  grid-template-columns: 0 1fr; align-items: start; row-gap: 0.25rem;
  border: 1px solid var(--cronus-border); border-radius: var(--cronus-radius-lg);
  padding: 0.75rem 1rem; font-size: 0.875rem; line-height: 1.25rem;
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="alert"][role="alert"] {
  background: color-mix(in oklch, var(--cronus-error) 10%, transparent);
  border-color: color-mix(in oklch, var(--cronus-error) 30%, transparent);
}
[data-slot="alert-title"] {
  grid-column-start: 2; min-height: 1rem;
  display: -webkit-box; -webkit-box-orient: vertical; -webkit-line-clamp: 1; overflow: hidden;
  font-weight: 500; color: var(--cronus-fg);
}
[data-slot="alert-description"] {
  grid-column-start: 2; display: grid; justify-items: start; gap: 0.25rem;
  font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg-secondary);
}

[data-slot="banner"] {
  display: flex; width: 100%; align-items: center; gap: 0.25rem 0.75rem;
  box-sizing: border-box;
  border-bottom: 1px solid var(--cronus-border);
  padding: 0.625rem 1rem; font-size: 0.875rem; line-height: 1.25rem; text-align: center;
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="banner-content"] {
  display: flex; min-width: 0; flex: 1; flex-wrap: wrap;
  align-items: center; gap: 0.125rem 0.5rem;
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
  margin: 0; font-size: 0.75rem; line-height: 1rem; color: var(--cronus-fg-secondary);
}
[data-slot="input-group"] {
  display: flex; height: 2.5rem; width: 100%; align-items: stretch;
  overflow: hidden; box-sizing: border-box;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
  font-size: 0.875rem; line-height: 1.25rem;
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
  border-radius: var(--cronus-radius-md);
}
[data-slot="rating"] svg { width: 1.25rem; height: 1.25rem; }
[data-slot="rating-item"] { display: inline-flex; cursor: default; }
[data-slot="rating-star"] { position: relative; display: inline-flex; flex-shrink: 0; }
[data-slot="rating-star"] > svg { color: var(--cronus-fg-muted); }
[data-slot="rating-star"] > span { position: absolute; inset: 0; overflow: hidden; width: 0%; }
[data-slot="rating-item"][data-state="on"] [data-slot="rating-star"] > span { width: 100%; }
[data-slot="rating-star"] > span > svg { color: var(--cronus-warning); fill: var(--cronus-warning); }
[data-slot="copy-button"] {
  display: inline-flex; align-items: center; justify-content: center; gap: 0.5rem;
  white-space: nowrap; border-radius: var(--cronus-radius-lg); font-weight: 500;
  line-height: 1.25rem; cursor: pointer; text-decoration: none;
  outline: none; border: 0;
  font-family: inherit; font-size: 0.875rem;
  width: 2.25rem; height: 2.25rem; padding: 0;
  background: transparent; color: var(--cronus-fg-secondary);
  transition: background 150ms var(--ease-out-quart), box-shadow 150ms var(--ease-out-quart),
    transform 150ms var(--ease-out-quart), opacity 150ms var(--ease-out-quart), border-color 150ms;
}
[data-slot="copy-button"]:active { transform: scale(0.98); }
[data-slot="copy-button"]:hover { background: var(--cronus-surface-overlay); color: var(--cronus-fg); }
[data-slot="copy-button"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="copy-button"] svg {
  width: 1rem; height: 1rem; flex-shrink: 0; pointer-events: none;
}
[data-slot="copy-button"] > [aria-live] {
  position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;
  overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border-width: 0;
}
[data-slot="fab"] {
  position: relative; display: inline-flex;
}
[data-slot="fab"] > button > span { display: inline-flex; }
[data-slot="fab"] > button svg {
  width: 1.5rem; height: 1.5rem; flex-shrink: 0; pointer-events: none;
}
[data-slot="fab"] > button {
  display: inline-flex; align-items: center; justify-content: center;
  width: 3.5rem; height: 3.5rem; padding: 0; margin: 0;
  border: 0; border-radius: 9999px;
  background: var(--cronus-primary); color: var(--cronus-primary-foreground);
  box-shadow: var(--cronus-shadow-md, 0 4px 6px rgba(0,0,0,.2));
  cursor: pointer; outline: none; font: inherit;
  transition: transform 150ms var(--ease-out-quart), box-shadow 150ms var(--ease-out-quart),
    opacity 150ms var(--ease-out-quart);
}
[data-slot="fab"] > button:hover { opacity: 0.9; }
[data-slot="fab"] > button:active { transform: scale(0.95); }
[data-slot="fab"] > button:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="toggle-group"] {
  display: flex; align-items: center; gap: 0.25rem;
}
[data-slot="toggle-group-item"] {
  height: 2.5rem; padding: 0 0.75rem; border-radius: var(--cronus-radius-lg);
  border: 0; background: transparent; color: var(--cronus-fg-secondary);
  font-size: 0.875rem; line-height: 1.25rem; font-weight: 500; font-family: inherit;
  cursor: pointer; outline: none;
  transition: background 150ms var(--ease-out-quart), color 150ms var(--ease-out-quart);
}
[data-slot="toggle-group-item"][data-state="on"] {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="toggle-group-item"][data-disabled] { opacity: 0.5; pointer-events: none; }
[data-slot="metric"] {
  display: flex; flex-direction: column; gap: 0.25rem;
}
[data-slot="metric-label"] {
  font-size: 0.75rem; line-height: 1rem; font-weight: 500; text-transform: uppercase;
  letter-spacing: 0.05em; color: var(--cronus-fg-tertiary);
}
[data-slot="metric-value"] {
  font-family: var(--cronus-font-display, inherit);
  font-size: 1.5rem; line-height: 2rem; font-weight: 600; color: var(--cronus-fg);
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
  font-weight: 500; font-size: 0.875rem; line-height: 1.25rem;
  box-shadow: 0 0 0 2px var(--cronus-surface-base);
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
[data-slot="button-group"] > [data-slot="button"] { line-height: 1.25rem; }
[data-slot="button-group"] > [data-slot="button"][data-variant="primary"] { border-width: 0; }
[data-slot="combobox-trigger"] {
  display: inline-flex; align-items: center; justify-content: space-between; gap: 0.5rem;
  width: 100%; height: 2.5rem; padding: 0 1rem; box-sizing: border-box; white-space: nowrap;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  background: transparent; color: var(--cronus-fg); box-shadow: var(--cronus-shadow-xs, none);
  font-family: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 400; cursor: default;
}
[data-slot="combobox-trigger"][data-placeholder] { color: var(--cronus-fg-tertiary); }
[data-slot="combobox-trigger"] > span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
[data-slot="combobox-trigger"] > svg { width: 1rem; height: 1rem; flex-shrink: 0; margin-left: 0.5rem; opacity: 0.6; }
[data-slot="combobox-trigger"][data-disabled] { opacity: 0.5; }
[data-slot="combobox-content"]:popover-open {
  display: flex; flex-direction: column; min-width: 16rem; padding: 0.25rem;
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
  flex-direction: row;
}
[data-slot="stepper"][data-orientation="vertical"] {
  flex-direction: column;
}
[data-slot="stepper-list"] {
  display: flex; width: 100%; list-style: none; margin: 0; padding: 0;
}
[data-slot="stepper-list"][data-orientation="horizontal"] { flex-direction: row; align-items: center; }
[data-slot="stepper-list"][data-orientation="vertical"] { flex-direction: column; }
[data-slot="stepper-item"] {
  position: relative; display: flex;
}
[data-slot="stepper-item"][data-orientation="horizontal"] { flex-direction: row; align-items: center; }
[data-slot="stepper-item"][data-orientation="horizontal"]:not(:last-child) { flex: 1; }
[data-slot="stepper-item"][data-orientation="vertical"] { flex-direction: column; }
[data-slot="stepper-indicator"] {
  display: inline-flex; width: 2rem; height: 2rem; flex-shrink: 0; user-select: none;
  align-items: center; justify-content: center; box-sizing: border-box;
  border-radius: 9999px; font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;
}
[data-slot="stepper-indicator"] svg { width: 1rem; height: 1rem; flex-shrink: 0; }
[data-slot="stepper-indicator"][data-state="active"] {
  background: var(--cronus-primary); color: #fff;
  box-shadow: var(--cronus-shadow-xs, none);
}
[data-slot="stepper-indicator"][data-state="completed"] {
  background: color-mix(in oklab, var(--cronus-primary) 15%, transparent);
  color: var(--cronus-primary-text);
}
[data-slot="stepper-indicator"][data-state="upcoming"] {
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-overlay); color: var(--cronus-fg-tertiary);
}
[data-slot="stepper-item-state"] {
  position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;
  overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border-width: 0;
}
[data-slot="stepper-title"] {
  font-size: 0.875rem; font-weight: 500; line-height: 1; color: var(--cronus-fg);
}
[data-slot="stepper-item"][data-state="upcoming"] [data-slot="stepper-title"] {
  color: var(--cronus-fg-tertiary);
}
[data-input-otp-container] {
  position: relative; display: flex; align-items: center; gap: 0.5rem;
  line-height: 1.5; cursor: text; user-select: none; pointer-events: none;
}
[data-input-otp-container] > div:last-child {
  position: absolute; inset: 0; pointer-events: none;
}
[data-slot="input-otp"] {
  position: absolute; inset: 0; width: 100%; height: 100%;
  display: flex; margin: 0; padding: 0; box-sizing: border-box;
  opacity: 1; color: transparent; background: transparent; caret-color: transparent;
  border: 0 solid transparent; outline: 0 solid transparent; box-shadow: none;
  pointer-events: all; text-align: left; line-height: 1; letter-spacing: -0.5em;
  font-size: 2.5rem; font-family: monospace;
}
[data-slot="input-otp-group"] {
  display: flex; align-items: center;
}
[data-slot="input-otp-slot"] {
  position: relative; display: flex; align-items: center; justify-content: center;
  height: 2.5rem; width: 2.5rem; box-sizing: border-box;
  border: 0 solid var(--cronus-border);
  border-top-width: 1px; border-right-width: 1px; border-bottom-width: 1px;
  font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg);
  font-variant-numeric: tabular-nums;
}
[data-slot="input-otp-slot"]:first-child {
  border-left-width: 1px;
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
  font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg-secondary);
}
[data-slot="file-dropzone"] > svg { width: 1.5rem; height: 1.5rem; color: var(--cronus-fg-muted); }
[data-slot="file-dropzone"] > span > span { font-weight: 500; color: var(--cronus-fg); }
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
}
[data-slot="dropdown-menu-content"]:popover-open {
  z-index: 50; min-width: 8rem; overflow: hidden; box-sizing: border-box;
  padding: 0.25rem; border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-lg);
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
  box-shadow: var(--cronus-shadow-lg, none);
}
[data-slot="dropdown-menu-item"] {
  position: relative; display: flex; align-items: center; gap: 0.5rem;
  border-radius: var(--cronus-radius-md); padding: 0.375rem 0.5rem;
  font-size: 0.875rem; line-height: 1.25rem; cursor: default; user-select: none; outline: none;
}
[data-slot="dropdown-menu-item"]:hover {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="collapsible"] {
  display: flex; flex-direction: column; gap: 0.25rem;
}
[data-slot="collapsible"] > button {
  display: inline-flex; align-items: center;
}
[data-slot="collapsible-content"] {
  overflow: hidden; font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg-secondary);
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
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="mode-toggle"][data-disabled] { opacity: 0.5; pointer-events: none; }
[data-slot="mode-toggle"] svg { width: 1.25rem; height: 1.25rem; flex-shrink: 0; }
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
  display: flex; flex-direction: column; overflow: hidden;
  width: 18rem; height: 12rem; box-sizing: border-box;
  border-radius: var(--cronus-radius-lg);
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
}
[data-slot="command"] > label {
  position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;
  overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border-width: 0;
}
[data-slot="command-input-wrapper"] {
  display: flex; align-items: center; gap: 0.5rem;
  border-bottom: 1px solid var(--cronus-border); padding: 0 0.75rem;
}
[data-slot="command-input-wrapper"] > svg {
  width: 1rem; height: 1rem; flex-shrink: 0; color: var(--cronus-fg-tertiary);
}
[data-slot="command-input"] {
  display: flex; width: 100%; height: 2.5rem; box-sizing: border-box;
  border: 0; background: transparent; color: var(--cronus-fg);
  padding: 0.75rem 0; font-family: inherit; font-size: 0.875rem; line-height: 1.25rem;
  outline: none;
}
[data-slot="command-input"]::placeholder { color: var(--cronus-fg-tertiary); }
[data-slot="command-list"] {
  max-height: 20rem; overflow-y: auto; overflow-x: hidden; padding: 0.25rem;
  scroll-padding-block: 0.25rem;
}
[data-slot="command-item"] {
  position: relative; display: flex; align-items: center; gap: 0.5rem;
  border-radius: var(--cronus-radius-md); padding: 0.375rem 0.5rem;
  font-size: 0.875rem; line-height: 1.25rem; cursor: default; user-select: none; outline: none;
}
[data-slot="command-list"]:not(:hover) [data-slot="command-item"][data-selected="true"],
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
  padding: 0.25rem 0.75rem; background: transparent; color: var(--cronus-fg);
  font: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;
  cursor: default; user-select: none; outline: none;
}
[data-slot="menubar-trigger"][data-state="open"],
[data-slot="menubar-trigger"]:hover {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="menubar-content"]:not(:popover-open) { display: none; }
[data-slot="menubar-content"]:popover-open {
  z-index: 50; min-width: 12rem; overflow: hidden; box-sizing: border-box;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
  padding: 0.25rem; box-shadow: var(--cronus-shadow-lg, none);
}
[data-slot="menubar-item"] {
  position: relative; display: flex; align-items: center; gap: 0.5rem;
  border-radius: var(--cronus-radius-md); padding: 0.375rem 0.5rem;
  font-size: 0.875rem; line-height: 1.25rem; cursor: default; user-select: none; outline: none;
}
[data-slot="menubar-item"]:hover {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
button:has(+ [data-slot="context-menu-content"]) {
  font: inherit; cursor: pointer; color: var(--cronus-fg);
  background: transparent; border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-lg);
  height: 2.5rem; padding: 0 1rem; font-size: 0.875rem;
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
}
button:has(+ [data-slot="sheet-content"]) {
}
[data-slot="sheet-content"]:popover-open {
  z-index: 50;
}
[data-slot="sheet-title"] {
  font-size: 1.125rem; color: var(--cronus-fg);
}
[data-slot="sheet-description"] {
  font-size: 0.875rem; color: var(--cronus-fg-secondary);
}
[data-slot="calendar"] {
  display: block; color: var(--cronus-fg);
}
[data-slot="calendar"] > div { padding: 0.75rem; }
[data-slot="calendar"] > div > div {
  position: relative; display: flex; flex-direction: column; gap: 1rem;
}
@media (min-width: 40rem) {
  [data-slot="calendar"] > div > div { flex-direction: row; }
}
[data-slot="calendar"] > div > div > div {
  display: flex; flex-direction: column; gap: 1rem;
}
[data-slot="calendar"] div:has(> [role="status"]) {
  position: relative; display: flex; align-items: center; justify-content: center;
  padding-top: 0.25rem;
}
[data-slot="calendar"] [role="status"] {
  font-size: 0.875rem; line-height: 1.25rem; font-weight: 500; color: var(--cronus-fg);
}
[data-slot="calendar"] table {
  width: 100%; border-collapse: collapse;
}
[data-slot="calendar"] tr { display: flex; }
[data-slot="calendar"] tbody tr { width: 100%; margin-top: 0.5rem; }
[data-slot="calendar"] th {
  width: 2.25rem; padding: 0; border-radius: var(--cronus-radius-md);
  font-size: 0.75rem; line-height: 1rem; font-weight: 400;
  color: var(--cronus-fg-tertiary);
}
[data-slot="calendar"] td {
  position: relative; padding: 0; text-align: center;
  font-size: 0.875rem; line-height: 1.25rem;
}
[data-slot="calendar"] nav {
  position: absolute; inset-inline: 0; top: 0; display: flex; align-items: center; gap: 0.25rem;
}
[data-slot="calendar"] nav > button {
  position: absolute; top: 0; display: inline-flex; align-items: center; justify-content: center;
  width: 1.75rem; height: 1.75rem; padding: 0; box-sizing: border-box;
  border: 1px solid var(--cronus-border); border-radius: var(--cronus-radius-lg);
  background: transparent; color: var(--cronus-fg); box-shadow: var(--cronus-shadow-xs, none);
  opacity: 0.6; cursor: default;
}
[data-slot="calendar"] nav > button:first-child { inset-inline-start: 0.25rem; }
[data-slot="calendar"] nav > button:last-child { inset-inline-end: 0.25rem; }
[data-slot="calendar"] nav > button > svg { width: 1rem; height: 1rem; }
[data-slot="calendar"] td > button {
  display: flex; width: 2.25rem; height: 2.25rem; align-items: center; justify-content: center;
  padding: 0; border: 0; border-radius: var(--cronus-radius-lg);
  background: transparent; color: var(--cronus-fg-secondary);
  font: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 400; cursor: default;
}
[data-slot="calendar"] td[data-outside="true"] > button { color: var(--cronus-fg-muted); }
[data-slot="calendar"] td[aria-selected="true"] > button {
  background: var(--cronus-primary); color: var(--cronus-primary-foreground);
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
[data-slot="time-picker"]:disabled { opacity: 0.5; pointer-events: none; }
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
  display: inline-flex; align-items: center; justify-content: flex-start; gap: 0.5rem;
  width: 18.75rem; height: 2.5rem; padding: 0 1rem; box-sizing: border-box;
  white-space: nowrap;
  border: 1px solid var(--cronus-border); border-radius: var(--cronus-radius-lg);
  background: transparent; box-shadow: var(--cronus-shadow-xs, none);
  color: var(--cronus-fg);
  font: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 400;
  cursor: pointer; outline: none;
}
[data-slot="date-range-picker-trigger"][data-empty] { color: var(--cronus-fg-tertiary); }
[data-slot="date-range-picker-trigger"]:hover { background: var(--cronus-surface-overlay); }
[data-slot="date-range-picker-trigger"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="date-range-picker-trigger"] svg { width: 1rem; height: 1rem; flex-shrink: 0; opacity: 0.7; }
[data-slot="date-range-picker-trigger"] > span {
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
[data-slot="date-range-picker-content"]:not(:popover-open) { display: none; }
[data-slot="date-range-picker-content"]:popover-open {
  z-index: 50; padding: 0; outline: none;
  display: flex; flex-direction: row;
  border: 1px solid var(--cronus-border); border-radius: var(--cronus-radius-lg);
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
  box-shadow: var(--cronus-shadow-lg, none);
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
  display: block; width: 100%; height: 16rem;
}
[data-slot="bar-chart"] {
  display: block; width: 100%; height: 16rem;
}
[data-slot="line-chart"] {
  display: block; width: 100%; height: 16rem;
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
  display: block; width: 100%; height: 16rem;
}
[data-slot="radar-chart"] {
  display: block; width: 100%; height: 16rem;
}
[data-slot="scatter-chart"] {
  display: block; width: 100%; height: 16rem;
}
[data-slot="ring-chart"] {
  display: block; width: 100%; height: 16rem;
}
[data-slot="ring-chart"] tspan:first-child {
  fill: var(--cronus-fg); font-size: 1.5rem; font-weight: 500;
}
[data-slot="ring-chart"] tspan + tspan {
  fill: var(--cronus-fg-tertiary); font-size: 0.75rem;
}
[data-slot="data-table"] {
  display: flex; flex-direction: column; gap: 0.75rem;
  color: var(--cronus-fg);
}
[data-slot="data-table-container"] {
  overflow: hidden; border-radius: var(--cronus-radius-xl); border: 1px solid var(--cronus-border);
}
[data-slot="data-table"] [data-slot="table-container"] {
  position: relative; display: block; width: 100%; overflow-x: auto; outline: none;
}
[data-slot="data-table"] [data-slot="table"] {
  width: 100%; caption-side: bottom; font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg);
}
[data-slot="data-table"] [data-slot="table-row"] { border-bottom: 1px solid var(--cronus-border); }
[data-slot="data-table"] [data-slot="table-body"] > [data-slot="table-row"]:last-child { border-bottom: 0; }
[data-slot="data-table"] [data-slot="table-head"] {
  height: 2.5rem; padding: 0 0.75rem; text-align: left; vertical-align: middle;
  font-size: inherit; font-weight: 500; color: var(--cronus-fg-secondary); white-space: nowrap; border: 0;
}
[data-slot="data-table"] [data-slot="table-cell"] {
  padding: 0.75rem; vertical-align: middle; white-space: nowrap; border: 0; color: inherit;
}
[data-slot="sidebar-wrapper"] {
  display: flex; width: 100%; min-height: 100svh;
}
[data-audit-canvas] > [data-slot="sidebar-wrapper"]:not([data-app-shell]) {
  width: 13rem; height: 14rem; min-height: 0;
}
[data-slot="sidebar"] {
  position: relative; display: flex; flex-direction: column;
  width: 16rem; height: 100svh; box-sizing: border-box;
  background: var(--cronus-surface-base); color: var(--cronus-fg);
  border-right: 1px solid var(--cronus-border);
}
[data-audit-canvas] [data-slot="sidebar"] { height: 100%; }
[data-slot="sidebar"] > nav {
  display: flex; height: 100%; width: 100%; flex-direction: column;
}
[data-slot="sidebar-content"] {
  min-height: 0; flex: 1;
}
[data-slot="sidebar-content"] > [data-slot="scroll-area"] {
  position: relative; overflow: hidden; height: 100%; max-height: none;
}
[data-slot="sidebar-content"] > [data-slot="scroll-area"] > div {
  display: flex; flex-direction: column; gap: 0.5rem; padding: 0.5rem;
}
[data-slot="sidebar-menu"] {
  list-style: none; margin: 0; padding: 0;
  display: flex; width: 100%; min-width: 0; flex-direction: column; gap: 0.25rem;
}
[data-slot="sidebar-menu-item"] { position: relative; }
[data-slot="sidebar-menu-button"] {
  display: flex; align-items: center; gap: 0.5rem; overflow: hidden;
  width: 100%; box-sizing: border-box;
  height: 2rem; padding: 0 0.5rem; border: 0;
  border-radius: var(--cronus-radius-md);
  background: transparent; color: var(--cronus-fg-secondary); text-decoration: none;
  font-family: inherit; font-size: 0.875rem; line-height: 1.25rem; text-align: left;
  cursor: pointer; outline: none;
}
[data-slot="sidebar-menu-button"]:hover,
[data-slot="sidebar-menu-button"][data-active="true"] {
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
}
[data-slot="sidebar-menu-button"][data-active="true"] { font-weight: 500; }
[data-slot="toaster"] { display: block; }
[data-slot="navigation-menu"] {
  position: relative; z-index: 10;
  display: flex; max-width: max-content; flex: 1;
  align-items: center; justify-content: center;
}
[data-slot="navigation-menu"] > div { position: relative; }
[data-slot="navigation-menu-list"] {
  display: flex; flex: 1; list-style: none; margin: 0; padding: 0;
  align-items: center; justify-content: center; gap: 0.25rem;
}
[data-slot="navigation-menu-item"] { position: relative; }
[data-slot="navigation-menu-trigger"] {
  display: inline-flex; width: max-content; height: 2.25rem; box-sizing: border-box;
  align-items: center; justify-content: center; gap: 0.25rem;
  padding: 0.5rem 1rem; border: 0; border-radius: var(--cronus-radius-lg);
  background: transparent; color: var(--cronus-fg-secondary);
  font: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;
  cursor: pointer; outline: none;
  transition: background 150ms var(--cronus-ease), color 150ms var(--cronus-ease);
}
[data-slot="navigation-menu-trigger"][data-state="open"],
[data-slot="navigation-menu-trigger"]:hover {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="navigation-menu-trigger"] > svg {
  position: relative; top: 1px; width: 0.875rem; height: 0.875rem; flex-shrink: 0;
  color: var(--cronus-fg-tertiary);
}
[data-slot="phone-input"] {
  display: flex; height: 2.5rem; width: 100%; align-items: center;
  box-sizing: border-box;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
}
[data-slot="phone-input"][data-invalid="true"] {
  border-color: var(--cronus-error);
}
[data-slot="phone-input-country"] {
  display: flex; height: 100%; flex-shrink: 0; align-items: center; gap: 0.375rem;
  padding: 0 0.5rem 0 0.75rem; border: 0; outline: none;
  border-start-start-radius: var(--cronus-radius-lg); border-end-start-radius: var(--cronus-radius-lg);
  background: transparent; color: var(--cronus-fg-secondary);
  font: inherit; font-size: 0.875rem; line-height: 1.25rem; cursor: default;
}
[data-slot="phone-input"][aria-disabled="true"] { opacity: 0.5; pointer-events: none; }
[data-slot="phone-input-country"] > span:first-child { font-size: 1rem; line-height: 1; }
[data-slot="phone-input-country"] > span:nth-child(2) { font-variant-numeric: tabular-nums; }
[data-slot="phone-input-country"] > svg { width: 0.875rem; height: 0.875rem; flex-shrink: 0; color: var(--cronus-fg-tertiary); }
[data-slot="phone-input"] > span[aria-hidden="true"] {
  height: 1.25rem; width: 1px; flex-shrink: 0; background: var(--cronus-border);
}
[data-slot="phone-input-field"] {
  height: 100%; width: auto; flex: 1; min-width: 0;
  border: 0; border-radius: 0; background: transparent; box-shadow: none;
  padding: 0 0.75rem; color: var(--cronus-fg); font: inherit;
  font-size: 0.875rem; line-height: 1.25rem; outline: none;
}
[data-slot="phone-input-field"]::placeholder { color: var(--cronus-fg-tertiary); }
[data-slot="currency-input"] {
  display: flex; height: 2.5rem; width: 100%; align-items: stretch; overflow: hidden;
  box-sizing: border-box;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
  box-shadow: var(--cronus-shadow-xs, none);
}
[data-slot="currency-input"][data-invalid=""] { border-color: var(--cronus-error); }
[data-slot="currency-input"][data-disabled=""] { opacity: 0.5; pointer-events: none; }
[data-slot="currency-input-selector"],
[data-slot="currency-input-prefix"] {
  display: flex; flex-shrink: 0; align-items: center; gap: 0.375rem; user-select: none;
  border: 0; border-inline-end: 1px solid var(--cronus-border); outline: none;
  background: transparent; color: var(--cronus-fg);
  padding: 0 0.75rem; font-family: inherit; font-size: 0.875rem; line-height: 1.25rem;
}
[data-slot="currency-input-selector"] { padding: 0 0.625rem 0 0.75rem; cursor: default; }
[data-slot="currency-input-selector"] > span:first-child,
[data-slot="currency-input-prefix"] > span:first-child { font-weight: 500; color: var(--cronus-fg); }
[data-slot="currency-input-selector"] > span:nth-child(2),
[data-slot="currency-input-prefix"] > span:nth-child(2) {
  font-size: 0.75rem; line-height: 1rem; font-weight: 500; color: var(--cronus-fg-tertiary);
}
[data-slot="currency-input-selector"] > svg { width: 0.875rem; height: 0.875rem; flex-shrink: 0; color: var(--cronus-fg-tertiary); }
[data-slot="currency-input-field"] {
  min-width: 0; flex: 1; border: 0; outline: none; background: transparent;
  padding: 0 0.75rem; text-align: end; font-variant-numeric: tabular-nums;
  color: var(--cronus-fg); font-size: 0.875rem; line-height: 1.25rem;
}
[data-slot="currency-input-field"]::placeholder { color: var(--cronus-fg-tertiary); }
[data-slot="color-picker-trigger"] {
  display: inline-flex; align-items: center; justify-content: flex-start; gap: 0.5rem;
  width: 100%; height: 2.5rem; padding: 0 1rem; box-sizing: border-box; white-space: nowrap;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  background: transparent; color: var(--cronus-fg); box-shadow: var(--cronus-shadow-xs, none);
  font-family: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 400; cursor: default;
}
[data-slot="color-picker-trigger"][data-disabled] { opacity: 0.5; }
[data-slot="color-picker-trigger"] > span:last-child {
  overflow: hidden; text-overflow: ellipsis; font-variant-numeric: tabular-nums;
}
[data-slot="color-picker-swatch"] {
  width: 1.25rem; height: 1.25rem; flex-shrink: 0; box-sizing: border-box;
  border-radius: var(--cronus-radius-md); border: 1px solid var(--cronus-border);
  background-color: var(--cronus-primary);
  background-color: attr(data-color type(<color>), var(--cronus-primary));
}[data-slot="scroll-area"] {
  position: relative; overflow: hidden;
  width: 12rem; height: 8rem;
}
[data-slot="scroll-area"] > [role="region"] {
  width: 100%; height: 100%; border-radius: inherit; outline: none;
  overflow-x: hidden; overflow-y: scroll; scrollbar-width: none;
}
[data-slot="scroll-area"] > [role="region"]::-webkit-scrollbar { display: none; }
[data-slot="scroll-area"] > [role="region"]:focus-visible {
  box-shadow: 0 0 0 2px var(--cronus-surface-base), 0 0 0 4px var(--cronus-ring);
}
[data-slot="scroll-area"] > [role="region"] > div {
  display: flex; flex-direction: column; gap: 0.25rem; padding: 0.5rem;
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
  font: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 500; cursor: pointer;
  background: transparent; color: var(--cronus-fg-secondary);
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
  font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg-secondary);
}
[data-slot="status-dot-sr-label"] {
  position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;
  overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border-width: 0;
}
[data-slot="tags-input"] {
  display: flex; min-height: 2.5rem; width: 100%; flex-wrap: wrap;
  align-items: center; gap: 0.375rem; box-sizing: border-box;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
  padding: 0.375rem 0.75rem; font-size: 0.875rem; line-height: 1.25rem;
}
[data-slot="tags-input"][data-disabled=""] { opacity: 0.5; pointer-events: none; }
[data-slot="tags-input"][aria-invalid="true"] { border-color: var(--cronus-error); }
[data-slot="tags-input"] > [data-slot="badge"] {
  gap: 0.25rem; padding-right: 0.25rem; line-height: 1rem;
}
[data-slot="tags-input"] > [data-slot="badge"] > span {
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
[data-slot="tags-input-remove"] {
  display: flex; align-items: center; justify-content: center;
  width: 0.875rem; height: 0.875rem; padding: 0; border: 0; outline: none; background: transparent;
  border-radius: var(--cronus-radius-sm); color: var(--cronus-fg-tertiary); cursor: default;
}
[data-slot="tags-input-remove"] svg { width: 0.75rem; height: 0.75rem; }
[data-slot="tags-input-field"] {
  min-width: 6rem; flex: 1; border: 0; background: transparent; padding: 0;
  color: var(--cronus-fg); font: inherit; font-size: 0.875rem; line-height: 1.25rem; outline: none;
}
[data-slot="tags-input-field"]::placeholder { color: var(--cronus-fg-tertiary); }
[data-slot="autocomplete"] {
  position: relative; display: block; width: 100%;
}
div:has(> [data-slot="autocomplete"] + [data-slot="autocomplete-content"]) {
  position: relative; width: 100%;
}
[data-slot="autocomplete-input"] {
  display: flex; height: 2.5rem; width: 100%; box-sizing: border-box;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
  padding: 0 0.75rem; font-family: inherit; font-size: 0.875rem; line-height: 1.25rem; outline: none;
}
[data-slot="autocomplete-input"]::placeholder { color: var(--cronus-fg-tertiary); }
[data-slot="autocomplete-input"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="autocomplete-content"] {
  position: absolute; top: calc(100% + 4px); left: 0; z-index: 50;
  width: 100%; min-width: 8rem; box-sizing: border-box; padding: 0; outline: none;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
  box-shadow: var(--cronus-shadow-lg, none);
}
[data-slot="autocomplete-command"] {
  display: flex; width: 100%; height: 100%; flex-direction: column; overflow: hidden;
  border-radius: var(--cronus-radius-lg);
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
}
[data-slot="autocomplete-content"] [data-slot="command-list"] {
  max-height: 20rem; overflow-x: hidden; overflow-y: auto; padding: 0.25rem;
}
[data-slot="autocomplete-content"] [data-slot="command-item"] {
  position: relative; display: flex; align-items: center; gap: 0.5rem;
  border-radius: var(--cronus-radius-md); padding: 0.375rem 0.5rem;
  font-size: 0.875rem; line-height: 1.25rem; color: inherit;
  cursor: default; user-select: none; outline: none;
}
[data-slot="autocomplete-content"] [data-slot="command-item"][data-selected="true"] {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="autocomplete-content"] [data-slot="command-item"] > span {
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
[data-slot="multi-select"] {
  position: relative; display: block; width: 100%; line-height: 1.5;
}
[data-slot="multi-select-trigger"] {
  display: flex; align-items: center; justify-content: space-between; gap: 0.5rem;
  width: 100%; min-height: 2.5rem; padding: 0.375rem 0.75rem; box-sizing: border-box;
  border: 1px solid var(--cronus-border); border-radius: var(--cronus-radius-lg);
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
  font-size: 0.875rem; line-height: 1.25rem; font-weight: 400; font-family: inherit;
  cursor: default; outline: none;
  transition: border-color 150ms var(--cronus-ease), box-shadow 150ms var(--cronus-ease);
}
[data-slot="multi-select-trigger"][aria-expanded="true"] { border-color: var(--cronus-border-strong); }
[data-slot="multi-select-trigger"]:focus-visible {
  box-shadow: 0 0 0 2px var(--cronus-surface-base), 0 0 0 4px var(--cronus-ring, var(--cronus-primary));
}
[data-slot="multi-select-trigger"][aria-disabled="true"] { opacity: 0.5; pointer-events: none; }
[data-slot="multi-select-trigger"] > span:first-child {
  display: flex; flex: 1 1 0%; flex-wrap: wrap; align-items: center; gap: 0.375rem; overflow: hidden;
}
[data-slot="multi-select-trigger"][data-placeholder] > span:first-child > span { color: var(--cronus-fg-muted); }
[data-slot="multi-select-trigger"] > span:last-child {
  display: flex; flex-shrink: 0; align-items: center; gap: 0.25rem;
}
[data-slot="multi-select-trigger"] svg { width: 1rem; height: 1rem; flex-shrink: 0; opacity: 0.6; }
[data-slot="multi-select-trigger"][aria-invalid="true"] { border-color: var(--cronus-error); }
[data-slot="multi-select"] > [data-slot="popover-content"] {
  position: absolute; top: calc(100% + 4px); left: 0; z-index: 50;
  display: block; width: 100%; box-sizing: border-box; padding: 0; outline: none;
  border: 1px solid var(--cronus-border); border-radius: var(--cronus-radius-lg);
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
  font-size: 1rem; line-height: 1.5;
  box-shadow: var(--cronus-shadow-lg, none);
}
[data-slot="multi-select"] [data-slot="command"] {
  display: flex; flex-direction: column; overflow: hidden; width: 100%; height: 100%;
  box-sizing: border-box; border: 0; border-radius: var(--cronus-radius-lg);
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
}
[data-slot="multi-select"] [data-slot="command-list"] {
  max-height: 20rem; overflow-x: hidden; overflow-y: auto; padding: 0.25rem;
}
[data-slot="multi-select"] [data-slot="command-item"] {
  position: relative; display: flex; align-items: center; gap: 0.5rem;
  border-radius: var(--cronus-radius-md); padding: 0.375rem 0.5rem;
  font-size: 0.875rem; line-height: 1.25rem; color: inherit;
  cursor: default; user-select: none; outline: none;
}
[data-slot="multi-select"] [data-slot="command-list"]:not(:hover) [data-slot="command-item"][data-selected="true"],
[data-slot="multi-select"] [data-slot="command-item"]:hover {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="multi-select"] [data-slot="command-item"] > span:last-child {
  flex: 1 1 0%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
[data-slot="multi-select-indicator"] {
  display: flex; align-items: center; justify-content: center; flex-shrink: 0;
  width: 1rem; height: 1rem; box-sizing: border-box;
  border: 1px solid var(--cronus-border); border-radius: var(--cronus-radius-sm);
  background: var(--cronus-surface-inset);
}
[data-slot="multi-select-indicator"][data-state="checked"] {
  border-color: var(--cronus-primary); background: var(--cronus-primary);
  color: var(--cronus-primary-foreground);
}
[data-slot="multi-select-indicator"] svg { width: 0.75rem; height: 0.75rem; }
[data-slot="credit-card-input"] {
  display: flex; width: 100%; flex-direction: column; gap: 0.375rem;
}
[data-slot="credit-card-input"] > fieldset {
  margin: 0; display: flex; min-width: 0; flex-wrap: wrap; align-items: center;
  column-gap: 0.75rem; row-gap: 0.5rem; box-sizing: border-box;
  border-radius: var(--cronus-radius-xl); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
  padding: 0.625rem 0.875rem; font-size: 0.875rem; line-height: 1.25rem;
  box-shadow: var(--cronus-shadow-xs, none);
}
[data-slot="credit-card-input"] > fieldset[data-invalid="true"] { border-color: var(--cronus-error); }
[data-slot="credit-card-input"] > fieldset:disabled { opacity: 0.6; }
[data-slot="credit-card-input"] .sr-only {
  position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;
  overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0;
}
[data-slot="credit-card-input"] input {
  min-width: 0; background: transparent; border: 0; outline: none; padding: 0;
  color: var(--cronus-fg); font: inherit; font-variant-numeric: tabular-nums;
}
[data-slot="credit-card-input"] input::placeholder { color: var(--cronus-fg-tertiary); }
[data-slot="credit-card-input"] input:disabled { cursor: not-allowed; }
[data-slot="credit-card-input"] input:nth-of-type(1) { min-width: 11ch; flex: 1; letter-spacing: 0.02em; }
[data-slot="credit-card-input"] input:nth-of-type(2) { width: 3.25rem; flex-shrink: 0; }
[data-slot="credit-card-input"] input:nth-of-type(3) { width: 3rem; flex-shrink: 0; }
[data-slot="credit-card-input"] fieldset > svg:not(:last-child) {
  height: 1.25rem; width: 2rem; flex-shrink: 0; color: var(--cronus-fg-secondary);
  font-family: var(--cronus-font-display, inherit);
}
[data-slot="credit-card-input"] fieldset > svg[data-brand-fallback] { padding: 0.125rem; }
[data-slot="credit-card-input"] fieldset > svg:last-child {
  width: 1rem; height: 1rem; flex-shrink: 0; color: var(--cronus-success);
  opacity: 0; transform: scale(0.75);
}
[data-slot="floating-label-input"] {
  position: relative; display: block; width: 100%;
}
[data-slot="floating-label-input"] > div { position: relative; }
[data-slot="floating-label-input"] [data-slot="input"] {
  height: 3.5rem; padding-top: 1rem; line-height: 1.25rem;
}
[data-slot="floating-label-input-label"] {
  position: absolute; left: 0.75rem; top: 50%; z-index: 10;
  transform-origin: left; transform: translateY(-1.6rem) scale(0.8);
  pointer-events: none; user-select: none;
  color: var(--cronus-fg-secondary); font-size: 0.875rem; line-height: 1.25rem;
}
[data-slot="floating-label-input"]:has([data-slot="input"]:placeholder-shown) [data-slot="floating-label-input-label"] {
  transform: translateY(-50%) scale(1);
  color: var(--cronus-fg-tertiary);
}
[data-slot="floating-label-input"]:has([data-slot="input"]:focus) [data-slot="floating-label-input-label"],
[data-slot="floating-label-input"]:has([data-slot="input"]:not(:placeholder-shown)) [data-slot="floating-label-input-label"] {
  transform: translateY(-1.6rem) scale(0.8);
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
  isolation: isolate; display: inline-flex; align-items: stretch;
}
[data-slot="split-button"] > [data-slot="button"] { position: relative; line-height: 1.25rem; }
[data-slot="split-button"][data-variant="primary"] > [data-slot="button"] { border-width: 0; }
[data-slot="split-button"] > [data-slot="button"]:first-of-type {
  border-top-right-radius: 0; border-bottom-right-radius: 0; border-right-width: 0;
}
[data-slot="split-button"] > [data-slot="button"]:last-of-type {
  border-top-left-radius: 0; border-bottom-left-radius: 0; border-left-width: 0;
  aspect-ratio: 1; padding: 0; min-width: 2.5rem;
}
[data-slot="split-button"] > [data-slot="button"]:last-of-type::before {
  content: ""; pointer-events: none; position: absolute; top: 0.5rem; bottom: 0.5rem; left: 0;
  width: 1px; background: color-mix(in oklab, currentColor 15%, transparent);
}
[data-slot="split-button"] > [data-slot="button"]:focus-visible {
  position: relative; z-index: 10;
}
[data-slot="split-button"][data-disabled=""] { opacity: 0.5; pointer-events: none; }
[data-slot="pill-nav"] {
  display: inline-flex; align-items: center; gap: 0.25rem;
  border-radius: 9999px;
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset);
  padding: 0.25rem;
}
[data-slot="pill-nav-item"] {
  position: relative; z-index: 0;
  border: 0; border-radius: 9999px;
  padding: 0.375rem 0.875rem;
  background: transparent; color: var(--cronus-fg-secondary);
  font: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;
  text-decoration: none; cursor: pointer;
}
[data-slot="pill-nav-item"][aria-current="page"] {
  color: var(--cronus-fg);
}
[data-slot="pill-nav-item"][aria-current="page"]::before {
  content: ""; position: absolute; inset: 0; z-index: -1;
  border-radius: 9999px;
  background: var(--cronus-surface-floating);
  box-shadow: var(--cronus-shadow-xs, none);
}
[data-slot="dock"] {
  display: inline-flex; align-items: flex-end; gap: 0.5rem;
  border-radius: calc(var(--cronus-radius, 14px) + 8px);
  border: 1px solid var(--cronus-border);
  background: color-mix(in oklab, var(--cronus-surface-raised) 70%, transparent);
  -webkit-backdrop-filter: blur(8px); backdrop-filter: blur(8px);
  color: var(--cronus-fg); line-height: 1.5;
  padding: 0.5rem 0.75rem;
}
[data-slot="dock-item"] {
  display: inline-flex; align-items: center; justify-content: center;
  width: 2.75rem; height: 2.75rem; padding: 0;
  border: 0; border-radius: var(--cronus-radius-xl); outline: none;
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
  font: inherit; cursor: pointer; text-decoration: none;
}
[data-slot="dock-item"]:hover { background: var(--cronus-surface-base); }
[data-slot="dock-item"]:focus-visible {
  box-shadow: 0 0 0 2px var(--cronus-surface-base), 0 0 0 4px var(--cronus-ring, var(--cronus-primary));
}
[data-slot="dock-item"] > span { display: contents; }
[data-slot="dock-item"] svg { width: 50%; height: 50%; flex-shrink: 0; pointer-events: none; }
[data-slot="workspace-switcher"] {
  display: flex; width: 100%; min-width: 0; align-items: center; gap: 0.5rem;
  border: 0; border-radius: var(--cronus-radius-lg); padding: 0.375rem 0.5rem;
  background: transparent; color: var(--cronus-fg);
  font: inherit; font-size: 0.875rem; line-height: 1.25rem; cursor: pointer; outline: none;
}
[data-slot="workspace-switcher"]:hover { background: var(--cronus-surface-overlay); }
[data-slot="workspace-switcher"] [data-slot="avatar"] {
  width: 1.5rem; height: 1.5rem;
}
[data-slot="workspace-switcher"] [data-slot="avatar-fallback"] {
  font-size: 0.75rem; line-height: 1rem;
}
[data-slot="workspace-switcher"] > span:not([data-slot]) {
  min-width: 0; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  text-align: start; font-weight: 500;
}
[data-slot="workspace-switcher"] > svg {
  width: 1rem; height: 1rem; flex-shrink: 0; color: var(--cronus-fg-tertiary);
}
[data-audit-canvas] > [data-slot="sidebar-wrapper"][data-app-shell] {
  width: 20rem; height: 14rem; min-height: 0; overflow: hidden;
}
[data-slot="app-shell-content"] {
  position: relative; display: flex; min-height: 100svh; min-width: 0;
  flex: 1; flex-direction: column;
  background: var(--cronus-surface-base);
}
[data-slot="app-shell-header"] {
  position: sticky; top: 0; z-index: 30;
  display: flex; height: 3.5rem; flex-shrink: 0; align-items: center; gap: 0.5rem;
  box-sizing: border-box;
  border-bottom: 1px solid var(--cronus-border);
  background: color-mix(in oklab, var(--cronus-surface-base) 80%, transparent);
  -webkit-backdrop-filter: blur(8px); backdrop-filter: blur(8px);
  padding: 0 1rem;
}
[data-slot="app-shell-body"] {
  display: flex; min-height: 0; flex: 1; flex-direction: column;
}
[data-slot="app-shell-body"] > div { padding: 0.75rem; font-size: 0.875rem; line-height: 1.25rem; }
[data-slot="table-of-contents"] {
  position: relative; font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg);
}
[data-slot="table-of-contents-indicator"] {
  position: absolute; inset-inline-start: 0; top: 0; width: 0.125rem;
  border-radius: 9999px; background: var(--cronus-primary); opacity: 0;
}
[data-slot="table-of-contents-list"] {
  list-style: none; margin: 0; padding: 0;
  border-left: 1px solid var(--cronus-border);
}
[data-slot="table-of-contents-link"] {
  display: block; padding: 0.375rem 0.75rem;
  border-start-end-radius: var(--cronus-radius-md);
  border-end-end-radius: var(--cronus-radius-md);
  color: var(--cronus-fg-tertiary, var(--cronus-fg-secondary));
  text-decoration: none; line-height: 1.375;
}
[data-slot="table-of-contents-link"]:hover { color: var(--cronus-fg-secondary); }
[data-slot="table-of-contents-link"][aria-current="location"] {
  color: var(--cronus-fg); font-weight: 500;
}
[data-slot="form"] {
  display: block; line-height: 1.5;
}
[data-slot="form-item"] {
  display: flex; flex-direction: column; gap: 0.375rem;
}
[data-slot="form-item"] + [data-slot="form-item"] { margin-top: 0.75rem; }
[data-slot="form-item"] > [data-slot="label"] {
  display: block; line-height: 1; user-select: none;
}
[data-slot="form-item"] > [data-slot="input"] {
  line-height: 1.25rem;
}
[data-slot="form-item"] > [data-slot="input"]::placeholder {
  color: var(--cronus-fg-tertiary);
}
[data-slot="form-description"] {
  margin: 0; font-size: 0.75rem; color: var(--cronus-fg-secondary);
}
[data-slot="signature-pad"] {
  position: relative; height: 10rem; width: 100%; overflow: hidden;
  box-sizing: border-box; touch-action: none;
  border-radius: var(--cronus-radius-xl);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
  box-shadow: var(--cronus-shadow-xs, none);
}
[data-slot="signature-pad-canvas"] {
  position: absolute; inset: 0; width: 100%; height: 100%; display: block;
}
[data-slot="signature-pad-hint"] {
  pointer-events: none; user-select: none; position: absolute;
  left: 1.25rem; right: 1.25rem; bottom: 1.75rem;
}
[data-slot="signature-pad-hint"] > div { border-top: 1px dashed var(--cronus-border-strong); }
[data-slot="signature-pad-hint"] > span {
  display: block; margin-top: 0.375rem; font-size: 0.75rem; line-height: 1rem; color: var(--cronus-fg-muted);
}
[data-slot="signature-pad"] > div:last-child {
  position: absolute; right: 0.375rem; bottom: 0.375rem; display: flex; gap: 0.25rem;
}
[data-slot="signature-pad"] [data-slot="button"] {
  border-width: 0; font-size: 0.875rem; line-height: 1.25rem;
}
[data-slot="signature-pad"] [data-slot="button"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="signature-pad"] [data-slot="button"] svg { width: 0.875rem; height: 0.875rem; flex-shrink: 0; }
[data-slot="resizable-panel-group"] {
  display: flex; flex-direction: row; height: 100%; width: 100%; overflow: hidden;
}
[data-slot="resizable-panel-group"][data-panel-group-direction="vertical"] { flex-direction: column; }
[data-slot="resizable-panel-group"] > [data-panel] {
  flex: 50 1 0px; overflow: hidden;
}
[data-slot="resizable-handle"] {
  position: relative; display: flex; width: 1px;
  align-items: center; justify-content: center;
  background: var(--cronus-border);
}
[data-slot="resizable-handle"]::after {
  content: ""; position: absolute; top: 0; bottom: 0; left: 50%;
  width: 0.25rem; transform: translateX(-50%);
}
[data-slot="resizable-handle"][data-panel-group-direction="vertical"] { height: 1px; width: 100%; }
[data-slot="scheduler"] {
  display: block; width: 100%;
  border-radius: var(--cronus-radius-xl); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-base); color: var(--cronus-fg);
}
[data-slot="scheduler"] > div:first-child {
  display: flex; align-items: center; justify-content: space-between; gap: 0.5rem;
  padding: 0.75rem; border-bottom: 1px solid var(--cronus-border);
}
[data-slot="scheduler"] > div:first-child > div { display: flex; align-items: center; gap: 0.25rem; }
[data-slot="scheduler"] [data-slot="button"] { font-size: 0.875rem; line-height: 1.25rem; cursor: default; }
[data-slot="scheduler"] [data-slot="button"][data-size="sm"] { font-size: 0.75rem; line-height: 1rem; gap: 0.375rem; }
[data-slot="scheduler"] [data-slot="button"] svg { width: 0.875rem; height: 0.875rem; flex-shrink: 0; }
[data-slot="scheduler-title"] { font-size: 0.875rem; line-height: 1.25rem; font-weight: 600; color: var(--cronus-fg); }
[data-slot="scheduler"] > div:first-child > [data-slot="scheduler-title"] {
  font-weight: 600; letter-spacing: normal; font-variant-numeric: tabular-nums;
}
[data-slot="scheduler-grid"] {
  width: 100%; table-layout: fixed; border-collapse: collapse;
}
[data-slot="scheduler-weekdays"] th {
  padding: 0.375rem 0.5rem; border-bottom: 1px solid var(--cronus-border);
  font-size: 0.75rem; line-height: 1rem; font-weight: 400;
  color: var(--cronus-fg-tertiary); text-align: center;
}
[data-slot="scheduler-grid"] td {
  height: 6rem; min-width: 0; padding: 0.375rem; vertical-align: top;
  border-right: 1px solid var(--cronus-border); border-bottom: 1px solid var(--cronus-border);
  color: var(--cronus-fg);
}
[data-slot="scheduler-grid"] td:last-child { border-right: 0; }
[data-slot="scheduler-grid"] td[data-outside] { color: var(--cronus-fg-tertiary); }
[data-slot="scheduler-grid"] td > span:first-child {
  display: inline-flex; width: 1.5rem; height: 1.5rem; flex-shrink: 0;
  align-items: center; justify-content: center; border-radius: 9999px;
  font-size: 0.75rem; line-height: 1rem; font-weight: 500;
}
[data-slot="scheduler-grid"] td[aria-current="date"] > span:first-child {
  background: var(--cronus-primary); color: var(--cronus-primary-foreground);
}
[data-slot="scheduler-grid"] td > span:last-child {
  display: flex; flex-direction: column; gap: 0.125rem; margin-top: 0.25rem;
}
[data-slot="scheduler-event"] {
  display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  border: 0; border-radius: 0.25rem; padding: 0.125rem 0.375rem; text-align: left;
  font-size: 0.75rem; line-height: 1rem; font-weight: 500;
  background: color-mix(in oklab, var(--cronus-primary) 15%, transparent);
  color: var(--cronus-primary-text); cursor: default;
}
[data-slot="scheduler-event-overflow"] {
  padding: 0 0.375rem; font-size: 0.75rem; line-height: 1rem; font-weight: 500;
  color: var(--cronus-fg-tertiary);
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
  position: relative; display: inline-flex; align-items: center; justify-content: center;
  gap: 0.5rem; width: 2.25rem; height: 2.25rem; padding: 0; box-sizing: border-box;
  flex-shrink: 0; border: 0; border-radius: var(--cronus-radius-lg);
  background: transparent; color: var(--cronus-fg-secondary);
  font: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;
  white-space: nowrap; cursor: pointer; outline: none;
}
[data-slot="notification-trigger"]:hover {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="notification-trigger"] > svg {
  width: 1rem; height: 1rem; flex-shrink: 0; pointer-events: none;
}
[data-slot="notification-badge"] {
  position: absolute; top: -0.25rem; right: -0.25rem;
  display: flex; align-items: center; justify-content: center; gap: 0.25rem;
  height: 1rem; min-width: 1rem; box-sizing: border-box;
  padding: 0.125rem 0.25rem; border: 1px solid transparent; border-radius: 9999px;
  background: var(--cronus-primary); color: var(--cronus-primary-foreground);
  font-size: 0.625rem; line-height: 1; font-weight: 500; font-variant-numeric: tabular-nums;
  white-space: nowrap; pointer-events: none;
}
[data-slot="notification-center"]:not(:popover-open) { display: none; }
[data-slot="notification-center"]:popover-open {
  z-index: 50; display: flex; flex-direction: column;
  width: 20rem; padding: 0; box-sizing: border-box; overflow: hidden;
  border: 1px solid var(--cronus-border); border-radius: var(--cronus-radius-lg);
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
  box-shadow: var(--cronus-shadow-lg, none);
}
[data-slot="notification-center"] > div:first-child {
  display: flex; align-items: center; justify-content: space-between; gap: 0.5rem;
  padding: 0.625rem 0.75rem; border-bottom: 1px solid var(--cronus-border);
}
[data-slot="notification-center"] > div:first-child > p {
  margin: 0; font-family: var(--cronus-font-display, inherit);
  font-size: 0.875rem; line-height: 1.25rem; font-weight: 600; color: var(--cronus-fg);
}
[data-slot="notification-center"] > div:nth-child(2) {
  max-height: 20rem; overflow-y: auto; padding: 0.25rem;
}
[data-slot="notification-list"] > ul {
  display: flex; flex-direction: column; margin: 0; padding: 0; list-style: none;
}
[data-slot="notification-row"] {
  display: flex; width: 100%; box-sizing: border-box;
  align-items: flex-start; gap: 0.75rem;
  padding: 0.625rem 0.5rem; border: 0; border-radius: var(--cronus-radius-md);
  background: transparent; color: inherit;
  font: inherit; text-align: start; cursor: pointer; outline: none;
}
[data-slot="notification-row"][data-unread] {
  background: color-mix(in oklch, var(--cronus-surface-overlay) 40%, transparent);
}
[data-slot="notification-row"]:hover {
  background: var(--cronus-surface-overlay);
}
[data-slot="notification-row"] > span {
  display: flex; min-width: 0; flex: 1 1 0%; flex-direction: column; gap: 0.125rem;
}
[data-slot="notification-row"] > span > span {
  display: flex; min-width: 0; align-items: flex-start; gap: 0.5rem;
}
[data-slot="notification-row"] > span > span > span:first-child {
  min-width: 0; flex: 1 1 0%; overflow-wrap: break-word;
  font-size: 0.875rem; line-height: 1.25rem; font-weight: 500; color: var(--cronus-fg);
}
[data-slot="notification-unread-dot"] {
  margin-top: 0.375rem; width: 0.5rem; height: 0.5rem; flex-shrink: 0;
  border-radius: 9999px; background: var(--cronus-primary);
}
[data-slot="notification-empty"] {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  gap: 0.5rem; padding: 2.5rem 1rem; text-align: center;
  font-size: 0.875rem; color: var(--cronus-fg-secondary);
}
[data-slot="segmented-control"] {
  position: relative; isolation: isolate; vertical-align: middle;
  display: inline-flex; align-items: stretch; gap: 0.25rem;
  border-radius: var(--cronus-radius-lg);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-overlay); padding: 0.25rem;
  line-height: 1.5;
}
[data-slot="segmented-control-item"] {
  position: relative;
  display: inline-flex; align-items: center; justify-content: center; gap: 0.375rem;
  padding: 0.375rem 0.75rem; white-space: nowrap; user-select: none;
  border: 0; border-radius: var(--cronus-radius-md);
  background: transparent; color: var(--cronus-fg-secondary);
  font: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 500; cursor: pointer;
}
[data-slot="segmented-control-item"][data-state="active"] {
  color: var(--cronus-fg);
}
[data-slot="segmented-control-thumb"] {
  position: absolute; inset: 0; z-index: 0; border-radius: inherit;
  background: var(--cronus-surface-floating);
  box-shadow: var(--cronus-shadow-sm, none);
}
[data-slot="segmented-control-item"] > span {
  position: relative; z-index: 1; display: inline-flex; align-items: center; gap: 0.375rem;
}
[data-slot="segmented-control-item"]:disabled { opacity: 0.5; pointer-events: none; }
[data-slot="usage-meter"] {
  display: flex; flex-direction: column; gap: 0.5rem;
}
[data-slot="usage-meter"] > div:first-child {
  display: flex; align-items: baseline; justify-content: space-between; gap: 0.5rem;
  font-size: 0.875rem; line-height: 1.25rem;
}
[data-slot="usage-meter-label"] {
  font-weight: 500; color: var(--cronus-fg);
}
[data-slot="usage-meter-value"] {
  display: flex; align-items: baseline; gap: 0.5rem;
  font-variant-numeric: tabular-nums; color: var(--cronus-fg-secondary);
}
[data-slot="usage-meter-value"] > span + span { color: var(--cronus-fg-tertiary); }
[data-slot="usage-meter-track"] {
  position: relative; height: 0.5rem; width: 100%; overflow: hidden;
  border-radius: 9999px; background: var(--cronus-surface-overlay);
}
[data-slot="usage-meter-fill"] {
  height: 100%; border-radius: 9999px;
  width: calc(attr(data-value type(<number>), 0) * 1%);
  background: var(--cronus-primary);
}
[data-slot="usage-meter-fill"][data-tone="warning"] { background: var(--cronus-warning); }
[data-slot="usage-meter-fill"][data-tone="error"] { background: var(--cronus-error); }
[data-slot="masonry"] {
  box-sizing: border-box; width: 18rem; max-width: 100%;
  column-count: 2; column-gap: 1rem;
}
[data-slot="masonry"] > * { margin-bottom: 1rem; break-inside: avoid; }
[data-slot="masonry"] > div {
  box-sizing: border-box; border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-lg); background: var(--cronus-surface-raised);
  padding: 0.75rem; font-size: 0.875rem; line-height: 1.25rem;
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
[data-slot="heatmap"] > [data-slot="heatmap-legend"] { line-height: 1rem; }
[data-slot="comparison-slider"] {
  position: relative; aspect-ratio: 16 / 9; width: 100%;
  overflow: hidden; user-select: none; touch-action: none;
  border-radius: var(--cronus-radius-xl);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset);
}
[data-slot="comparison-after"], [data-slot="comparison-before"] {
  position: absolute; inset: 0; width: 100%; height: 100%;
}
[data-slot="comparison-after"] > div, [data-slot="comparison-before"] > div {
  display: flex; width: 100%; height: 100%; align-items: center; justify-content: center;
  font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg);
}
[data-slot="comparison-after"] > div { background: var(--cronus-surface-raised); }
[data-slot="comparison-before"] > div { background: var(--cronus-surface-overlay); }
[data-slot="comparison-before"] { clip-path: inset(0 50% 0 0); }
[data-slot="comparison-before"]:dir(rtl) { clip-path: inset(0 0 0 50%); }
[data-slot="comparison-slider"] > [aria-hidden] {
  position: absolute; top: 0; bottom: 0; left: 50%; z-index: 10;
  width: 2px; transform: translateX(-50%); pointer-events: none;
  background: color-mix(in oklab, var(--cronus-surface-base) 90%, transparent);
  box-shadow: var(--cronus-shadow-sm, none);
}
[data-slot="comparison-slider"] > [role="slider"] {
  position: absolute; top: 50%; left: 50%; z-index: 20;
  display: flex; width: 2.25rem; height: 2.25rem; align-items: center; justify-content: center;
  transform: translate(-50%, -50%); pointer-events: none; box-sizing: border-box;
  border-radius: 9999px; border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-raised); color: var(--cronus-fg);
  box-shadow: var(--cronus-shadow-sm, none);
}
[data-slot="comparison-slider"] > [role="slider"] svg {
  width: 1rem; height: 1rem; color: var(--cronus-fg-secondary);
}
[data-slot="code-tabs"] {
  display: flex; flex-direction: column; gap: 0;
  overflow: hidden; border-radius: var(--cronus-radius-xl);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-raised); color: var(--cronus-fg);
}
[data-slot="code-tabs-header"] {
  display: flex; align-items: center; justify-content: space-between; gap: 0.75rem;
  padding-right: 0.75rem; border-bottom: 1px solid var(--cronus-border);
  background: var(--cronus-surface-overlay);
}
[data-slot="code-tabs-list"] {
  position: relative; display: inline-flex; width: fit-content;
  align-items: center; justify-content: center; gap: 0;
  color: var(--cronus-fg-secondary); anchor-scope: --code-tabs-active;
}
[data-slot="code-tabs-trigger"] {
  display: inline-flex; align-items: center; justify-content: center; gap: 0.375rem;
  white-space: nowrap; padding: 0.625rem 0.875rem; border: 0; border-radius: 0;
  background: transparent; color: var(--cronus-fg-tertiary);
  font-size: 0.875rem; line-height: 1.25rem; font-weight: 500; cursor: default;
}
[data-slot="code-tabs-trigger"][data-state="active"] { color: var(--cronus-fg); anchor-name: --code-tabs-active; }
[data-slot="code-tabs-indicator"] {
  position: absolute; bottom: 0; height: 0.125rem; pointer-events: none;
  border-radius: 9999px; background: var(--cronus-primary);
  position-anchor: --code-tabs-active;
  left: anchor(--code-tabs-active left); width: anchor-size(--code-tabs-active width);
}
[data-slot="code-tabs-language"] {
  flex-shrink: 0; font-family: var(--cronus-font-mono, ui-monospace, monospace);
  font-size: 0.75rem; line-height: 1rem; color: var(--cronus-fg-tertiary);
}
[data-slot="code-tabs-panels"] { position: relative; }
[data-slot="code-tabs-panels"] > [data-slot="copy-button"] {
  position: absolute; top: 0.5rem; right: 0.5rem; z-index: 10;
  width: 2rem; height: 2rem; padding: 0; border: 0; border-radius: var(--cronus-radius-lg);
  background: color-mix(in oklab, var(--cronus-surface-overlay) 80%, transparent);
  backdrop-filter: blur(8px); color: var(--cronus-fg-secondary);
  font-size: 0.875rem; line-height: 1.25rem;
  opacity: 1; cursor: default;
}
[data-slot="code-tabs-panels"] > [data-slot="copy-button"] svg { width: 0.875rem; height: 0.875rem; }
[data-slot="code-tabs-panel"] {
  flex: 1; overflow-x: auto; outline: none;
}
[data-slot="code-tabs-pre"] {
  margin: 0; padding: 1rem 3.5rem 1rem 1rem;
  font-family: var(--cronus-font-mono, ui-monospace, monospace);
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
  font: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 500; cursor: pointer;
}
[data-slot="expandable-tabs-item"][aria-selected="true"] {
  color: var(--cronus-fg);
  background: var(--cronus-surface-floating);
  box-shadow: var(--cronus-shadow-xs, none);
}
[data-slot="expandable-tabs-item"] > span[aria-hidden="true"] {
  display: grid; width: 1rem; height: 1rem; place-items: center;
}
[data-slot="expandable-tabs-item"] > span[aria-hidden="true"] > svg { width: 1rem; height: 1rem; }
[data-slot="expandable-tabs-item"] > span:last-child { overflow: hidden; white-space: nowrap; }
[data-slot="expandable-tabs-item"][aria-selected="false"] > span:last-child {
  position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;
  overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border-width: 0;
}
[data-slot="live-line-chart"] {
  display: block; width: 100%; height: 16rem;
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
  display: block; width: 100%; height: 16rem;
}
[data-slot="gauge-chart"] {
  display: block; width: 100%; height: 16rem;
}
[data-slot="gauge-chart"] text {
  fill: var(--cronus-fg); font-size: 1.5rem; font-weight: 500;
}
[data-slot="funnel-chart"] {
  display: block; width: 100%; height: 16rem;
}
[data-slot="funnel-chart"] text {
  fill: var(--cronus-fg);
}
[data-slot="candlestick-chart"] {
  display: block; width: 100%; height: 16rem;
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
[data-slot="rich-text-editor-toolbar"] [data-slot="toggle"] {
  display: inline-flex; align-items: center; justify-content: center; gap: 0.5rem;
  width: 2rem; height: 2rem; padding: 0; border: 0;
  border-radius: var(--cronus-radius-lg);
  background: transparent; color: var(--cronus-fg-secondary);
  font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;
}
[data-slot="rich-text-editor-toolbar"] [data-slot="toggle"]:disabled { cursor: default; }
[data-slot="rich-text-editor-toolbar"] [data-slot="toggle"][data-disabled] { opacity: 0.5; }
[data-slot="rich-text-editor-toolbar"] [data-slot="toggle"] svg { width: 1rem; height: 1rem; flex-shrink: 0; }
[data-slot="rich-text-editor-toolbar"] [data-slot="separator"][data-orientation="vertical"] {
  flex-shrink: 0; height: 1.5rem; width: 1px; margin: 0 0.25rem;
  background: var(--cronus-border);
}
[data-slot="rich-text-editor-content-wrapper"] { position: relative; }
[data-slot="rich-text-editor-placeholder"] {
  pointer-events: none; position: absolute; inset-inline-start: 1rem; top: 0.75rem;
  margin: 0; user-select: none; color: var(--cronus-fg-tertiary, var(--cronus-fg-secondary));
}
[data-slot="rich-text-editor-content-wrapper"] [role="textbox"] {
  box-sizing: border-box; min-height: 10rem; width: 100%;
  padding: 0.75rem 1rem; color: var(--cronus-fg); outline: none;
}
[data-slot="rich-text-editor-content-wrapper"] [role="textbox"] p { margin: 0.5rem 0; }
[data-slot="rich-text-editor-content-wrapper"] [role="textbox"] p:first-child { margin-top: 0; }
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
  margin: 0;
  position: relative;
  line-height: 1.5;
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
  position: relative; isolation: isolate;
  display: inline-flex; align-items: center; justify-content: center;
}
[data-slot="morphing-popover-trigger"] {
  display: inline-flex; align-items: center; justify-content: center; gap: 0.5rem;
  height: 2.25rem; padding: 0 0.75rem; box-sizing: border-box;
  border-radius: 10px;
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-floating);
  color: var(--cronus-fg);
  box-shadow: var(--cronus-shadow-sm, none);
  font: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;
  cursor: pointer; outline: none;
}
[data-slot="morphing-popover-trigger"]:hover { background: var(--cronus-surface-overlay); }
[data-slot="morphing-popover-trigger"][aria-hidden="true"] { pointer-events: none; }
[data-slot="morphing-popover-trigger"] > span { display: inline-flex; }
[data-slot="morphing-popover-content"] {
  position: absolute; z-index: 50;
  display: flex; flex-direction: column;
  width: 18rem; box-sizing: border-box; overflow: hidden; outline: none;
  border: 1px solid var(--cronus-border); border-radius: 16px;
  background: var(--cronus-surface-floating); color: var(--cronus-fg);
  box-shadow: var(--cronus-shadow-lg, none);
}
[data-slot="morphing-popover-content"] > div {
  display: flex; width: 100%; flex: 1 1 0%; flex-direction: column;
}
[data-slot="bouncy-accordion"] {
  display: flex; flex-direction: column; align-items: center;
  width: 100%; max-width: 300px;
}
[data-slot="bouncy-accordion"] > p {
  position: relative; margin: 0 0 5rem; max-width: 18ch; text-align: center;
  font-size: 0.75rem; line-height: 1.25; text-transform: uppercase; color: var(--cronus-fg-tertiary);
}
[data-slot="bouncy-accordion"] > p::after {
  content: ""; position: absolute; left: 0; right: 0; top: 100%; margin: 0 auto;
  width: 1px; height: 4rem; background: linear-gradient(to bottom, transparent, var(--cronus-fg));
}
[data-slot="bouncy-accordion"] > ul { width: 100%; margin: 0; padding: 0; list-style: none; }
[data-slot="bouncy-accordion"] > ul > li {
  position: relative; overflow: hidden; height: 45px;
  background: var(--cronus-surface-base);
}
[data-slot="bouncy-accordion"] > ul > li:hover { background: var(--cronus-surface-raised); }
[data-slot="bouncy-accordion"] > ul > li[data-state="open"] {
  height: auto; margin-block: 10px; border-radius: 20px;
}
[data-slot="bouncy-accordion"] > ul > li:first-child,
[data-slot="bouncy-accordion"] > ul > li[data-state="open"] + li {
  border-top-left-radius: 20px; border-top-right-radius: 20px;
}
[data-slot="bouncy-accordion"] > ul > li:last-child,
[data-slot="bouncy-accordion"] > ul > li:has(+ li[data-state="open"]) {
  border-bottom-left-radius: 20px; border-bottom-right-radius: 20px;
}
[data-slot="bouncy-accordion-trigger"] {
  display: flex; flex-direction: column; width: 100%; padding: 0 0.5rem;
  text-align: start; cursor: pointer; outline: none;
}
[data-slot="bouncy-accordion-trigger"]:focus-visible {
  box-shadow: 0 0 0 2px var(--cronus-surface-base), 0 0 0 4px var(--cronus-ring);
}
[data-slot="bouncy-accordion-trigger"] > span:first-child {
  display: flex; height: 45px; align-items: center; gap: 0.5rem; padding-inline-start: 0.75rem;
}
[data-slot="bouncy-accordion-trigger"] > span:first-child > span {
  font-size: 0.875rem; line-height: 1.25rem; letter-spacing: -0.025em;
  color: color-mix(in oklab, var(--cronus-fg) 75%, transparent);
}
[data-slot="bouncy-accordion-trigger"] > span:last-child {
  padding: 0.5rem 0.75rem; font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg-tertiary);
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
  [data-slot="typing-text"]::after { display: none; animation: none; }
}
[data-slot="word-rotate"] {
  position: relative; display: inline-grid;
  height: 1.2em; overflow: hidden;
  vertical-align: baseline; white-space: nowrap;
}
[data-slot="timeline"] {
  display: flex; flex-direction: column; width: 100%; min-width: 0;
  list-style: none; margin: 0; padding: 0; line-height: 1.5;
}
[data-slot="timeline-item"] {
  position: relative; display: grid; grid-template-columns: auto minmax(0, 1fr);
  column-gap: 0.75rem; padding-bottom: 1.5rem;
}
[data-slot="timeline-item"]:last-child { padding-bottom: 0; }
[data-slot="timeline-rail"] {
  position: relative; display: flex; flex-direction: column; align-items: center;
}
[data-slot="timeline-dot"] {
  position: relative; z-index: 10; display: flex; flex-shrink: 0;
  align-items: center; justify-content: center;
  width: 0.625rem; height: 0.625rem;
  border-radius: calc(infinity * 1px); background: var(--cronus-fg-tertiary);
}
[data-slot="timeline-connector"] {
  flex: 1 1 0%; width: 1px; margin-top: 0.25rem;
  border-radius: calc(infinity * 1px); background: var(--cronus-border);
}
[data-slot="timeline-body"] { min-width: 0; padding-top: 0.125rem; }
[data-slot="timeline-content"] {
  display: flex; flex-direction: column; gap: 0.25rem; min-width: 0;
}
[data-slot="timeline-title"] {
  font-size: 0.875rem; font-weight: 500; line-height: 1; color: var(--cronus-fg);
}
[data-slot="tree-view"] {
  display: flex; flex-direction: column; width: 100%; min-width: 0;
  font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg); user-select: none;
}
[data-slot="tree-view-item"] { min-width: 0; }
[data-slot="tree-view-item-trigger"] {
  display: flex; align-items: center; gap: 0.375rem;
  height: 2rem; padding: 0 0.5rem;
  border-radius: var(--cronus-radius-md);
  color: var(--cronus-fg-secondary);
}
[data-slot="tree-view-item-trigger"][aria-level="2"] { padding-inline-start: 1.5rem; }
[data-slot="tree-view-item-trigger"][aria-level="3"] { padding-inline-start: 2.5rem; }
[data-slot="tree-view-item-trigger"]:hover {
  background: var(--cronus-surface-overlay); color: var(--cronus-fg);
}
[data-slot="tree-view-chevron"], [data-slot="tree-view-item-trigger"] > span[aria-hidden="true"] {
  width: 1rem; height: 1rem; flex-shrink: 0;
}
[data-slot="tree-view-chevron"] { color: var(--cronus-fg-tertiary); }
[data-slot="tree-view-item-trigger"][data-state="open"] > [data-slot="tree-view-chevron"] { transform: rotate(90deg); }
[data-slot="tree-view-item-trigger"] > span:last-child {
  flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
[data-slot="tree-view-group"] { display: flex; flex-direction: column; }
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
  box-sizing: border-box; width: 18rem; max-width: 100%; overflow: hidden;
  border: 1px solid var(--cronus-border); border-radius: var(--cronus-radius-xl);
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
  box-shadow: var(--cronus-shadow-sm, none);
}
[data-slot="terminal-chrome"] {
  display: flex; align-items: center; gap: 0.5rem;
  border-bottom: 1px solid var(--cronus-border);
  background: var(--cronus-surface-raised); padding: 0.625rem 1rem;
}
[data-slot="terminal-chrome"] > [aria-hidden="true"] {
  display: flex; flex-shrink: 0; align-items: center; gap: 0.375rem;
}
[data-slot="terminal-chrome"] > [aria-hidden="true"] > span {
  width: 0.625rem; height: 0.625rem; border-radius: 9999px;
  background: color-mix(in oklch, var(--cronus-error) 80%, transparent);
}
[data-slot="terminal-chrome"] > [aria-hidden="true"] > span:nth-child(2) { background: color-mix(in oklch, var(--cronus-warning) 80%, transparent); }
[data-slot="terminal-chrome"] > [aria-hidden="true"] > span:nth-child(3) { background: color-mix(in oklch, var(--cronus-success) 80%, transparent); }
[data-slot="terminal-title"] {
  min-width: 0; flex: 1; user-select: none;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap; text-align: center;
  font-size: 0.75rem; line-height: 1rem; font-weight: 500;
  color: var(--cronus-fg-tertiary, var(--cronus-fg-secondary));
}
[data-slot="terminal-chrome"] [data-slot="copy-button"] {
  width: 2rem; height: 2rem; padding: 0; margin-right: -0.375rem; flex-shrink: 0;
  border: 0; border-radius: var(--cronus-radius-lg);
  font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;
  background: transparent; color: var(--cronus-fg-secondary);
}
[data-slot="terminal-chrome"] [data-slot="copy-button"]:disabled { opacity: 1; pointer-events: none; }
[data-slot="terminal-chrome"] [data-slot="copy-button"] svg { width: 0.875rem; height: 0.875rem; flex-shrink: 0; }
[data-slot="terminal-body"] {
  position: relative; overflow: auto; outline: none;
  font-family: var(--cronus-font-mono, ui-monospace, monospace);
  font-size: 0.875rem; line-height: 1.625; font-variant-numeric: tabular-nums;
}
[data-slot="terminal-body"] > .sr-only {
  position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;
  overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0;
}
[data-slot="terminal-body"] > div { position: relative; padding: 1rem; }
[data-slot="terminal-sizer"] { visibility: hidden; }
[data-slot="terminal-screen"] { position: absolute; inset: 0; overflow: hidden; padding: 1rem; }
[data-slot="terminal-line"] { display: flex; gap: 0.5rem; }
[data-slot="terminal-prompt"] {
  user-select: none; color: var(--cronus-fg-tertiary, var(--cronus-fg-secondary));
}
[data-slot="terminal-line"] > span:last-child {
  min-width: 0; flex: 1; white-space: pre-wrap; overflow-wrap: break-word;
  color: var(--cronus-fg);
}
[data-slot="terminal-line"][data-line-type="output"] > span:last-child { color: var(--cronus-fg-secondary); }
[data-slot="video-player"] {
  position: relative; isolation: isolate; width: 100%; box-sizing: border-box;
  overflow: hidden; border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-xl);
  background: var(--cronus-surface-inset);
  aspect-ratio: 16 / 9;
}
[data-slot="video-player-video"] {
  position: absolute; inset: 0; width: 100%; height: 100%;
  object-fit: contain;
}
[data-slot="video-player-overlay-play"] {
  position: absolute; inset: 0; z-index: 10; margin: auto;
  display: flex; width: 3.5rem; height: 3.5rem; box-sizing: border-box;
  align-items: center; justify-content: center; padding: 0;
  border: 1px solid var(--cronus-border); border-radius: 9999px;
  background: color-mix(in oklch, var(--cronus-surface-base) 85%, transparent);
  color: var(--cronus-fg); box-shadow: var(--cronus-shadow-md, none);
  backdrop-filter: blur(8px);
}
[data-slot="video-player-overlay-play"] svg { width: 1.5rem; height: 1.5rem; }
[data-slot="video-player-controls"] {
  position: absolute; inset-inline: 0; bottom: 0; z-index: 20;
  display: flex; align-items: center; gap: 0.375rem;
  border-top: 1px solid var(--cronus-border);
  background: color-mix(in oklch, var(--cronus-surface-base) 90%, transparent);
  padding: 0.5rem 0.625rem; backdrop-filter: blur(8px);
}
[data-slot="video-player-play"], [data-slot="video-player-rate"],
[data-slot="video-player-mute"], [data-slot="video-player-fullscreen"] {
  display: inline-flex; width: 2rem; height: 2rem; flex-shrink: 0;
  align-items: center; justify-content: center; padding: 0; border: 0;
  border-radius: var(--cronus-radius-md);
  background: transparent; color: var(--cronus-fg);
}
[data-slot="video-player-rate"] {
  width: auto; min-width: 2rem; padding: 0 0.375rem;
  font-size: 0.75rem; line-height: 1rem; font-weight: 600; font-variant-numeric: tabular-nums;
}
[data-slot="video-player"] button:disabled { cursor: default; }
[data-slot="video-player-controls"] button svg { width: 1rem; height: 1rem; flex-shrink: 0; }
[data-slot="video-player-time"] {
  flex-shrink: 0; padding: 0 0.25rem;
  font-size: 0.75rem; line-height: 1rem; font-variant-numeric: tabular-nums;
  color: var(--cronus-fg-secondary);
}
[data-slot="video-player-seek"], [data-slot="video-player-volume"] {
  appearance: none; height: 0.25rem; margin: 0; padding: 0; border: 0;
  border-radius: 9999px; background: var(--cronus-border);
  accent-color: var(--cronus-primary); outline: none;
}
[data-slot="video-player-seek"] { min-width: 0; flex: 1; }
[data-slot="video-player-volume"] { width: 4rem; flex-shrink: 0; }
[data-slot="video-player-seek"]::-webkit-slider-thumb, [data-slot="video-player-volume"]::-webkit-slider-thumb {
  appearance: none; width: 0.75rem; height: 0.75rem; border-radius: 9999px;
  background: var(--cronus-primary);
}
[data-slot="video-player-seek"]::-moz-range-thumb, [data-slot="video-player-volume"]::-moz-range-thumb {
  width: 0.75rem; height: 0.75rem; border: 0; border-radius: 9999px;
  background: var(--cronus-primary);
}
@media (max-width: 639.98px) {
  [data-slot="video-player-volume"] { display: none; }
}
[data-slot="text-effect"] {
  display: block;
  margin: 0;
  line-height: 1.5;
  animation: cui-text-effect 400ms ease-out both;
}
[data-slot="text-effect"] > .sr-only {
  position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;
  overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0;
}
@keyframes cui-text-effect {
  from { opacity: 0; filter: blur(8px); }
  to { opacity: 1; filter: none; }
}
@media (prefers-reduced-motion: reduce) {
  [data-slot="text-effect"] { animation: none; }
}
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
[data-slot="carousel"] { position: relative; outline: none; width: 18rem; max-width: 100%; }
[data-slot="carousel-content"] {
  margin-left: -1rem; display: flex; overflow-x: auto;
  scroll-snap-type: x mandatory; outline: none;
  scrollbar-width: none;
}
[data-slot="carousel-content"]::-webkit-scrollbar { display: none; }
[data-slot="carousel-item"] {
  min-width: 0; flex: 0 0 100%; padding-left: 1rem;
  scroll-snap-align: start; box-sizing: border-box;
}
[data-slot="carousel-item"] > div {
  box-sizing: border-box; border: 1px solid var(--cronus-border);
  border-radius: var(--cronus-radius-lg); background: var(--cronus-surface-raised);
  padding: 1.5rem; font-size: 0.875rem; line-height: 1.25rem;
}
[data-slot="carousel"] > div:last-child {
  margin-top: 0.75rem; display: flex; align-items: center; justify-content: center; gap: 0.5rem;
}
[data-slot="carousel-previous"], [data-slot="carousel-next"] {
  display: inline-flex; align-items: center; justify-content: center; gap: 0.5rem;
  width: 2.25rem; height: 2.25rem; padding: 0; box-sizing: border-box; white-space: nowrap;
  border-radius: 9999px; border: 1px solid var(--cronus-border);
  background: transparent; color: var(--cronus-fg);
  font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;
  box-shadow: var(--cronus-shadow-xs, none);
}
[data-slot="carousel-previous"]:disabled, [data-slot="carousel-next"]:disabled { cursor: default; }
[data-slot="carousel-previous"][data-disabled], [data-slot="carousel-next"][data-disabled] { opacity: 0.5; }
[data-slot="carousel-previous"] svg, [data-slot="carousel-next"] svg { width: 1rem; height: 1rem; flex-shrink: 0; }
[data-slot="code-block"] {
  box-sizing: border-box; width: 18rem; max-width: 100%;
  overflow: hidden;
  border-radius: var(--cronus-radius-xl);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-raised);
  color: var(--cronus-fg);
  line-height: 1.5;
}
[data-slot="code-block-header"] {
  display: flex; align-items: center; justify-content: space-between; gap: 0.75rem;
  padding: 0.5rem 1rem; border: 0 solid var(--cronus-border); border-bottom-width: 1px;
  background: var(--cronus-surface-overlay);
}
[data-slot="code-block-header"] > div { display: flex; align-items: center; gap: 0.5rem; min-width: 0; min-height: 2rem; }
[data-slot="code-block-filename"] {
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  font-family: var(--cronus-font-mono, ui-monospace, monospace);
  font-size: 0.75rem; line-height: 1rem; color: var(--cronus-fg-secondary);
}
[data-slot="code-block-language"] {
  display: inline-flex; align-items: center; gap: 0.25rem;
  padding: 0.125rem 0.5rem; border-radius: var(--cronus-radius-md);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-overlay); color: var(--cronus-fg-secondary);
  font-size: 0.75rem; line-height: 1rem; font-weight: 500; white-space: nowrap;
}
[data-slot="code-block-scroll"] { display: block; overflow: auto; outline: none; }
[data-slot="code-block-pre"] {
  margin: 0; padding: 1rem; font-size: 0.875rem; line-height: 1.625;
  font-family: var(--cronus-font-mono, ui-monospace, monospace);
}
[data-slot="code-block-code"] { display: block; white-space: pre; font-family: inherit; }
[data-slot="description-list"] {
  display: block; width: 18rem; max-width: 100%; min-width: 0; margin: 0;
  font-size: 0.875rem; line-height: 1.25rem;
}
[data-slot="description-item"] { min-width: 0; }
[data-slot="description-item"] + [data-slot="description-item"] { margin-top: 1rem; }
[data-slot="description-term"] { font-weight: 500; color: var(--cronus-fg-secondary); }
[data-slot="description-details"] {
  min-width: 0; margin: 0.25rem 0 0;
  color: var(--cronus-fg); overflow-wrap: break-word;
}
[data-slot="kanban"] {
  display: flex; width: 100%; gap: 1rem;
  overflow-x: auto; padding-bottom: 0.5rem;
}
[data-slot="kanban-column"] {
  display: flex; flex-direction: column; width: 18rem; flex-shrink: 0;
  border-radius: var(--cronus-radius-xl); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset);
}
[data-slot="kanban-column-header"] {
  display: flex; align-items: center; justify-content: space-between; gap: 0.5rem;
  padding: 0.625rem 0.75rem;
}
[data-slot="kanban-column-title"] {
  min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  font-family: var(--cronus-font-display, inherit);
  font-size: 0.875rem; line-height: 1.25rem; font-weight: 600; color: var(--cronus-fg);
}
[data-slot="kanban-column-count"] {
  display: inline-flex; align-items: center; gap: 0.25rem; white-space: nowrap;
  border-radius: var(--cronus-radius-md); border: 1px solid var(--cronus-border);
  padding: 0.125rem 0.5rem; font-size: 0.75rem; line-height: 1rem; font-weight: 500;
  background: var(--cronus-surface-overlay); color: var(--cronus-fg-secondary);
}
[data-slot="kanban-column-header"] > [data-slot="kanban-column-title"] {
  font-weight: 600; letter-spacing: normal; font-variant-numeric: tabular-nums;
}
[data-slot="kanban-column-header"] > [data-slot="kanban-column-count"] { font-variant-numeric: tabular-nums; }
[data-slot="kanban-column-list"] {
  display: flex; flex: 1; flex-direction: column; gap: 0.5rem; min-height: 6rem;
  overflow-y: auto; margin: 0; padding: 0.5rem;
  border-radius: 0 0 var(--cronus-radius-xl) var(--cronus-radius-xl);
}
[data-slot="kanban-card"] {
  display: flex; align-items: flex-start; gap: 0.5rem;
  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-raised); color: var(--cronus-fg);
  padding: 0.75rem;
  box-shadow: var(--cronus-shadow-sm, none);
}
[data-slot="kanban-drag-handle"] {
  display: block; flex-shrink: 0; margin-top: 0.125rem; padding: 0.125rem;
  border: 0; border-radius: var(--cronus-radius-md); background: transparent;
  color: var(--cronus-fg-muted); cursor: default;
}
[data-slot="kanban-drag-handle"] svg { width: 1rem; height: 1rem; }
[data-slot="kanban-card-body"] { flex: 1; min-width: 0; }
[data-slot="kanban-card-title"] { display: block; font-weight: 500; line-height: 1.375; color: var(--cronus-fg); }
[data-slot="kanban-card-description"] {
  display: block; margin-top: 0.25rem; font-size: 0.875rem; line-height: 1.375; color: var(--cronus-fg-secondary);
}
[data-slot="json-viewer"] {
  display: block; width: 18rem; max-width: 100%; overflow-x: auto; padding: 1rem;
  border: 1px solid var(--cronus-border); border-radius: var(--cronus-radius-xl);
  background: var(--cronus-surface-inset); color: var(--cronus-fg);
  font-family: var(--cronus-font-mono, ui-monospace, monospace); font-size: 0.875rem; line-height: 1.5rem;
}
[data-slot="json-viewer-row"], [data-slot="json-viewer-closer"] {
  display: flex; align-items: flex-start; gap: 0.25rem;
}
[data-slot="json-viewer-row"] > [data-slot="copy-button"] {
  width: 1.25rem; height: 1.25rem; flex-shrink: 0; margin-top: 0.125rem; padding: 0;
  border: 0; border-radius: var(--cronus-radius-md); background: transparent;
  color: var(--cronus-fg-secondary); font-size: 0.875rem; line-height: 1.25rem;
  opacity: 0; cursor: default;
}
[data-slot="json-viewer-row"] > [data-slot="copy-button"] svg { width: 0.75rem; height: 0.75rem; }
[data-slot="json-viewer-row"] > span:not([aria-hidden]),
[data-slot="json-viewer-closer"] > span:not([aria-hidden]) {
  flex: 1; min-width: 0; overflow-wrap: break-word;
}
[data-slot="json-viewer-row"] > span[aria-hidden="true"],
[data-slot="json-viewer-closer"] > span[aria-hidden="true"] {
  width: 1.25rem; height: 1.25rem; flex-shrink: 0;
}
[data-slot="json-viewer-toggle"] {
  display: inline-flex; width: 1.25rem; height: 1.25rem; flex-shrink: 0; margin-top: 0.125rem;
  align-items: center; justify-content: center; padding: 0; border: 0;
  border-radius: var(--cronus-radius-md); background: transparent;
  color: var(--cronus-fg-tertiary); cursor: default;
}
[data-slot="json-viewer-toggle"] svg { width: 0.875rem; height: 0.875rem; }
[data-slot="json-viewer-toggle"][data-state="open"] svg { transform: rotate(90deg); }
[data-slot="json-viewer-children"] { margin-left: 0.625rem; padding-left: 0.875rem; border-left: 1px solid var(--cronus-border); }
[data-slot="json-viewer"] [data-json="punct"] { color: var(--cronus-fg-tertiary); }
[data-slot="json-viewer-key"] { color: var(--cronus-fg-secondary); }
[data-slot="json-viewer-value"] { word-break: break-all; color: var(--cronus-fg); }
[data-slot="json-viewer-value"][data-type="string"] { color: var(--cronus-success-text); }
[data-slot="json-viewer-value"][data-type="number"] { color: var(--cronus-info-text); font-variant-numeric: tabular-nums; }
[data-slot="json-viewer-value"][data-type="boolean"] { color: var(--cronus-warning-text); }
[data-slot="json-viewer-value"][data-type="null"] { color: var(--cronus-fg-tertiary); }
[data-slot="animated-number"] {
  display: inline-block;
  font-variant-numeric: tabular-nums;
  font-family: var(--cronus-font-display, inherit);
}
[data-slot="marquee"] {
  position: relative; display: flex; overflow: hidden;
  mask-image: linear-gradient(to right, transparent, #000 12%, #000 88%, transparent);
  -webkit-mask-image: linear-gradient(to right, transparent, #000 12%, #000 88%, transparent);
}
[data-slot="marquee-group"] {
  display: flex; flex-shrink: 0; align-items: center; gap: 1rem;
  width: max-content;
  animation: cui-marquee 20s linear infinite;
}
[data-slot="marquee"]:hover [data-slot="marquee-group"],
[data-slot="marquee"]:focus-within [data-slot="marquee-group"] {
  animation-play-state: paused;
}
[data-slot="marquee-group"] > span {
  white-space: nowrap; color: var(--cronus-fg);
}
@keyframes cui-marquee {
  from { transform: translateX(0); }
  to { transform: translateX(-100%); }
}
@media (prefers-reduced-motion: reduce) {
  [data-slot="marquee-group"] { animation: none; }
}
[data-slot="gradient-text"] {
  color: transparent;
  background-image: linear-gradient(135deg, var(--cronus-primary), var(--cronus-accent));
  -webkit-background-clip: text;
  background-clip: text;
}
[data-slot="shiny-text"] {
  display: inline;
  background-image: linear-gradient(
    90deg,
    var(--cronus-fg-tertiary, var(--cronus-fg-secondary)) 0%,
    var(--cronus-fg-tertiary, var(--cronus-fg-secondary)) 40%,
    var(--cronus-fg) 50%,
    var(--cronus-fg-tertiary, var(--cronus-fg-secondary)) 60%,
    var(--cronus-fg-tertiary, var(--cronus-fg-secondary)) 100%
  );
  background-size: 200% 100%;
  animation: cui-shiny-text 3s linear infinite;
}
@keyframes cui-shiny-text {
  0% { background-position: 100% 0; }
  100% { background-position: -100% 0; }
}
@media (prefers-reduced-motion: reduce) {
  [data-slot="shiny-text"] { animation: none; }
}
[data-slot="aspect-ratio"] {
  position: relative; width: 18rem;
  aspect-ratio: 16 / 9;
}
[data-slot="frame"] {
  width: 18rem; overflow: hidden;
  border-radius: var(--cronus-radius-xl);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-raised);
  box-shadow: var(--cronus-shadow-sm, none);
}
[data-slot="frame-chrome"] {
  display: flex; align-items: center; gap: 0.75rem;
  border-bottom: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset);
  padding: 0.5rem 0.75rem;
}
[data-slot="frame-chrome"] [aria-hidden] {
  display: flex; align-items: center; gap: 0.375rem;
}
[data-slot="frame-chrome"] [aria-hidden] span {
  width: 0.75rem; height: 0.75rem; border-radius: 999px;
}
[data-slot="frame-chrome"] [aria-hidden] span:nth-child(1) { background: var(--cronus-error); }
[data-slot="frame-chrome"] [aria-hidden] span:nth-child(2) { background: var(--cronus-warning); }
[data-slot="frame-chrome"] [aria-hidden] span:nth-child(3) { background: var(--cronus-success); }
[data-slot="frame-address-bar"] {
  margin: 0 auto; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  border-radius: var(--cronus-radius-md); background: var(--cronus-surface-raised);
  padding: 0.25rem 0.75rem;
  font-size: 0.75rem; line-height: 1rem; color: var(--cronus-fg-tertiary);
}
[data-slot="frame-content"] { color: var(--cronus-fg); }
[data-slot="flip-card"] {
  position: relative; isolation: isolate;
  width: 18rem;
  min-height: 16rem; border-radius: calc(var(--cronus-radius, 14px) + 8px);
  perspective: 1600px;
  line-height: 1.5;
  cursor: pointer; outline: none;
}
[data-slot="flip-card"]:focus-visible {
  box-shadow: 0 0 0 2px var(--cronus-surface-base), 0 0 0 4px var(--cronus-ring);
}
[data-slot="flip-card"] > div {
  position: absolute; inset: 0;
  transform-style: preserve-3d;
  transition: transform 600ms var(--ease-spring, var(--cronus-ease));
  will-change: transform;
}
[data-slot="flip-card"]:hover > div,
[data-slot="flip-card"]:focus-within > div {
  transform: rotateY(180deg);
}
[data-slot="flip-card-front"],
[data-slot="flip-card-back"] {
  position: absolute; inset: 0;
  display: flex; flex-direction: column;
  overflow: hidden; border-radius: calc(var(--cronus-radius, 14px) + 8px);
  border: 1px solid var(--cronus-border);
  color: var(--cronus-fg);
  box-shadow: var(--cronus-shadow-sm, none);
  backface-visibility: hidden;
}
[data-slot="flip-card-front"] {
  background: var(--cronus-surface-raised);
}
[data-slot="flip-card-back"] {
  background: var(--cronus-surface-elevated, var(--cronus-surface-overlay));
  transform: rotateY(180deg);
}
@media (prefers-reduced-motion: reduce) {
  [data-slot="flip-card"] > div { transition: none; }
  [data-slot="flip-card"]:hover > div,
  [data-slot="flip-card"]:focus-within > div { transform: none; }
  [data-slot="flip-card-front"],
  [data-slot="flip-card-back"] { backface-visibility: visible; }
  [data-slot="flip-card-back"] { visibility: hidden; transform: none; }
  [data-slot="flip-card"]:hover [data-slot="flip-card-front"],
  [data-slot="flip-card"]:focus-within [data-slot="flip-card-front"] { visibility: hidden; }
  [data-slot="flip-card"]:hover [data-slot="flip-card-back"],
  [data-slot="flip-card"]:focus-within [data-slot="flip-card-back"] { visibility: visible; }
}
[data-slot="countdown"] {
  display: inline-flex; align-items: flex-start; gap: 0.375rem;
}
[data-slot="countdown-unit"] {
  display: flex; flex-direction: column; align-items: center; gap: 0.125rem;
  min-width: 3.5rem; padding: 0.5rem 0.625rem;
  border-radius: var(--cronus-radius-lg);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-raised);
  box-shadow: var(--cronus-shadow-xs, none);
}
[data-slot="countdown-value"] {
  display: block; overflow: hidden;
  font-weight: 600; font-size: 1.5rem; line-height: 2rem;
  font-variant-numeric: tabular-nums;
  color: var(--cronus-fg);
}
[data-slot="countdown-label"] {
  font-weight: 500; font-size: 0.6875rem;
  letter-spacing: 0.08em; text-transform: uppercase;
  color: var(--cronus-fg-tertiary, var(--cronus-fg-secondary));
}
[data-slot="animated-button"] {
  display: inline-flex; align-items: center; justify-content: center; gap: 0.5rem;
  height: 2.5rem; padding: 0 1rem;
  border-radius: var(--cronus-radius-lg); border: 1px solid transparent;
  background: var(--cronus-primary); color: var(--cronus-primary-foreground);
  font: inherit; font-size: 0.875rem; font-weight: 500; line-height: 1;
  cursor: pointer;
  box-shadow: var(--cronus-shadow-xs, 0 1px 2px rgba(0,0,0,.2));
  transition: transform 150ms var(--ease-out-quart), opacity 150ms var(--ease-out-quart);
}
[data-slot="animated-button"]:hover { transform: translateY(-1px); }
[data-slot="animated-button"]:active { transform: scale(0.97); }
@media (prefers-reduced-motion: reduce) {
  [data-slot="animated-button"] { transition: none; }
  [data-slot="animated-button"]:hover,
  [data-slot="animated-button"]:active { transform: none; }
}
[data-slot="card-stack"] {
  position: relative; isolation: isolate;
  height: 14rem; width: 18rem; max-width: 24rem;
  line-height: 1.5;
}
[data-slot="card-stack"] > span {
  position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;
  overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border-width: 0;
}
[data-slot="card-stack-item"] {
  position: absolute; inset: 0;
  border-radius: calc(var(--cronus-radius, 14px) + 8px);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-raised);
  padding: 1.25rem;
  color: var(--cronus-fg);
  box-shadow: var(--cronus-shadow-sm, none);
  transform-origin: top center;
  outline: none;
}
[data-slot="card-stack-item"]:nth-of-type(1) { translate: 0px 0px; }
[data-slot="card-stack-item"]:nth-of-type(2) {
  translate: 10px 10px; transform: scale(0.96) rotate(-1.5deg);
}
[data-slot="card-stack-item"]:nth-of-type(3) {
  translate: 20px 20px; transform: scale(0.92) rotate(-3deg);
}
[data-slot="card-stack-item"]:nth-last-of-type(1) { z-index: 1; }
[data-slot="card-stack-item"]:nth-last-of-type(2) { z-index: 2; }
[data-slot="card-stack-item"]:nth-last-of-type(3) { z-index: 3; }
@media (prefers-reduced-motion: reduce) {
  [data-slot="card-stack-item"] { transform: none; }
}
[data-slot="logo-carousel"] {
  display: grid; grid-auto-flow: column; grid-auto-columns: minmax(0, 1fr);
  gap: 0.75rem; width: 18rem; margin: 0; padding: 0; list-style: none;
  line-height: 1.5;
}
[data-slot="logo-carousel-item"] {
  position: relative; box-sizing: border-box;
  display: flex; min-width: 0; align-items: center; justify-content: center;
  overflow: hidden; height: 5rem; padding: 0 0.75rem;
  border-radius: var(--cronus-radius-xl);
  border: 1px solid transparent;
  background: transparent;
  color: color-mix(in oklab, var(--cronus-fg-secondary) 70%, transparent);
}
[data-slot="logo-carousel-item"]:hover { color: var(--cronus-fg); }
[data-slot="logo-carousel-item"] > div {
  position: absolute; inset: 0;
  display: flex; align-items: center; justify-content: center; padding: 0 1rem;
}
[data-slot="logo-carousel-item"] > div > span {
  display: inline-flex; max-width: 100%; align-items: center; justify-content: center; gap: 0.5rem;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  font-family: var(--cronus-font-display, inherit);
  font-size: 2.25rem; font-weight: 600; line-height: 1; color: currentColor;
}
@media (max-width: 639.98px) {
  [data-slot="logo-carousel"] { grid-auto-flow: row; grid-template-columns: repeat(2, minmax(0, 1fr)); }
}
@media (min-width: 640px) {
  [data-slot="logo-carousel-item"] { height: 6rem; }
  [data-slot="logo-carousel-item"] > div > span { font-size: 3.75rem; }
}
[data-slot="dynamic-island"] {
  display: inline-flex; flex-direction: column; align-items: center; gap: 0.75rem;
}
[data-slot="dynamic-island-shell"] {
  overflow: hidden; border-radius: 9999px;
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-floating);
  color: var(--cronus-fg);
  padding: 0.5rem 1rem;
  box-shadow: var(--cronus-shadow-md, none);
}
[data-slot="dynamic-island-shell"] > div { display: flex; align-items: center; gap: 0.75rem; }
[data-slot="dynamic-island"] [role="tablist"] {
  display: flex; align-items: center; gap: 0.25rem;
}
[data-slot="dynamic-island-trigger"] {
  width: 0.5rem; height: 0.5rem; padding: 0; border: 0;
  border-radius: 9999px; background: var(--cronus-border); cursor: pointer;
}
[data-slot="dynamic-island-trigger"][aria-selected="true"] {
  background: var(--cronus-fg);
}
[data-slot="image-zoom"] {
  position: relative; display: block; width: 18rem; overflow: hidden;
  cursor: zoom-in; user-select: none; padding: 0; font: inherit; color: inherit;
  border-radius: var(--cronus-radius-xl);
  border: 1px solid var(--cronus-border);
  background: var(--cronus-surface-inset);
}
[data-slot="image-zoom-content"] {
  display: block; width: 100%; height: 100%; pointer-events: none;
  transition: transform 300ms var(--ease-out-quart, var(--cronus-ease));
}
[data-slot="image-zoom-content"] img {
  display: block; width: 100%; height: 100%; object-fit: cover; user-select: none;
}
[data-slot="image-zoom"]:hover [data-slot="image-zoom-content"],
[data-slot="image-zoom"]:focus-visible [data-slot="image-zoom-content"] {
  transform: scale(1.5);
}
[data-slot="image-zoom-indicator"] {
  position: absolute; inset-inline-end: 0.5rem; bottom: 0.5rem;
  display: inline-flex; width: 2rem; height: 2rem;
  align-items: center; justify-content: center;
  border-radius: 9999px; border: 1px solid var(--cronus-border);
  background: color-mix(in oklab, var(--cronus-surface-overlay) 90%, transparent);
  color: var(--cronus-fg-secondary); pointer-events: none;
  box-shadow: var(--cronus-shadow-sm, none);
}
[data-slot="image-zoom-indicator"] svg { width: 1rem; height: 1rem; }
@media (prefers-reduced-motion: reduce) {
  [data-slot="image-zoom"]:hover [data-slot="image-zoom-content"],
  [data-slot="image-zoom"]:focus-visible [data-slot="image-zoom-content"] {
    transform: none;
  }
}
[data-slot="aurora-background"] {
  position: relative; overflow: hidden;
  color: var(--cronus-fg);
}
[data-slot="aurora-background"] > [aria-hidden] {
  position: absolute; inset: 0; z-index: 0; pointer-events: none;
}
[data-slot="aurora-background"] > div:last-child {
  position: relative; z-index: 1;
}
[data-slot="aurora-blob"] {
  position: absolute; border-radius: 999px;
  background: linear-gradient(135deg, var(--cronus-primary), var(--cronus-accent));
  filter: blur(48px);
  opacity: 0.3;
  animation: cui-aurora 18s ease-in-out infinite alternate;
}
[data-slot="aurora-blob"]:nth-child(1) {
  top: -33%; inset-inline-start: 25%; width: 66%; height: 66%;
}
[data-slot="aurora-blob"]:nth-child(2) {
  bottom: -33%; inset-inline-end: 25%; width: 66%; height: 66%;
  animation-delay: -6s;
}
[data-slot="aurora-blob"]:nth-child(3) {
  top: 25%; inset-inline-start: 25%; width: 50%; height: 50%;
  opacity: 0.2; animation-delay: -12s;
}
@keyframes cui-aurora {
  0% { transform: translate(0, 0) scale(1); }
  50% { transform: translate(3%, -4%) scale(1.08); }
  100% { transform: translate(-3%, 4%) scale(1.04); }
}
@media (prefers-reduced-motion: reduce) {
  [data-slot="aurora-blob"] { animation: none; }
}
[data-slot="border-beam"] {
  position: relative;
  border-radius: var(--cronus-radius-xl);
  color: var(--cronus-fg);
}
[data-slot="border-beam-layer"] {
  position: absolute; inset: 0; pointer-events: none;
  border-radius: inherit;
}
[data-slot="border-beam-layer"]::after {
  content: "";
  position: absolute;
  width: 60px; height: 60px;
  background: linear-gradient(to left, var(--cronus-fg), var(--cronus-primary), transparent);
  offset-path: rect(0 auto auto 0 round 60px);
  animation: cui-border-beam 8s linear infinite;
}
@keyframes cui-border-beam {
  from { offset-distance: 0%; }
  to { offset-distance: 100%; }
}
@media (prefers-reduced-motion: reduce) {
  [data-slot="border-beam-layer"] { display: none; }
}
[data-slot="border-beam-content"] {
  position: relative;
}
[data-slot="confetti"] {
  position: relative;
  color: var(--cronus-fg);
}
[data-slot="confetti-piece"] {
  position: absolute; pointer-events: none;
  width: 0.4rem; height: 0.25rem; border-radius: 1px;
  background: var(--cronus-primary);
  animation: cui-confetti 1.4s ease-out infinite;
}
[data-slot="confetti-piece"]:nth-of-type(1) {
  left: 12%; top: 18%; background: var(--cronus-primary);
}
[data-slot="confetti-piece"]:nth-of-type(2) {
  left: 28%; top: 8%; background: var(--cronus-accent);
  animation-delay: 0.12s;
}
[data-slot="confetti-piece"]:nth-of-type(3) {
  left: 46%; top: 22%; background: var(--cronus-success, var(--cronus-primary));
  animation-delay: 0.24s;
}
[data-slot="confetti-piece"]:nth-of-type(4) {
  left: 62%; top: 10%; background: var(--cronus-warning, var(--cronus-accent));
  animation-delay: 0.36s;
}
[data-slot="confetti-piece"]:nth-of-type(5) {
  left: 74%; top: 28%; background: var(--cronus-error);
  animation-delay: 0.48s;
}
[data-slot="confetti-piece"]:nth-of-type(6) {
  left: 86%; top: 16%; background: var(--cronus-primary);
  animation-delay: 0.6s;
}
@keyframes cui-confetti {
  0% { transform: translate(0, 0) rotate(0deg); opacity: 1; }
  100% { transform: translate(1.25rem, 3.25rem) rotate(220deg); opacity: 0; }
}
@media (prefers-reduced-motion: reduce) {
  [data-slot="confetti-piece"] { display: none; animation: none; }
}
[data-slot="composed-chart"] {
  display: block; width: 100%; height: 16rem;
}
[data-slot="heatmap-chart"] {
  display: block; width: 100%;
}
[data-slot="heatmap-chart"] [data-slot="heatmap"] {
  display: inline-flex; flex-direction: column; gap: 0.5rem;
}
[data-slot="heatmap-chart"] [role="img"] {
  display: grid; grid-auto-flow: column; grid-template-rows: repeat(7, minmax(0, 1fr)); gap: 0.25rem;
}
[data-slot="heatmap-chart"] [data-slot="heatmap-day"],
[data-slot="heatmap-chart"] [data-slot="heatmap-legend-swatch"] {
  width: 0.75rem; height: 0.75rem; border-radius: 3px;
  background: var(--cronus-surface-inset);
}
[data-slot="heatmap-chart"] [data-slot="heatmap-day"][data-level="1"],
[data-slot="heatmap-chart"] [data-slot="heatmap-legend-swatch"][data-level="1"] {
  background: color-mix(in oklch, var(--cronus-primary) 25%, transparent);
}
[data-slot="heatmap-chart"] [data-slot="heatmap-day"][data-level="2"],
[data-slot="heatmap-chart"] [data-slot="heatmap-legend-swatch"][data-level="2"] {
  background: color-mix(in oklch, var(--cronus-primary) 45%, transparent);
}
[data-slot="heatmap-chart"] [data-slot="heatmap-day"][data-level="3"],
[data-slot="heatmap-chart"] [data-slot="heatmap-legend-swatch"][data-level="3"] {
  background: color-mix(in oklch, var(--cronus-primary) 70%, transparent);
}
[data-slot="heatmap-chart"] [data-slot="heatmap-day"][data-level="4"],
[data-slot="heatmap-chart"] [data-slot="heatmap-legend-swatch"][data-level="4"] {
  background: var(--cronus-primary);
}
[data-slot="heatmap-chart"] [data-slot="heatmap-legend"] {
  display: flex; align-items: center; gap: 0.25rem;
  font-size: 0.75rem; line-height: 1rem; color: var(--cronus-fg-tertiary);
}
[data-slot="chart"] {
  display: flex; justify-content: center;
  width: 100%; height: 16rem; aspect-ratio: 16 / 9;
  font-size: 0.75rem; line-height: 1rem;
}
[data-slot="chart"] svg {
  display: block; width: 100%; height: 100%;
}
[data-slot="chart"] line[stroke-dasharray] {
  stroke: color-mix(in oklch, var(--cronus-border) 50%, transparent);
}
[data-slot="click-spark"] {
  position: relative; overflow: hidden;
  color: var(--cronus-fg);
}
[data-slot="click-spark-content"] {
  position: relative;
}
[data-slot="glare-hover"] {
  position: relative; overflow: hidden;
  color: var(--cronus-fg);
}
[data-slot="glare-hover-layer"] {
  position: absolute; inset: 0; pointer-events: none;
  opacity: 0;
  background: linear-gradient(115deg, transparent 32%, color-mix(in oklch, var(--cronus-fg) 18%, transparent) 50%, transparent 68%);
  background-size: 220% 220%;
  background-position: 50% 50%;
  transition: opacity 300ms var(--cronus-ease);
}
[data-slot="glare-hover"]:hover [data-slot="glare-hover-layer"],
[data-slot="glare-hover"]:focus-within [data-slot="glare-hover-layer"] {
  opacity: 1;
}
[data-slot="glare-hover-content"] {
  position: relative;
}
@media (prefers-reduced-motion: reduce) {
  [data-slot="glare-hover-layer"] { display: none; }
}
[data-slot="magnetic"] {
  display: inline-block;
  color: var(--cronus-fg);
}
[data-slot="magnetic-target"] {
  display: inline-block;
  transition: transform 300ms var(--cronus-ease);
}
[data-slot="magnetic"]:hover [data-slot="magnetic-target"] {
  transform: translate(4px, -4px);
}
@media (prefers-reduced-motion: reduce) {
  [data-slot="magnetic"] [data-slot="magnetic-target"],
  [data-slot="magnetic"]:hover [data-slot="magnetic-target"] {
    transform: none;
    transition: none;
  }
}
[data-slot="dot-pattern"] {
  position: relative; overflow: hidden;
  color: var(--cronus-fg);
}
[data-slot="dot-pattern-field"] {
  position: absolute; inset: 0; pointer-events: none;
  background-image: repeating-radial-gradient(circle at 8px 8px, var(--cronus-fg) 0 1px, transparent 1.25px 16px);
  opacity: 0.18;
}
[data-slot="dot-pattern-content"] {
  position: relative; z-index: 1;
}
[data-slot="flickering-grid"] {
  position: relative; overflow: hidden;
  color: var(--cronus-fg);
}
[data-slot="flickering-grid-field"] {
  position: absolute; inset: 0; pointer-events: none;
  background-image:
    linear-gradient(to right, color-mix(in oklch, var(--cronus-fg) 18%, transparent) 1px, transparent 1px),
    linear-gradient(to bottom, color-mix(in oklch, var(--cronus-fg) 18%, transparent) 1px, transparent 1px);
  background-size: 12px 12px;
  animation: cui-flicker 1.8s ease-in-out infinite;
}
[data-slot="flickering-grid-content"] {
  position: relative; z-index: 1;
}
@keyframes cui-flicker {
  0%, 100% { opacity: 0.08; }
  50% { opacity: 0.45; }
}
@media (prefers-reduced-motion: reduce) {
  [data-slot="flickering-grid-field"] { animation: none; opacity: 0.4; }
}
[data-slot="grid-pattern"] {
  position: relative; overflow: hidden;
  color: var(--cronus-border);
}
[data-slot="grid-pattern-field"] {
  position: absolute; inset: 0; pointer-events: none;
  background-image:
    repeating-linear-gradient(to right, var(--cronus-border) 0 1px, transparent 1px 24px),
    repeating-linear-gradient(to bottom, var(--cronus-border) 0 1px, transparent 1px 24px);
}
[data-slot="grid-pattern-content"] {
  position: relative; z-index: 1;
}
[data-slot="highlighter"] {
  position: relative;
  display: inline;
  white-space: nowrap;
}
[data-slot="highlighter-mark"] {
  position: absolute;
  inset-inline: 0;
  bottom: 0.08em;
  z-index: -1;
  height: 0.45em;
  transform-origin: 0 100%;
  border-radius: 0.125rem;
  background: color-mix(in oklch, var(--cronus-primary) 25%, transparent);
  animation: cui-highlighter 0.6s cubic-bezier(.22, 1, .36, 1) both;
}
@keyframes cui-highlighter {
  from { transform: scaleX(0); }
  to { transform: scaleX(1); }
}
@media (prefers-reduced-motion: reduce) {
  [data-slot="highlighter-mark"] { animation: none; }
}
[data-slot="scramble-text"] {
  display: inline;
  font-family: var(--cronus-font-mono, ui-monospace, monospace);
}
[data-slot="spinning-text"] {
  position: relative;
  display: inline-grid;
  place-items: center;
  width: 6rem;
  height: 6rem;
}
[data-slot="spinning-text-label"] {
  position: absolute;
  width: 1px; height: 1px; padding: 0; margin: -1px;
  overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0;
}
[data-slot="spinning-text-orbit"] {
  display: inline-block;
  font-size: 0.75rem;
  font-weight: 500;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--cronus-fg-secondary);
  animation: cui-spinning-text 16s linear infinite;
}
@keyframes cui-spinning-text {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
@media (prefers-reduced-motion: reduce) {
  [data-slot="spinning-text-orbit"] { animation: none; }
}
[data-slot="gradient-border"] {
  display: inline-block;
  padding: 1px;
  border-radius: var(--cronus-radius-xl);
  background: linear-gradient(135deg, var(--cronus-primary), var(--cronus-accent));
  color: var(--cronus-fg);
}
[data-slot="gradient-border-inner"] {
  height: 100%;
  width: 100%;
  border-radius: inherit;
  background: var(--cronus-surface-raised);
  padding: 0.75rem 1.25rem;
}
[data-slot="light-rays"] {
  position: relative;
  overflow: hidden;
  color: var(--cronus-fg);
}
[data-slot="light-rays-field"] {
  position: absolute;
  inset: -50%;
  pointer-events: none;
  background: repeating-conic-gradient(
    from 0deg,
    color-mix(in oklch, var(--cronus-primary) 22%, transparent) 0deg 6deg,
    transparent 6deg 28deg
  );
  -webkit-mask-image: radial-gradient(ellipse at center, black 20%, transparent 70%);
  mask-image: radial-gradient(ellipse at center, black 20%, transparent 70%);
  animation: cui-light-rays 18s linear infinite;
}
@keyframes cui-light-rays {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
[data-slot="light-rays-content"] {
  position: relative;
  z-index: 1;
}
@media (prefers-reduced-motion: reduce) {
  [data-slot="light-rays-field"] { display: none; }
}
[data-slot="orbit"] {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18rem;
  height: 18rem;
  line-height: 1.5;
  color: var(--cronus-fg);
}
[data-slot="orbit-ring"] {
  box-sizing: border-box;
  position: absolute;
  left: 50%;
  top: 50%;
  width: 16rem;
  height: 16rem;
  transform: translate(-50%, -50%);
  border: 1px solid color-mix(in oklch, var(--cronus-border) 40%, transparent);
  border-radius: 9999px;
  pointer-events: none;
}
@keyframes cui-orbit-spin {
  from { transform: rotate(0turn); }
  to { transform: rotate(1turn); }
}
[data-slot="orbit-positioner"] {
  position: absolute;
  inset: 0;
  rotate: var(--orbit-angle, 0deg);
  animation: cui-orbit-spin 24s linear infinite;
  will-change: transform;
}
[data-slot="orbit-holder"] {
  position: absolute;
  left: 50%;
  top: 0;
  transform: translate(-50%, -50%);
}
[data-slot="orbit-item"] {
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: auto;
  rotate: calc(var(--orbit-angle, 0deg) * -1);
  animation: cui-orbit-spin 24s linear infinite reverse;
}
[data-slot="orbit"]:hover [data-slot="orbit-positioner"],
[data-slot="orbit"]:hover [data-slot="orbit-item"],
[data-slot="orbit"]:focus-within [data-slot="orbit-positioner"],
[data-slot="orbit"]:focus-within [data-slot="orbit-item"] {
  animation-play-state: paused;
}
[data-slot="orbit-positioner"]:nth-child(2):nth-last-child(1) { --orbit-angle: 180deg; }
[data-slot="orbit-positioner"]:nth-child(2):nth-last-child(2) { --orbit-angle: 120deg; }
[data-slot="orbit-positioner"]:nth-child(3):nth-last-child(1) { --orbit-angle: 240deg; }
[data-slot="orbit-positioner"]:nth-child(2):nth-last-child(3) { --orbit-angle: 90deg; }
[data-slot="orbit-positioner"]:nth-child(3):nth-last-child(2) { --orbit-angle: 180deg; }
[data-slot="orbit-positioner"]:nth-child(4):nth-last-child(1) { --orbit-angle: 270deg; }
[data-slot="orbit-positioner"]:nth-child(2):nth-last-child(4) { --orbit-angle: 72deg; }
[data-slot="orbit-positioner"]:nth-child(3):nth-last-child(3) { --orbit-angle: 144deg; }
[data-slot="orbit-positioner"]:nth-child(4):nth-last-child(2) { --orbit-angle: 216deg; }
[data-slot="orbit-positioner"]:nth-child(5):nth-last-child(1) { --orbit-angle: 288deg; }
[data-slot="orbit-positioner"]:nth-child(2):nth-last-child(5) { --orbit-angle: 60deg; }
[data-slot="orbit-positioner"]:nth-child(3):nth-last-child(4) { --orbit-angle: 120deg; }
[data-slot="orbit-positioner"]:nth-child(4):nth-last-child(3) { --orbit-angle: 180deg; }
[data-slot="orbit-positioner"]:nth-child(5):nth-last-child(2) { --orbit-angle: 240deg; }
[data-slot="orbit-positioner"]:nth-child(6):nth-last-child(1) { --orbit-angle: 300deg; }
@media (prefers-reduced-motion: reduce) {
  [data-slot="orbit-positioner"],
  [data-slot="orbit-item"] { animation: none; }
}
[data-progressive-blur-host] {
  position: relative;
  width: 18rem;
  min-height: 8rem;
  line-height: 1.5;
  color: var(--cronus-fg);
}
[data-slot="progressive-blur"] {
  position: absolute; inset-inline: 0; bottom: 0;
  height: 6rem; z-index: 10; pointer-events: none;
}
[data-slot="progressive-blur"] > div {
  position: absolute; inset: 0;
}
[data-slot="progressive-blur"] > div:nth-child(1) {
  backdrop-filter: blur(1px);
  -webkit-backdrop-filter: blur(1px);
  mask-image: linear-gradient(to top, black, transparent);
  -webkit-mask-image: linear-gradient(to top, black, transparent);
}
[data-slot="progressive-blur"] > div:nth-child(2) {
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
  mask-image: linear-gradient(to top, black 30%, transparent 70%);
  -webkit-mask-image: linear-gradient(to top, black 30%, transparent 70%);
}
[data-slot="progressive-blur"] > div:nth-child(3) {
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  mask-image: linear-gradient(to top, black 10%, transparent 45%);
  -webkit-mask-image: linear-gradient(to top, black 10%, transparent 45%);
}
@media (prefers-reduced-motion: reduce) {
  [data-slot="progressive-blur"] { display: none; }
}
[data-slot="retro-grid"] {
  position: relative; overflow: hidden;
  perspective: 240px;
  color: var(--cronus-fg);
}
[data-slot="retro-grid-field"] {
  position: absolute; inset: 0; pointer-events: none;
  overflow: hidden;
  mask-image: linear-gradient(to bottom, transparent, black 20%, black 70%, transparent);
  -webkit-mask-image: linear-gradient(to bottom, transparent, black 20%, black 70%, transparent);
}
[data-slot="retro-grid-field"]::after {
  content: "";
  position: absolute; inset-inline: 0; bottom: -50%;
  height: 200%;
  transform: rotateX(60deg);
  transform-origin: 50% 0%;
  background-image:
    linear-gradient(to right, color-mix(in oklch, var(--cronus-border) 70%, transparent) 1px, transparent 1px),
    linear-gradient(to bottom, color-mix(in oklch, var(--cronus-border) 70%, transparent) 1px, transparent 1px);
  background-size: 48px 48px;
  animation: cui-retro-grid 8s linear infinite;
}
@keyframes cui-retro-grid {
  from { transform: rotateX(60deg) translateY(0); }
  to { transform: rotateX(60deg) translateY(48px); }
}
@media (prefers-reduced-motion: reduce) {
  [data-slot="retro-grid-field"]::after { animation: none; }
}
[data-slot="retro-grid-content"] {
  position: relative; z-index: 1;
}
[data-slot="ripple"] {
  position: relative; overflow: hidden;
  color: var(--cronus-fg);
}
[data-slot="ripple-field"] {
  position: absolute; inset: 0; pointer-events: none;
}
[data-slot="ripple-ring"] {
  position: absolute; left: 50%; top: 50%;
  width: 220%; aspect-ratio: 1;
  border-radius: 999px;
  border: 1px solid color-mix(in oklch, var(--cronus-primary) 40%, transparent);
  opacity: 0;
  transform: translate(-50%, -50%) scale(0);
  animation: cui-ripple 8s ease-out infinite backwards;
}
[data-slot="ripple-ring"]:nth-of-type(1) { animation-delay: 0s; }
[data-slot="ripple-ring"]:nth-of-type(2) { animation-delay: 2s; }
[data-slot="ripple-ring"]:nth-of-type(3) { animation-delay: 4s; }
[data-slot="ripple-ring"]:nth-of-type(4) { animation-delay: 6s; }
@keyframes cui-ripple {
  from { transform: translate(-50%, -50%) scale(0); opacity: 0.35; }
  to { transform: translate(-50%, -50%) scale(1); opacity: 0; }
}
@media (prefers-reduced-motion: reduce) {
  [data-slot="ripple-field"] { display: none; }
}
[data-slot="ripple-content"] {
  position: relative;
}
[data-slot="motion-presets"] {
  display: block;
  line-height: 1.5;
  color: var(--cronus-fg);
}
[data-slot="motion-preset"] { display: block; }

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
        assert!(!html.contains("style="));
        let css = component_chrome_css();
        assert!(css.contains("[data-slot=\"button\"]:disabled { opacity: 0.5; pointer-events: none; }"));
    }

    #[test]
    fn button_wave1t_geometry_matches_react() {
        // React primary/ghost/destructive: no border; text-sm -> 1.25rem line-height.
        let css = component_chrome_css();
        assert!(css.contains("line-height: 1.25rem; cursor: pointer; text-decoration: none;\n  outline: none; border: 0 solid transparent;"));
        assert!(css.contains("font-size: 0.75rem; line-height: 1rem; }"));
        assert!(css.contains("font-size: 1rem; line-height: 1.5rem; }"));
        assert!(css.contains("border-width: 1px; border-color: var(--cronus-border);"));
        let input = &css[css.find("[data-slot=\"input\"] { line-height").expect("input lh")..];
        assert!(input.starts_with("[data-slot=\"input\"] { line-height: 1.25rem; }"));
        assert!(css.contains("[data-slot=\"textarea\"] { line-height: 1.25rem; }"));
        assert!(css.contains("[data-slot=\"toggle\"] {\n  line-height: 1.25rem;"));
        assert!(css.contains("font-size: 0.75rem; line-height: 1rem; color: var(--cronus-fg-secondary);"));
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

    /// The React audit reference renders with Tailwind v4 preflight: form
    /// controls inherit font/line-height (else 13.33px / `normal`), everything
    /// is border-box with zeroed margins. The audit document must match, and the
    /// preflight must precede the chrome so component rules still win.
    #[test]
    fn audit_stylesheet_starts_with_tailwind_preflight() {
        let css = audit_stylesheet();
        assert!(css.starts_with(AUDIT_PREFLIGHT));
        assert!(css.find(AUDIT_PREFLIGHT).unwrap() < css.find(COMPONENT_CHROME).unwrap());
        assert!(AUDIT_PREFLIGHT.contains("box-sizing: border-box; margin: 0; padding: 0; border: 0 solid;"));
        assert!(AUDIT_PREFLIGHT.contains("font: inherit; font-feature-settings: inherit;"));
        assert!(AUDIT_PREFLIGHT.contains("letter-spacing: inherit; color: inherit; border-radius: 0; background-color: transparent;"));
        assert!(AUDIT_PREFLIGHT.contains("ol, ul, menu { list-style: none; }"));
        assert_eq!(
            AUDIT_PREFLIGHT.matches('{').count(),
            AUDIT_PREFLIGHT.matches('}').count()
        );
        assert!(!component_chrome_css().contains(AUDIT_PREFLIGHT));
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
