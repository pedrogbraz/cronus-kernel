//! Output safety gate for every dedicated cronus-ui family renderer.
//!
//! Renders each family in `PORTED_FAMILIES` through the real dispatcher with a
//! bare stub and with hostile inputs (`javascript:` URLs in every link-ish key,
//! markup and attribute-breaking quotes in every text-ish key), then scans the
//! HTML. Dedicated renderers are zero-JS: no `<script>`, `<style>`, inline
//! `style=`, `on*=` handlers, `<canvas>`, executable URLs or unescaped markup.
//!
//! The `on*` and URL checks tokenize tags and attributes instead of grepping the
//! raw string: correctly escaped hostile text (`&lt;img … onerror=…&gt;`) would
//! otherwise read as an attribute. Executable URLs are checked on URL-bearing
//! attributes after entity decoding and stripping control characters, so
//! `java&#9;script:` is caught while the literal text "javascript:" shown in a
//! label (inert) is not.
//!
//! Also gates the source: helper copies (`fn esc(`, `fn attr(`, `fn flag(`) live
//! only in `cronus_ui_kit.rs`.

use crate::cronus_ui_kit::stub;
use crate::cronus_ui_widgets::{render, PORTED_FAMILIES};
use crate::parser::{ComponentItemNode, ComponentNode};
use std::collections::HashMap;

/// Families that still emit inline JS (`onclick=`). This list must only shrink;
/// it is empty since tabs (radio labels) and dialog (open-by-default divs) went
/// zero-JS, so every ported family is gated.
const KNOWN_JS_OFFENDERS: &[&str] = &[];

const JS_URL: &str = "javascript:alert(1)";
const XSS_TEXT: &str = "<img src=x onerror=alert(1)>\" onmouseover=\"alert(1)' onfocus='alert(1)";

/// Keys a renderer may turn into `href` / `src` / `action`.
const URL_KEYS: &[&str] = &[
    "link", "href", "src", "url", "image", "poster", "action", "avatar", "logo", "to", "source",
];

/// Keys renderers read as text / attributes (from `get("…")` / `attr(comp, "…")`).
const TEXT_KEYS: &[&str] = &[
    "aria-label",
    "aria-describedby",
    "value",
    "disabled",
    "description",
    "placeholder",
    "invalid",
    "variant",
    "checked",
    "label",
    "pressed",
    "orientation",
    "on",
    "selected",
    "max",
    "unread",
    "type",
    "prefix",
    "for",
    "htmlFor",
    "decorative",
    "currencies",
    "amount",
    "withLabel",
    "with-label",
    "unit",
    "total",
    "tone",
    "today",
    "title",
    "text",
    "target",
    "status",
    "size",
    "showSeconds",
    "show-seconds",
    "progress",
    "name",
    "multiple",
    "month",
    "mode",
    "menuLabel",
    "menu-label",
    "loading",
    "language",
    "id",
    "hourCycle",
    "hour-cycle",
    "helperText",
    "helper-text",
    "helper",
    "filename",
    "error",
    "entity",
    "drop",
    "defaultValue",
    "default-value",
    "defaultCurrency",
    "default-currency",
    "defaultCountry",
    "default-country",
    "currency",
    "country",
    "date",
    "data",
    "centerLabel",
    "browse",
    "area",
    "addon",
    "accept",
    "alt",
    "code",
    "content",
    "caption",
    "subtitle",
    "brand",
    "message",
    "author",
    "icon",
    "shortcut",
    "badge",
    "suffix",
];

/// Item kinds renderers branch on. Hostile variants add one item of each.
const ITEM_KINDS: &[&str] = &[
    "label",
    "title",
    "description",
    "text",
    "value",
    "item",
    "tab",
    "columns",
    "option",
    "link",
    "image",
    "url",
    "source",
    "code",
    "header",
    "footer",
    "action",
];

/// Attributes whose value is navigated / fetched by the browser.
const URL_ATTRS: &[&str] = &[
    "href",
    "src",
    "action",
    "formaction",
    "poster",
    "xlink:href",
    "data",
    "srcset",
    "background",
    "cite",
    "ping",
    "manifest",
];

fn hostile_config(text: &str, url: &str) -> HashMap<String, String> {
    let mut cfg = HashMap::new();
    for k in TEXT_KEYS {
        cfg.insert((*k).to_string(), text.to_string());
    }
    for k in URL_KEYS {
        cfg.insert((*k).to_string(), url.to_string());
    }
    cfg
}

