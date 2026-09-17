//! Dedicated JsonViewer renderer. DOM matches React (`json-viewer.tsx`):
//! `<div data-slot="json-viewer">` → a `json-viewer-branch` per object/array:
//! opening `json-viewer-row` (`json-viewer-toggle` + key + `{` / `[` + row
//! `copy-button`), `json-viewer-children` of rows (`"key": value,` with
//! `json-viewer-key` / `json-viewer-value`) and the `json-viewer-closer`
//! (`}` / `]`); a collapsed branch shows React's `json-viewer-summary`
//! (` … 2 items `) and closing token on its row instead.
//!
//! The value is a flat outline of `item` lines: `item "id" value:ord_8kX2
//! level:2` is a leaf (values typed like JSON literals: `true`/`false`
//! boolean, `null`, numbers, everything else a quoted string) and
//! `item "items" type:array level:2` / `type:object` opens a branch whose
//! children are the deeper lines that follow; the root is an object of the
//! level-1 lines (an object key with `type:array` at level 1 nests under it).
//! Plain `key: value` texts are the root object's entries. With no entries the
//! renderer shows the React audit harness fallback payload `{ "name": "Ada",
//! "ok": true }` — the emitter cannot carry the fixture's `data` object, so
//! both sides use one placeholder. `depth:N` is `defaultExpandedDepth` (root =
//! depth 0, default 1); `depth:all` expands everything. The `label` names the
//! widget (`aria-label`), it is never an entry.
//!
//! Zero JS: each branch toggle is React's `json-viewer-toggle` button behind a
//! visually hidden checkbox in a `<label>`; CSS shows the children and closer
//! (or the collapsed summary) and rotates the chevron by `:checked`. The
//! per-row copy buttons (clipboard) stay native `disabled` buttons in React's
//! idle state (`opacity: 0`, revealed on row hover/focus). Not interact
//! `codey()` (`<pre style=SURF>`) or catalog `display()` SURF.

use crate::cronus_ui_kit::{attr_nonempty, esc, instance_id};
use crate::parser::ComponentNode;

const NAME_KINDS: &[&str] = &["label", "title"];

/// React `react-fixture-render.tsx` json-viewer fallback payload.
const DEMO_ENTRIES: [(&str, &str); 2] = [("name", "Ada"), ("ok", "true")];

/// lucide `ChevronRight`.
const CHEVRON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m9 18 6-6-6-6\"/></svg>";

/// lucide `Copy`.
const COPY_ICON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><rect width=\"14\" height=\"14\" x=\"8\" y=\"8\" rx=\"2\" ry=\"2\"/><path d=\"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2\"/></svg>";

const SPACER: &str = "<span aria-hidden=\"true\"></span>";

enum Value {
    Leaf(String),
    Object(Vec<(String, Value)>),
    Array(Vec<Value>),
}

pub fn render(comp: &ComponentNode) -> String {
    let root = value_of(comp);
    let depth = match attr_nonempty(comp, "depth").map(str::trim) {
        Some("all") | Some("infinity") => usize::MAX,
        Some(n) => n.parse().unwrap_or(1),
        None => 1,
    };
    let aria = aria_label(comp)
        .map(|v| format!(" aria-label=\"{}\"", esc(v)))
        .unwrap_or_default();
    let body = node(comp, None, false, &root, 0, depth, true);
    format!("<div data-slot=\"json-viewer\"{aria}>{body}</div>")
}

/// React `CopyButton` (ghost, icon-sm) — inert without a clipboard runtime.
fn copy_button(label: &str) -> String {
    format!(
        "<button type=\"button\" data-slot=\"copy-button\" data-variant=\"ghost\" aria-label=\"{label}\" disabled>{COPY_ICON}</button>"
    )
}

fn copy_label(name: Option<&str>) -> String {
    match name {
        Some(n) => format!("Copy value of {n}"),
        None => "Copy value".into(),
    }
}

fn punct(text: &str) -> String {
    format!("<span data-json=\"punct\">{text}</span>")
}

fn key_label(name: Option<&str>, quoted: bool) -> String {
    match name {
        Some(n) => {
            let shown = if quoted { esc(&quote(n)) } else { esc(n) };
            format!(
                "<span data-slot=\"json-viewer-key\">{shown}</span>{}",
                punct(": ")
            )
        }
        None => String::new(),
    }
}

