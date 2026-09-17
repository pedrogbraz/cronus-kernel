//! Dedicated Tool renderer (AI suite). DOM matches React/Radix `Tool` +
//! `ToolHeader` + `ToolContent` + `ToolInput` + `ToolOutput`:
//! `<div data-state data-slot="tool">` > `<button data-slot="tool-header">`
//! (`<div>` wrench, name `<span>`, status `badge` secondary, chevron) >
//! `<div data-slot="tool-content">` > optional `<div data-slot="tool-input">`
//! (`<h4>` Parameters + the `AiCodeBlock` of the pretty-printed JSON built
//! from `field "query" value:"Cronus tokens"` items) + optional
//! `<div data-slot="tool-output">` (`<h4>` Result / Error + the box with the
//! `errorText:` paragraph and/or the `AiCodeBlock` of the `output:` string).
//! The code block is React's DOM (`ai-code-block` > `ai-code-block-body` >
//! `ai-code-block-item` > `<pre data-slot="ai-code-block-content"
//! data-language="json"><code>`), styled here since it has no kernel family.
//!
//! Zero JS: the header and content sit in a native `<details>` inside the root,
//! the button in its `<summary>` (decorative: `aria-hidden`, `tabindex="-1"`,
//! `pointer-events: none`), so the panel toggles natively and the chevron turns
//! on `details[open]`. Radix `Collapsible` starts closed: `defaultOpen:true`
//! (or `open:true`) opens it. Name is `title:` or React's
//! `type.split("-").slice(1).join("-")`; `state:` (`input-streaming` Pending,
//! `input-available` Running, `output-available` Completed, `output-error`
//! Error) picks the badge. Labels localize with `pending:`, `running:`,
//! `completed:`, `error:`, `parameters:`, `result:`, `errorHeading:`.

use crate::cronus_ui_kit::{attr, attr_nonempty, esc, truthy};
use crate::parser::ComponentNode;

const SVG_OPEN: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"";
const WRENCH: &str = "><path d=\"M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.106-3.105c.32-.322.863-.22.983.218a6 6 0 0 1-8.259 7.057l-7.91 7.91a1 1 0 0 1-2.999-3l7.91-7.91a6 6 0 0 1 7.057-8.259c.438.12.54.662.219.984z\"></path></svg>";
const CHEVRON: &str = "><path d=\"m6 9 6 6 6-6\"></path></svg>";
const CIRCLE: &str = "<circle cx=\"12\" cy=\"12\" r=\"10\"></circle>";

/// `(state, label key, default label, icon marker, icon body)` for React's `ToolUIPartState`.
const STATES: &[(&str, &str, &str, &str, &str)] = &[
    ("input-streaming", "pending", "Pending", "pending", ""),
    (
        "input-available",
        "running",
        "Running",
        "running",
        "<path d=\"M12 6v6l4 2\"></path>",
    ),
    (
        "output-available",
        "completed",
        "Completed",
        "completed",
        "<path d=\"m9 12 2 2 4-4\"></path>",
    ),
    (
        "output-error",
        "error",
        "Error",
        "error",
        "<path d=\"m15 9-6 6\"></path><path d=\"m9 9 6 6\"></path>",
    ),
];

pub fn render(comp: &ComponentNode) -> String {
    let open = ["open", "defaultOpen"]
        .iter()
        .find_map(|key| attr(comp, key))
        .is_some_and(truthy);
    let state = if open { "open" } else { "closed" };
    let (_, label_key, label, marker, icon) = status_of(attr(comp, "state"));
    let label = esc(attr_nonempty(comp, label_key).unwrap_or(label));
    let name = esc(&tool_name(comp));
    let header = format!(
        "<button type=\"button\" aria-expanded=\"{open}\" data-state=\"{state}\" data-slot=\"tool-header\" tabindex=\"-1\" aria-hidden=\"true\"><div>{SVG_OPEN} data-icon=\"wrench\"{WRENCH}<span>{name}</span><span data-slot=\"badge\" data-variant=\"secondary\">{SVG_OPEN} data-icon=\"{marker}\">{CIRCLE}{icon}</svg>{label}</span></div>{SVG_OPEN} data-icon=\"chevron\"{CHEVRON}</button>"
    );
    let details_open = if open { " open" } else { "" };
    format!(
        "<div data-state=\"{state}\" data-slot=\"tool\"><details{details_open}><summary>{header}</summary><div data-state=\"{state}\" data-slot=\"tool-content\">{}{}</div></details></div>",
        input_html(comp),
        output_html(comp)
    )
}

fn status_of(
    raw: Option<&str>,
) -> (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
) {
    let raw = raw.map(str::trim).unwrap_or("");
    STATES
        .iter()
        .copied()
        .find(|(state, ..)| *state == raw)
        .unwrap_or(STATES[0])
}

fn tool_name(comp: &ComponentNode) -> String {
    if let Some(title) = attr_nonempty(comp, "title") {
        return title.to_string();
    }
    attr(comp, "type")
        .unwrap_or("")
        .split('-')
        .skip(1)
        .collect::<Vec<_>>()
        .join("-")
}