/// `text` in every item / text key, `url` in every link and URL key.
/// `on_props` false leaves props empty so the item-config fallback is exercised.
fn hostile(family: &str, text: &str, url: &str, on_props: bool) -> ComponentNode {
    let mut comp = stub(family, text);
    comp.items[0].link = Some(url.to_string());
    comp.items[0].config = hostile_config(text, url);
    for kind in ITEM_KINDS {
        comp.items.push(ComponentItemNode {
            item_type: (*kind).to_string(),
            text: text.to_string(),
            link: Some(url.to_string()),
            tone: None,
            config: hostile_config(text, url),
        });
    }
    if on_props {
        comp.props = hostile_config(text, url);
    }
    comp
}

fn variants(family: &str) -> Vec<(&'static str, ComponentNode)> {
    vec![
        ("bare", stub(family, "Label")),
        ("hostile-props", hostile(family, XSS_TEXT, JS_URL, true)),
        ("hostile-config", hostile(family, XSS_TEXT, JS_URL, false)),
        // Every text is a JS URL too: catches renderers that sniff URLs from
        // item text or read `src` from a non-URL key.
        ("hostile-all-urls", hostile(family, JS_URL, JS_URL, true)),
    ]
}

struct Tag {
    name: String,
    attrs: Vec<(String, String)>,
}

/// Minimal HTML tag / attribute tokenizer (enough for renderer output).
fn tags(html: &str) -> Vec<Tag> {
    let b = html.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'<' {
            i += 1;
            continue;
        }
        if html[i..].starts_with("<!--") {
            i = html[i..].find("-->").map(|e| i + e + 3).unwrap_or(b.len());
            continue;
        }
        let mut j = i + 1;
        if j >= b.len() || !b[j].is_ascii_alphabetic() {
            i += 1;
            continue;
        }
        let ns = j;
        while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'-' || b[j] == b':') {
            j += 1;
        }
        let name = html[ns..j].to_ascii_lowercase();
        let mut attrs = Vec::new();
        loop {
            while j < b.len() && (b[j].is_ascii_whitespace() || b[j] == b'/') {
                j += 1;
            }
            if j >= b.len() || b[j] == b'>' {
                break;
            }
            let an = j;
            while j < b.len() && !b[j].is_ascii_whitespace() && !matches!(b[j], b'=' | b'>' | b'/')
            {
                j += 1;
            }
            let aname = html[an..j].to_ascii_lowercase();
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            let mut value = String::new();
            if j < b.len() && b[j] == b'=' {
                j += 1;
                while j < b.len() && b[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < b.len() && (b[j] == b'"' || b[j] == b'\'') {
                    let q = b[j];
                    let vs = j + 1;
                    j = vs;
                    while j < b.len() && b[j] != q {
                        j += 1;
                    }
                    value = html[vs..j.min(b.len())].to_string();
                    j += 1;
                } else {
                    let vs = j;
                    while j < b.len() && !b[j].is_ascii_whitespace() && b[j] != b'>' {
                        j += 1;
                    }
                    value = html[vs..j].to_string();
                }
            }
            if aname.is_empty() {
                j += 1;
            } else {
                attrs.push((aname, value));
            }
        }
        out.push(Tag { name, attrs });
        i = j + 1;
    }
    out
}