fn node(
    comp: &ComponentNode,
    name: Option<&str>,
    quoted: bool,
    value: &Value,
    depth: usize,
    expanded_depth: usize,
    is_last: bool,
) -> String {
    let comma = if is_last { String::new() } else { punct(",") };
    let copy = copy_button(&copy_label(name.map(esc).as_deref()));
    let key = key_label(name, quoted);
    match value {
        Value::Leaf(raw) => format!(
            "<div data-slot=\"json-viewer-row\">{SPACER}<span>{key}{}{comma}</span>{copy}</div>",
            value_span(raw)
        ),
        Value::Object(entries) if entries.is_empty() => format!(
            "<div data-slot=\"json-viewer-row\">{SPACER}<span>{key}{}{comma}</span>{copy}</div>",
            punct("{}")
        ),
        Value::Array(items) if items.is_empty() => format!(
            "<div data-slot=\"json-viewer-row\">{SPACER}<span>{key}{}{comma}</span>{copy}</div>",
            punct("[]")
        ),
        Value::Object(_) | Value::Array(_) => {
            let (open, close, count, noun) = match value {
                Value::Object(entries) => (
                    "{",
                    "}",
                    entries.len(),
                    if entries.len() == 1 { "key" } else { "keys" },
                ),
                Value::Array(items) => (
                    "[",
                    "]",
                    items.len(),
                    if items.len() == 1 { "item" } else { "items" },
                ),
                Value::Leaf(_) => unreachable!(),
            };
            let expanded = depth < expanded_depth;
            let (state, aria_expanded, checked) = if expanded {
                ("open", "true", " checked")
            } else {
                ("closed", "false", "")
            };
            let toggle_name = esc(name.unwrap_or("root"));
            let id = instance_id(comp, "toggle");
            let toggle = format!(
                "<label class=\"cui-json-toggle\"><input type=\"checkbox\" id=\"{id}\" aria-label=\"Toggle {toggle_name}\"{checked}><button type=\"button\" data-slot=\"json-viewer-toggle\" data-state=\"{state}\" aria-expanded=\"{aria_expanded}\" aria-label=\"Toggle {toggle_name}\" tabindex=\"-1\" aria-hidden=\"true\">{CHEVRON}</button></label>"
            );
            let summary = format!(
                "<span data-slot=\"json-viewer-summary\"> … {count} {noun} </span><span data-json=\"punct\" class=\"when-collapsed\">{close}</span>{}",
                if is_last {
                    String::new()
                } else {
                    "<span data-json=\"punct\" class=\"when-collapsed\">,</span>".to_string()
                }
            );
            let children: String = match value {
                Value::Object(entries) => entries
                    .iter()
                    .enumerate()
                    .map(|(i, (k, v))| {
                        node(
                            comp,
                            Some(k),
                            true,
                            v,
                            depth + 1,
                            expanded_depth,
                            i + 1 == count,
                        )
                    })
                    .collect(),
                Value::Array(items) => items
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        node(
                            comp,
                            Some(&i.to_string()),
                            false,
                            v,
                            depth + 1,
                            expanded_depth,
                            i + 1 == count,
                        )
                    })
                    .collect(),
                Value::Leaf(_) => unreachable!(),
            };
            format!(
                "<div data-slot=\"json-viewer-branch\"><div data-slot=\"json-viewer-row\">{toggle}<span>{key}{}{summary}</span>{copy}</div><div data-slot=\"json-viewer-children\">{children}</div><div data-slot=\"json-viewer-closer\">{SPACER}<span>{}{comma}</span></div></div>",
                punct(open),
                punct(close)
            )
        }
    }
}

fn aria_label(comp: &ComponentNode) -> Option<&str> {
    attr_nonempty(comp, "aria-label")
}

/// The root value: the outline of `item` lines, else `key: value` texts, else
/// the harness payload.
fn value_of(comp: &ComponentNode) -> Value {
    let lines: Vec<&crate::parser::ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty() && !NAME_KINDS.contains(&i.item_type.as_str()))
        .collect();
    if lines
        .iter()
        .any(|i| i.config.contains_key("value") || i.config.contains_key("type"))
    {
        let mut index = 0;
        return Value::Object(outline(&lines, &mut index, 1));
    }
    let parsed: Vec<(String, Value)> = lines
        .iter()
        .filter_map(|i| {
            i.text
                .split_once(':')
                .map(|(k, v)| (k.trim().to_string(), Value::Leaf(v.trim().to_string())))
        })
        .collect();
    if !parsed.is_empty() {
        return Value::Object(parsed);
    }
    Value::Object(
        DEMO_ENTRIES
            .iter()
            .map(|(k, v)| ((*k).to_string(), Value::Leaf((*v).to_string())))
            .collect(),
    )
}

