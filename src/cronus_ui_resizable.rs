//! Dedicated Resizable renderer. Static two-panel split, no JS drag.
//! DOM matches React `ResizablePanelGroup` (react-resizable-panels):
//! `<div data-slot="resizable-panel-group" data-panel-group-direction aria-label>`
//! with two `<div data-panel data-panel-size="50.0">` panels (React panels carry no
//! data-slot) around `<div data-slot="resizable-handle" role="separator">`.
//! The handle is not focusable: without JS it cannot resize, so it only separates.
//! The `label` names the group; it is never a panel.
//! Docs three-pane layout: `item "Sidebar" size:35` sets a panel's share
//! (`data-panel-size`, painted by per-value CSS rules), consecutive
//! `vertical:true` items form a nested vertical group in one panel (sized to
//! the remainder), `handle:true` adds the grip (`resizable-handle-grip`) to
//! the outer handles and `height:48` (Tailwind `h-48`) / `bordered:true`
//! give the group the docs' box. Each panel's text sits in the docs' centred
//! `p-4 text-sm` cell (an unslotted `<div>`).
//! Not catalog `display()` SURF (`<section>` without panel-group).

use crate::cronus_ui_kit::{attr_nonempty, choice_texts, esc, flag, label_of};
use crate::parser::{ComponentItemNode, ComponentNode};

const GRIP: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><circle cx=\"9\" cy=\"12\" r=\"1\"></circle><circle cx=\"9\" cy=\"5\" r=\"1\"></circle><circle cx=\"9\" cy=\"19\" r=\"1\"></circle><circle cx=\"15\" cy=\"12\" r=\"1\"></circle><circle cx=\"15\" cy=\"5\" r=\"1\"></circle><circle cx=\"15\" cy=\"19\" r=\"1\"></circle></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let aria = attr_nonempty(comp, "aria-label")
        .map(|a| format!(" aria-label=\"{}\"", esc(a)))
        .unwrap_or_default();
    let rich: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty())
        .collect();
    if rich.is_empty() {
        let (left, right) = panels(comp);
        return format!(
            "<div data-slot=\"resizable-panel-group\" data-panel-group-direction=\"horizontal\"{aria}><div data-panel=\"\" data-panel-size=\"50.0\">{left}</div><div data-slot=\"resizable-handle\" role=\"separator\" aria-valuenow=\"50\" aria-valuemin=\"0\" aria-valuemax=\"100\" data-panel-group-direction=\"horizontal\"></div><div data-panel=\"\" data-panel-size=\"50.0\">{right}</div></div>"
        );
    }
    let with_handle = flag(comp, "handle");
    let mut classes: Vec<String> = Vec::new();
    if let Some(h) = attr_nonempty(comp, "height").map(str::trim) {
        if matches!(h, "40" | "48" | "56" | "64" | "72" | "80" | "96") {
            classes.push(format!("h-{h}"));
        }
    }
    if flag(comp, "bordered") {
        classes.push("bordered".into());
    }
    let class = if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    };
    // Outer sequence: plain panels and one nested vertical group per run of `vertical:true` items.
    let is_vertical = |i: &ComponentItemNode| {
        i.config
            .get("vertical")
            .is_some_and(|v| crate::cronus_ui_kit::truthy(v))
    };
    let size_of = |i: &ComponentItemNode| -> Option<u32> {
        i.config
            .get("size")
            .and_then(|s| s.trim().parse::<u32>().ok())
            .filter(|n| (1..=100).contains(n))
    };
    let mut outer: Vec<(Option<u32>, String)> = Vec::new();
    let mut idx = 0;
    while idx < rich.len() {
        if is_vertical(rich[idx]) {
            let mut end = idx;
            while end < rich.len() && is_vertical(rich[end]) {
                end += 1;
            }
            let run = &rich[idx..end];
            let inner = group_html(
                "vertical",
                &run.iter()
                    .map(|i| (size_of(i), cell(i)))
                    .collect::<Vec<_>>(),
                false,
                "",
            );
            outer.push((None, inner));
            idx = end;
        } else {
            outer.push((size_of(rich[idx]), cell(rich[idx])));
            idx += 1;
        }
    }
    format!(
        "<div data-slot=\"resizable-panel-group\" data-panel-group-direction=\"horizontal\"{aria}{class}>{}</div>",
        group_body("horizontal", &outer, with_handle)
    )
}

