//! Light/system colours for the kernel's section renderers and full-page
//! dashboards.
//!
//! Those renderers paint a fixed dark palette (inline `style=""` hex/rgba,
//! `<style>` blocks, a few Tailwind classes). Instead of forking every
//! renderer, their output passes through [`adapt`] at the render boundary:
//!
//! - `dark` (and unknown themes): the HTML is returned untouched, so dark
//!   output is byte-identical.
//! - `light`: each dark palette literal becomes its light counterpart.
//! - `system`: renderers take their dark branch ([`render_theme`]) and each
//!   literal becomes `light-dark(<light>, <dark>)`, resolved by the layout's
//!   `color-scheme: light dark` (dashboards get it from [`adapt_document`]).
//!
//! This is the same literal / `light-dark()` scheme `layout.rs` uses for
//! legacy (non-preset) pages. Only colour *values* in CSS contexts are
//! rewritten: `style` attributes, `<style>` declarations, SVG paint
//! attributes, `this.style.*='…'` in inline handlers, and dark text/border/bg
//! classes on non-interactive elements (as an added inline declaration).
//! Text content, scripts and cronus-ui token output (`var(--cronus-*)`) are
//! never touched. `#fff` text is only remapped on non-interactive elements:
//! white text on accent buttons/links stays white.

use crate::cronus_ui::normalize_mode;

/// Theme the renderers should branch on: `system` renders the dark branch
/// (then [`adapt`] makes it follow the OS); everything else is unchanged.
pub(crate) fn render_theme(theme: &str) -> &str {
    if normalize_mode(theme) == "system" {
        "dark"
    } else {
        theme
    }
}

/// Map a renderer's HTML to the page's colour mode (see module docs).
pub(crate) fn adapt(html: String, theme: &str) -> String {
    let mode = normalize_mode(theme);
    if mode == "dark" {
        return html;
    }
    rewrite_html(&html, mode)
}

/// [`adapt`] for a full HTML document: also declares `color-scheme` (needed by
/// `light-dark()`) and swaps `<html class="dark">` for `light` in light mode.
pub(crate) fn adapt_document(html: String, theme: &str) -> String {
    let mode = normalize_mode(theme);
    if mode == "dark" {
        return html;
    }
    let mut out = rewrite_html(&html, mode);
    if mode == "light" {
        out = out.replacen("<html class=\"dark\"", "<html class=\"light\"", 1);
    }
    let scheme = if mode == "light" {
        "light"
    } else {
        "light dark"
    };
    if let Some(pos) = out.find("<head>") {
        out.insert_str(
            pos + "<head>".len(),
            &format!(
                "\n  <meta name=\"color-scheme\" content=\"{scheme}\">\n  <style>:root{{color-scheme:{scheme}}}</style>"
            ),
        );
    }
    out
}

fn mode_value(mode: &str, light: &str, dark: &str) -> String {
    if mode == "system" {
        format!("light-dark({light}, {dark})")
    } else {
        light.to_string()
    }
}

// ── HTML scanning ────────────────────────────────────────────────

fn rewrite_html(html: &str, mode: &str) -> String {
    let mut out = String::with_capacity(html.len() + html.len() / 16);
    let mut rest = html;
    while let Some(lt) = rest.find('<') {
        out.push_str(&rest[..lt]);
        rest = &rest[lt..];
        let lower_head: String = rest
            .chars()
            .take(8)
            .collect::<String>()
            .to_ascii_lowercase();
        if rest.starts_with("<!--") {
            let end = rest.find("-->").map(|e| e + 3).unwrap_or(rest.len());
            out.push_str(&rest[..end]);
            rest = &rest[end..];
            continue;
        }
        let Some(tag_end) = tag_end(rest) else {
            out.push_str(rest);
            return out;
        };
        let tag = &rest[..tag_end];
        if lower_head.starts_with("<script") {
            let end = find_ci(rest, "</script>")
                .map(|e| e + "</script>".len())
                .unwrap_or(rest.len());
            out.push_str(&rest[..end]);
            rest = &rest[end..];
            continue;
        }
        if lower_head.starts_with("<style") {
            out.push_str(&rewrite_tag(tag, mode));
            let body = &rest[tag_end..];
            let close = find_ci(body, "</style>").unwrap_or(body.len());
            out.push_str(&rewrite_css(&body[..close], mode));
            rest = &body[close..];
            continue;
        }
        if rest[1..].starts_with(|c: char| c.is_ascii_alphabetic()) {
            out.push_str(&rewrite_tag(tag, mode));
        } else {
            out.push_str(tag);
        }
        rest = &rest[tag_end..];
    }
    out.push_str(rest);
    out
}