/// Entries at `level`, consuming their deeper children.
fn outline(
    lines: &[&crate::parser::ComponentItemNode],
    index: &mut usize,
    level: u8,
) -> Vec<(String, Value)> {
    let mut out = Vec::new();
    while *index < lines.len() {
        let line = lines[*index];
        let l: u8 = line
            .config
            .get("level")
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(1)
            .max(1);
        if l < level {
            break;
        }
        if l > level {
            // Orphan deeper line: treat it as a sibling at this level.
        }
        *index += 1;
        let value = match line.config.get("type").map(|t| t.trim()) {
            Some("array") => {
                let children = outline(lines, index, l + 1);
                Value::Array(children.into_iter().map(|(_, v)| v).collect())
            }
            Some("object") => Value::Object(outline(lines, index, l + 1)),
            _ => Value::Leaf(line.config.get("value").cloned().unwrap_or_default()),
        };
        out.push((line.text.clone(), value));
    }
    out
}

/// `JSON.stringify` for a string (quotes + escapes), before HTML escaping.
fn quote(s: &str) -> String {
    serde_json::Value::String(s.to_string()).to_string()
}

/// Leaf value typed like a JSON literal, colored like React `formatPrimitive`.
fn value_span(raw: &str) -> String {
    let (kind, text) = match raw {
        "true" | "false" => ("boolean", raw.to_string()),
        "null" => ("null", raw.to_string()),
        _ if raw.parse::<f64>().map(f64::is_finite).unwrap_or(false) => ("number", raw.to_string()),
        _ => ("string", quote(raw)),
    };
    format!(
        "<span data-slot=\"json-viewer-value\" data-type=\"{kind}\">{}</span>",
        esc(&text)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::{reset_instance_ids, stub};
    use crate::parser::ComponentItemNode;

    fn extra(kind: &str, text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn line(key: &str, pairs: &[(&str, &str)]) -> ComponentItemNode {
        let mut i = extra("item", key);
        for (k, v) in pairs {
            i.config.insert(k.to_string(), v.to_string());
        }
        i
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<pre"));
        assert!(!html.contains("<code"));
        assert!(!html.contains("<section"));
        assert!(!html.contains("style="));
        assert!(!html.contains("SURF"));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("navigator.clipboard"));
        assert!(!html.contains("codey("));
    }

    fn copy(label: &str) -> String {
        format!("<button type=\"button\" data-slot=\"copy-button\" data-variant=\"ghost\" aria-label=\"{label}\" disabled>{COPY_ICON}</button>")
    }

    /// Emitted audit source (`label "Payload"` + `aria-label:"Payload"`, no
    /// entries) renders the React harness payload as an expanded root object;
    /// the root toggle is a checked checkbox behind React's toggle button.
    #[test]
    fn label_only_renders_react_harness_payload_tree() {
        reset_instance_ids();
        let mut c = stub("json-viewer", "Payload");
        c.items[0]
            .config
            .insert("aria-label".into(), "Payload".into());
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"json-viewer\" aria-label=\"Payload\"><div data-slot=\"json-viewer-branch\"><div data-slot=\"json-viewer-row\"><label class=\"cui-json-toggle\"><input type=\"checkbox\" id=\"cui-json-viewer-toggle\" aria-label=\"Toggle root\" checked><button type=\"button\" data-slot=\"json-viewer-toggle\" data-state=\"open\" aria-expanded=\"true\" aria-label=\"Toggle root\" tabindex=\"-1\" aria-hidden=\"true\">{CHEVRON}</button></label><span><span data-json=\"punct\">{{</span><span data-slot=\"json-viewer-summary\"> … 2 keys </span><span data-json=\"punct\" class=\"when-collapsed\">}}</span></span>{}</div><div data-slot=\"json-viewer-children\"><div data-slot=\"json-viewer-row\">{SPACER}<span><span data-slot=\"json-viewer-key\">&quot;name&quot;</span><span data-json=\"punct\">: </span><span data-slot=\"json-viewer-value\" data-type=\"string\">&quot;Ada&quot;</span><span data-json=\"punct\">,</span></span>{}</div><div data-slot=\"json-viewer-row\">{SPACER}<span><span data-slot=\"json-viewer-key\">&quot;ok&quot;</span><span data-json=\"punct\">: </span><span data-slot=\"json-viewer-value\" data-type=\"boolean\">true</span></span>{}</div></div><div data-slot=\"json-viewer-closer\">{SPACER}<span><span data-json=\"punct\">}}</span></span></div></div></div>",
                copy("Copy value"),
                copy("Copy value of name"),
                copy("Copy value of ok")
            )
        );
        assert!(!html.contains(">Payload<"));
        reject_interact(&html);
    }

    #[test]
    fn key_value_texts_are_typed_entries() {
        let mut c = stub("json-viewer", "Payload");
        for t in ["id: 7", "role: Engineer", "active: false", "manager: null"] {
            c.items.push(extra("text", t));
        }
        let html = render(&c);
        assert_eq!(html.matches("data-slot=\"json-viewer-row\"").count(), 5);
        assert_eq!(html.matches("data-slot=\"copy-button\"").count(), 5);
        assert!(html.contains("data-type=\"number\">7</span>"));
        assert!(html.contains("data-type=\"string\">&quot;Engineer&quot;</span>"));
        assert!(html.contains("data-type=\"boolean\">false</span>"));
        assert!(html.contains("data-type=\"null\">null</span></span>"));
        assert!(!html.contains("&quot;Ada&quot;"));
        reject_interact(&html);
    }

    #[test]
    fn keys_and_values_are_escaped() {
        let mut c = stub("json-viewer", "Payload");
        c.items.push(extra("text", "a <b>: c & \"d\""));
        let html = render(&c);
        assert!(html.contains("json-viewer-key\">&quot;a &lt;b&gt;&quot;</span>"));
        assert!(html.contains("data-type=\"string\">&quot;c &amp; \\&quot;d\\&quot;&quot;</span>"));
        assert!(html.contains("aria-label=\"Copy value of a &lt;b&gt;\" disabled>"));
        reject_interact(&html);
    }

    /// Docs "API response" / "Expanded depth": the level outline nests
    /// objects and arrays; `depth:2` opens two levels and leaves `items`
    /// collapsed (summary shown), `depth:all` opens everything.
    #[test]
    fn outline_nests_objects_and_arrays_by_depth() {
        reset_instance_ids();
        let mut c = stub("json-viewer", "Payload");
        c.items.clear();
        c.items.push(line("order", &[("type", "object")]));
        c.items
            .push(line("id", &[("value", "ord_8kX2"), ("level", "2")]));
        c.items
            .push(line("total", &[("value", "248.9"), ("level", "2")]));
        c.items
            .push(line("paid", &[("value", "true"), ("level", "2")]));
        c.items
            .push(line("items", &[("type", "array"), ("level", "2")]));
        c.items
            .push(line("0", &[("type", "object"), ("level", "3")]));
        c.items
            .push(line("sku", &[("value", "tee"), ("level", "4")]));
        c.items.push(line("qty", &[("value", "2"), ("level", "4")]));
        c.items
            .push(line("1", &[("type", "object"), ("level", "3")]));
        c.items
            .push(line("sku", &[("value", "sticker"), ("level", "4")]));
        c.props.insert("depth".into(), "2".into());
        let html = render(&c);
        // root (depth 0) and order (depth 1) open; items (depth 2) collapsed.
        assert!(html.contains("aria-label=\"Toggle root\" checked>"));
        assert!(html.contains("aria-label=\"Toggle order\" checked>"));
        assert!(html.contains("<input type=\"checkbox\" id=\"cui-json-viewer-toggle-3\" aria-label=\"Toggle items\"><button type=\"button\" data-slot=\"json-viewer-toggle\" data-state=\"closed\" aria-expanded=\"false\""));
        assert!(html.contains("<span data-slot=\"json-viewer-key\">&quot;items&quot;</span><span data-json=\"punct\">: </span><span data-json=\"punct\">[</span><span data-slot=\"json-viewer-summary\"> … 2 items </span><span data-json=\"punct\" class=\"when-collapsed\">]</span></span>"));
        // Array children keep unquoted index keys; nested objects open per depth.
        assert!(html.contains("<span data-slot=\"json-viewer-key\">0</span><span data-json=\"punct\">: </span><span data-json=\"punct\">{</span>"));
        assert!(html.contains("data-type=\"number\">248.9</span>"));
        assert!(html.contains("data-type=\"string\">&quot;tee&quot;</span>"));
        assert!(html.contains("aria-label=\"Copy value of sku\" disabled>"));
        assert_eq!(html.matches("data-slot=\"json-viewer-branch\"").count(), 5);
        assert_eq!(html.matches(" checked>").count(), 2);
        c.props.insert("depth".into(), "all".into());
        assert_eq!(render(&c).matches(" checked>").count(), 5);
        reject_interact(&html);
        let css = include_str!("cronus_ui_css/json-viewer.css");
        assert!(css.contains("[data-slot=\"json-viewer-branch\"]:has(> [data-slot=\"json-viewer-row\"] > .cui-json-toggle > input:not(:checked)) > [data-slot=\"json-viewer-children\"],\n[data-slot=\"json-viewer-branch\"]:has(> [data-slot=\"json-viewer-row\"] > .cui-json-toggle > input:not(:checked)) > [data-slot=\"json-viewer-closer\"] { display: none; }"));
        assert!(css.contains("[data-slot=\"json-viewer-branch\"]:has(> [data-slot=\"json-viewer-row\"] > .cui-json-toggle > input:checked) > [data-slot=\"json-viewer-row\"] :is([data-slot=\"json-viewer-summary\"], .when-collapsed) { display: none; }"));
        assert!(css.contains("[data-slot=\"json-viewer-summary\"] { color: var(--cronus-fg-tertiary); font-variant-numeric: tabular-nums; }"));
    }

    #[test]
    fn copy_buttons_are_disabled_native_buttons() {
        let html = render(&stub("json-viewer", "Payload"));
        assert!(html.contains("aria-expanded=\"true\" aria-label=\"Toggle root\" tabindex=\"-1\" aria-hidden=\"true\">"));
        assert_eq!(html.matches("data-slot=\"copy-button\"").count(), 3);
        assert_eq!(
            html.matches("data-slot=\"copy-button\" data-variant=\"ghost\"")
                .count(),
            html.matches(" disabled>").count()
        );
        assert!(!crate::cli::stub_renderer_gate::looks_like_interact_generic(&html));
    }

    #[test]
    fn skips_interact_pre_and_display_surf() {
        let mut c = stub("json-viewer", "Payload");
        c.items.push(extra("item", "name: Ada"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"json-viewer-row\""));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("json-viewer", "Payload"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"json-viewer-key\""));
        });
    }

    /// Geometry parity (Wave 1t): mono 14px/24px on an 18px-radius inset
    /// surface sized `w-72` by the harness, 20px toggle/spacer/copy columns,
    /// 10px + 1px + 14px indent guide, strong-token value colors.
    #[test]
    fn chrome_matches_react_geometry() {
        let css = include_str!("cronus_ui_css/json-viewer.css");
        assert!(css.contains("[data-slot=\"json-viewer\"] {\n  display: block; width: var(--cui-json-viewer-w, 100%); box-sizing: border-box; max-width: 100%; overflow-x: auto; padding: 1rem;\n  border: 1px solid var(--cronus-border); border-radius: var(--cronus-radius-xl);\n  background: var(--cronus-surface-inset); color: var(--cronus-fg);\n  font-family: var(--cronus-font-mono, ui-monospace, monospace); font-size: 0.875rem; line-height: 1.5rem;\n}"));
        assert!(css.contains("[data-slot=\"json-viewer-children\"] { margin-inline-start: 0.625rem; padding-inline-start: 0.875rem; border-inline-start: 1px solid var(--cronus-border); }"));
        assert!(css.contains("[data-slot=\"json-viewer-row\"] > [data-slot=\"copy-button\"] {\n  width: 1.25rem; height: 1.25rem; flex-shrink: 0; margin-top: 0.125rem; padding: 0;\n  border: 0; border-radius: var(--cronus-radius-md); background: transparent;\n  color: var(--cronus-fg-secondary); font-size: 0.875rem; line-height: 1.25rem;\n  opacity: 0; cursor: default;\n}"));
        assert!(css.contains("[data-slot=\"json-viewer-value\"][data-type=\"string\"] { color: var(--cronus-success-text); }"));
        assert!(css.contains("[data-slot=\"json-viewer-value\"][data-type=\"boolean\"] { color: var(--cronus-warning-text); }"));
        assert!(!css.contains("zinc-"));
    }
}