fn cell(i: &ComponentItemNode) -> String {
    format!("<div>{}</div>", esc(&i.text))
}

/// Panels with `None` sizes share what the explicit sizes leave (React's
/// `defaultSize` omitted), split evenly.
fn resolved_sizes(panels: &[(Option<u32>, String)]) -> Vec<u32> {
    let explicit: u32 = panels.iter().filter_map(|(s, _)| *s).sum();
    let free = panels.iter().filter(|(s, _)| s.is_none()).count() as u32;
    let rest = 100u32.saturating_sub(explicit.min(100));
    let share = if free > 0 { rest / free } else { 0 };
    panels
        .iter()
        .map(|(s, _)| s.unwrap_or(share).max(1))
        .collect()
}

fn group_body(direction: &str, panels: &[(Option<u32>, String)], with_handle: bool) -> String {
    let sizes = resolved_sizes(panels);
    let grip = if with_handle {
        format!("<div data-slot=\"resizable-handle-grip\" aria-hidden=\"true\">{GRIP}</div>")
    } else {
        String::new()
    };
    panels
        .iter()
        .zip(sizes.iter())
        .enumerate()
        .map(|(n, ((_, inner), size))| {
            let handle = if n > 0 {
                format!(
                    "<div data-slot=\"resizable-handle\" role=\"separator\" aria-valuenow=\"{}\" aria-valuemin=\"0\" aria-valuemax=\"100\" data-panel-group-direction=\"{direction}\">{grip}</div>",
                    sizes[..n].iter().sum::<u32>()
                )
            } else {
                String::new()
            };
            format!("{handle}<div data-panel=\"\" data-panel-size=\"{size}.0\">{inner}</div>")
        })
        .collect()
}

fn group_html(
    direction: &str,
    panels: &[(Option<u32>, String)],
    with_handle: bool,
    aria: &str,
) -> String {
    format!(
        "<div data-slot=\"resizable-panel-group\" data-panel-group-direction=\"{direction}\"{aria}>{}</div>",
        group_body(direction, panels, with_handle)
    )
}