fn find_ci(hay: &str, needle: &str) -> Option<usize> {
    hay.to_ascii_lowercase().find(needle)
}

/// Byte index just past the `>` closing the tag at the start of `s`,
/// honouring quoted attribute values.
fn tag_end(s: &str) -> Option<usize> {
    let mut quote: Option<u8> = None;
    for (i, b) in s.bytes().enumerate().skip(1) {
        match quote {
            Some(q) if b == q => quote = None,
            Some(_) => {}
            None if b == b'"' || b == b'\'' => quote = Some(b),
            None if b == b'>' => return Some(i + 1),
            None => {}
        }
    }
    None
}

struct Attr<'a> {
    /// Text before the value (whitespace, name, `=`, opening quote).
    lead: &'a str,
    name: String,
    value: Option<String>,
    /// Closing quote (empty when unquoted / no value).
    close: &'a str,
}

const INTERACTIVE: &[&str] = &["a", "button", "input", "select", "textarea", "option"];

fn rewrite_tag(tag: &str, mode: &str) -> String {
    let name_end = tag[1..]
        .find(|c: char| c.is_ascii_whitespace() || c == '>' || c == '/')
        .map(|i| i + 1)
        .unwrap_or(tag.len());
    let tag_name = tag[1..name_end].to_ascii_lowercase();
    if name_end >= tag.len() - 1 {
        return tag.to_string();
    }
    let interactive = INTERACTIVE.contains(&tag_name.as_str());

    // Parse attributes.
    let bytes = tag.as_bytes();
    let mut attrs: Vec<Attr> = Vec::new();
    let mut i = name_end;
    let body_end = tag.len() - 1; // the '>'
    let tail_start;
    loop {
        let start = i;
        while i < body_end && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= body_end || bytes[i] == b'/' {
            tail_start = start;
            break;
        }
        let n0 = i;
        while i < body_end
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'='
            && bytes[i] != b'/'
        {
            i += 1;
        }
        let name = tag[n0..i].to_ascii_lowercase();
        if i < body_end && bytes[i] == b'=' {
            i += 1;
            if i < body_end && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let q = bytes[i];
                let v0 = i + 1;
                let v1 = tag[v0..body_end]
                    .bytes()
                    .position(|b| b == q)
                    .map(|p| v0 + p)
                    .unwrap_or(body_end);
                attrs.push(Attr {
                    lead: &tag[start..v0],
                    name,
                    value: Some(tag[v0..v1].to_string()),
                    close: &tag[v1..(v1 + 1).min(body_end)],
                });
                i = (v1 + 1).min(body_end);
            } else {
                let v0 = i;
                while i < body_end && !bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                attrs.push(Attr {
                    lead: &tag[start..v0],
                    name,
                    value: Some(tag[v0..i].to_string()),
                    close: "",
                });
            }
        } else {
            attrs.push(Attr {
                lead: &tag[start..i],
                name,
                value: None,
                close: "",
            });
        }
        if i == start {
            tail_start = i;
            break;
        }
    }

    let mut changed = false;
    let class_decls: Vec<(&'static str, String)> = if interactive {
        Vec::new()
    } else {
        attrs
            .iter()
            .find(|a| a.name == "class")
            .and_then(|a| a.value.as_deref())
            .map(|c| class_declarations(c, mode))
            .unwrap_or_default()
    };
    let mut has_style = false;
    for a in attrs.iter_mut() {
        let Some(v) = a.value.as_ref() else { continue };
        let new_v = match a.name.as_str() {
            "style" => {
                has_style = true;
                let mut s = rewrite_decls(v, mode, interactive);
                append_missing(&mut s, &class_decls);
                s
            }
            "stroke" | "fill" | "stop-color" | "flood-color" | "lighting-color" => {
                map_value(v, mode, Ctx::Paint, interactive)
            }
            n if n.starts_with("on") => rewrite_handler(v, mode, interactive),
            _ => continue,
        };
        if &new_v != v {
            a.value = Some(new_v);
            changed = true;
        }
    }
    if !changed && (class_decls.is_empty() || has_style) {
        return tag.to_string();
    }
    let mut out = String::with_capacity(tag.len() + 64);
    out.push_str(&tag[..name_end]);
    for a in &attrs {
        out.push_str(a.lead);
        if let Some(v) = &a.value {
            out.push_str(v);
        }
        out.push_str(a.close);
    }
    if !has_style && !class_decls.is_empty() {
        let mut s = String::new();
        append_missing(&mut s, &class_decls);
        out.push_str(&format!(" style=\"{s}\""));
    }
    out.push_str(&tag[tail_start..]);
    out
}

