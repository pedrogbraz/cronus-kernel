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
    "accordion", "alert", "alert-dialog", "animated-button", "animated-list", "animated-number",
    "app-shell", "area-chart", "aspect-ratio", "aurora-background", "autocomplete", "avatar",
    "avatar-group", "badge", "banner", "bar-chart", "border-beam", "bouncy-accordion",
    "breadcrumb", "button", "button-group", "calendar", "candlestick-chart", "card",
    "card-stack", "carousel", "catalog", "chart", "checkbox", "chip", "choropleth-chart",
    "click-spark", "code-block", "code-tabs", "collapsible", "color-picker", "combobox",
    "command", "comparison-slider", "composed-chart", "confetti", "confirmation-dialog",
    "context-menu", "copy-button", "countdown", "credit-card-input", "currency-input",
    "data-table", "date-picker", "date-range-picker", "description-list", "dialog", "dock",
    "dot-pattern", "drawer", "dropdown-menu", "dynamic-island", "empty", "expandable-tabs",
    "fab", "field", "file-dropzone", "flickering-grid", "flip-card", "floating-label-input",
    "form", "frame", "funnel-chart", "gauge-chart", "glare-hover", "glass-card",
    "gradient-border", "gradient-text", "grid-pattern", "heatmap", "heatmap-chart",
    "highlighter", "hover-card", "image-zoom", "input", "input-group", "input-otp",
    "invite-dialog", "json-viewer", "kanban", "kbd", "label", "light-rays", "lightbox",
    "line-chart", "live-line-chart", "logo-carousel", "magnetic", "marquee", "masonry",
    "menubar", "metric", "mode-toggle", "morphing-popover", "motion-presets", "multi-select",
    "navigation-menu", "noise", "notification-center", "number-input", "orbit", "pagination",
    "particles", "password-input", "phone-input", "pie-chart", "pill-nav", "popover",
    "profit-loss-chart", "progress", "progressive-blur", "radar-chart", "radio-group", "rating",
    "resizable", "retro-grid", "reveal", "rich-text-editor", "ring-chart", "ripple",
    "scatter-chart", "scheduler", "scramble-text", "scroll-area", "scroll-progress",
    "segmented-control", "select", "separator", "shared", "sheet", "shimmer", "shiny-text",
    "sidebar", "signature-pad", "skeleton", "slider", "sparkles-text", "sparkline", "spinner",
    "spinning-text", "split-button", "spotlight-card", "star-border", "status-dot", "stepper",
    "sunburst-chart", "switch", "table", "table-of-contents", "tabs", "tags-input", "terminal",
    "text-effect", "text-shimmer", "textarea", "theme", "tilt-card", "time-picker", "timeline",
    "toast", "toggle", "toggle-group", "toolbar", "tooltip", "tree-view", "typing-text",
    "usage-meter", "video-player", "word-rotate", "workspace-switcher",
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
    assemble(
        &tokens_part(preset, mode, audit),
        base,
        &components_part(usage, false),
        layered,
    )
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
        "drop 0080 dd57e9b1ef15bbdd duplicate-of 0630",
        "drop 0531 10570e9feebc0200 dead-slot combobox-content",
        "drop 0532 19d506c62fe39e86 dead-slot combobox-item",
        "drop 0533 e4b51a5bc215dfbe dead-slot combobox-item",
    ];

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
                    // meteors/sankey-chart stubs emit `data-slot=\"{family}\"`.
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
        for dead in ["combobox-content", "combobox-item"] {
            assert!(!full_components().contains(dead), "{dead}");
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
        assert!(BASE_CSS.contains("[data-slot] *::before"));
        assert!(BASE_CSS.contains("animation-iteration-count: 1 !important;"));
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