/// React `AiCodeBlock` (single file, no header) around escaped `code`.
fn code_block(code: &str) -> String {
    format!(
        "<div data-slot=\"ai-code-block\"><div data-slot=\"ai-code-block-body\"><div data-slot=\"ai-code-block-item\"><pre data-slot=\"ai-code-block-content\" data-language=\"json\"><code>{}</code></pre></div></div></div>",
        esc(code)
    )
}

/// `ToolInput`: the `field` items as `JSON.stringify(input, null, 2)`.
fn input_html(comp: &ComponentNode) -> String {
    let fields: Vec<(&str, &str)> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "field" && !i.text.is_empty())
        .map(|i| {
            (
                i.text.as_str(),
                i.config.get("value").map(String::as_str).unwrap_or(""),
            )
        })
        .collect();
    if fields.is_empty() {
        return String::new();
    }
    let rows = fields
        .iter()
        .map(|(k, v)| format!("  {}: {}", json_string(k), json_value(v)))
        .collect::<Vec<_>>()
        .join(",\n");
    let heading = esc(attr_nonempty(comp, "parameters").unwrap_or("Parameters"));
    format!(
        "<div data-slot=\"tool-input\"><h4>{heading}</h4><div>{}</div></div>",
        code_block(&format!("{{\n{rows}\n}}"))
    )
}

/// `ToolOutput`: nothing without `output:` / `errorText:`, like React.
fn output_html(comp: &ComponentNode) -> String {
    let output = attr_nonempty(comp, "output");
    let error = attr_nonempty(comp, "errorText");
    if output.is_none() && error.is_none() {
        return String::new();
    }
    let heading = esc(if error.is_some() {
        attr_nonempty(comp, "errorHeading").unwrap_or("Error")
    } else {
        attr_nonempty(comp, "result").unwrap_or("Result")
    });
    let error_box = error
        .map(|e| format!("<div>{}</div>", esc(e)))
        .unwrap_or_default();
    let body = match output {
        Some(out) => code_block(out),
        None => "<div></div>".to_string(),
    };
    let marker = if error.is_some() {
        " data-error=\"\""
    } else {
        ""
    };
    format!(
        "<div data-slot=\"tool-output\"><h4>{heading}</h4><div{marker}>{error_box}{body}</div></div>"
    )
}