fn append_missing(style: &mut String, decls: &[(&'static str, String)]) {
    for (prop, value) in decls {
        let present = style.split(';').any(|d| {
            d.split_once(':')
                .map(|(p, _)| p.trim().eq_ignore_ascii_case(prop))
                .unwrap_or(false)
        });
        if !present {
            let trimmed = style.trim_end();
            if !trimmed.is_empty() && !trimmed.ends_with(';') {
                style.push(';');
            }
            style.push_str(&format!("{prop}:{value}"));
        }
    }
}

/// `this.style.background='#1f1f1f'` in inline handlers (hover restore).
fn rewrite_handler(v: &str, mode: &str, interactive: bool) -> String {
    let mut out = String::with_capacity(v.len());
    let mut rest = v;
    while let Some(p) = rest.find("style.") {
        let after = &rest[p + 6..];
        let prop_len = after
            .find(|c: char| !c.is_ascii_alphanumeric())
            .unwrap_or(after.len());
        let prop = &after[..prop_len];
        let tail = &after[prop_len..];
        let eq = tail.find(|c: char| !c.is_ascii_whitespace() && c != '=');
        let quote_ok = eq.map(|e| tail[..e].contains('=') && tail[e..].starts_with('\''));
        if quote_ok == Some(true) {
            let e = eq.unwrap();
            let v0 = e + 1;
            if let Some(v1) = tail[v0..].find('\'') {
                let ctx = if prop.eq_ignore_ascii_case("color") {
                    Ctx::Text
                } else {
                    Ctx::Paint
                };
                out.push_str(&rest[..p + 6 + prop_len]);
                out.push_str(&tail[..v0]);
                out.push_str(&map_value(&tail[v0..v0 + v1], mode, ctx, interactive));
                rest = &tail[v0 + v1..];
                continue;
            }
        }
        out.push_str(&rest[..p + 6]);
        rest = after;
    }
    out.push_str(rest);
    out
}

fn is_prop_name(p: &str) -> bool {
    !p.is_empty() && p.bytes().all(|b| b.is_ascii_lowercase() || b == b'-')
}

fn ctx_for(prop: &str) -> Ctx {
    match prop {
        "color" | "-webkit-text-fill-color" | "caret-color" | "text-decoration-color" => Ctx::Text,
        _ => Ctx::Paint,
    }
}

/// `prop:value;prop:value` (inline style).
fn rewrite_decls(s: &str, mode: &str, interactive: bool) -> String {
    let mut out = String::with_capacity(s.len());
    for (idx, decl) in s.split(';').enumerate() {
        if idx > 0 {
            out.push(';');
        }
        match decl.split_once(':') {
            Some((p, v)) if is_prop_name(&p.trim().to_ascii_lowercase()) => {
                let prop = p.trim().to_ascii_lowercase();
                out.push_str(p);
                out.push(':');
                out.push_str(&map_value(v, mode, ctx_for(&prop), interactive));
            }
            _ => out.push_str(decl),
        }
    }
    out
}

/// Declarations inside a `<style>` block. Selectors are never rewritten.
fn rewrite_css(css: &str, mode: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut seg_start = 0;
    let mut in_block = false; // last delimiter was `{` or `;`
    for (i, c) in css.char_indices() {
        if c == '{' || c == '}' || c == ';' {
            let seg = &css[seg_start..i];
            if in_block {
                out.push_str(&rewrite_css_decl(seg, mode));
            } else {
                out.push_str(seg);
            }
            out.push(c);
            seg_start = i + 1;
            in_block = c != '}';
        }
    }
    let seg = &css[seg_start..];
    if in_block {
        out.push_str(&rewrite_css_decl(seg, mode));
    } else {
        out.push_str(seg);
    }
    out
}

fn rewrite_css_decl(seg: &str, mode: &str) -> String {
    match seg.split_once(':') {
        Some((p, v)) if is_prop_name(p.trim()) => {
            // `#fff` text inside a stylesheet may belong to a button: keep it.
            format!("{p}:{}", map_value(v, mode, ctx_for(p.trim()), true))
        }
        _ => seg.to_string(),
    }
}

// ── Colour literals ──────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Ctx {
    Text,
    Paint,
}

/// Replace mapped colour literals in a CSS value. Text already inside
/// `light-dark(…)` is left alone, so adapting twice is a no-op.
fn map_value(v: &str, mode: &str, ctx: Ctx, interactive: bool) -> String {
    let lower = v.to_ascii_lowercase();
    // Named white text (`color:white`) on non-interactive elements.
    if ctx == Ctx::Text && !interactive && lower.trim().split(' ').next() == Some("white") {
        let trimmed = v.trim_start();
        let lead = &v[..v.len() - trimmed.len()];
        return format!(
            "{lead}{}{}",
            mode_value(mode, "#1a1c1c", "white"),
            &trimmed["white".len()..]
        );
    }
    if !lower.contains('#') && !lower.contains("rgb") {
        return v.to_string();
    }
    let bytes = v.as_bytes();
    let mut out = String::with_capacity(v.len() + 16);
    let mut i = 0;
    let mut last = 0;
    while i < bytes.len() {
        if lower[i..].starts_with("light-dark(") {
            // skip to the matching paren
            let mut depth = 0;
            let mut j = i;
            while j < bytes.len() {
                match bytes[j] {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            i = j + 1;
            continue;
        }
        let lit_end = if bytes[i] == b'#' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_hexdigit() {
                j += 1;
            }
            let boundary = j >= bytes.len() || !bytes[j].is_ascii_alphanumeric();
            if boundary && matches!(j - i - 1, 3 | 6) {
                Some(j)
            } else {
                None
            }
        } else if lower[i..].starts_with("rgb")
            && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'-')
        {
            lower[i..].find(')').map(|e| i + e + 1)
        } else {
            None
        };
        if let Some(end) = lit_end {
            let lit = &v[i..end];
            if let Some(light) = light_for(&lower[i..end], ctx, interactive) {
                out.push_str(&v[last..i]);
                out.push_str(&mode_value(mode, &light, lit));
                last = end;
            }
            i = end;
        } else {
            i += 1;
        }
    }
    out.push_str(&v[last..]);
    out
}

fn expand_hex(h: &str) -> String {
    let d = &h[1..];
    if d.len() == 3 {
        d.chars().flat_map(|c| [c, c]).collect()
    } else {
        d.to_string()
    }
}

fn parse_rgba(s: &str) -> Option<(u32, u32, u32, f64)> {
    let inner = s[s.find('(')? + 1..s.rfind(')')?].replace('/', ",");
    let parts: Vec<&str> = inner
        .split(|c: char| c == ',' || c.is_ascii_whitespace())
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() < 3 {
        return None;
    }
    let r = parts[0].parse().ok()?;
    let g = parts[1].parse().ok()?;
    let b = parts[2].parse().ok()?;
    let a = match parts.get(3) {
        Some(p) if p.ends_with('%') => p.trim_end_matches('%').parse::<f64>().ok()? / 100.0,
        Some(p) => p.parse().ok()?,
        None => 1.0,
    };
    Some((r, g, b, a))
}

/// Light text for a translucent light-on-dark text colour, by opacity.
fn muted_text(a: f64) -> &'static str {
    if a >= 0.9 {
        "#1a1c1c"
    } else if a >= 0.5 {
        "#52525b"
    } else {
        "#71717a"
    }
}