fn panels(comp: &ComponentNode) -> (String, String) {
    let mut items = choice_texts(comp);
    if items.is_empty() {
        items = comp
            .items
            .iter()
            .filter(|i| !matches!(i.item_type.as_str(), "label" | "title") && !i.text.is_empty())
            .map(|i| esc(&i.text))
            .collect();
    }
    let left = items.first().cloned().unwrap_or_else(|| label_of(comp));
    let right = items.get(1).cloned().unwrap_or_default();
    (left, right)
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
        assert!(!html.starts_with("<section"));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("tabindex"));
        assert!(!html.contains("max-height:12rem;overflow:auto"));
        assert!(html.contains("data-slot=\"resizable-panel-group\""));
        assert!(html.contains("data-slot=\"resizable-handle\""));
    }

    #[test]
    fn emitted_fixture_matches_react_dom_label_is_not_a_panel() {
        let mut c = stub("resizable", "Panels");
        c.items.push(extra("text", "One"));
        c.items.push(extra("text", "Two"));
        c.items[2]
            .config
            .insert("aria-label".into(), "Panels".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"resizable-panel-group\" data-panel-group-direction=\"horizontal\" aria-label=\"Panels\"><div data-panel=\"\" data-panel-size=\"50.0\">One</div><div data-slot=\"resizable-handle\" role=\"separator\" aria-valuenow=\"50\" aria-valuemin=\"0\" aria-valuemax=\"100\" data-panel-group-direction=\"horizontal\"></div><div data-panel=\"\" data-panel-size=\"50.0\">Two</div></div>"
        );
        assert!(!html.contains("data-slot=\"resizable\""));
        reject_interact(&html);
    }

    #[test]
    fn label_only_uses_label_for_first_panel() {
        let html = render(&stub("resizable", "Sidebar"));
        assert!(html.contains("<div data-panel=\"\" data-panel-size=\"50.0\">Sidebar</div>"));
        assert_eq!(html.matches("data-panel=\"\"").count(), 2);
        reject_interact(&html);
    }

    #[test]
    fn skips_display_and_scroll_surf() {
        let mut c = stub("resizable", "Split");
        c.items.push(extra("item", "Sidebar"));
        c.items.push(extra("item", "Main"));
        let html = render(&c);
        assert!(!crate::cli::stub_renderer_gate::looks_like_interact_generic(&html));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            reject_interact(&render(&stub("resizable", "Sidebar")));
        });
    }

    #[test]
    fn docs_three_pane_layout_with_nested_vertical_group_and_grip() {
        let mut c = stub("resizable", "Panes");
        c.props.insert("handle".into(), "true".into());
        c.props.insert("height".into(), "48".into());
        c.props.insert("bordered".into(), "true".into());
        let mut side = extra("item", "Sidebar");
        side.config.insert("size".into(), "35".into());
        c.items.push(side);
        let mut content = extra("item", "Content");
        content.config.insert("size".into(), "60".into());
        content.config.insert("vertical".into(), "true".into());
        c.items.push(content);
        let mut console = extra("item", "Console");
        console.config.insert("size".into(), "40".into());
        console.config.insert("vertical".into(), "true".into());
        c.items.push(console);
        let html = render(&c);
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"resizable-panel-group\" data-panel-group-direction=\"horizontal\" class=\"h-48 bordered\"><div data-panel=\"\" data-panel-size=\"35.0\"><div>Sidebar</div></div><div data-slot=\"resizable-handle\" role=\"separator\" aria-valuenow=\"35\" aria-valuemin=\"0\" aria-valuemax=\"100\" data-panel-group-direction=\"horizontal\"><div data-slot=\"resizable-handle-grip\" aria-hidden=\"true\">{GRIP}</div></div><div data-panel=\"\" data-panel-size=\"65.0\"><div data-slot=\"resizable-panel-group\" data-panel-group-direction=\"vertical\"><div data-panel=\"\" data-panel-size=\"60.0\"><div>Content</div></div><div data-slot=\"resizable-handle\" role=\"separator\" aria-valuenow=\"60\" aria-valuemin=\"0\" aria-valuemax=\"100\" data-panel-group-direction=\"vertical\"></div><div data-panel=\"\" data-panel-size=\"40.0\"><div>Console</div></div></div></div></div>"
            )
        );
        reject_interact(&html);
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"resizable-panel-group\"] > [data-panel][data-panel-size=\"35.0\"] { flex: 35 1 0px; }"));
        assert!(css.contains("[data-slot=\"resizable-panel-group\"] > [data-panel][data-panel-size=\"100.0\"] { flex: 100 1 0px; }"));
        assert!(css.contains("[data-slot=\"resizable-handle-grip\"] {\n  z-index: 10; display: flex; height: 1rem; width: 0.75rem;"));
        assert!(css.contains("[data-slot=\"resizable-panel-group\"].h-48 { height: 12rem; }"));
        assert!(css.contains("[data-slot=\"resizable-panel-group\"].bordered {\n  border: 1px solid var(--cronus-border); border-radius: var(--cronus-radius-lg);"));
    }

    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(!css.contains("[data-slot=\"resizable\"] {"));
        assert!(css.contains("[data-slot=\"resizable-panel-group\"] > [data-panel]"));
        assert!(css.contains("flex: 50 1 0px; overflow: hidden;"));
        assert!(css.contains("[data-slot=\"resizable-handle\"]::after"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(!css.contains("zinc-"));
    }
}