/// JSON string literal.
fn json_string(raw: &str) -> String {
    let mut out = String::from("\"");
    for ch in raw.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// JSON literal for a field value: numbers, booleans and `null` stay bare,
/// everything else is a string.
fn json_value(raw: &str) -> String {
    let t = raw.trim();
    let numeric = !t.is_empty()
        && t.chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E'))
        && t.parse::<f64>().is_ok();
    if t == "true" || t == "false" || t == "null" || numeric {
        t.to_string()
    } else {
        json_string(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn tool(props: &[(&str, &str)]) -> ComponentNode {
        let mut c = stub("tool", "default");
        for (k, v) in props {
            c.props.insert((*k).into(), (*v).into());
        }
        c
    }

    fn field(name: &str, value: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "field".into(),
            text: name.into(),
            link: None,
            tone: None,
            config: [("value".to_string(), value.to_string())]
                .into_iter()
                .collect(),
        }
    }

    /// Docs "Completed tool": `defaultOpen`, `tool-search` / `output-available`
    /// / title `search`, input `{ query: "Cronus tokens" }`, string output.
    #[test]
    fn completed_tool_matches_react_dom() {
        let mut c = tool(&[
            ("defaultOpen", "true"),
            ("type", "tool-search"),
            ("state", "output-available"),
            ("title", "search"),
            ("output", "Semantic tokens live in @cronus-ui/tokens."),
        ]);
        c.items.push(field("query", "Cronus tokens"));
        let html = render(&c);
        assert!(html.starts_with("<div data-state=\"open\" data-slot=\"tool\"><details open><summary><button type=\"button\" aria-expanded=\"true\" data-state=\"open\" data-slot=\"tool-header\" tabindex=\"-1\" aria-hidden=\"true\"><div><svg"));
        assert!(html.contains(
            "<span>search</span><span data-slot=\"badge\" data-variant=\"secondary\"><svg"
        ));
        assert!(html.contains("data-icon=\"completed\"><circle cx=\"12\" cy=\"12\" r=\"10\"></circle><path d=\"m9 12 2 2 4-4\"></path></svg>Completed</span></div><svg"));
        assert!(html.contains("</button></summary><div data-state=\"open\" data-slot=\"tool-content\"><div data-slot=\"tool-input\"><h4>Parameters</h4><div><div data-slot=\"ai-code-block\"><div data-slot=\"ai-code-block-body\"><div data-slot=\"ai-code-block-item\"><pre data-slot=\"ai-code-block-content\" data-language=\"json\"><code>{\n  &quot;query&quot;: &quot;Cronus tokens&quot;\n}</code></pre></div></div></div></div></div>"));
        assert!(html.ends_with("<div data-slot=\"tool-output\"><h4>Result</h4><div><div data-slot=\"ai-code-block\"><div data-slot=\"ai-code-block-body\"><div data-slot=\"ai-code-block-item\"><pre data-slot=\"ai-code-block-content\" data-language=\"json\"><code>Semantic tokens live in @cronus-ui/tokens.</code></pre></div></div></div></div></div></div></details></div>"));
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
        assert!(!html.contains(" disabled"));
    }

    #[test]
    fn error_state_matches_react_dom() {
        let html = render(&tool(&[
            ("open", "true"),
            ("type", "tool-web-search"),
            ("state", "output-error"),
            ("errorText", "Request timed out"),
        ]));
        assert!(html.contains("<span>web-search</span>"));
        assert!(html.contains("data-icon=\"error\"><circle cx=\"12\" cy=\"12\" r=\"10\"></circle><path d=\"m15 9-6 6\"></path><path d=\"m9 9 6 6\"></path></svg>Error</span>"));
        assert!(html.ends_with("<div data-state=\"open\" data-slot=\"tool-content\"><div data-slot=\"tool-output\"><h4>Error</h4><div data-error=\"\"><div>Request timed out</div><div></div></div></div></div></details></div>"));
    }

    #[test]
    fn name_follows_react_type_split_and_title_wins() {
        assert_eq!(tool_name(&tool(&[("type", "tool-a-b")])), "a-b");
        assert_eq!(tool_name(&tool(&[("type", "search")])), "");
        assert_eq!(
            tool_name(&tool(&[("type", "tool-x"), ("title", "Search")])),
            "Search"
        );
    }

    #[test]
    fn states_map_labels_and_unknown_is_pending() {
        assert!(render(&tool(&[("state", "output-available")])).contains("</svg>Completed</span>"));
        assert!(render(&tool(&[("state", "input-available")])).contains("data-icon=\"running\"><circle cx=\"12\" cy=\"12\" r=\"10\"></circle><path d=\"M12 6v6l4 2\"></path></svg>Running</span>"));
        assert!(render(&tool(&[("state", "input-streaming")])).contains("data-icon=\"pending\"><circle cx=\"12\" cy=\"12\" r=\"10\"></circle></svg>Pending</span>"));
        assert!(render(&tool(&[("state", "bogus")])).contains("</svg>Pending</span>"));
        assert!(render(&tool(&[
            ("state", "input-available"),
            ("running", "Em execução")
        ]))
        .contains("</svg>Em execução</span>"));
    }

    /// Radix Collapsible starts closed: no `defaultOpen` keeps the content in a
    /// closed `<details>`; texts are escaped and JSON-quoted.
    #[test]
    fn closed_by_default_and_input_is_json_escaped() {
        let closed = render(&tool(&[("errorText", "x")]));
        assert!(
            closed.starts_with("<div data-state=\"closed\" data-slot=\"tool\"><details><summary>")
        );
        assert!(closed
            .contains("aria-expanded=\"false\" data-state=\"closed\" data-slot=\"tool-header\""));
        assert!(closed.contains("<div data-state=\"closed\" data-slot=\"tool-content\">"));
        let open = render(&tool(&[("open", "true"), ("errorText", "<img onerror=1>")]));
        assert!(open.contains("&lt;img onerror=1&gt;"));
        assert!(!open.contains("<img"));

        let mut c = tool(&[("open", "true"), ("parameters", "Args")]);
        c.items.push(field("q\"<", "a\nb"));
        c.items.push(field("n", "3"));
        c.items.push(field("ok", "true"));
        let html = render(&c);
        assert!(html.contains("<h4>Args</h4>"));
        assert!(html.contains("<code>{\n  &quot;q\\&quot;&lt;&quot;: &quot;a\\nb&quot;,\n  &quot;n&quot;: 3,\n  &quot;ok&quot;: true\n}</code>"));
        assert!(!html.contains("tool-output"));
    }

    #[test]
    fn chrome_toggles_on_details_and_styles_the_code_block() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"tool\"] > details > summary {"));
        assert!(css.contains("[data-slot=\"tool\"] > details[open] > summary [data-slot=\"tool-header\"] > svg[data-icon=\"chevron\"] {\n  rotate: 180deg;\n}"));
        assert!(css.contains("[data-slot=\"tool-header\"] svg[data-icon=\"running\"] {\n  animation: cui-tool-pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;\n}"));
        assert!(css.contains("[data-slot=\"tool-input\"] {\n  padding: 1rem; overflow: hidden;\n}"));
        assert!(css.contains("[data-slot=\"ai-code-block\"] {"));
        assert!(css.contains("[data-slot=\"ai-code-block-content\"] {\n  margin: 0; overflow: auto; padding: 1rem;\n  font-family: var(--cronus-font-mono); font-size: 0.875rem; line-height: 1.25rem; color: var(--cronus-fg);\n}"));
        assert!(!css.contains("[data-slot=\"tool-header\"][data-state=\"open\"]"));
    }
}