fn decode_entities(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(p) = rest.find('&') {
        out.push_str(&rest[..p]);
        let tail = &rest[p..];
        let end = tail.find(';').filter(|e| *e <= 10);
        let decoded = end.and_then(|e| {
            let ent = &tail[1..e];
            let ch = match ent {
                "amp" => Some('&'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "colon" => Some(':'),
                "Tab" | "tab" => Some('\t'),
                "NewLine" => Some('\n'),
                _ if ent.starts_with("#x") || ent.starts_with("#X") => {
                    u32::from_str_radix(&ent[2..], 16)
                        .ok()
                        .and_then(char::from_u32)
                }
                _ if ent.starts_with('#') => ent[1..].parse().ok().and_then(char::from_u32),
                _ => None,
            };
            ch.map(|c| (c, e))
        });
        match decoded {
            Some((c, e)) => {
                out.push(c);
                rest = &tail[e + 1..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn executable_url(value: &str) -> bool {
    let v: String = decode_entities(value)
        .chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect::<String>()
        .to_ascii_lowercase();
    v.contains("javascript:") || v.contains("vbscript:")
}

/// Every violation in `html`, as short human-readable snippets.
fn offenses(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    let lower = html.to_ascii_lowercase();
    for needle in ["<script", "<style", "<canvas", " style=\"", "<img src=x"] {
        if let Some(p) = lower.find(needle) {
            let end = (p + 60).min(html.len());
            let end = (end..=html.len())
                .find(|e| html.is_char_boundary(*e))
                .unwrap_or(html.len());
            out.push(format!("`{needle}` at …{}…", &html[p..end]));
        }
    }
    for tag in tags(html) {
        if matches!(tag.name.as_str(), "script" | "style" | "canvas" | "iframe") {
            out.push(format!("<{}> element", tag.name));
        }
        for (name, value) in &tag.attrs {
            let handler = name.len() > 2
                && name.starts_with("on")
                && name[2..].bytes().all(|c| c.is_ascii_lowercase());
            if handler {
                out.push(format!("<{} {name}=\"{value}\">", tag.name));
            }
            if name == "style" {
                out.push(format!("<{} style=\"{value}\">", tag.name));
            }
            if URL_ATTRS.contains(&name.as_str()) && executable_url(value) {
                out.push(format!("<{} {name}=\"{value}\">", tag.name));
            }
        }
    }
    out
}

#[test]
fn every_ported_family_output_is_zero_js_and_escaped() {
    let mut failures = Vec::new();
    for family in PORTED_FAMILIES {
        if KNOWN_JS_OFFENDERS.contains(family) {
            continue;
        }
        for (variant, comp) in variants(family) {
            let html = render(&comp).unwrap_or_else(|| panic!("{family}: dispatcher gave None"));
            for o in offenses(&html) {
                failures.push(format!("{family} [{variant}]: {o}"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} unsafe renderer outputs:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn known_js_offenders_are_ported_and_still_offend() {
    // When a listed family is fixed, remove it from KNOWN_JS_OFFENDERS.
    for family in KNOWN_JS_OFFENDERS {
        assert!(PORTED_FAMILIES.contains(family), "{family} not ported");
        let still = variants(family)
            .into_iter()
            .any(|(_, comp)| !offenses(&render(&comp).unwrap()).is_empty());
        assert!(
            still,
            "{family} is clean now: drop it from KNOWN_JS_OFFENDERS"
        );
    }
}

#[test]
fn gate_detects_each_offense_kind() {
    let bad = [
        "<script>x</script>",
        "<div><style>a{}</style></div>",
        "<canvas></canvas>",
        "<div style=\"color:red\"></div>",
        "<button onclick=\"x()\"></button>",
        "<button ONCLICK='x()'></button>",
        "<img src=x onerror=alert(1)>",
        "<a href=\"javascript:alert(1)\">x</a>",
        "<a href=\"JavaScript:alert(1)\">x</a>",
        "<a href=\"java&#9;script:alert(1)\">x</a>",
        "<a href=' javascript:alert(1)'>x</a>",
        "<img data-slot=\"a\" src=\"javascript:alert(1)\">",
    ];
    for html in bad {
        assert!(!offenses(html).is_empty(), "gate missed: {html}");
    }
    let good = [
        "<span>&lt;img src=x onerror=alert(1)&gt;&quot; onmouseover=&quot;alert(1)' onfocus='alert(1)</span>",
        "<div aria-label=\"&lt;img src=x onerror=alert(1)&gt;&quot; onmouseover=&quot;alert(1)\"></div>",
        "<span>javascript:alert(1)</span>",
        "<a href=\"#\" aria-label=\"javascript:alert(1)\">x</a>",
        "<a href=\"https://example.com/?a=1&amp;b=2\">x</a>",
        "<svg viewBox=\"0 0 1 1\"><polyline points=\"0,0\"></polyline></svg>",
    ];
    for html in good {
        assert!(
            offenses(html).is_empty(),
            "gate false positive: {html} → {:?}",
            offenses(html)
        );
    }
}

#[test]
fn renderer_sources_do_not_redefine_kit_helpers() {
    let mut failures = Vec::new();
    let dir = std::fs::read_dir("src").expect("read src");
    for entry in dir.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !name.starts_with("cronus_ui") || !name.ends_with(".rs") {
            continue;
        }
        if name == "cronus_ui_kit.rs" || name == "cronus_ui_output_gate.rs" {
            continue;
        }
        let src = std::fs::read_to_string(&path).expect("read renderer");
        for (i, line) in src.lines().enumerate() {
            let t = line.trim_start();
            for needle in ["fn esc(", "fn attr(", "fn attr<", "fn flag("] {
                if t.starts_with(needle) || t.contains(&format!(" {needle}")) {
                    failures.push(format!("{name}:{}: {}", i + 1, line.trim()));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "use crate::cronus_ui_kit::{{esc, attr, flag}} instead of local copies:\n{}",
        failures.join("\n")
    );
}
