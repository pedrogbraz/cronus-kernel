//! Dedicated Sheet renderer (wave1t geometry parity with the React Radix Sheet).
//!
//! React renders the open sheet portaled to `<body>` with no trigger:
//! `sheet-overlay` (fixed scrim) + `sheet-content` (fixed, right-pinned, border-left)
//! holding `sheet-header` > `h2 sheet-title` + `p sheet-description`, then the
//! `sheet-close` button (X icon + sr-only "Close").
//! The kernel renders the same tree open by default, zero JS. That open specimen
//! is not a popover and cannot close, so `sheet-close` is the same native
//! `<button>` with `disabled`, keeping React's idle look (no dimming).
//! Closed mode (`trigger:"…"` prop, or `open:false` / `defaultOpen:false`; see
//! `cronus_ui_kit::overlay_trigger`): an outline `button` trigger opens
//! `sheet-content` as a native `popover="auto"` (right-pinned, scrim on
//! `::backdrop`, no overlay div) and `sheet-close` hides it. Gaps: a popover is
//! not modal (no focus trap, background not inert, `aria-expanded` not reflected).
//! Description comes from `description:"…"` (props or item
//! config), else extra `text`. Not interact `dialog("sheet")` `<dialog>` + `showModal()`.

use crate::cronus_ui_kit::{
    attr, esc, item, label_of, modal_close_attrs, modal_dialog_open, modal_open_button,
    overlay_trigger, widget_id,
};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let title = item(comp, "title")
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| label_of(comp));
    let desc = description(comp, &title);
    let title_id = widget_id(comp, "sheet-title");
    let desc_id = widget_id(comp, "sheet-description");
    let (described_by, desc_html) = if desc.is_empty() {
        (String::new(), String::new())
    } else {
        (
            format!(" aria-describedby=\"{desc_id}\""),
            format!("<p id=\"{desc_id}\" data-slot=\"sheet-description\">{desc}</p>"),
        )
    };
    let header = format!(
        "<div data-slot=\"sheet-header\"><h2 id=\"{title_id}\" data-slot=\"sheet-title\">{title}</h2>{desc_html}</div>"
    );
    match overlay_trigger(comp, "Open") {
        None => format!(
            "<div data-slot=\"sheet-overlay\" aria-hidden=\"true\"></div><div role=\"dialog\" aria-labelledby=\"{title_id}\"{described_by} data-slot=\"sheet-content\">{header}<button type=\"button\" data-slot=\"sheet-close\" disabled><span>Close</span></button></div>"
        ),
        Some(trigger) => {
            let trigger_id = widget_id(comp, "sheet-trigger");
            let pop_id = widget_id(comp, "sheet");
            format!(
                "{}{}{header}<button type=\"button\" data-slot=\"sheet-close\"{}><span>Close</span></button></dialog>",
                modal_open_button(&trigger_id, &pop_id, &trigger),
                modal_dialog_open(
                    &pop_id,
                    "sheet-content",
                    "dialog",
                    &title_id,
                    &described_by,
                    true
                ),
                modal_close_attrs(&pop_id),
            )
        }
    }
}

