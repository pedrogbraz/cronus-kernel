//! Per-page cronus-ui CSS.
//!
//! The former `COMPONENT_CHROME` string (220 KB, shipped whole on every page)
//! lives in `src/cronus_ui_css/`:
//!
//! - `<family>.css`: blocks owned by one family (named by family slug).
//! - `shared.css`: generic overlay rules (`[popover]`, `*-control`) and blocks
//!   whose selector list spans several families.
//! - `catalog.css`: `/components` + `type:components` kit catalog only.
//! - `theme.css`: `html[data-cronus-theme]` chrome (emitted with the tokens).
//! - `tokens.css` (vendored `@cronus-ui/tokens`, do not hand-edit palettes),
//!   `fallback.css` (legacy `--cronus-*` aliases), `base.css` (focus ring +
//!   reduced motion), `audit.css` (Tailwind preflight, audit document only).
//! - `MANIFEST`: one line per original block, in the original cascade order,
//!   with its file, owners and hash. Emission walks it, so any subset keeps the
//!   old relative order and cannot flip a cascade tie.
//!
//! A page gets tokens (+ theme) always; base, shared and family blocks only for
//! the families found in its rendered HTML (`data-slot` scan) or noted by
//! `cronus_ui_widgets::render`. Layered output wraps everything in
//! [`LAYER_ORDER`] so unlayered author CSS wins without `!important`.

use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::sync::OnceLock;

pub const LAYER_ORDER: &str = "@layer cronus.tokens, cronus.base, cronus.components;";
pub const TOKENS_CSS: &str = include_str!("cronus_ui_css/tokens.css");
pub const FALLBACK_CSS: &str = include_str!("cronus_ui_css/fallback.css");
pub const BASE_CSS: &str = include_str!("cronus_ui_css/base.css");
pub const AUDIT_CSS: &str = include_str!("cronus_ui_css/audit.css");
const MANIFEST: &str = include_str!("cronus_ui_css/MANIFEST");

/// `data-slot` prefixes that are not family names.
const SLOT_ALIASES: &[(&str, &[&str])] = &[
    ("notification", &["notification-center"]),
    ("comparison", &["comparison-slider"]),
    ("description", &["description-list"]),
    ("motion-preset", &["motion-presets"]),
    ("toaster", &["sonner", "toast"]),
];

/// Files that are not a family slug.
const SPECIAL_FILES: &[&str] = &["shared", "catalog", "theme"];

// @generated FILES: one entry per `src/cronus_ui_css/<name>.css` named in MANIFEST.
macro_rules! css_files {
    ($($name:literal),* $(,)?) => {
        &[$(($name, include_str!(concat!("cronus_ui_css/", $name, ".css")))),*]
    };
}
#[rustfmt::skip]
const FILES: &[(&str, &str)] = css_files![
    "accordion", "alert", "alert-dialog", "animated-button", "animated-checkbox", "animated-list",
    "animated-number", "app-shell", "area-chart", "aspect-ratio", "aurora-background",
    "autocomplete", "avatar",
    "avatar-group", "badge", "banner", "bar-chart", "border-beam", "bouncy-accordion",
    "breadcrumb", "button", "button-group", "calendar", "candlestick-chart", "card",
    "card-stack", "carousel", "catalog", "chart", "checkbox", "chip", "choropleth-chart",
    "click-spark", "code-block", "code-tabs", "collapsible", "color-picker", "combobox",
    "command", "comparison-slider", "composed-chart", "confetti", "confirmation-dialog",
    "context-menu", "copy-button", "countdown", "credit-card-input", "currency-input",
    "data-table", "date-picker", "date-range-picker", "description-list", "dialog", "dock",
    "dot-pattern", "drawer", "dropdown-menu", "dynamic-island", "empty", "expandable-tabs",
    "fab", "field", "file-dropzone", "flickering-grid", "flip-card", "floating-label-input",
    "form", "frame", "funnel-chart", "gauge-chart", "glare-hover", "glass-card", "goal-card",
    "gradient-border", "gradient-text", "grid-pattern", "heatmap", "heatmap-chart",
    "highlighter", "hover-card", "image-zoom", "input", "input-group", "input-otp",
    "invite-dialog", "json-viewer", "kanban", "kbd", "label", "light-rays", "lightbox",
    "line-chart", "live-line-chart", "loader", "logo-carousel", "magnetic", "marquee", "masonry",
    "menubar", "meteors", "metric", "mode-toggle", "morphing-popover", "motion-presets", "multi-select",
    "navigation-menu", "noise", "notification-center", "number-flow", "number-input", "orbit",
    "pagination",
    "particles", "password-input", "phone-input", "pie-chart", "pill-nav", "popover",
    "profit-loss-chart", "progress", "progressive-blur", "radar-chart", "radio-group", "rating",
    "resizable", "retro-grid", "reveal", "rich-text-editor", "ring-chart", "ripple",
    "sankey-chart", "scatter-chart", "scheduler", "scramble-text", "scroll-area", "scroll-nav", "scroll-progress",
    "segmented-control", "select", "separator", "shared", "sheet", "shimmer", "shiny-text",
    "sidebar", "signature-pad", "skeleton", "slide-up-text", "slider", "sparkles-text",
    "sparkline", "spinner",
    "spinning-text", "split-button", "spotlight-card", "star-border", "status-dot", "stepper",
    "sunburst-chart", "switch", "table", "table-of-contents", "tabs", "tags-input", "terminal",
    "text-effect", "text-shimmer", "textarea", "theme", "tilt-card", "time-picker", "timeline",
    "toast", "todo-item", "toggle", "toggle-group", "toolbar", "tooltip", "tree-view", "typing-text",
    "usage-meter", "video-player", "word-rotate", "workspace-switcher",
    // Sprint 5 C2 — AI suite (alphabetical).
    "conversation", "inline-citation", "message", "prompt-input", "reasoning", "sources",
    "suggestion", "tool",
];
// @end FILES

