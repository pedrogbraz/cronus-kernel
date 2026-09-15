//! Dedicated Tool renderer (AI suite). DOM matches React/Radix `Tool` +
//! `ToolHeader` + `ToolContent` + `ToolOutput`:
//! `<div data-state data-slot="tool">` > `<button data-slot="tool-header">`
//! (`<div>` wrench, name `<span>`, status `badge` secondary, chevron) >
//! `<div data-slot="tool-content">` > `<div data-slot="tool-output">` (`<h4>`
//! heading + error box). Name is `title:` or React's
//! `type.split("-").slice(1).join("-")`; `state:` picks the status label and
//! icon. Toggling needs JS: the kernel renders `defaultOpen` (open unless
//! `defaultOpen:false`) with the header `disabled`. Only `errorText:` output is
//! rendered: React shows `input` and object/string `output` in `AiCodeBlock`,
//! which has no kernel family, so those props are not rendered.

use crate::cronus_ui_kit::{attr, attr_nonempty, esc, truthy};
use crate::parser::ComponentNode;

const SVG_OPEN: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"";
const WRENCH: &str = "><path d=\"M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.106-3.105c.32-.322.863-.22.983.218a6 6 0 0 1-8.259 7.057l-7.91 7.91a1 1 0 0 1-2.999-3l7.91-7.91a6 6 0 0 1 7.057-8.259c.438.12.54.662.219.984z\"></path></svg>";
const CHEVRON: &str = "><path d=\"m6 9 6 6 6-6\"></path></svg>";
const CIRCLE: &str = "<circle cx=\"12\" cy=\"12\" r=\"10\"></circle>";

/// `(state, label, icon marker, icon body)` for React's `ToolUIPartState`.
const STATES: &[(&str, &str, &str, &str)] = &[
    ("input-streaming", "Pending", "pending", ""),
    (
        "input-available",
        "Running",
        "running",
        "<path d=\"M12 6v6l4 2\"></path>",
    ),
    (
        "output-available",
        "Completed",
        "completed",
        "<path d=\"m9 12 2 2 4-4\"></path>",
    ),
    (
        "output-error",
        "Error",
        "error",
        "<path d=\"m15 9-6 6\"></path><path d=\"m9 9 6 6\"></path>",
    ),
];

pub fn render(comp: &ComponentNode) -> String {
    let open = attr(comp, "defaultOpen").is_none_or(truthy);
    let state = if open { "open" } else { "closed" };
    let (_, label, marker, icon) = status_of(attr(comp, "state"));
    let name = esc(&tool_name(comp));
    let header = format!(
        "<button type=\"button\" aria-expanded=\"{open}\" data-state=\"{state}\" data-slot=\"tool-header\" disabled><div>{SVG_OPEN} data-icon=\"wrench\"{WRENCH}<span>{name}</span><span data-slot=\"badge\" data-variant=\"secondary\">{SVG_OPEN} data-icon=\"{marker}\">{CIRCLE}{icon}</svg>{label}</span></div>{SVG_OPEN} data-icon=\"chevron\"{CHEVRON}</button>"
    );
    let content = match (open, attr_nonempty(comp, "errorText")) {
        (true, Some(error)) => format!(
            "<div data-state=\"open\" data-slot=\"tool-content\"><div data-slot=\"tool-output\"><h4>Error</h4><div data-error=\"\"><div>{}</div><div></div></div></div></div>",
            esc(error)
        ),
        (true, None) => "<div data-state=\"open\" data-slot=\"tool-content\"></div>".to_string(),
        (false, _) => String::new(),
    };
    format!("<div data-state=\"{state}\" data-slot=\"tool\">{header}{content}</div>")
}

fn status_of(raw: Option<&str>) -> (&'static str, &'static str, &'static str, &'static str) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn tool(props: &[(&str, &str)]) -> ComponentNode {
        let mut c = stub("tool", "default");
        for (k, v) in props {
            c.props.insert((*k).into(), (*v).into());
        }
        c
    }

    #[test]
    fn error_state_matches_react_dom() {
        let html = render(&tool(&[
            ("type", "tool-web-search"),
            ("state", "output-error"),
            ("errorText", "Request timed out"),
        ]));
        assert!(html.starts_with("<div data-state=\"open\" data-slot=\"tool\"><button type=\"button\" aria-expanded=\"true\" data-state=\"open\" data-slot=\"tool-header\" disabled><div><svg"));
        assert!(html.contains(
            "<span>web-search</span><span data-slot=\"badge\" data-variant=\"secondary\"><svg"
        ));
        assert!(html.contains("data-icon=\"error\"><circle cx=\"12\" cy=\"12\" r=\"10\"></circle><path d=\"m15 9-6 6\"></path><path d=\"m9 9 6 6\"></path></svg>Error</span>"));
        assert!(html.ends_with("<div data-state=\"open\" data-slot=\"tool-content\"><div data-slot=\"tool-output\"><h4>Error</h4><div data-error=\"\"><div>Request timed out</div><div></div></div></div></div></div>"));
        assert!(!html.contains("style="));
        assert!(!html.contains("<script"));
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
        assert!(render(&tool(&[("state", "input-available")])).contains("</svg>Running</span>"));
        assert!(render(&tool(&[("state", "bogus")])).contains("</svg>Pending</span>"));
    }

    #[test]
    fn closed_has_no_content_and_error_text_is_escaped() {
        let closed = render(&tool(&[("defaultOpen", "false"), ("errorText", "x")]));
        assert!(closed.contains("data-state=\"closed\" data-slot=\"tool\""));
        assert!(!closed.contains("tool-content"));
        let open = render(&tool(&[("errorText", "<img onerror=1>")]));
        assert!(open.contains("&lt;img onerror=1&gt;"));
        assert!(!open.contains("<img"));
    }
}