fn alpha(a: f64) -> String {
    let s = format!("{a:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Light counterpart of a dark-palette literal (lowercase), if it is one.
fn light_for(lit: &str, ctx: Ctx, interactive: bool) -> Option<String> {
    if lit.starts_with('#') {
        let hex = expand_hex(lit);
        let v = match ctx {
            Ctx::Paint => match hex.as_str() {
                "050505" | "080808" | "09090b" | "0a0a0a" | "0e0e0e" | "0f0f0f" | "111111"
                | "131313" => "#f9f9f9",
                "171717" | "18181b" | "1a1a1a" | "1b1b1b" | "1b1c1c" | "1c1b1b" | "1f1f1f" => {
                    "#ffffff"
                }
                "202020" | "222222" | "262626" | "27272a" | "2a2a2a" | "2f3131" => "#f3f3f3",
                "353535" | "393939" | "3f3f46" | "404040" | "474747" => "#e5e5e5",
                _ => return None,
            },
            Ctx::Text => match hex.as_str() {
                "e2e2e2" | "e5e2e1" | "e4e4e7" | "f4f4f5" | "fafafa" | "f9fafb" | "fafaf9" => {
                    "#1a1c1c"
                }
                "d4d4d8" | "d4d4d4" | "c6c6c6" | "cfc4c5" | "c0c0c0" => "#3f3f46",
                "ffffff" if !interactive => "#1a1c1c",
                _ => return None,
            },
        };
        return Some(v.to_string());
    }
    let (r, g, b, a) = parse_rgba(lit)?;
    match ctx {
        Ctx::Paint => {
            if (r, g, b) == (76, 69, 70)
                || ((r, g, b) == (255, 255, 255) || (r, g, b) == (226, 226, 226)) && a <= 0.12
            {
                Some(format!("rgba(0,0,0,{})", alpha(a)))
            } else if r == g && g == b && (10..=40).contains(&r) {
                Some(format!("rgba(250,250,250,{})", alpha(a)))
            } else {
                None
            }
        }
        Ctx::Text => {
            let light_on_dark = matches!(
                (r, g, b),
                (226, 226, 226) | (207, 196, 197) | (250, 250, 250) | (255, 255, 255)
            );
            if !light_on_dark || (a >= 1.0 && interactive) {
                None
            } else {
                Some(muted_text(a).to_string())
            }
        }
    }
}

/// Inline declarations replacing dark-only Tailwind classes (non-interactive
/// elements only). Hover/state variants are left alone.
fn class_declarations(class: &str, mode: &str) -> Vec<(&'static str, String)> {
    let mut decls: Vec<(&'static str, String)> = Vec::new();
    let mut push = |prop: &'static str, light: &str, dark: &str| {
        if !decls.iter().any(|(p, _)| *p == prop) {
            decls.push((prop, mode_value(mode, light, dark)));
        }
    };
    for token in class.split_ascii_whitespace() {
        let (base, opacity) = match token.split_once('/') {
            Some((b, o)) => (b, o.parse::<f64>().ok().map(|o| o / 100.0)),
            None => (token, None),
        };
        match (base, opacity) {
            ("text-white", None) => push("color", "#1a1c1c", "#fff"),
            ("text-white", Some(a)) => push(
                "color",
                muted_text(a),
                &format!("rgba(255,255,255,{})", alpha(a)),
            ),
            ("text-neutral-200", None) => push("color", "#3f3f46", "#e5e5e5"),
            ("text-zinc-200", None) => push("color", "#3f3f46", "#e4e4e7"),
            ("text-neutral-300", None) => push("color", "#3f3f46", "#d4d4d4"),
            ("text-zinc-300", None) => push("color", "#3f3f46", "#d4d4d8"),
            ("border-white", Some(a)) if a <= 0.2 => push(
                "border-color",
                &format!("rgba(0,0,0,{})", alpha(a.max(0.08))),
                &format!("rgba(255,255,255,{})", alpha(a)),
            ),
            ("border-neutral-800", None) => push("border-color", "#e5e5e5", "#262626"),
            ("border-neutral-800", Some(a)) => push(
                "border-color",
                "#e5e5e5",
                &format!("rgb(38 38 38 / {})", alpha(a)),
            ),
            ("border-zinc-800", None) => push("border-color", "#e5e5e5", "#27272a"),
            ("bg-white", Some(a)) if a <= 0.12 => push(
                "background-color",
                &format!("rgba(0,0,0,{})", alpha(a)),
                &format!("rgba(255,255,255,{})", alpha(a)),
            ),
            ("bg-zinc-900", None) => push("background-color", "#ffffff", "#18181b"),
            ("bg-neutral-900", None) => push("background-color", "#ffffff", "#171717"),
            ("bg-zinc-950", None) => push("background-color", "#f9f9f9", "#09090b"),
            ("bg-neutral-950", None) => push("background-color", "#f9f9f9", "#0a0a0a"),
            _ => {}
        }
    }
    decls
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_is_byte_identical() {
        let html = r#"<div style="background:#1b1b1b;color:#e2e2e2" class="text-white">x</div><style>a{color:#e2e2e2}</style>"#;
        assert_eq!(adapt(html.to_string(), "dark"), html);
        assert_eq!(adapt(html.to_string(), "obsidian"), html);
        assert_eq!(adapt_document(html.to_string(), "dark"), html);
        assert_eq!(render_theme("system"), "dark");
        assert_eq!(render_theme("light"), "light");
        assert_eq!(render_theme("dark"), "dark");
    }

    #[test]
    fn light_maps_styles_css_svg_handlers_and_classes() {
        let html = concat!(
            r#"<section style="background:#1b1b1b;border:0.5px solid rgba(76,69,70,0.15)">"#,
            r#"<h3 style="color:#e2e2e2">T</h3><p style="color:rgba(226,226,226,0.4)">s</p>"#,
            r#"<h1 class="text-5xl text-white">H</h1><a class="text-white" style="color:#fff;background:#3b82f6">b</a>"#,
            r#"<span style="color:#ffffff">w</span>"#,
            r#"<svg><line stroke="rgba(226,226,226,0.05)"/></svg>"#,
            r#"<tr onmouseover="this.style.background='#1f1f1f'" onmouseout="this.style.background='#0e0e0e'">"#,
            r#"<style>body { background:#131313; color:#e2e2e2; } a:hover{color:#fff}</style>"#,
            r#"<script>var c='#1b1b1b';</script>background:#1b1b1b</section>"#,
        );
        let out = adapt(html.to_string(), "light");
        assert!(
            out.contains(r#"style="background:#ffffff;border:0.5px solid rgba(0,0,0,0.15)""#),
            "{out}"
        );
        assert!(out.contains(r#"<h3 style="color:#1a1c1c">"#), "{out}");
        assert!(out.contains(r#"<p style="color:#71717a">"#), "{out}");
        assert!(
            out.contains(r#"<h1 class="text-5xl text-white" style="color:#1a1c1c">"#),
            "{out}"
        );
        // interactive white text stays
        assert!(
            out.contains(r#"<a class="text-white" style="color:#fff;background:#3b82f6">"#),
            "{out}"
        );
        assert!(out.contains(r#"<span style="color:#1a1c1c">"#), "{out}");
        assert!(out.contains(r#"stroke="rgba(0,0,0,0.05)""#), "{out}");
        assert!(out.contains("this.style.background='#ffffff'"), "{out}");
        assert!(out.contains("this.style.background='#f9f9f9'"), "{out}");
        assert!(
            out.contains("body { background:#f9f9f9; color:#1a1c1c; } a:hover{color:#fff}"),
            "{out}"
        );
        // scripts and text content untouched
        assert!(
            out.contains("<script>var c='#1b1b1b';</script>background:#1b1b1b</section>"),
            "{out}"
        );
        // idempotent
        assert_eq!(adapt(out.clone(), "light"), out);
    }

    #[test]
    fn system_uses_light_dark_and_keeps_dark_side() {
        let html = r#"<div class="border-white/10" style="background:#1b1b1b;color:rgba(207,196,197,1)">x</div>"#;
        let out = adapt(html.to_string(), "system");
        assert_eq!(
            out,
            r#"<div class="border-white/10" style="background:light-dark(#ffffff, #1b1b1b);color:light-dark(#1a1c1c, rgba(207,196,197,1));border-color:light-dark(rgba(0,0,0,0.1), rgba(255,255,255,0.1))">x</div>"#
        );
        assert_eq!(adapt(out.clone(), "system"), out);
    }

    #[test]
    fn document_declares_color_scheme() {
        let doc = r#"<!DOCTYPE html><html class="dark" lang="en"><head><style>body{background:#131313}</style></head><body></body></html>"#;
        let light = adapt_document(doc.to_string(), "light");
        assert!(light.contains(r#"<html class="light""#));
        assert!(light.contains(r#"<meta name="color-scheme" content="light">"#));
        assert!(light.contains("body{background:#f9f9f9}"));
        let system = adapt_document(doc.to_string(), "system");
        assert!(system.contains(r#"<html class="dark""#));
        assert!(system.contains(":root{color-scheme:light dark}"));
        assert!(system.contains("background:light-dark(#f9f9f9, #131313)"));
    }
}