fn description(comp: &ComponentNode, title: &str) -> String {
    if let Some(d) = attr(comp, "description").filter(|d| !d.trim().is_empty()) {
        return esc(d);
    }
    comp.items
        .iter()
        .filter(|i| i.item_type == "text" && !i.text.is_empty())
        .map(|i| esc(&i.text))
        .filter(|t| t != title)
        .collect::<Vec<_>>()
        .join(" ")
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
        assert!(!html.contains("<dialog"));
        assert!(!html.contains("showModal("));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("max-width:28rem"));
        assert!(!html.contains("popover"));
        assert!(!html.contains("sheet-trigger"));
    }

    #[test]
    fn react_dom_open_by_default_with_disabled_close() {
        let mut c = stub("sheet", "Edit profile");
        c.props.insert(
            "description".into(),
            "Make changes to your profile here.".into(),
        );
        let html = render(&c);
        let t = widget_id(&c, "sheet-title");
        let d = widget_id(&c, "sheet-description");
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"sheet-overlay\" aria-hidden=\"true\"></div><div role=\"dialog\" aria-labelledby=\"{t}\" aria-describedby=\"{d}\" data-slot=\"sheet-content\"><div data-slot=\"sheet-header\"><h2 id=\"{t}\" data-slot=\"sheet-title\">Edit profile</h2><p id=\"{d}\" data-slot=\"sheet-description\">Make changes to your profile here.</p></div><button type=\"button\" data-slot=\"sheet-close\" disabled><span>Close</span></button></div>"
            )
        );
        reject_interact(&html);
    }

    #[test]
    fn description_from_item_config_or_text() {
        let mut c = stub("sheet", "Edit profile");
        c.items[0]
            .config
            .insert("description".into(), "Cfg.".into());
        assert!(render(&c).contains("data-slot=\"sheet-description\">Cfg.</p>"));
        let mut t = stub("sheet", "Filters");
        t.items.push(extra("text", "Narrow the list."));
        assert!(render(&t).contains("data-slot=\"sheet-description\">Narrow the list.</p>"));
    }

    #[test]
    fn title_item_wins_and_description_is_optional() {
        let mut c = stub("sheet", "Open");
        c.items.push(extra("title", "Filters"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"sheet-title\">Filters</h2>"));
        assert!(!html.contains("sheet-description"));
        assert!(!html.contains("aria-describedby"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_dialog_surf() {
        let c = stub("sheet", "Filters");
        let html = render(&c);
        reject_interact(&html);
    }

    #[test]
    fn trigger_item_renders_closed_modal_dialog_with_working_close() {
        let mut c = stub("sheet", "Edit profile");
        c.items.push(extra("trigger", "Open sheet"));
        let html = render(&c);
        let tid = widget_id(&c, "sheet-trigger");
        let pid = widget_id(&c, "sheet");
        assert!(html.starts_with(&format!(
            "<button type=\"button\" id=\"{tid}\" data-slot=\"button\" data-variant=\"outline\" commandfor=\"{pid}\" command=\"show-modal\" aria-haspopup=\"dialog\">Open sheet</button><dialog id=\"{pid}\" data-slot=\"sheet-content\" role=\"dialog\" aria-modal=\"true\" aria-labelledby="
        )));
        assert!(html.contains("closedby=\"any\""));
        assert!(!html.contains("sheet-overlay"));
        assert!(!html.contains("popover"));
        assert!(
            !html.contains("data-slot=\"sheet-description\""),
            "trigger is not description"
        );
        assert!(html.ends_with(&format!(
            "<button type=\"button\" data-slot=\"sheet-close\" commandfor=\"{pid}\" command=\"close\"><span>Close</span></button></dialog>"
        )));
    }

    #[test]
    fn chrome_closed_mode_is_native_modal_dialog() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"sheet-content\"]:modal {"));
        assert!(css.contains("[data-slot=\"sheet-content\"]::backdrop {"));
        assert!(!css.contains("[data-slot=\"sheet-content\"][popover]"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("sheet", "Filters"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"sheet-content\""));
        });
    }

    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        let block = |sel: &str| {
            let start = css
                .find(&format!("{sel} {{"))
                .unwrap_or_else(|| panic!("{sel}"));
            let end = start + css[start..].find('}').unwrap();
            css[start..end].to_string()
        };
        let overlay = block("[data-slot=\"sheet-overlay\"]");
        assert!(overlay.contains("position: fixed; inset: 0;"));
        assert!(overlay.contains("backdrop-filter: blur(8px)"));
        let content = block("[data-slot=\"sheet-content\"]");
        assert!(content.contains("position: fixed; top: 0; bottom: 0; right: 0;"));
        assert!(content.contains("width: 75%; max-width: 24rem; height: 100%;"));
        assert!(content.contains("padding: 1.5rem;"));
        assert!(content.contains("border-left-width: 1px;"));
        assert!(content.contains("background: var(--cronus-surface-floating)"));
        let header = block("[data-slot=\"sheet-header\"]");
        assert!(header.contains("flex-direction: column; gap: 0.375rem;"));
        // Scoped under the header so it beats `html[data-cronus-theme] h2`
        // (font-weight 400, letter-spacing -0.025em); React's title is 600 / normal.
        let title = block("[data-slot=\"sheet-header\"] > [data-slot=\"sheet-title\"]");
        assert!(title.contains("letter-spacing: normal;"));
        assert!(title.contains("font-size: 1.125rem; line-height: 1.75rem; font-weight: 600;"));
        let close = block("[data-slot=\"sheet-close\"]");
        assert!(close.contains("position: absolute; top: 1rem; inset-inline-end: 1rem;"));
        assert!(close.contains("width: 1rem; height: 1rem;"));
        assert!(close.contains("color: var(--cronus-fg-tertiary)"));
        // The old closed-popover sheet must not come back: it had no box.
        assert!(!css.contains("[data-slot=\"sheet-content\"]:not(:popover-open)"));
        assert!(!css.contains("[data-slot=\"sheet-trigger\"]"));
        assert!(!css.contains("zinc-"));
    }
}
