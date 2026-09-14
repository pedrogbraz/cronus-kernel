//! Dedicated JsonViewer renderer. DOM matches React (`json-viewer.tsx`, root
//! object, `defaultExpandedDepth = 1`):
//! `<div data-slot="json-viewer">` → `json-viewer-branch` → opening
//! `json-viewer-row` (`json-viewer-toggle` + `{` + row `copy-button`),
//! `json-viewer-children` of leaf rows (`"key": value,` with `json-viewer-key`
//! / `json-viewer-value` + row `copy-button`), then `json-viewer-closer` (`}`).
//!
//! Entries come from `key: value` content texts; values are typed like JSON
//! literals (`true`/`false` boolean, `null`, numbers, everything else a quoted
//! string) and colored by token. With no entries the renderer shows the React
//! audit harness fallback payload `{ "name": "Ada", "ok": true }` — the emitter
//! cannot carry the fixture's `data` object, so both sides use one placeholder.
//! The `label` names the widget (`aria-label`), it is never an entry.
//!
//! Zero JS (Wave 1t control contract): the branch toggle (collapse) and the
//! per-row copy buttons (clipboard) are the same native `<button>`s React
//! renders, marked `disabled`, in React's idle state — expanded toggle, copy
//! buttons at `opacity: 0` (React reveals them only on row hover/focus).
//! Not interact `codey()` (`<pre style=SURF>`) or catalog `display()` SURF.

use crate::cronus_ui_kit::esc;
use crate::parser::ComponentNode;

const NAME_KINDS: &[&str] = &["label", "title"];

/// React `react-fixture-render.tsx` json-viewer fallback payload.
const DEMO_ENTRIES: [(&str, &str); 2] = [("name", "Ada"), ("ok", "true")];

/// lucide `ChevronRight`.
const CHEVRON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m9 18 6-6-6-6\"/></svg>";

/// lucide `Copy`.
const COPY_ICON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><rect width=\"14\" height=\"14\" x=\"8\" y=\"8\" rx=\"2\" ry=\"2\"/><path d=\"M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2\"/></svg>";

const SPACER: &str = "<span aria-hidden=\"true\"></span>";

pub fn render(comp: &ComponentNode) -> String {
    let entries = entries(comp);
    let count = entries.len();
    let rows = entries
        .iter()
        .enumerate()
        .map(|(i, (key, value))| {
            let comma = if i + 1 < count {
                "<span data-json=\"punct\">,</span>"
            } else {
                ""
            };
            format!(
                "<div data-slot=\"json-viewer-row\">{SPACER}<span><span data-slot=\"json-viewer-key\">{}</span><span data-json=\"punct\">: </span>{}{comma}</span>{}</div>",
                esc(&quote(key)),
                value_span(value),
                copy_button(&format!("Copy value of {}", esc(key)))
            )
        })
        .collect::<String>();
    let aria = aria_label(comp)
        .map(|v| format!(" aria-label=\"{}\"", esc(v)))
        .unwrap_or_default();
    format!(
        "<div data-slot=\"json-viewer\"{aria}><div data-slot=\"json-viewer-branch\"><div data-slot=\"json-viewer-row\"><button type=\"button\" data-slot=\"json-viewer-toggle\" data-state=\"open\" aria-expanded=\"true\" aria-label=\"Toggle root\" disabled>{CHEVRON}</button><span><span data-json=\"punct\">{{</span></span>{}</div><div data-slot=\"json-viewer-children\">{rows}</div><div data-slot=\"json-viewer-closer\">{SPACER}<span><span data-json=\"punct\">}}</span></span></div></div></div>",
        copy_button("Copy value")
    )
}

/// React `CopyButton` (ghost, icon-sm) — inert without a clipboard runtime.
fn copy_button(label: &str) -> String {
    format!(
        "<button type=\"button\" data-slot=\"copy-button\" data-variant=\"ghost\" aria-label=\"{label}\" disabled>{COPY_ICON}</button>"
    )
}

fn aria_label(comp: &ComponentNode) -> Option<&str> {
    comp.props
        .get("aria-label")
        .or_else(|| comp.items.iter().find_map(|i| i.config.get("aria-label")))
        .map(String::as_str)
        .filter(|v| !v.is_empty())
}