enum Owners {
    /// The file's own family.
    File,
    /// Any page with a `data-slot`.
    AnySlot,
    /// Any of these families.
    Families(Vec<&'static str>),
}

struct Block {
    file: &'static str,
    owners: Owners,
    text: &'static str,
}

struct Registry {
    theme: String,
    stream: Vec<Block>,
}

/// Top-level blocks of a stylesheet (comments and strings aware). Leading
/// comments stay attached to the block that follows them.
fn split_blocks(css: &'static str) -> Vec<&'static str> {
    let b = css.as_bytes();
    let (mut i, mut depth, mut start) = (0usize, 0usize, 0usize);
    let mut out = Vec::new();
    while i < b.len() {
        match b[i] {
            b'/' if b.get(i + 1) == Some(&b'*') => {
                i = css[i + 2..]
                    .find("*/")
                    .map(|p| i + 2 + p + 2)
                    .unwrap_or(b.len());
                continue;
            }
            q @ (b'"' | b'\'') => {
                i += 1;
                while i < b.len() && b[i] != q {
                    if b[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    out.push(css[start..=i].trim());
                    start = i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    out
}

fn manifest_entries() -> impl Iterator<Item = (&'static str, &'static str)> {
    MANIFEST.lines().filter_map(|line| {
        let f: Vec<&'static str> = line.split_whitespace().collect();
        match f.first().copied() {
            Some("keep") if f.len() >= 5 => Some((f[2], f[4])),
            Some("edit") if f.len() >= 6 => Some((f[2], f[5])),
            Some("new") if f.len() >= 4 => Some((f[1], f[3])),
            _ => None,
        }
    })
}

fn registry() -> &'static Registry {
    static REG: OnceLock<Registry> = OnceLock::new();
    REG.get_or_init(|| {
        let mut queues: HashMap<&str, std::vec::IntoIter<&'static str>> = FILES
            .iter()
            .map(|(name, css)| (*name, split_blocks(css).into_iter()))
            .collect();
        let mut theme = String::new();
        let mut stream = Vec::new();
        for (file, owners) in manifest_entries() {
            let text = queues
                .get_mut(file)
                .and_then(Iterator::next)
                .unwrap_or_else(|| panic!("cronus_ui_css MANIFEST: no block left in {file}.css"));
            if file == "theme" {
                theme.push_str(text);
                theme.push('\n');
                continue;
            }
            let owners = match owners {
                "-" => Owners::File,
                "*" => Owners::AnySlot,
                list => Owners::Families(list.split(',').collect()),
            };
            stream.push(Block { file, owners, text });
        }
        Registry { theme, stream }
    })
}

fn slot_index() -> &'static HashMap<&'static str, &'static [&'static str]> {
    static IDX: OnceLock<HashMap<&'static str, &'static [&'static str]>> = OnceLock::new();
    IDX.get_or_init(|| {
        let families: &'static [&'static str] = crate::cronus_ui_widgets::FAMILIES;
        let mut map: HashMap<&'static str, &'static [&'static str]> = families
            .iter()
            .map(|f| (*f, std::slice::from_ref(f)))
            .collect();
        map.insert("catalog", &["catalog"]);
        for (prefix, owners) in SLOT_ALIASES {
            map.entry(prefix).or_insert(owners);
        }
        map
    })
}

/// Families that own a `data-slot` value: the longest family (or alias)
/// prefix at a `-` boundary. `input-group-addon` → `input-group`.
pub fn slot_families(slot: &str) -> &'static [&'static str] {
    let idx = slot_index();
    let mut candidate = slot;
    loop {
        if let Some(owners) = idx.get(candidate) {
            return owners;
        }
        match candidate.rfind('-') {
            Some(i) => candidate = &candidate[..i],
            None => return &[],
        }
    }
}

/// cronus-ui usage of one rendered document.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Usage {
    /// The markup carries at least one `data-slot`.
    pub any_slot: bool,
    pub families: BTreeSet<&'static str>,
}

/// Families present in rendered HTML, by `data-slot` value. Nested renderers
/// (a dialog's buttons) are found because their slots are in the markup.
pub fn usage_of(html: &str) -> Usage {
    const NEEDLE: &str = "data-slot=\"";
    let mut usage = Usage::default();
    let mut rest = html;
    while let Some(p) = rest.find(NEEDLE) {
        rest = &rest[p + NEEDLE.len()..];
        usage.any_slot = true;
        let end = rest.find('"').unwrap_or(rest.len());
        usage
            .families
            .extend(slot_families(&rest[..end]).iter().copied());
        rest = &rest[end..];
    }
    usage
}

thread_local! {
    static NOTED: RefCell<BTreeSet<&'static str>> = const { RefCell::new(BTreeSet::new()) };
}

/// Family-usage registry: `cronus_ui_widgets::render` notes each family it
/// renders; the next stylesheet built on this thread takes the set. Rendering
/// and layout run synchronously on one thread, and a stale entry can only add
/// CSS, never remove it. The `data-slot` scan stays the primary source.
pub fn note_family(family: &str) {
    let owners = slot_families(family);
    if !owners.is_empty() {
        NOTED.with(|n| n.borrow_mut().extend(owners.iter().copied()));
    }
}

fn take_noted() -> BTreeSet<&'static str> {
    NOTED.with(|n| std::mem::take(&mut *n.borrow_mut()))
}

fn page_usage(html: &str) -> Usage {
    crate::cronus_ui_kit::reset_instance_ids();
    let mut usage = usage_of(html);
    let noted = take_noted();
    if !noted.is_empty() {
        usage.any_slot = true;
        usage.families.extend(noted);
    }
    usage
}

/// `system` mode for a named preset: its vendored other-mode block (Aurora's
/// light delta, or a light-first preset's dark delta) re-keyed from
/// `[data-cronus-mode="light|dark"]` to `[data-cronus-mode="system"]` inside
/// `@media (prefers-color-scheme: …)`. The base block is already on `:root`
/// and the preset attribute, so only the delta ships (~1.5 KB), and nothing
/// applies when `<html>` sets `light`/`dark` explicitly (the audit canvas).
pub fn system_mode_css(preset: &str) -> String {
    let preset = preset.trim();
    for other in ["light", "dark"] {
        let selector = format!("[data-cronus-theme=\"{preset}\"][data-cronus-mode=\"{other}\"] {{");
        let Some(start) = TOKENS_CSS.find(&selector) else {
            continue;
        };
        let body_start = start + selector.len();
        let Some(len) = TOKENS_CSS[body_start..].find('}') else {
            continue;
        };
        let body = &TOKENS_CSS[body_start..body_start + len];
        return format!(
            "@media (prefers-color-scheme: {other}) {{\n[data-cronus-theme=\"{preset}\"][data-cronus-mode=\"system\"] {{{body}}}\n}}\n"
        );
    }
    String::new()
}

/// Presets in the vendored tokens, each with a base block and one other-mode block.
const MODE_TOGGLE_PRESETS: &[&str] = &["aurora", "neutral", "midnight", "sunset", "emerald"];

/// Zero-JS `mode-toggle`: while a toggle checkbox on the page is checked, the
/// document takes the other colour mode's tokens. For each preset, the vendored
/// other-mode block (Aurora's light delta, a light-first preset's dark delta)
/// is re-keyed onto the default mode + `:has(checked)`, and the base block onto
/// the other mode + `:has(checked)`. `system` mode is left alone. Layouts and
/// the audit document always stamp both attributes on `<html>`.
pub fn mode_toggle_css() -> String {
    const ON: &str = ":has(input[data-cui-mode-toggle]:checked)";
    let body = |selector: &str| {
        let start = TOKENS_CSS.find(selector)? + selector.len();
        let len = TOKENS_CSS[start..].find('}')?;
        Some(&TOKENS_CSS[start..start + len])
    };
    let mut css = String::new();
    for p in MODE_TOGGLE_PRESETS {
        let Some(base) = body(&format!("[data-cronus-theme=\"{p}\"] {{")) else {
            continue;
        };
        let Some((other, delta)) = ["light", "dark"].iter().find_map(|m| {
            body(&format!(
                "[data-cronus-theme=\"{p}\"][data-cronus-mode=\"{m}\"] {{"
            ))
            .map(|b| (*m, b))
        }) else {
            continue;
        };
        let theme = format!("[data-cronus-theme=\"{p}\"]");
        css.push_str(&format!(
            "{theme}:not([data-cronus-mode=\"{other}\"]):not([data-cronus-mode=\"system\"]){ON} {{{delta}}}\n"
        ));
        css.push_str(&format!(
            "{theme}[data-cronus-mode=\"{other}\"]{ON} {{{base}}}\n"
        ));
    }
    css
}

fn tokens_part(preset: &str, mode: &str, audit: bool) -> String {
    let preset = preset.trim();
    let mut css = String::new();
    if audit {
        css.push_str(AUDIT_CSS);
        css.push('\n');
    }
    css.push_str(FALLBACK_CSS);
    css.push('\n');
    if audit {
        // Theme/mode come from `data-cronus-theme`/`data-cronus-mode` on <html>.
        css.push_str(TOKENS_CSS);
        css.push('\n');
    } else if crate::cronus_ui::is_named_preset(preset) {
        // Author opted in: apply that preset on :root as well as the data attr.
        let needle = format!("[data-cronus-theme=\"{preset}\"] {{");
        let repl = format!(":root, [data-cronus-theme=\"{preset}\"] {{");
        css.push_str(&TOKENS_CSS.replacen(&needle, &repl, 1));
        css.push('\n');
        if crate::cronus_ui::normalize_mode(mode) == "system" {
            css.push_str(&system_mode_css(preset));
        }
    }
    css.push_str(&registry().theme);
    css
}

fn components_part(usage: &Usage, all: bool) -> String {
    let mut css = String::new();
    for block in &registry().stream {
        let on = all
            || match &block.owners {
                Owners::File => usage.families.contains(block.file),
                Owners::AnySlot => usage.any_slot,
                Owners::Families(list) => list.iter().any(|f| usage.families.contains(f)),
            };
        if on {
            css.push_str(block.text);
            css.push('\n');
        }
    }
    css
}

/// `@layer <name> { css }`, or nothing for empty CSS.
pub fn wrap_layer(layer: &str, css: &str) -> String {
    if css.trim().is_empty() {
        String::new()
    } else {
        format!("@layer {layer} {{\n{css}\n}}\n")
    }
}

fn assemble(tokens: &str, base: &str, components: &str, layered: bool) -> String {
    if !layered {
        return format!("{tokens}\n{base}\n{components}");
    }
    let mut css = String::with_capacity(tokens.len() + base.len() + components.len() + 128);
    css.push_str(LAYER_ORDER);
    css.push('\n');
    css.push_str(&wrap_layer("cronus.tokens", tokens));
    css.push_str(&wrap_layer("cronus.base", base));
    css.push_str(&wrap_layer("cronus.components", components));
    css
}

fn build(preset: &str, mode: &str, usage: &Usage, layered: bool, audit: bool) -> String {
    let base = if usage.any_slot || !usage.families.is_empty() {
        BASE_CSS
    } else {
        ""
    };
    let mut tokens = tokens_part(preset, mode, audit);
    if usage.families.contains("mode-toggle") {
        tokens.push_str(&mode_toggle_css());
    }
    assemble(&tokens, base, &components_part(usage, false), layered)
}

/// Dark-mode page stylesheet (see `page_stylesheet_for`).
pub fn page_stylesheet(preset: &str, html: &str, layered: bool) -> String {
    page_stylesheet_for(preset, html, layered, "dark")
}

/// Stylesheet for one rendered page: tokens and theme always; base, shared and
/// family blocks only for the families used by `html`. `mode` is the
/// `style { theme }` value; only `system` changes the CSS (light/dark are
/// selected by `data-cronus-mode` on `<html>`).
pub fn page_stylesheet_for(preset: &str, html: &str, layered: bool, mode: &str) -> String {
    build(preset, mode, &page_usage(html), layered, false)
}

/// Self-contained audit canvas stylesheet: preflight + tokens (theme/mode from
/// `<html>` attributes) + the families in `widget_html`. Always layered.
pub fn audit_stylesheet(widget_html: &str) -> String {
    build("", "dark", &page_usage(widget_html), true, true)
}

/// Every family, unlayered: the pre-split `token_css` output (dark mode).
pub fn full_stylesheet(preset: &str) -> String {
    full_stylesheet_for(preset, "dark")
}

/// Every family, unlayered, for a color mode.
pub fn full_stylesheet_for(preset: &str, mode: &str) -> String {
    assemble(
        &tokens_part(preset, mode, false),
        BASE_CSS,
        &components_part(&Usage::default(), true),
        false,
    )
}

/// Every family, layered, with the audit preflight.
pub fn full_audit_stylesheet() -> String {
    assemble(
        &tokens_part("", "dark", true),
        BASE_CSS,
        &components_part(&Usage::default(), true),
        true,
    )
}

/// Theme chrome followed by every component block, in cascade order.
pub fn full_components() -> &'static str {
    static FULL: OnceLock<String> = OnceLock::new();
    FULL.get_or_init(|| {
        format!(
            "{}{}",
            registry().theme,
            components_part(&Usage::default(), true)
        )
    })
}

/// `<meta name="cronus-ui-css">` with a hash of the page's cronus-ui CSS. SPA
/// routers (`render.rs`, landing layout) swap only `<main>`; when the target
/// page's hash differs they do a full load so its `<head>` CSS applies.
pub fn css_meta(css: &str) -> String {
    format!(
        "<meta name=\"cronus-ui-css\" content=\"{:016x}\">",
        fnv64(css)
    )
}

fn fnv64(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in s.bytes() {
        h ^= u64::from(byte);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Comments removed, whitespace runs collapsed, trimmed (same as the
    /// generator that wrote MANIFEST).
    fn normalize(css: &str) -> String {
        let mut out = String::new();
        let mut rest = css;
        while let Some(p) = rest.find("/*") {
            out.push_str(&rest[..p]);
            rest = match rest[p + 2..].find("*/") {
                Some(e) => &rest[p + 2 + e + 2..],
                None => "",
            };
        }
        out.push_str(rest);
        out.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    const ORIGINAL_BLOCKS: usize = 1461;

    /// Every non-`keep` MANIFEST line. Removing or editing an original block
    /// must be deliberate: add it here with its reason.
    const INTENTIONAL_CHANGES: &[&str] = &[
        "edit 0053 dropdown-menu d670cf6fc08947ff 09fc1d645c4420f0 - dead-slot combobox-item",
        "edit 0054 dropdown-menu 7b2598b77fae9ea5 424e7bf3dae03a3a - dead-slot combobox-item",
        "edit 0055 shared b29c1ce7d338311d 1c4516271890d812 dropdown-menu,popover dead-slot combobox-content",
        "drop 0067 3d448f4b3251ec55 dead-slot combobox-content",
        "drop 0068 4acc7eb25a02c57e dead-slot combobox-item",
        "drop 0073 337a0fe059b71091 native-popover",
        "drop 0080 dd57e9b1ef15bbdd duplicate-of 0630",
        "edit 0105 button 67c3db2e8b37557e caafafee8dbeb2e9 - kit-catalog",
        "edit 0107 button e1fdba7aa3e3c216 f814a7aa8c8beea8 - kit-catalog",
        "edit 0120 button 5cb9b933e3f74fd5 5657b0938aba861b - kit-catalog",
        "edit 0142 checkbox eb979a3554dba3e8 b5310d852a8eb084 - kit-catalog",
        "edit 0148 switch e00ad3198951ae9e 9f78192e8a3bc05c - kit-catalog",
        "edit 0159 toggle eeb40eb98caba78f 193e5e7c62302cd5 - kit-catalog",
        "edit 0264 skeleton a3f1f5f9ca9c93a2 d482cce5f3ac16e9 - fluid-width",
        "edit 0268 slider 954c72ddabb73151 1e6810c23e740a83 - fluid-width+rtl-logical",
        "edit 0269 slider 887d176a3ed16b9d 4dc9197403523f10 - fluid-width+rtl-logical",
        "edit 0270 slider 05f68fea52e42da3 4b78b130edc813cf - kit-catalog",
        "edit 0372 radio-group 89448c1119e8ba2d d8b3d2f239f00eee - kit-catalog",
        "edit 0373 radio-group d3fa4fd94e07f04e 9328efb1dd5ca626 - kit-catalog",
        "edit 0377 chip e498c5f834ec50f9 520cda3814384f75 - kit-catalog",
        "edit 0382 spotlight-card f56a3123befe8a71 08bd99cd223cd4d4 - fluid-width",
        "edit 0388 card 0a5cd4803ec7e826 b8ce1556f40a92d8 - rtl-logical",
        "edit 0391 card cce28aac88cba255 9d340a383b050423 - rtl-logical",
        "edit 0392 card 50eedcfbe719006d ad25a5791836fd2d - rtl-logical",
        "edit 0476 table 6866c7987fda2b41 d15054ac9a2e6364 - rtl-logical",
        "edit 0482 input-group e45fa6f9750bb519 96d1eb514b775d12 - rtl-logical",
        "edit 0484 rating d515febefbbb4013 571ad69467d87304 - kit-catalog",
        "edit 0488 rating 29a82bbc1e4252f7 ca1bb70e41aa2575 - kit-catalog",
        "edit 0491 rating 57b00a1231c5bc9d f0a6c8d2a700ae89 - kit-catalog",
        "edit 0492 copy-button de94e28b343662ec da1840f7f8d2d11e - kit-catalog",
        "edit 0499 fab f783e26579c521ec 32fe82cee1347648 - kit-catalog",
        "edit 0500 fab 1e7ecc7e46a33609 f59d002cb3cccf08 - kit-catalog",
        "edit 0501 fab 88a782746cd95ce3 fa7853c9ca1359e2 - kit-catalog",
        "edit 0506 toggle-group 813afdcb8219a0d9 534a033122a46abf - kit-catalog",
        "edit 0513 avatar-group be253346a180b206 f08c85e8bc1bc67f - rtl-logical",
        "edit 0518 button-group c21578a2bd4d8565 1848b154301a2bdb - rtl-logical",
        "edit 0519 button-group 98cd1d9976ec1434 55def2d1e9120385 - rtl-logical",
        "edit 0521 button-group 7380cd574da85b4a 4ebee069dc7fbe84 - rtl-logical",
        "edit 0522 button-group 4e0ab56457700ec5 5b32fded6c630d57 - rtl-logical",
        "edit 0529 combobox 3b3ae8f5b25cdc80 98993517048e52eb - rtl-logical",
        "drop 0531 10570e9feebc0200 dead-slot combobox-content",
        "drop 0532 19d506c62fe39e86 dead-slot combobox-item",
        "drop 0533 e4b51a5bc215dfbe dead-slot combobox-item",
        "edit 0554 input-otp 51e017e6f999d072 132b2c080cc8e205 - rtl-logical",
        "edit 0556 input-otp a8b7a6b8de41f3c1 299734ae117d05ae - rtl-logical",
        "edit 0557 input-otp 1844db63f820786e 13f000ca45b25f7c - rtl-logical",
        "edit 0558 input-otp 1f27f44142134629 b7df97b1e2b74928 - rtl-logical",
        "edit 0572 mode-toggle 2d6bdb526d5735c5 cb290ab77936d748 - kit-catalog",
        "edit 0582 command ccbe07bf303dbbd8 13f807887a1fbe6e - fluid-width",
        "edit 0602 drawer af5a827f67c2126f f7c352b8e723144d - rtl-logical",
        "edit 0654 date-range-picker dcb356a0dde5d611 f9180117126f3fe4 - rtl-logical",
        "edit 0655 date-range-picker f2293e1cbd120009 239be8fcbc037ab6 - rtl-logical",
        "edit 0686 data-table 0101660718339584 5fec1161d3905c4d - rtl-logical",
        "edit 0690 sidebar 513f59157d798b32 260562aca047148d - rtl-logical",
        "edit 0698 sidebar dd117fea5d3de876 042615eea0855cb1 - rtl-logical",
        "edit 0711 phone-input 2c15a6677afaf871 d3d9be5bc7e3f6b3 - rtl-logical",
        "edit 0723 currency-input 7bff3ec4c879d8ba 2859e20157ffb9e0 - rtl-logical",
        "edit 0734 scroll-area 20361d594149c881 2936aee452e7930d - fluid-width",
        "edit 0755 tags-input a98cfa9cfcb73b32 0e1a3a1bdfa7ab31 - rtl-logical",
        "edit 0766 autocomplete 740db6c4ad65d7f2 7e0633b7e571569f - rtl-logical",
        "edit 0782 multi-select 43ae9c8e4da0ae91 073b070e899e2554 - rtl-logical",
        "edit 0808 floating-label-input 59b37c3b116b8128 13677f1e77d038e3 - rtl-logical",
        "edit 0818 split-button 6a5eb8f0d12bf538 875fa4f701c673f2 - rtl-logical",
        "edit 0819 split-button d8249b2b29909828 93cd18848374e2e3 - rtl-logical",
        "edit 0820 split-button f9215cafb019da6a daa88e3912342c7f - rtl-logical",
        "edit 0823 split-button 59edad02355663a6 d775fdd8c34fa939 - kit-catalog",
        "edit 0847 table-of-contents 3e1e8b2fdc1f93af 7fc83188aa81b0b0 - rtl-logical",
        "edit 0860 signature-pad 7217d5f697a92e8b f9cc627ed88ad1bb - rtl-logical",
        "edit 0863 signature-pad 76052660b192ab23 e40c1f98435b6ff2 - rtl-logical",
        "edit 0884 scheduler 4d5d21e381f6af4d 2482f5f28959edd8 - rtl-logical",
        "edit 0885 scheduler 10dad5d652712d58 8be82d141d7b1027 - rtl-logical",
        "edit 0890 scheduler cb65834052e5ac77 b236d8b84d6bc942 - rtl-logical",
        "edit 0920 lightbox 82b3286ae471910d f2ceccd0f0755159 - zero-js-label",
        "edit 0921 lightbox 0762cdd3f7f3743d 403126419ab403fa - zero-js-label",
        "edit 0922 lightbox bbc473b0963f8aaf b381e113ff88f0d3 - zero-js-label",
        "edit 0956 masonry 6414a25b66c839a7 8c3ed0c3b05a18db - fluid-width",
        "edit 0979 code-tabs 6baaa7ae42f4eb81 edb15edd51f5965c - rtl-logical",
        "edit 0986 code-tabs f3e70770a8690f64 c721c723290e304b - rtl-logical",
        "edit 0989 code-tabs 08d19d373b04cc10 d07c2d20d98a262f - rtl-logical",
        "edit 1010 scroll-progress db40775229ca10ab b310519200952d73 - fluid-width",
        "edit 1048 shimmer f8c62b68eea13a38 234944956a7f500e - fluid-width",
        "edit 1055 particles 29eb1a3ae353c6b5 52524b75c6ae6c81 - fluid-width",
        "edit 1062 sparkles-text 6fa54eca8e9bd5d0 2eebac6a12602315 - rtl-logical",
        "edit 1064 noise 2a2085735d7b43c8 40c7e42be65c5f7b - fluid-width",
        "edit 1076 bouncy-accordion 9c0a088c53ddba70 20b4beeb4e38d26c - rtl-logical",
        "edit 1081 bouncy-accordion 15ef72afa120c690 8c68b9b58b8e90ec - rtl-logical",
        "edit 1082 bouncy-accordion 6a3bb3cd1605645d d0fc05bba8141aa1 - rtl-logical",
        "edit 1116 tilt-card e0a951fc10377b0c 5baacc4b2564ff15 - fluid-width",
        "edit 1118 star-border 91b699d57df3bfb9 d8efd27effb23799 - fluid-width",
        "edit 1125 glass-card 7b88a3bbe9d43e1d c372a7125e994ff4 - fluid-width",
        "edit 1128 terminal b8d796e8670ccb9f 867aa3d9e48c20da - fluid-width",
        "edit 1135 terminal ba7c352a6dd33b96 5da0f48d75ed00c3 - rtl-logical",
        "edit 1175 carousel ac856209dbebf162 d3906a0db9feadfb - fluid-width",
        "edit 1176 carousel 7428138cc2f22eab 7ba6ee385ca9c362 - rtl-logical",
        "edit 1178 carousel 5e064e3340602b72 b321774864b9c7d9 - rtl-logical",
        "edit 1185 code-block d3f6a5be1bac270f 6b19407d1a69cee7 - fluid-width",
        "edit 1186 code-block 4c0391de0ef74a5b e7d861b9d622d854 - rtl-logical",
        "edit 1195 description-list b88a2533160bd592 14e7b27386e0c11a - fluid-width",
        "edit 1214 json-viewer 340994334bf4dca0 6765cf28b9315c93 - fluid-width",
        "edit 1223 json-viewer 610466e85825412c e26ebb254cb3a55b - rtl-logical",
        "edit 1232 marquee 76ec4d3744788704 7d7492452d74f5f9 - fluid-width",
        "edit 1243 aspect-ratio d01ccaa4a2e0b3fd 89f829e272089d02 - fluid-width",
        "edit 1244 frame 84ef520bd9e87b7a 6af87c1b603ae0f3 - fluid-width",
        "edit 1253 flip-card 1a59654b409cc6d6 26705dd4113f1883 - fluid-width",
        "edit 1266 animated-button fcaa9878c1279215 377218798187f27c - kit-catalog",
        "edit 1269 animated-button e87f2c585d3172fe b4e77a5db2ffd764 - kit-catalog",
        "edit 1270 card-stack 45e51d5120424d60 b087280a33e8b302 - fluid-width",
        "edit 1280 logo-carousel 1aa806280af8fb37 1d4e40ead78ef924 - fluid-width",
        "edit 1293 image-zoom b53cd2d150098233 141aca21869c38fe - fluid-width",
        "edit 1300 aurora-background 8bc3792c8ef7888e 64633a0c9c89f16a - fluid-width",
        "edit 1309 border-beam bda1e5c4d9decb54 57065944325ae64a - fluid-width",
        "edit 1315 confetti e2002970e087288c 2178eda0da462f93 - fluid-width",
        "edit 1330 click-spark 471c5ba1bb7d58e1 c5d36a49488a2e7c - fluid-width",
        "edit 1333 glare-hover f6215effcad9630a 5cfdce37f871dc89 - fluid-width",
        "edit 1338 magnetic b8ea707717573f67 27f96e97791afe92 - fluid-width+rtl-logical",
        "edit 1342 dot-pattern 85b3813b645613af 01c74d722cd922d4 - fluid-width",
        "edit 1345 flickering-grid 384afc00cec0b896 c0635d5a27ce646e - fluid-width",
        "edit 1350 grid-pattern 702a57a3e0e5baa7 6bb52c4c9aba8cf1 - fluid-width",
        "edit 1401 gradient-border 55d5663218f54f99 fb44a5ed19e85a6a - fluid-width",
        "edit 1403 light-rays 9723bf9870382629 c25d6927a7ea8060 - fluid-width",
        "edit 1409 orbit 0889fd4cb288103f 76d4501727d4c7a5 - fluid-width",
        "edit 1432 progressive-blur c30b57293e11a852 ab568ec7444f1330 - fluid-width",
        "edit 1439 retro-grid cd563f0e6f2c6ad3 6553063ed787dbbd - fluid-width",
        "edit 1446 ripple 5a1e5fb407ddfcb1 011c6227d04460a0 - fluid-width",
    ];

    /// `(selector, property, value)` for every declaration of a stylesheet
    /// (comments removed; nested at-rules flattened; selector = innermost).
    fn declarations(css: &str) -> Vec<(String, String, String)> {
        let text = normalize(css);
        let mut out = Vec::new();
        let mut selectors: Vec<String> = Vec::new();
        let mut seg = String::new();
        for c in text.chars() {
            match c {
                '{' => {
                    selectors.push(seg.trim().to_string());
                    seg.clear();
                }
                ';' | '}' => {
                    if let Some((prop, value)) = seg.split_once(':') {
                        let prop = prop.trim();
                        if !prop.is_empty() && !prop.contains(' ') {
                            out.push((
                                selectors.last().cloned().unwrap_or_default(),
                                prop.to_string(),
                                value.trim().to_string(),
                            ));
                        }
                    }
                    seg.clear();
                    if c == '}' {
                        selectors.pop();
                    }
                }
                _ => seg.push(c),
            }
        }
        out
    }

    /// Family CSS this lint owns (vendored tokens, theme and catalog excluded).
    fn family_css() -> impl Iterator<Item = (&'static str, &'static str)> {
        FILES
            .iter()
            .copied()
            .filter(|(name, _)| !matches!(*name, "theme" | "catalog"))
            .chain([("base", BASE_CSS)])
    }

    /// Physical sides that stay physical, as `(file, property, value)`.
    /// Each has a `/* physical: … */` comment next to it in the CSS.
    const PHYSICAL_EXCEPTIONS: &[(&str, &str, &str)] = &[
        // Centred with a -50% translate (translate has no logical form).
        ("comparison-slider", "left", "50%"),
        ("lightbox", "left", "50%"),
        ("hover-card", "left", "50%"),
        ("resizable", "left", "50%"),
        ("orbit", "left", "50%"),
        ("sheet", "left", "50%"),
        ("shared", "left", "50%"),
        ("tooltip", "left", "anchor(center)"),
        // anchor() edges: the anchor's physical box, right in both directions.
        ("code-tabs", "left", "anchor(--code-tabs-active left)"),
        ("shared", "left", "anchor(left)"),
        // React's API side is physical: Sheet side="right", badge `-right-1`.
        ("sheet", "right", "0"),
        ("sheet", "border-left-width", "1px"),
        ("notification-center", "right", "-0.25rem"),
    ];

    #[test]
    fn family_css_uses_logical_sides() {
        const PHYSICAL: &[&str] = &[
            "margin-left",
            "margin-right",
            "padding-left",
            "padding-right",
            "left",
            "right",
            "border-left",
            "border-right",
            "border-left-width",
            "border-right-width",
            "border-left-color",
            "border-right-color",
            "border-left-style",
            "border-right-style",
        ];
        let mut bad = Vec::new();
        for (file, css) in family_css() {
            for (selector, prop, value) in declarations(css) {
                let physical = PHYSICAL.contains(&prop.as_str())
                    || (prop == "text-align" && (value == "left" || value == "right"));
                if physical
                    && !PHYSICAL_EXCEPTIONS
                        .iter()
                        .any(|(f, p, v)| *f == file && *p == prop && *v == value)
                {
                    bad.push(format!("{file}.css `{selector}` {prop}: {value}"));
                }
            }
        }
        assert!(
            bad.is_empty(),
            "use logical properties (margin-inline-start, inset-inline-end, border-inline-start, text-align: start…) or document a PHYSICAL_EXCEPTIONS entry:\n{}",
            bad.join("\n")
        );
    }

    /// Fixed widths the React component itself has by default (not a fixture
    /// `className`), as `(file, selector substring, value)`.
    const REACT_DEFAULT_WIDTHS: &[(&str, &str, &str)] = &[
        ("popover", "popover-content", "18rem"), // w-72
        ("morphing-popover", "morphing-popover-content", "18rem"), // w-72
        ("kanban", "kanban-column", "18rem"),    // w-72
        ("hover-card", "hover-card-content", "16rem"), // w-64
        ("notification-center", "notification-center", "20rem"), // w-80
        ("date-picker", "date-picker-trigger", "15rem"), // w-[240px]
        ("time-picker", "time-picker", "15rem"), // w-[240px]
        ("date-range-picker", "date-range-picker-trigger", "18.75rem"), // w-[300px]
        ("sidebar", "sidebar", "16rem"),         // --sidebar-width
        // Ring geometry needs a square box; override with --cui-orbit-size.
        ("orbit", "orbit", "var(--cui-orbit-size, 18rem)"),
        ("orbit", "orbit", "16rem"),
        ("inline-citation", "inline-citation-card-body", "20rem"), // w-80
    ];

    fn length_rem(token: &str) -> Option<f64> {
        let token = token.trim_matches(|c: char| c == '(' || c == ')' || c == ',');
        if let Some(n) = token.strip_suffix("rem") {
            n.parse().ok()
        } else if let Some(n) = token.strip_suffix("px") {
            n.parse::<f64>().ok().map(|px| px / 16.0)
        } else {
            None
        }
    }

    /// Fixture widths (`w-72` = 18rem…) live in audit.css, scoped to the audit
    /// canvas; family CSS is fluid unless React's component has the width.
    #[test]
    fn fixed_widths_only_in_audit_css_or_react_defaults() {
        let mut bad = Vec::new();
        for (file, css) in family_css() {
            for (selector, prop, value) in declarations(css) {
                if !matches!(prop.as_str(), "width" | "inline-size") {
                    continue;
                }
                let fixed = value
                    .split_whitespace()
                    .filter_map(length_rem)
                    .any(|rem| rem >= 10.0);
                let allowed = selector.starts_with("[data-audit-canvas]")
                    || REACT_DEFAULT_WIDTHS
                        .iter()
                        .any(|(f, s, v)| *f == file && selector.contains(s) && *v == value);
                if fixed && !allowed {
                    bad.push(format!("{file}.css `{selector}` {prop}: {value}"));
                }
            }
        }
        assert!(
            bad.is_empty(),
            "fixed widths ≥ 10rem: make the family fluid (`width: var(--cui-<family>-w, 100%)`) and set the fixture width in audit.css, or add a REACT_DEFAULT_WIDTHS entry:\n{}",
            bad.join("\n")
        );
        assert!(AUDIT_CSS.contains("--cui-carousel-w: 18rem;"));
        assert!(declarations(AUDIT_CSS)
            .iter()
            .filter(|(_, p, _)| p.starts_with("--cui-"))
            .all(|(s, _, _)| s == "[data-audit-canvas]"));
    }

    #[test]
    fn declarations_parser_sees_nested_rules() {
        let d = declarations(
            "/* left: 1px */ a:hover { left: 0; } @media (min-width: 40rem) { b { width: 18rem } }",
        );
        assert_eq!(
            d,
            vec![
                ("a:hover".into(), "left".into(), "0".into()),
                ("b".into(), "width".into(), "18rem".into()),
            ]
        );
    }

    fn manifest_lines() -> Vec<&'static str> {
        MANIFEST
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .collect()
    }

    #[test]
    fn manifest_lists_every_original_block_exactly_once() {
        let lines: Vec<&str> = manifest_lines()
            .into_iter()
            .filter(|l| !l.starts_with("new "))
            .collect();
        assert_eq!(lines.len(), ORIGINAL_BLOCKS);
        let mut seen = vec![false; ORIGINAL_BLOCKS];
        for line in &lines {
            let idx: usize = line.split_whitespace().nth(1).unwrap().parse().unwrap();
            assert!(!seen[idx], "block {idx} listed twice");
            seen[idx] = true;
        }
        assert!(seen.iter().all(|s| *s));
        let changes: Vec<&str> = lines
            .iter()
            .copied()
            .filter(|l| !l.starts_with("keep "))
            .collect();
        assert_eq!(changes, INTENTIONAL_CHANGES);
    }

    /// Each kept block is in its file byte-for-byte (modulo comments and
    /// whitespace), in MANIFEST order, and the files hold nothing else.
    #[test]
    fn css_files_match_manifest_hashes() {
        let mut queues: HashMap<&str, std::vec::IntoIter<&'static str>> = FILES
            .iter()
            .map(|(name, css)| (*name, split_blocks(css).into_iter()))
            .collect();
        for line in manifest_lines() {
            let f: Vec<&str> = line.split_whitespace().collect();
            let (file, hash) = match f[0] {
                "keep" => (f[2], f[3]),
                "edit" => (f[2], f[4]),
                "new" => (f[1], f[2]),
                "drop" => continue,
                other => panic!("unknown MANIFEST verb {other}"),
            };
            let block = queues
                .get_mut(file)
                .and_then(Iterator::next)
                .unwrap_or_else(|| panic!("{file}.css ran out of blocks at {line}"));
            assert_eq!(
                format!("{:016x}", fnv64(&normalize(block))),
                hash,
                "{line}\n{block}"
            );
        }
        for (file, rest) in queues {
            let extra: Vec<String> = rest
                .map(|b| format!("new {file} {:016x} -", fnv64(&normalize(b))))
                .collect();
            assert!(
                extra.is_empty(),
                "{file}.css has blocks not in MANIFEST; add at their cascade position:\n{}",
                extra.join("\n")
            );
        }
    }

    #[test]
    fn every_css_file_is_a_family_or_known() {
        for (name, _) in FILES {
            assert!(
                crate::cronus_ui_widgets::FAMILIES.contains(name) || SPECIAL_FILES.contains(name),
                "{name}.css is neither a family nor a known shared file"
            );
        }
        for (name, css) in FILES {
            assert_eq!(
                css.matches('{').count(),
                css.matches('}').count(),
                "{name}.css braces"
            );
        }
    }

    fn rs_sources(dir: &std::path::Path, out: &mut String) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                rs_sources(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs")
                && !path.ends_with("cronus_ui_css.rs")
                && !path.ends_with("cronus_ui.rs")
            {
                out.push_str(&std::fs::read_to_string(&path).unwrap());
            }
        }
    }

    /// Dead-selector gate: every exact `[data-slot="x"]` in the stylesheet is
    /// emitted by some Rust source (renderer markup, raw or escaped).
    #[test]
    fn css_slots_are_emitted_by_renderers() {
        let mut sources = String::new();
        rs_sources(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
            &mut sources,
        );
        let mut missing = BTreeSet::new();
        for (_, css) in FILES {
            let mut rest = *css;
            while let Some(p) = rest.find("[data-slot=\"") {
                rest = &rest[p + 12..];
                let end = rest.find('"').unwrap();
                let slot = &rest[..end];
                let emitted = sources.contains(&format!("data-slot=\\\"{slot}\\\""))
                    || sources.contains(&format!("data-slot=\"{slot}\""))
                    // Family renderers emit `data-slot=\"{family}\"`.
                    || crate::cronus_ui_widgets::FAMILIES.contains(&slot);
                if !emitted {
                    missing.insert(slot.to_string());
                }
            }
        }
        assert!(
            missing.is_empty(),
            "CSS for slots no renderer emits: {missing:?}"
        );
        // Once dead (dropped above), now emitted by the native-popover combobox.
        for live in ["combobox-content", "combobox-item"] {
            assert!(
                full_components().contains(&format!("[data-slot=\"{live}\"]")),
                "{live}"
            );
        }
    }

    #[test]
    fn slot_families_use_longest_prefix_and_aliases() {
        assert_eq!(slot_families("button"), &["button"]);
        assert_eq!(slot_families("input-group-addon"), &["input-group"]);
        assert_eq!(slot_families("input"), &["input"]);
        assert_eq!(slot_families("notification-row"), &["notification-center"]);
        assert_eq!(slot_families("toaster"), &["sonner", "toast"]);
        assert_eq!(slot_families("catalog-grid"), &["catalog"]);
        assert!(slot_families("nope").is_empty());
    }

    #[test]
    fn page_without_slots_gets_tokens_only() {
        let css = page_stylesheet("legacy", "<p>ok</p>", false);
        assert!(css.contains("--cronus-primary: var(--primary"));
        assert!(!css.contains("[data-slot"));
        assert!(!css.contains("[popover]"));
        assert!(!css.contains("catalog"));
        let named = page_stylesheet("aurora", "<p>ok</p>", true);
        assert!(named.starts_with(LAYER_ORDER));
        assert!(named.contains(":root, [data-cronus-theme=\"aurora\"]"));
        assert!(named.contains("html[data-cronus-theme] body"));
        // Vendored tokens carry `[data-cronus-look] [data-slot=…]` looks; no
        // base or component CSS beyond that.
        assert!(!named.contains("@layer cronus.components"));
        assert!(!named.contains("@layer cronus.base"));
        assert!(!named.contains("[popover]"));
    }

    const NAMED_PRESETS: &[&str] = &["aurora", "neutral", "midnight", "sunset", "emerald"];

    #[test]
    fn system_mode_ships_other_mode_tokens_under_media_query() {
        for preset in NAMED_PRESETS {
            let dark = page_stylesheet_for(preset, "<p>ok</p>", true, "dark");
            let system = page_stylesheet_for(preset, "<p>ok</p>", true, "system");
            let rule = format!("[data-cronus-theme=\"{preset}\"][data-cronus-mode=\"system\"] {{");
            assert!(system.contains(&rule), "{preset}: missing system rule");
            assert!(!dark.contains(&rule));
            // Only the delta block is added, not a second copy of the tokens.
            let added = system.len() - dark.len();
            assert!(added > 500 && added < 3000, "{preset}: added {added} bytes");
            // The system rule sits inside the media query, inside the tokens layer.
            let media = system.find("@media (prefers-color-scheme: ").unwrap();
            assert!(system.find(&rule).unwrap() > media);
            assert!(media > system.find("@layer cronus.tokens {").unwrap());
        }
        let aurora = page_stylesheet_for("aurora", "", true, "system");
        let block = &aurora[aurora.find("@media (prefers-color-scheme: light)").unwrap()..];
        assert!(block.contains("color-scheme: light;"));
        assert!(block.contains("--cronus-surface-base: oklch(1 0 0);"));
        let neutral = page_stylesheet_for("neutral", "", true, "system");
        let block = &neutral[neutral.find("@media (prefers-color-scheme: dark)").unwrap()..];
        assert!(block.contains("color-scheme: dark;"));
        assert!(block.contains("--cronus-surface-base: oklch(0.11 0 0);"));
    }

    #[test]
    fn light_and_dark_modes_do_not_change_page_css() {
        for preset in ["aurora", "neutral", "legacy"] {
            let html = "<button data-slot=\"button\">Save</button>";
            let base = page_stylesheet(preset, html, true);
            assert_eq!(page_stylesheet_for(preset, html, true, "dark"), base);
            assert_eq!(page_stylesheet_for(preset, html, true, "light"), base);
            assert!(!base.contains("prefers-color-scheme"));
        }
        // Legacy pages get no vendored tokens, so `system` adds nothing either.
        assert_eq!(
            page_stylesheet_for("legacy", "<p></p>", false, "system"),
            page_stylesheet("legacy", "<p></p>", false)
        );
        assert_eq!(
            crate::cronus_ui::token_css("aurora", "light"),
            full_stylesheet("aurora")
        );
        assert!(crate::cronus_ui::token_css("aurora", "system").contains("mode=\"system\""));
    }

    #[test]
    fn audit_stylesheet_has_no_system_mode_css() {
        let css = audit_stylesheet("<button data-slot=\"button\">Save</button>");
        assert!(!css.contains("prefers-color-scheme: light"));
        assert!(!css.contains("data-cronus-mode=\"system\""));
        assert!(!full_audit_stylesheet().contains("data-cronus-mode=\"system\""));
    }

    fn components_layer(css: &str) -> &str {
        &css[css
            .find("@layer cronus.components {")
            .expect("components layer")..]
    }

    #[test]
    fn one_button_page_ships_button_only() {
        let html = crate::cronus_ui::button("Save", "primary", "md", None);
        let css = page_stylesheet("aurora", &html, true);
        assert!(css.starts_with(LAYER_ORDER));
        assert!(css.contains("@layer cronus.components {"));
        assert!(
            css.contains("[data-slot=\"button\"]:disabled { opacity: 0.5; pointer-events: none; }")
        );
        assert!(css.contains("[data-slot]:focus-visible"));
        assert!(css.contains("[popover]"));
        let components = components_layer(&css);
        assert!(!components.contains("[data-slot=\"card\"]"));
        assert!(!components.contains("catalog"));
        assert!(!css.contains(AUDIT_CSS));
        assert!(css.len() * 5 < full_stylesheet("aurora").len());
    }

    #[test]
    fn subset_keeps_original_cascade_order() {
        let html = "<div data-slot=\"card\"><button data-slot=\"button\"></button><div data-slot=\"dialog-content\"></div></div>";
        let css = page_stylesheet("legacy", html, false);
        let full = full_components();
        let mut last = 0;
        for block in split_blocks(Box::leak(css.into_boxed_str())) {
            if let Some(pos) = full[last..].find(block) {
                last += pos + block.len();
            } else {
                assert!(
                    FALLBACK_CSS.contains(block) || BASE_CSS.contains(block),
                    "out of order or unknown: {block}"
                );
            }
        }
    }

    #[test]
    fn shared_blocks_follow_each_owner() {
        let css = page_stylesheet(
            "legacy",
            "<div data-slot=\"usage-meter-fill\"></div>",
            false,
        );
        assert!(css.contains("--cui-progress-value: 37;"));
        let css = page_stylesheet("legacy", "<div data-slot=\"toaster\"></div>", false);
        assert!(css.contains("[data-slot=\"toaster\"]"));
    }

    #[test]
    fn catalog_css_only_with_catalog_markup() {
        let plain = page_stylesheet("aurora", "<div data-slot=\"card\"></div>", true);
        assert!(!plain.contains("catalog-"));
        let kit = page_stylesheet("aurora", "<div data-slot=\"catalog\"></div>", true);
        assert!(kit.contains("[data-slot=\"catalog-specimen\"]"));
    }

    #[test]
    fn audit_stylesheet_is_self_contained() {
        let css = audit_stylesheet("<button data-slot=\"button\">Save</button>");
        assert!(css.starts_with(LAYER_ORDER));
        let tokens = css.find("@layer cronus.tokens {").unwrap();
        assert!(css[tokens..].contains(AUDIT_CSS));
        assert!(css.find(AUDIT_CSS).unwrap() < css.find(FALLBACK_CSS).unwrap());
        assert!(css.contains("[data-cronus-theme=\"aurora\"]"));
        assert!(!css.contains(":root, [data-cronus-theme=\"aurora\"]"));
        let components = components_layer(&css);
        assert!(components.contains("[data-slot=\"button\"]"));
        assert!(!components.contains("catalog-"));
        assert!(!components.contains("[data-slot=\"card\"]"));
    }

    #[test]
    fn noted_family_is_taken_once() {
        note_family("card");
        let css = page_stylesheet("legacy", "<p></p>", false);
        assert!(css.contains("[data-slot=\"card\"]"));
        let again = page_stylesheet("legacy", "<p></p>", false);
        assert!(!again.contains("[data-slot=\"card\"]"));
    }

    #[test]
    fn reduced_motion_covers_descendants_and_focus_ring_is_focus_only() {
        assert!(BASE_CSS
            .contains("[data-slot]:not([data-force-motion]):not([data-force-motion] *) *::before"));
        assert!(BASE_CSS.contains("animation-iteration-count: 1 !important;"));
        // Same subtree opt-out as tokens.css: a `data-force-motion` frame keeps motion.
        assert!(BASE_CSS.contains("[data-slot]:not([data-force-motion]):not([data-force-motion] *) { animation: none !important;"));
        assert!(TOKENS_CSS.contains("--cronus-ring:"));
        // Every outline in base is behind :focus-visible (idle render unchanged).
        let before_media = &BASE_CSS[..BASE_CSS.find("@media").unwrap()];
        for rule in split_blocks(Box::leak(before_media.to_string().into_boxed_str())) {
            if rule.contains("outline") {
                assert!(rule.contains(":focus-visible"), "{rule}");
            }
        }
    }

    #[test]
    fn css_meta_is_keyed_by_content() {
        let a = css_meta("a{}");
        assert!(a.starts_with("<meta name=\"cronus-ui-css\" content=\""));
        assert_eq!(a, css_meta("a{}"));
        assert_ne!(a, css_meta("b{}"));
    }
}