fn entries(comp: &ComponentNode) -> Vec<(String, String)> {
    let parsed: Vec<(String, String)> = comp
        .items
        .iter()
        .filter(|i| !i.text.is_empty() && !NAME_KINDS.contains(&i.item_type.as_str()))
        .filter_map(|i| {
            i.text
                .split_once(':')
                .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        })
        .collect();
    if !parsed.is_empty() {
        return parsed;
    }
    DEMO_ENTRIES
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
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
    use crate::cronus_ui_kit::stub;
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

    /// Emitted audit source (`label "Payload"` + `aria-label:"Payload"`, no
    /// entries) renders the React harness payload as an expanded root object.
    #[test]
    fn label_only_renders_react_harness_payload_tree() {
        let mut c = stub("json-viewer", "Payload");
        c.items[0]
            .config
            .insert("aria-label".into(), "Payload".into());
        let html = render(&c);
        let copy = |label: &str| {
            format!("<button type=\"button\" data-slot=\"copy-button\" data-variant=\"ghost\" aria-label=\"{label}\" disabled>{COPY_ICON}</button>")
        };
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"json-viewer\" aria-label=\"Payload\"><div data-slot=\"json-viewer-branch\"><div data-slot=\"json-viewer-row\"><button type=\"button\" data-slot=\"json-viewer-toggle\" data-state=\"open\" aria-expanded=\"true\" aria-label=\"Toggle root\" disabled>{CHEVRON}</button><span><span data-json=\"punct\">{{</span></span>{}</div><div data-slot=\"json-viewer-children\"><div data-slot=\"json-viewer-row\">{SPACER}<span><span data-slot=\"json-viewer-key\">&quot;name&quot;</span><span data-json=\"punct\">: </span><span data-slot=\"json-viewer-value\" data-type=\"string\">&quot;Ada&quot;</span><span data-json=\"punct\">,</span></span>{}</div><div data-slot=\"json-viewer-row\">{SPACER}<span><span data-slot=\"json-viewer-key\">&quot;ok&quot;</span><span data-json=\"punct\">: </span><span data-slot=\"json-viewer-value\" data-type=\"boolean\">true</span></span>{}</div></div><div data-slot=\"json-viewer-closer\">{SPACER}<span><span data-json=\"punct\">}}</span></span></div></div></div>",
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

    #[test]
    fn js_controls_are_disabled_native_buttons() {
        let html = render(&stub("json-viewer", "Payload"));
        assert!(html.contains("aria-expanded=\"true\" aria-label=\"Toggle root\" disabled>"));
        assert_eq!(html.matches("data-slot=\"copy-button\"").count(), 3);
        assert_eq!(
            html.matches("data-slot=\"copy-button\" data-variant=\"ghost\"")
                .count(),
            html.matches(" disabled>").count() - 1
        );
        assert!(!crate::cli::stub_renderer_gate::looks_like_interact_generic(&html));
    }

    #[test]
    fn skips_interact_pre_and_display_surf() {
        let mut c = stub("json-viewer", "Payload");
        c.items.push(extra("item", "name: Ada"));
        let html = render(&c);
        let interact = crate::cronus_ui_interact::render("json-viewer", &c).unwrap();
        assert_ne!(html, interact);
        assert!(interact.starts_with("<pre data-slot=\"json-viewer\""));
        assert!(!interact.contains("data-slot=\"json-viewer-row\""));
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
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"json-viewer\"] {\n  display: block; width: 18rem; max-width: 100%; overflow-x: auto; padding: 1rem;\n  border: 1px solid var(--cronus-border); border-radius: var(--cronus-radius-xl);\n  background: var(--cronus-surface-inset); color: var(--cronus-fg);\n  font-family: var(--cronus-font-mono, ui-monospace, monospace); font-size: 0.875rem; line-height: 1.5rem;\n}"));
        assert!(css.contains("[data-slot=\"json-viewer-children\"] { margin-left: 0.625rem; padding-left: 0.875rem; border-left: 1px solid var(--cronus-border); }"));
        assert!(css.contains("[data-slot=\"json-viewer-row\"] > [data-slot=\"copy-button\"] {\n  width: 1.25rem; height: 1.25rem; flex-shrink: 0; margin-top: 0.125rem; padding: 0;\n  border: 0; border-radius: var(--cronus-radius-md); background: transparent;\n  color: var(--cronus-fg-secondary); font-size: 0.875rem; line-height: 1.25rem;\n  opacity: 0; cursor: default;\n}"));
        assert!(css.contains("[data-slot=\"json-viewer-value\"][data-type=\"string\"] { color: var(--cronus-success-text); }"));
        assert!(css.contains("[data-slot=\"json-viewer-value\"][data-type=\"boolean\"] { color: var(--cronus-warning-text); }"));
        assert!(!css.contains("zinc-"));
    }
}
